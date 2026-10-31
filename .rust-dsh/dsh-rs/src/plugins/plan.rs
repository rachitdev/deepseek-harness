//! The `plan` plugin: plan mode, the Rust analogue of the TypeScript host's
//! `plan-mode` package.
//!
//! Plan mode is logged as `plan/mode { active }` (log-only, last wins). A
//! user's `/plan` choice made while a turn is open stays pending until the
//! next `agent/step-start`, where it is appended and, when the last request
//! header described the other mode, narrated to the model as a notice. While
//! plan mode is active the deployment's guidance joins the system prompt, so
//! everything the model sees is reconstructable from the log. The
//! `exit_plan_mode` tool presents the plan for review through the
//! `userQuestions` channel and, on approval, leaves plan mode from the next
//! step.
//!
//! Unlike the TypeScript host, a review the user dismisses is reported as a
//! failed tool call (the answerer's refusal) instead of concluding the turn.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cordis::plugin::BoxFuture;
use cordis::Plugin;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::services::SessionView;
use crate::plugins::ask_user::{QuestionBroker, USER_QUESTIONS_SERVICE};
use crate::plugins::{context_message, register_tools, tool_spec_ctx};
use crate::llm::ContentBlock;
use crate::types::{ContextForm, SessionEvent, SessionEventData, ToolExecutionResult};

/// The `planMode` service key.
pub const PLAN_SERVICE: &str = "planMode";

/// The tool that presents a plan for review.
pub const EXIT_TOOL: &str = "exit_plan_mode";

const REVIEW_ID: &str = "plan-review";
const APPROVE_LABEL: &str = "Approve";
const KEEP_PLANNING_LABEL: &str = "Keep planning";

const DEFAULT_SECTION: &str = "You are in plan mode. Stay in plan mode until the user approves your plan through exit_plan_mode or switches the session mode. Imperative language to implement changes means plan the implementation, not execute it. A user's conversational agreement — including an answer confirming something you asked — approves nothing and does not end plan mode; fold the confirmed decision into the plan and submit it through exit_plan_mode.

Explore first. Use non-mutating reads, searches, static analysis, and checks to ground the plan in the actual repository. Do not edit or write files, change configuration, run formatters or code generation that rewrites tracked files, commit, or otherwise carry out the plan. Prefer existing functions and patterns over new machinery.

The tool catalog stays the same across modes for request-cache stability. These plan-mode rules override any later tool description or guidance that suggests using mutation tools; those tools remain listed only to keep the request shape stable. Do not use todo_write to track this planning phase: it tracks implementation after an approved plan, while the plan itself belongs in exit_plan_mode.

Resolve discoverable facts by inspection. Use ask_user_question only for user-owned choices or material ambiguity that inspection cannot answer. Do not ask the user where code lives or how current behavior works when you can find out.

Make the plan decision-complete: state the goal and success criteria; group implementation changes by subsystem; identify public API, schema, and data-flow changes; cover edge cases, failure modes, tests, acceptance criteria, and explicit assumptions. Keep it concise enough to review but detailed enough that another engineer can implement it without making design decisions.

When ready, call exit_plan_mode with the complete plan markdown, starting with a # title. Make exit_plan_mode the only and final tool call in that assistant response: it presents the plan for approval, and implementation begins only in a later step after approval. Do not paste the final plan as a plain reply or ask \"should I proceed?\" through prose or ask_user_question. If review rejects it, incorporate the feedback and present again. If the review channel is unavailable or aborted, stay in plan mode and ask the user to switch modes manually; do not proceed with implementation.";

const EXIT_DESCRIPTION: &str = "Use only in plan mode. Present your plan for the user's review and, on approval, leave plan mode. The user may approve (carry out the plan from your next step) or keep planning — their feedback comes back in the tool result; revise and present again.";

/// Configuration for the `plan` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlanConfig {
    /// The guidance added to the system prompt while plan mode is active.
    pub section: String,
}

impl Default for PlanConfig {
    fn default() -> Self {
        PlanConfig { section: DEFAULT_SECTION.to_string() }
    }
}

/// What a `/plan` choice did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOutcome {
    /// Logged now.
    Committed,
    /// Waits for the next step of the open turn.
    Queued,
    /// An opposite pending choice was cleared; the log already matches.
    Cancelled,
    /// Already in that state.
    Noop,
}

/// A choice waiting for the next step.
#[derive(Debug, Clone, Copy)]
struct Pending {
    active: bool,
    /// Whether the model is told about the switch. False for the exit tool,
    /// whose result already says so.
    narrate: bool,
}

/// The plan state a session log folds to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanState {
    /// Whether the last `plan/mode` says active.
    pub active: bool,
    /// What a finished `/plan` wanted, while it differs from `active`.
    wanted: Option<bool>,
    /// The `/plan` command now running, with what it wants.
    running: Option<(String, bool)>,
    /// `active` as of the last `request/header`.
    active_at_last_header: Option<bool>,
    /// `exit_plan_mode` calls logged without a result yet.
    open_exits: Vec<String>,
    /// An `exit_plan_mode` review was approved and the exit is not logged yet.
    exit_approved: bool,
}

impl PlanState {
    /// Fold the log.
    pub fn fold(events: &[SessionEvent]) -> PlanState {
        events.iter().fold(PlanState::default(), |mut state, event| {
            match &event.data {
                SessionEventData::CommandRun { command_id, name, args } if name == "plan" => {
                    // A bare `/plan` may be logged without arguments; it means "on".
                    let wants_on = args.as_deref().is_none_or(|a| a.trim() != "off");
                    state.running = Some((command_id.clone(), wants_on));
                }
                SessionEventData::CommandDone { command_id, success, .. }
                    if state.running.as_ref().is_some_and(|(id, _)| id == command_id) =>
                {
                    let wanted = state.running.as_ref().is_some_and(|(_, wanted)| *wanted);
                    state.wanted = (*success && wanted != state.active).then_some(wanted);
                    state.running = None;
                }
                SessionEventData::PlanMode { active } => {
                    state.active = *active;
                    state.wanted = None;
                    state.exit_approved = false;
                }
                SessionEventData::ToolCall { name, call_id, .. } if name == EXIT_TOOL => {
                    state.open_exits.push(call_id.clone());
                }
                SessionEventData::ToolResult { message, .. } => {
                    for block in &message.content {
                        if let ContentBlock::ToolResult { tool_call_id, is_error, .. } = block {
                            if let Some(at) = state.open_exits.iter().position(|id| id == tool_call_id) {
                                state.open_exits.remove(at);
                                if state.active && *is_error != Some(true) {
                                    state.exit_approved = true;
                                }
                            }
                        }
                    }
                }
                SessionEventData::RequestHeader { .. } => {
                    state.active_at_last_header = Some(state.active);
                }
                _ => {}
            }
            state
        })
    }

    /// The `plan` projection value.
    pub fn view(&self) -> Value {
        let wanted = self.running.as_ref().map(|(_, wanted)| *wanted).or(self.wanted);
        let pending = wanted.is_some_and(|w| w != self.active) || (self.exit_approved && self.active);
        json!({ "active": self.active, "pending": pending })
    }
}

/// The `plan` projection of a log, as of the events with `seq <= upto` when
/// `upto` is given.
pub fn projection(events: &[SessionEvent], upto: Option<u64>) -> Value {
    let kept: Vec<SessionEvent> = events
        .iter()
        .filter(|event| upto.is_none_or(|seq| event.seq <= seq))
        .cloned()
        .collect();
    PlanState::fold(&kept).view()
}

/// Plan mode for the process: pending choices and the guidance text.
#[derive(Clone)]
pub struct PlanMode {
    section: Arc<str>,
    pending: Arc<Mutex<HashMap<String, Pending>>>,
}

impl PlanMode {
    fn new(section: String) -> Self {
        PlanMode { section: section.into(), pending: Arc::default() }
    }

    /// Whether plan mode is in force in the log (not counting a pending choice).
    pub fn logged_active(session: &dyn SessionView) -> bool {
        PlanState::fold(&session.events()).active
    }

    /// Choose plan mode on or off for a session. In an open turn the choice
    /// waits for the next step; otherwise it is logged now. The whole decision
    /// runs under the pending-choice lock, so it cannot interleave with a step
    /// applying the choice it replaces.
    pub fn set(&self, session: &dyn SessionView, active: bool) -> SetOutcome {
        let mut pending = self.pending.lock().unwrap();
        let state = PlanState::fold(&session.events());
        let logged = state.active;
        let target = Self::intent(&state, pending.get(session.id())).map_or(logged, |p| p.active);
        if active == target {
            return SetOutcome::Noop;
        }
        if session.open_turn().is_some() {
            if logged == active {
                pending.remove(session.id());
                return SetOutcome::Cancelled;
            }
            pending.insert(session.id().to_string(), Pending { active, narrate: true });
            return SetOutcome::Queued;
        }
        pending.remove(session.id());
        if active != logged {
            session.append(SessionEventData::PlanMode { active });
        }
        SetOutcome::Committed
    }

    /// The choice waiting for the next step: the one held in memory, else the
    /// one the log implies (a `/plan` that finished queued, or an approved
    /// plan review), so a restart does not lose it.
    fn intent(state: &PlanState, held: Option<&Pending>) -> Option<Pending> {
        if let Some(held) = held {
            return Some(*held);
        }
        if state.exit_approved && state.active {
            return Some(Pending { active: false, narrate: false });
        }
        state
            .wanted
            .filter(|wanted| *wanted != state.active)
            .map(|active| Pending { active, narrate: true })
    }

    /// Whether a choice waits for the next step.
    pub fn has_pending(&self, session_id: &str) -> bool {
        self.pending.lock().unwrap().contains_key(session_id)
    }

    /// Apply the pending choice (if any) and say what the step should add:
    /// the notice for the model and whether the guidance applies.
    fn at_step_start(&self, session: &dyn SessionView) -> (Option<String>, bool) {
        let mut pending = self.pending.lock().unwrap();
        let state = PlanState::fold(&session.events());
        let Some(choice) = Self::intent(&state, pending.get(session.id())) else {
            return (None, state.active);
        };
        if choice.active != state.active {
            session.append(SessionEventData::PlanMode { active: choice.active });
        }
        pending.remove(session.id());
        let notice = match state.active_at_last_header {
            Some(told) if choice.narrate && told != choice.active => Some(
                if choice.active {
                    "The user switched this session to plan mode."
                } else {
                    "The user switched this session back to the default mode."
                }
                .to_string(),
            ),
            _ => None,
        };
        (notice, choice.active)
    }

    /// The call id of the `exit_plan_mode` call being executed: the oldest
    /// one logged without a result (a batch runs its calls in order).
    fn open_exit_call(session: &dyn SessionView) -> Option<String> {
        PlanState::fold(&session.events()).open_exits.into_iter().next()
    }
}

/// Append `item` to the array under `key` in a step-start decision, creating
/// the array when the chain below did not.
fn push_to(decision: &mut Value, key: &str, item: Value) {
    if !decision.is_object() {
        *decision = json!({});
    }
    if let Some(object) = decision.as_object_mut() {
        let list = object.entry(key.to_string()).or_insert_with(|| json!([]));
        if let Some(list) = list.as_array_mut() {
            list.push(item);
        }
    }
}

/// The plan's `#` heading, which names it: one `#`, whitespace, then text.
fn heading(plan: &str) -> Option<&str> {
    let first = plan.trim_start().lines().next()?;
    let title = first.strip_prefix('#')?;
    (title.starts_with(char::is_whitespace) && !title.trim().is_empty()).then(|| title.trim())
}

/// The `plan` plugin. Requires the `tools` and `agents` services.
pub fn plan_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "plan",
        vec![
            cordis::Injection::new(crate::api::TOOLS_SERVICE.to_string()),
            cordis::Injection::new(crate::api::AGENTS_SERVICE.to_string()),
        ],
        |ctx, config: PlanConfig| async move {
            if config.section.trim().is_empty() {
                return Err(cordis::Error::msg("plan: `section` must not be empty".to_string()));
            }
            let plan = PlanMode::new(config.section);
            ctx.provide(PLAN_SERVICE, plan.clone()).await?;

            let at_step = plan.clone();
            ctx.on("agent/step-start", move |ctx, payload, next| {
                let plan = at_step.clone();
                Box::pin(async move {
                    // Apply the choice first, so listeners later in the chain
                    // already see the mode this step runs in.
                    let session = payload
                        .get("agent")
                        .and_then(Value::as_str)
                        .and_then(|agent| crate::permission::session_of(&ctx, agent));
                    let applied = session.as_ref().map(|session| plan.at_step_start(&**session));
                    let mut decision = next.run(payload).await?;
                    let Some((notice, active)) = applied else {
                        return Ok(decision);
                    };
                    if let Some(text) = notice {
                        let summary = text.clone();
                        push_to(&mut decision, "messages", context_message(
                            "plan-mode",
                            ContextForm::Notice { summary },
                            text,
                        ));
                    }
                    if active {
                        push_to(&mut decision, "system", json!(&*plan.section));
                    }
                    Ok(decision)
                }) as BoxFuture<cordis::Result<Value>>
            })
            .await?;

            let tool_plan = plan.clone();
            let spec = tool_spec_ctx(
                EXIT_TOOL,
                EXIT_DESCRIPTION,
                json!({
                    "type": "object",
                    "properties": {
                        "plan": {
                            "type": "string",
                            "description": "The complete plan, as markdown, starting with a # heading that names it.",
                        }
                    },
                    "required": ["plan"],
                }),
                move |args, run| {
                    let plan = tool_plan.clone();
                    async move { exit_plan_mode(&plan, args, run).await }
                },
            );
            register_tools(&ctx, "plan tools", vec![spec]).await
        },
    )
}

/// The body of `exit_plan_mode`.
async fn exit_plan_mode(
    plan: &PlanMode,
    args: Value,
    run: crate::types::ToolRunContext,
) -> ToolExecutionResult {
    let fail = |code: &'static str, message: &str| ToolExecutionResult::error(code, message);
    let Some(agent) = run.agent_id.as_deref() else {
        return fail("NO_AGENT", &format!("{EXIT_TOOL} requires a calling agent (no session to switch)"));
    };
    let Some(session) = crate::permission::session_of(&run.ctx, agent) else {
        return fail("NO_AGENT", &format!("{EXIT_TOOL} requires a live session to switch"));
    };
    if !PlanMode::logged_active(&*session) {
        return fail("NOT_PLANNING", &format!("{EXIT_TOOL} is only available in plan mode"));
    }
    let text = args.get("plan").and_then(Value::as_str).unwrap_or("");
    if heading(text).is_none() {
        return fail(
            "BAD_ARGS",
            &format!("{EXIT_TOOL} requires a non-empty markdown plan starting with a # heading"),
        );
    }
    let Some(broker) = run.ctx.get::<QuestionBroker>(USER_QUESTIONS_SERVICE) else {
        return fail(
            "NO_PROVIDER",
            "no user-questions channel is available to review the plan; ask the user to switch the session mode instead",
        );
    };
    let mut intent = json!({ "kind": "plan-review", "approve": APPROVE_LABEL });
    if let Some(call_id) = PlanMode::open_exit_call(&*session) {
        intent["callId"] = json!(call_id);
    }
    let questions = json!([{
        "id": REVIEW_ID,
        "header": "Plan review",
        "question": "Approve this plan and leave plan mode?",
        "detail": text,
        "options": [
            { "label": APPROVE_LABEL, "description": "Leave plan mode; the plan is carried out from the next step." },
            { "label": KEEP_PLANNING_LABEL, "description": "Stay in plan mode; feedback goes back to the model." },
        ],
        "intent": intent,
    }]);
    let answers = match broker.ask(questions, Some(agent.to_string()), &run.signal).await {
        Ok(answers) => answers,
        Err(error) => return ToolExecutionResult::error(error.code, error.message),
    };
    let mut reviews = answers.iter().filter(|answer| answer.id == REVIEW_ID);
    let item = reviews.next().filter(|_| reviews.next().is_none());
    let approved = item.is_some_and(|answer| {
        answer.selected == [APPROVE_LABEL.to_string()]
            && answer.custom.as_deref().is_none_or(|text| text.trim().is_empty())
    });
    if !approved {
        let feedback = item.and_then(|answer| answer.custom.as_deref()).unwrap_or("");
        return fail(
            "KEEP_PLANNING",
            &if feedback.is_empty() {
                "The user chose to keep planning; revise the plan and present it again.".to_string()
            } else {
                format!("The user chose to keep planning; their feedback: {feedback}")
            },
        );
    }
    // The guidance stays for the rest of this tool batch; the silent switch is
    // logged at the next step.
    plan.pending
        .lock()
        .unwrap()
        .insert(session.id().to_string(), Pending { active: false, narrate: false });
    ToolExecutionResult::success_text(
        "Plan approved — plan mode exited; carry out the plan starting with your next step.",
        json!({ "approved": true }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(seq: u64, data: SessionEventData) -> SessionEvent {
        SessionEvent { seq, time: 0, data }
    }

    fn run(seq: u64, id: &str, args: &str) -> SessionEvent {
        ev(
            seq,
            SessionEventData::CommandRun {
                command_id: id.into(),
                name: "plan".into(),
                args: Some(args.into()),
            },
        )
    }

    fn done(seq: u64, id: &str, success: bool) -> SessionEvent {
        ev(
            seq,
            SessionEventData::CommandDone {
                command_id: id.into(),
                success,
                text: None,
                source_event_seq: None,
            },
        )
    }

    #[test]
    fn a_running_plan_command_reads_pending_until_the_mode_is_logged() {
        let log = vec![run(0, "c1", "")];
        assert_eq!(PlanState::fold(&log).view(), json!({"active": false, "pending": true}));
        let log = vec![run(0, "c1", ""), done(1, "c1", true)];
        assert_eq!(PlanState::fold(&log).view(), json!({"active": false, "pending": true}));
        let log = vec![
            run(0, "c1", ""),
            done(1, "c1", true),
            ev(2, SessionEventData::PlanMode { active: true }),
        ];
        assert_eq!(PlanState::fold(&log).view(), json!({"active": true, "pending": false}));
    }

    #[test]
    fn a_failed_or_redundant_command_is_not_pending() {
        let log = vec![run(0, "c1", ""), done(1, "c1", false)];
        assert_eq!(PlanState::fold(&log).view(), json!({"active": false, "pending": false}));
        let log = vec![run(0, "c1", "off"), done(1, "c1", true)];
        assert_eq!(PlanState::fold(&log).view(), json!({"active": false, "pending": false}));
    }

    #[test]
    fn the_projection_can_be_read_as_of_an_earlier_event() {
        let log = vec![run(0, "c1", ""), ev(1, SessionEventData::PlanMode { active: true })];
        assert_eq!(projection(&log, Some(0)), json!({"active": false, "pending": true}));
        assert_eq!(projection(&log, None), json!({"active": true, "pending": false}));
    }

    fn exit_call(seq: u64, id: &str) -> SessionEvent {
        ev(
            seq,
            SessionEventData::ToolCall {
                turn: 1,
                step: 1,
                call_id: id.into(),
                name: EXIT_TOOL.into(),
                arguments: "{}".into(),
            },
        )
    }

    fn exit_result(seq: u64, id: &str, is_error: bool) -> SessionEvent {
        ev(
            seq,
            SessionEventData::ToolResult {
                turn: 1,
                step: 1,
                message: crate::types::Message {
                    id: format!("r-{id}"),
                    role: crate::types::Role::User,
                    content: vec![ContentBlock::ToolResult {
                        tool_call_id: id.into(),
                        content: vec![],
                        is_error: is_error.then_some(true),
                    }],
                    source: crate::llm::MessageSource::User { rpc_id: None },
                },
            },
        )
    }

    #[test]
    fn a_bare_plan_command_logged_without_arguments_means_on() {
        let log = vec![ev(
            0,
            SessionEventData::CommandRun { command_id: "c".into(), name: "plan".into(), args: None },
        )];
        assert_eq!(PlanState::fold(&log).view(), json!({"active": false, "pending": true}));
    }

    #[test]
    fn an_approved_review_is_pending_until_the_exit_is_logged() {
        let mut log = vec![ev(0, SessionEventData::PlanMode { active: true }), exit_call(1, "p1")];
        assert_eq!(PlanState::fold(&log).open_exits, ["p1"]);
        assert_eq!(PlanState::fold(&log).view(), json!({"active": true, "pending": false}));
        log.push(exit_result(2, "p1", false));
        let state = PlanState::fold(&log);
        assert!(state.open_exits.is_empty());
        assert_eq!(state.view(), json!({"active": true, "pending": true}));
        // The intent survives a restart because the log implies it.
        let intent = PlanMode::intent(&state, None).unwrap();
        assert!(!intent.active && !intent.narrate);
        log.push(ev(3, SessionEventData::PlanMode { active: false }));
        assert_eq!(PlanState::fold(&log).view(), json!({"active": false, "pending": false}));
    }

    #[test]
    fn a_refused_review_leaves_plan_mode_in_force() {
        let log = vec![
            ev(0, SessionEventData::PlanMode { active: true }),
            exit_call(1, "p1"),
            exit_result(2, "p1", true),
        ];
        assert_eq!(PlanState::fold(&log).view(), json!({"active": true, "pending": false}));
    }

    #[test]
    fn the_review_names_the_oldest_call_without_a_result() {
        let log = vec![
            ev(0, SessionEventData::PlanMode { active: true }),
            exit_call(1, "p1"),
            exit_call(2, "p2"),
            exit_result(3, "p1", true),
        ];
        assert_eq!(PlanState::fold(&log).open_exits, ["p2"]);
    }

    #[test]
    fn a_decision_without_arrays_still_receives_what_a_step_adds() {
        let mut decision = Value::Null;
        push_to(&mut decision, "messages", json!("m"));
        push_to(&mut decision, "system", json!("s"));
        assert_eq!(decision, json!({"messages": ["m"], "system": ["s"]}));
    }

    #[test]
    fn a_plan_is_named_by_its_first_heading() {
        assert_eq!(heading("# Ship it\nbody"), Some("Ship it"));
        assert_eq!(heading("\n  # Sub"), Some("Sub"));
        assert_eq!(heading("## Sub"), None);
        assert_eq!(heading("no heading"), None);
        assert_eq!(heading("#nospace"), None);
        assert_eq!(heading("# "), None);
    }
}
