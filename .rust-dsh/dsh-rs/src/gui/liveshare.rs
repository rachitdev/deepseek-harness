//! Owner live-sharing endpoints for a host with no shares or owner credential.
//!
//! Saving a Relay address does not install a publisher. Addresses live for the
//! service lifetime; no credential, viewer, share, or capability is fabricated.

use serde_json::{json, Value};
use tokio::sync::Mutex;

use super::wire::{Failure, RpcResult};

const NO_ADDRESS: &str = "no Liveshare server address is saved";

/// Local owner state shared by all gateway calls.
#[derive(Default)]
pub struct LiveshareService {
    relay: Mutex<Option<String>>,
}

impl LiveshareService {
    /// Construct a host with no saved address, identity, or shares.
    pub fn new() -> Self {
        Self::default()
    }

    /// Answer one owner endpoint after the gateway's descriptor validation.
    pub async fn dispatch(&self, method: &str, args: &Value) -> Option<RpcResult> {
        let operation = method.strip_prefix("liveshareShares/")?;
        let request = &args["request"];
        let answer = match operation {
            "list" => RpcResult::ok(json!({"shares": []})),
            "capabilities" => RpcResult::ok(json!({"capabilities": []})),
            "relay" => relay_state(self.relay.lock().await.as_deref()),
            "setRelay" => {
                let next = match request["relayUrl"].as_str() {
                    None | Some("") => None,
                    Some(value) if value.trim().is_empty() => None,
                    Some(value) => match normalize_relay(value.trim()) {
                        Some(relay) => Some(relay),
                        None => {
                            return Some(refusal(
                                "invalid-relay-url",
                                "relayUrl is not a usable base URL",
                                json!({}),
                            ))
                        }
                    },
                };
                let mut saved = self.relay.lock().await;
                *saved = next;
                relay_state(saved.as_deref())
            }
            "identity" => {
                let saved = self.relay.lock().await;
                let address = request["relay"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .or(saved.as_deref());
                match address {
                    Some(address) => match normalize_relay(address) {
                        Some(relay) => RpcResult::ok(identity(&relay)),
                        None => refusal(
                            "invalid-relay",
                            format!("{address} is not a usable Relay address"),
                            json!({"relay": address}),
                        ),
                    },
                    None => refusal(
                        "relay-url-required",
                        format!("{NO_ADDRESS}; save one in the Share dialog first"),
                        json!({}),
                    ),
                }
            }
            "create" | "testPublish" => {
                if request["sessionId"] == "" {
                    refusal(
                        "missing-session-id",
                        format!("{operation} requires a sessionId"),
                        json!({}),
                    )
                } else {
                    let saved = self.relay.lock().await;
                    match (operation, saved.as_deref()) {
                        ("create", None) => refusal(
                            "relay-url-required",
                            format!("{NO_ADDRESS}; save one in the Share dialog first"),
                            json!({}),
                        ),
                        ("create", Some(relay)) => refusal(
                            "owner-login-required",
                            format!("no verified owner credential is stored for {relay}"),
                            json!({"identity": identity(relay)}),
                        ),
                        (_, None) => RpcResult::ok(
                            json!({"ok": false, "blocking": true, "error": NO_ADDRESS}),
                        ),
                        (_, Some(relay)) => RpcResult::ok(
                            json!({"ok": false, "blocking": true, "error": format!("{relay}: no verified owner credential is stored for this address")}),
                        ),
                    }
                }
            }
            "audit" | "stop" | "setCapabilities" | "updatePermissions" => {
                let share_id = request["shareId"].as_str().unwrap_or_default();
                if share_id.is_empty()
                    && matches!(operation, "setCapabilities" | "updatePermissions")
                {
                    refusal(
                        "missing-share-id",
                        format!("{operation} requires a shareId"),
                        json!({}),
                    )
                } else if operation == "updatePermissions"
                    && ["sessionMode", "allowsFork", "ttlMinutes"]
                        .iter()
                        .all(|field| request.get(field).is_none())
                {
                    refusal(
                        "missing-permission",
                        "updatePermissions requires a permission or lifetime change",
                        json!({}),
                    )
                } else if operation == "updatePermissions"
                    && request
                        .get("ttlMinutes")
                        .is_some_and(|ttl| !ttl.is_null() && !valid_ttl(ttl))
                {
                    refusal(
                        "invalid-ttl",
                        "ttlMinutes is outside the allowed range",
                        json!({}),
                    )
                } else {
                    refusal(
                        "share-not-found",
                        format!("share {share_id} is not published"),
                        json!({"shareId": share_id}),
                    )
                }
            }
            "testRelay" => {
                let saved = self.relay.lock().await.clone();
                match saved {
                    None => RpcResult::ok(
                        json!({"ok": false, "httpReachable": false, "blocking": true, "error": NO_ADDRESS}),
                    ),
                    Some(relay) => test_relay(&relay).await,
                }
            }
            _ => return None,
        };
        Some(answer)
    }
}

fn refusal(code: &str, message: impl Into<String>, details: Value) -> RpcResult {
    RpcResult::failed(Failure {
        code: format!("liveshareShares/{code}"),
        message: message.into(),
        details,
    })
}

fn identity(relay: &str) -> Value {
    json!({"state": "unverified", "relay": relay})
}

fn relay_state(relay: Option<&str>) -> RpcResult {
    RpcResult::ok(match relay {
        Some(relay) => json!({"override": relay, "effective": relay}),
        None => json!({}),
    })
}

fn valid_ttl(ttl: &Value) -> bool {
    ttl.as_f64()
        .is_some_and(|n| n.fract() == 0.0 && n > 0.0 && n <= 10080.0)
}

fn normalize_relay(value: &str) -> Option<String> {
    let mut url = reqwest::Url::parse(value).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    url.set_query(None);
    url.set_fragment(None);
    let path = url.path().trim_end_matches('/').to_string();
    url.set_path(&path);
    Some(url.as_str().trim_end_matches('/').to_string())
}

async fn test_relay(relay: &str) -> RpcResult {
    let result = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(client) => client.get(relay).send().await,
        Err(error) => {
            return RpcResult::ok(
                json!({"ok": false, "httpReachable": false, "blocking": true, "error": format!("{relay}: {error}")}),
            )
        }
    };
    RpcResult::ok(match result {
        Ok(_) => json!({"ok": true, "httpReachable": true, "blocking": false}),
        Err(error) => {
            json!({"ok": false, "httpReachable": false, "blocking": true, "error": format!("{relay}: {error}")})
        }
    })
}
