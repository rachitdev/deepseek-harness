//! Permission presets: the `/permission` command, the preset catalog, the
//! `permissions` projection, and the sandbox and approval enforcement that
//! follows from a session's knobs.

use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView, ToolsService};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::plugins::approvals::{ApprovalBroker, APPROVALS_SERVICE};
use dsh_rs::session::SessionEventData;
use dsh_rs::types::{AgentOptions, CancelToken, ToolExecutionResult, ToolRunContext};
use serde_json::{json, Value};

const TOKEN: &str = "permission-token-0123456789";

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    agent: Arc<dyn AgentView>,
    tools: ToolsService,
    workspace: tempfile::TempDir,
}

async fn rig(plugins: Value) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": plugins })).await.unwrap();
    let workspace = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let agent = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .create(
            None,
            AgentOptions::mock("m"),
            Some(workspace.path().to_string_lossy().into_owned()),
            None,
            None,
        )
        .unwrap();
    let router = Router::new(RouterConfig {
        default_model: DefaultModel { provider: "mock".into(), model: "m".into() },
        ..RouterConfig::default()
    });
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.into() }, None));
    let tools = (*ctx.require::<ToolsService>("tools").unwrap()).clone();
    Rig { ctx, gateway, agent, tools, workspace }
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": args },
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

async fn command(rig: &Rig, line: &str) -> (String, String) {
    let reply = call(
        rig,
        "commands/execute",
        json!({ "agentId": rig.agent.id(), "line": line, "submittedAttachments": [] }),
    )
    .await;
    assert_eq!(reply["ok"], true, "{reply}");
    let result = &reply["value"]["result"];
    (
        result["kind"].as_str().unwrap().to_string(),
        result["text"].as_str().unwrap_or("").to_string(),
    )
}

async fn projection(rig: &Rig, key: &str) -> Value {
    let reply = call(
        rig,
        "session/projections",
        json!({ "request": { "sessionId": rig.agent.id() } }),
    ).await;
    reply["value"]["values"][key].clone()
}

fn knob_events(rig: &Rig) -> Vec<String> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .filter_map(|event| match event.data {
            SessionEventData::PermissionPreset { preset } => Some(format!("preset {preset}")),
            SessionEventData::SandboxMode { mode } => Some(format!("sandbox {mode}")),
            SessionEventData::ApprovalPolicy { policy, .. } => Some(format!("approval {policy}")),
            _ => None,
        })
        .collect()
}

async fn run_tool(rig: &Rig, name: &str, args: Value) -> ToolExecutionResult {
    let run = ToolRunContext {
        ctx: rig.ctx.clone(),
        signal: CancelToken::new(),
        agent_id: Some(rig.agent.id().to_string()),
        cwd: Some(rig.workspace.path().to_string_lossy().into_owned()),
    };
    rig.tools.execute("c1".into(), name.into(), args, run).await
}

fn denial(result: &ToolExecutionResult) -> String {
    match result {
        ToolExecutionResult::Error { code, message, .. } => {
            assert_eq!(code, "DENIED");
            message.clone()
        }
        other => panic!("expected a denial, got {other:?}"),
    }
}

#[tokio::test]
async fn without_the_plugin_the_command_is_not_offered() {
    let rig = rig(json!({})).await;
    let list = call(&rig, "commands/list", json!({ "agentId": rig.agent.id() })).await;
    let names: Vec<&str> = list["value"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"permission"));
    let catalog = call(&rig, "permissionPresets/catalog", json!({})).await;
    assert_eq!(catalog["value"]["defaultPreset"], "auto");
    assert_eq!(projection(&rig, "permissions").await, json!({ "currentValue": "auto" }));
}

#[tokio::test]
async fn the_command_is_listed_like_the_typescript_one() {
    let rig = rig(json!({ "permission": {} })).await;
    let list = call(&rig, "commands/list", json!({ "agentId": rig.agent.id() })).await;
    let item = list["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "permission")
        .expect("offered")
        .clone();
    assert_eq!(item["definitionId"], "@deepseek-ai/dsh-permission-presets");
    assert_eq!(
        item["description"],
        "Switch the permission preset (sandbox mode + approval policy)"
    );
    assert_eq!(item["input"], json!({ "hint": "<preset>" }));
}

#[tokio::test]
async fn the_catalog_lists_the_configured_presets() {
    let rig = rig(json!({ "permission": {} })).await;
    let catalog = call(&rig, "permissionPresets/catalog", json!({})).await;
    let value = &catalog["value"];
    let values: Vec<&str> = value["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["value"].as_str().unwrap())
        .collect();
    assert_eq!(values, ["profile", "workspace-write", "danger-full-access"]);
    assert_eq!(value["defaultPreset"], "profile");
    assert_eq!(value["defaultOptions"], value["options"]);
    assert_eq!(value["options"][2]["description"], "Full file access without approval prompts.");
}

#[tokio::test]
async fn a_new_session_reports_the_default_preset() {
    let rig = rig(json!({ "permission": {} })).await;
    assert_eq!(projection(&rig, "permissions").await, json!({ "currentValue": "profile" }));
    let (kind, text) = command(&rig, "/permission").await;
    assert_eq!(kind, "success");
    assert_eq!(
        text,
        "current preset profile (available: profile, workspace-write, danger-full-access)"
    );
}

#[tokio::test]
async fn switching_records_the_preset_then_each_changed_knob() {
    let rig = rig(json!({ "permission": {} })).await;
    let (kind, text) = command(&rig, "/permission danger-full-access").await;
    assert_eq!((kind.as_str(), text.as_str()), ("success", "preset danger-full-access"));
    // Sandbox is already danger-full-access, so only the approval knob moves.
    assert_eq!(knob_events(&rig), ["preset danger-full-access", "approval never"]);
    assert_eq!(
        projection(&rig, "permissions").await,
        json!({ "currentValue": "danger-full-access" })
    );
    command(&rig, "/permission workspace-write").await;
    assert_eq!(
        knob_events(&rig)[2..],
        ["preset workspace-write", "sandbox workspace-write", "approval ask"]
    );
}

#[tokio::test]
async fn selecting_the_preset_in_force_appends_nothing() {
    let rig = rig(json!({ "permission": {} })).await;
    command(&rig, "/permission profile").await;
    assert!(knob_events(&rig).is_empty());
}

#[tokio::test]
async fn an_unknown_preset_is_refused_with_the_available_names() {
    let rig = rig(json!({ "permission": {} })).await;
    let (kind, text) = command(&rig, "/permission yolo").await;
    assert_eq!(kind, "error");
    assert_eq!(
        text,
        "unknown preset \"yolo\" (available: profile, workspace-write, danger-full-access)"
    );
    assert!(knob_events(&rig).is_empty());
}

#[tokio::test]
async fn knobs_that_match_no_preset_read_custom() {
    let rig = rig(json!({ "permission": {} })).await;
    rig.agent.session().append(SessionEventData::SandboxMode { mode: "read-only".into() });
    assert_eq!(projection(&rig, "permissions").await, json!({ "currentValue": "custom" }));
}

#[tokio::test]
async fn a_bad_table_stops_the_profile_with_the_reason() {
    let ctx = Context::new();
    let Err(err) = install_profile(
        &ctx,
        &json!({ "plugins": { "permission": { "presets": [
            { "id": "a", "sandbox": "moat", "approval": "ask" }
        ] } } }),
    )
    .await
    else {
        panic!("a bad table must stop the profile");
    };
    assert!(err.contains("moat"), "{err}");
}

#[tokio::test]
async fn read_only_refuses_every_mutating_tool() {
    let rig = rig(json!({ "permission": {
        "sandbox": "read-only",
        "presets": [{ "id": "ro", "sandbox": "read-only", "approval": "ask" }],
    } })).await;
    let target = rig.workspace.path().join("a.txt");
    for (tool, args) in [
        ("write_file", json!({ "path": target, "content": "x" })),
        ("edit_file", json!({ "path": target, "old_string": "a", "new_string": "b" })),
        ("bash", json!({ "command": "true" })),
    ] {
        let message = denial(&run_tool(&rig, tool, args).await);
        assert!(message.contains("read-only"), "{message}");
    }
    assert!(!target.exists());
    // Reading is still allowed.
    std::fs::write(&target, "hello").unwrap();
    let read = run_tool(&rig, "read_file", json!({ "path": target })).await;
    assert!(!read.is_error());
}

#[tokio::test]
async fn workspace_write_keeps_file_changes_inside_the_workspace() {
    let rig = rig(json!({ "permission": {}, "approvals": { "ask": [], "default": "allow" } })).await;
    command(&rig, "/permission workspace-write").await;
    let inside = rig.workspace.path().join("ok.txt");
    let wrote = run_tool(&rig, "write_file", json!({ "path": inside, "content": "x" })).await;
    assert!(!wrote.is_error(), "{wrote:?}");
    let relative = run_tool(&rig, "write_file", json!({ "path": "rel.txt", "content": "x" })).await;
    assert!(!relative.is_error());
    // A path that climbs out of the workspace is refused, however it is spelled.
    let escape = rig.workspace.path().join("..").join("escaped.txt");
    let message = denial(&run_tool(&rig, "write_file", json!({ "path": escape, "content": "x" })).await);
    assert!(message.contains("outside the workspace"), "{message}");
    let message = denial(&run_tool(&rig, "write_file", json!({ "path": "/etc/dsh-nope", "content": "x" })).await);
    assert!(message.contains("workspace-write"), "{message}");
    // A link inside the workspace cannot lead out of it.
    let outside = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    std::os::unix::fs::symlink(outside.path(), rig.workspace.path().join("link")).unwrap();
    let message = denial(
        &run_tool(&rig, "write_file", json!({ "path": "link/x.txt", "content": "x" })).await,
    );
    assert!(message.contains("outside the workspace"), "{message}");
    assert!(!outside.path().join("x.txt").exists());
    // The temporary directory is a permitted root.
    let scratch = std::env::temp_dir().join("dsh-permission-scratch.txt");
    let wrote = run_tool(&rig, "write_file", json!({ "path": scratch, "content": "x" })).await;
    assert!(!wrote.is_error(), "{wrote:?}");
    let _ = std::fs::remove_file(scratch);
}

#[tokio::test]
async fn approval_never_stops_the_questions_but_not_the_deny_list() {
    let rig = rig(json!({
        "permission": {},
        "approvals": { "ask": ["bash"], "deny": ["glob"], "default": "allow", "timeoutMs": 5000 },
    })).await;
    let broker = (*rig.ctx.require::<ApprovalBroker>(APPROVALS_SERVICE).unwrap()).clone();
    let mut inbox = broker.attach();
    // Under `ask` the bash call waits for a person.
    let waiting = tokio::spawn({
        let (tools, ctx, agent) = (rig.tools.clone(), rig.ctx.clone(), rig.agent.id().to_string());
        async move {
            let run = ToolRunContext { ctx, signal: CancelToken::new(), agent_id: Some(agent), cwd: None };
            tools.execute("c1".into(), "bash".into(), json!({ "command": "true" }), run).await
        }
    });
    let pending = tokio::time::timeout(Duration::from_secs(5), inbox.questions.recv())
        .await
        .expect("a question arrives")
        .expect("the inbox is open");
    pending.answer(dsh_rs::plugins::approvals::ApprovalDecision::Deny);
    assert!(waiting.await.unwrap().is_error());

    command(&rig, "/permission danger-full-access").await;
    let ran = run_tool(&rig, "bash", json!({ "command": "echo quiet" })).await;
    assert!(!ran.is_error(), "{ran:?}");
    assert!(inbox.questions.try_recv().is_err(), "nobody was asked");
    let message = denial(&run_tool(&rig, "glob", json!({ "pattern": "*" })).await);
    assert!(message.contains("approvals policy"), "{message}");
}

#[tokio::test]
async fn workspace_write_with_no_one_to_approve_refuses_the_shell() {
    let rig = rig(json!({ "permission": {
        "presets": [{ "id": "locked", "sandbox": "workspace-write", "approval": "never" }],
    } })).await;
    command(&rig, "/permission locked").await;
    let message = denial(&run_tool(&rig, "bash", json!({ "command": "true" })).await);
    assert!(message.contains("cannot confine a shell"), "{message}");
}

#[tokio::test]
async fn a_link_inside_the_workspace_cannot_lead_a_write_out_of_it() {
    let rig = rig(json!({ "permission": {}, "approvals": { "default": "allow" } })).await;
    command(&rig, "/permission workspace-write").await;
    let outside = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ws = rig.workspace.path();
    // A dangling link whose target does not exist yet.
    std::os::unix::fs::symlink(outside.path().join("new.txt"), ws.join("dangling")).unwrap();
    let message = denial(&run_tool(&rig, "write_file", json!({ "path": "dangling", "content": "x" })).await);
    assert!(message.contains("outside the workspace"), "{message}");
    assert!(!outside.path().join("new.txt").exists());
    // `link/..` leaves the workspace even though it folds to a name inside it.
    std::os::unix::fs::symlink(outside.path(), ws.join("link")).unwrap();
    let message = denial(&run_tool(&rig, "write_file", json!({ "path": "link/../x.txt", "content": "x" })).await);
    assert!(message.contains("outside the workspace"), "{message}");
    assert!(!ws.join("x.txt").exists());
}

#[tokio::test]
async fn a_file_call_without_a_path_is_refused_under_workspace_write() {
    let rig = rig(json!({ "permission": {}, "approvals": { "default": "allow" } })).await;
    command(&rig, "/permission workspace-write").await;
    let message = denial(&run_tool(&rig, "write_file", json!({ "content": "x" })).await);
    assert!(message.contains("names no path"), "{message}");
}

#[tokio::test]
async fn the_sandbox_holds_without_the_approvals_plugin() {
    let rig = rig(json!({ "permission": {
        "sandbox": "read-only",
        "presets": [{ "id": "ro", "sandbox": "read-only", "approval": "ask" }],
    } })).await;
    let message = denial(&run_tool(&rig, "bash", json!({ "command": "true" })).await);
    assert!(message.contains("read-only"), "{message}");
}

#[tokio::test]
async fn workspace_write_refuses_the_shell_when_nothing_can_approve_it() {
    let rig = rig(json!({ "permission": {} })).await;
    command(&rig, "/permission workspace-write").await;
    let message = denial(&run_tool(&rig, "bash", json!({ "command": "true" })).await);
    assert!(message.contains("no approvals plugin"), "{message}");
}

#[tokio::test]
async fn a_delegated_agent_starts_under_its_parents_permissions() {
    let rig = rig(json!({ "permission": {} })).await;
    command(&rig, "/permission workspace-write").await;
    let child = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .create(None, AgentOptions::mock("m"), None, None, None)
        .unwrap();
    dsh_rs::permission::Permissions::inherit(&*rig.agent.session(), &*child.session());
    let table = dsh_rs::permission::provided(&rig.ctx).unwrap();
    assert_eq!(table.current(&child.session().events()), "workspace-write");
}

#[tokio::test]
async fn a_switch_is_written_tightening_first() {
    let rig = rig(json!({ "permission": { "presets": [
        { "id": "open", "sandbox": "danger-full-access", "approval": "never" },
        { "id": "shut", "sandbox": "read-only", "approval": "ask" },
        { "id": "mixed", "sandbox": "danger-full-access", "approval": "ask" },
    ] } })).await;
    command(&rig, "/permission open").await;
    command(&rig, "/permission shut").await;
    let log = knob_events(&rig);
    // Leaving `open` for `shut` tightens both; neither order may loosen one.
    assert_eq!(log[log.len() - 3..], ["preset shut", "sandbox read-only", "approval ask"]);
    command(&rig, "/permission mixed").await;
    let log = knob_events(&rig);
    // Loosening the sandbox while tightening approval writes approval first.
    assert_eq!(log[log.len() - 2..], ["preset mixed", "sandbox danger-full-access"]);
}
