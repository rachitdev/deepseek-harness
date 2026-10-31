//! The plugin inventory, the registry probe and the default-model call,
//! through the gateway.

use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::BoxFuture;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::registry_probe::{Pinger, RegistryProbe};
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use serde_json::{json, Value};

const TOKEN: &str = "inventory-token-0123456789";

/// Answers `up` for the registry whose host contains the given text.
struct Up(&'static [&'static str]);

impl Pinger for Up {
    fn ping(&self, registry: &str) -> BoxFuture<Result<(), String>> {
        let ok = self.0.iter().any(|host| registry.contains(host));
        Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(2)).await;
            if ok {
                Ok(())
            } else {
                Err("unreachable".to_string())
            }
        })
    }
}

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
}

async fn rig(profile: Value, up: &'static [&'static str]) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &profile)
        .await
        .expect("profile installs");
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel {
            provider: "mock".to_string(),
            model: "m".to_string(),
        },
        max_page_events: 100,
        max_search_results: 20,
        max_title_bytes: 120,
        max_bytes: 2 * 1024 * 1024,
        max_file_bytes: 32 * 1024 * 1024,
        max_lines: 5_000,
        max_entries: 2_000,
        default_max_goal_rounds: 256,
        export_compression_level: 6,
        models: Vec::new(),
    })
    .with_registry_probe(Arc::new(RegistryProbe::new(Arc::new(Up(up)))));
    let gateway = Arc::new(Gateway::new(
        router,
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    ));
    Rig { ctx, gateway }
}

async fn call(rig: &Rig, method: &str, payload: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request",
        "rpcId": "rpc-1",
        "method": method,
        "payload": { "args": payload },
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    let envelope: Value = serde_json::from_slice(&reply.body).expect("the reply is JSON");
    envelope["result"].clone()
}

#[tokio::test]
async fn the_inventory_lists_the_plugins_the_bundle_started_with_their_state() {
    let rig = rig(json!({ "plugins": {} }), &[]).await;
    let reply = call(&rig, "pluginInventory/list", json!({})).await;
    assert_eq!(reply["ok"], true);
    let entries = reply["value"]["entries"].as_array().expect("entries");
    let names: Vec<_> = entries
        .iter()
        .map(|e| e["moduleName"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "manifest",
            "llm",
            "sessions",
            "tools",
            "system-prompt",
            "agent-loop"
        ]
    );
    for entry in entries {
        assert_eq!(entry["enabled"], true);
        assert_eq!(entry["fiberPhase"], "active", "{entry}");
        assert_eq!(entry["entryId"], entry["moduleName"]);
        assert!(entry.get("meta").is_none());
    }
    assert!(reply["value"].get("agentPresets").is_none());
    assert!(reply["value"].get("managementAvailable").is_none());
}

#[tokio::test]
async fn a_plugin_the_profile_switches_off_is_listed_as_disabled_without_a_phase() {
    let rig = rig(
        json!({ "plugins": { "skills": { "enabled": false }, "approvals": {} } }),
        &[],
    )
    .await;
    let reply = call(&rig, "pluginInventory/list", json!({})).await;
    let entries = reply["value"]["entries"].as_array().unwrap();
    let find = |name: &str| entries.iter().find(|e| e["moduleName"] == name).cloned();
    assert_eq!(
        find("skills"),
        Some(
            json!({ "entryId": "skills", "moduleName": "skills", "enabled": false, "fiberPhase": null })
        )
    );
    let approvals = find("approvals").expect("a started plugin is listed");
    assert_eq!(approvals["enabled"], true);
    assert_eq!(approvals["fiberPhase"], "active");
}

#[tokio::test]
async fn the_inventory_and_default_model_calls_take_no_arguments() {
    let rig = rig(json!({ "plugins": {} }), &[]).await;
    for method in [
        "pluginInventory/list",
        "session/initializeDefaultModel",
        "pluginRegistryProbe/fastest",
    ] {
        let refused = call(&rig, method, json!({ "x": 1 })).await;
        assert_eq!(
            refused["error"]["code"], "gateway/arguments-invalid",
            "{method}"
        );
    }
    assert_eq!(
        call(&rig, "session/initializeDefaultModel", json!({})).await,
        json!({ "ok": true }),
        "the default model comes from the profile, so there is nothing to initialise"
    );
}

#[tokio::test]
async fn the_probe_answers_with_the_registry_that_responded() {
    let both = rig(json!({ "plugins": {} }), &["npmjs", "npmmirror"]).await;
    let reply = call(&both, "pluginRegistryProbe/fastest", json!({})).await;
    assert!(
        [
            "https://registry.npmjs.org/",
            "https://registry.npmmirror.com/"
        ]
        .contains(&reply["value"].as_str().unwrap()),
        "{reply}"
    );

    let mirror_only = rig(json!({ "plugins": {} }), &["npmmirror"]).await;
    assert_eq!(
        call(&mirror_only, "pluginRegistryProbe/fastest", json!({})).await,
        json!({ "ok": true, "value": "https://registry.npmmirror.com/" })
    );

    let offline = rig(json!({ "plugins": {} }), &[]).await;
    assert_eq!(
        call(&offline, "pluginRegistryProbe/fastest", json!({})).await,
        json!({ "ok": true, "value": null })
    );
}

#[tokio::test]
async fn the_settings_document_is_reported_absent_rather_than_opening_another_file() {
    let rig = rig(json!({ "plugins": {} }), &[]).await;
    let described = call(&rig, "settings/describe", json!({})).await;
    assert_eq!(described["value"]["hasDocument"], false);
    let reply = call(&rig, "settings/openSettingsDocument", json!({})).await;
    assert_eq!(reply["ok"], false);
    assert_eq!(reply["error"]["code"], "gateway/service-unavailable");
    let refused = call(&rig, "settings/openSettingsDocument", json!({ "x": 1 })).await;
    assert_eq!(refused["error"]["code"], "gateway/arguments-invalid");
}

#[tokio::test]
async fn installing_a_plugin_again_leaves_one_current_row() {
    let rig = rig(json!({ "plugins": { "approvals": {} } }), &[]).await;
    dsh_rs::bundle::install_native_plugins(&rig.ctx, &json!({ "approvals": { "enabled": false } }))
        .await
        .expect("a switched-off plugin installs as nothing");
    let reply = call(&rig, "pluginInventory/list", json!({})).await;
    let entries = reply["value"]["entries"].as_array().unwrap();
    let rows: Vec<_> = entries
        .iter()
        .filter(|e| e["moduleName"] == "approvals")
        .collect();
    assert_eq!(rows.len(), 1, "{entries:?}");
    assert_eq!(rows[0]["enabled"], false);
    assert_eq!(rows[0]["fiberPhase"], Value::Null);
}

#[tokio::test]
async fn a_switched_off_name_this_build_lacks_is_not_listed() {
    let rig = rig(json!({ "plugins": {} }), &[]).await;
    dsh_rs::bundle::install_native_plugins(
        &rig.ctx,
        &json!({ "no-such-plugin": { "enabled": false } }),
    )
    .await
    .expect("a switched-off name is skipped");
    let reply = call(&rig, "pluginInventory/list", json!({})).await;
    let entries = reply["value"]["entries"].as_array().unwrap();
    assert!(entries.iter().all(|e| e["moduleName"] != "no-such-plugin"));
}

#[tokio::test]
async fn native_plugins_refuse_to_start_where_no_roster_can_record_them() {
    let bare = Context::new();
    let refused = dsh_rs::bundle::install_native_plugins(&bare, &json!({ "approvals": {} })).await;
    assert!(refused
        .err()
        .expect("the install is refused")
        .contains("plugin roster is missing"),);
}

#[tokio::test]
async fn an_inventory_without_a_roster_fails_instead_of_listing_nothing() {
    let bare = Rig {
        ctx: Context::new(),
        gateway: rig(json!({ "plugins": {} }), &[]).await.gateway,
    };
    let reply = call(&bare, "pluginInventory/list", json!({})).await;
    assert_eq!(reply["error"]["code"], "gateway/service-unavailable");
}
