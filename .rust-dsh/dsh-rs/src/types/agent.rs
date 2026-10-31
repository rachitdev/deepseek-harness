//! Agent-facing vocabulary shared by the agent registry, the loop, and
//! consumers.

use crate::types::llm::ContentBlock;
use serde::{Deserialize, Serialize};

/// Creation options for one agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentOptions {
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl AgentOptions {
    pub fn mock(model: impl Into<String>) -> Self {
        AgentOptions {
            provider: "mock".to_string(),
            model: model.into(),
            max_tokens: None,
        }
    }
}

/// One preset's instructions and the revision they belong to.
///
/// A revision changes when the roster changes a preset's instructions, so a
/// session can tell whether it still runs the current ones.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetRevision {
    /// The revision's identity. It differs for every change of the text.
    pub generation: u64,
    /// The instructions the preset contributes. Empty when it has none.
    pub text: String,
}

/// The preset instructions a session runs, bound to the revision it was
/// created, resumed, or last changed with.
///
/// A roster change does not alter a bound session: it runs the instructions
/// it was bound to until it selects the preset again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetBinding {
    /// The preset the instructions belong to.
    pub id: String,
    /// The revision the instructions belong to.
    pub generation: u64,
    /// The instructions themselves.
    pub text: String,
}

/// An agent's lifecycle state: `idle` means no driver activity; `running`
/// spans the driver draining its inbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentStatus {
    Idle,
    Running,
}

impl AgentStatus {
    pub fn as_str_name(self) -> &'static str {
        match self {
            AgentStatus::Idle => "idle",
            AgentStatus::Running => "running",
        }
    }
}

/// Which pending-message list an inbox mutation applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InboxTarget {
    /// Prompts waiting to open the next turn.
    NextTurn,
    /// Input waiting for the next step boundary.
    NextStep,
}

impl InboxTarget {
    /// The list name the TypeScript inbox projection uses.
    pub fn as_str_name(self) -> &'static str {
        match self {
            InboxTarget::NextTurn => "next-turn",
            InboxTarget::NextStep => "next-step",
        }
    }
}

/// How an inbox mutation ended the pending work it touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InboxOutcome {
    /// The mutation canceled pending work.
    Canceled,
}

/// Where an admitted prompt goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptTarget {
    /// A queued turn message.
    Turn,
    /// Step input for the running turn.
    Step,
}

/// What the admission of one prompt decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptAdmission {
    /// The prompt is queued.
    Queued,
    /// The identity was already accepted, so nothing was queued.
    Duplicate,
    /// The message identity is already pending, so the prompt was refused.
    Refused,
}

/// One mutation of a pending queue item.
#[derive(Debug, Clone, PartialEq)]
pub enum QueueAction {
    /// Replace the item's content, keeping its identity.
    Edit(Vec<ContentBlock>),
    /// Drop the item.
    Remove,
    /// Steer the running turn with the item.
    Steer,
}

/// What a queue mutation did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueOutcome {
    /// The mutation was applied.
    Applied,
    /// The item was no longer pending.
    NotFound,
    /// The running turn no longer accepts steering.
    SteerUnavailable,
}

/// Why an active driver was cancelled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AgentCancelCause {
    User,
    Parent,
    Hook { reason: String },
    Disposed,
}
