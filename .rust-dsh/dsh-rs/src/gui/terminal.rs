//! The six unary `terminal/*` controller endpoints.

use cordis::Context;
use serde_json::{json, Value};

use super::router::{boundary, require_session, Router};
use super::wire::{codes, Failure, RpcResult};

/// Dispatch only the unary controller methods; streams have separate ownership.
pub(super) async fn dispatch(router: &Router, ctx: &Context, method: &str, payload: &Value) -> Option<RpcResult> {
    let name = method.strip_prefix("terminal/")?;
    if !matches!(name, "shells" | "environment" | "list" | "create" | "close" | "rename") { return None; }
    Some(match run(router, ctx, method, name, payload).await {
        Ok(Some(value)) => RpcResult::ok(value),
        Ok(None) => RpcResult::Empty,
        Err(error) => RpcResult::failed(error),
    })
}

async fn run(router: &Router, ctx: &Context, method: &str, name: &str, payload: &Value) -> Result<Option<Value>, Failure> {
    let service = router.terminal.as_ref().ok_or_else(|| Failure::at(codes::SERVICE_UNAVAILABLE, "the terminal service is not wired into this build", method))?;
    let text = |key: &str| payload[key].as_str().ok_or_else(|| boundary(method, key));
    if name == "list" { return Ok(Some(json!(service.list(text("sessionId")?).await))); }
    let session = text("agentId")?;
    require_session(ctx, method, session)?;
    match name {
        "shells" => Ok(Some(json!(service.shells(session)?))),
        "environment" => Ok(Some(service.environment(session)?)),
        "create" => Ok(Some(service.create(session, &payload["request"]).await?)),
        "close" => { service.close(session, text("id")?).await; Ok(None) }
        "rename" => { service.rename(session, text("id")?, text("title")?).await?; Ok(None) }
        _ => unreachable!("dispatch filters unary methods"),
    }
}
