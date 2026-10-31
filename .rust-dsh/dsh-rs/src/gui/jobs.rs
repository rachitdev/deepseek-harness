//! The `job/*` endpoints: the background jobs a model started.
//!
//! The TypeScript host keeps a registry of background jobs a model started.
//! This build has none: a shell command that outlives its tool call is left to
//! the command, and nothing tracks it. Every job is therefore unknown, which is
//! what the host answers for an id it has not seen, down to the `Error:` prefix
//! its message carries. When a job registry exists this module is where the
//! lookup goes.
//!
//! The two streams keep the TypeScript host's shape: `job/list` answers the
//! session's rows — empty here — and then watches for changes, and `job/follow`
//! answers one job's output, which no id here has.

use serde_json::{json, Value};
use tokio::sync::mpsc;

use super::mux::{deliver, Outcome};
use super::router::{boundary, check_descriptor};
use super::wire::{codes, Failure, RpcResult};
use tokio_util::sync::CancellationToken;

const ENDPOINT: &str = "job/kill";
const LIST: &str = "job/list";
const FOLLOW: &str = "job/follow";

/// Serve `job/kill`.
pub(super) fn kill(payload: &Value) -> RpcResult {
    match unknown_job(payload) {
        Ok(failure) | Err(failure) => RpcResult::failed(failure),
    }
}

/// `job/list`: the session's job rows, then the changes to them.
pub(super) async fn list(
    endpoint: &str,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Outcome::Failed(failure);
    }
    // This build tracks no jobs, so the session's rows are empty. The stream
    // stays open, as the TypeScript host's does, because a row can still
    // appear.
    if deliver(out, stream_id, json!({ "type": "rows", "jobs": [] }), cancel).await.is_err() {
        return Outcome::Cancelled;
    }
    cancel.cancelled().await;
    Outcome::Cancelled
}

/// `job/follow`: one job's output.
pub(super) async fn follow(
    endpoint: &str,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Outcome::Failed(failure);
    }
    let request = &args["request"];
    let Some(job_id) = request.get("jobId").and_then(Value::as_str) else {
        // The TypeScript host's boundary refuses a request with no job.
        return Outcome::Failed(Failure {
            code: codes::INPUT_INVALID.to_string(),
            message: format!("typert gateway: {endpoint}: wire field \"request\" failed boundary validation"),
            details: json!({ "endpoint": endpoint, "field": "request" }),
        });
    };
    let _ = (stream_id, out, cancel);
    Outcome::Failed(Failure {
        code: codes::INTERNAL.to_string(),
        message: format!("unknown job {job_id}"),
        details: json!({}),
    })
}

/// The failure for a well-formed request, or the failure for a malformed one.
fn unknown_job(payload: &Value) -> Result<Failure, Failure> {
    check_descriptor(ENDPOINT, &["request"], payload)?;
    let request = &payload["request"];
    let text = |name: &str| request.get(name).and_then(Value::as_str);
    let (Some(session_id), Some(job_id)) = (text("sessionId"), text("jobId")) else {
        return Err(boundary(ENDPOINT, "request"));
    };
    Ok(Failure {
        code: "job/not-found".to_string(),
        message: format!("Error: unknown job {job_id}"),
        details: json!({ "sessionId": session_id, "jobId": job_id }),
    })
}
