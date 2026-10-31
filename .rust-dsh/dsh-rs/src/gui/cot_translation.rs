//! Reader translation preferences and sanitized failures when no provider is installed.

use serde_json::{json, Value};

use super::settings::SettingsStore;
use super::wire::{Failure, RpcResult};

/// The translator's default maximum request length, in UTF-16 code units.
pub const MAX_TEXT_CHARS: usize = 4_000;

/// Read the accepted preferences from the settings namespace view.
///
/// This build's settings store does not yet expose the translation namespace;
/// absent fields retain the TypeScript controller's defaults.
fn preferences(settings: &SettingsStore) -> Value {
    let description = settings.describe();
    let value = description["namespaces"]
        .as_array()
        .and_then(|namespaces| namespaces.iter().find(|ns| ns["ns"] == "client-ui-cot-translation"))
        .map(|namespace| &namespace["value"]);
    json!({
        "provider": value.and_then(|value| value["provider"].as_str()).unwrap_or("bing"),
        "targetLanguage": value.and_then(|value| value["targetLanguage"].as_str()).unwrap_or("auto"),
    })
}

/// Serve the two `cotTranslation/*` calls without dispatching to a provider.
pub(super) fn dispatch(settings: &SettingsStore, endpoint: &str, args: &Value) -> Option<RpcResult> {
    if !matches!(endpoint, "cotTranslation/limits" | "cotTranslation/translate") {
        return None;
    }
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Some(RpcResult::failed(failure));
    }
    Some(match endpoint {
        "cotTranslation/limits" => RpcResult::ok(json!({
            "maxTextChars": MAX_TEXT_CHARS,
            "preferences": preferences(settings),
            "availableProviders": [],
        })),
        // The controller sanitizes every translator failure, including text-limit
        // and unavailable-provider errors, to this exact public failure.
        _ => RpcResult::failed(Failure {
            code: "cotTranslation/failed".to_string(),
            message: "Translation failed".to_string(),
            details: json!({}),
        }),
    })
}
