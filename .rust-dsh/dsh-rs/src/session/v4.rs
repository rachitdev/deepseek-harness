//! Row codec for the TS harness's session-log format v4.
//!
//! A v4 log is one header row followed by event rows `{type, seq, time, data}`
//! with optional `surfaceOp`, `sourceEventSeqs` and `ignorable` envelope keys.
//! This module is pure (no I/O): it maps Rust [`SessionEvent`]s to rows and
//! back. [`crate::session::v4_store`] owns files, compression and locking.
//!
//! Mapping rules, all checked against the real TS validator (see
//! `tests/oracle/ts_validate.mjs`):
//!
//! * One Rust event encodes to exactly one row and keeps its `seq`, so Rust
//!   references such as `Compaction::before_seq` stay valid.
//! * Events with a TS equivalent use the TS vocabulary. Rust-only events
//!   (`assistant-chunk`, `goal`, `compaction`) are written as `ignorable`
//!   `plugin:dsh-rs/*` rows: the TS reader skips them, Rust restores them
//!   losslessly. A TS reader therefore sees an uncompacted history.
//! * The Rust system prompt rides in `request/header` as `data.dshSystem`;
//!   the TS retired `header.system` field is never written.
//! * Reading is a projection: TS rows without a Rust equivalent (hooks, jobs,
//!   approvals, titles, ...) are counted and skipped, never invented.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::types::agent::{InboxOutcome, InboxTarget};
use crate::types::{
    GoalChange, ContentBlock, ContextForm, EpochHeader, LlmCallConfig, Message, MessageSource, Role,
    SessionEvent, SessionEventData, TodoItem, TodoStatus, TokenUsage, ToolSchema, TurnEndReason,
};

/// The format generation this codec reads and writes.
pub const FORMAT_VERSION: u64 = 4;

/// Version of the `goal/change` payload this codec reads and writes.
const GOAL_CHANGE_VERSION: u64 = 1;

const PLUGIN_ROW_CHUNK: &str = "plugin:dsh-rs/assistant-chunk";
const PLUGIN_ROW_GOAL: &str = "plugin:dsh-rs/goal";
const PLUGIN_ROW_COMPACTION: &str = "plugin:dsh-rs/compaction";

/// A codec failure; the message names the offending row where there is one.
#[derive(Debug, thiserror::Error)]
pub enum V4Error {
    #[error("v4 row at seq {seq} ({kind}): {reason}")]
    Row {
        seq: u64,
        kind: String,
        reason: String,
    },
    #[error("v4 log: {0}")]
    Log(String),
}

fn row_err(seq: u64, kind: &str, reason: impl Into<String>) -> V4Error {
    V4Error::Row {
        seq,
        kind: kind.to_string(),
        reason: reason.into(),
    }
}

/// Header line of a v4 log (`{"type":"session","version":4,...}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct V4Header {
    #[serde(rename = "type")]
    pub kind: String,
    pub version: u64,
    pub id: String,
    pub created_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default)]
    pub is_seeded: bool,
    #[serde(default)]
    pub delegation_depth: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_preset: Option<String>,
}

impl V4Header {
    /// A header for a session created by this runtime.
    pub fn new(
        id: &str,
        created_at: u64,
        cwd: Option<String>,
        parent_session: Option<String>,
        is_seeded: bool,
    ) -> Self {
        Self::with_preset(id, created_at, cwd, parent_session, is_seeded, None)
    }

    /// A header that names the agent preset the session runs.
    pub fn with_preset(
        id: &str,
        created_at: u64,
        cwd: Option<String>,
        parent_session: Option<String>,
        is_seeded: bool,
        agent_preset: Option<String>,
    ) -> Self {
        V4Header {
            kind: "session".to_string(),
            version: FORMAT_VERSION,
            id: id.to_string(),
            created_at,
            cwd,
            is_seeded,
            delegation_depth: 0,
            parent_session,
            origin: None,
            agent_preset,
        }
    }

    /// Parse and check one header line; a non-v4 header is a hard error.
    pub fn parse(line: &str) -> Result<Self, V4Error> {
        let header: V4Header = serde_json::from_str(line)
            .map_err(|err| V4Error::Log(format!("header line is not a v4 header: {err}")))?;
        if header.kind != "session" || header.version != FORMAT_VERSION {
            return Err(V4Error::Log(format!(
                "unsupported header (type {:?}, version {}); only v{FORMAT_VERSION} is read",
                header.kind, header.version
            )));
        }
        Ok(header)
    }
}

/// One event row with its envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct V4Row {
    #[serde(rename = "type")]
    pub kind: String,
    pub seq: u64,
    pub time: u64,
    pub data: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_op: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_event_seqs: Option<Vec<u64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignorable: Option<bool>,
}

/// Per-session state the encoder needs across rows.
#[derive(Debug, Default)]
pub struct EncodeState {
    /// `tool/call` row seq by call id, for the `sourceEventSeqs` of its result.
    call_seqs: HashMap<String, u64>,
    headers_written: u64,
    /// The log header says `isSeeded`: its first end-seed row is the inherited marker.
    seeded: bool,
    inherited_marker_written: bool,
}

impl EncodeState {
    /// State for a log whose header has the given `isSeeded` value.
    pub fn new(seeded: bool) -> Self {
        EncodeState {
            seeded,
            ..Default::default()
        }
    }

    /// Whether the inherited end-seed marker TS requires of a seeded log is written.
    pub fn inherited_marker_written(&self) -> bool {
        self.inherited_marker_written
    }

    /// Record an inherited end-seed row that is already on disk.
    pub fn note_inherited_marker(&mut self) {
        self.inherited_marker_written = true;
    }

    /// Record a `tool/call` row that is already on disk.
    pub fn remember_call(&mut self, call_id: &str, seq: u64) {
        self.call_seqs.insert(call_id.to_string(), seq);
    }

    /// Record a `request/header` row that is already on disk.
    pub fn note_header_written(&mut self) {
        self.headers_written += 1;
    }
}

fn plugin_row(kind: &str, event: &SessionEvent, data: Value) -> V4Row {
    V4Row {
        kind: kind.to_string(),
        seq: event.seq,
        time: event.time,
        data,
        surface_op: None,
        source_event_seqs: None,
        ignorable: Some(true),
    }
}

fn plain_row(event: &SessionEvent, kind: &str, data: Value) -> V4Row {
    V4Row {
        kind: kind.to_string(),
        seq: event.seq,
        time: event.time,
        data,
        surface_op: None,
        source_event_seqs: None,
        ignorable: None,
    }
}

fn surface_row(event: &SessionEvent, kind: &str, data: Value, sources: Option<Vec<u64>>) -> V4Row {
    V4Row {
        surface_op: Some(json!("append")),
        source_event_seqs: sources,
        ..plain_row(event, kind, data)
    }
}

fn to_value<T: Serialize>(value: &T, event: &SessionEvent) -> Result<Value, V4Error> {
    serde_json::to_value(value)
        .map_err(|err| row_err(event.seq, event.event_type(), err.to_string()))
}

fn encode_source(source: &MessageSource, event: &SessionEvent) -> Result<Value, V4Error> {
    Ok(match source {
        MessageSource::User { rpc_id } => {
            let mut map = Map::new();
            map.insert("kind".into(), json!("user"));
            if let Some(rpc_id) = rpc_id {
                map.insert("rpcId".into(), json!(rpc_id));
            }
            Value::Object(map)
        }
        MessageSource::Model { provider, model } => {
            json!({"kind": "model", "provider": provider, "model": model})
        }
        // The reminder source is the TypeScript `schedule` kind, written bare so
        // the TypeScript reader sees the same row.
        MessageSource::Plugin { plugin, form: None } if plugin == "schedule" => json!({"kind": "schedule"}),
        MessageSource::Plugin { plugin, form } => {
            let mut map = Map::new();
            map.insert("kind".into(), json!(format!("plugin:{plugin}")));
            map.insert("plugin".into(), json!(plugin));
            if let Some(form) = form {
                if let Value::Object(fields) = to_value(form, event)? {
                    map.extend(fields);
                }
            }
            Value::Object(map)
        }
        MessageSource::Tool { tool } => json!({"kind": "tool", "tool": tool}),
    })
}

fn encode_message(message: &Message, event: &SessionEvent) -> Result<Value, V4Error> {
    Ok(json!({
        "content": to_value(&message.content, event)?,
        "id": message.id,
        "role": "user",
        "source": encode_source(&message.source, event)?,
    }))
}

fn encode_reason(reason: &TurnEndReason) -> Value {
    match reason {
        TurnEndReason::Completed => json!({"kind": "completed"}),
        TurnEndReason::Blocked => json!({"kind": "blocked"}),
        TurnEndReason::MaxTokens => json!({"kind": "max-tokens"}),
        TurnEndReason::Interrupted => json!({"kind": "interrupted"}),
        TurnEndReason::Error { message, code } => {
            json!({"kind": "error", "error": {"message": message, "code": code}})
        }
        TurnEndReason::Aborted { cause } => {
            let cause = match cause.as_str() {
                "user" | "parent" | "disposed" | "legacy" => json!({"kind": cause}),
                other => {
                    json!({"kind": "hook", "reason": other.strip_prefix("hook:").unwrap_or(other)})
                }
            };
            json!({"kind": "aborted", "reason": cause})
        }
    }
}

fn encode_usage(usage: &TokenUsage) -> Value {
    let mut map = Map::new();
    map.insert("inputTokens".into(), json!(usage.input_tokens));
    map.insert("outputTokens".into(), json!(usage.output_tokens));
    for (key, value) in [
        ("cacheReadTokens", usage.cache_read_tokens),
        ("cacheWriteTokens", usage.cache_write_tokens),
        ("reasoningTokens", usage.reasoning_tokens),
    ] {
        if let Some(value) = value {
            map.insert(key.into(), json!(value));
        }
    }
    Value::Object(map)
}

fn encode_config(config: &LlmCallConfig) -> Value {
    let mut map = Map::new();
    map.insert("provider".into(), json!(config.provider));
    map.insert("model".into(), json!(config.model));
    if let Some(value) = config.temperature {
        map.insert("temperature".into(), json!(value));
    }
    if let Some(value) = config.max_tokens {
        map.insert("maxTokens".into(), json!(value));
    }
    if let Some(value) = &config.stop {
        map.insert("stop".into(), json!(value));
    }
    Value::Object(map)
}

/// Encode one Rust event as exactly one v4 row.
pub fn encode_event(event: &SessionEvent, state: &mut EncodeState) -> Result<V4Row, V4Error> {
    use SessionEventData as D;
    Ok(match &event.data {
        D::TurnStart { turn } => plain_row(event, "turn/start", json!({"turn": turn})),
        D::TurnEnd { turn, reason } => plain_row(
            event,
            "turn/end",
            json!({"turn": turn, "reason": encode_reason(reason)}),
        ),
        D::StepStart { turn, step } => {
            plain_row(event, "step/start", json!({"turn": turn, "step": step}))
        }
        D::StepEnd { turn, step } => {
            plain_row(event, "step/end", json!({"turn": turn, "step": step}))
        }
        D::UserMessage { message } => {
            surface_row(event, "user/message", encode_message(message, event)?, None)
        }
        D::AssistantChunk { turn, step, chunk } => plugin_row(
            PLUGIN_ROW_CHUNK,
            event,
            json!({"turn": turn, "step": step, "chunk": to_value(chunk, event)?}),
        ),
        D::AssistantMessage {
            turn,
            step,
            message,
            usage,
            interrupted,
        } => {
            let MessageSource::Model { provider, model } = &message.source else {
                return Err(row_err(
                    event.seq,
                    "assistant/message",
                    "message source must be a model",
                ));
            };
            let mut data = json!({
                "turn": turn,
                "step": step,
                "message": {
                    "role": "assistant",
                    "id": message.id,
                    "content": to_value(&message.content, event)?,
                    "source": {"kind": "model", "provider": provider, "model": model},
                },
                "stream": [],
            });
            if let Some(usage) = usage {
                data["usage"] = encode_usage(usage);
            }
            if *interrupted == Some(true) {
                data["interrupted"] = json!(true);
            }
            surface_row(event, "assistant/message", data, None)
        }
        D::PermissionPreset { preset } => {
            plain_row(event, "permission/preset", json!({ "preset": preset }))
        }
        D::SandboxMode { mode } => plain_row(event, "sandbox/mode", json!({ "mode": mode })),
        D::ApprovalPolicy { policy, source } => {
            let mut data = json!({ "policy": policy });
            if let Some(source) = source {
                data["source"] = json!(source);
            }
            plain_row(event, "approval/policy", data)
        }
        D::PlanMode { active } => plain_row(event, "plan/mode", json!({ "active": active })),
        D::AgentPresetSelected { agent_preset } => plain_row(
            event,
            "agent-preset/selected",
            json!({"agentPreset": agent_preset}),
        ),
        D::ToolCall {
            turn,
            step,
            call_id,
            name,
            arguments,
        } => {
            state.call_seqs.insert(call_id.clone(), event.seq);
            plain_row(
                event,
                "tool/call",
                json!({"turn": turn, "step": step, "callId": call_id, "name": name, "arguments": arguments}),
            )
        }
        D::ToolResult {
            turn,
            step,
            message,
        } => {
            let [ContentBlock::ToolResult {
                tool_call_id,
                content,
                is_error,
            }] = message.content.as_slice()
            else {
                return Err(row_err(
                    event.seq,
                    "tool/result",
                    "message must hold exactly one tool-result block",
                ));
            };
            let tool = match &message.source {
                MessageSource::Tool { tool } => tool.clone(),
                _ => String::new(),
            };
            let call_seq = *state.call_seqs.get(tool_call_id).ok_or_else(|| {
                row_err(
                    event.seq,
                    "tool/result",
                    format!("no tool/call row for call {tool_call_id}"),
                )
            })?;
            let data = json!({
                "turn": turn,
                "step": step,
                "message": {
                    "role": "tool",
                    "id": message.id,
                    "source": {"kind": "tool", "callId": tool_call_id, "tool": tool},
                    "toolCallId": tool_call_id,
                    "content": to_value(content, event)?,
                    "isError": is_error.unwrap_or(false),
                },
            });
            surface_row(event, "tool/result", data, Some(vec![call_seq]))
        }
        D::TodoWrite { todos } => {
            let todos: Vec<Value> = todos
                .iter()
                .map(|todo| json!({"content": todo.content, "status": encode_todo_status(todo.status)}))
                .collect();
            plain_row(event, "todo/write", json!({"todos": todos}))
        }
        D::Compaction {
            summary,
            before_seq,
        } => plugin_row(
            PLUGIN_ROW_COMPACTION,
            event,
            json!({"summary": summary, "beforeSeq": before_seq}),
        ),
        D::Goal {
            objective,
            status,
            note,
        } => plugin_row(
            PLUGIN_ROW_GOAL,
            event,
            json!({"objective": objective, "status": to_value(status, event)?, "note": note}),
        ),
        D::GoalChange { change } => plain_row(event, "goal/change", goal_change_data(change, event)?),
        // TS requires `messageSeqs` empty exactly when the source is `user`.
        D::SessionTitle { title } => plain_row(
            event,
            "session/title",
            json!({ "title": title, "messageSeqs": [], "source": { "kind": "user" } }),
        ),
        D::CommandRun { command_id, name, args } => {
            let mut data = serde_json::Map::new();
            data.insert("commandId".to_string(), json!(command_id));
            data.insert("name".to_string(), json!(name));
            if let Some(args) = args {
                data.insert("args".to_string(), json!(args));
            }
            data.insert("source".to_string(), json!({ "kind": "user" }));
            plain_row(event, "command/run", Value::Object(data))
        }
        D::CommandDone { command_id, success, text, source_event_seq } => {
            let mut data = serde_json::Map::new();
            data.insert("commandId".to_string(), json!(command_id));
            data.insert("kind".to_string(), json!(if *success { "success" } else { "error" }));
            if let Some(text) = text {
                data.insert("text".to_string(), json!(text));
            }
            if let Some(seq) = source_event_seq {
                data.insert("sourceEventSeq".to_string(), json!(seq));
            }
            plain_row(event, "command/done", Value::Object(data))
        }
        D::FeedbackRecord { text, category } => {
            let mut data = serde_json::Map::new();
            if let Some(text) = text {
                data.insert("text".to_string(), json!(text));
            }
            if let Some(category) = category {
                data.insert("category".to_string(), json!(category));
            }
            plain_row(event, "feedback/record", Value::Object(data))
        }
        D::FeedbackMessagePut { session_id, item } => plain_row(
            event,
            "feedback/message-put",
            json!({ "sessionId": session_id, "item": to_value(item, event)? }),
        ),
        D::FeedbackMessageDelete { session_id, message_id } => plain_row(
            event,
            "feedback/message-delete",
            json!({ "sessionId": session_id, "messageId": message_id }),
        ),
        D::ModelSelection { provider, model } => plain_row(
            event,
            "model/selection",
            json!({ "provider": provider, "model": model }),
        ),
        D::InboxSpliced {
            target,
            start,
            removed_count,
            inserted,
            outcome,
        } => {
            let mut data = json!({
                "target": target.as_str_name(),
                "start": start,
                "inserted": to_value(inserted, event)?,
            });
            if let Some(count) = removed_count {
                data["removedCount"] = json!(count);
            }
            if let Some(outcome) = outcome {
                data["outcome"] = json!(match outcome {
                    crate::types::agent::InboxOutcome::Canceled => "canceled",
                });
            }
            plain_row(event, "agent/inbox/spliced", data)
        }
        D::RequestHeader { header } => {
            let mut inner = Map::new();
            inner.insert("config".into(), encode_config(&header.config));
            if let Some(tools) = &header.tools {
                inner.insert("tools".into(), to_value(tools, event)?);
            }
            let reason = if state.headers_written == 0 {
                "initial"
            } else {
                "series"
            };
            state.headers_written += 1;
            let mut data = json!({"header": Value::Object(inner), "reason": reason});
            if let Some(system) = &header.system {
                data["dshSystem"] = json!(system);
            }
            plain_row(event, "request/header", data)
        }
        D::SessionEndSeed => {
            // TS: a seeded log needs exactly one marker with `inherited: true`; an
            // unseeded log must have none.
            let inherited = state.seeded && !state.inherited_marker_written;
            state.inherited_marker_written |= inherited;
            plain_row(
                event,
                "session/end-seed",
                if inherited {
                    json!({"inherited": true})
                } else {
                    json!({})
                },
            )
        }
    })
}

fn encode_todo_status(status: TodoStatus) -> &'static str {
    match status {
        TodoStatus::Pending => "pending",
        TodoStatus::InProgress => "in_progress",
        TodoStatus::Completed => "completed",
    }
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

fn get<'a>(row: &V4Row, value: &'a Value, key: &str) -> Result<&'a Value, V4Error> {
    value
        .get(key)
        .ok_or_else(|| row_err(row.seq, &row.kind, format!("missing field `{key}`")))
}

fn get_u64(row: &V4Row, value: &Value, key: &str) -> Result<u64, V4Error> {
    get(row, value, key)?.as_u64().ok_or_else(|| {
        row_err(
            row.seq,
            &row.kind,
            format!("field `{key}` is not an unsigned integer"),
        )
    })
}

/// A string field that may be absent; present but not a string is an error.
fn get_opt_str(row: &V4Row, value: &Value, key: &str) -> Result<Option<String>, V4Error> {
    match value.get(key) {
        None => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(row_err(row.seq, &row.kind, format!("field `{key}` is not a string"))),
    }
}

fn get_str<'a>(row: &V4Row, value: &'a Value, key: &str) -> Result<&'a str, V4Error> {
    get(row, value, key)?
        .as_str()
        .ok_or_else(|| row_err(row.seq, &row.kind, format!("field `{key}` is not a string")))
}

/// Count of content the projection could not carry; surfaced to the caller.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Lossy {
    /// Content blocks of a type Rust has no equivalent for, kept as text.
    pub placeholder_blocks: usize,
}

fn decode_blocks(
    row: &V4Row,
    value: &Value,
    lossy: &mut Lossy,
) -> Result<Vec<ContentBlock>, V4Error> {
    let blocks = value
        .as_array()
        .ok_or_else(|| row_err(row.seq, &row.kind, "message content is not a list"))?;
    let mut out = Vec::with_capacity(blocks.len());
    for block in blocks {
        let kind = block.get("type").and_then(Value::as_str).unwrap_or("");
        let text = |key: &str| block.get(key).and_then(Value::as_str).map(str::to_string);
        out.push(match kind {
            "text" => ContentBlock::Text {
                text: text("text").unwrap_or_default(),
            },
            "reasoning" => ContentBlock::Reasoning {
                text: text("text").unwrap_or_default(),
            },
            "tool-call" => ContentBlock::ToolCall {
                id: text("id").unwrap_or_default(),
                name: text("name").unwrap_or_default(),
                arguments: text("arguments").unwrap_or_default(),
            },
            "image" => image_block(block).unwrap_or_else(|| {
                lossy.placeholder_blocks += 1;
                ContentBlock::Text {
                    text: "[image: unreadable attachment reference]".to_string(),
                }
            }),
            "file" => file_block(block).unwrap_or_else(|| {
                // A reference without an id, name and size cannot be read back.
                lossy.placeholder_blocks += 1;
                ContentBlock::Text {
                    text: "[file: unreadable attachment reference]".to_string(),
                }
            }),
            other => {
                lossy.placeholder_blocks += 1;
                ContentBlock::Text {
                    text: format!("[unsupported content block: {other}]"),
                }
            }
        });
    }
    Ok(out)
}

fn image_block(block: &Value) -> Option<ContentBlock> {
    let reference = block.get("attachment")?;
    let size = |name: &str| u32::try_from(reference.get(name)?.as_u64()?).ok();
    let original = match reference.get("originalDimensions") {
        None => None,
        Some(dimensions) => Some(crate::types::ImageSize {
            width: u32::try_from(dimensions.get("width")?.as_u64()?).ok()?,
            height: u32::try_from(dimensions.get("height")?.as_u64()?).ok()?,
        }),
    };
    Some(ContentBlock::Image {
        attachment: crate::types::ImageRef {
            attachment_id: reference.get("attachmentId")?.as_str()?.to_string(),
            media_type: reference.get("mediaType")?.as_str()?.to_string(),
            bytes: reference.get("bytes")?.as_u64()?,
            width: size("width")?,
            height: size("height")?,
            name: reference
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string),
            original_dimensions: original,
        },
    })
}

fn file_block(block: &Value) -> Option<ContentBlock> {
    let reference = block.get("attachment")?;
    Some(ContentBlock::File {
        attachment: crate::types::FileRef {
            attachment_id: reference.get("attachmentId")?.as_str()?.to_string(),
            name: reference.get("name")?.as_str()?.to_string(),
            bytes: reference.get("bytes")?.as_u64()?,
        },
    })
}

fn decode_form(source: &Value) -> Option<ContextForm> {
    let form = source.get("form")?.as_str()?;
    let mut object = Map::new();
    object.insert("form".into(), json!(form));
    match form {
        "snapshot" => {
            let sections = source.get("sections").cloned().unwrap_or_else(|| json!([]));
            object.insert("sections".into(), sections);
        }
        "notice" => {
            let summary = source.get("summary").cloned().unwrap_or_else(|| json!(""));
            object.insert("summary".into(), summary);
        }
        _ => {}
    }
    serde_json::from_value(Value::Object(object)).ok()
}

fn decode_source(source: &Value) -> MessageSource {
    let kind = source.get("kind").and_then(Value::as_str).unwrap_or("");
    let field = |key: &str| source.get(key).and_then(Value::as_str).map(str::to_string);
    match kind {
        "user" => MessageSource::User {
            rpc_id: field("rpcId"),
        },
        "model" => MessageSource::Model {
            provider: field("provider").unwrap_or_default(),
            model: field("model").unwrap_or_default(),
        },
        "tool" => MessageSource::Tool {
            tool: field("tool").unwrap_or_default(),
        },
        other => MessageSource::Plugin {
            plugin: field("plugin")
                .unwrap_or_else(|| other.strip_prefix("plugin:").unwrap_or(other).to_string()),
            form: decode_form(source),
        },
    }
}

fn decode_reason(row: &V4Row, reason: &Value) -> Result<TurnEndReason, V4Error> {
    let kind = get_str(row, reason, "kind")?;
    Ok(match kind {
        "completed" => TurnEndReason::Completed,
        "blocked" => TurnEndReason::Blocked,
        "max-tokens" => TurnEndReason::MaxTokens,
        "interrupted" | "forked" => TurnEndReason::Interrupted,
        "error" => {
            let error = get(row, reason, "error")?;
            TurnEndReason::Error {
                message: error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                code: error
                    .get("code")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            }
        }
        "aborted" => {
            let cause = get(row, reason, "reason")?;
            let cause_kind = get_str(row, cause, "kind")?;
            let cause = match cause_kind {
                "hook" => format!(
                    "hook:{}",
                    cause.get("reason").and_then(Value::as_str).unwrap_or("")
                ),
                other => other.to_string(),
            };
            TurnEndReason::Aborted { cause }
        }
        other => {
            return Err(row_err(
                row.seq,
                &row.kind,
                format!("unknown turn end reason `{other}`"),
            ))
        }
    })
}

fn decode_usage(usage: &Value) -> TokenUsage {
    let num = |key: &str| usage.get(key).and_then(Value::as_u64);
    TokenUsage {
        input_tokens: num("inputTokens").unwrap_or(0),
        output_tokens: num("outputTokens").unwrap_or(0),
        cache_read_tokens: num("cacheReadTokens"),
        cache_write_tokens: num("cacheWriteTokens"),
        reasoning_tokens: num("reasoningTokens"),
    }
}

fn decode_todo_status(row: &V4Row, status: &str) -> Result<TodoStatus, V4Error> {
    match status {
        "pending" => Ok(TodoStatus::Pending),
        "in_progress" | "in-progress" => Ok(TodoStatus::InProgress),
        "completed" => Ok(TodoStatus::Completed),
        other => Err(row_err(
            row.seq,
            &row.kind,
            format!("unknown todo status `{other}`"),
        )),
    }
}

fn decode_header(row: &V4Row, data: &Value) -> Result<EpochHeader, V4Error> {
    let header = get(row, data, "header")?;
    let config = get(row, header, "config")?;
    let text = |key: &str| config.get(key).and_then(Value::as_str).map(str::to_string);
    let tools: Option<Vec<ToolSchema>> = match header.get("tools") {
        Some(tools) => Some(
            serde_json::from_value(tools.clone())
                .map_err(|err| row_err(row.seq, &row.kind, format!("tools: {err}")))?,
        ),
        None => None,
    };
    Ok(EpochHeader {
        config: LlmCallConfig {
            provider: text("provider").unwrap_or_default(),
            model: text("model").unwrap_or_default(),
            temperature: config
                .get("temperature")
                .and_then(Value::as_f64)
                .map(|v| v as f32),
            max_tokens: config
                .get("maxTokens")
                .and_then(Value::as_u64)
                .map(|v| v as u32),
            stop: config.get("stop").and_then(Value::as_array).map(|list| {
                list.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            }),
        },
        system: data
            .get("dshSystem")
            .and_then(Value::as_str)
            .map(str::to_string),
        tools,
    })
}

fn decode_user_message(row: &V4Row, lossy: &mut Lossy) -> Result<Message, V4Error> {
    decode_message_value(row, &row.data, lossy)
}

/// One message carried inside a row or a list element.
fn decode_message_value(
    row: &V4Row,
    value: &Value,
    lossy: &mut Lossy,
) -> Result<Message, V4Error> {
    // A row written before a message carried its role held a user message.
    let role = match value.get("role") {
        None | Some(Value::Null) => Role::User,
        Some(Value::String(role)) => match role.as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "system" => Role::System,
            other => {
                return Err(row_err(
                    row.seq,
                    &row.kind,
                    format!("unknown message role {other:?}"),
                ))
            }
        },
        Some(_) => return Err(row_err(row.seq, &row.kind, "field `role` is not a string")),
    };
    Ok(Message {
        id: get_str(row, value, "id")?.to_string(),
        role,
        content: decode_blocks(row, get(row, value, "content")?, lossy)?,
        source: decode_source(get(row, value, "source")?),
    })
}

/// One `agent/inbox/spliced` row: a durable mutation of one pending list.
fn decode_inbox_spliced(
    row: &V4Row,
    data: &Value,
    lossy: &mut Lossy,
) -> Result<SessionEventData, V4Error> {
    let target = match get_str(row, data, "target")? {
        "next-turn" => InboxTarget::NextTurn,
        "next-step" => InboxTarget::NextStep,
        other => {
            return Err(row_err(
                row.seq,
                &row.kind,
                format!("unknown inbox target {other:?}"),
            ))
        }
    };
    let start = get_u64(row, data, "start")? as usize;
    let removed_count = match data.get("removedCount") {
        None | Some(Value::Null) => None,
        Some(value) => Some(value.as_u64().ok_or_else(|| {
            row_err(
                row.seq,
                &row.kind,
                "field `removedCount` is not an unsigned integer",
            )
        })? as usize),
    };
    let inserted_value = get(row, data, "inserted")?;
    let inserted_list = inserted_value
        .as_array()
        .ok_or_else(|| row_err(row.seq, &row.kind, "field `inserted` is not a list"))?;
    let mut inserted = Vec::with_capacity(inserted_list.len());
    for message in inserted_list {
        inserted.push(decode_message_value(row, message, lossy)?);
    }
    let outcome = match data.get("outcome") {
        None | Some(Value::Null) => None,
        Some(Value::String(other)) if other == "canceled" => Some(InboxOutcome::Canceled),
        Some(other) => {
            return Err(row_err(
                row.seq,
                &row.kind,
                format!("unknown inbox outcome {other}"),
            ))
        }
    };
    Ok(SessionEventData::InboxSpliced {
        target,
        start,
        removed_count,
        inserted,
        outcome,
    })
}

fn decode_tool_result(row: &V4Row, lossy: &mut Lossy) -> Result<Message, V4Error> {
    let message = get(row, &row.data, "message")?;
    let call_id = get_str(row, message, "toolCallId")?.to_string();
    let source = decode_source(get(row, message, "source")?);
    let inner = decode_blocks(row, get(row, message, "content")?, lossy)?;
    Ok(Message {
        id: get_str(row, message, "id")?.to_string(),
        role: Role::User,
        content: vec![ContentBlock::ToolResult {
            tool_call_id: call_id,
            content: inner,
            is_error: Some(
                message
                    .get("isError")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            ),
        }],
        source,
    })
}

fn decode_assistant_message(row: &V4Row, lossy: &mut Lossy) -> Result<SessionEventData, V4Error> {
    let message = get(row, &row.data, "message")?;
    let source = decode_source(get(row, message, "source")?);
    Ok(SessionEventData::AssistantMessage {
        turn: get_u64(row, &row.data, "turn")?,
        step: get_u64(row, &row.data, "step")?,
        message: Message {
            id: get_str(row, message, "id")?.to_string(),
            role: Role::Assistant,
            content: decode_blocks(row, get(row, message, "content")?, lossy)?,
            source,
        },
        usage: row.data.get("usage").map(decode_usage),
        interrupted: row.data.get("interrupted").and_then(Value::as_bool),
    })
}

/// The result of decoding one row.
#[derive(Debug, PartialEq)]
pub enum Decoded {
    Event(SessionEventData),
    /// The row has no Rust equivalent (or is a surface edit handled by [`decode_log`]).
    Skipped,
}

fn decode_plugin_row(row: &V4Row) -> Result<Decoded, V4Error> {
    let from = |value: &Value| -> Result<_, V4Error> { Ok(value.clone()) };
    let data = from(&row.data)?;
    let bad = |err: serde_json::Error| row_err(row.seq, &row.kind, err.to_string());
    Ok(match row.kind.as_str() {
        PLUGIN_ROW_CHUNK => Decoded::Event(SessionEventData::AssistantChunk {
            turn: get_u64(row, &data, "turn")?,
            step: get_u64(row, &data, "step")?,
            chunk: serde_json::from_value(get(row, &data, "chunk")?.clone()).map_err(bad)?,
        }),
        PLUGIN_ROW_GOAL => Decoded::Event(SessionEventData::Goal {
            objective: get_str(row, &data, "objective")?.to_string(),
            status: serde_json::from_value(get(row, &data, "status")?.clone()).map_err(bad)?,
            note: data.get("note").and_then(Value::as_str).map(str::to_string),
        }),
        PLUGIN_ROW_COMPACTION => Decoded::Event(SessionEventData::Compaction {
            summary: get_str(row, &data, "summary")?.to_string(),
            before_seq: get_u64(row, &data, "beforeSeq")?,
        }),
        _ => Decoded::Skipped,
    })
}

/// The `goal/change` payload TypeScript writes: a version, the operation, and
/// the whole post-mutation goal or the clear tombstone.
fn goal_change_data(change: &GoalChange, event: &SessionEvent) -> Result<Value, V4Error> {
    Ok(match change {
        GoalChange::Snapshot {
            operation,
            goal,
            rounds_started,
            created_at,
            updated_at,
        } => json!({
            "kind": "goal/change",
            "version": GOAL_CHANGE_VERSION,
            "operation": operation.as_str(),
            "goal": to_value(goal, event)?,
            "roundsStarted": rounds_started,
            "createdAt": created_at,
            "updatedAt": updated_at,
        }),
        GoalChange::Clear {
            cleared,
            cleared_at,
        } => json!({
            "kind": "goal/change",
            "version": GOAL_CHANGE_VERSION,
            "operation": "clear",
            "cleared": to_value(cleared, event)?,
            "clearedAt": cleared_at,
        }),
    })
}

/// Decode a `goal/change` row. A row of another version, or one that does not
/// parse, is an error: the goal projection would otherwise fold a wrong state.
fn decode_goal_change(row: &V4Row, data: &Value) -> Result<GoalChange, V4Error> {
    let version = get_u64(row, data, "version")?;
    if version != GOAL_CHANGE_VERSION {
        return Err(row_err(
            row.seq,
            &row.kind,
            format!("unsupported goal change version {version}"),
        ));
    }
    let bad = |err: serde_json::Error| row_err(row.seq, &row.kind, err.to_string());
    if get_str(row, data, "operation")? == "clear" {
        return Ok(GoalChange::Clear {
            cleared: serde_json::from_value(get(row, data, "cleared")?.clone()).map_err(bad)?,
            cleared_at: get_u64(row, data, "clearedAt")?,
        });
    }
    Ok(GoalChange::Snapshot {
        operation: serde_json::from_value(get(row, data, "operation")?.clone()).map_err(bad)?,
        goal: serde_json::from_value(get(row, data, "goal")?.clone()).map_err(bad)?,
        rounds_started: get_u64(row, data, "roundsStarted")?,
        created_at: get_u64(row, data, "createdAt")?,
        updated_at: get_u64(row, data, "updatedAt")?,
    })
}

/// Decode one row. Surface edits (`surfaceOp: replace`) are decoded as the
/// message they carry; [`decode_log`] applies their replace semantics.
pub fn decode_event(row: &V4Row, lossy: &mut Lossy) -> Result<Decoded, V4Error> {
    use SessionEventData as D;
    let data = &row.data;
    Ok(Decoded::Event(match row.kind.as_str() {
        "turn/start" => D::TurnStart {
            turn: get_u64(row, data, "turn")?,
        },
        "turn/end" => D::TurnEnd {
            turn: get_u64(row, data, "turn")?,
            reason: decode_reason(row, get(row, data, "reason")?)?,
        },
        "step/start" => D::StepStart {
            turn: get_u64(row, data, "turn")?,
            step: get_u64(row, data, "step")?,
        },
        "step/end" => D::StepEnd {
            turn: get_u64(row, data, "turn")?,
            step: get_u64(row, data, "step")?,
        },
        "user/message" => D::UserMessage {
            message: decode_user_message(row, lossy)?,
        },
        "assistant/message" => decode_assistant_message(row, lossy)?,
        "tool/call" => D::ToolCall {
            turn: get_u64(row, data, "turn")?,
            step: get_u64(row, data, "step")?,
            call_id: get_str(row, data, "callId")?.to_string(),
            name: get_str(row, data, "name")?.to_string(),
            arguments: get_str(row, data, "arguments")?.to_string(),
        },
        "tool/result" => D::ToolResult {
            turn: get_u64(row, data, "turn")?,
            step: get_u64(row, data, "step")?,
            message: decode_tool_result(row, lossy)?,
        },
        "todo/write" => {
            let list = get(row, data, "todos")?
                .as_array()
                .ok_or_else(|| row_err(row.seq, &row.kind, "todos is not a list"))?;
            let mut todos = Vec::with_capacity(list.len());
            for todo in list {
                todos.push(TodoItem {
                    content: get_str(row, todo, "content")?.to_string(),
                    status: decode_todo_status(row, get_str(row, todo, "status")?)?,
                });
            }
            D::TodoWrite { todos }
        }
        // Only a user rename is modelled; a derived title is counted and skipped.
        "session/title"
            if data.pointer("/source/kind").and_then(Value::as_str) == Some("user") =>
        {
            D::SessionTitle {
                title: get_str(row, data, "title")?.to_string(),
            }
        }
        "goal/change" => D::GoalChange {
            change: decode_goal_change(row, data)?,
        },
        "command/run" => D::CommandRun {
            command_id: get_str(row, data, "commandId")?.to_string(),
            name: get_str(row, data, "name")?.to_string(),
            args: get_opt_str(row, data, "args")?,
        },
        "command/done" => D::CommandDone {
            command_id: get_str(row, data, "commandId")?.to_string(),
            success: match get_str(row, data, "kind")? {
                "success" => true,
                "error" => false,
                other => {
                    return Err(row_err(
                        row.seq,
                        &row.kind,
                        format!("field `kind` is {other:?}, not success or error"),
                    ))
                }
            },
            text: get_opt_str(row, data, "text")?,
            source_event_seq: match data.get("sourceEventSeq") {
                None => None,
                Some(value) => Some(value.as_u64().ok_or_else(|| {
                    row_err(row.seq, &row.kind, "field `sourceEventSeq` is not an unsigned integer".to_string())
                })?),
            },
        },
        "feedback/record" => D::FeedbackRecord {
            text: get_opt_str(row, data, "text")?,
            category: get_opt_str(row, data, "category")?,
        },
        "feedback/message-put" => D::FeedbackMessagePut {
            session_id: get_str(row, data, "sessionId")?.to_string(),
            item: serde_json::from_value(get(row, data, "item")?.clone())
                .map_err(|err| row_err(row.seq, &row.kind, format!("field `item`: {err}")))?,
        },
        "feedback/message-delete" => D::FeedbackMessageDelete {
            session_id: get_str(row, data, "sessionId")?.to_string(),
            message_id: get_str(row, data, "messageId")?.to_string(),
        },
        "model/selection" => D::ModelSelection {
            provider: get_str(row, data, "provider")?.to_string(),
            model: get_str(row, data, "model")?.to_string(),
        },
        "permission/preset" => D::PermissionPreset {
            preset: get_str(row, data, "preset")?.to_string(),
        },
        "sandbox/mode" => D::SandboxMode {
            mode: get_str(row, data, "mode")?.to_string(),
        },
        "approval/policy" => D::ApprovalPolicy {
            policy: get_str(row, data, "policy")?.to_string(),
            source: data.get("source").and_then(Value::as_str).map(str::to_string),
        },
        "plan/mode" => D::PlanMode {
            active: data
                .get("active")
                .and_then(Value::as_bool)
                .ok_or_else(|| row_err(row.seq, &row.kind, "active is not a boolean"))?,
        },
        "agent-preset/selected" => D::AgentPresetSelected {
            agent_preset: get_str(row, data, "agentPreset")?.to_string(),
        },
        "agent/inbox/spliced" => decode_inbox_spliced(row, data, lossy)?,
        "request/header" => D::RequestHeader {
            header: decode_header(row, data)?,
        },
        "session/end-seed" => D::SessionEndSeed,
        kind if kind.starts_with("plugin:dsh-rs/") => return decode_plugin_row(row),
        _ => return Ok(Decoded::Skipped),
    }))
}

/// A whole log decoded to Rust events.
#[derive(Debug, Default)]
pub struct DecodedLog {
    pub events: Vec<SessionEvent>,
    /// Skipped row count by TS event type.
    pub skipped: BTreeMap<String, usize>,
    /// The seq after the last row, kept even when the last row was dropped.
    pub next_seq: u64,
    /// Single-node replaces folded into the node they replaced.
    pub folded_replaces: usize,
    /// Compaction checkpoints whose range started after the first surviving
    /// non-system node; Rust drops everything before the range end, so the
    /// reconstructed model view can omit nodes the TS surface kept.
    pub approximate_compactions: usize,
    pub lossy: Lossy,
}

#[derive(Debug, Clone, Copy)]
struct Replace {
    start: u64,
    end: u64,
}

fn parse_surface_op(row: &V4Row) -> Result<Option<Replace>, V4Error> {
    match &row.surface_op {
        None => Ok(None),
        Some(Value::String(op)) if op == "append" => Ok(None),
        Some(op) => {
            let valid = op.get("op").and_then(Value::as_str) == Some("replace");
            let start = op.get("startSeq").and_then(Value::as_u64);
            let end = op.get("endSeq").and_then(Value::as_u64);
            match (valid, start, end) {
                // Ranges are in surface order, so after an earlier compaction
                // `startSeq` may exceed `endSeq`; `sourceEventSeqs` is the set.
                (true, Some(start), Some(end)) => Ok(Some(Replace { start, end })),
                _ => Err(row_err(row.seq, &row.kind, "malformed surfaceOp")),
            }
        }
    }
}

fn is_checkpoint(row: &V4Row) -> bool {
    row.data.pointer("/source/kind").and_then(Value::as_str) == Some("compact-checkpoint")
}

fn message_text(message: &Message) -> String {
    message.text()
}

/// Decode every row of a log in order, applying surface replacements.
///
/// Fails on a malformed row or on a replace this projection cannot express;
/// rows that merely have no Rust equivalent are counted in `skipped`.
pub fn decode_log(rows: &[V4Row]) -> Result<DecodedLog, V4Error> {
    let mut log = DecodedLog::default();
    let mut index: HashMap<u64, usize> = HashMap::new();
    let mut surface: Vec<u64> = Vec::new();
    let mut expected: Option<u64> = None;
    for row in rows {
        if let Some(expected) = expected {
            if row.seq != expected {
                return Err(row_err(
                    row.seq,
                    &row.kind,
                    format!("seq gap (expected {expected})"),
                ));
            }
        }
        expected = Some(row.seq + 1);
        log.next_seq = row.seq + 1;
        let replace = parse_surface_op(row)?;
        let mut skip =
            |log: &mut DecodedLog| *log.skipped.entry(row.kind.clone()).or_default() += 1;
        if let Some(range) = replace {
            apply_replace(row, range, &mut log, &mut index, &mut surface, &mut skip)?;
            continue;
        }
        match decode_event(row, &mut log.lossy)? {
            Decoded::Skipped => skip(&mut log),
            Decoded::Event(data) => {
                track_surface(&mut surface, row.seq, &data);
                index.insert(row.seq, log.events.len());
                log.events.push(SessionEvent::new(row.seq, row.time, data));
            }
        }
    }
    Ok(log)
}

fn track_surface(surface: &mut Vec<u64>, seq: u64, data: &SessionEventData) {
    match data {
        SessionEventData::Compaction { before_seq, .. } => {
            surface.retain(|node| node >= before_seq);
            surface.insert(0, seq);
        }
        data if data.is_surface() => surface.push(seq),
        _ => {}
    }
}

fn apply_replace(
    row: &V4Row,
    range: Replace,
    log: &mut DecodedLog,
    index: &mut HashMap<u64, usize>,
    surface: &mut Vec<u64>,
    skip: &mut dyn FnMut(&mut DecodedLog),
) -> Result<(), V4Error> {
    // The system prompt is not a Rust surface node.
    if row.kind == "system/message" {
        skip(log);
        return Ok(());
    }
    if is_checkpoint(row) {
        let mut lossy = Lossy::default();
        let message = decode_user_message(row, &mut lossy)?;
        log.lossy.placeholder_blocks += lossy.placeholder_blocks;
        let shadowed: Vec<u64> = row.source_event_seqs.clone().unwrap_or_default();
        let shadowed_surface: Vec<u64> = shadowed
            .iter()
            .copied()
            .filter(|seq| surface.contains(seq))
            .collect();
        // Rust folds "everything before `before_seq`"; TS names an exact set.
        let before_seq = shadowed_surface.iter().max().map_or(0, |max| max + 1);
        if surface
            .iter()
            .any(|node| *node < before_seq && !shadowed.contains(node))
        {
            log.approximate_compactions += 1;
        }
        let data = SessionEventData::Compaction {
            summary: message_text(&message),
            before_seq,
        };
        track_surface(surface, row.seq, &data);
        index.insert(row.seq, log.events.len());
        log.events.push(SessionEvent::new(row.seq, row.time, data));
        return Ok(());
    }
    if range.start != range.end || !matches!(row.kind.as_str(), "tool/result" | "user/message") {
        return Err(row_err(
            row.seq,
            &row.kind,
            format!(
                "unsupported surface replace of seqs {}..={}",
                range.start, range.end
            ),
        ));
    }
    // A single-node replace swaps the node's message in place, which keeps its
    // position in the model-visible order.
    let Some(&at) = index.get(&range.start) else {
        skip(log);
        return Ok(());
    };
    let mut lossy = Lossy::default();
    let replacement = match row.kind.as_str() {
        "tool/result" => decode_tool_result(row, &mut lossy)?,
        _ => decode_user_message(row, &mut lossy)?,
    };
    log.lossy.placeholder_blocks += lossy.placeholder_blocks;
    match &mut log.events[at].data {
        SessionEventData::ToolResult { message, .. }
        | SessionEventData::UserMessage { message } => {
            *message = replacement;
        }
        _ => {
            return Err(row_err(
                row.seq,
                &row.kind,
                "replace target is not a user or tool message",
            ));
        }
    }
    log.folded_replaces += 1;
    Ok(())
}
