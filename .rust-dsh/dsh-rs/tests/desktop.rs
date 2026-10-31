//! The desktop endpoints: opening a workspace path and listing the
//! applications registered for it, through the gateway.

use std::sync::{Arc, Mutex};

use cordis::Context;
use dsh_rs::api::services::BoxFuture;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::desktop::{CommandRunner, Desktop, Platform};
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use serde_json::{json, Value};

const TOKEN: &str = "desktop-token-0123456789";

/// Records each command and answers from a script.
struct Script {
    calls: Mutex<Vec<(String, Vec<String>)>>,
    answers: Mutex<Vec<Result<String, String>>>,
}

impl CommandRunner for Script {
    fn run(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<String, String>> {
        self.calls.lock().unwrap().push((program.to_string(), args));
        let mut answers = self.answers.lock().unwrap();
        let answer = if answers.is_empty() {
            Err("no answer scripted".to_string())
        } else {
            answers.remove(0)
        };
        Box::pin(async move { answer })
    }

    fn launch(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<(), String>> {
        let answered = self.run(program, args);
        Box::pin(async move { answered.await.map(drop) })
    }
}

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    script: Arc<Script>,
}

async fn rig(platform: Platform, answers: Vec<Result<String, String>>) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .expect("profile installs");
    let script = Arc::new(Script {
        calls: Mutex::new(Vec::new()),
        answers: Mutex::new(answers),
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
    })
    .with_desktop(Arc::new(Desktop::new(
        platform,
        true,
        None,
        None,
        script.clone(),
    )));
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
        script,
    }
}

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
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    let envelope: Value = serde_json::from_slice(&reply.body).expect("the reply is JSON");
    envelope["result"].clone()
}

fn canonical_temp() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let real = std::fs::canonicalize(dir.path()).unwrap();
    std::fs::write(real.join("a.txt"), "x").unwrap();
    (dir, real)
}

fn handlers() -> String {
    json!([
        { "id": "/Apps/Editor.app", "name": "Editor", "default": true, "icon": "data:image/png;base64,AAAA", "bundle": "x.editor", "version": "1" },
        { "id": "/Apps/Editor copy.app", "name": "Editor", "default": false, "icon": "data:image/png;base64,AAAA", "bundle": "x.editor", "version": "2" },
        { "id": "/Apps/Viewer.app", "name": "Viewer", "default": false, "icon": null, "bundle": null, "version": null },
    ])
    .to_string()
}

#[tokio::test]
async fn the_host_says_whether_it_can_open_paths() {
    let rig = rig(Platform::Mac, vec![]).await;
    assert_eq!(
        call(&rig, "session/canOpenWorkspacePath", json!({})).await,
        json!({ "ok": true, "value": true })
    );
    let bare = rig_without_desktop().await;
    assert_eq!(
        call(&bare, "session/canOpenWorkspacePath", json!({})).await,
        json!({ "ok": true, "value": false })
    );
    let refused = call(&rig, "session/canOpenWorkspacePath", json!({ "x": 1 })).await;
    assert_eq!(refused["error"]["code"], "gateway/arguments-invalid");
}

async fn rig_without_desktop() -> Rig {
    rig(Platform::Other, vec![]).await
}

#[tokio::test]
async fn applications_list_collapses_copies_and_keeps_the_default() {
    let (_dir, real) = canonical_temp();
    let rig = rig(Platform::Mac, vec![Ok(handlers())]).await;
    let path = real.join("a.txt").display().to_string();
    let result = call(&rig, "session/workspacePathApplications", json!({ "request": { "path": path } })).await;
    assert_eq!(
        result,
        json!({ "ok": true, "value": [
            { "id": "/Apps/Editor.app", "name": "Editor", "default": true, "icon": "data:image/png;base64,AAAA" },
            { "id": "/Apps/Viewer.app", "name": "Viewer", "default": false, "icon": null },
        ]}),
        "the copy of the default application is folded away and no grouping fields leak"
    );
    let calls = rig.script.calls.lock().unwrap();
    assert_eq!(calls[0].0, "osascript");
    assert_eq!(calls[0].1.last().map(String::as_str), Some("icons"));
    assert_eq!(calls[0].1[calls[0].1.len() - 2], path, "the path is an argument, not script source");
}

#[tokio::test]
async fn applications_list_is_empty_when_the_host_has_no_desktop() {
    let rig = rig_without_desktop().await;
    let result = call(&rig, "session/workspacePathApplications", json!({ "request": { "path": "/definitely/missing" } })).await;
    assert_eq!(result, json!({ "ok": true, "value": [] }));
    assert!(rig.script.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_malformed_request_is_refused_before_anything_runs() {
    let rig = rig(Platform::Mac, vec![]).await;
    for (method, payload) in [
        ("session/workspacePathApplications", json!({})),
        ("session/workspacePathApplications", json!({ "request": {} })),
        ("session/workspacePathApplications", json!({ "request": { "path": 7 } })),
        ("session/openWorkspacePath", json!({ "request": { "path": "/p", "action": "nope" } })),
        ("session/openWorkspacePath", json!({ "request": { "path": "/p", "application": 5 } })),
        ("session/openWorkspacePath", json!({ "request": { "path": "/p", "action": null } })),
        ("session/openWorkspacePath", json!({ "request": { "path": "/p", "application": null } })),
        ("session/openWorkspacePath", json!({ "request": { "path": "/p", "action": "reveal", "application": 5 } })),
    ] {
        let result = call(&rig, method, payload.clone()).await;
        assert_eq!(result["ok"], false, "{method} {payload}");
        assert!(
            ["gateway/input-invalid", "gateway/arguments-invalid"]
                .contains(&result["error"]["code"].as_str().unwrap()),
            "{method} {payload}: {result}"
        );
    }
    assert!(rig.script.calls.lock().unwrap().is_empty(), "no native command ran");
}

#[tokio::test]
async fn an_unverifiable_path_is_a_bad_request_and_nothing_opens() {
    let rig = rig(Platform::Mac, vec![]).await;
    let empty = call(&rig, "session/openWorkspacePath", json!({ "request": { "path": "" } })).await;
    assert_eq!(empty["error"]["code"], "gateway/bad-request");
    assert_eq!(empty["error"]["message"], "A non-empty file path is required");

    let (_dir, real) = canonical_temp();
    std::os::unix::fs::symlink(real.join("a.txt"), real.join("link.txt")).unwrap();
    let path = real.join("link.txt").display().to_string();
    for method in ["session/openWorkspacePath", "session/workspacePathApplications"] {
        let result = call(&rig, method, json!({ "request": { "path": path } })).await;
        assert_eq!(result["error"]["code"], "gateway/bad-request", "{method}");
        assert_eq!(result["error"]["message"], "Path has no verified Host path", "{method}");
    }
    assert!(rig.script.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn opening_a_verified_path_uses_the_requested_way() {
    let (_dir, real) = canonical_temp();
    let path = real.join("a.txt").display().to_string();

    let rig = rig(Platform::Mac, vec![Ok(String::new())]).await;
    let opened = call(&rig, "session/openWorkspacePath", json!({ "request": { "path": path } })).await;
    assert_eq!(opened, json!({ "ok": true, "value": { "opened": true } }));
    assert_eq!(rig.script.calls.lock().unwrap()[0], ("open".to_string(), vec![path.clone()]));

    let rig = rig_with(vec![Ok(String::new())]).await;
    call(&rig, "session/openWorkspacePath", json!({ "request": { "path": path, "action": "reveal" } })).await;
    assert_eq!(rig.script.calls.lock().unwrap()[0].1, ["-R".to_string(), path.clone()]);

    let rig = rig_with(vec![Ok(handlers()), Ok(String::new())]).await;
    let chosen = call(&rig, "session/openWorkspacePath", json!({ "request": { "path": path, "application": "/Apps/Editor copy.app" } })).await;
    assert_eq!(chosen["ok"], true, "a collapsed copy stays openable: {chosen}");
    assert_eq!(
        rig.script.calls.lock().unwrap()[1].1,
        ["-a".to_string(), "/Apps/Editor copy.app".to_string(), path.clone()]
    );

    let rig = rig_with(vec![Ok(handlers())]).await;
    let stale = call(&rig, "session/openWorkspacePath", json!({ "request": { "path": path, "application": "/Apps/Gone.app" } })).await;
    assert_eq!(stale["error"]["code"], "gateway/internal");
    assert_eq!(stale["error"]["message"], "path open failed", "the cause stays in the host log");
    assert_eq!(rig.script.calls.lock().unwrap().len(), 1, "an unregistered application is never launched");
}

async fn rig_with(answers: Vec<Result<String, String>>) -> Rig {
    rig(Platform::Mac, answers).await
}

#[tokio::test]
async fn a_failing_opener_reports_a_generic_failure() {
    let (_dir, real) = canonical_temp();
    let path = real.join("a.txt").display().to_string();
    let rig = rig(Platform::Mac, vec![Err("open exited with 1: secret detail".to_string())]).await;
    let result = call(&rig, "session/openWorkspacePath", json!({ "request": { "path": path } })).await;
    assert_eq!(result["error"]["code"], "gateway/internal");
    assert_eq!(result["error"]["message"], "path open failed");
    assert!(!result.to_string().contains("secret detail"));

    let none = rig_without_desktop().await;
    let result = call(&none, "session/openWorkspacePath", json!({ "request": { "path": path } })).await;
    assert_eq!(result["error"]["message"], "path open failed", "a host with no desktop cannot open anything");
    assert!(none.script.calls.lock().unwrap().is_empty());
}
