//! Cold reload: the sessions an earlier run stored are adopted again, so a
//! restart finds them.
//!
//! The stored log is the session's own history, not an inherited seed: the
//! backend reconciles the rows it already holds with the events the session
//! resumes from, and the log on disk stays authoritative for its header.

use cordis::Context;

use std::sync::Arc;

use crate::api::services::{AgentRegistryService, SessionPersistenceApi};
use crate::types::{AgentOptions, SessionEvent, SessionEventData};

/// What a resume did.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResumeOutcome {
    /// Sessions that are live again.
    pub resumed: usize,
    /// Sessions the runtime could not read or adopt.
    pub failed: usize,
}

/// Adopt the stored sessions, so a restart finds them again.
pub async fn resume_stored_sessions(ctx: &Context, options: AgentOptions) -> ResumeOutcome {
    // The backend the bundle provides is the one the stored sessions are read
    // from; a backend attached to the store alone is not one.
    let Some(persistence) =
        ctx.get::<Arc<dyn SessionPersistenceApi>>(
            crate::api::services::SESSION_PERSISTENCE_SERVICE,
        )
    else {
        // Without a backend there is nothing stored to resume.
        return ResumeOutcome::default();
    };
    let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
        return ResumeOutcome::default();
    };
    let mut outcome = ResumeOutcome::default();
    for id in persistence.list() {
        let events = match persistence.load(&id) {
            Ok(events) => events,
            Err(reason) => {
                report(ctx, &id, &format!("cannot read the stored log: {reason}"));
                outcome.failed += 1;
                continue;
            }
        };
        // A session with only a header is still a session.
        let header = match persistence.header(&id) {
            Ok(header) => header,
            Err(reason) => {
                report(ctx, &id, &format!("cannot read the stored header: {reason}"));
                outcome.failed += 1;
                continue;
            }
        };
        // A session keeps the route it last used.
        let route = last_route(&events).unwrap_or_else(|| options.clone());
        match agents.resume(id.clone(), header, events, route) {
            Ok(_) => outcome.resumed += 1,
            Err(reason) => {
                report(ctx, &id, &format!("cannot resume the session: {reason}"));
                outcome.failed += 1;
            }
        }
    }
    outcome
}

/// Adopt one stored session, returning its agent. `Ok(None)` when nothing is stored under `id`.
pub fn resume_one(ctx: &Context, id: &str) -> Result<Option<Arc<dyn crate::api::services::AgentView>>, String> {
    let Some(persistence) = ctx.get::<Arc<dyn SessionPersistenceApi>>(crate::api::services::SESSION_PERSISTENCE_SERVICE) else {
        return Ok(None);
    };
    if !persistence.list().iter().any(|stored| stored == id) {
        return Ok(None);
    }
    let agents = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .ok_or_else(|| "the agents service is missing".to_string())?;
    let events = persistence.load(id).map_err(|reason| format!("cannot read the stored log: {reason}"))?;
    let header = persistence.header(id).map_err(|reason| format!("cannot read the stored header: {reason}"))?;
    // The route the session last used; a log with none cannot say, so it takes the mock route.
    let route = last_route(&events).unwrap_or_else(|| AgentOptions::mock("m"));
    agents.resume(id.to_string(), header, events, route).map(Some)
}

/// Report a session the runtime cannot resume.
fn report(ctx: &Context, id: &str, reason: &str) {
    ctx.logger().log_event(
        cordis::logger::LogLevel::Warn,
        "session".to_string(),
        None,
        format!("{reason} ({id})"),
    );
}

/// The provider and model a session last used.
///
/// An explicit selection wins over the route of the last request: it is the
/// choice the next request would follow.
fn last_route(events: &[SessionEvent]) -> Option<AgentOptions> {
    let mut selected = None;
    let mut requested = None;
    for event in events {
        match &event.data {
            SessionEventData::ModelSelection { provider, model } => {
                selected = Some(AgentOptions {
                    provider: provider.clone(),
                    model: model.clone(),
                    max_tokens: None,
                });
            }
            SessionEventData::RequestHeader { header } => {
                requested = Some(AgentOptions {
                    provider: header.config.provider.clone(),
                    model: header.config.model.clone(),
                    max_tokens: header.config.max_tokens,
                });
            }
            _ => {}
        }
    }
    selected.or(requested)
}
