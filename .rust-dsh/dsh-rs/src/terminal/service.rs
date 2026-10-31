//! Session-owned terminal processes and the unary controller operations.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::follower::{Follower, Utf8Decoder};
use super::{pty::Pty, screen::Screen};
use crate::api::services::{AgentRegistryService, SessionPersistenceApi};
use crate::gui::wire::Failure;

/// The context service key.
pub const TERMINAL_SERVICE: &str = "terminalController";

/// A verified interactive shell profile.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TerminalShell {
    /// Executable path, resolved before use.
    pub path: String,
    /// Interactive arguments.
    pub args: Vec<String>,
    /// Display name.
    pub name: String,
}

/// Deployment limits for session-owned terminals.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TerminalConfig {
    /// Optional preferred shell profile.
    pub shell: Option<TerminalShell>,
    /// Ordered discovery candidates.
    pub shell_candidates: Vec<String>,
    /// Retained terminals per session.
    pub max_terminals: usize,
    /// Largest terminal width.
    pub max_cols: u16,
    /// Largest terminal height.
    pub max_rows: u16,
    /// Retained history rows.
    pub scrollback: usize,
    /// Per-follower encoded buffer limit.
    pub max_buffered_bytes: usize,
    /// Raw input command limit.
    pub max_input_bytes: usize,
    /// Process termination grace period.
    pub dispose_grace_ms: u64,
    /// Idle reclamation timeout; zero disables reclamation.
    pub unattended_timeout_ms: u64,
    /// How often a terminal's activity is inspected for reclamation.
    pub activity_poll_interval_ms: u64,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: None,
            shell_candidates: ["zsh", "bash", "fish", "pwsh", "powershell", "cmd"].map(str::to_string).to_vec(),
            max_terminals: 8, max_cols: 500, max_rows: 200, scrollback: 1000,
            max_buffered_bytes: 2 * 1024 * 1024, max_input_bytes: 64 * 1024,
            dispose_grace_ms: 1000, unattended_timeout_ms: 7_200_000, activity_poll_interval_ms: 30_000,
        }
    }
}

impl TerminalConfig {
    /// Validate limits before the plugin allocates processes.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_terminals == 0 || self.max_cols < 2 || self.max_rows == 0
            || self.max_buffered_bytes < 1024 || self.max_input_bytes == 0 || self.dispose_grace_ms == 0
            || self.unattended_timeout_ms > crate::gui::router::MAX_SAFE_INTEGER
            || self.shell_candidates.iter().any(String::is_empty)
        {
            return Err("gui: invalid terminal limits".into());
        }
        Ok(())
    }
}

fn failure(code: &str, message: impl Into<String>) -> Failure {
    Failure { code: code.into(), message: message.into(), details: json!({}) }
}
fn internal(message: impl Into<String>) -> Failure { failure("gateway/internal", message) }

/// One retained process. Output is consumed even when no client is attached.
pub struct Terminal {
    /// The process, for the stream half's write and resize operations.
    pub pty: Arc<Pty>,
    /// The screen, updated before each output notification.
    pub screen: Arc<Mutex<Screen>>,
    /// The terminal's metadata: id, title, shell, cwd, dimensions and state.
    pub info: Mutex<Value>,
    /// The terminal's frame-production lock: an output frame and a state frame
    /// reach the attachments in the order the terminal produced them.
    pub frame: Mutex<()>,
    /// The sequence of the last frame the terminal produced.
    sequence: std::sync::atomic::AtomicU64,
    /// The attachments watching this terminal.
    attachments: Mutex<Attachments>,
    /// The terminal's lifetime: it is cancelled when the terminal closes.
    lifetime: CancellationToken,
    /// The retention holds that keep the terminal from being reclaimed.
    holds: std::sync::atomic::AtomicUsize,
    /// Whether the retention monitor reclaimed this terminal.
    reclaimed: std::sync::atomic::AtomicBool,
    /// The activity epoch, moved on by every accepted input.
    activity_epoch: std::sync::atomic::AtomicU64,
    drained: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

/// The attachments of one terminal: who may write, and who is watching.
#[derive(Default)]
pub struct Attachments {
    /// The attachment that may write: the most recent one.
    controller: Option<u64>,
    /// The watching attachments and their frame queues.
    followers: Vec<(u64, Arc<Follower>)>,
    next_id: u64,
}

/// What an attachment may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    /// The attachment may write.
    Writable,
    /// Another attachment holds the input.
    ReadOnly,
}

impl Terminal {
    /// The terminal's lifetime: it is cancelled when the terminal closes.
    pub fn lifetime(&self) -> &CancellationToken { &self.lifetime }

    /// The activity epoch: every accepted input moves it on, so the retention
    /// monitor's idle observation does not survive a command.
    pub fn activity_epoch(&self) -> u64 { self.activity_epoch.load(std::sync::atomic::Ordering::SeqCst) }

    /// Record that the terminal was used, so an idle observation does not
    /// survive it.
    pub fn touch(&self) {
        self.activity_epoch.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Whether the retention monitor reclaimed this terminal.
    pub fn reclaimed(&self) -> bool { self.reclaimed.load(std::sync::atomic::Ordering::SeqCst) }

    /// The number of retention holds that keep this terminal from being reclaimed.
    pub fn holds(&self) -> usize { self.holds.load(std::sync::atomic::Ordering::SeqCst) }

    /// Take a retention hold; the terminal is not reclaimed while it is held.
    ///
    /// The hold invalidates the idle observation, so a terminal that was idle
    /// before the hold does not age out the moment the hold is released.
    pub fn hold(&self) {
        self.holds.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.touch();
    }

    /// Release a retention hold.
    pub fn release(&self) {
        self.holds.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        self.touch();
    }

    /// The sequence of the last frame the terminal produced.
    pub fn sequence(&self) -> u64 { self.sequence.load(std::sync::atomic::Ordering::SeqCst) }

    /// Tell every attachment the terminal's state, except the one named.
    ///
    /// Every attachment gets its own frame: whether it may write depends on
    /// which one holds the input.
    pub async fn broadcast_state_to(&self, except: Option<u64>) {
        let info = self.info().await;
        let running = info["state"] == "running";
        let attachments = self.attachments.lock().await;
        let controller = attachments.controller;
        let frames: Vec<(Arc<Follower>, Value)> = attachments
            .followers
            .iter()
            .filter(|(id, _)| Some(*id) != except)
            .map(|(id, follower)| {
                let writable = running && controller == Some(*id);
                (Arc::clone(follower), json!({ "type": "state", "info": info, "writable": writable }))
            })
            .collect();
        drop(attachments);
        for (follower, frame) in frames {
            follower.push(frame);
        }
    }

    /// Add a frame to every attachment.
    ///
    /// The push never waits for the attachment that reads the queue, so a slow
    /// client cannot hold up the others.
    pub async fn broadcast_frame(&self, frame: Value) {
        let followers: Vec<Arc<Follower>> = {
            let attachments = self.attachments.lock().await;
            attachments.followers.iter().map(|(_, follower)| Arc::clone(follower)).collect()
        };
        for follower in followers {
            follower.push(frame.clone());
        }
    }

    /// The attachment that holds the input, when there is one.
    pub async fn control(&self) -> Option<u64> {
        self.attachments.lock().await.controller
    }

    /// The attachments finish after the frames they hold are delivered.
    pub async fn finish_followers(&self) {
        let followers: Vec<Arc<Follower>> = {
            let attachments = self.attachments.lock().await;
            attachments.followers.iter().map(|(_, follower)| Arc::clone(follower)).collect()
        };
        for follower in followers {
            follower.finish();
        }
    }

    /// Tell every attachment the terminal's state.
    pub async fn broadcast_state(&self) {
        self.broadcast_state_to(None).await;
    }

    /// The next sequence number, for the frame the terminal is about to produce.
    pub fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }

    /// Current metadata, including the exit state after output drainage.
    pub async fn info(&self) -> Value { self.info.lock().await.clone() }

    async fn close(&self, grace: Duration) {
        self.pty.terminate(grace).await;
        if let Some(task) = self.drained.lock().await.take() { let _ = task.await; }
        // The attachments finish after the frames they hold are delivered, and
        // the retention holds end with the terminal. The frame lock keeps an
        // attachment from registering after the sweep.
        let _guard = self.frame.lock().await;
        self.finish_followers().await;
        self.lifetime.cancel();
    }
}

#[derive(Default)]
struct Owner {
    terminals: Vec<(String, Arc<Terminal>)>,
    closed: HashSet<String>,
}

#[derive(Default)]
struct State {
    owners: HashMap<String, Owner>,
    stopping: bool,
}

/// The controller owns processes, not requests or browser connections.
pub struct TerminalService {
    ctx: Context,
    /// Validated deployment limits, shared with the stream half.
    pub config: TerminalConfig,
    state: Mutex<State>,
    /// The service itself, so the retention monitor can reclaim a terminal.
    self_ref: std::sync::Mutex<std::sync::Weak<TerminalService>>,
}

impl std::fmt::Debug for TerminalService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalService").field("config", &self.config).finish_non_exhaustive()
    }
}

impl TerminalService {
    /// Construct a controller in the session registry's context.
    pub fn new(ctx: Context, config: TerminalConfig) -> Arc<Self> {
        let service = Arc::new(Self {
            ctx,
            config,
            state: Mutex::new(State::default()),
            self_ref: std::sync::Mutex::new(std::sync::Weak::new()),
        });
        *service.self_ref.lock().unwrap() = Arc::downgrade(&service);
        service
    }

    fn cwd(&self, session: &str) -> Result<String, Failure> {
        let live = self.ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
            .and_then(|agents| agents.get(session)).and_then(|agent| agent.session().header_cwd());
        let stored = || self.ctx.get::<Arc<dyn SessionPersistenceApi>>(crate::api::services::SESSION_PERSISTENCE_SERVICE)
            .and_then(|store| store.header(session).ok().flatten()).and_then(|header| header.cwd);
        live.or_else(stored).map(Ok).unwrap_or_else(|| std::env::current_dir()
            .map(|path| path.to_string_lossy().into_owned()).map_err(|error| internal(error.to_string())))
    }

    /// Current working directory and deployment limits, without filesystem validation.
    pub fn environment(&self, session: &str) -> Result<Value, Failure> {
        Ok(json!({ "cwd": self.cwd(session)?, "maxInputBytes": self.config.max_input_bytes,
            "maxBufferedBytes": self.config.max_buffered_bytes, "maxCols": self.config.max_cols,
            "maxRows": self.config.max_rows, "scrollback": self.config.scrollback }))
    }

    fn preferred(&self) -> Result<TerminalShell, Failure> {
        let shell = self.config.shell.clone().unwrap_or_else(|| profile(&std::env::var("SHELL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "/bin/sh".into())));
        resolve(shell)
    }

    /// Preferred shell followed by installed candidates, deduplicated by kind.
    pub fn shells(&self, _session: &str) -> Result<Vec<TerminalShell>, Failure> {
        let mut shells = vec![self.preferred()?];
        for candidate in &self.config.shell_candidates {
            if let Ok(shell) = resolve(profile(candidate)) {
                if !shells.iter().any(|found| kind(&found.path) == kind(&shell.path)) { shells.push(shell); }
            }
        }
        Ok(shells)
    }

    /// Retained terminals in creation order; an unknown session has none.
    pub async fn list(&self, session: &str) -> Vec<Value> {
        let state = self.state.lock().await;
        let mut result = Vec::new();
        if let Some(owner) = state.owners.get(session) {
            for (_, terminal) in &owner.terminals {
                if terminal.reclaimed() { continue; }
                result.push(terminal.info().await);
            }
        }
        result
    }

    /// Allocate once per open identity. Validation precedes idempotency lookup.
    pub async fn create(&self, session: &str, request: &Value) -> Result<Value, Failure> {
        let id = request["id"].as_str().unwrap_or_default();
        if id.is_empty() || id.len() > 128 || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
            return Err(internal("Invalid terminal identity"));
        }
        let dimension = |key: &str, min: u64, max: u16| request[key].as_f64()
            .filter(|n| n.is_finite() && n.fract() == 0.0 && *n >= min as f64 && *n <= max as f64).map(|n| n as u16);
        let (Some(cols), Some(rows)) = (dimension("cols", 2, self.config.max_cols), dimension("rows", 1, self.config.max_rows)) else {
            return Err(internal("Terminal dimensions exceed the configured limits"));
        };
        let mut state = self.state.lock().await;
        if state.stopping { return Err(internal("Terminal controller disposed")); }
        let owner = state.owners.entry(session.into()).or_default();
        if owner.closed.contains(id) { return Err(failure("terminal/unavailable", "Terminal was closed in this Session")); }
        if let Some((_, terminal)) = owner.terminals.iter().find(|(key, _)| key == id) { return Ok(terminal.info().await); }
        if owner.terminals.len() >= self.config.max_terminals {
            let mut error = failure("terminal/limit-reached", "Session terminal limit reached");
            error.details = json!({ "limit": self.config.max_terminals });
            return Err(error);
        }
        let cwd = self.cwd(session)?;
        let shell = match request.get("shellPath") {
            None => self.preferred()?,
            Some(path) => self.shells(session)?.into_iter().find(|shell| path.as_str() == Some(shell.path.as_str()))
                .ok_or_else(|| internal("Selected shell is not available in this execution environment"))?,
        };
        let argv = std::iter::once(shell.path.clone()).chain(shell.args.clone()).collect::<Vec<_>>();
        let pty = Arc::new(Pty::spawn(&argv, &cwd, cols, rows, &[("DSH_SESSION_ID".into(), session.into()), ("TERM".into(), "xterm-256color".into())])
            .map_err(|error| internal(error.to_string()))?);
        let info = json!({ "id": id, "title": shell.name, "shell": shell, "cwd": cwd,
            "cols": cols, "rows": rows, "state": "running", "exitCode": null });
        let terminal = Arc::new(Terminal { pty, screen: Arc::new(Mutex::new(Screen::new(cols, rows, self.config.scrollback))),
            info: Mutex::new(info.clone()), frame: Mutex::new(()), sequence: std::sync::atomic::AtomicU64::new(0),
            attachments: Mutex::new(Attachments::default()), lifetime: CancellationToken::new(),
            holds: std::sync::atomic::AtomicUsize::new(0), reclaimed: std::sync::atomic::AtomicBool::new(false),
            activity_epoch: std::sync::atomic::AtomicU64::new(0),
            drained: Mutex::new(None) });
        let mut source = terminal.pty.take_output().ok_or_else(|| internal("Terminal output unavailable"))?;
        let consuming = Arc::clone(&terminal);
        let task = tokio::spawn(async move {
            // The output is decoded across chunk boundaries, so a character
            // split between two reads is one character.
            let mut decoder = Utf8Decoder::new();
            while let Some(data) = source.recv().await {
                let text = decoder.push(&data);
                if text.is_empty() { continue; }
                // The screen is updated before the frame goes out, so a snapshot
                // taken now carries the output it describes.
                let _guard = consuming.frame.lock().await;
                consuming.screen.lock().await.write(text.as_bytes());
                let sequence = consuming.next_sequence();
                consuming.broadcast_frame(json!({ "type": "output", "sequence": sequence, "data": text })).await;
            }
            let tail = decoder.finish();
            if !tail.is_empty() {
                let _guard = consuming.frame.lock().await;
                consuming.screen.lock().await.write(tail.as_bytes());
                let sequence = consuming.next_sequence();
                consuming.broadcast_frame(json!({ "type": "output", "sequence": sequence, "data": tail })).await;
            }
            let mut exit = consuming.pty.exit();
            while exit.borrow().is_none() { if exit.changed().await.is_err() { break; } }
            let code = exit.borrow().and_then(|exit| exit.code);
            let mut info = consuming.info.lock().await;
            info["state"] = json!("exited");
            info["exitCode"] = json!(code);
            drop(info);
            let _guard = consuming.frame.lock().await;
            consuming.broadcast_state().await;
            // The attachments stay: a terminal that ended still answers an
            // input with a refusal, as the TypeScript host does. The followers
            // finish when the terminal is closed.
        });
        *terminal.drained.lock().await = Some(task);
        // The retention monitor closes a terminal nobody uses, as the
        // TypeScript host does after the unattended timeout.
        self.watch_retention(session, id, &terminal);
        owner.terminals.push((id.into(), terminal));
        Ok(info)
    }

    /// Register an attachment and give it the input.
    ///
    /// The newest attachment holds the input; the others are told that they
    /// are read-only. The attachment's first frame is the snapshot, taken in
    /// the same step as the registration, so no output is lost between the
    /// screen it shows and the frames that follow it.
    pub async fn attach(
        &self,
        terminal: &Arc<Terminal>,
        cap: usize,
        input_window: Value,
    ) -> (u64, Arc<Follower>, Value) {
        let _guard = terminal.frame.lock().await;
        let (id, follower) = {
            let mut attachments = terminal.attachments.lock().await;
            attachments.next_id += 1;
            let id = attachments.next_id;
            let follower = Arc::new(Follower::new(cap));
            attachments.controller = Some(id);
            (id, follower)
        };
        // The other attachments learn that the input moved, before this one
        // joins them.
        terminal.broadcast_state_to(Some(id)).await;
        {
            let mut attachments = terminal.attachments.lock().await;
            attachments.followers.push((id, Arc::clone(&follower)));
        }
        let info = terminal.info().await;
        let running = info["state"] == "running";
        let writable = running && terminal.control().await == Some(id);
        let screen = terminal.screen.lock().await.serialize();
        let snapshot = json!({
            "type": "snapshot",
            "sequence": terminal.sequence(),
            "screen": screen,
            "info": info,
            "writable": writable,
            "inputWindow": input_window,
        });
        // The snapshot is the attachment's first frame, delivered outside the
        // bounded queue: a client that reconnects gets the current screen even
        // when the queue would not hold it.
        (id, follower, snapshot)
    }

    /// Remove an attachment. When it held the input, nobody holds it.
    pub async fn detach(&self, terminal: &Arc<Terminal>, id: u64) {
        let _guard = terminal.frame.lock().await;
        {
            let mut attachments = terminal.attachments.lock().await;
            attachments.followers.retain(|(follower, _)| *follower != id);
            if attachments.controller == Some(id) {
                attachments.controller = None;
            }
        }
        terminal.broadcast_state_to(None).await;
    }

    /// Whether the attachment may write.
    pub async fn control(&self, terminal: &Arc<Terminal>, id: u64) -> Control {
        let attachments = terminal.attachments.lock().await;
        if attachments.controller == Some(id) { Control::Writable } else { Control::ReadOnly }
    }

    /// Tell every attachment the terminal's state, except the one named.
    pub async fn broadcast_state_to(&self, terminal: &Arc<Terminal>, except: Option<u64>) {
        terminal.broadcast_state_to(except).await;
    }

    /// Tell every attachment the terminal's state.
    pub async fn broadcast_state(&self, terminal: &Arc<Terminal>) {
        terminal.broadcast_state().await;
    }

    /// Watch a terminal for inactivity and close it when it stays idle.
    ///
    /// A terminal with a retention hold is never reclaimed: a window is
    /// watching it.
    fn watch_retention(&self, session: &str, id: &str, terminal: &Arc<Terminal>) {
        let timeout = self.config.unattended_timeout_ms;
        if timeout == 0 {
            return;
        }
        let poll = Duration::from_millis(self.config.activity_poll_interval_ms.max(1));
        let service = self.self_ref.lock().unwrap().upgrade();
        let session = session.to_string();
        let id = id.to_string();
        let grace = Duration::from_millis(self.config.dispose_grace_ms);
        let terminal = Arc::clone(terminal);
        tokio::spawn(async move {
            let mut idle_since: Option<std::time::Instant> = None;
            let mut epoch = terminal.activity_epoch();
            loop {
                tokio::time::sleep(poll).await;
                if terminal.lifetime().is_cancelled() {
                    return;
                }
                if terminal.holds() > 0 {
                    idle_since = None;
                    epoch = terminal.activity_epoch();
                    continue;
                }
                // Input since the last observation ends the idle interval.
                let current = terminal.activity_epoch();
                if current != epoch {
                    epoch = current;
                    idle_since = None;
                }
                match terminal.pty.activity().await {
                    super::pty::Activity::Idle => {
                        let since = *idle_since.get_or_insert_with(std::time::Instant::now);
                        if since.elapsed() >= Duration::from_millis(timeout) {
                            // The inspection yielded: a hold or an input may have
                            // arrived while it ran, so the observation is checked
                            // again before the terminal is reclaimed.
                            let after = terminal.activity_epoch();
                            if after != epoch || terminal.holds() > 0 {
                                epoch = after;
                                idle_since = None;
                                continue;
                            }
                            terminal.reclaimed.store(true, std::sync::atomic::Ordering::SeqCst);
                            match service {
                                // The identity is closed and forgotten, so the
                                // slot it held is free again.
                                Some(service) => service.reclaim(&session, &id).await,
                                None => terminal.close(grace).await,
                            }
                            return;
                        }
                    }
                    _ => idle_since = None,
                }
            }
        });
    }

    /// Access a retained process and its screen/output for stream attachment.
    pub async fn terminal(&self, session: &str, id: &str) -> Result<Arc<Terminal>, Failure> {
        let state = self.state.lock().await;
        let owner = state.owners.get(session);
        // A closed identity is gone, even while its process is still ending.
        if owner.is_some_and(|owner| owner.closed.contains(id)) {
            return Err(failure("terminal/unavailable", "Terminal no longer exists in this Session"));
        }
        owner.and_then(|owner| owner.terminals.iter().find(|(key, _)| key == id))
            .map(|(_, terminal)| Arc::clone(terminal))
            .ok_or_else(|| failure("terminal/unavailable", "Terminal no longer exists in this Session"))
    }

    /// Store a trimmed display title, measured in JavaScript UTF-16 units.
    ///
    /// The length is checked on the title as it arrived, and the trimmed title
    /// is what is stored.
    pub async fn rename(&self, session: &str, id: &str, title: &str) -> Result<(), Failure> {
        if title.encode_utf16().count() > 120 {
            return Err(internal("Terminal title must contain 1\u{2013}120 characters"));
        }
        let title = title.trim_matches(|c: char| c == '\u{feff}' || c.is_whitespace());
        if title.is_empty() {
            return Err(internal("Terminal title must contain 1\u{2013}120 characters"));
        }
        let terminal = self.terminal(session, id).await?;
        terminal.info.lock().await["title"] = json!(title);
        self.broadcast_state(&terminal).await;
        Ok(())
    }

    /// Close even an unknown identity to subsequent creation.
    ///
    /// The process ends outside the registry lock: a close waits for the
    /// process to go, and holding the lock would block every other session.
    pub async fn close(&self, session: &str, id: &str) {
        let terminal = {
            let mut state = self.state.lock().await;
            let owner = state.owners.entry(session.into()).or_default();
            owner.closed.insert(id.into());
            owner.terminals.iter().find(|(key, _)| key == id).map(|(_, terminal)| Arc::clone(terminal))
        };
        let Some(terminal) = terminal else { return };
        terminal.close(Duration::from_millis(self.config.dispose_grace_ms)).await;
        let mut state = self.state.lock().await;
        if let Some(owner) = state.owners.get_mut(session) {
            owner.terminals.retain(|(key, _)| key != id);
        }
    }

    /// Reclaim a terminal the retention monitor found idle: its identity is
    /// closed and its record forgotten, so the slot it held is free again.
    pub async fn reclaim(&self, session: &str, id: &str) {
        let terminal = {
            let mut state = self.state.lock().await;
            let owner = state.owners.entry(session.into()).or_default();
            owner.closed.insert(id.into());
            owner.terminals.iter().find(|(key, _)| key == id).map(|(_, terminal)| Arc::clone(terminal))
        };
        if let Some(terminal) = terminal {
            terminal.close(Duration::from_millis(self.config.dispose_grace_ms)).await;
        }
        let mut state = self.state.lock().await;
        if let Some(owner) = state.owners.get_mut(session) {
            owner.terminals.retain(|(key, _)| key != id);
        }
    }

    /// Dispose a session's complete process ownership before forgetting it.
    pub async fn dispose_session(&self, session: &str) {
        let terminals: Vec<Arc<Terminal>> = {
            let mut state = self.state.lock().await;
            match state.owners.remove(session) {
                Some(owner) => owner.terminals.into_iter().map(|(_, terminal)| terminal).collect(),
                None => Vec::new(),
            }
        };
        for terminal in terminals {
            terminal.close(Duration::from_millis(self.config.dispose_grace_ms)).await;
        }
    }

    /// Stop allocations and dispose every process owned by this controller.
    pub async fn dispose(&self) {
        let terminals: Vec<Arc<Terminal>> = {
            let mut state = self.state.lock().await;
            state.stopping = true;
            state
                .owners
                .drain()
                .flat_map(|(_, owner)| owner.terminals.into_iter().map(|(_, terminal)| terminal))
                .collect()
        };
        for terminal in terminals {
            terminal.close(Duration::from_millis(self.config.dispose_grace_ms)).await;
        }
    }

    /// Bind terminal ownership to the session registry's disposal events.
    pub async fn observe(self: &Arc<Self>) -> Result<(), cordis::Error> {
        let service = Arc::clone(self);
        crate::api::events::on::<crate::api::events::SessionDisposedPayload, _>(&self.ctx, move |_ctx, payload| {
            let service = Arc::clone(&service);
            Box::pin(async move { service.dispose_session(&payload.session).await; Ok(Value::Null) })
        }).await?;
        Ok(())
    }
}

fn name(path: &str) -> &str { path.rsplit(['/', '\\']).next().unwrap_or(path) }
fn kind(path: &str) -> String { name(path).to_lowercase().trim_end_matches(".exe").into() }
fn profile(path: &str) -> TerminalShell {
    let args = match kind(path).as_str() { "cmd" => vec![], "pwsh" | "powershell" => vec!["-NoLogo".into()], _ => vec!["-i".into()] };
    TerminalShell { path: path.into(), name: name(path).into(), args }
}
fn resolve(mut shell: TerminalShell) -> Result<TerminalShell, Failure> {
    use std::os::unix::fs::PermissionsExt;
    let path = Path::new(&shell.path);
    if !path.is_absolute() && shell.path.contains('/') {
        return Err(internal(format!("subprocess-local: command {:?} is a relative path; use an absolute path or a bare PATH name", shell.path)));
    }
    let paths: Vec<PathBuf> = if path.is_absolute() { vec![path.into()] } else {
        let cwd = std::env::current_dir().map_err(|error| internal(error.to_string()))?;
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).map(|dir| cwd.join(dir).join(path)).collect()
    };
    for candidate in paths {
        if candidate.metadata().is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0) {
            shell.path = candidate.to_string_lossy().into_owned();
            return Ok(shell);
        }
    }
    Err(internal(format!("subprocess-local: command {:?} {}", shell.path,
        if path.is_absolute() { "is not an executable file" } else { "was not found on PATH" })))
}
