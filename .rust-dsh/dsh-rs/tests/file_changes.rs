//! Workspace file watch wire contracts, exercised through the real mux socket.

use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, Gateway, Router, RouterConfig};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const TOKEN: &str = "file-changes-test";
type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Rig {
    _directory: tempfile::TempDir,
    root: std::path::PathBuf,
    session: String,
    socket: Socket,
}

async fn rig() -> Rig {
    let directory = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(directory.path()).unwrap();
    let ctx = Context::new();
    // The GUI plugin provides the sampling service; tests override the default
    // 300 ms through the same plugin configuration, without a global timer.
    install_profile(
        &ctx,
        &json!({ "plugins": { "gui": {
        "port": 0, "fileWatchPollIntervalMs": 10
    } } }),
    )
    .await
    .unwrap();
    let gateway = Arc::new(
        Gateway::new(
            Router::new(RouterConfig::default()),
            Auth {
                token: TOKEN.into(),
            },
            None,
        )
        .with_heartbeat(Duration::from_secs(60)),
    );
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "create", "method": "session/create",
        "payload": { "args": { "request": { "cwd": root } } }
    }))
    .unwrap();
    let (reply, _) = gateway
        .route(&ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    let response: Value = serde_json::from_slice(&reply.body).unwrap();
    assert_eq!(response["result"]["ok"], true, "{response}");
    let session = response["result"]["value"]["sessionId"]
        .as_str()
        .unwrap()
        .to_string();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let serving_ctx = ctx.clone();
    tokio::spawn(async move {
        dsh_rs::gui::server::serve(gateway, serving_ctx, listener)
            .await
            .ok();
    });
    let mut request = format!("ws://{address}/api/remote.mux")
        .into_client_request()
        .unwrap();
    request.headers_mut().insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
    Rig {
        _directory: directory,
        root,
        session,
        socket,
    }
}

async fn send(socket: &mut Socket, frame: Value) {
    socket.send(Message::text(frame.to_string())).await.unwrap();
}

async fn next(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .expect("frame arrives")
            .expect("socket stays open")
            .unwrap();
        if let Message::Text(text) = message {
            return serde_json::from_str(text.as_str()).unwrap();
        }
    }
}

async fn open(rig: &mut Rig, id: &str, args: Value) {
    send(
        &mut rig.socket,
        json!({ "type": "open", "streamId": id,
        "endpoint": "workspaceFiles/changes", "payload": { "args": args } }),
    )
    .await;
}

async fn watch(rig: &mut Rig, path: &str) {
    let session = rig.session.clone();
    open(
        rig,
        "watch",
        json!({ "workspaceFileScopeId": session, "path": path }),
    )
    .await;
    assert_eq!(
        next(&mut rig.socket).await,
        json!({
            "type": "item", "streamId": "watch", "value": { "kind": "ready" }
        })
    );
}

fn version(path: &std::path::Path) -> String {
    let metadata = std::fs::metadata(path).unwrap();
    let modified = metadata
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{modified}-{}", metadata.len())
}

async fn changed(rig: &mut Rig, path: &std::path::Path) {
    let frame = next(&mut rig.socket).await;
    assert_eq!(
        frame,
        json!({ "type": "item", "streamId": "watch", "value": {
        "kind": "change", "change": { "absolutePath": path, "version": version(path) }
    } })
    );
}

#[tokio::test]
async fn argument_refusals_precede_filesystem_access() {
    let mut rig = rig().await;
    open(
        &mut rig,
        "missing",
        json!({ "workspaceFileScopeId": "unknown", "extra": true }),
    )
    .await;
    let frame = next(&mut rig.socket).await;
    assert_eq!(
        frame["error"],
        json!({
            "code": "gateway/arguments-invalid",
            "message": "typert gateway: workspaceFiles/changes: args fields do not match the descriptor: missing \"path\"; unexpected \"extra\"",
            "details": { "endpoint": "workspaceFiles/changes" }
        })
    );
    open(
        &mut rig,
        "type",
        json!({ "workspaceFileScopeId": "unknown", "path": 7 }),
    )
    .await;
    let frame = next(&mut rig.socket).await;
    assert_eq!(
        frame["error"],
        json!({
            "code": "gateway/input-invalid",
            "message": "typert gateway: workspaceFiles/changes: wire field \"path\" failed boundary validation",
            "details": { "endpoint": "workspaceFiles/changes", "field": "path" }
        })
    );
}

#[tokio::test]
async fn file_watch_reports_current_metadata_deletion_and_recreation() {
    let mut rig = rig().await;
    let path = rig.root.join("file.txt");
    std::fs::write(&path, "before").unwrap();
    watch(&mut rig, "file.txt").await;
    std::fs::write(&path, "different length after").unwrap();
    changed(&mut rig, &path).await;
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        next(&mut rig.socket).await["value"],
        json!({
            "kind": "change", "change": { "absolutePath": path, "absent": true }
        })
    );
    std::fs::write(&path, "again").unwrap();
    changed(&mut rig, &path).await;
}

#[tokio::test]
async fn directory_watch_observes_direct_entry_appearance_and_disappearance() {
    let mut rig = rig().await;
    let root = rig.root.clone();
    watch(&mut rig, ".").await;
    let entry = root.join("new.txt");
    std::fs::write(&entry, "hello").unwrap();
    changed(&mut rig, &root).await;
    // Editing a child does not normally change the directory mtime.
    std::fs::write(&entry, "world").unwrap();
    changed(&mut rig, &root).await;
    std::fs::remove_file(entry).unwrap();
    changed(&mut rig, &root).await;
}

#[tokio::test]
async fn prospective_file_appears_and_outside_directory_is_refused() {
    let mut rig = rig().await;
    let session = rig.session.clone();
    watch(&mut rig, "missing.txt").await;
    let appearing = rig.root.join("missing.txt");
    std::fs::write(&appearing, "appeared").unwrap();
    changed(&mut rig, &appearing).await;
    let outside = tempfile::tempdir().unwrap();
    let path = outside.path().to_string_lossy().to_string();
    open(
        &mut rig,
        "outside",
        json!({ "workspaceFileScopeId": session, "path": path }),
    )
    .await;
    assert_eq!(
        next(&mut rig.socket).await["error"],
        json!({
            "code": "workspace-file/outside-workspace", "message": "Directory is outside the workspace",
            "details": { "path": path }
        })
    );
}

#[tokio::test]
async fn cancellation_stops_delivery_and_releases_the_stream_id() {
    let mut rig = rig().await;
    let path = rig.root.join("cancel.txt");
    std::fs::write(&path, "before").unwrap();
    watch(&mut rig, "cancel.txt").await;
    send(
        &mut rig.socket,
        json!({ "type": "cancel", "streamId": "watch" }),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(40)).await;
    std::fs::write(path, "after cancellation").unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(60), rig.socket.next())
            .await
            .is_err()
    );
    watch(&mut rig, "cancel.txt").await;
}
