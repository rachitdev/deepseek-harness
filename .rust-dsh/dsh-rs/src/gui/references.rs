//! The session reference endpoints: the candidates a `@` mention offers.
//!
//! The TypeScript host answers with every session the caller can see, each
//! carrying the canonical mention a host inserts into the prompt draft. The
//! candidates of the same working directory rank first, and the query filters
//! on the session id, the working directory and the titles.

use cordis::Context;
use serde_json::{json, Value};

use super::base64;
use super::router::{argument, input_invalid, Router};
use super::wire::RpcResult;
use crate::api::services::AgentRegistryService;

/// Serve one session reference endpoint.
///
/// Returns `None` when the method is not one of them, so the router can keep
/// looking.
pub(super) fn dispatch(
    router: &Router,
    ctx: &Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    match method {
        "sessionReferenceResolver/candidates" => Some(candidates(router, ctx, payload)),
        _ => None,
    }
}

/// One session a mention can name.
struct Candidate {
    /// The session id.
    id: String,
    /// The session's latest title, falling back to its id.
    label: String,
    /// The working directory the session ran in.
    cwd: Option<String>,
    /// When the session was created, in epoch milliseconds.
    created_at: u64,
    /// Where the session sits in the listing, which breaks a tie.
    order: usize,
}

/// The candidates one query names.
fn candidates(router: &Router, ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "sessionReferenceResolver/candidates";
    let Some(session_id) = argument(payload, "agentId", ENDPOINT).and_then(Value::as_str) else {
        return RpcResult::failed(input_invalid(ENDPOINT, "agentId", "agentId must be a string"));
    };
    let query = match argument(payload, "query", ENDPOINT) {
        Some(value) => match value.as_str() {
            Some(query) => query.to_lowercase(),
            None => {
                return RpcResult::failed(input_invalid(ENDPOINT, "query", "query must be a string"))
            }
        },
        None => String::new(),
    };
    let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
        return super::router::unavailable(ENDPOINT, "agents");
    };
    let target_cwd = working_directory(ctx, session_id);
    let mut rows: Vec<Candidate> = Vec::new();
    for agent in agents.list() {
        if agent.id() == session_id {
            continue;
        }
        let events = agent.session().events();
        rows.push(Candidate {
            id: agent.id().to_string(),
            label: title_of(&events).unwrap_or_else(|| agent.id().to_string()),
            cwd: agent.session().header_cwd(),
            created_at: agent.session().header_created_at(),
            order: rows.len(),
        });
    }
    for (id, header, _readable) in super::router::stored_sessions(ctx, &agents) {
        let Some(header) = header else { continue };
        if id == session_id || rows.iter().any(|row| row.id == id) {
            continue;
        }
        let title = super::router::stored_title(ctx, &id);
        rows.push(Candidate {
            id: id.clone(),
            label: title.unwrap_or_else(|| id.clone()),
            cwd: header.cwd.clone(),
            created_at: header.created_at,
            order: rows.len(),
        });
    }
    let mut matching: Vec<Candidate> = rows
        .into_iter()
        .filter(|row| {
            query.is_empty()
                || row.id.to_lowercase().contains(&query)
                || row.label.to_lowercase().contains(&query)
                || row
                    .cwd
                    .as_ref()
                    .is_some_and(|cwd| cwd.to_lowercase().contains(&query))
        })
        .collect();
    // The sessions of the caller's own working directory come first.
    matching.sort_by(|left, right| {
        workspace_rank(&left.cwd, &target_cwd)
            .cmp(&workspace_rank(&right.cwd, &target_cwd))
            .then_with(|| left.order.cmp(&right.order))
    });
    let cap = router.config().max_search_results;
    let candidates: Vec<Value> = matching
        .iter()
        .take(cap)
        .map(|row| {
            let mention = mention(&row.id, &row.label);
            let mut candidate = json!({
                "mention": mention,
                "sessionId": row.id,
                "label": row.label,
                "displayTitle": row.label,
                "sameWorkspace": row.cwd.is_some() && row.cwd == target_cwd,
                "createdAt": row.created_at,
            });
            if let Some(cwd) = &row.cwd {
                candidate["cwd"] = json!(cwd);
            }
            candidate
        })
        .collect();
    RpcResult::ok(Value::Array(candidates))
}

/// The working directory a session ran in.
fn working_directory(ctx: &Context, session_id: &str) -> Option<String> {
    if let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) {
        if let Some(agent) = agents.get(session_id) {
            return agent.session().header_cwd();
        }
    }
    super::router::stored_header(ctx, session_id).and_then(|header| header.cwd)
}

/// The rank a working directory takes: the caller's own directory first, then
/// the candidates that recorded none, then the rest.
fn workspace_rank(cwd: &Option<String>, target: &Option<String>) -> u8 {
    match (cwd.as_deref(), target.as_deref()) {
        (Some(cwd), Some(target)) if cwd == target => 0,
        (None, _) => 1,
        _ => 2,
    }
}

/// The newest `session/title` text in a log.
fn title_of(events: &[crate::types::SessionEvent]) -> Option<String> {
    events.iter().rev().find_map(|event| match &event.data {
        crate::types::SessionEventData::SessionTitle { title, .. } => Some(title.clone()),
        _ => None,
    })
}

/// The canonical mention a host inserts into the prompt draft.
fn mention(session_id: &str, label: &str) -> String {
    let payload = serde_json::to_string(session_id).unwrap_or_else(|_| format!("\"{session_id}\""));
    format!("@[{}](dsh-session:{})", escape_label(label), base64::encode_url(payload.as_bytes()))
}

/// The label as a mention shows it: the backslash and the closing bracket a
/// title carries are escaped, as the TypeScript host escapes them.
fn escape_label(label: &str) -> String {
    let mut escaped = String::with_capacity(label.len());
    for character in label.chars() {
        if matches!(character, '\\' | ']') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}