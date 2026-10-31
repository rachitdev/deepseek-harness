//! File backend for TS-compatible session logs (format v4).
//!
//! Layout, as the TS `session-persistence-jsonl` backend writes it:
//! `<root>/--<project>--/<session>/session.v4.jsonl[.zstd]`, one header line
//! then one row per event. Compressed logs are concatenated zstd frames, one
//! per flush, so a crash tears at most the last frame.
//!
//! Appending to an existing log is allowed only when this runtime can account
//! for every row already on disk (same next `seq`, intact tail). Anything else
//! marks the session diverged: its events are dropped and the next
//! [`flush`](SessionPersistence::flush) returns the reason as an error.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::session::persistence::SessionPersistence;
use crate::session::v4::{decode_log, EncodeState, V4Header, V4Row};
use crate::types::{SessionEvent, SessionHeader};

/// Physical encoding of the log artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Zstd,
    None,
}

impl Compression {
    fn file_name(self) -> &'static str {
        match self {
            Compression::Zstd => "session.v4.jsonl.zstd",
            Compression::None => "session.v4.jsonl",
        }
    }
}

const ZSTD_LEVEL: i32 = 3;
const MAX_SEGMENT: usize = 251;

// ---------------------------------------------------------------------------
// Path encoding (mirrors `format.ts`: encodeSegment / projectKey)
// ---------------------------------------------------------------------------

fn is_safe_unit(unit: u16) -> bool {
    unit != u16::from(b'~')
        && u8::try_from(unit)
            .is_ok_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Escape one path segment: safe units stay, every other UTF-16 unit becomes `~XXXX`.
pub fn encode_segment(raw: &str) -> String {
    if raw == "." {
        return "~002E".to_string();
    }
    if raw == ".." {
        return "~002E~002E".to_string();
    }
    let mut out = String::new();
    for unit in raw.encode_utf16() {
        if is_safe_unit(unit) {
            out.push(char::from(unit as u8));
        } else {
            out.push_str(&format!("~{unit:04X}"));
        }
    }
    out
}

/// Inverse of [`encode_segment`]; `None` when the name is not a valid encoding.
pub fn decode_segment(encoded: &str) -> Option<String> {
    let mut units: Vec<u16> = Vec::new();
    let bytes = encoded.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'~' {
            let hex = encoded.get(at + 1..at + 5)?;
            units.push(u16::from_str_radix(hex, 16).ok()?);
            at += 5;
        } else {
            units.push(u16::from(bytes[at]));
            at += 1;
        }
    }
    String::from_utf16(&units).ok()
}

/// The readable project directory name for a working directory.
pub fn project_key(cwd: &str) -> String {
    let mut readable = String::new();
    let mut separator_run = false;
    for unit in cwd.encode_utf16() {
        let is_separator = matches!(char::from_u32(u32::from(unit)), Some('/' | '\\' | ':'));
        if is_separator {
            if !separator_run {
                readable.push('-');
            }
            separator_run = true;
        } else if is_safe_unit(unit) {
            readable.push(char::from(unit as u8));
            separator_run = false;
        } else {
            readable.push_str(&format!("~{unit:04X}"));
            separator_run = false;
        }
    }
    let trimmed = readable.trim_start_matches('-');
    let slug = if trimmed.is_empty() { "root" } else { trimmed };
    let cut: String = slug.chars().take(MAX_SEGMENT).collect();
    format!("--{cut}--")
}

/// `<root>/<project>/<session>` for a session.
pub fn session_dir(root: &Path, cwd: Option<&str>, id: &str) -> PathBuf {
    let project = match cwd {
        Some(cwd) => project_key(cwd),
        None => "_no-cwd".to_string(),
    };
    root.join(project).join(encode_segment(id))
}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

/// A log read from disk.
#[derive(Debug)]
pub struct ReadLog {
    pub header: V4Header,
    pub rows: Vec<V4Row>,
    /// Bytes of a torn final frame or line that were discarded.
    pub torn_tail_bytes: usize,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// Decompress concatenated frames; a final frame that cannot be sized is a
/// torn write and is dropped, while a sized frame that fails to decode is corruption.
fn decompress_frames(bytes: &[u8]) -> io::Result<(Vec<u8>, usize)> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let rest = &bytes[at..];
        let Ok(size) = zstd::zstd_safe::find_frame_compressed_size(rest) else {
            return Ok((out, rest.len()));
        };
        let frame = &rest[..size];
        let mut decoder = zstd::stream::read::Decoder::new(frame)?;
        decoder
            .read_to_end(&mut out)
            .map_err(|err| invalid(format!("corrupt zstd frame at byte {at}: {err}")))?;
        at += size;
    }
    Ok((out, 0))
}

/// The whole lines of a log's bytes (already located by suffix), and the bytes
/// of a torn tail that were dropped. An unterminated final line is kept only
/// when it is a whole JSON value.
fn log_lines(bytes: &[u8], compressed: bool) -> io::Result<(Vec<String>, usize)> {
    let (text, mut torn) = if compressed {
        decompress_frames(bytes)?
    } else {
        (bytes.to_vec(), 0)
    };
    let text = String::from_utf8(text).map_err(|_| invalid("log is not valid UTF-8"))?;
    let mut lines: Vec<&str> = text.split('\n').collect();
    // The text ends with "\n" when complete, leaving an empty last element.
    let last = lines.pop().unwrap_or("");
    if !last.is_empty() {
        if serde_json::from_str::<Value>(last).is_ok() {
            lines.push(last);
        } else {
            torn += last.len();
        }
    }
    let lines = lines
        .into_iter()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    Ok((lines, torn))
}

fn parse_log(lines: &[String], torn_tail_bytes: usize) -> io::Result<ReadLog> {
    let mut iter = lines.iter();
    let first = iter
        .next()
        .ok_or_else(|| invalid("log has no header line"))?;
    let header = V4Header::parse(first).map_err(|err| invalid(err.to_string()))?;
    let mut rows = Vec::new();
    for (n, line) in iter.enumerate() {
        let row: V4Row = serde_json::from_str(line)
            .map_err(|err| invalid(format!("row {} is not a v4 row: {err}", n + 1)))?;
        rows.push(row);
    }
    Ok(ReadLog {
        header,
        rows,
        torn_tail_bytes,
    })
}

/// Parse a log's bytes (already located by suffix).
pub fn read_log_bytes(bytes: &[u8], compressed: bool) -> io::Result<ReadLog> {
    let (lines, torn) = log_lines(bytes, compressed)?;
    parse_log(&lines, torn)
}

/// A stored log as uncompressed JSONL, one header line then one line per row,
/// each as written. It is checked to parse as a v4 log first.
pub fn read_log_text(path: &Path) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    let compressed = path.extension().is_some_and(|ext| ext == "zstd");
    let (lines, torn) = log_lines(&bytes, compressed)?;
    parse_log(&lines, torn)?;
    Ok(lines.join("\n") + "\n")
}

/// Find a session's log file under `dir`, preferring the compressed form.
fn find_artifact(dir: &Path) -> Option<(PathBuf, bool)> {
    for compression in [Compression::Zstd, Compression::None] {
        let path = dir.join(compression.file_name());
        if path.is_file() {
            return Some((path, compression == Compression::Zstd));
        }
    }
    None
}

/// Read a log file (compressed or not, by its name).
pub fn read_log_file(path: &Path) -> io::Result<ReadLog> {
    let bytes = std::fs::read(path)?;
    let compressed = path.extension().is_some_and(|ext| ext == "zstd");
    read_log_bytes(&bytes, compressed)
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

#[cfg(unix)]
fn try_lock(file: &File) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    // SAFETY: `flock` on a descriptor we own for the life of `file`.
    let status = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if status == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(unix))]
fn try_lock(_file: &File) -> io::Result<()> {
    Ok(())
}

struct Log {
    dir: PathBuf,
    header: V4Header,
    lines: Vec<String>,
    encode: EncodeState,
    next_seq: u64,
    on_disk: bool,
    lease: Option<File>,
    diverged: Option<String>,
}

impl Log {
    fn diverge(&mut self, reason: String) {
        eprintln!("[session-v4] {}: log diverged: {reason}", self.header.id);
        self.diverged.get_or_insert(reason);
    }

    fn push(&mut self, event: &SessionEvent) {
        if self.diverged.is_some() {
            return;
        }
        if event.seq != self.next_seq {
            self.diverge(format!(
                "event seq {} where {} was expected",
                event.seq, self.next_seq
            ));
            return;
        }
        let line = crate::session::v4::encode_event(event, &mut self.encode).and_then(|row| {
            serde_json::to_string(&row).map_err(|e| crate::session::v4::V4Error::Log(e.to_string()))
        });
        match line {
            Ok(line) => {
                self.lines.push(line);
                self.next_seq += 1;
            }
            Err(err) => self.diverge(err.to_string()),
        }
    }
}

/// TS-compatible session persistence.
pub struct V4Persistence {
    root: PathBuf,
    compression: Compression,
    logs: Mutex<HashMap<String, Arc<Mutex<Log>>>>,
}

impl V4Persistence {
    pub fn new(root: impl Into<PathBuf>, compression: Compression) -> Self {
        V4Persistence {
            root: root.into(),
            compression,
            logs: Mutex::new(HashMap::new()),
        }
    }

    fn log(&self, session: &str) -> Option<Arc<Mutex<Log>>> {
        self.logs.lock().unwrap().get(session).cloned()
    }

    /// Locate a stored session by id when it has not been opened this run.
    fn locate(&self, session: &str) -> Option<PathBuf> {
        let encoded = encode_segment(session);
        let projects = std::fs::read_dir(&self.root).ok()?;
        for project in projects.flatten() {
            let dir = project.path().join(&encoded);
            if find_artifact(&dir).is_some() {
                return Some(dir);
            }
        }
        None
    }

    fn open_new(&self, dir: PathBuf, header: V4Header, seed: &[SessionEvent]) -> Log {
        let mut log = Log {
            dir,
            header: header.clone(),
            lines: Vec::new(),
            encode: EncodeState::new(header.is_seeded),
            next_seq: 0,
            on_disk: false,
            lease: None,
            diverged: None,
        };
        for event in seed {
            log.push(event);
        }
        log
    }

    fn open_existing(
        &self,
        dir: PathBuf,
        path: &Path,
        seed: &[SessionEvent],
        header: V4Header,
    ) -> Log {
        let mut log = Log {
            dir,
            encode: EncodeState::new(header.is_seeded),
            header,
            lines: Vec::new(),
            next_seq: 0,
            on_disk: true,
            lease: None,
            diverged: None,
        };
        let outcome = read_log_file(path)
            .map_err(|e| e.to_string())
            .and_then(|stored| {
                if stored.torn_tail_bytes > 0 {
                    return Err(format!(
                        "existing log has a torn tail ({} bytes)",
                        stored.torn_tail_bytes
                    ));
                }
                let decoded = decode_log(&stored.rows).map_err(|e| e.to_string())?;
                let seed_next = seed.iter().map(|e| e.seq + 1).max().unwrap_or(0);
                if decoded.next_seq != seed_next {
                    return Err(format!(
                        "stored log ends at seq {} but the session resumed at seq {seed_next}",
                        decoded.next_seq
                    ));
                }
                Ok((stored, decoded.next_seq))
            });
        match outcome {
            Ok((stored, next_seq)) => {
                // The header already on disk is authoritative for `isSeeded`.
                log.encode = EncodeState::new(stored.header.is_seeded);
                log.header = stored.header.clone();
                log.next_seq = next_seq;
                for row in &stored.rows {
                    if row.kind == "tool/call" {
                        if let Some(call_id) = row.data.get("callId").and_then(Value::as_str) {
                            log.encode.remember_call(call_id, row.seq);
                        }
                    }
                    if row.kind == "session/end-seed"
                        && row.data.get("inherited") == Some(&Value::Bool(true))
                    {
                        log.encode.note_inherited_marker();
                    }
                    if row.kind == "request/header" {
                        log.encode.note_header_written();
                    }
                }
            }
            Err(reason) => log.diverge(reason),
        }
        log
    }
}

impl SessionPersistence for V4Persistence {
    fn on_session_open(&self, session: &String, header: &SessionHeader, seed: &[SessionEvent]) {
        let dir = session_dir(&self.root, header.cwd.as_deref(), session);
        let v4 = V4Header::with_preset(
            session,
            header.created_at,
            header.cwd.clone(),
            header.parent_session.clone(),
            !seed.is_empty(),
            header.agent_preset.clone(),
        );
        let log = match find_artifact(&dir) {
            Some((path, _)) => self.open_existing(dir, &path, seed, v4),
            None => self.open_new(dir, v4, seed),
        };
        self.logs
            .lock()
            .unwrap()
            .insert(session.clone(), Arc::new(Mutex::new(log)));
    }

    fn on_event(&self, session: &String, event: &SessionEvent) {
        if let Some(log) = self.log(session) {
            log.lock().unwrap().push(event);
        }
    }

    fn flush(&self, session: &String) -> io::Result<()> {
        let Some(log) = self.log(session) else {
            return Ok(());
        };
        let mut log = log.lock().unwrap();
        if let Some(reason) = &log.diverged {
            return Err(io::Error::other(format!(
                "session log {session} diverged: {reason}"
            )));
        }
        if log.lines.is_empty() && log.on_disk {
            return Ok(());
        }
        if !log.on_disk && log.header.is_seeded && !log.encode.inherited_marker_written() {
            return Err(io::Error::other(format!(
                "seeded session log {session} has no end-seed marker yet; TS would reject it"
            )));
        }
        std::fs::create_dir_all(&log.dir)?;
        if log.lease.is_none() {
            let lock = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log.dir.join("session.lock"))?;
            try_lock(&lock).map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!("session {session} is owned by another process: {err}"),
                )
            })?;
            log.lease = Some(lock);
        }
        let mut body = String::new();
        if !log.on_disk {
            body.push_str(&serde_json::to_string(&log.header).map_err(io::Error::other)?);
            body.push('\n');
        }
        for line in &log.lines {
            body.push_str(line);
            body.push('\n');
        }
        let payload = match self.compression {
            Compression::Zstd => zstd::encode_all(body.as_bytes(), ZSTD_LEVEL)?,
            Compression::None => body.into_bytes(),
        };
        let path = log.dir.join(self.compression.file_name());
        let mut file = if log.on_disk {
            OpenOptions::new().append(true).open(&path)?
        } else {
            OpenOptions::new()
                .create_new(true)
                .append(true)
                .open(&path)?
        };
        file.write_all(&payload)?;
        file.sync_data()?;
        log.on_disk = true;
        log.lines.clear();
        Ok(())
    }

    fn load(&self, session: &String) -> io::Result<Vec<SessionEvent>> {
        let dir = match self.log(session) {
            Some(log) => log.lock().unwrap().dir.clone(),
            None => match self.locate(session) {
                Some(dir) => dir,
                None => return Ok(Vec::new()),
            },
        };
        let Some((path, _)) = find_artifact(&dir) else {
            return Ok(Vec::new());
        };
        let stored = read_log_file(&path)?;
        let decoded = decode_log(&stored.rows).map_err(|err| invalid(err.to_string()))?;
        Ok(decoded.events)
    }

    fn list(&self) -> Vec<String> {
        let mut ids = Vec::new();
        let Ok(projects) = std::fs::read_dir(&self.root) else {
            return ids;
        };
        for project in projects.flatten() {
            let Ok(sessions) = std::fs::read_dir(project.path()) else {
                continue;
            };
            for session in sessions.flatten() {
                if find_artifact(&session.path()).is_some() {
                    if let Some(id) = session.file_name().to_str().and_then(decode_segment) {
                        ids.push(id);
                    }
                }
            }
        }
        ids.sort();
        ids
    }

    fn header(&self, session: &String) -> io::Result<Option<crate::types::SessionHeader>> {
        let Some(dir) = self.locate(session) else {
            return Ok(None);
        };
        let Some((path, _)) = find_artifact(&dir) else {
            return Ok(None);
        };
        let stored = read_log_file(&path)?;
        Ok(Some(crate::types::SessionHeader {
            cwd: stored.header.cwd.clone(),
            created_at: stored.header.created_at,
            parent_session: stored.header.parent_session.clone(),
            seed_length: 0,
            agent_preset: stored.header.agent_preset.clone(),
        }))
    }
}

impl V4Persistence {
    /// The stored log of `session` as JSONL, or `None` when no log is stored.
    /// Rows still buffered in a live session are not included: flush first.
    pub fn log_text(&self, session: &str) -> io::Result<Option<String>> {
        let dir = match self.log(session) {
            Some(log) => log.lock().unwrap().dir.clone(),
            None => match self.locate(session) {
                Some(dir) => dir,
                None => return Ok(None),
            },
        };
        match find_artifact(&dir) {
            Some((path, _)) => read_log_text(&path).map(Some),
            None => Ok(None),
        }
    }
}

impl crate::api::services::SessionPersistenceApi for V4Persistence {
    fn on_session_open(&self, session: &str, header: &SessionHeader, seed: &[SessionEvent]) {
        SessionPersistence::on_session_open(self, &session.to_string(), header, seed)
    }

    fn on_event(&self, session: &str, event: &SessionEvent) {
        SessionPersistence::on_event(self, &session.to_string(), event)
    }

    fn flush(&self, session: &str) -> io::Result<()> {
        SessionPersistence::flush(self, &session.to_string())
    }

    fn load(&self, session: &str) -> io::Result<Vec<SessionEvent>> {
        SessionPersistence::load(self, &session.to_string())
    }

    fn list(&self) -> Vec<String> {
        SessionPersistence::list(self)
    }

    fn header(&self, session: &str) -> io::Result<Option<crate::types::SessionHeader>> {
        SessionPersistence::header(self, &session.to_string())
    }

    fn log_text(&self, session: &str) -> io::Result<Option<String>> {
        V4Persistence::log_text(self, session)
    }
}

/// The TS-compatible persistence plugin: attaches a [`V4Persistence`] to the
/// session store and provides it as `sessionPersistence`. Requires `sessions`.
pub fn v4_persistence_plugin(
    root: PathBuf,
    compression: Compression,
) -> Arc<dyn cordis::plugin::Plugin> {
    cordis::plugin::plugin_with(
        "session-persistence",
        vec![cordis::plugin::Injection::new(
            crate::session::store::SESSIONS_SERVICE.to_string(),
        )],
        move |ctx, _config: Value| {
            let root = root.clone();
            async move {
                let store = ctx
                    .get::<crate::api::services::SessionService>(
                        crate::session::store::SESSIONS_SERVICE,
                    )
                    .ok_or_else(|| cordis::Error::msg("sessions service missing"))?;
                let backend = Arc::new(V4Persistence::new(root, compression))
                    as Arc<dyn crate::api::services::SessionPersistenceApi>;
                store.attach_persistence(backend.clone());
                ctx.provide(crate::api::services::SESSION_PERSISTENCE_SERVICE, backend)
                    .await?;
                Ok(())
            }
        },
    )
}
