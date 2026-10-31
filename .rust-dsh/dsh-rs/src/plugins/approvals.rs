//! The `approvals` plugin: a policy on every tool call, and a way for a person
//! to answer it.
//!
//! A `tools/pre-execute` listener sorts each call into **deny**, **ask** or
//! **allow** by tool name (a trailing `*` matches a prefix). Calls that need an
//! answer go to the [`ApprovalBroker`] service, which hands them to whichever
//! frontend is attached (the terminal chat UI attaches itself). The call is
//! denied when
//!
//! - no frontend is attached (piped `--line` mode, `dsh run`, a sub-agent run
//!   unattended): approvals fail closed rather than hang or default to allow;
//! - nobody answers within `timeoutMs`;
//! - the frontend goes away, or is replaced by another, with the question open
//!   (an answer it sends afterwards is ignored);
//! - the frontend's queue is full (it is not reading its questions).
//!
//! "Allow for this run" is remembered per tool name until the process exits; it
//! is not written anywhere.
//!
//! Configuration is checked when the plugin loads: a tool matched by both
//! `allow` and `ask`, or an unknown `default`, is an error.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cordis::Plugin;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::oneshot;

use crate::api::services::BoxFuture;
use crate::plugins::frontend::{Frontend, Inbox, Refusal};

/// The `approvals` service key.
pub const APPROVALS_SERVICE: &str = "approvals";

/// What the policy says about one tool call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Ask,
    Deny,
}

impl Verdict {
    fn parse(text: &str) -> Option<Verdict> {
        match text {
            "allow" => Some(Verdict::Allow),
            "ask" => Some(Verdict::Ask),
            "deny" => Some(Verdict::Deny),
            _ => None,
        }
    }
}

/// A person's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Allow,
    /// Allow this tool, without asking again, until the process exits.
    AllowForRun,
    Deny,
}

/// One question waiting for an answer.
pub struct PendingApproval {
    pub tool: String,
    /// The complete arguments, as JSON text. Never truncated: the person approving
    /// must be able to read everything they are authorising.
    pub arguments: String,
    /// The agent making the call, when known.
    pub agent: Option<String>,
    respond: oneshot::Sender<ApprovalDecision>,
}

impl PendingApproval {
    /// A question and the receiver for its answer (for frontends and tests).
    pub fn new(
        tool: impl Into<String>,
        arguments: impl Into<String>,
        agent: Option<String>,
    ) -> (PendingApproval, oneshot::Receiver<ApprovalDecision>) {
        let (respond, answer) = oneshot::channel();
        let question = PendingApproval {
            tool: tool.into(),
            arguments: arguments.into(),
            agent,
            respond,
        };
        (question, answer)
    }

    /// Send the answer. Dropping a `PendingApproval` unanswered denies the call.
    pub fn answer(self, decision: ApprovalDecision) {
        let _ = self.respond.send(decision);
    }

    /// Whether anyone is still waiting for the answer. It is false once the call
    /// timed out or was cancelled, so a frontend should drop such a question.
    pub fn is_open(&self) -> bool {
        !self.respond.is_closed()
    }
}

/// The frontend's end: questions arrive here. Dropping it detaches the frontend.
pub type ApprovalInbox = Inbox<PendingApproval>;

/// Routes approval questions to the attached frontend. Cheap to clone.
#[derive(Clone, Default)]
pub struct ApprovalBroker {
    frontend: Frontend<PendingApproval>,
}

impl ApprovalBroker {
    /// Become the frontend that answers questions. A later attach replaces an
    /// earlier one: the earlier inbox stops receiving, and calls still waiting on
    /// it are denied.
    pub fn attach(&self) -> ApprovalInbox {
        self.frontend.attach()
    }

    /// Ask the attached frontend. `Err` carries the reason the call is denied.
    async fn ask(
        &self,
        tool: &str,
        arguments: &Value,
        agent: Option<String>,
        timeout: Duration,
    ) -> Result<ApprovalDecision, String> {
        let mut lease = self.frontend.lease().ok_or_else(|| {
            format!(
                "`{tool}` needs approval and no interactive approver is attached \
                 (use `dsh chat` in a terminal, or list the tool under `allow`)"
            )
        })?;
        let (respond, answer) = oneshot::channel();
        let question = PendingApproval {
            tool: tool.to_string(),
            arguments: arguments.to_string(),
            agent,
            respond,
        };
        lease.send(question).map_err(|refusal| match refusal {
            Refusal::Full => {
                format!("`{tool}` needs approval but the approver is not reading its questions")
            }
            Refusal::Closed => format!("`{tool}` needs approval but the approver has gone away"),
        })?;
        let gone = || format!("`{tool}`: the approver went away before answering; denied");
        let outcome = tokio::select! {
            biased;
            _ = lease.revoked.wait_for(|revoked| *revoked) => return Err(gone()),
            outcome = tokio::time::timeout(timeout, answer) => outcome,
        };
        if !self.frontend.is_current(&lease) {
            return Err(gone());
        }
        match outcome {
            Ok(Ok(decision)) => Ok(decision),
            Ok(Err(_)) => Err(format!("`{tool}`: the approver closed without answering")),
            Err(_) => Err(format!(
                "`{tool}`: no answer within {} ms; denied",
                timeout.as_millis()
            )),
        }
    }
}

/// Configuration for the `approvals` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ApprovalsConfig {
    /// Tools that always need an answer.
    pub ask: Vec<String>,
    /// Tools that never do.
    pub allow: Vec<String>,
    /// Tools that are always refused.
    pub deny: Vec<String>,
    /// Verdict for a tool named in none of the lists: `allow`, `ask` or `deny`.
    pub default: String,
    /// How long a question waits for an answer, in milliseconds.
    pub timeout_ms: u64,
}

impl Default for ApprovalsConfig {
    fn default() -> Self {
        ApprovalsConfig {
            ask: ["bash", "write_file", "edit_file", "dsh_ecosystem_run"]
                .map(String::from)
                .to_vec(),
            allow: Vec::new(),
            deny: Vec::new(),
            default: "allow".to_string(),
            timeout_ms: 120_000,
        }
    }
}

/// A validated policy.
#[derive(Debug, Clone)]
pub struct Policy {
    ask: Vec<String>,
    allow: Vec<String>,
    deny: Vec<String>,
    default: Verdict,
}

fn matches(pattern: &str, tool: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => tool.starts_with(prefix),
        None => pattern == tool,
    }
}

impl Policy {
    /// Check `config` and build the policy.
    pub fn from_config(config: &ApprovalsConfig) -> Result<Policy, String> {
        let default = Verdict::parse(&config.default).ok_or_else(|| {
            format!(
                "approvals: unknown default \"{}\" (use allow, ask or deny)",
                config.default
            )
        })?;
        for (list, name) in [
            (&config.ask, "ask"),
            (&config.allow, "allow"),
            (&config.deny, "deny"),
        ] {
            if let Some(empty) = list.iter().find(|p| p.trim().is_empty()) {
                return Err(format!(
                    "approvals: `{name}` contains an empty pattern ({empty:?})"
                ));
            }
        }
        // A tool matched by two lists has no single right answer; refuse to guess.
        for (a_name, a, b_name, b) in [
            ("allow", &config.allow, "ask", &config.ask),
            ("allow", &config.allow, "deny", &config.deny),
            ("ask", &config.ask, "deny", &config.deny),
        ] {
            for left in a {
                if let Some(right) = b.iter().find(|r| matches(left, r) || matches(r, left)) {
                    return Err(format!(
                        "approvals: \"{left}\" in `{a_name}` overlaps \"{right}\" in `{b_name}`"
                    ));
                }
            }
        }
        Ok(Policy {
            ask: config.ask.clone(),
            allow: config.allow.clone(),
            deny: config.deny.clone(),
            default,
        })
    }

    /// The verdict for one tool.
    pub fn verdict(&self, tool: &str) -> Verdict {
        let hit = |list: &[String]| list.iter().any(|p| matches(p, tool));
        if hit(&self.deny) {
            Verdict::Deny
        } else if hit(&self.ask) {
            Verdict::Ask
        } else if hit(&self.allow) {
            Verdict::Allow
        } else {
            self.default
        }
    }
}

/// The `approvals` plugin. Requires the `tools` service.
pub fn approvals_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "approvals",
        vec![cordis::Injection::new(
            crate::api::TOOLS_SERVICE.to_string(),
        )],
        |ctx, config: ApprovalsConfig| async move {
            let policy = Arc::new(Policy::from_config(&config).map_err(cordis::Error::msg)?);
            if config.timeout_ms == 0 {
                return Err(cordis::Error::msg(
                    "approvals: timeoutMs must be above 0".to_string(),
                ));
            }
            let timeout = Duration::from_millis(config.timeout_ms);
            let broker = ApprovalBroker::default();
            let granted: Arc<Mutex<HashSet<String>>> = Arc::default();
            ctx.provide(APPROVALS_SERVICE, broker.clone()).await?;

            ctx.on("tools/pre-execute", move |ctx, payload, next| {
                let policy = Arc::clone(&policy);
                let broker = broker.clone();
                let granted = Arc::clone(&granted);
                Box::pin(async move {
                    let tool = payload
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    // The `permission` plugin's own listener refuses a call the
                    // sandbox forbids; checking here too keeps a person from
                    // being asked about a call that would be refused anyway.
                    if let Some(reason) = crate::plugins::permission::sandbox_verdict(&ctx, &payload) {
                        return Ok(json!({ "kind": "deny", "reason": reason }));
                    }
                    let verdict = match policy.verdict(&tool) {
                        Verdict::Ask if prompts_off(&ctx, &payload) => Verdict::Allow,
                        other => other,
                    };
                    match verdict {
                        Verdict::Allow => {}
                        Verdict::Deny => {
                            return Ok(json!({
                                "kind": "deny",
                                "reason": format!("`{tool}` is denied by the approvals policy"),
                            }));
                        }
                        Verdict::Ask => {
                            if !granted.lock().unwrap().contains(&tool) {
                                let arguments =
                                    payload.get("arguments").cloned().unwrap_or(Value::Null);
                                let agent = payload
                                    .get("agent")
                                    .and_then(Value::as_str)
                                    .map(String::from);
                                match broker.ask(&tool, &arguments, agent, timeout).await {
                                    Ok(ApprovalDecision::Allow) => {}
                                    Ok(ApprovalDecision::AllowForRun) => {
                                        granted.lock().unwrap().insert(tool.clone());
                                    }
                                    Ok(ApprovalDecision::Deny) => {
                                        return Ok(json!({
                                            "kind": "deny",
                                            "reason": format!("`{tool}` was denied by the user"),
                                        }));
                                    }
                                    Err(reason) => {
                                        return Ok(json!({ "kind": "deny", "reason": reason }));
                                    }
                                }
                            }
                        }
                    }
                    next.run(payload).await
                }) as BoxFuture<cordis::Result<Value>>
            })
            .await?;
            Ok(())
        },
    )
}

/// Whether the calling session's approval policy is `never`: no question is
/// shown, whatever the profile's ask list says.
fn prompts_off(ctx: &cordis::Context, payload: &Value) -> bool {
    let Some(table) = crate::permission::provided(ctx) else { return false };
    let Some(agent) = payload.get("agent").and_then(Value::as_str) else { return false };
    let Some(session) = crate::permission::session_of(ctx, agent) else { return false };
    let knobs = crate::permission::Permissions::knobs(&session.events());
    table.effective(&knobs).1 == "never"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(
        ask: &[&str],
        allow: &[&str],
        deny: &[&str],
        default: &str,
    ) -> Result<Policy, String> {
        Policy::from_config(&ApprovalsConfig {
            ask: ask.iter().map(|s| s.to_string()).collect(),
            allow: allow.iter().map(|s| s.to_string()).collect(),
            deny: deny.iter().map(|s| s.to_string()).collect(),
            default: default.to_string(),
            timeout_ms: 1000,
        })
    }

    #[test]
    fn verdicts_follow_the_lists_and_prefix_wildcards() {
        let p = policy(
            &["bash", "mcp__shell__*"],
            &["read_file"],
            &["rm_rf"],
            "allow",
        )
        .unwrap();
        assert_eq!(p.verdict("bash"), Verdict::Ask);
        assert_eq!(p.verdict("mcp__shell__run"), Verdict::Ask);
        assert_eq!(p.verdict("read_file"), Verdict::Allow);
        assert_eq!(p.verdict("rm_rf"), Verdict::Deny);
        assert_eq!(p.verdict("something_else"), Verdict::Allow);
        let strict = policy(&[], &["read_file"], &[], "deny").unwrap();
        assert_eq!(strict.verdict("something_else"), Verdict::Deny);
    }

    #[test]
    fn contradictory_or_unknown_config_is_rejected() {
        assert!(policy(&["bash"], &["bash"], &[], "allow")
            .unwrap_err()
            .contains("overlaps"));
        assert!(policy(&["mcp__*"], &["mcp__jev__x"], &[], "allow")
            .unwrap_err()
            .contains("overlaps"));
        assert!(policy(&[], &[], &[], "sometimes")
            .unwrap_err()
            .contains("unknown default"));
        assert!(policy(&[""], &[], &[], "allow")
            .unwrap_err()
            .contains("empty"));
    }

    #[tokio::test]
    async fn no_attached_approver_denies_with_a_reason() {
        let broker = ApprovalBroker::default();
        let err = broker
            .ask("bash", &json!({}), None, Duration::from_secs(1))
            .await
            .unwrap_err();
        assert!(err.contains("no interactive approver"), "{err}");
    }

    #[tokio::test]
    async fn an_attached_approver_answers_and_detaches_on_drop() {
        let broker = ApprovalBroker::default();
        let mut inbox = broker.attach();
        let asking = {
            let broker = broker.clone();
            tokio::spawn(async move {
                broker
                    .ask(
                        "bash",
                        &json!({"command": "ls"}),
                        Some("a1".into()),
                        Duration::from_secs(5),
                    )
                    .await
            })
        };
        let question = inbox.questions.recv().await.unwrap();
        assert_eq!(
            (question.tool.as_str(), question.agent.as_deref()),
            ("bash", Some("a1"))
        );
        assert!(question.arguments.contains("ls"));
        question.answer(ApprovalDecision::AllowForRun);
        assert_eq!(
            asking.await.unwrap().unwrap(),
            ApprovalDecision::AllowForRun
        );

        drop(inbox);
        assert!(broker
            .ask("bash", &json!({}), None, Duration::from_secs(1))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn silence_and_a_dropped_question_both_deny() {
        let broker = ApprovalBroker::default();
        let mut inbox = broker.attach();
        let err = broker
            .ask("bash", &json!({}), None, Duration::from_millis(50))
            .await
            .unwrap_err();
        assert!(err.contains("no answer"), "{err}");
        let _ = inbox.questions.try_recv(); // the unanswered question from above

        // The inbox stays attached; only the question is dropped.
        let dropper = tokio::spawn(async move {
            let question = inbox.questions.recv().await.unwrap();
            drop(question);
            inbox
        });
        let err = broker
            .ask("bash", &json!({}), None, Duration::from_secs(5))
            .await
            .unwrap_err();
        assert!(err.contains("closed without answering"), "{err}");
        let _inbox = dropper.await.unwrap();
    }

    #[tokio::test]
    async fn a_newer_attach_replaces_an_older_one_and_the_old_drop_does_not_detach_it() {
        let broker = ApprovalBroker::default();
        let first = broker.attach();
        let mut second = broker.attach();
        drop(first);
        let asking = {
            let broker = broker.clone();
            tokio::spawn(async move {
                broker
                    .ask("t", &json!({}), None, Duration::from_secs(5))
                    .await
            })
        };
        second
            .questions
            .recv()
            .await
            .unwrap()
            .answer(ApprovalDecision::Deny);
        assert_eq!(asking.await.unwrap().unwrap(), ApprovalDecision::Deny);
    }

    #[tokio::test]
    async fn detaching_denies_a_question_the_frontend_already_took() {
        let broker = ApprovalBroker::default();
        let mut inbox = broker.attach();
        let asking = {
            let broker = broker.clone();
            tokio::spawn(async move {
                broker
                    .ask("bash", &json!({}), None, Duration::from_secs(5))
                    .await
            })
        };
        let question = inbox.questions.recv().await.unwrap();
        drop(inbox); // the frontend goes away holding the question
        let err = asking.await.unwrap().unwrap_err();
        assert!(err.contains("went away"), "{err}");
        question.answer(ApprovalDecision::Allow); // too late; must change nothing
    }

    #[tokio::test]
    async fn replacing_the_frontend_denies_questions_waiting_on_the_old_one() {
        let broker = ApprovalBroker::default();
        let mut old = broker.attach();
        let asking = {
            let broker = broker.clone();
            tokio::spawn(async move {
                broker
                    .ask("bash", &json!({}), None, Duration::from_secs(5))
                    .await
            })
        };
        let question = old.questions.recv().await.unwrap();
        let _new = broker.attach();
        question.answer(ApprovalDecision::Allow);
        let err = asking.await.unwrap().unwrap_err();
        assert!(err.contains("went away"), "{err}");
    }

    #[tokio::test]
    async fn a_timed_out_question_reports_closed_and_arguments_are_complete() {
        let broker = ApprovalBroker::default();
        let mut inbox = broker.attach();
        let long = format!("{}; rm -rf /tmp/x", "echo ok ".repeat(200));
        let err = broker
            .ask(
                "bash",
                &json!({ "command": long }),
                None,
                Duration::from_millis(30),
            )
            .await
            .unwrap_err();
        assert!(
            err.contains("30 ms"),
            "the configured timeout is honoured: {err}"
        );
        let question = inbox.questions.recv().await.unwrap();
        assert!(
            !question.is_open(),
            "a frontend can tell the question is stale"
        );
        assert!(
            question.arguments.contains("rm -rf /tmp/x"),
            "nothing was cut from the arguments"
        );
    }

    #[tokio::test]
    async fn a_frontend_that_never_reads_cannot_grow_the_queue_without_bound() {
        let broker = ApprovalBroker::default();
        let _inbox = broker.attach();
        let mut denied = None;
        for _ in 0..(crate::plugins::frontend::QUEUE_CAPACITY + 5) {
            let broker = broker.clone();
            // Each call times out almost at once, but its question stays queued unread.
            if let Err(e) = broker
                .ask("t", &json!({}), None, Duration::from_millis(1))
                .await
            {
                if e.contains("not reading") {
                    denied = Some(e);
                    break;
                }
            }
        }
        assert!(
            denied.is_some(),
            "the queue filled and calls were denied rather than queued forever"
        );
    }

    #[tokio::test]
    async fn an_answer_that_lands_before_a_replacement_is_still_denied_if_the_call_wakes_after_it()
    {
        let broker = ApprovalBroker::default();
        let mut old = broker.attach();
        let asking = {
            let broker = broker.clone();
            tokio::spawn(async move {
                broker
                    .ask("bash", &json!({}), None, Duration::from_secs(5))
                    .await
            })
        };
        let question = old.questions.recv().await.unwrap();
        // Same task, no await in between: the waiting call has not run since the answer.
        question.answer(ApprovalDecision::Allow);
        let _new = broker.attach();
        let err = asking.await.unwrap().unwrap_err();
        assert!(err.contains("went away"), "{err}");
    }
}
