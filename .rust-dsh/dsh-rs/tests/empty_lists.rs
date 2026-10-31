//! The reminder endpoints' argument checks on a host holding no reminders, and
//! `job/kill`, which has no job to stop, through the gateway.

use std::sync::Arc;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::types::AgentOptions;
use serde_json::{json, Value};

const TOKEN: &str = "lists-token-0123456789";

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

fn with(mut payload: Value, key: &str, value: Value) -> Value {
    payload[key] = value;
    payload
}

#[tokio::test]
async fn each_list_is_empty_for_the_arguments_the_typescript_host_accepts() {
    let rig = rig().await;
    let id = rig.agent.id().to_string();
    for (endpoint, args) in [
        ("schedule/list", json!({ "request": { "sessionId": id } })),
        // An unknown session has no reminders, and extra request fields are ignored.
        (
            "schedule/list",
            json!({ "request": { "sessionId": "ghost" } }),
        ),
        (
            "schedule/list",
            json!({ "request": { "sessionId": "", "x": 1 } }),
        ),
        ("schedule/catalog", json!({})),
    ] {
        let reply = call(&rig, endpoint, args.clone()).await;
        assert_eq!(
            reply,
            json!({ "ok": true, "value": [] }),
            "{endpoint} {args}"
        );
    }
}

#[tokio::test]
async fn wrong_argument_names_are_a_descriptor_failure_like_typescript() {
    let rig = rig().await;
    for (endpoint, args, needle) in [
        (
            "schedule/catalog",
            json!({ "agentId": rig.agent.id() }),
            "unexpected \"agentId\"",
        ),
        ("schedule/list", json!({}), "missing \"request\""),
    ] {
        let reply = call(&rig, endpoint, args.clone()).await;
        assert_eq!(
            reply["error"]["code"], "gateway/arguments-invalid",
            "{endpoint} {args}: {reply}"
        );
        let message = reply["error"]["message"].as_str().unwrap();
        assert!(message.contains(needle), "{message}");
    }
}

#[tokio::test]
async fn a_reminder_request_without_a_string_session_fails_at_the_gateway() {
    let rig = rig().await;
    for request in [json!({}), json!({ "sessionId": 5 }), json!(null), json!([])] {
        let reply = call(
            &rig,
            "schedule/list",
            with(json!({}), "request", request.clone()),
        )
        .await;
        assert_eq!(
            reply["error"]["code"], "gateway/input-invalid",
            "{request}: {reply}"
        );
        assert_eq!(reply["error"]["details"]["field"], "request");
    }
}

#[tokio::test]
async fn every_job_is_unknown_and_the_message_keeps_the_typescript_prefix() {
    let rig = rig().await;
    for (session, job) in [("s", "j"), ("", ""), ("ghost", "job-1")] {
        let reply = call(
            &rig,
            "job/kill",
            json!({ "request": { "sessionId": session, "jobId": job, "x": 1 } }),
        )
        .await;
        assert_eq!(reply["error"]["code"], "job/not-found", "{reply}");
        assert_eq!(
            reply["error"]["message"],
            format!("Error: unknown job {job}")
        );
        assert_eq!(
            reply["error"]["details"],
            json!({ "sessionId": session, "jobId": job })
        );
    }
}

#[tokio::test]
async fn a_malformed_kill_request_fails_at_the_gateway_before_the_job_lookup() {
    let rig = rig().await;
    for request in [
        json!({ "sessionId": "s" }),
        json!({ "jobId": "j" }),
        json!({ "sessionId": 5, "jobId": "j" }),
        json!({ "sessionId": "s", "jobId": 5 }),
        json!(null),
    ] {
        let reply = call(&rig, "job/kill", json!({ "request": request })).await;
        assert_eq!(
            reply["error"]["code"], "gateway/input-invalid",
            "{request}: {reply}"
        );
        assert_eq!(reply["error"]["details"]["field"], "request");
    }
    let missing = call(&rig, "job/kill", json!({})).await;
    assert_eq!(missing["error"]["code"], "gateway/arguments-invalid");
    let extra = call(
        &rig,
        "job/kill",
        json!({ "request": { "sessionId": "s", "jobId": "j" }, "y": 1 }),
    )
    .await;
    assert_eq!(extra["error"]["code"], "gateway/arguments-invalid");
}
