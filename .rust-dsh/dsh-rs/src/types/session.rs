//! The session event vocabulary — the Rust analogue of `SessionEventMap`.
//!
//! A [`Session`](crate::Session) is an append-only log of [`SessionEvent`]s,
//! the single source of truth for an agent interaction. LLM message history
//! is *derived* from the log (`derive_messages`), never stored separately.

use crate::types::agent::{InboxOutcome, InboxTarget};
use crate::types::{LlmCallConfig, Message, StreamChunk, TokenUsage, ToolSchema};
use serde::{Deserialize, Serialize};

/// Shared agent/session identity.
pub type SessionId = String;

/// Why a turn ended — `TurnEndReasonMap`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum TurnEndReason {
    Completed,
    /// A cancellation request interrupted the live turn.
    Aborted { cause: String },
    Blocked,
    /// The turn failed.
    Error { message: String, code: String },
    /// At least one step reached its output-token ceiling.
    MaxTokens,
    /// A persistence backend closed a crash-orphaned turn on reload.
    Interrupted,
}

/// One entry in an agent's todo list — the unit of the `todo/write` event's
/// whole-list snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

/// Where a session goal stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GoalStatus {
    Active,
    Achieved,
    Abandoned,
}

/// Durable continuation phase of a session goal, as TypeScript names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GoalPhase {
    Active,
    Paused,
    Blocked,
    Complete,
}

impl GoalPhase {
    /// The phase's name on the wire and in the log.
    pub fn as_str(self) -> &'static str {
        match self {
            GoalPhase::Active => "active",
            GoalPhase::Paused => "paused",
            GoalPhase::Blocked => "blocked",
            GoalPhase::Complete => "complete",
        }
    }
}

/// The state-changing verb a `goal/change` row records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GoalOperation {
    Create,
    Edit,
    Pause,
    Resume,
    Complete,
    Block,
}

impl GoalOperation {
    /// The operation's name in the log.
    pub fn as_str(self) -> &'static str {
        match self {
            GoalOperation::Create => "create",
            GoalOperation::Edit => "edit",
            GoalOperation::Pause => "pause",
            GoalOperation::Resume => "resume",
            GoalOperation::Complete => "complete",
            GoalOperation::Block => "block",
        }
    }
}

/// Compare-and-set identity of one exact goal revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalRef {
    pub id: String,
    pub revision: u64,
}

/// Why a goal is blocked: a stable kebab-case code and a human message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalBlockReason {
    pub code: String,
    pub message: String,
}

/// The full durable state every non-clear goal mutation writes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalSnapshot {
    pub id: String,
    pub revision: u64,
    pub objective: String,
    pub phase: GoalPhase,
    /// Present exactly while `phase` is blocked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<GoalBlockReason>,
    pub max_goal_rounds: u64,
}

/// One durable goal mutation: the whole post-mutation state, or the tombstone
/// a clear leaves behind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum GoalChange {
    Snapshot {
        operation: GoalOperation,
        goal: GoalSnapshot,
        /// Highest admitted goal round for this goal.
        rounds_started: u64,
        created_at: u64,
        updated_at: u64,
    },
    Clear {
        cleared: GoalRef,
        cleared_at: u64,
    },
}

/// The logged request envelope: call config, rendered system prompt, and
/// assembled tool schemas — the latest snapshot reconstructs the next request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpochHeader {
    pub config: LlmCallConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSchema>>,
}

/// How a message-producing event entered the ordered surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceOp {
    Append,
    /// Replaces surface nodes from `start` through `end` (inclusive).
    Replace { start: u64, end: u64 },
}

/// The merge-extensible, append-only source of truth for an agent interaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SessionEventData {
    /// Opens turn `turn`.
    TurnStart { turn: u64 },
    /// Closes turn `turn` with the reason that ended it.
    TurnEnd { turn: u64, reason: TurnEndReason },
    /// Opens step `step` of turn `turn`.
    StepStart { turn: u64, step: u64 },
    /// Closes step `step` of turn `turn`.
    StepEnd { turn: u64, step: u64 },
    /// A user-role message on the model-visible surface.
    UserMessage { message: Message },
    /// Raw stream chunk — token-level replay fidelity.
    AssistantChunk { turn: u64, step: u64, chunk: StreamChunk },
    /// Assembled assistant message for one step (derived history uses this).
    AssistantMessage {
        turn: u64,
        step: u64,
        message: Message,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<TokenUsage>,
        #[serde(skip_serializing_if = "Option::is_none")]
        interrupted: Option<bool>,
    },
    /// The model requested one tool invocation; `arguments` is the RAW JSON
    /// string exactly as produced.
    ToolCall {
        turn: u64,
        step: u64,
        call_id: String,
        name: String,
        arguments: String,
    },
    /// A completed tool call's model-facing result.
    ToolResult { turn: u64, step: u64, message: Message },
    /// Whole-list snapshot; latest write wins on replay.
    TodoWrite { todos: Vec<TodoItem> },
    /// Replaces every earlier surface node (seq < `before_seq`) with `summary`.
    /// The summary is itself a surface node, so it reaches the model through the log.
    Compaction { summary: String, before_seq: u64 },
    /// Whole-state snapshot of the session goal; latest write wins on replay.
    Goal {
        objective: String,
        status: GoalStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    /// One mutation of the TypeScript-compatible session goal. The `goal`
    /// projection folds these: the latest change is the current goal, and a
    /// clear tombstone means there is none.
    GoalChange { change: GoalChange },
    /// A title the user set by renaming. Latest wins; log-only, so it never
    /// reaches the model. Titles that TypeScript derives from messages name
    /// source messages and are not modelled here: they are skipped on read.
    SessionTitle { title: String },
    /// Validated provider/model intent for the next request; not model-visible.
    ModelSelection { provider: String, model: String },
    /// The agent preset a session runs, chosen after creation while the
    /// session was still blank. The header names the preset the session
    /// started with; this event names the one it runs now.
    AgentPresetSelected { agent_preset: String },
    /// The permission preset a user selected. Log-only user intent: the knob
    /// events that follow it control execution, and this one lets the
    /// `permissions` projection keep a selection when two presets share a
    /// bundle.
    PermissionPreset { preset: String },
    /// The sandbox mode in force from here on (`read-only`,
    /// `workspace-write` or `danger-full-access`). Log-only; the last one wins.
    SandboxMode { mode: String },
    /// The approval policy in force from here on (`ask` or `never`).
    /// Log-only; the last one wins. `source` is `delegation` for a policy
    /// seeded into a delegated child.
    ApprovalPolicy {
        policy: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
    /// Whether plan mode is in force from here on. Log-only; the last one wins.
    PlanMode { active: bool },
    /// One durable mutation of the agent's pending-message lists. The inbox
    /// projection folds these, so pending work survives a restart and reaches
    /// the client as `inbox`.
    InboxSpliced {
        /// Which pending list changed.
        target: InboxTarget,
        /// Index in that list where the change starts.
        start: usize,
        /// How many pending items the change removed.
        removed_count: Option<usize>,
        /// The items the change inserted.
        inserted: Vec<Message>,
        /// Set when the mutation canceled pending work.
        outcome: Option<InboxOutcome>,
    },
    /// Full header for the next request, appended inside its step.
    RequestHeader { header: EpochHeader },
    /// Marks the end of a constructor seed (resume/fork/replay).
    SessionEndSeed,
    /// A slash command entered its handler. Log-only. Paired with
    /// [`SessionEventData::CommandDone`] by `command_id`.
    CommandRun {
        command_id: String,
        /// The command name, lower case, without the slash.
        name: String,
        /// The text after the name, separator included. Absent when the
        /// command's own event owns that text.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        args: Option<String>,
    },
    /// The paired command settled. Log-only.
    CommandDone {
        command_id: String,
        /// Whether the command succeeded.
        success: bool,
        /// What the command reported to the human.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        /// The earlier event that owns a richer presentation of the result.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source_event_seq: Option<u64>,
    },
    /// A remark the human filed about the session. Log-only: it never reaches
    /// the model.
    FeedbackRecord {
        /// The remark, trimmed; absent when it was blank.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        /// The category the human filed it under.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        category: Option<String>,
    },
    /// A rating created or replaced for one assistant message. Log-only. The
    /// feedback list folds these: the latest put for a message is current.
    FeedbackMessagePut {
        /// The session the rating belongs to; a fork inherits its parent's
        /// events, and the rating stays the parent's.
        session_id: String,
        /// The complete current value.
        item: FeedbackItem,
    },
    /// A rating removed. Log-only; earlier puts stay in the log.
    FeedbackMessageDelete { session_id: String, message_id: String },
}

/// One current rating of an assistant message and its compare-and-set token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackItem {
    pub message_id: String,
    /// `positive` or `negative`.
    pub rating: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Replaced by every material change; callers compare it for equality.
    pub version: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl SessionEventData {
    /// Whether this event type is message-producing (a surface node).
    pub fn is_surface(&self) -> bool {
        matches!(
            self,
            SessionEventData::UserMessage { .. }
                | SessionEventData::AssistantMessage { .. }
                | SessionEventData::ToolResult { .. }
                | SessionEventData::Compaction { .. }
        )
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            SessionEventData::TurnStart { .. } => "turn/start",
            SessionEventData::TurnEnd { .. } => "turn/end",
            SessionEventData::StepStart { .. } => "step/start",
            SessionEventData::StepEnd { .. } => "step/end",
            SessionEventData::UserMessage { .. } => "user/message",
            SessionEventData::AssistantChunk { .. } => "assistant/chunk",
            SessionEventData::AssistantMessage { .. } => "assistant/message",
            SessionEventData::ToolCall { .. } => "tool/call",
            SessionEventData::ToolResult { .. } => "tool/result",
            SessionEventData::TodoWrite { .. } => "todo/write",
            SessionEventData::Compaction { .. } => "compaction",
            SessionEventData::Goal { .. } => "goal",
            SessionEventData::GoalChange { .. } => "goal/change",
            SessionEventData::SessionTitle { .. } => "session/title",
            SessionEventData::ModelSelection { .. } => "model/selection",
            SessionEventData::AgentPresetSelected { .. } => "agent-preset/selected",
            SessionEventData::PermissionPreset { .. } => "permission/preset",
            SessionEventData::SandboxMode { .. } => "sandbox/mode",
            SessionEventData::ApprovalPolicy { .. } => "approval/policy",
            SessionEventData::PlanMode { .. } => "plan/mode",
            SessionEventData::InboxSpliced { .. } => "agent/inbox/spliced",
            SessionEventData::RequestHeader { .. } => "request/header",
            SessionEventData::SessionEndSeed => "session/end-seed",
            SessionEventData::CommandRun { .. } => "command/run",
            SessionEventData::CommandDone { .. } => "command/done",
            SessionEventData::FeedbackRecord { .. } => "feedback/record",
            SessionEventData::FeedbackMessagePut { .. } => "feedback/message-put",
            SessionEventData::FeedbackMessageDelete { .. } => "feedback/message-delete",
        }
    }
}

/// One immutable entry in the session log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Monotonic position in the log (`seq = log length`).
    pub seq: u64,
    /// Unix epoch milliseconds.
    pub time: u64,
    #[serde(flatten)]
    pub data: SessionEventData,
}

impl SessionEvent {
    pub fn new(seq: u64, time: u64, data: SessionEventData) -> Self {
        SessionEvent { seq, time, data }
    }

    pub fn event_type(&self) -> &'static str {
        self.data.event_type()
    }
}

/// Storage metadata attached to a session, kept out of the event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHeader {
    /// Validated absolute working directory.
    pub cwd: Option<String>,
    pub created_at: u64,
    /// Durable fork lineage: the parent session id, when forked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session: Option<SessionId>,
    /// Length of the constructor seed (replay/fork boundary).
    pub seed_length: u64,
    /// The agent preset the session runs, when the deployment composes a roster.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_preset: Option<String>,
}

impl Default for SessionHeader {
    fn default() -> Self {
        SessionHeader {
            cwd: None,
            created_at: now_ms(),
            parent_session: None,
            seed_length: 0,
            agent_preset: None,
        }
    }
}

/// Unix epoch milliseconds.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}


/// Options for creating a session.
#[derive(Debug, Clone, Default)]
pub struct CreateSessionOptions {
    pub id: Option<SessionId>,
    pub cwd: Option<String>,
    /// Seed events (replay/fork).
    pub seed: Vec<SessionEvent>,
    /// Parent session lineage for forks.
    pub parent_session: Option<SessionId>,
    /// The agent preset the session runs.
    pub agent_preset: Option<String>,
}

impl CreateSessionOptions {
    pub fn with_id(id: impl Into<SessionId>) -> Self {
        CreateSessionOptions {
            id: Some(id.into()),
            ..Default::default()
        }
    }
}
