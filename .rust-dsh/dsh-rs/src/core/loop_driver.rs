//! The default agent driver — the Rust analogue of `packages/core/agent-loop`.
//!
//! One driver task per agent: wait for inbox work, then drain turns. Each
//! turn runs the reference flow:
//!
//! ```text
//! turn/start
//!   claim next-step input plus one queued message
//!   assemble prompt sections + tool schemas
//!   -> agent/pre-step                    reject | enter(messages)
//!      agent/step-start                  extra messages + system text, every step
//!      step/start
//!      append entered messages as user/message
//!      derive model history from the log
//!      agent/request -> llm/stream -> assistant/chunk* -> assistant/message
//!      tool/call* -> tools/execute -> tool/result*
//!      step/end
//!      tools owe another request, or next-step input arrived -> next step
//!   -> agent/turn-stopping
//! turn/end
//! ```

use std::sync::Arc;

use serde_json::{json, Value};

use crate::api::services::{LlmService, SessionView, ToolsService};
use crate::llm::{
    BlockAssembler, CancelToken, ContentBlock, GenerateOptions, LlmCallConfig, Message,
    MessageSource, Role, StreamChunk, StreamTable, stream_via_waterfall,
};
use crate::session::{
    EpochHeader, SessionEventData, TurnEndReason,
};
use crate::tools::{ToolExecutionResult, ToolRunContext};

use crate::core::agent::Agent;

/// Spawn the driver task for one agent (detached).
pub fn spawn_driver(agent: Arc<Agent>) {
    tokio::spawn(drive(agent));
}

async fn drive(agent: Arc<Agent>) {
    let mut wake = agent.wake_receiver();
    loop {
        // Wait for work: if none, block until the inbox generation changes.
        // Copy the version out so the read guard drops before awaiting.
        if !agent.has_work() {
            let _version = *wake.borrow_and_update();
            if agent.is_disposed() {
                break;
            }
            if wake.changed().await.is_err() {
                break;
            }
            continue;
        }
        if agent.is_disposed() {
            break;
        }
        agent.begin_turn(CancelToken::new());
        loop {
            let claimed = agent.claim_batch();
            if claimed.is_empty() {
                break;
            }
            if let Err(err) = run_turn(&agent, claimed).await {
                crate::api::events::emit(
                    &agent.ctx,
                    &crate::api::events::AgentErrorPayload {
                        agent: agent.id.clone(),
                        error: err.to_string(),
                    },
                );
            }
            if agent.is_disposed() {
                break;
            }
        }
        agent.end_turn().await;
    }
}

fn services(agent: &Agent) -> (LlmService, StreamTable, ToolsService) {
    let llm = agent
        .ctx
        .get::<LlmService>(crate::api::LLM_SERVICE)
        .expect("llm service present while agent loop is active");
    let streams = agent
        .ctx
        .get::<StreamTable>(crate::api::LLM_STREAMS_SERVICE)
        .expect("llmStreams service present while agent loop is active");
    let tools = agent
        .ctx
        .get::<ToolsService>(crate::api::TOOLS_SERVICE)
        .expect("tools service present while agent loop is active");
    ((*llm).clone(), (*streams).clone(), (*tools).clone())
}

/// Run one turn. A failure after the turn opened (a refused or failed model
/// request, a bad hook result) closes the open step and turn with an `error`
/// reason, so the log never keeps a turn open that nothing will finish and
/// the client can show why the turn ended.
async fn run_turn(agent: &Arc<Agent>, claimed: Vec<Message>) -> Result<(), crate::core::AgentError> {
    let result = run_turn_steps(agent, claimed).await;
    if let Err(error) = &result {
        close_failed_turn(&*agent.session, &error.to_string());
    }
    result
}

/// Append the closers a failed turn still owes: its open step, then the turn.
fn close_failed_turn(session: &dyn SessionView, message: &str) {
    let Some(turn) = session.open_turn() else {
        return;
    };
    let open_step = session
        .events()
        .iter()
        .rev()
        .take_while(|event| !matches!(event.data, SessionEventData::TurnStart { .. }))
        .find_map(|event| match event.data {
            SessionEventData::StepEnd { .. } => Some(None),
            SessionEventData::StepStart { step, .. } => Some(Some(step)),
            _ => None,
        })
        .flatten();
    if let Some(step) = open_step {
        session.append(SessionEventData::StepEnd { turn, step });
    }
    session.append(SessionEventData::TurnEnd {
        turn,
        reason: TurnEndReason::Error {
            message: message.to_string(),
            code: "agent/turn-failed".to_string(),
        },
    });
}

/// Join the assembled prompt and a preset's instructions, one blank line apart.
fn join_system(base: &str, extra: &str) -> String {
    let base = base.trim_end();
    let extra = extra.trim();
    if extra.is_empty() {
        return base.to_string();
    }
    if base.is_empty() {
        extra.to_string()
    } else {
        format!("{base}\n\n{extra}")
    }
}

async fn run_turn_steps(agent: &Arc<Agent>, claimed: Vec<Message>) -> Result<(), crate::core::AgentError> {
    let session = &agent.session; // Arc<dyn SessionView>
    let (llm, streams, tools) = services(agent);

    let turn = next_turn_number(&**session)?;
    session.append(SessionEventData::TurnStart { turn });

    // agent/pre-step waterfall: listeners may reject or rewrite the batch.
    let pre_payload = json!({
        "agent": agent.id,
        "messages": claimed,
        "turn": turn,
    });
    let fallback_messages = claimed.clone();
    let decision = agent
        .ctx
        .waterfall("agent/pre-step", pre_payload, move |_payload| {
            let fallback = fallback_messages.clone();
            Box::pin(async move {
                Ok(json!({ "kind": "enter", "messages": fallback }))
            })
        })
        .await
        .map_err(|err| crate::core::AgentError::Loop(err.to_string()))?;

    let mut messages: Vec<Message> = match decision.get("kind").and_then(|k| k.as_str()) {
        Some("enter") => serde_json::from_value(
            decision
                .get("messages")
                .cloned()
                .unwrap_or_else(|| Value::Array(vec![])),
        )
        .map_err(|err| crate::core::AgentError::Loop(err.to_string()))?,
        Some("reject") => {
            session.append(SessionEventData::TurnEnd {
                turn,
                reason: TurnEndReason::Blocked,
            });
            return Ok(());
        }
        _ => {
            session.append(SessionEventData::TurnEnd {
                turn,
                reason: TurnEndReason::Blocked,
            });
            return Ok(());
        }
    };

    if messages.is_empty() {
        session.append(SessionEventData::TurnEnd {
            turn,
            reason: TurnEndReason::Blocked,
        });
        return Ok(());
    }

    let config = resolve_config(agent, turn).await?;
    let prompt_service = agent
        .ctx
        .get::<crate::api::services::SystemPromptService>(crate::api::SYSTEM_PROMPT_SERVICE);
    let mut prompt = prompt_service
        .as_ref()
        .map(|p| p.assemble())
        .unwrap_or_default();
    // The preset the session runs contributes its own instructions, so the
    // composition a preset names is the one the model sees. The session is
    // bound to the revision it was created or last changed with, so a roster
    // change does not alter a running session.
    if let Some(text) = session.preset_instructions() {
        prompt.system = join_system(&prompt.system, &text);
    }

    let mut step: u64 = 0;
    loop {
        step += 1;
        // agent/step-start waterfall: runs before every step, so a plugin can
        // apply state that changes mid-turn. Listeners add messages (logged
        // below like any entered message) and system text for this request.
        let (extra_messages, extra_system) = step_start(agent, turn, step).await;
        messages.extend(extra_messages);
        let system = join_system(&prompt.system, &extra_system.join("\n\n"));
        session.append(SessionEventData::StepStart { turn, step });
        for message in &messages {
            session.append(SessionEventData::UserMessage { message: message.clone() });
            // The prompt is in the log now, so the log answers for its
            // identity and the claimed set does not have to.
            agent.forget_claimed_prompt(message);
        }

        let history = crate::attachments::request_messages(
            &agent.ctx,
            session.derive_messages(),
            &config.provider,
            &config.model,
        )
        .map_err(|error| {
            crate::core::AgentError::Loop(format!("{}: {}", error.code, error.message))
        })?;
        let tool_schemas = tools.schemas();
        let options = GenerateOptions {
            provider: config.provider.clone(),
            model: config.model.clone(),
            messages: history,
            system: if system.is_empty() { None } else { Some(system) },
            tools: if tool_schemas.is_empty() { None } else { Some(tool_schemas) },
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            stop: config.stop.clone(),
            session_id: Some(agent.id.clone()),
        };

        let header = EpochHeader {
            config: config.clone(),
            system: options.system.clone(),
            tools: options.tools.clone(),
        };
        session.append(SessionEventData::RequestHeader { header });

        // The logged request prefix is durable before the model is called, so a
        // crash cannot leave a request the log does not explain.
        agent
            .checkpoint()
            .await
            .map_err(crate::core::AgentError::Loop)?;

        // Stream the model response, logging raw chunks for replay fidelity.
        // The whole stream setup is raced with the turn token so cancellation
        // interrupts a provider that hangs before emitting any chunk.
        let stream = match agent.turn_token() {
            Some(token) => {
                match token
                    .race(stream_via_waterfall(&agent.ctx, llm.clone(), streams.clone(), options))
                    .await
                {
                    None => {
                        session.append(SessionEventData::StepEnd { turn, step });
                        session.append(SessionEventData::TurnEnd {
                            turn,
                            reason: TurnEndReason::Aborted { cause: "user".to_string() },
                        });
                        return Ok(());
                    }
                    Some(result) => result
                        .map_err(|err| crate::core::AgentError::Loop(err.to_string()))?,
                }
            }
            None => stream_via_waterfall(&agent.ctx, llm.clone(), streams.clone(), options)
                .await
                .map_err(|err| crate::core::AgentError::Loop(err.to_string()))?,
        };
        let mut assembler = BlockAssembler::new();
        let mut stream = std::pin::pin!(stream);
        let mut interrupted = false;
        let mut usage = None;
        loop {
            let next = futures::StreamExt::next(&mut stream);
            let chunk = match agent.turn_token() {
                Some(token) => match token.race(next).await {
                    None => {
                        interrupted = true;
                        break;
                    }
                    Some(chunk) => chunk,
                },
                None => next.await,
            };
            let Some(chunk) = chunk else { break };
            session.append(SessionEventData::AssistantChunk {
                turn,
                step,
                chunk: chunk.clone(),
            });
            if let StreamChunk::Usage { usage: u } = &chunk {
                usage = Some(*u);
            }
            assembler.push(&chunk);
            if matches!(chunk, StreamChunk::Finish { .. }) {
                break;
            }
        }

        if interrupted {
            let blocks = assembler.interrupted_blocks();
            if !blocks.is_empty() {
                let message = Message {
                    id: format!("m-{turn}-{step}"),
                    role: Role::Assistant,
                    content: blocks,
                    source: MessageSource::Model {
                        provider: config.provider.clone(),
                        model: config.model.clone(),
                    },
                };
                session.append(SessionEventData::AssistantMessage {
                    turn,
                    step,
                    message,
                    usage: None,
                    interrupted: Some(true),
                });
            }
            session.append(SessionEventData::StepEnd { turn, step });
            session.append(SessionEventData::TurnEnd {
                turn,
                reason: TurnEndReason::Aborted { cause: "user".to_string() },
            });
            return Ok(());
        }

        let message = assembler.message(
            format!("m-{turn}-{step}"),
            MessageSource::Model {
                provider: config.provider.clone(),
                model: config.model.clone(),
            },
        );
        session.append(SessionEventData::AssistantMessage {
            turn,
            step,
            message: message.clone(),
            usage,
            interrupted: None,
        });

        if matches!(assembler.finish(), crate::llm::FinishReason::MaxTokens) {
            session.append(SessionEventData::StepEnd { turn, step });
            session.append(SessionEventData::TurnEnd {
                turn,
                reason: TurnEndReason::MaxTokens,
            });
            return Ok(());
        }

        let tool_calls = message.tool_calls();
        if tool_calls.is_empty() {
            session.append(SessionEventData::StepEnd { turn, step });
            break;
        }

        // Dispatch tool calls (sequentially in this driver; the registry's
        // `is_concurrency_safe` flag is the future parallel-group seam).
        for call in &tool_calls {
            let ContentBlock::ToolCall { id, name, arguments } = call else { continue };
            session.append(SessionEventData::ToolCall {
                turn,
                step,
                call_id: id.clone(),
                name: name.clone(),
                arguments: arguments.clone(),
            });
            // The recorded call is durable before the tool body runs, so a
            // crash cannot leave a side effect the log does not explain.
            agent
                .checkpoint()
                .await
                .map_err(crate::core::AgentError::Loop)?;
            let parsed_args = serde_json::from_str(arguments).unwrap_or(Value::Null);
            let run_ctx = ToolRunContext {
                ctx: agent.ctx.clone(),
                signal: agent.turn_token().unwrap_or_default(),
                agent_id: Some(agent.id.clone()),
                cwd: agent.session.header_cwd(),
            };
            let result = tools
                .execute(id.clone(), name.clone(), parsed_args, run_ctx)
                .await;
            let result_message = tool_result_message(id, name, &result);
            let committed = session.append(SessionEventData::ToolResult {
                turn,
                step,
                message: result_message,
            });
            // The executor's own value is what a client waiting for the result
            // reads; the log carries the model's view of it, which is not the
            // same value.
            if let ToolExecutionResult::Success { value, .. } = &result {
                let _ = agent
                    .ctx
                    .emit_sync(
                        "toolResults/value",
                        serde_json::json!({
                            "session": session.id(),
                            "seq": committed.seq,
                            "value": value,
                        }),
                    )
                    .await;
            }
        }
        agent.completed_turns.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        session.append(SessionEventData::StepEnd { turn, step });

        // A tool call owes another model request. Claim any steering; an
        // empty batch still enters the next step with no new messages.
        messages = agent.claim_next_step();
        let owes_request = !tool_calls.is_empty();
        if !owes_request && messages.is_empty() {
            break;
        }
        // When only tool results continue the turn, the next step's entered
        // messages are empty (history derives the tool results).
    }

    // agent/turn-stopping: serial extension point before the turn closes.
    let _ = agent
        .ctx
        .serial(
            "agent/turn-stopping",
            json!({ "agent": agent.id, "turn": turn }),
        )
        .await;
    session.append(SessionEventData::TurnEnd {
        turn,
        reason: TurnEndReason::Completed,
    });
    Ok(())
}

/// Run the `agent/step-start` waterfall: the messages and system text the
/// listeners add to this step. A failed waterfall adds nothing, and an entry
/// that does not parse is dropped; both are reported on stderr, since a
/// listener's contribution (plan mode's guidance, say) is then missing.
async fn step_start(agent: &Arc<Agent>, turn: u64, step: u64) -> (Vec<Message>, Vec<String>) {
    let payload = json!({ "agent": agent.id, "turn": turn, "step": step });
    let outcome = agent
        .ctx
        .waterfall("agent/step-start", payload, |_payload| {
            Box::pin(async { Ok(json!({ "messages": [], "system": [] })) })
        })
        .await;
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => {
            eprintln!("agent/step-start failed for {} (turn {turn}, step {step}): {error}", agent.id);
            return (Vec::new(), Vec::new());
        }
    };
    let entries = |key: &str| outcome.get(key).and_then(Value::as_array).cloned().unwrap_or_default();
    let mut messages = Vec::new();
    for entry in entries("messages") {
        match serde_json::from_value::<Message>(entry) {
            Ok(message) => messages.push(message),
            Err(error) => eprintln!("agent/step-start: dropped a malformed message: {error}"),
        }
    }
    let mut system = Vec::new();
    for entry in entries("system") {
        match entry {
            Value::String(text) => system.push(text),
            other => eprintln!("agent/step-start: dropped a non-text system entry: {other}"),
        }
    }
    (messages, system)
}

/// The next turn number: one past the last logged `turn/start`.
fn next_turn_number(session: &dyn SessionView) -> Result<u64, crate::core::AgentError> {
    let mut max = 0u64;
    for event in session.events() {
        if let SessionEventData::TurnStart { turn } = event.data {
            max = max.max(turn);
        }
    }
    Ok(max + 1)
}

/// Resolve the call configuration through the `agent/request` waterfall.
async fn resolve_config(agent: &Arc<Agent>, turn: u64) -> Result<LlmCallConfig, crate::core::AgentError> {
    let options = agent.options();
    let default = LlmCallConfig {
        provider: options.provider,
        model: options.model,
        temperature: None,
        max_tokens: options.max_tokens,
        stop: None,
    };
    let fallback = default.clone();
    let result = agent
        .ctx
        .waterfall(
            "agent/request",
            json!({ "agent": agent.id, "turn": turn }),
            move |_payload| {
                let fallback = fallback.clone();
                Box::pin(async move { Ok(json!(fallback)) })
            },
        )
        .await
        .map_err(|err| crate::core::AgentError::Loop(err.to_string()))?;
    serde_json::from_value(result).map_err(|err| crate::core::AgentError::Loop(err.to_string()))
}

/// Build the user-role message carrying a tool's model-facing result.
fn tool_result_message(call_id: &str, name: &str, result: &ToolExecutionResult) -> Message {
    let (text, is_error) = match result {
        ToolExecutionResult::Success { content, .. } => {
            let text = content
                .iter()
                .filter_map(|b| b.as_text())
                .collect::<Vec<_>>()
                .join("\n");
            (text, false)
        }
        ToolExecutionResult::Error { message, .. } => (message.clone(), true),
    };
    Message {
        id: format!("tr-{call_id}"),
        role: Role::User,
        content: vec![ContentBlock::ToolResult {
            tool_call_id: call_id.to_string(),
            content: vec![ContentBlock::text(text)],
            is_error: Some(is_error),
        }],
        source: MessageSource::Tool {
            tool: name.to_string(),
        },
    }
}

/// Build a user-role message carrying an injected context snapshot.
pub fn injected_message(id: impl Into<String>, text: impl Into<String>) -> Message {
    Message {
        id: id.into(),
        role: Role::User,
        content: vec![ContentBlock::text(text)],
        source: MessageSource::Plugin {
            plugin: "dsh-core/loop".to_string(),
            form: None,
        },
    }
}
