//! The `permission` plugin: switchable permission presets.
//!
//! It provides the validated [`Permissions`] table as the `permissions` service
//! (which makes the Web GUI offer `/permission`, the preset catalog and the
//! `permissions` projection) and enforces the sandbox mode before a tool runs.
//! The approval policy is honoured by the `approvals` plugin, which reads the
//! same service. See [`crate::permission`] for what is and is not enforced.

use std::sync::Arc;

use cordis::plugin::BoxFuture;
use cordis::Plugin;
use serde_json::{json, Value};

use crate::permission::{
    session_of, Knobs, PermissionConfig, Permissions, SandboxCall, PERMISSIONS_SERVICE,
};
use crate::plugins::approvals::APPROVALS_SERVICE;

/// The `permission` plugin. Requires the `tools` and `agents` services.
pub fn permission_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "permission",
        vec![
            cordis::Injection::new(crate::api::TOOLS_SERVICE.to_string()),
            cordis::Injection::new(crate::api::AGENTS_SERVICE.to_string()),
        ],
        |ctx, config: PermissionConfig| async move {
            let table = Permissions::from_config(&config).map_err(cordis::Error::msg)?;
            ctx.provide(PERMISSIONS_SERVICE, table).await?;
            // A new session logs the permission setup it runs, as the
            // TypeScript host does when it creates one.
            ctx.on("session/created", |ctx, payload, _next| {
                Box::pin(async move {
                    let Some(session) = payload["session"]
                        .as_str()
                        .and_then(|id| session_of(&ctx, id))
                    else {
                        return Ok(Value::Null);
                    };
                    if let Some(table) = crate::permission::provided(&ctx) {
                        table.pin(&*session);
                    }
                    Ok(Value::Null)
                }) as BoxFuture<cordis::Result<Value>>
            })
            .await?;
            ctx.on("tools/pre-execute", |ctx, payload, next| {
                Box::pin(async move {
                    if let Some(reason) = sandbox_verdict(&ctx, &payload) {
                        return Ok(json!({ "kind": "deny", "reason": reason }));
                    }
                    next.run(payload).await
                }) as BoxFuture<cordis::Result<Value>>
            })
            .await?;
            Ok(())
        },
    )
}

/// Why the sandbox refuses the call in `payload`, if it does. Without the
/// `permission` plugin nothing is refused. A call from no known session is
/// judged by the deployment's default knobs, with no working directory.
pub fn sandbox_verdict(ctx: &cordis::Context, payload: &Value) -> Option<String> {
    let table = crate::permission::provided(ctx)?;
    let Some(tool) = payload.get("name").and_then(Value::as_str) else {
        return Some("a tool call without a name is refused by the sandbox".to_string());
    };
    let session = payload
        .get("agent")
        .and_then(Value::as_str)
        .and_then(|agent| session_of(ctx, agent));
    let knobs = session
        .as_ref()
        .map_or_else(Knobs::default, |session| Permissions::knobs(&session.events()));
    // The file tools resolve a relative path against the session's working
    // directory, else the process's; that directory is the workspace.
    let workspace = session
        .as_ref()
        .and_then(|session| session.header_cwd())
        .or_else(|| std::env::current_dir().ok().map(|dir| dir.to_string_lossy().into_owned()));
    table.sandbox_denial(&SandboxCall {
        knobs: &knobs,
        workspace: workspace.as_deref(),
        base: None,
        tool,
        arguments: payload.get("arguments").unwrap_or(&Value::Null),
        approvals_present: ctx
            .get::<crate::plugins::approvals::ApprovalBroker>(APPROVALS_SERVICE)
            .is_some(),
    })
}
