//! System-prompt assembly — the Rust analogue of the reference harness's
//! `packages/core/system-prompt`. Plugins contribute ordered [`PromptSection`]s
//! and [`PromptContext`]s; one assembly call concatenates them, interpolates
//! `{{variables}}`, and yields the `system` slot for the next request.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cordis::{plugin, Context, Plugin};
use crate::types::{PromptAssembly, VariableProvider};
use serde_json::Value;

/// The `systemPrompt` service key.
pub const SYSTEM_PROMPT_SERVICE: &str = "systemPrompt";

/// Static text or a provider evaluated at each assembly.
#[derive(Clone)]
pub enum PromptText {
    Static(String),
    Dynamic(Arc<dyn Fn() -> String + Send + Sync>),
}

impl PromptText {
    pub fn resolve(&self) -> String {
        match self {
            PromptText::Static(text) => text.clone(),
            PromptText::Dynamic(provider) => provider(),
        }
    }
}

impl From<String> for PromptText {
    fn from(value: String) -> Self {
        PromptText::Static(value)
    }
}

impl From<&str> for PromptText {
    fn from(value: &str) -> Self {
        PromptText::Static(value.to_string())
    }
}

/// One contributed section of the system prompt.
#[derive(Clone)]
pub struct PromptSection {
    /// Unique name; a duplicate registration replaces the earlier one.
    pub name: String,
    /// Sections concatenate in ascending order. Convention: `-100` is the
    /// harness identity, `0` the deployment persona, tool guidance 100–199.
    pub order: i32,
    pub text: PromptText,
    /// Treat this contribution as the complete system prompt.
    pub complete: bool,
}

impl PromptSection {
    pub fn new(name: impl Into<String>, order: i32, text: impl Into<PromptText>) -> Self {
        PromptSection {
            name: name.into(),
            order,
            text: text.into(),
            complete: false,
        }
    }

    pub fn complete(mut self) -> Self {
        self.complete = true;
        self
    }
}

/// Dynamic model context materialized as a durable user-role snapshot.
#[derive(Clone)]
pub struct PromptContext {
    pub name: String,
    /// Contexts are joined in ascending order.
    pub order: i32,
    pub text: PromptText,
}

impl PromptContext {
    pub fn new(name: impl Into<String>, order: i32, text: impl Into<PromptText>) -> Self {
        PromptContext {
            name: name.into(),
            order,
            text: text.into(),
        }
    }
}

struct SystemPromptInner {
    sections: Mutex<Vec<PromptSection>>,
    contexts: Mutex<Vec<PromptContext>>,
    variables: Mutex<HashMap<String, VariableProvider>>,
    /// The instructions each preset contributes, with the revision they
    /// belong to. The snapshot is replaced whole.
    preset_prompts: Mutex<HashMap<String, crate::types::PresetRevision>>,
    /// The revision the next changed preset receives.
    next_generation: std::sync::atomic::AtomicU64,
    ctx: Context,
}

/// The `ctx.systemPrompt` service. Cheap-clone handle.
#[derive(Clone)]
pub struct SystemPromptService {
    inner: Arc<SystemPromptInner>,
}

impl SystemPromptService {
    pub fn new(ctx: Context) -> Self {
        SystemPromptService {
            inner: Arc::new(SystemPromptInner {
                sections: Mutex::new(Vec::new()),
                contexts: Mutex::new(Vec::new()),
                variables: Mutex::new(HashMap::new()),
                preset_prompts: Mutex::new(HashMap::new()),
                next_generation: std::sync::atomic::AtomicU64::new(1),
                ctx,
            }),
        }
    }

    /// Register an ordered prompt section (replaces same-name duplicates).
    pub fn section(&self, section: PromptSection) {
        let mut sections = self.inner.sections.lock().unwrap();
        if let Some(existing) = sections.iter_mut().find(|s| s.name == section.name) {
            *existing = section;
        } else {
            sections.push(section);
        }
        drop(sections);
        crate::api::events::emit(
            &self.inner.ctx,
            &crate::api::events::SystemPromptChangePayload,
        );
    }

    /// Remove a section by name; returns whether it existed.
    pub fn remove_section(&self, name: &str) -> bool {
        let mut sections = self.inner.sections.lock().unwrap();
        let before = sections.len();
        sections.retain(|s| s.name != name);
        let removed = sections.len() != before;
        drop(sections);
        if removed {
            crate::api::events::emit(
                &self.inner.ctx,
                &crate::api::events::SystemPromptChangePayload,
            );
        }
        removed
    }

    /// Register ordered dynamic context.
    pub fn context(&self, context: PromptContext) {
        let mut contexts = self.inner.contexts.lock().unwrap();
        if let Some(existing) = contexts.iter_mut().find(|c| c.name == context.name) {
            *existing = context;
        } else {
            contexts.push(context);
        }
        drop(contexts);
        crate::api::events::emit(
            &self.inner.ctx,
            &crate::api::events::SystemPromptChangePayload,
        );
    }

    /// Register a prompt variable. The provider may return `None`, in which
    /// case rendering a section that references it leaves the placeholder.
    pub fn variable(
        &self,
        name: &str,
        provider: impl Fn() -> Option<String> + Send + Sync + 'static,
    ) {
        self.inner
            .variables
            .lock()
            .unwrap()
            .insert(name.to_string(), Arc::new(provider) as VariableProvider);
    }

    /// Replace the mounted preset instructions.
    ///
    /// The snapshot is replaced whole, so a reader never sees a roster half
    /// mounted. A preset whose instructions did not change keeps its revision;
    /// one that changed receives the next revision, and a session bound to the
    /// old revision keeps running it until it selects the preset again.
    pub fn set_preset_prompts(&self, roster: Vec<(String, String)>) {
        let mut mounted = self.inner.preset_prompts.lock().unwrap();
        let mut next = HashMap::new();
        for (preset, text) in roster {
            let generation = match mounted.get(&preset) {
                Some(revision) if revision.text == text => revision.generation,
                _ => self
                    .inner
                    .next_generation
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            };
            next.insert(
                preset,
                crate::types::PresetRevision {
                    generation,
                    text,
                },
            );
        }
        *mounted = next;
    }

    /// The revision a preset's instructions belong to, when it has any.
    pub fn preset_revision(&self, preset: &str) -> Option<crate::types::PresetRevision> {
        self.inner.preset_prompts.lock().unwrap().get(preset).cloned()
    }

    /// Assemble the system prompt: order sections, concatenate, interpolate
    /// `{{variables}}`.
    pub fn assemble(&self) -> PromptAssembly {
        let mut sections: Vec<PromptSection> = self.inner.sections.lock().unwrap().clone();
        sections.sort_by_key(|s| s.order);
        // An effective complete section is the sole prompt section.
        if let Some(complete) = sections.iter().find(|s| s.complete) {
            return PromptAssembly { system: complete.text.resolve() };
        }
        let mut text = sections
            .iter()
            .map(|s| s.text.resolve())
            .filter(|t| !t.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        let variables = self.inner.variables.lock().unwrap().clone();
        for (name, provider) in &variables {
            let placeholder = format!("{{{{{name}}}}}");
            if text.contains(&placeholder) {
                if let Some(value) = provider() {
                    text = text.replace(&placeholder, &value);
                }
            }
        }
        PromptAssembly { system: text }
    }
}

/// The `systemPrompt` plugin: provides the `ctx.systemPrompt` service.
pub fn system_prompt_plugin() -> Arc<dyn Plugin> {
    plugin("system-prompt", |ctx, _config: Value| async move {
        let service = SystemPromptService::new(ctx.clone());
        let api: Arc<dyn crate::api::services::SystemPromptApi> = Arc::new(service.clone());
        ctx.provide(
            crate::api::SYSTEM_PROMPT_SERVICE,
            crate::api::services::SystemPromptService::new(api),
        )
        .await?;
        Ok(())
    })
}

impl crate::api::services::SystemPromptApi for SystemPromptService {
    fn section(&self, name: &str, order: i32, text: &str, complete: bool) {
        self.section(PromptSection {
            name: name.to_string(),
            order,
            text: PromptText::Static(text.to_string()),
            complete,
        });
    }

    fn remove_section(&self, name: &str) -> bool {
        self.remove_section(name)
    }

    fn variable(&self, name: &str, provider: crate::types::VariableProvider) {
        self.variable(name, move || provider());
    }

    fn assemble(&self) -> crate::types::PromptAssembly {
        self.assemble()
    }

    fn set_preset_prompts(&self, roster: Vec<(String, String)>) {
        self.set_preset_prompts(roster);
    }

    fn preset_revision(&self, preset: &str) -> Option<crate::types::PresetRevision> {
        self.preset_revision(preset)
    }
}
