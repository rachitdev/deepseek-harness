//! The `compaction` plugin: keeps a long conversation inside the model's
//! context window by replacing old turns with a model-written summary.
//!
//! It listens on `agent/pre-step`, before the new turn's messages enter the
//! log. When the visible history is estimated above `triggerTokens`, it
//!
//! 1. picks a cut at the start of the Nth-newest user turn already in the log
//!    (`keepRecentTurns`; the incoming message is extra). The cut only ever
//!    lands where no tool call is still waiting for its result, so a call and
//!    its result are never separated;
//! 2. asks the model to summarise everything before the cut;
//! 3. appends one `compaction` session event. The summary is a surface event,
//!    so the model sees it because it is in the log, and a reloaded session
//!    shows exactly the same history.
//!
//! The plugin fails open: if the summary call errors, times out, or comes back
//! empty or no shorter than what it replaces, nothing is logged and the turn
//! runs with the full history. A failed compaction is therefore never a lost
//! conversation; the worst case is the provider's own context-length error.
//!
//! Limits, by design:
//! - when a history has no user turn it may spare (one long turn, or fewer
//!   turns than `keepRecentTurns`), the cut falls back to the newest safe
//!   boundary that leaves `keepTailEvents` in place;
//! - the token count is a local BPE count (`tokenizer`), not the provider's
//!   own accounting; proprietary tokenizers differ, so it is a budget, not a bill;
//! - the summary call is not interrupted by cancelling the turn, only by
//!   `timeoutMs`;
//! - input to the summariser is capped (`maxInputChars`); when the replaced
//!   history is larger, the oldest part is omitted from the summary request.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use cordis::Plugin;
use serde::Deserialize;
use serde_json::Value;

use crate::api::services::{AgentRegistryService, LlmService};
use crate::llm::StreamTable;
use crate::plugins::{register_tools, snippet, tool_spec_ctx};
use crate::session::project_message;
use crate::types::{
    ContentBlock, GenerateOptions, LlmCallConfig, Message, MessageSource, Role, SessionEvent,
    SessionEventData,
};

const SUMMARY_SYSTEM: &str = "You compress conversations for an AI coding agent that will continue the work. \
Write a faithful, compact summary of the transcript you are given. Keep: the user's goals and constraints, \
decisions made and why, files and commands touched, results and errors that matter, open problems, and the \
next steps. Do not invent anything. Do not address the user. Output only the summary.";

/// The service name under which the plugin provides its validated
/// `Arc<CompactionConfig>`.
pub const COMPACTION_SERVICE: &str = "compactionConfig";

/// Configuration for the `compaction` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CompactionConfig {
    /// Estimated history size, in tokens, above which a turn compacts first.
    pub trigger_tokens: usize,
    /// Newest completed user turns left untouched (the incoming message is extra).
    pub keep_recent_turns: usize,
    /// Skip when fewer estimated tokens than this would be replaced.
    pub min_reclaim_tokens: usize,
    /// Output cap for the summary call.
    pub summary_max_tokens: u32,
    /// Deadline for the summary call, in milliseconds.
    pub timeout_ms: u64,
    /// Longest transcript sent to the summariser, in characters.
    pub max_input_chars: usize,
    /// Longest rendering of one tool call or result in the transcript.
    pub max_item_chars: usize,
    /// Summariser provider; defaults to the agent's own.
    pub provider: Option<String>,
    /// Summariser model; defaults to the agent's own.
    pub model: Option<String>,
    /// BPE encoding used to count context; see [`Tokenizer`].
    pub tokenizer: Tokenizer,
    /// Surface events left in place when a cut cannot be placed on a user-turn
    /// boundary (one long turn, or fewer turns than `keepRecentTurns`).
    pub keep_tail_events: usize,
    /// Compaction passes allowed in one turn: a history far above the trigger is
    /// brought under it in one turn instead of one pass per turn.
    pub max_passes: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        CompactionConfig {
            trigger_tokens: 60_000,
            keep_recent_turns: 2,
            min_reclaim_tokens: 2_000,
            summary_max_tokens: 1_500,
            timeout_ms: 60_000,
            max_input_chars: 200_000,
            max_item_chars: 2_000,
            provider: None,
            model: None,
            tokenizer: Tokenizer::default(),
            keep_tail_events: 8,
            max_passes: 3,
        }
    }
}

impl CompactionConfig {
    fn validate(&self) -> Result<(), String> {
        if self.keep_recent_turns == 0 {
            return Err("compaction: keepRecentTurns must be at least 1".to_string());
        }
        if self.trigger_tokens == 0 || self.summary_max_tokens == 0 || self.timeout_ms == 0 {
            return Err(
                "compaction: triggerTokens, summaryMaxTokens and timeoutMs must be above 0"
                    .to_string(),
            );
        }
        if self.max_passes == 0 {
            return Err("compaction: maxPasses must be at least 1".to_string());
        }
        if self.keep_tail_events == 0 {
            return Err("compaction: keepTailEvents must be at least 1".to_string());
        }
        if self.max_item_chars < 100 || self.max_input_chars < 1_000 {
            return Err("compaction: maxItemChars and maxInputChars are too small".to_string());
        }
        if self.provider.is_some() != self.model.is_some() {
            return Err("compaction: set both provider and model, or neither".to_string());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Local BPE encoding for context budgeting. Provider-specific framing and
/// proprietary tokenizers can differ; these are not billing counts.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tokenizer {
    #[default]
    Cl100kBase,
    O200kBase,
}

impl Tokenizer {
    fn text_tokens(self, text: &str) -> usize {
        let bpe = match self {
            Self::Cl100kBase => tiktoken_rs::cl100k_base_singleton(),
            Self::O200kBase => tiktoken_rs::o200k_base_singleton(),
        };
        // Special-looking strings in user/tool data are ordinary text, not control tokens.
        bpe.encode_ordinary(text).len()
    }

    fn messages(self, messages: &[Message]) -> usize {
        fn count(tokenizer: Tokenizer, block: &ContentBlock) -> usize {
            match block {
                ContentBlock::Text { text } | ContentBlock::Reasoning { text } => {
                    tokenizer.text_tokens(text)
                }
                ContentBlock::File { attachment } => tokenizer.text_tokens(
                    &crate::attachments::file_handle_text(attachment, None),
                ),
                // The image is billed by its pixels, whether or not its bytes
                // are in the request yet.
                ContentBlock::Image { attachment } | ContentBlock::ImageData { attachment, .. } => {
                    crate::images::estimate_tokens(attachment.width, attachment.height)
                }
                ContentBlock::ToolCall {
                    id,
                    name,
                    arguments,
                } => {
                    tokenizer.text_tokens(id)
                        + tokenizer.text_tokens(name)
                        + tokenizer.text_tokens(arguments)
                }
                ContentBlock::ToolResult {
                    tool_call_id,
                    content,
                    ..
                } => {
                    tokenizer.text_tokens(tool_call_id)
                        + content.iter().map(|b| count(tokenizer, b)).sum::<usize>()
                }
            }
        }
        messages
            .iter()
            .map(|m| {
                // Framing allowance: exact chat templates are owned by the provider.
                4 + m.content.iter().map(|b| count(self, b)).sum::<usize>()
            })
            .sum()
    }
}

/// BPE count of history content with a per-message framing allowance, using
/// `cl100k_base`. Configure `tokenizer` on the plugin for another supported encoding.
pub fn estimate_tokens(messages: &[Message]) -> usize {
    Tokenizer::default().messages(messages)
}

fn is_user_turn_start(event: &SessionEvent) -> bool {
    matches!(
        &event.data,
        SessionEventData::UserMessage { message } if matches!(message.source, MessageSource::User { .. })
    )
}

/// Positions where a cut cannot orphan a tool result.
///
/// A boundary before surface event `i` is safe when every tool call logged
/// before it already has its result. Cutting only at these positions lets a
/// long single turn be compacted without separating a call from its result.
pub fn safe_boundaries(surface: &[&SessionEvent]) -> Vec<usize> {
    let mut open: HashSet<String> = HashSet::new();
    let mut safe = Vec::new();
    for (index, event) in surface.iter().enumerate() {
        if open.is_empty() {
            safe.push(index);
        }
        let calls: Vec<&str> = match &event.data {
            SessionEventData::AssistantMessage { message, .. } => message
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::ToolCall { id, .. } => Some(id.as_str()),
                    _ => None,
                })
                .collect(),
            SessionEventData::ToolCall { call_id, .. } => vec![call_id.as_str()],
            _ => Vec::new(),
        };
        for id in calls {
            open.insert(id.to_string());
        }
        if let SessionEventData::ToolResult { message, .. } = &event.data {
            for block in &message.content {
                if let ContentBlock::ToolResult { tool_call_id, .. } = block {
                    open.remove(tool_call_id.as_str());
                }
            }
        }
    }
    if open.is_empty() {
        safe.push(surface.len());
    }
    safe
}

/// The seq at which the kept history starts, or `None` when nothing should be
/// replaced.
///
/// `surface` is the model-visible events in order. The preferred cut is the
/// user message that begins the `keep_recent` newest user turns. When that
/// position is unsafe, or when the history has too few turns (one long turn),
/// the cut moves to the newest safe boundary that leaves `keep_tail_events`
/// in place, so a tool call is never separated from its result.
pub fn choose_cut(
    surface: &[&SessionEvent],
    keep_recent: usize,
    keep_tail_events: usize,
) -> Option<u64> {
    let boundaries = safe_boundaries(surface);
    if boundaries.is_empty() {
        return None;
    }
    let starts: Vec<usize> = surface
        .iter()
        .enumerate()
        .filter(|(_, event)| is_user_turn_start(event))
        .map(|(i, _)| i)
        .collect();
    if starts.len() > keep_recent {
        // The newest `keep_recent` turns stay whole: the cut may move earlier than
        // the turn start, but never later, or it would eat into a kept turn.
        let want = starts[starts.len() - keep_recent];
        let at = if boundaries.contains(&want) {
            Some(want)
        } else {
            boundaries.iter().copied().filter(|b| *b <= want).max()
        };
        return at.filter(|at| *at > 0).map(|at| surface[at].seq);
    }
    let limit = surface.len().saturating_sub(keep_tail_events.max(1));
    boundaries
        .iter()
        .copied()
        .filter(|at| *at > 0 && *at <= limit)
        .max()
        .map(|at| surface[at].seq)
}

/// Render `messages` as a plain-text transcript for the summariser.
///
/// Each tool call and result is cut to `max_item_chars`; when the whole is
/// longer than `max_input_chars` the oldest part is dropped and says so.
pub fn render_transcript(
    messages: &[Message],
    max_item_chars: usize,
    max_input_chars: usize,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    for message in messages {
        let who = match message.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
            _ => "System",
        };
        for block in &message.content {
            match block {
                ContentBlock::Text { text } => parts.push(format!("{who}: {text}")),
                ContentBlock::Reasoning { .. } => {}
                ContentBlock::File { attachment } => {
                    parts.push(format!("{who} attached file {:?}", attachment.name))
                }
                ContentBlock::Image { attachment } | ContentBlock::ImageData { attachment, .. } => {
                    parts.push(format!(
                        "{who} attached an image {:?}",
                        attachment.name.as_deref().unwrap_or("(unnamed)")
                    ))
                }
                ContentBlock::ToolCall {
                    name, arguments, ..
                } => {
                    parts.push(format!(
                        "Assistant called {name}: {}",
                        snippet(arguments, max_item_chars)
                    ));
                }
                ContentBlock::ToolResult {
                    content, is_error, ..
                } => {
                    let text: String = content.iter().filter_map(|b| b.as_text()).collect();
                    let label = if *is_error == Some(true) {
                        "Tool error"
                    } else {
                        "Tool result"
                    };
                    parts.push(format!("{label}: {}", snippet(&text, max_item_chars)));
                }
            }
        }
    }
    let full = parts.join("\n\n");
    let total = full.chars().count();
    if total <= max_input_chars {
        return full;
    }
    let notice = "[Earlier part of the conversation omitted: it was too long to send.]\n\n";
    // The notice counts against the cap; a cap smaller than the notice cuts the notice.
    let notice: String = notice.chars().take(max_input_chars).collect();
    let room = max_input_chars.saturating_sub(notice.chars().count());
    let tail: String = full.chars().skip(total - room).collect();
    format!("{notice}{tail}")
}

// ---------------------------------------------------------------------------
// Summary call
// ---------------------------------------------------------------------------

/// One summary request: what to summarise, with which model, and where to log it.
struct SummaryCall<'a> {
    focus: Option<&'a str>,
    config: LlmCallConfig,
    transcript: String,
    session_id: String,
}

async fn summarise(
    ctx: &cordis::Context,
    llm: LlmService,
    streams: StreamTable,
    settings: &CompactionConfig,
    call: SummaryCall<'_>,
) -> Result<String, String> {
    let SummaryCall {
        focus,
        config,
        transcript,
        session_id,
    } = call;
    let request = Message::user(
        "compaction-request",
        vec![ContentBlock::text(format!(
            "Summarise this conversation so the work can continue from the summary alone.\n\n<transcript>\n{transcript}\n</transcript>"
        ))],
    );
    let options = GenerateOptions {
        provider: settings.provider.clone().unwrap_or(config.provider),
        model: settings.model.clone().unwrap_or(config.model),
        messages: vec![request],
        system: Some(match focus {
            Some(focus) => format!("{SUMMARY_SYSTEM} Focus especially on: {focus}"),
            None => SUMMARY_SYSTEM.to_string(),
        }),
        tools: None,
        temperature: Some(0.0),
        max_tokens: Some(settings.summary_max_tokens),
        stop: None,
        session_id: Some(format!("{session_id}-compaction")),
    };
    crate::plugins::one_shot_text(
        ctx,
        &llm,
        &streams,
        options,
        Duration::from_millis(settings.timeout_ms),
    )
    .await
}

/// What one compaction attempt did.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Below the trigger, or nothing worth replacing.
    Skipped,
    /// A `compaction` event was logged, replacing `replaced_tokens` estimated tokens.
    Compacted {
        replaced_tokens: usize,
        summary_tokens: usize,
        /// How many history items the summary replaced.
        replaced_items: usize,
        /// The `compaction` event that holds the summary.
        summary_seq: u64,
    },
}

/// Compact `agent_id`'s session if it needs it.
/// Why a compaction is running. Tool-driven compaction sets `force` and may
/// carry a `focus` that steers what the summary keeps.
#[derive(Debug, Clone, Default)]
pub struct CompactionRequest {
    /// Compact even below the trigger and the minimum reclaim.
    pub force: bool,
    /// Extra instruction for the summariser.
    pub focus: Option<String>,
}

pub async fn compact_if_needed(
    ctx: &cordis::Context,
    agent_id: &str,
    settings: &CompactionConfig,
    request: &CompactionRequest,
) -> Result<Outcome, String> {
    let agents = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .ok_or("agents service is not available")?;
    let Some(agent) = agents.get(agent_id) else {
        return Ok(Outcome::Skipped);
    };
    let session = agent.session();

    let history = session.derive_messages();
    if !request.force && settings.tokenizer.messages(&history) < settings.trigger_tokens {
        return Ok(Outcome::Skipped);
    }
    // Never reach into the turn that is running: its opening message and any
    // step landed so far are what the model is answering right now.
    let open_turn = session.open_turn();
    let visible: Vec<SessionEvent> = match open_turn {
        Some(turn) => session
            .surface_events()
            .into_iter()
            .take_while(|event| {
                !matches!(&event.data, SessionEventData::TurnStart { turn: t } if *t == turn)
            })
            .collect(),
        None => session.surface_events(),
    };
    let surface: Vec<&SessionEvent> = visible.iter().collect();
    let Some(cut) = choose_cut(
        &surface,
        settings.keep_recent_turns,
        settings.keep_tail_events,
    ) else {
        return Ok(Outcome::Skipped);
    };
    let replaced: Vec<Message> = surface
        .iter()
        // By position, not seq: an earlier summary sits first but has a later seq.
        .take_while(|e| e.seq != cut)
        .filter_map(|e| project_message(e))
        .collect();
    let replaced_tokens = settings.tokenizer.messages(&replaced);
    if !request.force && replaced_tokens < settings.min_reclaim_tokens {
        return Ok(Outcome::Skipped);
    }

    // Which model writes the summary: the configured one, else the one that
    // answered this session's latest request.
    let config = match (
        &settings.provider,
        &settings.model,
        session.request_header(),
    ) {
        (Some(provider), Some(model), _) => LlmCallConfig {
            provider: provider.clone(),
            model: model.clone(),
            ..Default::default()
        },
        (_, _, Some(header)) => header.config,
        _ => return Err("cannot tell which model should write the summary".to_string()),
    };
    let llm = (*ctx
        .get::<LlmService>(crate::api::LLM_SERVICE)
        .ok_or("llm service is not available")?)
    .clone();
    let streams = (*ctx
        .get::<StreamTable>(crate::api::LLM_STREAMS_SERVICE)
        .ok_or("llmStreams service is not available")?)
    .clone();

    let transcript =
        render_transcript(&replaced, settings.max_item_chars, settings.max_input_chars);
    let summary = summarise(
        ctx,
        llm,
        streams,
        settings,
        SummaryCall {
            focus: request.focus.as_deref(),
            config,
            transcript,
            session_id: agent_id.to_string(),
        },
    )
    .await?;
    let summary = summary.trim().to_string();
    if summary.is_empty() {
        return Err("the model returned an empty summary".to_string());
    }
    let summary_tokens = settings.tokenizer.text_tokens(&summary);
    if summary_tokens >= replaced_tokens {
        return Err(format!(
            "the summary (~{summary_tokens} tokens) is not shorter than what it replaces (~{replaced_tokens})"
        ));
    }
    let replaced_items = replaced.len();
    let summary_seq = session
        .append(SessionEventData::Compaction {
            summary,
            before_seq: cut,
        })
        .seq;
    Ok(Outcome::Compacted {
        replaced_tokens,
        summary_tokens,
        replaced_items,
        summary_seq,
    })
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// The `compaction` plugin. Requires the `agents`, `llm` and `llmStreams` services.
pub fn compaction_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "compaction",
        vec![
            cordis::Injection::new(crate::api::AGENTS_SERVICE.to_string()),
            cordis::Injection::new(crate::api::LLM_SERVICE.to_string()),
            cordis::Injection::new(crate::api::LLM_STREAMS_SERVICE.to_string()),
        ],
        |ctx, config: CompactionConfig| async move {
            config.validate().map_err(cordis::Error::msg)?;
            let config = Arc::new(config);
            // `/compact` compacts by these settings, so it reads them here
            // instead of keeping a second copy.
            ctx.provide(COMPACTION_SERVICE, Arc::clone(&config)).await?;

            let tool_config = Arc::clone(&config);
            register_tools(
                &ctx,
                "compaction tools",
                vec![tool_spec_ctx(
                    "compact",
                    "Compact this conversation now: fold older history into a model-written summary. \n\
                     Use it when the history is long and the next steps need the room. The summary \n\
                     keeps goals, decisions, files, results and open problems; recent turns stay intact.",
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "focus": {
                                "type": "string",
                                "description": "What the summary must preserve, e.g. the failing test and the files already changed."
                            }
                        }
                    }),
                    move |args, run| {
                        let config = Arc::clone(&tool_config);
                        async move {
                            let Some(agent_id) = run.agent_id.clone() else {
                                return crate::types::ToolExecutionResult::error(
                                    "NO_AGENT",
                                    "compact can only be called by an agent",
                                );
                            };
                            let request = CompactionRequest {
                                force: true,
                                focus: args
                                    .get("focus")
                                    .and_then(Value::as_str)
                                    .map(str::to_string),
                            };
                            match compact_if_needed(&run.ctx, &agent_id, &config, &request).await {
                                Ok(Outcome::Compacted { replaced_tokens, summary_tokens, .. }) => {
                                    crate::types::ToolExecutionResult::success_text(
                                        format!(
                                            "compacted: ~{replaced_tokens} tokens of older history became a ~{summary_tokens}-token summary"
                                        ),
                                        serde_json::json!({
                                            "compacted": true,
                                            "replacedTokens": replaced_tokens,
                                            "summaryTokens": summary_tokens
                                        }),
                                    )
                                }
                                Ok(Outcome::Skipped) => {
                                    crate::types::ToolExecutionResult::success_text(
                                        "nothing to compact: no older history can be replaced".to_string(),
                                        serde_json::json!({ "compacted": false }),
                                    )
                                }
                                Err(reason) => crate::types::ToolExecutionResult::error(
                                    "COMPACTION_FAILED",
                                    reason,
                                ),
                            }
                        }
                    },
                )],
            )
            .await?;

            ctx.on("agent/pre-step", move |ctx, payload, next| {
                let config = Arc::clone(&config);
                Box::pin(async move {
                    let agent = payload.get("agent").and_then(Value::as_str).unwrap_or("").to_string();
                    // A history far above the trigger is brought back under it in one
                    // turn: each pass re-derives the surface, and a pass that has
                    // nothing to reclaim ends the loop.
                    for _ in 0..config.max_passes {
                        match compact_if_needed(&ctx, &agent, &config, &CompactionRequest::default()).await {
                            Ok(Outcome::Skipped) => break,
                            Ok(Outcome::Compacted { replaced_tokens, summary_tokens, .. }) => {
                                ctx.logger().log_event(
                                    cordis::logger::LogLevel::Info,
                                    "compaction".to_string(),
                                    None,
                                    format!(
                                        "agent {agent}: ~{replaced_tokens} tokens replaced by a ~{summary_tokens}-token summary"
                                    ),
                                );
                            }
                            Err(reason) => {
                                ctx.logger().log_event(
                                    cordis::logger::LogLevel::Warn,
                                    "compaction".to_string(),
                                    Some("SKIPPED".to_string()),
                                    format!("agent {agent}: not compacted ({reason}); continuing with full history"),
                                );
                                break;
                            }
                        }
                    }
                    next.run(payload).await
                }) as crate::api::services::BoxFuture<cordis::Result<Value>>
            })
            .await?;
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CallId, Message};

    fn ev(seq: u64, data: SessionEventData) -> SessionEvent {
        SessionEvent::new(seq, 0, data)
    }
    fn user(seq: u64, text: &str) -> SessionEvent {
        ev(
            seq,
            SessionEventData::UserMessage {
                message: Message::user(format!("u{seq}"), vec![ContentBlock::text(text)]),
            },
        )
    }
    fn plugin_ctx(seq: u64, text: &str) -> SessionEvent {
        let mut message = Message::user(format!("p{seq}"), vec![ContentBlock::text(text)]);
        message.source = MessageSource::Plugin {
            plugin: "memory".into(),
            form: None,
        };
        ev(seq, SessionEventData::UserMessage { message })
    }
    fn assistant(seq: u64, content: Vec<ContentBlock>) -> SessionEvent {
        ev(
            seq,
            SessionEventData::AssistantMessage {
                turn: 1,
                step: 1,
                message: Message {
                    id: format!("a{seq}"),
                    role: Role::Assistant,
                    content,
                    source: MessageSource::User { rpc_id: None },
                },
                usage: None,
                interrupted: None,
            },
        )
    }
    fn tool_result(seq: u64, id: &str) -> SessionEvent {
        ev(
            seq,
            SessionEventData::ToolResult {
                turn: 1,
                step: 1,
                message: Message::user(
                    format!("r{seq}"),
                    vec![ContentBlock::ToolResult {
                        tool_call_id: id.to_string() as CallId,
                        content: vec![ContentBlock::text("out")],
                        is_error: None,
                    }],
                ),
            },
        )
    }

    #[test]
    fn tokens_come_from_the_configured_encoding_not_from_a_character_estimate() {
        // Deterministic BPE facts: these hold for the pinned tiktoken data.
        let cl = Tokenizer::Cl100kBase;
        let o2 = Tokenizer::O200kBase;
        assert_eq!(cl.text_tokens("hello world"), 2);
        assert_eq!(cl.text_tokens(""), 0);
        // A repeated word tokenises cheaper per character than random punctuation.
        assert!(cl.text_tokens(&"hello ".repeat(100)) < "hello ".repeat(100).len() / 4 + 5);
        assert!(cl.text_tokens("fingerprint") >= 1);
        assert!(o2.text_tokens("fingerprint") >= 1);
        // Framing is allowed per message, so a message costs more than its text alone.
        let one = Message::user("m", vec![ContentBlock::text("hello world")]);
        assert_eq!(estimate_tokens(&[one]), 4 + 2);
        // Tool calls and results are counted from their own text.
        let call = Message {
            id: "a".into(),
            role: Role::Assistant,
            content: vec![ContentBlock::ToolCall {
                id: "c".into(),
                name: "bash".into(),
                arguments: "{\"command\":\"ls\"}".into(),
            }],
            source: MessageSource::User { rpc_id: None },
        };
        let result = Message::user(
            "r",
            vec![ContentBlock::ToolResult {
                tool_call_id: "c".into(),
                content: vec![ContentBlock::text("file.txt")],
                is_error: None,
            }],
        );
        assert!(estimate_tokens(&[call, result]) > 8);
    }

    #[test]
    fn the_cut_is_the_start_of_the_nth_newest_user_turn() {
        let events = [
            user(1, "t1"),
            assistant(2, vec![ContentBlock::text("r1")]),
            user(3, "t2"),
            assistant(4, vec![ContentBlock::text("r2")]),
            user(5, "t3"),
            assistant(6, vec![ContentBlock::text("r3")]),
        ];
        let surface: Vec<&SessionEvent> = events.iter().collect();
        assert_eq!(choose_cut(&surface, 2, 1), Some(3), "keeps turns 2 and 3");
        assert_eq!(choose_cut(&surface, 1, 1), Some(5));
        // Three turns with `keepRecentTurns` three: the turn rule leaves nothing to
        // replace, and the tail rule refuses too while the tail covers the history.
        assert_eq!(choose_cut(&surface, 3, 8), None);
        // With a tiny tail the same history may be compacted mid-history, keeping the
        // last event whole: the fallback exists so a long history is not uncompactable.
        assert_eq!(choose_cut(&surface, 3, 1), Some(6));
        assert_eq!(choose_cut(&surface, 9, 8), None);
    }

    #[test]
    fn a_tool_call_is_never_separated_from_its_result() {
        let call = ContentBlock::ToolCall {
            id: "c1".into(),
            name: "bash".into(),
            arguments: "{}".into(),
        };
        let events = [
            user(1, "t1"),
            assistant(2, vec![call]),
            tool_result(3, "c1"),
            assistant(4, vec![ContentBlock::text("done")]),
            user(5, "t2"),
            assistant(6, vec![ContentBlock::text("r2")]),
        ];
        let surface: Vec<&SessionEvent> = events.iter().collect();
        let cut = choose_cut(&surface, 1, 1).unwrap();
        assert_eq!(cut, 5);
        let before: Vec<u64> = surface
            .iter()
            .map(|e| e.seq)
            .take_while(|s| *s < cut)
            .collect();
        assert_eq!(before, [1, 2, 3, 4], "call 2 and result 3 stay together");
    }

    #[test]
    fn a_long_single_turn_is_compactable_without_splitting_a_call_from_its_result() {
        let call1 = ContentBlock::ToolCall {
            id: "c1".into(),
            name: "bash".into(),
            arguments: "{}".into(),
        };
        let call2 = ContentBlock::ToolCall {
            id: "c2".into(),
            name: "bash".into(),
            arguments: "{}".into(),
        };
        // One user turn: user, call1, result1, call2, result2, closing text.
        let events = [
            user(1, "t1"),
            assistant(2, vec![call1]),
            tool_result(3, "c1"),
            assistant(4, vec![call2]),
            tool_result(5, "c2"),
            assistant(6, vec![ContentBlock::text("done")]),
        ];
        let surface: Vec<&SessionEvent> = events.iter().collect();
        let safe = safe_boundaries(&surface);
        assert!(
            !safe.contains(&2),
            "a call without its result is not a boundary"
        );
        assert!(safe.contains(&3), "after call1 and result1 is a boundary");
        assert!(safe.contains(&5), "after call2 and result2 is a boundary");

        // Too few turns for the turn rule, so the tail rule applies.
        let cut = choose_cut(&surface, 2, 2).expect("a long single turn can be compacted");
        assert_eq!(
            cut, 4,
            "cut moves to the newest safe boundary leaving two events"
        );
        let replaced: Vec<u64> = surface
            .iter()
            .map(|e| e.seq)
            .take_while(|s| *s < cut)
            .collect();
        assert_eq!(replaced, [1, 2, 3], "call1 and result1 stay together");
    }

    #[test]
    fn plugin_context_messages_do_not_count_as_user_turns() {
        let events = [
            plugin_ctx(1, "memory block"),
            user(2, "t1"),
            assistant(3, vec![ContentBlock::text("r1")]),
            plugin_ctx(4, "memory block"),
            user(5, "t2"),
            assistant(6, vec![ContentBlock::text("r2")]),
        ];
        let surface: Vec<&SessionEvent> = events.iter().collect();
        assert_eq!(choose_cut(&surface, 1, 1), Some(5));
    }

    #[test]
    fn an_earlier_summary_counts_as_replaceable_history_even_with_a_later_seq() {
        // After a compaction the surface is [summary, kept turns...] and the summary's
        // seq is higher than the kept turns'. The cut is chosen by position.
        let events = [
            ev(
                40,
                SessionEventData::Compaction {
                    summary: "old".into(),
                    before_seq: 3,
                },
            ),
            user(3, "t2"),
            assistant(4, vec![ContentBlock::text("r2")]),
            user(5, "t3"),
            assistant(6, vec![ContentBlock::text("r3")]),
            user(41, "t4"),
        ];
        let surface: Vec<&SessionEvent> = events.iter().collect();
        assert_eq!(
            choose_cut(&surface, 3, 8),
            None,
            "three turns, keep three, full tail: nothing to replace"
        );
        assert_eq!(
            choose_cut(&surface, 2, 1),
            Some(5),
            "keeps t3 and t4; t2 and the old summary go"
        );
    }

    #[test]
    fn the_transcript_labels_roles_trims_items_and_omits_the_oldest_when_too_long() {
        let messages = vec![
            Message::user("u", vec![ContentBlock::text("fix the bug")]),
            Message {
                id: "a".into(),
                role: Role::Assistant,
                content: vec![ContentBlock::ToolCall {
                    id: "c".into(),
                    name: "bash".into(),
                    arguments: "q".repeat(500),
                }],
                source: MessageSource::User { rpc_id: None },
            },
            Message::user(
                "r",
                vec![ContentBlock::ToolResult {
                    tool_call_id: "c".into(),
                    content: vec![ContentBlock::text("boom")],
                    is_error: Some(true),
                }],
            ),
        ];
        let text = render_transcript(&messages, 120, 10_000);
        assert!(text.contains("User: fix the bug"));
        assert!(text.contains("Assistant called bash:"));
        assert!(text.contains("Tool error: boom"));
        assert!(!text.contains(&"q".repeat(200)), "the argument is trimmed");

        let long: Vec<Message> = (0..50)
            .map(|i| {
                Message::user(
                    format!("m{i}"),
                    vec![ContentBlock::text(format!("line {i} {}", "w".repeat(80)))],
                )
            })
            .collect();
        let cut = render_transcript(&long, 120, 1_000);
        assert!(
            cut.chars().count() <= 1_000,
            "the cap includes the notice: {}",
            cut.chars().count()
        );
        assert!(cut.starts_with("[Earlier part of the conversation omitted"));
        assert!(cut.contains("line 49") && !cut.contains("line 0 "));
    }

    #[test]
    fn bad_settings_are_rejected() {
        let ok = CompactionConfig::default();
        assert!(ok.validate().is_ok());
        for bad in [
            CompactionConfig {
                keep_recent_turns: 0,
                ..Default::default()
            },
            CompactionConfig {
                trigger_tokens: 0,
                ..Default::default()
            },
            CompactionConfig {
                provider: Some("p".into()),
                ..Default::default()
            },
            CompactionConfig {
                max_input_chars: 10,
                ..Default::default()
            },
        ] {
            assert!(bad.validate().is_err(), "{bad:?}");
        }
    }
}
