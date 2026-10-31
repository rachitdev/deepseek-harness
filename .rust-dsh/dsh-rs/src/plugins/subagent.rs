//! The `subagent` plugin: a `subagent` tool that hands a self-contained task to
//! a fresh agent and returns its final answer.
//!
//! The child is a full agent: its own session, the same tools (so the same
//! approvals policy), the parent's model and working directory. It sees only
//! the task text, not the parent's conversation, so the task must carry
//! everything the child needs.
//!
//! Safety rails, all configurable:
//! - `maxDepth`: how deep delegation may nest. A child at the limit gets an
//!   error from the tool instead of a grandchild.
//! - `maxConcurrent`: children alive at once, across all parents.
//! - `timeoutMs`: wall-clock limit for one child.
//! - `maxResultChars`: the answer returned to the parent is cut to this.
//!
//! A child is torn down by a guard however the call ends: it finished, timed
//! out, the parent turn was cancelled, or the tool future was dropped. The
//! child's session stays in the log; only the live agent is disposed.
//!
//! Limits, by design:
//! - a child that needs a decision calls `ask_parent`. The parent's own turn is
//!   blocked while the child runs, so the question is answered by the same
//!   model on the parent's recent history, and the exchange is logged in the
//!   parent session. The child gets an answer; it does not get a conversation;
//! - a child that outlives `timeoutMs` is cancelled; any public text already
//!   logged is returned with an explicit incomplete marker, never as a finished answer;
//! - tool calls the child makes are gated by the same approvals policy, and
//!   the question names the child's agent id; with no approver attached they
//!   are denied.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cordis::Plugin;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::services::{AgentRegistryService, AgentView, LlmService};
use crate::llm::{stream_via_waterfall, BlockAssembler, StreamTable};
use crate::plugins::{register_tools, tool_spec_ctx, unique_id};
use crate::types::{
    AgentCancelCause, AgentOptions, ContentBlock, ContextForm, FinishReason, GenerateOptions,
    Message, MessageSource, Role, SessionEventData, ToolExecutionResult, TurnEndReason,
};

/// System prompt for the answer a child gets when it consults the parent's model.
const ASK_SYSTEM: &str = "You are the main agent. A sub-agent you delegated to is blocked on the question \
below. Answer from the conversation you already have: give the concrete decision or missing fact it \
needs, in a few sentences. If the conversation does not settle it, say what you would try and why. \
Do not ask it questions back; it cannot answer follow-ups.";

/// Configuration for the `subagent` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SubagentConfig {
    /// Nesting depth allowed: 1 means children may not delegate further.
    pub max_depth: usize,
    /// Children alive at once.
    pub max_concurrent: usize,
    /// Wall-clock limit for one child, in milliseconds.
    pub timeout_ms: u64,
    /// Longest answer returned to the parent, in characters.
    pub max_result_chars: usize,
    /// Output-token cap for the child's model calls; the parent's when unset.
    pub max_tokens: Option<u32>,
    /// Deadline for one `ask_parent` answer, in milliseconds.
    pub ask_timeout_ms: u64,
    /// Output-token cap for the parent-model answer.
    pub ask_max_tokens: u32,
    /// Parent messages replayed to the parent's model when answering a child.
    pub ask_context_messages: usize,
}

impl Default for SubagentConfig {
    fn default() -> Self {
        SubagentConfig {
            max_depth: 1,
            max_concurrent: 4,
            timeout_ms: 600_000,
            max_result_chars: 20_000,
            max_tokens: None,
            ask_timeout_ms: 60_000,
            ask_max_tokens: 800,
            ask_context_messages: 12,
        }
    }
}

impl SubagentConfig {
    fn validate(&self) -> Result<(), String> {
        if self.max_depth == 0 || self.max_concurrent == 0 {
            return Err("subagent: maxDepth and maxConcurrent must be at least 1".to_string());
        }
        if self.ask_timeout_ms == 0 || self.ask_max_tokens < 64 || self.ask_context_messages == 0 {
            return Err(
                "subagent: askTimeoutMs, askMaxTokens (>=64) and askContextMessages must be set"
                    .to_string(),
            );
        }
        if self.timeout_ms == 0 || self.max_result_chars < 200 {
            return Err(
                "subagent: timeoutMs must be above 0 and maxResultChars at least 200".to_string(),
            );
        }
        Ok(())
    }
}

/// The `subagent-lineage` service key.
pub const LINEAGE_SERVICE: &str = "subagent-lineage";

/// Which live agents are sub-agents, for plugins that only a root agent may use.
#[derive(Clone)]
pub struct Lineage(Arc<Registry>);

impl Lineage {
    /// Whether `agent` is a live sub-agent started by another agent.
    pub fn is_child(&self, agent: &str) -> bool {
        self.0.parent_of(agent).is_some()
    }

    /// The agent that started `child`, or `None` when `child` is not a live sub-agent.
    pub fn parent_of(&self, child: &str) -> Option<String> {
        self.0.parent_of(child)
    }

    /// Record `child` as started by `parent`. For tests that need a delegated agent
    /// without running the `subagent` tool.
    #[doc(hidden)]
    pub fn adopt_for_test(&self, child: &str, parent: &str) {
        self.0.link(child, parent);
    }
}

/// Live-child bookkeeping shared by every call of one plugin instance.
#[derive(Default)]
struct Registry {
    /// Nesting depth of each live child, by agent id. Agents not listed are roots (depth 0).
    depth: Mutex<HashMap<String, usize>>,
    /// Live child id to the agent that started it, for `ask_parent`.
    parent: Mutex<HashMap<String, String>>,
    live: AtomicUsize,
}

impl Registry {
    fn depth_of(&self, agent: &str) -> usize {
        self.depth.lock().unwrap().get(agent).copied().unwrap_or(0)
    }

    fn link(&self, child: &str, parent: &str) {
        self.parent
            .lock()
            .unwrap()
            .insert(child.to_string(), parent.to_string());
    }

    /// The agent that started `child`, or `None` when `child` is not a sub-agent.
    fn parent_of(&self, child: &str) -> Option<String> {
        self.parent.lock().unwrap().get(child).cloned()
    }

    /// Reserve a slot for a child of depth `depth`, or say why not.
    fn reserve(&self, limit: usize) -> Result<(), String> {
        let mut live = self.live.load(Ordering::SeqCst);
        loop {
            if live >= limit {
                return Err(format!(
                    "{limit} sub-agent(s) are already running; wait for one to finish"
                ));
            }
            match self
                .live
                .compare_exchange(live, live + 1, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return Ok(()),
                Err(now) => live = now,
            }
        }
    }
}

/// Disposes a child and frees its slot when dropped, however the call ends.
struct ChildGuard {
    agents: AgentRegistryService,
    agent: Arc<dyn AgentView>,
    registry: Arc<Registry>,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.agent.cancel(AgentCancelCause::Parent, false);
        self.agents.dispose(&self.agent);
        self.registry.depth.lock().unwrap().remove(self.agent.id());
        self.registry.parent.lock().unwrap().remove(self.agent.id());
        self.registry.live.fetch_sub(1, Ordering::SeqCst);
    }
}

/// How a child ended, read from its log.
#[derive(Debug, PartialEq, Eq)]
pub enum ChildEnd {
    Answered(String),
    /// Stopped by the token ceiling; the text is what it had.
    Truncated(String),
    Failed(String),
}

/// Read the outcome of a child's last turn from its log.
pub fn child_outcome(agent: &dyn AgentView) -> ChildEnd {
    let session = agent.session();
    let reason = session
        .events()
        .into_iter()
        .rev()
        .find_map(|e| match e.data {
            SessionEventData::TurnEnd { reason, .. } => Some(reason),
            _ => None,
        });
    let answer = session
        .derive_messages()
        .iter()
        .rev()
        .find(|m| m.role == Role::Assistant && !m.text().trim().is_empty())
        .map(|m| m.text())
        .unwrap_or_default();
    match reason {
        Some(TurnEndReason::Completed) if !answer.trim().is_empty() => ChildEnd::Answered(answer),
        Some(TurnEndReason::Completed) => {
            ChildEnd::Failed("the sub-agent finished without writing an answer".into())
        }
        Some(TurnEndReason::MaxTokens) if !answer.trim().is_empty() => ChildEnd::Truncated(answer),
        Some(TurnEndReason::Error { message, .. }) => {
            ChildEnd::Failed(format!("the sub-agent failed: {message}"))
        }
        Some(TurnEndReason::Aborted { cause }) => {
            ChildEnd::Failed(format!("the sub-agent was stopped ({cause})"))
        }
        Some(other) => ChildEnd::Failed(format!("the sub-agent ended early ({other:?})")),
        None => ChildEnd::Failed("the sub-agent never started a turn".into()),
    }
}

/// Recover public text already committed by the child. The current stream is
/// assembled with the same delta/block-end rules as the driver; reasoning and
/// incomplete tool calls are not returned. If no current text exists, use its
/// most recent assembled public message.
fn partial_answer(agent: &dyn AgentView) -> String {
    let events = agent.session().events();
    let after = events
        .iter()
        .rposition(|e| matches!(e.data, SessionEventData::AssistantMessage { .. }))
        .map_or(0, |i| i + 1);
    let mut assembler = crate::llm::BlockAssembler::new();
    for event in &events[after..] {
        if let SessionEventData::AssistantChunk { chunk, .. } = &event.data {
            assembler.push(chunk);
        }
    }
    let text: String = assembler
        .blocks()
        .iter()
        .filter_map(ContentBlock::as_text)
        .collect();
    if !text.trim().is_empty() {
        return text;
    }
    events
        .iter()
        .rev()
        .find_map(|event| match &event.data {
            SessionEventData::AssistantMessage { message, .. } => {
                let text: String = message
                    .content
                    .iter()
                    .filter_map(ContentBlock::as_text)
                    .collect();
                (!text.trim().is_empty()).then_some(text)
            }
            _ => None,
        })
        .unwrap_or_default()
}

const TIMEOUT_NOTE: &str = "\n[sub-agent timed out; partial answer is incomplete and unverified]";
const CUT_NOTE: &str = "\n[answer cut]";
const INCOMPLETE_NOTE: &str =
    "\n[the sub-agent hit its output limit; this answer may be incomplete]";

/// `text` followed by `note`, never longer than `max` characters in total.
/// When it does not fit, the body is cut and [`CUT_NOTE`] says so.
fn bounded(text: &str, note: &str, max: usize) -> String {
    let note_len = note.chars().count();
    if text.chars().count() + note_len <= max {
        return format!("{text}{note}");
    }
    let room = max.saturating_sub(note_len + CUT_NOTE.chars().count());
    let head: String = text.chars().take(room).collect();
    format!("{head}{CUT_NOTE}{note}")
}

async fn run_child(
    config: &SubagentConfig,
    registry: &Arc<Registry>,
    args: Value,
    run: crate::types::ToolRunContext,
) -> ToolExecutionResult {
    let Some(task) = args
        .get("task")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|t| !t.is_empty())
    else {
        return ToolExecutionResult::error("BAD_ARGS", "task is required and must not be empty");
    };
    let Some(parent_id) = run.agent_id.clone() else {
        return ToolExecutionResult::error("NO_AGENT", "this tool can only be called by an agent");
    };
    let depth = registry.depth_of(&parent_id) + 1;
    if depth > config.max_depth {
        return ToolExecutionResult::error(
            "DEPTH_LIMIT",
            format!(
                "sub-agents may not delegate further (limit {}). Do this task yourself.",
                config.max_depth
            ),
        );
    }
    let Some(agents) = run
        .ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
    else {
        return ToolExecutionResult::error("NO_AGENTS", "the agents service is not available");
    };
    let agents = (*agents).clone();
    let Some(parent) = agents.get(&parent_id) else {
        return ToolExecutionResult::error(
            "NO_PARENT",
            "the calling agent is no longer registered",
        );
    };
    let Some(header) = parent.session().request_header() else {
        return ToolExecutionResult::error(
            "NO_MODEL",
            "cannot tell which model the parent is using",
        );
    };
    if let Err(reason) = registry.reserve(config.max_concurrent) {
        return ToolExecutionResult::error("CONCURRENCY_LIMIT", reason);
    }

    let options = AgentOptions {
        provider: header.config.provider.clone(),
        model: header.config.model.clone(),
        max_tokens: config.max_tokens.or(header.config.max_tokens),
    };
    let child_id = format!("{parent_id}-sub-{}", unique_id());
    // The child runs the preset its parent runs.
    let parent_preset = agents
        .get(&parent_id)
        .and_then(|parent| parent.session().agent_preset());
    let child = match agents.create(Some(child_id.clone()), options, run.cwd.clone(), None, parent_preset) {
        Ok(child) => child,
        Err(reason) => {
            registry.live.fetch_sub(1, Ordering::SeqCst);
            return ToolExecutionResult::error("SPAWN_FAILED", reason);
        }
    };
    // The child starts under the permissions of the agent that started it.
    if let (Some(parent), true) = (agents.get(&parent_id), crate::permission::provided(&run.ctx).is_some()) {
        crate::permission::Permissions::inherit(&*parent.session(), &*child.session());
    }
    registry
        .depth
        .lock()
        .unwrap()
        .insert(child_id.clone(), depth);
    registry.link(&child_id, &parent_id);
    let _guard = ChildGuard {
        agents,
        agent: Arc::clone(&child),
        registry: Arc::clone(registry),
    };

    child.followup(Message {
        id: format!("{child_id}-task"),
        role: Role::User,
        content: vec![ContentBlock::text(task)],
        source: crate::types::MessageSource::User { rpc_id: None },
    });

    let timed_out = tokio::select! {
        biased;
        _ = run.signal.cancelled() => {
            return ToolExecutionResult::error("SUBAGENT_STOPPED", "the parent turn was cancelled; the sub-agent was stopped");
        }
        result = tokio::time::timeout(Duration::from_millis(config.timeout_ms), child.when_idle()) => result.is_err(),
    };
    if timed_out {
        child.cancel(AgentCancelCause::Parent, false);
        let text = partial_answer(child.as_ref());
        if !text.trim().is_empty() {
            return ToolExecutionResult::success_text(
                bounded(&text, TIMEOUT_NOTE, config.max_result_chars),
                json!({ "agent": child_id, "depth": depth, "timedOut": true, "complete": false }),
            );
        }
        return ToolExecutionResult::error("SUBAGENT_STOPPED", format!(
            "the sub-agent did not finish within {} ms and was stopped; no public text was logged", config.timeout_ms
        ));
    }

    let meta = json!({ "agent": child_id, "depth": depth });
    match child_outcome(child.as_ref()) {
        ChildEnd::Answered(text) => {
            ToolExecutionResult::success_text(bounded(&text, "", config.max_result_chars), meta)
        }
        ChildEnd::Truncated(text) => ToolExecutionResult::success_text(
            bounded(&text, INCOMPLETE_NOTE, config.max_result_chars),
            meta,
        ),
        ChildEnd::Failed(reason) => ToolExecutionResult::error("SUBAGENT_FAILED", reason),
    }
}

/// Answer a sub-agent's question from the parent model.
///
/// The parent's own turn is blocked while the child runs, so nothing can reach
/// it live. Instead the plugin replays the parent's recent history to the same
/// model and returns its answer, then logs the exchange in the parent session
/// so the parent can see what it was asked and what it decided.
async fn ask_parent(
    config: &SubagentConfig,
    registry: &Arc<Registry>,
    args: Value,
    run: crate::types::ToolRunContext,
) -> ToolExecutionResult {
    let Some(question) = args
        .get("question")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|q| !q.is_empty())
    else {
        return ToolExecutionResult::error(
            "BAD_ARGS",
            "question is required and must not be empty",
        );
    };
    let Some(child_id) = run.agent_id.clone() else {
        return ToolExecutionResult::error("NO_AGENT", "this tool can only be called by an agent");
    };
    let Some(parent_id) = registry.parent_of(&child_id) else {
        return ToolExecutionResult::error(
            "NOT_A_CHILD",
            "ask_parent is for sub-agents; you are the main agent, so decide it yourself",
        );
    };
    let Some(agents) = run
        .ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
    else {
        return ToolExecutionResult::error("NO_AGENTS", "the agents service is not available");
    };
    let Some(parent) = agents.get(&parent_id) else {
        return ToolExecutionResult::error("NO_PARENT", "the agent that started you is gone");
    };
    let Some(header) = parent.session().request_header() else {
        return ToolExecutionResult::error(
            "NO_MODEL",
            "cannot tell which model the parent is using",
        );
    };
    let Some(llm) = run.ctx.get::<LlmService>(crate::api::LLM_SERVICE) else {
        return ToolExecutionResult::error("NO_LLM", "the llm service is not available");
    };
    let Some(streams) = run.ctx.get::<StreamTable>(crate::api::LLM_STREAMS_SERVICE) else {
        return ToolExecutionResult::error("NO_LLM", "the llm stream table is not available");
    };
    let llm = (*llm).clone();
    let streams = (*streams).clone();

    // The tail of the parent's conversation is the context the answer is drawn
    // from; an older head would cost tokens without changing the decision.
    let mut messages: Vec<Message> = {
        let all = match crate::attachments::request_messages(
            &run.ctx,
            parent.session().derive_messages(),
            &header.config.provider,
            &header.config.model,
        ) {
            Ok(all) => all,
            Err(error) => {
                return ToolExecutionResult::error(
                    "ATTACHMENT_UNAVAILABLE",
                    &format!("{}: {}", error.code, error.message),
                )
            }
        };
        let keep = config.ask_context_messages.min(all.len());
        all[all.len() - keep..].to_vec()
    };
    messages.push(Message::user(
        format!("ask-{child_id}"),
        vec![ContentBlock::text(format!(
            "Sub-agent `{child_id}` asks:\n{question}"
        ))],
    ));
    let options = GenerateOptions {
        provider: header.config.provider.clone(),
        model: header.config.model.clone(),
        messages,
        system: Some(ASK_SYSTEM.to_string()),
        tools: None,
        temperature: Some(0.0),
        max_tokens: Some(config.ask_max_tokens),
        stop: None,
        session_id: Some(format!("{child_id}-ask")),
    };

    let work = async {
        let stream = stream_via_waterfall(&run.ctx, llm, streams, options)
            .await
            .map_err(|e| e.to_string())?;
        let mut stream = std::pin::pin!(stream);
        let mut assembler = BlockAssembler::new();
        while let Some(chunk) = futures::StreamExt::next(&mut stream).await {
            assembler.push(&chunk);
        }
        match assembler.finish() {
            FinishReason::Error { failure } | FinishReason::Aborted { failure } => {
                Err(format!("the parent model failed: {failure:?}"))
            }
            _ => Ok(assembler
                .blocks()
                .iter()
                .filter_map(|b| b.as_text())
                .collect::<String>()),
        }
    };
    let answer =
        match tokio::time::timeout(Duration::from_millis(config.ask_timeout_ms), work).await {
            Err(_) => {
                return ToolExecutionResult::error(
                    "ASK_TIMEOUT",
                    format!(
                    "the main agent did not answer within {} ms; proceed with your best judgement",
                    config.ask_timeout_ms
                ),
                )
            }
            Ok(Err(reason)) => return ToolExecutionResult::error("ASK_FAILED", reason),
            Ok(Ok(text)) => text.trim().to_string(),
        };
    if answer.is_empty() {
        return ToolExecutionResult::error("ASK_FAILED", "the main agent returned no text");
    }

    // Model-visible means logged: the parent sees what it was asked and answered.
    parent
        .session()
        .append(SessionEventData::UserMessage {
            message: Message {
                id: format!("ask-log-{child_id}"),
                role: Role::User,
                content: vec![ContentBlock::text(format!(
                    "Sub-agent `{child_id}` asked: {question}\n\nAnswer given from this conversation: {answer}"
                ))],
                source: MessageSource::Plugin {
                    plugin: "subagent".to_string(),
                    form: Some(ContextForm::Relay),
                },
            },
        });

    ToolExecutionResult::success_text(
        answer.clone(),
        json!({ "answeredBy": "main agent", "parent": parent_id, "chars": answer.len() }),
    )
}

/// The `subagent` plugin. Requires the `tools` and `agents` services.
pub fn subagent_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "subagent",
        vec![
            cordis::Injection::new(crate::api::TOOLS_SERVICE.to_string()),
            cordis::Injection::new(crate::api::AGENTS_SERVICE.to_string()),
        ],
        |ctx, config: SubagentConfig| async move {
            config.validate().map_err(cordis::Error::msg)?;
            let config = Arc::new(config);
            let registry = Arc::new(Registry::default());
            ctx.provide(LINEAGE_SERVICE, Lineage(Arc::clone(&registry)))
                .await?;
            let tool_config = Arc::clone(&config);
            let tool_registry = Arc::clone(&registry);
            let spec = tool_spec_ctx(
                "subagent",
                "Hand a self-contained task to a fresh sub-agent and get its final answer. The sub-agent has the same tools but does NOT see this conversation, so put everything it needs in `task`. Use it for independent research or checks, not for work you can do in one step.",
                json!({
                    "type": "object",
                    "properties": {
                        "task": { "type": "string", "description": "Complete instructions, including any file paths, constraints and the form of answer wanted." }
                    },
                    "required": ["task"]
                }),
                move |args, run| {
                    let config = Arc::clone(&tool_config);
                    let registry = Arc::clone(&tool_registry);
                    async move { run_child(&config, &registry, args, run).await }
                },
            );
            let ask_config = Arc::clone(&config);
            let ask_registry = Arc::clone(&registry);
            let ask = tool_spec_ctx(
                "ask_parent",
                "Ask the main agent a question when a decision or missing fact blocks you. It answers from the conversation it already has, one answer per call, with no follow-ups. Use it once for something you cannot resolve yourself.",
                json!({
                    "type": "object",
                    "properties": {
                        "question": { "type": "string", "description": "The specific decision or fact you need, and what you plan to do with it." }
                    },
                    "required": ["question"]
                }),
                move |args, run| {
                    let config = Arc::clone(&ask_config);
                    let registry = Arc::clone(&ask_registry);
                    async move { ask_parent(&config, &registry, args, run).await }
                },
            );
            register_tools(&ctx, "subagent tools", vec![spec, ask]).await
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slot_is_reserved_until_the_limit_and_the_counter_never_overshoots() {
        let registry = Registry::default();
        assert!(registry.reserve(2).is_ok());
        assert!(registry.reserve(2).is_ok());
        assert!(registry.reserve(2).unwrap_err().contains("2 sub-agent(s)"));
        assert_eq!(registry.live.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn agents_not_in_the_map_are_roots() {
        let registry = Registry::default();
        assert_eq!(registry.depth_of("main"), 0);
        registry
            .depth
            .lock()
            .unwrap()
            .insert("main-sub-1".into(), 1);
        assert_eq!(registry.depth_of("main-sub-1"), 1);
    }

    #[test]
    fn the_returned_answer_never_exceeds_the_limit_notes_included() {
        assert_eq!(bounded("short", "", 300), "short");
        for (text_len, note, max) in [
            (500, "", 300),
            (500, INCOMPLETE_NOTE, 300),
            (250, INCOMPLETE_NOTE, 300),
            (300, "", 300),
            (10, INCOMPLETE_NOTE, 200),
        ] {
            let out = bounded(&"x".repeat(text_len), note, max);
            assert!(
                out.chars().count() <= max,
                "{text_len}/{note:?}/{max}: {}",
                out.chars().count()
            );
            if text_len + note.chars().count() > max {
                assert!(out.contains(CUT_NOTE), "a cut is announced");
            }
            assert!(out.ends_with(note));
        }
    }

    #[test]
    fn settings_are_validated() {
        assert!(SubagentConfig::default().validate().is_ok());
        for bad in [
            SubagentConfig {
                max_depth: 0,
                ..Default::default()
            },
            SubagentConfig {
                max_concurrent: 0,
                ..Default::default()
            },
            SubagentConfig {
                timeout_ms: 0,
                ..Default::default()
            },
            SubagentConfig {
                max_result_chars: 5,
                ..Default::default()
            },
        ] {
            assert!(bad.validate().is_err());
        }
    }
}
