//! The WebSocket half of the gateway: one socket carrying many logical streams.
//!
//! Wire protocol, read from `packages/api/gateway/src/stream-protocol.ts`:
//!
//! - client → host: `open {streamId, endpoint, payload}`, `item`, `end`, `cancel`
//! - host → client: `item {streamId, value}`, `error {streamId, error}`, `end {streamId}`
//!
//! A malformed frame or a duplicate `open` closes the socket (code 1008); an
//! `item`, `end` or `cancel` for a stream that is not open is dropped, because a
//! finished stream leaves the table while the client's last frames are still in
//! flight. A cancelled stream sends nothing further.
//!
//! [`Mux`] never touches a socket: it consumes text frames and writes text
//! frames to a channel, so the whole protocol is testable without a port.
//!
//! Streams served: `$events`, `session/follow` and `workspace/follow`. Every other stream endpoint
//! answers `gateway/lookup-not-found`.
//!
//! `$events` sends the ready frame, then forwards tool-approval questions as
//! `approval/request` waterfall frames (see [`super::events`]). It forwards no
//! other host event: the TypeScript list of forwarded emits and waterfalls is
//! not ported. Opening it makes this socket the harness's approver, replacing
//! any other approver in the process (such as the terminal chat UI); closing it
//! denies any question still open.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cordis::Context;
use serde_json::{json, Map, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::router::{argument, input_invalid, unavailable, unwrap_args, RecordEncoder};
use super::product_analytics::ProductAnalyticsService;
use super::uplink::{Uplink, DEFAULT_UPLINK_BYTES};
use super::wire::{codes, Failure};
use crate::api::events::{self, SessionEventPayload};
use crate::api::services::AgentRegistryService;
use crate::plugins::approvals::{
    ApprovalBroker, ApprovalInbox, PendingApproval, APPROVALS_SERVICE,
};
use crate::plugins::ask_user::{PendingQuestion, QuestionBroker, QuestionInbox, USER_QUESTIONS_SERVICE};
use crate::types::{AgentStatus, SessionEvent};

/// Stream endpoints this build serves, pinned by a test so the parity table
/// cannot drift silently.
pub const STREAM_ENDPOINTS: &[&str] = &[
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
];

/// WebSocket close code for a protocol violation.
pub const CLOSE_POLICY: u16 = 1008;

/// Settings the mux needs from the gateway.
#[derive(Debug, Clone)]
pub struct MuxConfig {
    /// Host account home, published in the `$events` ready frame so the client
    /// can abbreviate displayed paths.
    pub home: String,
    /// Most events one `session/follow` snapshot carries.
    pub max_page_events: usize,
    /// The agent presets the deployment offers.
    pub agent_presets: Vec<super::router::AgentPreset>,
    /// What images the deployment admits, as the projection reports it.
    pub image_limits: crate::images::ImageLimits,
    /// Where `$events` keeps the approval questions it has sent.
    pub events: Arc<super::events::EventHub>,
    /// How often `$events` withdraws questions whose call stopped waiting.
    pub approval_sweep: std::time::Duration,
    /// The registry `workspace/follow` streams from.
    pub workspaces: Arc<super::workspaces::WorkspaceRegistry>,
    /// How often `workspace/follow` re-reads which sessions each workspace owns,
    /// since a session can appear without going through a workspace call.
    pub workspace_poll: std::time::Duration,
    /// The model a session uses until one is chosen, for its projections.
    pub default_model: super::router::DefaultModel,
    /// The terminal controller, for the two terminal streams.
    pub terminal: Option<std::sync::Arc<crate::terminal::service::TerminalService>>,
    /// The analytics policy, for the policy stream.
    pub product_analytics: Option<std::sync::Arc<ProductAnalyticsService>>,
    /// The shared preferences the speech catalog stream reads.
    pub settings: std::sync::Arc<super::settings::SettingsStore>,
}

/// Why the socket must close.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuxClose {
    /// WebSocket close code.
    pub code: u16,
    /// Short reason sent in the close frame.
    pub reason: &'static str,
}

const INVALID_FRAME: MuxClose = MuxClose {
    code: CLOSE_POLICY,
    reason: "invalid Remote stream request",
};
const DUPLICATE_STREAM: MuxClose = MuxClose {
    code: CLOSE_POLICY,
    reason: "duplicate Remote stream id",
};

/// One decoded client frame.
#[derive(Debug, PartialEq)]
enum ClientFrame {
    Open {
        stream_id: String,
        endpoint: String,
        payload: Value,
    },
    Item {
        stream_id: String,
        value: Option<Value>,
    },
    End {
        stream_id: String,
    },
    Cancel {
        stream_id: String,
    },
}

/// How a stream finished, which decides its terminal frame.
pub(super) enum Outcome {
    Ended,
    Failed(Failure),
    /// The client cancelled or the socket closed; nothing more is sent.
    Cancelled,
}

/// One open logical stream: its cancellation and its uplink inbox.
struct StreamEntry {
    cancel: CancellationToken,
    uplink: Arc<Uplink>,
}

type Streams = Arc<Mutex<HashMap<String, StreamEntry>>>;

/// The logical-stream table of one socket.
pub struct Mux {
    ctx: Context,
    config: Arc<MuxConfig>,
    out: mpsc::Sender<String>,
    streams: Streams,
}

impl Mux {
    /// A mux that writes its outbound text frames to `out`.
    pub fn new(ctx: Context, config: MuxConfig, out: mpsc::Sender<String>) -> Self {
        Mux {
            ctx,
            config: Arc::new(config),
            out,
            streams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Handle one inbound text frame. An error means the socket must close.
    pub fn receive(&self, text: &str) -> Result<(), MuxClose> {
        match parse_client_frame(text).ok_or(INVALID_FRAME)? {
            ClientFrame::Open {
                stream_id,
                endpoint,
                payload,
            } => self.open(stream_id, endpoint, payload),
            ClientFrame::Item { stream_id, value } => {
                // An item for a stream this connection no longer owns is dropped.
                let entry = self.streams.lock().unwrap().get(&stream_id).map(|entry| Arc::clone(&entry.uplink));
                if let Some(uplink) = entry {
                    uplink.push(value.unwrap_or(Value::Null), text.len());
                }
                Ok(())
            }
            ClientFrame::End { stream_id } => {
                let entry = self.streams.lock().unwrap().get(&stream_id).map(|entry| Arc::clone(&entry.uplink));
                if let Some(uplink) = entry {
                    uplink.end();
                }
                Ok(())
            }
            ClientFrame::Cancel { stream_id } => {
                let entry = {
                    let table = self.streams.lock().unwrap();
                    table.get(&stream_id).map(|entry| (entry.cancel.clone(), Arc::clone(&entry.uplink)))
                };
                if let Some((cancel, uplink)) = entry {
                    uplink.close();
                    cancel.cancel();
                }
                Ok(())
            }
        }
    }

    /// Cancel every open stream; called when the socket closes.
    pub fn shutdown(&self) {
        let entries: Vec<(CancellationToken, Arc<Uplink>)> = self
            .streams
            .lock()
            .unwrap()
            .values()
            .map(|entry| (entry.cancel.clone(), Arc::clone(&entry.uplink)))
            .collect();
        for (cancel, uplink) in entries {
            uplink.close();
            cancel.cancel();
        }
    }

    /// How many logical streams are currently open.
    pub fn open_streams(&self) -> usize {
        self.streams.lock().unwrap().len()
    }

    fn open(&self, stream_id: String, endpoint: String, payload: Value) -> Result<(), MuxClose> {
        let cancel = CancellationToken::new();
        let uplink = Arc::new(Uplink::new(endpoint.clone(), DEFAULT_UPLINK_BYTES, Arc::new(cancel.clone())));
        {
            let mut streams = self.streams.lock().unwrap();
            if streams.contains_key(&stream_id) {
                return Err(DUPLICATE_STREAM);
            }
            streams.insert(stream_id.clone(), StreamEntry { cancel: cancel.clone(), uplink: Arc::clone(&uplink) });
        }
        let ctx = self.ctx.clone();
        let config = Arc::clone(&self.config);
        let out = self.out.clone();
        let streams = Arc::clone(&self.streams);
        tokio::spawn(async move {
            let outcome = run_stream(
                &ctx, &config, &endpoint, &payload, &stream_id, &out, &cancel, &uplink,
            )
            .await;
            // A client that broke the uplink rules fails the stream, even when the
            // host method was busy when the rules broke.
            let outcome = match uplink.failure() {
                Some(failure) => Outcome::Failed(failure),
                None => outcome,
            };
            let terminal = match outcome {
                Outcome::Ended => Some(json!({ "type": "end", "streamId": stream_id })),
                Outcome::Failed(failure) => Some(json!({
                    "type": "error",
                    "streamId": stream_id,
                    "error": failure,
                })),
                Outcome::Cancelled => None,
            };
            // The id stays claimed until the terminal frame is queued, and is
            // released in the same locked step as the queueing. A client may
            // reuse the id the moment it reads the terminal frame, so releasing
            // later would reject it as a duplicate; releasing earlier would let
            // a new stream's items overtake the old stream's terminal frame.
            // The reservation waits for room in the socket's output queue. A
            // cancelled stream sends nothing further, so the wait ends with the
            // cancellation instead of holding the stream's id.
            let reserved = match terminal {
                Some(_) => {
                    tokio::select! {
                        biased;
                        _ = cancel.cancelled() => None,
                        permit = out.reserve() => permit.ok(),
                    }
                }
                None => None,
            };
            let mut table = streams.lock().unwrap();
            table.remove(&stream_id);
            if let (Some(permit), Some(frame)) = (reserved, terminal) {
                permit.send(frame.to_string());
            }
            drop(table);
        });
        Ok(())
    }
}

fn parse_client_frame(text: &str) -> Option<ClientFrame> {
    let value: Value = serde_json::from_str(text).ok()?;
    let object = value.as_object()?;
    let kind = object.get("type")?.as_str()?;
    let stream_id = object
        .get("streamId")?
        .as_str()
        .filter(|id| !id.is_empty())?
        .to_string();
    let keys = |allowed: &[&str]| {
        object.len() == allowed.len() && allowed.iter().all(|key| object.contains_key(*key))
    };
    match kind {
        "open" if keys(&["type", "streamId", "endpoint", "payload"]) => {
            let endpoint = object.get("endpoint")?.as_str().filter(|e| !e.is_empty())?;
            Some(ClientFrame::Open {
                stream_id,
                endpoint: endpoint.to_string(),
                payload: object.get("payload")?.clone(),
            })
        }
        "item" if keys(&["type", "streamId"]) || keys(&["type", "streamId", "value"]) => {
            Some(ClientFrame::Item { stream_id, value: object.get("value").cloned() })
        }
        "end" if keys(&["type", "streamId"]) => Some(ClientFrame::End { stream_id }),
        "cancel" if keys(&["type", "streamId"]) => Some(ClientFrame::Cancel { stream_id }),
        _ => None,
    }
}

async fn run_stream(
    ctx: &Context,
    config: &MuxConfig,
    endpoint: &str,
    payload: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
    uplink: &Arc<Uplink>,
) -> Outcome {
    if cancel.is_cancelled() {
        return Outcome::Cancelled;
    }
    let args = match unwrap_args(payload) {
        Ok(args) => args,
        Err(reason) => return Outcome::Failed(Failure::at(codes::PROTOCOL, reason, endpoint)),
    };
    match endpoint {
        "$events" => events_stream(ctx, config, args, stream_id, out, cancel).await,
        "session/follow" => follow(ctx, config, args, stream_id, out, cancel).await,
        "workspace/follow" => workspace_follow(ctx, config, args, stream_id, out, cancel).await,
        "remoteAccess/follow" => {
            // Whoever holds the launch token is the owner; there are no scoped grants.
            let baseline = json!({ "type": "baseline", "owner": true });
            held_open(args, endpoint, baseline, stream_id, out, cancel).await
        }
        "account/watch" => {
            // dsh-rs has no DeepSeek account: it is always signed out.
            held_open(args, endpoint, super::account::signed_out_view(), stream_id, out, cancel).await
        }
        "account/watchExpiry" => {
            // No session exists to expire, so the stream sends nothing until cancelled.
            if args.as_object().is_none_or(|object| !object.is_empty()) {
                return Outcome::Failed(input_invalid(endpoint, "args", "this stream takes an empty args object"));
            }
            cancel.cancelled().await;
            Outcome::Cancelled
        }
        "session/control" => {
            control_stream(ctx, config, args, stream_id, out, cancel).await
        }
        "session/observeState" => {
            super::session_read::observe(ctx, config, endpoint, args, stream_id, out, cancel).await
        }
        "session/observeEvent" => {
            super::session_read::observe(ctx, config, endpoint, args, stream_id, out, cancel).await
        }
        "session/observeProjections" => {
            super::session_read::observe(ctx, config, endpoint, args, stream_id, out, cancel).await
        }
        "productAnalytics/watchPolicy" => {
            // The policy is delivered by the service, through the mux's delivery.
            let Some(service) = config.product_analytics.clone() else {
                return Outcome::Failed(Failure::at(
                    codes::SERVICE_UNAVAILABLE,
                    "the product analytics service is not wired into this build",
                    endpoint,
                ));
            };
            let end = service
                .watch_policy(cancel, |value| async move {
                    deliver(out, stream_id, Value::Bool(value), cancel).await
                })
                .await;
            match end {
                super::product_analytics::PolicyEnd::Disposed => Outcome::Ended,
                super::product_analytics::PolicyEnd::Cancelled => Outcome::Cancelled,
            }
        }
        "claudeCodeMods/watchBand" => {
            super::claude_code_mods::watch_band(ctx, endpoint, args, stream_id, out, cancel).await
        }
        "toolResults/watch" => {
            super::tool_results::watch(ctx, args, stream_id, out, cancel).await
        }
        "userQuestions/attachWait" => {
            super::user_questions::attach_wait(ctx, args, stream_id, out, cancel).await
        }
        "workspaceFiles/changes" => {
            super::file_changes::changes(ctx, args, stream_id, out, cancel).await
        }
        "forgePanel/follow" => {
            super::forge_panel::stream(ctx, endpoint, args, stream_id, out, cancel).await
        }
        "forgePanel/changeRequestStates" => {
            super::forge_panel::stream(ctx, endpoint, args, stream_id, out, cancel).await
        }
        "speech/follow" => {
            super::speech::follow_owned(ctx, &config.settings, args, stream_id, out, cancel).await
        }
        "job/list" => {
            super::jobs::list(endpoint, args, stream_id, out, cancel).await
        }
        "job/follow" => {
            super::jobs::follow(endpoint, args, stream_id, out, cancel).await
        }
        "terminal/attach" => {
            super::terminal_stream::dispatch(ctx, config, endpoint, args, stream_id, out, cancel, uplink).await
        }
        "terminal/retain" => {
            super::terminal_stream::dispatch(ctx, config, endpoint, args, stream_id, out, cancel, uplink).await
        }
        other => Outcome::Failed(Failure::at(
            codes::LOOKUP_NOT_FOUND,
            format!("no stream endpoint is registered as {other:?}"),
            other,
        )),
    }
}

static NEXT_CLIENT: AtomicU64 = AtomicU64::new(1);

/// `$events`: the ready handshake, then silence until cancelled. The ready
/// frame binds later `$events/result` calls to this generation.
async fn events_stream(
    ctx: &Context,
    config: &MuxConfig,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    const ENDPOINT: &str = "$events";
    if args.as_object().is_none_or(|object| !object.is_empty()) {
        return Outcome::Failed(input_invalid(
            ENDPOINT,
            "args",
            "the forwarded event stream takes an empty args object",
        ));
    }
    let client_id = format!("client-{}", NEXT_CLIENT.fetch_add(1, Ordering::Relaxed));
    // Register and attach before the ready frame, so a question asked in
    // between is queued rather than denied for want of an approver.
    let registration = config.events.register(&client_id);
    let mut inbox = ctx
        .get::<ApprovalBroker>(APPROVALS_SERVICE)
        .map(|broker| broker.attach());
    let mut ask_inbox = ctx
        .get::<QuestionBroker>(USER_QUESTIONS_SERVICE)
        .map(|broker| broker.attach());
    // The bus can deliver `agent/status` out of order, so it only names the
    // agent that changed; the status sent is read from the agent afterwards.
    // Pending agents are kept as a set, so a slow client costs one entry per
    // agent however many notifications arrive.
    let pending: Arc<Mutex<std::collections::BTreeSet<String>>> = Arc::default();
    let wake = Arc::new(tokio::sync::Notify::new());
    let status_subscription = {
        let (pending, wake) = (Arc::clone(&pending), Arc::clone(&wake));
        events::on::<crate::api::events::AgentStatusPayload, _>(ctx, move |_ctx, payload| {
            let (pending, wake) = (Arc::clone(&pending), Arc::clone(&wake));
            Box::pin(async move {
                pending
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(payload.agent);
                wake.notify_one();
                Ok(Value::Null)
            })
        })
        .await
    };
    let status_subscription = match status_subscription {
        Ok(guard) => guard,
        Err(error) => {
            return Outcome::Failed(Failure::at(
                codes::INTERNAL,
                format!("could not subscribe to agent status: {error}"),
                ENDPOINT,
            ))
        }
    };
    let mut reported: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    let ready = json!({
        "type": "ready",
        "clientId": client_id,
        "host": { "home": config.home },
    });
    if deliver(out, stream_id, ready, cancel).await.is_err() {
        let _ = status_subscription.dispose().await;
        return Outcome::Cancelled;
    }
    let mut sweep = tokio::time::interval(config.approval_sweep);
    sweep.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let outcome = 'stream: {
        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => break 'stream Outcome::Cancelled,
                question = next_question(&mut inbox) => {
                    let Some(question) = question else {
                        // The broker replaced this approver; nothing more will arrive.
                        inbox = None;
                        continue;
                    };
                    let event_id = crate::plugins::unique_id();
                    let frame = approval_frame(&event_id, &question);
                    registration.hold(&event_id, question);
                    if deliver(out, stream_id, frame, cancel).await.is_err() {
                        break 'stream Outcome::Cancelled;
                    }
                }
                question = next_ask(&mut ask_inbox) => {
                    let Some(question) = question else {
                        ask_inbox = None;
                        continue;
                    };
                    let event_id = crate::plugins::unique_id();
                    let frame = question_frame(&event_id, &question);
                    registration.hold(&event_id, question);
                    if deliver(out, stream_id, frame, cancel).await.is_err() {
                        break 'stream Outcome::Cancelled;
                    }
                }
                _ = wake.notified() => {
                    let agents = std::mem::take(&mut *pending.lock().unwrap_or_else(|e| e.into_inner()));
                    for agent in agents {
                        let Some(running) = ctx
                            .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                            .and_then(|agents| agents.get(&agent))
                            .map(|view| view.status() == AgentStatus::Running)
                        else {
                            continue;
                        };
                        if reported.insert(agent.clone(), running) == Some(running) {
                            continue;
                        }
                        let frame = json!({ "type": "emit", "event": "api-session/status", "args": [agent, running] });
                        if deliver(out, stream_id, frame, cancel).await.is_err() {
                            break 'stream Outcome::Cancelled;
                        }
                    }
                }
                _ = sweep.tick() => {
                    for event_id in registration.sweep() {
                        let frame = json!({ "type": "cancel", "eventId": event_id });
                        if deliver(out, stream_id, frame, cancel).await.is_err() {
                            break 'stream Outcome::Cancelled;
                        }
                    }
                }
            }
        }
    };
    let _ = status_subscription.dispose().await;
    outcome
}

/// The next question, or pending forever when no approver is attached.
async fn next_question(inbox: &mut Option<ApprovalInbox>) -> Option<PendingApproval> {
    match inbox {
        Some(inbox) => inbox.questions.recv().await,
        None => std::future::pending().await,
    }
}

/// The next question batch, or pending forever when no answerer is attached.
async fn next_ask(inbox: &mut Option<QuestionInbox>) -> Option<PendingQuestion> {
    match inbox {
        Some(inbox) => inbox.questions.recv().await,
        None => std::future::pending().await,
    }
}

/// The `user-questions/request` waterfall frame for one question batch.
fn question_frame(event_id: &str, question: &PendingQuestion) -> Value {
    json!({
        "type": "waterfall",
        "event": "user-questions/request",
        "eventId": event_id,
        "agentId": question.agent.as_deref().unwrap_or("host"),
        "request": { "questions": question.questions },
    })
}

/// The `approval/request` waterfall frame for one question.
///
/// The TypeScript client shows `toolName` and `reason` only, so the complete
/// arguments go into `reason`: the person approving reads everything they are
/// authorising, and it is never truncated.
fn approval_frame(event_id: &str, question: &PendingApproval) -> Value {
    json!({
        "type": "waterfall",
        "event": "approval/request",
        "eventId": event_id,
        "agentId": question.agent.as_deref().unwrap_or("host"),
        "request": {
            "toolName": question.tool,
            "reason": format!(
                "`{}` was called with these arguments:\n{}",
                question.tool, question.arguments
            ),
        },
    })
}

/// Queue one `item` frame, giving up the moment the stream is cancelled, even
/// while the queue is full. Cancellation wins a tie, so a cancelled stream never
/// starts a new delivery.
pub(super) async fn deliver(
    out: &mpsc::Sender<String>,
    stream_id: &str,
    value: Value,
    cancel: &CancellationToken,
) -> Result<(), ()> {
    let frame = json!({ "type": "item", "streamId": stream_id, "value": value });
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err(()),
        sent = out.send(frame.to_string()) => sent.map_err(|_| ()),
    }
}

/// `session/control`: the baseline, then one `projection` frame whenever a
/// live session's model selection, request header, title, or pending inbox
/// changes. Other projection keys are fixed in this build, so there is nothing
/// to push for them.
async fn control_stream(
    ctx: &Context,
    config: &MuxConfig,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    const ENDPOINT: &str = "session/control";
    if args.as_object().is_none_or(|object| !object.is_empty()) {
        return Outcome::Failed(input_invalid(
            ENDPOINT,
            "args",
            "this stream takes an empty args object",
        ));
    }
    // Subscribe before the baseline goes out, so a change in between is not lost.
    // Keep subscription callbacks nonblocking; a stalled socket can accumulate
    // frames in this unbounded queue until the stream is cancelled.
    let (frames, mut changes) = mpsc::unbounded_channel::<Value>();
    let subscription = events::on::<SessionEventPayload, _>(ctx, move |ctx, payload| {
        let frames = frames.clone();
        Box::pin(async move {
            let Some((key, value)) = projection_update(&ctx, &payload) else {
                return Ok(Value::Null);
            };
            let _ = frames.send(json!({
                "type": "projection",
                "sessionId": payload.session,
                "key": key,
                "value": value,
                "seq": payload.event.seq,
            }));
            Ok(Value::Null)
        })
    }).await;
    let subscription = match subscription {
        Ok(guard) => guard,
        Err(error) => return Outcome::Failed(Failure::at(
            codes::INTERNAL,
            format!("could not subscribe to session changes: {error}"),
            ENDPOINT,
        )),
    };
    let outcome = 'stream: {
        let baseline = control_baseline(ctx, &config.agent_presets, &config.image_limits);
        if deliver(out, stream_id, baseline, cancel).await.is_err() {
            break 'stream Outcome::Cancelled;
        }
        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => break 'stream Outcome::Cancelled,
                frame = changes.recv() => {
                    let Some(frame) = frame else { break 'stream Outcome::Cancelled };
                    if deliver(out, stream_id, frame, cancel).await.is_err() {
                        break 'stream Outcome::Cancelled;
                    }
                }
            }
        }
    };
    let _ = subscription.dispose().await;
    outcome
}

/// The projection key a committed event changes, with its new value, or `None`
/// when the event changes no projection this build computes.
pub(super) fn projection_update(
    ctx: &Context,
    payload: &SessionEventPayload,
) -> Option<(&'static str, Value)> {
    let update = match &payload.event.data {
        crate::types::SessionEventData::SessionTitle { title } =>
            ("title", json!(title)),
        crate::types::SessionEventData::ModelSelection { provider, model } => {
            let Some(agent) = ctx
                .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                .and_then(|agents| agents.get(&payload.session))
            else { return None; };
            let next = super::router::DefaultModel {
                provider: provider.clone(), model: model.clone(),
            };
            ("modelSelection", super::router::model_selection_values(
                &*agent.session(), &next, Some(payload.event.seq),
            ))
        }
        crate::types::SessionEventData::RequestHeader { header } => {
            let Some(agent) = ctx
                .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                .and_then(|agents| agents.get(&payload.session))
            else { return None; };
            let next = super::router::selected_route(&*agent);
            let mut value = super::router::model_selection_values(
                &*agent.session(), &next, Some(payload.event.seq),
            );
            value["lastUsed"] = json!({
                "provider": header.config.provider, "model": header.config.model,
            });
            ("modelSelection", value)
        }
        crate::types::SessionEventData::PermissionPreset { .. }
        | crate::types::SessionEventData::SandboxMode { .. }
        | crate::types::SessionEventData::ApprovalPolicy { .. } => {
            let Some(table) = crate::permission::provided(ctx) else { return None; };
            let Some(agent) = ctx
                .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                .and_then(|agents| agents.get(&payload.session))
            else { return None; };
            let events = agent.session().events();
            let upto: Vec<_> =
                events.into_iter().filter(|e| e.seq <= payload.event.seq).collect();
            ("permissions", table.view(&upto))
        }
        crate::types::SessionEventData::PlanMode { .. }
        | crate::types::SessionEventData::CommandRun { .. }
        | crate::types::SessionEventData::CommandDone { .. } => {
            // Only a `/plan` run can change the projection by itself.
            if matches!(&payload.event.data,
                crate::types::SessionEventData::CommandRun { name, .. } if name != "plan")
            {
                return None;
            }
            if ctx
                .get::<crate::plugins::plan::PlanMode>(crate::plugins::plan::PLAN_SERVICE)
                .is_none()
            {
                return None;
            }
            let Some(agent) = ctx
                .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                .and_then(|agents| agents.get(&payload.session))
            else { return None; };
            (
                "plan",
                crate::plugins::plan::projection(
                    &agent.session().events(),
                    Some(payload.event.seq),
                ),
            )
        }
        crate::types::SessionEventData::GoalChange { .. } => {
            let Some(agent) = ctx
                .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                .and_then(|agents| agents.get(&payload.session))
            else { return None; };
            ("goal", super::goals::projection_as_of(&agent.session().events(), payload.event.seq))
        }
        crate::types::SessionEventData::InboxSpliced { .. } => {
            let Some(agent) = ctx
                .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                .and_then(|agents| agents.get(&payload.session))
            else { return None; };
            match super::router::inbox_values(
                &*agent.session(),
                Some(payload.event.seq),
            ) {
                Ok(value) => ("inbox", value),
                Err(reason) => {
                    // A log that cannot be folded is reported, not
                    // papered over with a wrong pending list.
                    crate::api::events::emit(
                        ctx,
                        &crate::api::events::AgentErrorPayload {
                            agent: payload.session.to_string(),
                            error: reason,
                        },
                    );
                    return None;
                }
            }
        }
        _ => return None,
    };
    Some(update)
}

/// Every live session with the projection values dsh-rs can compute, each at
/// its current event cursor.
fn control_baseline(
    ctx: &Context,
    presets: &[super::router::AgentPreset],
    image_limits: &crate::images::ImageLimits,
) -> Value {
    let mut projections = Map::new();
    if let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) {
        for agent in agents.list() {
            let cursor = agent.session().events().last().map_or(-1, |e| e.seq as i64);
            let values =
                match super::router::projection_values(
                    ctx,
                    &*agent.session(),
                    &super::router::selected_route(&*agent),
                    presets,
                    image_limits,
                )
                {
                    Ok(values) => values,
                    Err(reason) => {
                        // A session whose log cannot be folded has no
                        // trustworthy projection block.
                        crate::api::events::emit(
                            ctx,
                            &crate::api::events::AgentErrorPayload {
                                agent: agent.id().to_string(),
                                error: reason,
                            },
                        );
                        continue;
                    }
                };
            projections.insert(
                agent.id().to_string(),
                json!({
                    "asOfSeq": cursor,
                    "values": values,
                }),
            );
        }
    }
    json!({ "type": "baseline", "value": { "projections": projections } })
}

/// A stream whose whole content is one baseline: send it, then stay open until
/// cancelled, as a TypeScript stream does while nothing changes.
async fn held_open(
    args: &Value,
    endpoint: &str,
    baseline: Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    if args.as_object().is_none_or(|object| !object.is_empty()) {
        return Outcome::Failed(input_invalid(
            endpoint,
            "args",
            "this stream takes an empty args object",
        ));
    }
    if deliver(out, stream_id, baseline, cancel).await.is_err() {
        return Outcome::Cancelled;
    }
    cancel.cancelled().await;
    Outcome::Cancelled
}

/// `workspace/follow`: the complete registry, then every change in commit order.
///
/// A follower that falls further behind than the channel holds is sent a
/// `replace` carrying the current state, which supersedes whatever it missed.
async fn workspace_follow(
    ctx: &Context,
    config: &MuxConfig,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    const ENDPOINT: &str = "workspace/follow";
    if args.as_object().is_none_or(|object| !object.is_empty()) {
        return Outcome::Failed(input_invalid(
            ENDPOINT,
            "args",
            "the workspace stream takes an empty args object",
        ));
    }
    let registry = &config.workspaces;
    let (baseline, mut frames) = registry.subscribe(&super::router::session_refs(ctx));
    if deliver(
        out,
        stream_id,
        json!({ "type": "baseline", "value": baseline }),
        cancel,
    )
    .await
    .is_err()
    {
        return Outcome::Cancelled;
    }
    let mut poll = tokio::time::interval(config.workspace_poll);
    poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    poll.tick().await; // the first tick is immediate; the baseline just covered it
    loop {
        let frame = tokio::select! {
            biased;
            _ = cancel.cancelled() => return Outcome::Cancelled,
            _ = poll.tick() => {
                registry.sessions_changed(&super::router::session_refs(ctx));
                continue;
            }
            received = frames.recv() => match received {
                Ok(frame) => frame,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    // Frames still queued may predate a fresh baseline, so the
                    // baseline and a new subscription are taken together and
                    // the old queue is dropped.
                    let (baseline, fresh) = registry.subscribe(&super::router::session_refs(ctx));
                    frames = fresh;
                    json!({ "type": "replace", "value": baseline })
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return Outcome::Ended,
            },
        };
        if deliver(out, stream_id, frame, cancel).await.is_err() {
            return Outcome::Cancelled;
        }
    }
}

/// `session/follow`: one snapshot, then every later event in order.
///
/// The cordis bus delivers `session/event` through detached tasks, so events can
/// arrive out of order. The bus therefore only *wakes* this loop; the session
/// log is read from the cursor on each wake, which keeps the stream ordered and
/// gap-free whatever the delivery order, and it subscribes before reading the
/// snapshot so no append can fall between the two.
async fn follow(
    ctx: &Context,
    config: &MuxConfig,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    const ENDPOINT: &str = "session/follow";
    let Some(request) = argument(args, "request", ENDPOINT) else {
        return Outcome::Failed(input_invalid(ENDPOINT, "request", "request is required"));
    };
    let Some(address) = request.get("address") else {
        return Outcome::Failed(input_invalid(ENDPOINT, "address", "address is required"));
    };
    let Some(session_id) = address.get("sessionId").and_then(Value::as_str) else {
        return Outcome::Failed(input_invalid(
            ENDPOINT,
            "address.sessionId",
            "only { kind: \"session\" } addresses are served by this build",
        ));
    };
    let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
        return failure_of(unavailable(ENDPOINT, "agents"));
    };
    let Some(agent) = agents.get(session_id) else {
        return Outcome::Failed(Failure::at(
            codes::LOOKUP_NOT_FOUND,
            format!("no live session {session_id:?}"),
            ENDPOINT,
        ));
    };

    let (wake, mut woken) = mpsc::channel::<()>(1);
    let watched = session_id.to_string();
    let subscription = events::on::<SessionEventPayload, _>(ctx, move |_ctx, payload| {
        let wake = wake.clone();
        let watched = watched.clone();
        Box::pin(async move {
            if payload.session == watched {
                // A full channel means a wake is already pending; that is enough.
                let _ = wake.try_send(());
            }
            Ok(Value::Null)
        })
    })
    .await;
    let subscription = match subscription {
        Ok(guard) => guard,
        Err(error) => {
            return Outcome::Failed(Failure::at(
                codes::INTERNAL,
                format!("could not subscribe to session events: {error}"),
                ENDPOINT,
            ))
        }
    };

    let session = agent.session();
    let mut cursor: Option<u64> = None;
    let opening = session.events();
    // One encoder for the whole stream: a tool result cites its call's seq.
    let mut encoder = RecordEncoder::default();
    let (records, has_more) = window(&opening, config.max_page_events, &mut encoder);
    cursor = opening.last().map(|event| event.seq).or(cursor);
    // Inclusive watermark of the last event the snapshot covers, `-1` for an
    // empty log. Reporting 0 there would claim event 0 was already delivered.
    let watermark: i64 = cursor.map_or(-1, |seq| seq as i64);
    // A follower that opts into the assistant stream insists on its baseline.
    // dsh-rs publishes no token-level frames, so the baseline is the empty one
    // and replies arrive as committed events.
    let wants_assistant_stream =
        request.get("assistantStream").and_then(Value::as_bool) == Some(true);
    let mut snapshot = json!({
        "type": "snapshot",
        "header": {
            "version": 4,
            "id": session_id,
            "createdAt": opening.first().map(|event| event.time).unwrap_or(0),
            "cwd": session.header_cwd(),
            "isSeeded": false,
        },
        "cursor": watermark,
        "records": records,
        "hasMore": has_more,
        "projections": {
            "asOfSeq": watermark,
            "values": match super::router::projection_values(
                &ctx,
                &*session,
                &super::router::selected_route(&*agent),
                &config.agent_presets,
                &config.image_limits,
            ) {
                Ok(values) => values,
                Err(reason) => {
                    crate::api::events::emit(
                        &ctx,
                        &crate::api::events::AgentErrorPayload {
                            agent: agent.id().to_string(),
                            error: reason,
                        },
                    );
                    Value::Null
                }
            },
        },
    });

    if wants_assistant_stream {
        snapshot["assistantStream"] = json!({ "revision": 0 });
    }

    let outcome = 'stream: {
        if deliver(out, stream_id, snapshot, cancel).await.is_err() {
            break 'stream Outcome::Cancelled;
        }
        loop {
            // Drain first: events appended while the snapshot was sent, or
            // while the previous batch was written, are already in the log.
            let pending: Vec<SessionEvent> = session
                .events()
                .into_iter()
                .filter(|event| cursor.is_none_or(|seen| event.seq > seen))
                .collect();
            for event in pending {
                cursor = Some(event.seq);
                if deliver(out, stream_id, encoder.record(&event), cancel)
                    .await
                    .is_err()
                {
                    break 'stream Outcome::Cancelled;
                }
            }
            tokio::select! {
                _ = cancel.cancelled() => break 'stream Outcome::Cancelled,
                woke = woken.recv() => {
                    if woke.is_none() {
                        break 'stream Outcome::Ended;
                    }
                }
            }
        }
    };
    let _ = subscription.dispose().await;
    outcome
}

/// The newest `limit` events as records, and whether older ones were left out.
fn window(
    events: &[SessionEvent],
    limit: usize,
    encoder: &mut RecordEncoder,
) -> (Vec<Value>, bool) {
    let skipped = events.len().saturating_sub(limit);
    // Events before the window are still encoded, so the encoder knows the
    // calls a later result cites.
    let mut records: Vec<Value> = events.iter().map(|event| encoder.record(event)).collect();
    records.drain(..skipped);
    (records, skipped > 0)
}

fn failure_of(result: super::wire::RpcResult) -> Outcome {
    match result {
        super::wire::RpcResult::Failed(failure) => Outcome::Failed(failure),
        super::wire::RpcResult::Ok(_)
        | super::wire::RpcResult::WithAttachments { .. }
        | super::wire::RpcResult::Empty => Outcome::Ended,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quiet_mux() -> (Mux, mpsc::Receiver<String>) {
        let (out, frames) = mpsc::channel(8);
        let config = MuxConfig {
            home: "/h".into(),
            max_page_events: 10,
            agent_presets: crate::gui::router::default_presets(),
            image_limits: crate::images::ImageConfig::default().limits(),
            events: Default::default(),
            approval_sweep: std::time::Duration::from_secs(1),
            workspaces: Default::default(),
            workspace_poll: std::time::Duration::from_secs(1),
            default_model: crate::gui::router::DefaultModel {
                provider: "p".into(),
                model: "m".into(),
            },
            terminal: None,
            product_analytics: None,
            settings: std::sync::Arc::default(),
        };
        (Mux::new(Context::new(), config, out), frames)
    }

    #[tokio::test]
    async fn a_stream_cancelled_before_it_first_runs_sends_nothing() {
        // The default test runtime is single-threaded, so the spawned stream
        // cannot run until this test yields: `cancel` is certain to land first.
        let (mux, mut frames) = quiet_mux();
        mux.receive(r#"{"type":"open","streamId":"s","endpoint":"$events","payload":{"args":{}}}"#)
            .unwrap();
        mux.receive(r#"{"type":"cancel","streamId":"s"}"#).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert!(frames.try_recv().is_err(), "no ready frame after a cancel");
        assert_eq!(
            mux.open_streams(),
            0,
            "the cancelled stream leaves the table"
        );
    }

    #[tokio::test]
    async fn the_terminal_frame_is_queued_before_the_id_can_be_reused() {
        let (mux, mut frames) = quiet_mux();
        let open =
            r#"{"type":"open","streamId":"s","endpoint":"$events","payload":{"args":{"x":1}}}"#;
        mux.receive(open).unwrap();
        let terminal = tokio::time::timeout(std::time::Duration::from_secs(1), frames.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(terminal.contains(r#""type":"error""#));
        // Reading the terminal frame is the client's cue to reuse the id.
        assert!(
            mux.receive(open).is_ok(),
            "the id is free once the terminal frame is out"
        );
    }

    /// A mux whose one-slot queue is already full, so any send must wait.
    fn jammed_mux() -> (Mux, mpsc::Receiver<String>) {
        let (out, frames) = mpsc::channel(1);
        out.try_send("jam".into()).unwrap();
        let config = MuxConfig {
            home: "/h".into(),
            max_page_events: 10,
            agent_presets: crate::gui::router::default_presets(),
            image_limits: crate::images::ImageConfig::default().limits(),
            events: Default::default(),
            approval_sweep: std::time::Duration::from_secs(1),
            workspaces: Default::default(),
            workspace_poll: std::time::Duration::from_secs(1),
            default_model: crate::gui::router::DefaultModel {
                provider: "p".into(),
                model: "m".into(),
            },
            terminal: None,
            product_analytics: None,
            settings: std::sync::Arc::default(),
        };
        (Mux::new(Context::new(), config, out), frames)
    }

    #[tokio::test]
    async fn cancel_frees_a_stream_that_is_blocked_on_a_full_queue() {
        let (mux, _frames) = jammed_mux();
        mux.receive(r#"{"type":"open","streamId":"s","endpoint":"$events","payload":{"args":{}}}"#)
            .unwrap();
        // Let the stream run until it is stuck waiting to queue its ready frame.
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        assert_eq!(mux.open_streams(), 1, "the stream is waiting on the queue");
        mux.receive(r#"{"type":"cancel","streamId":"s"}"#).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        assert_eq!(
            mux.open_streams(),
            0,
            "cancel must not wait for queue space"
        );
    }

    #[tokio::test]
    async fn an_id_stays_claimed_until_its_terminal_frame_is_queued() {
        let (mux, mut frames) = jammed_mux();
        let open =
            r#"{"type":"open","streamId":"s","endpoint":"$events","payload":{"args":{"x":1}}}"#;
        mux.receive(open).unwrap();
        // The failing stream now waits for room for its error frame.
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        assert_eq!(
            mux.receive(open),
            Err(DUPLICATE_STREAM),
            "a new generation must not start while the old terminal frame is unsent"
        );
        // Make room: the old terminal frame goes out, and only then is the id free.
        assert_eq!(frames.recv().await.as_deref(), Some("jam"));
        let terminal = frames.recv().await.unwrap();
        assert!(terminal.contains(r#""type":"error""#));
        assert!(mux.receive(open).is_ok());
    }

    #[test]
    fn open_requires_exactly_its_four_fields() {
        let ok = r#"{"type":"open","streamId":"s","endpoint":"e","payload":{}}"#;
        assert!(matches!(
            parse_client_frame(ok),
            Some(ClientFrame::Open { .. })
        ));
        let extra = r#"{"type":"open","streamId":"s","endpoint":"e","payload":{},"x":1}"#;
        assert_eq!(parse_client_frame(extra), None);
        let missing = r#"{"type":"open","streamId":"s","endpoint":"e"}"#;
        assert_eq!(parse_client_frame(missing), None);
    }

    #[test]
    fn an_empty_stream_id_or_endpoint_is_invalid() {
        assert_eq!(
            parse_client_frame(r#"{"type":"cancel","streamId":""}"#),
            None
        );
        assert_eq!(
            parse_client_frame(r#"{"type":"open","streamId":"s","endpoint":"","payload":{}}"#),
            None
        );
    }

    #[test]
    fn item_may_omit_its_value_but_nothing_else() {
        assert!(parse_client_frame(r#"{"type":"item","streamId":"s"}"#).is_some());
        assert!(parse_client_frame(r#"{"type":"item","streamId":"s","value":1}"#).is_some());
        assert_eq!(
            parse_client_frame(r#"{"type":"item","streamId":"s","value":1,"y":2}"#),
            None
        );
    }

    #[test]
    fn unknown_frame_types_and_non_objects_are_invalid() {
        assert_eq!(
            parse_client_frame(r#"{"type":"nope","streamId":"s"}"#),
            None
        );
        assert_eq!(parse_client_frame("[1]"), None);
        assert_eq!(parse_client_frame("not json"), None);
    }

    #[test]
    fn the_window_keeps_the_newest_events_and_reports_the_cut() {
        use crate::types::SessionEventData;
        let events: Vec<SessionEvent> = (0..5)
            .map(|n| SessionEvent::new(n, 0, SessionEventData::TurnStart { turn: n }))
            .collect();
        let (records, more) = window(&events, 2, &mut RecordEncoder::default());
        assert!(more);
        let seqs: Vec<u64> = records
            .iter()
            .map(|r| r["event"]["seq"].as_u64().unwrap())
            .collect();
        assert_eq!(seqs, vec![3, 4]);
        let (all, more) = window(&events, 10, &mut RecordEncoder::default());
        assert!(!more);
        assert_eq!(all.len(), 5);
    }
}
