//! Empty dynamic-runner behavior through the authenticated unary gateway.

use cordis::Context;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, Gateway, Router, RouterConfig};
use serde_json::{json, Value};

const TOKEN: &str = "dynamic-runner-test-token";
const PLUGIN: &str = "missing\"plugin";
const MISSING: &str = "no dynamic plugin \"missing\"plugin\" in this process \u{2014} it may have been removed or lost on DSH restart";

struct Rig {
    ctx: Context,
    gateway: Gateway,
}

async fn rig() -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .unwrap();
    let gateway = Gateway::new(
        Router::new(RouterConfig::default()),
        Auth {
            token: TOKEN.into(),
        },
        None,
    );
    Rig { ctx, gateway }
}

async fn call(rig: &Rig, endpoint: &str, args: Value) -> Value {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "runner", "method": endpoint,
        "payload": { "args": args },
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    assert_eq!(
        reply.status,
        200,
        "{}",
        String::from_utf8_lossy(&reply.body)
    );
    let response: Value = serde_json::from_slice(&reply.body).unwrap();
    assert_eq!(response["rpcId"], "runner");
    response["result"].clone()
}

fn cases(agent: &str) -> Vec<(&'static str, Value, Value)> {
    vec![
        (
            "getClientCode",
            json!({ "agentId": agent, "pluginId": PLUGIN, "pluginRunId": "run" }),
            json!({ "ok": false, "error": { "code": "gateway/internal", "message": MISSING, "details": {} } }),
        ),
        (
            "invoke",
            json!({ "pluginId": PLUGIN, "pluginRunId": "run", "method": "hello", "args": null }),
            json!({ "ok": true, "value": { "ok": false, "code": "plugin-not-running", "message": "dynamic plugin \"missing\"plugin\" is not running" } }),
        ),
        (
            "reportClientGuardFailure",
            json!({ "agentId": agent, "pluginId": PLUGIN, "pluginRunId": "run", "failure": { "message": "guard", "stack": "trace" } }),
            json!({ "ok": true, "value": null }),
        ),
        (
            "reportRenderFailure",
            json!({ "agentId": agent, "pluginId": PLUGIN, "pluginRunId": "run", "failure": { "slot": "panel", "message": "render", "abdicated": true } }),
            json!({ "ok": true, "value": null }),
        ),
        (
            "resolveInspectQuery",
            json!({ "agentId": agent, "requestId": "query", "resolution": { "ok": true, "data": null } }),
            json!({ "ok": true, "value": { "accepted": false } }),
        ),
        (
            "resolveRequestRun",
            json!({ "requestId": "request", "resolution": { "ok": true, "pluginRunId": "run" } }),
            json!({ "ok": true, "value": { "accepted": false } }),
        ),
        (
            "runHostHalf",
            json!({ "agentId": agent, "pluginId": PLUGIN, "packageId": "package", "mode": "update", "requestId": "request", "approveFutureVersions": true }),
            json!({ "ok": true, "value": { "ok": false, "message": MISSING } }),
        ),
        (
            "settleUserRun",
            json!({ "agentId": agent, "pluginId": PLUGIN, "resolution": { "ok": false, "reason": "rejected" } }),
            json!({ "ok": true, "value": { "ok": false, "reason": "plugin-missing", "message": MISSING } }),
        ),
        (
            "stopFromPanel",
            json!({ "agentId": agent, "pluginId": PLUGIN }),
            json!({ "ok": true, "value": { "ok": false, "reason": "plugin-missing", "message": MISSING } }),
        ),
        (
            "undefineFromPanel",
            json!({ "agentId": agent, "pluginId": PLUGIN }),
            json!({ "ok": true, "value": { "ok": false, "reason": "plugin-missing", "message": MISSING } }),
        ),
    ]
}

#[tokio::test]
async fn every_empty_runner_result_matches_typescript_without_extra_fields() {
    let rig = rig().await;
    let made = call(&rig, "session/create", json!({ "request": {} })).await;
    let agent = made["value"]["sessionId"].as_str().unwrap();
    assert_eq!(
        call(&rig, "dynamicCordisRunner/inventory", json!({})).await,
        json!({ "ok": true, "value": [] })
    );
    for (name, args, expected) in cases(agent) {
        let endpoint = format!("dynamicCordisRunner/{name}");
        assert_eq!(call(&rig, &endpoint, args).await, expected, "{endpoint}");
    }
}

#[tokio::test]
async fn unknown_agent_lookup_precedes_every_runner_refusal() {
    let rig = rig().await;
    // Quoted identities pin the lookup's direct interpolation, not debug escaping.
    let agent = "unknown\"agent";
    for (name, args, _) in cases(agent) {
        if args.get("agentId").is_none() {
            continue;
        }
        let endpoint = format!("dynamicCordisRunner/{name}");
        assert_eq!(
            call(&rig, &endpoint, args).await,
            json!({
                "ok": false, "error": {
                    "code": "session/not-found", "message": "session \"unknown\"agent\" not found",
                    "details": { "sessionId": agent },
                },
            }),
            "{endpoint}"
        );
    }
}

#[tokio::test]
async fn gateway_refuses_missing_unexpected_and_wrongly_typed_arguments() {
    let rig = rig().await;
    for (name, args, _) in cases("unknown") {
        let endpoint = format!("dynamicCordisRunner/{name}");
        let mut missing = args.clone();
        let field = if args.get("pluginId").is_some() {
            "pluginId"
        } else {
            "requestId"
        };
        missing.as_object_mut().unwrap().remove(field);
        missing["surprise"] = json!(true);
        let result = call(&rig, &endpoint, missing).await;
        assert_eq!(
            result["error"]["code"], "gateway/arguments-invalid",
            "{endpoint}: {result}"
        );
        assert_eq!(result["error"]["message"], format!(
            "typert gateway: {endpoint}: args fields do not match the descriptor: missing \"{field}\"; unexpected \"surprise\""
        ));
        let mut wrong = args.clone();
        wrong[field] = json!(42);
        let result = call(&rig, &endpoint, wrong).await;
        assert_eq!(
            result["error"]["code"], "gateway/input-invalid",
            "{endpoint}: {result}"
        );
        assert_eq!(result["error"]["details"]["field"], field);
        if args.get("agentId").is_some() {
            let mut wrong = args;
            wrong["agentId"] = json!(42);
            let result = call(&rig, &endpoint, wrong).await;
            assert_eq!(
                result["error"]["code"], "gateway/input-invalid",
                "{endpoint}: {result}"
            );
            assert_eq!(result["error"]["details"]["field"], "agentId");
        }
    }
}
