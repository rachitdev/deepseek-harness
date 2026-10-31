//! Dynamic Cordis runner refusals for a host with no JavaScript engine.
//!
//! No Plugin, run, or pending request exists in this process, consistently with
//! `dynamicCordisRunner/inventory`. Lookup refusals precede runner results;
//! activation refusals use `reason`, while invocation refusals use `code`.

use cordis::Context;
use serde_json::{json, Value};

use super::router::{boundary, require_session};
use super::wire::{codes, Failure, RpcResult};

/// Serve the ten runner calls not handled by the inventory/catalog module.
pub(super) fn dispatch(ctx: &Context, endpoint: &str, args: &Value) -> Option<RpcResult> {
    let needs_agent = match endpoint {
        "dynamicCordisRunner/getClientCode"
        | "dynamicCordisRunner/reportClientGuardFailure"
        | "dynamicCordisRunner/reportRenderFailure"
        | "dynamicCordisRunner/resolveInspectQuery"
        | "dynamicCordisRunner/runHostHalf"
        | "dynamicCordisRunner/settleUserRun"
        | "dynamicCordisRunner/stopFromPanel"
        | "dynamicCordisRunner/undefineFromPanel" => true,
        "dynamicCordisRunner/invoke" | "dynamicCordisRunner/resolveRequestRun" => false,
        _ => return None,
    };
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Some(RpcResult::failed(failure));
    }
    if needs_agent {
        let Some(agent_id) = args["agentId"].as_str() else {
            return Some(RpcResult::failed(boundary(endpoint, "agentId")));
        };
        if let Err(mut failure) = require_session(ctx, endpoint, agent_id) {
            if failure.code == "session/not-found" {
                // The session lookup interpolates the identity without escaping it.
                failure.message = format!("session \"{agent_id}\" not found");
            }
            return Some(RpcResult::failed(failure));
        }
    }
    // The descriptor guarantees strings; request-only calls need no plugin id.
    let plugin_id = args["pluginId"].as_str().unwrap_or_default();
    Some(match endpoint {
        "dynamicCordisRunner/getClientCode" => RpcResult::failed(Failure {
            code: codes::INTERNAL.to_string(),
            message: missing_plugin_message(plugin_id),
            details: json!({}),
        }),
        "dynamicCordisRunner/invoke" => RpcResult::ok(json!({
            "ok": false,
            "code": "plugin-not-running",
            "message": format!("dynamic plugin \"{plugin_id}\" is not running"),
        })),
        "dynamicCordisRunner/reportClientGuardFailure"
        | "dynamicCordisRunner/reportRenderFailure" => RpcResult::ok(Value::Null),
        "dynamicCordisRunner/resolveInspectQuery" | "dynamicCordisRunner/resolveRequestRun" => {
            RpcResult::ok(json!({ "accepted": false }))
        }
        "dynamicCordisRunner/runHostHalf" => RpcResult::ok(json!({
            "ok": false, "message": missing_plugin_message(plugin_id),
        })),
        _ => RpcResult::ok(json!({
            "ok": false, "reason": "plugin-missing", "message": missing_plugin_message(plugin_id),
        })),
    })
}

fn missing_plugin_message(plugin_id: &str) -> String {
    format!("no dynamic plugin \"{plugin_id}\" in this process \u{2014} it may have been removed or lost on DSH restart")
}
