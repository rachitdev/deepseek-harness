//! The parity ratchet: the TypeScript host's endpoint inventory
//! (`docs/gui-endpoints.json`, from `tools/gui-endpoints.mjs`) against what
//! this host serves.
//!
//! `docs/parity-missing.txt` lists the endpoints not yet served. The test
//! fails when the list and reality differ in either direction, so a newly
//! served endpoint must leave the list and a new upstream endpoint must join it.
//! Every served unary endpoint must also refuse an unknown argument at the
//! gateway the way the TypeScript descriptor check does.

use std::collections::BTreeSet;
use std::sync::Arc;

use cordis::Context;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::mux::STREAM_ENDPOINTS;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use serde_json::{json, Value};

const TOKEN: &str = "parity-token-0123456789";

fn inventory() -> Vec<(String, bool)> {
    let doc: Value = serde_json::from_str(include_str!("../docs/gui-endpoints.json")).unwrap();
    doc["endpoints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["endpoint"].as_str().unwrap().to_string(),
                e["transport"] == "stream",
            )
        })
        .collect()
}

fn served() -> BTreeSet<String> {
    Router::endpoints()
        .iter()
        .chain(STREAM_ENDPOINTS.iter())
        .map(|s| s.to_string())
        .collect()
}

fn recorded_missing() -> BTreeSet<String> {
    include_str!("../docs/parity-missing.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

#[test]
fn the_missing_list_is_exactly_the_endpoints_not_served() {
    let served = served();
    let actual: BTreeSet<String> = inventory()
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| !served.contains(name))
        .collect();
    let recorded = recorded_missing();
    let stale: Vec<_> = recorded.difference(&actual).collect();
    let unlisted: Vec<_> = actual.difference(&recorded).collect();
    assert!(
        stale.is_empty() && unlisted.is_empty(),
        "docs/parity-missing.txt is out of date.\n  served now (remove from the list): {stale:?}\n  missing but unlisted (add): {unlisted:?}"
    );
}

#[test]
fn this_host_serves_nothing_the_inventory_lacks() {
    let known: BTreeSet<String> = inventory().into_iter().map(|(n, _)| n).collect();
    let extra: Vec<_> = served()
        .into_iter()
        .filter(|name| !known.contains(name) && name != "$events/result" && name != "$events")
        .collect();
    assert!(extra.is_empty(), "served but not in the TypeScript inventory: {extra:?}");
}

#[test]
fn stream_endpoints_are_declared_as_streams() {
    let streams: BTreeSet<String> = inventory()
        .into_iter()
        .filter(|(_, stream)| *stream)
        .map(|(n, _)| n)
        .collect();
    for name in STREAM_ENDPOINTS.iter().filter(|n| **n != "$events") {
        assert!(streams.contains(*name), "{name} is served as a stream but is unary upstream");
    }
}

async fn rig() -> (Context, Arc<Gateway>) {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} })).await.expect("profile installs");
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel { provider: "mock".into(), model: "m".into() },
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
    });
    (ctx, Arc::new(Gateway::new(router, Auth { token: TOKEN.to_string() }, None)))
}

async fn call(ctx: &Context, gw: &Gateway, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": args },
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    let (reply, _) = gw.route(ctx, &http::Method::POST, "/api", &headers, &body).await;
    let envelope: Value = serde_json::from_slice(&reply.body).expect("JSON reply");
    envelope["result"].clone()
}

#[tokio::test]
async fn every_served_unary_endpoint_refuses_an_unknown_argument() {
    let (ctx, gw) = rig().await;
    let streams: BTreeSet<String> = inventory().into_iter().filter(|(_, s)| *s).map(|(n, _)| n).collect();
    let mut offenders = Vec::new();
    for name in Router::endpoints() {
        if streams.contains(*name) || name.starts_with('$') {
            continue;
        }
        let mut args = dsh_rs::gui::descriptor::sample_args(name).expect("the inventory lists it");
        args["zzUnexpected"] = json!(1);
        let reply = call(&ctx, &gw, name, args).await;
        if reply["error"]["code"] != "gateway/arguments-invalid" {
            offenders.push(format!("{name}: {reply}"));
        }
    }
    assert!(offenders.is_empty(), "endpoints that accept an unknown argument:\n{}", offenders.join("\n"));
}

#[tokio::test]
async fn every_missing_endpoint_answers_lookup_not_found_over_http() {
    let (ctx, gw) = rig().await;
    for name in recorded_missing() {
        if inventory().iter().any(|(n, s)| *n == name && *s) {
            continue;
        }
        let args = dsh_rs::gui::descriptor::sample_args(&name).expect("the inventory lists it");
        let reply = call(&ctx, &gw, &name, args).await;
        assert_eq!(reply["error"]["code"], "gateway/lookup-not-found", "{name}: {reply}");
    }
}
