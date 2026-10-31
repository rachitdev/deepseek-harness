//! The WebSocket mux over a real socket, with a real agent behind it.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::{BoxFuture, LlmAdapterApi, LlmService};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::types::{stream_from_chunks, BoxStream, GenerateOptions, LlmError, StreamChunk};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

mod common;
use common::text_response;

const TOKEN: &str = "launch-token-abc123";

struct Script(Mutex<VecDeque<String>>);

impl LlmAdapterApi for Script {
    fn name(&self) -> &'static str {
        "script"
    }
    fn stream(
        &self,
        _options: GenerateOptions,
    ) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>> {
        let reply = self
            .0
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| "ok".into());
        Box::pin(async move { Ok(stream_from_chunks(text_response(&reply))) })
    }
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Rig {
    ctx: Context,
    address: std::net::SocketAddr,
    gateway: Arc<Gateway>,
}

async fn rig(heartbeat: Duration) -> Rig {
    rig_with(heartbeat, 3000).await
}

/// `approval_ms` is how long a tool call waits for a person's answer.
async fn rig_with(heartbeat: Duration, approval_ms: u64) -> Rig {
    let ctx = Context::new();
    install_profile(
        &ctx,
        &json!({ "plugins": { "approvals": { "timeoutMs": approval_ms }, "ask-user": {} } }),
    )
    .await
    .unwrap();
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(&["mock"], Arc::new(Script(Mutex::new(VecDeque::new()))))
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
        .with_heartbeat(heartbeat)
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

/// The HTTP status a failed upgrade returns.
async fn refused(rig: &Rig, headers: &[(&str, &str)]) -> u16 {
    match tokio_tungstenite::connect_async(request(rig, headers)).await {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => response.status().as_u16(),
        Ok(_) => 101,
        Err(other) => panic!("unexpected error: {other}"),
    }
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

async fn new_session(rig: &Rig) -> String {
    let agents = rig
        .ctx
        .get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    // Reuse the router's own creation path through the public unary endpoint.
    let _ = agents;
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "c", "method": "session/create",
        "payload": { "args": { "request": {} } }
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::HOST,
        format!("{}", rig.address).parse().unwrap(),
    );
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
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
    let gateway = Gateway::new(
        router,
        Auth {
            token: TOKEN.into(),
        },
        None,
    );
    let (reply, _) = gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    let parsed: Value = serde_json::from_slice(&reply.body).unwrap();
    parsed["result"]["value"]["sessionId"]
        .as_str()
        .expect("a session id")
        .to_string()
}

#[tokio::test]
async fn an_upgrade_without_the_token_is_refused_with_401() {
    let rig = rig(Duration::from_secs(5)).await;
    assert_eq!(refused(&rig, &[]).await, 401);
}

#[tokio::test]
async fn an_upgrade_from_a_foreign_origin_is_refused_with_403() {
    let rig = rig(Duration::from_secs(5)).await;
    let bearer = format!("Bearer {TOKEN}");
    assert_eq!(
        refused(
            &rig,
            &[
                ("authorization", &bearer),
                ("origin", "https://evil.example")
            ]
        )
        .await,
        403
    );
}

#[tokio::test]
async fn a_second_socket_is_refused_with_409_until_the_first_closes() {
    let rig = rig(Duration::from_secs(5)).await;
    let bearer = format!("Bearer {TOKEN}");
    let mut first = connect(&rig).await;
    assert_eq!(refused(&rig, &[("authorization", &bearer)]).await, 409);
    first.close(None).await.unwrap();
    drop(first);
    // The slot frees once the server notices the close.
    let mut status = 409;
    for _ in 0..50 {
        status = refused(&rig, &[("authorization", &bearer)]).await;
        if status == 101 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(status, 101, "a closed socket must release its slot");
}

#[tokio::test]
async fn events_opens_with_a_ready_frame_naming_the_client_and_host() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "ev", "$events", json!({})).await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["type"], "item");
    assert_eq!(frame["streamId"], "ev");
    assert_eq!(frame["value"]["type"], "ready");
    assert!(frame["value"]["clientId"]
        .as_str()
        .is_some_and(|id| !id.is_empty()));
    assert!(frame["value"]["host"]["home"].is_string());
}

#[tokio::test]
async fn events_with_arguments_fails_as_invalid_input() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "ev", "$events", json!({ "x": 1 })).await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["type"], "error");
    assert_eq!(frame["error"]["code"], "gateway/input-invalid");
}

#[tokio::test]
async fn a_payload_without_the_args_wrapper_fails_the_stream_not_the_socket() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    send(
        &mut socket,
        json!({ "type": "open", "streamId": "bare", "endpoint": "$events", "payload": {} }),
    )
    .await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["error"]["code"], "gateway/protocol");
    // The socket is still usable.
    open(&mut socket, "after", "$events", json!({})).await;
    assert_eq!(next(&mut socket).await["value"]["type"], "ready");
}

#[tokio::test]
async fn the_job_streams_answer_as_the_typescript_host_does() {
    let rig = rig(Duration::from_secs(5)).await;
    let session = new_session(&rig).await;
    let mut socket = connect(&rig).await;

    // The rows of a session with no jobs, then the stream stays open.
    open(&mut socket, "j1", "job/list", json!({ "request": { "sessionId": session } })).await;
    let rows = next(&mut socket).await;
    assert_eq!(rows["value"], json!({ "type": "rows", "jobs": [] }), "{rows}");

    // A job this build does not have.
    open(&mut socket, "j2", "job/follow", json!({ "request": { "sessionId": session, "jobId": "nope" } })).await;
    let failure = next(&mut socket).await;
    assert_eq!(failure["type"], "error");
    assert_eq!(failure["error"]["code"], "gateway/internal");
    assert_eq!(failure["error"]["message"], "unknown job nope");
    assert_eq!(failure["error"]["details"], json!({}));

    // A request with no job is refused at the boundary.
    open(&mut socket, "j3", "job/follow", json!({ "request": { "sessionId": session } })).await;
    let refused = next(&mut socket).await;
    assert_eq!(refused["error"]["code"], "gateway/input-invalid");
    assert_eq!(
        refused["error"]["message"],
        "typert gateway: job/follow: wire field \"request\" failed boundary validation"
    );
    assert_eq!(refused["error"]["details"], json!({ "endpoint": "job/follow", "field": "request" }));
}

#[tokio::test]
async fn an_unknown_stream_endpoint_is_a_lookup_failure() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "x", "terminal/nope", json!({})).await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["type"], "error");
    assert_eq!(frame["error"]["code"], "gateway/lookup-not-found");
    assert_eq!(frame["streamId"], "x");
}

#[tokio::test]
async fn a_duplicate_stream_id_closes_the_socket_with_1008() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "dup", "$events", json!({})).await;
    next(&mut socket).await; // ready: the first stream is now open
    open(&mut socket, "dup", "$events", json!({})).await;
    let closing = loop {
        match tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
        {
            Some(Ok(Message::Close(frame))) => break frame,
            Some(Ok(_)) => continue,
            other => panic!("expected a close frame, got {other:?}"),
        }
    };
    assert_eq!(
        u16::from(closing.expect("a close frame with a code").code),
        1008
    );
}

#[tokio::test]
async fn a_malformed_frame_closes_the_socket_with_1008() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    socket.send(Message::text("not json")).await.unwrap();
    let closing = loop {
        match tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
        {
            Some(Ok(Message::Close(frame))) => break frame,
            Some(Ok(_)) => continue,
            other => panic!("expected a close frame, got {other:?}"),
        }
    };
    assert_eq!(u16::from(closing.unwrap().code), 1008);
}

#[tokio::test]
async fn follow_sends_a_snapshot_then_each_new_event_in_order() {
    let rig = rig(Duration::from_secs(5)).await;
    let session_id = new_session(&rig).await;
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "f",
        "session/follow",
        json!({ "request": { "address": { "kind": "session", "sessionId": session_id } } }),
    )
    .await;
    let snapshot = next(&mut socket).await;
    assert_eq!(snapshot["streamId"], "f");
    assert_eq!(snapshot["value"]["type"], "snapshot");
    assert_eq!(snapshot["value"]["header"]["id"], session_id);
    // The cursor is an inclusive watermark: -1 for an empty log.
    let baseline = snapshot["value"]["cursor"].as_i64().unwrap();
    assert_eq!(baseline, -1, "a fresh session has no events yet");

    // Append three events to the live session; they must arrive once each, in order.
    let agents = rig
        .ctx
        .get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let session = agents.get(&session_id).unwrap().session();
    for turn in 1..=3u64 {
        session.append(dsh_rs::types::SessionEventData::TurnStart { turn });
    }
    let mut seen = Vec::new();
    for _ in 0..3 {
        let frame = next(&mut socket).await;
        assert_eq!(frame["value"]["type"], "event");
        seen.push(frame["value"]["event"]["seq"].as_i64().unwrap());
    }
    assert_eq!(seen, vec![baseline + 1, baseline + 2, baseline + 3]);
    assert_eq!(
        seen[0], 0,
        "the first event after an empty snapshot is seq 0, not skipped"
    );
}

#[tokio::test]
async fn follow_of_an_unknown_session_fails_the_stream_by_name() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "f",
        "session/follow",
        json!({ "request": { "address": { "kind": "session", "sessionId": "nope" } } }),
    )
    .await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["type"], "error");
    assert_eq!(frame["error"]["code"], "gateway/lookup-not-found");
}

#[tokio::test]
async fn a_cancelled_follow_sends_nothing_and_frees_its_subscription() {
    let rig = rig(Duration::from_secs(5)).await;
    let session_id = new_session(&rig).await;
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "f",
        "session/follow",
        json!({ "request": { "address": { "kind": "session", "sessionId": session_id } } }),
    )
    .await;
    // Without this check the test passes even when the stream failed to open.
    assert_eq!(next(&mut socket).await["value"]["type"], "snapshot");
    send(&mut socket, json!({ "type": "cancel", "streamId": "f" })).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let agents = rig
        .ctx
        .get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    agents
        .get(&session_id)
        .unwrap()
        .session()
        .append(dsh_rs::types::SessionEventData::TurnStart { turn: 9 });
    let quiet = tokio::time::timeout(Duration::from_millis(300), socket.next()).await;
    assert!(
        quiet.is_err(),
        "a cancelled stream must not send: {quiet:?}"
    );

    // The id is reusable once the stream has left the table.
    open(&mut socket, "f", "$events", json!({})).await;
    assert_eq!(next(&mut socket).await["value"]["type"], "ready");
}

#[tokio::test]
async fn a_silent_peer_is_dropped_after_two_missed_heartbeats() {
    let rig = rig(Duration::from_millis(50)).await;
    // Hand-rolled client that completes the upgrade and then never reads again,
    // so it cannot answer pings.
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut raw = TcpStream::connect(rig.address).await.unwrap();
    let handshake = format!(
        "GET /api/remote.mux HTTP/1.1\r\nHost: {}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Authorization: Bearer {TOKEN}\r\n\r\n",
        rig.address
    );
    raw.write_all(handshake.as_bytes()).await.unwrap();
    let mut head = [0u8; 512];
    let read = raw.read(&mut head).await.unwrap();
    assert!(String::from_utf8_lossy(&head[..read]).starts_with("HTTP/1.1 101"));
    // The raw socket never answers; the server must hang up within a few intervals.
    let closed = tokio::time::timeout(Duration::from_secs(2), async {
        let mut sink = [0u8; 256];
        loop {
            match raw.read(&mut sink).await {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
        }
    })
    .await;
    assert!(
        closed.is_ok(),
        "the server must drop a peer that never answers pings"
    );
}

/// Complete the upgrade on a raw socket so the test controls every read.
async fn raw_upgrade(rig: &Rig) -> TcpStream {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut raw = TcpStream::connect(rig.address).await.unwrap();
    let handshake = format!(
        "GET /api/remote.mux HTTP/1.1\r\nHost: {}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Authorization: Bearer {TOKEN}\r\n\r\n",
        rig.address
    );
    raw.write_all(handshake.as_bytes()).await.unwrap();
    let mut head = [0u8; 512];
    let read = raw.read(&mut head).await.unwrap();
    assert!(String::from_utf8_lossy(&head[..read]).starts_with("HTTP/1.1 101"));
    raw
}

/// One masked client text frame (RFC 6455 §5.2) for a raw socket.
fn masked_text(payload: &str) -> Vec<u8> {
    let bytes = payload.as_bytes();
    let mask = [0x12u8, 0x34, 0x56, 0x78];
    let mut frame = vec![0x81];
    match bytes.len() {
        0..=125 => frame.push(0x80 | bytes.len() as u8),
        126..=65535 => {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        }
        _ => {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        }
    }
    frame.extend_from_slice(&mask);
    frame.extend(bytes.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
    frame
}

#[tokio::test]
async fn a_peer_that_stops_reading_cannot_stall_its_own_disconnect() {
    // The client opens a follow whose events are far larger than the socket
    // buffers, then never reads. The server's write blocks; it must still notice
    // the missed pings and release the socket slot rather than hang with it.
    use tokio::io::AsyncWriteExt;
    let rig = rig(Duration::from_millis(60)).await;
    let session_id = new_session(&rig).await;
    let agents = rig
        .ctx
        .get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let session = agents.get(&session_id).unwrap().session();
    let big = "x".repeat(512 * 1024);
    for n in 0..64u64 {
        session.append(dsh_rs::types::SessionEventData::UserMessage {
            message: dsh_rs::types::Message::user(
                format!("m{n}"),
                vec![dsh_rs::types::ContentBlock::Text { text: big.clone() }],
            ),
        });
    }
    // max_page_events is 100, so all 64 large events (about 32 MB) enter the snapshot.
    let mut raw = raw_upgrade(&rig).await;
    let open = json!({
        "type": "open", "streamId": "big", "endpoint": "session/follow",
        "payload": { "args": { "request": { "address": { "kind": "session", "sessionId": session_id } } } }
    })
    .to_string();
    raw.write_all(&masked_text(&open)).await.unwrap();
    // Never read from `raw`. The slot must free well inside the timeout.
    let bearer = format!("Bearer {TOKEN}");
    let mut freed = false;
    for _ in 0..100 {
        if refused(&rig, &[("authorization", &bearer)]).await == 101 {
            freed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(
        freed,
        "a peer that stopped reading must not hold the socket slot forever"
    );
}

#[tokio::test]
async fn a_peer_close_frame_is_answered_with_a_close_frame() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    socket.send(Message::Close(None)).await.unwrap();
    let mut got_close = false;
    while let Ok(Some(Ok(message))) =
        tokio::time::timeout(Duration::from_secs(2), socket.next()).await
    {
        if matches!(message, Message::Close(_)) {
            got_close = true;
            break;
        }
    }
    assert!(got_close, "the server must complete the closing handshake");
}

#[tokio::test]
async fn a_malformed_websocket_key_is_refused_with_400_and_keeps_the_slot() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let rig = rig(Duration::from_secs(5)).await;
    for key in [
        "x",
        "dGhlIHNhbXBsZSBub25jZQ",
        "dGhlIHNhbXBsZSBub25jZR==",
        "!!!!!!!!!!!!!!!!!!!!!!==",
    ] {
        let mut raw = TcpStream::connect(rig.address).await.unwrap();
        let handshake = format!(
            "GET /api/remote.mux HTTP/1.1\r\nHost: {}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\
             Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {key}\r\n\
             Authorization: Bearer {TOKEN}\r\n\r\n",
            rig.address
        );
        raw.write_all(handshake.as_bytes()).await.unwrap();
        let mut head = [0u8; 256];
        let read = raw.read(&mut head).await.unwrap();
        let text = String::from_utf8_lossy(&head[..read]);
        assert!(text.starts_with("HTTP/1.1 400"), "key {key:?} gave: {text}");
    }
    // A rejected upgrade must not have taken the slot.
    let _first = connect(&rig).await;
}

// --- approvals over `$events` -------------------------------------------------------------------

use dsh_rs::api::services::ToolsService;
use dsh_rs::types::{CancelToken, ToolExecutionResult, ToolRunContext};

/// Run `bash` the way an agent would; resolves with the tool's result.
fn call_bash(rig: &Rig, command: String) -> tokio::task::JoinHandle<ToolExecutionResult> {
    let ctx = rig.ctx.clone();
    tokio::spawn(async move {
        let tools = ctx.require::<ToolsService>("tools").unwrap();
        let run = ToolRunContext {
            ctx: ctx.clone(),
            signal: CancelToken::new(),
            agent_id: Some("agent-7".into()),
            cwd: Some("/tmp".into()),
        };
        tools
            .execute(
                "call-1".into(),
                "bash".into(),
                json!({ "command": command }),
                run,
            )
            .await
    })
}

/// Open `$events` and return the client id from its ready frame.
async fn open_events(socket: &mut Socket) -> String {
    open(socket, "ev", "$events", json!({})).await;
    let ready = next(socket).await;
    assert_eq!(ready["value"]["type"], "ready", "{ready}");
    ready["value"]["clientId"].as_str().unwrap().to_string()
}

/// What the browser does: POST `$events/result`.
async fn answer(rig: &Rig, client_id: &str, event_id: &str, outcome: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "r1", "method": "$events/result",
        "payload": { "args": { "clientId": client_id, "eventId": event_id, "outcome": outcome } }
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, rig.address.to_string().parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}

fn marker(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("dsh-gui-approval-{name}-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn denial(result: &ToolExecutionResult) -> String {
    match result {
        ToolExecutionResult::Error { code, message, .. } => {
            assert_eq!(code, "DENIED", "{message}");
            message.clone()
        }
        other => panic!("expected a denial, got {other:?}"),
    }
}

#[tokio::test]
async fn a_tool_call_reaches_the_browser_and_runs_once_it_is_allowed() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    let client = open_events(&mut socket).await;
    let file = marker("allow");
    let call = call_bash(&rig, format!("touch {}", file.display()));

    let frame = next(&mut socket).await;
    let wire = &frame["value"];
    assert_eq!(wire["type"], "waterfall");
    assert_eq!(wire["event"], "approval/request");
    assert_eq!(wire["agentId"], "agent-7");
    assert_eq!(wire["request"]["toolName"], "bash");
    let reason = wire["request"]["reason"].as_str().unwrap();
    assert!(
        reason.contains(&format!("touch {}", file.display())),
        "full arguments: {reason}"
    );
    assert!(!file.exists(), "nothing runs before an answer");

    let outcome = json!({ "kind": "result", "value": "allowed-once" });
    let sent = answer(&rig, &client, wire["eventId"].as_str().unwrap(), outcome).await;
    assert_eq!(sent["ok"], true, "{sent}");
    assert!(!call.await.unwrap().is_error());
    assert!(file.exists());
}

#[tokio::test]
async fn a_rejected_or_unrecognised_answer_never_runs_the_tool() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    let client = open_events(&mut socket).await;
    let outcomes = [
        json!({ "kind": "result", "value": "rejected" }),
        json!({ "kind": "result", "value": "allowed-always" }),
        json!({ "kind": "next" }),
    ];
    for (index, outcome) in outcomes.into_iter().enumerate() {
        let file = marker(&format!("deny{index}"));
        let call = call_bash(&rig, format!("touch {}", file.display()));
        let frame = next(&mut socket).await;
        let event = frame["value"]["eventId"].as_str().unwrap().to_string();
        let sent = answer(&rig, &client, &event, outcome.clone()).await;
        assert_eq!(sent["ok"], true, "{sent}");
        denial(&call.await.unwrap());
        assert!(!file.exists(), "{outcome} must not run the command");
    }
}

#[tokio::test]
async fn closing_the_events_stream_denies_the_waiting_call() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    let client = open_events(&mut socket).await;
    let file = marker("closed");
    let call = call_bash(&rig, format!("touch {}", file.display()));
    let frame = next(&mut socket).await;
    let event = frame["value"]["eventId"].as_str().unwrap().to_string();

    send(&mut socket, json!({ "type": "cancel", "streamId": "ev" })).await;
    let message = denial(&call.await.unwrap());
    assert!(message.contains("went away"), "{message}");

    // The browser's answer arrives too late: refused, and nothing ran.
    let late = json!({ "kind": "result", "value": "allowed-once" });
    let sent = answer(&rig, &client, &event, late).await;
    assert_eq!(sent["ok"], false, "{sent}");
    assert!(!file.exists());
}

#[tokio::test]
async fn a_question_that_times_out_is_withdrawn_from_the_browser() {
    let rig = rig_with(Duration::from_secs(5), 300).await;
    let mut socket = connect(&rig).await;
    open_events(&mut socket).await;
    let call = call_bash(&rig, "true".into());
    let frame = next(&mut socket).await;
    let event = frame["value"]["eventId"].as_str().unwrap().to_string();

    let message = denial(&call.await.unwrap());
    assert!(message.contains("no answer within"), "{message}");
    let cancel = next(&mut socket).await;
    assert_eq!(
        cancel["value"],
        json!({ "type": "cancel", "eventId": event })
    );
}

#[tokio::test]
async fn without_an_events_stream_the_call_is_denied() {
    let rig = rig(Duration::from_secs(5)).await;
    let _socket = connect(&rig).await; // connected, but `$events` not opened
    let call = call_bash(&rig, "true".into());
    let message = denial(&call.await.unwrap());
    assert!(message.contains("no interactive approver"), "{message}");
}

#[tokio::test]
async fn a_result_for_the_wrong_client_or_a_malformed_one_is_refused() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    open_events(&mut socket).await;
    let next_outcome = json!({ "kind": "next" });
    let wrong = answer(&rig, "client-nope", "e", next_outcome).await;
    assert_eq!(
        wrong["error"]["code"], "gateway/lookup-not-found",
        "{wrong}"
    );
    let malformed = answer(&rig, "client-nope", "e", json!("allowed-once")).await;
    assert_eq!(
        malformed["error"]["code"], "gateway/input-invalid",
        "{malformed}"
    );
}

async fn call_unary(rig: &Rig, method: &str, request: Value) -> Value {
    call_unary_raw(rig, method, json!({ "request": request })).await
}

/// A unary call whose `args` are sent exactly as given.
async fn call_unary_raw(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "w", "method": method,
        "payload": { "args": args }
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::HOST,
        format!("{}", rig.address).parse().unwrap(),
    );
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    let response: Value = serde_json::from_slice(&reply.body).unwrap();
    assert_eq!(response["result"]["ok"], true, "{method}: {response}");
    response["result"]["value"].clone()
}

#[tokio::test]
async fn workspace_follow_sends_a_baseline_then_each_change_in_commit_order() {
    let rig = rig(Duration::from_secs(30)).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "w1", "workspace/follow", json!({})).await;
    let first = next(&mut socket).await;
    assert_eq!(first["streamId"], "w1");
    assert_eq!(first["value"]["type"], "baseline");
    assert_eq!(first["value"]["value"]["items"], json!([]));
    assert_eq!(first["value"]["value"]["pinnedSessionIds"], json!([]));

    let dir = tempfile::tempdir().unwrap();
    let path = std::fs::canonicalize(dir.path())
        .unwrap()
        .display()
        .to_string();
    let made = call_unary(&rig, "workspace/create", json!({ "path": path })).await;
    let id = made["workspace"]["workspaceId"]
        .as_str()
        .unwrap()
        .to_string();
    let upsert = next(&mut socket).await;
    assert_eq!(upsert["value"]["type"], "upsert");
    assert_eq!(upsert["value"]["workspace"]["workspaceId"], id.as_str());

    // A session started outside any workspace call still reaches the follower
    // once it lands in this workspace's directory.
    let session = call_unary(&rig, "session/create", json!({ "workspaceId": id })).await;
    let sid = session["sessionId"].as_str().unwrap();
    let joined = next(&mut socket).await;
    assert_eq!(joined["value"]["type"], "upsert");
    assert_eq!(joined["value"]["workspace"]["sessionIds"], json!([sid]));

    call_unary(&rig, "workspace/pinSession", json!({ "sessionId": sid })).await;
    let pinned = next(&mut socket).await;
    assert_eq!(
        pinned["value"],
        json!({ "type": "pinned", "pinnedSessionIds": [sid] })
    );

    call_unary(&rig, "workspace/delete", json!({ "workspaceId": id })).await;
    let removed = next(&mut socket).await;
    assert_eq!(
        removed["value"],
        json!({ "type": "remove", "workspaceId": id })
    );
}

#[tokio::test]
async fn workspace_follow_refuses_arguments() {
    let rig = rig(Duration::from_secs(30)).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "w1", "workspace/follow", json!({ "x": 1 })).await;
    let reply = next(&mut socket).await;
    assert_eq!(reply["type"], "error");
    assert_eq!(reply["error"]["code"], "gateway/input-invalid");
}

#[tokio::test]
async fn remote_access_reports_the_token_holder_as_owner_and_stays_open() {
    let rig = rig(Duration::from_secs(30)).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "r1", "remoteAccess/follow", json!({})).await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["value"], json!({ "type": "baseline", "owner": true }));
    send(&mut socket, json!({ "type": "cancel", "streamId": "r1" })).await;
}

#[tokio::test]
async fn session_control_lists_each_live_session_with_its_computable_projections() {
    let rig = rig(Duration::from_secs(30)).await;
    let session = new_session(&rig).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "c1", "session/control", json!({})).await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["value"]["type"], "baseline");
    let block = &frame["value"]["value"]["projections"][session.as_str()];
    assert_eq!(block["asOfSeq"], -1);
    assert_eq!(block["values"]["modelSelection"]["next"]["model"], "m");
    assert!(block["values"].get("tokenUsage").is_none());
}

#[tokio::test]
async fn the_account_is_always_signed_out() {
    let rig = rig(Duration::from_secs(30)).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "a1", "account/watch", json!({})).await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["value"]["status"], "signed-out");
    assert_eq!(frame["value"]["attempt"], Value::Null);
    assert!(frame["value"]["links"]["usageUrl"]
        .as_str()
        .unwrap()
        .starts_with("https://"));
}

#[tokio::test]
async fn a_follower_that_opts_into_the_assistant_stream_gets_the_empty_baseline() {
    let rig = rig(Duration::from_secs(30)).await;
    let session = new_session(&rig).await;
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "f1",
        "session/follow",
        json!({ "request": { "address": { "kind": "session", "sessionId": session }, "assistantStream": true } }),
    )
    .await;
    let snapshot = next(&mut socket).await;
    assert_eq!(
        snapshot["value"]["assistantStream"],
        json!({ "revision": 0 })
    );

    open(
        &mut socket,
        "f2",
        "session/follow",
        json!({ "request": { "address": { "kind": "session", "sessionId": session } } }),
    )
    .await;
    let plain = next(&mut socket).await;
    assert!(
        plain["value"].get("assistantStream").is_none(),
        "only sent when asked for"
    );
}

#[tokio::test]
async fn followed_records_carry_the_envelope_the_typescript_reader_requires() {
    let rig = rig(Duration::from_secs(30)).await;
    let session = new_session(&rig).await;
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "f",
        "session/follow",
        json!({ "request": { "address": { "kind": "session", "sessionId": session } } }),
    )
    .await;
    let snapshot = next(&mut socket).await;
    assert_eq!(snapshot["value"]["records"], json!([]));

    let prompted = call_unary(
        &rig,
        "session/prompt",
        json!({ "sessionId": session, "requestId": "t-req", "mode": "queue", "content": [{ "type": "text", "text": "hello" }] }),
    )
    .await;
    assert_eq!(prompted["accepted"], true, "{prompted}");

    let mut by_type = std::collections::HashMap::new();
    while !by_type.contains_key("assistant/message") {
        let frame = next(&mut socket).await;
        let event = frame["value"]["event"].clone();
        by_type.insert(event["type"].as_str().unwrap().to_string(), event);
    }
    for kind in ["user/message", "assistant/message"] {
        assert_eq!(
            by_type[kind]["surfaceOp"], "append",
            "{kind} is a surface event"
        );
    }
    assert!(
        by_type["turn/start"].get("surfaceOp").is_none(),
        "only message events are surface events"
    );
    assert_eq!(
        by_type["user/message"]["data"]["source"]["kind"].is_string(),
        true
    );
}

#[tokio::test]
async fn the_event_stream_tells_the_browser_when_a_session_starts_and_stops_running() {
    let rig = rig(Duration::from_secs(30)).await;
    let session = new_session(&rig).await;
    let mut socket = connect(&rig).await;
    open_events(&mut socket).await;

    let prompted = call_unary(
        &rig,
        "session/prompt",
        json!({ "sessionId": session, "requestId": "t-req", "mode": "queue", "content": [{ "type": "text", "text": "hello" }] }),
    )
    .await;
    assert_eq!(prompted["accepted"], true, "{prompted}");

    let mut seen = Vec::new();
    while seen.last() != Some(&false) {
        let frame = next(&mut socket).await;
        let value = &frame["value"];
        if value["type"] == "emit" && value["event"] == "api-session/status" {
            assert_eq!(value["args"][0], session.as_str());
            seen.push(value["args"][1].as_bool().unwrap());
        }
    }
    assert_eq!(seen.first(), Some(&true), "it reports running first: {seen:?}");
    assert_eq!(seen.last(), Some(&false), "and idle at the end: {seen:?}");
}

#[tokio::test]
async fn session_control_pushes_the_goal_when_it_is_created_and_cleared() {
    let rig = rig(Duration::from_secs(30)).await;
    let session = new_session(&rig).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "c1", "session/control", json!({})).await;
    let baseline = next(&mut socket).await;
    assert_eq!(baseline["value"]["type"], "baseline");

    let created = call_unary_raw(
        &rig,
        "goals/create",
        json!({ "agentId": session, "request": { "objective": "ship it" } }),
    )
    .await;
    let goal_ref = created["ref"].clone();
    let frame = next(&mut socket).await;
    assert_eq!(frame["value"]["key"], "goal");
    assert_eq!(frame["value"]["value"]["goal"]["objective"], "ship it");
    assert_eq!(frame["value"]["value"]["goal"]["phase"], "active");

    call_unary_raw(&rig, "goals/clear", json!({ "agentId": session, "ref": goal_ref })).await;
    let frame = next(&mut socket).await;
    assert_eq!(frame["value"]["key"], "goal");
    assert_eq!(frame["value"]["value"], Value::Null, "a cleared goal projects as null");

    // Two changes appended back to back each push the state they made, not
    // the state the log has reached by the time the push is built.
    let created = call_unary_raw(&rig, "goals/create", json!({ "agentId": session, "request": { "objective": "again" } })).await;
    let agents = rig.ctx.get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE).unwrap();
    let id = created["ref"]["id"].as_str().unwrap().to_string();
    agents.get(&session).unwrap().session().append(dsh_rs::types::SessionEventData::GoalChange {
        change: dsh_rs::types::GoalChange::Clear {
            cleared: dsh_rs::types::GoalRef { id, revision: 2 },
            cleared_at: u64::MAX / 2,
        },
    });
    let first = next(&mut socket).await;
    assert_eq!(first["value"]["value"]["goal"]["objective"], "again", "the create frame reports the create");
    let second = next(&mut socket).await;
    assert_eq!(second["value"]["value"], Value::Null);
}

#[tokio::test]
async fn session_control_pushes_a_projection_frame_for_a_model_change_and_a_rename() {
    let rig = rig(Duration::from_secs(30)).await;
    let session = new_session(&rig).await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "c1", "session/control", json!({})).await;
    let agents = rig
        .ctx
        .get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents.get(&session).unwrap();
    agent.session().append(dsh_rs::types::SessionEventData::SessionTitle { title: "Before".into() });
    let baseline = next(&mut socket).await;
    assert_eq!(baseline["value"]["type"], "baseline");
    let previous_seq = baseline["value"]["value"]["projections"][&session]["asOfSeq"].as_u64().unwrap();
    agent.select_model("mock".into(), "big".into());
    let frame = next(&mut socket).await;
    assert_eq!(frame["value"]["type"], "projection");
    assert_eq!(frame["value"]["sessionId"], session.as_str());
    assert_eq!(frame["value"]["key"], "modelSelection");
    assert_eq!(frame["value"]["seq"], previous_seq + 1, "the TS client rejects frames with the same seq as its baseline");
    assert_eq!(frame["value"]["value"]["next"]["model"], "big");

    let seq = agent
        .session()
        .append(dsh_rs::types::SessionEventData::SessionTitle { title: "Renamed".into() });
    let frame = next(&mut socket).await;
    assert_eq!(frame["value"]["key"], "title");
    assert_eq!(frame["value"]["value"], "Renamed");
    assert_eq!(frame["value"]["seq"], seq.seq);

    let header = dsh_rs::types::EpochHeader {
        config: dsh_rs::types::LlmCallConfig {
            provider: "mock".into(), model: "big".into(),
            ..Default::default()
        },
        system: None, tools: None,
    };
    let used = agent.session().append(dsh_rs::types::SessionEventData::RequestHeader { header });
    let frame = next(&mut socket).await;
    assert_eq!(frame["value"]["key"], "modelSelection");
    assert_eq!(frame["value"]["seq"], used.seq);
    assert_eq!(frame["value"]["value"]["lastUsed"]["model"], "big");

    // A cancelled stream stops receiving: the next change produces no frame.
    send(&mut socket, json!({ "type": "cancel", "streamId": "c1" })).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    agent.select_model("mock".into(), "m".into());
    let silent = tokio::time::timeout(Duration::from_millis(300), socket.next()).await;
    assert!(silent.is_err(), "a cancelled control stream sends nothing: {silent:?}");
}

// --- user questions over `$events` --------------------------------------------------------------

use dsh_rs::plugins::ask_user::{QuestionBroker, USER_QUESTIONS_SERVICE};

/// Run `ask_user_question` the way an agent would.
fn call_ask(rig: &Rig, signal: CancelToken) -> tokio::task::JoinHandle<ToolExecutionResult> {
    let ctx = rig.ctx.clone();
    tokio::spawn(async move {
        let tools = ctx.require::<ToolsService>("tools").unwrap();
        let run = ToolRunContext {
            ctx: ctx.clone(),
            signal,
            agent_id: Some("agent-7".into()),
            cwd: None,
        };
        let args = json!({ "questions": [
            { "id": "a", "question": "First?", "multi_select": true, "options": [{ "label": "x" }] },
            { "id": "b", "question": "Second?" }
        ]});
        tools
            .execute("call-9".into(), "ask_user_question".into(), args, run)
            .await
    })
}

fn answers(items: Value) -> Value {
    json!({ "kind": "result", "value": { "answers": items } })
}

#[tokio::test]
async fn a_question_reaches_the_browser_and_its_answer_returns_to_the_tool() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    let client = open_events(&mut socket).await;
    let call = call_ask(&rig, CancelToken::new());

    let frame = next(&mut socket).await;
    let wire = &frame["value"];
    assert_eq!(wire["type"], "waterfall");
    assert_eq!(wire["event"], "user-questions/request");
    assert_eq!(wire["agentId"], "agent-7");
    assert_eq!(wire["request"]["questions"][0]["multiSelect"], true);
    assert_eq!(wire["request"]["questions"][1]["id"], "b");

    let given = answers(json!([
        { "id": "b", "selected": [], "custom": "free text" },
        { "id": "a", "selected": ["x"] }
    ]));
    let sent = answer(&rig, &client, wire["eventId"].as_str().unwrap(), given).await;
    assert_eq!(sent["ok"], true, "{sent}");
    match call.await.unwrap() {
        ToolExecutionResult::Success { value, .. } => assert_eq!(
            value["answers"],
            json!([
                { "id": "b", "selected": [], "custom": "free text" },
                { "id": "a", "selected": ["x"] }
            ])
        ),
        other => panic!("expected the answer, got {other:?}"),
    }
}

#[tokio::test]
async fn an_answer_that_misses_a_question_is_refused_and_the_question_stays_open() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    let client = open_events(&mut socket).await;
    let call = call_ask(&rig, CancelToken::new());
    let event = next(&mut socket).await["value"]["eventId"]
        .as_str()
        .unwrap()
        .to_string();

    for bad in [
        answers(json!([{ "id": "a", "selected": [] }])),
        answers(json!([{ "id": "a", "selected": [] }, { "id": "a", "selected": [] }])),
        answers(json!([{ "id": "a", "selected": [] }, { "id": "zzz", "selected": [] }])),
        answers(json!([{ "id": "a", "selected": "x" }, { "id": "b", "selected": [] }])),
        json!({ "kind": "result" }),
        json!({ "kind": "result", "value": { "answers": [], "extra": 1 } }),
    ] {
        let sent = answer(&rig, &client, &event, bad.clone()).await;
        assert_eq!(sent["ok"], false, "{bad}: {sent}");
        assert_eq!(sent["error"]["code"], "gateway/input-invalid", "{sent}");
    }
    assert!(!call.is_finished(), "a malformed answer does not end the call");

    let good = answers(json!([{ "id": "a", "selected": [] }, { "id": "b", "selected": [], "custom": "ok" }]));
    assert_eq!(answer(&rig, &client, &event, good).await["ok"], true);
    assert!(!call.await.unwrap().is_error());
}

#[tokio::test]
async fn a_rejected_or_unclaimed_question_fails_the_call_with_the_reason() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    let client = open_events(&mut socket).await;
    for (outcome, needle) in [
        (
            json!({ "kind": "rejected", "error": { "name": "E", "message": "the user cancelled ask_user_question" } }),
            "the user cancelled",
        ),
        (json!({ "kind": "next" }), "no user-questions answerer"),
    ] {
        let call = call_ask(&rig, CancelToken::new());
        let event = next(&mut socket).await["value"]["eventId"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(answer(&rig, &client, &event, outcome).await["ok"], true);
        match call.await.unwrap() {
            ToolExecutionResult::Error { code, message, .. } => {
                assert_eq!(code, "NO_PROVIDER");
                assert!(message.contains(needle), "{message}");
            }
            other => panic!("expected a failure, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn closing_the_events_stream_fails_the_waiting_question() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    let _client = open_events(&mut socket).await;
    let call = call_ask(&rig, CancelToken::new());
    next(&mut socket).await;
    drop(socket);
    let result = tokio::time::timeout(Duration::from_secs(3), call)
        .await
        .expect("the call ends once its stream is gone")
        .unwrap();
    assert!(result.is_error());
    let broker = rig.ctx.get::<QuestionBroker>(USER_QUESTIONS_SERVICE).unwrap();
    assert!(!broker.attached(), "the stream took its answerer with it");
}

#[tokio::test]
async fn a_cancelled_turn_withdraws_the_question_from_the_browser() {
    let rig = rig(Duration::from_secs(5)).await;
    let mut socket = connect(&rig).await;
    let _client = open_events(&mut socket).await;
    let cancel = CancelToken::new();
    let call = call_ask(&rig, cancel.clone());
    let event = next(&mut socket).await["value"]["eventId"].clone();
    cancel.cancel();
    assert!(call.await.unwrap().is_error());
    let withdrawn = next(&mut socket).await;
    assert_eq!(withdrawn["value"]["type"], "cancel");
    assert_eq!(withdrawn["value"]["eventId"], event);
}

#[tokio::test]
async fn without_an_events_stream_a_question_fails_at_once() {
    let rig = rig(Duration::from_secs(5)).await;
    let result = tokio::time::timeout(Duration::from_secs(2), call_ask(&rig, CancelToken::new()))
        .await
        .expect("no browser means an immediate failure")
        .unwrap();
    assert!(result.is_error());
}
