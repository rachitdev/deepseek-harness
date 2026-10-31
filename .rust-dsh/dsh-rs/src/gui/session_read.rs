//! Passive session reads: `session/state`, `session/events`, `session/wait`
//! and the streams `session/observeState`, `session/observeEvent`,
//! `session/observeProjections`.
//!
//! None of these starts, resumes or cancels a session. A live session is read
//! from its agent; a stored session no agent covers is read from the session
//! store and reported inactive with no readable queue, as the TypeScript host
//! does. A session whose header has no working directory does not exist for
//! these calls.
//!
//! The rules follow the TypeScript `HistoryController` and `StateController`:
//!
//! - `session/events` returns one message-aligned page ending at `throughSeq`
//!   (default: the log end), at most `maxMessages` (default 50) messages, cut at
//!   the group start of the oldest message so a message and the events it
//!   replaces are never split. `turnWindow` extends the page back to a turn start.
//! - `session/state` reports `cursor` (last seq, `-1` for an empty log), whether
//!   the agent is `active` and `running`, its pending `queue` (`null` when no
//!   agent is live) and the kinds of question awaiting a human (`pendingInput`).
//! - `session/wait` returns when input is required, the session is idle, or the
//!   deadline passes (default 30 s), with the newest-first assistant messages in
//!   `(afterSeq, cursor]`.
//!
//! **Divergences.** This build keeps no durable record that a session is a
//! sub-agent, so a `subagent` address is accepted when the child's header names
//! the parent (and, when the child is live, when the sub-agent plugin started
//! it), and a `mode` other than `unknown` or `one-shot` is refused: sub-agents
//! here are always one-shot. `pendingInput` reports `approval` and `question`
//! only; there is no `other` kind. A `session` address is accepted for a
//! sub-agent child, where TypeScript refuses it with `session/agent-busy`.

use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use serde_json::{json, Map, Value};
use tokio::sync::{mpsc, Notify};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use super::events::EventHub;
use super::mux::{deliver, MuxConfig, Outcome};
use super::router::{
    boundary, persistence, session_not_found, stored_header, unencodable_row, Router,
};
use super::wire::{codes, Failure, RpcResult};
use crate::api::events::{self, SessionEventPayload};
use crate::api::services::{AgentRegistryService, AgentView};
use crate::session::v4::{encode_event, EncodeState, V4Row};
use crate::types::{AgentStatus, InboxTarget, SessionEvent};

/// Messages in a page when the request names no budget.
const DEFAULT_MAX_MESSAGES: u64 = 50;
/// How long `session/wait` runs when the request names no timeout.
const DEFAULT_WAIT_TIMEOUT_MS: u64 = 30_000;
/// The largest delay a timer accepts (2^31 - 1 ms), the TypeScript limit.
const MAX_TIMER_DELAY_MS: u64 = 2_147_483_647;
/// How often a state is sampled again when no session event woke the reader.
/// The TypeScript host refreshes on the same interval, because a question
/// appearing or disappearing is not a session event.
const RESAMPLE: Duration = Duration::from_secs(1);

/// Serve one of the three unary endpoints, or `None` for another method.
pub(super) async fn dispatch(
    router: &Router,
    ctx: &Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    let result = match method {
        "session/state" => state(router, ctx, payload),
        "session/events" => events_page(ctx, payload),
        "session/wait" => wait(router, ctx, payload).await,
        _ => return None,
    };
    Some(match result {
        Ok(value) => RpcResult::ok(value),
        Err(failure) => RpcResult::failed(failure),
    })
}

// --- request parsing ----------------------------------------------------

/// The `request` object, which the gateway has already typed as an object.
fn request_of<'a>(endpoint: &str, payload: &'a Value) -> Result<&'a Map<String, Value>, Failure> {
    payload
        .get("request")
        .and_then(Value::as_object)
        .ok_or_else(|| boundary(endpoint, "request"))
}

fn text_field<'a>(endpoint: &str, request: &'a Map<String, Value>, name: &str) -> Result<&'a str, Failure> {
    request
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| boundary(endpoint, "request"))
}

/// An optional number the wire type allows; any other JSON type fails the gateway.
fn number_field(endpoint: &str, request: &Map<String, Value>, name: &str) -> Result<Option<f64>, Failure> {
    match request.get(name) {
        None => Ok(None),
        Some(value) => value
            .as_f64()
            .map(Some)
            .ok_or_else(|| boundary(endpoint, "request")),
    }
}

fn bad_request(message: &str) -> Failure {
    Failure {
        code: codes::BAD_REQUEST.to_string(),
        message: message.to_string(),
        details: json!({}),
    }
}

/// `value` as a safe integer, or `None`. Negative zero is not an integer here.
fn safe_integer(value: f64) -> Option<i64> {
    let in_range = value.fract() == 0.0 && value.abs() <= 9_007_199_254_740_991.0;
    (in_range && !(value == 0.0 && value.is_sign_negative())).then_some(value as i64)
}

/// A session address: an ordinary session or a direct sub-agent.
enum Address {
    Session { id: String },
    Subagent { parent: String, child: String, mode: String },
}

impl Address {
    fn parse(endpoint: &str, value: Option<&Value>) -> Result<Address, Failure> {
        let object = value
            .and_then(Value::as_object)
            .ok_or_else(|| boundary(endpoint, "request"))?;
        let text = |name: &str| text_field(endpoint, object, name).map(str::to_string);
        match object.get("kind").and_then(Value::as_str) {
            Some("session") => Ok(Address::Session { id: text("sessionId")? }),
            Some("subagent") => {
                let mode = text("mode")?;
                if !matches!(mode.as_str(), "one-shot" | "continuable" | "unknown") {
                    return Err(boundary(endpoint, "request"));
                }
                Ok(Address::Subagent {
                    parent: text("parentSessionId")?,
                    child: text("childSessionId")?,
                    mode,
                })
            }
            _ => Err(boundary(endpoint, "request")),
        }
    }

    /// The session whose log the address reads.
    fn session_id(&self) -> &str {
        match self {
            Address::Session { id } => id,
            Address::Subagent { child, .. } => child,
        }
    }

    fn not_found(&self) -> Failure {
        match self {
            Address::Session { id } => session_not_found_failure(id),
            Address::Subagent { parent, child, .. } => Failure {
                code: "subagent/not-found".to_string(),
                message: "subagent is unavailable".to_string(),
                details: json!({ "parentSessionId": parent, "childSessionId": child }),
            },
        }
    }
}

fn session_not_found_failure(id: &str) -> Failure {
    match session_not_found(id) {
        RpcResult::Failed(failure) => failure,
        _ => unreachable!("session_not_found always answers a failure"),
    }
}

// --- reading a session --------------------------------------------------

/// One session as a passive reader sees it.
pub(super) struct Observed {
    pub(super) events: Vec<SessionEvent>,
    pub(super) cwd: Option<String>,
    pub(super) parent: Option<String>,
    /// The live agent, when one covers the session.
    pub(super) agent: Option<Arc<dyn AgentView>>,
}

impl Observed {
    /// The last seq in the log, `-1` for an empty log.
    fn cursor(&self) -> i64 {
        self.events.last().map_or(-1, |event| event.seq as i64)
    }
}

/// Read `session_id` from its live agent, else from the store; `None` when
/// neither has it.
pub(super) fn observe_session(ctx: &Context, session_id: &str) -> Result<Option<Observed>, Failure> {
    if let Some(agent) = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .and_then(|agents| agents.get(session_id))
    {
        let session = agent.session();
        return Ok(Some(Observed {
            events: session.events(),
            cwd: session.header_cwd(),
            parent: session.parent_session(),
            agent: Some(agent),
        }));
    }
    let Some(store) = persistence(ctx) else {
        return Ok(None);
    };
    if !store.list().iter().any(|id| id == session_id) {
        return Ok(None);
    }
    let events = store.load(session_id).map_err(|error| {
        Failure::at(
            codes::INTERNAL,
            format!("the stored session could not be read: {error}"),
            "session",
        )
    })?;
    let header = stored_header(ctx, session_id);
    Ok(Some(Observed {
        events,
        cwd: header.as_ref().and_then(|h| h.cwd.clone()),
        parent: header.and_then(|h| h.parent_session.map(|id| id.to_string())),
        agent: None,
    }))
}

/// The session behind `address`, checked the way the TypeScript host checks it.
fn observe_address(ctx: &Context, address: &Address) -> Result<Observed, Failure> {
    let observed = observe_session(ctx, address.session_id())?
        .filter(|observed| observed.cwd.is_some())
        .ok_or_else(|| address.not_found())?;
    if let Address::Subagent { parent, child, mode } = address {
        let unauthorized = |message: &str| Failure {
            code: "subagent/unauthorized".to_string(),
            message: message.to_string(),
            details: json!({ "childSessionId": child }),
        };
        if observed.parent.as_deref() != Some(parent.as_str()) {
            return Err(unauthorized("subagent does not belong to the supplied parent"));
        }
        let started_here = ctx
            .get::<crate::plugins::subagent::Lineage>(crate::plugins::subagent::LINEAGE_SERVICE)
            .map(|lineage| lineage.parent_of(child));
        if observed.agent.is_some() && started_here.is_some_and(|started| started.as_deref() != Some(parent.as_str())) {
            return Err(unauthorized("subagent does not belong to the supplied parent"));
        }
        if !matches!(mode.as_str(), "unknown" | "one-shot") {
            return Err(unauthorized("subagent mode does not match the supplied address"));
        }
    }
    Ok(observed)
}

/// Every event as its v4 row, in log order, so a tool result can cite its call.
fn rows_of(events: &[SessionEvent]) -> Vec<V4Row> {
    let mut state = EncodeState::default();
    events
        .iter()
        .map(|event| {
            encode_event(event, &mut state)
                .unwrap_or_else(|error| unencodable_row(event, &error.to_string()))
        })
        .collect()
}

fn record(row: &V4Row) -> Value {
    json!({ "type": "event", "event": row })
}

/// Whether the row is a user or assistant message that appends to the surface.
fn is_message(row: &V4Row) -> bool {
    matches!(row.kind.as_str(), "user/message" | "assistant/message")
        && row.surface_op.as_ref().and_then(Value::as_str) == Some("append")
}

// --- session/events -----------------------------------------------------

struct PageRequest {
    through: Option<i64>,
    before: Option<i64>,
    max_messages: u64,
    min_messages: Option<u64>,
    min_turns: Option<u64>,
}

impl PageRequest {
    fn parse(endpoint: &str, request: &Map<String, Value>) -> Result<PageRequest, Failure> {
        let through = number_field(endpoint, request, "throughSeq")?;
        let before = number_field(endpoint, request, "beforeSeq")?;
        let max = number_field(endpoint, request, "maxMessages")?;
        let window = match request.get("turnWindow") {
            None => None,
            Some(Value::Object(window)) => Some((
                number_field(endpoint, window, "minMessages")?.ok_or_else(|| boundary(endpoint, "request"))?,
                number_field(endpoint, window, "minTurns")?.ok_or_else(|| boundary(endpoint, "request"))?,
            )),
            Some(_) => return Err(boundary(endpoint, "request")),
        };
        let through = match through {
            None => None,
            Some(value) => Some(
                safe_integer(value)
                    .filter(|seq| *seq >= -1)
                    .ok_or_else(|| bad_request("throughSeq must be an integer greater than or equal to -1"))?,
            ),
        };
        let before = match before {
            None => None,
            Some(value) => Some(
                safe_integer(value)
                    .filter(|seq| *seq >= 0)
                    .ok_or_else(|| bad_request("beforeSeq must be a non-negative safe integer"))?,
            ),
        };
        let positive = |value: f64| safe_integer(value).filter(|n| *n > 0).map(|n| n as u64);
        let max_messages = match max {
            None => DEFAULT_MAX_MESSAGES,
            Some(value) => positive(value)
                .ok_or_else(|| bad_request("maxMessages must be a positive safe integer"))?,
        };
        let (mut min_messages, mut min_turns) = (None, None);
        if let Some((messages, turns)) = window {
            let messages = positive(messages)
                .filter(|n| *n <= max_messages)
                .ok_or_else(|| {
                    bad_request("turnWindow.minMessages must be a positive safe integer no greater than maxMessages")
                })?;
            let turns = positive(turns)
                .ok_or_else(|| bad_request("turnWindow.minTurns must be a positive safe integer"))?;
            (min_messages, min_turns) = (Some(messages), Some(turns));
        }
        Ok(PageRequest { through, before, max_messages, min_messages, min_turns })
    }
}

/// `session/events`: one chronological, message-aligned page.
fn events_page(ctx: &Context, payload: &Value) -> Result<Value, Failure> {
    const ENDPOINT: &str = "session/events";
    let request = request_of(ENDPOINT, payload)?;
    let address = Address::parse(ENDPOINT, request.get("address"))?;
    let page = PageRequest::parse(ENDPOINT, request)?;
    let observed = observe_address(ctx, &address)?;
    let log_end = observed.cursor();
    let cursor = page.through.unwrap_or(log_end);
    if cursor > log_end {
        return Err(bad_request(&format!("session history cut {cursor} exceeds {log_end}")));
    }
    let rows = rows_of(&observed.events);
    let end = usize::try_from(cursor + 1)
        .unwrap_or(0)
        .min(page.before.map_or(usize::MAX, |before| before as usize))
        .min(rows.len());
    let cut = page_start(&rows[..end], &page);
    let records: Vec<Value> = rows[cut..end].iter().map(record).collect();
    Ok(json!({ "cursor": cursor, "records": records, "hasMore": cut > 0 }))
}

/// The index of the first row of the page that ends before `rows.len()`.
fn page_start(rows: &[V4Row], page: &PageRequest) -> usize {
    let (mut messages, mut turns) = (0u64, 0u64);
    for index in (0..rows.len()).rev() {
        let row = &rows[index];
        if let (Some(min_messages), Some(min_turns)) = (page.min_messages, page.min_turns) {
            if row.kind == "turn/start" {
                turns += 1;
                if messages >= min_messages && turns >= min_turns {
                    return index;
                }
            }
        }
        if !is_message(row) {
            continue;
        }
        messages += 1;
        if messages >= page.max_messages {
            let sources = row.source_event_seqs.as_deref().unwrap_or_default();
            return sources.iter().copied().fold(row.seq, u64::min) as usize;
        }
    }
    0
}

// --- session/state ------------------------------------------------------

/// The passive state of `session_id` as the wire carries it.
fn read_state(hub: &EventHub, ctx: &Context, session_id: &str) -> Result<Value, Failure> {
    let observed = observe_session(ctx, session_id)?
        .filter(|observed| observed.cwd.is_some())
        .ok_or_else(|| session_not_found_failure(session_id))?;
    let pending: Vec<&str> = hub.pending_kinds(session_id);
    let (active, running, queue) = match &observed.agent {
        Some(agent) => (
            true,
            agent.status() == AgentStatus::Running,
            queue_items(&*agent.session())?,
        ),
        None => (false, false, Value::Null),
    };
    Ok(json!({
        "cursor": observed.cursor(),
        "active": active,
        "running": running,
        "queue": queue,
        "pendingInput": pending,
    }))
}

/// The pending inbox: next-turn items are `queued`; next-step items are
/// `steering` when a person sent them and `context` otherwise.
fn queue_items(session: &dyn crate::api::services::SessionView) -> Result<Value, Failure> {
    use crate::types::MessageSource;
    let inbox = crate::core::agent::Inbox::fold(&session.events(), None).map_err(|reason| {
        Failure::at(codes::INTERNAL, format!("the session inbox could not be read: {reason}"), "session/state")
    })?;
    let mut items = Vec::new();
    for (target, steering) in [(InboxTarget::NextTurn, false), (InboxTarget::NextStep, true)] {
        for message in inbox.pending(target) {
            let from_user = matches!(message.source, MessageSource::User { .. });
            let placement = match (steering, from_user) {
                (false, _) => "queued",
                (true, true) => "steering",
                (true, false) => "context",
            };
            let mut item = json!({
                "id": message.id,
                "placement": placement,
                "content": serde_json::to_value(&message.content).unwrap_or(Value::Null),
            });
            if let MessageSource::User { rpc_id: Some(rpc_id) } = &message.source {
                item["rpcId"] = json!(rpc_id);
            }
            items.push(item);
        }
    }
    Ok(Value::Array(items))
}

fn state(router: &Router, ctx: &Context, payload: &Value) -> Result<Value, Failure> {
    const ENDPOINT: &str = "session/state";
    let request = request_of(ENDPOINT, payload)?;
    let session_id = text_field(ENDPOINT, request, "sessionId")?;
    read_state(&router.events(), ctx, session_id)
}

// --- session/wait -------------------------------------------------------

/// Wake a waiting reader whenever the session appends an event.
async fn watch(ctx: &Context, session_id: &str) -> Result<(Arc<Notify>, cordis::fiber::EffectGuard), Failure> {
    let notify = Arc::new(Notify::new());
    let (wake, watched) = (Arc::clone(&notify), session_id.to_string());
    let guard = events::on::<SessionEventPayload, _>(ctx, move |_ctx, payload| {
        let (wake, watched) = (Arc::clone(&wake), watched.clone());
        Box::pin(async move {
            if payload.session == watched {
                wake.notify_one();
            }
            Ok(Value::Null)
        })
    })
    .await
    .map_err(|error| {
        Failure::at(codes::INTERNAL, format!("could not subscribe to session events: {error}"), "session")
    })?;
    Ok((notify, guard))
}

async fn wait(router: &Router, ctx: &Context, payload: &Value) -> Result<Value, Failure> {
    const ENDPOINT: &str = "session/wait";
    let request = request_of(ENDPOINT, payload)?;
    let session_id = text_field(ENDPOINT, request, "sessionId")?;
    let timeout = number_field(ENDPOINT, request, "timeoutMs")?;
    let after = number_field(ENDPOINT, request, "afterSeq")?;
    let max = number_field(ENDPOINT, request, "maxMessages")?;
    let after_seq = match after {
        None => -1,
        Some(value) => safe_integer(value)
            .filter(|seq| *seq >= -1)
            .ok_or_else(|| bad_request("afterSeq must be an integer greater than or equal to -1"))?,
    };
    let max_messages = match max {
        None => DEFAULT_MAX_MESSAGES,
        Some(value) => safe_integer(value)
            .filter(|n| *n > 0)
            .map(|n| n as u64)
            .ok_or_else(|| bad_request("maxMessages must be a positive safe integer"))?,
    };
    let timeout_ms = match timeout {
        None => DEFAULT_WAIT_TIMEOUT_MS,
        Some(value) => safe_integer(value)
            .filter(|n| *n > 0 && *n as u64 <= MAX_TIMER_DELAY_MS)
            .map(|n| n as u64)
            .ok_or_else(|| bad_request("timeoutMs must be a positive supported timer delay"))?,
    };
    let hub = router.events();
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let (woken, subscription) = watch(ctx, session_id).await?;
    let outcome = wait_loop(&hub, ctx, session_id, deadline, &woken).await;
    let _ = subscription.dispose().await;
    let (reason, state) = outcome?;
    let cursor = state["cursor"].as_i64().unwrap_or(-1);
    let output = assistant_output(ctx, session_id, after_seq, cursor, max_messages)?;
    Ok(json!({ "reason": reason, "state": state, "output": output }))
}

/// Sample the state until a stopping condition or the deadline; the last
/// complete state is the one a timeout reports.
async fn wait_loop(
    hub: &EventHub,
    ctx: &Context,
    session_id: &str,
    deadline: Instant,
    woken: &Notify,
) -> Result<(&'static str, Value), Failure> {
    loop {
        let state = read_state(hub, ctx, session_id)?;
        if !state["pendingInput"].as_array().is_none_or(Vec::is_empty) {
            return Ok(("input-required", state));
        }
        let running = state["running"] == json!(true);
        if state["active"] == json!(false) {
            return Ok(("idle", state));
        }
        let idle = async {
            if running {
                std::future::pending::<()>().await
            } else if let Some(agent) = ctx
                .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                .and_then(|agents| agents.get(session_id))
            {
                agent.when_idle().await;
            }
        };
        tokio::select! {
            _ = idle => {
                // The agent settled; the stopping state is the one sampled now.
                let settled = read_state(hub, ctx, session_id)?;
                if settled["running"] == json!(false)
                    && settled["pendingInput"].as_array().is_none_or(Vec::is_empty)
                {
                    return Ok(("idle", settled));
                }
            }
            _ = woken.notified() => {}
            _ = tokio::time::sleep(RESAMPLE) => {}
            _ = tokio::time::sleep_until(deadline) => return Ok(("timeout", state)),
        }
    }
}

/// Committed assistant messages in `(after_seq, through]`, newest first, and
/// whether more matching messages were left out.
fn assistant_output(
    ctx: &Context,
    session_id: &str,
    after_seq: i64,
    through: i64,
    max_messages: u64,
) -> Result<Value, Failure> {
    let observed = observe_session(ctx, session_id)?
        .ok_or_else(|| session_not_found_failure(session_id))?;
    if through > observed.cursor() {
        return Err(Failure::at(
            codes::INTERNAL,
            "Session history no longer covers the wait cursor",
            "session/wait",
        ));
    }
    let rows = rows_of(&observed.events);
    let mut records = Vec::new();
    let mut index = through;
    while index > after_seq {
        let row = &rows[index as usize];
        index -= 1;
        if row.kind != "assistant/message" || !is_message(row) {
            continue;
        }
        if records.len() as u64 == max_messages {
            return Ok(json!({ "records": records, "truncated": true }));
        }
        records.push(record(row));
    }
    Ok(json!({ "records": records, "truncated": false }))
}

// --- streams ------------------------------------------------------------

/// Serve `session/observeState`, `session/observeEvent` or
/// `session/observeProjections`.
pub(super) async fn observe(
    ctx: &Context,
    config: &MuxConfig,
    endpoint: &str,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    // Only these streams check their fields against the inventory: the older
    // streams keep their own refusals, whose wording was matched to the
    // TypeScript host one by one.
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Outcome::Failed(failure);
    }
    let request = match request_of(endpoint, args) {
        Ok(request) => request,
        Err(failure) => return Outcome::Failed(failure),
    };
    let parsed = if endpoint == "session/observeEvent" {
        Address::parse(endpoint, request.get("address")).map(|a| a.session_id().to_string())
    } else {
        text_field(endpoint, request, "sessionId").map(str::to_string)
    };
    let session_id = match parsed {
        Ok(id) => id,
        Err(failure) => return Outcome::Failed(failure),
    };
    let (woken, subscription) = match watch(ctx, &session_id).await {
        Ok(pair) => pair,
        Err(failure) => return Outcome::Failed(failure),
    };
    let outcome = match endpoint {
        "session/observeState" => {
            observe_state(ctx, &config.events, &session_id, &woken, stream_id, out, cancel).await
        }
        "session/observeEvent" => observe_events(ctx, &session_id, &woken, stream_id, out, cancel).await,
        _ => observe_projections(ctx, &session_id, stream_id, out, cancel).await,
    };
    let _ = subscription.dispose().await;
    outcome
}

/// `session/observeState`: the current state, then each state that differs
/// from the last one sent.
async fn observe_state(
    ctx: &Context,
    hub: &EventHub,
    session_id: &str,
    woken: &Notify,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    let mut last: Option<Value> = None;
    loop {
        let state = match read_state(hub, ctx, session_id) {
            Ok(state) => state,
            Err(failure) => return Outcome::Failed(failure),
        };
        if last.as_ref() != Some(&state) {
            if deliver(out, stream_id, state.clone(), cancel).await.is_err() {
                return Outcome::Cancelled;
            }
            last = Some(state);
        }
        tokio::select! {
            _ = cancel.cancelled() => return Outcome::Cancelled,
            _ = woken.notified() => {}
            _ = tokio::time::sleep(RESAMPLE) => {}
        }
    }
}

/// `session/observeEvent`: `ready`, then every event appended afterwards, in
/// order. No snapshot is read, and readiness does not say the session exists.
async fn observe_events(
    ctx: &Context,
    session_id: &str,
    woken: &Notify,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    // Events already in the log are not this stream's; later ones are.
    let mut sent: Option<i64> = observe_session(ctx, session_id)
        .ok()
        .flatten()
        .map(|observed| observed.cursor());
    if deliver(out, stream_id, json!({ "type": "ready" }), cancel).await.is_err() {
        return Outcome::Cancelled;
    }
    loop {
        match observe_session(ctx, session_id) {
            Ok(Some(observed)) => {
                let rows = rows_of(&observed.events);
                // A log that starts before what was sent is a different log.
                let from = sent.map_or(0, |seq| (seq + 1) as usize);
                for row in rows.iter().skip(from) {
                    if deliver(out, stream_id, record(row), cancel).await.is_err() {
                        return Outcome::Cancelled;
                    }
                    sent = Some(row.seq as i64);
                }
            }
            Ok(None) => {}
            Err(failure) => return Outcome::Failed(failure),
        }
        tokio::select! {
            _ = cancel.cancelled() => return Outcome::Cancelled,
            _ = woken.notified() => {}
        }
    }
}

/// `session/observeProjections`: `ready`, then one frame per projection key
/// that changes, with the seq of the event that changed it.
async fn observe_projections(
    ctx: &Context,
    session_id: &str,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    let (frames, mut changes) = mpsc::unbounded_channel::<Value>();
    let watched = session_id.to_string();
    let subscription = events::on::<SessionEventPayload, _>(ctx, move |ctx, payload| {
        let (frames, watched) = (frames.clone(), watched.clone());
        Box::pin(async move {
            if payload.session != watched {
                return Ok(Value::Null);
            }
            if let Some((key, value)) = super::mux::projection_update(&ctx, &payload) {
                let _ = frames.send(json!({
                    "type": "projection", "key": key, "value": value, "seq": payload.event.seq,
                }));
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
                format!("could not subscribe to session changes: {error}"),
                "session/observeProjections",
            ))
        }
    };
    let outcome = 'stream: {
        if deliver(out, stream_id, json!({ "type": "ready" }), cancel).await.is_err() {
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
