//! The terminal controller's unary endpoints, through the gateway.

use std::sync::Arc;

use cordis::Context;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::terminal::service::{TerminalConfig, TerminalService};
use serde_json::{json, Value};

const TOKEN: &str = "test-token-0123456789abcdef";

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
}

async fn rig() -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} })).await.unwrap();
    let terminal = TerminalService::new(ctx.clone(), TerminalConfig::default());
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
    })
    .with_terminal(terminal);
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.into() }, None));
    Rig { ctx, gateway }
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    let body = serde_json::to_vec(&json!({ "type": "client-request", "rpcId": "r", "method": method, "payload": { "args": args } })).unwrap();
    let (reply, _) = rig.gateway.route(&rig.ctx, &http::Method::POST, "/api", &headers, &body).await;
    assert_eq!(reply.status, 200, "{}", String::from_utf8_lossy(&reply.body));
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}

async fn session(rig: &Rig) -> String {
    call(rig, "session/create", json!({ "request": {} })).await["value"]["sessionId"].as_str().unwrap().to_string()
}

/// The failure of a call: its code and message.
fn failure(reply: &Value) -> (String, String) {
    (
        reply["error"]["code"].as_str().unwrap_or_default().to_string(),
        reply["error"]["message"].as_str().unwrap_or_default().to_string(),
    )
}

#[tokio::test]
async fn the_shells_are_the_preferred_one_first_then_the_installed_candidates() {
    let rig = rig().await;
    let one = session(&rig).await;
    let reply = call(&rig, "terminal/shells", json!({ "agentId": one })).await;
    let shells = reply["value"].as_array().expect("a list of shells");
    assert!(!shells.is_empty());
    let first = &shells[0];
    assert_eq!(first["name"], std::path::Path::new(&first["path"].as_str().unwrap()).file_name().unwrap().to_str().unwrap());
    assert_eq!(first["args"], json!(["-i"]));
    // The candidates are deduplicated by shell kind.
    let mut kinds: Vec<String> = shells
        .iter()
        .map(|shell| shell["name"].as_str().unwrap().to_lowercase())
        .collect();
        let count = kinds.len();
        kinds.sort();
        kinds.dedup();
    assert_eq!(kinds.len(), count, "two shells with the same kind");
}

#[tokio::test]
async fn the_environment_names_the_working_directory_and_the_limits() {
    let rig = rig().await;
    let one = session(&rig).await;
    let reply = call(&rig, "terminal/environment", json!({ "agentId": one })).await;
    let environment = &reply["value"];
    assert_eq!(environment["cwd"], std::env::current_dir().unwrap().to_string_lossy().as_ref());
    assert_eq!(environment["maxInputBytes"], 65_536);
    assert_eq!(environment["maxBufferedBytes"], 2_097_152);
    assert_eq!(environment["maxCols"], 500);
    assert_eq!(environment["maxRows"], 200);
    assert_eq!(environment["scrollback"], 1000);
}

#[tokio::test]
async fn an_unknown_session_is_refused_and_an_unknown_session_has_no_terminals() {
    let rig = rig().await;
    for method in ["terminal/shells", "terminal/environment"] {
        let reply = call(&rig, method, json!({ "agentId": "ghost" })).await;
        assert_eq!(failure(&reply).0, "session/not-found", "{method}");
    }
    let reply = call(&rig, "terminal/list", json!({ "sessionId": "ghost" })).await;
    assert_eq!(reply["value"], json!([]));
}

#[tokio::test]
async fn a_created_terminal_is_named_after_its_shell_and_answered_by_the_list() {
    let rig = rig().await;
    let one = session(&rig).await;
    let created = call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "t1", "cols": 80, "rows": 24 } })).await;
    let info = &created["value"];
    assert_eq!(info["id"], "t1");
    assert_eq!(info["cols"], 80);
    assert_eq!(info["rows"], 24);
    assert_eq!(info["state"], "running");
    assert_eq!(info["exitCode"], Value::Null);
    assert_eq!(info["title"], info["shell"]["name"], "the title starts as the shell's name");
    assert_eq!(info["cwd"], std::env::current_dir().unwrap().to_string_lossy().as_ref());

    let listed = call(&rig, "terminal/list", json!({ "sessionId": one })).await;
    assert_eq!(listed["value"].as_array().unwrap().len(), 1);
    assert_eq!(listed["value"][0]["id"], "t1");

    // Creating the same id again answers the terminal that is there.
    let again = call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "t1", "cols": 80, "rows": 24 } })).await;
    assert_eq!(again["value"], created["value"]);
    assert_eq!(call(&rig, "terminal/list", json!({ "sessionId": one })).await["value"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_refused_creation_names_what_is_wrong() {
    let rig = rig().await;
    let one = session(&rig).await;
    for (request, message) in [
        (json!({ "id": "bad id!", "cols": 80, "rows": 24 }), "Invalid terminal identity"),
        (json!({ "id": "", "cols": 80, "rows": 24 }), "Invalid terminal identity"),
        (json!({ "id": "x".repeat(129), "cols": 80, "rows": 24 }), "Invalid terminal identity"),
        (json!({ "id": "t2", "cols": 1, "rows": 24 }), "Terminal dimensions exceed the configured limits"),
        (json!({ "id": "t2", "cols": 80, "rows": 0 }), "Terminal dimensions exceed the configured limits"),
        (json!({ "id": "t2", "cols": 501, "rows": 24 }), "Terminal dimensions exceed the configured limits"),
        (json!({ "id": "t2", "cols": 80, "rows": 201 }), "Terminal dimensions exceed the configured limits"),
        (json!({ "id": "t2", "cols": 80.5, "rows": 24 }), "Terminal dimensions exceed the configured limits"),
        (json!({ "id": "t2", "cols": 80, "rows": 24, "shellPath": "/bin/nope" }), "Selected shell is not available in this execution environment"),
    ] {
        let reply = call(&rig, "terminal/create", json!({ "agentId": one, "request": request })).await;
        let (code, text) = failure(&reply);
        assert_eq!(code, "gateway/internal", "{request}");
        assert_eq!(text, message, "{request}");
    }
    assert_eq!(call(&rig, "terminal/list", json!({ "sessionId": one })).await["value"], json!([]), "no refusal created a terminal");
}

#[tokio::test]
async fn a_session_holds_at_most_eight_terminals() {
    let rig = rig().await;
    let one = session(&rig).await;
    for index in 0..8 {
        let reply = call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": format!("m{index}"), "cols": 80, "rows": 24 } })).await;
        assert!(reply["error"].is_null(), "{index}: {reply}");
    }
    let refused = call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "m8", "cols": 80, "rows": 24 } })).await;
    assert_eq!(failure(&refused).0, "terminal/limit-reached");
    assert_eq!(refused["error"]["details"]["limit"], 8);
    assert_eq!(call(&rig, "terminal/list", json!({ "sessionId": one })).await["value"].as_array().unwrap().len(), 8);
}

#[tokio::test]
async fn a_renamed_terminal_keeps_its_trimmed_title() {
    let rig = rig().await;
    let one = session(&rig).await;
    call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "t1", "cols": 80, "rows": 24 } })).await;
    let renamed = call(&rig, "terminal/rename", json!({ "agentId": one, "id": "t1", "title": "  My term  " })).await;
    assert_eq!(renamed["ok"], true);
    assert_eq!(renamed.get("value"), None, "a rename answers ok and nothing else");
    let listed = call(&rig, "terminal/list", json!({ "sessionId": one })).await;
    assert_eq!(listed["value"][0]["title"], "My term");

    for title in ["   ", &"x".repeat(121)] {
        let refused = call(&rig, "terminal/rename", json!({ "agentId": one, "id": "t1", "title": title })).await;
        assert_eq!(failure(&refused).0, "gateway/internal", "{title}");
        assert_eq!(failure(&refused).1, "Terminal title must contain 1\u{2013}120 characters");
    }
    let unknown = call(&rig, "terminal/rename", json!({ "agentId": one, "id": "nope", "title": "x" })).await;
    assert_eq!(failure(&unknown).0, "terminal/unavailable");
    assert_eq!(failure(&unknown).1, "Terminal no longer exists in this Session");
}

#[tokio::test]
async fn an_unattended_terminal_is_reclaimed_and_its_slot_is_free_again() {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} })).await.unwrap();
    let mut config = TerminalConfig::default();
    // The reclamation fires quickly, so the test does not wait two hours.
    config.unattended_timeout_ms = 1_000;
    config.activity_poll_interval_ms = 1_000;
    let terminal = TerminalService::new(ctx.clone(), config);
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
    })
    .with_terminal(Arc::clone(&terminal));
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.into() }, None));
    let rig = Rig { ctx, gateway };

    let one = session(&rig).await;
    let created = call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "t1", "cols": 80, "rows": 24 } })).await;
    assert!(created["error"].is_null(), "{created}");
    // The shell sits idle, so the monitor reclaims it.
    let mut listed = Vec::new();
    for _ in 0..40 {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        listed = call(&rig, "terminal/list", json!({ "sessionId": one })).await["value"].as_array().unwrap().clone();
        if listed.is_empty() {
            break;
        }
    }
    assert!(listed.is_empty(), "the idle terminal was reclaimed: {}", serde_json::to_string(&listed).unwrap());
    // The identity is closed, and the slot it held is free again.
    let refused = call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "t1", "cols": 80, "rows": 24 } })).await;
    assert_eq!(failure(&refused).0, "terminal/unavailable");
    let other = call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "t2", "cols": 80, "rows": 24 } })).await;
    assert!(other["error"].is_null(), "the slot is free: {other}");
}

#[tokio::test]
async fn a_closed_terminal_cannot_be_created_again() {
    let rig = rig().await;
    let one = session(&rig).await;
    call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "t1", "cols": 80, "rows": 24 } })).await;
    for _ in 0..2 {
        let closed = call(&rig, "terminal/close", json!({ "agentId": one, "id": "t1" })).await;
        assert_eq!(closed["ok"], true);
    }
    let unknown = call(&rig, "terminal/close", json!({ "agentId": one, "id": "zzz" })).await;
    assert_eq!(unknown["ok"], true, "closing an unknown terminal is not an error");
    let refused = call(&rig, "terminal/create", json!({ "agentId": one, "request": { "id": "t1", "cols": 80, "rows": 24 } })).await;
    assert_eq!(failure(&refused).0, "terminal/unavailable");
    assert_eq!(failure(&refused).1, "Terminal was closed in this Session");
    assert_eq!(call(&rig, "terminal/list", json!({ "sessionId": one })).await["value"], json!([]));
}
