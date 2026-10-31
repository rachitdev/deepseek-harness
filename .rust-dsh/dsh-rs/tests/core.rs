use std::sync::{Arc, Mutex};

use cordis::plugin::BoxFuture;
use cordis::Context;
use dsh_rs::core::{AgentOptions, PromptSection, SystemPromptService, agent_loop_plugin, prompt::system_prompt_plugin};
use dsh_rs::core::agent::user_message_with_text;
use dsh_rs::llm::{ContentBlock, GenerateOptions, LlmAdapter, LlmError, StreamChunk, llm_plugin, runtime::stream_from_chunks};
use dsh_rs::session::{SessionEventData, TurnEndReason, session_plugin};
use dsh_rs::tools::tools_plugin;
use serde_json::{json, Value};

/// Compose the minimal harness plugin stack and wait for convergence.
async fn compose(ctx: &Context) -> Vec<cordis::FiberHandle> {
    let llm = ctx.plugin(llm_plugin(), Some(json!({})));
    let sessions = ctx.plugin(session_plugin(), None);
    let tools = ctx.plugin(tools_plugin(), Some(Value::Null));
    let prompt = ctx.plugin(system_prompt_plugin(), None);
    let agent_loop = ctx.plugin(agent_loop_plugin(), None);
    llm.join().await.unwrap();
    sessions.join().await.unwrap();
    tools.join().await.unwrap();
    prompt.join().await.unwrap();
    agent_loop.join().await.unwrap();
    vec![llm, sessions, tools, prompt, agent_loop]
}

#[tokio::test]
async fn prompt_assemble_orders_and_interpolates() {
    let ctx = Context::new();
    let service = SystemPromptService::new(ctx.clone());
    service.section(PromptSection::new("persona", 0, "You are {{name}}, a helpful agent."));
    service.section(PromptSection::new("harness", -100, "dsh-rs harness identity"));
    service.section(PromptSection::new("tools", 100, "You have tools."));
    service.variable("name", || Some("rover".to_string()));
    let assembly = service.assemble();
    assert!(assembly.system.starts_with("dsh-rs harness identity"));
    assert!(assembly.system.contains("You are rover, a helpful agent."));
    assert!(assembly.system.ends_with("You have tools."));
}

#[tokio::test]
async fn prompt_complete_section_wins() {
    let ctx = Context::new();
    let service = SystemPromptService::new(ctx.clone());
    service.section(PromptSection::new("persona", 0, "normal persona"));
    service.section(PromptSection::new("complete", -50, "ONLY THIS MATTERS").complete());
    let assembly = service.assemble();
    assert_eq!(assembly.system, "ONLY THIS MATTERS");
}

/// A message whose identity is already pending is refused: queueing it again
/// would write a log the inbox fold rejects.
#[tokio::test]
async fn a_message_that_is_already_pending_is_not_queued_twice() {
    let ctx = Context::new();
    compose(&ctx).await;
    let agents = ctx.require::<dsh_rs::api::services::AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("mock-1"), Some("/tmp".to_string()), None, None)
        .unwrap();

    // Injected context waits for a turn, so the first row stays pending.
    agent.inject(user_message_with_text("same-id", "one"));
    agent.inject(user_message_with_text("same-id", "two"));

    let splices = agent
        .session()
        .events()
        .iter()
        .filter(|event| matches!(event.data, SessionEventData::InboxSpliced { .. }))
        .count();
    assert_eq!(splices, 1, "the second message is refused");
}

/// The pending lists are rebuilt from the durable splice events, and a splice
/// the log cannot explain is refused rather than repaired.
#[test]
fn the_inbox_fold_rebuilds_the_pending_rows_and_refuses_a_broken_splice() {
    use dsh_rs::core::agent::Inbox;
    use dsh_rs::types::{InboxOutcome, InboxTarget, Message, SessionEvent};

    let message = |id: &str| Message {
        id: id.to_string(),
        role: dsh_rs::types::Role::User,
        content: vec![ContentBlock::text("queued")],
        source: dsh_rs::types::MessageSource::User { rpc_id: None },
    };
    let splice = |seq: u64, start: usize, removed: Option<usize>, inserted: Vec<Message>| {
        SessionEvent::new(
            seq,
            0,
            dsh_rs::types::SessionEventData::InboxSpliced {
                target: InboxTarget::NextTurn,
                start,
                removed_count: removed,
                inserted,
                outcome: removed.map(|_| InboxOutcome::Canceled),
            },
        )
    };

    let events = vec![
        splice(0, 0, None, vec![message("m-1")]),
        splice(1, 0, Some(1), Vec::new()),
    ];
    let inbox = Inbox::fold(&events, None).unwrap();
    assert!(
        inbox.pending(InboxTarget::NextTurn).is_empty(),
        "the removed row is gone"
    );

    let broken = vec![splice(0, 3, None, vec![message("m-1")])];
    let error = Inbox::fold(&broken, None).unwrap_err();
    assert!(
        error.contains("outside the pending list"),
        "a splice outside the list is reported: {error}"
    );
}

/// An adapter that records the messages of every request it answers.
struct Recording {
    seen: Arc<Mutex<Vec<String>>>,
}

impl LlmAdapter for Recording {
    fn name(&self) -> &'static str {
        "recording"
    }
    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<dsh_rs::llm::BoxStream<StreamChunk>, LlmError>> {
        let texts: Vec<String> = options.messages.iter().map(|m| m.text()).collect();
        self.seen.lock().unwrap().push(texts.join("|"));
        Box::pin(async move {
            Ok(stream_from_chunks(
                dsh_rs::llm::adapters::mock::MockAdapter::text_response("ok"),
            ))
        })
    }
}

#[tokio::test]
async fn a_turn_reads_the_step_input_before_the_queued_prompt() {
    let ctx = Context::new();
    compose(&ctx).await;
    let runtime = ctx.require::<dsh_rs::api::services::LlmService>("llm").unwrap();
    runtime.unregister_adapter(&["mock"]);
    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    runtime
        .register_adapter(
            &["mock"],
            Arc::new(Recording {
                seen: seen.clone(),
            }),
        )
        .unwrap();

    let agents = ctx.require::<dsh_rs::api::services::AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("mock-1"), Some("/tmp".to_string()), None, None)
        .unwrap();

    // The step input is queued first and does not wake the driver; the prompt
    // wakes it, so one batch claims both.
    agent.inject(user_message_with_text("s-1", "step context"));
    agent.followup(user_message_with_text("u-1", "the prompt"));
    agent.when_idle().await;

    let requests = seen.lock().unwrap().clone();
    assert_eq!(requests.len(), 1, "one turn, one request");
    assert_eq!(
        requests[0], "step context|the prompt",
        "the claimed batch reads the step input before the queued prompt"
    );
}

#[tokio::test]
async fn agent_loop_runs_tool_then_finishes() {
    let ctx = Context::new();
    compose(&ctx).await;

    // Script the mock adapter: first a bash tool call, then the final answer.
    let runtime = ctx.require::<dsh_rs::api::services::LlmService>("llm").unwrap();
    runtime.unregister_adapter(&["mock"]);
    runtime
        .register_adapter(
            &["mock"],
            Arc::new(dsh_rs::llm::adapters::mock::MockAdapter::scripted(vec![
                dsh_rs::llm::adapters::mock::MockAdapter::tool_call_response(
                    "call-1",
                    "bash",
                    json!({ "command": "echo hello-dsh" }),
                ),
                dsh_rs::llm::adapters::mock::MockAdapter::text_response("task complete"),
            ])),
        )
        .unwrap();

    let agents = ctx.require::<dsh_rs::api::services::AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("mock-1"), Some("/tmp".to_string()), None, None)
        .unwrap();

    agent.followup(user_message_with_text("u-1", "list the files please"));
    agent.when_idle().await;

    let events = agent.session().events();
    let types: Vec<&str> = events.iter().map(|e| e.event_type()).collect();
    assert!(types.contains(&"turn/start"));
    assert!(types.contains(&"user/message"));
    assert!(types.contains(&"assistant/chunk"));
    assert!(types.contains(&"tool/call"));
    assert!(types.contains(&"tool/result"));
    assert!(types.contains(&"assistant/message"));
    assert!(types.contains(&"turn/end"));

    // The turn completed cleanly.
    let ends: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.data, SessionEventData::TurnEnd { .. }))
        .collect();
    assert_eq!(ends.len(), 1);
    match &ends[0].data {
        SessionEventData::TurnEnd { reason, .. } => {
            assert_eq!(*reason, TurnEndReason::Completed);
        }
        _ => unreachable!(),
    }

    // The final assistant message carries the model's answer.
    let messages = agent.session().derive_messages();
    let last = messages.last().unwrap();
    assert_eq!(last.text(), "task complete");

    // The bash tool result is in the log, with its output nested inside the
    // tool-result content block.
    let has_bash_result = events.iter().any(|e| {
        matches!(
            &e.data,
            SessionEventData::ToolResult { message, .. }
                if message_tool_result_text(message).contains("hello-dsh")
        )
    });
    assert!(has_bash_result);

    // Request headers were logged for the steps.
    let headers = events
        .iter()
        .filter(|e| matches!(e.data, SessionEventData::RequestHeader { .. }))
        .count();
    assert!(headers >= 2, "one header per model request, got {headers}");
}

#[tokio::test]
async fn agent_loop_uses_system_prompt_and_tool_schemas() {
    let ctx = Context::new();
    compose(&ctx).await;

    // Record what the model saw.
    let seen: Arc<std::sync::Mutex<Vec<GenerateOptions>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen_for_adapter = seen.clone();
    let runtime = ctx.require::<dsh_rs::api::services::LlmService>("llm").unwrap();
    runtime.unregister_adapter(&["mock"]);
    runtime
        .register_adapter(
            &["mock"],
            Arc::new(RecordingAdapter {
                inner: dsh_rs::llm::adapters::mock::MockAdapter::new(),
                seen: seen_for_adapter,
            }),
        )
        .unwrap();

    // Add a prompt section and a variable.
    let prompt = ctx.require::<dsh_rs::api::services::SystemPromptService>("systemPrompt").unwrap();
    prompt.section("persona", 0, "You are {{who}}, an agent.", false);
    prompt.variable("who", std::sync::Arc::new(|| Some("dsh-rs".to_string())));

    let agents = ctx.require::<dsh_rs::api::services::AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("mock-1"), Some("/tmp".to_string()), None, None)
        .unwrap();
    agent.followup(user_message_with_text("u-1", "hello"));
    agent.when_idle().await;

    let requests = seen.lock().unwrap().clone();
    let first = requests.first().expect("at least one request");
    assert!(first.system.as_deref().unwrap().contains("You are dsh-rs, an agent."));
    let tool_names: Vec<&str> = first
        .tools
        .as_ref()
        .unwrap()
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert!(tool_names.contains(&"bash"));
    assert!(tool_names.contains(&"todo_write"));
}

/// An adapter that records every request it serves, then delegates to mock.
struct RecordingAdapter {
    inner: dsh_rs::llm::adapters::mock::MockAdapter,
    seen: Arc<std::sync::Mutex<Vec<GenerateOptions>>>,
}

impl LlmAdapter for RecordingAdapter {
    fn name(&self) -> &'static str {
        "recording"
    }

    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<dsh_rs::llm::BoxStream<StreamChunk>, LlmError>> {
        self.seen.lock().unwrap().push(options.clone());
        self.inner.stream(options)
    }
}

/// An adapter that never answers until its gate opens; used to test cancel.
#[derive(Clone)]
struct GateAdapter {
    gate: Arc<tokio::sync::Notify>,
}

impl LlmAdapter for GateAdapter {
    fn name(&self) -> &'static str {
        "gate"
    }

    fn stream(
        &self,
        _options: GenerateOptions,
    ) -> BoxFuture<Result<dsh_rs::llm::BoxStream<StreamChunk>, LlmError>> {
        let gate = self.gate.clone();
        Box::pin(async move {
            gate.notified().await;
            Ok(stream_from_chunks(
                dsh_rs::llm::adapters::mock::MockAdapter::text_response("late answer"),
            ))
        })
    }
}

#[tokio::test]
async fn cancel_aborts_the_turn() {
    let ctx = Context::new();
    compose(&ctx).await;

    let runtime = ctx.require::<dsh_rs::api::services::LlmService>("llm").unwrap();
    runtime.unregister_adapter(&["mock"]);
    runtime
        .register_adapter(
            &["mock"],
            Arc::new(GateAdapter {
                gate: Arc::new(tokio::sync::Notify::new()),
            }),
        )
        .unwrap();

    let agents = ctx.require::<dsh_rs::api::services::AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("mock-1"), Some("/tmp".to_string()), None, None)
        .unwrap();
    agent.followup(user_message_with_text("u-1", "do something slow"));
    // Wait until the driver is mid-request.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    while !agent.driver_busy() && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(agent.driver_busy(), "driver should be running the request");
    agent.cancel(dsh_rs::core::AgentCancelCause::User, false);
    agent.when_idle().await;

    let events = agent.session().events();
    let ends: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.data, SessionEventData::TurnEnd { .. }))
        .collect();
    assert_eq!(ends.len(), 1);
    match &ends[0].data {
        SessionEventData::TurnEnd { reason, .. } => {
            assert_eq!(*reason, TurnEndReason::Aborted { cause: "user".to_string() });
        }
        _ => unreachable!(),
    }
}

#[tokio::test]
async fn todo_tool_writes_session_event() {
    let ctx = Context::new();
    compose(&ctx).await;

    let runtime = ctx.require::<dsh_rs::api::services::LlmService>("llm").unwrap();
    runtime.unregister_adapter(&["mock"]);
    runtime
        .register_adapter(
            &["mock"],
            Arc::new(dsh_rs::llm::adapters::mock::MockAdapter::scripted(vec![
                dsh_rs::llm::adapters::mock::MockAdapter::tool_call_response(
                    "call-1",
                    "todo_write",
                    json!({
                        "todos": [
                            { "content": "design", "status": "completed" },
                            { "content": "implement" }
                        ]
                    }),
                ),
                dsh_rs::llm::adapters::mock::MockAdapter::text_response("todos recorded"),
            ])),
        )
        .unwrap();

    let agents = ctx.require::<dsh_rs::api::services::AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("mock-1"), Some("/tmp".to_string()), None, None)
        .unwrap();
    agent.followup(user_message_with_text("u-1", "track my todos"));
    agent.when_idle().await;

    let events = agent.session().events();
    let todos: Vec<_> = events
        .iter()
        .filter_map(|e| match &e.data {
            SessionEventData::TodoWrite { todos } => Some(todos.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].len(), 2);
    assert_eq!(todos[0][0].content, "design");
    assert_eq!(todos[0][0].status, dsh_rs::session::TodoStatus::Completed);
}

#[tokio::test]
async fn agents_registry_tracks_live_agents() {
    let ctx = Context::new();
    compose(&ctx).await;
    let agents = ctx.require::<dsh_rs::api::services::AgentRegistryService>("agents").unwrap();
    let a = agents
        .create(Some("agent-x".into()), AgentOptions::mock("mock-1"), None, None, None)
        .unwrap();
    assert_eq!(a.id(), "agent-x");
    assert_eq!(agents.get("agent-x").unwrap().id(), "agent-x");
    assert_eq!(agents.list().len(), 1);
    agents.dispose(&a);
    assert!(agents.get("agent-x").is_none());
}

#[tokio::test]
async fn seed_prompt_enters_the_log() {
    let ctx = Context::new();
    compose(&ctx).await;
    let agents = ctx.require::<dsh_rs::api::services::AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("mock-1"),
            Some("/tmp".to_string()),
            Some("context seed".to_string()),
            None,
        )
        .unwrap();
    agent.followup(user_message_with_text("u-1", "go"));
    agent.when_idle().await;
    let events = agent.session().events();
    let users: Vec<_> = events
        .iter()
        .filter_map(|e| match &e.data {
            SessionEventData::UserMessage { message } => Some(message.text()),
            _ => None,
        })
        .collect();
    assert!(users.iter().any(|t| t == "context seed"));
    assert!(users.iter().any(|t| t == "go"));
}


/// Extract text nested inside a message's tool-result blocks.
fn message_tool_result_text(message: &dsh_rs::llm::Message) -> String {
    let mut out = String::new();
    for block in &message.content {
        if let ContentBlock::ToolResult { content, .. } = block {
            for inner in content {
                if let ContentBlock::Text { text } = inner {
                    out.push_str(text);
                }
            }
        }
    }
    out
}
