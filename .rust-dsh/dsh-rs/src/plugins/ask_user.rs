//! The `ask-user` plugin: the `ask_user_question` tool and the broker that
//! carries its questions to a person.
//!
//! The tool pauses the calling agent's turn until the attached frontend (a
//! browser's `$events` stream) returns an answer, then hands the answer back to
//! the model as the tool result. It mirrors the blocking mode of the TypeScript
//! `tool-ask-user`; the timed mode (a question that outlives its foreground wait
//! and is answered later through `userQuestions/answer`) is not ported.
//!
//! The call fails, with the TypeScript error code, when
//!
//! - no frontend is attached (`NO_PROVIDER`): a piped run or an unattended agent
//!   must not hang on a question nobody can see;
//! - the frontend goes away or is replaced with the question open (`NO_PROVIDER`);
//! - the turn is cancelled while the question is open (`ASK_ABORTED`);
//! - the calling agent is a sub-agent (`DELEGATED_CALLER`): only a root agent has
//!   a person to ask, so a child must put its question in its final result;
//! - the request has no questions (`EMPTY_QUESTIONS`).
//!
//! There is no answer deadline: like the TypeScript blocking tool, a question
//! waits until it is answered or the turn stops.

use std::sync::Arc;

use cordis::Plugin;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::oneshot;

use crate::plugins::frontend::{Frontend, Inbox, Refusal};
use crate::plugins::subagent::Lineage;
use crate::plugins::{register_tools, tool_spec_ctx};
use crate::types::{CancelToken, ToolExecutionResult};

/// The `userQuestions` service key (the TypeScript name of the same seam).
pub const USER_QUESTIONS_SERVICE: &str = "userQuestions";

/// The tool's model-facing name.
pub const TOOL_NAME: &str = "ask_user_question";

/// One question's answer, as the TypeScript `AskUserQuestionAnswerItem`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    /// The answered question's id.
    pub id: String,
    /// Selected option labels; may accompany `custom` for a multi-select.
    pub selected: Vec<String>,
    /// Free-text "Other" answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<String>,
}

/// What a person's frontend sent back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// The whole answer batch.
    Answered(Vec<Answer>),
    /// The frontend declined or failed; the text says why.
    Refused(String),
}

/// A call that cannot get an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskError {
    /// The TypeScript `UserQuestionError` code.
    pub code: &'static str,
    pub message: String,
}

impl AskError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        AskError {
            code,
            message: message.into(),
        }
    }
}

/// One question batch waiting for an answer.
pub struct PendingQuestion {
    /// The questions, in the TypeScript wire form (`multiSelect`, not `multi_select`).
    pub questions: Value,
    /// The agent that asked.
    pub agent: Option<String>,
    respond: oneshot::Sender<Reply>,
}

impl PendingQuestion {
    /// A question and the receiver for its reply (for frontends and tests).
    pub fn new(
        questions: Value,
        agent: Option<String>,
    ) -> (PendingQuestion, oneshot::Receiver<Reply>) {
        let (respond, reply) = oneshot::channel();
        (
            PendingQuestion {
                questions,
                agent,
                respond,
            },
            reply,
        )
    }

    /// Send the reply. Dropping a `PendingQuestion` unanswered fails the call.
    pub fn answer(self, reply: Reply) {
        let _ = self.respond.send(reply);
    }

    /// Whether the call is still waiting. False once the turn was cancelled, so
    /// a frontend should withdraw such a question.
    pub fn is_open(&self) -> bool {
        !self.respond.is_closed()
    }
}

/// The frontend's end: question batches arrive here.
pub type QuestionInbox = Inbox<PendingQuestion>;

/// Routes question batches to the attached frontend. Cheap to clone.
#[derive(Clone, Default)]
pub struct QuestionBroker {
    frontend: Frontend<PendingQuestion>,
}

impl QuestionBroker {
    /// Become the frontend that answers questions. A later attach replaces an
    /// earlier one, and calls still waiting on it fail.
    pub fn attach(&self) -> QuestionInbox {
        self.frontend.attach()
    }

    /// Whether a frontend is attached right now.
    pub fn attached(&self) -> bool {
        self.frontend.lease().is_some()
    }

    /// Ask the attached frontend and wait for its reply or for `cancel`.
    pub async fn ask(
        &self,
        questions: Value,
        agent: Option<String>,
        cancel: &CancelToken,
    ) -> Result<Vec<Answer>, AskError> {
        let mut lease = self.frontend.lease().ok_or_else(|| {
            AskError::new(
                "NO_PROVIDER",
                "no user-questions answerer accepted the request \
                 (open the Web GUI, or run `dsh chat` where questions can be shown)",
            )
        })?;
        let (question, reply) = PendingQuestion::new(questions, agent);
        lease.send(question).map_err(|refusal| match refusal {
            Refusal::Full => AskError::new(
                "NO_PROVIDER",
                "the user-questions answerer is not reading its questions",
            ),
            Refusal::Closed => {
                AskError::new("NO_PROVIDER", "the user-questions answerer has gone away")
            }
        })?;
        let gone = || {
            AskError::new(
                "NO_PROVIDER",
                "the user-questions answerer went away before answering",
            )
        };
        let outcome = tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                return Err(AskError::new(
                    "ASK_ABORTED",
                    "ask_user_question was aborted before the user answered",
                ));
            }
            _ = lease.revoked.wait_for(|revoked| *revoked) => return Err(gone()),
            outcome = reply => outcome,
        };
        // An answer is accepted only from the frontend that is still attached.
        if !self.frontend.is_current(&lease) {
            return Err(gone());
        }
        match outcome {
            Ok(Reply::Answered(answers)) => Ok(answers),
            Ok(Reply::Refused(reason)) => Err(AskError::new("NO_PROVIDER", reason)),
            Err(_) => Err(AskError::new(
                "NO_PROVIDER",
                "the user-questions answerer closed without answering",
            )),
        }
    }
}

/// Rename the tool's `multi_select` to the wire name `multiSelect` and check the
/// batch the way the TypeScript tool schema does. `Err` is the model-facing reason.
pub fn wire_questions(arguments: &Value) -> Result<Value, String> {
    let list = arguments
        .get("questions")
        .and_then(Value::as_array)
        .ok_or("`questions` must be an array")?;
    let mut wire = Vec::with_capacity(list.len());
    let mut seen = std::collections::HashSet::new();
    for (index, item) in list.iter().enumerate() {
        let text = |key: &str| item.get(key).and_then(Value::as_str);
        let (Some(id), Some(question)) = (text("id"), text("question")) else {
            return Err(format!(
                "questions[{index}] needs a string `id` and a string `question`"
            ));
        };
        if !seen.insert(id) {
            return Err(format!(
                "questions[{index}].id {id:?} repeats an earlier question; ids must be unique"
            ));
        }
        let mut entry = serde_json::Map::new();
        entry.insert("id".into(), json!(id));
        entry.insert("question".into(), json!(question));
        match item.get("header") {
            None | Some(Value::Null) => {}
            Some(Value::String(header)) => {
                entry.insert("header".into(), json!(header));
            }
            Some(_) => return Err(format!("questions[{index}].header must be a string")),
        }
        match item.get("options") {
            None | Some(Value::Null) => {}
            Some(Value::Array(options)) => {
                let mut wire_options = Vec::with_capacity(options.len());
                for (at, option) in options.iter().enumerate() {
                    let Some(label) = option.get("label").and_then(Value::as_str) else {
                        return Err(format!(
                            "questions[{index}].options[{at}] needs a string `label`"
                        ));
                    };
                    let mut wire_option = serde_json::Map::new();
                    wire_option.insert("label".into(), json!(label));
                    match option.get("description") {
                        None | Some(Value::Null) => {}
                        Some(Value::String(description)) => {
                            wire_option.insert("description".into(), json!(description));
                        }
                        Some(_) => {
                            return Err(format!(
                                "questions[{index}].options[{at}].description must be a string"
                            ))
                        }
                    }
                    wire_options.push(Value::Object(wire_option));
                }
                entry.insert("options".into(), Value::Array(wire_options));
            }
            Some(_) => return Err(format!("questions[{index}].options must be an array")),
        }
        match item.get("multi_select") {
            None | Some(Value::Null) => {}
            Some(Value::Bool(flag)) => {
                entry.insert("multiSelect".into(), json!(flag));
            }
            Some(_) => return Err(format!("questions[{index}].multi_select must be a boolean")),
        }
        wire.push(Value::Object(entry));
    }
    Ok(Value::Array(wire))
}

fn tool_parameters() -> Value {
    json!({
        "type": "object",
        "properties": {
            "questions": {
                "type": "array",
                "description": "Questions to ask the user before continuing.",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Stable id for this question; echoed in the answer." },
                        "question": { "type": "string", "description": "The specific question to ask the user." },
                        "header": { "type": "string", "description": "Optional short heading for the question, such as \"Confirm\" or \"Choose Mode\"." },
                        "options": {
                            "type": "array",
                            "description": "Optional choices to show the user. If you recommend one, put it first and append \"(Recommended)\" to that label.",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "label": { "type": "string", "description": "Short user-facing option label." },
                                    "description": { "type": "string", "description": "One sentence explaining the tradeoff or impact." }
                                },
                                "required": ["label"]
                            }
                        },
                        "multi_select": { "type": "boolean", "description": "Whether the user may select more than one option. Defaults to false." }
                    },
                    "required": ["id", "question"]
                }
            }
        },
        "required": ["questions"]
    })
}

/// Configuration for the `ask-user` plugin: it has no settings.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskUserConfig {}

/// The `ask-user` plugin. Requires the `tools` and `agents` services.
pub fn ask_user_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "ask-user",
        vec![
            cordis::Injection::new(crate::api::TOOLS_SERVICE.to_string()),
            cordis::Injection::new(crate::api::AGENTS_SERVICE.to_string()),
        ],
        |ctx, _config: AskUserConfig| async move {
            let broker = QuestionBroker::default();
            ctx.provide(USER_QUESTIONS_SERVICE, broker.clone()).await?;
            let spec = tool_spec_ctx(
                TOOL_NAME,
                "Ask the user a concise question when you need confirmation, a choice, or missing information before proceeding.",
                tool_parameters(),
                move |args, run| {
                    let broker = broker.clone();
                    async move {
                        if let Some(agent_id) = &run.agent_id {
                            let child = run
                                .ctx
                                .get::<Lineage>(crate::plugins::subagent::LINEAGE_SERVICE)
                                .is_some_and(|lineage| lineage.is_child(agent_id));
                            if child {
                                return ToolExecutionResult::error(
                                    "DELEGATED_CALLER",
                                    "human interaction is unavailable while the calling agent is owned by another live agent; include the unresolved question or decision in the child agent's final result",
                                );
                            }
                        }
                        let questions = match wire_questions(&args) {
                            Ok(questions) => questions,
                            Err(reason) => return ToolExecutionResult::error("BAD_ARGS", reason),
                        };
                        if questions.as_array().is_some_and(Vec::is_empty) {
                            return ToolExecutionResult::error(
                                "EMPTY_QUESTIONS",
                                "ask_user_question requires at least one question",
                            );
                        }
                        match broker.ask(questions, run.agent_id.clone(), &run.signal).await {
                            Ok(answers) => {
                                let value = json!({ "answers": answers });
                                ToolExecutionResult::success_text(value.to_string(), value)
                            }
                            Err(error) => ToolExecutionResult::error(error.code, error.message),
                        }
                    }
                },
            );
            register_tools(&ctx, "ask-user tools", vec![spec]).await
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn batch() -> Value {
        json!([{ "id": "q1", "question": "Which?", "options": [{ "label": "A" }] }])
    }

    #[test]
    fn the_snake_case_flag_becomes_the_wire_name_and_bad_shapes_say_where() {
        let wire = wire_questions(&json!({ "questions": [
            { "id": "a", "question": "?", "header": "H", "multi_select": true,
              "options": [{ "label": "x", "description": "d" }], "extra": 1 }
        ]}))
        .unwrap();
        assert_eq!(
            wire,
            json!([{ "id": "a", "question": "?", "header": "H", "multiSelect": true,
                     "options": [{ "label": "x", "description": "d" }] }])
        );
        for (bad, needle) in [
            (json!({}), "`questions`"),
            (json!({ "questions": [{ "id": "a" }] }), "questions[0]"),
            (
                json!({ "questions": [{ "id": "a", "question": "?", "multi_select": "yes" }] }),
                "multi_select",
            ),
            (
                json!({ "questions": [{ "id": "a", "question": "?", "options": [{}] }] }),
                "options[0]",
            ),
            (
                json!({ "questions": [{ "id": "a", "question": "?", "header": 3 }] }),
                "header",
            ),
        ] {
            let error = wire_questions(&bad).unwrap_err();
            assert!(error.contains(needle), "{bad}: {error}");
        }
    }

    #[tokio::test]
    async fn without_a_frontend_the_call_fails_at_once() {
        let error = QuestionBroker::default()
            .ask(batch(), None, &CancelToken::new())
            .await
            .unwrap_err();
        assert_eq!(error.code, "NO_PROVIDER");
    }

    #[tokio::test]
    async fn a_frontend_answers_and_the_answer_reaches_the_caller() {
        let broker = QuestionBroker::default();
        let mut inbox = broker.attach();
        let asking = {
            let broker = broker.clone();
            tokio::spawn(async move {
                broker
                    .ask(batch(), Some("a1".into()), &CancelToken::new())
                    .await
            })
        };
        let question = inbox.questions.recv().await.unwrap();
        assert_eq!(question.agent.as_deref(), Some("a1"));
        let answers = vec![Answer {
            id: "q1".into(),
            selected: vec!["A".into()],
            custom: None,
        }];
        question.answer(Reply::Answered(answers.clone()));
        assert_eq!(asking.await.unwrap().unwrap(), answers);
    }

    #[tokio::test]
    async fn cancelling_the_turn_aborts_the_question_and_marks_it_stale() {
        let broker = QuestionBroker::default();
        let mut inbox = broker.attach();
        let cancel = CancelToken::new();
        let asking = {
            let (broker, cancel) = (broker.clone(), cancel.clone());
            tokio::spawn(async move { broker.ask(batch(), None, &cancel).await })
        };
        let question = inbox.questions.recv().await.unwrap();
        cancel.cancel();
        assert_eq!(asking.await.unwrap().unwrap_err().code, "ASK_ABORTED");
        assert!(!question.is_open());
    }

    #[tokio::test]
    async fn detaching_or_replacing_the_frontend_fails_the_waiting_call() {
        let broker = QuestionBroker::default();
        let mut inbox = broker.attach();
        let asking = {
            let broker = broker.clone();
            tokio::spawn(async move { broker.ask(batch(), None, &CancelToken::new()).await })
        };
        let question = inbox.questions.recv().await.unwrap();
        let _replacement = broker.attach();
        // The old frontend answers just after being replaced: the answer is refused.
        question.answer(Reply::Answered(Vec::new()));
        let error = tokio::time::timeout(Duration::from_secs(2), asking)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(error.code, "NO_PROVIDER");
        assert!(error.message.contains("went away"), "{}", error.message);
    }

    #[tokio::test]
    async fn a_refusal_and_a_dropped_question_both_fail_the_call() {
        let broker = QuestionBroker::default();
        let mut inbox = broker.attach();
        for refuse in [true, false] {
            let asking = {
                let broker = broker.clone();
                tokio::spawn(async move { broker.ask(batch(), None, &CancelToken::new()).await })
            };
            let question = inbox.questions.recv().await.unwrap();
            if refuse {
                question.answer(Reply::Refused("the user closed the panel".into()));
            } else {
                drop(question);
            }
            let error = asking.await.unwrap().unwrap_err();
            assert_eq!(error.code, "NO_PROVIDER");
            if refuse {
                assert_eq!(error.message, "the user closed the panel");
            }
        }
    }

    #[test]
    fn an_answer_item_rejects_unknown_fields() {
        assert!(
            serde_json::from_value::<Answer>(json!({ "id": "a", "selected": [], "x": 1 })).is_err()
        );
        assert!(serde_json::from_value::<Answer>(
            json!({ "id": "a", "selected": ["b"], "custom": "c" })
        )
        .is_ok());
    }
}
