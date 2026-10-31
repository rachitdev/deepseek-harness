//! Computer-use contracts through the real gateway.

use serde_json::{json, Value};

async fn call(method: &str, args: Value) -> Value {
    use dsh_rs::gui::{Auth, Gateway, Router, RouterConfig};
    let gateway = Gateway::new(
        Router::new(RouterConfig::default()),
        Auth {
            token: "cua-test-token".into(),
        },
        None,
    );
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        "Bearer cua-test-token".parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "cua-test", "method": format!("cuaAccess/{method}"),
        "payload": {"args": args},
    }))
    .unwrap();
    let (reply, _) = gateway
        .route(
            &cordis::Context::new(),
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
async fn observations_report_only_capabilities_this_host_has() {
    let ptc = call("ptcConfiguration", json!({})).await;
    assert_eq!(ptc["ok"], true);
    assert_eq!(ptc["value"]["persistent"], false);
    assert_eq!(ptc["value"]["writable"], false);
    assert_eq!(ptc["value"].as_object().unwrap().len(), 2);

    let apps = call("applications", json!({})).await;
    assert_eq!(apps["ok"], true);
    assert_eq!(apps["value"]["allowedApps"], json!([]));
    assert_eq!(apps["value"]["blockedApps"], json!([]));
    assert_eq!(apps["value"]["persistentAvailable"], false);
    assert_eq!(apps["value"].as_object().unwrap().len(), 3);

    let status = call("systemStatus", json!({})).await;
    // Without a native client there are no observed privileges or positive PIDs.
    assert_eq!(
        status,
        json!({"ok": false, "error": {
            "code": "gateway/internal",
            "message": "dsh-rs: system status is unavailable because this host has no native computer-use client",
            "details": {},
        }})
    );
    assert_eq!(
        call("applicationIcons", json!({})).await,
        json!({ "ok": true, "value": {} })
    );
}

#[tokio::test]
async fn refusals_preserve_plain_error_messages_and_gateway_code() {
    assert_eq!(
        call("enablePersistentPtc", json!({})).await,
        json!({
            "ok": false, "error": {
                "code": "gateway/internal",
                "message": "PTC runtime has no editable profile entry", "details": {},
            },
        })
    );
    for method in ["requestPermission", "openSettings"] {
        for permission in ["accessibility", "screenRecording"] {
            assert_eq!(
                call(method, json!({ "permission": permission })).await,
                json!({
                    "ok": false, "error": {
                        "code": "gateway/internal", "message": "Native connection failed", "details": {},
                    },
                })
            );
        }
    }
}

#[tokio::test]
async fn unknown_application_ids_complete_without_a_value_or_changed_observations() {
    let before = call("applications", json!({})).await;
    for method in ["revoke", "retryApproval"] {
        for id in ["a".repeat(64), "0".repeat(64)] {
            assert_eq!(
                call(method, json!({ "id": id })).await,
                json!({ "ok": true })
            );
            assert_eq!(
                call(method, json!({ "id": id })).await,
                json!({ "ok": true })
            );
            assert_eq!(call("applications", json!({})).await, before);
        }
    }
}

#[tokio::test]
async fn descriptor_refuses_missing_unexpected_wrong_type_and_unknown_permissions() {
    for (method, field) in [
        ("requestPermission", "permission"),
        ("openSettings", "permission"),
        ("revoke", "id"),
        ("retryApproval", "id"),
    ] {
        let missing = call(method, json!({})).await;
        assert_eq!(missing["error"]["code"], "gateway/arguments-invalid");
        assert!(missing["error"]["message"]
            .as_str()
            .unwrap()
            .contains(&format!("missing \"{field}\"")));
        let both = call(method, json!({ "extra": true })).await;
        assert_eq!(both["error"]["code"], "gateway/arguments-invalid");
        assert!(both["error"]["message"]
            .as_str()
            .unwrap()
            .contains(&format!("missing \"{field}\"; unexpected \"extra\"")));
        let wrong = call(method, json!({ (field): 42 })).await;
        assert_eq!(wrong["error"]["code"], "gateway/input-invalid");
        assert_eq!(wrong["error"]["details"]["field"], field);
    }
    for method in [
        "ptcConfiguration",
        "enablePersistentPtc",
        "applications",
        "systemStatus",
        "applicationIcons",
    ] {
        assert_eq!(
            call(method, json!({ "extra": true })).await["error"]["code"],
            "gateway/arguments-invalid"
        );
    }
    for permission in ["camera", "", "ScreenRecording"] {
        let result = call("requestPermission", json!({ "permission": permission })).await;
        assert_eq!(result["error"]["code"], "gateway/input-invalid");
        assert_eq!(result["error"]["details"]["field"], "permission");
    }
}
