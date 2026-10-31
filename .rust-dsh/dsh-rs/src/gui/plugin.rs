//! The `gui` plugin: binds the gateway and serves it until the harness stops.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use cordis::Plugin;
use serde::Deserialize;

use super::router::{DefaultModel, Router, RouterConfig};
use super::server::{serve, Auth, Gateway};

/// One extra route the model picker offers.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuiModel {
    /// Registered provider id, e.g. `omniroute`.
    pub provider: String,
    /// Model id inside that provider.
    pub model: String,
}

/// Configuration for the `gui` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GuiConfig {
    /// Interface to bind. Loopback by default: the gateway is not a public service.
    pub host: String,
    /// Port to bind; 0 asks the OS for a free one.
    pub port: u16,
    /// Launch token. Generated when omitted, and logged so the operator can open the GUI.
    pub token: Option<String>,
    /// Directory holding the built GUI bundle. Without it only `/api` answers.
    pub assets: Option<String>,
    /// Provider for sessions created through the GUI.
    pub default_provider: Option<String>,
    /// Model for sessions created through the GUI.
    pub default_model: Option<String>,
    /// Most events one `session/page` will serialize.
    pub max_page_events: usize,
    /// Most hits one `session/search` returns.
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
    /// Routes offered besides the default one, selectable per session.
    pub models: Vec<GuiModel>,
    /// Root of the built TypeScript checkout whose `packages/*/*/lib/client.js`
    /// bundles the page loads. Without it the static shell is served unmodified
    /// and the browser client cannot boot.
    pub client_root: Option<String>,
    /// Exact client packages to load, instead of the default roster.
    pub client_packages: Option<Vec<String>>,
    /// Client packages left out of the default roster.
    pub client_exclude: Vec<String>,
    /// Whether `experimental-*` client packages join the default roster.
    pub client_experimental: bool,
    /// JSON file the workspace registry persists to. Without it, workspaces
    /// last as long as the process.
    pub workspace_file: Option<String>,
    /// JSON file the settings store persists to. Without it, settings last
    /// as long as the process.
    pub settings_file: Option<String>,
    /// Directory `workspace/initializeDefault` registers (created if missing).
    /// Without it that call reports there is no default workspace.
    pub default_workspace: Option<String>,
    /// Whether this host can hand a workspace path to its desktop. Without it
    /// the answer follows the platform: always on macOS, on Linux only with a
    /// display, never elsewhere.
    pub native_open: Option<bool>,
    /// Longest a native open or application query may run before it is killed.
    pub desktop_command_timeout_ms: u64,
    /// Longest one registry ping of `pluginRegistryProbe/fastest` may take.
    pub registry_probe_timeout_ms: u64,
    /// Deadline for one market HTTP read, including its body.
    pub market_fetch_timeout_ms: u64,
    /// Largest accepted market index body.
    pub market_max_bytes: usize,
    /// Longest one `llm/discoverModels` request to a provider endpoint may take.
    pub llm_discovery_timeout_ms: u64,
    /// Longest feedback note, in UTF-8 bytes, `messageFeedback/put` accepts.
    pub max_note_bytes: usize,
    /// Absolute directory uploaded files are kept in. Unset serves no uploads:
    /// the store is never placed in a default location, because the TypeScript
    /// host's own store is not this process's to write.
    pub attachments_dir: Option<String>,
    /// Largest file one upload may carry, in bytes.
    pub max_upload_bytes: usize,
    /// Absolute path of the file holding the anonymous user id that `/feedback`
    /// reports. Unset, it is the TypeScript host's own file,
    /// `$DSH_HOME/.anonymous-user-id` (else `~/.dsh/.anonymous-user-id`), so
    /// both hosts report one id; an id already there is read, never replaced.
    /// With no home directory known the id lasts as long as the process.
    pub anonymous_user_file: Option<String>,
    /// Absolute path of the credential document. Unset means
    /// `.credentials.yaml` in the harness home, the file the TypeScript host uses.
    pub credentials_file: Option<String>,
    /// Absolute path of the reminder store. Unset means
    /// `storages/schedule.json` in the harness home, the file the TypeScript host uses.
    pub schedule_file: Option<String>,
    /// Session-owned terminal profiles and limits.
    pub terminal: crate::terminal::service::TerminalConfig,
    /// The office document converter and its limits.
    pub office_converter: crate::gui::office::OfficeConfig,
    /// Whether this host accepts analytics events.
    pub product_analytics: crate::gui::product_analytics::ProductAnalyticsConfig,
    /// Milliseconds between workspace file-watch samples.
    pub file_watch_poll_interval_ms: u64,
    /// The Platform browser sign-in's configuration.
    pub account: super::account::AccountConfig,
    /// Days of delivery receipts kept per reminder.
    pub schedule_delivery_history_days: u64,
    /// Delivery receipts kept per reminder.
    pub schedule_delivery_history_records: usize,
    /// Limits and normalization for images attached to prompts.
    pub images: crate::images::ImageConfig,
    /// Routes that accept image input. No adapter can report this, so the
    /// deployment lists them; a route left out is refused images, as the
    /// TypeScript host refuses a model without image input.
    pub image_models: Vec<GuiModel>,
    /// The agent presets the deployment offers. A deployment that names none
    /// offers the default one. A preset names what a session runs: this build
    /// composes one agent loop, so presets differ by the instructions they
    /// contribute, not by the tools they get.
    pub agent_presets: Vec<super::router::AgentPreset>,
}

/// Client packages the default roster omits. HMR needs a rebuild watcher this
/// host does not run; product analytics is desktop-only in the TS profile; the
/// native directory picker needs an OS dialog (the TS host resolves exactly one
/// of the browse and native pickers, since both fill the same slots).
const DEFAULT_CLIENT_EXCLUDE: &[&str] = &[
    "@deepseek-ai/dsh-client-hmr",
    "@deepseek-ai/dsh-client-product-analytics",
    "@deepseek-ai/dsh-client-ui-directory-picker-native",
];

impl Default for GuiConfig {
    fn default() -> Self {
        GuiConfig {
            host: "127.0.0.1".to_string(),
            // 3081 so a `gui` profile can run beside the live Web GUI on 3080.
            port: 3081,
            token: None,
            assets: None,
            default_provider: None,
            default_model: None,
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
            client_root: None,
            client_packages: None,
            client_exclude: DEFAULT_CLIENT_EXCLUDE.iter().map(|s| s.to_string()).collect(),
            client_experimental: false,
            agent_presets: super::router::default_presets(),
            workspace_file: None,
            settings_file: None,
            default_workspace: None,
            native_open: None,
            desktop_command_timeout_ms: 30_000,
            market_fetch_timeout_ms: 15_000,
            market_max_bytes: 128 * 1024 * 1024,
            registry_probe_timeout_ms: 5_000,
            llm_discovery_timeout_ms: 15_000,
            max_note_bytes: 8_192,
            attachments_dir: None,
            credentials_file: None,
            schedule_file: None,
            terminal: crate::terminal::service::TerminalConfig::default(),
            office_converter: crate::gui::office::OfficeConfig::default(),
            product_analytics: crate::gui::product_analytics::ProductAnalyticsConfig::default(),
            file_watch_poll_interval_ms: 300,
            account: super::account::AccountConfig::default(),
            schedule_delivery_history_days: 30,
            schedule_delivery_history_records: 200,
            max_upload_bytes: 32 * 1024 * 1024,
            anonymous_user_file: None,
            images: crate::images::ImageConfig::default(),
            image_models: Vec::new(),
        }
    }
}

impl GuiConfig {
    fn validate(&self) -> Result<(), String> {
        if self.host.trim().is_empty() {
            return Err("gui: host must not be empty".to_string());
        }
        if self.max_page_events == 0 {
            return Err("gui: maxPageEvents must be at least 1".to_string());
        }
        if self.max_search_results == 0 {
            return Err("gui: maxSearchResults must be at least 1".to_string());
        }
        if self.max_title_bytes == 0 {
            return Err("gui: maxTitleBytes must be at least 1".to_string());
        }
        if self.max_bytes == 0 {
            return Err("gui: maxBytes must be at least 1".to_string());
        }
        if self.max_file_bytes == 0 {
            return Err("gui: maxFileBytes must be at least 1".to_string());
        }
        if self.max_lines == 0 {
            return Err("gui: maxLines must be at least 1".to_string());
        }
        if self.max_entries == 0 {
            return Err("gui: maxEntries must be at least 1".to_string());
        }
        if self.max_note_bytes == 0 {
            return Err("gui: maxNoteBytes must be at least 1".to_string());
        }
        if self.max_upload_bytes == 0 {
            return Err("gui: maxUploadBytes must be at least 1".to_string());
        }
        self.images.validate()?;
        self.terminal.validate()?;
        self.office_converter.validate()?;
        if !self.image_models.is_empty() && self.attachments_dir.is_none() {
            return Err("gui: imageModels needs attachmentsDir, which is not set".to_string());
        }
        if let Some(file) = &self.anonymous_user_file {
            if !std::path::Path::new(file).is_absolute() {
                return Err(format!("gui: anonymousUserFile must be an absolute path, got {file:?}"));
            }
        }
        if let Some(file) = &self.schedule_file {
            if !std::path::Path::new(file).is_absolute() {
                return Err(format!("gui: scheduleFile must be an absolute path, got {file:?}"));
            }
        }
        if !(1..=3650).contains(&self.schedule_delivery_history_days) {
            return Err("gui: scheduleDeliveryHistoryDays must be between 1 and 3650".to_string());
        }
        if !(1..=10_000).contains(&self.schedule_delivery_history_records) {
            return Err("gui: scheduleDeliveryHistoryRecords must be between 1 and 10000".to_string());
        }
        if let Some(file) = &self.credentials_file {
            if !std::path::Path::new(file).is_absolute() {
                return Err(format!("gui: credentialsFile must be an absolute path, got {file:?}"));
            }
        }
        if let Some(dir) = &self.attachments_dir {
            if !std::path::Path::new(dir).is_absolute() {
                return Err(format!("gui: attachmentsDir must be an absolute path, got {dir:?}"));
            }
        }
        if self.llm_discovery_timeout_ms == 0 {
            return Err("gui: llmDiscoveryTimeoutMs must be positive".to_string());
        }
        if self.market_fetch_timeout_ms == 0 || self.market_max_bytes == 0 {
            return Err("gui: marketFetchTimeoutMs and marketMaxBytes must be positive".to_string());
        }
        if self.registry_probe_timeout_ms == 0 {
            return Err("gui: registryProbeTimeoutMs must be positive".to_string());
        }
        if self.desktop_command_timeout_ms == 0 {
            return Err("gui: desktopCommandTimeoutMs must be positive".to_string());
        }
        if self.export_compression_level > 9 {
            return Err("gui: exportCompressionLevel must be between 0 and 9".to_string());
        }
        if self.default_max_goal_rounds == 0 || self.default_max_goal_rounds > super::router::MAX_SAFE_INTEGER {
            return Err("gui: defaultMaxGoalRounds must be a positive safe integer".to_string());
        }
        if let Some(bad) = self
            .models
            .iter()
            .find(|route| route.provider.trim().is_empty() || route.model.trim().is_empty())
        {
            return Err(format!(
                "gui: models entry {:?}/{:?} needs a provider and a model",
                bad.provider, bad.model
            ));
        }
        if self.agent_presets.is_empty() {
            return Err("gui: agentPresets must name at least one preset".to_string());
        }
        for (index, preset) in self.agent_presets.iter().enumerate() {
            if preset.id.trim().is_empty() {
                return Err("gui: an agent preset needs an id".to_string());
            }
            if self.agent_presets[index + 1..]
                .iter()
                .any(|other| other.id == preset.id)
            {
                return Err(format!(
                    "gui: the agent preset id {:?} is used twice",
                    preset.id
                ));
            }
        }
        let defaults = self
            .agent_presets
            .iter()
            .filter(|preset| preset.is_default)
            .count();
        if defaults != 1 {
            return Err(format!(
                "gui: agentPresets must name exactly one default preset, found {defaults}"
            ));
        }
        if self.client_root.is_some() && self.assets.is_none() {
            return Err("gui: clientRoot needs assets, the shell page it injects into".to_string());
        }
        if let Some(assets) = &self.assets {
            // The gateway joins the requested name onto this path, so a file
            // here answers every page with a 404 that names no cause.
            if !Path::new(assets).is_dir() {
                return Err(format!(
                    "gui: assets must be the directory that holds index.html, got {assets:?}"
                ));
            }
        }
        if self.client_root.is_none() && self.client_packages.is_some() {
            return Err("gui: clientPackages needs clientRoot".to_string());
        }
        if let Some(token) = &self.token {
            if token.len() < 16 {
                return Err("gui: a supplied token must be at least 16 characters".to_string());
            }
        }
        Ok(())
    }
}

/// A random launch token: 32 hex characters from the OS entropy source.
/// A random version-4 UUID in its canonical text form.
pub fn uuid_v4() -> String {
    let mut hex: Vec<u8> = launch_token().into_bytes();
    // Version nibble 4 and an RFC 4122 variant nibble (8, 9, a or b).
    hex[12] = b'4';
    hex[16] = b"89ab"[(hex[16] % 4) as usize];
    let text = String::from_utf8(hex).unwrap_or_default();
    format!(
        "{}-{}-{}-{}-{}",
        &text[0..8],
        &text[8..12],
        &text[12..16],
        &text[16..20],
        &text[20..32]
    )
}

pub(super) fn launch_token() -> String {
    let mut bytes = [0u8; 16];
    let fresh = std::fs::File::open("/dev/urandom")
        .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut bytes));
    if fresh.is_err() {
        // No entropy device: fall back to the clock, still unique per process.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0);
        bytes.copy_from_slice(&now.to_le_bytes()[..16]);
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The `gui` plugin. Requires the `agents` service.
pub fn gui_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "gui",
        vec![cordis::Injection::new(
            crate::api::AGENTS_SERVICE.to_string(),
        )],
        |ctx, config: GuiConfig| async move {
            config.validate().map_err(cordis::Error::msg)?;
            let token = config.token.clone().unwrap_or_else(launch_token);
            let default_provider = config
                .default_provider
                .clone()
                .unwrap_or_else(|| "omniroute".to_string());
            let default_model = config
                .default_model
                .clone()
                .unwrap_or_else(|| "cc/claude-sonnet-5-5".to_string());
            // Each preset's instructions reach the model while a session runs
            // that preset. The roster is mounted before the stored sessions are
            // adopted, so a log naming a preset this deployment does not offer
            // is refused instead of running a different composition.
            super::router::mount_preset_prompts(&ctx, &config.agent_presets)
                .map_err(cordis::Error::msg)?;
            // Sessions an earlier run stored are adopted again, so the
            // operator finds them after a restart.
            let resumed = crate::session::resume::resume_stored_sessions(
                &ctx,
                crate::types::AgentOptions {
                    provider: default_provider.clone(),
                    model: default_model.clone(),
                    max_tokens: None,
                },
            )
            .await;
            if resumed.resumed > 0 || resumed.failed > 0 {
                ctx.logger().log_event(
                    cordis::logger::LogLevel::Info,
                    "gui".to_string(),
                    None,
                    format!(
                        "resumed {} stored session(s), {} could not be resumed",
                        resumed.resumed, resumed.failed
                    ),
                );
            }
            let router = Router::new(RouterConfig {
                agent_presets: config.agent_presets.clone(),
                default_model: DefaultModel {
                    provider: default_provider,
                    model: default_model,
                },
                max_page_events: config.max_page_events,
                max_search_results: config.max_search_results,
                max_title_bytes: config.max_title_bytes,
                max_bytes: config.max_bytes,
                max_file_bytes: config.max_file_bytes,
                max_lines: config.max_lines,
                max_entries: config.max_entries,
                default_max_goal_rounds: config.default_max_goal_rounds,
                export_compression_level: config.export_compression_level,
                models: config
                    .models
                    .iter()
                    .map(|route| DefaultModel {
                        provider: route.provider.clone(),
                        model: route.model.clone(),
                    })
                    .collect(),
            });
            let registry = super::workspaces::WorkspaceRegistry::new(
                config.workspace_file.clone().map(PathBuf::from),
                config.default_workspace.clone().map(PathBuf::from),
            )
            .map_err(cordis::Error::msg)?;
            let settings = Arc::new(
                super::settings::SettingsStore::new(config.settings_file.clone().map(PathBuf::from))
                    .map_err(cordis::Error::msg)?,
            );
            let desktop = super::desktop::Desktop::system(
                config.native_open,
                std::time::Duration::from_millis(config.desktop_command_timeout_ms),
            );
            let probe = super::registry_probe::RegistryProbe::system(
                std::time::Duration::from_millis(config.registry_probe_timeout_ms),
            )
            .map_err(cordis::Error::msg)?;
            let discovery = super::model_discovery::Discovery::new(
                std::time::Duration::from_millis(config.llm_discovery_timeout_ms),
            )
            .map_err(cordis::Error::msg)?;
            let store = match &config.attachments_dir {
                Some(dir) => Some(
                    crate::attachments::AttachmentStore::open(std::path::Path::new(dir))
                        .map_err(cordis::Error::msg)?,
                ),
                None => None,
            };
            if let Some(store) = &store {
                ctx.provide(crate::attachments::ATTACHMENTS_SERVICE, store.clone()).await?;
            }
            let image_models = crate::images::ImageModels::new(
                config
                    .image_models
                    .iter()
                    .map(|route| (route.provider.clone(), route.model.clone())),
            );
            ctx.provide(crate::images::IMAGE_MODELS_SERVICE, image_models.clone()).await?;
            let images = Arc::new(super::images::ImageAdmission::new(
                store.clone(),
                &config.images,
                image_models,
            ));
            let uploads = Arc::new(
                super::uploads::Uploads::new(store, config.max_upload_bytes)
                    .map_err(cordis::Error::msg)?,
            );
            uploads.observe(&ctx).await?;
            let schedule_store = match config
                .schedule_file
                .clone()
                .map(PathBuf::from)
                .or_else(crate::schedule::store::default_path)
            {
                Some(path) => crate::schedule::store::Store::open(&path).map_err(|e| cordis::Error::msg(e.0))?,
                None => crate::schedule::store::Store::in_memory(),
            };
            let schedule = crate::schedule::service::ScheduleService::new(
                schedule_store,
                crate::schedule::store::Retention {
                    days: config.schedule_delivery_history_days,
                    records: config.schedule_delivery_history_records,
                },
            );
            schedule.start(ctx.clone());
            {
                let schedule = Arc::clone(&schedule);
                ctx.effect("schedule.driver".to_string(), async move {
                    let disposer: cordis::Disposer = Box::new(move || {
                        Box::pin(async move { schedule.stop() })
                    });
                    Ok(Some(disposer))
                })
                .await?;
            }
            ctx.provide(crate::schedule::service::SCHEDULE_SERVICE, Arc::clone(&schedule)).await?;
            let terminal = crate::terminal::service::TerminalService::new(ctx.clone(), config.terminal.clone());
            terminal.observe().await?;
            ctx.provide(crate::terminal::service::TERMINAL_SERVICE, Arc::clone(&terminal)).await?;
            {
                let terminal = Arc::clone(&terminal);
                ctx.effect("terminal.processes".to_string(), async move {
                    let disposer: cordis::Disposer = Box::new(move || Box::pin(async move { terminal.dispose().await }));
                    Ok(Some(disposer))
                }).await?;
            }
            let office = Arc::new(super::office::OfficeService::new(config.office_converter.clone()));
            let product_analytics = Arc::new(super::product_analytics::ProductAnalyticsService::new(config.product_analytics.clone()));
            let cards = Arc::new(super::cards::CardsService::new());
            ctx.provide(super::claude_code_mods::BANDS_SERVICE, Arc::new(super::claude_code_mods::Bands::default())).await?;
            // The tool-result watch reads the executor's live values, so it must
            // be mounted before any tool runs.
            super::tool_results::observe(&ctx).await?;
            ctx.provide(
                super::file_changes::FILE_CHANGES_SERVICE,
                super::file_changes::FileChangesConfig {
                    poll_interval_ms: config.file_watch_poll_interval_ms,
                },
            )
            .await?;
            // The profile's bundle names are the manager's bundles.
            let bundles = ctx
                .get::<Vec<String>>(crate::bundle::PROFILE_BUNDLES_SERVICE)
                .map(|names| names.as_ref().clone())
                .unwrap_or_else(|| vec!["base".to_string()]);
            let plugin_manager = Arc::new(super::plugin_manager::PluginManager::new(bundles));
            let plugin_market = Arc::new(
                super::plugin_market::PluginMarket::new(Arc::new(
                    super::plugin_market::SettingsIndexes::new(Arc::clone(&settings)),
                ))
                .with_fetch_limits(config.market_fetch_timeout_ms, config.market_max_bytes),
            );
            // One credential store: the account service keeps its grant in it.
            let credentials = Arc::new(match &config.credentials_file {
                Some(file) => super::credentials::CredentialStore::new(Some(PathBuf::from(file))),
                None => super::credentials::CredentialStore::at_harness_home(),
            });
            let account = super::account::Account::system(
                &config.account.platform_origin,
                Arc::clone(&credentials),
            )
            .map_err(|error| cordis::Error::msg(error.to_string()))?;
            let router = router
                .with_account(account)
                .with_plugin_manager(plugin_manager)
                .with_plugin_market(plugin_market)
                .with_liveshare(Arc::new(super::liveshare::LiveshareService::new()))
                .with_product_analytics(product_analytics)
                .with_cards(cards)
                .with_office(office)
                .with_picker(Arc::new(super::picker::Picker::system()))
                .with_terminal(terminal)
                .with_schedule(schedule)
                .with_commands(Arc::new(super::commands::Commands::new(
                    config.anonymous_user_file.clone().map(PathBuf::from).or_else(|| {
                        super::commands::default_anonymous_user_file(|name| std::env::var(name).ok())
                    }),
                )))
                .with_uploads(uploads)
                .with_images(images)
                .with_discovery(Arc::new(discovery))
                .with_desktop(Arc::new(desktop))
                .with_registry_probe(Arc::new(probe))
                .with_max_note_bytes(config.max_note_bytes)
                .map_err(cordis::Error::msg)?
                .with_workspaces(Arc::new(registry))
                .with_credentials(credentials)
                .with_settings(settings);
            let mut gateway = Gateway::new(
                router,
                Auth {
                    token: token.clone(),
                },
                config.assets.clone().map(PathBuf::from),
            );
            if let Some(root) = &config.client_root {
                let host = super::boot::BootHost::load(&super::boot::BootConfig {
                    client_root: PathBuf::from(root),
                    packages: config.client_packages.clone(),
                    exclude: config.client_exclude.clone(),
                    include_experimental: config.client_experimental,
                })
                .map_err(cordis::Error::msg)?;
                ctx.logger().log_event(
                    cordis::logger::LogLevel::Info,
                    "gui".to_string(),
                    None,
                    format!("client module graph: {} packages", host.package_count()),
                );
                gateway = gateway.with_boot(host);
            }
            let gateway = Arc::new(gateway);

            let address = format!("{}:{}", config.host, config.port);
            let listener = tokio::net::TcpListener::bind(&address)
                .await
                .map_err(|error| {
                    cordis::Error::msg(format!("gui: cannot listen on {address}: {error}"))
                })?;
            let bound = listener
                .local_addr()
                .map(|addr| addr.to_string())
                .unwrap_or(address);
            let url = format!(
                "http://{bound}/?token={token}",
                bound = bound.replace("0.0.0.0", "127.0.0.1")
            );
            ctx.logger().log_event(
                cordis::logger::LogLevel::Info,
                "gui".to_string(),
                None,
                format!("listening on {url}"),
            );

            let serving = Arc::clone(&gateway);
            let serving_ctx = ctx.clone();
            let handle = tokio::spawn(async move {
                if let Err(error) = serve(serving, serving_ctx, listener).await {
                    eprintln!("gui: server stopped: {error}");
                }
            });
            ctx.effect("gui server", async move {
                let disposer: cordis::Disposer =
                    Box::new(move || Box::pin(async move { handle.abort() }));
                Ok(Some(disposer))
            })
            .await?;
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_supplied_token_is_refused() {
        let config = GuiConfig {
            token: Some("short".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn a_generated_token_is_long_random_hex() {
        let first = launch_token();
        let second = launch_token();
        assert_eq!(first.len(), 32);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(first, second, "two launches must not share a token");
    }

    #[test]
    fn a_zero_page_budget_is_refused() {
        let config = GuiConfig {
            max_page_events: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn an_assets_path_that_is_not_a_directory_is_refused() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let config = GuiConfig {
            assets: Some(file.path().to_string_lossy().into_owned()),
            ..Default::default()
        };
        let error = config.validate().unwrap_err();
        assert!(
            error.contains("assets must be the directory that holds index.html"),
            "the error must name the expectation, got {error:?}"
        );
    }

    #[test]
    fn an_assets_directory_is_accepted() {
        let directory = tempfile::tempdir().unwrap();
        let config = GuiConfig {
            assets: Some(directory.path().to_string_lossy().into_owned()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
