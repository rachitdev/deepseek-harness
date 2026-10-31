//! The plugin inventory and the default-model initialisation.
//!
//! `pluginInventory/list` shows the plugins this process runs, read from the
//! bundle's roster on every call so a state the kernel has left is never shown.
//! The roster provider itself is machinery, not a plugin, so it is not listed.
//! Without a roster the call fails: an empty list would hide running plugins.
//! A plugin the profile switched off appears as disabled with no fiber phase;
//! a plugin whose fiber was unloaded has no phase either. The reply carries no
//! `agentPresets` group: presets here differ by instructions, not by plugins, so
//! they have no composition rows to show, and no `managementAvailable` flag
//! because no plugin manager is served.
//!
//! `session/initializeDefaultModel` has nothing to do: the default model is
//! named by the profile (`defaultModel`) and is always configured. The TypeScript
//! host uses the call to adopt a model from a signed-in account when no API
//! key exists, and this host has no account layer.

use cordis::{Context, FiberState};
use serde_json::{json, Value};

use super::router::{check_descriptor, unavailable, Router};
use super::wire::RpcResult;
use crate::bundle::roster::{roster_of, RosterRow};

const ENDPOINT: &str = "pluginInventory/list";

/// Serve one inventory endpoint, or `None` when the method is not one of them.
pub(super) fn dispatch(
    _router: &Router,
    ctx: &Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    match method {
        ENDPOINT => Some(match check_descriptor(method, &[], payload) {
            Ok(()) => inventory(ctx),
            Err(failure) => RpcResult::failed(failure),
        }),
        "session/initializeDefaultModel" => Some(match check_descriptor(method, &[], payload) {
            Ok(()) => RpcResult::Empty,
            Err(failure) => RpcResult::failed(failure),
        }),
        _ => None,
    }
}

/// The public name of a fiber state; an unloaded fiber has none.
fn phase(state: FiberState) -> Option<&'static str> {
    match state {
        FiberState::Pending => Some("pending"),
        FiberState::Loading => Some("loading"),
        FiberState::Active => Some("active"),
        FiberState::Failed => Some("failed"),
        FiberState::Unloading => Some("unloading"),
        FiberState::Disposed => None,
    }
}

fn entry(row: &RosterRow) -> Value {
    json!({
        "entryId": row.name,
        "moduleName": row.name,
        "enabled": row.enabled(),
        "fiberPhase": row.state().and_then(phase),
    })
}

fn inventory(ctx: &Context) -> RpcResult {
    let Some(roster) = roster_of(ctx) else {
        // Every bundle installs the roster, so its absence is a broken
        // composition; an empty list would hide running plugins.
        ctx.logger().log_event(
            cordis::logger::LogLevel::Error,
            "gui".to_string(),
            None,
            format!("{ENDPOINT}: the plugin roster service is missing"),
        );
        return unavailable(ENDPOINT, "plugin roster");
    };
    RpcResult::ok(json!({ "entries": roster.rows().iter().map(entry).collect::<Vec<_>>() }))
}
