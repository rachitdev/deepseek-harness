use cordis::Context;
use dsh_rs::gui::liveshare::LiveshareService;
use dsh_rs::gui::{Auth, Gateway, Router, RouterConfig};
use serde_json::{json, Value};
use std::sync::Arc;

struct Rig {
    ctx: Context,
    gateway: Gateway,
}

impl Rig {
    fn new() -> Self {
        Self {
            ctx: Context::new(),
            gateway: Gateway::new(
                Router::new(RouterConfig::default())
                    .with_liveshare(Arc::new(LiveshareService::new())),
                Auth {
                    token: "liveshare-test-token-0123456789".into(),
                },
                None,
            ),
        }
    }

    async fn call(&self, operation: &str, args: Value) -> Value {
        let method = format!("liveshareShares/{operation}");
        let mut headers = http::HeaderMap::new();
        headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
        headers.insert(
            http::header::AUTHORIZATION,
            "Bearer liveshare-test-token-0123456789".parse().unwrap(),
        );
        let body = serde_json::to_vec(&json!({"type": "client-request", "rpcId": "live", "method": method, "payload": {"args": args}})).unwrap();
        let (reply, _) = self
            .gateway
            .route(&self.ctx, &http::Method::POST, "/api", &headers, &body)
            .await;
        assert_eq!(reply.status, 200);
        let result = serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone();
        result
    }
}

fn success(value: Value) -> Value {
    json!({"ok": true, "value": value})
}
fn failure(code: &str, message: &str, details: Value) -> Value {
    json!({"ok": false, "error": {"code": format!("liveshareShares/{code}"), "message": message, "details": details}})
}

#[tokio::test]
async fn empty_host_shapes_and_failed_create_are_stable() {
    let rig = Rig::new();
    assert_eq!(
        rig.call("list", json!({})).await,
        success(json!({"shares": []}))
    );
    assert_eq!(
        rig.call("capabilities", json!({})).await,
        success(json!({"capabilities": []}))
    );
    assert_eq!(rig.call("relay", json!({})).await, success(json!({})));
    assert_eq!(
        rig.call("identity", json!({"request": {}})).await,
        failure(
            "relay-url-required",
            "no Liveshare server address is saved; save one in the Share dialog first",
            json!({})
        )
    );
    assert_eq!(
        rig.call("testRelay", json!({})).await,
        success(
            json!({"ok": false, "httpReachable": false, "blocking": true, "error": "no Liveshare server address is saved"})
        )
    );
    assert_eq!(
        rig.call("testPublish", json!({"request": {"sessionId": "unknown"}}))
            .await,
        success(
            json!({"ok": false, "blocking": true, "error": "no Liveshare server address is saved"})
        )
    );
    assert_eq!(
        rig.call("create", json!({"request": {"sessionId": "unknown"}}))
            .await,
        failure(
            "relay-url-required",
            "no Liveshare server address is saved; save one in the Share dialog first",
            json!({})
        )
    );
    assert_eq!(
        rig.call("list", json!({})).await,
        success(json!({"shares": []}))
    );
}

#[tokio::test]
async fn unknown_share_refusals_preserve_codes_messages_and_details() {
    let rig = Rig::new();
    for operation in ["audit", "stop", "setCapabilities", "updatePermissions"] {
        for id in ["missing", "share / with spaces"] {
            let mut request = json!({"shareId": id});
            if operation == "updatePermissions" {
                request["allowsFork"] = json!(false);
            }
            assert_eq!(
                rig.call(operation, json!({"request": request})).await,
                failure(
                    "share-not-found",
                    &format!("share {id} is not published"),
                    json!({"shareId": id})
                ),
                "{operation}"
            );
        }
    }
}

#[tokio::test]
async fn relay_changes_normalize_and_clear_without_granting_identity() {
    let rig = Rig::new();
    let relay = "https://example.com/base";
    let state = success(json!({"override": relay, "effective": relay}));
    assert_eq!(
        rig.call(
            "setRelay",
            json!({"request": {"relayUrl": "  https://EXAMPLE.com:443/base///?x=1#frag  "}})
        )
        .await,
        state
    );
    assert_eq!(rig.call("relay", json!({})).await, state);
    assert_eq!(
        rig.call("identity", json!({"request": {}})).await,
        success(json!({"state": "unverified", "relay": relay}))
    );
    assert_eq!(
        rig.call(
            "identity",
            json!({"request": {"relay": "http://elsewhere.test/"}})
        )
        .await,
        success(json!({"state": "unverified", "relay": "http://elsewhere.test"}))
    );
    assert_eq!(
        rig.call(
            "create",
            json!({"request": {"sessionId": "s", "ttlMinutes": -1}})
        )
        .await,
        failure(
            "owner-login-required",
            "no verified owner credential is stored for https://example.com/base",
            json!({"identity": {"state": "unverified", "relay": relay}})
        )
    );
    assert_eq!(
        rig.call("testPublish", json!({"request": {"sessionId": "s"}}))
            .await,
        success(
            json!({"ok": false, "blocking": true, "error": "https://example.com/base: no verified owner credential is stored for this address"})
        )
    );
    for url in [
        "relative",
        "ftp://example.com",
        "https://user:pass@example.com",
    ] {
        assert_eq!(
            rig.call("setRelay", json!({"request": {"relayUrl": url}}))
                .await,
            failure(
                "invalid-relay-url",
                "relayUrl is not a usable base URL",
                json!({})
            )
        );
        assert_eq!(rig.call("relay", json!({})).await, state);
    }
    assert_eq!(
        rig.call("identity", json!({"request": {"relay": "invalid"}}))
            .await,
        failure(
            "invalid-relay",
            "invalid is not a usable Relay address",
            json!({"relay": "invalid"})
        )
    );
    for clear in [Value::Null, json!("  "), json!("")] {
        assert_eq!(
            rig.call("setRelay", json!({"request": {"relayUrl": clear}}))
                .await,
            success(json!({}))
        );
        assert_eq!(rig.call("relay", json!({})).await, success(json!({})));
    }
    assert_eq!(
        rig.call("list", json!({})).await,
        success(json!({"shares": []}))
    );
}

#[tokio::test]
async fn semantic_refusals_follow_ts_validation_order() {
    let rig = Rig::new();
    for operation in ["create", "testPublish"] {
        assert_eq!(
            rig.call(operation, json!({"request": {"sessionId": ""}}))
                .await,
            failure(
                "missing-session-id",
                &format!("{operation} requires a sessionId"),
                json!({})
            )
        );
    }
    for operation in ["updatePermissions", "setCapabilities"] {
        assert_eq!(
            rig.call(operation, json!({"request": {"shareId": ""}}))
                .await,
            failure(
                "missing-share-id",
                &format!("{operation} requires a shareId"),
                json!({})
            )
        );
    }
    assert_eq!(
        rig.call(
            "updatePermissions",
            json!({"request": {"shareId": "missing"}})
        )
        .await,
        failure(
            "missing-permission",
            "updatePermissions requires a permission or lifetime change",
            json!({})
        )
    );
    for ttl in [json!(0), json!(-1), json!(1.5), json!(10081)] {
        assert_eq!(
            rig.call(
                "updatePermissions",
                json!({"request": {"shareId": "missing", "ttlMinutes": ttl}})
            )
            .await,
            failure(
                "invalid-ttl",
                "ttlMinutes is outside the allowed range",
                json!({})
            )
        );
    }
    for ttl in [Value::Null, json!(1), json!(10080)] {
        assert_eq!(
            rig.call(
                "updatePermissions",
                json!({"request": {"shareId": "missing", "ttlMinutes": ttl}})
            )
            .await,
            failure(
                "share-not-found",
                "share missing is not published",
                json!({"shareId": "missing"})
            )
        );
    }
}

#[tokio::test]
async fn gateway_refuses_field_sets_and_wrong_types_before_dispatch() {
    let rig = Rig::new();
    let missing = rig.call("create", json!({"extra": true})).await;
    assert_eq!(missing["error"]["code"], "gateway/arguments-invalid");
    assert_eq!(missing["error"]["message"], "typert gateway: liveshareShares/create: args fields do not match the descriptor: missing \"request\"; unexpected \"extra\"");
    for (operation, request) in [
        ("create", json!({"sessionId": 3})),
        ("setRelay", json!({"relayUrl": false})),
        ("audit", json!({})),
        (
            "setCapabilities",
            json!({"shareId": "s", "capabilities": [{"plugin": "p", "readable": [1], "writable": []}]}),
        ),
    ] {
        let result = rig.call(operation, json!({"request": request})).await;
        assert_eq!(
            result["error"]["code"], "gateway/input-invalid",
            "{operation}"
        );
        assert_eq!(result["error"]["details"]["field"], "request");
    }
}

#[tokio::test]
async fn anonymous_relay_refusals_still_count_as_reachable() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0; 4096];
        socket.read(&mut request).await.unwrap();
        socket
            .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
    });
    let rig = Rig::new();
    rig.call("setRelay", json!({"request": {"relayUrl": address}}))
        .await;
    assert_eq!(
        rig.call("testRelay", json!({})).await,
        success(json!({"ok": true, "httpReachable": true, "blocking": false}))
    );
    server.await.unwrap();
}
