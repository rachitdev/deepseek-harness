//! # dsh-bundle
//!
//! Composition of the dsh-rs **base bundle** — the first layer of every
//! profile, mirroring the reference harness's `dsh-base`: model adapters,
//! tools, persistence, and the agent loop, all as plugins on the cordis
//! kernel. A profile is just a named composition; the base bundle is the one
//! every profile stacks first.

pub mod dynamic;
pub mod roster;

use std::path::PathBuf;
use std::sync::Arc;

use cordis::{Context, FiberHandle, Plugin};
use serde_json::{json, Value};

/// One row of a bundle: a plugin plus its config.
pub struct BundleEntry {
    pub name: &'static str,
    pub plugin: Arc<dyn Plugin>,
    pub config: Option<Value>,
}

/// Configuration for the base bundle.
#[derive(Debug, Clone, Default)]
pub struct BaseConfig {
    /// Session persistence directory; `None` keeps sessions in memory only.
    pub store_dir: Option<PathBuf>,
    /// Session log format under `store_dir`: `jsonl` (default, Rust-native),
    /// `v4` (TS-compatible, zstd) or `v4-plain` (TS-compatible, uncompressed).
    /// An unknown value fails the bundle install.
    pub store_format: Option<String>,
    /// OpenAI-compatible provider section (see `dsh_llm`'s openai adapter).
    pub openai: Option<Value>,
    /// Additional OpenAI-compatible endpoints, one adapter per segment:
    /// `{ "opencode": { "base_url", "api_key", "model" }, ... }`. Merged
    /// into the `llm` plugin's `adapters` map alongside `openai`.
    pub adapters: Option<Value>,
    /// Default provider route for agents (falls back to `mock`).
    pub default_provider: Option<String>,
    /// Default model id for agents (falls back per provider).
    pub default_model: Option<String>,
}

impl BaseConfig {
    /// Parse a `config` object (the shape used by profiles and
    /// `~/.dsh/config.json`):
    ///
    /// ```json
    /// {
    ///   "store_dir": "/tmp/dsh-sessions",
    ///   "provider": "deepseek",
    ///   "model": "deepseek-chat",
    ///   "openai": { "providers": [...], "base_url": "...", "api_key": "..." },
    ///   "adapters": { "opencode": { "base_url": "...", "api_key": "..." } }
    /// }
    /// ```
    pub fn from_value(value: &Value) -> BaseConfig {
        let mut config = BaseConfig::default();
        if let Some(dir) = value.get("store_dir").and_then(|d| d.as_str()) {
            config.store_dir = Some(PathBuf::from(dir));
        }
        if let Some(format) = value.get("store_format").and_then(|f| f.as_str()) {
            config.store_format = Some(format.to_string());
        }
        if let Some(openai) = value.get("openai") {
            config.openai = Some(openai.clone());
        }
        if let Some(adapters) = value.get("adapters") {
            config.adapters = Some(adapters.clone());
        }
        if let Some(provider) = value.get("provider").and_then(|p| p.as_str()) {
            config.default_provider = Some(provider.to_string());
        }
        if let Some(model) = value.get("model").and_then(|m| m.as_str()) {
            config.default_model = Some(model.to_string());
        }
        config
    }
}

/// The declarative service contract of one plugin.
pub use crate::api::manifest::{ManifestApi, ManifestRegistry, PluginManifest};

/// The manifest plugin: provides `ctx.manifest`, the contract registry.
pub fn manifest_plugin() -> Arc<dyn Plugin> {
    cordis::plugin::plugin("manifest", |ctx, _config: Value| async move {
        let registry = Arc::new(ManifestRegistry::new());
        let api: Arc<dyn crate::api::manifest::ManifestApi> = registry.clone();
        ctx.provide(
            crate::api::MANIFEST_SERVICE,
            crate::api::services::ManifestService::new(api),
        )
        .await?;
        Ok(())
    })
}

/// Register every bundle plugin's manifest on the manifest service. This is
/// the declarative contract (`dsh dump-config` / coverage validation):
/// what each plugin provides, requires, and which tools it registers.
///
/// The contract of a plugin is what it DECLARES, independent of whether it
/// was activated (e.g. `session-persistence` is only installed when a store
/// dir is configured, but its contract is part of the base bundle). The
/// `tools` manifest is filled from the live registry to avoid drift between
/// this list and the actually registered built-ins.
pub fn register_manifests(ctx: &Context) {
    let Some(manifest) =
        ctx.get::<crate::api::services::ManifestService>(crate::api::MANIFEST_SERVICE)
    else {
        return;
    };

    // The contract registry itself.
    manifest.register(
        PluginManifest::new("manifest")
            .describe("declarative plugin contracts and coverage validation")
            .provides(
                crate::api::MANIFEST_SERVICE,
                "manifest registry (ctx.manifest)",
            ),
    );

    manifest.register(
        PluginManifest::new("llm")
            .describe("model adapters and the LLM adapter seam")
            .provides(
                crate::api::LLM_SERVICE,
                "provider routing and streaming (ctx.llm)",
            )
            .provides(crate::api::LLM_STREAMS_SERVICE, "in-process stream handles"),
    );
    manifest.register(
        PluginManifest::new("sessions")
            .describe("event-sourced session log and store")
            .provides(
                crate::api::SESSIONS_SERVICE,
                "session create/get/list/fork/flush",
            ),
    );
    manifest.register(
        PluginManifest::new("session-persistence")
            .describe("JSONL durability backend")
            .provides(crate::api::SESSION_PERSISTENCE_SERVICE, "JSONL backend")
            .requires(crate::api::SESSIONS_SERVICE),
    );
    manifest.register(
        PluginManifest::new("tools")
            .describe("scoped tool registry and guarded execution pipeline")
            .provides(crate::api::TOOLS_SERVICE, "tool registry (ctx.tools)")
            .tools(
                ctx.get::<crate::api::services::ToolsService>(crate::api::TOOLS_SERVICE)
                    .map(|registry| registry.schemas())
                    .unwrap_or_default(),
            ),
    );
    manifest.register(
        PluginManifest::new("system-prompt")
            .describe("prompt-section and variable assembly")
            .provides(
                crate::api::SYSTEM_PROMPT_SERVICE,
                "system-prompt assembly (ctx.systemPrompt)",
            ),
    );
    manifest.register(
        PluginManifest::new("agent-loop")
            .describe("the default agent driver")
            .provides(crate::api::AGENTS_SERVICE, "agent registry (ctx.agents)")
            .requires(crate::api::SESSIONS_SERVICE)
            .requires(crate::api::SYSTEM_PROMPT_SERVICE)
            .requires(crate::api::TOOLS_SERVICE)
            .requires(crate::api::LLM_SERVICE)
            .requires(crate::api::LLM_STREAMS_SERVICE),
    );
}

/// Start the base bundle on `ctx` and wait for every fiber to converge.
pub async fn install_base(ctx: &Context, config: BaseConfig) -> Result<Vec<FiberHandle>, String> {
    let mut entries: Vec<BundleEntry> = Vec::new();

    // The plugin inventory reads this record of what the bundle started.
    let roster = Arc::new(roster::PluginRoster::default());
    entries.push(BundleEntry {
        name: "plugin-roster",
        plugin: roster::roster_plugin(roster.clone()),
        config: None,
    });

    // 0. The manifest registry (contract layer) is always present first: its
    // `ctx.manifest` service is what `register_manifests` writes into later.
    entries.push(BundleEntry {
        name: "manifest",
        plugin: manifest_plugin(),
        config: None,
    });

    // 1. LLM seam: the mock adapter is always registered; every
    // OpenAI-compatible endpoint from `adapters` (plus the legacy `openai`
    // section) is mounted as its own adapter.
    let mut llm_config = json!({});
    let mut adapters = config.adapters.clone().unwrap_or_else(|| json!({}));
    if let Some(openai) = &config.openai {
        if let Some(map) = adapters.as_object_mut() {
            map.insert("openai".to_string(), openai.clone());
        }
    }
    llm_config["adapters"] = adapters;
    entries.push(BundleEntry {
        name: "llm",
        plugin: crate::llm::llm_plugin(),
        config: Some(llm_config),
    });

    // 2. Event-sourced sessions.
    entries.push(BundleEntry {
        name: "sessions",
        plugin: crate::session::session_plugin(),
        config: None,
    });

    // 3. Tools registry + built-in tools.
    entries.push(BundleEntry {
        name: "tools",
        plugin: crate::tools::tools_plugin(),
        config: Some(Value::Null),
    });

    // 4. System-prompt assembly.
    entries.push(BundleEntry {
        name: "system-prompt",
        plugin: crate::core::prompt::system_prompt_plugin(),
        config: None,
    });

    let mut handles: Vec<FiberHandle> = Vec::new();
    for entry in &entries {
        handles.push(ctx.plugin(entry.plugin.clone(), entry.config.clone()));
    }

    // 5. JSONL persistence attaches to the session store (needs sessions up).
    if let Some(dir) = &config.store_dir {
        let plugin = match config.store_format.as_deref() {
            None | Some("jsonl") => crate::session::jsonl_persistence_plugin(dir.clone()),
            Some("v4") => crate::session::v4_persistence_plugin(dir.clone(), crate::session::Compression::Zstd),
            Some("v4-plain") => crate::session::v4_persistence_plugin(dir.clone(), crate::session::Compression::None),
            Some(other) => {
                return Err(format!("unknown store_format `{other}` (expected jsonl, v4 or v4-plain)"));
            }
        };
        handles.push(ctx.plugin(plugin, None));
    }

    // 6. The agent loop wires sessions + prompt + tools + llm into agents.
    handles.push(ctx.plugin(crate::core::agent_loop_plugin(), None));
    for handle in &handles {
        roster.started(handle);
    }

    // Wait for convergence; surface the first failure.
    for handle in &handles {
        eprintln!("[bundle] joining {}", handle.name());
        handle
            .join()
            .await
            .map_err(|err| format!("bundle plugin {} failed to start: {err}", handle.name()))?;
        eprintln!("[bundle] joined {}", handle.name());
    }

    // Declare every plugin's contract on the manifest registry.
    register_manifests(ctx);

    Ok(handles)
}

/// Convenience: load a dynamic plugin cdylib (see [`dynamic`]).
pub fn load_dynamic_plugin(path: &std::path::Path) -> dynamic::Result<Arc<dynamic::DynamicPlugin>> {
    dynamic::load_dynamic_plugin(path)
}

/// Convenience: install the base bundle with an in-memory store.
pub async fn install_base_default(ctx: &Context) -> Result<Vec<FiberHandle>, String> {
    install_base(ctx, BaseConfig::default()).await
}

/// Read a JSON profile and install the bundles it stacks. Currently the
/// only supported bundle is `base`; unknown bundles are an error.
///
/// ```json
/// {
///   "bundles": ["base"],
///   "config": {
///     "store_dir": "/tmp/dsh-sessions",
///     "openai": { "base_url": "...", "api_key": "..." }
///   }
/// }
/// ```
/// The context service that carries the profile's bundle names.
pub const PROFILE_BUNDLES_SERVICE: &str = "profile.bundles";

pub async fn install_profile(ctx: &Context, profile: &Value) -> Result<Vec<FiberHandle>, String> {
    // Default only when the field is absent; a present field of the wrong type is an error.
    let bundles = match profile.get("bundles") {
        None => vec![json!("base")],
        Some(Value::Array(list)) => list.clone(),
        Some(other) => {
            return Err(format!(
                "`bundles` must be an array of bundle names; got {}",
                json_type(other)
            ))
        }
    };
    // The plugin manager reads the profile's bundle names from here.
    let names: Vec<String> = bundles
        .iter()
        .filter_map(|bundle| bundle.as_str().map(str::to_string))
        .collect();
    ctx.provide(PROFILE_BUNDLES_SERVICE, names).await;
    let config = profile.get("config").cloned().unwrap_or_else(|| json!({}));
    let base = BaseConfig::from_value(&config);
    // Plugins attach after the base bundle, so tools/sessions are live first.
    let plugins = profile.get("plugins").cloned().unwrap_or_else(|| json!({}));
    // Reject every bad bundle name and the plugin map's type before starting anything.
    for bundle in &bundles {
        if bundle.as_str() != Some("base") {
            return Err(format!(
                "unknown bundle \"{}\" (supported: \"base\")",
                bundle.as_str().unwrap_or("?")
            ));
        }
    }
    if !plugins.is_object() {
        return Err(format!(
            "`plugins` must be an object mapping plugin name to config; got {}",
            json_type(&plugins)
        ));
    }
    let mut handles = Vec::new();
    for _ in &bundles {
        match install_base(ctx, base.clone()).await {
            Ok(started) => handles.extend(started),
            Err(message) => {
                dispose_all(&handles).await;
                return Err(message);
            }
        }
    }
    match install_native_plugins(ctx, &plugins).await {
        Ok(native) => handles.extend(native),
        Err(message) => {
            dispose_all(&handles).await;
            return Err(message);
        }
    }
    Ok(handles)
}

/// Name of a JSON value's type, for diagnostics that must not echo configuration values.
fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Unload started plugins, newest first.
async fn dispose_all(handles: &[FiberHandle]) {
    for handle in handles.iter().rev() {
        handle.dispose().await;
    }
}

/// The names a profile's `plugins` object may use.
const NATIVE_PLUGIN_NAMES: [&str; 15] = [
    "cortexdb-memory",
    "skillforge-router",
    "mcp-client",
    "skills",
    "ts-dsh-bridge",
    "web",
    "approvals",
    "ask-user",
    "compaction",
    "subagent",
    "goal",
    "permission",
    "plan",
    "schedule",
    "gui",
];

/// The native plugin a profile names, or `None` for a name this build lacks.
fn native_plugin(name: &str) -> Option<Arc<dyn Plugin>> {
    Some(match name {
        "cortexdb-memory" => crate::plugins::cortexdb::cortexdb_plugin(),
        "skillforge-router" => crate::plugins::skillforge::skillforge_plugin(),
        "mcp-client" => crate::plugins::mcp::mcp_plugin(),
        "skills" => crate::plugins::skills::skills_plugin(),
        "ts-dsh-bridge" => crate::plugins::bridge::bridge_plugin(),
        "web" => crate::plugins::web::web_plugin(),
        "approvals" => crate::plugins::approvals::approvals_plugin(),
        "ask-user" => crate::plugins::ask_user::ask_user_plugin(),
        "compaction" => crate::plugins::compaction::compaction_plugin(),
        "subagent" => crate::plugins::subagent::subagent_plugin(),
        "goal" => crate::plugins::goal::goal_plugin(),
        "permission" => crate::plugins::permission::permission_plugin(),
        "plan" => crate::plugins::plan::plan_plugin(),
        "schedule" => crate::plugins::schedule::plugin(),
        "gui" => crate::gui::gui_plugin(),
        _ => return None,
    })
}

/// Start the native ecosystem plugins named in a profile's `plugins` object:
/// `{ "cortexdb-memory": {..config..}, "skillforge-router": {..} }`.
/// A plugin whose config has `"enabled": false` is skipped; an unknown name
/// is an error so a typo never silently disables memory or skills.
pub async fn install_native_plugins(
    ctx: &Context,
    plugins: &Value,
) -> Result<Vec<FiberHandle>, String> {
    let Some(map) = plugins.as_object() else {
        return Err(format!(
            "`plugins` must be an object mapping plugin name to config; got {}",
            json_type(plugins)
        ));
    };
    // The inventory lists what runs, so a plugin may only start where the
    // bundle's roster can record it.
    let Some(roster) = roster::roster_of(ctx) else {
        return Err("the plugin roster is missing: install the base bundle before native plugins".to_string());
    };
    // Validate every name before starting anything, so a typo leaves nothing running.
    let mut selected = Vec::new();
    let mut switched_off = Vec::new();
    for (name, config) in map {
        if config.get("enabled").and_then(Value::as_bool) == Some(false) {
            // An unknown name stays unrecorded, as before; only a plugin this
            // build offers appears in the inventory as switched off.
            if NATIVE_PLUGIN_NAMES.contains(&name.as_str()) {
                switched_off.push(name.clone());
            }
            continue;
        }
        let Some(plugin) = native_plugin(name) else {
            return Err(format!(
                "unknown plugin \"{name}\" (supported: {})",
                NATIVE_PLUGIN_NAMES.join(", ")
            ));
        };
        selected.push((plugin, config.clone()));
    }
    let mut handles: Vec<FiberHandle> = Vec::new();
    for (plugin, config) in selected {
        handles.push(ctx.plugin(plugin, Some(config)));
    }
    for name in &switched_off {
        roster.skipped(name);
    }
    for handle in &handles {
        roster.started(handle);
    }
    let mut failure = None;
    for handle in &handles {
        if let Err(err) = handle.join().await {
            failure = Some(format!("plugin {} failed to start: {err}", handle.name()));
            break;
        }
        // `join` also returns for a plugin still waiting on a missing service.
        if handle.state() != cordis::FiberState::Active {
            failure = Some(format!(
                "plugin {} did not activate (state {:?}); a service it requires is missing from the profile",
                handle.name(),
                handle.state()
            ));
            break;
        }
    }
    if let Some(message) = failure {
        dispose_all(&handles).await;
        return Err(message);
    }
    Ok(handles)
}
