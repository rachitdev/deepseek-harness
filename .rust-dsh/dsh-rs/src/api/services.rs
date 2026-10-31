//! Service interfaces — the dependency-injected seams of dsh-rs.
//!
//! Consumers depend on THIS crate (the interface) and never on the
//! implementation crates (`dsh-llm`, `dsh-session`, ...). Each service is a
//! cheap-clone wrapper around a trait object, provided on the cordis context
//! by its implementing plugin:
//!
//! ```text
//! consumer  ---depends on-->  dsh-api (trait + wrapper)
//!     ^                              ^
//!     |                       implements / provides
//!     +-- ctx.require::<LlmService>()--+
//!                                      |
//!                              dsh-llm (implementation plugin)
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::types::{
    AgentCancelCause, AgentOptions, AgentStatus, BoxStream, EpochHeader, GenerateOptions, LlmError,
    LlmProviderInfo, Message, PromptAdmission, PromptAssembly, PromptTarget, QueueAction,
    QueueOutcome, SessionEvent,
    SessionEventData, StreamChunk, ToolExecutionResult, ToolSchema,
};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Service names
// ---------------------------------------------------------------------------

pub const LLM_SERVICE: &str = "llm";
pub const LLM_STREAMS_SERVICE: &str = "llmStreams";
pub const SESSIONS_SERVICE: &str = "sessions";
pub const TOOLS_SERVICE: &str = "tools";
pub const SYSTEM_PROMPT_SERVICE: &str = "systemPrompt";
pub const AGENTS_SERVICE: &str = "agents";
pub const SESSION_PERSISTENCE_SERVICE: &str = "sessionPersistence";
pub const MANIFEST_SERVICE: &str = "manifest";
/// The installed skills, as a [`crate::plugins::skills::SkillCatalog`].
pub const SKILLS_SERVICE: &str = "skills";

/// `BoxFuture` re-export for interface method signatures.
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

// ---------------------------------------------------------------------------
// Session view
// ---------------------------------------------------------------------------

/// The read/write surface of a session, as seen by consumers. Implemented by
/// `crate::session::Session`; consumers never need the concrete type.
pub trait SessionView: Send + Sync + 'static {
    fn id(&self) -> &str;
    fn header_cwd(&self) -> Option<String>;
    /// The session this one was forked or delegated from, when it has one.
    fn parent_session(&self) -> Option<String>;
    /// When the session was created, in Unix epoch milliseconds.
    fn header_created_at(&self) -> u64;
    /// The agent preset the session runs: the last recorded selection, else
    /// the preset the session was created with.
    fn agent_preset(&self) -> Option<String>;
    /// Whether a turn has started. A session that has not started one is
    /// blank: it can still change its composition.
    fn has_started_turn(&self) -> bool {
        self.events()
            .iter()
            .any(|event| matches!(event.data, SessionEventData::TurnStart { .. }))
    }
    /// Choose the preset this session runs, when it is still blank. The
    /// blankness check, the revision check and the record happen together, so
    /// a turn cannot start between them. Returns the preset the session runs,
    /// or `None` when the session has started a turn and cannot be reused.
    fn select_preset(
        &self,
        agent_preset: &str,
        revision: Option<crate::types::PresetRevision>,
    ) -> Option<String>;
    /// Bind the preset instructions this session runs.
    fn bind_preset(&self, binding: crate::types::PresetBinding);
    /// The preset instructions this session is bound to.
    fn preset_binding(&self) -> Option<crate::types::PresetBinding>;
    /// The instructions this session sends to the model, when it has any.
    fn preset_instructions(&self) -> Option<String>;
    fn events(&self) -> Vec<SessionEvent>;
    fn surface(&self) -> Vec<u64>;
    fn derive_messages(&self) -> Vec<Message>;
    /// The model-visible events in order, as one consistent snapshot.
    fn surface_events(&self) -> Vec<SessionEvent>;
    fn append(&self, data: SessionEventData) -> SessionEvent;
    fn request_header(&self) -> Option<EpochHeader>;
    /// The open turn number, if any.
    fn open_turn(&self) -> Option<u64>;
}

// ---------------------------------------------------------------------------
// Interfaces
// ---------------------------------------------------------------------------

/// A durability backend for session logs, implemented by `dsh-session`'s
/// JSONL backend.
pub trait SessionPersistenceApi: Send + Sync + 'static {
    /// A session was created or entered; see
    /// [`crate::session::SessionPersistence::on_session_open`].
    fn on_session_open(&self, _session: &str, _header: &crate::types::SessionHeader, _seed: &[SessionEvent]) {}
    fn on_event(&self, session: &str, event: &SessionEvent);
    fn flush(&self, session: &str) -> std::io::Result<()>;
    fn load(&self, session: &str) -> std::io::Result<Vec<SessionEvent>>;
    /// The header of a stored session, when the backend keeps one.
    fn header(&self, _session: &str) -> std::io::Result<Option<crate::types::SessionHeader>> {
        Ok(None)
    }
    fn list(&self) -> Vec<String>;
    /// The stored log as uncompressed JSONL in the TS session-log format, or
    /// `None` when none is stored. Backends that do not keep that format
    /// report `Unsupported`.
    fn log_text(&self, _session: &str) -> std::io::Result<Option<String>> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "this session backend does not keep logs in the TS session-log format",
        ))
    }
}

/// The session store service (`ctx.sessions`) implemented by `dsh-session`.
pub trait SessionStoreApi: Send + Sync + 'static {
    fn create(&self, options: crate::types::CreateSessionOptions) -> Arc<dyn SessionView>;
    /// Create a session that resumes a stored log: the stored header is kept
    /// and the events are the session's own history.
    fn resume(
        &self,
        id: String,
        header: Option<crate::types::SessionHeader>,
        events: Vec<SessionEvent>,
    ) -> Arc<dyn SessionView>;
    fn get(&self, id: &str) -> Option<Arc<dyn SessionView>>;
    fn list(&self) -> Vec<Arc<dyn SessionView>>;
    fn remove(&self, id: &str) -> Option<Arc<dyn SessionView>>;
    fn fork(
        &self,
        source_id: &str,
        boundary: Option<u64>,
        child_id: Option<String>,
    ) -> Result<Arc<dyn SessionView>, String>;
    fn flush(&self, session_id: &str) -> BoxFuture<Result<(), String>>;
    fn attach_persistence(&self, backend: Arc<dyn SessionPersistenceApi>);
}

/// A model-wire adapter (`ctx.llm` route provider), implemented by
/// `dsh-llm`'s adapters (mock, openai-compatible).
pub trait LlmAdapterApi: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>>;
}

/// The LLM runtime service (`ctx.llm`) implemented by `dsh-llm`.
pub trait LlmRuntimeApi: Send + Sync + 'static {
    fn list_providers(&self) -> Vec<LlmProviderInfo>;
    fn has_provider(&self, name: &str) -> bool;
    fn register_adapter(
        &self,
        providers: &[&str],
        adapter: Arc<dyn LlmAdapterApi>,
    ) -> Result<(), LlmError>;
    fn unregister_adapter(&self, providers: &[&str]);
    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>>;
}

/// An externally supplied tool (e.g. from a dynamically loaded plugin):
/// schema plus the execution callback. Lets the host adapt foreign code
/// through the interface layer without exposing the concrete `ToolDefinition`.
pub struct DynamicToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    /// `arguments` is the parsed JSON args; returns the model-facing result.
    pub exec: Arc<dyn Fn(Value) -> BoxFuture<ToolExecutionResult> + Send + Sync>,
    /// When set, called instead of `exec` with the calling agent's run context
    /// (agent id, cwd, cancellation, kernel handle).
    pub exec_ctx: Option<
        Arc<dyn Fn(Value, crate::types::ToolRunContext) -> BoxFuture<ToolExecutionResult> + Send + Sync>,
    >,
}

/// The tool registry service (`ctx.tools`) implemented by `dsh-tools`.
pub trait ToolRegistryApi: Send + Sync + 'static {
    fn schemas(&self) -> Vec<ToolSchema>;
    fn list(&self) -> Vec<String>;
    fn execute(
        &self,
        call_id: String,
        name: String,
        arguments: Value,
        run_ctx: crate::types::ToolRunContext,
    ) -> BoxFuture<ToolExecutionResult>;
    fn register_dynamic_tool(&self, spec: DynamicToolSpec);
    fn unregister_dynamic_tool(&self, name: &str);
}

/// The system-prompt service (`ctx.systemPrompt`) implemented by `dsh-core`.
pub trait SystemPromptApi: Send + Sync + 'static {
    fn section(&self, name: &str, order: i32, text: &str, complete: bool);
    /// Remove a section by name; returns whether it existed.
    fn remove_section(&self, name: &str) -> bool;
    fn variable(
        &self,
        name: &str,
        provider: crate::types::VariableProvider,
    );
    fn assemble(&self) -> PromptAssembly;
    /// Replace the mounted preset instructions. The snapshot is replaced
    /// whole, so a reader never sees a roster half mounted. A preset whose
    /// instructions changed receives a new revision; a session bound to the
    /// old revision keeps running it until it selects the preset again.
    fn set_preset_prompts(&self, roster: Vec<(String, String)>);
    /// The revision a preset's instructions belong to, when it has any.
    fn preset_revision(&self, preset: &str) -> Option<crate::types::PresetRevision>;
}

/// The live agent handles held by the agent registry (`ctx.agents`).
pub trait AgentView: Send + Sync + 'static {
    fn id(&self) -> &str;
    fn session(&self) -> Arc<dyn SessionView>;
    fn followup(&self, message: Message);
    fn steer(&self, message: Message);
    fn inject(&self, message: Message);
    fn cancel(&self, cause: AgentCancelCause, keep_inbox: bool);
    fn status(&self) -> AgentStatus;
    /// The options the next request is built from.
    fn options(&self) -> AgentOptions;
    /// Use this provider and model from the next request on.
    fn select_model(&self, provider: String, model: String);
    /// Apply one mutation to a pending queue item: replace its content, drop
    /// it, or steer the running turn with it.
    fn update_queue(&self, id: &str, action: QueueAction) -> QueueOutcome;
    /// Queue one prompt unless its client identity was already accepted, so a
    /// retried prompt is not queued twice.
    fn admit_prompt(&self, message: Message, target: PromptTarget) -> PromptAdmission;
    /// Whether a prompt with this client identity is pending or already logged.
    fn has_prompt_request(&self, rpc_id: &str) -> bool;
    fn driver_busy(&self) -> bool;
    fn when_idle(&self) -> BoxFuture<()>;
}

/// The agent registry service (`ctx.agents`) implemented by `dsh-core`.
pub trait AgentRegistryApi: Send + Sync + 'static {
    fn create(
        &self,
        id: Option<String>,
        options: AgentOptions,
        cwd: Option<String>,
        seed_prompt: Option<String>,
        agent_preset: Option<String>,
    ) -> Result<Arc<dyn AgentView>, String>;
    /// Fork a live agent's session at an inclusive `boundary` seq, or at the
    /// latest completed turn when none is given. `Ok(None)` means the source has
    /// no completed turn to fork from.
    fn fork(
        &self,
        source_id: &str,
        boundary: Option<u64>,
        options: AgentOptions,
    ) -> Result<Option<Arc<dyn AgentView>>, String>;
    /// Adopt a stored session and run its agent over it, so a session from an
    /// earlier run appears again.
    fn resume(
        &self,
        id: String,
        header: Option<crate::types::SessionHeader>,
        events: Vec<SessionEvent>,
        options: AgentOptions,
    ) -> Result<Arc<dyn AgentView>, String>;
    fn get(&self, id: &str) -> Option<Arc<dyn AgentView>>;
    fn list(&self) -> Vec<Arc<dyn AgentView>>;
    fn dispose(&self, agent: &Arc<dyn AgentView>);
}

// ---------------------------------------------------------------------------
// Service wrappers (the values provided on the cordis context)
// ---------------------------------------------------------------------------

macro_rules! wrapper {
    ($name:ident, $trait:ident) => {
        /// Cheap-clone handle to the [`$trait`] service.
        #[derive(Clone)]
        pub struct $name {
            pub inner: Arc<dyn $trait>,
        }

        impl $name {
            pub fn new(inner: Arc<dyn $trait>) -> Self {
                $name { inner }
            }
        }

        impl std::ops::Deref for $name {
            type Target = dyn $trait;
            fn deref(&self) -> &Self::Target {
                &*self.inner
            }
        }
    };
}

wrapper!(SessionService, SessionStoreApi);
wrapper!(LlmService, LlmRuntimeApi);
wrapper!(ToolsService, ToolRegistryApi);
wrapper!(SystemPromptService, SystemPromptApi);
wrapper!(AgentRegistryService, AgentRegistryApi);

/// Cheap-clone handle to the manifest registry service.
#[derive(Clone)]
pub struct ManifestService {
    pub inner: Arc<dyn super::manifest::ManifestApi>,
}

impl ManifestService {
    pub fn new(inner: Arc<dyn super::manifest::ManifestApi>) -> Self {
        ManifestService { inner }
    }
}

impl std::ops::Deref for ManifestService {
    type Target = dyn super::manifest::ManifestApi;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}