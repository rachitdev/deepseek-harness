//! Configuration-owned analytics policy and a drop-only event intake.
//!
//! There is no telemetry backend or volatile loader in this build. Configuration
//! fixes the policy for the service lifetime; disposal makes it ineffective.

use std::future::Future;

use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use super::wire::{codes, Failure, RpcResult};

/// Analytics configuration carried by the GUI plugin.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default)]
pub struct ProductAnalyticsConfig {
    /// Whether this host accepts analytics events (which it drops locally).
    pub enabled: bool,
}

impl Default for ProductAnalyticsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Why a policy subscription stopped.
#[derive(Debug, PartialEq, Eq)]
pub enum PolicyEnd {
    /// The service was disposed.
    Disposed,
    /// The client cancelled or its output closed.
    Cancelled,
}

/// The effective collection policy; accepted events are never persisted or sent.
#[derive(Debug)]
pub struct ProductAnalyticsService {
    config: ProductAnalyticsConfig,
    lifetime: CancellationToken,
}

impl ProductAnalyticsService {
    /// Construct an active service from the GUI configuration.
    pub fn new(config: ProductAnalyticsConfig) -> Self {
        Self {
            config,
            lifetime: CancellationToken::new(),
        }
    }

    /// Whether the configured service is still accepting events.
    pub fn enabled(&self) -> bool {
        !self.lifetime.is_cancelled() && self.config.enabled
    }

    /// Disable intake and end every policy subscription.
    pub fn dispose(&self) {
        self.lifetime.cancel();
    }

    /// Deliver the initial policy and wait until cancellation or disposal.
    ///
    /// There are no live configuration edits in this build. The sender is
    /// supplied by the mux so its normal delivery and backpressure apply.
    pub async fn watch_policy<F, Fut>(&self, cancel: &CancellationToken, mut send: F) -> PolicyEnd
    where
        F: FnMut(bool) -> Fut,
        Fut: Future<Output = Result<(), ()>>,
    {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return PolicyEnd::Cancelled,
            _ = self.lifetime.cancelled() => return PolicyEnd::Disposed,
            sent = send(self.enabled()) => {
                if sent.is_err() { return PolicyEnd::Cancelled; }
            }
        }
        tokio::select! {
            biased;
            _ = cancel.cancelled() => PolicyEnd::Cancelled,
            _ = self.lifetime.cancelled() => PolicyEnd::Disposed,
        }
    }

    /// Serve an analytics unary call after the gateway's descriptor check.
    pub fn dispatch(&self, method: &str, args: &Value) -> Option<RpcResult> {
        Some(match method {
            "productAnalytics/enabled" => RpcResult::ok(json!(self.enabled())),
            "productAnalytics/report" => {
                // ProductEvent has no runtime codec in TS. Only the spec's
                // object boundary is enforced; no event-name allowlist exists.
                if !args.get("event").is_some_and(Value::is_object) {
                    return Some(RpcResult::failed(Failure {
                        code: codes::INPUT_INVALID.to_string(),
                        message: "typert gateway: productAnalytics/report: wire field \"event\" failed boundary validation".to_string(),
                        details: json!({ "endpoint": "productAnalytics/report", "field": "event" }),
                    }));
                }
                RpcResult::Empty
            }
            _ => return None,
        })
    }
}
