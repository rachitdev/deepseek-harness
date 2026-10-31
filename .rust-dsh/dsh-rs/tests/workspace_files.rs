//! The workspace file endpoints: a session's files, read through the
//! filesystem.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use cordis::Context;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use serde_json::{json, Value};

const TOKEN: &str = "files-token-0123456789";

/// A router with the filesystem endpoints and a context to serve them.
struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
}

async fn rig() -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .expect("profile installs");
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
    Rig { ctx, gateway }
}

/// One unary call through the gateway.
async fn call(rig: &Rig, method: &str, payload: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request",
        "rpcId": "rpc-1",
        "method": method,
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
        .route(
            &rig.ctx,
            &http::Method::POST,
            "/api",
            &headers,
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

/// The value one call answered.
async fn value_of(rig: &Rig, method: &str, payload: Value) -> Value {
    let response = call(rig, method, payload).await;
    assert_eq!(response["result"]["ok"], true, "{method}: {response}");
    response["result"]["value"].clone()
}

/// The code one call refused with.
async fn code_of(rig: &Rig, method: &str, payload: Value) -> String {
    let response = call(rig, method, payload).await;
    assert_eq!(response["result"]["ok"], false, "{method}: {response}");
    response["result"]["error"]["code"]
        .as_str()
        .unwrap()
        .to_string()
}

/// A workspace directory that no other test shares.
fn workspace(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("dsh-files-{name}-{unique}"));
    std::fs::create_dir_all(&path).expect("the workspace is created");
    path
}

/// A session whose workspace is `root`.
async fn session_for(rig: &Rig, root: &Path) -> String {
    let created = value_of(
        rig,
        "session/create",
        json!({ "request": { "cwd": root.to_string_lossy() } }),
    )
    .await;
    created["sessionId"].as_str().unwrap().to_string()
}

/// A file with three lines.
fn write_lines(path: &Path, lines: &str) {
    std::fs::write(path, lines).expect("the file is written");
}

#[tokio::test]
async fn a_file_read_returns_its_page_and_its_stat() {
    let rig = rig().await;
    let root = workspace("read");
    std::fs::write(root.join("notes.txt"), "one\ntwo\nthree\n").expect("written");
    let session = session_for(&rig, &root).await;

    let page = value_of(
        &rig,
        "workspaceFiles/read",
        json!({ "workspaceFileScopeId": session, "path": "notes.txt", "range": {} }),
    )
    .await;
    assert_eq!(page["offset"], 1, "{page}");
    assert_eq!(page["text"], "one\ntwo\nthree", "{page}");
    assert_eq!(page["lines"], 3, "{page}");
    assert_eq!(page["eof"], true, "{page}");
    assert_eq!(page["bytes"], 14, "{page}");
    assert!(
        page["absolutePath"]
            .as_str()
            .unwrap()
            .ends_with("notes.txt"),
        "{page}"
    );
    assert!(page["version"].as_str().is_some(), "{page}");
}

#[tokio::test]
async fn a_page_starts_where_it_is_asked_and_reports_the_end() {
    let rig = rig().await;
    let root = workspace("page");
    write_lines(&root.join("notes.txt"), "one\ntwo\nthree\n");
    let session = session_for(&rig, &root).await;

    let page = value_of(
        &rig,
        "workspaceFiles/read",
        json!({ "workspaceFileScopeId": session, "path": "notes.txt", "range": { "offset": 2, "limit": 1 } }),
    )
    .await;
    assert_eq!(page["offset"], 2, "{page}");
    assert_eq!(page["text"], "two", "{page}");
    assert_eq!(page["lines"], 1, "{page}");
    assert_eq!(page["eof"], false, "{page}");

    let past = value_of(
        &rig,
        "workspaceFiles/read",
        json!({ "workspaceFileScopeId": session, "path": "notes.txt", "range": { "offset": 9 } }),
    )
    .await;
    assert_eq!(past["text"], "", "{past}");
    assert_eq!(past["lines"], 0, "{past}");
    assert_eq!(past["eof"], true, "{past}");
}

#[tokio::test]
async fn a_page_above_the_line_cap_is_refused() {
    let rig = rig().await;
    let root = workspace("cap");
    write_lines(&root.join("notes.txt"), "one\n");
    let session = session_for(&rig, &root).await;

    let code = code_of(
        &rig,
        "workspaceFiles/read",
        json!({ "workspaceFileScopeId": session, "path": "notes.txt", "range": { "limit": 5_001 } }),
    )
    .await;
    assert_eq!(code, "gateway/bad-request");
}

#[tokio::test]
async fn a_byte_window_returns_the_bytes_it_names() {
    let rig = rig().await;
    let root = workspace("bytes");
    std::fs::write(root.join("bytes.bin"), [0u8, 1, 2, 3, 4]).expect("written");
    let session = session_for(&rig, &root).await;

    let window = value_of(
        &rig,
        "workspaceFiles/readBytes",
        json!({ "workspaceFileScopeId": session, "path": "bytes.bin", "options": { "range": { "offset": 1, "length": 3 } } }),
    )
    .await;
    assert_eq!(window["offset"], 1, "{window}");
    assert_eq!(window["data"], "AQID", "{window}");
    assert_eq!(window["eof"], false, "{window}");

    let whole = value_of(
        &rig,
        "workspaceFiles/readBytes",
        json!({ "workspaceFileScopeId": session, "path": "bytes.bin", "options": {} }),
    )
    .await;
    assert_eq!(whole["data"], "AAECAwQ=", "{whole}");
    assert_eq!(whole["eof"], true, "{whole}");
}

#[tokio::test]
async fn a_file_that_is_not_text_is_refused() {
    let rig = rig().await;
    let root = workspace("binary");
    std::fs::write(root.join("binary.bin"), [0u8, 65, 66]).expect("written");
    let session = session_for(&rig, &root).await;

    let code = code_of(
        &rig,
        "workspaceFiles/read",
        json!({ "workspaceFileScopeId": session, "path": "binary.bin", "range": {} }),
    )
    .await;
    assert_eq!(code, "workspace-file/not-text");
}

#[tokio::test]
async fn a_directory_listing_names_its_children() {
    let rig = rig().await;
    let root = workspace("list");
    std::fs::create_dir_all(root.join("src")).expect("created");
    write_lines(&root.join("src/main.rs"), "fn main() {}\n");
    let session = session_for(&rig, &root).await;

    let listing = value_of(
        &rig,
        "workspaceFiles/list",
        json!({ "workspaceFileScopeId": session, "path": "." }),
    )
    .await;
    assert_eq!(listing["path"], "", "{listing}");
    assert_eq!(listing["truncated"], false, "{listing}");
    let entries = listing["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 1, "{listing}");
    assert_eq!(entries[0]["name"], "src", "{listing}");
    assert_eq!(entries[0]["type"], "directory", "{listing}");

    let inside = value_of(
        &rig,
        "workspaceFiles/list",
        json!({ "workspaceFileScopeId": session, "path": "src" }),
    )
    .await;
    assert_eq!(inside["path"], "src", "{inside}");
    let entries = inside["entries"].as_array().expect("entries");
    assert_eq!(entries[0]["name"], "main.rs", "{inside}");
    assert_eq!(entries[0]["type"], "file", "{inside}");
    assert_eq!(entries[0]["size"], 13, "{inside}");
}

#[tokio::test]
async fn a_listing_outside_the_workspace_is_refused() {
    let rig = rig().await;
    let root = workspace("outside");
    let session = session_for(&rig, &root).await;

    let code = code_of(
        &rig,
        "workspaceFiles/list",
        json!({ "workspaceFileScopeId": session, "path": "/etc" }),
    )
    .await;
    assert_eq!(code, "workspace-file/outside-workspace");
}

#[tokio::test]
async fn a_session_without_a_workspace_is_not_found() {
    let rig = rig().await;
    let code = code_of(
        &rig,
        "workspaceFiles/stat",
        json!({ "workspaceFileScopeId": "session-nonesuch", "path": "notes.txt" }),
    )
    .await;
    assert_eq!(code, "gateway/lookup-not-found");
}

#[tokio::test]
async fn the_file_candidates_are_ranked() {
    let rig = rig().await;
    let root = workspace("candidates");
    std::fs::create_dir_all(root.join("src")).expect("created");
    write_lines(&root.join("src/main.rs"), "fn main() {}\n");
    write_lines(&root.join("notes.txt"), "notes\n");
    let session = session_for(&rig, &root).await;

    let candidates = value_of(
        &rig,
        "fileReferences/list",
        json!({ "agentId": session, "query": "main" }),
    )
    .await;
    let candidates = candidates.as_array().expect("candidates");
    assert_eq!(candidates.len(), 1, "{candidates:?}");
    assert_eq!(candidates[0]["path"], "src/main.rs", "{candidates:?}");
    assert_eq!(candidates[0]["kind"], "file", "{candidates:?}");
}

#[tokio::test]
async fn the_directory_browser_lists_its_directories() {
    let rig = rig().await;
    let root = workspace("picker");
    std::fs::create_dir_all(root.join("child")).expect("created");
    write_lines(&root.join("file.txt"), "text\n");

    let level = value_of(
        &rig,
        "directoryPicker/list",
        json!({ "path": root.to_string_lossy() }),
    )
    .await;
    assert_eq!(level["path"], root.to_string_lossy().as_ref(), "{level}");
    let entries = level["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 1, "{level}");
    assert_eq!(entries[0]["name"], "child", "{level}");
    let crumbs = level["crumbs"].as_array().expect("crumbs");
    assert!(crumbs.len() > 1, "{level}");
    assert_eq!(crumbs.last().unwrap()["path"], root.to_string_lossy().as_ref(), "{level}");
}

#[tokio::test]
async fn a_listing_without_a_path_is_the_home_directory_and_a_non_string_path_is_refused() {
    let rig = rig().await;
    let home = std::env::var("HOME").expect("tests run with a home directory");
    let level = value_of(&rig, "directoryPicker/list", json!({})).await;
    assert_eq!(level["path"], home.as_str(), "{level}");
    for junk in [json!(null), json!(5), json!(["/tmp"]), json!({})] {
        let code = code_of(&rig, "directoryPicker/list", json!({ "path": junk })).await;
        assert_eq!(code, "gateway/input-invalid", "{junk}");
    }
}

#[tokio::test]
async fn a_directory_is_created_under_the_listed_one() {
    let rig = rig().await;
    let root = workspace("create");

    let created = value_of(
        &rig,
        "directoryPicker/createDirectory",
        json!({ "path": root.to_string_lossy(), "name": "child" }),
    )
    .await;
    assert_eq!(created, json!(root.join("child").to_string_lossy()));
    assert!(root.join("child").is_dir());

    let refused = code_of(
        &rig,
        "directoryPicker/createDirectory",
        json!({ "path": root.to_string_lossy(), "name": "../escape" }),
    )
    .await;
    assert_eq!(refused, "gateway/input-invalid");
}

#[tokio::test]
async fn a_listing_of_a_file_is_refused() {
    let rig = rig().await;
    let root = workspace("not-a-directory");
    write_lines(&root.join("notes.txt"), "one\n");
    let session = session_for(&rig, &root).await;

    let code = code_of(
        &rig,
        "workspaceFiles/list",
        json!({ "workspaceFileScopeId": session, "path": "notes.txt" }),
    )
    .await;
    assert_eq!(code, "workspace-file/not-directory");
}

#[tokio::test]
async fn a_byte_window_above_the_cap_is_refused() {
    let rig = rig().await;
    let root = workspace("window");
    write_lines(&root.join("notes.txt"), "one\n");
    let session = session_for(&rig, &root).await;

    let code = code_of(
        &rig,
        "workspaceFiles/readBytes",
        json!({ "workspaceFileScopeId": session, "path": "notes.txt", "options": { "range": { "length": 3 * 1024 * 1024 } } }),
    )
    .await;
    assert_eq!(code, "workspace-file/too-large");
}

#[tokio::test]
async fn a_query_naming_a_parent_offers_nothing() {
    let rig = rig().await;
    let root = workspace("escape");
    write_lines(&root.join("notes.txt"), "one\n");
    let session = session_for(&rig, &root).await;

    let candidates = value_of(
        &rig,
        "fileReferences/list",
        json!({ "agentId": session, "query": "../" }),
    )
    .await;
    assert_eq!(candidates.as_array().expect("candidates").len(), 0, "{candidates}");
}

#[cfg(unix)]
#[tokio::test]
async fn a_symlink_is_not_followed_by_a_read() {
    let rig = rig().await;
    let root = workspace("symlink");
    write_lines(&root.join("notes.txt"), "one\n");
    std::os::unix::fs::symlink(root.join("notes.txt"), root.join("link.txt")).expect("linked");
    let session = session_for(&rig, &root).await;

    let code = code_of(
        &rig,
        "workspaceFiles/read",
        json!({ "workspaceFileScopeId": session, "path": "link.txt", "range": {} }),
    )
    .await;
    assert_eq!(code, "workspace-file/not-regular-file");
}

#[cfg(unix)]
#[tokio::test]
async fn a_symlinked_directory_outside_the_workspace_is_refused() {
    let rig = rig().await;
    let root = workspace("linked-directory");
    let outside = workspace("outside-directory");
    write_lines(&outside.join("secret.txt"), "secret\n");
    std::os::unix::fs::symlink(&outside, root.join("escape")).expect("linked");
    let session = session_for(&rig, &root).await;

    let code = code_of(
        &rig,
        "workspaceFiles/list",
        json!({ "workspaceFileScopeId": session, "path": "escape" }),
    )
    .await;
    assert_eq!(code, "workspace-file/outside-workspace");

    // The candidate search neither follows the link nor offers it.
    let candidates = value_of(
        &rig,
        "fileReferences/list",
        json!({ "agentId": session, "query": "secret" }),
    )
    .await;
    assert_eq!(candidates.as_array().expect("candidates").len(), 0, "{candidates}");

    // A query naming the linked directory lists nothing either.
    let linked = value_of(
        &rig,
        "fileReferences/list",
        json!({ "agentId": session, "query": "escape/" }),
    )
    .await;
    assert_eq!(linked.as_array().expect("candidates").len(), 0, "{linked}");
}