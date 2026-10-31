//! Manager and market contracts through the real gateway.

use cordis::Context;
use dsh_rs::gui::plugin_manager::{PluginManager, NO_NPM_STORE};
use dsh_rs::gui::plugin_market::{PluginMarket, SettingsIndexes};
use dsh_rs::gui::settings::SettingsStore;
use dsh_rs::gui::{Auth, Gateway, Router, RouterConfig};
use dsh_rs::{bundle, gui::wire};
use serde_json::{json, Value};
use std::sync::Arc;
use wire::{Failure, RpcResult};

fn value(result: RpcResult) -> Value {
    match result {
        RpcResult::Ok(value) => value,
        other => panic!("expected value, got {other:?}"),
    }
}

fn error(result: RpcResult) -> Failure {
    match result {
        RpcResult::Failed(error) => error,
        other => panic!("expected failure, got {other:?}"),
    }
}

#[tokio::test]
async fn reads_use_profile_and_live_roster_not_typescript_package_names() {
    let ctx = Context::new();
    let profile = json!({"bundles": ["base"], "plugins": {"schedule": {"enabled": false}}});
    let handles = bundle::install_profile(&ctx, &profile).await.unwrap();
    let gateway = manager_gateway(PluginManager::from_profile(&profile));
    let call = |method: &str, args: Value| {
        let gateway = &gateway;
        let ctx = &ctx;
        let method = method.to_string();
        async move { rpc(gateway_request(gateway, ctx, &method, args).await) }
    };
    assert_eq!(
        value(call("pluginManager/listBundles", json!({})).await),
        json!([{
            "name": "base", "official": false, "availability": "installation", "enabled": true,
            "installed": false, "optional": false, "removable": false, "updatable": false,
            "readOnlyReason": "unaddressable", "rows": [], "overrides": [],
        }])
    );
    let plugins = value(call("pluginManager/listPlugins", json!({})).await);
    let rows = plugins.as_array().unwrap();
    let roster = bundle::roster::roster_of(&ctx).unwrap();
    assert_eq!(rows.len(), roster.rows().len());
    assert!(!rows.is_empty());
    for (row, source) in rows.iter().zip(roster.rows()) {
        assert_eq!(row["entryId"], source.name);
        assert_eq!(row["moduleName"], source.name);
        assert_eq!(row["enabled"], source.enabled());
        assert_eq!(row["readOnlyReason"], "unaddressable");
        assert!(row.get("patchId").is_none());
        if source.enabled() {
            assert_eq!(row["fiberPhase"], "active");
        } else {
            assert!(row.get("fiberPhase").is_none());
        }
    }
    assert_eq!(
        value(call("pluginManager/registries", json!({})).await),
        json!({
            "registry": null, "fallbackRegistries": ["https://registry.npmmirror.com/"], "resolved": null,
        })
    );
    assert_eq!(
        value(call("pluginManager/listVersions", json!({"name": "core"})).await),
        json!({
            "status": "refused", "problem": "not-updatable",
            "reason": "core is not a third-party package installed in this profile", "attempts": [],
        })
    );
    assert_eq!(
        value(call("pluginManager/listVersionExemptions", json!({})).await),
        json!({"exemptions": {}, "warnings": []})
    );
    let missing = error(
        call(
            "pluginManager/readBundleDetail",
            json!({"name": "core", "language": "en"}),
        )
        .await,
    );
    assert_eq!(missing.code, "plugin-manager/unknown-bundle");
    assert_eq!(missing.message, "No such bundle: core");
    assert_eq!(missing.details, json!({"name": "core"}));
    assert_eq!(
        value(
            call(
                "pluginManager/readBundleDetail",
                json!({"name": "base", "language": "en"})
            )
            .await
        ),
        json!({"status": "none"})
    );
    let language = error(
        call(
            "pluginManager/readBundleDetail",
            json!({"name": "base", "language": "!"}),
        )
        .await,
    );
    assert_eq!(language.code, "plugin-manager/invalid-language");
    assert_eq!(language.message, "Not a language id: !");
    assert_eq!(language.details, json!({"language": "!"}));
    let known = value(
        call(
            "pluginManager/setPluginEnabled",
            json!({"id": rows[0]["entryId"], "enabled": false}),
        )
        .await,
    );
    assert_eq!(
        known["error"],
        json!({"code": "operation-error", "diagnostic": NO_NPM_STORE})
    );
    assert_eq!(known["application"], "failed");
    assert_eq!(known["changed"], false);
    for handle in handles.iter().rev() {
        handle.dispose().await;
    }
}

#[tokio::test]
async fn absent_roster_is_not_an_empty_plugin_list() {
    let ctx = Context::new();
    let gateway = manager_gateway(PluginManager::default());
    let failure = error(rpc(gateway_request(
        &gateway,
        &ctx,
        "pluginManager/listPlugins",
        json!({}),
    )
    .await));
    assert_eq!(failure.code, "gateway/service-unavailable");
    assert_eq!(
        failure.message,
        "the plugin roster service is not wired into this build"
    );
    assert_eq!(
        value(rpc(gateway_request(
            &manager_gateway(PluginManager::from_profile(&json!({"bundles": []}))),
            &ctx,
            "pluginManager/listBundles",
            json!({}),
        )
        .await)),
        json!([])
    );
}

#[tokio::test]
async fn mutations_never_claim_to_change_or_apply_npm_state() {
    let ctx = Context::new();
    let gateway = manager_gateway(PluginManager::default());
    let call = |method: &str, args: Value| {
        let gateway = &gateway;
        let ctx = &ctx;
        let method = method.to_string();
        async move { rpc(gateway_request(gateway, ctx, &method, args).await) }
    };
    let cases = [
        (
            "removeBundle",
            json!({"name": "nope"}),
            "remove",
            "nope",
            "not-removable",
            None,
        ),
        (
            "setBundleEnabled",
            json!({"name": "nope", "enabled": true}),
            "enable",
            "nope",
            "operation-error",
            Some(true),
        ),
        (
            "setPluginEnabled",
            json!({"id": "nope", "enabled": false}),
            "enable",
            "nope",
            "unknown-plugin",
            Some(false),
        ),
        (
            "setVersionExemption",
            json!({"packageVersion": "a", "runtimeVersion": "b", "enabled": true}),
            "enable",
            "a",
            "operation-error",
            Some(true),
        ),
        (
            "installBundle",
            json!({"spec": "x"}),
            "install",
            "x",
            "invalid-spec",
            None,
        ),
    ];
    for (method, args, stage, target, code, enabled) in cases {
        let mut expected = json!({"stage": stage, "target": target, "changed": false, "application": "failed", "error": {"code": code}});
        if code != "unknown-plugin" {
            expected["error"]["diagnostic"] = json!(NO_NPM_STORE);
        }
        if let Some(enabled) = enabled {
            expected["enabled"] = json!(enabled);
        }
        assert_eq!(
            value(call(&format!("pluginManager/{method}"), args).await),
            expected
        );
    }
    assert_eq!(
        value(
            call(
                "pluginManager/updateBundle",
                json!({"name": "x", "version": "1"})
            )
            .await
        ),
        json!({
            "from": "", "to": "1", "step": "check", "result": {
                "stage": "install", "target": "x", "changed": false, "application": "failed",
                "error": {"code": "invalid-spec", "diagnostic": format!("x is not a third-party package installed in this profile; {NO_NPM_STORE}")},
            },
        })
    );
    assert_eq!(
        value(call("pluginManager/inspect", json!({"spec": "x"})).await),
        json!({"status": "refused", "problem": "invalid-spec", "reason": NO_NPM_STORE})
    );
    assert_eq!(
        value(call("pluginManager/waitForInstall", json!({"requestId": "x"})).await),
        Value::Null
    );
    assert_eq!(
        value(call("pluginManager/cancelInstall", json!({"requestId": "x"})).await),
        json!({"status": "not-running"})
    );
}

#[tokio::test]
async fn market_settings_normalize_preserve_order_and_do_not_commit_invalid_edits() {
    let settings = Arc::new(SettingsStore::new(None).unwrap());
    let market = MarketRig::new(settings.clone());
    assert!(matches!(
        market
            .dispatch(
                "pluginMarket/setIndexes",
                &json!({"urls": [" https://EXAMPLE.org", "https://second.org/i"]})
            )
            .await,
        Some(RpcResult::Empty)
    ));
    assert_eq!(
        settings
            .user(dsh_rs::gui::settings::MARKET_NAMESPACE)
            .unwrap()["indexes"],
        json!(["https://example.org/", "https://second.org/i"])
    );
    assert_eq!(
        value(
            market
                .dispatch("pluginMarket/listIndexes", &json!({}))
                .await
                .unwrap()
        ),
        json!([{"url": "https://example.org/"}, {"url": "https://second.org/i"}])
    );
    for (urls, reason, message) in [
        (json!(["bad"]), "not-absolute", "Not an absolute URL: bad"),
        (
            json!(["file:///tmp/a"]),
            "scheme",
            "Index URLs use http or https: file:///tmp/a",
        ),
        (
            json!(["https://e.org/#"]),
            "credentials-or-fragment",
            "Index URLs carry no credentials or fragment: https://e.org/#",
        ),
        (
            json!(["https://e.org", "https://e.org/"]),
            "duplicate",
            "Index source listed twice: https://e.org/",
        ),
    ] {
        let failure = error(
            market
                .dispatch("pluginMarket/setIndexes", &json!({"urls": urls}))
                .await
                .unwrap(),
        );
        assert_eq!(failure.code, "plugin-market/invalid-index-url");
        assert_eq!(failure.message, message);
        assert_eq!(failure.details["reason"], reason);
        assert_eq!(
            settings
                .user(dsh_rs::gui::settings::MARKET_NAMESPACE)
                .unwrap()["indexes"],
            json!(["https://example.org/", "https://second.org/i"])
        );
    }
    for method in ["pluginMarket/search", "pluginMarket/readEntryDetail"] {
        let failure = error(
            market
                .dispatch(
                    method,
                    &json!({"url": "https://unknown.org/", "query": {"sort": {"kind": "index"}, "offset": 0, "limit": 10}, "language": "en"}),
                ).await
                .unwrap(),
        );
        assert_eq!(failure.code, "plugin-market/unknown-index");
        assert_eq!(
            failure.message,
            "Not a configured index source: https://unknown.org/"
        );
        assert_eq!(failure.details, json!({"url": "https://unknown.org/"}));
    }
    for (field, number, message) in [
        ("offset", -1, "offset must be a non-negative integer"),
        ("limit", 101, "limit must be an integer from 1 to 100"),
    ] {
        let mut query = json!({"sort": {"kind": "index"}, "offset": 0, "limit": 10});
        query[field] = json!(number);
        let failure = error(
            market
                .dispatch(
                    "pluginMarket/search",
                    &json!({"url": "unknown", "query": query}),
                )
                .await
                .unwrap(),
        );
        assert_eq!(failure.code, "plugin-market/invalid-query");
        assert_eq!(failure.message, message);
        assert_eq!(failure.details, json!({"field": field, "reason": "range"}));
    }
    settings
        .update(
            dsh_rs::gui::settings::MARKET_NAMESPACE,
            &serde_json::Map::from_iter([(
                "indexes".into(),
                json!(["https://EXAMPLE.org", "bad", "bad"]),
            )]),
            None,
        )
        .unwrap();
    assert_eq!(
        value(
            market
                .dispatch("pluginMarket/listIndexes", &json!({}))
                .await
                .unwrap()
        ),
        json!([
            {"url": "https://EXAMPLE.org", "error": "Index URL is not in canonical form; use https://example.org/"},
            {"url": "bad", "error": "Not an absolute URL: bad"},
        ])
    );
    let failure = error(
        market
            .dispatch(
                "pluginMarket/search",
                &json!({"url": "bad", "query": {"sort": {"kind": "index"}, "offset": 0, "limit": 10}, "language": "en"}),
            )
            .await
            .unwrap(),
    );
    assert_eq!(failure.code, "plugin-market/invalid-index-url");
    assert_eq!(failure.message, "Not an absolute URL: bad");
    assert_eq!(
        failure.details,
        json!({"url": "bad", "reason": "configured"})
    );
}

async fn gateway_call(method: &str, args: Value) -> Value {
    let gateway = Gateway::new(
        Router::new(RouterConfig::default()),
        Auth {
            token: "plugin-manager-test-token".to_string(),
        },
        None,
    );
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        "Bearer plugin-manager-test-token".parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({"type": "client-request", "rpcId": "r", "method": method, "payload": {"args": args}})).unwrap();
    let (reply, _) = gateway
        .route(
            &Context::new(),
            &http::Method::POST,
            "/api",
            &headers,
            &body,
        )
        .await;
    assert_eq!(reply.status, 200);
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}

#[tokio::test]
async fn real_gateway_refuses_missing_unexpected_wrong_types_and_object_specs() {
    let missing = gateway_call("pluginManager/listVersions", json!({"extra": true})).await;
    assert_eq!(missing["ok"], false);
    assert_eq!(missing["error"]["code"], "gateway/arguments-invalid");
    assert_eq!(missing["error"]["message"], "typert gateway: pluginManager/listVersions: args fields do not match the descriptor: missing \"name\"; unexpected \"extra\"");
    for (method, args, field) in [
        (
            "pluginManager/setBundleEnabled",
            json!({"name": "x", "enabled": "true"}),
            "enabled",
        ),
        (
            "pluginManager/installBundle",
            json!({"spec": {"name": "x"}}),
            "spec",
        ),
        (
            "pluginManager/inspect",
            json!({"spec": {"name": "x"}}),
            "spec",
        ),
        ("pluginMarket/setIndexes", json!({"urls": [5]}), "urls"),
    ] {
        let result = gateway_call(method, args).await;
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["code"], "gateway/input-invalid");
        assert_eq!(result["error"]["details"]["field"], field);
        assert_eq!(
            result["error"]["message"],
            format!("typert gateway: {method}: wire field \"{field}\" failed boundary validation")
        );
    }
}

#[test]
fn service_endpoint_tables_cover_the_nineteen_inventory_methods() {
    assert_eq!(
        dsh_rs::gui::plugin_manager::PLUGIN_MANAGER_SERVICE,
        "pluginManager"
    );
    assert_eq!(
        dsh_rs::gui::plugin_market::PLUGIN_MARKET_SERVICE,
        "pluginMarket"
    );
    let inventory: Value =
        serde_json::from_str(include_str!("../docs/gui-endpoints.json")).unwrap();
    let methods: Vec<_> = inventory["endpoints"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["endpoint"].as_str())
        .filter(|name| name.starts_with("pluginManager/") || name.starts_with("pluginMarket/"))
        .collect();
    let mut implemented = dsh_rs::gui::plugin_manager::ENDPOINTS.to_vec();
    implemented.extend(dsh_rs::gui::plugin_market::ENDPOINTS);
    assert_eq!(implemented, methods);
}

struct MarketRig {
    ctx: Context,
    gateway: Gateway,
}
impl MarketRig {
    fn new(settings: Arc<SettingsStore>) -> Self {
        let market = Arc::new(PluginMarket::new(Arc::new(SettingsIndexes::new(
            settings.clone(),
        ))));
        Self {
            ctx: Context::new(),
            gateway: Gateway::new(
                Router::new(RouterConfig::default())
                    .with_settings(settings)
                    .with_plugin_market(market),
                Auth {
                    token: "plugin-manager-test-token".into(),
                },
                None,
            ),
        }
    }
    async fn dispatch(&self, method: &str, args: &Value) -> Option<RpcResult> {
        let mut args = args.clone();
        if method == "pluginMarket/readEntryDetail" {
            args.as_object_mut().unwrap().remove("query");
            args["name"] = json!("example");
            args["version"] = json!("1.0.0");
        } else {
            args.as_object_mut().unwrap().remove("language");
        }
        Some(rpc(
            gateway_request(&self.gateway, &self.ctx, method, args).await
        ))
    }
}
fn rpc(result: Value) -> RpcResult {
    if result["ok"] == true {
        match result.get("value") {
            Some(value) => RpcResult::ok(value.clone()),
            None => RpcResult::Empty,
        }
    } else {
        RpcResult::failed(Failure {
            code: result["error"]["code"].as_str().unwrap().into(),
            message: result["error"]["message"].as_str().unwrap().into(),
            details: result["error"]["details"].clone(),
        })
    }
}
async fn gateway_request(gateway: &Gateway, ctx: &Context, method: &str, args: Value) -> Value {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        "Bearer plugin-manager-test-token".parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({"type": "client-request", "rpcId": "r", "method": method, "payload": {"args": args}})).unwrap();
    let (reply, _) = gateway
        .route(ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    assert_eq!(reply.status, 200);
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}
#[tokio::test]
async fn detail_language_validation_precedes_unknown_index() {
    let rig = MarketRig::new(Arc::new(SettingsStore::new(None).unwrap()));
    for language in ["!", "e", "englishhh", "en-", "en_Us", "en-123456789"] {
        let result = gateway_request(&rig.gateway, &rig.ctx, "pluginMarket/readEntryDetail", json!({
            "url": "https://unknown.org/", "name": "example", "version": "1", "language": language,
        })).await;
        assert_eq!(
            result,
            json!({"ok": false, "error": {
                "code": "plugin-market/invalid-query", "message": format!("Not a language id: {language}"),
                "details": {"field": "language", "reason": "format"},
            }})
        );
    }
    for language in ["en", "zh-CN", "EN-us-123"] {
        let result = gateway_request(&rig.gateway, &rig.ctx, "pluginMarket/readEntryDetail", json!({
            "url": "https://unknown.org/", "name": "example", "version": "1", "language": language,
        })).await;
        assert_eq!(result["error"]["code"], "plugin-market/unknown-index");
    }
}

fn manager_gateway(manager: PluginManager) -> Gateway {
    Gateway::new(
        Router::new(RouterConfig::default()).with_plugin_manager(Arc::new(manager)),
        Auth {
            token: "plugin-manager-test-token".into(),
        },
        None,
    )
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn market_fetches_search_pages_and_detail_through_gateway() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let index = json!({"format": 1, "title": "Local Market", "description": "Fixtures",
        "plugins": [
            {"name": "example", "description": "Useful widgets", "tags": ["tools"],
             "versions": [{"version": "1.0.0", "publishTimestamp": 1, "source": {"type": "npm"}, "engines": {}}],
             "files": {"locale/en.json": "{\"meta\":{\"detail\":\"guide.md\"}}", "guide.md": "# Example\nFull detail"}},
            {"name": "other", "description": "Another package", "tags": ["editor"], "version": "2.0.0", "readme": "# Other"}
        ]}).to_string();
    let server = std::thread::spawn(move || {
        for _ in 0..9 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 8192];
            stream.read(&mut request).unwrap();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", index.len(), index).unwrap();
        }
    });
    let settings = Arc::new(SettingsStore::default());
    settings
        .update(
            dsh_rs::gui::settings::MARKET_NAMESPACE,
            &serde_json::Map::from_iter([("indexes".into(), json!([url]))]),
            None,
        )
        .unwrap();
    let rig = MarketRig::new(settings.clone());
    for text in ["EXAMPLE", "WIDGETS", "TOOLS"] {
        let result = value(
            rig.dispatch(
                "pluginMarket/search",
                &json!({"url": url,
            "query": {"text": text, "sort": {"kind": "index"}, "offset": 0, "limit": 10}}),
            )
            .await
            .unwrap(),
        );
        assert_eq!(result["title"], "Local Market");
        assert_eq!(result["total"], 1);
        assert_eq!(result["entries"][0]["name"], "example");
        assert_eq!(result["entries"][0]["defaultVersion"], "1.0.0");
        assert_eq!(
            result["entries"][0]["versions"][0]["install"],
            json!({"spec": "example@1.0.0"})
        );
        assert_eq!(result["nextOffset"], Value::Null);
        assert!(result["fetchedAt"].as_u64().unwrap() > 0);
    }
    for offset in 0..3 {
        let result = value(
            rig.dispatch(
                "pluginMarket/search",
                &json!({"url": url,
            "query": {"sort": {"kind": "index"}, "offset": offset, "limit": 1}}),
            )
            .await
            .unwrap(),
        );
        assert_eq!(result["total"], 2);
        assert_eq!(
            result["entries"].as_array().unwrap().len(),
            if offset < 2 { 1 } else { 0 }
        );
        assert_eq!(
            result["nextOffset"],
            if offset == 0 { json!(1) } else { Value::Null }
        );
    }
    let result = value(
        rig.dispatch(
            "pluginMarket/readEntryDetail",
            &json!({"url": url, "language": "en"}),
        )
        .await
        .unwrap(),
    );
    assert_eq!(result["status"], "found");
    assert_eq!(result["origin"], "locale");
    assert_eq!(result["markdown"], "# Example\nFull detail");
    assert_eq!(result["document"].as_str().unwrap().len(), 32);
    let result = gateway_request(
        &rig.gateway,
        &rig.ctx,
        "pluginMarket/readEntryDetail",
        json!({"url": url, "name": "missing", "version": "1.0.0", "language": "en"}),
    )
    .await;
    assert_eq!(result["error"]["code"], "plugin-market/unknown-plugin");
    let result = gateway_request(
        &rig.gateway,
        &rig.ctx,
        "pluginMarket/readEntryDetail",
        json!({"url": url, "name": "other", "version": "2.0.0", "language": "en"}),
    )
    .await;
    assert_eq!(result["value"]["markdown"], "# Other");
    server.join().unwrap();
    for method in ["pluginMarket/search", "pluginMarket/readEntryDetail"] {
        let args = json!({"url": url, "language": "en", "query": {"sort": {"kind": "index"}, "offset": 0, "limit": 10}});
        let failure = error(rig.dispatch(method, &args).await.unwrap());
        assert_eq!(failure.code, "plugin-market/fetch-failed");
        assert_eq!(failure.details["url"], url);
    }
}
