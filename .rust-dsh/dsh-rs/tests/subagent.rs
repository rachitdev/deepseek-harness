//! Sub-agents end to end: a real parent agent calls the real `subagent` tool.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::{
    AgentRegistryService, AgentView, BoxFuture, LlmAdapterApi, LlmService,
};
use dsh_rs::bundle::install_profile;
use dsh_rs::types::{
    stream_from_chunks, AgentCancelCause, AgentOptions, BoxStream, ContentBlock, GenerateOptions,
    LlmError, Message, SessionEventData, StreamChunk,
};
use futures::StreamExt;
use serde_json::{json, Value};

mod common;
use common::{text_response, tool_call_response};

#[derive(Clone)]
enum Reply {
    Text(&'static str),
    Tool(&'static str, Value),
    Hang,
    PartialHang,
    Fail,
}

struct Script {
    replies: Mutex<VecDeque<Reply>>,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
}

impl LlmAdapterApi for Script {
    fn name(&self) -> &'static str {
        "script"
    }
    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>> {
        self.seen.lock().unwrap().push(options);
        let reply = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Reply::Text("(script exhausted)"));
        Box::pin(async move {
            match reply {
                Reply::Text(t) => Ok(stream_from_chunks(text_response(t))),
                Reply::Tool(name, args) => {
                    Ok(stream_from_chunks(tool_call_response("call-1", name, args)))
                }
                Reply::PartialHang => Ok(Box::pin(
                    futures::stream::iter(vec![
                        StreamChunk::TextDelta {
                            index: 0,
                            text: "I found the config at src/index.rs".into(),
                        },
                        StreamChunk::ReasoningDelta {
                            index: 1,
                            text: "private reasoning is not an answer".into(),
                        },
                    ])
                    .chain(futures::stream::pending()),
                ) as BoxStream<StreamChunk>),
                Reply::Fail => Err(LlmError::code("PROVIDER_DOWN", "down")),
                Reply::Hang => {
                    std::future::pending::<()>().await;
                    unreachable!()
                }
            }
        })
    }
}

struct Rig {
    ctx: Context,
    parent: Arc<dyn AgentView>,
    agents: AgentRegistryService,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
}

async fn rig(config: Value, replies: Vec<Reply>) -> Rig {
    let ctx = Context::new();
    let profile = json!({ "plugins": { "subagent": config, "ask-user": {} } });
    install_profile(&ctx, &profile)
        .await
        .expect("profile installs");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(
        &["mock"],
        Arc::new(Script {
            replies: Mutex::new(replies.into()),
            seen: seen.clone(),
        }),
    )
    .unwrap();
    let agents = (*ctx.require::<AgentRegistryService>("agents").unwrap()).clone();
    let parent = agents
        .create(
            Some("main".into()),
            AgentOptions::mock("m"),
            Some("/tmp".into()),
            None,
            None,
        )
        .unwrap();
    Rig {
        ctx,
        parent,
        agents,
        seen,
    }
}

fn ask(rig: &Rig, text: &str) {
    rig.parent
        .followup(Message::user("u1", vec![ContentBlock::text(text)]));
}

fn tool_results(agent: &dyn AgentView) -> Vec<(String, bool)> {
    agent
        .session()
        .events()
        .into_iter()
        .filter_map(|e| match e.data {
            SessionEventData::ToolResult { message, .. } => {
                let is_error = message.content.iter().any(|b| {
                    matches!(
                        b,
                        ContentBlock::ToolResult {
                            is_error: Some(true),
                            ..
                        }
                    )
                });
                Some((
                    message
                        .content
                        .iter()
                        .filter_map(|b| match b {
                            ContentBlock::ToolResult { content, .. } => Some(
                                content
                                    .iter()
                                    .filter_map(|c| c.as_text())
                                    .collect::<String>(),
                            ),
                            _ => None,
                        })
                        .collect(),
                    is_error,
                ))
            }
            _ => None,
        })
        .collect()
}

fn children(rig: &Rig) -> Vec<String> {
    rig.agents
        .list()
        .iter()
        .map(|a| a.id().to_string())
        .filter(|id| id != "main")
        .collect()
}

#[tokio::test]
async fn the_child_answer_comes_back_as_the_tool_result_and_the_child_is_cleaned_up() {
    let rig = rig(
        json!({}),
        vec![
            Reply::Tool("subagent", json!({ "task": "find the bug in foo.rs" })),
            Reply::Text("the bug is on line 4"),    // the child
            Reply::Text("The sub-agent found it."), // the parent, after the tool result
        ],
    )
    .await;
    ask(&rig, "please investigate");
    rig.parent.when_idle().await;

    let results = tool_results(rig.parent.as_ref());
    assert_eq!(results, vec![("the bug is on line 4".to_string(), false)]);

    // The child saw only its task, not the parent's conversation.
    let seen = rig.seen.lock().unwrap();
    let child_request = &seen[1];
    let texts: Vec<String> = child_request.messages.iter().map(|m| m.text()).collect();
    assert_eq!(texts, ["find the bug in foo.rs"]);
    assert_eq!(child_request.model, "m", "inherits the parent's model");
    drop(seen);

    assert!(
        children(&rig).is_empty(),
        "the child agent is disposed: {:?}",
        children(&rig)
    );
    let _ = &rig.ctx;
}

#[tokio::test]
async fn a_child_cannot_delegate_further_at_the_default_depth() {
    let rig = rig(
        json!({}),
        vec![
            Reply::Tool("subagent", json!({ "task": "outer" })),
            Reply::Tool("subagent", json!({ "task": "inner" })), // the child tries to delegate
            Reply::Text("did it myself"),                        // the child, after the refusal
            Reply::Text("done"),                                 // the parent
        ],
    )
    .await;
    ask(&rig, "go");
    rig.parent.when_idle().await;
    assert_eq!(
        tool_results(rig.parent.as_ref()),
        vec![("did it myself".to_string(), false)]
    );
    // The refusal reached the child as an error result.
    let seen = rig.seen.lock().unwrap();
    let refusal = seen[2]
        .messages
        .iter()
        .flat_map(|m| m.content.iter())
        .find_map(|b| match b {
            ContentBlock::ToolResult {
                content,
                is_error: Some(true),
                ..
            } => content.first().and_then(|c| c.as_text().map(String::from)),
            _ => None,
        })
        .expect("the child was told no");
    assert!(refusal.contains("may not delegate further"), "{refusal}");
}

#[tokio::test]
async fn depth_two_allows_a_grandchild() {
    let rig = rig(
        json!({ "maxDepth": 2 }),
        vec![
            Reply::Tool("subagent", json!({ "task": "outer" })),
            Reply::Tool("subagent", json!({ "task": "inner" })),
            Reply::Text("grandchild says hi"),
            Reply::Text("child relays: grandchild says hi"),
            Reply::Text("done"),
        ],
    )
    .await;
    ask(&rig, "go");
    rig.parent.when_idle().await;
    assert_eq!(
        tool_results(rig.parent.as_ref()),
        vec![("child relays: grandchild says hi".to_string(), false)]
    );
    assert!(children(&rig).is_empty());
}

#[tokio::test]
async fn a_child_that_never_finishes_is_stopped_and_its_slot_is_freed() {
    let rig = rig(
        json!({ "timeoutMs": 300, "maxConcurrent": 1 }),
        vec![
            Reply::Tool("subagent", json!({ "task": "hangs" })),
            Reply::Hang, // the child's model call never returns
            Reply::Tool("subagent", json!({ "task": "second try" })), // parent after the failure
            Reply::Text("fine"), // the second child
            Reply::Text("all done"), // the parent
        ],
    )
    .await;
    ask(&rig, "go");
    tokio::time::timeout(Duration::from_secs(20), rig.parent.when_idle())
        .await
        .expect("parent finishes");

    let results = tool_results(rig.parent.as_ref());
    assert_eq!(results.len(), 2, "{results:?}");
    assert!(
        results[0].1 && results[0].0.contains("did not finish within 300 ms"),
        "{results:?}"
    );
    assert_eq!(
        results[1],
        ("fine".to_string(), false),
        "the single slot was freed for the retry"
    );
    assert!(children(&rig).is_empty());
}

#[tokio::test]
async fn timeout_returns_only_logged_public_text_marked_incomplete_and_frees_the_slot() {
    let rig = rig(
        json!({ "timeoutMs": 80, "maxResultChars": 200, "maxConcurrent": 1 }),
        vec![
            Reply::Tool("subagent", json!({ "task": "investigate" })),
            Reply::PartialHang,
            Reply::Tool("subagent", json!({ "task": "try again" })),
            Reply::Text("second child finished"),
            Reply::Text("done"),
        ],
    )
    .await;
    ask(&rig, "go");
    tokio::time::timeout(Duration::from_secs(5), rig.parent.when_idle())
        .await
        .unwrap();
    let results = tool_results(rig.parent.as_ref());
    assert!(
        !results[0].1,
        "a usable partial answer is not an empty error: {results:?}"
    );
    assert!(
        results[0].0.contains("src/index.rs"),
        "logged stream prefix returned: {results:?}"
    );
    assert!(results[0].0.contains("timed out") && results[0].0.contains("incomplete"));
    assert!(!results[0].0.contains("private reasoning"));
    assert!(results[0].0.chars().count() <= 200);
    assert_eq!(results[1], ("second child finished".into(), false));
    assert!(children(&rig).is_empty());
}

#[tokio::test]
async fn a_failing_child_reports_failure_instead_of_an_empty_answer() {
    let rig = rig(
        json!({}),
        vec![
            Reply::Tool("subagent", json!({ "task": "x" })),
            Reply::Fail,
            Reply::Text("ok"),
        ],
    )
    .await;
    ask(&rig, "go");
    rig.parent.when_idle().await;
    let results = tool_results(rig.parent.as_ref());
    assert!(results[0].1, "{results:?}");
    assert!(
        results[0].0.to_lowercase().contains("sub-agent"),
        "{results:?}"
    );
    assert!(children(&rig).is_empty());
}

#[tokio::test]
async fn cancelling_the_parent_stops_and_removes_the_child() {
    let rig = rig(
        json!({}),
        vec![
            Reply::Tool("subagent", json!({ "task": "long job" })),
            Reply::Hang,
        ],
    )
    .await;
    ask(&rig, "go");
    // Wait until the child exists, then cancel the parent.
    for _ in 0..200 {
        if !children(&rig).is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert_eq!(children(&rig).len(), 1, "the child started");
    rig.parent.cancel(AgentCancelCause::User, false);
    tokio::time::timeout(Duration::from_secs(10), rig.parent.when_idle())
        .await
        .expect("parent settles");
    for _ in 0..100 {
        if children(&rig).is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(
        children(&rig).is_empty(),
        "the child is gone after the parent was cancelled"
    );
}

#[tokio::test]
async fn bad_arguments_and_bad_settings_are_refused() {
    let rig = rig(
        json!({}),
        vec![
            Reply::Tool("subagent", json!({ "task": "   " })),
            Reply::Text("ok"),
        ],
    )
    .await;
    ask(&rig, "go");
    rig.parent.when_idle().await;
    let results = tool_results(rig.parent.as_ref());
    assert!(
        results[0].1 && results[0].0.contains("task is required"),
        "{results:?}"
    );

    let ctx = Context::new();
    let err = install_profile(
        &ctx,
        &json!({ "plugins": { "subagent": { "maxDepth": 0 } } }),
    )
    .await
    .map(|_| ())
    .unwrap_err();
    assert!(err.contains("maxDepth"), "{err}");
}

#[tokio::test]
async fn a_child_can_ask_the_main_agent_and_the_exchange_is_logged() {
    let rig = rig(
        json!({}),
        vec![
            Reply::Tool(
                "subagent",
                json!({ "task": "pick a port for the dev server" }),
            ),
            Reply::Tool(
                "ask_parent",
                json!({ "question": "which port should the dev server use?" }),
            ),
            Reply::Text("use port 8080"), // the main agent answering the child
            Reply::Text("the dev server uses port 8080"), // the child's answer
            Reply::Text("Done."),         // the parent, after the tool result
        ],
    )
    .await;
    ask(&rig, "please set up the dev server");
    rig.parent.when_idle().await;

    // The child's answer reached the parent as the tool result.
    let results = tool_results(rig.parent.as_ref());
    assert_eq!(results.len(), 1);
    assert!(
        results[0].0.contains("8080"),
        "the child used the main agent's answer: {results:?}"
    );
    assert!(!results[0].1, "not an error");

    let seen = rig.seen.lock().unwrap();
    // The question was answered by the parent's model, on the parent's history.
    let ask_request = &seen[2];
    assert_eq!(ask_request.model, "m");
    let system = ask_request.system.clone().unwrap_or_default();
    assert!(
        system.contains("main agent"),
        "the answerer is told what it is: {system}"
    );
    let ask_texts: Vec<String> = ask_request.messages.iter().map(|m| m.text()).collect();
    assert!(
        ask_texts
            .iter()
            .any(|t| t.contains("please set up the dev server")),
        "the parent's own conversation is the context: {ask_texts:?}"
    );
    assert!(ask_texts.iter().any(|t| t.contains("which port")));
    drop(seen);

    // Model-visible means logged: the parent can read what it was asked.
    let logged: Vec<String> = rig
        .parent
        .session()
        .events()
        .into_iter()
        .filter_map(|e| match e.data {
            SessionEventData::UserMessage { message } => {
                matches!(message.source, dsh_rs::types::MessageSource::Plugin { .. })
                    .then(|| message.text())
            }
            _ => None,
        })
        .collect();
    assert!(
        logged.iter().any(
            |t| t.contains("asked: which port should the dev server use?")
                && t.contains("Answer given from this conversation: use port 8080")
        ),
        "the exchange is in the parent log: {logged:?}"
    );
}

#[tokio::test]
async fn the_main_agent_is_told_not_to_use_ask_parent() {
    let rig = rig(
        json!({}),
        vec![Reply::Tool(
            "ask_parent",
            json!({ "question": "what now?" }),
        )],
    )
    .await;
    ask(&rig, "decide yourself");
    rig.parent.when_idle().await;

    let results = tool_results(rig.parent.as_ref());
    assert_eq!(results.len(), 1);
    assert!(results[0].1, "it is an error for the root agent");
    assert!(
        results[0].0.contains("ask_parent is for sub-agents"),
        "the main agent gets an actionable refusal: {}",
        results[0].0
    );
}

#[tokio::test]
async fn a_child_is_refused_when_it_tries_to_ask_the_user() {
    let rig = rig(
        json!({}),
        vec![
            Reply::Tool("subagent", json!({ "task": "decide" })),
            Reply::Tool(
                "ask_user_question",
                json!({ "questions": [{ "id": "q", "question": "Which?" }] }),
            ),
            Reply::Text("I will report the question instead"), // the child, after the refusal
            Reply::Text("done"),                               // the parent
        ],
    )
    .await;
    ask(&rig, "go");
    rig.parent.when_idle().await;
    let seen = rig.seen.lock().unwrap();
    let refusal = seen[2]
        .messages
        .iter()
        .flat_map(|m| m.content.iter())
        .find_map(|b| match b {
            ContentBlock::ToolResult {
                content,
                is_error: Some(true),
                ..
            } => content.first().and_then(|c| c.as_text().map(String::from)),
            _ => None,
        })
        .expect("the child was told no");
    assert!(refusal.contains("owned by another live agent"), "{refusal}");
}
