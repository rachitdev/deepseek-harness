//! The GUI gateway end to end: the real envelope over a real socket, against a
//! real agent, with a recording model.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cordis::Context;
use dsh_rs::api::services::{
    AgentRegistryService, BoxFuture, LlmAdapterApi, LlmService, SessionService,
};
use dsh_rs::session::store::SESSIONS_SERVICE;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::types::{stream_from_chunks, BoxStream, GenerateOptions, LlmError, StreamChunk};
use serde_json::{json, Value};

mod common;
use common::text_response;

const TOKEN: &str = "launch-token-abc123";

struct Script {
    /// A reply of `!message` makes that request fail instead of answering.
    replies: Mutex<VecDeque<String>>,
    /// The `provider/model` of every request, in order.
    routes: Arc<Mutex<Vec<String>>>,
    /// The system prompt of every request, in order.
    systems: Arc<Mutex<Vec<Option<String>>>>,
    /// When set, a reply of `@text` waits for this gate before it answers, so
    /// a test can queue work while the agent is busy.
    gate: Option<Arc<tokio::sync::Notify>>,
}

impl LlmAdapterApi for Script {
    fn name(&self) -> &'static str {
        "script"
    }
    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>> {
        self.routes
            .lock()
            .unwrap()
            .push(format!("{}/{}", options.provider, options.model));
        self.systems
            .lock()
            .unwrap()
            .push(options.system.clone());
        let reply = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| "ok".to_string());
        let gate = self.gate.clone();
        Box::pin(async move {
            if let Some(text) = reply.strip_prefix('@') {
                if let Some(gate) = gate {
                    gate.notified().await;
                }
                let answer = if text.is_empty() { "ok" } else { text };
                return Ok(stream_from_chunks(text_response(answer)));
            }
            match reply.strip_prefix('!') {
                Some(message) => Err(LlmError::code("QUOTA", message)),
                None => Ok(stream_from_chunks(text_response(&reply))),
            }
        })
    }
}

/// A live harness plus a gateway over it.
struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    routes: Arc<Mutex<Vec<String>>>,
    systems: Arc<Mutex<Vec<Option<String>>>>,
}

async fn rig(replies: Vec<&str>) -> Rig {
    rig_with_models(replies, Vec::new()).await
}

async fn rig_with_models(replies: Vec<&str>, models: Vec<DefaultModel>) -> Rig {
    rig_gated(replies, models, None).await
}

/// A rig whose model can be held open, so a test can queue work while the
/// agent is busy with a turn.
async fn rig_gated(
    replies: Vec<&str>,
    models: Vec<DefaultModel>,
    gate: Option<Arc<tokio::sync::Notify>>,
) -> Rig {
    rig_with_profile(replies, models, gate, json!({ "plugins": {} })).await
}

/// A rig whose profile is the caller's, so a test can store its sessions.
async fn rig_with_profile(
    replies: Vec<&str>,
    models: Vec<DefaultModel>,
    gate: Option<Arc<tokio::sync::Notify>>,
    profile: serde_json::Value,
) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &profile)
        .await
        .expect("profile installs");
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    let routes = Arc::new(Mutex::new(Vec::new()));
    let systems = Arc::new(Mutex::new(Vec::new()));
    llm.register_adapter(
        &["mock"],
        Arc::new(Script {
            replies: Mutex::new(replies.into_iter().map(str::to_string).collect()),
            routes: routes.clone(),
            systems: systems.clone(),
            gate,
        }),
    )
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
        models,
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
        routes,
        systems,
    }
}

/// A rig whose router has the office document service, with no converter.
async fn rig_with_office() -> Rig {
    let rig = rig(vec![]).await;
    let service = Arc::new(dsh_rs::gui::office::OfficeService::new(Default::default()));
    let router = dsh_rs::gui::Router::new(dsh_rs::gui::RouterConfig {
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
    })
    .with_office(service);
    let gateway = Arc::new(Gateway::new(
        router,
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    ));
    Rig {
        ctx: rig.ctx,
        gateway,
        routes: rig.routes,
        systems: rig.systems,
    }
}

/// Headers of a same-origin browser request with no credentials yet.
fn local_headers() -> http::HeaderMap {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers
}

fn auth_headers() -> http::HeaderMap {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    headers
}

/// One unary call through `route`, returning the parsed response envelope.
async fn call(rig: &Rig, method: &str, payload: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request",
        "rpcId": "rpc-1",
        "method": method,
        "payload": { "args": payload },
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(
            &rig.ctx,
            &http::Method::POST,
            "/api",
            &auth_headers(),
            &body,
        )
        .await;
    assert_eq!(
        reply.status,
        200,
        "body: {}",
        String::from_utf8_lossy(&reply.body)
    );
    serde_json::from_slice(&reply.body).expect("response is JSON")
}

#[tokio::test]
async fn a_result_with_binary_parts_is_answered_as_multipart() {
    // The TypeScript gateway sends a binary result as multipart: the parts
    // first, then the metadata that names them.
    let rig = rig_with_office().await;
    let directory = tempfile::tempdir().unwrap();
    let document = directory.path().join("a.docx");
    std::fs::write(&document, b"not really a document").unwrap();
    // A converter that writes a document where it is told to.
    let converter = directory.path().join("converter.sh");
    std::fs::write(
        &converter,
        "#!/bin/sh\nout=\"\"\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in --output) out=\"$2\"; shift 2;; *) shift;; esac\ndone\nprintf '%%PDF-1.4 stub\\n' > \"$out\"\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&converter, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let service = Arc::new(dsh_rs::gui::office::OfficeService::new(dsh_rs::gui::office::OfficeConfig {
        command: Some(vec![converter.display().to_string()]),
        ..Default::default()
    }));
    let router = dsh_rs::gui::Router::new(dsh_rs::gui::RouterConfig {
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
    })
    .with_office(service);
    let gateway = Arc::new(Gateway::new(
        router,
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    ));
    let rig = Rig {
        ctx: rig.ctx,
        gateway,
        routes: rig.routes,
        systems: rig.systems,
    };
    let session = value_of(&rig, "session/create", json!({ "cwd": directory.path().display().to_string() })).await["sessionId"]
        .as_str()
        .unwrap()
        .to_string();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "r", "method": "officeToPdf/render",
        "payload": { "args": { "workspaceFileScopeId": session, "path": "a.docx", "priority": "foreground" } }
    }))
    .unwrap();
    let (reply, _) = rig.gateway.route(&rig.ctx, &http::Method::POST, "/api", &headers, &body).await;
    assert!(reply.content_type.starts_with("multipart/form-data"), "{}", reply.content_type);
    let text = String::from_utf8_lossy(&reply.body);
    assert!(text.contains("name=\"bytes-0\""), "the binary part is there: {text}");
    assert!(text.contains("%PDF-1.4"), "the bytes are there: {text}");
    assert!(text.contains("name=\"metadata\""), "the metadata is there: {text}");
    let metadata = text.split("name=\"metadata\"").nth(1).expect("the metadata part");
    let json = metadata.split("\r\n\r\n").nth(1).expect("the metadata body");
    let json = json.split("\r\n--").next().expect("the metadata's own text");
    let envelope: Value = serde_json::from_str(json.trim()).unwrap();
    assert_eq!(envelope["result"]["ok"], true);
    assert_eq!(envelope["result"]["value"]["data"], Value::Null, "the value's own field is null");
    assert_eq!(envelope["attachments"][0]["path"], json!(["data"]));
    assert_eq!(envelope["attachments"][0]["codec"], "bytes");
    assert_eq!(envelope["attachments"][0]["part"], "bytes-0");
}

#[tokio::test]
async fn a_request_without_the_launch_token_is_refused() {
    let rig = rig(vec![]).await;
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "r", "method": "session/list", "payload": { "args": {} }
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(
            &rig.ctx,
            &http::Method::POST,
            "/api",
            &local_headers(),
            &body,
        )
        .await;
    assert_eq!(reply.status, 401);
}

#[tokio::test]
async fn the_index_sets_the_token_cookie_so_a_browser_can_connect() {
    let rig = rig(vec![]).await;
    let (reply, extra) = rig
        .gateway
        .route(&rig.ctx, &http::Method::GET, "/", &local_headers(), b"")
        .await;
    // No bundle is installed in this rig, so the page is missing, but the cookie
    // that makes the next request work must still be set.
    assert_eq!(reply.status, 404);
    let cookie = extra.get(http::header::SET_COOKIE).expect("cookie is set");
    let cookie = cookie.to_str().unwrap();
    assert!(
        cookie.starts_with("dsh_token=launch-token-abc123"),
        "{cookie}"
    );
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Strict"), "{cookie}");
}

#[tokio::test]
async fn a_cookie_authenticates_the_same_as_a_bearer_token() {
    let rig = rig(vec![]).await;
    let mut headers = local_headers();
    headers.insert(
        http::header::COOKIE,
        format!("other=1; dsh_token={TOKEN}").parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "r", "method": "session/list", "payload": { "args": {} }
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    assert_eq!(reply.status, 200);
}

#[tokio::test]
async fn the_wire_round_trip_matches_the_typescript_envelope() {
    let rig = rig(vec![]).await;
    let response = call(&rig, "session/list", json!({ "_request": {} })).await;
    assert_eq!(response["type"], "server-response");
    assert_eq!(response["rpcId"], "rpc-1");
    assert_eq!(response["result"]["ok"], true);
    assert!(response["result"]["value"]["items"].is_array());
}

#[tokio::test]
async fn an_unknown_endpoint_is_a_lookup_failure_that_names_it() {
    let rig = rig(vec![]).await;
    let response = call(&rig, "session/nope", json!({})).await;
    assert_eq!(response["result"]["ok"], false);
    assert_eq!(
        response["result"]["error"]["code"],
        "gateway/lookup-not-found"
    );
    assert_eq!(
        response["result"]["error"]["details"]["endpoint"],
        "session/nope"
    );
}

#[tokio::test]
async fn a_malformed_envelope_answers_with_the_correlation_id_it_could_read() {
    let rig = rig(vec![]).await;
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-9", "method": 42
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(
            &rig.ctx,
            &http::Method::POST,
            "/api",
            &auth_headers(),
            &body,
        )
        .await;
    assert_eq!(reply.status, 400);
    let response: Value = serde_json::from_slice(&reply.body).unwrap();
    assert_eq!(response["rpcId"], "rpc-9");
    assert_eq!(response["result"]["error"]["code"], "gateway/protocol");
}

#[tokio::test]
async fn a_session_is_created_listed_prompted_and_paged_through_the_gateway() {
    let rig = rig(vec!["the answer"]).await;

    let created = call(
        &rig,
        "session/create",
        json!({ "request": { "cwd": "/tmp" } }),
    )
    .await;
    assert_eq!(created["result"]["ok"], true, "{created}");
    let session_id = created["result"]["value"]["sessionId"]
        .as_str()
        .expect("a session id")
        .to_string();

    let listed = call(&rig, "session/list", json!({ "_request": {} })).await;
    let items = listed["result"]["value"]["items"].as_array().unwrap();
    assert!(
        items
            .iter()
            .any(|row| row["sessionId"] == session_id.as_str()),
        "the new session is listed: {items:?}"
    );
    let row = items
        .iter()
        .find(|row| row["sessionId"] == session_id.as_str())
        .unwrap();
    assert_eq!(
        row["blank"], true,
        "a fresh session has no user message yet"
    );
    assert_eq!(row["agentAvailable"], true);

    let prompted = call(
        &rig,
        "session/prompt",
        json!({
            "request": {
                "sessionId": session_id, "requestId": fresh_request_id(),
                "mode": "queue",
                "content": [{ "type": "text", "text": "hello there" }]
            }
        }),
    )
    .await;
    assert_eq!(prompted["result"]["value"]["accepted"], true, "{prompted}");

    // Wait for the turn to finish, then read it back as the client would.
    let agents = (*rig.ctx.require::<AgentRegistryService>("agents").unwrap()).clone();
    agents.get(&session_id).unwrap().when_idle().await;

    let page = call(
        &rig,
        "session/page",
        json!({ "request": { "address": { "kind": "session", "sessionId": session_id } } }),
    )
    .await;
    let records = page["result"]["value"]["records"].as_array().unwrap();
    let types: Vec<&str> = records
        .iter()
        .filter_map(|record| record["event"]["type"].as_str())
        .collect();
    assert!(
        types.contains(&"user/message"),
        "the prompt is in the page: {types:?}"
    );
    assert!(
        types.contains(&"turn/start"),
        "the turn is in the page: {types:?}"
    );
    assert_eq!(records[0]["type"], "event");
    assert_eq!(records[0]["event"]["seq"], 0);
    assert!(records[0]["event"]["time"].is_number());
    // The payload is nested under `data`, and the discriminant is not repeated there.
    assert!(records[0]["event"]["data"].is_object());
    assert!(records[0]["event"]["data"].get("type").is_none());
}

#[tokio::test]
async fn prompting_an_unknown_session_says_so_instead_of_inventing_one() {
    let rig = rig(vec![]).await;
    let response = call(
        &rig,
        "session/prompt",
        json!({ "request": { "sessionId": "nope", "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "x" }] } }),
    )
    .await;
    assert_eq!(
        response["result"]["error"]["code"],
        "session/not-found"
    );
}

#[tokio::test]
async fn a_content_block_the_wire_does_not_describe_is_refused_rather_than_silently_dropped() {
    let rig = rig(vec![]).await;
    let created = call(&rig, "session/create", json!({ "request": {} })).await;
    let session_id = created["result"]["value"]["sessionId"]
        .as_str()
        .unwrap()
        .to_string();
    let response = call(
        &rig,
        "session/prompt",
        json!({
            "request": {
                "sessionId": session_id, "requestId": fresh_request_id(), "mode": "queue",
                "content": [{ "type": "image", "mediaType": "image/png" }]
            }
        }),
    )
    .await;
    assert_eq!(response["result"]["error"]["code"], "gateway/input-invalid");
    assert_eq!(response["result"]["error"]["details"]["field"], "request");
}

#[tokio::test]
async fn a_static_path_cannot_escape_the_asset_root() {
    // A secret next to the asset root: a traversal bug would serve it, so its
    // presence is what makes this test able to fail.
    let parent = std::env::temp_dir().join(format!("dsh-gui-parent-{}", std::process::id()));
    let root = parent.join("public");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(parent.join("secret.txt"), b"do not serve me").unwrap();
    std::fs::write(root.join("index.html"), b"<h1>gui</h1>").unwrap();
    std::fs::write(root.join("app.js"), b"console.log(1)").unwrap();

    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .unwrap();
    let gateway = Gateway::new(
        Router::new(RouterConfig::default()),
        Auth {
            token: TOKEN.to_string(),
        },
        Some(root.clone()),
    );

    // The bundle is served, with the content type the browser needs.
    let (reply, _) = gateway
        .route(&ctx, &http::Method::GET, "/app.js", &auth_headers(), b"")
        .await;
    assert_eq!(reply.status, 200);
    assert_eq!(reply.content_type, "text/javascript; charset=utf-8");
    assert_eq!(reply.body, b"console.log(1)");

    // Escaping the root is refused even though the target exists and is readable.
    for path in [
        "/../secret.txt",
        "/a/../../secret.txt",
        "/sub/../../secret.txt",
    ] {
        let (reply, _) = gateway
            .route(&ctx, &http::Method::GET, path, &auth_headers(), b"")
            .await;
        assert_eq!(reply.status, 404, "{path} must not be served");
        assert!(
            !reply.body.windows(9).any(|window| window == b"do not se"),
            "{path} leaked the file outside the root"
        );
    }
    std::fs::remove_dir_all(&parent).ok();
}

/// The full path a browser takes: a real TCP connection, a real HTTP request.
#[tokio::test]
async fn a_real_http_request_gets_the_envelope_over_a_socket() {
    let rig = rig(vec![]).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let gateway = Arc::clone(&rig.gateway);
    let ctx = rig.ctx.clone();
    tokio::spawn(async move {
        dsh_rs::gui::server::serve(gateway, ctx, listener)
            .await
            .ok();
    });

    let body = serde_json::to_vec(&json!({
        "type": "client-request",
        "rpcId": "over-the-wire",
        "method": "session/list",
        "payload": { "args": { "_request": {} } }
    }))
    .unwrap();

    // A minimal HTTP/1.1 exchange, so the test owns no client dependency.
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let request = format!(
        "POST /api HTTP/1.1\r\nHost: {address}\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(&body).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    let text = String::from_utf8_lossy(&response);
    assert!(text.starts_with("HTTP/1.1 200"), "{text}");
    let payload = text.split("\r\n\r\n").nth(1).expect("a body");
    let envelope: Value = serde_json::from_str(payload).expect("JSON body");
    assert_eq!(envelope["rpcId"], "over-the-wire");
    assert_eq!(envelope["result"]["ok"], true);
}

#[tokio::test]
async fn a_busy_port_fails_loudly_at_install_instead_of_silently_not_listening() {
    // Hold the port so the plugin has to fail.
    let held = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = held.local_addr().unwrap().port();

    let ctx = Context::new();
    let result = install_profile(
        &ctx,
        &json!({ "plugins": { "gui": { "host": "127.0.0.1", "port": port } } }),
    )
    .await;
    let error = match result {
        Ok(_) => panic!("install should fail when the port is taken"),
        Err(error) => error,
    };
    assert!(error.contains("cannot listen"), "{error}");
    assert!(
        error.contains(&port.to_string()),
        "the message names the port: {error}"
    );
    drop(held);
}

#[tokio::test]
async fn the_gui_plugin_serves_a_real_request_end_to_end() {
    // The plugin picks the port; the test learns it from the OS-assigned bind.
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);

    let ctx = Context::new();
    install_profile(
        &ctx,
        &json!({ "plugins": { "gui": { "host": "127.0.0.1", "port": port, "token": "a-test-token-123456" } } }),
    )
    .await
    .expect("the gui plugin installs");

    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "plug", "method": "session/list", "payload": { "args": { "_request": {} } }
    }))
    .unwrap();
    let request = format!(
        "POST /api HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer a-test-token-123456\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );

    let mut last = String::new();
    for _ in 0..50 {
        if let Ok(mut stream) = tokio::net::TcpStream::connect(("127.0.0.1", port)).await {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            if stream.write_all(request.as_bytes()).await.is_ok()
                && stream.write_all(&body).await.is_ok()
            {
                let mut response = Vec::new();
                if stream.read_to_end(&mut response).await.is_ok() {
                    last = String::from_utf8_lossy(&response).to_string();
                    if last.contains("server-response") {
                        break;
                    }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(last.starts_with("HTTP/1.1 200"), "got: {last}");
    let payload = last.split("\r\n\r\n").nth(1).expect("a body");
    let envelope: Value = serde_json::from_str(payload).expect("JSON body");
    assert_eq!(envelope["rpcId"], "plug");
    assert_eq!(envelope["result"]["ok"], true);
}

#[tokio::test]
async fn head_returns_the_get_headers_without_a_body() {
    let dir = std::env::temp_dir().join(format!("dsh-gui-head-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("app.js"), b"console.log(1)").unwrap();

    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .unwrap();
    let gateway = Gateway::new(
        Router::new(RouterConfig::default()),
        Auth {
            token: TOKEN.to_string(),
        },
        Some(dir.clone()),
    );

    let (get, _) = gateway
        .route(&ctx, &http::Method::GET, "/app.js", &auth_headers(), b"")
        .await;
    let (head, _) = gateway
        .route(&ctx, &http::Method::HEAD, "/app.js", &auth_headers(), b"")
        .await;
    assert_eq!(head.status, get.status);
    assert_eq!(head.content_type, get.content_type);
    assert!(head.body.is_empty(), "HEAD must not carry a body");
    assert!(!get.body.is_empty(), "GET must carry the body");
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn a_long_session_page_returns_the_newest_events_not_the_oldest() {
    // The client polls this for a live transcript; a window pinned to the start
    // of the session would never show the current turn.
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .unwrap();
    let agents = (*ctx.require::<AgentRegistryService>("agents").unwrap()).clone();
    let agent = agents
        .create(None, dsh_rs::types::AgentOptions::mock("m"), None, None, None)
        .unwrap();
    let session_id = agent.id();

    // Append more events than the page budget allows.
    let session = agent.session();
    for index in 0..10 {
        session.append(dsh_rs::types::SessionEventData::TurnStart { turn: index });
    }

    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel {
            provider: "mock".to_string(),
            model: "m".to_string(),
        },
        max_page_events: 4,
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
    let rig = Rig {
        ctx,
        gateway,
        routes: Arc::default(),
        systems: Arc::default(),
    };
    let page = call(
        &rig,
        "session/page",
        json!({ "request": { "address": { "kind": "session", "sessionId": session_id } } }),
    )
    .await;
    let records = page["result"]["value"]["records"].as_array().unwrap();
    assert_eq!(records.len(), 4);
    let seqs: Vec<u64> = records
        .iter()
        .map(|record| record["event"]["seq"].as_u64().unwrap())
        .collect();
    assert_eq!(
        seqs,
        vec![6, 7, 8, 9],
        "the newest events are kept, in order"
    );
}

#[tokio::test]
async fn a_payload_without_the_args_wrapper_is_refused_as_a_protocol_error() {
    // The shipped client always sends `{ args: {...} }`; the bare shape this
    // gateway first accepted was a misreading, so it must not come back.
    let rig = rig(vec![]).await;
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "bare", "method": "session/list",
        "payload": { "_request": {} }
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(
            &rig.ctx,
            &http::Method::POST,
            "/api",
            &auth_headers(),
            &body,
        )
        .await;
    let parsed: Value = serde_json::from_slice(&reply.body).unwrap();
    assert_eq!(parsed["result"]["ok"], false);
    assert_eq!(parsed["result"]["error"]["code"], "gateway/protocol");
}

async fn status_with(extra: &[(&str, &str)]) -> u16 {
    let rig = rig(vec![]).await;
    let mut headers = auth_headers();
    for (name, value) in extra {
        headers.insert(
            http::header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
    }
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "f", "method": "session/list", "payload": { "args": {} }
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    reply.status
}

#[tokio::test]
async fn a_valid_token_does_not_excuse_a_foreign_host_or_origin() {
    // DNS rebinding: the token is not the only thing a request must prove.
    assert_eq!(status_with(&[]).await, 200);
    assert_eq!(
        status_with(&[("host", "evil.example")]).await,
        403,
        "rebound Host"
    );
    assert_eq!(
        status_with(&[("origin", "https://evil.example")]).await,
        403,
        "foreign Origin"
    );
    assert_eq!(
        status_with(&[("origin", "null")]).await,
        403,
        "opaque origin"
    );
    assert_eq!(
        status_with(&[("sec-fetch-site", "cross-site")]).await,
        403,
        "cross-site fetch"
    );
    assert_eq!(
        status_with(&[("origin", "http://127.0.0.1:3081")]).await,
        200,
        "the page's own origin is fine"
    );
    assert_eq!(
        status_with(&[("origin", "http://127.0.0.1:9999")]).await,
        403,
        "same host, different port is a different origin"
    );
}

#[tokio::test]
async fn a_missing_host_header_is_refused() {
    let rig = rig(vec![]).await;
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route(
            &rig.ctx,
            &http::Method::GET,
            "/api/endpoints",
            &headers,
            b"",
        )
        .await;
    assert_eq!(reply.status, 403);
}

#[tokio::test]
async fn the_endpoint_listing_names_both_unary_and_stream_endpoints() {
    let rig = rig(vec![]).await;
    let (reply, _) = rig
        .gateway
        .route(
            &rig.ctx,
            &http::Method::GET,
            "/api/endpoints",
            &auth_headers(),
            b"",
        )
        .await;
    assert_eq!(reply.status, 200);
    let listing: serde_json::Value = serde_json::from_slice(&reply.body).unwrap();
    assert!(listing["endpoints"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "session/page"));
    assert!(listing["endpoints"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "$events/result"));
    for name in ["session/cancel", "session/search", "llm/listProviders"] {
        assert!(
            listing["endpoints"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e == name),
            "{name} missing from the listing"
        );
    }
    assert_eq!(listing["streams"], json!([
            "$events",
            "session/follow",
            "workspace/follow",
            "remoteAccess/follow",
            "session/control",
            "account/watch",
            "session/observeState",
            "session/observeEvent",
            "session/observeProjections",
            "account/watchExpiry",
            "terminal/attach",
            "terminal/retain",
            "job/list",
            "job/follow",
            "productAnalytics/watchPolicy",
            "claudeCodeMods/watchBand",
            "toolResults/watch",
            "userQuestions/attachWait",
            "workspaceFiles/changes",
            "forgePanel/follow",
            "forgePanel/changeRequestStates",
            "speech/follow",
        ])
    );
}

async fn health(rig: &Rig, headers: &http::HeaderMap) -> (u16, serde_json::Value) {
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::GET, "/api/health", headers, b"")
        .await;
    (
        reply.status,
        serde_json::from_slice(&reply.body).unwrap_or_default(),
    )
}

#[tokio::test]
async fn health_reports_ready_when_a_chat_turn_could_start() {
    let rig = rig(vec![]).await;
    let (status, body) = health(&rig, &auth_headers()).await;
    assert_eq!(status, 200);
    assert_eq!(body["status"], "ready");
    assert_eq!(body["checks"]["defaultProvider"], true);
    assert_eq!(body["muxSocketOpen"], false);
}

#[tokio::test]
async fn health_names_the_failed_check_when_the_default_provider_is_missing() {
    let rig = rig(vec![]).await;
    let gateway = dsh_rs::gui::Gateway::new(
        dsh_rs::gui::Router::new(dsh_rs::gui::RouterConfig {
            agent_presets: dsh_rs::gui::router::default_presets(),
            default_model: dsh_rs::gui::DefaultModel {
                provider: "no-such-provider".into(),
                model: "m".into(),
            },
            max_page_events: 10,
            max_search_results: 20,
        max_title_bytes: 120,
        max_bytes: 2 * 1024 * 1024,
        max_file_bytes: 32 * 1024 * 1024,
        max_lines: 5_000,
        max_entries: 2_000,
        default_max_goal_rounds: 256,
        export_compression_level: 6,
        models: Vec::new(),
        }),
        dsh_rs::gui::Auth {
            token: TOKEN.into(),
        },
        None,
    );
    let (reply, _) = gateway
        .route(
            &rig.ctx,
            &http::Method::GET,
            "/api/health",
            &auth_headers(),
            b"",
        )
        .await;
    let body: serde_json::Value = serde_json::from_slice(&reply.body).unwrap();
    assert_eq!(reply.status, 503);
    assert_eq!(body["status"], "unavailable");
    assert_eq!(body["checks"]["defaultProvider"], false);
    assert_eq!(
        body["checks"]["agents"], true,
        "only the failing check is false"
    );
}

#[tokio::test]
async fn health_is_not_readable_without_the_token() {
    let rig = rig(vec![]).await;
    let (status, _) = health(&rig, &local_headers()).await;
    assert_eq!(status, 401);
}

// -- session/cancel, session/search, llm/listProviders -------------------------

/// A model that never answers, so a turn stays open until it is cancelled.
struct Hang;

impl LlmAdapterApi for Hang {
    fn name(&self) -> &'static str {
        "hang"
    }
    fn stream(
        &self,
        _options: GenerateOptions,
    ) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>> {
        Box::pin(async move {
            std::future::pending::<()>().await;
            unreachable!("a hung model never answers")
        })
    }
}

async fn new_session(rig: &Rig) -> String {
    let created = call(
        rig,
        "session/create",
        json!({ "request": { "cwd": "/tmp" } }),
    )
    .await;
    created["result"]["value"]["sessionId"]
        .as_str()
        .unwrap()
        .to_string()
}

/// A client prompt identity no other prompt in this test run uses; a repeated
/// identity is a retry, which the host answers without queueing a second copy.
fn fresh_request_id() -> String {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    format!("t-req-{}", NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
}

async fn prompt(rig: &Rig, session: &str, text: &str) {
    let sent = call(
        rig,
        "session/prompt",
        json!({ "request": { "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": text }] } }),
    )
    .await;
    assert_eq!(sent["result"]["ok"], true, "{sent}");
}

#[tokio::test]
async fn cancel_stops_a_turn_that_is_waiting_on_the_model() {
    let rig = rig(vec![]).await;
    let llm = rig.ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(&["mock"], Arc::new(Hang)).unwrap();
    let session = new_session(&rig).await;
    prompt(&rig, &session, "never answered").await;

    let agents = rig.ctx.require::<AgentRegistryService>("agents").unwrap();
    let agent = agents.get(&session).unwrap();
    // Wait until the turn is really open and the model call is pending.
    for _ in 0..200 {
        if agent.status() == dsh_rs::types::AgentStatus::Running {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(
        agent.status(),
        dsh_rs::types::AgentStatus::Running,
        "turn never started"
    );

    let cancelled = call(
        &rig,
        "session/cancel",
        json!({ "request": { "sessionId": session } }),
    )
    .await;
    assert_eq!(
        cancelled["result"],
        json!({ "ok": true, "value": { "accepted": true } })
    );

    tokio::time::timeout(std::time::Duration::from_secs(5), agent.when_idle())
        .await
        .expect("the agent must go idle after a cancel");
    let events = agent.session().events();
    assert!(
        events.iter().any(|e| matches!(
            &e.data,
            dsh_rs::types::SessionEventData::TurnEnd {
                reason: dsh_rs::types::TurnEndReason::Aborted { .. },
                ..
            }
        )),
        "the turn must end as aborted"
    );
}

#[tokio::test]
async fn cancel_of_an_unknown_session_names_it_and_a_bad_request_is_invalid() {
    let rig = rig(vec![]).await;
    let missing = call(
        &rig,
        "session/cancel",
        json!({ "request": { "sessionId": "nope" } }),
    )
    .await;
    assert_eq!(
        missing["result"]["error"]["code"],
        "session/not-found"
    );
    let bad = call(&rig, "session/cancel", json!({ "request": {} })).await;
    assert_eq!(bad["result"]["error"]["code"], "gateway/input-invalid");
    let none = call(&rig, "session/cancel", json!({})).await;
    assert_eq!(none["result"]["error"]["code"], "gateway/arguments-invalid");
}

#[tokio::test]
async fn search_finds_text_case_insensitively_with_a_snippet_and_skips_other_sessions() {
    let rig = rig(vec!["a reply about Pineapples", "unrelated"]).await;
    let first = new_session(&rig).await;
    prompt(&rig, &first, "tell me about fruit").await;
    let second = new_session(&rig).await;
    prompt(&rig, &second, "hello there").await;
    for id in [&first, &second] {
        let agents = rig.ctx.require::<AgentRegistryService>("agents").unwrap();
        agents.get(id).unwrap().when_idle().await;
    }

    let found = call(
        &rig,
        "session/search",
        json!({ "request": { "query": "PINEAPPLE" } }),
    )
    .await;
    let value = &found["result"]["value"];
    assert_eq!(value["hasMore"], false, "{found}");
    let items = value["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "{found}");
    assert_eq!(items[0]["sessionId"], first.as_str());
    assert!(items[0]["snippet"].as_str().unwrap().contains("Pineapples"));

    let none = call(
        &rig,
        "session/search",
        json!({ "request": { "query": "zzz-absent" } }),
    )
    .await;
    assert_eq!(
        none["result"]["value"],
        json!({ "items": [], "hasMore": false })
    );
    let blank = call(
        &rig,
        "session/search",
        json!({ "request": { "query": "   " } }),
    )
    .await;
    // The TypeScript host refuses an empty query as a bad request.
    assert_eq!(blank["result"]["error"]["code"], "gateway/bad-request");
}

#[tokio::test]
async fn search_caps_hits_and_says_when_there_are_more() {
    let rig = rig(vec![]).await; // default mock echoes the prompt, so every session matches
    let cap = 20;
    let mut ids = Vec::new();
    for n in 0..cap + 2 {
        let id = new_session(&rig).await;
        prompt(&rig, &id, &format!("needle {n}")).await;
        ids.push(id);
    }
    let agents = rig.ctx.require::<AgentRegistryService>("agents").unwrap();
    for id in &ids {
        agents.get(id).unwrap().when_idle().await;
    }
    let found = call(
        &rig,
        "session/search",
        json!({ "request": { "query": "needle" } }),
    )
    .await;
    assert_eq!(
        found["result"]["value"]["items"].as_array().unwrap().len(),
        cap
    );
    assert_eq!(found["result"]["value"]["hasMore"], true);
}

#[tokio::test]
async fn the_provider_list_names_the_registered_routes() {
    let rig = rig(vec![]).await;
    let listed = call(&rig, "llm/listProviders", json!({})).await;
    let providers = listed["result"]["value"].as_array().expect("an array");
    assert!(
        providers
            .iter()
            .any(|p| p["id"] == "mock" && p["name"].is_string()),
        "{listed}"
    );
}

#[tokio::test]
async fn cancel_also_drops_prompts_queued_behind_the_running_turn() {
    let rig = rig(vec![]).await;
    let llm = rig.ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(&["mock"], Arc::new(Hang)).unwrap();
    let session = new_session(&rig).await;
    prompt(&rig, &session, "first").await;
    let agents = rig.ctx.require::<AgentRegistryService>("agents").unwrap();
    let agent = agents.get(&session).unwrap();
    for _ in 0..200 {
        if agent.status() == dsh_rs::types::AgentStatus::Running {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    prompt(&rig, &session, "queued behind the first").await;

    call(
        &rig,
        "session/cancel",
        json!({ "request": { "sessionId": session } }),
    )
    .await;
    // A kept queue would start a second turn on the hung model and never go idle.
    tokio::time::timeout(std::time::Duration::from_secs(5), agent.when_idle())
        .await
        .expect("cancel must clear the queue so the agent settles");
    let started = agent
        .session()
        .events()
        .iter()
        .filter(|e| matches!(e.data, dsh_rs::types::SessionEventData::TurnStart { .. }))
        .count();
    assert_eq!(started, 1, "the queued prompt must not have started a turn");
}

/// Post a client-request to an explicit URL path.
async fn call_at(rig: &Rig, path: &str, method: &str) -> (u16, Value) {
    // `session/list` names its one parameter `_request`; the rest take none here.
    let args = if method == "session/list" { json!({ "_request": {} }) } else { json!({}) };
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-9", "method": method, "payload": { "args": args },
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, path, &auth_headers(), &body)
        .await;
    (reply.status, serde_json::from_slice(&reply.body).unwrap_or(Value::Null))
}

#[tokio::test]
async fn the_shipped_client_posts_to_the_endpoint_path_and_is_served_there() {
    let rig = rig(vec![]).await;
    let (status, reply) = call_at(&rig, "/api/session/list", "session/list").await;
    assert_eq!(status, 200);
    assert_eq!(reply["rpcId"], "rpc-9");
    assert_eq!(reply["result"]["ok"], true, "{reply}");
}

#[tokio::test]
async fn a_method_that_contradicts_its_url_is_refused_not_dispatched() {
    let rig = rig(vec![]).await;
    let (status, reply) = call_at(&rig, "/api/session/list", "session/create").await;
    assert_eq!(status, 200);
    assert_eq!(reply["result"]["error"]["code"], "gateway/bad-request", "{reply}");
}

#[tokio::test]
async fn malformed_endpoint_paths_are_not_found() {
    let rig = rig(vec![]).await;
    for path in ["/api/", "/api//list", "/api/session/../list", "/api/session/li st", "/api/a%2Fb"] {
        let (status, reply) = call_at(&rig, path, "session/list").await;
        assert_eq!(status, 404, "{path}");
        assert_eq!(reply["result"]["error"]["code"], "gateway/lookup-not-found", "{path}");
    }
}

#[tokio::test]
async fn a_created_session_names_the_preset_it_runs() {
    let rig = rig(vec![]).await;
    let created = value_of(&rig, "session/create", json!({ "agentPreset": "default" })).await;
    assert_eq!(created["agentPreset"], "default");
    let session = created["sessionId"].as_str().unwrap().to_string();
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(projected["values"]["agentPreset"], "default");
}

#[tokio::test]
async fn an_unknown_preset_is_refused_and_the_roster_is_named() {
    let rig = rig(vec![]).await;
    let reply = call(&rig, "session/create", json!({ "request": { "agentPreset": "nope" } })).await;
    assert_eq!(reply["result"]["ok"], false, "{reply}");
    assert_eq!(reply["result"]["error"]["code"], "agent-preset/not-found", "{reply}");
    assert_eq!(reply["result"]["error"]["details"]["agentPreset"], "nope", "{reply}");
    assert_eq!(
        reply["result"]["error"]["details"]["available"],
        json!(["default"]),
        "{reply}"
    );
}

#[tokio::test]
async fn a_session_without_a_preset_runs_the_rosters_default() {
    let rig = rig(vec![]).await;
    let created = value_of(&rig, "session/create", json!({})).await;
    assert_eq!(created["agentPreset"], "default");
}

/// Selecting a preset records it once; selecting it again records nothing.
#[tokio::test]
async fn selecting_a_preset_records_it_once_and_then_keeps_the_log_quiet() {
    let rig = rig_with_presets(vec!["done".into()]).await;
    let session = new_session(&rig).await;
    let selected = call(
        &rig,
        "agentPresets/select",
        json!({ "agentId": session, "agentPreset": "reviewer" }),
    )
    .await;
    assert_eq!(selected["result"]["value"], json!("reviewer"), "{selected}");
    let first = selection_rows(&rig, &session).await;
    assert_eq!(first.len(), 1, "{first:?}");

    let again = call(
        &rig,
        "agentPresets/select",
        json!({ "agentId": session, "agentPreset": "reviewer" }),
    )
    .await;
    assert_eq!(again["result"]["value"], json!("reviewer"), "{again}");
    let second = selection_rows(&rig, &session).await;
    assert_eq!(second.len(), 1, "the same preset is not recorded twice");
}

/// A session that started a turn keeps its preset.
#[tokio::test]
async fn a_started_session_refuses_a_preset_change() {
    let rig = rig_with_presets(vec!["done".into()]).await;
    let session = new_session(&rig).await;
    prompt(&rig, &session, "hello").await;
    // The turn has to be open before the preset is locked.
    for _ in 0..200 {
        let page = value_of(
            &rig,
            "session/page",
            json!({ "address": { "kind": "session", "sessionId": session }, "throughSeq": 1_000 }),
        )
        .await;
        if page["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["event"]["type"] == "turn/start")
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let refused = call(
        &rig,
        "agentPresets/select",
        json!({ "agentId": session, "agentPreset": "reviewer" }),
    )
    .await;
    assert_eq!(refused["result"]["ok"], false, "{refused}");
    assert_eq!(
        refused["result"]["error"]["code"],
        "agent-preset/locked",
        "{refused}"
    );
}

/// A preset this deployment does not offer cannot be selected.
#[tokio::test]
async fn an_unknown_preset_is_not_selected() {
    let rig = rig_with_presets(vec!["done".into()]).await;
    let session = new_session(&rig).await;
    let refused = call(
        &rig,
        "agentPresets/select",
        json!({ "agentId": session, "agentPreset": "nonesuch" }),
    )
    .await;
    assert_eq!(refused["result"]["ok"], false, "{refused}");
    assert_eq!(
        refused["result"]["error"]["code"],
        "agent-preset/not-found",
        "{refused}"
    );
}

/// One preset's declaration reads as a document beside its metadata.
#[tokio::test]
async fn one_preset_declaration_reads_as_a_document() {
    let rig = rig_with_presets(vec![]).await;
    let document = call(
        &rig,
        "agentPresets/read",
        json!({ "agentPreset": "reviewer" }),
    )
    .await;
    assert_eq!(document["result"]["ok"], true, "{document}");
    let value = &document["result"]["value"];
    assert_eq!(value["agentPreset"], "reviewer", "{document}");
    assert_eq!(value["name"], "Reviewer", "{document}");
    let content = value["content"].as_str().expect("the declaration is text");
    // The declaration is the preset's child plugin list as entry-list YAML.
    // A dsh-rs preset declares none, so the list is empty.
    assert_eq!(content.trim(), "[]", "{content}");

    let missing = call(
        &rig,
        "agentPresets/read",
        json!({ "agentPreset": "nonesuch" }),
    )
    .await;
    assert_eq!(missing["result"]["ok"], false, "{missing}");
    assert_eq!(
        missing["result"]["error"]["code"],
        "agent-preset/not-found",
        "{missing}"
    );
}

/// The preset selections a session's log carries.
async fn selection_rows(rig: &Rig, session: &str) -> Vec<Value> {
    let page = value_of(
        rig,
        "session/page",
        json!({ "address": { "kind": "session", "sessionId": session }, "throughSeq": 1_000 }),
    )
    .await;
    page["records"]
        .as_array()
        .expect("the page carries records")
        .iter()
        .filter(|record| record["event"]["type"] == "agent-preset/selected")
        .cloned()
        .collect()
}

/// A rig whose deployment offers two presets.
async fn rig_with_presets(replies: Vec<&str>) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .expect("profile installs");
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    let systems = Arc::new(Mutex::new(Vec::new()));
    llm.register_adapter(
        &["mock"],
        Arc::new(Script {
            replies: Mutex::new(replies.into_iter().map(str::to_string).collect()),
            routes: Arc::new(Mutex::new(Vec::new())),
            systems: systems.clone(),
            gate: None,
        }),
    )
    .unwrap();
    let mut presets = dsh_rs::gui::router::default_presets();
    presets.push(dsh_rs::gui::router::AgentPreset {
        id: "reviewer".to_string(),
        name: "Reviewer".to_string(),
        description: None,
        is_default: false,
        system_prompt: Some("You review code.".to_string()),
    });
    dsh_rs::gui::router::mount_preset_prompts(&ctx, &presets).expect("the roster mounts");
    let router = Router::new(RouterConfig {
        agent_presets: presets,
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
        routes: Arc::new(Mutex::new(Vec::new())),
        systems,
    }
}

#[tokio::test]
async fn a_blank_session_changes_its_preset_and_the_change_is_recorded() {
    let rig = rig_with_presets(vec![]).await;
    let created = value_of(&rig, "session/create", json!({ "agentPreset": "default" })).await;
    let session = created["sessionId"].as_str().unwrap().to_string();
    let again = value_of(&rig, "session/create", json!({ "sessionId": session })).await;
    assert_eq!(again["agentPreset"], "default", "the same preset is fine");

    let changed = value_of(
        &rig,
        "session/create",
        json!({ "sessionId": session, "agentPreset": "reviewer" }),
    )
    .await;
    assert_eq!(changed["sessionId"], session, "the blank session is reused");
    assert_eq!(changed["agentPreset"], "reviewer", "the new preset runs");

    // The change is in the log, so the session says which composition it runs.
    let page = value_of(
        &rig,
        "session/page",
        json!({ "address": { "kind": "session", "sessionId": session } }),
    )
    .await;
    let rows = page["records"].as_array().expect("records");
    let selection = rows
        .iter()
        .find(|row| row["event"]["type"] == "agent-preset/selected")
        .unwrap_or_else(|| panic!("the preset change is logged: {rows:?}"));
    assert_eq!(selection["event"]["data"]["agentPreset"], "reviewer", "{selection}");

    // The projection reports the preset the session runs.
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(projected["values"]["agentPreset"], "reviewer");
}

#[tokio::test]
async fn a_started_session_is_not_reused_for_another_preset() {
    let rig = rig_with_presets(vec!["hi".into()]).await;
    let created = value_of(&rig, "session/create", json!({ "agentPreset": "default" })).await;
    let session = created["sessionId"].as_str().unwrap().to_string();
    let prompted = call(
        &rig,
        "session/prompt",
        json!({ "request": { "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "x" }] } }),
    )
    .await;
    assert_eq!(prompted["result"]["ok"], true, "{prompted}");
    // Wait for the turn to start, so the session is no longer blank.
    for _ in 0..50 {
        let listed = value_of(&rig, "session/list", json!({})).await;
        if listed["items"][0]["blank"] == false {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    // The started session is not reusable, so a fresh session runs the
    // request under the requested preset, as the TypeScript host does.
    let fresh = value_of(
        &rig,
        "session/create",
        json!({ "sessionId": session, "agentPreset": "reviewer" }),
    )
    .await;
    assert_ne!(fresh["sessionId"], session.as_str(), "a started session is never reused");
    assert_eq!(fresh["agentPreset"], "reviewer");
}

#[tokio::test]
async fn the_preset_instructions_reach_the_model() {
    let rig = rig_with_presets(vec!["done".into()]).await;
    let created = value_of(&rig, "session/create", json!({ "agentPreset": "reviewer" })).await;
    let session = created["sessionId"].as_str().unwrap().to_string();
    let prompted = call(
        &rig,
        "session/prompt",
        json!({ "request": { "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "x" }] } }),
    )
    .await;
    assert_eq!(prompted["result"]["ok"], true, "{prompted}");
    for _ in 0..50 {
        if rig.systems.lock().unwrap().len() > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let systems = rig.systems.lock().unwrap().clone();
    let system = systems.first().and_then(|s| s.clone()).expect("a request ran");
    assert!(
        system.contains("You review code."),
        "the preset's instructions reach the model: {system}"
    );
}

/// A roster that changes does not alter a session that already runs a preset:
/// the session keeps the instructions it was bound to, and a session created
/// after the change runs the current ones.
#[tokio::test]
async fn a_changed_roster_leaves_a_bound_session_alone() {
    let rig = rig_with_presets(vec!["done".into(), "done".into()]).await;
    let created = value_of(&rig, "session/create", json!({ "agentPreset": "reviewer" })).await;
    let session = created["sessionId"].as_str().unwrap().to_string();

    // The deployment changes what the preset contributes.
    let mut presets = dsh_rs::gui::router::default_presets();
    presets.push(dsh_rs::gui::router::AgentPreset {
        id: "reviewer".to_string(),
        name: "Reviewer".to_string(),
        description: None,
        is_default: false,
        system_prompt: Some("You review code carefully.".to_string()),
    });
    dsh_rs::gui::router::mount_preset_prompts(&rig.ctx, &presets).expect("the roster mounts");

    call(
        &rig,
        "session/prompt",
        json!({ "request": { "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "x" }] } }),
    )
    .await;
    for _ in 0..50 {
        if !rig.systems.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let systems = rig.systems.lock().unwrap().clone();
    let bound = systems.first().and_then(|s| s.clone()).expect("a request ran");
    assert!(
        bound.contains("You review code.") && !bound.contains("carefully"),
        "the bound session keeps the instructions it was bound to: {bound}"
    );

    // A session created after the change runs the current instructions.
    let created = value_of(&rig, "session/create", json!({ "agentPreset": "reviewer" })).await;
    let fresh = created["sessionId"].as_str().unwrap().to_string();
    call(
        &rig,
        "session/prompt",
        json!({ "request": { "sessionId": fresh, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "x" }] } }),
    )
    .await;
    for _ in 0..50 {
        if rig.systems.lock().unwrap().len() > 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let systems = rig.systems.lock().unwrap().clone();
    let current = systems.get(1).and_then(|s| s.clone()).expect("the second request ran");
    assert!(
        current.contains("You review code carefully."),
        "a new session runs the current instructions: {current}"
    );
}

/// A stored session no live agent covers is still listed and still
/// searchable: the TypeScript host serves it from the stored record.
#[tokio::test]
async fn a_stored_session_no_live_agent_covers_is_listed_and_searchable() {
    let dir = std::env::temp_dir().join(format!("dsh-gui-stored-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let rig = rig_with_profile(
        vec!["done".into()],
        Vec::new(),
        None,
        json!({
            "plugins": {},
            "config": {
                "store_dir": dir.to_string_lossy(),
                "store_format": "v4-plain"
            }
        }),
    )
    .await;
    let created = value_of(&rig, "session/create", json!({ "cwd": "/tmp" })).await;
    let session = created["sessionId"].as_str().unwrap().to_string();
    call(
        &rig,
        "session/prompt",
        json!({ "request": { "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "a needle in the transcript" }] } }),
    )
    .await;
    for _ in 0..50 {
        let page = value_of(
            &rig,
            "session/page",
            json!({ "address": { "kind": "session", "sessionId": session } }),
        )
        .await;
        if page["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["event"]["type"] == "assistant/message")
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    // The log is on the disk, and the runtime forgets the session.
    let store = rig.ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.flush(&session).await.unwrap();
    let agents = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents.get(&session).expect("the session is live");
    agents.dispose(&agent);

    let listed = call(&rig, "session/list", json!({ "_request": {} })).await;
    let rows = listed["result"]["value"]["items"].as_array().unwrap().clone();
    let row = rows
        .iter()
        .find(|row| row["sessionId"] == session.as_str())
        .expect("the stored session is listed");
    assert_eq!(row["agentAvailable"], false, "{row}");
    assert_eq!(row["running"], false, "{row}");
    assert_eq!(row["formatStatus"], "current", "{row}");

    let found = value_of(&rig, "session/search", json!({ "query": "needle" })).await;
    let items = found["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "{found}");
    assert_eq!(items[0]["sessionId"], session.as_str(), "{found}");
    assert!(
        items[0]["snippet"].as_str().unwrap().contains("needle"),
        "{found}"
    );
}

/// The list orders the rows by activity, newest first, the way the
/// TypeScript host does.
#[tokio::test]
async fn the_list_orders_the_rows_by_activity() {
    let rig = rig(vec!["done".into()]).await;
    let first = value_of(&rig, "session/create", json!({ "cwd": "/tmp" })).await;
    let first = first["sessionId"].as_str().unwrap().to_string();
    let second = value_of(&rig, "session/create", json!({ "cwd": "/tmp" })).await;
    let second = second["sessionId"].as_str().unwrap().to_string();
    // Activity is a millisecond clock: prompt strictly after both creations.
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    call(
        &rig,
        "session/prompt",
        json!({ "request": { "sessionId": second, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "later" }] } }),
    )
    .await;
    for _ in 0..50 {
        let page = value_of(
            &rig,
            "session/page",
            json!({ "address": { "kind": "session", "sessionId": second } }),
        )
        .await;
        if page["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["event"]["type"] == "assistant/message")
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let listed = call(&rig, "session/list", json!({ "_request": {} })).await;
    let rows = listed["result"]["value"]["items"].as_array().unwrap().clone();
    let order: Vec<&str> = rows
        .iter()
        .map(|row| row["sessionId"].as_str().unwrap())
        .collect();
    let second_at = order.iter().position(|id| *id == second).expect("listed");
    let first_at = order.iter().position(|id| *id == first).expect("listed");
    assert!(second_at < first_at, "the active session comes first: {order:?}");
}

/// An empty search query is refused as the TypeScript host refuses it.
#[tokio::test]
async fn an_empty_search_query_is_refused_as_a_bad_request() {
    let rig = rig(vec![]).await;
    let code = code_of(&rig, "session/search", json!({ "query": "   " })).await;
    assert_eq!(code, "gateway/bad-request");
}

#[tokio::test]
async fn an_unserved_endpoint_reports_lookup_not_found_rather_than_a_transport_error() {
    let rig = rig(vec![]).await;
    let (status, reply) = call_at(&rig, "/api/pluginManager/listBundles", "pluginManager/listBundles").await;
    assert_eq!(status, 200);
    assert_eq!(reply["result"]["error"]["code"], "gateway/lookup-not-found", "{reply}");
}

#[tokio::test]
async fn the_boot_catalog_answers_with_the_shapes_the_client_reads() {
    let rig = rig(vec![]).await;
    let presets = call(&rig, "agentPresets/list", json!({})).await;
    let list = presets["result"]["value"]["presets"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["isDefault"], true);
    assert!(list[0].get("broken").is_none(), "the one preset can compose a session");

    let perms = call(&rig, "permissionPresets/catalog", json!({})).await;
    let value = &perms["result"]["value"];
    assert_eq!(value["defaultPreset"], "auto");
    assert!(value["options"].as_array().unwrap().iter().any(|o| o["value"] == "auto"));

    let settings = call(&rig, "settings/describe", json!({})).await;
    assert_eq!(settings["result"]["value"]["writable"], true);
    assert_eq!(settings["result"]["value"]["namespaces"][0]["ns"], "ui-settings-general");

    let inventory = call(&rig, "dynamicCordisRunner/inventory", json!({})).await;
    assert_eq!(inventory["result"]["value"], json!([]));
}

#[tokio::test]
async fn the_model_catalog_offers_the_default_route_only_when_its_provider_is_registered() {
    let rig = rig(vec![]).await;
    let reply = call(&rig, "session/modelCatalog", json!({})).await;
    let value = &reply["result"]["value"];
    // The rig's default route is `mock`, which the rig registers.
    assert_eq!(value["default"]["provider"], "mock");
    assert_eq!(value["routableProviders"], json!(["mock"]));
    let models = value["groups"][0]["models"].as_array().unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0]["id"], value["default"]["model"]);
    assert_eq!(value["failures"], json!([]));
}

#[tokio::test]
async fn the_inspect_manifest_is_accepted_only_as_a_list_of_providers() {
    let rig = rig(vec![]).await;
    let ok = call(&rig, "dynamicCordisRunner/syncInspectManifest", json!({ "providers": [] })).await;
    assert_eq!(ok["result"], json!({ "ok": true, "value": null }));
    let bad = call(&rig, "dynamicCordisRunner/syncInspectManifest", json!({ "providers": 1 })).await;
    assert_eq!(bad["result"]["error"]["code"], "gateway/input-invalid");
}

#[tokio::test]
async fn open_in_app_lists_no_applications_and_needs_the_token() {
    let rig = rig(vec![]).await;
    let (denied, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::GET, "/open-in-app/apps", &local_headers(), b"")
        .await;
    assert_eq!(denied.status, 401);
    let (ok, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::GET, "/open-in-app/apps", &auth_headers(), b"")
        .await;
    assert_eq!(ok.status, 200);
    assert_eq!(serde_json::from_slice::<Value>(&ok.body).unwrap(), json!({ "apps": [] }));
}

// --- workspaces ---------------------------------------------------------

fn directory(name: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    std::fs::create_dir(&path).unwrap();
    let canonical = std::fs::canonicalize(&path).unwrap().display().to_string();
    (dir, canonical)
}

async fn value_of(rig: &Rig, method: &str, request: Value) -> Value {
    // Wire names per the descriptor: `session/list` takes `_request`, the model
    // catalog takes nothing.
    let args = match method {
        "session/list" => json!({ "_request": request }),
        "session/modelCatalog" => json!({}),
        _ => json!({ "request": request }),
    };
    let response = call(rig, method, args).await;
    assert_eq!(response["result"]["ok"], true, "{method}: {response}");
    response["result"]["value"].clone()
}

async fn code_of(rig: &Rig, method: &str, request: Value) -> String {
    let response = call(rig, method, json!({ "request": request })).await;
    assert_eq!(response["result"]["ok"], false, "{method}: {response}");
    response["result"]["error"]["code"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn creating_a_workspace_twice_resolves_to_the_same_one() {
    let rig = rig(vec![]).await;
    let (_keep, path) = directory("proj");
    let first = value_of(&rig, "workspace/create", json!({ "path": path })).await;
    assert_eq!(first["created"], true);
    assert_eq!(first["workspace"]["path"], path.as_str());
    assert_eq!(first["workspace"]["title"], "proj");
    let again = value_of(&rig, "workspace/create", json!({ "path": path })).await;
    assert_eq!(again["created"], false);
    assert_eq!(again["workspace"]["workspaceId"], first["workspace"]["workspaceId"]);
}

#[tokio::test]
async fn a_path_that_is_not_a_directory_is_refused_as_invalid() {
    let rig = rig(vec![]).await;
    let (keep, _) = directory("proj");
    let file = keep.path().join("file.txt");
    std::fs::write(&file, "x").unwrap();
    for bad in [file.display().to_string(), "/no/such/dir/at-all".to_string(), "  ".to_string()] {
        assert_eq!(
            code_of(&rig, "workspace/create", json!({ "path": bad })).await,
            "workspace/invalid-path"
        );
    }
}

#[tokio::test]
async fn rename_needs_a_unique_non_blank_title() {
    let rig = rig(vec![]).await;
    let (_a, one) = directory("one");
    let (_b, two) = directory("two");
    let first = value_of(&rig, "workspace/create", json!({ "path": one })).await;
    let second = value_of(&rig, "workspace/create", json!({ "path": two })).await;
    let id = first["workspace"]["workspaceId"].as_str().unwrap();
    assert_eq!(
        code_of(&rig, "workspace/rename", json!({ "workspaceId": id, "title": "   " })).await,
        "gateway/bad-request"
    );
    assert_eq!(
        code_of(&rig, "workspace/rename", json!({ "workspaceId": id, "title": "two" })).await,
        "workspace/name-conflict"
    );
    assert_eq!(
        code_of(&rig, "workspace/rename", json!({ "workspaceId": "nope", "title": "x" })).await,
        "workspace/not-found"
    );
    let renamed = value_of(&rig, "workspace/rename", json!({ "workspaceId": id, "title": " Mine " })).await;
    assert_eq!(renamed["workspace"]["title"], "Mine");
    let _ = second;
}

#[tokio::test]
async fn a_session_created_in_a_workspace_is_accounted_to_it_and_survives_its_deletion() {
    let rig = rig(vec![]).await;
    let (_keep, path) = directory("proj");
    let workspace = value_of(&rig, "workspace/create", json!({ "path": path })).await;
    let id = workspace["workspace"]["workspaceId"].as_str().unwrap();
    let created = value_of(&rig, "session/create", json!({ "workspaceId": id })).await;
    let session = created["sessionId"].as_str().unwrap();
    let page = value_of(&rig, "session/list", json!({})).await;
    let row = page["items"].as_array().unwrap().iter().find(|r| r["sessionId"] == session).unwrap();
    assert_eq!(row["cwd"], path.as_str());

    let moved = value_of(
        &rig,
        "workspace/insertSessionBefore",
        json!({ "workspaceId": id, "sessionId": session }),
    )
    .await;
    assert_eq!(moved["workspace"]["sessionIds"], json!([session]));

    assert_eq!(
        code_of(&rig, "session/create", json!({ "workspaceId": "ghost" })).await,
        "workspace/not-found"
    );
    assert_eq!(
        code_of(&rig, "session/create", json!({ "workspaceId": id, "cwd": path })).await,
        "gateway/input-invalid"
    );

    let deleted = value_of(&rig, "workspace/delete", json!({ "workspaceId": id })).await;
    assert_eq!(deleted["deleted"], true);
    assert_eq!(
        code_of(&rig, "workspace/delete", json!({ "workspaceId": id })).await,
        "workspace/not-found"
    );
    let after = value_of(&rig, "session/list", json!({})).await;
    assert!(after["items"].as_array().unwrap().iter().any(|r| r["sessionId"] == session));
}

#[tokio::test]
async fn workspaces_reorder_and_a_move_names_what_it_could_not_find() {
    let rig = rig(vec![]).await;
    let (_a, one) = directory("one");
    let (_b, two) = directory("two");
    let (_c, three) = directory("three");
    let mut ids = Vec::new();
    for path in [one, two, three] {
        let made = value_of(&rig, "workspace/create", json!({ "path": path })).await;
        ids.push(made["workspace"]["workspaceId"].as_str().unwrap().to_string());
    }
    let order = value_of(
        &rig,
        "workspace/insertBefore",
        json!({ "workspaceId": ids[2], "beforeWorkspaceId": ids[0] }),
    )
    .await;
    assert_eq!(order["workspaceIds"], json!([ids[2], ids[0], ids[1]]));
    let order = value_of(&rig, "workspace/insertBefore", json!({ "workspaceId": ids[2] })).await;
    assert_eq!(order["workspaceIds"], json!([ids[0], ids[1], ids[2]]));
    assert_eq!(
        code_of(&rig, "workspace/insertBefore", json!({ "workspaceId": ids[0], "beforeWorkspaceId": "ghost" })).await,
        "workspace/not-found"
    );
    assert_eq!(
        code_of(
            &rig,
            "workspace/insertSessionBefore",
            json!({ "workspaceId": ids[0], "sessionId": "not-there" })
        )
        .await,
        "workspace/move-invalid"
    );
}

#[tokio::test]
async fn pinning_and_archiving_follow_the_registry_rules() {
    let rig = rig(vec![]).await;
    let one = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    let two = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();

    let pins = value_of(&rig, "workspace/pinSession", json!({ "sessionId": one })).await;
    assert_eq!(pins["pinnedSessionIds"], json!([one]));
    let pins = value_of(&rig, "workspace/pinSession", json!({ "sessionId": two })).await;
    assert_eq!(pins["pinnedSessionIds"], json!([two, one]), "most recent first");
    assert_eq!(
        code_of(&rig, "workspace/pinSession", json!({ "sessionId": "ghost" })).await,
        "session/not-found"
    );

    let archived = value_of(&rig, "workspace/archiveSession", json!({ "sessionId": one })).await;
    assert_eq!(archived["archivedSessionIds"], json!([one]));
    let pins = value_of(&rig, "workspace/unpinSession", json!({ "sessionId": two })).await;
    assert_eq!(pins["pinnedSessionIds"], json!([]), "archiving already unpinned the first");
    assert_eq!(
        code_of(&rig, "workspace/pinSession", json!({ "sessionId": one })).await,
        "gateway/bad-request",
        "an archived session cannot be pinned"
    );
    let archived = value_of(&rig, "workspace/unarchiveSession", json!({ "sessionId": one })).await;
    assert_eq!(archived["archivedSessionIds"], json!([]));
    let again = value_of(&rig, "workspace/unarchiveSession", json!({ "sessionId": one })).await;
    assert_eq!(again["archivedSessionIds"], json!([]), "unarchiving twice is not an error");
    assert_eq!(
        code_of(&rig, "workspace/archiveSession", json!({ "sessionId": "ghost" })).await,
        "session/not-found"
    );
}

#[tokio::test]
async fn the_default_workspace_is_created_only_when_one_is_configured() {
    let rig = rig(vec![]).await;
    let none = call(&rig, "workspace/initializeDefault", json!({})).await;
    assert_eq!(none["result"]["ok"], true);
    assert_eq!(none["result"]["value"], Value::Null);

    let parent = tempfile::tempdir().unwrap();
    let target = parent.path().join("default-workspace");
    let registry = dsh_rs::gui::workspaces::WorkspaceRegistry::new(None, Some(target.clone())).unwrap();
    let made = registry.initialize_default(&[]).unwrap().expect("configured");
    assert_eq!(made["title"], "default-workspace");
    assert!(target.is_dir());
    let again = registry.initialize_default(&[]).unwrap().unwrap();
    assert_eq!(again["workspaceId"], made["workspaceId"]);
}

#[tokio::test]
async fn the_registry_persists_and_a_corrupt_file_is_refused_not_replaced() {
    let store = tempfile::tempdir().unwrap();
    let file = store.path().join("workspaces.json");
    let (_keep, path) = directory("proj");
    let first = dsh_rs::gui::workspaces::WorkspaceRegistry::new(Some(file.clone()), None).unwrap();
    let (made, _) = first.create(&path, &[]).unwrap();
    first.rename(made["workspaceId"].as_str().unwrap(), "Saved", &[]).unwrap();
    drop(first);

    let reopened = dsh_rs::gui::workspaces::WorkspaceRegistry::new(Some(file.clone()), None).unwrap();
    let state = reopened.baseline(&[]);
    assert_eq!(state["items"][0]["title"], "Saved");
    assert_eq!(state["items"][0]["workspaceId"], made["workspaceId"]);

    std::fs::write(&file, "{ not json").unwrap();
    let error = dsh_rs::gui::workspaces::WorkspaceRegistry::new(Some(file.clone()), None)
        .expect_err("a corrupt registry is reported");
    assert!(error.contains("workspaces.json"), "{error}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "{ not json", "the file is left alone");
}

#[tokio::test]
async fn a_failed_save_leaves_the_registry_as_it_was() {
    let store = tempfile::tempdir().unwrap();
    let file = store.path().join("w.json");
    let registry = dsh_rs::gui::workspaces::WorkspaceRegistry::new(Some(file.clone()), None).unwrap();
    // A directory now sits where the registry file belongs, so the save cannot
    // replace it and the mutation must be refused without taking effect.
    std::fs::create_dir_all(file.join("occupied")).unwrap();
    let (_keep, path) = directory("proj");
    let error = registry.create(&path, &[]).expect_err("the save fails");
    assert_eq!(error.code, "gateway/internal");
    assert_eq!(registry.baseline(&[])["items"], json!([]), "nothing was committed");
}

// --- settings -----------------------------------------------------------

fn ack(version: &str) -> Value {
    json!({ "ns": "ui-settings-general", "ops": [{ "op": "set", "path": ["welcomeNoticeVersion"], "value": version }] })
}

async fn settings_call(rig: &Rig, method: &str, args: Value) -> Value {
    call(rig, method, args).await["result"].clone()
}

#[tokio::test]
async fn acknowledging_the_welcome_notice_is_stored_and_visible_to_the_next_read() {
    let rig = rig(vec![]).await;
    let before = settings_call(&rig, "settings/describe", json!({})).await;
    assert_eq!(before["value"]["namespaces"][0]["revision"], 0);
    assert_eq!(before["value"]["namespaces"][0]["user"], json!({}));

    let wrote = settings_call(&rig, "settings/mutate", ack("v1")).await;
    assert_eq!(wrote["ok"], true, "{wrote}");
    assert_eq!(wrote["value"]["revision"], 1);
    assert_eq!(wrote["value"]["value"]["welcomeNoticeVersion"], "v1");
    assert_eq!(wrote["value"]["secrets"], json!([]));
    assert!(wrote["value"]["schema"]["refs"].is_object(), "the client rehydrates this");

    let after = settings_call(&rig, "settings/describe", json!({})).await;
    assert_eq!(after["value"]["namespaces"][0]["user"]["welcomeNoticeVersion"], "v1");

    let unset = settings_call(
        &rig,
        "settings/mutate",
        json!({ "ns": "ui-settings-general", "ops": [{ "op": "unset", "path": ["welcomeNoticeVersion"] }] }),
    )
    .await;
    assert_eq!(unset["value"]["user"], json!({}));
    assert_eq!(unset["value"]["revision"], 2);
}

#[tokio::test]
async fn a_stale_revision_is_a_conflict_and_changes_nothing() {
    let rig = rig(vec![]).await;
    settings_call(&rig, "settings/mutate", ack("v1")).await;
    let mut stale = ack("v2");
    stale["expectedRevision"] = json!(0);
    let result = settings_call(&rig, "settings/mutate", stale).await;
    assert_eq!(result["ok"], false);
    assert_eq!(result["error"]["code"], "settings/conflict");
    assert_eq!(result["error"]["details"], json!({ "ns": "ui-settings-general", "expected": 0, "actual": 1 }));
    let mut current = ack("v2");
    current["expectedRevision"] = json!(1);
    assert_eq!(settings_call(&rig, "settings/mutate", current).await["value"]["revision"], 2);
}

#[tokio::test]
async fn settings_writes_refuse_unknown_namespaces_fields_and_types() {
    let rig = rig(vec![]).await;
    let cases = [
        json!({ "ns": "llm-deepseek", "ops": [] }),
        json!({ "ns": "ui-settings-general", "ops": [{ "op": "set", "path": ["other"], "value": "x" }] }),
        json!({ "ns": "ui-settings-general", "ops": [{ "op": "set", "path": ["welcomeNoticeVersion"], "value": 3 }] }),
        json!({ "ns": "ui-settings-general", "ops": [{ "op": "set", "path": ["welcomeNoticeVersion", "deep"], "value": "x" }] }),
    ];
    for case in cases {
        let result = settings_call(&rig, "settings/mutate", case.clone()).await;
        assert_eq!(result["error"]["code"], "settings/rejected", "{case}");
    }
    let after = settings_call(&rig, "settings/describe", json!({})).await;
    assert_eq!(after["value"]["namespaces"][0]["revision"], 0, "no refusal changed anything");
    // An op outside the wire vocabulary fails the gateway, as in TS.
    let unknown_op = settings_call(&rig, "settings/mutate", json!({ "ns": "ui-settings-general", "ops": [{ "op": "rename", "path": ["welcomeNoticeVersion"] }] })).await;
    assert_eq!(unknown_op["error"]["code"], "gateway/input-invalid");
    assert_eq!(unknown_op["error"]["details"]["field"], "ops");
    let malformed = settings_call(&rig, "settings/mutate", json!({ "ops": [] })).await;
    assert_eq!(malformed["error"]["code"], "gateway/arguments-invalid");
}

#[tokio::test]
async fn update_merges_and_replace_resets_the_user_section() {
    let rig = rig(vec![]).await;
    let updated = settings_call(
        &rig,
        "settings/update",
        json!({ "ns": "ui-settings-general", "patch": { "welcomeNoticeVersion": "a" } }),
    )
    .await;
    assert_eq!(updated["value"]["user"], json!({ "welcomeNoticeVersion": "a" }));
    let replaced = settings_call(
        &rig,
        "settings/replace",
        json!({ "ns": "ui-settings-general", "section": {} }),
    )
    .await;
    assert_eq!(replaced["value"]["user"], json!({}));
    let same = settings_call(
        &rig,
        "settings/replace",
        json!({ "ns": "ui-settings-general", "section": {} }),
    )
    .await;
    assert_eq!(same["value"]["revision"], replaced["value"]["revision"], "a no-op write is not a new revision");
}

#[tokio::test]
async fn settings_persist_and_a_bad_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    let first = dsh_rs::gui::settings::SettingsStore::new(Some(file.clone())).unwrap();
    first
        .mutate("ui-settings-general", &[json!({ "op": "set", "path": ["welcomeNoticeVersion"], "value": "keep" })], None)
        .unwrap();
    drop(first);
    let reopened = dsh_rs::gui::settings::SettingsStore::new(Some(file.clone())).unwrap();
    assert_eq!(
        reopened.describe()["namespaces"][0]["user"]["welcomeNoticeVersion"],
        "keep"
    );
    // The revision survives a restart, so a client holding an old one still conflicts.
    let revision = reopened.describe()["namespaces"][0]["revision"].clone();
    assert_eq!(revision, 1);
    let stale = reopened.mutate("ui-settings-general", &[], Some(0)).unwrap_err();
    assert_eq!(stale.code, "settings/conflict");
    for bad in [
        "{ nope",
        "[]",
        "null",
        r#"{"other":{}}"#,
        r#"{"ui-settings-general":{"surprise":"x"}}"#,
        r#"{"ui-settings-general":{"welcomeNoticeVersion":42}}"#,
        r#"{"ui-settings-general":[]}"#,
        r#"{"revision":-1}"#,
    ] {
        std::fs::write(&file, bad).unwrap();
        assert!(dsh_rs::gui::settings::SettingsStore::new(Some(file.clone())).is_err(), "{bad}");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), bad, "the file is left alone");
    }
}

#[tokio::test]
async fn projections_report_only_what_a_live_session_can_derive() {
    let rig = rig(vec![]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(projected["kind"], "sequenced");
    assert_eq!(projected["values"]["modelSelection"]["next"]["model"], "m");
    assert_eq!(projected["values"]["inbox"], json!({ "next-turn": [], "next-step": [] }));
    assert!(projected["values"].get("tokenUsage").is_none(), "unknown values are omitted, not zeroed");
    let unknown = value_of(&rig, "session/projections", json!({ "sessionId": "ghost" })).await;
    assert_eq!(unknown, Value::Null);
    let providers = call(&rig, "llm/listConfigurableProviders", json!({})).await;
    assert_eq!(providers["result"]["value"], json!([]));
}

#[tokio::test]
async fn asking_for_a_blank_session_again_reuses_it_but_a_used_one_gets_a_fresh_id() {
    let rig = rig(vec!["hi".into()]).await;
    let first = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    let again = value_of(&rig, "session/create", json!({ "sessionId": first })).await;
    assert_eq!(again["sessionId"], first.as_str(), "a blank session is handed back");

    let prompted = call(
        &rig,
        "session/prompt",
        json!({ "request": { "sessionId": first, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "x" }] } }),
    )
    .await;
    assert_eq!(prompted["result"]["ok"], true, "{prompted}");
    // Wait for the user message to land so the session is no longer blank.
    for _ in 0..50 {
        let listed = value_of(&rig, "session/list", json!({})).await;
        if listed["items"][0]["blank"] == false {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let fresh = value_of(&rig, "session/create", json!({ "sessionId": first })).await;
    assert_ne!(fresh["sessionId"], first.as_str(), "a used session is never reused");
}

#[tokio::test]
async fn skills_are_listed_from_the_installed_catalog_and_reminder_lists_are_empty() {
    let rig = rig(vec![]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    let none = value_of(&rig, "skills/list", json!({ "sessionId": session })).await;
    assert_eq!(none, json!({ "skills": [] }), "nothing installed in the test profile");

    let skill = dsh_rs::plugins::skills::Skill {
        name: "tidy".into(),
        description: "Tidy things".into(),
        dir: "/skills/tidy".into(),
    };
    let _held = rig
        .ctx
        .provide(
            dsh_rs::api::SKILLS_SERVICE,
            dsh_rs::plugins::skills::SkillCatalog { skills: vec![skill] },
        )
        .await
        .unwrap();
    let listed = value_of(&rig, "skills/list", json!({ "sessionId": session })).await;
    assert_eq!(listed["skills"][0]["name"], "tidy");
    assert_eq!(listed["skills"][0]["path"], "/skills/tidy/SKILL.md");
    assert_eq!(listed["skills"][0]["modelInvocable"], true);

    let bad = call(&rig, "skills/list", json!({ "request": {} })).await;
    assert_eq!(bad["result"]["error"]["code"], "gateway/input-invalid");
    for (endpoint, args) in [
        ("schedule/catalog", json!({})),
        ("schedule/list", json!({ "request": { "sessionId": session } })),
    ] {
        let reply = call(&rig, endpoint, args).await;
        assert_eq!(reply["result"]["value"], json!([]), "{endpoint}: {reply}");
    }
    // The test profile loads neither the compaction nor the goal plugin, so
    // only the commands that need no plugin are offered.
    let commands = call(&rig, "commands/list", json!({ "agentId": session })).await["result"]["value"].clone();
    let names: Vec<&str> = commands.as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["feedback", "goal"]);
}

#[tokio::test]
async fn a_blank_session_in_another_directory_is_not_handed_back() {
    let rig = rig(vec![]).await;
    let dir = tempfile::tempdir().unwrap();
    let here = dir.path().canonicalize().unwrap().display().to_string();
    let first = value_of(&rig, "session/create", json!({ "cwd": here })).await["sessionId"].as_str().unwrap().to_string();
    let same = value_of(&rig, "session/create", json!({ "sessionId": first, "cwd": here })).await;
    assert_eq!(same["sessionId"], first.as_str());
    let elsewhere = tempfile::tempdir().unwrap();
    let other = elsewhere.path().canonicalize().unwrap().display().to_string();
    let moved = value_of(&rig, "session/create", json!({ "sessionId": first, "cwd": other })).await;
    assert_ne!(moved["sessionId"], first.as_str(), "it would run in the wrong directory");
}

#[tokio::test]
async fn renaming_a_session_logs_a_title_the_projection_and_the_ts_reader_see() {
    let rig = rig(vec![]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    let before = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(before["values"]["title"], Value::Null);

    let renamed = value_of(
        &rig,
        "session/rename",
        json!({ "sessionId": session, "title": "  My \x1b[31mplan\x1b[0m\u{202E}  " }),
    )
    .await;
    assert_eq!(renamed["title"], "My plan", "cleaned like the TS normaliser");
    let seq = renamed["seq"].as_u64().unwrap();

    let after = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(after["values"]["title"], "My plan");

    // The wire record is the v4 row the TypeScript reader folds, source `user`.
    let page = value_of(
        &rig,
        "session/page",
        json!({ "address": { "kind": "session", "sessionId": session } }),
    )
    .await;
    let row = page["records"].as_array().unwrap().iter().find(|r| r["event"]["seq"] == seq).unwrap()["event"].clone();
    assert_eq!(row["type"], "session/title");
    assert_eq!(row["data"]["source"], json!({ "kind": "user" }));
    assert_eq!(row["data"]["messageSeqs"], json!([]));
}

#[tokio::test]
async fn a_rename_that_leaves_nothing_visible_or_names_no_session_is_refused() {
    let rig = rig(vec![]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    let invisible = json!({ "sessionId": session, "title": " \u{200B}\x07 " });
    assert_eq!(code_of(&rig, "session/rename", invisible).await, "session/title-invalid");
    let ghost = json!({ "sessionId": "ghost", "title": "x" });
    assert_eq!(code_of(&rig, "session/rename", ghost).await, "session/not-found");
    let long = "é".repeat(200);
    let cut = value_of(&rig, "session/rename", json!({ "sessionId": session, "title": long })).await;
    assert!(cut["title"].as_str().unwrap().len() <= 120, "cut to the configured byte budget");
}

async fn finished_turn(rig: &Rig, session: &str, text: &str) {
    let prompted = call(
        rig,
        "session/prompt",
        json!({ "request": { "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": text }] } }),
    )
    .await;
    assert_eq!(prompted["result"]["ok"], true, "{prompted}");
    let agents = (*rig.ctx.require::<AgentRegistryService>("agents").unwrap()).clone();
    agents.get(session).unwrap().when_idle().await;
}

fn user_texts(records: &Value) -> Vec<String> {
    records["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["event"]["type"] == "user/message")
        .map(|r| r["event"]["data"]["content"][0]["text"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn a_fork_is_a_new_live_session_holding_the_source_up_to_the_cut() {
    let rig = rig(vec!["one", "two", "three"]).await;
    let source = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    finished_turn(&rig, &source, "first question").await;
    finished_turn(&rig, &source, "second question").await;

    // Default cut: the latest completed turn, so both questions are kept.
    let forked = value_of(&rig, "session/fork", json!({ "sessionId": source })).await;
    let child = forked["sessionId"].as_str().unwrap().to_string();
    assert_ne!(child, source);

    let address = |id: &str| json!({ "address": { "kind": "session", "sessionId": id } });
    let child_page = value_of(&rig, "session/page", address(&child)).await;
    assert_eq!(user_texts(&child_page), ["first question", "second question"]);

    // The child is a live agent: it takes a prompt of its own and the source is untouched.
    finished_turn(&rig, &child, "only in the fork").await;
    let child_page = value_of(&rig, "session/page", address(&child)).await;
    assert_eq!(user_texts(&child_page).last().unwrap(), "only in the fork");
    let source_page = value_of(&rig, "session/page", address(&source)).await;
    assert_eq!(user_texts(&source_page), ["first question", "second question"]);

    // An explicit cut after the first turn drops the second question.
    let first_turn_end = source_page["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["event"]["type"] == "turn/end")
        .unwrap()["event"]["seq"]
        .as_u64()
        .unwrap();
    let early = value_of(&rig, "session/fork", json!({ "sessionId": source, "atSeq": first_turn_end })).await;
    let early_page = value_of(&rig, "session/page", address(early["sessionId"].as_str().unwrap())).await;
    assert_eq!(user_texts(&early_page), ["first question"]);
}

#[tokio::test]
async fn a_fork_that_cannot_be_made_says_why() {
    let rig = rig(vec!["one"]).await;
    let source = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    assert_eq!(
        code_of(&rig, "session/fork", json!({ "sessionId": source })).await,
        "session/fork-unavailable",
        "a blank session has no completed turn"
    );
    assert_eq!(
        code_of(&rig, "session/fork", json!({ "sessionId": "ghost" })).await,
        "session/not-found"
    );
    assert_eq!(
        code_of(&rig, "session/fork", json!({ "sessionId": source, "atSeq": -1 })).await,
        "gateway/bad-request"
    );
    assert_eq!(
        code_of(&rig, "session/fork", json!({ "sessionId": source, "atSeq": 9007199254740992u64 })).await,
        "gateway/bad-request",
        "a seq a JavaScript client cannot read back exactly"
    );
    finished_turn(&rig, &source, "hello").await;
    assert_eq!(
        code_of(&rig, "session/fork", json!({ "sessionId": source, "atSeq": 9999 })).await,
        "session/fork-unavailable",
        "a seq past the end of the log"
    );
    // A cut inside the first (completed) turn but before its end would start
    // the child mid-turn.
    let page = value_of(
        &rig,
        "session/page",
        json!({ "address": { "kind": "session", "sessionId": source } }),
    )
    .await;
    let inside = page["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["event"]["type"] == "step/start")
        .expect("the first step")
        ["event"]["seq"]
        .as_u64()
        .unwrap();
    assert_eq!(
        code_of(&rig, "session/fork", json!({ "sessionId": source, "atSeq": inside })).await,
        "session/fork-unavailable",
        "a cut inside a turn would start the child mid-turn"
    );
}

fn route(provider: &str, model: &str) -> DefaultModel {
    DefaultModel { provider: provider.to_string(), model: model.to_string() }
}

#[tokio::test]
async fn a_selected_model_is_used_by_the_next_request_and_shown_by_the_projection() {
    let rig = rig_with_models(vec!["a", "b"], vec![route("mock", "big")]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();

    // The catalog offers both routes under the one registered provider.
    let catalog = value_of(&rig, "session/modelCatalog", json!({})).await;
    let ids: Vec<&str> = catalog["groups"][0]["models"].as_array().unwrap().iter().map(|m| m["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["m", "big"]);

    finished_turn(&rig, &session, "first").await;
    let agents = rig.ctx.get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE).unwrap();
    let before = agents.get(&session).unwrap().session().events().last().unwrap().seq;
    let chosen = value_of(&rig, "session/selectModel", json!({ "sessionId": session, "provider": "mock", "model": "big" })).await;
    let event = agents.get(&session).unwrap().session().events().last().unwrap().clone();
    assert_eq!(event.seq, before + 1, "a projection update must advance its watermark");
    assert!(matches!(event.data, dsh_rs::types::SessionEventData::ModelSelection { ref provider, ref model } if provider == "mock" && model == "big"));
    assert_eq!(chosen, json!({ "selected": { "provider": "mock", "model": "big" } }));
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(projected["values"]["modelSelection"]["next"], json!({ "provider": "mock", "model": "big" }));
    assert_eq!(projected["values"]["modelSelection"]["lastUsed"]["model"], "m");

    finished_turn(&rig, &session, "second").await;
    assert_eq!(*rig.routes.lock().unwrap(), ["mock/m", "mock/big"], "the change applies from the next request");
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(projected["values"]["modelSelection"]["lastUsed"]["model"], "big");

    // Another session keeps the default.
    let other = value_of(&rig, "session/create", json!({ "cwd": "/tmp" })).await["sessionId"].as_str().unwrap().to_string();
    let other_view = value_of(&rig, "session/projections", json!({ "sessionId": other })).await;
    assert_eq!(other_view["values"]["modelSelection"]["next"]["model"], "m");
}

#[tokio::test]
async fn a_model_that_is_not_offered_or_not_routable_is_refused_and_changes_nothing() {
    let rig = rig_with_models(vec![], vec![route("ghost", "x")]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    let not_offered = json!({ "sessionId": session, "provider": "mock", "model": "nope" });
    assert_eq!(code_of(&rig, "session/selectModel", not_offered).await, "session/model-unavailable");
    // Offered by the profile but its provider is not registered.
    let unroutable = json!({ "sessionId": session, "provider": "ghost", "model": "x" });
    assert_eq!(code_of(&rig, "session/selectModel", unroutable).await, "session/model-unavailable");
    let effort = json!({ "sessionId": session, "provider": "mock", "model": "m", "reasoningEffort": "high" });
    assert_eq!(code_of(&rig, "session/selectModel", effort).await, "session/model-unavailable", "an unapplied setting is refused, not ignored");
    let unknown = json!({ "sessionId": "ghost", "provider": "mock", "model": "m" });
    assert_eq!(code_of(&rig, "session/selectModel", unknown).await, "session/not-found");
    assert_eq!(code_of(&rig, "session/selectModel", json!({ "sessionId": session })).await, "gateway/input-invalid");
    let view = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(view["values"]["modelSelection"]["next"]["model"], "m", "the refusals left the model alone");
    let catalog = value_of(&rig, "session/modelCatalog", json!({})).await;
    assert_eq!(catalog["routableProviders"], json!(["mock"]), "an unregistered provider is not offered");
}

#[tokio::test]
async fn a_model_failure_ends_the_turn_with_its_reason_and_the_session_still_works() {
    let rig = rig(vec!["!quota exhausted", "recovered"]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    finished_turn(&rig, &session, "first").await;

    let address = json!({ "address": { "kind": "session", "sessionId": session } });
    let page = value_of(&rig, "session/page", address.clone()).await;
    let ends: Vec<&Value> = page["records"].as_array().unwrap().iter()
        .map(|r| &r["event"]).filter(|e| e["type"] == "turn/end").collect();
    assert_eq!(ends.len(), 1, "the failed turn is closed, not left open");
    assert_eq!(ends[0]["data"]["reason"]["kind"], "error");
    assert!(ends[0]["data"]["reason"]["error"]["message"].as_str().unwrap().contains("quota exhausted"));

    // Closed means the session accepts more work and can be forked at its end.
    finished_turn(&rig, &session, "second").await;
    let page = value_of(&rig, "session/page", address).await;
    let kinds: Vec<&str> = page["records"].as_array().unwrap().iter()
        .filter(|r| r["event"]["type"] == "turn/end")
        .map(|r| r["event"]["data"]["reason"]["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["error", "completed"]);
}

/// Wait until the agent owns a running turn, so a prompt queues behind it.
async fn wait_until_busy(rig: &Rig, session: &str) {
    let agents = (*rig.ctx.require::<AgentRegistryService>("agents").unwrap()).clone();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let busy = agents
            .get(session)
            .map(|agent| agent.driver_busy())
            .unwrap_or(false);
        if busy {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the agent should be running a turn"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

/// Wait until the model has answered `count` requests.
async fn wait_for_routes(rig: &Rig, count: usize) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if rig.routes.lock().unwrap().len() >= count {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the model should have answered {count} requests"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

/// The pending item ids of one inbox list, as the client sees them.
async fn pending_ids(rig: &Rig, session: &str, list: &str) -> Vec<String> {
    let projected = value_of(rig, "session/projections", json!({ "sessionId": session })).await;
    projected["values"]["inbox"][list]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value["id"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn a_queued_item_can_be_edited_and_removed_while_a_turn_runs() {
    let gate = Arc::new(tokio::sync::Notify::new());
    let rig = rig_gated(vec!["@first", "second"], Vec::new(), Some(gate.clone())).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();

    // The first turn waits on the gate, so the agent stays busy with it.
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "first" }]
    })).await;
    wait_until_busy(&rig, &session).await;

    // The second prompt queues behind the running turn.
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "second" }]
    })).await;
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    let pending = projected["values"]["inbox"]["next-turn"].as_array().unwrap();
    assert_eq!(pending.len(), 1, "the second prompt waits in the turn list");
    let item_id = pending[0]["id"].as_str().unwrap().to_string();

    // Editing replaces the pending text and keeps the identity.
    let edited = value_of(&rig, "session/updateQueue", json!({
        "sessionId": session, "itemId": item_id,
        "action": { "kind": "edit", "content": [{ "type": "text", "text": "edited" }] }
    })).await;
    assert_eq!(edited, json!({ "accepted": true }));
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    let pending = &projected["values"]["inbox"]["next-turn"][0];
    assert_eq!(pending["id"], item_id.as_str(), "an edit keeps the identity");
    assert_eq!(pending["content"][0]["text"], "edited");

    // Removing takes the item out of the pending list.
    let removed = value_of(&rig, "session/updateQueue", json!({
        "sessionId": session, "itemId": item_id, "action": { "kind": "remove" }
    })).await;
    assert_eq!(removed, json!({ "accepted": true }));
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert!(
        projected["values"]["inbox"]["next-turn"].as_array().unwrap().is_empty(),
        "a removed item is no longer pending"
    );

    // A vanished item is reported so the client drops the row silently.
    assert_eq!(
        code_of(&rig, "session/updateQueue", json!({
            "sessionId": session, "itemId": item_id, "action": { "kind": "remove" }
        })).await,
        "session/queue-item-not-found"
    );
    gate.notify_one();
}

#[tokio::test]
async fn steering_moves_a_queued_item_into_the_running_turn() {
    let gate = Arc::new(tokio::sync::Notify::new());
    let rig = rig_gated(vec!["@first"], Vec::new(), Some(gate.clone())).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "first" }]
    })).await;
    wait_until_busy(&rig, &session).await;
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "second" }]
    })).await;
    let item_id = pending_ids(&rig, &session, "next-turn")
        .await
        .first()
        .cloned()
        .expect("a queued item");

    let steered = value_of(&rig, "session/updateQueue", json!({
        "sessionId": session, "itemId": item_id, "action": { "kind": "steer" }
    })).await;
    assert_eq!(steered, json!({ "accepted": true }));
    assert!(
        pending_ids(&rig, &session, "next-turn").await.is_empty(),
        "the steered item leaves the turn list"
    );
    assert_eq!(
        pending_ids(&rig, &session, "next-step").await,
        vec![item_id],
        "the steered item joins the step list the running turn drains"
    );
    gate.notify_one();
}

#[tokio::test]
async fn a_queue_edit_refuses_content_it_cannot_apply() {
    let gate = Arc::new(tokio::sync::Notify::new());
    let rig = rig_gated(vec!["@first"], Vec::new(), Some(gate.clone())).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "first" }]
    })).await;
    wait_until_busy(&rig, &session).await;
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "second" }]
    })).await;
    let item_id = pending_ids(&rig, &session, "next-turn")
        .await
        .first()
        .cloned()
        .expect("a queued item");

    let image = json!({
        "sessionId": session, "itemId": item_id,
        "action": { "kind": "edit", "content": [{ "type": "image", "mediaType": "image/png" }] }
    });
    // The wire type admits text blocks only, so TS refuses an image block at its
    // boundary before any queue rule runs.
    assert_eq!(
        code_of(&rig, "session/updateQueue", image).await,
        "gateway/input-invalid",
        "a queue edit carries text only"
    );
    let blank = json!({
        "sessionId": session, "itemId": item_id,
        "action": { "kind": "edit", "content": [{ "type": "text", "text": "   " }] }
    });
    assert_eq!(
        code_of(&rig, "session/updateQueue", blank).await,
        "gateway/bad-request",
        "an empty edit is refused"
    );
    // The refusals left the item pending and unchanged.
    assert_eq!(pending_ids(&rig, &session, "next-turn").await, vec![item_id]);
    gate.notify_one();
}

#[tokio::test]
async fn a_prompt_keeps_its_client_identity_and_a_retry_is_not_queued_twice() {
    let gate = Arc::new(tokio::sync::Notify::new());
    let rig = rig_gated(vec!["@first"], Vec::new(), Some(gate.clone())).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "first" }]
    })).await;
    wait_until_busy(&rig, &session).await;

    let prompt = json!({
        "sessionId": session, "requestId": "req-1", "mode": "queue",
        "content": [{ "type": "text", "text": "second" }]
    });
    value_of(&rig, "session/prompt", prompt.clone()).await;
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    let row = &projected["values"]["inbox"]["next-turn"][0];
    assert_eq!(row["source"]["kind"], "user");
    assert_eq!(row["source"]["rpcId"], "req-1", "the queued row carries the client identity");

    // The same identity again is accepted, but the row is not duplicated.
    let again = value_of(&rig, "session/prompt", prompt).await;
    assert_eq!(again, json!({ "accepted": true }));
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(
        projected["values"]["inbox"]["next-turn"].as_array().unwrap().len(),
        1,
        "a retried prompt is not queued twice"
    );

    // An empty identity is refused rather than silently accepted.
    assert_eq!(
        code_of(&rig, "session/prompt", json!({
            "sessionId": session, "mode": "queue", "requestId": "",
            "content": [{ "type": "text", "text": "third" }]
        })).await,
        "gateway/input-invalid"
    );
    gate.notify_one();
}

#[tokio::test]
async fn every_queued_prompt_runs_in_its_own_turn() {
    let gate = Arc::new(tokio::sync::Notify::new());
    let rig = rig_gated(vec!["@first", "second", "third"], Vec::new(), Some(gate.clone())).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "one" }]
    })).await;
    wait_until_busy(&rig, &session).await;

    // Two prompts queue behind the running turn.
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "two" }]
    })).await;
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "three" }]
    })).await;
    gate.notify_one();

    // Each queued prompt runs in its own turn; none is stranded.
    wait_for_routes(&rig, 3).await;
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(
        projected["values"]["inbox"]["next-turn"].as_array().unwrap().len(),
        0,
        "the queue drains"
    );
}

#[tokio::test]
async fn two_prompts_with_one_identity_are_queued_once() {
    let gate = Arc::new(tokio::sync::Notify::new());
    let rig = rig_gated(vec!["@first"], Vec::new(), Some(gate.clone())).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "first" }]
    })).await;
    wait_until_busy(&rig, &session).await;

    // Two retries of one client identity arrive together; only one of them may
    // reach the queue.
    let prompt = json!({
        "sessionId": session, "requestId": "req-race", "mode": "queue",
        "content": [{ "type": "text", "text": "second" }]
    });
    let (first, second) = tokio::join!(
        call(&rig, "session/prompt", json!({ "request": prompt.clone() })),
        call(&rig, "session/prompt", json!({ "request": prompt.clone() })),
    );
    assert_eq!(first["result"]["ok"], true, "{first}");
    assert_eq!(second["result"]["ok"], true, "{second}");

    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(
        projected["values"]["inbox"]["next-turn"].as_array().unwrap().len(),
        1,
        "one client identity, one queued row"
    );
    gate.notify_one();
}

/// A retry that lands after the driver claimed its prompt but before the
/// prompt is logged must not queue a second turn.
#[tokio::test]
async fn a_retry_that_lands_before_the_prompt_is_logged_is_not_queued_twice() {
    let rig = rig(vec!["ok"]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();

    // The first turn is held after the driver claimed its prompt and before
    // the prompt is written to the log.
    let hold = Arc::new(tokio::sync::Notify::new());
    let first = Arc::new(AtomicBool::new(true));
    let listener_hold = hold.clone();
    let listener_first = first.clone();
    rig.ctx
        .on("agent/pre-step", move |_ctx, payload, next| {
            let hold = Arc::clone(&listener_hold);
            let first = Arc::clone(&listener_first);
            Box::pin(async move {
                if first.swap(false, Ordering::SeqCst) {
                    hold.notified().await;
                }
                next.run(payload).await
            }) as BoxFuture<cordis::Result<Value>>
        })
        .await
        .expect("the pre-step listener registers");

    let prompt = json!({
        "sessionId": session, "requestId": "req-window", "mode": "queue",
        "content": [{ "type": "text", "text": "only once" }]
    });
    value_of(&rig, "session/prompt", prompt.clone()).await;
    wait_until_busy(&rig, &session).await;

    // The retry arrives in the window between the claim and the log.
    let retry = value_of(&rig, "session/prompt", prompt).await;
    assert_eq!(retry["accepted"], true, "{retry}");
    hold.notify_one();

    wait_for_routes(&rig, 1).await;
    // A second turn would start right after the first one ends.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert_eq!(
        rig.routes.lock().unwrap().len(),
        1,
        "the retried prompt did not open a second turn"
    );
}

/// A prompt with no non-whitespace text is refused the way the TypeScript
/// controller refuses it.
#[tokio::test]
async fn a_prompt_without_any_text_is_refused() {
    let rig = rig(vec!["ok"]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    let code = code_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "   " }]
    })).await;
    assert_eq!(code, "gateway/bad-request", "{code}");
    let code = code_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": []
    })).await;
    assert_eq!(code, "gateway/bad-request", "{code}");
}

#[tokio::test]
async fn a_steered_prompt_is_queued_once_too() {
    let gate = Arc::new(tokio::sync::Notify::new());
    let rig = rig_gated(vec!["@first"], Vec::new(), Some(gate.clone())).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    value_of(&rig, "session/prompt", json!({
        "sessionId": session, "requestId": fresh_request_id(), "mode": "queue", "content": [{ "type": "text", "text": "first" }]
    })).await;
    wait_until_busy(&rig, &session).await;

    let prompt = json!({
        "sessionId": session, "requestId": "req-steer", "mode": "steer",
        "content": [{ "type": "text", "text": "steered" }]
    });
    value_of(&rig, "session/prompt", prompt.clone()).await;
    // The same identity again is accepted, but the row is not duplicated.
    value_of(&rig, "session/prompt", prompt).await;
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(
        projected["values"]["inbox"]["next-step"].as_array().unwrap().len(),
        1,
        "a retried steered prompt is not queued twice"
    );
    gate.notify_one();
}

#[tokio::test]
async fn steering_an_idle_session_is_refused_and_an_unknown_item_is_reported() {
    let rig = rig(vec![]).await;
    let session = value_of(&rig, "session/create", json!({})).await["sessionId"].as_str().unwrap().to_string();
    // Nothing is pending, so the item is already gone.
    assert_eq!(
        code_of(&rig, "session/updateQueue", json!({
            "sessionId": session, "itemId": "nope", "action": { "kind": "steer" }
        })).await,
        "session/queue-item-not-found"
    );
    assert_eq!(
        code_of(&rig, "session/updateQueue", json!({
            "sessionId": "ghost", "itemId": "nope", "action": { "kind": "remove" }
        })).await,
        "session/queue-item-not-found",
        "an unknown session reports the item as gone"
    );
    assert_eq!(
        code_of(&rig, "session/updateQueue", json!({
            "sessionId": session, "itemId": "nope"
        })).await,
        "gateway/input-invalid",
        "an action is required"
    );
}

/// The session reference candidates name the other sessions with a canonical
/// mention.
#[tokio::test]
async fn the_reference_candidates_name_the_other_sessions() {
    let rig = rig(Vec::new()).await;
    let first = call(&rig, "session/create", json!({ "request": { "cwd": "/tmp" } })).await;
    let first_id = first["result"]["value"]["sessionId"]
        .as_str()
        .expect("a session id")
        .to_string();
    let second = call(&rig, "session/create", json!({ "request": { "cwd": "/tmp" } })).await;
    let second_id = second["result"]["value"]["sessionId"]
        .as_str()
        .expect("a session id")
        .to_string();

    let response = call(
        &rig,
        "sessionReferenceResolver/candidates",
        json!({ "agentId": first_id, "query": "" }),
    )
    .await;
    assert_eq!(response["result"]["ok"], true, "{response}");
    let candidates = response["result"]["value"]
        .as_array()
        .expect("candidates");
    assert_eq!(candidates.len(), 1, "{response}");
    let candidate = &candidates[0];
    assert_eq!(candidate["sessionId"], second_id, "{response}");
    assert_eq!(candidate["sameWorkspace"], true, "{response}");
    assert_eq!(candidate["label"], second_id, "{response}");
    let mention = candidate["mention"].as_str().expect("a mention");
    assert!(mention.starts_with(&format!("@[{second_id}]")), "{mention}");
    assert!(mention.contains("dsh-session:"), "{mention}");

    // A query that names nothing offers nothing.
    let none = call(
        &rig,
        "sessionReferenceResolver/candidates",
        json!({ "agentId": first_id, "query": "no-such-session" }),
    )
    .await;
    assert_eq!(none["result"]["value"].as_array().expect("candidates").len(), 0, "{none}");
}

/// The candidates rank the caller's own workspace first, then the sessions
/// that recorded none, and a title keeps its brackets in the mention.
#[tokio::test]
async fn the_reference_candidates_rank_and_mention_the_way_the_host_does() {
    let rig = rig(Vec::new()).await;
    let agent = value_of(&rig, "session/create", json!({ "cwd": "/tmp" })).await;
    let agent_id = agent["sessionId"].as_str().unwrap().to_string();
    let same = value_of(&rig, "session/create", json!({ "cwd": "/tmp" })).await;
    let same_id = same["sessionId"].as_str().unwrap().to_string();
    let without = value_of(&rig, "session/create", json!({})).await;
    let without_id = without["sessionId"].as_str().unwrap().to_string();
    let other = value_of(&rig, "session/create", json!({ "cwd": "/elsewhere" })).await;
    let other_id = other["sessionId"].as_str().unwrap().to_string();

    // A title with brackets is carried into the mention unescaped.
    value_of(
        &rig,
        "session/rename",
        json!({ "sessionId": without_id, "title": "Plan (Q4)" }),
    )
    .await;

    let response = call(
        &rig,
        "sessionReferenceResolver/candidates",
        json!({ "agentId": agent_id, "query": "" }),
    )
    .await;
    assert_eq!(response["result"]["ok"], true, "{response}");
    let response = response["result"]["value"].clone();
    let candidates = response.as_array().expect("candidates");
    let order: Vec<&str> = candidates
        .iter()
        .map(|candidate| candidate["sessionId"].as_str().unwrap())
        .collect();
    assert_eq!(
        order,
        vec![same_id.as_str(), without_id.as_str(), other_id.as_str()],
        "{response}"
    );
    let without = candidates
        .iter()
        .find(|candidate| candidate["sessionId"] == without_id)
        .expect("the candidate");
    assert_eq!(without["label"], "Plan (Q4)", "{response}");
    assert_eq!(without["sameWorkspace"], false, "{response}");
    assert!(
        without["mention"]
            .as_str()
            .unwrap()
            .starts_with("@[Plan (Q4)](dsh-session:"),
        "{response}"
    );

    // A live session reports the time it was created, not the time of its
    // first event.
    let same = candidates
        .iter()
        .find(|candidate| candidate["sessionId"] == same_id)
        .expect("the candidate");
    assert!(
        same["createdAt"].as_u64().unwrap_or(0) > 0,
        "{response}"
    );
}

/// Call a goal endpoint and return the whole `result`.
async fn goal_call(rig: &Rig, method: &str, args: Value) -> Value {
    call(rig, method, args).await["result"].clone()
}

async fn goal_session(rig: &Rig) -> String {
    value_of(rig, "session/create", json!({})).await["sessionId"]
        .as_str()
        .unwrap()
        .to_string()
}

fn goal_ref(view: &Value) -> Value {
    json!({ "id": view["id"], "revision": view["revision"] })
}

#[tokio::test]
async fn a_session_without_a_goal_answers_with_no_value() {
    let rig = rig(vec![]).await;
    let session = goal_session(&rig).await;
    let result = goal_call(&rig, "goals/get", json!({ "agentId": session })).await;
    assert_eq!(result, json!({ "ok": true }), "TypeScript sends no value key for undefined");
    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(projected["values"]["goal"], Value::Null);
}

#[tokio::test]
async fn a_goal_runs_its_lifecycle_and_every_step_is_logged() {
    let rig = rig(vec![]).await;
    let session = goal_session(&rig).await;

    let created = goal_call(
        &rig,
        "goals/create",
        json!({ "agentId": session, "request": { "objective": "  ship it  ", "maxGoalRounds": 3 } }),
    )
    .await;
    assert_eq!(created["ok"], true, "{created}");
    assert_eq!(created["value"]["ref"]["revision"], 1);
    assert!(created["value"]["ref"]["id"].as_str().unwrap().starts_with("goal-"));

    let got = goal_call(&rig, "goals/get", json!({ "agentId": session })).await["value"].clone();
    assert_eq!(got["objective"], "ship it", "the objective is trimmed");
    assert_eq!(
        (got["phase"].as_str(), got["maxGoalRounds"].as_u64(), got["roundsStarted"].as_u64()),
        (Some("active"), Some(3), Some(0))
    );
    assert_eq!(got["activation"], "armed");
    assert_eq!(got["createdAt"], got["updatedAt"]);

    let edited = goal_call(
        &rig,
        "goals/edit",
        json!({ "agentId": session, "ref": goal_ref(&got), "request": { "objective": "ship it v2", "maxGoalRounds": 5 } }),
    )
    .await["value"]
        .clone();
    assert_eq!((edited["revision"].as_u64(), edited["maxGoalRounds"].as_u64()), (Some(2), Some(5)));
    assert_eq!(edited["activation"], "armed", "an edit keeps the activation");

    let paused = goal_call(&rig, "goals/pause", json!({ "agentId": session, "ref": goal_ref(&edited) })).await["value"].clone();
    assert_eq!((paused["phase"].as_str(), paused["activation"].as_str()), (Some("paused"), Some("disarmed")));

    let resumed = goal_call(&rig, "goals/resume", json!({ "agentId": session, "ref": goal_ref(&paused) })).await["value"].clone();
    assert_eq!((resumed["phase"].as_str(), resumed["activation"].as_str()), (Some("active"), Some("armed")));

    let done = goal_call(&rig, "goals/complete", json!({ "agentId": session, "ref": goal_ref(&resumed) })).await["value"].clone();
    assert_eq!((done["phase"].as_str(), done["revision"].as_u64()), (Some("complete"), Some(5)));
    assert!(done["updatedAt"].as_u64() >= done["createdAt"].as_u64());

    let projected = value_of(&rig, "session/projections", json!({ "sessionId": session })).await;
    assert_eq!(projected["values"]["goal"]["goal"]["phase"], "complete");
    assert!(
        projected["values"]["goal"].get("activation").is_none(),
        "activation is process-local and not projected"
    );

    let cleared = goal_call(&rig, "goals/clear", json!({ "agentId": session, "ref": goal_ref(&done) })).await;
    assert_eq!(cleared["value"]["revision"], 6, "the tombstone is one past the cleared snapshot");
    assert_eq!(goal_call(&rig, "goals/get", json!({ "agentId": session })).await, json!({ "ok": true }));

    let page = value_of(&rig, "session/page", json!({ "address": { "kind": "session", "sessionId": session } })).await;
    let operations: Vec<_> = page["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["event"]["type"] == "goal/change")
        .map(|record| record["event"]["data"]["operation"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(operations, ["create", "edit", "pause", "resume", "complete", "clear"]);
}

#[tokio::test]
async fn a_goal_mutation_is_refused_with_the_typescript_message() {
    let rig = rig(vec![]).await;
    let session = goal_session(&rig).await;
    let message = |result: &Value| result["error"]["message"].as_str().unwrap_or_default().to_string();
    let args = |request: Value| json!({ "agentId": session, "request": request });

    let none = goal_call(&rig, "goals/pause", json!({ "agentId": session, "ref": { "id": "g", "revision": 1 } })).await;
    assert_eq!((none["error"]["code"].as_str(), message(&none).as_str()), (Some("gateway/internal"), "no current goal"));

    let blank = goal_call(&rig, "goals/create", args(json!({ "objective": "   " }))).await;
    assert_eq!(message(&blank), "goal objective must be a non-empty string");
    for rounds in [json!(0), json!(1.5), json!(9_007_199_254_740_992u64)] {
        let bad = goal_call(&rig, "goals/create", args(json!({ "objective": "x", "maxGoalRounds": rounds }))).await;
        assert_eq!(message(&bad), "maxGoalRounds must be a positive safe integer", "{rounds}");
    }

    let created = goal_call(&rig, "goals/create", args(json!({ "objective": "x" }))).await["value"]["ref"].clone();
    let again = goal_call(&rig, "goals/create", args(json!({ "objective": "y" }))).await;
    assert_eq!(message(&again), format!("goal \"{}\" already exists with phase \"active\"", created["id"].as_str().unwrap()));

    let stale = goal_call(&rig, "goals/edit", json!({ "agentId": session, "ref": { "id": created["id"], "revision": 9 }, "request": { "objective": "z" } })).await;
    assert!(message(&stale).starts_with("stale goal ref"), "{stale}");
    let empty = goal_call(&rig, "goals/edit", json!({ "agentId": session, "ref": created, "request": {} })).await;
    assert_eq!(message(&empty), "goal edit requires objective and/or maxGoalRounds");

    let already = goal_call(&rig, "goals/resume", json!({ "agentId": session, "ref": created })).await;
    assert!(message(&already).ends_with("is already active and armed"), "{already}");
    let paused = goal_call(&rig, "goals/pause", json!({ "agentId": session, "ref": created })).await["value"].clone();
    let twice = goal_call(&rig, "goals/pause", json!({ "agentId": session, "ref": goal_ref(&paused) })).await;
    assert!(message(&twice).ends_with("from phase \"paused\"; expected active"), "{twice}");
}

#[tokio::test]
async fn a_completed_goal_can_be_replaced_and_an_omitted_cap_takes_the_default() {
    let rig = rig(vec![]).await;
    let session = goal_session(&rig).await;
    let first = goal_call(&rig, "goals/create", json!({ "agentId": session, "request": { "objective": "one" } })).await["value"]["ref"].clone();
    let done = goal_call(&rig, "goals/complete", json!({ "agentId": session, "ref": first })).await["value"].clone();
    let second = goal_call(&rig, "goals/create", json!({ "agentId": session, "request": { "objective": "two" } })).await;
    assert_eq!(second["ok"], true, "a completed goal may be replaced: {second}");
    assert_ne!(second["value"]["ref"]["id"], done["id"]);
    assert_eq!(
        goal_call(&rig, "goals/get", json!({ "agentId": session })).await["value"]["maxGoalRounds"],
        256,
        "an omitted cap takes the configured default"
    );
}

#[tokio::test]
async fn goal_arguments_are_checked_the_way_the_gateway_checks_them() {
    let rig = rig(vec![]).await;
    let session = goal_session(&rig).await;
    let fields = goal_call(&rig, "goals/edit", json!({})).await;
    assert_eq!(fields["error"]["code"], "gateway/arguments-invalid");
    assert_eq!(
        fields["error"]["message"],
        "typert gateway: goals/edit: args fields do not match the descriptor: missing \"agentId\", \"ref\", \"request\""
    );
    let extra = goal_call(&rig, "goals/get", json!({ "agentId": session, "agent": session })).await;
    assert!(extra["error"]["message"].as_str().unwrap().ends_with("unexpected \"agent\""), "{extra}");
    for request in [json!(5), json!("x"), json!({}), json!({ "objective": 5 })] {
        let bad = goal_call(&rig, "goals/create", json!({ "agentId": session, "request": request })).await;
        assert_eq!(bad["error"]["code"], "gateway/input-invalid", "{request}");
        assert_eq!(bad["error"]["details"], json!({ "endpoint": "goals/create", "field": "request" }));
    }
    let bad_ref = goal_call(&rig, "goals/pause", json!({ "agentId": session, "ref": "x" })).await;
    assert_eq!(bad_ref["error"]["details"]["field"], "ref");
    let ghost = goal_call(&rig, "goals/get", json!({ "agentId": "ghost" })).await;
    assert_eq!(ghost["error"]["code"], "session/not-found");
}

#[tokio::test]
async fn a_goal_id_is_a_version_4_uuid_and_a_spent_budget_refuses_to_resume() {
    let rig = rig(vec![]).await;
    let session = goal_session(&rig).await;
    let created = goal_call(&rig, "goals/create", json!({ "agentId": session, "request": { "objective": "x", "maxGoalRounds": 2 } })).await["value"]["ref"].clone();
    let id = created["id"].as_str().unwrap();
    let uuid = id.strip_prefix("goal-").expect("goal- prefix");
    let parts: Vec<&str> = uuid.split('-').collect();
    assert_eq!(parts.iter().map(|p| p.len()).collect::<Vec<_>>(), [8, 4, 4, 4, 12], "{id}");
    assert!(parts[2].starts_with('4'), "version nibble: {id}");
    assert!("89ab".contains(parts[3].chars().next().unwrap()), "variant nibble: {id}");

    // The round driver is not part of this build, so a log that carries two
    // admitted rounds stands in for one that ran them.
    let agents = rig.ctx.get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE).unwrap();
    let current = goal_call(&rig, "goals/get", json!({ "agentId": session })).await["value"].clone();
    let paused = goal_call(&rig, "goals/pause", json!({ "agentId": session, "ref": goal_ref(&current) })).await["value"].clone();
    let mut spent: dsh_rs::types::GoalSnapshot = serde_json::from_value(json!({
        "id": paused["id"], "revision": 4, "objective": "x", "phase": "paused", "maxGoalRounds": 2
    })).unwrap();
    spent.revision = paused["revision"].as_u64().unwrap() + 1;
    agents.get(&session).unwrap().session().append(dsh_rs::types::SessionEventData::GoalChange {
        change: dsh_rs::types::GoalChange::Snapshot {
            operation: dsh_rs::types::GoalOperation::Edit,
            goal: spent.clone(),
            rounds_started: 2,
            created_at: paused["createdAt"].as_u64().unwrap(),
            updated_at: paused["updatedAt"].as_u64().unwrap(),
        },
    });
    let refused = goal_call(&rig, "goals/resume", json!({ "agentId": session, "ref": { "id": spent.id, "revision": spent.revision } })).await;
    assert_eq!(
        refused["error"]["message"],
        format!("goal \"{}\" exhausted 2 goal rounds; increase maxGoalRounds before resuming", spent.id)
    );
}

#[tokio::test]
async fn an_odd_revision_in_a_ref_is_a_stale_ref_not_a_field_error() {
    let rig = rig(vec![]).await;
    let session = goal_session(&rig).await;
    let created = goal_call(&rig, "goals/create", json!({ "agentId": session, "request": { "objective": "x" } })).await["value"]["ref"].clone();
    for (revision, shown) in [(json!(-1), "-1"), (json!(1.5), "1.5"), (json!(0), "0"), (json!(1e100), "1e+100")] {
        let result = goal_call(&rig, "goals/pause", json!({ "agentId": session, "ref": { "id": created["id"], "revision": revision } })).await;
        assert_eq!(result["error"]["code"], "gateway/internal", "{revision}: {result}");
        assert_eq!(
            result["error"]["message"],
            format!("stale goal ref \"{}\" revision {shown}; current is \"{}\" revision 1", created["id"].as_str().unwrap(), created["id"].as_str().unwrap()),
        );
    }
}
