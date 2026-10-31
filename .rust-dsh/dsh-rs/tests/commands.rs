//! Slash commands through the gateway: what `commands/list` offers, how a line
//! is admitted, what the log records around a handler, and what each command
//! does to the session.

use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use cordis::plugin::BoxFuture;
use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView, LlmService};
use dsh_rs::attachments::{AttachmentStore, ATTACHMENTS_SERVICE};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::commands::Commands;
use dsh_rs::gui::images::ImageAdmission;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::images::{ImageConfig, ImageModels};
use dsh_rs::llm::{LlmAdapter, LlmError, StreamChunk};
use dsh_rs::session::SessionEventData;
use dsh_rs::types::{AgentOptions, ContentBlock, GenerateOptions, Message};
use serde_json::{json, Value};

mod common;

const TOKEN: &str = "commands-token-0123456789";

/// Answers from a queue (`Err` = provider failure) and records every request.
struct Scripted {
    replies: Mutex<VecDeque<Result<String, String>>>,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
    /// When set, each request waits for a permit before it answers.
    gate: Option<Arc<tokio::sync::Semaphore>>,
}

impl LlmAdapter for Scripted {
    fn name(&self) -> &'static str {
        "scripted"
    }

    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<dsh_rs::llm::BoxStream<StreamChunk>, LlmError>> {
        self.seen.lock().unwrap().push(options);
        let reply = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok("ok".into()));
        let gate = self.gate.clone();
        Box::pin(async move {
            if let Some(gate) = gate {
                gate.acquire().await.expect("the gate stays open").forget();
            }
            match reply {
                Ok(text) => Ok(dsh_rs::llm::runtime::stream_from_chunks(
                    common::text_response(&text),
                )),
                Err(message) => Err(LlmError::code("PROVIDER_DOWN", message)),
            }
        })
    }
}

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    agent: Arc<dyn AgentView>,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
    anonymous_user_file: std::path::PathBuf,
    _dirs: Vec<tempfile::TempDir>,
}

struct Setup {
    compaction: bool,
    store: bool,
    replies: Vec<Result<String, String>>,
    gate: Option<Arc<tokio::sync::Semaphore>>,
}

impl Default for Setup {
    fn default() -> Self {
        Setup {
            compaction: true,
            store: true,
            replies: Vec::new(),
            gate: None,
        }
    }
}

async fn rig(setup: Setup) -> Rig {
    let ctx = Context::new();
    let mut plugins = json!({});
    if setup.compaction {
        plugins["compaction"] =
            json!({ "triggerTokens": 50, "keepRecentTurns": 1, "minReclaimTokens": 1 });
    }
    install_profile(&ctx, &json!({ "plugins": plugins }))
        .await
        .unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let runtime = ctx.require::<LlmService>("llm").unwrap();
    runtime.unregister_adapter(&["mock"]);
    runtime
        .register_adapter(
            &["mock"],
            Arc::new(Scripted {
                replies: Mutex::new(setup.replies.into()),
                seen: seen.clone(),
                gate: setup.gate,
            }),
        )
        .unwrap();
    let agent = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .create(
            None,
            AgentOptions::mock("m"),
            Some("/tmp".into()),
            None,
            None,
        )
        .unwrap();
    let mut dirs = Vec::new();
    let mut images = ImageAdmission::default();
    if setup.store {
        let dir = tempfile::tempdir().unwrap();
        let store = AttachmentStore::open(dir.path()).unwrap();
        ctx.provide(ATTACHMENTS_SERVICE, store.clone())
            .await
            .unwrap();
        images = ImageAdmission::new(Some(store), &ImageConfig::default(), ImageModels::default());
        dirs.push(dir);
    }
    let home = tempfile::tempdir().unwrap();
    let anonymous_user_file = home.path().join("anonymous-user-id");
    let commands = Commands::new(Some(anonymous_user_file.clone()));
    dirs.push(home);
    let router = Router::new(RouterConfig {
        default_model: DefaultModel {
            provider: "mock".to_string(),
            model: "m".to_string(),
        },
        ..RouterConfig::default()
    })
    .with_images(Arc::new(images))
    .with_commands(Arc::new(commands));
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
        seen,
        anonymous_user_file,
        _dirs: dirs,
    }
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

async fn run(rig: &Rig, line: &str, attachments: Value) -> Value {
    call(
        rig,
        "commands/execute",
        json!({ "agentId": rig.agent.id(), "line": line, "submittedAttachments": attachments }),
    )
    .await
}

/// The result of a command that was admitted: its kind and text.
async fn outcome(rig: &Rig, line: &str) -> (String, String) {
    let reply = run(rig, line, json!([])).await;
    assert_eq!(reply["ok"], true, "{reply}");
    let result = &reply["value"]["result"];
    (
        result["kind"].as_str().unwrap().to_string(),
        result["text"].as_str().unwrap_or("").to_string(),
    )
}

fn events(rig: &Rig) -> Vec<SessionEventData> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .map(|event| event.data)
        .collect()
}

fn command_events(rig: &Rig) -> Vec<SessionEventData> {
    events(rig)
        .into_iter()
        .filter(|data| {
            matches!(
                data,
                SessionEventData::CommandRun { .. } | SessionEventData::CommandDone { .. }
            )
        })
        .collect()
}

fn png() -> String {
    let raster = image::RgbImage::from_fn(16, 16, |x, y| {
        image::Rgb([(x * 9) as u8, (y * 9) as u8, 40])
    });
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(raster)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    dsh_rs::gui::base64::encode(&out)
}

fn image_attachment() -> Value {
    json!({ "type": "image", "mediaType": "image/png", "data": png(), "name": "a.png" })
}

#[tokio::test]
async fn the_list_offers_what_this_build_can_run() {
    let rig = rig(Setup::default()).await;
    let reply = call(&rig, "commands/list", json!({ "agentId": rig.agent.id() })).await;
    let names: Vec<&str> = reply["value"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["compact", "feedback", "goal"]);
    let goal = &reply["value"][2];
    assert_eq!(goal["definitionId"], "@deepseek-ai/dsh-command-goal");
    assert_eq!(
        goal["input"],
        json!({ "hint": "[<objective>|clear|edit <objective>|pause|resume]", "attachments": true })
    );
    assert_eq!(
        reply["value"][0].get("input"),
        None,
        "a command without input has no descriptor"
    );
    assert_eq!(reply["value"][1]["input"], json!({ "hint": "<text>" }));
}

#[tokio::test]
async fn commands_are_listed_for_a_known_session_only() {
    let rig = rig(Setup::default()).await;
    let ghost = call(&rig, "commands/list", json!({ "agentId": "ghost" })).await;
    assert_eq!(ghost["error"]["code"], "session/not-found", "{ghost}");
    assert_eq!(ghost["error"]["details"], json!({ "sessionId": "ghost" }));
    let empty = call(&rig, "commands/list", json!({ "agentId": "" })).await;
    assert_eq!(empty["error"]["code"], "gateway/internal", "{empty}");
    let number = call(&rig, "commands/list", json!({ "agentId": 5 })).await;
    assert_eq!(number["error"]["code"], "gateway/input-invalid");
    assert_eq!(number["error"]["details"]["field"], "agentId");
    for (args, needle) in [
        (json!({}), "missing \"agentId\""),
        (json!({ "agent": rig.agent.id() }), "unexpected \"agent\""),
    ] {
        let reply = call(&rig, "commands/list", args.clone()).await;
        assert_eq!(
            reply["error"]["code"], "gateway/arguments-invalid",
            "{args}: {reply}"
        );
        assert!(
            reply["error"]["message"].as_str().unwrap().contains(needle),
            "{reply}"
        );
    }
}

#[tokio::test]
async fn compact_is_not_offered_or_run_without_the_compaction_plugin() {
    let rig = rig(Setup {
        compaction: false,
        ..Setup::default()
    })
    .await;
    let reply = call(&rig, "commands/list", json!({ "agentId": rig.agent.id() })).await;
    let names: Vec<&str> = reply["value"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["feedback", "goal"]);
    let reply = run(&rig, "/compact", json!([])).await;
    assert_eq!(
        reply,
        json!({ "ok": true }),
        "an unlisted command is an unknown one"
    );
    assert!(command_events(&rig).is_empty());
}

#[tokio::test]
async fn arguments_are_checked_as_the_typescript_host_checks_them() {
    let rig = rig(Setup::default()).await;
    let id = rig.agent.id();
    let code = |reply: &Value| {
        reply["error"]["code"]
            .as_str()
            .unwrap_or("none")
            .to_string()
    };
    let field = |reply: &Value| {
        reply["error"]["details"]["field"]
            .as_str()
            .unwrap_or("none")
            .to_string()
    };

    let reply = call(
        &rig,
        "commands/execute",
        json!({ "agentId": id, "line": "/goal", "submittedAttachments": [], "request": 1 }),
    )
    .await;
    assert_eq!(code(&reply), "gateway/arguments-invalid");
    assert!(reply["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unexpected \"request\""));
    let reply = call(&rig, "commands/execute", json!({})).await;
    assert!(reply["error"]["message"]
        .as_str()
        .unwrap()
        .contains("missing \"agentId\", \"line\", \"submittedAttachments\""));

    let reply = call(
        &rig,
        "commands/execute",
        json!({ "agentId": id, "line": 5, "submittedAttachments": [] }),
    )
    .await;
    assert_eq!(
        (code(&reply).as_str(), field(&reply).as_str()),
        ("gateway/input-invalid", "line")
    );
    for bad in [
        json!("x"),
        json!([{ "type": "x" }]),
        json!([null]),
        json!([{ "type": "file" }]),
        json!([{ "type": "image", "mediaType": "image/bmp", "data": "AA==" }]),
    ] {
        let reply = call(
            &rig,
            "commands/execute",
            json!({ "agentId": id, "line": "/goal", "submittedAttachments": bad }),
        )
        .await;
        assert_eq!(
            (code(&reply).as_str(), field(&reply).as_str()),
            ("gateway/input-invalid", "submittedAttachments"),
            "{bad}"
        );
    }
    // A ghost session is looked up after the arguments are checked.
    let reply = call(
        &rig,
        "commands/execute",
        json!({ "agentId": "ghost", "line": "hi", "submittedAttachments": [] }),
    )
    .await;
    assert_eq!(code(&reply), "session/not-found");
    let reply = call(
        &rig,
        "commands/execute",
        json!({ "agentId": "ghost", "line": 5, "submittedAttachments": [] }),
    )
    .await;
    assert_eq!(code(&reply), "gateway/input-invalid");
    assert!(
        command_events(&rig).is_empty(),
        "a refused call logs nothing"
    );
}

#[tokio::test]
async fn a_line_that_is_not_a_known_command_is_answered_with_no_value_and_logs_nothing() {
    let rig = rig(Setup::default()).await;
    for line in [
        "",
        "hello",
        "/",
        "/nope",
        " /goal",
        "/Feedback x",
        "/goal-x",
        "/goal\u{a0}x",
        "/export",
        "/plan",
        "/permission x",
    ] {
        let reply = run(&rig, line, json!([])).await;
        assert_eq!(reply, json!({ "ok": true }), "{line:?}");
    }
    assert!(command_events(&rig).is_empty());
}

#[tokio::test]
async fn the_log_brackets_a_handler_with_run_and_done_under_one_id() {
    let rig = rig(Setup::default()).await;
    let reply = run(&rig, "/goal ship it", json!([])).await;
    let id = reply["value"]["commandId"].as_str().unwrap().to_string();
    assert!(
        id.starts_with("cmd-") && id.matches('-').count() == 2,
        "{id}"
    );
    let kinds: Vec<String> = events(&rig)
        .iter()
        .map(|d| d.event_type().to_string())
        .collect();
    assert_eq!(kinds, ["command/run", "goal/change", "command/done"]);
    match &events(&rig)[0] {
        SessionEventData::CommandRun {
            command_id,
            name,
            args,
        } => {
            assert_eq!(
                (command_id, name.as_str(), args.as_deref()),
                (&id, "goal", Some(" ship it"))
            );
        }
        other => panic!("{other:?}"),
    }
    match &events(&rig)[2] {
        SessionEventData::CommandDone {
            command_id,
            success,
            text,
            source_event_seq,
        } => {
            assert_eq!(command_id, &id);
            assert!(*success);
            assert!(text
                .as_deref()
                .unwrap()
                .starts_with("Goal created\nStatus: active"));
            assert_eq!(*source_event_seq, None);
        }
        other => panic!("{other:?}"),
    }
    let next = run(&rig, "/goal", json!([])).await["value"]["commandId"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(next, id, "each run gets its own id");
}

#[tokio::test]
async fn feedback_is_recorded_without_its_text_in_the_run_event() {
    let rig = rig(Setup::default()).await;
    let (kind, text) = outcome(&rig, "/feedback").await;
    assert_eq!(kind, "error");
    assert_eq!(text, "Feedback text is required. Usage: /feedback <text>");
    assert!(!events(&rig)
        .iter()
        .any(|d| matches!(d, SessionEventData::FeedbackRecord { .. })));

    let (kind, text) = outcome(&rig, "/feedback \t the plan  worked \n").await;
    assert_eq!(kind, "success");
    let user = std::fs::read_to_string(&rig.anonymous_user_file).unwrap();
    assert_eq!(
        text,
        format!(
            "Feedback recorded for session {}\nAnonymous user: {}.",
            rig.agent.id(),
            user.trim()
        )
    );
    let recorded: Vec<_> = events(&rig)
        .into_iter()
        .filter_map(|d| match d {
            SessionEventData::FeedbackRecord { text, category } => Some((text, category)),
            _ => None,
        })
        .collect();
    assert_eq!(recorded, [(Some("the plan  worked".to_string()), None)]);
    let runs: Vec<_> = command_events(&rig)
        .into_iter()
        .filter_map(|d| match d {
            SessionEventData::CommandRun { name, args, .. } => Some((name, args)),
            _ => None,
        })
        .collect();
    assert_eq!(
        runs,
        [
            ("feedback".to_string(), None),
            ("feedback".to_string(), None)
        ]
    );
}

#[tokio::test]
async fn the_anonymous_user_id_is_minted_once_and_kept_in_the_named_file() {
    let rig = rig(Setup::default()).await;
    assert!(!rig.anonymous_user_file.exists());
    let first = outcome(&rig, "/feedback a").await.1;
    let second = outcome(&rig, "/feedback b").await.1;
    let id = |text: &str| {
        text.rsplit("Anonymous user: ")
            .next()
            .unwrap()
            .trim_end_matches('.')
            .to_string()
    };
    assert_eq!(id(&first), id(&second));
    let stored = std::fs::read_to_string(&rig.anonymous_user_file).unwrap();
    assert_eq!(stored.trim(), id(&first));
    assert_eq!(stored.matches('\n').count(), 1, "a bare line");
    assert_eq!(
        stored.trim().split('-').map(str::len).collect::<Vec<_>>(),
        [8, 4, 4, 4, 12]
    );
    assert_eq!(&stored.trim()[14..15], "4", "a version-4 UUID");
}

#[tokio::test]
async fn an_existing_anonymous_user_id_is_reused_and_a_corrupt_one_is_replaced() {
    let rig = rig(Setup::default()).await;
    std::fs::write(
        &rig.anonymous_user_file,
        " 6f14a7bf-63e6-46f0-9160-c61fd77706a1 \n",
    )
    .unwrap();
    let text = outcome(&rig, "/feedback hi").await.1;
    assert!(
        text.ends_with("Anonymous user: 6f14a7bf-63e6-46f0-9160-c61fd77706a1."),
        "{text}"
    );

    let rig = self::rig(Setup::default()).await;
    std::fs::write(&rig.anonymous_user_file, "not a uuid").unwrap();
    // The corrupt file stays: the id is kept for the run and the file is not overwritten.
    let text = outcome(&rig, "/feedback hi").await.1;
    assert!(text.contains("Anonymous user: "), "{text}");
    assert_eq!(
        std::fs::read_to_string(&rig.anonymous_user_file).unwrap(),
        "not a uuid"
    );
}

#[tokio::test]
async fn goal_walks_through_its_states_with_the_typescript_texts() {
    let rig = rig(Setup::default()).await;
    let usage = "Usage: /goal [<objective>|clear|edit <objective>|pause|resume]";
    let go = |line: &'static str| outcome(&rig, line);
    assert_eq!(
        go("/goal").await,
        (
            "success".into(),
            format!("No goal is currently set.\n{usage}")
        )
    );
    assert_eq!(
        go("/goal clear").await,
        ("success".into(), "No goal to clear.".into())
    );
    for action in ["pause", "resume"] {
        let line = format!("/goal {action}");
        assert_eq!(
            outcome(&rig, &line).await,
            (
                "error".into(),
                format!("No goal is currently set; /goal {action} requires one. {usage}")
            ),
        );
    }
    assert_eq!(
        go("/goal edit x").await.1,
        format!("No goal is currently set; /goal edit requires one. {usage}")
    );
    assert_eq!(
        go("/goal edit").await.1,
        format!("Goal editing requires a replacement objective.\n{usage}")
    );

    let created = go("/goal build it").await;
    assert_eq!(
        created,
        ("success".into(), "Goal created\nStatus: active\nObjective: build it\nRounds: 0/256\nActivation: armed\n\nCommands: /goal edit <objective>, /goal pause, /goal clear".into())
    );
    assert_eq!(
        go("/goal again").await,
        ("error".into(), "A goal is already active. Use /goal edit <objective> to change it or /goal clear before replacing it.".into())
    );
    assert!(go("/goal   EDIT  upper")
        .await
        .1
        .starts_with("Goal updated\nStatus: active\nObjective: upper\n"));
    let paused = go("/goal pause").await.1;
    assert!(
        paused.starts_with("Goal paused\nStatus: paused\n")
            && paused.contains("Activation: disarmed"),
        "{paused}"
    );
    assert!(paused.ends_with("Commands: /goal edit <objective>, /goal resume, /goal clear"));
    assert_eq!(
        go("/goal pause").await,
        ("error".into(), "The goal command is not valid for the current state. Run /goal to view available commands.".into()),
        "pausing a paused goal is refused by the goal domain"
    );
    assert!(go("/goal resume")
        .await
        .1
        .starts_with("Goal resumed\nStatus: active"));
    assert!(
        go("/goal editor")
            .await
            .1
            .starts_with("A goal is already active."),
        "`editor` is an objective, not an edit"
    );
    assert_eq!(
        go("/goal CLEAR").await,
        ("success".into(), "Goal cleared.".into())
    );
    assert_eq!(
        go("/goal").await.1,
        format!("No goal is currently set.\n{usage}")
    );
}

#[tokio::test]
async fn a_command_and_the_goal_endpoints_see_the_same_goal() {
    let rig = rig(Setup::default()).await;
    outcome(&rig, "/goal via command").await;
    let got = call(&rig, "goals/get", json!({ "agentId": rig.agent.id() })).await;
    assert_eq!(got["value"]["objective"], "via command");
    assert_eq!(got["value"]["activation"], "armed");
    let goal = got["value"].clone();
    let paused = call(&rig, "goals/pause", json!({ "agentId": rig.agent.id(), "ref": { "id": goal["id"], "revision": goal["revision"] } })).await;
    assert_eq!(paused["ok"], true, "{paused}");
    let shown = outcome(&rig, "/goal").await.1;
    assert!(
        shown.starts_with("Goal\nStatus: paused") && shown.contains("Activation: disarmed"),
        "{shown}"
    );
}

#[tokio::test]
async fn a_completed_goal_is_replaced_by_create_and_by_edit() {
    let rig = rig(Setup::default()).await;
    outcome(&rig, "/goal first").await;
    let goal = call(&rig, "goals/get", json!({ "agentId": rig.agent.id() })).await["value"].clone();
    let done = call(&rig, "goals/complete", json!({ "agentId": rig.agent.id(), "ref": { "id": goal["id"], "revision": goal["revision"] } })).await;
    assert_eq!(done["ok"], true, "{done}");
    let shown = outcome(&rig, "/goal").await.1;
    assert!(shown.starts_with("Goal\nStatus: complete"), "{shown}");
    assert!(
        shown.ends_with("Commands: /goal <objective>, /goal clear"),
        "{shown}"
    );
    for line in ["/goal pause", "/goal resume"] {
        assert!(
            outcome(&rig, line)
                .await
                .1
                .starts_with("The goal command is not valid"),
            "{line}"
        );
    }
    assert!(outcome(&rig, "/goal edit second")
        .await
        .1
        .starts_with("Goal created\nStatus: active\nObjective: second"));
}

#[tokio::test]
async fn goal_attachments_are_stored_and_follow_the_goal_to_the_model() {
    let rig = rig(Setup::default()).await;
    let reply = run(&rig, "/goal with pictures", json!([image_attachment(), { "type": "image", "mediaType": "image/png", "data": png(), "name": "b.png" }])).await;
    assert_eq!(reply["value"]["result"]["kind"], "success", "{reply}");
    rig.agent.when_idle().await;
    let prompts: Vec<Message> = events(&rig)
        .into_iter()
        .filter_map(|d| match d {
            SessionEventData::UserMessage { message } => Some(message),
            _ => None,
        })
        .collect();
    let message = prompts
        .iter()
        .find(|m| {
            m.content
                .iter()
                .any(|b| matches!(b, ContentBlock::Image { .. }))
        })
        .expect("a follow-up with the images");
    assert_eq!(message.content.len(), 3);
    assert!(matches!(message.content[0], ContentBlock::Image { .. }));
    assert!(matches!(message.content[1], ContentBlock::Image { .. }));
    assert_eq!(
        message.text(),
        "Reference attachments for the goal objective."
    );
}

#[tokio::test]
async fn attachments_are_refused_before_a_handler_that_cannot_use_them_and_never_stored() {
    let rig = rig(Setup::default()).await;
    let objects = rig._dirs[0].path().join("objects");
    let stored = || {
        std::fs::read_dir(&objects)
            .map(|shards| {
                shards
                    .flatten()
                    .map(|s| std::fs::read_dir(s.path()).unwrap().count())
                    .sum::<usize>()
            })
            .unwrap_or(0)
    };
    for (line, expected) in [
        ("/feedback hi", "/feedback does not accept attachments"),
        ("/compact", "/compact does not accept attachments"),
    ] {
        let reply = run(&rig, line, json!([image_attachment()])).await;
        assert_eq!(
            reply["value"]["result"],
            json!({ "kind": "error", "text": expected }),
            "{line}"
        );
    }
    assert_eq!(
        stored(),
        0,
        "a command that takes no attachments never stores them"
    );
    assert!(!events(&rig)
        .iter()
        .any(|d| matches!(d, SessionEventData::FeedbackRecord { .. })));

    // /goal takes attachments, so they are admitted before its grammar is read.
    let message =
        "Attachments only accompany a goal objective: /goal <objective> or /goal edit <objective>.";
    for line in ["/goal pause", "/goal   "] {
        let reply = run(&rig, line, json!([image_attachment()])).await;
        assert_eq!(
            reply["value"]["result"],
            json!({ "kind": "error", "text": message }),
            "{line}"
        );
    }
    assert_eq!(stored(), 1, "the same image is one object");
    assert!(!events(&rig)
        .iter()
        .any(|d| matches!(d, SessionEventData::GoalChange { .. })));
}

#[tokio::test]
async fn bad_attachments_settle_as_an_error_result_inside_the_run_and_done_pair() {
    let rig = rig(Setup::default()).await;
    let bad_image = json!({ "type": "image", "mediaType": "image/png", "data": "!!" });
    let reply = run(&rig, "/goal x", json!([bad_image])).await;
    assert_eq!(
        reply["value"]["result"],
        json!({ "kind": "error", "text": "Image upload is not canonical base64." })
    );
    let reply = run(
        &rig,
        "/goal x",
        json!([{ "type": "file", "receiptId": "nope" }]),
    )
    .await;
    assert_eq!(
        reply["value"]["result"],
        json!({ "kind": "error", "text": "File upload receipt is unknown for this session." })
    );
    let kinds: Vec<String> = command_events(&rig)
        .iter()
        .map(|d| d.event_type().to_string())
        .collect();
    assert_eq!(
        kinds,
        ["command/run", "command/done", "command/run", "command/done"]
    );
    assert!(!events(&rig)
        .iter()
        .any(|d| matches!(d, SessionEventData::GoalChange { .. })));
}

#[tokio::test]
async fn without_an_attachment_store_attachments_are_refused_by_name() {
    let rig = rig(Setup {
        store: false,
        ..Setup::default()
    })
    .await;
    let reply = run(&rig, "/goal x", json!([image_attachment()])).await;
    assert_eq!(
        reply["value"]["result"],
        json!({ "kind": "error", "text": "/goal: attachments are unavailable because no attachment store is composed" })
    );
}

#[tokio::test]
async fn compact_reports_what_it_replaced_and_where_the_summary_is() {
    let big = "0123456789 this is a long message padding the history out. ".repeat(8);
    let rig = rig(Setup {
        replies: vec![
            Ok("answer one".into()),
            Ok("answer two".into()),
            Ok("SUMMARY: two things were asked".into()),
        ],
        ..Setup::default()
    })
    .await;
    assert_eq!(
        outcome(&rig, "/compact").await,
        ("success".into(), "No compactable history yet.".into())
    );
    for (id, text) in [
        ("u1", format!("first {big}")),
        ("u2", format!("second {big}")),
    ] {
        rig.agent
            .followup(Message::user(id, vec![ContentBlock::text(text)]));
        rig.agent.when_idle().await;
    }
    let reply = run(&rig, "/compact", json!([])).await;
    let result = &reply["value"]["result"];
    assert_eq!(result["kind"], "success", "{reply}");
    let text = result["text"].as_str().unwrap();
    assert!(
        text.starts_with("Compacted ")
            && text.contains(" history items (~")
            && text.ends_with(" tokens)."),
        "{text}"
    );
    let summary_seq = events_with_seq(&rig)
        .into_iter()
        .find(|(_, d)| matches!(d, SessionEventData::Compaction { .. }))
        .unwrap()
        .0;
    assert_eq!(result["sourceEventSeq"], summary_seq);
    // The done event carries the same pointer, so a reload shows it too.
    let done = command_events(&rig)
        .into_iter()
        .rev()
        .find_map(|d| match d {
            SessionEventData::CommandDone {
                source_event_seq, ..
            } => Some(source_event_seq),
            _ => None,
        });
    assert_eq!(done, Some(Some(summary_seq)));
    let _ = rig.seen.lock().unwrap().len();
}

fn events_with_seq(rig: &Rig) -> Vec<(u64, SessionEventData)> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .map(|e| (e.seq, e.data))
        .collect()
}

#[tokio::test]
async fn compact_with_arguments_is_a_usage_error_and_a_failed_summary_is_an_error_not_a_crash() {
    let big = "0123456789 this is a long message padding the history out. ".repeat(8);
    let rig = rig(Setup {
        replies: vec![Ok("one".into()), Ok("two".into()), Err("model down".into())],
        ..Setup::default()
    })
    .await;
    assert_eq!(
        outcome(&rig, "/compact now").await,
        ("error".into(), "Usage: /compact (no arguments)".into())
    );
    assert_eq!(
        outcome(&rig, "/compact   ").await.0,
        "success",
        "blank input is no input"
    );
    for (id, text) in [
        ("u1", format!("first {big}")),
        ("u2", format!("second {big}")),
    ] {
        rig.agent
            .followup(Message::user(id, vec![ContentBlock::text(text)]));
        rig.agent.when_idle().await;
    }
    let (kind, text) = outcome(&rig, "/compact").await;
    assert_eq!(kind, "error");
    assert!(text.starts_with("Compaction failed: "), "{text}");
    assert!(
        !events(&rig)
            .iter()
            .any(|d| matches!(d, SessionEventData::Compaction { .. })),
        "history is left alone"
    );
    let done = command_events(&rig).into_iter().last();
    assert!(
        matches!(
            done,
            Some(SessionEventData::CommandDone { success: false, .. })
        ),
        "{done:?}"
    );
}

#[tokio::test]
async fn compact_refuses_while_the_agent_is_running_a_turn() {
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let rig = rig(Setup {
        gate: Some(gate.clone()),
        ..Setup::default()
    })
    .await;
    rig.agent
        .followup(Message::user("u1", vec![ContentBlock::text("work")]));
    for _ in 0..200 {
        if rig.agent.driver_busy() && rig.seen.lock().unwrap().len() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(rig.agent.driver_busy(), "the turn is held at the model");
    let (kind, text) = outcome(&rig, "/compact").await;
    assert_eq!(
        (kind.as_str(), text.as_str()),
        ("error", "Compaction is unavailable because this process has an active compaction, or the agent is not idle.")
    );
    assert!(!events(&rig)
        .iter()
        .any(|d| matches!(d, SessionEventData::Compaction { .. })));
    gate.add_permits(8);
    rig.agent.when_idle().await;
    assert_eq!(
        outcome(&rig, "/compact").await.0,
        "success",
        "idle again, so it runs"
    );
}

#[tokio::test]
async fn command_events_survive_a_reload_of_the_stored_log() {
    let rig = rig(Setup::default()).await;
    outcome(&rig, "/goal persisted").await;
    outcome(&rig, "/feedback keep").await;
    let page = call(
        &rig,
        "session/page",
        json!({ "request": { "address": { "kind": "session", "sessionId": rig.agent.id() } } }),
    )
    .await;
    let records = page["value"]["records"].as_array().unwrap();
    let kinds: Vec<&str> = records
        .iter()
        .map(|r| r["event"]["type"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        [
            "command/run",
            "goal/change",
            "command/done",
            "command/run",
            "feedback/record",
            "command/done"
        ]
    );
    let run = &records[3]["event"]["data"];
    assert_eq!(run["name"], "feedback");
    assert_eq!(
        run.get("args"),
        None,
        "feedback text lives in feedback/record only"
    );
    assert_eq!(run["source"], json!({ "kind": "user" }));
    let done = &records[2]["event"]["data"];
    assert_eq!(done["kind"], "success");
    assert!(done["text"].as_str().unwrap().starts_with("Goal created"));
}
