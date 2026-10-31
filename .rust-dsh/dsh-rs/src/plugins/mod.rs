//! Native Rust plugins that attach the DSH ecosystem services to dsh-rs.
//!
//! Each plugin mirrors its TypeScript counterpart in the live harness:
//!
//! - [`cortexdb`] — long-term memory over the CortexDB sidecar's REST API.
//! - [`skillforge`] — skill routing through the `skillforge` CLI.
//! - [`skills`] — installs `SKILL.md` skills: system-prompt catalog plus a `skill` loader tool.
//! - [`bridge`] — delegates tasks to the full TypeScript DSH ecosystem over the SDK JSON-RPC protocol.
//! - [`mcp`] — stdio MCP client; attaches MCP servers (e.g. `jev_review`) as tools.
//!
//! Shared helpers here register tools as fiber effects and inject recalled
//! context on `agent/pre-step`.

pub mod approvals;
pub mod ask_user;
pub mod frontend;
pub mod bridge;
pub mod compaction;
pub mod cortexdb;
pub mod goal;
pub mod mcp;
pub mod permission;
pub mod plan;
pub mod schedule;
pub mod skillforge;
pub mod skills;
pub mod subagent;
pub mod web;

use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use serde_json::Value;

use crate::api::services::{BoxFuture, DynamicToolSpec, ToolsService};
use crate::types::{ContentBlock, ContextForm, Message, MessageSource, Role, ToolExecutionResult};

/// Minimum prompt length (chars) before a plugin spends a lookup on it.
pub const MIN_PROMPT_CHARS: usize = 12;

/// Longest a single pre-step lookup may delay a turn. The lookup runs before
/// the turn proceeds, so this is a hard ceiling per listener, not a promise
/// that enrichment is free; each plugin also applies its own, shorter timeout.
pub const PRE_STEP_DEADLINE: Duration = Duration::from_secs(8);

static ID_SEQ: AtomicU64 = AtomicU64::new(0);

/// A process-unique id: wall-clock nanoseconds, pid, and a monotonic counter,
/// so two calls in the same clock tick or after a clock step still differ.
pub fn unique_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!(
        "{nanos:x}-{:x}-{:x}",
        std::process::id(),
        ID_SEQ.fetch_add(1, Ordering::Relaxed)
    )
}

/// Register `specs` on the tools service; unloading the plugin unregisters them.
pub async fn register_tools(
    ctx: &Context,
    label: &str,
    specs: Vec<DynamicToolSpec>,
) -> cordis::Result<()> {
    let tools = ctx.require::<ToolsService>(crate::api::TOOLS_SERVICE)?;
    let names: Vec<String> = specs.iter().map(|spec| spec.name.clone()).collect();
    for spec in specs {
        tools.register_dynamic_tool(spec);
    }
    ctx.effect(label.to_string(), async move {
        let disposer: cordis::Disposer = Box::new(move || {
            Box::pin(async move {
                for name in &names {
                    tools.unregister_dynamic_tool(name);
                }
            })
        });
        Ok(Some(disposer))
    })
    .await?;
    Ok(())
}

/// Build a tool spec from an async body over the parsed JSON arguments.
pub fn tool_spec<F, Fut>(
    name: &str,
    description: &str,
    parameters: Value,
    body: F,
) -> DynamicToolSpec
where
    F: Fn(Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ToolExecutionResult> + Send + 'static,
{
    DynamicToolSpec {
        name: name.to_string(),
        description: description.to_string(),
        parameters,
        exec: Arc::new(move |args| Box::pin(body(args)) as BoxFuture<ToolExecutionResult>),
        exec_ctx: None,
    }
}

/// Like [`tool_spec`], but the body also receives the calling agent's run context.
pub fn tool_spec_ctx<F, Fut>(
    name: &str,
    description: &str,
    parameters: Value,
    body: F,
) -> DynamicToolSpec
where
    F: Fn(Value, crate::types::ToolRunContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ToolExecutionResult> + Send + 'static,
{
    DynamicToolSpec {
        name: name.to_string(),
        description: description.to_string(),
        parameters,
        exec: Arc::new(|_| {
            Box::pin(async {
                ToolExecutionResult::error(
                    "NO_CONTEXT",
                    "this tool needs the calling agent's context",
                )
            }) as BoxFuture<ToolExecutionResult>
        }),
        exec_ctx: Some(Arc::new(move |args, run| {
            Box::pin(body(args, run)) as BoxFuture<ToolExecutionResult>
        })),
    }
}

/// Text of the latest user-authored message in a pre-step batch.
pub fn last_user_text(messages: &[Value]) -> String {
    messages
        .iter()
        .rev()
        .find(|m| m.pointer("/source/kind").and_then(Value::as_str) == Some("user"))
        .and_then(|m| m.get("content").and_then(Value::as_array))
        .map(|blocks| {
            blocks
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

/// A plugin-sourced context message, serialized the way the session log expects.
pub fn context_message(plugin: &str, form: ContextForm, text: String) -> Value {
    let message = Message {
        id: format!("ctx-{plugin}-{}", unique_id()),
        role: Role::User,
        content: vec![ContentBlock::text(text)],
        source: MessageSource::Plugin {
            plugin: plugin.to_string(),
            form: Some(form),
        },
    };
    serde_json::to_value(message).unwrap_or(Value::Null)
}

/// Append a context block to every `agent/pre-step` batch that enters.
///
/// `build` receives the latest user prompt and returns the block to inject,
/// or `None` to leave the batch untouched. The lookup is awaited before the
/// turn proceeds, so it delays the turn by at most [`PRE_STEP_DEADLINE`]; a
/// lookup that fails or exceeds that deadline is skipped.
pub async fn inject_on_pre_step<B>(
    ctx: &Context,
    plugin: &'static str,
    form: ContextForm,
    build: B,
) -> cordis::Result<()>
where
    B: Fn(String) -> BoxFuture<Option<String>> + Send + Sync + 'static,
{
    let build = Arc::new(build);
    ctx.on("agent/pre-step", move |_ctx, payload, next| {
        let build = Arc::clone(&build);
        let form = form.clone();
        Box::pin(async move {
            let prompt = payload
                .get("messages")
                .and_then(Value::as_array)
                .map(|messages| last_user_text(messages))
                .unwrap_or_default();
            let block = if prompt.chars().count() >= MIN_PROMPT_CHARS {
                tokio::time::timeout(PRE_STEP_DEADLINE, build(prompt))
                    .await
                    .unwrap_or_default()
            } else {
                None
            };
            let mut decision = next.run(payload).await?;
            let Some(text) = block else {
                return Ok(decision);
            };
            if decision.get("kind").and_then(Value::as_str) != Some("enter") {
                return Ok(decision);
            }
            if let Some(messages) = decision.get_mut("messages").and_then(Value::as_array_mut) {
                messages.push(context_message(plugin, form, text));
            }
            Ok(decision)
        }) as BoxFuture<cordis::Result<Value>>
    })
    .await?;
    Ok(())
}

/// A short, single-line excerpt of `text`, at most `max` characters.
pub fn snippet(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        return flat;
    }
    let cut: String = flat.chars().take(max).collect();
    format!("{}…", cut.trim_end())
}

/// Run one tool-less model request to completion and return its text.
///
/// Used where a plugin needs a model's judgement outside the agent loop (a
/// compaction summary, a goal verdict). The request goes through the same
/// `llm/stream` waterfall as an agent's own, so providers, retries and
/// recording behave identically. A provider error, an abort and a timeout are
/// all `Err`; the caller decides whether failing closed or open is right.
pub async fn one_shot_text(
    ctx: &Context,
    llm: &crate::api::services::LlmService,
    streams: &crate::llm::StreamTable,
    options: crate::types::GenerateOptions,
    timeout: Duration,
) -> Result<String, String> {
    use crate::llm::{stream_via_waterfall, BlockAssembler, FinishReason};
    let work = async {
        let stream = stream_via_waterfall(ctx, llm.clone(), streams.clone(), options)
            .await
            .map_err(|e| e.to_string())?;
        let mut stream = std::pin::pin!(stream);
        let mut assembler = BlockAssembler::new();
        while let Some(chunk) = futures::StreamExt::next(&mut stream).await {
            assembler.push(&chunk);
        }
        match assembler.finish() {
            FinishReason::Error { failure } | FinishReason::Aborted { failure } => {
                Err(format!("model request failed: {failure:?}"))
            }
            _ => Ok(assembler
                .blocks()
                .iter()
                .filter_map(|b| b.as_text())
                .collect::<String>()),
        }
    };
    tokio::time::timeout(timeout, work)
        .await
        .map_err(|_| format!("model request timed out after {} ms", timeout.as_millis()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_ids_never_repeat() {
        let ids: std::collections::HashSet<String> = (0..1000).map(|_| unique_id()).collect();
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn snippet_flattens_and_truncates() {
        assert_eq!(snippet("a  b\n c", 10), "a b c");
        assert_eq!(snippet("abcdef", 3), "abc…");
    }

    #[test]
    fn last_user_text_picks_latest_user_message() {
        let messages = vec![
            serde_json::json!({"source":{"kind":"user"},"content":[{"type":"text","text":"first"}]}),
            serde_json::json!({"source":{"kind":"plugin"},"content":[{"type":"text","text":"ctx"}]}),
            serde_json::json!({"source":{"kind":"user"},"content":[{"type":"text","text":"second"}]}),
        ];
        assert_eq!(last_user_text(&messages), "second");
    }

    #[test]
    fn context_message_round_trips_as_a_message() {
        let value = context_message("t", ContextForm::Recall, "hello".into());
        let message: Message = serde_json::from_value(value).expect("valid Message");
        assert_eq!(message.text(), "hello");
        assert!(matches!(message.source, MessageSource::Plugin { .. }));
    }
}
