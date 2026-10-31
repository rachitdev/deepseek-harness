//! MCP stdio client plugin (`mcp-client`).
//!
//! Spawns each configured MCP server as a child process, speaks
//! newline-delimited JSON-RPC 2.0 over its stdio (`initialize`,
//! `tools/list`, `tools/call`), and registers every server tool as
//! `mcp__<server>__<tool>`, the naming the live harness uses. Unloading the
//! plugin unregisters the tools and kills the children.
//!
//! Config: `{ "servers": { "<name>": { "command", "args", "env",
//! "failOnStartupError", "toolCallTimeoutMs", "startupTimeoutMs" } } }`.
//! `command` and `args` expand a leading `~` and `$HOME`.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cordis::Plugin;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use super::{register_tools, tool_spec};
use crate::types::ToolExecutionResult;

const PROTOCOL_VERSION: &str = "2024-11-05";

/// One MCP server entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    /// Extra environment for the child (added to the inherited one).
    pub env: HashMap<String, String>,
    /// Fail plugin start when this server cannot start (default: skip it).
    pub fail_on_startup_error: bool,
    pub tool_call_timeout_ms: u64,
    pub startup_timeout_ms: u64,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        McpServerConfig {
            command: String::new(),
            args: Vec::new(),
            env: HashMap::new(),
            fail_on_startup_error: false,
            tool_call_timeout_ms: 60_000,
            startup_timeout_ms: 15_000,
        }
    }
}

/// Configuration for the `mcp-client` plugin.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    pub servers: HashMap<String, McpServerConfig>,
}

fn expand(text: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let text = match text.strip_prefix("~/") {
        Some(rest) => format!("{home}/{rest}"),
        None => text.to_string(),
    };
    text.replace("$HOME", &home)
}

type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, String>>>>>;

/// A frame waiting for the writer. A request carries its id: the writer drops it
/// unsent if the caller has already timed out or been cancelled (its pending
/// entry is gone), so a call the caller was told failed cannot run later.
/// Notifications and replies have no id and are always sent.
struct Outgoing {
    frame: Vec<u8>,
    request_id: Option<u64>,
}

/// Frames that may wait for the writer; a full queue means a stalled server.
const OUTBOX_FRAMES: usize = 64;

/// Longest the writer may spend on one frame before the connection is dropped.
const FRAME_WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// Pagination limits for `tools/list`.
const MAX_TOOL_PAGES: usize = 20;
const MAX_TOOLS_PER_SERVER: usize = 500;

/// The protocol revisions this client speaks.
const SUPPORTED_PROTOCOLS: &[&str] = &["2024-11-05", "2025-03-26", "2025-06-18"];

/// Reject a server that answers `initialize` with a revision we do not speak.
fn check_initialize(result: &Value) -> Result<(), String> {
    match result.get("protocolVersion").and_then(Value::as_str) {
        Some(v) if SUPPORTED_PROTOCOLS.contains(&v) => Ok(()),
        Some(v) => Err(format!(
            "unsupported MCP protocol version {v} (supported: {})",
            SUPPORTED_PROTOCOLS.join(", ")
        )),
        None => Err("initialize result has no protocolVersion".to_string()),
    }
}

/// Largest single JSON line accepted from a server; a longer line closes the connection.
const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;

/// Removes a request's pending entry when dropped, so a cancelled or timed-out
/// request can never leave its entry behind.
struct PendingGuard {
    pending: Pending,
    id: u64,
}

impl Drop for PendingGuard {
    fn drop(&mut self) {
        self.pending.lock().unwrap().remove(&self.id);
    }
}

/// A running MCP server connection.
pub struct McpConnection {
    /// Frames queued for the writer task, which alone touches the server's stdin.
    /// A caller that is cancelled can therefore never leave a half-written frame,
    /// and the stdout reader never waits on a write.
    outbox: mpsc::Sender<Outgoing>,
    child: Arc<tokio::sync::Mutex<Child>>,
    reader: Mutex<Option<tokio::task::JoinHandle<()>>>,
    writer: Mutex<Option<tokio::task::JoinHandle<()>>>,
    pending: Pending,
    next_id: AtomicU64,
    /// Set once the connection is unusable (shutdown, EOF, oversized line, or
    /// an interrupted write); later requests fail immediately.
    closed: Arc<AtomicBool>,
}

/// Read one `\n`-terminated line, failing once it exceeds `MAX_LINE_BYTES`.
/// Returns `Ok(None)` at EOF.
async fn read_line_capped<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Option<Vec<u8>>, String> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf().await.map_err(|e| e.to_string())?;
        if available.is_empty() {
            return Ok(if line.is_empty() { None } else { Some(line) });
        }
        let (take, done) = match available.iter().position(|b| *b == b'\n') {
            Some(i) => (i + 1, true),
            None => (available.len(), false),
        };
        if line.len() + take > MAX_LINE_BYTES {
            return Err(format!("server line exceeded {MAX_LINE_BYTES} bytes"));
        }
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if done {
            return Ok(Some(line));
        }
    }
}

impl McpConnection {
    /// Spawn the server and complete the `initialize` handshake.
    pub async fn start(name: &str, config: &McpServerConfig) -> Result<Arc<Self>, String> {
        if config.command.is_empty() {
            return Err(format!("mcp server {name}: command is required"));
        }
        let mut command = Command::new(expand(&config.command));
        command
            .args(config.args.iter().map(|a| expand(a)))
            .envs(&config.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|e| format!("mcp server {name}: cannot start: {e}"))?;
        let stdin = child.stdin.take().ok_or("no stdin")?;
        let stdout = child.stdout.take().ok_or("no stdout")?;
        let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

        let child = Arc::new(tokio::sync::Mutex::new(child));
        let reader_child = Arc::clone(&child);
        let (outbox, mut frames) = mpsc::channel::<Outgoing>(OUTBOX_FRAMES);
        let reader_outbox = outbox.clone();
        let closed = Arc::new(AtomicBool::new(false));
        let (reader_pending, reader_closed) = (Arc::clone(&pending), Arc::clone(&closed));
        let reader = tokio::spawn(async move {
            let mut stdout = BufReader::new(stdout);
            loop {
                let line = match read_line_capped(&mut stdout).await {
                    Ok(Some(line)) => line,
                    Ok(None) => break,
                    Err(e) => {
                        eprintln!("mcp-client: closing connection: {e}");
                        break;
                    }
                };
                let Ok(message) = serde_json::from_slice::<Value>(&line) else {
                    continue;
                };
                if message.get("method").is_some() {
                    // Server-initiated request: this client supports none, so say so
                    // instead of leaving the server waiting for an answer.
                    if let Some(id) = message.get("id").cloned() {
                        let reply = json!({ "jsonrpc": "2.0", "id": id,
                            "error": { "code": -32601, "message": "dsh-rs MCP client does not support server requests" } });
                        let mut frame = serde_json::to_vec(&reply).unwrap_or_default();
                        frame.push(b'\n');
                        // Never wait here: stdout must keep draining. A reply that cannot be
                        // queued would leave the server waiting for it forever, so losing one
                        // is fatal to the connection rather than silently ignored.
                        if reader_outbox
                            .try_send(Outgoing {
                                frame,
                                request_id: None,
                            })
                            .is_err()
                        {
                            eprintln!("mcp-client: cannot queue a reply to a server request; closing the connection");
                            break;
                        }
                    }
                    continue;
                }
                let Some(id) = message.get("id").and_then(Value::as_u64) else {
                    continue;
                };
                let outcome = match message.get("error") {
                    Some(error) => Err(error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("MCP error")
                        .to_string()),
                    None => Ok(message.get("result").cloned().unwrap_or(Value::Null)),
                };
                if let Some(sender) = reader_pending.lock().unwrap().remove(&id) {
                    let _ = sender.send(outcome);
                }
            }
            reader_closed.store(true, Ordering::SeqCst);
            for (_, sender) in reader_pending.lock().unwrap().drain() {
                let _ = sender.send(Err("MCP server closed".to_string()));
            }
            // A connection that cannot read or reply is finished: stop the server too.
            let _ = reader_child.lock().await.kill().await;
        });

        let (writer_closed, writer_pending, writer_child) = (
            Arc::clone(&closed),
            Arc::clone(&pending),
            Arc::clone(&child),
        );
        let mut stdin = stdin;
        let writer = tokio::spawn(async move {
            while let Some(Outgoing { frame, request_id }) = frames.recv().await {
                if let Some(id) = request_id {
                    if !writer_pending.lock().unwrap().contains_key(&id) {
                        continue; // the caller gave up while this was queued
                    }
                }
                let sent = tokio::time::timeout(FRAME_WRITE_TIMEOUT, async {
                    stdin.write_all(&frame).await?;
                    stdin.flush().await
                })
                .await;
                if !matches!(sent, Ok(Ok(()))) {
                    eprintln!(
                        "mcp-client: write to server failed or stalled; closing the connection"
                    );
                    writer_closed.store(true, Ordering::SeqCst);
                    for (_, sender) in writer_pending.lock().unwrap().drain() {
                        let _ = sender.send(Err("MCP server closed".to_string()));
                    }
                    let _ = writer_child.lock().await.kill().await;
                    return;
                }
            }
        });

        let connection = Arc::new(McpConnection {
            outbox,
            child,
            reader: Mutex::new(Some(reader)),
            writer: Mutex::new(Some(writer)),
            pending,
            next_id: AtomicU64::new(1),
            closed,
        });
        let startup = Duration::from_millis(config.startup_timeout_ms);
        let init = json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "dsh-rs", "version": env!("CARGO_PKG_VERSION") }
        });
        let negotiated = match connection.request("initialize", init, startup).await {
            Ok(result) => result,
            Err(e) => {
                connection.shutdown().await;
                return Err(format!("mcp server {name}: initialize: {e}"));
            }
        };
        if let Err(e) = check_initialize(&negotiated) {
            connection.shutdown().await;
            return Err(format!("mcp server {name}: {e}"));
        }
        if let Err(e) = connection
            .notify("notifications/initialized", startup)
            .await
        {
            connection.shutdown().await;
            return Err(format!("mcp server {name}: {e}"));
        }
        Ok(connection)
    }

    /// Queue one frame for the writer task, waiting at most `timeout` for room.
    async fn write(
        &self,
        message: &Value,
        request_id: Option<u64>,
        timeout: Duration,
    ) -> Result<(), String> {
        let mut frame = serde_json::to_vec(message).map_err(|e| e.to_string())?;
        frame.push(b'\n');
        if self.closed.load(Ordering::SeqCst) {
            return Err("MCP server closed".to_string());
        }
        self.outbox
            .send_timeout(Outgoing { frame, request_id }, timeout)
            .await
            .map_err(|e| match e {
                mpsc::error::SendTimeoutError::Timeout(_) => {
                    format!(
                        "MCP server is not reading input (queue full after {} ms)",
                        timeout.as_millis()
                    )
                }
                mpsc::error::SendTimeoutError::Closed(_) => "MCP server closed".to_string(),
            })
    }

    async fn notify(&self, method: &str, timeout: Duration) -> Result<(), String> {
        self.write(
            &json!({ "jsonrpc": "2.0", "method": method }),
            None,
            timeout,
        )
        .await
    }

    /// Send a request and await its result; `timeout` bounds the write and
    /// the response together, so a server that stops reading cannot stall
    /// callers past it.
    pub async fn request(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, String> {
        if self.closed.load(Ordering::SeqCst) {
            return Err("MCP server closed".to_string());
        }
        let started = tokio::time::Instant::now();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, sender);
        let _guard = PendingGuard {
            pending: Arc::clone(&self.pending),
            id,
        };
        // Re-check after registering: the reader may have drained `pending` just before.
        if self.closed.load(Ordering::SeqCst) {
            return Err("MCP server closed".to_string());
        }
        self.write(
            &json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
            Some(id),
            timeout,
        )
        .await?;
        let remaining = timeout.saturating_sub(started.elapsed());
        match tokio::time::timeout(remaining, receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => Err("MCP server closed".to_string()),
            Err(_) => Err(format!(
                "{method} timed out after {} ms",
                timeout.as_millis()
            )),
        }
    }

    /// List the server's tools.
    pub async fn list_tools(&self, timeout: Duration) -> Result<Vec<Value>, String> {
        let mut tools = Vec::new();
        let mut cursor: Option<String> = None;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..MAX_TOOL_PAGES {
            let params = match &cursor {
                Some(c) => json!({ "cursor": c }),
                None => json!({}),
            };
            let result = self.request("tools/list", params, timeout).await?;
            let page = result
                .get("tools")
                .and_then(Value::as_array)
                .ok_or("tools/list result has no `tools` array")?;
            tools.extend(page.iter().cloned());
            if tools.len() > MAX_TOOLS_PER_SERVER {
                return Err(format!(
                    "server lists more than {MAX_TOOLS_PER_SERVER} tools"
                ));
            }
            match result
                .get("nextCursor")
                .and_then(Value::as_str)
                .filter(|c| !c.is_empty())
            {
                None => return Ok(tools),
                Some(next) if seen.insert(next.to_string()) => cursor = Some(next.to_string()),
                Some(_) => return Err("tools/list pagination loops".to_string()),
            }
        }
        Err(format!("tools/list exceeded {MAX_TOOL_PAGES} pages"))
    }

    /// Call one tool and flatten the answer into model-facing text.
    pub async fn call_tool(
        &self,
        tool: &str,
        arguments: Value,
        timeout: Duration,
    ) -> ToolExecutionResult {
        match self
            .request(
                "tools/call",
                json!({ "name": tool, "arguments": arguments }),
                timeout,
            )
            .await
        {
            Err(e) => ToolExecutionResult::error("MCP", e),
            Ok(result) => {
                let text = result_text(&result);
                if result
                    .get("isError")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    ToolExecutionResult::error("MCP_TOOL", text)
                } else {
                    ToolExecutionResult::success_text(
                        text,
                        result
                            .get("structuredContent")
                            .cloned()
                            .unwrap_or(Value::Null),
                    )
                }
            }
        }
    }

    fn abort_tasks(&self) {
        for slot in [&self.reader, &self.writer] {
            if let Some(task) = slot.lock().unwrap().take() {
                task.abort();
            }
        }
    }

    /// Mark the connection closed, stop the reader, kill the child, and fail
    /// every request still waiting. Safe to call repeatedly.
    pub async fn shutdown(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.abort_tasks();
        let _ = self.child.lock().await.kill().await;
        for (_, sender) in self.pending.lock().unwrap().drain() {
            let _ = sender.send(Err("MCP server closed".to_string()));
        }
    }
}

impl Drop for McpConnection {
    /// A dropped connection (including every startup-failure path) stops its
    /// tasks; dropping a `JoinHandle` alone would detach them. The child dies
    /// through `kill_on_drop`.
    fn drop(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
        self.abort_tasks();
    }
}

/// Concatenate the text blocks of a `tools/call` result.
pub fn result_text(result: &Value) -> String {
    let text: Vec<&str> = result
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .collect();
    if text.is_empty() {
        result.to_string()
    } else {
        text.join("\n")
    }
}

/// The registered name of a server tool, e.g. `mcp__jev_review__jev_review`.
pub fn tool_name(server: &str, tool: &str) -> String {
    format!("mcp__{server}__{tool}")
}

/// The `mcp-client` plugin. Requires the `tools` service.
pub fn mcp_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "mcp-client",
        vec![cordis::Injection::new(
            crate::api::TOOLS_SERVICE.to_string(),
        )],
        |ctx, config: McpConfig| async move {
            let mut connections: Vec<Arc<McpConnection>> = Vec::new();
            let mut specs = Vec::new();
            let mut registered: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for server in config.servers.keys() {
                if server.is_empty() || server.contains("__") {
                    return Err(cordis::Error::msg(format!(
                        "mcp server name \"{server}\" must be non-empty and must not contain \"__\" (it separates server from tool in tool names)"
                    )));
                }
            }
            for (server, server_config) in &config.servers {
                let timeout = Duration::from_millis(server_config.tool_call_timeout_ms);
                let started = async {
                    let connection = McpConnection::start(server, server_config).await?;
                    match connection
                        .list_tools(Duration::from_millis(server_config.startup_timeout_ms))
                        .await
                    {
                        Ok(tools) => Ok((connection, tools)),
                        Err(e) => {
                            connection.shutdown().await;
                            Err(format!("mcp server {server}: tools/list: {e}"))
                        }
                    }
                }
                .await;
                let (connection, tools) = match started {
                    Ok(pair) => pair,
                    Err(e) if server_config.fail_on_startup_error => {
                        return Err(cordis::Error::msg(e))
                    }
                    Err(e) => {
                        eprintln!("mcp-client: {server} unavailable, skipped: {e}");
                        continue;
                    }
                };
                for tool in tools {
                    let Some(name) = tool.get("name").and_then(Value::as_str).map(str::to_string)
                    else {
                        continue;
                    };
                    let description = tool
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let schema = tool
                        .get("inputSchema")
                        .cloned()
                        .unwrap_or_else(|| json!({ "type": "object" }));
                    let full_name = tool_name(server, &name);
                    if !registered.insert(full_name.clone()) {
                        eprintln!("mcp-client: {server}: duplicate tool name {full_name} skipped");
                        continue;
                    }
                    let remote = name.clone();
                    let call = Arc::clone(&connection);
                    specs.push(tool_spec(&full_name, &description, schema, move |args| {
                        let connection = Arc::clone(&call);
                        let remote = remote.clone();
                        async move { connection.call_tool(&remote, args, timeout).await }
                    }));
                }
                connections.push(connection);
            }
            register_tools(&ctx, "mcp-client tools", specs).await?;
            ctx.effect("mcp-client servers", async move {
                let disposer: cordis::Disposer = Box::new(move || {
                    Box::pin(async move {
                        for connection in &connections {
                            connection.shutdown().await;
                        }
                    })
                });
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
    fn tool_names_follow_the_harness_convention() {
        assert_eq!(
            tool_name("jev_review", "jev_review"),
            "mcp__jev_review__jev_review"
        );
    }

    #[test]
    fn result_text_joins_text_blocks_and_falls_back_to_json() {
        let r = json!({ "content": [{ "type": "text", "text": "a" }, { "type": "image" }, { "type": "text", "text": "b" }] });
        assert_eq!(result_text(&r), "a\nb");
        assert_eq!(result_text(&json!({ "x": 1 })), r#"{"x":1}"#);
    }

    #[test]
    fn expand_resolves_home() {
        let home = std::env::var("HOME").unwrap();
        assert_eq!(expand("~/x"), format!("{home}/x"));
        assert_eq!(expand("$HOME/y"), format!("{home}/y"));
    }

    #[tokio::test]
    async fn an_oversized_line_is_rejected_without_buffering_it_all() {
        let data = vec![b'x'; MAX_LINE_BYTES + 10];
        let mut reader = BufReader::new(&data[..]);
        assert!(read_line_capped(&mut reader).await.is_err());
        let mut ok = BufReader::new(&b"{\"a\":1}\nrest"[..]);
        assert_eq!(
            read_line_capped(&mut ok).await.unwrap().unwrap(),
            b"{\"a\":1}\n"
        );
        assert_eq!(read_line_capped(&mut ok).await.unwrap().unwrap(), b"rest");
        assert!(read_line_capped(&mut ok).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn a_silent_server_times_out_and_leaves_no_pending_entry() {
        // Answers initialize, then ignores everything.
        let script = r#"read l; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'; read l; sleep 30"#;
        let config = McpServerConfig {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            ..Default::default()
        };
        let connection = McpConnection::start("t", &config).await.expect("handshake");
        let err = connection
            .request("tools/list", json!({}), Duration::from_millis(300))
            .await
            .unwrap_err();
        assert!(err.contains("timed out"), "{err}");
        assert!(
            connection.pending.lock().unwrap().is_empty(),
            "timed-out request must not linger"
        );
        connection.shutdown().await;
        assert!(connection
            .request("tools/list", json!({}), Duration::from_secs(1))
            .await
            .unwrap_err()
            .contains("closed"));
    }

    #[test]
    fn initialize_requires_a_supported_protocol_version() {
        assert!(check_initialize(&json!({"protocolVersion": "2025-03-26"})).is_ok());
        assert!(check_initialize(&json!({"protocolVersion": "1999-01-01"})).is_err());
        assert!(check_initialize(&json!({})).is_err());
    }

    #[tokio::test]
    async fn tool_listing_follows_cursors_and_rejects_loops() {
        let paged = r#"
read l; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
read l
read l; echo '{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"a"}],"nextCursor":"p2"}}'
read l; echo '{"jsonrpc":"2.0","id":3,"result":{"tools":[{"name":"b"}]}}'
"#;
        let config = McpServerConfig {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), paged.into()],
            ..Default::default()
        };
        let connection = McpConnection::start("t", &config).await.unwrap();
        let tools = connection.list_tools(Duration::from_secs(5)).await.unwrap();
        assert_eq!(tools.len(), 2);
        connection.shutdown().await;

        let looping = r#"
read l; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
read l
read l; echo '{"jsonrpc":"2.0","id":2,"result":{"tools":[],"nextCursor":"x"}}'
read l; echo '{"jsonrpc":"2.0","id":3,"result":{"tools":[],"nextCursor":"x"}}'
"#;
        let config = McpServerConfig {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), looping.into()],
            ..Default::default()
        };
        let connection = McpConnection::start("t", &config).await.unwrap();
        assert!(connection
            .list_tools(Duration::from_secs(5))
            .await
            .unwrap_err()
            .contains("loops"));
        connection.shutdown().await;
    }

    #[tokio::test]
    async fn a_server_that_never_reads_input_cannot_stall_callers() {
        // Answers initialize, then neither reads stdin nor writes: the OS pipe and
        // the outbox fill, and callers must still get an error within their deadline.
        let script = r#"read l; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'; read l; sleep 60"#;
        let config = McpServerConfig {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            ..Default::default()
        };
        let connection = McpConnection::start("t", &config).await.expect("handshake");
        let big = json!({ "blob": "x".repeat(512 * 1024) });
        let started = std::time::Instant::now();
        let mut last = Ok(Value::Null);
        for _ in 0..(OUTBOX_FRAMES + 80) {
            last = connection
                .request("tools/call", big.clone(), Duration::from_millis(150))
                .await;
            if last.is_err() && last.as_ref().unwrap_err().contains("not reading") {
                break;
            }
        }
        assert!(
            last.unwrap_err().contains("not reading") || connection.closed.load(Ordering::SeqCst)
        );
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "bounded by deadlines, not by the server"
        );
        connection.shutdown().await;
    }

    #[tokio::test]
    async fn a_call_that_timed_out_while_queued_is_never_sent() {
        let out =
            std::env::temp_dir().join(format!("dsh-rs-mcp-queue-{}", crate::plugins::unique_id()));
        // After the handshake the server stops reading for a while, then copies stdin to a file.
        let script = format!(
            r#"read l; echo '{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":"2024-11-05"}}}}'; read l; sleep 1; cat > {}"#,
            out.display()
        );
        let config = McpServerConfig {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), script],
            ..Default::default()
        };
        let connection = McpConnection::start("t", &config).await.expect("handshake");
        // The first, large request fills the pipe and blocks the writer mid-frame.
        let big = json!({ "blob": "x".repeat(512 * 1024) });
        let first = connection
            .request("tools/call", big, Duration::from_millis(100))
            .await;
        assert!(first.unwrap_err().contains("timed out"));
        // The second request is queued behind it and its caller gives up before it is sent.
        let second = connection
            .request(
                "tools/call/EXPIRED_MARKER",
                json!({}),
                Duration::from_millis(100),
            )
            .await;
        assert!(second.unwrap_err().contains("timed out"));
        tokio::time::sleep(Duration::from_millis(2500)).await; // server resumes and drains stdin
        connection.shutdown().await;
        let received = std::fs::read_to_string(&out).unwrap_or_default();
        let _ = std::fs::remove_file(&out);
        assert!(
            received.contains("\"blob\""),
            "the in-flight frame is completed whole"
        );
        assert!(
            !received.contains("EXPIRED_MARKER"),
            "an expired queued request must not be sent"
        );
    }

    #[tokio::test]
    async fn server_requests_are_answered_with_method_not_found() {
        // Sends a server request (id 77), then collects client lines (the tools/list
        // request and the -32601 reply may arrive in either order) before answering.
        let script = r#"
read l; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'
read l
echo '{"jsonrpc":"2.0","id":77,"method":"sampling/createMessage","params":{}}'
text=silent; got_list=no
while [ "$got_list" = no ] || [ "$text" = silent ]; do
  read -r l || break
  case "$l" in *-32601*) text=replied;; *tools/list*) got_list=yes;; esac
done
echo "{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"tools\":[{\"name\":\"$text\"}]}}"
"#;
        let config = McpServerConfig {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            ..Default::default()
        };
        let connection = McpConnection::start("t", &config).await.expect("handshake");
        let tools = connection
            .list_tools(Duration::from_secs(5))
            .await
            .expect("list");
        assert_eq!(tools[0]["name"], "replied");
        connection.shutdown().await;
    }

    #[tokio::test]
    async fn a_cancelled_request_leaves_no_pending_entry() {
        let script = r#"read l; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"}}'; read l; sleep 30"#;
        let config = McpServerConfig {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            ..Default::default()
        };
        let connection = McpConnection::start("t", &config).await.expect("handshake");
        let call = connection.request("tools/list", json!({}), Duration::from_secs(30));
        let _ = tokio::time::timeout(Duration::from_millis(200), call).await; // drops the future
        assert!(connection.pending.lock().unwrap().is_empty());
        connection.shutdown().await;
    }

    #[tokio::test]
    async fn a_missing_server_is_an_error_not_a_hang() {
        let config = McpServerConfig {
            command: "/nonexistent/server".into(),
            ..Default::default()
        };
        assert!(McpConnection::start("x", &config).await.is_err());
    }

    #[tokio::test]
    async fn handshake_lists_and_calls_a_scripted_server() {
        // A shell "server": answers initialize, tools/list, tools/call by request id order.
        let script = r#"
read l; echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{}}}'
read l
read l; echo '{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"echo","description":"d","inputSchema":{"type":"object"}}]}}'
read l; echo '{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"pong"}]}}'
"#;
        let config = McpServerConfig {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            ..Default::default()
        };
        let connection = McpConnection::start("t", &config).await.expect("handshake");
        let tools = connection
            .list_tools(Duration::from_secs(5))
            .await
            .expect("list");
        assert_eq!(tools[0]["name"], "echo");
        let result = connection
            .call_tool("echo", json!({}), Duration::from_secs(5))
            .await;
        assert_eq!(result.content()[0].as_text(), Some("pong"));
        connection.shutdown().await;
    }
}
