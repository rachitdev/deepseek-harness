//! The Claude Code band's empty drawing in a host with no mods installed.
//!
//! TypeScript ignores stale or unknown actions and returns the current snapshot;
//! it does not turn an ignored press into a remote failure.

use cordis::Context;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::mux::{deliver, Outcome};
use super::wire::RpcResult;

/// Serve `claudeCodeMods/pressBand`.
pub(super) fn dispatch(ctx: &Context, endpoint: &str, args: &Value) -> Option<RpcResult> {
    if endpoint != "claudeCodeMods/pressBand" {
        return None;
    }
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Some(RpcResult::failed(failure));
    }
    let session = args["agentId"].as_str().unwrap_or_default();
    if let Err(failure) = super::router::require_session(ctx, endpoint, session) {
        return Some(RpcResult::failed(failure));
    }
    // A press does not draw an unobserved band. Watching draws the empty tree
    // once (generation 1); before that, the table's initial generation is 0.
    let generation = ctx.get::<Bands>(BANDS_SERVICE).map(|bands| bands.generation(session)).unwrap_or(0);
    Some(RpcResult::ok(json!({ "generation": generation, "tree": null })))
}

/// Session bands that have received their first drawing.
#[derive(Default)]
pub struct Bands(std::sync::Mutex<std::collections::HashSet<String>>);

/// Context service key holding the band's generation across unary and stream calls.
pub const BANDS_SERVICE: &str = "gui.claude-code-bands";

impl Bands {
    fn generation(&self, session: &str) -> u8 {
        match self.0.lock() {
            Ok(drawn) => u8::from(drawn.contains(session)),
            Err(poisoned) => u8::from(poisoned.into_inner().contains(session)),
        }
    }

    fn draw(&self, session: &str) {
        match self.0.lock() {
            Ok(mut drawn) => { drawn.insert(session.to_string()); }
            Err(poisoned) => { poisoned.into_inner().insert(session.to_string()); }
        }
    }
}

/// Deliver the current empty drawing, then wait until the client cancels.
pub(super) async fn watch_band(
    ctx: &Context,
    endpoint: &str,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Outcome::Failed(failure);
    }
    let session = args["agentId"].as_str().unwrap_or_default();
    if let Err(failure) = super::router::require_session(ctx, endpoint, session) {
        return Outcome::Failed(failure);
    }
    if let Some(bands) = ctx.get::<Bands>(BANDS_SERVICE) {
        bands.draw(session);
    }
    if deliver(out, stream_id, json!({ "generation": 1, "tree": null }), cancel).await.is_err() {
        return Outcome::Cancelled;
    }
    cancel.cancelled().await;
    Outcome::Cancelled
}
