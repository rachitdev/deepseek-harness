//! TypeScript-DSH bridge plugin (`ts-dsh-bridge`).
//!
//! Makes the TypeScript DeepSeek Harness ecosystem (its packages, plugins,
//! profiles and tools) reachable from dsh-rs as one plugin. The protocol work
//! is delegated to the [`deepseek-harness-sdk`](https://github.com/42ch-dev/dsh-rust-sdk)
//! crate (Apache-2.0): it spawns the TS runtime under a named profile, runs
//! the `initialize` handshake, matches each prompt's inbox receipt, collects
//! events until the root session is idle, and tears the runtime down in
//! stages (shutdown request, EOF, SIGTERM, SIGKILL).
//!
//! The bridge registers one tool, `dsh_ecosystem_run`, which delegates a task
//! to a TS-DSH session and returns the final assistant text and finish
//! reason. It does not load TypeScript plugins natively: the TS runtime keeps
//! its own tools, approval policy and sandbox, and runs unattended (approval
//! policy `ask` has no answerer on this channel, so approval-gated tools fail
//! closed). The runtime starts on first use and stops when the plugin unloads.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use cordis::Plugin;
use deepseek_harness_sdk::{Config, DeepSeekHarness, Input};
use serde::Deserialize;
use serde_json::json;

use super::{register_tools, tool_spec};
use crate::types::ToolExecutionResult;

/// Configuration for the `ts-dsh-bridge` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BridgeConfig {
    /// The TS `dsh` executable (e.g. `apps/cli/lib/bin.js`, a Node script).
    /// Empty falls back to `DSH_RUNTIME_BIN` in the environment.
    pub dsh_bin: String,
    /// TS DSH profile to boot (`dsh --profile <name>`).
    pub profile: String,
    /// Overlay patches, applied in order after the profile.
    pub patches: Vec<PathBuf>,
    /// Harness home of the TS runtime (`DSH_HOME`); empty uses the SDK's default.
    pub dsh_home: String,
    /// Extra environment for the runtime, layered over the inherited one.
    pub env: HashMap<String, String>,
    /// Working directory recorded on TS sessions.
    pub session_cwd: String,
    /// Provider route and model every TS session runs on.
    pub provider: String,
    pub model: String,
    pub startup_timeout_ms: u64,
    pub turn_timeout_ms: u64,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        BridgeConfig {
            dsh_bin: String::new(),
            profile: "sdk".into(),
            patches: Vec::new(),
            dsh_home: String::new(),
            env: HashMap::new(),
            session_cwd: ".".into(),
            provider: "omniroute".into(),
            model: "auto/best-coding".into(),
            startup_timeout_ms: 60_000,
            turn_timeout_ms: 600_000,
        }
    }
}

impl BridgeConfig {
    fn to_sdk(&self) -> Config {
        let non_empty = |s: &String| (!s.is_empty()).then(|| s.clone());
        Config {
            provider: self.provider.clone(),
            model: self.model.clone(),
            cwd: Some(PathBuf::from(&self.session_cwd)),
            dsh_bin: non_empty(&self.dsh_bin),
            profile: self.profile.clone(),
            patches: self.patches.clone(),
            dsh_home: non_empty(&self.dsh_home).map(PathBuf::from),
            env: (!self.env.is_empty()).then(|| self.env.clone()),
            initialize_timeout: Some(Duration::from_millis(self.startup_timeout_ms)),
            ..Config::default()
        }
    }
}

/// Lifecycle of the lazily started runtime. `Closed` is terminal: once the
/// plugin unloads, a tool call that was already in flight cannot start a new
/// runtime that nothing would ever stop.
enum Runtime {
    NotStarted,
    Running(Arc<DeepSeekHarness>),
    Closed,
}

type Slot = Arc<tokio::sync::Mutex<Runtime>>;

/// Unload signal: `true` once the plugin is unloading. Turns and start-ups
/// select on it so unload never waits for them to finish on their own.
type Unload = tokio::sync::watch::Receiver<bool>;

async fn unloaded(unload: &mut Unload) {
    let _ = unload.wait_for(|gone| *gone).await;
}

async fn runtime(
    slot: &Slot,
    config: &BridgeConfig,
    unload: &mut Unload,
) -> Result<Arc<DeepSeekHarness>, String> {
    let mut guard = slot.lock().await;
    match &*guard {
        Runtime::Running(started) => return Ok(Arc::clone(started)),
        Runtime::Closed => return Err("the ts-dsh-bridge plugin is unloaded".to_string()),
        Runtime::NotStarted => {}
    }
    // Dropping the start-up future on unload drops the half-started client, whose
    // child is killed on drop.
    let harness = tokio::select! {
        started = DeepSeekHarness::start(config.to_sdk()) => started.map_err(|e| e.to_string())?,
        _ = unloaded(unload) => return Err("the ts-dsh-bridge plugin is unloading".to_string()),
    };
    let started = Arc::new(harness);
    *guard = Runtime::Running(Arc::clone(&started));
    Ok(started)
}

/// Stop the runtime. In-flight turns hold a clone of the `Arc`. Unload is
/// signalled so they abandon their turn and release it, the slot is marked
/// `Closed` so none can start another, and the runtime is then closed. A turn
/// that still holds the runtime after `grace` keeps it alive until it ends
/// (the SDK kills the child when the last clone drops).
async fn shut_down(slot: &Slot, unload: &tokio::sync::watch::Sender<bool>, grace: Duration) {
    // Signal first: in-flight turns drop their clone of the runtime, so the
    // wait below normally ends at once instead of at the grace limit.
    let _ = unload.send(true);
    let mut started = match std::mem::replace(&mut *slot.lock().await, Runtime::Closed) {
        Runtime::Running(started) => started,
        _ => return,
    };
    let deadline = tokio::time::Instant::now() + grace;
    loop {
        match Arc::try_unwrap(started) {
            Ok(mut harness) => {
                if let Err(e) = harness.close().await {
                    eprintln!("ts-dsh-bridge: close reported: {e}");
                }
                return;
            }
            Err(still_shared) => {
                if tokio::time::Instant::now() >= deadline {
                    eprintln!(
                        "ts-dsh-bridge: a turn still holds the runtime {} ms after unload; it stops when that turn ends",
                        grace.as_millis()
                    );
                    return;
                }
                started = still_shared;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}

/// How a delegated turn ended.
enum TurnEnd<T> {
    Done(T),
    TimedOut,
    Unloaded,
}

/// Run `turn` until it finishes, `limit` passes, or the plugin unloads. Losing
/// the race drops `turn`, which is the only cancellation the SDK offers; the
/// remote side may still be working, so callers report the outcome as unknown.
async fn race_turn<T>(
    turn: impl std::future::Future<Output = T>,
    limit: Duration,
    unload: &mut Unload,
) -> TurnEnd<T> {
    tokio::select! {
        finished = tokio::time::timeout(limit, turn) => match finished {
            Ok(value) => TurnEnd::Done(value),
            Err(_) => TurnEnd::TimedOut,
        },
        _ = unloaded(unload) => TurnEnd::Unloaded,
    }
}

/// A session id unique across runs (the TS runtime persists sessions).
///
/// The wall clock alone cannot guarantee that: its resolution can be coarser
/// than the gap between two calls (microseconds on macOS), so two sessions
/// started back to back would collide. The counter separates calls within one
/// process; the timestamp and pid separate processes.
fn fresh_session_id() -> String {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("dsh-rs-{nanos:x}-{:x}-{sequence:x}", std::process::id())
}

/// The `ts-dsh-bridge` plugin. Requires the `tools` service.
pub fn bridge_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "ts-dsh-bridge",
        vec![cordis::Injection::new(
            crate::api::TOOLS_SERVICE.to_string(),
        )],
        |ctx, config: BridgeConfig| async move {
            let slot: Slot = Arc::new(tokio::sync::Mutex::new(Runtime::NotStarted));
            let (unload_tx, unload_rx) = tokio::sync::watch::channel(false);
            let turn_timeout = Duration::from_millis(config.turn_timeout_ms);
            let launch = config.clone();
            let tool_slot = Arc::clone(&slot);

            let spec = tool_spec(
                "dsh_ecosystem_run",
                "Delegate a task to the full TypeScript DeepSeek Harness ecosystem (its plugins, tools and profiles) and return its final answer. Use for work that needs a capability dsh-rs does not have natively. Runs unattended: approval-gated actions fail closed.",
                json!({
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "The task, fully self-contained." },
                        "session": { "type": "string", "description": "Session id returned by an earlier call, to continue that delegated conversation. Omit to start a new one." }
                    },
                    "required": ["prompt"]
                }),
                move |args| {
                    let slot = Arc::clone(&tool_slot);
                    let launch = launch.clone();
                    let mut unload = unload_rx.clone();
                    async move {
                        let Some(prompt) = args.get("prompt").and_then(|v| v.as_str()) else {
                            return ToolExecutionResult::error("BAD_ARGS", "prompt is required");
                        };
                        let harness = match runtime(&slot, &launch, &mut unload).await {
                            Ok(harness) => harness,
                            Err(e) => return ToolExecutionResult::error("BRIDGE_START", e),
                        };
                        // The TS runtime persists sessions, so a fixed id would collide on
                        // the next run: a fresh call gets a fresh id; a supplied id resumes.
                        let requested = args.get("session").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
                        let session_id = requested.map(str::to_string).unwrap_or_else(fresh_session_id);
                        // Resume (or start) and run share one deadline and the unload signal.
                        let turn = async {
                            let session = match requested {
                                Some(id) => harness.resume_session(id).await.map_err(|e| ("BRIDGE_RESUME", e.to_string()))?,
                                None => harness.start_session(Some(session_id.clone())),
                            };
                            session.run(Input::Text(prompt.to_string()), None).await.map_err(|e| ("BRIDGE_RUN", e.to_string()))
                        };
                        let run = race_turn(turn, turn_timeout, &mut unload).await;
                        match run {
                            TurnEnd::Unloaded => ToolExecutionResult::error(
                                "BRIDGE_UNLOADED",
                                format!("the ts-dsh-bridge plugin unloaded during the turn; the outcome of work it already started is unknown (session {session_id})"),
                            ),
                            TurnEnd::TimedOut => ToolExecutionResult::error(
                                "BRIDGE_TIMEOUT",
                                format!(
                                    "the TS DSH turn exceeded {} ms and was abandoned here; the outcome of work it already started is unknown (session {session_id})",
                                    turn_timeout.as_millis()
                                ),
                            ),
                            TurnEnd::Done(Err((code, message))) => ToolExecutionResult::error(code, message),
                            TurnEnd::Done(Ok(result)) => ToolExecutionResult::success_text(
                                result.final_response,
                                json!({ "session": session_id, "finishReason": result.finish_reason }),
                            ),
                        }
                    }
                },
            );
            register_tools(&ctx, "ts-dsh-bridge tools", vec![spec]).await?;

            ctx.effect("ts-dsh-bridge runtime", async move {
                let disposer: cordis::Disposer = Box::new(move || {
                    Box::pin(
                        async move { shut_down(&slot, &unload_tx, Duration::from_secs(10)).await },
                    )
                });
                Ok(Some(disposer))
            })
            .await?;
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_maps_onto_the_sdk_config() {
        let cfg: BridgeConfig = serde_json::from_value(json!({
            "dshBin": "/x/bin.js", "profile": "sdk", "patches": ["p.yml"],
            "dshHome": "/h", "env": {"A": "b"}, "provider": "omniroute", "model": "m"
        }))
        .unwrap();
        let sdk = cfg.to_sdk();
        assert_eq!(sdk.dsh_bin.as_deref(), Some("/x/bin.js"));
        assert_eq!(sdk.profile, "sdk");
        assert_eq!(sdk.patches, vec![PathBuf::from("p.yml")]);
        assert_eq!(sdk.dsh_home, Some(PathBuf::from("/h")));
        assert_eq!(sdk.env.unwrap().get("A").map(String::as_str), Some("b"));
        assert_eq!(
            (sdk.provider.as_str(), sdk.model.as_str()),
            ("omniroute", "m")
        );
    }

    #[tokio::test]
    async fn a_turn_ends_by_completion_deadline_or_unload() {
        let (tx, mut rx) = tokio::sync::watch::channel(false);
        let quick = race_turn(async { 7 }, Duration::from_secs(5), &mut rx).await;
        assert!(matches!(quick, TurnEnd::Done(7)));

        let slow = race_turn(
            std::future::pending::<()>(),
            Duration::from_millis(30),
            &mut rx,
        )
        .await;
        assert!(matches!(slow, TurnEnd::TimedOut));

        let canceller = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
            tx.send(true).unwrap();
        });
        let started = tokio::time::Instant::now();
        let cut = race_turn(
            std::future::pending::<()>(),
            Duration::from_secs(30),
            &mut rx,
        )
        .await;
        assert!(matches!(cut, TurnEnd::Unloaded));
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "unload must not wait for the deadline"
        );
        canceller.await.unwrap();
    }

    #[tokio::test]
    async fn shut_down_signals_unload_first_is_idempotent_and_stays_closed() {
        let slot: Slot = Arc::new(tokio::sync::Mutex::new(Runtime::NotStarted));
        let (tx, mut rx) = tokio::sync::watch::channel(false);
        shut_down(&slot, &tx, Duration::from_millis(10)).await;
        assert!(matches!(*slot.lock().await, Runtime::Closed));
        assert!(
            *rx.borrow_and_update(),
            "unload is signalled before the slot closes"
        );
        // Idempotent: a second unload neither panics nor reopens the slot.
        shut_down(&slot, &tx, Duration::from_millis(10)).await;
        assert!(matches!(*slot.lock().await, Runtime::Closed));
    }

    #[test]
    fn fresh_session_ids_differ() {
        // A tight loop makes a clock-resolution collision near certain if the
        // id relied on the timestamp alone.
        let ids: std::collections::HashSet<String> =
            (0..10_000).map(|_| fresh_session_id()).collect();
        assert_eq!(ids.len(), 10_000);
    }

    #[tokio::test]
    async fn a_call_after_unload_cannot_start_a_runtime() {
        let slot: Slot = Arc::new(tokio::sync::Mutex::new(Runtime::NotStarted));
        let (tx, rx) = tokio::sync::watch::channel(false);
        shut_down(&slot, &tx, Duration::from_millis(10)).await;
        let config = BridgeConfig {
            dsh_bin: "/nonexistent/dsh".into(),
            ..Default::default()
        };
        let mut unload = rx;
        let Err(err) = runtime(&slot, &config, &mut unload).await else {
            panic!("must be refused")
        };
        assert!(err.contains("unload"), "{err}");
    }

    #[test]
    fn empty_strings_mean_unset() {
        let sdk = BridgeConfig::default().to_sdk();
        assert!(sdk.dsh_bin.is_none() && sdk.dsh_home.is_none() && sdk.env.is_none());
    }

    #[tokio::test]
    async fn a_missing_runtime_binary_is_a_clear_error() {
        let slot: Slot = Arc::new(tokio::sync::Mutex::new(Runtime::NotStarted));
        let config = BridgeConfig {
            dsh_bin: "/nonexistent/dsh".into(),
            session_cwd: "/tmp".into(),
            ..Default::default()
        };
        let (_tx, mut unload) = tokio::sync::watch::channel(false);
        let Err(err) = runtime(&slot, &config, &mut unload).await else {
            panic!("a missing runtime binary must fail")
        };
        assert!(!err.is_empty());
        assert!(
            matches!(*slot.lock().await, Runtime::NotStarted),
            "a failed start must not be cached"
        );
    }
}
