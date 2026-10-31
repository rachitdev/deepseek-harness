//! Reminders: the host-side scheduler behind `schedule/*` and the
//! `schedule_*` tools.
//!
//! The TypeScript package (`@deepseek-ai/dsh-schedule`) is the reference. A
//! reminder is bound to the session that created it and, when it falls due, is
//! delivered into that session as a user message, whether or not the session's
//! agent is live.

pub mod cron;
pub mod domain;
pub mod instant;
pub mod service;
pub mod store;
pub mod update;

/// A refusal of a caller's input, with the stable `code` the TypeScript host
/// uses and a message that is safe to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleError {
    code: &'static str,
    message: String,
}

impl ScheduleError {
    /// An input refusal under `code` (`invalid_prompt`, `invalid_rule`, ...).
    pub fn input(code: &'static str, message: impl Into<String>) -> Self {
        ScheduleError { code, message: message.into() }
    }

    /// The stable code.
    pub fn code(&self) -> &'static str {
        self.code
    }

    /// The message.
    pub fn message(&self) -> &str {
        &self.message
    }
}
