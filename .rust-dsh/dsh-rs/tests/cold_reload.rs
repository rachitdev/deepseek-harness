//! Cold reload: a session one run stored is found again by the next run.

use std::path::{Path, PathBuf};
use std::time::Duration;

use std::sync::{Arc, Mutex};

use cordis::Context;
use dsh_rs::api::services::{
    AgentRegistryService, BoxFuture, LlmAdapterApi, LlmService, SessionPersistenceApi, SessionService,
};
use dsh_rs::bundle::install_profile;
use dsh_rs::core::AgentOptions;
use dsh_rs::session::store::SESSIONS_SERVICE;
use dsh_rs::types::{
    stream_from_chunks, BoxStream, GenerateOptions, LlmError, SessionEvent, SessionEventData, StreamChunk,
};
use serde_json::json;

/// One assistant text-block response.
fn text_response(text: &str) -> Vec<StreamChunk> {
    vec![
        StreamChunk::BlockStart {
            index: 0,
            block_type: "text".to_string(),
        },
        StreamChunk::TextDelta {
            index: 0,
            text: text.to_string(),
        },
        StreamChunk::BlockEnd {
            index: 0,
            block: dsh_rs::llm::ContentBlock::Text {
                text: text.to_string(),
            },
        },
        StreamChunk::Finish {
            reason: dsh_rs::llm::FinishReason::Stop,
        },
    ]
}

/// A profile that keeps its session logs in `dir`.
fn profile(dir: &Path) -> serde_json::Value {
    json!({
        "config": {
            "store_dir": dir.to_string_lossy(),
            "store_format": "v4-plain"
        }
    })
}

/// The stored log of `id`, once it is on disk.
fn log_path(dir: &Path, id: &str) -> Option<PathBuf> {
    let mut found = None;
    for project in std::fs::read_dir(dir).ok()?.flatten() {
        let candidate = project.path().join(id).join("session.v4.jsonl");
        if candidate.is_file() {
            found = Some(candidate);
        }
    }
    found
}

/// Wait for the session log to reach the disk.
async fn wait_for_log(dir: &Path, id: &str) -> Option<PathBuf> {
    for _ in 0..200 {
        if let Some(path) = log_path(dir, id) {
            return Some(path);
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    None
}

/// Wait until the agent's turn has ended.
async fn wait_for_turn(agent: &dyn dsh_rs::api::services::AgentView) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let ended = agent
            .session()
            .events()
            .iter()
            .any(|event| matches!(event.data, SessionEventData::TurnEnd { .. }));
        if ended {
            return;
        }
        assert!(std::time::Instant::now() < deadline, "the turn ends");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dsh-cold-reload-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// The log of a session reaches the disk when its turn ends, so the next run
/// can find it.
#[tokio::test]
async fn a_turn_writes_the_session_log_to_the_disk() {
    let dir = scratch("checkpoint");
    let ctx = Context::new();
    install_profile(&ctx, &profile(&dir)).await.unwrap();

    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("mock-1"),
            Some("/tmp".to_string()),
            None,
            None,
        )
        .unwrap();
    let id = agent.id().to_string();
    agent.followup(dsh_rs::core::agent::user_message_with_text("m1", "hello"));
    wait_for_turn(agent.as_ref()).await;

    let path = wait_for_log(&dir, &id)
        .await
        .expect("the turn writes the session log");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("hello"), "the log holds the prompt");
}

/// A resumed session keeps the agent preset it ran with.
#[tokio::test]
async fn a_resumed_session_keeps_the_preset_it_ran_with() {
    let dir = scratch("preset");
    let ctx = Context::new();
    install_profile(&ctx, &profile(&dir)).await.unwrap();
    dsh_rs::gui::router::mount_preset_prompts(&ctx, &dsh_rs::gui::router::default_presets())
        .expect("the roster mounts");

    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("mock-1"),
            Some("/tmp".to_string()),
            None,
            Some("default".to_string()),
        )
        .unwrap();
    let id = agent.id().to_string();
    agent.followup(dsh_rs::core::agent::user_message_with_text("m1", "hello"));
    wait_for_turn(agent.as_ref()).await;
    let store = ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.flush(&id).await.unwrap();

    let next = Context::new();
    install_profile(&next, &profile(&dir)).await.unwrap();
    dsh_rs::gui::router::mount_preset_prompts(&next, &dsh_rs::gui::router::default_presets())
        .expect("the roster mounts");
    dsh_rs::session::resume_stored_sessions(&next, AgentOptions::mock("mock-1")).await;
    let agents = next
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let restored = agents
        .get(&id)
        .expect("the resumed session is live again");
    assert_eq!(
        restored.session().agent_preset().as_deref(),
        Some("default"),
        "the stored preset survives the restart"
    );
}

/// A roster that offers the default preset and one more.
fn roster_with_reviewer() -> Vec<dsh_rs::gui::router::AgentPreset> {
    let mut presets = dsh_rs::gui::router::default_presets();
    presets.push(dsh_rs::gui::router::AgentPreset {
        id: "reviewer".to_string(),
        name: "Reviewer".to_string(),
        description: None,
        is_default: false,
        system_prompt: Some("You review code.".to_string()),
    });
    presets
}

/// A stored session whose preset the deployment does not offer is not
/// adopted: it would run a composition its log does not name.
#[tokio::test]
async fn a_resumed_session_whose_preset_is_gone_is_not_adopted() {
    let dir = scratch("preset-gone");
    let ctx = Context::new();
    install_profile(&ctx, &profile(&dir)).await.unwrap();
    dsh_rs::gui::router::mount_preset_prompts(&ctx, &roster_with_reviewer())
        .expect("the roster mounts");

    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("mock-1"),
            Some("/tmp".to_string()),
            None,
            Some("reviewer".to_string()),
        )
        .unwrap();
    let id = agent.id().to_string();
    let store = ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.flush(&id).await.unwrap();

    // The deployment no longer offers the preset the log names.
    let next = Context::new();
    install_profile(&next, &profile(&dir)).await.unwrap();
    dsh_rs::gui::router::mount_preset_prompts(&next, &dsh_rs::gui::router::default_presets())
        .expect("the roster mounts");
    let outcome = dsh_rs::session::resume_stored_sessions(&next, AgentOptions::mock("mock-1")).await;
    assert_eq!(outcome.failed, 1, "the session is reported as not resumed");
    assert_eq!(outcome.resumed, 0, "no session is adopted");
}

/// A session that changed its preset keeps that preset across a restart: the
/// log says which composition ran, not the header it was created with.
#[tokio::test]
async fn a_resumed_session_keeps_the_preset_it_changed_to() {
    let dir = scratch("preset-change");
    let ctx = Context::new();
    install_profile(&ctx, &profile(&dir)).await.unwrap();
    dsh_rs::gui::router::mount_preset_prompts(&ctx, &roster_with_reviewer())
        .expect("the roster mounts");

    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("mock-1"),
            Some("/tmp".to_string()),
            None,
            Some("default".to_string()),
        )
        .unwrap();
    let id = agent.id().to_string();
    assert_eq!(
        agent.session().select_preset("reviewer", None).as_deref(),
        Some("reviewer"),
        "a blank session changes its preset"
    );
    let store = ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.flush(&id).await.unwrap();

    let next = Context::new();
    install_profile(&next, &profile(&dir)).await.unwrap();
    dsh_rs::gui::router::mount_preset_prompts(&next, &roster_with_reviewer())
        .expect("the roster mounts");
    dsh_rs::session::resume_stored_sessions(&next, AgentOptions::mock("mock-1")).await;
    let agents = next
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let restored = agents
        .get(&id)
        .expect("the resumed session is live again");
    assert_eq!(
        restored.session().agent_preset().as_deref(),
        Some("reviewer"),
        "the recorded preset survives the restart"
    );
}

/// The session a run stored is adopted again by the next run, with its
/// history and the route it last used.
#[tokio::test]
async fn a_stored_session_is_found_again_by_the_next_run() {
    let dir = scratch("resume");
    let ctx = Context::new();
    install_profile(&ctx, &profile(&dir)).await.unwrap();

    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("mock-1"),
            Some("/tmp".to_string()),
            None,
            None,
        )
        .unwrap();
    let id = agent.id().to_string();
    agent.followup(dsh_rs::core::agent::user_message_with_text("m1", "hello"));
    wait_for_turn(agent.as_ref()).await;
    let store = ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.flush(&id).await.unwrap();

    // The next run boots over the same store.
    let next = Context::new();
    install_profile(&next, &profile(&dir)).await.unwrap();
    let resumed = dsh_rs::session::resume_stored_sessions(
        &next,
        AgentOptions::mock("another-model"),
    )
    .await;
    assert_eq!(resumed.resumed, 1, "the stored session is resumed");

    let agents = next
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let restored = agents
        .get(&id)
        .expect("the resumed session is live again");
    // A fresh session does not take the id a resumed session keeps.
    let fresh = agents
        .create(None, AgentOptions::mock("mock-1"), None, None, None)
        .unwrap();
    assert_ne!(fresh.id(), id, "the resumed id is not minted again");
    let events = restored.session().events();
    assert!(
        events
            .iter()
            .any(|event| matches!(event.data, SessionEventData::UserMessage { .. })),
        "the resumed session keeps its history"
    );
    assert_eq!(
        restored.options().model,
        "mock-1",
        "the resumed session keeps the route it used, not the deployment default"
    );
}

/// A backend whose flush always fails.
struct FailingBackend;

impl SessionPersistenceApi for FailingBackend {
    fn on_event(&self, _session: &str, _event: &SessionEvent) {}

    fn flush(&self, _session: &str) -> std::io::Result<()> {
        Err(std::io::Error::other("the disk is full"))
    }

    fn load(&self, _session: &str) -> std::io::Result<Vec<SessionEvent>> {
        Ok(Vec::new())
    }

    fn list(&self) -> Vec<String> {
        Vec::new()
    }
}

/// A model that counts its calls.
struct Counting {
    calls: Arc<Mutex<usize>>,
}

impl LlmAdapterApi for Counting {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn stream(
        &self,
        _options: GenerateOptions,
    ) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>> {
        *self.calls.lock().unwrap() += 1;
        Box::pin(async move { Ok(stream_from_chunks(text_response("ok"))) })
    }
}

/// A checkpoint that fails stops the turn before the model is called: the
/// logged request prefix must be durable before the request goes out.
#[tokio::test]
async fn a_failed_checkpoint_stops_the_turn_before_the_model_is_called() {
    let ctx = Context::new();
    install_profile(&ctx, &json!({})).await.unwrap();
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    let calls = Arc::new(Mutex::new(0usize));
    llm.register_adapter(
        &["mock"],
        Arc::new(Counting {
            calls: calls.clone(),
        }),
    )
    .unwrap();

    let store = ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.attach_persistence(Arc::new(FailingBackend));

    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("mock-1"), None, None, None)
        .unwrap();
    agent.followup(dsh_rs::core::agent::user_message_with_text("m1", "hello"));

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let ended = agent
            .session()
            .events()
            .iter()
            .any(|event| matches!(event.data, SessionEventData::TurnEnd { .. }));
        if ended {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the turn ends"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    assert_eq!(*calls.lock().unwrap(), 0, "the model is not called");
    let failure = agent
        .session()
        .events()
        .into_iter()
        .find_map(|event| match event.data {
            SessionEventData::TurnEnd {
                reason: dsh_rs::types::TurnEndReason::Error { message, .. },
                ..
            } => Some(message),
            _ => None,
        })
        .expect("the turn ends with the checkpoint failure");
    assert!(failure.contains("the disk is full"), "{failure}");
}

/// A session that stored only a header is still a session: it is found again.
#[tokio::test]
async fn a_session_with_only_a_header_is_found_again() {
    let dir = scratch("header-only");
    let ctx = Context::new();
    install_profile(&ctx, &profile(&dir)).await.unwrap();
    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("mock-1"),
            Some("/tmp".to_string()),
            None,
            None,
        )
        .unwrap();
    let id = agent.id().to_string();
    let store = ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.flush(&id).await.unwrap();

    let next = Context::new();
    install_profile(&next, &profile(&dir)).await.unwrap();
    let resumed = dsh_rs::session::resume_stored_sessions(&next, AgentOptions::mock("mock-1")).await;
    assert_eq!(resumed.resumed, 1, "the empty session is resumed");
    let agents = next
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    assert!(agents.get(&id).is_some(), "the session is live again");
}

/// The resumed session keeps the working directory its log records.
#[tokio::test]
async fn a_resumed_session_keeps_its_stored_working_directory() {
    let dir = scratch("cwd");
    let ctx = Context::new();
    install_profile(&ctx, &profile(&dir)).await.unwrap();
    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("mock-1"),
            Some("/tmp/dsh-cold-reload-workspace".to_string()),
            None,
            None,
        )
        .unwrap();
    let id = agent.id().to_string();
    agent.followup(dsh_rs::core::agent::user_message_with_text("m1", "hello"));
    let store = ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.flush(&id).await.unwrap();

    let next = Context::new();
    install_profile(&next, &profile(&dir)).await.unwrap();
    let resumed = dsh_rs::session::resume_stored_sessions(&next, AgentOptions::mock("mock-1")).await;
    assert_eq!(resumed.resumed, 1);
    let agents = next
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let restored = agents.get(&id).expect("the session is live again");
    assert_eq!(
        restored.session().header_cwd().as_deref(),
        Some("/tmp/dsh-cold-reload-workspace"),
        "the resumed session keeps its working directory"
    );
}

/// A backend whose stored logs cannot be read.
struct UnreadableBackend;

impl SessionPersistenceApi for UnreadableBackend {
    fn on_event(&self, _session: &str, _event: &SessionEvent) {}

    fn flush(&self, _session: &str) -> std::io::Result<()> {
        Ok(())
    }

    fn load(&self, _session: &str) -> std::io::Result<Vec<SessionEvent>> {
        Err(std::io::Error::other("the log is corrupt"))
    }

    fn list(&self) -> Vec<String> {
        vec!["session-1".to_string()]
    }
}

/// A stored log that cannot be read is reported, not skipped in silence: the
/// session is not resumed and the failure names the session.
#[tokio::test]
async fn a_stored_log_that_cannot_be_read_is_not_resumed() {
    let ctx = Context::new();
    install_profile(&ctx, &json!({})).await.unwrap();
    ctx.provide(
        dsh_rs::api::services::SESSION_PERSISTENCE_SERVICE,
        Arc::new(UnreadableBackend) as Arc<dyn SessionPersistenceApi>,
    )
    .await
    .unwrap();

    let resumed = dsh_rs::session::resume_stored_sessions(&ctx, AgentOptions::mock("mock-1")).await;
    assert_eq!(resumed.resumed, 0, "the unreadable session is not resumed");
    assert_eq!(resumed.failed, 1, "the unreadable session is reported");
    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    assert!(agents.get("session-1").is_none(), "no session is resumed");
}
