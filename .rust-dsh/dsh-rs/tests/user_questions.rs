//! `userQuestions/answer`, through the gateway.

use std::sync::Arc;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::llm::{ContentBlock, Message, MessageSource, Role};
use dsh_rs::types::{AgentOptions, SessionEventData};
use serde_json::{json, Value};

const TOKEN: &str = "questions-token-0123456789";

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    agent: Arc<dyn AgentView>,
}

async fn rig() -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .expect("profile installs");
    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("m"), None, None, None)
        .unwrap();
    let session = agent.session();
    session.append(SessionEventData::TurnStart { turn: 1 });
    session.append(SessionEventData::UserMessage {
        message: Message::user("u-1", vec![ContentBlock::text("hello")]),
    });
    session.append(SessionEventData::AssistantMessage {
        turn: 1,
        step: 1,
        message: Message {
            id: "a-1".into(),
            role: Role::Assistant,
            content: vec![ContentBlock::text("hi")],
            source: MessageSource::Model {
                provider: "mock".into(),
                model: "m".into(),
            },
        },
        usage: None,
        interrupted: None,
    });
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel {
            provider: "mock".to_string(),
            model: "m".to_string(),
        },
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
    let gateway = Arc::new(Gateway::new(
        router,
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    ));
    Rig {
        ctx,
        gateway,
        agent,
    }
}

async fn call(rig: &Rig, method: &str, payload: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": payload },
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    let envelope: Value = serde_json::from_slice(&reply.body).expect("the reply is JSON");
    envelope["result"].clone()
}

fn body(rig: &Rig) -> Value {
    json!({
        "agentId": rig.agent.id(), "callId": "call-1",
        "answer": { "answers": [{ "id": "q", "selected": ["a"] }] },
    })
}

fn with(mut payload: Value, key: &str, value: Value) -> Value {
    payload[key] = value;
    payload
}

#[tokio::test]
async fn a_live_session_with_no_continued_question_answers_false_and_logs_nothing() {
    let rig = rig().await;
    let before = rig.agent.session().events().len();
    let reply = call(&rig, "userQuestions/answer", body(&rig)).await;
    assert_eq!(reply, json!({ "ok": true, "value": false }));
    assert_eq!(rig.agent.session().events().len(), before);
}

#[tokio::test]
async fn an_unknown_session_is_session_not_found_even_for_an_empty_batch() {
    let rig = rig().await;
    let reply = call(
        &rig,
        "userQuestions/answer",
        with(body(&rig), "agentId", json!("ghost")),
    )
    .await;
    assert_eq!(reply["error"]["code"], "session/not-found", "{reply}");
    assert_eq!(reply["error"]["details"], json!({ "sessionId": "ghost" }));
}

#[tokio::test]
async fn an_empty_agent_id_fails_like_the_typescript_host() {
    let rig = rig().await;
    let reply = call(
        &rig,
        "userQuestions/answer",
        with(body(&rig), "agentId", json!("")),
    )
    .await;
    assert_eq!(reply["error"]["code"], "gateway/internal", "{reply}");
    assert_eq!(reply["error"]["details"], json!({}));
}

#[tokio::test]
async fn fields_are_checked_at_the_gateway_in_the_typescript_order() {
    let rig = rig().await;
    let ghost = with(body(&rig), "agentId", json!("ghost"));
    // The gateway checks agentId, then callId, then answer; all before any lookup.
    for (payload, field) in [
        (with(ghost.clone(), "agentId", json!(5)), "agentId"),
        (with(ghost.clone(), "callId", json!(5)), "callId"),
        (with(ghost.clone(), "answer", json!(null)), "answer"),
        (with(ghost.clone(), "answer", json!({})), "answer"),
        (
            with(
                ghost.clone(),
                "answer",
                json!({ "answers": [{ "id": 1, "selected": [] }] }),
            ),
            "answer",
        ),
        (
            with(
                ghost.clone(),
                "answer",
                json!({ "answers": [{ "id": "q", "selected": [1] }] }),
            ),
            "answer",
        ),
        (
            with(
                ghost.clone(),
                "answer",
                json!({ "answers": [{ "id": "q", "selected": [], "custom": 5 }] }),
            ),
            "answer",
        ),
    ] {
        let reply = call(&rig, "userQuestions/answer", payload.clone()).await;
        assert_eq!(
            reply["error"]["code"], "gateway/input-invalid",
            "{payload}: {reply}"
        );
        assert_eq!(
            reply["error"]["details"]["field"], field,
            "{payload}: {reply}"
        );
    }
}

#[tokio::test]
async fn the_wrong_argument_names_are_a_descriptor_failure() {
    let rig = rig().await;
    let bare = json!({ "agent": rig.agent.id(), "callId": "c", "answer": { "answers": [] } });
    let reply = call(&rig, "userQuestions/answer", bare).await;
    assert_eq!(
        reply["error"]["code"], "gateway/arguments-invalid",
        "{reply}"
    );
    let message = reply["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("missing \"agentId\"") && message.contains("unexpected \"agent\""),
        "{message}"
    );
    let missing = json!({ "agentId": rig.agent.id(), "callId": "c" });
    let reply = call(&rig, "userQuestions/answer", missing).await;
    assert_eq!(
        reply["error"]["code"], "gateway/arguments-invalid",
        "{reply}"
    );
}

#[tokio::test]
async fn extra_fields_and_duplicate_ids_pass_the_gateway_like_typescript() {
    let rig = rig().await;
    let payload = with(
        body(&rig),
        "answer",
        json!({ "answers": [{ "id": "q", "selected": [], "z": 1 }, { "id": "q", "selected": [] }], "x": 1 }),
    );
    let reply = call(&rig, "userQuestions/answer", payload).await;
    assert_eq!(reply, json!({ "ok": true, "value": false }));
}
