//! Human feedback on a session: `messageFeedback/{list,put,delete}` rate one
//! assistant message, `sessionFeedback/record` files a remark on the session.
//!
//! Both only add log-only events (`feedback/message-put`,
//! `feedback/message-delete`, `feedback/record`): they never reach the model.
//! The ratings a session shows are folded from its log on every call, so a
//! restarted or forked session shows the same ratings, and a fork keeps its
//! parent's ratings as the parent's.
//!
//! A rating carries a version token. A change must name the version it
//! observed (`null` to create), so two browser tabs cannot overwrite each
//! other unseen. Business outcomes (`session-not-found`, `target-not-found`,
//! `version-conflict`, `note-blank`, `note-too-large`) are answered as
//! `{ok:false,error}` inside a successful reply, as the TypeScript host does;
//! malformed requests fail at the gateway.
//!
//! The rating calls serve only loaded sessions. A stored session with no live
//! agent is refused with `gateway/service-unavailable` rather than reported
//! missing. A remark needs a running session, as in the TypeScript host, so a
//! stored session is `session-not-found` there. Every change is flushed before
//! the reply, so an answer means the entry is on disk (the TypeScript remark is
//! logged but not flushed; an idle session here would otherwise lose it).

use std::sync::Arc;

use cordis::Context;
use serde_json::{json, Map, Value};

use super::router::{boundary, check_descriptor, unavailable, Router};
use super::wire::{codes, Failure, RpcResult};
use crate::api::services::{AgentRegistryService, AgentView, SessionService};
use crate::types::{now_ms, FeedbackItem, SessionEvent, SessionEventData};

const LIST: &str = "messageFeedback/list";
const PUT: &str = "messageFeedback/put";
const DELETE: &str = "messageFeedback/delete";
const RECORD: &str = "sessionFeedback/record";

/// Categories a human files feedback under, in the order the host presents them.
const CATEGORIES: [&str; 7] = [
    "task-result",
    "instruction-following",
    "product-interaction",
    "service-stability",
    "resource-cost",
    "security-privacy-permission",
    "other",
];

/// Feedback policy and the lock that makes each read-compare-append atomic.
pub struct Feedback {
    max_note_bytes: usize,
    write: std::sync::Mutex<()>,
}

impl Feedback {
    /// Policy that refuses notes longer than `max_note_bytes` UTF-8 bytes.
    pub fn new(max_note_bytes: usize) -> Self {
        Feedback {
            max_note_bytes,
            write: std::sync::Mutex::new(()),
        }
    }
}

/// Serve one feedback endpoint, or `None` when the method is not one of them.
pub(super) async fn dispatch(
    router: &Router,
    ctx: &Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    if ![LIST, PUT, DELETE, RECORD].contains(&method) {
        return None;
    }
    Some(match serve(router, ctx, method, payload).await {
        Ok(value) => RpcResult::ok(value),
        Err(failure) => RpcResult::failed(failure),
    })
}

async fn serve(
    router: &Router,
    ctx: &Context,
    endpoint: &str,
    payload: &Value,
) -> Result<Value, Failure> {
    check_descriptor(endpoint, &["request"], payload)?;
    let request = payload["request"]
        .as_object()
        .ok_or_else(|| boundary(endpoint, "request"))?;
    let policy = &router.feedback;
    match endpoint {
        LIST => {
            let session_id = session_id(endpoint, request)?;
            match open(ctx, endpoint, &session_id)? {
                None => Ok(session_not_found(&session_id)),
                Some(agent) => {
                    let items = fold(&session_id, &agent.session().events());
                    Ok(success(json!({ "items": items })))
                }
            }
        }
        PUT => {
            let put = PutRequest::parse(endpoint, request)?;
            if let Some(note) = &put.note {
                if note.trim().is_empty() {
                    return Ok(rejected(json!({ "code": "note-blank" })));
                }
                if note.len() > policy.max_note_bytes {
                    return Ok(rejected(json!({
                        "code": "note-too-large",
                        "maxBytes": policy.max_note_bytes,
                        "actualBytes": note.len(),
                    })));
                }
            }
            let Some(agent) = open(ctx, endpoint, &put.session_id)? else {
                return Ok(session_not_found(&put.session_id));
            };
            let (reply, durable) = put_item(policy, &*agent, &put);
            if durable {
                make_durable(ctx, endpoint, &*agent, "feedback").await?;
            }
            Ok(reply)
        }
        DELETE => {
            let session_id = session_id(endpoint, request)?;
            let message_id = string(endpoint, request, "messageId")?;
            let if_version = string(endpoint, request, "ifVersion")?;
            let Some(agent) = open(ctx, endpoint, &session_id)? else {
                return Ok(session_not_found(&session_id));
            };
            let reply = delete_item(policy, &*agent, &session_id, &message_id, &if_version);
            make_durable(ctx, endpoint, &*agent, "feedback").await?;
            Ok(reply)
        }
        _ => {
            let session_id = request
                .get("sessionId")
                .and_then(Value::as_str)
                .ok_or_else(|| boundary(endpoint, "request"))?
                .to_string();
            let text = optional_string(endpoint, request, "text")?;
            let category = optional_category(endpoint, request)?;
            let agents = ctx
                .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                .ok_or_else(|| unavailable_failure(endpoint, "agents"))?;
            let Some(agent) = agents.get(&session_id) else {
                return Ok(session_not_found(&session_id));
            };
            let text = text
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty());
            agent
                .session()
                .append(SessionEventData::FeedbackRecord { text, category });
            make_durable(ctx, endpoint, &*agent, "feedback").await?;
            Ok(success(json!({ "recorded": true })))
        }
    }
}

/// A parsed `messageFeedback/put` request.
struct PutRequest {
    session_id: String,
    message_id: String,
    rating: String,
    note: Option<String>,
    category: Option<String>,
    /// The version the caller observed; `None` asks to create.
    if_version: Option<String>,
}

impl PutRequest {
    fn parse(endpoint: &str, request: &Map<String, Value>) -> Result<Self, Failure> {
        let rating = string(endpoint, request, "rating")?;
        if rating != "positive" && rating != "negative" {
            return Err(boundary(endpoint, "request"));
        }
        let if_version = match request.get("ifVersion") {
            Some(Value::Null) => None,
            Some(Value::String(version)) => Some(version.clone()),
            _ => return Err(boundary(endpoint, "request")),
        };
        Ok(PutRequest {
            session_id: session_id(endpoint, request)?,
            message_id: string(endpoint, request, "messageId")?,
            rating,
            note: optional_string(endpoint, request, "note")?,
            category: optional_category(endpoint, request)?,
            if_version,
        })
    }
}

fn string(endpoint: &str, request: &Map<String, Value>, key: &str) -> Result<String, Failure> {
    match request.get(key) {
        Some(Value::String(text)) => Ok(text.clone()),
        _ => Err(boundary(endpoint, "request")),
    }
}

fn session_id(endpoint: &str, request: &Map<String, Value>) -> Result<String, Failure> {
    string(endpoint, request, "sessionId")
}

/// A string that may be absent; `null` is not absent.
fn optional_string(
    endpoint: &str,
    request: &Map<String, Value>,
    key: &str,
) -> Result<Option<String>, Failure> {
    match request.get(key) {
        None => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(boundary(endpoint, "request")),
    }
}

fn optional_category(
    endpoint: &str,
    request: &Map<String, Value>,
) -> Result<Option<String>, Failure> {
    match optional_string(endpoint, request, "category")? {
        Some(category) if !CATEGORIES.contains(&category.as_str()) => {
            Err(boundary(endpoint, "request"))
        }
        other => Ok(other),
    }
}

fn success(value: Value) -> Value {
    json!({ "ok": true, "value": value })
}

fn rejected(error: Value) -> Value {
    json!({ "ok": false, "error": error })
}

fn session_not_found(session_id: &str) -> Value {
    rejected(json!({ "code": "session-not-found", "sessionId": session_id }))
}

fn version_conflict(current: Option<&FeedbackItem>) -> Value {
    rejected(json!({ "code": "version-conflict", "current": current }))
}

/// The loaded session, `None` when no session has the id.
fn open(
    ctx: &Context,
    endpoint: &str,
    session_id: &str,
) -> Result<Option<Arc<dyn AgentView>>, Failure> {
    if session_id.is_empty() {
        // The TypeScript host builds a storage path from the id and fails on
        // an empty segment; answering the same keeps clients' error paths alike.
        return Err(Failure {
            code: codes::INTERNAL.to_string(),
            message: "cannot encode an empty path segment".to_string(),
            details: json!({}),
        });
    }
    let agents = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .ok_or_else(|| unavailable_failure(endpoint, "agents"))?;
    if let Some(agent) = agents.get(session_id) {
        return Ok(Some(agent));
    }
    let stored = super::router::persistence(ctx)
        .is_some_and(|backend| backend.list().iter().any(|id| id == session_id));
    if stored {
        return Err(Failure::at(
            codes::SERVICE_UNAVAILABLE,
            format!(
                "session {session_id:?} is stored but not loaded; feedback needs a loaded session"
            ),
            endpoint,
        ));
    }
    Ok(None)
}

fn unavailable_failure(endpoint: &str, service: &str) -> Failure {
    match unavailable(endpoint, service) {
        RpcResult::Failed(failure) => failure,
        _ => unreachable!("unavailable always answers a failure"),
    }
}

/// The ratings the session's log leaves, in the order they were first given.
/// Events another session wrote (inherited through a fork) are not this one's.
fn fold(session_id: &str, events: &[SessionEvent]) -> Vec<FeedbackItem> {
    let mut items: Vec<FeedbackItem> = Vec::new();
    for event in events {
        match &event.data {
            SessionEventData::FeedbackMessagePut {
                session_id: owner,
                item,
            } if owner == session_id => {
                match items
                    .iter_mut()
                    .find(|held| held.message_id == item.message_id)
                {
                    Some(held) => *held = item.clone(),
                    None => items.push(item.clone()),
                }
            }
            SessionEventData::FeedbackMessageDelete {
                session_id: owner,
                message_id,
            } if owner == session_id => {
                items.retain(|held| &held.message_id != message_id);
            }
            _ => {}
        }
    }
    items
}

/// Create, replace, or keep one rating. Returns the reply and whether the log
/// must be made durable (it must even for a no-op, as in the TypeScript host).
fn put_item(policy: &Feedback, agent: &dyn AgentView, put: &PutRequest) -> (Value, bool) {
    let session = agent.session();
    let _held = policy
        .write
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let events = session.events();
    let is_assistant_message = events.iter().any(|event| {
        matches!(&event.data, SessionEventData::AssistantMessage { message, .. } if message.id == put.message_id)
    });
    if !is_assistant_message {
        return (
            rejected(json!({
                "code": "target-not-found",
                "sessionId": put.session_id,
                "messageId": put.message_id,
            })),
            false,
        );
    }
    let existing = fold(&put.session_id, &events)
        .into_iter()
        .find(|item| item.message_id == put.message_id);
    if put.if_version.as_deref() != existing.as_ref().map(|item| item.version.as_str()) {
        return (version_conflict(existing.as_ref()), false);
    }
    if let Some(held) = &existing {
        if held.rating == put.rating && held.note == put.note && held.category == put.category {
            return (success(json!(held)), true);
        }
    }
    let now = now_ms();
    let item = FeedbackItem {
        message_id: put.message_id.clone(),
        rating: put.rating.clone(),
        note: put.note.clone(),
        category: put.category.clone(),
        version: uuid::Uuid::new_v4().to_string(),
        created_at: existing.as_ref().map_or(now, |held| held.created_at),
        updated_at: existing
            .as_ref()
            .map_or(now, |held| now.max(held.updated_at)),
    };
    session.append(SessionEventData::FeedbackMessagePut {
        session_id: put.session_id.clone(),
        item: item.clone(),
    });
    (success(json!(item)), true)
}

/// Remove one rating the caller has seen; removing an absent one succeeds.
fn delete_item(
    policy: &Feedback,
    agent: &dyn AgentView,
    session_id: &str,
    message_id: &str,
    if_version: &str,
) -> Value {
    let session = agent.session();
    let _held = policy
        .write
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let existing = fold(session_id, &session.events())
        .into_iter()
        .find(|item| item.message_id == message_id);
    if let Some(held) = existing {
        if held.version != if_version {
            return version_conflict(Some(&held));
        }
        session.append(SessionEventData::FeedbackMessageDelete {
            session_id: session_id.to_string(),
            message_id: message_id.to_string(),
        });
    }
    success(json!({ "absent": true }))
}

/// Write the session log to its backend so the answer means the rating is on
/// disk. A failure is reported, never swallowed: the caller must not believe
/// the rating survives a restart.
pub(super) async fn make_durable(
    ctx: &Context,
    endpoint: &str,
    agent: &dyn AgentView,
    what: &str,
) -> Result<(), Failure> {
    let Some(store) = ctx.get::<SessionService>(crate::session::store::SESSIONS_SERVICE) else {
        let message = format!(
            "{endpoint}: the sessions service is missing, so {what} cannot be made durable"
        );
        log_error(ctx, &message);
        return Err(Failure::at(codes::SERVICE_UNAVAILABLE, message, endpoint));
    };
    match store.flush(agent.id()).await {
        Ok(()) => Ok(()),
        Err(reason) => {
            let message = format!(
                "{endpoint}: cannot make {what} durable for session {}: {reason}",
                agent.id()
            );
            log_error(ctx, &message);
            Err(Failure::at(codes::INTERNAL, message, endpoint))
        }
    }
}

fn log_error(ctx: &Context, message: &str) {
    ctx.logger().log_event(
        cordis::logger::LogLevel::Error,
        "gui".to_string(),
        None,
        message.to_string(),
    );
}
