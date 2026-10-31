//! Card endpoints for a host that never logs action or recommendation cards.

use super::router::require_session;
use crate::api::services::AgentRegistryService;
use crate::plugins::subagent::{Lineage, LINEAGE_SERVICE};
use cordis::Context;

use serde_json::{json, Value};

use super::wire::RpcResult;

/// A card service with no proposals, kinds, or recommendation catalog.
#[derive(Default)]
pub struct CardsService;

impl CardsService {
    /// Construct the empty card service.
    pub fn new() -> Self {
        Self
    }

    /// Resolve the agent after descriptor validation, then check root and card state.
    ///
    /// Unknown cards are successful refusal values, not gateway failures.
    pub fn dispatch(&self, ctx: &Context, method: &str, args: &Value) -> Option<RpcResult> {
        match method {
            "actionCards/act" | "pluginRecommendations/describe" => {
                let id = args["agentId"].as_str().unwrap_or_default();
                if let Err(failure) = require_session(ctx, method, id) {
                    return Some(RpcResult::failed(failure));
                }
                let live = ctx
                    .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                    .is_some_and(|agents| agents.get(id).is_some());
                // Stored sessions resolve, but only a live root can act on cards.
                let child = ctx
                    .get::<Lineage>(LINEAGE_SERVICE)
                    .is_some_and(|lineage| lineage.is_child(id));
                let reason = if !live || child {
                    "not-live-root"
                } else {
                    "unknown"
                };
                Some(RpcResult::ok(
                    json!({ "status": "refused", "reason": reason }),
                ))
            }
            _ => None,
        }
    }
}
