//! The [`Session`] — an append-only event log with an ordered *surface*
//! projection from which LLM message history is derived.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::llm::{Message, Role};
use serde_json::Value;

use crate::session::event::{
    now_ms, GoalChange, SessionEvent, SessionEventData, SessionHeader, SessionId,
};
use crate::types::{PresetBinding, PresetRevision};

/// A synchronous observer of committed appends (persistence, telemetry).
///
/// A notifier observes one session's appends in the order they were
/// committed. It must not append to the session it observes: the ordered
/// delivery would wait for itself.
pub type AppendNotifier = Arc<dyn Fn(&SessionEvent) + Send + Sync>;

thread_local! {
    /// The sessions whose notifier runs on this thread.
    static NOTIFYING: std::cell::RefCell<Vec<SessionId>> =
        std::cell::RefCell::new(Vec::new());
}

/// Marks the session whose notifier runs on this thread.
struct Notifying(Option<SessionId>);

impl Notifying {
    fn enter(id: SessionId) -> Self {
        NOTIFYING.with(|stack| stack.borrow_mut().push(id.clone()));
        Notifying(Some(id))
    }
}

impl Drop for Notifying {
    fn drop(&mut self) {
        if let Some(id) = self.0.take() {
            NOTIFYING.with(|stack| {
                let mut stack = stack.borrow_mut();
                if let Some(at) = stack.iter().rposition(|entry| *entry == id) {
                    stack.remove(at);
                }
            });
        }
    }
}

/// An event-sourced session: an append-only log of [`SessionEvent`]s.
pub struct Session {
    pub id: SessionId,
    pub header: SessionHeader,
    events: Mutex<Vec<SessionEvent>>,
    /// Surface node seqs in model-visible order.
    surface: Mutex<Vec<u64>>,
    replace_generation: Mutex<u64>,
    next_seq: AtomicU64,
    /// The folded request header from the latest `request/header` event.
    request_header_state: Mutex<Option<crate::session::event::EpochHeader>>,
    notifiers: Mutex<Vec<AppendNotifier>>,
    /// Serialises every append and its delivery, so the rows reach the store
    /// in the order their sequence numbers say.
    append_lock: Mutex<()>,
    /// The preset instructions this session runs, bound to the revision it
    /// was created, resumed, or last changed with.
    preset_binding: Mutex<Option<PresetBinding>>,
}

impl Session {
    /// Create a session, optionally seeded with existing events (replay/fork).
    pub fn new(id: SessionId, header: SessionHeader, seed: Vec<SessionEvent>) -> Self {
        let next_seq = seed.iter().map(|e| e.seq).max().map(|m| m + 1).unwrap_or(0);
        let mut surface: Vec<u64> = Vec::new();
        for event in &seed {
            fold_surface(&mut surface, event, &seed);
        }
        let session = Session {
            id,
            header,
            events: Mutex::new(seed),
            surface: Mutex::new(surface),
            replace_generation: Mutex::new(0),
            next_seq: AtomicU64::new(next_seq),
            request_header_state: Mutex::new(None),
            notifiers: Mutex::new(Vec::new()),
            append_lock: Mutex::new(()),
            preset_binding: Mutex::new(None),
        };
        // Replay folds the request header lazily.
        session.rebuild_header();
        session
    }

    /// Attach a synchronous append observer.
    pub fn add_notifier(&self, notifier: AppendNotifier) {
        self.notifiers.lock().unwrap().push(notifier);
    }

    /// The next event's sequence number — always the log length.
    pub fn seq(&self) -> u64 {
        self.next_seq.load(Ordering::SeqCst)
    }

    /// An immutable snapshot of the append-only log.
    pub fn events(&self) -> Vec<SessionEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Append one typed event. The hot path never blocks on I/O — notifiers
    /// run synchronously but must not await.
    ///
    /// The sequence number is taken under the event lock, so the log's rows
    /// carry the order they were committed in.
    pub fn append(&self, data: SessionEventData) -> SessionEvent {
        self.guard_notifier_reentry();
        let _append = self.append_lock.lock().unwrap();
        let event = {
            let mut events = self.events.lock().unwrap();
            let event = self.new_event(data);
            self.push_event(&mut events, event.clone());
            event
        };
        self.notify(&event);
        event
    }

    /// One event with its next seq and the current time. The caller holds the
    /// event lock, so the sequence numbers follow the log's order.
    fn new_event(&self, data: SessionEventData) -> SessionEvent {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        SessionEvent::new(seq, now_ms(), data)
    }

    /// Record an event while the caller holds the event lock.
    fn push_event(&self, events: &mut Vec<SessionEvent>, event: SessionEvent) {
        events.push(event.clone());
        fold_surface(&mut self.surface.lock().unwrap(), &event, events);
        if let SessionEventData::RequestHeader { header } = &event.data {
            *self.request_header_state.lock().unwrap() = Some(header.clone());
        }
    }

    /// Run the append observers for one event.
    fn notify(&self, event: &SessionEvent) {
        let notifiers = self.notifiers.lock().unwrap().clone();
        let _notifying = Notifying::enter(self.id.clone());
        for notifier in notifiers {
            notifier(event);
        }
    }

    /// Refuse an append made from inside this session's own notifier: the
    /// ordered delivery would wait for itself.
    fn guard_notifier_reentry(&self) {
        let inside = NOTIFYING.with(|stack| stack.borrow().contains(&self.id));
        assert!(
            !inside,
            "a session notifier must not append to its own session ({})",
            self.id
        );
    }

    /// The preset the session runs: the last recorded selection, else the
    /// preset the session was created with.
    pub fn agent_preset(&self) -> Option<String> {
        let events = self.events.lock().unwrap();
        effective_preset(&events, &self.header)
    }

    /// Choose the preset this session runs, when it is still blank.
    ///
    /// The blankness check, the revision check and the record happen under one
    /// lock, so a turn cannot start between them and two callers cannot both
    /// record the same change. Returns the preset the session runs, or `None`
    /// when the session has started a turn and cannot be reused.
    ///
    /// A session that already runs the requested preset at the requested
    /// revision keeps it and nothing is logged. A session bound to an older
    /// revision of the same preset records the change and runs the current
    /// instructions, the way the TypeScript host recomposes a changed preset.
    pub fn select_preset(
        &self,
        preset: &str,
        revision: Option<PresetRevision>,
    ) -> Option<String> {
        self.guard_notifier_reentry();
        let _append = self.append_lock.lock().unwrap();
        let mut events = self.events.lock().unwrap();
        if events
            .iter()
            .any(|event| matches!(event.data, SessionEventData::TurnStart { .. }))
        {
            return None;
        }
        let running = effective_preset(&events, &self.header);
        let bound = self
            .preset_binding
            .lock()
            .unwrap()
            .as_ref()
            .map(|bound| bound.generation);
        // Nothing is recorded only when the preset has a current revision and
        // the session already runs that revision.
        if let Some(current) = revision.as_ref() {
            if running.as_deref() == Some(preset) && bound == Some(current.generation) {
                return Some(preset.to_string());
            }
        }
        let event = self.new_event(SessionEventData::AgentPresetSelected {
            agent_preset: preset.to_string(),
        });
        self.push_event(&mut events, event.clone());
        drop(events);
        *self.preset_binding.lock().unwrap() = revision.map(|current| PresetBinding {
            id: preset.to_string(),
            generation: current.generation,
            text: current.text,
        });
        self.notify(&event);
        Some(preset.to_string())
    }

    /// Bind the preset instructions this session runs.
    pub fn bind_preset(&self, binding: PresetBinding) {
        *self.preset_binding.lock().unwrap() = Some(binding);
    }

    /// The preset instructions this session is bound to.
    pub fn preset_binding(&self) -> Option<PresetBinding> {
        self.preset_binding.lock().unwrap().clone()
    }

    /// The instructions this session sends to the model, when it has any.
    pub fn preset_instructions(&self) -> Option<String> {
        self.preset_binding
            .lock()
            .unwrap()
            .as_ref()
            .map(|binding| binding.text.clone())
    }

    /// Whether a turn has started. A session that has not started one is
    /// blank: it can still change its composition.
    pub fn has_started_turn(&self) -> bool {
        self.events
            .lock()
            .unwrap()
            .iter()
            .any(|event| matches!(event.data, SessionEventData::TurnStart { .. }))
    }

    /// The surface node seqs in model-visible order.
    pub fn surface(&self) -> Vec<u64> {
        self.surface.lock().unwrap().clone()
    }

    pub fn replace_generation(&self) -> u64 {
        *self.replace_generation.lock().unwrap()
    }

    /// The folded request header for the NEXT request, or `None` before the
    /// first `request/header` snapshot.
    pub fn request_header(&self) -> Option<crate::session::event::EpochHeader> {
        self.request_header_state.lock().unwrap().clone()
    }

    fn rebuild_header(&self) {
        let events = self.events.lock().unwrap().clone();
        let mut latest = None;
        for event in &events {
            if let SessionEventData::RequestHeader { header } = &event.data {
                latest = Some(header.clone());
            }
        }
        *self.request_header_state.lock().unwrap() = latest;
    }

    /// Derive the LLM message history by walking the surface nodes.
    ///
    /// Projection rules (mirroring the reference harness):
    /// - `user/message` → a user message carrying its content verbatim;
    /// - `assistant/message` → the assistant message, skipping empty content;
    /// - `tool/result` → the user-role message carrying the tool-result block.
    pub fn derive_messages(&self) -> Vec<Message> {
        self.surface_events()
            .iter()
            .filter_map(project_message)
            .collect()
    }

    /// The model-visible events in order, read as one consistent snapshot.
    ///
    /// The events and the surface are read under the same locks, in the same
    /// order as [`Session::append`], so a compaction landing between the two
    /// reads can never leave a surface that names an event the snapshot lacks.
    pub fn surface_events(&self) -> Vec<SessionEvent> {
        let events = self.events.lock().unwrap();
        let surface = self.surface.lock().unwrap();
        surface
            .iter()
            .filter_map(|seq| {
                // The log is appended in seq order, so the lookup is a binary search.
                let at = events.partition_point(|e| e.seq < *seq);
                events.get(at).filter(|e| e.seq == *seq).cloned()
            })
            .collect()
    }

    /// Project one event to a message, or `None` when it produces none.
    pub fn derive_event_message(&self, event: &SessionEvent) -> Option<Message> {
        project_message(event)
    }

    /// Turn numbers of the open turn (last `turn/start` without `turn/end`).
    pub fn open_turn(&self) -> Option<u64> {
        let events = self.events.lock().unwrap().clone();
        let mut open = None;
        for event in &events {
            match &event.data {
                SessionEventData::TurnStart { turn } => open = Some(*turn),
                SessionEventData::TurnEnd { .. } => open = None,
                _ => {}
            }
        }
        open
    }

    /// Serialize the log as a JSONL string (one event per line).
    pub fn to_jsonl(&self) -> String {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .map(|e| serde_json::to_string(e).expect("session events are JSON-serializable"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Full events for serialization/display, including a raw text rendering.
    pub fn render_event(event: &SessionEvent) -> String {
        match &event.data {
            SessionEventData::UserMessage { message } => {
                format!("[user] {}", message.text())
            }
            SessionEventData::AssistantMessage { message, .. } => {
                format!("[assistant] {}", message.text())
            }
            SessionEventData::ToolCall { name, arguments, .. } => {
                let parsed: Value = serde_json::from_str(arguments).unwrap_or(Value::Null);
                format!("[tool-call] {name} {parsed}")
            }
            SessionEventData::ToolResult { message, .. } => {
                format!("[tool-result] {}", message.text())
            }
            SessionEventData::TurnStart { .. } => "[turn/start]".to_string(),
            SessionEventData::TurnEnd { reason, .. } => format!("[turn/end] {reason:?}"),
            SessionEventData::StepStart { .. } => "[step/start]".to_string(),
            SessionEventData::StepEnd { .. } => "[step/end]".to_string(),
            SessionEventData::AssistantChunk { chunk, .. } => {
                format!("[assistant/chunk] {chunk:?}")
            }
            SessionEventData::TodoWrite { todos } => format!("[todo/write] {todos:?}"),
            SessionEventData::Compaction { before_seq, .. } => {
                format!("[compaction] events before #{before_seq} summarised")
            }
            SessionEventData::Goal { objective, status, .. } => {
                format!("[goal] {status:?}: {objective}")
            }
            SessionEventData::GoalChange { change } => match change {
                GoalChange::Snapshot { operation, goal, .. } => format!(
                    "[goal/change] {} {}: {}",
                    operation.as_str(),
                    goal.phase.as_str(),
                    goal.objective
                ),
                GoalChange::Clear { cleared, .. } => format!("[goal/change] clear {}", cleared.id),
            },
            SessionEventData::RequestHeader { .. } => "[request/header]".to_string(),
            SessionEventData::SessionTitle { title, .. } => format!("[session/title] {title}"),
            SessionEventData::ModelSelection { provider, model } => format!("[model/selection] {provider}/{model}"),
            SessionEventData::PermissionPreset { preset } => format!("[permission/preset] {preset}"),
            SessionEventData::SandboxMode { mode } => format!("[sandbox/mode] {mode}"),
            SessionEventData::ApprovalPolicy { policy, .. } => format!("[approval/policy] {policy}"),
            SessionEventData::PlanMode { active } => format!("[plan/mode] {active}"),
            SessionEventData::AgentPresetSelected { agent_preset } => {
                format!("[agent-preset/selected] {agent_preset}")
            }
            SessionEventData::InboxSpliced { target, start, removed_count, inserted, .. } => {
                format!(
                    "[agent/inbox/spliced] {} @{start} +{} -{}",
                    target.as_str_name(),
                    inserted.len(),
                    removed_count.unwrap_or_default()
                )
            }
            SessionEventData::SessionEndSeed => "[session/end-seed]".to_string(),
            SessionEventData::CommandRun { name, .. } => format!("[command/run] /{name}"),
            SessionEventData::CommandDone { success, .. } => {
                format!("[command/done] {}", if *success { "success" } else { "error" })
            }
            SessionEventData::FeedbackRecord { .. } => "[feedback/record]".to_string(),
            SessionEventData::FeedbackMessagePut { item, .. } => {
                format!("[feedback/message-put] {} {}", item.message_id, item.rating)
            }
            SessionEventData::FeedbackMessageDelete { message_id, .. } => {
                format!("[feedback/message-delete] {message_id}")
            }
        }
    }

    pub fn render(&self) -> String {
        self.events()
            .iter()
            .map(Session::render_event)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// A user-role message helper.
pub fn user_message(id: impl Into<String>, text: impl Into<String>) -> Message {
    Message {
        id: id.into(),
        role: Role::User,
        content: vec![crate::llm::ContentBlock::text(text)],
        source: crate::llm::MessageSource::User { rpc_id: None },
    }
}



/// The preset a log runs: the last recorded selection, else the preset the
/// session was created with.
fn effective_preset(events: &[SessionEvent], header: &SessionHeader) -> Option<String> {
    let selected = events.iter().rev().find_map(|event| match &event.data {
        SessionEventData::AgentPresetSelected { agent_preset } => Some(agent_preset.clone()),
        _ => None,
    });
    selected.or_else(|| header.agent_preset.clone())
}

impl crate::api::services::SessionView for Session {
    fn id(&self) -> &str {
        &self.id
    }

    fn header_cwd(&self) -> Option<String> {
        self.header.cwd.clone()
    }

    fn parent_session(&self) -> Option<String> {
        self.header.parent_session.clone()
    }

    fn header_created_at(&self) -> u64 {
        self.header.created_at
    }

    fn agent_preset(&self) -> Option<String> {
        self.agent_preset()
    }

    fn has_started_turn(&self) -> bool {
        self.has_started_turn()
    }

    fn select_preset(
        &self,
        agent_preset: &str,
        revision: Option<PresetRevision>,
    ) -> Option<String> {
        self.select_preset(agent_preset, revision)
    }

    fn bind_preset(&self, binding: PresetBinding) {
        self.bind_preset(binding);
    }

    fn preset_binding(&self) -> Option<PresetBinding> {
        self.preset_binding()
    }

    fn preset_instructions(&self) -> Option<String> {
        self.preset_instructions()
    }

    fn events(&self) -> Vec<SessionEvent> {
        self.events()
    }

    fn surface(&self) -> Vec<u64> {
        self.surface()
    }

    fn derive_messages(&self) -> Vec<Message> {
        self.derive_messages()
    }

    fn surface_events(&self) -> Vec<SessionEvent> {
        self.surface_events()
    }

    fn append(&self, data: SessionEventData) -> SessionEvent {
        self.append(data)
    }

    fn request_header(&self) -> Option<crate::types::EpochHeader> {
        self.request_header()
    }

    fn open_turn(&self) -> Option<u64> {
        self.open_turn()
    }
}

/// Apply one event to the model-visible surface.
///
/// A compaction drops every surface node before its `before_seq` and any earlier
/// compaction summary (the new summary was written from them), then becomes the
/// first node. Live appends and replay of a seed both go through here, so a
/// reloaded session shows the model exactly what the live one did.
fn fold_surface(surface: &mut Vec<u64>, event: &SessionEvent, events: &[SessionEvent]) {
    if !event.data.is_surface() {
        return;
    }
    if let SessionEventData::Compaction { before_seq, .. } = &event.data {
        let is_summary = |seq: u64| {
            events
                .iter()
                .find(|e| e.seq == seq)
                .is_some_and(|e| matches!(e.data, SessionEventData::Compaction { .. }))
        };
        surface.retain(|&seq| seq >= *before_seq && !is_summary(seq));
        surface.insert(0, event.seq);
        return;
    }
    surface.push(event.seq);
}

/// The model-facing form of a compaction summary.
fn compaction_message(seq: u64, summary: &str) -> crate::types::Message {
    crate::types::Message {
        id: format!("compaction-{seq}"),
        role: crate::types::Role::User,
        content: vec![crate::types::ContentBlock::text(format!(
            "[Summary of the earlier conversation, replacing its messages]\n{summary}"
        ))],
        source: crate::types::MessageSource::Plugin {
            plugin: "compaction".to_string(),
            form: Some(crate::types::ContextForm::Relay),
        },
    }
}

/// The model-facing message of one event, or `None` when it carries none.
///
/// - `user/message` and `tool/result` → their message verbatim;
/// - `assistant/message` → the message, unless it is empty;
/// - `compaction` → the summary as a plugin-sourced user message.
pub fn project_message(event: &SessionEvent) -> Option<crate::types::Message> {
    match &event.data {
        SessionEventData::UserMessage { message } | SessionEventData::ToolResult { message, .. } => {
            Some(message.clone())
        }
        SessionEventData::AssistantMessage { message, .. } if !message.content.is_empty() => {
            Some(message.clone())
        }
        SessionEventData::Compaction { summary, .. } => Some(compaction_message(event.seq, summary)),
        _ => None,
    }
}
