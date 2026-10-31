//! The read-only endpoints the shipped client calls while it boots.
//!
//! Each answers from what this build actually has, and says so when that is
//! nothing. The TypeScript host backs these with registries dsh-rs has no
//! counterpart for (settings namespaces, a credential store, dynamic plugins,
//! agent presets), so an empty answer here is the truthful one, not a stub that
//! pretends a feature exists. The client renders each empty state.

use cordis::Context;
use serde_json::{json, Value};

use super::router::{input_invalid, unavailable, AgentPreset, DefaultModel};
use super::wire::RpcResult;
use crate::api::services::LlmService;

/// The agent presets the deployment offers. A preset names the composition a
/// session runs; the roster comes from the profile.
pub(super) fn agent_presets(roster: &[AgentPreset]) -> RpcResult {
    RpcResult::ok(json!({
        "presets": roster
            .iter()
            .map(|preset| {
                json!({
                    "id": preset.id,
                    "name": preset.name,
                    "description": preset.description,
                    "isDefault": preset.is_default,
                })
            })
            .collect::<Vec<Value>>()
    }))
}

/// The preset catalog: the `permission` plugin's table, else the single
/// `auto` value, because approval policy then lives in the profile's
/// `approvals` plugin (per-tool ask/allow rules) and is not switchable.
pub(super) fn permission_presets(ctx: &Context) -> RpcResult {
    if let Some(table) = crate::permission::provided(ctx) {
        return RpcResult::ok(table.catalog());
    }
    let option = json!({
        "value": "auto",
        "name": "Profile rules",
        "description": "Tool approvals follow the approvals plugin configured in the profile.",
    });
    RpcResult::ok(json!({
        "options": [option.clone()],
        "defaultOptions": [option],
        "defaultPreset": "auto",
    }))
}

/// The default route plus any configured ones. An adapter does not enumerate
/// its models, so none beyond those is claimed; a route whose provider is not
/// registered is left out, and so is its provider's group.
pub(super) fn model_catalog(ctx: &Context, default: &DefaultModel, extra: &[DefaultModel]) -> RpcResult {
    const ENDPOINT: &str = "session/modelCatalog";
    let Some(llm) = ctx.get::<LlmService>(crate::api::LLM_SERVICE) else {
        return unavailable(ENDPOINT, "llm");
    };
    let registered = llm.list_providers();
    let mut groups: Vec<(String, String, Vec<String>)> = Vec::new();
    for route in std::iter::once(default).chain(extra) {
        let Some(info) = registered.iter().find(|p| p.id == route.provider) else {
            continue;
        };
        match groups.iter_mut().find(|(id, _, _)| *id == route.provider) {
            Some((_, _, models)) if models.contains(&route.model) => {}
            Some((_, _, models)) => models.push(route.model.clone()),
            None => groups.push((route.provider.clone(), info.name.clone(), vec![route.model.clone()])),
        }
    }
    let routable: Vec<&str> = groups.iter().map(|(id, _, _)| id.as_str()).collect();
    let rendered: Vec<Value> = groups
        .iter()
        .map(|(id, name, models)| {
            json!({
                "id": id,
                "name": name,
                "models": models.iter().map(|m| json!({ "id": m, "name": m })).collect::<Vec<_>>(),
            })
        })
        .collect();
    RpcResult::ok(json!({
        "default": { "provider": default.provider, "model": default.model },
        "routableProviders": routable,
        "groups": rendered,
        "failures": [],
    }))
}

/// No dynamic plugins exist in this build.
pub(super) fn plugin_inventory() -> RpcResult {
    RpcResult::ok(json!([]))
}

/// Accepts the client's inspect providers. They are not retained: nothing in
/// this build inspects plugins.
pub(super) fn sync_inspect_manifest(payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "dynamicCordisRunner/syncInspectManifest";
    match payload.get("providers") {
        Some(Value::Array(_)) => RpcResult::ok(Value::Null),
        _ => RpcResult::failed(input_invalid(ENDPOINT, "providers", "providers must be an array")),
    }
}

/// `GET /open-in-app/apps`: no application launcher exists, so none is installed.
pub(super) fn open_in_app_apps() -> Vec<u8> {
    serde_json::to_vec(&json!({ "apps": [] })).unwrap_or_default()
}
