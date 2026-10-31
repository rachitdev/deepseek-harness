//! `subagents/prompt` and `subagents/interruptByParent` through the gateway,
//! against a real parent whose `subagent` tool call is running a child.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView, BoxFuture, LlmAdapterApi, LlmService};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::types::{
    stream_from_chunks, AgentOptions, BoxStream, ContentBlock, GenerateOptions, LlmError, Message,
    SessionEventData, StreamChunk,
};
use serde_json::{json, Value};

mod common;
use common::{text_response, tool_call_response};

const TOKEN: &str = "subagents-token-0123456789";

#[derive(Clone)]
enum Reply {
    Text(&'static str),
    Tool(&'static str, Value),
    Hang,
}

struct Script(Mutex<VecDeque<Reply>>);

impl LlmAdapterApi for Script {
    fn name(&self) -> &'static str {
        "script"
    }
    fn stream(&self, _options: GenerateOptions) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>> {
        let reply = self.0.lock().unwrap().pop_front().unwrap_or(Reply::Text("(script exhausted)"));
        Box::pin(async move {
            match reply {
                Reply::Text(text) => Ok(stream_from_chunks(text_response(text))),
                Reply::Tool(name, args) => Ok(stream_from_chunks(tool_call_response("call-1", name, args))),
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
    gateway: Arc<Gateway>,
    agents: AgentRegistryService,
}

async fn rig(replies: Vec<Reply>) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": { "subagent": { "timeoutMs": 60000 }, "ask-user": {} } }))
        .await
        .expect("profile installs");
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(&["mock"], Arc::new(Script(Mutex::new(replies.into())))).unwrap();
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel { provider: "mock".into(), model: "m".into() },
        max_page_events: 100,
        max_search_results: 20,
        max_title_bytes: 120,
        max_bytes: 2 * 1024 * 1024,
        max_file_bytes: 32 * 1024 * 1024,
        max_lines: 5_000,
        max_entries: 2_000,
        default_max_goal_rounds: 256,
        export_compression_level: 6,
        models: Vec::new(),
    });
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.to_string() }, None));
    let agents = (*ctx.require::<AgentRegistryService>("agents").unwrap()).clone();
    Rig { ctx, gateway, agents }
}

fn root(rig: &Rig, id: &str) -> Arc<dyn AgentView> {
    rig.agents.create(Some(id.into()), AgentOptions::mock("m"), Some("/tmp".into()), None, None).unwrap()
}

/// Start a turn on `main` that runs a child whose model call never returns, and
/// wait until the child is live. Returns the parent and the child's id.
async fn parent_with_live_child(rig: &Rig) -> (Arc<dyn AgentView>, String) {
    let parent = root(rig, "main");
    parent.followup(Message::user("u1", vec![ContentBlock::text("go")]));
    for _ in 0..300 {
        if let Some(child) = rig.agents.list().iter().map(|a| a.id().to_string()).find(|id| id != "main") {
            return (parent, child);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("the child never started");
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method, "payload": { "args": args },
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    let (reply, _) = rig.gateway.route(&rig.ctx, &http::Method::POST, "/api", &headers, &body).await;
    serde_json::from_slice::<Value>(&reply.body).expect("JSON reply")["result"].clone()
}

fn prompt_request(parent: &str, child: &str, delivery: &str, content: Value) -> Value {
    // The request identity follows the delivery, so two prompts are never a retry.
    json!({ "request": {
        "requestId": format!("req-{delivery}"), "parentSessionId": parent, "childSessionId": child,
        "mode": "continuable", "delivery": delivery, "content": content,
    } })
}

fn text(t: &str) -> Value {
    json!([{ "type": "text", "text": t }])
}

fn code(reply: &Value) -> &str {
    reply["error"]["code"].as_str().unwrap_or("(none)")
}

#[tokio::test]
async fn a_prompt_for_a_parent_that_is_not_live_is_refused() {
    let rig = rig(vec![]).await;
    let reply = call(&rig, "subagents/prompt", prompt_request("p1", "c1", "queue", text("hi"))).await;
    assert_eq!(code(&reply), "subagent/parent-unavailable", "{reply}");
    assert_eq!(reply["error"]["details"], json!({ "parentSessionId": "p1" }));
    assert_eq!(reply["error"]["message"], "parent session \"p1\" is not live");
}

#[tokio::test]
async fn ids_are_checked_before_anything_else_and_every_empty_one_is_reported() {
    let rig = rig(vec![]).await;
    let reply = call(&rig, "subagents/prompt", prompt_request("", "", "queue", text("hi"))).await;
    assert_eq!(code(&reply), "gateway/bad-request");
    assert_eq!(reply["error"]["message"], "invalid payload for subagent.prompt");
    let issues = reply["error"]["details"]["issues"].as_array().unwrap();
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0]["path"], json!(["parentSessionId"]));
    assert_eq!(issues[1]["path"], json!(["childSessionId"]));
    assert_eq!(issues[0]["code"], "too_small");
    assert_eq!(issues[0]["minimum"], 1);

    let interrupt = call(&rig, "subagents/interruptByParent", json!({
        "childSessionId": "", "parentSessionId": "p", "mode": "continuable",
    }))
    .await;
    assert_eq!(code(&interrupt), "gateway/bad-request");
    assert_eq!(interrupt["error"]["message"], "invalid payload for subagent.interrupt");
    assert_eq!(interrupt["error"]["details"]["issues"][0]["path"], json!(["childSessionId"]));
}

#[tokio::test]
async fn a_bad_client_time_zone_is_refused_before_the_parent_is_looked_up() {
    let rig = rig(vec![]).await;
    for zone in ["Nope/Zone", "", " UTC", "utc", "Europe"] {
        let mut request = prompt_request("p1", "c1", "queue", text("hi"));
        request["request"]["clientTimeZone"] = json!(zone);
        let reply = call(&rig, "subagents/prompt", request).await;
        assert_eq!(code(&reply), "subagent/invalid-time-zone", "{zone:?}: {reply}");
        assert_eq!(reply["error"]["details"], json!({ "value": zone }));
    }
    for zone in ["UTC", "Europe/Paris", "europe/paris"] {
        let mut request = prompt_request("p1", "c1", "queue", text("hi"));
        request["request"]["clientTimeZone"] = json!(zone);
        let reply = call(&rig, "subagents/prompt", request).await;
        assert_eq!(code(&reply), "subagent/parent-unavailable", "{zone:?}: {reply}");
    }
}

#[tokio::test]
async fn a_session_that_is_not_a_child_is_unauthorized_and_a_missing_one_is_not_resumable() {
    let rig = rig(vec![]).await;
    root(&rig, "main");
    root(&rig, "other");
    let ghost = call(&rig, "subagents/prompt", prompt_request("main", "ghost", "queue", text("hi"))).await;
    assert_eq!(code(&ghost), "subagent/not-resumable", "{ghost}");
    assert_eq!(ghost["error"]["details"], json!({ "childSessionId": "ghost" }));
    let stranger = call(&rig, "subagents/prompt", prompt_request("main", "other", "queue", text("hi"))).await;
    assert_eq!(code(&stranger), "subagent/unauthorized", "{stranger}");
    let itself = call(&rig, "subagents/prompt", prompt_request("main", "main", "queue", text("hi"))).await;
    assert_eq!(code(&itself), "subagent/unauthorized", "{itself}");
}

#[tokio::test]
async fn interrupting_a_child_that_is_not_there_is_accepted() {
    let rig = rig(vec![]).await;
    root(&rig, "main");
    for (child, parent) in [("ghost", "main"), ("main", "main"), ("ghost", "nobody")] {
        let reply = call(&rig, "subagents/interruptByParent", json!({
            "childSessionId": child, "parentSessionId": parent, "mode": "continuable",
        }))
        .await;
        assert_eq!(reply, json!({ "ok": true, "value": { "accepted": true } }), "{child} {parent}");
    }
}

#[tokio::test]
async fn a_live_child_takes_a_queued_prompt_and_a_steer() {
    let rig = rig(vec![Reply::Tool("subagent", json!({ "task": "work" })), Reply::Hang]).await;
    let (parent, child) = parent_with_live_child(&rig).await;

    let queued = call(&rig, "subagents/prompt", prompt_request("main", &child, "queue", text("later"))).await;
    assert_eq!(queued["ok"], true, "{queued}");
    let queued_id = queued["value"]["messageId"].as_str().expect("the accepted message's id").to_string();

    let steered = call(&rig, "subagents/prompt", prompt_request("main", &child, "steer", text("now"))).await;
    assert_eq!(steered["ok"], true, "{steered}");
    let steered_id = steered["value"]["messageId"].as_str().unwrap().to_string();
    assert_ne!(queued_id, steered_id);

    let events = rig.agents.get(&child).expect("the child is live").session().events();
    let spliced = |id: &str| {
        events.iter().filter_map(|e| match &e.data {
            SessionEventData::InboxSpliced { target, inserted, .. } => Some((*target, inserted.clone())),
            _ => None,
        })
        .find(|(_, inserted)| inserted.iter().any(|m| m.id == id))
        .map(|(target, inserted)| (target, inserted.into_iter().find(|m| m.id == id).unwrap()))
    };
    let (target, message) = spliced(&queued_id).expect("the queued message reached the child's inbox");
    assert_eq!(format!("{target:?}"), "NextTurn");
    assert_eq!(message.source, dsh_rs::types::MessageSource::User { rpc_id: Some("req-queue".into()) });
    let (target, _) = spliced(&steered_id).expect("the steered message reached the child's inbox");
    assert_eq!(format!("{target:?}"), "NextStep");
    rig.agents.get(&child).unwrap().cancel(dsh_rs::types::AgentCancelCause::Parent, false);
    let _ = tokio::time::timeout(Duration::from_secs(5), parent.when_idle()).await;
}

#[tokio::test]
async fn another_parent_cannot_prompt_or_interrupt_a_live_child() {
    let rig = rig(vec![Reply::Tool("subagent", json!({ "task": "work" })), Reply::Hang]).await;
    let (parent, child) = parent_with_live_child(&rig).await;
    root(&rig, "intruder");

    let prompt = call(&rig, "subagents/prompt", prompt_request("intruder", &child, "queue", text("hi"))).await;
    assert_eq!(code(&prompt), "subagent/unauthorized", "{prompt}");
    assert_eq!(prompt["error"]["details"], json!({ "childSessionId": child }));
    let interrupt = call(&rig, "subagents/interruptByParent", json!({
        "childSessionId": child, "parentSessionId": "intruder", "mode": "continuable",
    }))
    .await;
    assert_eq!(code(&interrupt), "subagent/unauthorized", "{interrupt}");
    assert!(rig.agents.get(&child).is_some(), "the refusal left the child running");
    rig.agents.get(&child).unwrap().cancel(dsh_rs::types::AgentCancelCause::Parent, false);
    let _ = tokio::time::timeout(Duration::from_secs(5), parent.when_idle()).await;
}

#[tokio::test]
async fn the_parent_can_interrupt_its_child_and_the_tool_call_then_returns() {
    let rig = rig(vec![
        Reply::Tool("subagent", json!({ "task": "work" })),
        Reply::Hang,
        Reply::Text("the parent carries on"),
    ])
    .await;
    let (parent, child) = parent_with_live_child(&rig).await;
    let reply = call(&rig, "subagents/interruptByParent", json!({
        "childSessionId": child, "parentSessionId": "main", "mode": "continuable",
    }))
    .await;
    assert_eq!(reply, json!({ "ok": true, "value": { "accepted": true } }));
    tokio::time::timeout(Duration::from_secs(10), parent.when_idle())
        .await
        .expect("the parent's turn finishes once the child is stopped");
    let finished = parent.session().events().iter().any(|e| {
        matches!(&e.data, SessionEventData::AssistantMessage { message, .. } if message.text().contains("carries on"))
    });
    assert!(finished, "the parent continued after its child was interrupted");
}

#[tokio::test]
async fn an_image_for_a_child_whose_model_takes_none_is_refused_as_an_attachment_problem() {
    let rig = rig(vec![Reply::Tool("subagent", json!({ "task": "work" })), Reply::Hang]).await;
    let (parent, child) = parent_with_live_child(&rig).await;
    let image = json!([{ "type": "image", "mediaType": "image/png", "data": "AA==" }]);
    let reply = call(&rig, "subagents/prompt", prompt_request("main", &child, "queue", image)).await;
    assert_eq!(code(&reply), "subagent/attachment-invalid", "{reply}");
    assert_eq!(reply["error"]["details"]["reason"], "MODEL_DOES_NOT_SUPPORT_IMAGES");
    rig.agents.get(&child).unwrap().cancel(dsh_rs::types::AgentCancelCause::Parent, false);
    let _ = tokio::time::timeout(Duration::from_secs(5), parent.when_idle()).await;
}

#[tokio::test]
async fn wire_types_fail_the_gateway_before_any_rule() {
    let rig = rig(vec![]).await;
    for (method, args, field) in [
        ("subagents/interruptByParent", json!({ "childSessionId": "c", "parentSessionId": "p", "mode": "one-shot" }), "mode"),
        ("subagents/interruptByParent", json!({ "childSessionId": 5, "parentSessionId": "p", "mode": "continuable" }), "childSessionId"),
        ("subagents/prompt", json!({ "request": { "requestId": "r", "parentSessionId": "p", "childSessionId": "c", "mode": "continuable", "delivery": "weird", "content": [] } }), "request"),
        ("subagents/prompt", json!({ "request": { "requestId": "r", "parentSessionId": "p", "childSessionId": "c", "mode": "continuable", "delivery": "queue" } }), "request"),
        ("subagents/prompt", json!({ "request": { "requestId": "r", "parentSessionId": "p", "childSessionId": "c", "mode": "one-shot", "delivery": "queue", "content": [] } }), "request"),
    ] {
        let reply = call(&rig, method, args.clone()).await;
        assert_eq!(code(&reply), "gateway/input-invalid", "{method} {args}: {reply}");
        assert_eq!(reply["error"]["details"]["field"], field);
    }
    let missing = call(&rig, "subagents/interruptByParent", json!({ "childSessionId": "c", "parentSessionId": "p" })).await;
    assert_eq!(code(&missing), "gateway/arguments-invalid");
}
