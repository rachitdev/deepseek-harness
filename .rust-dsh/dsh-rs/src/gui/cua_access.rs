//! `cuaAccess/*` observations and refusals for a host without computer use.
//!
//! Empty application decisions never require a native connection. Mutating
//! native controls fail rather than implying that a permission was requested.

use serde_json::{json, Value};

use super::wire::{codes, Failure, RpcResult};

/// Serve the computer-use access endpoints with this host's absent capabilities.
///
/// Argument validation belongs to the gateway's inventory descriptor.
pub(super) fn dispatch(endpoint: &str, _args: &Value) -> Option<RpcResult> {
    Some(match endpoint {
        "cuaAccess/ptcConfiguration" => RpcResult::ok(json!({
            "persistent": false,
            "writable": false,
        })),
        "cuaAccess/enablePersistentPtc" => internal("PTC runtime has no editable profile entry"),
        "cuaAccess/applications" => RpcResult::ok(json!({
            "allowedApps": [],
            "blockedApps": [],
            "persistentAvailable": false,
        })),
        "cuaAccess/systemStatus" => internal(
            "dsh-rs: system status is unavailable because this host has no native computer-use client",
        ),
        "cuaAccess/applicationIcons" => RpcResult::ok(json!({})),
        "cuaAccess/requestPermission" | "cuaAccess/openSettings" => {
            internal("Native connection failed")
        }
        // Unknown application identities complete without granting any access.
        "cuaAccess/revoke" | "cuaAccess/retryApproval" => RpcResult::Empty,
        _ => return None,
    })
}

/// Plain native and profile errors retain their message, not their native code.
fn internal(message: &str) -> RpcResult {
    RpcResult::failed(Failure {
        code: codes::INTERNAL.to_string(),
        message: message.to_string(),
        details: json!({}),
    })
}
