//! `schedule/*`: the reminder endpoints.
//!
//! Each call is the TypeScript `ScheduleService` remote method of the same
//! name. Calls name a session by `request.sessionId`; an unknown session has
//! no reminders, so `list` answers `[]` and the others answer their
//! `schedule_not_found` miss instead of `session/not-found`.

use serde_json::{json, Value};

use super::router::{boundary, Router};
use super::wire::{codes, Failure, RpcResult};
use crate::schedule::service::{Failure as ServiceFailure, ScheduleService};

/// Serve one `schedule/*` endpoint, or `None` for another method.
pub(super) async fn dispatch(router: &Router, method: &str, payload: &Value) -> Option<RpcResult> {
    let name = method.strip_prefix("schedule/")?;
    let service = router.schedule();
    let result = match name {
        "list" | "catalog" | "history" | "delete" | "deleteInactive" | "update" => match &service {
            Some(service) => run(service, name, payload).await,
            None => Err(Failure::at(codes::SERVICE_UNAVAILABLE, "the schedule service is not wired into this build", method)),
        },
        _ => return None,
    };
    Some(match result {
        Ok(value) => RpcResult::ok(value),
        Err(failure) => RpcResult::failed(failure),
    })
}

/// A request object whose named strings are present.
fn request<'a>(endpoint: &str, payload: &'a Value, strings: &[&str]) -> Result<&'a serde_json::Map<String, Value>, Failure> {
    let object = payload["request"].as_object().ok_or_else(|| boundary(endpoint, "request"))?;
    if strings.iter().any(|key| !object.get(*key).is_some_and(Value::is_string)) {
        return Err(boundary(endpoint, "request"));
    }
    Ok(object)
}

fn text<'a>(object: &'a serde_json::Map<String, Value>, key: &str) -> &'a str {
    object[key].as_str().unwrap_or_default()
}

fn service_failure(failure: ServiceFailure) -> Failure {
    match failure {
        ServiceFailure::Input(error) => Failure {
            code: codes::INTERNAL.to_string(),
            message: error.message().to_string(),
            details: json!({}),
        },
        ServiceFailure::Service(error) => Failure {
            code: codes::INTERNAL.to_string(),
            message: error.0,
            details: json!({}),
        },
    }
}

async fn run(service: &ScheduleService, name: &str, payload: &Value) -> Result<Value, Failure> {
    let endpoint = format!("schedule/{name}");
    match name {
        "list" => {
            let object = request(&endpoint, payload, &["sessionId"])?;
            Ok(Value::Array(service.list(text(object, "sessionId")).await))
        }
        "catalog" => Ok(Value::Array(service.catalog().await)),
        "history" => {
            let object = request(&endpoint, payload, &["id", "sessionId"])?;
            let limit = object.get("limit").filter(|v| v.is_number()).ok_or_else(|| boundary(&endpoint, "request"))?;
            let before = match object.get("before") {
                None => None,
                Some(Value::String(before)) => Some(before.as_str()),
                Some(_) => return Err(boundary(&endpoint, "request")),
            };
            service.history(text(object, "sessionId"), text(object, "id"), limit, before).await.map_err(service_failure)
        }
        "delete" => {
            let object = request(&endpoint, payload, &["id", "sessionId"])?;
            service.delete(text(object, "sessionId"), text(object, "id")).await.map_err(service_failure)
        }
        "deleteInactive" => service.delete_inactive().await.map_err(service_failure),
        "update" => {
            let object = request(&endpoint, payload, &["id", "sessionId"])?;
            let expected = object.get("expected").filter(|v| v.is_object()).ok_or_else(|| boundary(&endpoint, "request"))?;
            let optional_text = |key: &str| match object.get(key) {
                None => Ok(None),
                Some(Value::String(value)) => Ok(Some(value.as_str())),
                Some(_) => Err(boundary(&endpoint, "request")),
            };
            let (title, prompt) = (optional_text("title")?, optional_text("prompt")?);
            let change = object.get("change");
            service
                .update(text(object, "sessionId"), text(object, "id"), expected, change, title, prompt)
                .await
                .map_err(service_failure)
        }
        _ => unreachable!("dispatch filters the names"),
    }
}
