//! Endpoint dispatch: one method name in, one result or failure out.
//!
//! The router owns no transport. `server` decodes an envelope and hands the
//! [`ClientRequest`] here, which keeps every endpoint testable without a socket.

use std::sync::Arc;

use cordis::Context;
use serde_json::{json, Value};

use super::wire::{codes, ClientRequest, Failure, RpcResult};
use crate::api::services::{AgentRegistryService, AgentView};
use crate::types::{
    AgentOptions, AgentStatus, ContentBlock, InboxTarget, Message, MessageSource, PromptAdmission,
    PromptTarget, QueueAction, QueueOutcome, Role, SessionEvent,
};

/// Model a new session runs when the client does not choose one.
#[derive(Debug, Clone)]
pub struct DefaultModel {
    /// Provider id, e.g. `omniroute`.
    pub provider: String,
    /// Model id inside that provider.
    pub model: String,
}

/// One agent preset the deployment offers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPreset {
    /// The id a request names.
    pub id: String,
    /// The name the client shows.
    pub name: String,
    /// What the preset is for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The preset a request without one runs.
    #[serde(default)]
    pub is_default: bool,
    /// The instructions the preset contributes to the request. They reach the
    /// model while a session runs the preset, and the request header logs them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

/// The roster a deployment offers when its profile names none.
pub fn default_presets() -> Vec<AgentPreset> {
    vec![AgentPreset {
        id: "default".to_string(),
        name: "Default".to_string(),
        description: Some("The dsh-rs agent loop with the tools its profile enables.".to_string()),
        is_default: true,
        system_prompt: None,
    }]
}

/// Mount each preset's instructions into the prompt service, so a session
/// running a preset sends them.
///
/// The mounted instructions are replaced, so a roster that no longer offers a
/// preset leaves no instructions behind. A roster that names instructions
/// without a prompt service to hold them is a misconfiguration and fails.
pub fn mount_preset_prompts(
    ctx: &cordis::Context,
    presets: &[AgentPreset],
) -> Result<(), String> {
    let roster: Vec<(String, String)> = presets
        .iter()
        .map(|preset| {
            (
                preset.id.clone(),
                preset.system_prompt.clone().unwrap_or_default(),
            )
        })
        .collect();
    let Some(service) =
        ctx.get::<crate::api::services::SystemPromptService>(crate::api::SYSTEM_PROMPT_SERVICE)
    else {
        if roster.iter().any(|(_, text)| !text.is_empty()) {
            return Err("gui: the roster names preset instructions, but no prompt service holds them"
                .to_string());
        }
        return Ok(());
    };
    service.set_preset_prompts(roster);
    Ok(())
}

/// What the router needs from the running harness.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Model for sessions created through the GUI.
    pub default_model: DefaultModel,
    /// The agent presets the deployment offers.
    pub agent_presets: Vec<super::router::AgentPreset>,
    /// Longest `session/page` the router will serialize in one response.
    pub max_page_events: usize,
    /// Most hits one `session/search` returns; more set `hasMore`.
    pub max_search_results: usize,
    /// Longest title, in UTF-8 bytes, `session/rename` accepts.
    pub max_title_bytes: usize,
    /// Inclusive byte cap on one file page's text.
    pub max_bytes: usize,
    /// Inclusive byte cap on a complete-file read.
    pub max_file_bytes: usize,
    /// Default and largest page size, in lines, of one file read.
    pub max_lines: usize,
    /// Cap on the entries one directory listing returns.
    pub max_entries: usize,
    /// Round cap a new goal gets when its request names none.
    pub default_max_goal_rounds: u64,
    /// DEFLATE level (0-9) of each entry in a session-log export.
    pub export_compression_level: u8,
    /// Routes offered besides the default one. An adapter cannot enumerate its
    /// models, so the catalog and `session/selectModel` know only these.
    pub models: Vec<DefaultModel>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        RouterConfig {
            agent_presets: super::router::default_presets(),
            default_model: DefaultModel {
                provider: "omniroute".to_string(),
                model: "cc/claude-sonnet-5-5".to_string(),
            },
            max_page_events: 500,
            max_search_results: 20,
            max_title_bytes: 120,
            max_bytes: 2 * 1024 * 1024,
            max_file_bytes: 32 * 1024 * 1024,
            max_lines: 5_000,
            max_entries: 2_000,
            default_max_goal_rounds: 256,
            export_compression_level: super::export::DEFAULT_COMPRESSION_LEVEL,
            models: Vec::new(),
        }
    }
}

/// Largest integer a JavaScript client reads back exactly (`Number.MAX_SAFE_INTEGER`).
pub const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;

/// The endpoint table.
pub struct Router {
    pub(super) config: RouterConfig,
    events: std::sync::Arc<super::events::EventHub>,
    workspaces: std::sync::Arc<super::workspaces::WorkspaceRegistry>,
    pub(super) settings: std::sync::Arc<super::settings::SettingsStore>,
    pub(super) goals: super::goals::GoalRuntime,
    pub(super) desktop: std::sync::Arc<super::desktop::Desktop>,
    /// The native directory picker, when the deployment has one.
    pub(super) picker: Option<std::sync::Arc<super::picker::Picker>>,
    /// The office document service, when the deployment has one.
    pub(super) office: Option<std::sync::Arc<super::office::OfficeService>>,
    /// The analytics policy, when the deployment has one.
    pub(super) product_analytics: Option<std::sync::Arc<super::product_analytics::ProductAnalyticsService>>,
    /// The card service, when the deployment has one.
    pub(super) cards: Option<std::sync::Arc<super::cards::CardsService>>,
    /// The live-sharing service, when the deployment has one.
    pub(super) liveshare: Option<std::sync::Arc<super::liveshare::LiveshareService>>,
    /// The plugin manager, when the deployment has one.
    pub(super) plugin_manager: Option<std::sync::Arc<super::plugin_manager::PluginManager>>,
    /// The plugin market, when the deployment has one.
    pub(super) plugin_market: Option<std::sync::Arc<super::plugin_market::PluginMarket>>,
    /// The Platform account, when the deployment has one.
    pub(super) account: Option<std::sync::Arc<super::account::Account>>,
    pub(super) registry_probe: std::sync::Arc<super::registry_probe::RegistryProbe>,
    pub(super) feedback: super::feedback::Feedback,
    pub(super) discovery: std::sync::Arc<super::model_discovery::Discovery>,
    uploads: std::sync::Arc<super::uploads::Uploads>,
    images: std::sync::Arc<super::images::ImageAdmission>,
    pub(super) commands: std::sync::Arc<super::commands::Commands>,
    credentials: std::sync::Arc<super::credentials::CredentialStore>,
    pub(super) terminal: Option<std::sync::Arc<crate::terminal::service::TerminalService>>,
    schedule: Option<std::sync::Arc<crate::schedule::service::ScheduleService>>,
}

impl Router {
    /// Build a router with the given configuration.
    pub fn new(config: RouterConfig) -> Self {
        Router {
            config,
            terminal: None,
            events: std::sync::Arc::default(),
            workspaces: std::sync::Arc::default(),
            settings: std::sync::Arc::default(),
            goals: super::goals::GoalRuntime::default(),
            picker: None,
            office: None,
            product_analytics: None,
            cards: None,
            liveshare: None,
            plugin_manager: None,
            plugin_market: None,
            account: None,
            desktop: std::sync::Arc::new(super::desktop::Desktop::system(
                None,
                std::time::Duration::from_secs(30),
            )),
            registry_probe: std::sync::Arc::new(
                super::registry_probe::RegistryProbe::system(std::time::Duration::from_secs(5))
                    .expect("the default registry probe client builds"),
            ),
            feedback: super::feedback::Feedback::new(8_192),
            discovery: std::sync::Arc::new(
                super::model_discovery::Discovery::new(std::time::Duration::from_secs(15))
                    .expect("the default model discovery client builds"),
            ),
            uploads: std::sync::Arc::new(
                super::uploads::Uploads::new(None, 32 * 1024 * 1024)
                    .expect("the default upload limit is positive"),
            ),
            images: std::sync::Arc::default(),
            commands: std::sync::Arc::new(super::commands::Commands::new(None)),
            // No file: a router built without a deployment never touches a home directory.
            credentials: std::sync::Arc::new(super::credentials::CredentialStore::new(None)),
            // Reminders are held in memory until a deployment supplies a store.
            schedule: Some(crate::schedule::service::ScheduleService::new(
                crate::schedule::store::Store::in_memory(),
                crate::schedule::store::Retention::default(),
            )),
        }
    }

    /// Use `service` for the terminal controller endpoints.
    pub fn with_terminal(mut self, service: std::sync::Arc<crate::terminal::service::TerminalService>) -> Self {
        self.terminal = Some(service);
        self
    }

    /// The analytics policy, when the deployment has one.
    pub fn product_analytics(&self) -> Option<std::sync::Arc<super::product_analytics::ProductAnalyticsService>> {
        self.product_analytics.clone()
    }

    /// The terminal controller, when the deployment has one.
    pub fn terminal(&self) -> Option<std::sync::Arc<crate::terminal::service::TerminalService>> {
        self.terminal.clone()
    }

    /// Use `service` for `schedule/*`.
    pub fn with_schedule(mut self, service: std::sync::Arc<crate::schedule::service::ScheduleService>) -> Self {
        self.schedule = Some(service);
        self
    }

    /// The reminder service, when the deployment has one.
    pub(super) fn schedule(&self) -> Option<std::sync::Arc<crate::schedule::service::ScheduleService>> {
        self.schedule.clone()
    }

    /// Use `store` for `credentials/*`.
    pub fn with_credentials(mut self, store: std::sync::Arc<super::credentials::CredentialStore>) -> Self {
        self.credentials = store;
        self
    }

    /// Use `commands` for the slash-command layer.
    pub fn with_commands(mut self, commands: std::sync::Arc<super::commands::Commands>) -> Self {
        self.commands = commands;
        self
    }

    /// Refuse feedback notes longer than `bytes` UTF-8 bytes; a limit of zero
    /// would refuse every note and is rejected.
    pub fn with_max_note_bytes(mut self, bytes: usize) -> Result<Self, String> {
        if bytes == 0 {
            return Err("the feedback note limit must be at least 1 byte".to_string());
        }
        self.feedback = super::feedback::Feedback::new(bytes);
        Ok(self)
    }

    /// Use `probe` for `pluginRegistryProbe/fastest`.
    pub fn with_registry_probe(
        mut self,
        probe: std::sync::Arc<super::registry_probe::RegistryProbe>,
    ) -> Self {
        self.registry_probe = probe;
        self
    }

    /// Use `discovery` for `llm/discoverModels`.
    pub fn with_discovery(mut self, discovery: std::sync::Arc<super::model_discovery::Discovery>) -> Self {
        self.discovery = discovery;
        self
    }

    /// Use `uploads` for file uploads and the receipts prompts redeem.
    pub fn with_uploads(mut self, uploads: std::sync::Arc<super::uploads::Uploads>) -> Self {
        self.uploads = uploads;
        self
    }

    /// Use `images` to admit the images prompts carry.
    pub fn with_images(mut self, images: std::sync::Arc<super::images::ImageAdmission>) -> Self {
        self.images = images;
        self
    }

    /// The DEFLATE level of each entry in a session-log export.
    pub(super) fn export_compression_level(&self) -> u8 {
        self.config.export_compression_level
    }

    /// The image admission policy.
    pub(super) fn images(&self) -> std::sync::Arc<super::images::ImageAdmission> {
        std::sync::Arc::clone(&self.images)
    }

    /// The upload service.
    pub(super) fn uploads(&self) -> &super::uploads::Uploads {
        &self.uploads
    }

    /// Use `service` for the Platform account endpoints.
    pub fn with_account(mut self, service: std::sync::Arc<super::account::Account>) -> Self {
        self.account = Some(service);
        self
    }

    /// Use `service` for the plugin manager endpoints.
    pub fn with_plugin_manager(
        mut self,
        service: std::sync::Arc<super::plugin_manager::PluginManager>,
    ) -> Self {
        self.plugin_manager = Some(service);
        self
    }

    /// Use `service` for the plugin market endpoints.
    pub fn with_plugin_market(
        mut self,
        service: std::sync::Arc<super::plugin_market::PluginMarket>,
    ) -> Self {
        self.plugin_market = Some(service);
        self
    }

    /// Use `service` for the live-sharing endpoints.
    pub fn with_liveshare(mut self, service: std::sync::Arc<super::liveshare::LiveshareService>) -> Self {
        self.liveshare = Some(service);
        self
    }

    /// Use `service` for the analytics endpoints.
    pub fn with_product_analytics(
        mut self,
        service: std::sync::Arc<super::product_analytics::ProductAnalyticsService>,
    ) -> Self {
        self.product_analytics = Some(service);
        self
    }

    /// Use `service` for the card endpoints.
    pub fn with_cards(mut self, service: std::sync::Arc<super::cards::CardsService>) -> Self {
        self.cards = Some(service);
        self
    }

    /// Use `service` for the office document endpoints.
    pub fn with_office(mut self, service: std::sync::Arc<super::office::OfficeService>) -> Self {
        self.office = Some(service);
        self
    }

    /// Use `picker` for `directoryPicker/pick`.
    pub fn with_picker(mut self, picker: std::sync::Arc<super::picker::Picker>) -> Self {
        self.picker = Some(picker);
        self
    }

    /// Use `desktop` for the endpoints that open paths on the host's desktop.
    pub fn with_desktop(mut self, desktop: std::sync::Arc<super::desktop::Desktop>) -> Self {
        self.desktop = desktop;
        self
    }

    /// Use `store` for `settings/*`, e.g. one that persists to a file.
    pub fn with_settings(mut self, store: std::sync::Arc<super::settings::SettingsStore>) -> Self {
        self.settings = store;
        self
    }

    /// Use `registry` for `workspace/*`, e.g. one that persists to a file.
    pub fn with_workspaces(
        mut self,
        registry: std::sync::Arc<super::workspaces::WorkspaceRegistry>,
    ) -> Self {
        self.workspaces = registry;
        self
    }

    /// The caps and routes this router serves with.
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    /// The workspace registry `workspace/follow` streams from.
    pub fn workspaces(&self) -> std::sync::Arc<super::workspaces::WorkspaceRegistry> {
        std::sync::Arc::clone(&self.workspaces)
    }

    /// The questions waiting on open `$events` streams.
    pub fn events(&self) -> std::sync::Arc<super::events::EventHub> {
        std::sync::Arc::clone(&self.events)
    }

    /// Names every registered endpoint, for `GET /api/endpoints` and tests.
    pub fn endpoints() -> &'static [&'static str] {
        &[
            "session/create",
            "session/list",
            "session/page",
            "session/prompt",
            "session/attachment",
            "session/cancel",
            "session/search",
            "session/rename",
            "session/fork",
            "session/selectModel",
            "session/updateQueue",
            "llm/listProviders",
            "llm/listConfigurableProviders",
            "session/projections",
            "skills/list",
            "commands/list",
            "commands/execute",
            "schedule/list",
            "schedule/catalog",
            "schedule/history",
            "schedule/delete",
            "schedule/deleteInactive",
            "schedule/update",
            "terminal/shells",
            "terminal/environment",
            "terminal/list",
            "terminal/create",
            "terminal/close",
            "terminal/rename",
            "directoryPicker/pick",
            "officeToPdf/generation",
            "officeToPdf/render",
            "productAnalytics/enabled",
            "productAnalytics/report",
            "actionCards/act",
            "pluginRecommendations/describe",
            "cotTranslation/limits",
            "cotTranslation/translate",
            "claudeCodeMods/pressBand",
            "cuaAccess/ptcConfiguration",
            "cuaAccess/enablePersistentPtc",
            "cuaAccess/applications",
            "cuaAccess/systemStatus",
            "cuaAccess/applicationIcons",
            "cuaAccess/requestPermission",
            "cuaAccess/openSettings",
            "cuaAccess/revoke",
            "cuaAccess/retryApproval",
            "forgePanel/branches",
            "forgePanel/cancelWorktree",
            "forgePanel/changeRequestDiff",
            "forgePanel/changeRequestText",
            "forgePanel/continueInProject",
            "forgePanel/dismiss",
            "forgePanel/localDiff",
            "forgePanel/localText",
            "forgePanel/markReady",
            "forgePanel/merge",
            "forgePanel/refresh",
            "forgePanel/requestWorktree",
            "forgePanel/restoreWorktree",
            "forgePanel/retryWorktree",
            "forgePanel/stopCreation",
            "forgePanel/track",
            "account/startSignIn",
"pluginManager/cancelInstall",
            "pluginManager/inspect",
            "pluginManager/installBundle",
            "pluginManager/listBundles",
            "pluginManager/listPlugins",
            "pluginManager/listVersionExemptions",
            "pluginManager/listVersions",
            "pluginManager/readBundleDetail",
            "pluginManager/registries",
            "pluginManager/removeBundle",
            "pluginManager/setBundleEnabled",
            "pluginManager/setPluginEnabled",
            "pluginManager/setVersionExemption",
            "pluginManager/updateBundle",
            "pluginManager/waitForInstall",
            "pluginMarket/listIndexes",
            "pluginMarket/readEntryDetail",
            "pluginMarket/search",
            "pluginMarket/setIndexes",
            "speech/cancelPreparation",
            "speech/catalog",
            "speech/configure",
            "speech/prepare",
            "speech/transcribe",
"liveshareShares/audit",
            "liveshareShares/capabilities",
            "liveshareShares/create",
            "liveshareShares/identity",
            "liveshareShares/list",
            "liveshareShares/relay",
            "liveshareShares/setCapabilities",
            "liveshareShares/setRelay",
            "liveshareShares/stop",
            "liveshareShares/testPublish",
            "liveshareShares/testRelay",
            "liveshareShares/updatePermissions",
            "agentPresets/list",
            "agentPresets/read",
            "agentPresets/select",
            "permissionPresets/catalog",
            "session/modelCatalog",
            "settings/describe",
            "settings/mutate",
            "settings/update",
            "settings/replace",
            "credentials/describe",
            "dynamicCordisRunner/inventory",
            "dynamicCordisRunner/syncInspectManifest",
"dynamicCordisRunner/getClientCode",
            "dynamicCordisRunner/invoke",
            "dynamicCordisRunner/reportClientGuardFailure",
            "dynamicCordisRunner/reportRenderFailure",
            "dynamicCordisRunner/resolveInspectQuery",
            "dynamicCordisRunner/resolveRequestRun",
            "dynamicCordisRunner/runHostHalf",
            "dynamicCordisRunner/settleUserRun",
            "dynamicCordisRunner/stopFromPanel",
            "dynamicCordisRunner/undefineFromPanel",
            "workspace/create",
            "workspace/initializeDefault",
            "workspace/rename",
            "workspace/delete",
            "workspace/insertBefore",
            "workspace/insertSessionBefore",
            "workspace/pinSession",
            "workspace/unpinSession",
            "workspace/archiveSession",
            "workspace/unarchiveSession",
            "workspaceFiles/list",
            "sessionReferenceResolver/candidates",
            "workspaceFiles/read",
            "workspaceFiles/readBytes",
            "workspaceFiles/stat",
            "fileReferences/list",
            "directoryPicker/list",
            "directoryPicker/createDirectory",
            "goals/get",
            "goals/create",
            "goals/edit",
            "goals/pause",
            "goals/resume",
            "goals/complete",
            "goals/clear",
            "session/canOpenWorkspacePath",
            "session/workspacePathApplications",
            "session/openWorkspacePath",
            "pluginInventory/list",
            "session/initializeDefaultModel",
            "pluginRegistryProbe/fastest",
            "settings/openSettingsDocument",
            "messageFeedback/list",
            "messageFeedback/put",
            "messageFeedback/delete",
            "sessionFeedback/record",
            "userQuestions/answer",
            "llm/discoverModels",
            "job/kill",
            "fileUploads/upload",
            "session/state",
            "session/events",
            "session/wait",
            "credentials/set",
            "credentials/unset",
            "subagents/prompt",
            "subagents/interruptByParent",
            "account/getState",
            "account/signOut",
            "account/cancelSignIn",
            "account/getProfile",
            "account/getBalance",
            "account/getUnnotifiedBonuses",
            "account/ackBonusNotified",
            "account/hasRunningAccountTasks",
            super::events::RESULT_ENDPOINT,
        ]
    }

    /// Dispatch one request.
    pub async fn dispatch(&self, ctx: &Context, request: &ClientRequest) -> RpcResult {
        let endpoint = request.method.as_str();
        // The shipped client wraps every call as `{ args: { … } }`, and the
        // TypeScript gateway rejects any other payload shape.
        let payload = match unwrap_args(&request.payload) {
            Ok(args) => args,
            Err(reason) => {
                return RpcResult::failed(Failure::at(codes::PROTOCOL, reason, endpoint));
            }
        };
        if let Err(failure) = super::descriptor::check(endpoint, payload) {
            return RpcResult::failed(failure);
        }
        // Unknown properties inside typed parameters never reach a handler.
        let stripped = super::descriptor::strip_args(endpoint, payload);
        let payload = &stripped;
        match endpoint {
            "session/list" => self.session_list(ctx, payload),
            "session/create" => self.session_create(ctx, payload),
            "session/page" => self.session_page(ctx, payload),
            "session/prompt" => self.session_prompt(ctx, payload).await,
            "session/cancel" => self.session_cancel(ctx, payload),
            "session/rename" => self.session_rename(ctx, payload),
            "session/fork" => self.session_fork(ctx, payload),
            "session/selectModel" => self.session_select_model(ctx, payload).await,
            "session/updateQueue" => self.session_update_queue(ctx, payload),
            "session/search" => self.session_search(ctx, payload),
            "llm/listProviders" => self.llm_list_providers(ctx),
            // Provider routes come from the profile, not from settings namespaces.
            "llm/listConfigurableProviders" => RpcResult::ok(json!([])),
            "session/projections" => self.session_projections(ctx, payload),
            "skills/list" => self.skills_list(ctx, payload),
            "commands/list" | "commands/execute" => {
                super::commands::serve(self, ctx, endpoint, payload).await
            }
            "agentPresets/list" => super::catalog::agent_presets(&self.config.agent_presets),
            "agentPresets/read" => self.agent_presets_read(payload),
            "agentPresets/select" => self.agent_presets_select(ctx, payload),
            "permissionPresets/catalog" => super::catalog::permission_presets(ctx),
            "session/modelCatalog" => {
                super::catalog::model_catalog(ctx, &self.config.default_model, &self.config.models)
            }
            "settings/describe" => RpcResult::ok(self.settings.describe()),
            // `settings/describe` says `hasDocument: false`: this host keeps its
            // settings in a store the user does not edit as a file, so there is
            // no document to open, and opening some other file would mislead.
            "settings/openSettingsDocument" => match check_descriptor(endpoint, &[], payload) {
                Ok(()) => unavailable(endpoint, "settings document"),
                Err(failure) => RpcResult::failed(failure),
            },
            "settings/mutate" | "settings/update" | "settings/replace" => {
                self.settings_write(endpoint, payload)
            }
            "dynamicCordisRunner/inventory" => super::catalog::plugin_inventory(),
            "dynamicCordisRunner/syncInspectManifest" => {
                super::catalog::sync_inspect_manifest(payload)
            }
            "workspace/archiveSession" => self.archive_session(ctx, payload).await,
            name if name.starts_with("workspace/") => self.workspace(ctx, name, payload, &[]),
            super::events::RESULT_ENDPOINT => match self.events.result(payload) {
                Ok(()) => RpcResult::ok(Value::Null),
                Err(failure) => RpcResult::failed(failure),
            },
            other => {
                // The endpoint groups that live in their own module answer
                // before the router gives up.
                if let Some(result) = super::files::dispatch(self, ctx, other, payload) {
                    return result;
                }
                if let Some(result) = super::goals::dispatch(self, ctx, other, payload) {
                    return result;
                }
                if let Some(result) = super::office::dispatch(ctx, self.office.as_deref(), other, payload).await {
                    return result;
                }
                if other == "directoryPicker/pick" {
                    return super::picker::pick(self.picker.as_deref()).await;
                }
                if let Some(result) = super::desktop::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if let Some(result) = super::feedback::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if other == "job/kill" {
                    return super::jobs::kill(payload);
                }
                if let Some(result) = super::uploads::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if let Some(result) = super::images::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if let Some(result) = super::model_discovery::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if let Some(result) = super::user_questions::dispatch(ctx, other, payload) {
                    return result;
                }
                if let Some(result) = super::dynamic_runner::dispatch(ctx, other, payload) {
                    return result;
                }
                if let Some(result) = super::inventory::dispatch(self, ctx, other, payload) {
                    return result;
                }
                if let Some(result) = super::registry_probe::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if let Some(result) = super::references::dispatch(self, ctx, other, payload) {
                    return result;
                }
                if let Some(result) = super::session_read::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if let Some(result) = super::credentials::dispatch(&self.credentials, other, payload).await {
                    return result;
                }
                if let Some(result) = super::subagents::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if let Some(account) = &self.account {
                    if let Some(result) = super::account::dispatch_signin(account, other, payload).await {
                        return result;
                    }
                }
                if let Some(result) = super::account::dispatch(ctx, other, payload) {
                    return result;
                }
                if let Some(service) = self.product_analytics.as_deref() {
                    if let Some(result) = service.dispatch(other, payload) {
                        return result;
                    }
                }
                if let Some(service) = self.cards.as_deref() {
                    if let Some(result) = service.dispatch(ctx, other, payload) {
                        return result;
                    }
                }
                if let Some(result) = super::cot_translation::dispatch(&self.settings, other, payload) {
                    return result;
                }
                if let Some(result) = super::speech::dispatch(&self.settings, other, payload) {
                    return result;
                }
                if let Some(result) = super::claude_code_mods::dispatch(ctx, other, payload) {
                    return result;
                }
                if let Some(result) = super::cua_access::dispatch(other, payload) {
                    return result;
                }
                if let Some(service) = self.liveshare.as_deref() {
                    if let Some(result) = service.dispatch(other, payload).await {
                        return result;
                    }
                }
                if let Some(result) = super::forge_panel::dispatch(ctx, other, payload) {
                    return result;
                }
                if let Some(service) = self.plugin_manager.as_deref() {
                    if let Some(result) = service.dispatch(ctx, other, payload) {
                        return result;
                    }
                }
                if let Some(service) = self.plugin_market.as_deref() {
                    if let Some(result) = service.dispatch(other, payload) {
                        return result;
                    }
                }
                if let Some(result) = super::terminal::dispatch(self, ctx, other, payload).await {
                    return result;
                }
                if let Some(result) = super::schedule::dispatch(self, other, payload).await {
                    return result;
                }
                RpcResult::failed(Failure::at(
                    codes::LOOKUP_NOT_FOUND,
                    format!("no endpoint is registered as {other:?}"),
                    endpoint,
                ))
            }
        }
    }

    /// The projection values dsh-rs can compute from a live session. Keys it
    /// cannot compute (token usage, goals as TypeScript models them, todos, a
    /// team) are omitted rather than reported as empty.
    fn session_projections(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/projections";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let Some(session_id) = request.get("sessionId").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "request.sessionId",
                "sessionId must be a string",
            ));
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(agent) = agents.get(session_id) else {
            return RpcResult::ok(Value::Null);
        };
        let session = agent.session();
        let as_of = session.events().last().map_or(-1, |event| event.seq as i64);
        RpcResult::ok(json!({
            "kind": "sequenced",
            "asOfSeq": as_of,
            "values": match projection_values(
                ctx,
                &*session,
                &selected_route(&*agent),
                &self.config.agent_presets,
                self.images.limits(),
            ) {
                Ok(values) => values,
                Err(reason) => {
                    return RpcResult::failed(Failure::at(
                        codes::INTERNAL,
                        reason,
                        "session/projections",
                    ))
                }
            },
        }))
    }

    /// The skills the model can load with the `skill` tool. dsh-rs does not
    /// expand `/name` in the composer, so every entry is model-invocable.
    /// Read one preset's declaration.
    ///
    /// The TypeScript host serves the preset's child plugin list as YAML; this
    /// build composes one agent loop for every preset, so the list is empty
    /// and a preset's instructions live in the prompt service.
    fn agent_presets_read(&self, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "agentPresets/read";
        let Some(preset) = argument(payload, "agentPreset", ENDPOINT).and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "agentPreset",
                "agentPreset must be a string",
            ));
        };
        let Some(declared) = self
            .config
            .agent_presets
            .iter()
            .find(|candidate| candidate.id == preset)
        else {
            return RpcResult::failed(preset_not_found(ENDPOINT, preset));
        };
        let mut document = json!({
            "agentPreset": declared.id,
            "name": declared.name,
            "content": preset_document(),
        });
        if let Some(description) = &declared.description {
            document["description"] = json!(description);
        }
        RpcResult::ok(document)
    }

    /// Select the preset a session runs, before it starts its first turn.
    fn agent_presets_select(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "agentPresets/select";
        let Some(session_id) = argument(payload, "agentId", ENDPOINT).and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "agentId", "agentId must be a string"));
        };
        let Some(preset) = argument(payload, "agentPreset", ENDPOINT).and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "agentPreset",
                "agentPreset must be a string",
            ));
        };
        if !self
            .config
            .agent_presets
            .iter()
            .any(|candidate| candidate.id == preset)
        {
            return RpcResult::failed(preset_not_found(ENDPOINT, preset));
        }
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(agent) = agents.get(session_id) else {
            return session_not_found(session_id);
        };
        let revision = ctx
            .get::<crate::api::services::SystemPromptService>(crate::api::SYSTEM_PROMPT_SERVICE)
            .and_then(|service| service.preset_revision(preset));
        let session = agent.session();
        match session.select_preset(preset, revision) {
            Some(committed) => RpcResult::ok(json!(committed)),
            None => RpcResult::failed(Failure {
                code: "agent-preset/locked".to_string(),
                message: "This session has already started".to_string(),
                details: json!({ "endpoint": ENDPOINT, "sessionId": session_id, "agentPreset": preset }),
            }),
        }
    }

    fn skills_list(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "skills/list";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        if request.get("sessionId").and_then(Value::as_str).is_none() {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "request.sessionId",
                "sessionId must be a string",
            ));
        }
        let skills: Vec<Value> = ctx
            .get::<crate::plugins::skills::SkillCatalog>(crate::api::SKILLS_SERVICE)
            .map(|catalog| {
                catalog
                    .skills
                    .iter()
                    .map(|skill| {
                        json!({
                            "name": skill.name,
                            "description": skill.description,
                            "path": skill.dir.join("SKILL.md").display().to_string(),
                            "modelInvocable": true,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        RpcResult::ok(json!({ "skills": skills }))
    }

    fn session_cancel(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/cancel";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let Some(session_id) = request.get("sessionId").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "request.sessionId",
                "sessionId must be a string",
            ));
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(agent) = agents.get(session_id) else {
            return session_not_found(session_id);
        };
        // Queued follow-ups are dropped too: a stop button means stop.
        agent.cancel(crate::types::AgentCancelCause::User, false);
        RpcResult::ok(json!({ "accepted": true }))
    }

    /// Append a user-owned title, as the TypeScript `rename` does: the text is
    /// cleaned and cut to `max_title_bytes`, and one that is empty afterwards is
    /// refused with `session/title-invalid`.
    fn session_rename(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/rename";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let Some(session_id) = request.get("sessionId").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request.sessionId", "sessionId must be a string"));
        };
        let Some(raw) = request.get("title").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request.title", "title must be a string"));
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(agent) = agents.get(session_id) else {
            return session_not_found(session_id);
        };
        let title = crate::session::title::normalize(raw, self.config.max_title_bytes);
        if title.is_empty() {
            return RpcResult::failed(Failure {
                code: "session/title-invalid".to_string(),
                message: "session title must contain visible characters".to_string(),
                details: json!({ "sessionId": session_id }),
            });
        }
        let event = agent.session().append(crate::types::SessionEventData::SessionTitle {
            title: title.clone(),
        });
        RpcResult::ok(json!({ "title": title, "seq": event.seq }))
    }

    /// Fork one live session into a new one. `atSeq` is the inclusive cut; without
    /// it the latest completed turn is kept. A cut inside an open turn is refused,
    /// since the child would start mid-turn.
    fn session_fork(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/fork";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let Some(session_id) = request.get("sessionId").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request.sessionId", "sessionId must be a string"));
        };
        let at_seq = match request.get("atSeq") {
            None | Some(Value::Null) => None,
            Some(value) => match value.as_u64().filter(|seq| *seq <= MAX_SAFE_INTEGER) {
                Some(seq) => Some(seq),
                None => {
                    return RpcResult::failed(Failure::at(
                        codes::BAD_REQUEST,
                        "atSeq must be a non-negative safe integer",
                        ENDPOINT,
                    ))
                }
            },
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(source) = agents.get(session_id) else {
            return session_not_found(session_id);
        };
        let unavailable_fork = |message: String| {
            RpcResult::failed(Failure {
                code: "session/fork-unavailable".to_string(),
                message,
                details: json!({ "sessionId": session_id }),
            })
        };
        if let Some(seq) = at_seq {
            let last = source.session().events().last().map(|e| e.seq);
            if last.map_or(true, |last| seq > last) {
                return unavailable_fork(format!(
                    "event {seq} does not exist in session {session_id:?} (last seq: {})",
                    last.map_or("none".to_string(), |l| l.to_string())
                ));
            }
        }
        let options = AgentOptions {
            provider: self.config.default_model.provider.clone(),
            model: self.config.default_model.model.clone(),
            max_tokens: None,
        };
        match agents.fork(session_id, at_seq, options) {
            Ok(Some(child)) => RpcResult::ok(json!({ "sessionId": child.id() })),
            Ok(None) => unavailable_fork(format!("session {session_id:?} has no completed turn to fork from")),
            // A cut inside an open turn is the caller's choice of boundary, not a fault.
            Err(message) if message.contains("ends inside open turn") => unavailable_fork(message),
            // The source can leave the registry between the lookup and the fork.
            Err(_) if agents.get(session_id).is_none() => session_not_found(session_id),
            Err(message) => RpcResult::failed(Failure::at(codes::INTERNAL, message, ENDPOINT)),
        }
    }

    /// Use another route for this session's next request. The route must be one
    /// the catalog offers and its provider must be registered; anything else is
    /// refused as `session/model-unavailable` and changes nothing.
    async fn session_select_model(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/selectModel";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let text = |field: &str| request.get(field).and_then(Value::as_str);
        let (Some(session_id), Some(provider), Some(model)) =
            (text("sessionId"), text("provider"), text("model"))
        else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "request",
                "sessionId, provider and model must be strings",
            ));
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(agent) = agents.get(session_id) else {
            return session_not_found(session_id);
        };
        let refuse = |message: String| {
            RpcResult::failed(Failure {
                code: "session/model-unavailable".to_string(),
                message,
                details: json!({ "provider": provider, "model": model }),
            })
        };
        if !matches!(request.get("reasoningEffort"), None | Some(Value::Null)) {
            return refuse("dsh-rs sends no reasoning setting, so reasoningEffort cannot be applied".to_string());
        }
        let offered = std::iter::once(&self.config.default_model)
            .chain(&self.config.models)
            .any(|route| route.provider == provider && route.model == model);
        if !offered {
            return refuse(format!("{provider}/{model} is not an offered model"));
        }
        let Some(llm) = ctx.get::<crate::api::LlmService>(crate::api::LLM_SERVICE) else {
            return unavailable(ENDPOINT, "llm");
        };
        if !llm.has_provider(provider) {
            return refuse(format!("provider {provider:?} is not registered"));
        }
        // Waits for image admissions already running, so a prompt's model gate
        // and the route it is sent to cannot differ.
        let _serialized = self.images.serialize(session_id).await;
        agent.select_model(provider.to_string(), model.to_string());
        RpcResult::ok(json!({ "selected": { "provider": provider, "model": model } }))
    }

    /// Mutate one still-pending queue item: replace its text, drop it, or
    /// steer the running turn with it. The TypeScript controller validates the
    /// edit content before it resolves the agent, and reports a vanished item
    /// as `session/queue-item-not-found` so the client converges silently.
    fn session_update_queue(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/updateQueue";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let Some(session_id) = request.get("sessionId").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "request.sessionId",
                "sessionId must be a string",
            ));
        };
        let Some(item_id) = request.get("itemId").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "request.itemId",
                "itemId must be a string",
            ));
        };
        let Some(action) = request.get("action").and_then(Value::as_object) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "request.action",
                "action must be an object",
            ));
        };
        let Some(kind) = action.get("kind").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "request.action.kind",
                "action kind must be a string",
            ));
        };
        // A queue edit carries text only.
        let action = match kind {
            "edit" => {
                let Some(content) = action.get("content").and_then(Value::as_array) else {
                    return RpcResult::failed(input_invalid(
                        ENDPOINT,
                        "request.action.content",
                        "content must be an array",
                    ));
                };
                let mut blocks = Vec::with_capacity(content.len());
                for (index, item) in content.iter().enumerate() {
                    if item.get("type").and_then(Value::as_str) != Some("text") {
                        return RpcResult::failed(Failure {
                            code: "session/attachment-invalid".to_string(),
                            message: "queue edits accept text content only".to_string(),
                            details: json!({ "reason": "QUEUE_EDIT_NON_TEXT" }),
                        });
                    }
                    let Some(text) = item.get("text").and_then(Value::as_str) else {
                        return RpcResult::failed(input_invalid(
                            ENDPOINT,
                            &format!("request.action.content[{index}].text"),
                            "a text block needs a string \"text\"",
                        ));
                    };
                    blocks.push(ContentBlock::text(text));
                }
                if !blocks
                    .iter()
                    .any(|block| !block.as_text().unwrap_or_default().trim().is_empty())
                {
                    return RpcResult::failed(Failure::at(
                        codes::BAD_REQUEST,
                        "queue edit content must include non-whitespace text",
                        ENDPOINT,
                    ));
                }
                QueueAction::Edit(blocks)
            }
            "remove" => QueueAction::Remove,
            "steer" => QueueAction::Steer,
            other => {
                return RpcResult::failed(input_invalid(
                    ENDPOINT,
                    "request.action.kind",
                    format!("action kind must be \"edit\", \"remove\" or \"steer\", got {other:?}"),
                ));
            }
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(agent) = agents.get(session_id) else {
            return queue_item_not_found(item_id);
        };
        // The agent applies the mutation and reports what it did, so an item
        // the driver claimed in between is reported rather than ignored.
        let removing = matches!(action, QueueAction::Remove);
        match agent.update_queue(item_id, action) {
            QueueOutcome::Applied => {
                // The queue item id is the message id the receipts were bound under.
                if removing {
                    self.uploads().retire(session_id, item_id);
                }
                RpcResult::ok(json!({ "accepted": true }))
            }
            QueueOutcome::NotFound => queue_item_not_found(item_id),
            QueueOutcome::SteerUnavailable => RpcResult::failed(Failure {
                code: "session/steer-unavailable".to_string(),
                message: "current turn no longer accepts steering".to_string(),
                details: json!({ "itemId": item_id }),
            }),
        }
    }

    fn session_search(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/search";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let query = match request.get("query").and_then(Value::as_str) {
            Some(query) if !query.trim().is_empty() => query.trim().to_lowercase(),
            _ => {
                return RpcResult::failed(bad_request(
                    ENDPOINT,
                    "session search query must not be empty",
                ));
            }
        };
        if query.encode_utf16().count() > SEARCH_QUERY_MAX_CHARS {
            return RpcResult::failed(bad_request(
                ENDPOINT,
                format!(
                    "session search query must contain at most {SEARCH_QUERY_MAX_CHARS} UTF-16 code units"
                ),
            ));
        }
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        // Every visible session, newest first: the live ones in memory, the
        // stored ones from their log.
        let mut hits: Vec<(u64, String, String)> = Vec::new();
        for agent in agents.list() {
            // The TypeScript host searches the sessions it can see: a session
            // without a working directory is not visible.
            if agent.session().header_cwd().is_none() {
                continue;
            }
            let Some(snippet) = first_match(&agent.session().events(), &query) else {
                continue;
            };
            hits.push((session_updated_at(agent.as_ref()), agent.id().to_string(), snippet));
        }
        if let Some(persistence) = persistence(ctx) {
            for (id, header) in stored_headers(ctx, &agents) {
                let Some(header) = header else { continue };
                if header.cwd.is_none() {
                    continue;
                }
                let Ok(events) = persistence.load(&id) else {
                    continue;
                };
                let Some(snippet) = first_match(&events, &query) else {
                    continue;
                };
                hits.push((header.created_at, id, snippet));
            }
        }
        hits.sort_by(|left, right| right.0.cmp(&left.0));
        let mut items = Vec::new();
        let mut has_more = false;
        for (_, session_id, snippet) in hits {
            if items.len() == self.config.max_search_results {
                has_more = true;
                break;
            }
            items.push(json!({ "sessionId": session_id, "snippet": snippet }));
        }
        RpcResult::ok(json!({ "items": items, "hasMore": has_more }))
    }

    fn llm_list_providers(&self, ctx: &Context) -> RpcResult {
        const ENDPOINT: &str = "llm/listProviders";
        let Some(llm) = ctx.get::<crate::api::services::LlmService>(crate::api::LLM_SERVICE) else {
            return unavailable(ENDPOINT, "llm");
        };
        let providers: Vec<Value> = llm
            .list_providers()
            .into_iter()
            .map(|provider| json!({ "id": provider.id, "name": provider.name }))
            .collect();
        RpcResult::ok(Value::Array(providers))
    }

    fn session_list(&self, ctx: &Context, payload: &Value) -> RpcResult {
        if let Err(failure) = expect_object("_request", payload, "session/list") {
            return RpcResult::failed(failure);
        }
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable("session/list", "agents");
        };
        let mut items: Vec<Value> = agents
            .list()
            .iter()
            .map(|agent| session_row(agent.as_ref()))
            .collect();
        // The stored sessions no live agent covers: a log the runtime could
        // not read is still a session the user must see.
        for (id, header, readable) in stored_sessions(ctx, &agents) {
            items.push(cold_session_row(&id, header.as_ref(), readable));
        }
        // The TypeScript host orders the rows by activity, newest first.
        items.sort_by(|left, right| row_updated_at(right).cmp(&row_updated_at(left)));
        RpcResult::ok(json!({ "items": items }))
    }

    fn session_create(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/create";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        if let Err(failure) = expect_object("request", request, ENDPOINT) {
            return RpcResult::failed(failure);
        }
        let mut cwd = match request.get("cwd") {
            None | Some(Value::Null) => None,
            Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
            Some(_) => {
                return RpcResult::failed(input_invalid(ENDPOINT, "cwd", "cwd must be a path"))
            }
        };
        match request.get("workspaceId") {
            None | Some(Value::Null) => {}
            Some(Value::String(id)) if cwd.is_none() => match self.workspaces.path_of(id) {
                Ok(path) => cwd = Some(path),
                Err(failure) => return RpcResult::failed(failure),
            },
            Some(Value::String(_)) => {
                return RpcResult::failed(input_invalid(
                    ENDPOINT,
                    "workspaceId",
                    "workspaceId and cwd are exclusive",
                ))
            }
            Some(_) => {
                return RpcResult::failed(input_invalid(
                    ENDPOINT,
                    "workspaceId",
                    "workspaceId must be a string",
                ))
            }
        };
        let session_id = match request.get("sessionId") {
            None | Some(Value::Null) => None,
            Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
            Some(_) => {
                return RpcResult::failed(input_invalid(
                    ENDPOINT,
                    "sessionId",
                    "sessionId must be a non-empty string",
                ))
            }
        };
        // The preset the request names, or the roster's default.
        let preset = match request.get("agentPreset") {
            None | Some(Value::Null) => default_preset_id(&self.config.agent_presets),
            Some(Value::String(id)) => {
                if !self.config.agent_presets.iter().any(|p| p.id == *id) {
                    let available = self
                        .config
                        .agent_presets
                        .iter()
                        .map(|p| p.id.clone())
                        .collect::<Vec<String>>();
                    let mut failure = Failure::at(
                        AGENT_PRESET_NOT_FOUND,
                        format!("Unknown agent preset: {id}"),
                        ENDPOINT,
                    );
                    failure.details = json!({
                        "endpoint": ENDPOINT,
                        "agentPreset": id,
                        "available": available,
                    });
                    return RpcResult::failed(failure);
                }
                Some(id.clone())
            }
            Some(_) => {
                return RpcResult::failed(input_invalid(
                    ENDPOINT,
                    "agentPreset",
                    "agentPreset must be a string",
                ))
            }
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let options = AgentOptions {
            provider: self.config.default_model.provider.clone(),
            model: self.config.default_model.model.clone(),
            max_tokens: None,
        };
        // A named session that is already live is reused while it is blank, and
        // otherwise a fresh id is issued, as the TypeScript host does ("the
        // created or reused Session, which may differ from the requested one").
        let session_id = match session_id.as_deref().and_then(|id| agents.get(id)) {
            // Only when it sits where the caller asked: a blank session in
            // another directory would run the new work against the wrong files.
            Some(existing)
                if is_blank(&*existing)
                    && cwd.as_ref().is_none_or(|wanted| {
                        existing.session().header_cwd().as_deref() == Some(wanted.as_str())
                    }) =>
            {
                // A blank session may change the preset it runs; the change is
                // recorded, so the log says which composition ran. The check
                // and the record happen together: a session that started a
                // turn is not reused, so a fresh session runs the request, as
                // the TypeScript host does when its preset change is refused.
                let session = existing.session();
                let running = session.agent_preset();
                let reusable = match (preset.as_deref(), running.as_deref()) {
                    // The TypeScript host returns the agent as it is when it
                    // already runs the requested preset: nothing is recorded
                    // and the composition is not recomposed.
                    (Some(wanted), Some(current)) if wanted == current => true,
                    (Some(wanted), _) => {
                        let revision = ctx
                            .get::<crate::api::services::SystemPromptService>(
                                crate::api::SYSTEM_PROMPT_SERVICE,
                            )
                            .and_then(|service| service.preset_revision(wanted));
                        session.select_preset(wanted, revision).is_some()
                    }
                    (None, _) => !session.has_started_turn(),
                };
                if reusable {
                    return RpcResult::ok(json!({
                        "sessionId": existing.id(),
                        "agentPreset": session.agent_preset(),
                    }));
                }
                None
            }
            Some(_) => None,
            None => session_id,
        };
        match agents.create(session_id, options, cwd, None, preset.clone()) {
            Ok(agent) => {
                self.workspaces.sessions_changed(&session_refs(ctx));
                RpcResult::ok(json!({
                    "sessionId": agent.id(),
                    "agentPreset": agent.session().agent_preset(),
                }))
            }
            Err(reason) => RpcResult::failed(Failure::at(codes::INTERNAL, reason, ENDPOINT)),
        }
    }

    async fn session_prompt(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/prompt";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let Some(session_id) = request.get("sessionId").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "sessionId",
                "sessionId is required",
            ));
        };
        let Some(content) = request.get("content").and_then(Value::as_array) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "content",
                "content must be an array",
            ));
        };
        let parts = match content_parts(content, ENDPOINT) {
            Ok(parts) => parts,
            Err(failure) => return RpcResult::failed(failure),
        };
        // The TypeScript controller refuses a prompt that carries no
        // non-whitespace text and no attachment.
        if !parts.iter().any(|part| match part {
            Part::Text(text) => !text.trim().is_empty(),
            Part::Receipt(_) | Part::Image(_) => true,
        }) {
            return RpcResult::failed(Failure {
                code: codes::BAD_REQUEST.to_string(),
                message: "prompt content must include non-whitespace text or an attachment"
                    .to_string(),
                details: json!({}),
            });
        }
        let steer = match request.get("mode").and_then(Value::as_str) {
            Some("steer") => true,
            Some("queue") | None => false,
            Some(other) => {
                return RpcResult::failed(input_invalid(
                    ENDPOINT,
                    "mode",
                    format!("mode must be \"queue\" or \"steer\", got {other:?}"),
                ))
            }
        };
        // The client-minted prompt identity; it rides on the accepted message
        // so the client can match its own echo to the queued row.
        let request_id = match request.get("requestId") {
            None => None,
            Some(value) => match value.as_str() {
                Some(id) if !id.is_empty() => Some(id.to_string()),
                _ => {
                    return RpcResult::failed(input_invalid(
                        ENDPOINT,
                        "requestId",
                        "requestId must be a non-empty string",
                    ))
                }
            },
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(agent) = agents.get(session_id) else {
            return session_not_found(session_id);
        };
        // A retried prompt is accepted again without queueing a second copy, and
        // before its receipts are looked up: the first copy already retired them.
        if request_id.as_deref().is_some_and(|id| agent.has_prompt_request(id)) {
            return RpcResult::ok(json!({ "accepted": true }));
        }
        let uploads = self.uploads();
        let images = self.images();
        // The model gate comes first, and image admissions for one session
        // run one at a time, so prompts keep their order into the log.
        let _serialized = if parts.iter().any(|part| matches!(part, Part::Image(_))) {
            let route = selected_route(&*agent);
            if let Err(failure) = images.require_model(&route.provider, &route.model) {
                return RpcResult::failed(failure);
            }
            Some(images.serialize(session_id).await)
        } else {
            None
        };
        let mut wire_images = Vec::new();
        let mut slots = Vec::with_capacity(parts.len());
        let mut receipts = Vec::new();
        for part in parts {
            match part {
                Part::Text(text) => slots.push(Slot::Block(ContentBlock::text(text))),
                Part::Receipt(receipt) => {
                    let Some(attachment) = uploads.resolve(session_id, &receipt) else {
                        return RpcResult::failed(super::uploads::file_not_staged());
                    };
                    slots.push(Slot::Block(ContentBlock::File { attachment }));
                    receipts.push(receipt);
                }
                Part::Image(image) => {
                    slots.push(Slot::Image(wire_images.len()));
                    wire_images.push(image);
                }
            }
        }
        let mut admitted = if wire_images.is_empty() {
            Vec::new()
        } else {
            let policy = std::sync::Arc::clone(&images);
            let result = tokio::task::spawn_blocking(move || policy.admit(&wire_images)).await;
            match result {
                Ok(Ok(refs)) => refs,
                Ok(Err(error)) => {
                    if let Some(cause) = &error.cause {
                        ctx.logger().log_event(
                            cordis::logger::LogLevel::Warn,
                            "gui".to_string(),
                            None,
                            format!("image admission: {}: {cause}", error.code),
                        );
                    }
                    return RpcResult::failed(super::uploads::attachment_failure(
                        error.code,
                        &error.message,
                    ));
                }
                Err(error) => {
                    return RpcResult::failed(Failure::at(
                        codes::INTERNAL,
                        format!("image admission failed: {error}"),
                        ENDPOINT,
                    ))
                }
            }
        }
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
        let blocks: Vec<ContentBlock> = slots
            .into_iter()
            .map(|slot| match slot {
                Slot::Block(block) => block,
                Slot::Image(index) => ContentBlock::Image {
                    attachment: admitted[index].take().expect("each image slot is used once"),
                },
            })
            .collect();
        // The session may have been disposed while the images were stored.
        if agents.get(session_id).is_none() {
            return session_not_found(session_id);
        }
        let message = Message {
            id: format!("gui-{}", crate::plugins::unique_id()),
            role: Role::User,
            content: blocks,
            source: MessageSource::User { rpc_id: request_id },
        };
        let message_id = message.id.clone();
        let Some(binding) = uploads.bind(session_id, &receipts, &message_id) else {
            return RpcResult::failed(super::uploads::file_not_staged());
        };
        let target = if steer {
            PromptTarget::Step
        } else {
            PromptTarget::Turn
        };
        match agent.admit_prompt(message, target) {
            PromptAdmission::Queued => RpcResult::ok(json!({ "accepted": true })),
            PromptAdmission::Duplicate => {
                // The first copy owns the receipts; this message id never lands.
                uploads.rollback(session_id, binding);
                RpcResult::ok(json!({ "accepted": true }))
            }
            PromptAdmission::Refused => {
                uploads.rollback(session_id, binding);
                RpcResult::failed(Failure::at(
                    codes::INTERNAL,
                    "the prompt identity is already pending",
                    ENDPOINT,
                ))
            }
        }
    }

    fn session_page(&self, ctx: &Context, payload: &Value) -> RpcResult {
        const ENDPOINT: &str = "session/page";
        let Some(request) = argument(payload, "request", ENDPOINT) else {
            return RpcResult::failed(input_invalid(ENDPOINT, "request", "request is required"));
        };
        let Some(address) = request.get("address") else {
            return RpcResult::failed(input_invalid(ENDPOINT, "address", "address is required"));
        };
        let Some(session_id) = address.get("sessionId").and_then(Value::as_str) else {
            return RpcResult::failed(input_invalid(
                ENDPOINT,
                "address.sessionId",
                "only { kind: \"session\" } addresses are served by this build",
            ));
        };
        let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
            return unavailable(ENDPOINT, "agents");
        };
        let Some(agent) = agents.get(session_id) else {
            return session_not_found(session_id);
        };
        let events = agent.session().events();
        let through = request.get("throughSeq").and_then(Value::as_u64);
        let mut records = RecordEncoder::records(&events);
        let mut visible: Vec<SessionEvent> = events
            .into_iter()
            .filter(|event| through.is_none_or(|limit| event.seq <= limit))
            .collect();
        records.truncate(visible.len());
        if visible.len() > self.config.max_page_events {
            // Keep the newest window. The client polls this for the live
            // transcript, so the oldest events are the ones it can afford to
            // lose; keeping them would freeze the view at session start.
            let dropped = visible.len() - self.config.max_page_events;
            visible.drain(..dropped);
            records.drain(..dropped);
        }
        RpcResult::ok(json!({ "records": records }))
    }
}

/// Characters of context kept on each side of a search hit.
const SNIPPET_CONTEXT: usize = 60;
/// The longest search query the TypeScript host accepts, in UTF-16 code units.
const SEARCH_QUERY_MAX_CHARS: usize = 500;

/// The first user or assistant text in `events` containing `needle` (already
/// lowercased), as a snippet with a little context around the hit.
fn first_match(events: &[SessionEvent], needle: &str) -> Option<String> {
    use crate::types::SessionEventData as D;
    events.iter().find_map(|event| {
        let message = match &event.data {
            D::UserMessage { message } | D::AssistantMessage { message, .. } => message,
            _ => return None,
        };
        let text = message.text();
        let chars: Vec<char> = text.chars().collect();
        let lowered: Vec<char> = text.to_lowercase().chars().collect();
        // Lowercasing can change length for a few scripts; skip those rather than mis-slice.
        if lowered.len() != chars.len() {
            return None;
        }
        let needle: Vec<char> = needle.chars().collect();
        let at = lowered
            .windows(needle.len())
            .position(|window| window == needle.as_slice())?;
        let start = at.saturating_sub(SNIPPET_CONTEXT);
        let end = (at + needle.len() + SNIPPET_CONTEXT).min(chars.len());
        Some(chars[start..end].iter().collect())
    })
}

/// Turns session events into the records the browser reads.
///
/// The browser validates each record with the TypeScript session reader, so a
/// record is the v4 row for the event: TypeScript-shaped `data`, plus the
/// `surfaceOp`, `sourceEventSeqs` and `ignorable` envelope keys that reader
/// requires. A tool result cites its call by seq, so one encoder must see a
/// session's events in order, from the start.
#[derive(Debug, Default)]
pub(super) struct RecordEncoder {
    state: crate::session::v4::EncodeState,
}

impl RecordEncoder {
    /// The record for the next event in the session's log.
    pub(super) fn record(&mut self, event: &SessionEvent) -> Value {
        let row = match crate::session::v4::encode_event(event, &mut self.state) {
            Ok(row) => row,
            Err(error) => unencodable_row(event, &error.to_string()),
        };
        json!({ "type": "event", "event": row })
    }

    /// Records for a whole log, in order.
    pub(super) fn records(events: &[SessionEvent]) -> Vec<Value> {
        let mut encoder = RecordEncoder::default();
        events.iter().map(|event| encoder.record(event)).collect()
    }
}

/// The record for an event the v4 projection cannot express.
///
/// The seq must stay dense, and the TypeScript reader accepts a gap-filling row
/// only under a type it does not know, marked ignorable. A known type with
/// the wrong `data` (or a message type without `surfaceOp`) would fail the
/// whole page instead.
pub(super) fn unencodable_row(event: &SessionEvent, reason: &str) -> crate::session::v4::V4Row {
    crate::session::v4::V4Row {
        kind: format!("dsh-rs/unencodable/{}", event.event_type()),
        seq: event.seq,
        time: event.time,
        data: json!({ "reason": reason }),
        surface_op: None,
        source_event_seqs: None,
        ignorable: Some(true),
    }
}

/// One `session/list` row.
/// Whether the session is blank: a session is blank until a turn starts.
fn is_blank(agent: &dyn AgentView) -> bool {
    !agent.session().has_started_turn()
}

/// The persistence backend the deployment stores its sessions in.
pub(super) fn persistence(
    ctx: &Context,
) -> Option<Arc<dyn crate::api::services::SessionPersistenceApi>> {
    ctx.get::<Arc<dyn crate::api::services::SessionPersistenceApi>>(
        crate::api::services::SESSION_PERSISTENCE_SERVICE,
    )
    .map(|backend| backend.as_ref().clone())
}

/// The stored sessions no live agent covers, with their stored header.
fn stored_headers(
    ctx: &Context,
    agents: &AgentRegistryService,
) -> Vec<(String, Option<crate::types::SessionHeader>)> {
    let Some(persistence) = persistence(ctx) else {
        return Vec::new();
    };
    let mut stored = Vec::new();
    for id in persistence.list() {
        if agents.get(&id).is_some() {
            continue;
        }
        stored.push((id.clone(), persistence.header(&id).ok().flatten()));
    }
    stored
}

/// The stored sessions no live agent covers, with their stored header and
/// whether their log can be read.
pub(super) fn stored_sessions(
    ctx: &Context,
    agents: &AgentRegistryService,
) -> Vec<(String, Option<crate::types::SessionHeader>, bool)> {
    let Some(persistence) = persistence(ctx) else {
        return Vec::new();
    };
    let mut stored = Vec::new();
    for id in persistence.list() {
        if agents.get(&id).is_some() {
            continue;
        }
        let header = persistence.header(&id).ok().flatten();
        let readable = persistence.load(&id).is_ok();
        stored.push((id, header, readable));
    }
    stored
}

/// The stored header of one session.
pub(super) fn stored_header(ctx: &Context, session_id: &str) -> Option<crate::types::SessionHeader> {
    persistence(ctx)?.header(session_id).ok().flatten()
}

/// The newest `session/title` text in a stored session's log.
pub(super) fn stored_title(ctx: &Context, session_id: &str) -> Option<String> {
    let persistence = persistence(ctx)?;
    let events = persistence.load(session_id).ok()?;
    events.iter().rev().find_map(|event| match &event.data {
        crate::types::SessionEventData::SessionTitle { title, .. } => Some(title.clone()),
        _ => None,
    })
}

/// One row for a stored session no live agent covers.
///
/// The TypeScript host serves a cold row from the stored record: the agent is
/// not available, the row is not blank because its metadata is unknown, and a
/// log that cannot be read needs a migration.
fn cold_session_row(
    id: &str,
    header: Option<&crate::types::SessionHeader>,
    readable: bool,
) -> Value {
    let mut row = json!({
        "sessionId": id,
        "agentAvailable": false,
        "running": false,
        "blank": false,
        "updatedAt": header.map(|header| header.created_at).unwrap_or(0),
        "formatStatus": if readable { "current" } else { "migration-required" },
    });
    set_cwd(&mut row, header.and_then(|header| header.cwd.clone()));
    if let Some(parent) = header.and_then(|header| header.parent_session.as_ref()) {
        row["parentSessionId"] = json!(parent);
    }
    row
}

/// When a session was last active: the later of its creation and its last
/// prompt, as the TypeScript host orders the list. A session nobody has
/// prompted keeps its creation time, so the page never shows an age of 1970.
fn session_updated_at(agent: &dyn AgentView) -> u64 {
    let session = agent.session();
    let last_prompt = session
        .events()
        .iter()
        .rev()
        .find(|event| matches!(event.data, crate::types::SessionEventData::UserMessage { .. }))
        .map_or(0, |event| event.time);
    session.header_created_at().max(last_prompt)
}

/// The activity time of a list row.
fn row_updated_at(row: &Value) -> u64 {
    row.get("updatedAt").and_then(Value::as_u64).unwrap_or(0)
}

fn session_row(agent: &dyn AgentView) -> Value {
    let session = agent.session();
    let blank = is_blank(agent);
    let updated_at = session_updated_at(agent);
    let mut row = json!({
        "sessionId": agent.id(),
        "agentAvailable": true,
        "running": agent.status() == AgentStatus::Running,
        "blank": blank,
        "updatedAt": updated_at,
        "formatStatus": "current",
    });
    set_cwd(&mut row, session.header_cwd());
    row
}

/// A session without a working directory has no `cwd` key: the page derives the
/// workspace title from the path and cannot take `null`.
fn set_cwd(row: &mut Value, cwd: Option<String>) {
    if let Some(cwd) = cwd {
        row["cwd"] = json!(cwd);
    }
}

/// One prompt content block as the wire carries it, before receipts resolve.
enum Part {
    Text(String),
    /// An image, still base64 text.
    Image(super::images::WireImage),
    /// A receipt minted by `fileUploads/upload` in this session.
    Receipt(String),
}

/// A prompt block once its receipts and images are resolved, or the place an
/// image's reference goes when admission has produced it.
enum Slot {
    Block(ContentBlock),
    Image(usize),
}

/// Decode the `content` array. A block the wire contract does not describe is
/// a boundary failure on `request`, as the TypeScript gateway reports it; an
/// image block is well formed but not served by this build.
fn content_parts(content: &[Value], endpoint: &str) -> Result<Vec<Part>, Failure> {
    let mut parts = Vec::with_capacity(content.len());
    for item in content {
        match item.get("type").and_then(Value::as_str) {
            Some("text") => match item.get("text").and_then(Value::as_str) {
                Some(text) => parts.push(Part::Text(text.to_string())),
                None => return Err(boundary(endpoint, "request")),
            },
            Some("file") => match item.get("receiptId").and_then(Value::as_str) {
                Some(receipt) => parts.push(Part::Receipt(receipt.to_string())),
                None => return Err(boundary(endpoint, "request")),
            },
            Some("image") => parts.push(Part::Image(wire_image(item, endpoint)?)),
            _ => return Err(boundary(endpoint, "request")),
        }
    }
    Ok(parts)
}

/// One `image` part: a declared type, base64 data and an optional name. The
/// wire accepts four types; which of them a deployment admits is decided later.
pub(super) fn wire_image(item: &Value, endpoint: &str) -> Result<super::images::WireImage, Failure> {
    let object = item.as_object().ok_or_else(|| boundary(endpoint, "request"))?;
    // Unknown keys are ignored, as the TypeScript wire decoder ignores them.
    let text = |key: &str| object.get(key).and_then(Value::as_str).map(str::to_string);
    let media_type = text("mediaType")
        .filter(|kind| crate::images::MEDIA_TYPES.contains(&kind.as_str()))
        .ok_or_else(|| boundary(endpoint, "request"))?;
    let data = text("data").ok_or_else(|| boundary(endpoint, "request"))?;
    let name = match object.get("name") {
        None => None,
        Some(Value::String(name)) => Some(name.clone()),
        Some(_) => return Err(boundary(endpoint, "request")),
    };
    Ok(super::images::WireImage { media_type, data, name })
}

/// One `image` part of a request, as the wire carries it.
pub(super) fn wire_image_part(item: &Value, endpoint: &str) -> Result<super::images::WireImage, Failure> {
    wire_image(item, endpoint)
}

pub(super) fn argument<'a>(payload: &'a Value, name: &str, _endpoint: &str) -> Option<&'a Value> {
    match payload.get(name) {
        Some(Value::Null) | None => None,
        Some(value) => Some(value),
    }
}

pub(super) fn expect_object(field: &str, value: &Value, endpoint: &str) -> Result<(), Failure> {
    if value.is_object() {
        return Ok(());
    }
    Err(input_invalid(
        endpoint,
        field,
        format!("{field} must be an object"),
    ))
}

/// The TypeScript host's answer to a request that cannot be served as asked.
pub(super) fn bad_request(endpoint: &str, message: impl Into<String>) -> Failure {
    Failure {
        code: codes::BAD_REQUEST.to_string(),
        message: message.into(),
        details: json!({ "endpoint": endpoint }),
    }
}

pub(super) fn input_invalid(endpoint: &str, field: &str, message: impl Into<String>) -> Failure {
    Failure {
        code: codes::INPUT_INVALID.to_string(),
        message: message.into(),
        details: json!({ "endpoint": endpoint, "field": field }),
    }
}

/// A wire field that failed validation of its value.
pub(super) fn boundary(endpoint: &str, field: &str) -> Failure {
    Failure {
        code: codes::INPUT_INVALID.to_string(),
        message: format!(
            "typert gateway: {endpoint}: wire field \"{field}\" failed boundary validation"
        ),
        details: json!({ "endpoint": endpoint, "field": field }),
    }
}

/// The payload must carry exactly the endpoint's fields.
pub(super) fn check_descriptor(endpoint: &str, fields: &[&str], payload: &Value) -> Result<(), Failure> {
    let object = payload.as_object();
    let present = |name: &str| object.is_some_and(|o| o.contains_key(name));
    let missing: Vec<_> = fields.iter().filter(|name| !present(name)).collect();
    let unexpected: Vec<_> = object
        .into_iter()
        .flat_map(|o| o.keys())
        .filter(|key| !fields.contains(&key.as_str()))
        .collect();
    if missing.is_empty() && unexpected.is_empty() {
        return Ok(());
    }
    let quote = |names: Vec<String>| {
        names.iter().map(|n| format!("{n:?}")).collect::<Vec<_>>().join(", ")
    };
    let mut parts = Vec::new();
    if !missing.is_empty() {
        parts.push(format!("missing {}", quote(missing.iter().map(|n| n.to_string()).collect())));
    }
    if !unexpected.is_empty() {
        parts.push(format!("unexpected {}", quote(unexpected.iter().map(|n| n.to_string()).collect())));
    }
    Err(Failure {
        code: codes::ARGUMENTS_INVALID.to_string(),
        message: format!(
            "typert gateway: {endpoint}: args fields do not match the descriptor: {}",
            parts.join("; ")
        ),
        details: json!({ "endpoint": endpoint }),
    })
}

/// The TypeScript host's answer to a preset this deployment does not offer.
fn preset_not_found(endpoint: &str, preset: &str) -> Failure {
    Failure {
        code: "agent-preset/not-found".to_string(),
        message: format!("Unknown agent preset: {preset}"),
        details: json!({ "endpoint": endpoint, "agentPreset": preset }),
    }
}

/// One preset's declaration, as the YAML the TypeScript host serves.
///
/// The TypeScript host serves the preset's declared child plugin list as
/// entry-list YAML. A dsh-rs preset declares no child plugins: the
/// composition is the same agent loop for every preset, and a preset
/// contributes its instructions to the prompt instead. The list is therefore
/// empty, which is what the client renders.
fn preset_document() -> String {
    "[]\n".to_string()
}

/// Check that `session_id` names a session this host knows, loaded or stored,
/// in the order the TypeScript host does: an empty id fails while the host builds
/// its storage path, an unknown id is `session/not-found`.
pub(super) fn require_session(ctx: &Context, endpoint: &str, session_id: &str) -> Result<(), Failure> {
    if session_id.is_empty() {
        return Err(Failure {
            code: codes::INTERNAL.to_string(),
            message: "resume failed for session \"\": SessionQueryError: failed to observe \
                      session \"\": cannot encode an empty path segment"
                .to_string(),
            details: json!({}),
        });
    }
    let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
        return match unavailable(endpoint, "agents") {
            RpcResult::Failed(failure) => Err(failure),
            _ => unreachable!("unavailable always answers a failure"),
        };
    };
    let known = agents.get(session_id).is_some()
        || persistence(ctx).is_some_and(|backend| backend.list().iter().any(|id| id == session_id));
    if known {
        return Ok(());
    }
    match session_not_found(session_id) {
        RpcResult::Failed(failure) => Err(failure),
        _ => unreachable!("session_not_found always answers a failure"),
    }
}

/// The TypeScript host's answer to any call naming a session it does not have;
/// the client branches on this code.
pub(super) fn session_not_found(session_id: &str) -> RpcResult {
    RpcResult::failed(Failure {
        code: "session/not-found".to_string(),
        message: format!("session {session_id:?} not found"),
        details: json!({ "sessionId": session_id }),
    })
}

/// A queue item that is no longer pending. The client drops the row silently
/// on this code, so an unknown session reports it too.
fn queue_item_not_found(item_id: &str) -> RpcResult {
    RpcResult::failed(Failure {
        code: "session/queue-item-not-found".to_string(),
        message: "queued item is no longer pending".to_string(),
        details: json!({ "itemId": item_id }),
    })
}

pub(super) fn unavailable(endpoint: &str, service: &str) -> RpcResult {
    RpcResult::failed(Failure::at(
        codes::SERVICE_UNAVAILABLE,
        format!("the {service} service is not wired into this build"),
        endpoint,
    ))
}

/// The `args` object of a client payload, which must be exactly `{ args: {…} }`.
pub(crate) fn unwrap_args(payload: &Value) -> Result<&Value, String> {
    let wrapped = payload
        .as_object()
        .filter(|object| object.len() == 1)
        .and_then(|object| object.get("args"))
        .filter(|args| args.is_object());
    wrapped
        .ok_or_else(|| "payload must contain exactly one object field named \"args\"".to_string())
}

/// Every live session as the workspace registry sees it.
pub(super) fn session_refs(ctx: &Context) -> Vec<super::workspaces::SessionRef> {
    let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
        return Vec::new();
    };
    agents
        .list()
        .iter()
        .map(|agent| super::workspaces::SessionRef {
            id: agent.id().to_string(),
            cwd: agent.session().header_cwd(),
            running: agent.status() == AgentStatus::Running,
        })
        .collect()
}

impl Router {
    /// Archive a session. Reminders armed for it count as activity (the archive is
    /// refused, naming them, unless `stopActivity` is set), and a stopping archive
    /// deletes them, as the TypeScript schedule service does on
    /// `workspace/session-activity` and `workspace/session-stop`.
    async fn archive_session(&self, ctx: &Context, payload: &Value) -> RpcResult {
        let endpoint = "workspace/archiveSession";
        let session = argument(payload, "request", endpoint)
            .and_then(|r| r.get("sessionId"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let stop = argument(payload, "request", endpoint)
            .and_then(|r| r.get("stopActivity"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let (Some(session), Some(schedule)) = (session, self.schedule()) else {
            return self.workspace(ctx, endpoint, payload, &[]);
        };
        let reminders: Vec<Value> = schedule.activity_of(&session).await.into_iter().collect();
        let result = self.workspace(ctx, endpoint, payload, &reminders);
        if stop && matches!(result, RpcResult::Ok(_)) {
            // A failure is reported and never keeps the session visible.
            if let Err(reason) = schedule.stop_session(&session).await {
                ctx.logger().log_event(
                    cordis::logger::LogLevel::Warn,
                    "workspace".to_string(),
                    None,
                    format!("workspace: stopping session '{session}' for archive failed: {reason:?}"),
                );
            }
        }
        result
    }

    /// `activity` is what providers outside the registry report for an archive
    /// (armed reminders); only `workspace/archiveSession` reads it.
    fn workspace(&self, ctx: &Context, endpoint: &str, payload: &Value, activity: &[Value]) -> RpcResult {
        let sessions = session_refs(ctx);
        let registry = &self.workspaces;
        let text = |field: &str| -> Result<String, Failure> {
            argument(payload, "request", endpoint)
                .and_then(|r| r.get(field))
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| {
                    input_invalid(
                        endpoint,
                        &format!("request.{field}"),
                        format!("{field} must be a string"),
                    )
                })
        };
        let optional = |field: &str| -> Result<Option<String>, Failure> {
            match argument(payload, "request", endpoint).and_then(|r| r.get(field)) {
                None | Some(Value::Null) => Ok(None),
                Some(Value::String(text)) => Ok(Some(text.clone())),
                Some(_) => Err(input_invalid(
                    endpoint,
                    &format!("request.{field}"),
                    format!("{field} must be a string"),
                )),
            }
        };
        let outcome: Result<Value, Failure> = (|| match endpoint {
            "workspace/create" => registry
                .create(&text("path")?, &sessions)
                .map(|(workspace, created)| json!({ "workspace": workspace, "created": created })),
            "workspace/initializeDefault" => registry.initialize_default(&sessions).map(|found| {
                found.map_or(Value::Null, |workspace| json!({ "workspace": workspace }))
            }),
            "workspace/rename" => registry
                .rename(&text("workspaceId")?, &text("title")?, &sessions)
                .map(|workspace| json!({ "workspace": workspace })),
            "workspace/delete" => registry
                .delete(&text("workspaceId")?, &sessions)
                .map(|()| json!({ "deleted": true })),
            "workspace/insertBefore" => registry
                .insert_before(
                    &text("workspaceId")?,
                    optional("beforeWorkspaceId")?.as_deref(),
                    &sessions,
                )
                .map(|ids| json!({ "workspaceIds": ids })),
            "workspace/insertSessionBefore" => registry
                .insert_session_before(
                    &text("workspaceId")?,
                    &text("sessionId")?,
                    optional("beforeSessionId")?.as_deref(),
                    &sessions,
                )
                .map(|workspace| json!({ "workspace": workspace })),
            "workspace/pinSession" => registry
                .pin(&text("sessionId")?, &sessions)
                .map(|ids| json!({ "pinnedSessionIds": ids })),
            "workspace/unpinSession" => registry
                .unpin(&text("sessionId")?, &sessions)
                .map(|ids| json!({ "pinnedSessionIds": ids })),
            "workspace/archiveSession" => {
                let session_id = text("sessionId")?;
                let stop = argument(payload, "request", endpoint)
                    .and_then(|r| r.get("stopActivity"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let (ids, was_running) = registry.archive(&session_id, stop, &sessions, activity)?;
                if was_running {
                    if let Some(agents) =
                        ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                    {
                        if let Some(agent) = agents.get(&session_id) {
                            agent.cancel(crate::types::AgentCancelCause::User, false);
                        }
                    }
                }
                Ok(json!({ "archivedSessionIds": ids }))
            }
            "workspace/unarchiveSession" => registry
                .unarchive(&text("sessionId")?, &sessions)
                .map(|ids| json!({ "archivedSessionIds": ids })),
            other => Err(Failure::at(
                codes::LOOKUP_NOT_FOUND,
                format!("no endpoint is registered as {other:?}"),
                other,
            )),
        })();
        match outcome {
            Ok(value) => RpcResult::ok(value),
            Err(failure) => RpcResult::failed(failure),
        }
    }
}

impl Router {
    fn settings_write(&self, endpoint: &str, payload: &Value) -> RpcResult {
        let args = payload;
        let ns = match args.get("ns").and_then(Value::as_str) {
            Some(ns) => ns,
            None => return RpcResult::failed(input_invalid(endpoint, "ns", "ns must be a string")),
        };
        let expected = match args.get("expectedRevision") {
            None | Some(Value::Null) => None,
            Some(value) => match value.as_u64() {
                Some(revision) => Some(revision),
                None => {
                    return RpcResult::failed(input_invalid(
                        endpoint,
                        "expectedRevision",
                        "expectedRevision must be a non-negative integer",
                    ))
                }
            },
        };
        let object = |field: &str| -> Result<&serde_json::Map<String, Value>, Failure> {
            args.get(field).and_then(Value::as_object).ok_or_else(|| {
                input_invalid(endpoint, field, format!("{field} must be an object"))
            })
        };
        let outcome = match endpoint {
            "settings/mutate" => match args.get("ops").and_then(Value::as_array) {
                Some(ops) => self.settings.mutate(ns, ops, expected),
                None => Err(input_invalid(endpoint, "ops", "ops must be an array")),
            },
            "settings/update" => object("patch").and_then(|p| self.settings.update(ns, p, expected)),
            _ => object("section").and_then(|s| self.settings.replace(ns, s, expected)),
        };
        match outcome {
            Ok(view) => RpcResult::ok(view),
            Err(failure) => RpcResult::failed(failure),
        }
    }
}

/// The route this agent's next request will use.
pub(super) fn selected_route(agent: &dyn crate::api::services::AgentView) -> DefaultModel {
    let options = agent.options();
    DefaultModel {
        provider: options.provider,
        model: options.model,
    }
}

/// The newest `session/title` text in the log, as the TypeScript title
/// projection folds it (`null` until one is written).
fn latest_title(session: &dyn crate::api::services::SessionView) -> Option<String> {
    session.events().into_iter().rev().find_map(|event| match event.data {
        crate::types::SessionEventData::SessionTitle { title, .. } => Some(title),
        _ => None,
    })
}

/// The model selection visible at `through`, with request use from durable
/// request headers and intent from the agent's next route.
pub(super) fn model_selection_values(
    session: &dyn crate::api::services::SessionView,
    next: &DefaultModel,
    through: Option<u64>,
) -> Value {
    let last_used = session.events().iter().rev().find_map(|event| {
        if through.is_some_and(|limit| event.seq > limit) { return None; }
        match &event.data {
            crate::types::SessionEventData::RequestHeader { header } => Some(json!({
                "provider": header.config.provider, "model": header.config.model,
            })),
            _ => None,
        }
    });
    json!({
        "lastUsed": last_used,
        "next": { "provider": next.provider, "model": next.model },
    })
}

/// The pending inbox at `through`, folded from the durable splice events the
/// same way the TypeScript inbox projection folds them. A log that cannot be
/// folded is reported rather than repaired.
pub(super) fn inbox_values(
    session: &dyn crate::api::services::SessionView,
    through: Option<u64>,
) -> Result<Value, String> {
    let inbox = crate::core::agent::Inbox::fold(&session.events(), through)?;
    let list = |target: InboxTarget| -> Result<Value, String> {
        let rows: Result<Vec<Value>, String> = inbox
            .pending(target)
            .iter()
            .map(|message| {
                serde_json::to_value(message)
                    .map_err(|err| format!("a pending message could not be projected: {err}"))
            })
            .collect();
        Ok(Value::Array(rows?))
    };
    Ok(json!({
        "next-turn": list(InboxTarget::NextTurn)?,
        "next-step": list(InboxTarget::NextStep)?,
    }))
}

/// The preset a request without one runs: the roster's default.
fn default_preset_id(roster: &[super::router::AgentPreset]) -> Option<String> {
    roster
        .iter()
        .find(|preset| preset.is_default)
        .or_else(|| roster.first())
        .map(|preset| preset.id.clone())
}

/// The code the TypeScript host uses for an unknown agent preset.
const AGENT_PRESET_NOT_FOUND: &str = "agent-preset/not-found";

/// The projection values dsh-rs can compute for a session: both
/// `session/projections` and the opening block of `session/follow` carry them,
/// and the model picker stays on "Loading models..." until it sees
/// `modelSelection`. Keys it cannot compute are omitted, not zeroed.
pub(super) fn projection_values(
    ctx: &Context,
    session: &dyn crate::api::services::SessionView,
    default: &DefaultModel,
    presets: &[super::router::AgentPreset],
    image_limits: &crate::images::ImageLimits,
) -> Result<Value, String> {
    let events = session.events();
    let permissions = match crate::permission::provided(ctx) {
        Some(table) => table.view(&events),
        None => json!({ "currentValue": "auto" }),
    };
    let plan = match ctx.get::<crate::plugins::plan::PlanMode>(crate::plugins::plan::PLAN_SERVICE) {
        Some(_) => crate::plugins::plan::projection(&events, None),
        None => json!({ "active": false, "pending": false }),
    };
    Ok(json!({
        "imageLimits": image_limits.projection(),
        "inbox": inbox_values(session, None)?,
        "workingDirectory": session.header_cwd(),
        "agentPreset": session.agent_preset().or_else(|| default_preset_id(presets)),
        "title": latest_title(session),
        "modelSelection": model_selection_values(session, default, None),
        "permissions": permissions,
        "plan": plan,
        "goal": super::goals::projection(&session.events()),
    }))
}

#[cfg(test)]
mod record_tests {
    use super::*;

    #[test]
    fn an_unencodable_event_keeps_its_seq_under_an_ignorable_unknown_type() {
        let event = SessionEvent {
            seq: 7,
            time: 3,
            data: crate::types::SessionEventData::TurnStart { turn: 1 },
        };
        let row = unencodable_row(&event, "why");
        assert_eq!((row.seq, row.time, row.ignorable), (7, 3, Some(true)));
        assert_eq!(row.kind, "dsh-rs/unencodable/turn/start");
        assert!(row.surface_op.is_none());
    }
}
