//! The [`Agent`] handle and [`AgentRegistry`] — the Rust analogue of
//! `packages/core/agent`.
//!
//! An agent owns one session (the durable event log), an inbox of pending
//! user messages, a lifecycle status, and a driver task running the
//! [`crate::core::loop_driver::drive`] loop.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cordis::Context;

use crate::llm::{CancelToken, Message, MessageSource, Role};
use crate::api::services::{SessionService, SessionView};
use crate::session::{CreateSessionOptions, SessionId, user_message};
use crate::types::{
    AgentCancelCause, AgentOptions, AgentStatus, InboxOutcome, InboxTarget, PromptAdmission,
    PromptTarget, QueueAction, QueueOutcome, SessionEvent, SessionEventData,
};


/// The `agents` service key.
pub const AGENTS_SERVICE: &str = "agents";

/// The two ordered pending-message lists owned by an agent.
#[derive(Debug, Default)]
pub struct Inbox {
    next_turn: VecDeque<Message>,
    next_step: VecDeque<Message>,
}

impl Inbox {
    /// Rebuild the pending lists from the durable splice events, the way the
    /// TypeScript inbox projection folds them. A splice outside the list, or a
    /// message pending twice, is a broken log and is reported.
    pub fn fold(events: &[SessionEvent], through: Option<u64>) -> Result<Self, String> {
        let mut inbox = Inbox::default();
        let mut pending: std::collections::HashSet<String> = std::collections::HashSet::new();
        for event in events {
            if through.is_some_and(|limit| event.seq > limit) {
                break;
            }
            let SessionEventData::InboxSpliced {
                target,
                start,
                removed_count,
                inserted,
                ..
            } = &event.data
            else {
                continue;
            };
            let start = *start;
            let removed = removed_count.unwrap_or(0);
            let list = inbox.list_mut(*target);
            if start > list.len() || start + removed > list.len() {
                return Err(inbox_splice_error(event.seq, "the splice is outside the pending list"));
            }
            // The pending identities are tracked while the log is folded, so
            // the check for a message pending twice costs one step per row.
            for message in list.drain(start..start + removed) {
                pending.remove(&message.id);
            }
            for (offset, message) in inserted.iter().enumerate() {
                if !pending.insert(message.id.clone()) {
                    return Err(inbox_splice_error(
                        event.seq,
                        &format!("message {:?} is already pending", message.id),
                    ));
                }
                list.insert(start + offset, message.clone());
            }
        }
        Ok(inbox)
    }

    /// The pending messages of one list.
    pub fn pending(&self, target: InboxTarget) -> &VecDeque<Message> {
        self.list(target)
    }

    /// The pending list one mutation applies to.
    fn list(&self, target: InboxTarget) -> &VecDeque<Message> {
        match target {
            InboxTarget::NextTurn => &self.next_turn,
            InboxTarget::NextStep => &self.next_step,
        }
    }

    /// The pending list one mutation applies to.
    fn list_mut(&mut self, target: InboxTarget) -> &mut VecDeque<Message> {
        match target {
            InboxTarget::NextTurn => &mut self.next_turn,
            InboxTarget::NextStep => &mut self.next_step,
        }
    }

    /// Where one pending message sits, if it is still pending.
    fn locate(&self, id: &str) -> Option<(InboxTarget, usize)> {
        for target in [InboxTarget::NextTurn, InboxTarget::NextStep] {
            if let Some(index) = self.list(target).iter().position(|m| m.id == id) {
                return Some((target, index));
            }
        }
        None
    }
}

/// Live agent handle: the surface every plugin programs against.
pub struct Agent {
    pub id: SessionId,
    /// The model the next request uses; `select_model` replaces provider and model.
    options: Mutex<AgentOptions>,
    selection_gate: Mutex<()>,
    pub session: Arc<dyn SessionView>,
    /// Agent-scoped context.
    pub ctx: Context,
    status: Mutex<AgentStatus>,
    inbox: Mutex<Inbox>,
    /// Serialises every inbox mutation, so the recorded splice and the live
    /// pending lists cannot disagree when two mutations interleave.
    mutation: Mutex<()>,
    cancel: Mutex<Option<CancelToken>>,
    /// Bumped whenever the inbox gains work (wakes the driver).
    wake_tx: tokio::sync::watch::Sender<u64>,
    wake_rx: tokio::sync::watch::Receiver<u64>,
    /// True while the driver is processing a batch.
    driver_busy: Arc<AtomicBool>,
    /// Number of pending (unclaimed) inbox messages.
    pending: Arc<AtomicUsize>,
    /// Client prompt identities the driver has claimed. A claimed prompt is
    /// logged as a user message a moment later, and a retry that lands in
    /// between must still be recognised as the same prompt.
    claimed_ids: Mutex<std::collections::HashSet<String>>,
    /// Whether a prompt asked the driver to look for work. A session that
    /// inherits pending rows owns them without running them.
    woken: AtomicBool,
    /// Bumped whenever the driver settles back to waiting.
    settle_tx: tokio::sync::watch::Sender<u64>,
    settle_rx: tokio::sync::watch::Receiver<u64>,
    disposed: Arc<AtomicBool>,
    /// Set by the loop when the assistant completes a turn for the first time.
    pub completed_turns: AtomicU64,
    _driver: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Agent {
    pub fn new(
        id: SessionId,
        options: AgentOptions,
        session: Arc<dyn SessionView>,
        ctx: Context,
    ) -> Arc<Agent> {
        let (wake_tx, wake_rx) = tokio::sync::watch::channel(0);
        let (settle_tx, settle_rx) = tokio::sync::watch::channel(0);
        Arc::new(Agent {
            id,
            options: Mutex::new(options),
            selection_gate: Mutex::new(()),
            session,
            ctx,
            status: Mutex::new(AgentStatus::Idle),
            inbox: Mutex::new(Inbox::default()),
            mutation: Mutex::new(()),
            cancel: Mutex::new(None),
            wake_tx,
            wake_rx,
            driver_busy: Arc::new(AtomicBool::new(false)),
            pending: Arc::new(AtomicUsize::new(0)),
            claimed_ids: Mutex::new(std::collections::HashSet::new()),
            woken: AtomicBool::new(false),
            settle_tx,
            settle_rx,
            disposed: Arc::new(AtomicBool::new(false)),
            completed_turns: AtomicU64::new(0),
            _driver: Mutex::new(None),
        })
    }

    pub fn status(&self) -> AgentStatus {
        *self.status.lock().unwrap()
    }

    /// The options the next request is built from.
    pub fn options(&self) -> AgentOptions {
        self.options.lock().unwrap().clone()
    }

    /// Use `provider`/`model` from the next request on. A request already in
    /// flight keeps the model it started with.
    pub fn select_model(&self, provider: String, model: String) {
        // Keep concurrent choices in log order, so each control frame has a
        // distinct session seq as the TS projection store expects.
        let _selection = self.selection_gate.lock().unwrap();
        {
            let mut options = self.options.lock().unwrap();
            options.provider = provider.clone();
            options.model = model.clone();
        }
        self.session.append(crate::types::SessionEventData::ModelSelection { provider, model });
    }

    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::SeqCst)
    }

    /// Queue an ordinary follow-up turn and wake the driver.
    pub fn followup(&self, message: Message) {
        let _mutation = self.mutation.lock().unwrap();
        if self.queue_locked(InboxTarget::NextTurn, message) {
            self.wake();
        }
    }

    /// Submit steering for the nearest step (does not open a new turn).
    pub fn steer(&self, message: Message) {
        let _mutation = self.mutation.lock().unwrap();
        if self.queue_locked(InboxTarget::NextStep, message) {
            self.wake();
        }
    }

    /// Queue model-facing context without waking the driver.
    pub fn inject(&self, message: Message) {
        let _mutation = self.mutation.lock().unwrap();
        self.queue_locked(InboxTarget::NextStep, message);
    }

    /// Queue one prompt unless its client identity was already accepted, so a
    /// retried prompt is not queued twice. The check and the queueing happen
    /// together, so two retries cannot both slip through.
    pub fn admit_prompt(&self, message: Message, target: PromptTarget) -> PromptAdmission {
        let _mutation = self.mutation.lock().unwrap();
        if let Some(rpc_id) = message_rpc_id(&message) {
            if self.has_prompt_request(rpc_id) {
                return PromptAdmission::Duplicate;
            }
        }
        let list = match target {
            PromptTarget::Turn => InboxTarget::NextTurn,
            PromptTarget::Step => InboxTarget::NextStep,
        };
        if !self.queue_locked(list, message) {
            return PromptAdmission::Refused;
        }
        self.wake();
        PromptAdmission::Queued
    }

    /// Queue one message at the end of a pending list and record the splice.
    /// A message whose identity is already pending is refused: queueing it
    /// again would write a log the inbox fold rejects.
    fn queue_locked(&self, target: InboxTarget, message: Message) -> bool {
        if self.inbox.lock().unwrap().locate(&message.id).is_some() {
            crate::api::events::emit(
                &self.ctx,
                &crate::api::events::AgentErrorPayload {
                    agent: self.id.clone(),
                    error: format!("message {:?} is already pending", message.id),
                },
            );
            return false;
        }
        let start = self.inbox.lock().unwrap().list(target).len();
        self.splice_inbox(target, start, 0, vec![message], None);
        self.recount_pending();
        true
    }

    /// Record how many inbox messages are still unclaimed.
    fn recount_pending(&self) {
        let inbox = self.inbox.lock().unwrap();
        let count = inbox.next_turn.len() + inbox.next_step.len();
        self.pending.store(count, Ordering::SeqCst);
    }

    /// Apply one inbox mutation: record it in the log, then keep the pending
    /// lists in step. The log is what a restart reads back, so it is written
    /// first. The caller holds the mutation lock, so the recorded splice and
    /// the live list describe the same change.
    fn splice_inbox(
        &self,
        target: InboxTarget,
        start: usize,
        removed: usize,
        inserted: Vec<Message>,
        outcome: Option<InboxOutcome>,
    ) {
        self.session
            .append(crate::types::SessionEventData::InboxSpliced {
                target,
                start,
                removed_count: (removed > 0).then_some(removed),
                inserted: inserted.clone(),
                outcome,
            });
        let mut inbox = self.inbox.lock().unwrap();
        let list = inbox.list_mut(target);
        for _ in 0..removed {
            list.remove(start);
        }
        for (offset, message) in inserted.into_iter().enumerate() {
            list.insert(start + offset, message);
        }
    }

    /// Apply one mutation to a pending item: replace its text, drop it, or
    /// steer the running turn with it. The eligibility check and the mutation
    /// happen together, so an item the driver claimed in between is reported
    /// instead of silently ignored.
    pub fn update_queue(&self, id: &str, action: QueueAction) -> QueueOutcome {
        let _mutation = self.mutation.lock().unwrap();
        let located = self.inbox.lock().unwrap().locate(id);
        let Some((target, index)) = located else {
            return QueueOutcome::NotFound;
        };
        match action {
            QueueAction::Edit(content) => {
                let replacement = {
                    let inbox = self.inbox.lock().unwrap();
                    let Some(message) = inbox.list(target).get(index) else {
                        return QueueOutcome::NotFound;
                    };
                    let mut replacement = message.clone();
                    replacement.content = content;
                    replacement
                };
                self.splice_inbox(target, index, 1, vec![replacement], Some(InboxOutcome::Canceled));
                QueueOutcome::Applied
            }
            QueueAction::Remove => {
                {
                    let inbox = self.inbox.lock().unwrap();
                    if inbox.list(target).get(index).is_none() {
                        return QueueOutcome::NotFound;
                    }
                }
                self.splice_inbox(target, index, 1, Vec::new(), Some(InboxOutcome::Canceled));
                self.recount_pending();
                QueueOutcome::Applied
            }
            QueueAction::Steer => {
                if target != InboxTarget::NextTurn
                    || *self.status.lock().unwrap() != AgentStatus::Running
                {
                    return QueueOutcome::SteerUnavailable;
                }
                let message = {
                    let inbox = self.inbox.lock().unwrap();
                    match inbox.list(target).get(index) {
                        Some(message) => message.clone(),
                        None => return QueueOutcome::NotFound,
                    }
                };
                self.splice_inbox(target, index, 1, Vec::new(), Some(InboxOutcome::Canceled));
                let start = self.inbox.lock().unwrap().list(InboxTarget::NextStep).len();
                self.splice_inbox(InboxTarget::NextStep, start, 0, vec![message], None);
                self.wake();
                QueueOutcome::Applied
            }
        }
    }

    /// Remember the client identities of the messages a claim took.
    fn remember_claimed(&self, messages: &[Message]) {
        let mut claimed = self.claimed_ids.lock().unwrap();
        for message in messages {
            if let Some(rpc_id) = message_rpc_id(message) {
                claimed.insert(rpc_id.to_string());
            }
        }
    }

    /// Drop the remembered identity of a prompt that is now logged, so the
    /// remembered set only holds the claims the log does not cover yet.
    pub(crate) fn forget_claimed_prompt(&self, message: &Message) {
        if let Some(rpc_id) = message_rpc_id(message) {
            self.claimed_ids.lock().unwrap().remove(rpc_id);
        }
    }

    /// Whether a prompt carrying this client identity is already pending,
    /// already claimed, or already logged, so a retried prompt is not queued
    /// twice.
    pub fn has_prompt_request(&self, rpc_id: &str) -> bool {
        let is_same_prompt = |message: &Message| {
            message_rpc_id(message) == Some(rpc_id)
        };
        let inbox = self.inbox.lock().unwrap();
        let pending = inbox
            .next_turn
            .iter()
            .chain(inbox.next_step.iter())
            .any(is_same_prompt);
        drop(inbox);
        if pending {
            return true;
        }
        if self.claimed_ids.lock().unwrap().contains(rpc_id) {
            return true;
        }
        self.session.events().iter().any(|event| {
            matches!(&event.data, SessionEventData::UserMessage { message } if is_same_prompt(message))
        })
    }

    /// Bump the wake channel so the driver re-checks its inbox.
    pub fn wake(&self) {
        self.woken.store(true, Ordering::SeqCst);
        let version = self.wake_tx.borrow().wrapping_add(1);
        let _ = self.wake_tx.send(version);
    }

    /// Cancel the active turn and (unless `keep_inbox`) clear pending work.
    pub fn cancel(&self, cause: AgentCancelCause, keep_inbox: bool) {
        if let Some(token) = self.cancel.lock().unwrap().as_ref() {
            token.cancel();
        }
        if !keep_inbox {
            let _mutation = self.mutation.lock().unwrap();
            let (turn_len, step_len) = {
                let inbox = self.inbox.lock().unwrap();
                (
                    inbox.list(InboxTarget::NextTurn).len(),
                    inbox.list(InboxTarget::NextStep).len(),
                )
            };
            // The TypeScript inbox clears the step list before the turn list.
            if step_len > 0 {
                self.splice_inbox(
                    InboxTarget::NextStep,
                    0,
                    step_len,
                    Vec::new(),
                    Some(InboxOutcome::Canceled),
                );
            }
            if turn_len > 0 {
                self.splice_inbox(
                    InboxTarget::NextTurn,
                    0,
                    turn_len,
                    Vec::new(),
                    Some(InboxOutcome::Canceled),
                );
            }
            self.pending.store(0, Ordering::SeqCst);
        }
        let _ = cause;
    }

    pub fn driver_busy(&self) -> bool {
        self.driver_busy.load(Ordering::SeqCst)
    }

    /// Resolve after the driver reaches quiescence: no active batch and no
    /// pending woken work.
    pub async fn when_idle(&self) {
        let mut rx = self.settle_rx.clone();
        loop {
            if !self.driver_busy.load(Ordering::SeqCst) && !self.has_work() {
                return;
            }
            // Copy the version out so the read guard drops BEFORE awaiting:
            // holding it across changed() deadlocks single-threaded runtimes.
            let _version = *rx.borrow_and_update();
            if rx.changed().await.is_err() {
                return;
            }
        }
    }

    pub(crate) fn has_work(&self) -> bool {
        self.pending.load(Ordering::SeqCst) > 0 && self.woken.load(Ordering::SeqCst)
    }

    /// Rebuild the pending inbox from the durable splice events the session
    /// already holds, so an adopted session owns the rows its projection
    /// shows. The rows wait for a prompt; they do not start a turn.
    pub fn restore_pending(&self) -> Result<(), String> {
        let inbox = Inbox::fold(&self.session.events(), None)?;
        let pending = inbox.next_turn.len() + inbox.next_step.len();
        *self.inbox.lock().unwrap() = inbox;
        self.pending.store(pending, Ordering::SeqCst);
        Ok(())
    }

    /// Claim one turn-opening batch: one `next-turn` message plus all
    /// `next-step` input.
    pub(crate) fn claim_batch(&self) -> Vec<Message> {
        let _mutation = self.mutation.lock().unwrap();
        let (turn_message, step_messages) = {
            let inbox = self.inbox.lock().unwrap();
            (
                inbox.list(InboxTarget::NextTurn).front().cloned(),
                inbox
                    .list(InboxTarget::NextStep)
                    .iter()
                    .cloned()
                    .collect::<Vec<Message>>(),
            )
        };
        // A claim is the loop's own read of pending work, so it carries no
        // cancellation outcome.
        if !step_messages.is_empty() {
            self.splice_inbox(
                InboxTarget::NextStep,
                0,
                step_messages.len(),
                Vec::new(),
                None,
            );
        }
        if turn_message.is_some() {
            self.splice_inbox(InboxTarget::NextTurn, 0, 1, Vec::new(), None);
        }
        self.recount_pending();
        self.remember_claimed(&step_messages);
        if let Some(message) = &turn_message {
            self.remember_claimed(std::slice::from_ref(message));
        }
        // The TypeScript inbox claims the step input first and the queued turn
        // message last, so the model reads the injected context before the
        // prompt that opened the turn.
        let mut batch = step_messages;
        if let Some(message) = turn_message {
            batch.push(message);
        }
        batch
    }

    /// Claim only the `next-step` input (a step continuation).
    pub(crate) fn claim_next_step(&self) -> Vec<Message> {
        let _mutation = self.mutation.lock().unwrap();
        let drained: Vec<Message> = {
            let inbox = self.inbox.lock().unwrap();
            inbox.list(InboxTarget::NextStep).iter().cloned().collect()
        };
        if !drained.is_empty() {
            self.splice_inbox(InboxTarget::NextStep, 0, drained.len(), Vec::new(), None);
        }
        self.recount_pending();
        self.remember_claimed(&drained);
        drained
    }

    /// Write the session log to its backend, so a later run can resume it.
    /// The failure is reported: a caller that must not lose the logged prefix
    /// (the model request, the tool body) stops when it fails.
    pub(crate) async fn checkpoint(&self) -> Result<(), String> {
        let Some(store) =
            self.ctx.get::<crate::api::services::SessionService>(crate::session::store::SESSIONS_SERVICE)
        else {
            // Without a store there is nothing to write.
            return Ok(());
        };
        store.flush(&self.id).await
    }

    pub(crate) fn begin_turn(&self, token: CancelToken) {
        *self.cancel.lock().unwrap() = Some(token);
        *self.status.lock().unwrap() = AgentStatus::Running;
        self.driver_busy.store(true, Ordering::SeqCst);
        self.emit_status();
    }

    /// End the turn. The session log is on disk before the agent reports
    /// itself idle, so a caller that waits for idle knows the turn is durable.
    pub(crate) async fn end_turn(&self) {
        if let Err(reason) = self.checkpoint().await {
            self.ctx.logger().log_event(
                cordis::logger::LogLevel::Warn,
                "session".to_string(),
                None,
                format!("cannot checkpoint session {}: {reason}", self.id),
            );
        }
        *self.status.lock().unwrap() = AgentStatus::Idle;
        self.driver_busy.store(false, Ordering::SeqCst);
        // Copy the version BEFORE send: the borrow guard must not outlive the
        // statement (a read lock held across the write lock self-deadlocks).
        let version = self.settle_tx.borrow().wrapping_add(1);
        let _ = self.settle_tx.send(version);
        self.emit_status();
    }

    pub(crate) fn turn_token(&self) -> Option<CancelToken> {
        self.cancel.lock().unwrap().clone()
    }

    pub(crate) fn wake_receiver(&self) -> tokio::sync::watch::Receiver<u64> {
        self.wake_rx.clone()
    }

    fn emit_status(&self) {
        crate::api::events::emit(
            &self.ctx,
            &crate::api::events::AgentStatusPayload {
                agent: self.id.clone(),
                status: self.status().as_str_name().to_string(),
            },
        );
    }
}

struct AgentRegistryInner {
    agents: Mutex<std::collections::HashMap<SessionId, Arc<Agent>>>,
    ctx: Context,
    sessions: SessionService,
}

/// Live agent registry (`ctx.agents`). Cheap-clone handle.
#[derive(Clone)]
pub struct AgentRegistry {
    inner: Arc<AgentRegistryInner>,
}

/// The seq a fork keeps by default: the last `turn/end`, extended over the
/// events after it until the next turn or user prompt begins. `None` when no
/// turn has completed.
fn latest_completed_prefix(events: &[SessionEvent]) -> Option<u64> {
    use crate::session::SessionEventData as D;
    let last_end = events.iter().rposition(|e| matches!(e.data, D::TurnEnd { .. }))?;
    let mut boundary = events[last_end].seq;
    for event in &events[last_end + 1..] {
        if matches!(event.data, D::TurnStart { .. } | D::UserMessage { .. }) {
            break;
        }
        boundary = event.seq;
    }
    Some(boundary)
}

impl AgentRegistry {
    pub fn new(ctx: Context, sessions: SessionService) -> Self {
        AgentRegistry {
            inner: Arc::new(AgentRegistryInner {
                agents: Mutex::new(std::collections::HashMap::new()),
                ctx,
                sessions,
            }),
        }
    }

    /// Create a session and agent under one identity, then start its driver.
    pub fn create(
        &self,
        id: Option<SessionId>,
        options: AgentOptions,
        cwd: Option<String>,
        seed_prompt: Option<String>,
        agent_preset: Option<String>,
    ) -> Result<Arc<Agent>, crate::core::AgentError> {
        let session = self.inner.sessions.create(CreateSessionOptions {
            id: id.clone(),
            cwd: cwd.clone(),
            agent_preset: agent_preset.clone(),
            ..Default::default()
        });
        // The session is bound to the revision of the preset's instructions
        // that is current now; a later roster change does not alter it.
        if let Some(preset) = &agent_preset {
            self.bind_preset(session.as_ref(), preset);
        }
        if let Some(seed) = seed_prompt {
            session.append(crate::session::SessionEventData::UserMessage {
                message: user_message(format!("seed-{}", session.id()), seed),
            });
        }
        self.adopt(session, options)
    }

    /// Fork a live agent's session at a stable prefix and run the child under
    /// its own agent. `boundary` is an inclusive source seq; without one the
    /// latest completed turn (and what follows it, up to the next turn or
    /// prompt) is kept, as the TypeScript host does. Returns `None` when the
    /// source has no completed turn to fork from.
    pub fn fork(
        &self,
        source_id: &str,
        boundary: Option<u64>,
        options: AgentOptions,
    ) -> Result<Option<Arc<Agent>>, crate::core::AgentError> {
        let source = self
            .get(source_id)
            .ok_or_else(|| crate::core::AgentError::Loop(format!("no live session {source_id:?}")))?;
        let events = source.session.events();
        let boundary = match boundary {
            Some(seq) => seq,
            None => match latest_completed_prefix(&events) {
                Some(seq) => seq,
                None => return Ok(None),
            },
        };
        let child = self
            .inner
            .sessions
            .fork(source_id, Some(boundary), None)
            .map_err(crate::core::AgentError::Loop)?;
        // A child joins the exact revision its parent retained, the way the
        // TypeScript host binds a child to its parent's revision.
        if let Some(binding) = source.session.preset_binding() {
            child.bind_preset(binding);
        }
        self.adopt(child, options).map(Some)
    }

    /// Adopt a stored session and run its agent over it, so a session from an
    /// earlier run appears again. The events are the stored log, and the
    /// session continues its own numbering from them.
    pub fn resume(
        &self,
        id: String,
        header: Option<crate::types::SessionHeader>,
        events: Vec<SessionEvent>,
        options: AgentOptions,
    ) -> Result<Arc<Agent>, crate::core::AgentError> {
        let session = self.inner.sessions.resume(id, header, events);
        // A resumed agent runs the current revision of the preset its log
        // names. A log naming a preset this deployment does not offer cannot
        // be adopted: the TypeScript host mounts the preset and fails when it
        // is unavailable.
        if let Some(preset) = session.agent_preset() {
            if self.preset_revision(&preset).is_none() {
                return Err(crate::core::AgentError::Loop(format!(
                    "the preset {preset} is not in this deployment's roster"
                )));
            }
            self.bind_preset(session.as_ref(), &preset);
        }
        self.adopt(session, options)
    }

    /// The current revision of a preset's instructions, when the deployment
    /// mounts any.
    fn preset_revision(&self, preset: &str) -> Option<crate::types::PresetRevision> {
        self.inner
            .ctx
            .get::<crate::api::services::SystemPromptService>(crate::api::SYSTEM_PROMPT_SERVICE)
            .and_then(|service| service.preset_revision(preset))
    }

    /// Bind a session to the current revision of a preset's instructions.
    fn bind_preset(&self, session: &dyn crate::api::services::SessionView, preset: &str) {
        let Some(revision) = self.preset_revision(preset) else {
            return;
        };
        session.bind_preset(crate::types::PresetBinding {
            id: preset.to_string(),
            generation: revision.generation,
            text: revision.text,
        });
    }

    /// Run an agent over an existing session and start its driver.
    fn adopt(
        &self,
        session: Arc<dyn SessionView>,
        options: AgentOptions,
    ) -> Result<Arc<Agent>, crate::core::AgentError> {
        let agent_id = session.id().to_string();
        let agent = Agent::new(agent_id.clone(), options, session, self.inner.ctx.clone());
        // The adopted session may already hold pending rows; the agent owns
        // them before it accepts work.
        agent
            .restore_pending()
            .map_err(crate::core::AgentError::Loop)?;
        {
            let mut map = self.inner.agents.lock().unwrap();
            if map.contains_key(&agent_id) {
                return Err(crate::core::AgentError::Duplicate(agent_id));
            }
            map.insert(agent_id.clone(), agent.clone());
        }
        crate::api::events::emit(
            &self.inner.ctx,
            &crate::api::events::AgentCreatedPayload { agent: agent_id },
        );
        crate::core::loop_driver::spawn_driver(agent.clone());
        Ok(agent)
    }

    pub fn get(&self, id: &str) -> Option<Arc<Agent>> {
        self.inner.agents.lock().unwrap().get(id).cloned()
    }

    pub fn list(&self) -> Vec<Arc<Agent>> {
        self.inner.agents.lock().unwrap().values().cloned().collect()
    }

    /// Dispose an agent: mark disposed, wake the driver so it exits.
    pub fn dispose(&self, agent: &Arc<Agent>) {
        agent.disposed.store(true, Ordering::SeqCst);
        agent.cancel(AgentCancelCause::Disposed, false);
        agent.wake();
        let removed = self.inner.agents.lock().unwrap().remove(&agent.id).is_some();
        if removed {
            crate::api::events::emit(
                &self.inner.ctx,
                &crate::api::events::AgentDisposedPayload {
                    agent: agent.id.clone(),
                },
            );
        }
    }

    /// Dispose every agent (harness shutdown).
    pub async fn dispose_all(&self) {
        let agents = self.list();
        for agent in agents {
            self.dispose(&agent);
            let _ = tokio::time::timeout(Duration::from_secs(2), agent.when_idle()).await;
        }
    }
}

/// Why a durable inbox splice cannot be folded.
pub fn inbox_splice_error(seq: u64, reason: &str) -> String {
    format!("invalid persisted inbox splice at session seq {seq}: {reason}")
}

/// The client-minted prompt identity a message carries, when it has one.
fn message_rpc_id(message: &Message) -> Option<&str> {
    match &message.source {
        MessageSource::User { rpc_id: Some(rpc_id) } => Some(rpc_id.as_str()),
        _ => None,
    }
}

/// Build a user-role message for the model.
pub fn user_message_with_text(id: impl Into<String>, text: impl Into<String>) -> Message {
    Message {
        id: id.into(),
        role: Role::User,
        content: vec![crate::llm::ContentBlock::text(text)],
        source: MessageSource::User { rpc_id: None },
    }
}

impl crate::api::services::AgentView for Agent {
    fn id(&self) -> &str {
        &self.id
    }

    fn session(&self) -> Arc<dyn crate::api::services::SessionView> {
        self.session.clone()
    }

    fn followup(&self, message: Message) {
        Agent::followup(self, message)
    }

    fn steer(&self, message: Message) {
        Agent::steer(self, message)
    }

    fn inject(&self, message: Message) {
        Agent::inject(self, message)
    }

    fn cancel(&self, cause: crate::types::AgentCancelCause, keep_inbox: bool) {
        Agent::cancel(self, cause, keep_inbox)
    }

    fn status(&self) -> crate::types::AgentStatus {
        Agent::status(self)
    }

    fn options(&self) -> AgentOptions {
        Agent::options(self)
    }

    fn select_model(&self, provider: String, model: String) {
        Agent::select_model(self, provider, model)
    }

    fn update_queue(&self, id: &str, action: QueueAction) -> QueueOutcome {
        Agent::update_queue(self, id, action)
    }

    fn admit_prompt(&self, message: Message, target: PromptTarget) -> PromptAdmission {
        Agent::admit_prompt(self, message, target)
    }

    fn has_prompt_request(&self, rpc_id: &str) -> bool {
        Agent::has_prompt_request(self, rpc_id)
    }

    fn driver_busy(&self) -> bool {
        Agent::driver_busy(self)
    }

    fn when_idle(&self) -> crate::api::services::BoxFuture<()> {
        let mut rx = self.settle_rx.clone();
        let busy = self.driver_busy.clone();
        let pending = self.pending.clone();
        Box::pin(async move {
            loop {
                if !busy.load(Ordering::SeqCst) && pending.load(Ordering::SeqCst) == 0 {
                    return;
                }
                let _version = *rx.borrow_and_update();
                if rx.changed().await.is_err() {
                    return;
                }
            }
        })
    }
}

impl crate::api::services::AgentRegistryApi for AgentRegistry {
    fn create(
        &self,
        id: Option<String>,
        options: AgentOptions,
        cwd: Option<String>,
        seed_prompt: Option<String>,
        agent_preset: Option<String>,
    ) -> Result<Arc<dyn crate::api::services::AgentView>, String> {
        self.create(id, options, cwd, seed_prompt, agent_preset)
            .map(|a| a as Arc<dyn crate::api::services::AgentView>)
            .map_err(|e| e.to_string())
    }

    fn fork(
        &self,
        source_id: &str,
        boundary: Option<u64>,
        options: AgentOptions,
    ) -> Result<Option<Arc<dyn crate::api::services::AgentView>>, String> {
        self.fork(source_id, boundary, options)
            .map(|agent| agent.map(|a| a as Arc<dyn crate::api::services::AgentView>))
            .map_err(|e| e.to_string())
    }

    fn resume(
        &self,
        id: String,
        header: Option<crate::types::SessionHeader>,
        events: Vec<SessionEvent>,
        options: AgentOptions,
    ) -> Result<Arc<dyn crate::api::services::AgentView>, String> {
        self.resume(id, header, events, options)
            .map(|a| a as Arc<dyn crate::api::services::AgentView>)
            .map_err(|e| e.to_string())
    }

    fn get(&self, id: &str) -> Option<Arc<dyn crate::api::services::AgentView>> {
        self.get(id)
            .map(|a| a as Arc<dyn crate::api::services::AgentView>)
    }

    fn list(&self) -> Vec<Arc<dyn crate::api::services::AgentView>> {
        self.list()
            .into_iter()
            .map(|a| a as Arc<dyn crate::api::services::AgentView>)
            .collect()
    }

    fn dispose(&self, agent: &Arc<dyn crate::api::services::AgentView>) {
        if let Some(concrete) = self.get(agent.id()) {
            self.dispose(&concrete);
        }
    }
}
