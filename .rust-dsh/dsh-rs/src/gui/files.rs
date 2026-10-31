//! The workspace file endpoints: a session's files, read through the
//! filesystem.
//!
//! The TypeScript host serves these from its filesystem backend: a session's
//! workspace root comes from the session's header, a path is absolute or
//! relative to that root, and a directory listing stays inside the workspace.
//! A file read may leave the workspace, the way the TypeScript host allows.
//!
//! This build reads the host filesystem directly. Every refusal the
//! TypeScript host states has the same code here:
//! `workspace-file/not-found`, `workspace-file/not-regular-file`,
//! `workspace-file/not-directory`, `workspace-file/outside-workspace`,
//! `workspace-file/not-text` and `workspace-file/too-large`.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cordis::Context;
use serde_json::{json, Value};

use super::router::{argument, bad_request, input_invalid, Router};
use super::wire::{codes, Failure, RpcResult};
use crate::api::services::{AgentRegistryService, SessionPersistenceApi};

/// Directory basenames a file search never walks into.
const EXCLUDED_DIRECTORIES: &[&str] = &[
    ".git",
    "node_modules",
    "dist",
    "build",
    "out",
    "coverage",
    "target",
    ".next",
    ".nuxt",
    ".turbo",
    ".venv",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".gradle",
];

/// Serve one file endpoint.
///
/// Returns `None` when the method is not one of them, so the router can keep
/// looking.
pub(super) fn dispatch(
    router: &Router,
    ctx: &Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    match method {
        "workspaceFiles/list" => Some(list(router, ctx, payload)),
        "workspaceFiles/read" => Some(read(router, ctx, payload)),
        "workspaceFiles/readBytes" => Some(read_bytes(router, ctx, payload)),
        "workspaceFiles/stat" => Some(stat(router, ctx, payload)),
        "fileReferences/list" => Some(file_references(router, ctx, payload)),
        "directoryPicker/list" => Some(directory_list(router, ctx, payload)),
        "directoryPicker/createDirectory" => Some(create_directory(router, ctx, payload)),
        _ => None,
    }
}

/// The workspace a session's files live in.
struct Scope {
    /// The workspace root.
    root: PathBuf,
}

/// Resolve the workspace of the session a request names.
///
/// The scope parameter differs by endpoint: the file endpoints name the
/// session `workspaceFileScopeId`, the reference candidates name it `agentId`: the wire
/// name of a lookup parameter is its key plus `Id`.
fn scope(
    ctx: &Context,
    payload: &Value,
    parameter: &str,
    endpoint: &str,
) -> Result<Scope, Failure> {
    let Some(session_id) = argument(payload, parameter, endpoint).and_then(Value::as_str) else {
        return Err(input_invalid(
            endpoint,
            parameter,
            format!("{parameter} must be a string"),
        ));
    };
    let Some(root) = workspace_root(ctx, session_id) else {
        // The scope is a lookup the gateway resolves; an identity it cannot
        // resolve is the gateway's own refusal, with the wire field's name.
        return Err(Failure {
            code: codes::LOOKUP_NOT_FOUND.to_string(),
            message: format!(
                "typert gateway: {endpoint}: lookup provider \"workspaceFileScope\" did not resolve the requested identity"
            ),
            details: json!({ "endpoint": endpoint, "field": parameter }),
        });
    };
    Ok(Scope { root })
}

/// The workspace root of one session.
///
/// A live session knows its own working directory; a cold one keeps it in the
/// stored header.
fn workspace_root(ctx: &Context, session_id: &str) -> Option<PathBuf> {
    if let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) {
        if let Some(agent) = agents.get(session_id) {
            return agent.session().header_cwd().map(PathBuf::from);
        }
    }
    let persistence = ctx
        .get::<Arc<dyn SessionPersistenceApi>>(crate::api::services::SESSION_PERSISTENCE_SERVICE)?;
    persistence
        .header(session_id)
        .ok()
        .flatten()
        .and_then(|header| header.cwd)
        .map(PathBuf::from)
}

/// The path a request names, absolute or relative to the workspace root.
fn resolve(root: &Path, path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    }
}

/// Read one regular file's identity, version and size.
fn stat(_router: &Router, ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "workspaceFiles/stat";
    let scope = match scope(ctx, payload, "workspaceFileScopeId", ENDPOINT) {
        Ok(scope) => scope,
        Err(failure) => return RpcResult::failed(failure),
    };
    let Some(path) = argument(payload, "path", ENDPOINT).and_then(Value::as_str) else {
        return RpcResult::failed(input_invalid(ENDPOINT, "path", "path must be a string"));
    };
    if path.is_empty() {
        return RpcResult::failed(bad_request(ENDPOINT, "path is required"));
    }
    let target = resolve(&scope.root, path);
    match regular_file(&target, path) {
        Ok(info) => RpcResult::ok(stat_of(&info)),
        Err(failure) => RpcResult::failed(failure),
    }
}

/// Read one page of lines from a UTF-8 file.
fn read(router: &Router, ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "workspaceFiles/read";
    let scope = match scope(ctx, payload, "workspaceFileScopeId", ENDPOINT) {
        Ok(scope) => scope,
        Err(failure) => return RpcResult::failed(failure),
    };
    let Some(path) = argument(payload, "path", ENDPOINT).and_then(Value::as_str) else {
        return RpcResult::failed(input_invalid(ENDPOINT, "path", "path must be a string"));
    };
    if path.is_empty() {
        return RpcResult::failed(bad_request(ENDPOINT, "path is required"));
    }
    let (offset, limit) = match page_window(argument(payload, "range", ENDPOINT), router) {
        Ok(window) => window,
        Err(failure) => return RpcResult::failed(failure),
    };
    let target = resolve(&scope.root, path);
    let info = match regular_file(&target, path) {
        Ok(info) => info,
        Err(failure) => return RpcResult::failed(failure),
    };
    let page = match cut_page(&target, offset, limit, router.config().max_bytes, path) {
        Ok(page) => page,
        Err(failure) => return RpcResult::failed(failure),
    };
    if page.text.contains('\0') {
        return RpcResult::failed(not_text(path));
    }
    let mut answer = stat_of(&info);
    answer["offset"] = json!(offset);
    answer["text"] = json!(page.text);
    answer["lines"] = json!(page.lines);
    answer["eof"] = json!(page.eof);
    RpcResult::ok(answer)
}

/// Read one byte window, or a complete file under the file cap.
fn read_bytes(router: &Router, ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "workspaceFiles/readBytes";
    let scope = match scope(ctx, payload, "workspaceFileScopeId", ENDPOINT) {
        Ok(scope) => scope,
        Err(failure) => return RpcResult::failed(failure),
    };
    let Some(path) = argument(payload, "path", ENDPOINT).and_then(Value::as_str) else {
        return RpcResult::failed(input_invalid(ENDPOINT, "path", "path must be a string"));
    };
    if path.is_empty() {
        return RpcResult::failed(bad_request(ENDPOINT, "path is required"));
    }
    let options = argument(payload, "options", ENDPOINT);
    let base_file = options
        .and_then(|options| options.get("baseFile"))
        .and_then(Value::as_str);
    let target = match base_file {
        Some(base) => match relative_to(&scope.root, base, path) {
            Ok(target) => target,
            Err(failure) => return RpcResult::failed(failure),
        },
        None => resolve(&scope.root, path),
    };
    let info = match regular_file(&target, path) {
        Ok(info) => info,
        Err(failure) => return RpcResult::failed(failure),
    };
    let window = options.and_then(|options| options.get("range"));
    let (offset, data, eof) = match window {
        Some(window) => {
            let (offset, length) = match byte_window(window, router, path) {
                Ok(window) => window,
                Err(failure) => return RpcResult::failed(failure),
            };
            match read_range(&target, offset, length) {
                Ok(data) => {
                    let eof = info
                        .size
                        .map_or(data.len() < length, |size| offset + data.len() as u64 >= size);
                    (offset, data, eof)
                }
                Err(failure) => return RpcResult::failed(failure),
            }
        }
        None => {
            let limit = router.config().max_file_bytes;
            match read_complete(&target, limit, path) {
                Ok(data) => (0, data, true),
                Err(failure) => return RpcResult::failed(failure),
            }
        }
    };
    let mut answer = stat_of(&info);
    answer["offset"] = json!(offset);
    answer["data"] = json!(super::base64::encode(&data));
    answer["eof"] = json!(eof);
    RpcResult::ok(answer)
}

/// List the direct children of one directory inside the workspace.
fn list(router: &Router, ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "workspaceFiles/list";
    let scope = match scope(ctx, payload, "workspaceFileScopeId", ENDPOINT) {
        Ok(scope) => scope,
        Err(failure) => return RpcResult::failed(failure),
    };
    let Some(path) = argument(payload, "path", ENDPOINT).and_then(Value::as_str) else {
        return RpcResult::failed(input_invalid(ENDPOINT, "path", "path must be a string"));
    };
    if path.is_empty() {
        return RpcResult::failed(bad_request(ENDPOINT, "path is required"));
    }
    let root = fs::canonicalize(&scope.root).unwrap_or_else(|_| scope.root.clone());
    let original = resolve(&root, path);
    // Gate on the path itself before following it, the way the backend does.
    let is_symlink = fs::symlink_metadata(&original)
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false);
    match fs::metadata(&original) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(metadata) if metadata.is_file() => {
            return RpcResult::failed(not_directory(path, "file"));
        }
        Ok(_) => return RpcResult::failed(not_directory(path, "other")),
        Err(_cause) if is_symlink => {
            return RpcResult::failed(not_directory(path, "symlink"));
        }
        Err(cause) => {
            if cause.kind() == io::ErrorKind::NotFound {
                return RpcResult::failed(not_found(path));
            }
            return RpcResult::failed(io_failure(path, &cause));
        }
    }
    let target = fs::canonicalize(&original).unwrap_or_else(|_| original.clone());
    if !target.starts_with(&root) {
        return RpcResult::failed(outside_workspace(path));
    }
    let children = match children(&target) {
        Ok(children) => children,
        Err(failure) => return RpcResult::failed(failure),
    };
    let cap = router.config().max_entries;
    let truncated = children.len() > cap;
    let entries: Vec<Value> = children
        .iter()
        .take(cap)
        .map(|child| {
            let mut entry = json!({ "name": child.name, "type": child.kind });
            if let Some(size) = child.size {
                entry["size"] = json!(size);
            }
            entry
        })
        .collect();
    RpcResult::ok(json!({
        "path": workspace_path(&root, &target),
        "entries": entries,
        "truncated": truncated,
    }))
}

/// The file and directory candidates one query names.
fn file_references(router: &Router, ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "fileReferences/list";
    let scope = match scope(ctx, payload, "agentId", ENDPOINT) {
        Ok(scope) => scope,
        Err(failure) => return RpcResult::failed(failure),
    };
    let Some(query) = argument(payload, "query", ENDPOINT).and_then(Value::as_str) else {
        return RpcResult::failed(input_invalid(ENDPOINT, "query", "query must be a string"));
    };
    // A query that names a directory lists that directory's children; a bare
    // query ranks the whole workspace.
    let mut candidates = if query.contains('/') || query.is_empty() {
        let (directory, fragment) = match query.rfind('/') {
            Some(slash) => (&query[..=slash], &query[slash + 1..]),
            None => ("", query),
        };
        directory_candidates(&scope.root, directory, fragment)
    } else {
        let mut candidates = Vec::new();
        walk(&scope.root, &scope.root, &mut candidates);
        // A path that names a hidden segment is offered only when the query
        // names one too.
        candidates.retain(|candidate| {
            score(candidate, query).is_some()
                && (query.starts_with('.')
                    || query.contains("/.")
                    || !candidate.path.split('/').any(|segment| segment.starts_with('.')))
        });
        candidates
    };
    candidates.sort_by(|left, right| {
        score(right, query)
            .cmp(&score(left, query))
            .then_with(|| kind_rank(&left.kind).cmp(&kind_rank(&right.kind)))
            .then_with(|| left.path.len().cmp(&right.path.len()))
            .then_with(|| left.path.cmp(&right.path))
    });
    let cap = router.config().max_search_results;
    let items: Vec<Value> = candidates
        .iter()
        .take(cap)
        .map(|candidate| json!({ "path": candidate.path, "kind": candidate.kind }))
        .collect();
    RpcResult::ok(json!(items))
}

/// One level of the directory browser.
fn directory_list(router: &Router, _ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "directoryPicker/list";
    // An absent path lists the home directory.
    let home;
    let path = match payload.get("path") {
        None => match std::env::var("HOME").ok().filter(|home| !home.is_empty()) {
            Some(directory) => {
                home = directory;
                home.as_str()
            }
            None => {
                return RpcResult::failed(Failure::at(
                    super::wire::codes::INTERNAL,
                    "no home directory is known to list",
                    ENDPOINT,
                ))
            }
        },
        Some(Value::String(path)) => path.as_str(),
        Some(_) => return RpcResult::failed(input_invalid(ENDPOINT, "path", "path must be a string")),
    };
    let target = PathBuf::from(path);
    let listed = match fs::metadata(&target) {
        Ok(metadata) if metadata.is_dir() => target,
        Ok(_) => {
            return RpcResult::failed(Failure {
                code: "workspace-file/not-directory".to_string(),
                message: format!("\"{path}\" is not a directory"),
                details: json!({ "endpoint": ENDPOINT, "path": path }),
            })
        }
        Err(cause) => return RpcResult::failed(io_failure(path, &cause)),
    };
    let mut directories = Vec::new();
    if let Ok(read) = fs::read_dir(&listed) {
        for child in read.flatten() {
            let name = child.file_name().to_string_lossy().to_string();
            if child.path().is_dir() {
                directories.push((name, child.path()));
            }
        }
    }
    directories.sort_by(|left, right| left.0.cmp(&right.0));
    let cap = router.config().max_entries;
    let truncated = directories.len() > cap;
    let entries: Vec<Value> = directories
        .iter()
        .take(cap)
        .map(|(name, child)| {
            json!({
                "name": name,
                "path": child.to_string_lossy(),
                "hidden": is_hidden(name),
            })
        })
        .collect();
    let crumbs: Vec<Value> = ancestry(&listed)
        .into_iter()
        .map(|ancestor| {
            let name = ancestor
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| ancestor.to_string_lossy().to_string());
            json!({
                "name": name,
                "path": ancestor.to_string_lossy(),
                "hidden": false,
            })
        })
        .collect();
    RpcResult::ok(json!({
        "path": listed.to_string_lossy(),
        "home": home_directory(),
        "crumbs": crumbs,
        "entries": entries,
        "truncated": truncated,
    }))
}

/// Create one child directory of a listed directory.
fn create_directory(_router: &Router, _ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "directoryPicker/createDirectory";
    let Some(path) = argument(payload, "path", ENDPOINT).and_then(Value::as_str) else {
        return RpcResult::failed(input_invalid(ENDPOINT, "path", "path must be a string"));
    };
    let Some(name) = argument(payload, "name", ENDPOINT).and_then(Value::as_str) else {
        return RpcResult::failed(input_invalid(ENDPOINT, "name", "name must be a string"));
    };
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return RpcResult::failed(input_invalid(
            ENDPOINT,
            "name",
            "name must be one directory segment",
        ));
    }
    let created = Path::new(path).join(name);
    match fs::create_dir(&created) {
        Ok(()) => RpcResult::ok(json!(created.to_string_lossy())),
        Err(cause) => RpcResult::failed(io_failure(path, &cause)),
    }
}

/// One file's identity, version and size.
struct Info {
    /// The absolute path, symlinks resolved.
    absolute_path: String,
    /// The opaque freshness token.
    version: String,
    /// The byte size of a regular file.
    size: Option<u64>,
}

/// The stat a file endpoint answers.
fn stat_of(info: &Info) -> Value {
    let mut answer = json!({
        "absolutePath": info.absolute_path,
        "version": info.version,
    });
    if let Some(size) = info.size {
        answer["bytes"] = json!(size);
    }
    answer
}

/// Stat a path that must be a regular file.
///
/// `shown` is the path as the caller wrote it: the TypeScript host's refusals
/// name that path, not the resolved one.
fn regular_file(path: &Path, shown: &str) -> Result<Info, Failure> {
    // A symlink is its own kind, even when its target is a regular file:
    // the backend refuses to follow it for a read/stat.
    if let Ok(link) = fs::symlink_metadata(path) {
        if link.file_type().is_symlink() {
            return Err(Failure {
                code: "workspace-file/not-regular-file".to_string(),
                message: format!("\"{shown}\" is a symlink"),
                details: json!({ "path": shown, "kind": "symlink" }),
            });
        }
    }
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(cause) => return Err(io_failure(shown, &cause)),
    };
    if !metadata.is_file() {
        let kind = if metadata.is_dir() {
            "directory"
        } else {
            "other"
        };
        return Err(Failure {
            code: "workspace-file/not-regular-file".to_string(),
            message: format!("\"{}\" is a {kind}", path.to_string_lossy()),
            details: json!({ "path": path.to_string_lossy(), "kind": kind }),
        });
    }
    Ok(Info {
        absolute_path: absolute(path),
        version: version_of(&metadata),
        size: Some(metadata.len()),
    })
}

/// The absolute path of a target, symlinks resolved.
fn absolute(path: &Path) -> String {
    fs::canonicalize(path)
        .map(|resolved| resolved.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

/// The opaque freshness token of one file.
pub(super) fn version_of(metadata: &fs::Metadata) -> String {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|since| since.as_nanos())
        .unwrap_or(0);
    format!("{modified}-{}", metadata.len())
}

/// One child of a listed directory.
struct Child {
    name: String,
    kind: &'static str,
    size: Option<u64>,
}

/// The children of one directory, name-sorted.
fn children(directory: &Path) -> Result<Vec<Child>, Failure> {
    let read = match fs::read_dir(directory) {
        Ok(read) => read,
        Err(cause) => return Err(io_failure(&directory.to_string_lossy(), &cause)),
    };
    let mut children = Vec::new();
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        // A symlink reports the kind of its destination.
        let metadata = match fs::metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(_) => {
                // The destination cannot be read. The entry itself is still
                // listed: its own kind, with a link that resolves to nothing
                // reported as `other`.
                let kind = match fs::symlink_metadata(entry.path()) {
                    Ok(link) if link.is_dir() => "directory",
                    Ok(link) if link.is_file() => "file",
                    _ => "other",
                };
                children.push(Child {
                    name,
                    kind,
                    size: None,
                });
                continue;
            }
        };
        let (kind, size) = if metadata.is_file() {
            ("file", Some(metadata.len()))
        } else if metadata.is_dir() {
            ("directory", None)
        } else {
            ("other", None)
        };
        children.push(Child { name, kind, size });
    }
    children.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(children)
}

/// The path of a target inside the workspace, `/`-joined.
fn workspace_path(root: &Path, target: &Path) -> String {
    match target.strip_prefix(root) {
        Ok(relative) => relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join("/"),
        Err(_) => target.to_string_lossy().to_string(),
    }
}

/// The ancestor chain of a directory, from the filesystem root down.
fn ancestry(directory: &Path) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut current = Some(directory);
    while let Some(path) = current {
        chain.push(path.to_path_buf());
        current = path.parent();
    }
    chain.reverse();
    chain
}

/// The host account's home directory.
fn home_directory() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}

/// Whether a name is hidden by the host platform's convention.
fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// The page window a read asks for.
fn page_window(range: Option<&Value>, router: &Router) -> Result<(usize, usize), Failure> {
    let max_lines = router.config().max_lines;
    let offset = match range.and_then(|range| range.get("offset")) {
        Some(value) => integer_at_least(value, 1, "offset", "workspaceFiles/read")?,
        None => 1,
    };
    let limit = match range.and_then(|range| range.get("limit")) {
        Some(value) => integer_at_least(value, 1, "limit", "workspaceFiles/read")?,
        None => max_lines,
    };
    if limit > max_lines {
        return Err(bad_request(
            "workspaceFiles/read",
            format!("limit must be at most {max_lines}"),
        ));
    }
    Ok((offset, limit))
}

/// The byte window a read asks for.
fn byte_window(range: &Value, router: &Router, path: &str) -> Result<(u64, usize), Failure> {
    let max_bytes = router.config().max_bytes;
    let offset = match range.get("offset") {
        Some(value) => integer_at_least(value, 0, "offset", "workspaceFiles/readBytes")? as u64,
        None => 0,
    };
    let length = match range.get("length") {
        Some(value) => integer_at_least(value, 1, "length", "workspaceFiles/readBytes")?,
        None => max_bytes,
    };
    if length > max_bytes {
        return Err(Failure {
            code: "workspace-file/too-large".to_string(),
            message: format!("{length} bytes of \"{path}\" exceed the {max_bytes} byte cap"),
            details: json!({ "path": path, "limit": max_bytes }),
        });
    }
    let offset64 = offset as u64;
    if offset64 + length as u64 > crate::gui::router::MAX_SAFE_INTEGER {
        return Err(bad_request(
            "workspaceFiles/readBytes",
            "offset plus length must stay a safe integer",
        ));
    }
    Ok((offset, length))
}

/// A number the wire admits but a window cannot use.
fn integer_at_least(value: &Value, minimum: u64, name: &str, endpoint: &str) -> Result<usize, Failure> {
    let number = value.as_u64().filter(|number| *number >= minimum && *number <= crate::gui::router::MAX_SAFE_INTEGER);
    match number {
        Some(number) if number <= usize::MAX as u64 => Ok(number as usize),
        _ => Err(bad_request(
            endpoint,
            format!("{name} must be a safe integer of at least {minimum}"),
        )),
    }
}

/// One page of a file's lines.
struct Page {
    /// The page's lines, joined by newlines.
    text: String,
    /// How many lines the page holds.
    lines: usize,
    /// Whether the page reaches the file's last line.
    eof: bool,
}

/// Cut the lines `offset` through `offset + limit - 1` from a file.
///
/// The lines before the page are counted, not kept, and the page is refused
/// the moment its bytes pass the cap, so one giant line cannot grow memory
/// past it either.
fn cut_page(
    path: &Path,
    offset: usize,
    limit: usize,
    max_bytes: usize,
    shown: &str,
) -> Result<Page, Failure> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(cause) => return Err(io_failure(shown, &cause)),
    };
    let mut reader = BufReader::new(file);
    // The page's last line, computed without wrapping around.
    let Some(last) = offset.checked_add(limit).and_then(|sum| sum.checked_sub(1)) else {
        return Err(bad_request(
            "workspaceFiles/read",
            "offset plus limit must stay a safe integer",
        ));
    };
    let mut lines: Vec<String> = Vec::new();
    let mut bytes = 0usize;
    let mut number = 1usize;
    loop {
        if number > last {
            return Ok(Page {
                text: lines.join("\n"),
                lines: lines.len(),
                eof: false,
            });
        }
        let mut line = String::new();
        let read = match reader.read_line(&mut line) {
            Ok(read) => read,
            Err(cause) if cause.kind() == io::ErrorKind::InvalidData => {
                return Err(not_text(shown));
            }
            Err(cause) => return Err(io_failure(shown, &cause)),
        };
        if read == 0 {
            return Ok(Page {
                text: lines.join("\n"),
                lines: lines.len(),
                eof: true,
            });
        }
        let content = line.trim_end_matches('\n');
        if number >= offset {
            bytes += content.len();
            if bytes > max_bytes {
                return Err(page_too_large(shown, offset, last, max_bytes));
            }
            lines.push(content.to_string());
            // Every line but the first costs one more byte of the cap.
            if lines.len() > 1 {
                bytes += 1;
                if bytes > max_bytes {
                    return Err(page_too_large(shown, offset, last, max_bytes));
                }
            }
        }
        number += 1;
        if number > last {
            // The page ends here: the file reaches its last line only when
            // nothing follows it.
            let mut peek = String::new();
            let peeked = match reader.read_line(&mut peek) {
                Ok(peeked) => peeked,
                Err(cause) if cause.kind() == io::ErrorKind::InvalidData => {
                    return Err(not_text(shown));
                }
                Err(cause) => return Err(io_failure(shown, &cause)),
            };
            return Ok(Page {
                text: lines.join("\n"),
                lines: lines.len(),
                eof: peeked == 0,
            });
        }
    }
}

/// Read one byte window of a file.
fn read_range(path: &Path, offset: u64, length: usize) -> Result<Vec<u8>, Failure> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(cause) => return Err(io_failure(&path.to_string_lossy(), &cause)),
    };
    if let Err(cause) = file.seek(SeekFrom::Start(offset)) {
        return Err(io_failure(&path.to_string_lossy(), &cause));
    }
    let mut buffer = vec![0u8; length];
    let mut read = 0usize;
    while read < length {
        match file.read(&mut buffer[read..]) {
            Ok(0) => break,
            Ok(count) => read += count,
            Err(cause) => return Err(io_failure(&path.to_string_lossy(), &cause)),
        }
    }
    buffer.truncate(read);
    Ok(buffer)
}

/// Read a complete file under the file cap.
fn read_complete(path: &Path, limit: usize, shown: &str) -> Result<Vec<u8>, Failure> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(cause) => return Err(io_failure(shown, &cause)),
    };
    if metadata.len() > limit as u64 {
        return Err(Failure {
            code: "workspace-file/too-large".to_string(),
            message: format!("\"{shown}\" exceeds the {limit} byte full-file cap"),
            details: json!({ "path": shown, "limit": limit }),
        });
    }
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(cause) => return Err(io_failure(shown, &cause)),
    };
    // The file may have grown between the size check and the read, so the
    // cap is checked against what was actually read.
    if bytes.len() > limit {
        return Err(Failure {
            code: "workspace-file/too-large".to_string(),
            message: format!("\"{shown}\" exceeds the {limit} byte full-file cap"),
            details: json!({ "path": shown, "limit": limit }),
        });
    }
    Ok(bytes)
}

/// The path a base file resolves a relative path against.
fn relative_to(root: &Path, base_file: &str, path: &str) -> Result<PathBuf, Failure> {
    if path.is_empty() || Path::new(path).is_absolute() || path.contains('\0') || path.contains(':') {
        return Err(bad_request(
            "workspaceFiles/readBytes",
            "path must be relative when baseFile is provided",
        ));
    }
    let base = resolve(root, base_file);
    let directory = base
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.to_path_buf());
    Ok(directory.join(path))
}

/// One file or directory a query names.
struct Candidate {
    /// The path relative to the workspace root.
    path: String,
    /// Whether the candidate is a file or a directory.
    kind: &'static str,
}

/// The children of one directory, as candidates.
fn directory_candidates(root: &Path, directory: &str, fragment: &str) -> Vec<Candidate> {
    if directory
        .split('/')
        .any(|segment| EXCLUDED_DIRECTORIES.contains(&segment))
    {
        return Vec::new();
    }
    let listed = root.join(directory);
    // The directory is resolved before it is read, so a query naming a
    // parent of the workspace cannot read outside it.
    let resolved = fs::canonicalize(&listed).unwrap_or_else(|_| listed.clone());
    if !resolved.starts_with(root) {
        return Vec::new();
    }
    // The segments are walked without following a symlink: the TypeScript
    // host refuses one on the way to the directory it lists.
    let mut segments: Vec<&str> = Vec::new();
    for segment in directory.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }
    let mut listed = root.to_path_buf();
    for segment in segments {
        listed = listed.join(segment);
        let metadata = match fs::symlink_metadata(&listed) {
            Ok(metadata) => metadata,
            Err(_) => return Vec::new(),
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Vec::new();
        }
    }
    let read = match fs::read_dir(&listed) {
        Ok(read) => read,
        Err(_) => return Vec::new(),
    };
    let mut candidates = Vec::new();
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && !fragment.starts_with('.') {
            continue;
        }
        // A symlink is neither a file nor a directory here: the TypeScript
        // host reads the directory without following one.
        let metadata = match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let kind = if metadata.is_dir() {
            if EXCLUDED_DIRECTORIES.contains(&name.as_str()) {
                continue;
            }
            "directory"
        } else if metadata.is_file() {
            "file"
        } else {
            continue;
        };
        candidates.push(Candidate {
            path: format!("{directory}{name}"),
            kind,
        });
    }
    candidates
}

/// Collect the candidates one query names.
fn walk(root: &Path, directory: &Path, candidates: &mut Vec<Candidate>) {
    let read = match fs::read_dir(directory) {
        Ok(read) => read,
        Err(_) => return,
    };
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if EXCLUDED_DIRECTORIES.contains(&name.as_str()) {
            continue;
        }
        let path = entry.path();
        // A symlink is not walked into: the TypeScript host neither follows
        // one nor offers it, so a link back up the tree cannot loop the
        // traversal or expose a directory outside the workspace.
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let kind = if metadata.is_file() {
            "file"
        } else if metadata.is_dir() {
            "directory"
        } else {
            continue;
        };
        let relative = match path.strip_prefix(root) {
            Ok(relative) => relative.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        candidates.push(Candidate {
            path: relative,
            kind,
        });
        if kind == "directory" {
            walk(root, &path, candidates);
        }
    }
}

/// How well a candidate answers a query.
///
/// `None` means the candidate does not answer the query at all.
fn score(candidate: &Candidate, query: &str) -> Option<i64> {
    if query.is_empty() {
        return Some(0);
    }
    let path = candidate.path.to_lowercase();
    let name = path.rsplit('/').next().unwrap_or(&path).to_string();
    let needle = query.to_lowercase();
    let bonus = if candidate.kind == "directory" { 25 } else { 0 };
    if name == needle {
        return Some(1_000 + bonus);
    }
    if name.starts_with(&needle) {
        return Some(900 + bonus);
    }
    if name.contains(&needle) {
        return Some(700 + bonus);
    }
    if path.contains(&needle) {
        return Some(500 + bonus);
    }
    subsequence(&path, &needle).map(|score| 300 + score + bonus)
}

/// How well a path matches a query's characters in order.
fn subsequence(target: &str, query: &str) -> Option<i64> {
    let mut index = 0usize;
    let mut gap = 0i64;
    for character in query.chars() {
        let found = target[index..].find(character)? + index;
        gap += (found - index) as i64;
        index = found + character.len_utf8();
    }
    Some(100 - gap)
}

/// The order kinds share when their scores tie.
fn kind_rank(kind: &str) -> u8 {
    if kind == "directory" {
        0
    } else {
        1
    }
}

/// The TypeScript host's answer to a path that is not there.
fn not_found(path: &str) -> Failure {
    Failure {
        code: "workspace-file/not-found".to_string(),
        message: format!("no entry at \"{path}\""),
        details: json!({ "path": path }),
    }
}

/// The TypeScript host's answer to a path that is not a directory.
fn not_directory(path: &str, kind: &str) -> Failure {
    Failure {
        code: "workspace-file/not-directory".to_string(),
        message: format!("\"{path}\" is a {kind}"),
        details: json!({ "path": path, "kind": kind }),
    }
}

/// The TypeScript host's answer to a file that is not text.
fn not_text(path: &str) -> Failure {
    Failure {
        code: "workspace-file/not-text".to_string(),
        message: format!("\"{path}\" is not UTF-8 text"),
        details: json!({ "path": path }),
    }
}

/// The TypeScript host's answer to a page above the cap.
fn page_too_large(path: &str, offset: usize, last: usize, limit: usize) -> Failure {
    Failure {
        code: "workspace-file/too-large".to_string(),
        message: format!("lines {offset}-{last} of \"{path}\" exceed the {limit} byte cap"),
        details: json!({ "path": path, "limit": limit }),
    }
}

/// The TypeScript host's answer to a directory outside the workspace.
fn outside_workspace(path: &str) -> Failure {
    Failure {
        code: "workspace-file/outside-workspace".to_string(),
        message: format!("\"{path}\" is outside the workspace"),
        details: json!({ "path": path }),
    }
}

/// The answer to a filesystem refusal.
fn io_failure(path: &str, cause: &io::Error) -> Failure {
    if cause.kind() == io::ErrorKind::NotFound {
        return not_found(path);
    }
    Failure {
        code: "workspace-file/io-error".to_string(),
        message: format!("\"{path}\" cannot be read: {cause}"),
        details: json!({ "path": path }),
    }
}

