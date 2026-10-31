//! Analytics and card contracts, with unary requests through the real gateway.

use cordis::Context;
use dsh_rs::api::services::AgentRegistryService;
use dsh_rs::gui::product_analytics;
use dsh_rs::gui::{Auth, Gateway, Router, RouterConfig};
use dsh_rs::plugins::subagent::{Lineage, LINEAGE_SERVICE};
use dsh_rs::types::AgentOptions;
use std::sync::Arc;

use product_analytics::{PolicyEnd, ProductAnalyticsConfig, ProductAnalyticsService};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

async fn answer(service: Arc<ProductAnalyticsService>, method: &str, args: Value) -> Value {
    let gateway = Gateway::new(
        Router::new(RouterConfig::default()).with_product_analytics(service),
        Auth {
            token: "analytics-test-token".into(),
        },
        None,
    );
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        "Bearer analytics-test-token".parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({"type": "client-request", "rpcId": "analytics", "method": method, "payload": {"args": args}})).unwrap();
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
async fn configured_policy_and_disposal() {
    assert!(ProductAnalyticsConfig::default().enabled);
    for enabled in [true, false] {
        let service = Arc::new(ProductAnalyticsService::new(ProductAnalyticsConfig {
            enabled,
        }));
        assert_eq!(
            answer(service.clone(), "productAnalytics/enabled", json!({})).await,
            json!({"ok": true, "value": enabled})
        );
        service.dispose();
        assert_eq!(
            answer(service.clone(), "productAnalytics/enabled", json!({})).await,
            json!({"ok": true, "value": false})
        );
    }
}

#[tokio::test]
async fn policy_is_first_then_cancellation_or_disposal_ends_the_stream() {
    for enabled in [true, false] {
        for dispose in [true, false] {
            let service =
                std::sync::Arc::new(ProductAnalyticsService::new(ProductAnalyticsConfig {
                    enabled,
                }));
            let cancel = CancellationToken::new();
            let (out, mut input) = mpsc::channel(4);
            let watching = service.clone();
            let token = cancel.clone();
            let task = tokio::spawn(async move {
                watching
                    .watch_policy(&token, |value| {
                        let out = out.clone();
                        async move { out.send(value).await.map_err(|_| ()) }
                    })
                    .await
            });
            assert_eq!(
                tokio::time::timeout(Duration::from_secs(1), input.recv())
                    .await
                    .unwrap(),
                Some(enabled)
            );
            assert!(
                tokio::time::timeout(Duration::from_millis(20), input.recv())
                    .await
                    .is_err()
            );
            if dispose {
                service.dispose();
            } else {
                cancel.cancel();
            }
            assert_eq!(
                tokio::time::timeout(Duration::from_secs(1), task)
                    .await
                    .unwrap()
                    .unwrap(),
                if dispose {
                    PolicyEnd::Disposed
                } else {
                    PolicyEnd::Cancelled
                }
            );
            assert_eq!(input.recv().await, None);
        }
    }
}

#[tokio::test]
async fn report_accepts_and_drops_objects_without_returning_a_value() {
    for enabled in [true, false] {
        let service = Arc::new(ProductAnalyticsService::new(ProductAnalyticsConfig {
            enabled,
        }));
        for event in [
            json!({"eventName": "desktop_app_launch", "timestamp": 123, "attributes": {}}),
            json!({}),
            json!({"eventName": "future_event", "attributes": {"extra": true}}),
        ] {
            assert_eq!(
                answer(
                    service.clone(),
                    "productAnalytics/report",
                    json!({"event": event})
                )
                .await,
                json!({"ok": true})
            );
        }
        for event in [
            Value::Null,
            json!(false),
            json!(42),
            json!("event"),
            json!([]),
        ] {
            assert_eq!(
                answer(
                    service.clone(),
                    "productAnalytics/report",
                    json!({"event": event})
                )
                .await,
                json!({
                    "ok": false,
                    "error": {
                        "code": "gateway/input-invalid",
                        "message": "typert gateway: productAnalytics/report: wire field \"event\" failed boundary validation",
                        "details": {"endpoint": "productAnalytics/report", "field": "event"}
                    }
                })
            );
        }
    }
}

#[tokio::test]
async fn cards_resolve_agents_before_checking_root_and_catalogue() {
    let ctx = Context::new();
    // The subagent plugin provides the lineage that says an agent is not a root.
    let _handles = dsh_rs::bundle::install_profile(&ctx, &json!({"plugins": { "subagent": {} }}))
        .await
        .unwrap();
    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let root = agents
        .create(None, AgentOptions::mock("m"), None, None, None)
        .unwrap();
    let child = agents
        .create(None, AgentOptions::mock("m"), None, None, None)
        .unwrap();
    ctx.require::<Lineage>(LINEAGE_SERVICE)
        .unwrap()
        .adopt_for_test(child.id(), root.id());
    let gateway = Gateway::new(
        Router::new(RouterConfig::default())
            .with_cards(Arc::new(dsh_rs::gui::cards::CardsService::new())),
        Auth {
            token: "cards-test-token".into(),
        },
        None,
    );
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        "Bearer cards-test-token".parse().unwrap(),
    );
    for method in ["actionCards/act", "pluginRecommendations/describe"] {
        for (id, reason) in [
            (root.id(), "unknown"),
            (child.id(), "not-live-root"),
            ("ghost", ""),
        ] {
            let mut args = json!({"agentId": id, "callId": "missing"});
            if method == "actionCards/act" {
                args["verb"] = json!("dismiss");
            }
            let body = serde_json::to_vec(&json!({
                "type": "client-request", "rpcId": "cards", "method": method,
                "payload": {"args": args},
            }))
            .unwrap();
            let (reply, _) = gateway
                .route(&ctx, &http::Method::POST, "/api", &headers, &body)
                .await;
            assert_eq!(reply.status, 200);
            let reply: Value = serde_json::from_slice(&reply.body).unwrap();
            let expected = if id == "ghost" {
                json!({"ok": false, "error": {"code": "session/not-found", "message": "session \"ghost\" not found", "details": {"sessionId": "ghost"}}})
            } else {
                json!({"ok": true, "value": {"status": "refused", "reason": reason}})
            };
            assert_eq!(reply["result"], expected, "{method}: {id}");
        }
    }
}
