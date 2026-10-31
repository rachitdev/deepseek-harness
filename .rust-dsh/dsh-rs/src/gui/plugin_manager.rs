//! Plugin management facts from the Rust profile and live roster.
//!
//! No operation claims to persist or apply an npm change: this host has no npm
//! bundle store. Profile bundle names are supplied by the composition owner,
//! never inferred from the plugin roster or from another process's files.

use cordis::{Context, FiberState};
use serde_json::{json, Value};

use super::wire::{Failure, RpcResult};
use crate::bundle::roster::roster_of;

/// The service key owned by the GUI plugin.
pub const PLUGIN_MANAGER_SERVICE: &str = "pluginManager";
/// Diagnostic for operations this host cannot perform.
pub const NO_NPM_STORE: &str = "dsh-rs: this host has no npm bundle store";
/// Manager endpoint names, in inventory order.
pub const ENDPOINTS: &[&str] = &[
    "pluginManager/cancelInstall",
    "pluginManager/inspect",
    "pluginManager/installBundle",
    "pluginManager/listBundles",
    "pluginManager/listPlugins",
    "pluginManager/listVersionExemptions",
    "pluginManager/listVersions",
    "pluginManager/readBundleDetail",
    "pluginManager/registries",
    "pluginManager/removeBundle",
    "pluginManager/setBundleEnabled",
    "pluginManager/setPluginEnabled",
    "pluginManager/setVersionExemption",
    "pluginManager/updateBundle",
    "pluginManager/waitForInstall",
];

/// Immutable composition facts; live plugin phases are read at request time.
#[derive(Default)]
pub struct PluginManager {
    bundles: Vec<String>,
}

impl PluginManager {
    /// Capture the profile's bundle names, preserving first-seen order.
    pub fn new(bundles: impl IntoIterator<Item = String>) -> Self {
        let mut names = Vec::new();
        for name in bundles {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        Self { bundles: names }
    }

    /// Capture a validated profile, including the loader's absent-field default.
    pub fn from_profile(profile: &Value) -> Self {
        match profile.get("bundles").and_then(Value::as_array) {
            Some(names) => Self::new(names.iter().filter_map(Value::as_str).map(str::to_owned)),
            None => Self::new(["base".to_string()]),
        }
    }

    /// Dispatch descriptor-validated arguments; unrelated methods return `None`.
    pub fn dispatch(&self, ctx: &Context, method: &str, args: &Value) -> Option<RpcResult> {
        let text = |key: &str| args[key].as_str().unwrap_or_default();
        let enabled = args["enabled"].as_bool().unwrap_or(false);
        let name = text("name");
        let result = match method {
            "pluginManager/listBundles" => RpcResult::ok(json!(self
                .bundles
                .iter()
                .map(|name| json!({
                    "name": name, "official": false, "availability": "installation",
                    "enabled": true, "installed": false, "optional": false,
                    "removable": false, "updatable": false,
                    "readOnlyReason": "unaddressable", "rows": [], "overrides": [],
                }))
                .collect::<Vec<_>>())),
            "pluginManager/listPlugins" => {
                let Some(roster) = roster_of(ctx) else {
                    return Some(RpcResult::failed(Failure::at(
                        "gateway/service-unavailable",
                        "the plugin roster service is not wired into this build",
                        method,
                    )));
                };
                RpcResult::ok(json!(roster
                    .rows()
                    .iter()
                    .map(|row| {
                        let mut entry = json!({"entryId": row.name, "moduleName": row.name,
                        "enabled": row.enabled(), "readOnlyReason": "unaddressable"});
                        if let Some(phase) = row.state().and_then(phase) {
                            entry["fiberPhase"] = json!(phase);
                        }
                        entry
                    })
                    .collect::<Vec<_>>()))
            }
            "pluginManager/registries" => RpcResult::ok(json!({
                "registry": null, "fallbackRegistries": ["https://registry.npmmirror.com/"],
                "resolved": null,
            })),
            "pluginManager/listVersions" => RpcResult::ok(json!({
                "status": "refused", "problem": "not-updatable",
                "reason": format!("{name} is not a third-party package installed in this profile"),
                "attempts": [],
            })),
            "pluginManager/listVersionExemptions" => {
                RpcResult::ok(json!({"exemptions": {}, "warnings": []}))
            }
            "pluginManager/readBundleDetail" => {
                if !self.bundles.iter().any(|bundle| bundle == name) {
                    failure(
                        "plugin-manager/unknown-bundle",
                        format!("No such bundle: {name}"),
                        json!({"name": name}),
                    )
                } else if !valid_language(text("language")) {
                    failure(
                        "plugin-manager/invalid-language",
                        format!("Not a language id: {}", text("language")),
                        json!({"language": text("language")}),
                    )
                } else {
                    RpcResult::ok(json!({"status": "none"}))
                }
            }
            "pluginManager/waitForInstall" => RpcResult::ok(Value::Null),
            "pluginManager/cancelInstall" => RpcResult::ok(json!({"status": "not-running"})),
            "pluginManager/removeBundle" => {
                operation("remove", name, None, "not-removable", Some(NO_NPM_STORE))
            }
            "pluginManager/setBundleEnabled" => operation(
                "enable",
                name,
                Some(enabled),
                "operation-error",
                Some(NO_NPM_STORE),
            ),
            "pluginManager/setPluginEnabled" => {
                let known = roster_of(ctx)
                    .is_some_and(|roster| roster.rows().iter().any(|row| row.name == text("id")));
                operation(
                    "enable",
                    text("id"),
                    Some(enabled),
                    if known {
                        "operation-error"
                    } else {
                        "unknown-plugin"
                    },
                    if known { Some(NO_NPM_STORE) } else { None },
                )
            }
            "pluginManager/setVersionExemption" => operation(
                "enable",
                text("packageVersion"),
                Some(enabled),
                "operation-error",
                Some(NO_NPM_STORE),
            ),
            "pluginManager/updateBundle" => {
                let diagnostic = format!(
                    "{name} is not a third-party package installed in this profile; {NO_NPM_STORE}"
                );
                let RpcResult::Ok(result) =
                    operation("install", name, None, "invalid-spec", Some(&diagnostic))
                else {
                    return None;
                };
                RpcResult::ok(
                    json!({"from": "", "to": text("version"), "step": "check", "result": result}),
                )
            }
            "pluginManager/installBundle" => operation(
                "install",
                text("spec"),
                None,
                "invalid-spec",
                Some(NO_NPM_STORE),
            ),
            "pluginManager/inspect" => RpcResult::ok(
                json!({"status": "refused", "problem": "invalid-spec", "reason": NO_NPM_STORE}),
            ),
            _ => return None,
        };
        Some(result)
    }
}

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

fn valid_language(language: &str) -> bool {
    let mut parts = language.split('-');
    let first = parts.next().unwrap_or_default();
    (2..=8).contains(&first.len())
        && first.bytes().all(|c| c.is_ascii_alphabetic())
        && parts.all(|part| {
            (1..=8).contains(&part.len()) && part.bytes().all(|c| c.is_ascii_alphanumeric())
        })
}

fn operation(
    stage: &str,
    target: &str,
    enabled: Option<bool>,
    code: &str,
    diagnostic: Option<&str>,
) -> RpcResult {
    let mut value = json!({"stage": stage, "target": target, "changed": false,
        "application": "failed", "error": {"code": code}});
    if let Some(enabled) = enabled {
        value["enabled"] = json!(enabled);
    }
    if let Some(diagnostic) = diagnostic {
        value["error"]["diagnostic"] = json!(diagnostic);
    }
    RpcResult::ok(value)
}

fn failure(code: &str, message: String, details: Value) -> RpcResult {
    RpcResult::failed(Failure {
        code: code.to_string(),
        message,
        details,
    })
}
