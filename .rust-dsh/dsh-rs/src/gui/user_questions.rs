//! `userQuestions/answer` and the `userQuestions/attachWait` stream.
//!
//! In the TypeScript host a timed `ask_user_question` can outlive its foreground
//! wait; the answer then arrives through this endpoint and is steered into the
//! agent as a user message. This build serves only the blocking tool (see
//! [`crate::plugins::ask_user`]), whose answers return through `$events/result`,
//! so no question is ever continued and the endpoint answers `false`, the
//! TypeScript reply for "this call is not a continued question".
//!
//! Everything that precedes that lookup follows the TypeScript host: the three
//! fields are validated at the gateway (`agentId`, `callId`, `answer`, in that
//! order), then the session is looked up (`session/not-found`).
//! `attachWait` validates the live runtime root and ends without frames: this
//! broker has no timed waits to claim, even while a blocking question is open.

use serde_json::{json, Value};

use super::router::{boundary, check_descriptor, require_session};
use super::wire::RpcResult;
use cordis::Context;

const ANSWER: &str = "userQuestions/answer";

/// Serve `method` when it is a user-questions endpoint.
pub fn dispatch(ctx: &Context, method: &str, payload: &Value) -> Option<RpcResult> {
    match method {
        ANSWER => Some(answer(ctx, payload)),
        _ => None,
    }
}

fn answer(ctx: &Context, payload: &Value) -> RpcResult {
    if let Err(failure) = check_descriptor(ANSWER, &["agentId", "callId", "answer"], payload) {
        return RpcResult::failed(failure);
    }
    let Some(agent_id) = payload["agentId"].as_str() else {
        return RpcResult::failed(boundary(ANSWER, "agentId"));
    };
    if !payload["callId"].is_string() {
        return RpcResult::failed(boundary(ANSWER, "callId"));
    }
    if !is_answer_batch(&payload["answer"]) {
        return RpcResult::failed(boundary(ANSWER, "answer"));
    }
    if let Err(failure) = require_session(ctx, ANSWER, agent_id) {
        return RpcResult::failed(failure);
    }
    RpcResult::ok(json!(false))
}

/// `{ answers: [{ id: string, selected: string[], custom?: string }] }`; fields
/// beyond these are ignored, as the TypeScript boundary ignores them.
fn is_answer_batch(value: &Value) -> bool {
    let Some(answers) = value.get("answers").and_then(Value::as_array) else {
        return false;
    };
    answers.iter().all(|item| {
        item.get("id").is_some_and(Value::is_string)
            && item
                .get("selected")
                .and_then(Value::as_array)
                .is_some_and(|selected| selected.iter().all(Value::is_string))
            && item.get("custom").is_none_or(|custom| custom.is_string())
    })
}

/// Serve `userQuestions/attachWait`. Blocking questions have no timed claim.
pub(super) async fn attach_wait(
    ctx: &Context,
    args: &Value,
    _stream_id: &str,
    _out: &tokio::sync::mpsc::Sender<String>,
    cancel: &tokio_util::sync::CancellationToken,
) -> super::mux::Outcome {
    use super::mux::Outcome;
    use super::wire::Failure;
    use crate::api::services::AgentRegistryService;
    use crate::plugins::subagent::{Lineage, LINEAGE_SERVICE};

    const ENDPOINT: &str = "userQuestions/attachWait";
    if cancel.is_cancelled() {
        return Outcome::Cancelled;
    }
    if let Err(failure) = super::descriptor::check(ENDPOINT, args) {
        return Outcome::Failed(failure);
    }
    let Some(id) = args["agentId"].as_str() else {
        return Outcome::Failed(boundary(ENDPOINT, "agentId"));
    };
    // The agent is a lookup parameter: the host resolves it before the method
    // runs, and an identity it does not hold is `session/not-found`.
    let Some(_agent) = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .and_then(|agents| agents.get(id))
    else {
        return Outcome::Failed(Failure {
            code: "session/not-found".into(),
            message: format!("session {id:?} not found"),
            details: json!({ "sessionId": id }),
        });
    };
    let parent = ctx
        .get::<Lineage>(LINEAGE_SERVICE)
        .and_then(|lineage| lineage.parent_of(id));
    let owned = parent.is_some_and(|parent| {
        ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
            .is_some_and(|agents| agents.get(&parent).is_some())
    });
    if owned {
        // The TypeScript host throws here, and a thrown error reaches the client
        // as `gateway/internal` with the error's own message.
        return Outcome::Failed(Failure {
            code: "gateway/internal".into(),
            message: "human interaction is unavailable while the calling agent is owned by another live agent; include the unresolved question or decision in the child agent's final result".into(),
            details: json!({}),
        });
    }
    Outcome::Ended
}
