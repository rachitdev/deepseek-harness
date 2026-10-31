//! Forge panel endpoints for a host without forge providers, Git, or worktrees.
//!
//! Reads preserve the TypeScript result types with empty contents. Mutations
//! cannot change session records and use the TypeScript refusal vocabulary.
//! Followers receive pending and settled states, then wait for owner disposal.

use cordis::Context;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::mux::{deliver, Outcome};
use super::router::{boundary, require_session};
use super::session_read::observe_session;
use super::wire::{Failure, RpcResult};

fn refusal(code: &str, message: &str) -> RpcResult {
    RpcResult::failed(Failure {
        code: format!("forgePanel/{code}"),
        message: message.to_string(),
        details: json!({}),
    })
}

fn session_id<'a>(ctx: &Context, endpoint: &str, args: &'a Value) -> Result<&'a str, Failure> {
    let id = args
        .get("sessionId")
        .and_then(Value::as_str)
        .ok_or_else(|| boundary(endpoint, "sessionId"))?;
    require_session(ctx, endpoint, id)?;
    Ok(id)
}

/// Serve the sixteen unary forge panel endpoints; other methods fall through.
pub(super) fn dispatch(ctx: &Context, endpoint: &str, args: &Value) -> Option<RpcResult> {
    if !matches!(
        endpoint,
        "forgePanel/branches"
            | "forgePanel/cancelWorktree"
            | "forgePanel/changeRequestDiff"
            | "forgePanel/changeRequestText"
            | "forgePanel/continueInProject"
            | "forgePanel/dismiss"
            | "forgePanel/localDiff"
            | "forgePanel/localText"
            | "forgePanel/markReady"
            | "forgePanel/merge"
            | "forgePanel/refresh"
            | "forgePanel/requestWorktree"
            | "forgePanel/restoreWorktree"
            | "forgePanel/retryWorktree"
            | "forgePanel/stopCreation"
            | "forgePanel/track"
    ) {
        return None;
    }
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Some(RpcResult::failed(failure));
    }
    if let Err(failure) = session_id(ctx, endpoint, args) {
        return Some(RpcResult::failed(failure));
    }
    Some(match endpoint {
        "forgePanel/branches" => RpcResult::ok(json!({ "current": null, "branches": [] })),
        "forgePanel/localDiff" => RpcResult::ok(json!({ "base": null, "files": [], "omitted": 0 })),
        "forgePanel/changeRequestDiff" => RpcResult::ok(json!({
            "headSha": "", "files": [], "localAhead": null, "fileUrls": {},
        })),
        "forgePanel/localText" | "forgePanel/changeRequestText" => RpcResult::ok(json!("")),
        "forgePanel/track" => refusal(
            "unrecognized-url",
            "No code host provider recognizes the change request URL",
        ),
        "forgePanel/merge" | "forgePanel/markReady" | "forgePanel/dismiss" => refusal(
            "not-bound",
            "The change request is not bound to this Session or no provider serves it",
        ),
        "forgePanel/continueInProject" | "forgePanel/restoreWorktree" => {
            refusal("no-worktree", "The Session has no created worktree")
        }
        "forgePanel/retryWorktree" => refusal(
            "not-waiting",
            "No message waits for another attempt at the worktree",
        ),
        "forgePanel/requestWorktree" if args.get("from").and_then(Value::as_str) == Some("") => {
            refusal("invalid-branch", "The base branch name is empty")
        }
        _ => refusal("worktrees-unavailable", "No worktree service is mounted"),
    })
}

/// Send the empty baseline of either forge stream and wait for cancellation.
pub(super) async fn stream(
    ctx: &Context,
    endpoint: &str,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    let (lifetime, effect) = match super::tool_results::lifetime(ctx).await {
        Ok(pair) => pair,
        Err(error) => {
            return Outcome::Failed(Failure::at(
                super::wire::codes::INTERNAL,
                error.to_string(),
                endpoint,
            ))
        }
    };
    let outcome = tokio::select! {
        biased;
        _ = lifetime.cancelled() => Outcome::Ended,
        _ = cancel.cancelled() => Outcome::Cancelled,
        outcome = stream_inner(ctx, endpoint, args, stream_id, out, cancel) => outcome,
    };
    let _ = effect.dispose().await;
    outcome
}

async fn stream_inner(
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
    let baseline = if endpoint == "forgePanel/changeRequestStates" {
        json!({ "type": "baseline", "states": [] })
    } else {
        let id = match session_id(ctx, endpoint, args) {
            Ok(id) => id,
            Err(failure) => return Outcome::Failed(failure),
        };
        let directory = match observe_session(ctx, id) {
            Ok(observed) => observed.and_then(|session| session.cwd).unwrap_or_default(),
            Err(failure) => return Outcome::Failed(failure),
        };
        json!({
            "directory": directory,
            "checkout": { "state": "none" },
            "repository": null,
            "readiness": null,
            "changeRequests": [],
            "worktree": {
                "editable": false, "blocker": "not-git", "request": null,
                "created": null, "creation": null, "failure": null,
                "missing": null, "restoration": null, "waiting": null,
            },
            "local": null,
        })
    };
    if endpoint == "forgePanel/follow" {
        let mut pending = baseline.clone();
        pending["checkout"] = json!({ "state": "pending" });
        pending["worktree"]["blocker"] = Value::Null;
        if deliver(out, stream_id, pending, cancel).await.is_err() {
            return Outcome::Cancelled;
        }
    }
    if deliver(out, stream_id, baseline, cancel).await.is_err() {
        return Outcome::Cancelled;
    }
    cancel.cancelled().await;
    Outcome::Cancelled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn owner_disposal_ends_states_reader() {
        let (ctx, owner) = super::super::tool_results::test_owner().await;
        let (out, mut frames) = mpsc::channel(1);
        let task = tokio::spawn(async move {
            stream(
                &ctx,
                "forgePanel/changeRequestStates",
                &json!({}),
                "forge",
                &out,
                &CancellationToken::new(),
            )
            .await
        });
        frames.recv().await.unwrap();
        owner.dispose().await;
        assert!(matches!(
            tokio::time::timeout(std::time::Duration::from_secs(2), task)
                .await
                .unwrap()
                .unwrap(),
            Outcome::Ended
        ));
    }
}
