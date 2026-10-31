//! Forge panel empty reads, exact refusals, and mux stream lifetime contracts.

use cordis::Context;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const TOKEN: &str = "forge-panel-test-token";
type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Rig {
    ctx: Context,
    address: std::net::SocketAddr,
    gateway: Arc<Gateway>,
}

async fn rig() -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .unwrap();
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel {
            provider: "mock".into(),
            model: "m".into(),
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
    let gateway = Arc::new(
        Gateway::new(
            router,
            Auth {
                token: TOKEN.into(),
            },
            None,
        )
        .with_heartbeat(Duration::from_secs(30))
        .with_approval_sweep(Duration::from_millis(50)),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let serve_ctx = ctx.clone();
    let serving = Arc::clone(&gateway);
    tokio::spawn(async move {
        dsh_rs::gui::server::serve(serving, serve_ctx, listener)
            .await
            .ok();
    });
    Rig {
        ctx,
        address,
        gateway,
    }
}
fn request(rig: &Rig, headers: &[(&str, &str)]) -> http::Request<()> {
    let mut request = format!("ws://{}/api/remote.mux", rig.address)
        .into_client_request()
        .unwrap();
    for (name, value) in headers {
        request.headers_mut().insert(
            http::header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
    }
    request
}

async fn connect(rig: &Rig) -> Socket {
    let bearer = format!("Bearer {TOKEN}");
    let (socket, response) =
        tokio_tungstenite::connect_async(request(rig, &[("authorization", &bearer)]))
            .await
            .expect("the upgrade is accepted");
    assert_eq!(response.status(), 101);
    socket
}

async fn send(socket: &mut Socket, frame: Value) {
    socket.send(Message::text(frame.to_string())).await.unwrap();
}

/// The next text frame as JSON, or panic after two seconds.
async fn next(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .expect("a frame arrives in time")
            .expect("the socket stays open")
            .expect("no transport error");
        if let Message::Text(text) = message {
            return serde_json::from_str(text.as_str()).unwrap();
        }
    }
}

async fn open(socket: &mut Socket, id: &str, endpoint: &str, args: Value) {
    send(
        socket,
        json!({ "type": "open", "streamId": id, "endpoint": endpoint, "payload": { "args": args } }),
    )
    .await;
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, rig.address.to_string().parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "forge", "method": method,
        "payload": { "args": args }
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    assert_eq!(reply.status, 200);
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}

async fn session(rig: &Rig) -> String {
    let result = call(
        rig,
        "session/create",
        json!({ "request": { "cwd": std::env::current_dir().unwrap().display().to_string() } }),
    )
    .await;
    assert_eq!(result["ok"], true, "{result}");
    result["value"]["sessionId"].as_str().unwrap().to_string()
}

fn failed(code: &str, message: &str) -> Value {
    json!({ "ok": false, "error": {
        "code": format!("forgePanel/{code}"), "message": message, "details": {}
    } })
}

#[tokio::test]
async fn every_read_returns_the_complete_empty_wire_shape() {
    let rig = rig().await;
    let id = session(&rig).await;
    for (method, extra, value) in [
        (
            "branches",
            json!({}),
            json!({ "current": null, "branches": [] }),
        ),
        (
            "changeRequestDiff",
            json!({ "url": "https://example.test/pull/1" }),
            json!({ "headSha": "", "files": [], "localAhead": null, "fileUrls": {} }),
        ),
        (
            "changeRequestText",
            json!({ "url": "u", "commit": "c", "path": "a.rs" }),
            json!(""),
        ),
    ] {
        let mut args = extra;
        args["sessionId"] = json!(id);
        assert_eq!(
            call(&rig, &format!("forgePanel/{method}"), args).await,
            json!({ "ok": true, "value": value }),
            "{method}"
        );
    }
    for kind in ["branch", "uncommitted", "unstaged", "staged"] {
        for (method, value) in [
            (
                "localDiff",
                json!({ "base": null, "files": [], "omitted": 0 }),
            ),
            ("localText", json!("")),
        ] {
            let mut args = json!({ "sessionId": id, "scope": { "kind": kind } });
            if method == "localText" {
                args["path"] = json!("a.rs");
            }
            assert_eq!(
                call(&rig, &format!("forgePanel/{method}"), args).await,
                json!({ "ok": true, "value": value }),
                "{method}/{kind}"
            );
        }
    }
}

#[tokio::test]
async fn every_mutation_is_refused_with_the_typescript_code_message_and_details() {
    let rig = rig().await;
    let id = session(&rig).await;
    for (method, extra, code, message) in [
        (
            "cancelWorktree",
            json!({}),
            "worktrees-unavailable",
            "No worktree service is mounted",
        ),
        (
            "continueInProject",
            json!({}),
            "no-worktree",
            "The Session has no created worktree",
        ),
        (
            "dismiss",
            json!({ "url": "u" }),
            "not-bound",
            "The change request is not bound to this Session or no provider serves it",
        ),
        (
            "markReady",
            json!({ "url": "u" }),
            "not-bound",
            "The change request is not bound to this Session or no provider serves it",
        ),
        (
            "merge",
            json!({ "url": "u", "method": "squash", "headSha": "c" }),
            "not-bound",
            "The change request is not bound to this Session or no provider serves it",
        ),
        (
            "refresh",
            json!({}),
            "worktrees-unavailable",
            "No worktree service is mounted",
        ),
        (
            "requestWorktree",
            json!({ "from": null }),
            "worktrees-unavailable",
            "No worktree service is mounted",
        ),
        (
            "requestWorktree",
            json!({ "from": "main" }),
            "worktrees-unavailable",
            "No worktree service is mounted",
        ),
        (
            "requestWorktree",
            json!({ "from": "" }),
            "invalid-branch",
            "The base branch name is empty",
        ),
        (
            "restoreWorktree",
            json!({}),
            "no-worktree",
            "The Session has no created worktree",
        ),
        (
            "retryWorktree",
            json!({}),
            "not-waiting",
            "No message waits for another attempt at the worktree",
        ),
        (
            "stopCreation",
            json!({}),
            "worktrees-unavailable",
            "No worktree service is mounted",
        ),
        (
            "track",
            json!({ "url": "u" }),
            "unrecognized-url",
            "No code host provider recognizes the change request URL",
        ),
    ] {
        let mut args = extra;
        args["sessionId"] = json!(id);
        assert_eq!(
            call(&rig, &format!("forgePanel/{method}"), args).await,
            failed(code, message),
            "{method}"
        );
    }
}

#[tokio::test]
async fn gateway_checks_argument_fields_types_and_session_lookup_before_serving() {
    let rig = rig().await;
    let id = session(&rig).await;
    let missing = call(
        &rig,
        "forgePanel/localText",
        json!({
            "sessionId": id, "scope": { "kind": "staged" }, "extra": true
        }),
    )
    .await;
    assert_eq!(missing["error"]["code"], "gateway/arguments-invalid");
    assert!(missing["error"]["message"]
        .as_str()
        .unwrap()
        .contains("missing \"path\"; unexpected \"extra\""));
    for (endpoint, args, field) in [
        ("branches", json!({ "sessionId": 7 }), "sessionId"),
        (
            "localDiff",
            json!({ "sessionId": id, "scope": { "kind": "other" } }),
            "scope",
        ),
        (
            "merge",
            json!({ "sessionId": id, "url": "u", "method": "other", "headSha": "c" }),
            "method",
        ),
    ] {
        let result = call(&rig, &format!("forgePanel/{endpoint}"), args).await;
        assert_eq!(result["error"]["code"], "gateway/input-invalid", "{result}");
        assert_eq!(result["error"]["details"]["field"], field);
    }
    let unknown = call(&rig, "forgePanel/branches", json!({ "sessionId": "ghost" })).await;
    assert_eq!(unknown["error"]["code"], "session/not-found");
}

#[tokio::test]
async fn streams_send_pending_then_settled_and_release_their_ids_on_cancel() {
    let rig = rig().await;
    let id = session(&rig).await;
    let agents = rig
        .ctx
        .require::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let directory = agents.get(&id).unwrap().session().header_cwd().unwrap();
    let mut socket = connect(&rig).await;
    for (endpoint, args, baseline) in [
        (
            "forgePanel/follow",
            json!({ "sessionId": id }),
            json!({
                "directory": directory, "checkout": { "state": "none" },
                "repository": null, "readiness": null, "changeRequests": [], "local": null,
                "worktree": { "editable": false, "blocker": "not-git", "request": null,
                    "created": null, "creation": null, "failure": null, "missing": null,
                    "restoration": null, "waiting": null }
            }),
        ),
        (
            "forgePanel/changeRequestStates",
            json!({}),
            json!({ "type": "baseline", "states": [] }),
        ),
    ] {
        open(&mut socket, "forge", endpoint, args.clone()).await;
        if endpoint == "forgePanel/follow" {
            let mut pending = baseline.clone();
            pending["checkout"] = json!({"state":"pending"});
            pending["worktree"]["blocker"] = Value::Null;
            assert_eq!(next(&mut socket).await["value"], pending);
        }
        assert_eq!(
            next(&mut socket).await,
            json!({ "type": "item", "streamId": "forge", "value": baseline })
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(100), socket.next())
                .await
                .is_err(),
            "the baseline must remain open without more frames"
        );
        send(
            &mut socket,
            json!({ "type": "cancel", "streamId": "forge" }),
        )
        .await;
        assert!(
            tokio::time::timeout(Duration::from_millis(100), socket.next())
                .await
                .is_err(),
            "cancellation must not send an end or error frame"
        );
        // Reusing the id proves cancellation removed the first stream, not just silenced it.
        open(&mut socket, "forge", endpoint, args).await;
        if endpoint == "forgePanel/follow" {
            let pending = next(&mut socket).await;
            assert_eq!(pending["value"]["checkout"], json!({"state":"pending"}));
            assert_eq!(pending["value"]["worktree"]["blocker"], Value::Null);
        }
        assert_eq!(next(&mut socket).await["value"], baseline);
        send(
            &mut socket,
            json!({ "type": "cancel", "streamId": "forge" }),
        )
        .await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    socket.close(None).await.unwrap();
}

#[tokio::test]
async fn streams_use_the_gateway_argument_refusals() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    for (id, endpoint, args, code) in [
        (
            "missing",
            "forgePanel/follow",
            json!({}),
            "gateway/arguments-invalid",
        ),
        (
            "typed",
            "forgePanel/follow",
            json!({ "sessionId": false }),
            "gateway/input-invalid",
        ),
        (
            "extra",
            "forgePanel/changeRequestStates",
            json!({ "sessionId": "s" }),
            "gateway/arguments-invalid",
        ),
    ] {
        open(&mut socket, id, endpoint, args).await;
        let result = next(&mut socket).await;
        assert_eq!(result["type"], "error", "{result}");
        assert_eq!(result["error"]["code"], code, "{result}");
    }
}
