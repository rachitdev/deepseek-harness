//! `GET /api/session.export`: one session's log, its subagent descendants' logs
//! and every attachment those logs reference, as a ZIP archive.
//!
//! The archive layout is the TypeScript host's (`session-log-export`): the root
//! log at `session.v4.jsonl`, each descendant at
//! `subagents/<id>/session.v4.jsonl`, each image at `media/<attachmentId>.<ext>`
//! and each file at `files/<prefix>/<digest>/<name>`. No manifest is written.
//!
//! Differences from the TypeScript host:
//! - The archive is built in memory and sent whole, not streamed, so a failure
//!   while reading a descendant or an attachment is answered as `500` before
//!   any byte is sent instead of cutting the download short.
//! - Attachments are optional here. A log that references none exports with no
//!   attachment store composed; one that references any needs the store.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::Arc;

use cordis::Context;
use http::header::CONTENT_DISPOSITION;
use http::{HeaderMap, HeaderValue, Method};
use serde_json::Value;

use super::router::{persistence, Router};
use super::server::HttpReply;
use crate::api::services::{SessionPersistenceApi, SessionService};
use crate::attachments::AttachmentStore;
use crate::types::{FileRef, ImageRef};

/// The route the Web client's download link points at.
pub const SESSION_LOG_EXPORT_PATH: &str = "/api/session.export";

/// The log's base name inside the archive: the current format generation.
const LOG_FILENAME: &str = "session.v4.jsonl";

const TEXT: &str = "text/plain; charset=utf-8";

/// DEFLATE level used when a deployment sets none.
pub const DEFAULT_COMPRESSION_LEVEL: u8 = 6;

/// Why an export cannot be produced, as the status and public text it answers.
#[derive(Debug)]
struct Refusal {
    status: u16,
    text: &'static str,
    /// Detail for the operator's log only: it can name host paths.
    detail: Option<String>,
}

impl Refusal {
    fn new(status: u16, text: &'static str) -> Self {
        Refusal {
            status,
            text,
            detail: None,
        }
    }

    fn with_detail(mut self, detail: impl std::fmt::Display) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    fn read_failed(detail: impl std::fmt::Display) -> Self {
        Refusal::new(500, "session log export failed to read the stored log").with_detail(detail)
    }

    /// A stored log that cannot be read; a backend that keeps no such format
    /// is told apart from a read that failed.
    fn log_failed(error: std::io::Error) -> Self {
        if error.kind() == std::io::ErrorKind::Unsupported {
            return Refusal::new(
                500,
                "session log export is unavailable: the session store does not keep logs in the session-log format",
            );
        }
        Refusal::read_failed(error)
    }
}

/// The query of an export request.
struct Request {
    session_id: String,
    descendants: bool,
}

fn parse_query(query: Option<&str>) -> Result<Request, Refusal> {
    let invalid = || Refusal::new(400, "missing or invalid sessionId query parameter");
    let mut session_id = None;
    let mut descendants = None;
    // The last occurrence wins, as `Object.fromEntries` over the URL's params does.
    for (key, value) in form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
        match key.as_ref() {
            "sessionId" => session_id = Some(value.into_owned()),
            "includeDescendants" => descendants = Some(value.into_owned()),
            _ => {}
        }
    }
    let descendants = match descendants.as_deref() {
        None | Some("false") => false,
        Some("true") => true,
        Some(_) => return Err(invalid()),
    };
    match session_id {
        Some(session_id) if !session_id.is_empty() => Ok(Request {
            session_id,
            descendants,
        }),
        _ => Err(invalid()),
    }
}

/// One path segment made of the characters a session id may safely contribute
/// to an archive entry.
fn safe_segment(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

/// The download name the browser offers.
fn archive_filename(session_id: &str) -> String {
    format!("dsh-session-{}.zip", safe_segment(session_id))
}

fn image_extension(media_type: &str) -> Option<&'static str> {
    match media_type {
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        _ => None,
    }
}

fn image_entry_path(image: &ImageRef) -> Option<String> {
    Some(format!(
        "media/{}.{}",
        image.attachment_id,
        image_extension(&image.media_type)?
    ))
}

fn file_entry_path(file: &FileRef) -> String {
    let digest = file
        .attachment_id
        .strip_prefix("sha256:")
        .unwrap_or(&file.attachment_id);
    let name: String = file
        .name
        .chars()
        .map(|c| if matches!(c, '/' | '\\' | '\u{0}'..='\u{1f}' | '\u{7f}') { '_' } else { c })
        .collect();
    let name = if matches!(name.as_str(), "" | "." | "..") {
        "file".to_string()
    } else {
        name
    };
    let prefix: String = digest.chars().take(2).collect();
    format!("files/{prefix}/{digest}/{name}")
}

/// The distinct attachments the logs reference, in first-seen order.
#[derive(Default)]
struct Attachments {
    images: Vec<ImageRef>,
    image_ids: HashSet<String>,
    files: Vec<FileRef>,
    file_keys: HashSet<(String, String)>,
}

impl Attachments {
    fn note_block(&mut self, block: &Value) -> Result<(), String> {
        let Some(object) = block.as_object() else {
            return Ok(());
        };
        let Some(attachment) = object.get("attachment").filter(|a| a.is_object()) else {
            return Ok(());
        };
        match object.get("type").and_then(Value::as_str) {
            Some("image") => {
                let image: ImageRef = serde_json::from_value(attachment.clone())
                    .map_err(|error| format!("an image reference cannot be read: {error}"))?;
                if self.image_ids.insert(image.attachment_id.clone()) {
                    self.images.push(image);
                }
            }
            Some("file") => {
                let file: FileRef = serde_json::from_value(attachment.clone())
                    .map_err(|error| format!("a file reference cannot be read: {error}"))?;
                if self
                    .file_keys
                    .insert((file.attachment_id.clone(), file.name.clone()))
                {
                    self.files.push(file);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn note_content(&mut self, content: Option<&Value>) -> Result<(), String> {
        for block in content.and_then(Value::as_array).into_iter().flatten() {
            self.note_block(block)?;
        }
        Ok(())
    }

    /// Note the references of one log row. Only the declared content fields
    /// of known row types count; every other row and field is opaque.
    fn note_row(&mut self, row: &Value) -> Result<(), String> {
        let (Some(kind), Some(data)) = (row.get("type").and_then(Value::as_str), row.get("data"))
        else {
            return Ok(());
        };
        if !data.is_object() {
            return Ok(());
        }
        let message_content = || data.get("message").and_then(|m| m.get("content"));
        match kind {
            "user/message" | "tool/ptc-dispatch" => self.note_content(data.get("content")),
            "system/message" | "developer/message" | "tool/result" | "team/message/queued" => {
                self.note_content(message_content())
            }
            "agent/inbox/spliced" => {
                for message in data
                    .get("inserted")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if message.is_object() {
                        self.note_content(message.get("content"))?;
                    }
                }
                Ok(())
            }
            "compaction/summary" => {
                self.note_content(data.get("summary"))?;
                self.note_content(data.get("rawOutput"))
            }
            "assistant/message" => {
                self.note_content(message_content())?;
                self.note_stream(data)
            }
            "assistant/attempt" => self.note_stream(data),
            _ => Ok(()),
        }
    }

    /// Blocks a streamed reply finished: `block-end` chunks carry the block.
    fn note_stream(&mut self, data: &Value) -> Result<(), String> {
        for record in data
            .get("stream")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let chunk = record.get("chunk");
            if record.get("type").and_then(Value::as_str) == Some("chunk")
                && chunk.and_then(|c| c.get("type")).and_then(Value::as_str) == Some("block-end")
            {
                if let Some(block) = chunk.and_then(|c| c.get("block")) {
                    self.note_block(block)?;
                }
            }
        }
        Ok(())
    }

    /// Note every row of one log's text. A line that does not parse cannot
    /// reference anything and is skipped; the text is archived as it is.
    fn note_log(&mut self, log: &str) -> Result<(), String> {
        for line in log.lines().filter(|line| !line.is_empty()) {
            if let Ok(row) = serde_json::from_str::<Value>(line) {
                self.note_row(&row)?;
            }
        }
        Ok(())
    }
}

/// A session with its parent, for lineage.
struct Node {
    id: String,
    parent: Option<String>,
    created_at: u64,
}

/// Every session below `root` in the TypeScript lineage trace's order: each
/// session is followed by its own descendants, siblings by creation time then
/// id. A session reached twice is kept once.
fn depth_first(root: &str, nodes: Vec<Node>) -> Vec<String> {
    let mut children: HashMap<String, Vec<Node>> = HashMap::new();
    for node in nodes {
        if let Some(parent) = node.parent.clone() {
            children.entry(parent).or_default().push(node);
        }
    }
    for siblings in children.values_mut() {
        siblings.sort_by(|a, b| a.created_at.cmp(&b.created_at).then_with(|| a.id.cmp(&b.id)));
    }
    let mut seen: HashSet<String> = HashSet::from([root.to_string()]);
    let mut ordered = Vec::new();
    // Each frame is the children of one parent still to visit, last first.
    let mut stack: Vec<Vec<String>> = Vec::new();
    let push_children = |parent: &str,
                         children: &mut HashMap<String, Vec<Node>>,
                         stack: &mut Vec<Vec<String>>| {
        if let Some(siblings) = children.remove(parent) {
            stack.push(siblings.into_iter().map(|n| n.id).rev().collect());
        }
    };
    push_children(root, &mut children, &mut stack);
    while let Some(frame) = stack.last_mut() {
        let Some(id) = frame.pop() else {
            stack.pop();
            continue;
        };
        if !seen.insert(id.clone()) {
            continue;
        }
        ordered.push(id.clone());
        push_children(&id, &mut children, &mut stack);
    }
    ordered
}

/// What the archive is built from.
struct Plan {
    persistence: Arc<dyn SessionPersistenceApi>,
    store: Option<Arc<AttachmentStore>>,
    root: String,
    descendants: Vec<String>,
    level: u8,
}

/// Build the archive. Blocking.
fn build(plan: &Plan) -> Result<Vec<u8>, Refusal> {
    let mut logs: Vec<(String, String)> = Vec::new();
    let mut attachments = Attachments::default();
    let read = |id: &str| plan.persistence.log_text(id).map_err(Refusal::log_failed);
    let root = read(&plan.root)?.ok_or_else(|| Refusal::new(404, "session not found"))?;
    attachments.note_log(&root).map_err(Refusal::read_failed)?;
    logs.push((LOG_FILENAME.to_string(), root));
    for id in &plan.descendants {
        let log = read(id)?.ok_or_else(|| {
            Refusal::read_failed(format!("subagent {id:?} has no stored log"))
        })?;
        attachments.note_log(&log).map_err(Refusal::read_failed)?;
        logs.push((
            format!("subagents/{}/{LOG_FILENAME}", safe_segment(id)),
            log,
        ));
    }

    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    // Level 0 asks for no compression. The `zip` crate refuses it for DEFLATE
    // (its range starts at 1), so the entries are stored instead.
    let options = if plan.level == 0 {
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)
    } else {
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(i64::from(plan.level)))
    }
    .large_file(true);
    let zip_failed = |error: &dyn std::fmt::Display| Refusal::read_failed(error);
    let mut add = |path: &str, data: &[u8]| -> Result<(), Refusal> {
        zip.start_file(path, options).map_err(|e| zip_failed(&e))?;
        zip.write_all(data).map_err(|e| zip_failed(&e))
    };
    for (path, log) in &logs {
        add(path, log.as_bytes())?;
    }
    let attachment_failed = |error: crate::attachments::AttachmentError| {
        Refusal::new(500, "session log export failed to read an attachment")
            .with_detail(format!("{}: {:?}", error.code, error.cause))
    };
    if !attachments.images.is_empty() || !attachments.files.is_empty() {
        let store = plan.store.as_ref().ok_or_else(|| {
            Refusal::new(
                500,
                "session log export is unavailable: the log references attachments but no attachment store is composed",
            )
        })?;
        for image in &attachments.images {
            let path = image_entry_path(image).ok_or_else(|| {
                Refusal::new(500, "session log export failed to read an attachment")
                    .with_detail(format!("unsupported image type {:?}", image.media_type))
            })?;
            add(&path, &store.read_image(image).map_err(attachment_failed)?)?;
        }
        for file in &attachments.files {
            add(
                &file_entry_path(file),
                &store.read_file(file).map_err(attachment_failed)?,
            )?;
        }
    }
    let cursor = zip.finish().map_err(|e| zip_failed(&e))?;
    Ok(cursor.into_inner())
}

/// Flush `id` when a live session has it, so the stored log holds its rows.
async fn flush_live(ctx: &Context, id: &str) -> Result<(), Refusal> {
    let Some(sessions) = ctx.get::<SessionService>(crate::session::store::SESSIONS_SERVICE) else {
        return Ok(());
    };
    if sessions.get(id).is_none() {
        return Ok(());
    }
    sessions
        .flush(id)
        .await
        .map_err(|reason| Refusal::read_failed(format!("flush of {id:?} failed: {reason}")))
}

/// Every known session's parent: the live sessions, then the stored ones a
/// live session does not cover.
fn lineage_nodes(ctx: &Context, persistence: &dyn SessionPersistenceApi) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut known: HashSet<String> = HashSet::new();
    if let Some(sessions) = ctx.get::<SessionService>(crate::session::store::SESSIONS_SERVICE) {
        for session in sessions.list() {
            known.insert(session.id().to_string());
            nodes.push(Node {
                id: session.id().to_string(),
                parent: session.parent_session(),
                created_at: session.header_created_at(),
            });
        }
    }
    for id in persistence.list() {
        if known.contains(&id) {
            continue;
        }
        if let Ok(Some(header)) = persistence.header(&id) {
            nodes.push(Node {
                id,
                parent: header.parent_session,
                created_at: header.created_at,
            });
        }
    }
    nodes
}

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, Refusal> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(Refusal::read_failed)
}

async fn export(
    router: &Router,
    ctx: &Context,
    request: &Request,
    head: bool,
) -> Result<Vec<u8>, Refusal> {
    let persistence = persistence(ctx).ok_or_else(|| {
        Refusal::new(
            500,
            "session log export is unavailable: missing session-persistence service",
        )
    })?;
    flush_live(ctx, &request.session_id).await?;
    let descendants = if request.descendants {
        let nodes = {
            let persistence = Arc::clone(&persistence);
            let ctx = ctx.clone();
            blocking(move || lineage_nodes(&ctx, persistence.as_ref())).await?
        };
        depth_first(&request.session_id, nodes)
    } else {
        Vec::new()
    };
    for id in &descendants {
        flush_live(ctx, id).await?;
    }
    let plan = Plan {
        persistence,
        store: ctx
            .get::<AttachmentStore>(crate::attachments::ATTACHMENTS_SERVICE)
            .map(|store| Arc::new(store.as_ref().clone())),
        root: request.session_id.clone(),
        descendants,
        level: router.export_compression_level(),
    };
    if head {
        // A HEAD request promises the status and headers a GET would send, so
        // it proves the root log exists but builds no archive.
        let plan = Plan { descendants: Vec::new(), ..plan };
        return blocking(move || {
            plan.persistence
                .log_text(&plan.root)
                .map_err(Refusal::log_failed)?
                .map(|_| Vec::new())
                .ok_or_else(|| Refusal::new(404, "session not found"))
        })
        .await?;
    }
    blocking(move || build(&plan)).await?
}

/// Answer one export request.
pub(super) async fn respond(
    router: &Router,
    ctx: &Context,
    method: &Method,
    query: Option<&str>,
) -> (HttpReply, HeaderMap) {
    let mut headers = HeaderMap::new();
    // The TypeScript route is registered for GET and HEAD alone, so any other
    // method reaches no handler and the gateway answers 404.
    if method != Method::GET && method != Method::HEAD {
        return (HttpReply::text(404, "not found", TEXT), headers);
    }
    let request = match parse_query(query) {
        Ok(request) => request,
        Err(refusal) => return refused(ctx, refusal, headers),
    };
    match export(router, ctx, &request, method == Method::HEAD).await {
        Ok(body) => {
            let disposition = format!(
                "attachment; filename=\"{}\"",
                archive_filename(&request.session_id)
            );
            if let Ok(value) = HeaderValue::from_str(&disposition) {
                headers.insert(CONTENT_DISPOSITION, value);
            }
            (HttpReply::text(200, body, "application/zip"), headers)
        }
        Err(refusal) => refused(ctx, refusal, headers),
    }
}

fn refused(ctx: &Context, refusal: Refusal, headers: HeaderMap) -> (HttpReply, HeaderMap) {
    if let Some(detail) = &refusal.detail {
        ctx.logger().log_event(
            cordis::logger::LogLevel::Error,
            "gui".to_string(),
            None,
            format!("session export: {}: {detail}", refusal.text),
        );
    }
    (HttpReply::text(refusal.status, refusal.text, TEXT), headers)
}
