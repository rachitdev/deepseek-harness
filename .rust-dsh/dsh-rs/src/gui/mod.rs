//! The GUI gateway: the transport the browser client already speaks.
//!
//! This is the Rust port of the TypeScript `@deepseek-ai/dsh-api-gateway`
//! unary half. It serves the exact envelope the shipped Web client sends, so a
//! browser build can drive this harness without a translation layer:
//!
//! - `POST /api` carries one [`wire::ClientRequest`] and returns one
//!   [`wire::ServerResponse`];
//! - static files under the asset root serve the GUI bundle itself;
//! - every request needs the launch token, as a bearer header or the
//!   `dsh_token` cookie that `GET /` sets.
//!
//! Streams run over one WebSocket at `/api/remote.mux` ([`mux`]); of the 15
//! stream endpoints in the measured inventory only `$events` and
//! `session/follow` are served. Tool approvals reach the browser through
//! `$events` and come back through the unary `$events/result` ([`events`]).
//!
//! Limits, by design:
//! - only text prompt content is accepted; image and file blocks are refused
//!   with `gateway/input-invalid` rather than dropped silently;
//! - `session/page`, `session/search` and `session/cancel` serve live sessions only, because this build keeps no
//!   on-disk session store to reopen a cold one;
//! - the token is a long-lived secret in one process; there is no rotation.

pub mod base64;
pub mod boot;
mod tool_results;
pub mod cards;
pub mod account;
mod catalog;
pub mod claude_code_mods;
mod cot_translation;
mod cua_access;
pub mod credentials;
pub mod desktop;
pub mod descriptor;
pub mod dynamic_runner;
pub mod events;
pub mod export;
mod forge_panel;
pub mod files;
pub mod file_changes;
pub mod goals;
pub mod images;
pub mod liveshare;
pub mod commands;
mod feedback;
mod jobs;
mod model_discovery;
mod user_questions;
mod inventory;
pub mod mux;
pub mod plugin;
pub mod plugin_manager;
pub mod speech;
pub mod plugin_market;
pub mod product_analytics;
pub mod references;
pub mod office;
pub mod picker;
pub mod registry_probe;
pub mod router;
pub mod server;
mod schedule;
mod terminal;
mod terminal_stream;
mod session_read;
mod subagents;
mod zone_aliases;
pub mod time_zone;
mod uplink;
pub mod settings;
pub mod uploads;
pub mod wire;
pub mod workspaces;

pub use commands::harness_home;
pub use plugin::{gui_plugin, GuiConfig};
pub use router::{DefaultModel, Router, RouterConfig};
pub use server::{Auth, Gateway, HttpReply, API_PATH, TOKEN_COOKIE};

#[cfg(test)]
mod coverage {
    /// Endpoints this build serves, pinned so the GUI parity table cannot drift
    /// silently: adding an endpoint updates this list in the same change.
    #[test]
    fn the_implemented_endpoint_list_is_explicit() {
        assert_eq!(
            crate::gui::Router::endpoints(),
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
            ],
        );
    }

    /// Same pin for the stream endpoints the mux answers. A new stream arm in
    /// `run_stream` without a matching entry here fails the second assertion.
    #[test]
    fn the_implemented_stream_list_is_explicit() {
        assert_eq!(
            crate::gui::mux::STREAM_ENDPOINTS,
            &[
                "$events",
                "session/follow",
                "workspace/follow",
                "remoteAccess/follow",
                "session/control",
                "account/watch",
                "session/observeState",
                "session/observeEvent",
                "session/observeProjections",
                "account/watchExpiry",
                "terminal/attach",
                "terminal/retain",
                "job/list",
                "job/follow",
                "productAnalytics/watchPolicy",
                "claudeCodeMods/watchBand",
                "toolResults/watch",
                "userQuestions/attachWait",
                "workspaceFiles/changes",
                "forgePanel/follow",
                "forgePanel/changeRequestStates",
                "speech/follow",
            ]
        );
        // Every endpoint string arm of `run_stream`'s match, however its body is written.
        let source = include_str!("mux.rs");
        let start = source.find("    match endpoint {").expect("run_stream matches on the endpoint");
        let end = start + source[start..].find("        other =>").expect("and ends in a fallback");
        let arms: Vec<&str> = source[start..end]
            .lines()
            .filter(|line| line.starts_with("        \"") && line.contains("=>"))
            .collect();
        assert_eq!(arms.len(), crate::gui::mux::STREAM_ENDPOINTS.len(), "{arms:?}");
        for endpoint in crate::gui::mux::STREAM_ENDPOINTS {
            assert!(
                arms.iter().any(|arm| arm.contains(&format!("\"{endpoint}\""))),
                "{endpoint} has no arm"
            );
        }
    }
}
