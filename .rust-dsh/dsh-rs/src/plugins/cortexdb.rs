//! CortexDB long-term memory plugin (`cortexdb-memory`).
//!
//! Talks to the CortexDB sidecar's REST API (`POST /v1/tools/<name>`), the
//! same tools the TypeScript plugin reaches over gRPC:
//! `knowledge_memory_recall`, `memory_save`, `memory_delete`.
//!
//! - Auto-recall: on `agent/pre-step`, relevant memories are appended to the
//!   batch as a `recall` context message. A slow or failing lookup is skipped.
//! - Tools: `cortexdb_memory_search`, and unless `read_only`,
//!   `cortexdb_memory_store` and `cortexdb_memory_forget`.

use std::sync::Arc;
use std::time::Duration;

use cordis::Plugin;
use serde::Deserialize;
use serde_json::{json, Value};

use super::{inject_on_pre_step, register_tools, snippet, tool_spec};
use crate::api::services::BoxFuture;
use crate::types::{ContextForm, ToolExecutionResult};

/// Largest sidecar response accepted; recall answers are far smaller.
const MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

/// Configuration for the `cortexdb-memory` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CortexConfig {
    /// REST base URL of the sidecar.
    pub rest_url: String,
    /// Bearer token, if the sidecar requires one.
    pub token: String,
    /// Narrow recall (and stored memories) to one user / scope / namespace.
    pub user_id: String,
    pub scope: String,
    pub namespace: String,
    /// Recall before each user prompt.
    pub auto_recall: bool,
    /// Memories and knowledge hits per recall.
    pub recall_top_k: u32,
    /// Automatic recall slower than this is skipped.
    pub recall_timeout_ms: u64,
    /// Timeout for the explicit tools.
    pub call_timeout_ms: u64,
    /// When true, store/forget tools are not registered (no writes).
    pub read_only: bool,
}

impl Default for CortexConfig {
    fn default() -> Self {
        CortexConfig {
            rest_url: "http://127.0.0.1:47822".into(),
            token: String::new(),
            user_id: String::new(),
            scope: String::new(),
            namespace: String::new(),
            auto_recall: true,
            recall_top_k: 3,
            recall_timeout_ms: 4000,
            call_timeout_ms: 30_000,
            read_only: false,
        }
    }
}

/// Thin REST client for the sidecar's tool endpoint.
#[derive(Clone)]
pub struct CortexClient {
    http: reqwest::Client,
    config: Arc<CortexConfig>,
}

impl CortexClient {
    pub fn new(config: CortexConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| e.to_string())?;
        Ok(CortexClient {
            http,
            config: Arc::new(config),
        })
    }

    /// Invoke one CortexDB tool and return its JSON answer.
    pub async fn call(&self, tool: &str, args: Value, timeout: Duration) -> Result<Value, String> {
        let url = format!(
            "{}/v1/tools/{tool}",
            self.config.rest_url.trim_end_matches('/')
        );
        let mut request = self.http.post(url).json(&args).timeout(timeout);
        if !self.config.token.is_empty() {
            request = request.bearer_auth(&self.config.token);
        }
        let response = request.send().await.map_err(|e| format!("{tool}: {e}"))?;
        let status = response.status();
        let bytes = read_capped(response, MAX_RESPONSE_BYTES)
            .await
            .map_err(|e| format!("{tool}: {e}"))?;
        let body: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{tool}: bad response: {e}"))?;
        if !status.is_success() {
            let message = body
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("request failed");
            return Err(format!("{tool}: HTTP {}: {message}", status.as_u16()));
        }
        Ok(body)
    }

    fn recall_args(&self, query: &str, top_k: u32, graph_light: bool) -> Value {
        let mut args = json!({
            "query": query,
            "top_k_memories": top_k,
            "top_k_knowledge": top_k,
            "graph_light": graph_light,
        });
        for (key, value) in [
            ("user_id", &self.config.user_id),
            ("scope", &self.config.scope),
            ("namespace", &self.config.namespace),
        ] {
            if !value.is_empty() {
                args[key] = json!(value);
            }
        }
        args
    }

    /// Recall hits for `query`, rendered as bullet lines.
    pub async fn recall_lines(
        &self,
        query: &str,
        top_k: u32,
        timeout: Duration,
    ) -> Result<Vec<String>, String> {
        let answer = self
            .call(
                "knowledge_memory_recall",
                self.recall_args(query, top_k, true),
                timeout,
            )
            .await?;
        require_recall_shape(&answer)?;
        Ok(hits_to_lines(&answer))
    }
}

/// Read a response body, aborting as soon as it exceeds `limit` bytes.
async fn read_capped(mut response: reqwest::Response, limit: u64) -> Result<Vec<u8>, String> {
    if response.content_length().is_some_and(|n| n > limit) {
        return Err(format!("response larger than {limit} bytes"));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if (body.len() + chunk.len()) as u64 > limit {
            return Err(format!("response larger than {limit} bytes"));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// A recall answer must carry a `memories` array; anything else is an
/// incompatible sidecar, not "no matches".
pub fn require_recall_shape(answer: &Value) -> Result<(), String> {
    let memories = answer
        .get("memories")
        .and_then(Value::as_array)
        .ok_or("knowledge_memory_recall: response has no `memories` array")?;
    for hit in memories {
        let memory = hit.get("memory");
        let well_formed = memory
            .and_then(|m| m.get("id"))
            .is_some_and(Value::is_string)
            && memory
                .and_then(|m| m.get("content"))
                .is_some_and(Value::is_string);
        if !well_formed {
            return Err(
                "knowledge_memory_recall: a memory hit lacks string `id`/`content`".to_string(),
            );
        }
    }
    match answer.get("knowledge") {
        None | Some(Value::Null) => Ok(()),
        Some(Value::Array(_)) => Ok(()),
        Some(_) => Err("knowledge_memory_recall: `knowledge` is not an array".to_string()),
    }
}

/// `memory_save` acknowledges with the stored memory; require its id.
pub fn require_saved(answer: &Value, id: &str) -> Result<(), String> {
    match answer.pointer("/memory/id").and_then(Value::as_str) {
        Some(saved) if saved == id => Ok(()),
        _ => Err(format!("memory_save did not acknowledge {id}")),
    }
}

/// `memory_delete` acknowledges with `deleted: true`.
pub fn require_deleted(answer: &Value, id: &str) -> Result<(), String> {
    if answer.get("deleted").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err(format!("memory_delete did not confirm deletion of {id}"))
    }
}

/// Render a recall answer as `- id: snippet` lines, memories first.
pub fn hits_to_lines(answer: &Value) -> Vec<String> {
    let mut lines = Vec::new();
    for hit in answer
        .get("memories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let memory = hit.get("memory").unwrap_or(&Value::Null);
        let id = memory.get("id").and_then(Value::as_str).unwrap_or("");
        let text = snippet(
            memory.get("content").and_then(Value::as_str).unwrap_or(""),
            220,
        );
        if !text.is_empty() {
            lines.push(format!("- {id}: {text}"));
        }
    }
    for hit in answer
        .get("knowledge")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let title = hit.get("title").and_then(Value::as_str).unwrap_or("");
        let text = snippet(
            hit.get("snippet").and_then(Value::as_str).unwrap_or(""),
            220,
        );
        if !text.is_empty() {
            lines.push(format!("- {title}: {text}"));
        }
    }
    lines
}

const HEADER: &str =
    "Relevant CortexDB memories for this prompt (retrieved automatically — verify before relying on them):";
const FOOTER: &str =
    "(If this exchange states a durable preference, decision, or fact, save it with cortexdb_memory_store.)";

fn recall_block(lines: &[String], read_only: bool) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let footer = if read_only { "" } else { FOOTER };
    Some(
        format!("{HEADER}\n{}\n{footer}", lines.join("\n"))
            .trim_end()
            .to_string(),
    )
}

fn ok_text(text: String, value: Value) -> ToolExecutionResult {
    ToolExecutionResult::success_text(text, value)
}

/// The `cortexdb-memory` plugin. Requires the `tools` service.
pub fn cortexdb_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "cortexdb-memory",
        vec![cordis::Injection::new(
            crate::api::TOOLS_SERVICE.to_string(),
        )],
        |ctx, config: CortexConfig| async move {
            let client = CortexClient::new(config.clone()).map_err(cordis::Error::msg)?;
            let call_timeout = Duration::from_millis(config.call_timeout_ms);
            let mut specs = Vec::new();

            let search = client.clone();
            specs.push(tool_spec(
                "cortexdb_memory_search",
                "Search CortexDB — the long-term memory and knowledge shared with your other agents — for what was decided, preferred, or learned before. Use it before answering questions about prior decisions, people, preferences or project history.",
                json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "What to look for, in natural language." },
                        "limit": { "type": "integer", "description": "How many memories and knowledge hits to return (default 8)." }
                    },
                    "required": ["query"]
                }),
                move |args| {
                    let client = search.clone();
                    async move {
                        let Some(query) = args.get("query").and_then(Value::as_str) else {
                            return ToolExecutionResult::error("BAD_ARGS", "query is required");
                        };
                        let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(8).clamp(1, 50) as u32;
                        match client.call("knowledge_memory_recall", client.recall_args(query, limit, false), call_timeout).await {
                            Ok(answer) => {
                                if let Err(e) = require_recall_shape(&answer) {
                                    return ToolExecutionResult::error("CORTEXDB", e);
                                }
                                let lines = hits_to_lines(&answer);
                                let text = if lines.is_empty() {
                                    "No CortexDB memories or knowledge matched.".to_string()
                                } else {
                                    lines.join("\n")
                                };
                                ok_text(text, json!({ "hits": lines }))
                            }
                            Err(e) => ToolExecutionResult::error("CORTEXDB", e),
                        }
                    }
                },
            ));

            if !config.read_only {
                let store = client.clone();
                let cfg = config.clone();
                specs.push(tool_spec(
                    "cortexdb_memory_store",
                    "Save one durable fact, preference or decision to CortexDB so later sessions — yours and your other agents' — can recall it. One fact per call, stated so it can be understood without this conversation.",
                    json!({
                        "type": "object",
                        "properties": {
                            "content": { "type": "string", "description": "The fact, self-contained." },
                            "importance": { "type": "number", "description": "0 to 1; how much it matters later (default 0.6)." },
                            "supersedes": { "type": "array", "items": { "type": "string" }, "description": "IDs of memories this one makes untrue." }
                        },
                        "required": ["content"]
                    }),
                    move |args| {
                        let client = store.clone();
                        let cfg = cfg.clone();
                        async move {
                            let Some(content) = args.get("content").and_then(Value::as_str) else {
                                return ToolExecutionResult::error("BAD_ARGS", "content is required");
                            };
                            let id = format!("dsh-rs:{}", super::unique_id());
                            let importance = match args.get("importance") {
                                None | Some(Value::Null) => 0.6,
                                Some(v) => match v.as_f64() {
                                    Some(n) if (0.0..=1.0).contains(&n) => n,
                                    _ => {
                                        return ToolExecutionResult::error(
                                            "BAD_ARGS",
                                            "importance must be a number between 0 and 1",
                                        )
                                    }
                                },
                            };
                            let mut body = json!({
                                "memory_id": id,
                                "scope": if cfg.scope.is_empty() { "global" } else { cfg.scope.as_str() },
                                "content": content,
                                "importance": importance,
                                "metadata": { "source": "dsh-rs", "agent": "dsh-rs" },
                            });
                            if !cfg.namespace.is_empty() { body["namespace"] = json!(cfg.namespace); }
                            if !cfg.user_id.is_empty() { body["user_id"] = json!(cfg.user_id); }
                            if let Some(raw) = args.get("supersedes") {
                                let ids: Option<Vec<&str>> = raw
                                    .as_array()
                                    .and_then(|items| items.iter().map(Value::as_str).collect());
                                match ids {
                                    Some(ids) if ids.iter().all(|i| !i.is_empty()) => body["supersedes"] = json!(ids),
                                    _ => return ToolExecutionResult::error("BAD_ARGS", "supersedes must be an array of non-empty ids"),
                                }
                            }
                            match client.call("memory_save", body, call_timeout).await.and_then(|a| require_saved(&a, &id)) {
                                Ok(()) => ok_text(format!("Saved to CortexDB as {id}."), json!({ "id": id })),
                                Err(e) => ToolExecutionResult::error("CORTEXDB", e),
                            }
                        }
                    },
                ));

                let forget = client.clone();
                specs.push(tool_spec(
                    "cortexdb_memory_forget",
                    "Delete one CortexDB memory by its exact ID. Prefer cortexdb_memory_store with supersedes when a fact has changed: that keeps the history.",
                    json!({
                        "type": "object",
                        "properties": { "id": { "type": "string", "description": "The memory ID, as search returned it." } },
                        "required": ["id"]
                    }),
                    move |args| {
                        let client = forget.clone();
                        async move {
                            let Some(id) = args.get("id").and_then(Value::as_str) else {
                                return ToolExecutionResult::error("BAD_ARGS", "id is required");
                            };
                            match client.call("memory_delete", json!({ "memory_id": id }), call_timeout).await.and_then(|a| require_deleted(&a, id)) {
                                Ok(()) => ok_text(format!("Deleted {id} from CortexDB."), json!({ "id": id })),
                                Err(e) => ToolExecutionResult::error("CORTEXDB", e),
                            }
                        }
                    },
                ));
            }

            register_tools(&ctx, "cortexdb-memory tools", specs).await?;

            if config.auto_recall {
                let recall = client.clone();
                let top_k = config.recall_top_k.clamp(1, 20);
                let timeout = Duration::from_millis(config.recall_timeout_ms);
                let read_only = config.read_only;
                inject_on_pre_step(
                    &ctx,
                    "cortexdb-memory",
                    ContextForm::Recall,
                    move |prompt| {
                        let client = recall.clone();
                        Box::pin(async move {
                            match client.recall_lines(&prompt, top_k, timeout).await {
                                Ok(lines) => recall_block(&lines, read_only),
                                Err(e) => {
                                    eprintln!("cortexdb-memory: recall skipped: {e}");
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
    fn hits_render_memories_then_knowledge() {
        let answer = json!({
            "memories": [{ "memory": { "id": "m1", "content": "a   b\nc" }, "score": 0.9 }],
            "knowledge": [{ "title": "T", "snippet": "snip" }]
        });
        assert_eq!(hits_to_lines(&answer), vec!["- m1: a b c", "- T: snip"]);
    }

    #[test]
    fn acknowledgements_are_checked_not_assumed() {
        assert!(require_saved(&json!({"memory": {"id": "a"}}), "a").is_ok());
        assert!(require_saved(&json!({"memory": {"id": "b"}}), "a").is_err());
        assert!(require_saved(&json!({}), "a").is_err());
        assert!(require_deleted(&json!({"deleted": true}), "a").is_ok());
        assert!(require_deleted(&json!({"deleted": false}), "a").is_err());
        assert!(require_recall_shape(&json!({"memories": []})).is_ok());
        assert!(require_recall_shape(&json!({"error": "x"})).is_err());
        assert!(require_recall_shape(&json!({"memories": [{}]})).is_err());
        assert!(require_recall_shape(&json!({"memories": [], "knowledge": "x"})).is_err());
        assert!(require_recall_shape(
            &json!({"memories": [{"memory": {"id": "a", "content": "b"}}], "knowledge": []})
        )
        .is_ok());
    }

    #[test]
    fn empty_recall_injects_nothing() {
        assert!(recall_block(&[], false).is_none());
    }

    #[test]
    fn read_only_block_omits_the_save_hint() {
        let block = recall_block(&["- x: y".into()], true).unwrap();
        assert!(block.starts_with(HEADER) && !block.contains("cortexdb_memory_store"));
        assert!(recall_block(&["- x: y".into()], false)
            .unwrap()
            .contains("cortexdb_memory_store"));
    }

    #[test]
    fn config_accepts_camel_case_and_defaults() {
        let cfg: CortexConfig =
            serde_json::from_value(json!({ "readOnly": true, "recallTopK": 5 })).unwrap();
        assert!(cfg.read_only && cfg.recall_top_k == 5 && cfg.auto_recall);
    }
}
