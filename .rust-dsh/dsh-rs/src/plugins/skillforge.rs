//! Skillforge router plugin (`skillforge-router`).
//!
//! Delegates all ranking, abstention and token budgeting to the single-source
//! Rust `skillforge` CLI, exactly like the TypeScript plugin:
//!
//! - Tools: `skillforge_route`, `skillforge_pack_atoms`.
//! - Auto-hint (optional): on `agent/pre-step`, up to `injectCards` card
//!   summaries are appended as a `catalog` context message. Without a ranker
//!   the selector is labelled `unranked` (BM25F order); a JEV ranker may
//!   later filter them by relevance.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use cordis::Plugin;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;

use super::{inject_on_pre_step, register_tools, tool_spec};
use crate::api::services::BoxFuture;
use crate::types::{ContextForm, ToolExecutionResult};

/// Largest CLI answer accepted.
const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 16 * 1024;

/// Read a pipe to the end, failing as soon as it exceeds `limit` bytes.
async fn read_limited<R: tokio::io::AsyncRead + Unpin>(
    mut pipe: R,
    limit: usize,
) -> Result<Vec<u8>, String> {
    use tokio::io::AsyncReadExt;
    let mut data = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = pipe.read(&mut chunk).await.map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(data);
        }
        if data.len() + n > limit {
            return Err(format!("output larger than {limit} bytes"));
        }
        data.extend_from_slice(&chunk[..n]);
    }
}

/// Marker that identifies a Skillforge hint message.
pub const HINT_MARKER: &str = "Skillforge perspectives for this prompt";

/// Configuration for the `skillforge-router` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SkillforgeConfig {
    /// Path to the `skillforge` CLI.
    pub binary_path: String,
    /// Path to the compiled federation `registry.json`.
    pub registry_path: String,
    /// Repository root passed through to the CLI.
    pub repo_root: String,
    /// Attach optional card summaries to each user turn.
    pub auto_hint: bool,
    /// Candidates considered for the hint.
    pub top_k: u32,
    /// Maximum cards injected per turn (contract maximum is 3).
    pub inject_cards: u32,
    /// Maximum characters of one card summary.
    pub card_chars: usize,
    /// Atom token budget for routing.
    pub max_atom_tokens: u32,
    /// CLI deadline.
    pub timeout_ms: u64,
}

impl Default for SkillforgeConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let dsh_root = std::env::var("DSH_HOME").unwrap_or_else(|_| format!("{home}/.dsh"));
        SkillforgeConfig {
            binary_path: std::env::var("SKILLFORGE_BIN")
                .unwrap_or_else(|_| format!("{dsh_root}/bin/skillforge")),
            registry_path: format!("{dsh_root}/skills/skillforge/registry.json"),
            repo_root: dsh_root,
            auto_hint: false,
            top_k: 5,
            inject_cards: 3,
            card_chars: 240,
            max_atom_tokens: 1800,
            timeout_ms: 15_000,
        }
    }
}

/// Runs the CLI and parses its JSON stdout.
#[derive(Clone)]
pub struct SkillforgeEngine {
    config: Arc<SkillforgeConfig>,
}

impl SkillforgeEngine {
    pub fn new(config: SkillforgeConfig) -> Self {
        SkillforgeEngine {
            config: Arc::new(config),
        }
    }

    async fn run(&self, args: Vec<String>) -> Result<Value, String> {
        let mut command = Command::new(&self.config.binary_path);
        command
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|e| format!("cannot run {}: {e}", self.config.binary_path))?;
        let stdout = child.stdout.take().ok_or("no stdout")?;
        let stderr = child.stderr.take().ok_or("no stderr")?;
        // Both pipes are drained concurrently with a byte limit each. Going over
        // the stdout limit or the deadline drops the future, which kills the child.
        let work = async {
            let (out, err) = tokio::join!(
                read_limited(stdout, MAX_OUTPUT_BYTES),
                read_limited(stderr, MAX_STDERR_BYTES)
            );
            let out = out?;
            let status = child.wait().await.map_err(|e| e.to_string())?;
            Ok::<_, String>((status, out, err.unwrap_or_default()))
        };
        let (status, stdout, stderr) =
            tokio::time::timeout(Duration::from_millis(self.config.timeout_ms), work)
                .await
                .map_err(|_| {
                    format!("skillforge timed out after {} ms", self.config.timeout_ms)
                })??;
        if !status.success() {
            return Err(format!(
                "skillforge exited with {status}: {}",
                String::from_utf8_lossy(&stderr).trim()
            ));
        }
        serde_json::from_slice(&stdout)
            .map_err(|e| format!("skillforge returned invalid JSON: {e}"))
    }

    fn common(&self) -> Vec<String> {
        vec![
            "--repo-root".into(),
            self.config.repo_root.clone(),
            "--skills-dir".into(),
            PathBuf::from(&self.config.registry_path)
                .to_string_lossy()
                .into_owned(),
        ]
    }

    /// `skillforge route`.
    pub async fn route(
        &self,
        query: &str,
        top_k: u32,
        max_atom_tokens: u32,
        stage: Option<&str>,
    ) -> Result<Value, String> {
        let mut args = vec![
            "route".to_string(),
            "--query".into(),
            query.into(),
            "--top-k".into(),
            top_k.clamp(1, 25).to_string(),
            "--max-atom-tokens".into(),
            max_atom_tokens.clamp(60, 8000).to_string(),
        ];
        args.extend(self.common());
        if let Some(stage) = stage.filter(|s| !s.trim().is_empty()) {
            args.extend(["--stage".into(), stage.trim().into()]);
        }
        self.run(args).await
    }

    /// `skillforge pack-atoms`.
    pub async fn pack_atoms(
        &self,
        skill_id: &str,
        query: &str,
        max_atom_tokens: u32,
    ) -> Result<Value, String> {
        let mut args = vec![
            "pack-atoms".to_string(),
            "--skill-id".into(),
            skill_id.into(),
            "--query".into(),
            query.into(),
            "--max-atom-tokens".into(),
            max_atom_tokens.clamp(60, 8000).to_string(),
        ];
        args.extend(self.common());
        self.run(args).await
    }
}

/// One-line summary of a candidate, cut on a word boundary.
pub fn card_summary(candidate: &Value, max: usize) -> String {
    let field = |k: &str| {
        candidate
            .get(k)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
    };
    let text = field("description")
        .or(field("title"))
        .or(field("id"))
        .unwrap_or("");
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        return flat;
    }
    let clipped: String = flat.chars().take(max.saturating_sub(1)).collect();
    match clipped.rfind(' ') {
        Some(cut) if cut > max / 2 => format!("{}…", &clipped[..cut]),
        _ => format!("{clipped}…"),
    }
}

/// Model-facing hint for up to `cards` candidates, or `None` when unmatched.
pub fn format_hint(route: &Value, cards: usize, card_chars: usize) -> Option<String> {
    if !route
        .get("matched")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }
    let candidates = route.get("candidates").and_then(Value::as_array)?;
    let lines: Vec<String> = candidates
        .iter()
        .take(cards)
        .map(|c| {
            format!(
                "- `{}` (hub=`{}`): {}",
                c.get("id").and_then(Value::as_str).unwrap_or("?"),
                c.get("anchor_id")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                card_summary(c, card_chars)
            )
        })
        .collect();
    if lines.is_empty() {
        return None;
    }
    let stage = route
        .get("routed_stage")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let anchor = route
        .pointer("/routed_anchor/id")
        .and_then(Value::as_str)
        .unwrap_or("?");
    Some(format!(
        "{HINT_MARKER}: selector=unranked | Stage={stage} | Anchor=`{anchor}`\n{}\nI can consider these optional perspectives, not mandatory steps. For any skill I use, I should call skillforge_pack_atoms to load its instructions before applying it.",
        lines.join("\n")
    ))
}

/// Clamp before narrowing, so a huge value cannot wrap into range.
fn clamp_u32(value: u64, min: u32, max: u32) -> u32 {
    value.clamp(min as u64, max as u64) as u32
}

fn tool_result(answer: Result<Value, String>) -> ToolExecutionResult {
    match answer {
        Ok(value) => ToolExecutionResult::success_value(value),
        Err(e) => ToolExecutionResult::error("SKILLFORGE", e),
    }
}

/// The `skillforge-router` plugin. Requires the `tools` service.
pub fn skillforge_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "skillforge-router",
        vec![cordis::Injection::new(
            crate::api::TOOLS_SERVICE.to_string(),
        )],
        |ctx, config: SkillforgeConfig| async move {
            let engine = SkillforgeEngine::new(config.clone());
            let default_budget = config.max_atom_tokens;
            let mut specs = Vec::new();

            let route = engine.clone();
            specs.push(tool_spec(
                "skillforge_route",
                "Route a task or natural-language query across all compiled Skill Pack v2 skills in DeepSeek Harness. Returns the matched DTGS Lifecycle Stage, Domain Anchor skill, top candidate skills, and packed 300-900 token granular atoms (atoms/*.md).",
                json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The user task, technical domain, or skill query to route." },
                        "top_k": { "type": "integer", "description": "Maximum candidate skills to return (default 5)." },
                        "max_atom_tokens": { "type": "integer", "description": "Maximum tokens of packed atoms (default 1800)." },
                        "stage": { "type": "string", "description": "Optional lifecycle stage filter." }
                    },
                    "required": ["query"]
                }),
                move |args| {
                    let engine = route.clone();
                    async move {
                        let Some(query) = args.get("query").and_then(Value::as_str) else {
                            return ToolExecutionResult::error("BAD_ARGS", "query is required");
                        };
                        let top_k = clamp_u32(args.get("top_k").and_then(Value::as_u64).unwrap_or(5), 1, 25);
                        let budget = clamp_u32(args.get("max_atom_tokens").and_then(Value::as_u64).unwrap_or(default_budget as u64), 60, 8000);
                        tool_result(engine.route(query, top_k, budget, args.get("stage").and_then(Value::as_str)).await)
                    }
                },
            ));

            let pack = engine.clone();
            specs.push(tool_spec(
                "skillforge_pack_atoms",
                "Load a specific Skill Pack v2 skill's compact card.md, core.md and top matching granular atoms (atoms/*.md) within a strict token budget.",
                json!({
                    "type": "object",
                    "properties": {
                        "skill_id": { "type": "string", "description": "Exact skill ID from the Skillforge federation." },
                        "query": { "type": "string", "description": "Optional focus query used to rank the atoms." },
                        "max_atom_tokens": { "type": "integer", "description": "Maximum tokens of packed atoms (default 1800)." }
                    },
                    "required": ["skill_id"]
                }),
                move |args| {
                    let engine = pack.clone();
                    async move {
                        let Some(skill_id) = args.get("skill_id").and_then(Value::as_str) else {
                            return ToolExecutionResult::error("BAD_ARGS", "skill_id is required");
                        };
                        let query = args.get("query").and_then(Value::as_str).unwrap_or("");
                        let budget = clamp_u32(args.get("max_atom_tokens").and_then(Value::as_u64).unwrap_or(default_budget as u64), 60, 8000);
                        tool_result(engine.pack_atoms(skill_id, query, budget).await)
                    }
                },
            ));

            register_tools(&ctx, "skillforge-router tools", specs).await?;

            if config.auto_hint {
                let cards = config.inject_cards.clamp(1, 3) as usize;
                let card_chars = config.card_chars;
                let top_k = config.top_k;
                let hinter = engine.clone();
                inject_on_pre_step(
                    &ctx,
                    "skillforge-router",
                    ContextForm::Catalog,
                    move |prompt| {
                        let engine = hinter.clone();
                        Box::pin(async move {
                            match engine.route(&prompt, top_k, 60, None).await {
                                Ok(route) => format_hint(&route, cards, card_chars),
                                Err(e) => {
                                    eprintln!("skillforge-router: hint skipped: {e}");
                                    None
                                }
                            }
                        }) as BoxFuture<Option<String>>
                    },
                )
                .await?;
            }
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_summary_cuts_on_a_word_boundary() {
        let c = json!({ "description": "alpha beta gamma delta epsilon" });
        assert_eq!(card_summary(&c, 15), "alpha beta…");
        assert_eq!(card_summary(&c, 100), "alpha beta gamma delta epsilon");
    }

    #[tokio::test]
    async fn pipes_over_the_limit_fail_without_unbounded_buffering() {
        let data = vec![b'x'; 5000];
        assert!(read_limited(&data[..], 4096).await.is_err());
        assert_eq!(read_limited(&data[..], 8192).await.unwrap().len(), 5000);
    }

    #[tokio::test]
    async fn a_runaway_cli_is_stopped_at_the_limit() {
        let client = SkillforgeEngine::new(SkillforgeConfig {
            binary_path: "/usr/bin/yes".into(),
            timeout_ms: 20_000,
            ..Default::default()
        });
        let started = std::time::Instant::now();
        let error = client.run(vec![]).await.unwrap_err();
        assert!(error.contains("larger than"), "{error}");
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the cap, not the timeout, must stop it"
        );
    }

    #[test]
    fn huge_numbers_clamp_instead_of_wrapping() {
        assert_eq!(clamp_u32(4_294_967_296, 60, 8000), 8000);
        assert_eq!(clamp_u32(0, 60, 8000), 60);
    }

    #[test]
    fn unmatched_route_makes_no_hint() {
        assert!(format_hint(&json!({ "matched": false }), 3, 80).is_none());
    }

    #[test]
    fn hint_lists_at_most_the_requested_cards_and_is_labelled_unranked() {
        let route = json!({
            "matched": true, "routed_stage": "Planning", "routed_anchor": { "id": "dtgs" },
            "candidates": [
                { "id": "a", "anchor_id": "h1", "description": "first" },
                { "id": "b", "anchor_id": "h2", "description": "second" },
                { "id": "c", "anchor_id": "h3", "description": "third" }
            ]
        });
        let hint = format_hint(&route, 2, 80).unwrap();
        assert!(hint.starts_with(HINT_MARKER) && hint.contains("selector=unranked"));
        assert!(hint.contains("`a`") && hint.contains("`b`") && !hint.contains("`c`"));
    }
}
