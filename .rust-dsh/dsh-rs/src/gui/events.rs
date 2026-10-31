//! Host→client events over `$events`: approval and user questions and their answers.
//!
//! TypeScript forwards the `approval/request` and `user-questions/request`
//! waterfalls to the browser as `waterfall` frames; the browser answers by
//! calling the unary endpoint `$events/result`. This module is the Rust side of
//! that pairing:
//!
//! - [`EventHub`] holds, per open `$events` stream, the questions still waiting
//!   (`clientId` → `eventId` → [`Pending`]);
//! - [`EventHub::result`] validates one `$events/result` payload and answers.
//!
//! Only `allowed-once` allows a call. Every other outcome, including `next`
//! (no listener claimed it), `rejected`, `cancelled`, `unavailable` and an
//! unrecognised value, denies: with a single approver there is nobody to
//! delegate to, and an unknown answer must never authorise anything. The
//! browser has no "allow for this run" outcome in the TypeScript vocabulary, so
//! this surface never grants it.
//!
//! A user question is answered by `{kind:"result", value:{answers:[…]}}` with one
//! item per question id; a batch that does not name each question exactly once is
//! refused with `gateway/input-invalid` and the question stays open. `rejected`
//! and `next` fail the asking tool call.
//!
//! A pending question is dropped, which denies or fails the call, when its stream
//! ends, when the client is replaced, or when the call stops waiting (timeout or
//! cancel).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use super::router::input_invalid;
use super::wire::{codes, Failure};
use crate::plugins::approvals::{ApprovalDecision, PendingApproval};
use crate::plugins::ask_user::{Answer, PendingQuestion, Reply};

/// The unary endpoint carrying a client's answer to a forwarded event.
pub const RESULT_ENDPOINT: &str = "$events/result";

/// One question the browser has been sent and has not answered.
pub enum Pending {
    Approval(PendingApproval),
    Question(PendingQuestion),
}

impl Pending {
    fn is_open(&self) -> bool {
        match self {
            Pending::Approval(approval) => approval.is_open(),
            Pending::Question(question) => question.is_open(),
        }
    }
}

impl From<PendingApproval> for Pending {
    fn from(approval: PendingApproval) -> Self {
        Pending::Approval(approval)
    }
}

impl From<PendingQuestion> for Pending {
    fn from(question: PendingQuestion) -> Self {
        Pending::Question(question)
    }
}

/// One open `$events` stream's unanswered questions.
#[derive(Default)]
struct EventClient {
    pending: Mutex<HashMap<String, Pending>>,
}

/// What the browser said about one event, before it is read for a question kind.
#[derive(Debug, Clone, PartialEq)]
enum Outcome {
    /// No listener claimed the event.
    Next,
    /// A result, with its value when it carried one.
    Result(Option<Value>),
    /// The listener threw; the text is its message.
    Rejected(String),
}

/// Open `$events` streams and what each is still waiting on.
#[derive(Default)]
pub struct EventHub {
    clients: Mutex<HashMap<String, Arc<EventClient>>>,
}

impl std::fmt::Debug for EventHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EventHub")
    }
}

/// A registered stream. Dropping it denies every question it still holds.
pub struct Registration {
    hub: Arc<EventHub>,
    client_id: String,
    client: Arc<EventClient>,
}

impl EventHub {
    /// Register a stream under `client_id`.
    pub fn register(self: &Arc<Self>, client_id: &str) -> Registration {
        let client = Arc::new(EventClient::default());
        self.clients
            .lock()
            .unwrap()
            .insert(client_id.to_string(), Arc::clone(&client));
        Registration {
            hub: Arc::clone(self),
            client_id: client_id.to_string(),
            client,
        }
    }

    /// The kinds of question still open for `agent` across every stream, as
    /// `approval` and `question`, each at most once, in that order.
    pub fn pending_kinds(&self, agent: &str) -> Vec<&'static str> {
        let (mut approval, mut question) = (false, false);
        for client in self.clients.lock().unwrap().values() {
            for pending in client.pending.lock().unwrap().values() {
                let (asked_by, is_approval) = match pending {
                    Pending::Approval(a) => (a.agent.as_deref(), true),
                    Pending::Question(q) => (q.agent.as_deref(), false),
                };
                if asked_by == Some(agent) && pending.is_open() {
                    if is_approval {
                        approval = true;
                    } else {
                        question = true;
                    }
                }
            }
        }
        let mut kinds = Vec::new();
        if approval {
            kinds.push("approval");
        }
        if question {
            kinds.push("question");
        }
        kinds
    }

    /// Apply one `$events/result` payload (the unwrapped `args` object).
    ///
    /// An unknown `clientId` is an error: the stream it names is gone. An
    /// unknown `eventId` is a no-op, because the question may have just timed
    /// out or been answered; the TypeScript gateway treats it the same way.
    pub fn result(&self, args: &Value) -> Result<(), Failure> {
        let (client_id, event_id, outcome) = parse_result(args)?;
        let client = self
            .clients
            .lock()
            .unwrap()
            .get(client_id)
            .cloned()
            .ok_or_else(|| {
                Failure::at(
                    codes::LOOKUP_NOT_FOUND,
                    "this result names no active event stream",
                    RESULT_ENDPOINT,
                )
            })?;
        let mut pending = client.pending.lock().unwrap();
        // A malformed answer leaves the question open for the browser to resend.
        let reply = match pending.get(event_id) {
            None => return Ok(()),
            Some(Pending::Approval(_)) => None,
            Some(Pending::Question(question)) => {
                Some(question_reply(&question.questions, &outcome)?)
            }
        };
        match (pending.remove(event_id), reply) {
            (Some(Pending::Approval(approval)), _) => approval.answer(approval_decision(&outcome)),
            (Some(Pending::Question(question)), Some(reply)) => question.answer(reply),
            _ => {}
        }
        Ok(())
    }
}

impl Registration {
    /// Hold `question` until it is answered; its `eventId` is returned.
    pub fn hold(&self, event_id: &str, question: impl Into<Pending>) {
        self.client
            .pending
            .lock()
            .unwrap()
            .insert(event_id.to_string(), question.into());
    }

    /// Drop questions nobody is waiting for any more and return their ids, so
    /// the client can be told to withdraw them.
    pub fn sweep(&self) -> Vec<String> {
        let mut pending = self.client.pending.lock().unwrap();
        let stale: Vec<String> = pending
            .iter()
            .filter(|(_, pending)| !pending.is_open())
            .map(|(id, _)| id.clone())
            .collect();
        for id in &stale {
            pending.remove(id);
        }
        stale
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        let mut clients = self.hub.clients.lock().unwrap();
        // A later registration may have reused the id; remove only our own.
        if clients
            .get(&self.client_id)
            .is_some_and(|current| Arc::ptr_eq(current, &self.client))
        {
            clients.remove(&self.client_id);
        }
        // `pending` drops with `client`, which denies whatever is left.
    }
}

fn bad(field: &str, message: &str) -> Failure {
    input_invalid(RESULT_ENDPOINT, field, message)
}

fn has_exact_keys(value: &Value, keys: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|o| o.len() == keys.len() && keys.iter().all(|k| o.contains_key(*k)))
}

fn non_empty<'a>(value: &'a Value, field: &str) -> Result<&'a str, Failure> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| bad(field, &format!("{field} must be a non-empty string")))
}

/// Check the payload the way the TypeScript gateway does (exact key sets) and
/// read the outcome.
fn parse_result(args: &Value) -> Result<(&str, &str, Outcome), Failure> {
    if !has_exact_keys(args, &["clientId", "eventId", "outcome"]) {
        return Err(bad(
            "args",
            "a result has exactly clientId, eventId and outcome",
        ));
    }
    let client_id = non_empty(args, "clientId")?;
    let event_id = non_empty(args, "eventId")?;
    let outcome = &args["outcome"];
    let parsed = match outcome.get("kind").and_then(Value::as_str) {
        Some("next") if has_exact_keys(outcome, &["kind"]) => Outcome::Next,
        Some("result")
            if has_exact_keys(outcome, &["kind"])
                || has_exact_keys(outcome, &["kind", "value"]) =>
        {
            Outcome::Result(outcome.get("value").cloned())
        }
        Some("rejected") if has_exact_keys(outcome, &["kind", "error"]) => Outcome::Rejected(
            outcome["error"]
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("the listener rejected the event")
                .to_string(),
        ),
        _ => return Err(bad("outcome", "outcome is not a recognised result")),
    };
    Ok((client_id, event_id, parsed))
}

/// Only `allowed-once` allows a call; every other outcome denies it.
fn approval_decision(outcome: &Outcome) -> ApprovalDecision {
    match outcome {
        Outcome::Result(Some(Value::String(value))) if value == "allowed-once" => {
            ApprovalDecision::Allow
        }
        _ => ApprovalDecision::Deny,
    }
}

/// Read an outcome as the reply to a question batch, or refuse a result that is
/// not an answer to exactly those questions.
fn question_reply(questions: &Value, outcome: &Outcome) -> Result<Reply, Failure> {
    match outcome {
        Outcome::Next => Ok(Reply::Refused(
            "no user-questions answerer accepted the request".to_string(),
        )),
        Outcome::Rejected(message) => Ok(Reply::Refused(message.clone())),
        Outcome::Result(value) => {
            let value = value
                .as_ref()
                .ok_or_else(|| bad("outcome", "an answer carries a value"))?;
            let list = value
                .get("answers")
                .filter(|_| has_exact_keys(value, &["answers"]))
                .ok_or_else(|| bad("outcome", "an answer is exactly { answers: [...] }"))?;
            let answers: Vec<Answer> = serde_json::from_value(list.clone())
                .map_err(|error| bad("outcome", &format!("malformed answers: {error}")))?;
            let mut asked: Vec<&str> = questions
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|item| item.get("id").and_then(Value::as_str))
                .collect();
            let mut given: Vec<&str> = answers.iter().map(|a| a.id.as_str()).collect();
            asked.sort_unstable();
            given.sort_unstable();
            if asked != given {
                return Err(bad(
                    "outcome",
                    &format!(
                        "the answers must name each of the {} questions exactly once",
                        asked.len()
                    ),
                ));
            }
            Ok(Reply::Answered(answers))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(outcome: Value) -> Value {
        json!({ "clientId": "c1", "eventId": "e1", "outcome": outcome })
    }

    fn decision(outcome: Value) -> Result<ApprovalDecision, Failure> {
        parse_result(&args(outcome)).map(|(_, _, outcome)| approval_decision(&outcome))
    }

    #[test]
    fn only_allowed_once_allows() {
        let allow = decision(json!({ "kind": "result", "value": "allowed-once" }));
        assert_eq!(allow.unwrap(), ApprovalDecision::Allow);
        for value in [
            json!("rejected"),
            json!("cancelled"),
            json!("unavailable"),
            json!("allowed-always"),
            json!("ALLOWED-ONCE"),
            json!(true),
            json!(null),
        ] {
            let outcome = json!({ "kind": "result", "value": value });
            assert_eq!(
                decision(outcome).unwrap(),
                ApprovalDecision::Deny,
                "{value}"
            );
        }
        assert_eq!(
            decision(json!({ "kind": "result" })).unwrap(),
            ApprovalDecision::Deny
        );
        assert_eq!(
            decision(json!({ "kind": "next" })).unwrap(),
            ApprovalDecision::Deny
        );
        let rejected = json!({ "kind": "rejected", "error": { "name": "E", "message": "m" } });
        assert_eq!(decision(rejected).unwrap(), ApprovalDecision::Deny);
    }

    #[test]
    fn malformed_results_are_refused_not_treated_as_answers() {
        for bad_args in [
            json!({ "clientId": "c1", "eventId": "e1" }),
            json!({ "clientId": "c1", "eventId": "e1", "outcome": { "kind": "next" }, "extra": 1 }),
            json!({ "clientId": "", "eventId": "e1", "outcome": { "kind": "next" } }),
            json!({ "clientId": "c1", "eventId": 7, "outcome": { "kind": "next" } }),
            args(json!({ "kind": "next", "value": "allowed-once" })),
            args(json!({ "kind": "maybe" })),
            args(json!({ "kind": "rejected" })),
            args(json!("allowed-once")),
            json!([]),
        ] {
            assert!(parse_result(&bad_args).is_err(), "{bad_args}");
        }
    }

    #[tokio::test]
    async fn a_result_answers_its_question_once_and_unknown_events_are_a_no_op() {
        let hub = Arc::new(EventHub::default());
        let registration = hub.register("c1");
        let (question, mut answer) = PendingApproval::new("bash", "{}", Some("a".into()));
        registration.hold("e1", question);
        let allow = json!({ "kind": "result", "value": "allowed-once" });
        hub.result(&args(allow.clone())).unwrap();
        assert_eq!(answer.try_recv().unwrap(), ApprovalDecision::Allow);
        // A repeat of the same answer, and one for an event never asked, are no-ops.
        hub.result(&args(allow.clone())).unwrap();
        let unknown = json!({ "clientId": "c1", "eventId": "nope", "outcome": allow });
        hub.result(&unknown).unwrap();
        let gone = json!({ "clientId": "zzz", "eventId": "e1", "outcome": { "kind": "next" } });
        assert_eq!(hub.result(&gone).unwrap_err().code, codes::LOOKUP_NOT_FOUND);
    }

    #[tokio::test]
    async fn dropping_a_registration_denies_what_it_held() {
        let hub = Arc::new(EventHub::default());
        let registration = hub.register("c1");
        let (question, mut answer) = PendingApproval::new("bash", "{}", None);
        registration.hold("e1", question);
        drop(registration);
        assert!(matches!(
            answer.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Closed)
        ));
        let late = args(json!({ "kind": "result", "value": "allowed-once" }));
        assert!(hub.result(&late).is_err(), "the stream is gone");
    }

    #[tokio::test]
    async fn a_replaced_client_id_is_not_unregistered_by_the_old_registration() {
        let hub = Arc::new(EventHub::default());
        let old = hub.register("c1");
        let new = hub.register("c1");
        drop(old);
        let (question, mut answer) = PendingApproval::new("bash", "{}", None);
        new.hold("e1", question);
        let allow = args(json!({ "kind": "result", "value": "allowed-once" }));
        hub.result(&allow).unwrap();
        assert_eq!(answer.try_recv().unwrap(), ApprovalDecision::Allow);
    }

    #[tokio::test]
    async fn sweep_returns_only_questions_nobody_waits_for() {
        let hub = Arc::new(EventHub::default());
        let registration = hub.register("c1");
        let (live, _keep) = PendingApproval::new("bash", "{}", None);
        let (stale, receiver) = PendingApproval::new("bash", "{}", None);
        registration.hold("live", live);
        registration.hold("stale", stale);
        drop(receiver); // the call stopped waiting
        assert_eq!(registration.sweep(), vec!["stale".to_string()]);
        assert!(registration.sweep().is_empty());
    }

    fn two_questions() -> Value {
        json!([{ "id": "a", "question": "?" }, { "id": "b", "question": "?" }])
    }

    fn held_question(hub: &Arc<EventHub>) -> (Registration, tokio::sync::oneshot::Receiver<Reply>) {
        let registration = hub.register("c1");
        let (question, reply) = PendingQuestion::new(two_questions(), Some("a1".into()));
        registration.hold("e1", question);
        (registration, reply)
    }

    fn answer_with(hub: &EventHub, outcome: Value) -> Result<(), Failure> {
        hub.result(&args(outcome))
    }

    #[tokio::test]
    async fn a_complete_batch_answers_the_question_once_in_the_order_given() {
        let hub = Arc::new(EventHub::default());
        let (_registration, mut reply) = held_question(&hub);
        let batch = json!({ "kind": "result", "value": { "answers": [
            { "id": "b", "selected": [], "custom": "free" }, { "id": "a", "selected": ["x"] }
        ]}});
        answer_with(&hub, batch.clone()).unwrap();
        let Reply::Answered(answers) = reply.try_recv().unwrap() else {
            panic!("expected an answer")
        };
        assert_eq!(
            answers.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
            ["b", "a"]
        );
        assert_eq!(answers[0].custom.as_deref(), Some("free"));
        // A repeat finds nothing waiting and is a no-op.
        answer_with(&hub, batch).unwrap();
    }

    #[tokio::test]
    async fn a_batch_that_misses_repeats_or_invents_a_question_is_refused_and_leaves_it_open() {
        let hub = Arc::new(EventHub::default());
        let (_registration, mut reply) = held_question(&hub);
        let item = |id: &str| json!({ "id": id, "selected": [] });
        for answers in [
            json!([]),
            json!([item("a")]),
            json!([item("a"), item("a")]),
            json!([item("a"), item("zzz")]),
            json!([item("a"), item("b"), item("b")]),
            json!([item("a"), { "id": "b", "selected": "x" }]),
            json!([item("a"), { "id": "b", "selected": [], "extra": 1 }]),
            json!([item("a"), { "id": "b", "selected": [], "custom": 3 }]),
        ] {
            let outcome = json!({ "kind": "result", "value": { "answers": answers } });
            let failure = answer_with(&hub, outcome.clone()).unwrap_err();
            assert_eq!(failure.code, codes::INPUT_INVALID, "{outcome}");
        }
        for outcome in [
            json!({ "kind": "result" }),
            json!({ "kind": "result", "value": "a" }),
            json!({ "kind": "result", "value": { "answers": [], "extra": 1 } }),
        ] {
            assert!(answer_with(&hub, outcome).is_err());
        }
        assert!(reply.try_recv().is_err(), "nothing was delivered");
        // The question is still there to be answered correctly.
        let good = json!({ "kind": "result", "value": { "answers": [item("a"), item("b")] } });
        answer_with(&hub, good).unwrap();
        assert!(matches!(reply.try_recv(), Ok(Reply::Answered(_))));
    }

    #[tokio::test]
    async fn next_and_rejected_fail_the_question_with_a_reason() {
        for (outcome, needle) in [
            (json!({ "kind": "next" }), "no user-questions answerer"),
            (
                json!({ "kind": "rejected", "error": { "name": "E", "message": "the user cancelled" } }),
                "the user cancelled",
            ),
            (
                json!({ "kind": "rejected", "error": {} }),
                "the listener rejected",
            ),
        ] {
            let hub = Arc::new(EventHub::default());
            let (_registration, mut reply) = held_question(&hub);
            answer_with(&hub, outcome).unwrap();
            let Reply::Refused(reason) = reply.try_recv().unwrap() else {
                panic!("expected a refusal")
            };
            assert!(reason.contains(needle), "{reason}");
        }
    }
}
