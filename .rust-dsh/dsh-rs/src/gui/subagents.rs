//! `subagents/prompt` and `subagents/interruptByParent`: a person talking to,
//! or stopping, a sub-agent their session started.
//!
//! A sub-agent here lives only while the parent's `subagent` tool call runs, so
//! the only child a message can reach is a live one. The rules follow the
//! TypeScript `SubagentRuntime`:
//!
//! - `prompt` validates the ids, the client time zone, then needs the parent
//!   live (`subagent/parent-unavailable`), admits image parts, and delivers to a
//!   live child of that parent as a queued turn or a steer. A session that
//!   belongs to another parent (or to none) is `subagent/unauthorized`; a child
//!   that is gone is `subagent/not-resumable`.
//! - `interruptByParent` cancels a live child of that parent and keeps its
//!   unclaimed inbox. An absent or finished child is accepted as a no-op; a live
//!   child of another parent is `subagent/unauthorized`. The reply says the
//!   signal was admitted, not that the child has stopped.
//!
//! **Divergences.** A child cannot be resumed after its tool call ends (the
//! TypeScript host revives a stored continuable child). `clientTimeZone` is
//! validated but not recorded on the message, as for `session/prompt`.

use cordis::Context;
use serde_json::{json, Value};

use super::router::{boundary, Router};
use super::session_read::observe_session;
use super::time_zone;
use super::wire::{codes, Failure, RpcResult};
use crate::api::services::AgentRegistryService;
use crate::plugins::subagent::{Lineage, LINEAGE_SERVICE};
use crate::types::{
    AgentCancelCause, ContentBlock, Message, MessageSource, PromptAdmission, PromptTarget, Role,
};

/// Serve one of the two endpoints, or `None` for another method.
pub(super) async fn dispatch(
    router: &Router,
    ctx: &Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    let result = match method {
        "subagents/prompt" => prompt(router, ctx, payload).await,
        "subagents/interruptByParent" => interrupt(ctx, payload),
        _ => return None,
    };
    Some(match result {
        Ok(value) => RpcResult::ok(value),
        Err(failure) => RpcResult::failed(failure),
    })
}

/// One schema issue in the shape the TypeScript validator reports it.
fn empty_id_issue(field: &str) -> Value {
    json!({
        "origin": "string", "code": "too_small", "minimum": 1, "inclusive": true,
        "path": [field], "message": "Too small: expected string to have >=1 characters",
    })
}

/// Refuse an empty parent or child id, reporting each one in declaration order.
fn require_ids(method: &str, parent: &str, child: &str) -> Result<(), Failure> {
    let issues: Vec<Value> = [("parentSessionId", parent), ("childSessionId", child)]
        .into_iter()
        .filter(|(_, id)| id.is_empty())
        .map(|(field, _)| empty_id_issue(field))
        .collect();
    if issues.is_empty() {
        return Ok(());
    }
    Err(Failure {
        code: codes::BAD_REQUEST.to_string(),
        message: format!("invalid payload for subagent.{method}"),
        details: json!({ "issues": issues }),
    })
}

fn unauthorized(child: &str) -> Failure {
    Failure {
        code: "subagent/unauthorized".to_string(),
        message: "subagent does not belong to this parent".to_string(),
        details: json!({ "childSessionId": child }),
    }
}

/// The live sub-agent registry, when the sub-agent plugin is composed.
fn lineage(ctx: &Context) -> Option<std::sync::Arc<Lineage>> {
    ctx.get::<Lineage>(LINEAGE_SERVICE)
}

async fn prompt(router: &Router, ctx: &Context, payload: &Value) -> Result<Value, Failure> {
    const ENDPOINT: &str = "subagents/prompt";
    let request = payload["request"].as_object().ok_or_else(|| boundary(ENDPOINT, "request"))?;
    let text = |name: &str| request.get(name).and_then(Value::as_str).unwrap_or_default();
    let (parent_id, child_id) = (text("parentSessionId"), text("childSessionId"));
    require_ids("prompt", parent_id, child_id)?;
    if let Some(zone) = request.get("clientTimeZone").and_then(Value::as_str) {
        if time_zone::canonical(zone).is_none() {
            return Err(Failure {
                code: "subagent/invalid-time-zone".to_string(),
                message: "clientTimeZone must be UTC or a valid IANA Area/Location name".to_string(),
                details: json!({ "value": zone }),
            });
        }
    }
    let agents = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .ok_or_else(|| Failure::at(codes::SERVICE_UNAVAILABLE, "the agents service is not wired into this build", ENDPOINT))?;
    if agents.get(parent_id).is_none() {
        return Err(Failure {
            code: "subagent/parent-unavailable".to_string(),
            message: format!("parent session \"{parent_id}\" is not live"),
            details: json!({ "parentSessionId": parent_id }),
        });
    }
    let content = admit_content(router, ctx, request, child_id, &agents).await?;
    let live_child = lineage(ctx)
        .and_then(|lineage| lineage.parent_of(child_id))
        .and_then(|started_by| agents.get(child_id).map(|agent| (started_by, agent)));
    let Some((started_by, child)) = live_child else {
        // Not a live child: the session either belongs to someone else, or is gone.
        return match observe_session(ctx, child_id)? {
            Some(other) if other.parent.as_deref() != Some(parent_id) || other.agent.is_none() => {
                if other.parent.as_deref() == Some(parent_id) {
                    Err(not_resumable(child_id))
                } else {
                    Err(unauthorized(child_id))
                }
            }
            _ => Err(not_resumable(child_id)),
        };
    };
    if started_by != parent_id {
        return Err(unauthorized(child_id));
    }
    let message = Message {
        id: format!("gui-{}", crate::plugins::unique_id()),
        role: Role::User,
        content,
        source: MessageSource::User {
            rpc_id: request.get("requestId").and_then(Value::as_str).map(str::to_string),
        },
    };
    let message_id = message.id.clone();
    let target = if request.get("delivery").and_then(Value::as_str) == Some("steer") {
        PromptTarget::Step
    } else {
        PromptTarget::Turn
    };
    match child.admit_prompt(message, target) {
        PromptAdmission::Queued | PromptAdmission::Duplicate => Ok(json!({ "messageId": message_id })),
        PromptAdmission::Refused => Err(delivery_unavailable(child_id)),
    }
}

fn not_resumable(child: &str) -> Failure {
    Failure {
        code: "subagent/not-resumable".to_string(),
        message: "subagent cannot be resumed".to_string(),
        details: json!({ "childSessionId": child }),
    }
}

fn delivery_unavailable(child: &str) -> Failure {
    Failure {
        code: "subagent/delivery-unavailable".to_string(),
        message: "subagent follow-up is temporarily unavailable".to_string(),
        details: json!({ "childSessionId": child }),
    }
}

/// The prompt's content as blocks. Text passes through; image parts are admitted
/// into the attachment store first, so the child's inbox only holds stored references.
async fn admit_content(
    router: &Router,
    ctx: &Context,
    request: &serde_json::Map<String, Value>,
    child_id: &str,
    agents: &AgentRegistryService,
) -> Result<Vec<ContentBlock>, Failure> {
    const ENDPOINT: &str = "subagents/prompt";
    let parts = request["content"].as_array().ok_or_else(|| boundary(ENDPOINT, "request"))?;
    let mut slots: Vec<Result<ContentBlock, usize>> = Vec::new();
    let mut images = Vec::new();
    for part in parts {
        match part["type"].as_str() {
            Some("text") => slots.push(Ok(ContentBlock::text(part["text"].as_str().unwrap_or_default()))),
            _ => {
                slots.push(Err(images.len()));
                images.push(super::router::wire_image_part(part, ENDPOINT)?);
            }
        }
    }
    if images.is_empty() {
        return Ok(slots.into_iter().filter_map(Result::ok).collect());
    }
    let admission = router.images();
    if let Some(child) = agents.get(child_id) {
        let route = super::router::selected_route(&*child);
        admission.require_model(&route.provider, &route.model).map_err(|failure| {
            Failure { code: "subagent/attachment-invalid".to_string(), ..failure }
        })?;
    }
    let policy = std::sync::Arc::clone(&admission);
    let admitted = tokio::task::spawn_blocking(move || policy.admit(&images))
        .await
        .map_err(|error| Failure::at(codes::INTERNAL, format!("image admission failed: {error}"), ENDPOINT))?
        .map_err(|error| {
            if let Some(cause) = &error.cause {
                ctx.logger().log_event(
                    cordis::logger::LogLevel::Warn,
                    "gui".to_string(),
                    None,
                    format!("subagent image admission: {}: {cause}", error.code),
                );
            }
            Failure {
                code: "subagent/attachment-invalid".to_string(),
                message: error.message.clone(),
                details: json!({ "reason": error.code }),
            }
        })?;
    let mut admitted: Vec<Option<_>> = admitted.into_iter().map(Some).collect();
    Ok(slots
        .into_iter()
        .map(|slot| match slot {
            Ok(block) => block,
            Err(index) => ContentBlock::Image { attachment: admitted[index].take().expect("each image slot is used once") },
        })
        .collect())
}

fn interrupt(ctx: &Context, payload: &Value) -> Result<Value, Failure> {
    let text = |name: &str| payload[name].as_str().unwrap_or_default();
    let (child_id, parent_id) = (text("childSessionId"), text("parentSessionId"));
    require_ids("interrupt", parent_id, child_id)?;
    let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
        return Ok(json!({ "accepted": true }));
    };
    let live = lineage(ctx).and_then(|lineage| lineage.parent_of(child_id)).and_then(|by| agents.get(child_id).map(|agent| (by, agent)));
    if let Some((started_by, child)) = live {
        if started_by != parent_id {
            return Err(unauthorized(child_id));
        }
        // Unclaimed inbox work is kept; the child may run on until it sees the signal.
        child.cancel(AgentCancelCause::User, true);
    }
    Ok(json!({ "accepted": true }))
}
