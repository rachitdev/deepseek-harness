//! The goal endpoints: one durable objective per session.
//!
//! The TypeScript host serves these from its goal registry: a goal has an
//! identity, a revision, a phase and a round budget, and every mutation is
//! recorded as a `goal/change` row holding the whole post-mutation state. The
//! wire keeps the same names: the session arrives as `agentId` (a lookup
//! parameter's wire name is its key plus `Id`), the expected revision as
//! `ref`, the change as `request`.
//!
//! Activation (whether this process may continue an active goal on its own) is
//! process-local and never logged, as in TypeScript. This build has no goal
//! round driver, so an armed goal records the permission but nothing consumes
//! it and `roundsStarted` stays at the value the log carries.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cordis::Context;
use serde_json::{json, Value};

use super::router::{boundary, check_descriptor, session_not_found, Router};
use super::wire::{codes, Failure, RpcResult};
use crate::api::services::{AgentRegistryService, SessionPersistenceApi};
use crate::types::{
    now_ms, GoalChange, GoalOperation, GoalPhase, GoalRef, GoalSnapshot, SessionEvent,
    SessionEventData,
};

/// Whether this process may automatically continue an active goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Activation {
    Armed,
    Disarmed,
}

impl Activation {
    fn as_str(self) -> &'static str {
        match self {
            Activation::Armed => "armed",
            Activation::Disarmed => "disarmed",
        }
    }
}

/// Process-local goal state. The lock also serializes mutations, so a
/// revision check and the append that follows it cannot interleave with
/// another mutation of the same session. The gui plugin builds one router for
/// the process, so the lock and the activation map are shared by every caller
/// that can mutate a goal.
#[derive(Default)]
pub struct GoalRuntime {
    activation: Mutex<HashMap<String, Activation>>,
}

/// The current goal and the counters the log carries for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CurrentGoal {
    goal: GoalSnapshot,
    rounds_started: u64,
    created_at: u64,
    updated_at: u64,
}

/// The goal endpoints this module serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Get,
    Create,
    Edit,
    Pause,
    Resume,
    Complete,
    Clear,
}

impl Op {
    const ALL: [Op; 7] = [
        Op::Get,
        Op::Create,
        Op::Edit,
        Op::Pause,
        Op::Resume,
        Op::Complete,
        Op::Clear,
    ];

    /// The endpoint name on the wire.
    fn endpoint(self) -> &'static str {
        match self {
            Op::Get => "goals/get",
            Op::Create => "goals/create",
            Op::Edit => "goals/edit",
            Op::Pause => "goals/pause",
            Op::Resume => "goals/resume",
            Op::Complete => "goals/complete",
            Op::Clear => "goals/clear",
        }
    }

    /// The wire fields the endpoint takes, in descriptor order.
    fn fields(self) -> &'static [&'static str] {
        match self {
            Op::Get => &["agentId"],
            Op::Create => &["agentId", "request"],
            Op::Edit => &["agentId", "ref", "request"],
            Op::Pause | Op::Resume | Op::Complete | Op::Clear => &["agentId", "ref"],
        }
    }
}

/// Serve one goal endpoint.
///
/// Returns `None` when the method is not one of them, so the router can keep
/// looking.
pub(super) fn dispatch(
    router: &Router,
    ctx: &Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    let op = Op::ALL.into_iter().find(|op| op.endpoint() == method)?;
    Some(serve(router, ctx, op, payload).unwrap_or_else(RpcResult::failed))
}

fn serve(router: &Router, ctx: &Context, op: Op, payload: &Value) -> Result<RpcResult, Failure> {
    let endpoint = op.endpoint();
    let fields = op.fields();
    check_descriptor(endpoint, fields, payload)?;
    let session_id = payload["agentId"]
        .as_str()
        .ok_or_else(|| boundary(endpoint, "agentId"))?;
    let expected = if fields.contains(&"ref") {
        Some(parse_ref(endpoint, &payload["ref"])?)
    } else {
        None
    };
    let request = if fields.contains(&"request") {
        Some(&payload["request"])
    } else {
        None
    };
    let change = match request {
        Some(request) => Some(parse_request(endpoint, request, op == Op::Create)?),
        None => None,
    };
    apply(router, ctx, op, session_id, expected, change)
}

/// Run one goal operation on a session: the part of an endpoint that comes
/// after its arguments are read. `/goal` shares it, so both surfaces take the
/// same lock, write the same log row and keep the same activation.
fn apply(
    router: &Router,
    ctx: &Context,
    op: Op,
    session_id: &str,
    expected: Option<ExpectedRef>,
    change: Option<Change>,
) -> Result<RpcResult, Failure> {
    let endpoint = op.endpoint();
    let agents = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .ok_or_else(|| {
            Failure::at(
                codes::SERVICE_UNAVAILABLE,
                "the agents service is not wired into this build",
                endpoint,
            )
        })?;
    let Some(agent) = agents.get(session_id) else {
        return Ok(session_not_found(session_id));
    };
    let session = agent.session();
    let mut activation = router.goals.activation.lock().unwrap();
    let current = current_goal(&session.events());
    let armed = activation.get(session_id).copied().unwrap_or(Activation::Disarmed);
    let default_rounds = router.config.default_max_goal_rounds;
    // With a persistence backend, a goal change is acknowledged only once the
    // log holds it, so a crash right after the reply cannot lose it. A session
    // without a backend is memory-only by configuration and has nothing to
    // write.
    let persistence = ctx.get::<Arc<dyn SessionPersistenceApi>>(
        crate::api::services::SESSION_PERSISTENCE_SERVICE,
    );
    let commit = |change: GoalChange| -> Result<(), Failure> {
        session.append(SessionEventData::GoalChange { change });
        match &persistence {
            Some(backend) => backend.flush(session_id).map_err(|err| {
                Failure::at(
                    codes::INTERNAL,
                    format!("the goal change is in memory but the session log write failed: {err}"),
                    endpoint,
                )
            }),
            None => Ok(()),
        }
    };
    let result = match op {
        Op::Get => match current {
            Some(current) => RpcResult::ok(view(&current, armed)),
            None => RpcResult::Empty,
        },
        Op::Create => {
            let change = change.unwrap_or_default();
            let objective = resolve_objective(change.objective.as_deref().unwrap_or(""))?;
            let max = resolve_rounds(change.max_goal_rounds.unwrap_or(Value::from(default_rounds)))?;
            if let Some(existing) = current.as_ref().filter(|c| c.goal.phase != GoalPhase::Complete) {
                return Err(domain(format!(
                    "goal \"{}\" already exists with phase \"{}\"",
                    existing.goal.id,
                    existing.goal.phase.as_str()
                )));
            }
            let now = now_ms();
            let goal = GoalSnapshot {
                id: new_goal_id(),
                revision: 1,
                objective,
                phase: GoalPhase::Active,
                blocked_reason: None,
                max_goal_rounds: max,
            };
            let created = CurrentGoal { goal, rounds_started: 0, created_at: now, updated_at: now };
            commit(snapshot(GoalOperation::Create, &created))?;
            activation.insert(session_id.to_string(), Activation::Armed);
            RpcResult::ok(json!({ "ref": { "id": created.goal.id, "revision": 1 } }))
        }
        Op::Clear => {
            let current = expect_current(current, expected.as_ref())?;
            let tombstone = GoalRef {
                id: current.goal.id.clone(),
                revision: current.goal.revision + 1,
            };
            commit(GoalChange::Clear {
                cleared: tombstone.clone(),
                cleared_at: next_time(&current),
            })?;
            activation.remove(session_id);
            RpcResult::ok(json!({ "id": tombstone.id, "revision": tombstone.revision }))
        }
        Op::Edit | Op::Pause | Op::Resume | Op::Complete => {
            let current = expect_current(current, expected.as_ref())?;
            let (operation, next, after) =
                transition(op, &current, change.unwrap_or_default(), armed)?;
            let next = CurrentGoal { updated_at: next_time(&current), goal: next, ..current };
            commit(snapshot(operation, &next))?;
            activation.insert(session_id.to_string(), after);
            RpcResult::ok(view(&next, after))
        }
    };
    Ok(result)
}

/// What `/goal` asks of the goal domain.
pub(super) enum Command {
    Create(String),
    Edit(GoalRef, String),
    Pause(GoalRef),
    Resume(GoalRef),
    Clear(GoalRef),
}

/// A goal as `/goal` renders it.
pub(super) struct View {
    pub goal: GoalSnapshot,
    pub rounds_started: u64,
    pub activation: &'static str,
}

/// The session's current goal with this process's activation, for rendering.
pub(super) fn current_view(router: &Router, session_id: &str, events: &[SessionEvent]) -> Option<View> {
    let current = current_goal(events)?;
    let armed = router
        .goals
        .activation
        .lock()
        .unwrap()
        .get(session_id)
        .copied()
        .unwrap_or(Activation::Disarmed);
    Some(View {
        goal: current.goal,
        rounds_started: current.rounds_started,
        activation: armed.as_str(),
    })
}

/// Apply one `/goal` operation through the same path as the `goals/*`
/// endpoints. A failure whose `details` are empty is the goal domain refusing
/// the operation (stale ref, wrong phase, blank objective); any other failure
/// is infrastructure, such as the log write.
pub(super) fn run(
    router: &Router,
    ctx: &Context,
    session_id: &str,
    command: Command,
) -> Result<(), Failure> {
    let expect = |goal: GoalRef| ExpectedRef { id: goal.id, revision: goal.revision as f64 };
    let (op, expected, change) = match command {
        Command::Create(objective) => (
            Op::Create,
            None,
            Some(Change { objective: Some(objective), max_goal_rounds: None }),
        ),
        Command::Edit(goal, objective) => (
            Op::Edit,
            Some(expect(goal)),
            Some(Change { objective: Some(objective), max_goal_rounds: None }),
        ),
        Command::Pause(goal) => (Op::Pause, Some(expect(goal)), None),
        Command::Resume(goal) => (Op::Resume, Some(expect(goal)), None),
        Command::Clear(goal) => (Op::Clear, Some(expect(goal)), None),
    };
    match apply(router, ctx, op, session_id, expected, change)? {
        RpcResult::Failed(failure) => Err(failure),
        _ => Ok(()),
    }
}

/// Whether a failure is the goal domain refusing an operation.
pub(super) fn is_domain_refusal(failure: &Failure) -> bool {
    failure.code == codes::INTERNAL && failure.details == json!({})
}

/// A request's replacement fields, checked for type but not yet for content.
#[derive(Default)]
struct Change {
    objective: Option<String>,
    max_goal_rounds: Option<Value>,
}

/// Decide one phase transition or edit: the operation to record, the next
/// snapshot and the activation that follows.
fn transition(
    op: Op,
    current: &CurrentGoal,
    change: Change,
    armed: Activation,
) -> Result<(GoalOperation, GoalSnapshot, Activation), Failure> {
    let goal = &current.goal;
    let phase_change = |operation: GoalOperation,
                        allowed: &[GoalPhase],
                        to: GoalPhase|
     -> Result<GoalSnapshot, Failure> {
        if !allowed.contains(&goal.phase) {
            let names: Vec<_> = allowed.iter().map(|p| p.as_str()).collect();
            return Err(domain(format!(
                "cannot {} goal \"{}\" from phase \"{}\"; expected {}",
                operation.as_str(),
                goal.id,
                goal.phase.as_str(),
                names.join(" or ")
            )));
        }
        Ok(GoalSnapshot {
            revision: goal.revision + 1,
            phase: to,
            blocked_reason: None,
            ..goal.clone()
        })
    };
    use GoalPhase::*;
    match op {
        Op::Edit => {
            if change.objective.is_none() && change.max_goal_rounds.is_none() {
                return Err(domain("goal edit requires objective and/or maxGoalRounds"));
            }
            let mut next = GoalSnapshot { revision: goal.revision + 1, ..goal.clone() };
            if let Some(objective) = &change.objective {
                next.objective = resolve_objective(objective)?;
            }
            if let Some(rounds) = change.max_goal_rounds {
                next.max_goal_rounds = resolve_rounds(rounds)?;
            }
            Ok((GoalOperation::Edit, next, armed))
        }
        Op::Pause => Ok((
            GoalOperation::Pause,
            phase_change(GoalOperation::Pause, &[Active], Paused)?,
            Activation::Disarmed,
        )),
        Op::Complete => Ok((
            GoalOperation::Complete,
            phase_change(GoalOperation::Complete, &[Active, Paused, Blocked], Complete)?,
            Activation::Disarmed,
        )),
        Op::Resume => {
            let next = phase_change(GoalOperation::Resume, &[Active, Paused, Blocked], Active)?;
            if goal.phase == Active && armed == Activation::Armed {
                return Err(domain(format!(
                    "goal \"{}\" is already active and armed",
                    goal.id
                )));
            }
            if current.rounds_started >= goal.max_goal_rounds {
                return Err(domain(format!(
                    "goal \"{}\" exhausted {} goal rounds; increase maxGoalRounds before resuming",
                    goal.id, goal.max_goal_rounds
                )));
            }
            Ok((GoalOperation::Resume, next, Activation::Armed))
        }
        Op::Get | Op::Create | Op::Clear => {
            unreachable!("{} is not a transition of an existing goal", op.endpoint())
        }
    }
}

/// The goal a log currently holds: the latest change, or none after a clear
/// or before the first create.
pub(super) fn current_goal(events: &[SessionEvent]) -> Option<CurrentGoal> {
    let mut current = None;
    for event in events {
        if let SessionEventData::GoalChange { change } = &event.data {
            current = match change {
                GoalChange::Snapshot { goal, rounds_started, created_at, updated_at, .. } => {
                    Some(CurrentGoal {
                        goal: goal.clone(),
                        rounds_started: *rounds_started,
                        created_at: *created_at,
                        updated_at: *updated_at,
                    })
                }
                GoalChange::Clear { .. } => None,
            };
        }
    }
    current
}

/// The `goal` projection value: the durable goal with its counters, or null.
/// Activation is process-local, so it is not part of the projection.
pub(super) fn projection(events: &[SessionEvent]) -> Value {
    projection_as_of(events, u64::MAX)
}

/// The projection as the log stood after event `seq`, so a push for one change
/// reports that change even when later ones are already appended.
pub(super) fn projection_as_of(events: &[SessionEvent], seq: u64) -> Value {
    let end = events.partition_point(|event| event.seq <= seq);
    match current_goal(&events[..end]) {
        Some(current) => json!({
            "goal": current.goal,
            "roundsStarted": current.rounds_started,
            "createdAt": current.created_at,
            "updatedAt": current.updated_at,
        }),
        None => Value::Null,
    }
}

/// The live view `goals/get` and the mutations answer with.
fn view(current: &CurrentGoal, activation: Activation) -> Value {
    let mut value = serde_json::to_value(&current.goal).unwrap_or(Value::Null);
    value["roundsStarted"] = json!(current.rounds_started);
    value["createdAt"] = json!(current.created_at);
    value["updatedAt"] = json!(current.updated_at);
    value["activation"] = json!(activation.as_str());
    value
}

fn snapshot(operation: GoalOperation, current: &CurrentGoal) -> GoalChange {
    GoalChange::Snapshot {
        operation,
        goal: current.goal.clone(),
        rounds_started: current.rounds_started,
        created_at: current.created_at,
        updated_at: current.updated_at,
    }
}

/// A mutation's timestamp, held at or after the goal's last one so a clock
/// that stepped back cannot make `updatedAt` precede `createdAt`.
fn next_time(current: &CurrentGoal) -> u64 {
    now_ms().max(current.updated_at)
}

fn expect_current(
    current: Option<CurrentGoal>,
    expected: Option<&ExpectedRef>,
) -> Result<CurrentGoal, Failure> {
    let current = current.ok_or_else(|| domain("no current goal"))?;
    let expected = expected.expect("a goal mutation names the ref it expects");
    if expected.id != current.goal.id || expected.revision != current.goal.revision as f64 {
        return Err(domain(format!(
            "stale goal ref \"{}\" revision {}; current is \"{}\" revision {}",
            expected.id,
            js_number(expected.revision),
            current.goal.id,
            current.goal.revision
        )));
    }
    Ok(current)
}

fn resolve_objective(value: &str) -> Result<String, Failure> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(domain("goal objective must be a non-empty string"));
    }
    Ok(trimmed.to_string())
}

/// A positive integer a JavaScript client reads back exactly.
fn resolve_rounds(value: Value) -> Result<u64, Failure> {
    let rounds = value
        .as_f64()
        .filter(|n| n.fract() == 0.0 && *n >= 1.0 && *n <= super::router::MAX_SAFE_INTEGER as f64);
    match rounds {
        Some(n) => Ok(n as u64),
        None => Err(domain("maxGoalRounds must be a positive safe integer")),
    }
}

/// `goal-` and a random version-4 UUID, the form TypeScript mints.
fn new_goal_id() -> String {
    format!("goal-{}", super::plugin::uuid_v4())
}

/// A rejection the goal domain raises; the TypeScript host reports these as
/// internal gateway errors with the domain message.
fn domain(message: impl Into<String>) -> Failure {
    Failure {
        code: codes::INTERNAL.to_string(),
        message: message.into(),
        details: json!({}),
    }
}

/// The revision a caller expects. Any JSON number passes the wire check, as in
/// TypeScript; one that is not the current revision is a stale ref.
struct ExpectedRef {
    id: String,
    revision: f64,
}

fn parse_ref(endpoint: &str, value: &Value) -> Result<ExpectedRef, Failure> {
    let id = value.get("id").and_then(Value::as_str);
    let revision = value.get("revision").and_then(Value::as_f64);
    match (id, revision) {
        (Some(id), Some(revision)) => Ok(ExpectedRef { id: id.to_string(), revision }),
        _ => Err(boundary(endpoint, "ref")),
    }
}

/// A number the way JavaScript prints it in a message: no negative zero, and
/// exponent form below 1e-6 and from 1e21.
pub(super) fn js_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if value.abs() >= 1e21 || value.abs() < 1e-6 {
        let text = format!("{value:e}");
        return match text.split_once('e') {
            Some((mantissa, exponent)) if !exponent.starts_with('-') => {
                format!("{mantissa}e+{exponent}")
            }
            _ => text,
        };
    }
    format!("{value}")
}

/// A create request names an objective; an edit may name either field. Both
/// accept an unknown extra field, as the TypeScript validator does.
fn parse_request(endpoint: &str, value: &Value, create: bool) -> Result<Change, Failure> {
    let bad = || boundary(endpoint, "request");
    let object = value.as_object().ok_or_else(bad)?;
    let objective = match object.get("objective") {
        None => None,
        Some(Value::String(text)) => Some(text.clone()),
        Some(_) => return Err(bad()),
    };
    let max_goal_rounds = match object.get("maxGoalRounds") {
        None => None,
        Some(number @ Value::Number(_)) => Some(number.clone()),
        Some(_) => return Err(bad()),
    };
    if create && objective.is_none() {
        return Err(bad());
    }
    Ok(Change { objective, max_goal_rounds })
}

#[cfg(test)]
mod tests {
    use super::js_number;

    /// Each pair was printed by `String(n)` in Node.
    #[test]
    fn numbers_print_the_way_javascript_prints_them() {
        for (value, printed) in [
            (0.0, "0"),
            (-0.0, "0"),
            (1.0, "1"),
            (-1.0, "-1"),
            (1.5, "1.5"),
            (1e-6, "0.000001"),
            (1e-7, "1e-7"),
            (-1.5e-7, "-1.5e-7"),
            (123456789012345680000.0, "123456789012345680000"),
            (1e21, "1e+21"),
            (1e100, "1e+100"),
            (-2.5e30, "-2.5e+30"),
        ] {
            assert_eq!(js_number(value), printed, "{value:e}");
        }
    }
}
