//! The workspace registry behind `workspace/*` and the `workspace/follow` stream.
//!
//! A workspace is a registered directory with a title. Which sessions belong to
//! it is not stored: a session belongs to the workspace whose canonical path is
//! its own working directory, as in the TypeScript registry. What is stored is
//! the registry order, each workspace's manual session order, and the
//! registry-global pin and archive sets. With a `file` they survive a restart;
//! without one they live as long as the process.
//!
//! Every mutation computes the next state, saves it, and only then swaps it in,
//! so a failed save leaves the registry as it was. Frames go out through a
//! broadcast channel under the same lock that orders the mutations, so a
//! follower sees them in commit order.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::broadcast;

use super::wire::{codes, Failure};

/// Frames a follower may fall behind before it is resynchronised with a
/// `replace` baseline instead of the missed increments.
const FRAME_BACKLOG: usize = 64;

/// A live session as the registry needs it: its id and working directory.
#[derive(Debug, Clone)]
pub struct SessionRef {
    /// Session id.
    pub id: String,
    /// The directory the session owns, when it has one.
    pub cwd: Option<String>,
    /// Whether a turn is running.
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
    id: String,
    path: String,
    title: String,
    created_at: String,
    updated_at: String,
    /// Manual session order. Sessions not listed follow in creation order.
    #[serde(default)]
    order: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct State {
    workspaces: Vec<Record>,
    #[serde(default)]
    archived: Vec<String>,
    #[serde(default)]
    pinned: Vec<String>,
    /// Session ids last reported per workspace, to emit only real changes.
    #[serde(skip)]
    reported: HashMap<String, Vec<String>>,
}

/// The registry. Cheap to share behind an `Arc`.
pub struct WorkspaceRegistry {
    state: Mutex<State>,
    frames: broadcast::Sender<Value>,
    file: Option<PathBuf>,
    default_directory: Option<PathBuf>,
}

impl std::fmt::Debug for WorkspaceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceRegistry")
            .field("file", &self.file)
            .finish_non_exhaustive()
    }
}

impl Default for WorkspaceRegistry {
    fn default() -> Self {
        Self::new(None, None).expect("an in-memory registry cannot fail to load")
    }
}

fn fail(code: &str, message: impl Into<String>, details: Value) -> Failure {
    Failure {
        code: code.to_string(),
        message: message.into(),
        details,
    }
}

fn not_found(workspace_id: &str) -> Failure {
    fail(
        "workspace/not-found",
        format!("no workspace {workspace_id:?}"),
        json!({ "workspaceId": workspace_id }),
    )
}

fn unknown_session(session_id: &str, action: &str) -> Failure {
    fail(
        "session/not-found",
        format!("cannot {action} session {session_id:?}: no such live session"),
        json!({ "sessionId": session_id }),
    )
}

impl WorkspaceRegistry {
    /// Open the registry. A `file` that exists must parse: a corrupt registry
    /// is reported, never silently replaced by an empty one.
    pub fn new(file: Option<PathBuf>, default_directory: Option<PathBuf>) -> Result<Self, String> {
        let state = match &file {
            Some(path) if path.exists() => {
                let text = std::fs::read_to_string(path)
                    .map_err(|e| format!("workspace registry {}: {e}", path.display()))?;
                serde_json::from_str(&text)
                    .map_err(|e| format!("workspace registry {}: {e}", path.display()))?
            }
            _ => State::default(),
        };
        let (frames, _) = broadcast::channel(FRAME_BACKLOG);
        Ok(WorkspaceRegistry {
            state: Mutex::new(state),
            frames,
            file,
            default_directory,
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn save(&self, next: &State) -> Result<(), Failure> {
        let Some(path) = &self.file else {
            return Ok(());
        };
        write_atomically(path, &serde_json::to_vec_pretty(next).expect("state serializes"))
            .map_err(|e| {
            fail(
                codes::INTERNAL,
                format!("could not save the workspace registry: {e}"),
                json!({ "path": path.display().to_string() }),
            )
        })
    }

    /// Apply `change` to a copy, save it, swap it in, then send `frames`.
    fn commit<T>(
        &self,
        change: impl FnOnce(&mut State) -> Result<(T, Vec<Frame>), Failure>,
        sessions: &[SessionRef],
    ) -> Result<T, Failure> {
        let mut guard = self.lock();
        let mut next = guard.clone();
        let (value, frames) = change(&mut next)?;
        self.save(&next)?;
        *guard = next;
        // What a commit sends already reflects the live sessions, so the poll
        // must not report the same state again.
        remember(&mut guard, sessions);
        for frame in frames {
            let json = frame.render(&guard, sessions);
            let _ = self.frames.send(json);
        }
        Ok(value)
    }

    /// The complete state, and a receiver for every later frame. Both are taken
    /// under one lock, so no frame falls between them.
    pub fn subscribe(&self, sessions: &[SessionRef]) -> (Value, broadcast::Receiver<Value>) {
        let mut guard = self.lock();
        remember(&mut guard, sessions);
        (snapshot(&guard, sessions), self.frames.subscribe())
    }

    /// A fresh complete state, for resynchronising a follower that lagged.
    pub fn baseline(&self, sessions: &[SessionRef]) -> Value {
        let mut guard = self.lock();
        remember(&mut guard, sessions);
        snapshot(&guard, sessions)
    }

    /// Emit an `upsert` for each workspace whose sessions changed since last reported.
    pub fn sessions_changed(&self, sessions: &[SessionRef]) {
        let mut guard = self.lock();
        let before = guard.reported.clone();
        remember(&mut guard, sessions);
        for record in &guard.workspaces {
            if before.get(&record.id) != guard.reported.get(&record.id) {
                let frame = json!({ "type": "upsert", "workspace": view(record, sessions) });
                let _ = self.frames.send(frame);
            }
        }
    }

    /// Register a directory, or return the workspace already over it.
    pub fn create(&self, path: &str, sessions: &[SessionRef]) -> Result<(Value, bool), Failure> {
        let canonical = canonical_directory(path)?;
        self.commit(
            |state| {
                if let Some(existing) = state.workspaces.iter().find(|w| w.path == canonical) {
                    return Ok(((view(existing, sessions), false), vec![]));
                }
                let record = new_record(state, &canonical, None);
                let id = record.id.clone();
                state.workspaces.push(record);
                let created = view(state.workspaces.last().expect("just pushed"), sessions);
                Ok(((created, true), vec![Frame::Upsert(id)]))
            },
            sessions,
        )
    }

    /// Register the configured first-use directory, creating it. `None` when
    /// no default directory is configured.
    pub fn initialize_default(&self, sessions: &[SessionRef]) -> Result<Option<Value>, Failure> {
        let Some(directory) = &self.default_directory else {
            return Ok(None);
        };
        std::fs::create_dir_all(directory).map_err(|e| {
            fail(
                "workspace/invalid-path",
                format!("cannot create the default workspace at {}: {e}", directory.display()),
                json!({ "path": directory.display().to_string() }),
            )
        })?;
        let (workspace, _) = self.create(&directory.display().to_string(), sessions)?;
        Ok(Some(workspace))
    }

    /// Rename a workspace to a unique, non-blank title.
    pub fn rename(&self, id: &str, title: &str, sessions: &[SessionRef]) -> Result<Value, Failure> {
        let title = title.trim();
        if title.is_empty() {
            return Err(fail(
                "gateway/bad-request",
                "Workspace rename requires a non-blank title",
                json!({}),
            ));
        }
        self.commit(
            |state| {
                let index = position(state, id)?;
                if state.workspaces[index].title != title {
                    if state.workspaces.iter().any(|w| w.id != id && w.title == title) {
                        return Err(fail(
                            "workspace/name-conflict",
                            format!("Workspace name '{title}' is already in use"),
                            json!({ "name": title }),
                        ));
                    }
                    state.workspaces[index].title = title.to_string();
                    state.workspaces[index].updated_at = now_iso();
                }
                Ok((
                    view(&state.workspaces[index], sessions),
                    vec![Frame::Upsert(id.to_string())],
                ))
            },
            sessions,
        )
    }

    /// Drop a registration. The directory and its sessions are untouched.
    pub fn delete(&self, id: &str, sessions: &[SessionRef]) -> Result<(), Failure> {
        self.commit(
            |state| {
                let index = position(state, id)?;
                state.workspaces.remove(index);
                state.reported.remove(id);
                Ok(((), vec![Frame::Remove(id.to_string())]))
            },
            sessions,
        )
    }

    /// Move a workspace before another, or to the end. Returns the full order.
    pub fn insert_before(
        &self,
        id: &str,
        before: Option<&str>,
        sessions: &[SessionRef],
    ) -> Result<Vec<String>, Failure> {
        self.commit(
            |state| {
                position(state, id)?;
                if let Some(anchor) = before {
                    position(state, anchor)?;
                }
                if before != Some(id) {
                    let at = state.workspaces.iter().position(|w| w.id == id).expect("checked");
                    let moved = state.workspaces.remove(at);
                    let to = before
                        .and_then(|anchor| state.workspaces.iter().position(|w| w.id == anchor))
                        .unwrap_or(state.workspaces.len());
                    state.workspaces.insert(to, moved);
                }
                let order: Vec<String> = state.workspaces.iter().map(|w| w.id.clone()).collect();
                Ok((order, vec![Frame::Order]))
            },
            sessions,
        )
    }

    /// Move a session within its workspace's manual order.
    pub fn insert_session_before(
        &self,
        workspace_id: &str,
        session_id: &str,
        before: Option<&str>,
        sessions: &[SessionRef],
    ) -> Result<Value, Failure> {
        self.commit(
            |state| {
                let index = position(state, workspace_id)?;
                let current = project(&state.workspaces[index], sessions);
                let invalid = |message: String| {
                    fail(
                        "workspace/move-invalid",
                        message,
                        json!({ "workspaceId": workspace_id, "sessionId": session_id }),
                    )
                };
                let path = state.workspaces[index].path.clone();
                if !current.iter().any(|s| s == session_id) {
                    return Err(invalid(format!(
                        "cannot move session '{session_id}' in workspace '{path}': the session is not accounted"
                    )));
                }
                if let Some(anchor) = before {
                    if !current.iter().any(|s| s == anchor) {
                        return Err(invalid(format!(
                            "cannot move session '{session_id}' before '{anchor}' in workspace '{path}': the anchor session is not accounted"
                        )));
                    }
                }
                let mut next: Vec<String> = current;
                if before != Some(session_id) {
                    next.retain(|s| s != session_id);
                    let to = before
                        .and_then(|anchor| next.iter().position(|s| s == anchor))
                        .unwrap_or(next.len());
                    next.insert(to, session_id.to_string());
                }
                state.workspaces[index].order = next;
                state.workspaces[index].updated_at = now_iso();
                Ok((
                    view(&state.workspaces[index], sessions),
                    vec![Frame::Upsert(workspace_id.to_string())],
                ))
            },
            sessions,
        )
    }

    /// Pin a live, unarchived session. Returns the full pin set, newest first.
    pub fn pin(&self, session_id: &str, sessions: &[SessionRef]) -> Result<Vec<String>, Failure> {
        self.commit(
            |state| {
                if state.pinned.iter().any(|s| s == session_id) {
                    return Ok((state.pinned.clone(), vec![]));
                }
                if !sessions.iter().any(|s| s.id == session_id) {
                    return Err(unknown_session(session_id, "pin"));
                }
                if state.archived.iter().any(|s| s == session_id) {
                    return Err(fail(
                        "gateway/bad-request",
                        format!("cannot pin archived session {session_id:?}"),
                        json!({}),
                    ));
                }
                state.pinned.insert(0, session_id.to_string());
                Ok((state.pinned.clone(), vec![Frame::Pinned]))
            },
            sessions,
        )
    }

    /// Unpin a session. Unpinning an unpinned session is not an error.
    pub fn unpin(&self, session_id: &str, sessions: &[SessionRef]) -> Result<Vec<String>, Failure> {
        self.commit(
            |state| {
                if !state.pinned.iter().any(|s| s == session_id) {
                    return Ok((state.pinned.clone(), vec![]));
                }
                state.pinned.retain(|s| s != session_id);
                Ok((state.pinned.clone(), vec![Frame::Pinned]))
            },
            sessions,
        )
    }

    /// Archive a live session. A running session, or one a provider reports
    /// active in `other_activity`, is refused unless `stop_activity`; the caller
    /// then stops it (`stopped` is returned) and clears the other activity.
    ///
    /// `other_activity` is the list of `{kind, items?}` entries that
    /// `workspace/session-activity` listeners reported, after the `turn` entry
    /// this registry adds itself.
    pub fn archive(
        &self,
        session_id: &str,
        stop_activity: bool,
        sessions: &[SessionRef],
        other_activity: &[Value],
    ) -> Result<(Vec<String>, bool), Failure> {
        self.commit(
            |state| {
                if state.archived.iter().any(|s| s == session_id) {
                    return Ok(((state.archived.clone(), false), vec![]));
                }
                let Some(session) = sessions.iter().find(|s| s.id == session_id) else {
                    return Err(unknown_session(session_id, "archive"));
                };
                if (session.running || !other_activity.is_empty()) && !stop_activity {
                    let mut activity = Vec::new();
                    if session.running {
                        activity.push(json!({ "kind": "turn" }));
                    }
                    activity.extend(other_activity.iter().cloned());
                    let kinds: Vec<&str> = activity.iter().filter_map(|entry| entry["kind"].as_str()).collect();
                    return Err(fail(
                        "workspace/session-active",
                        format!("cannot archive session '{session_id}': the session is active ({})", kinds.join(", ")),
                        json!({ "sessionId": session_id, "activity": activity }),
                    ));
                }
                state.archived.push(session_id.to_string());
                let was_pinned = state.pinned.iter().any(|s| s == session_id);
                state.pinned.retain(|s| s != session_id);
                let mut frames = vec![Frame::Archived];
                if was_pinned {
                    frames.push(Frame::Pinned);
                }
                Ok(((state.archived.clone(), session.running), frames))
            },
            sessions,
        )
    }

    /// Unarchive a session. Unarchiving a session that is not archived is not an error.
    pub fn unarchive(&self, session_id: &str, sessions: &[SessionRef]) -> Result<Vec<String>, Failure> {
        self.commit(
            |state| {
                if !state.archived.iter().any(|s| s == session_id) {
                    return Ok((state.archived.clone(), vec![]));
                }
                state.archived.retain(|s| s != session_id);
                Ok((state.archived.clone(), vec![Frame::Archived]))
            },
            sessions,
        )
    }

    /// The path a workspace owns, for creating a session in it.
    pub fn path_of(&self, id: &str) -> Result<String, Failure> {
        let guard = self.lock();
        guard
            .workspaces
            .iter()
            .find(|w| w.id == id)
            .map(|w| w.path.clone())
            .ok_or_else(|| not_found(id))
    }
}

/// Write `bytes` to `path` through a staged file, so a reader never sees half a file.
pub(super) fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_WRITE: AtomicU64 = AtomicU64::new(0);

    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let (staged, mut handle) = loop {
        let mut name = path.as_os_str().to_os_string();
        name.push(format!(".{}.{}.tmp", std::process::id(), NEXT_WRITE.fetch_add(1, Ordering::Relaxed)));
        let staged = PathBuf::from(name);
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&staged) {
            Ok(handle) => break (staged, handle),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    let result = (|| {
        handle.write_all(bytes)?;
        handle.sync_all()?;
        std::fs::rename(&staged, path)?;
        std::fs::File::open(parent)?.sync_all()
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

enum Frame {
    Upsert(String),
    Remove(String),
    Order,
    Pinned,
    Archived,
}

impl Frame {
    fn render(&self, state: &State, sessions: &[SessionRef]) -> Value {
        match self {
            Frame::Upsert(id) => match state.workspaces.iter().find(|w| &w.id == id) {
                Some(record) => json!({ "type": "upsert", "workspace": view(record, sessions) }),
                None => json!({ "type": "remove", "workspaceId": id }),
            },
            Frame::Remove(id) => json!({ "type": "remove", "workspaceId": id }),
            Frame::Order => json!({
                "type": "order",
                "workspaceIds": state.workspaces.iter().map(|w| w.id.clone()).collect::<Vec<_>>(),
            }),
            Frame::Pinned => json!({ "type": "pinned", "pinnedSessionIds": state.pinned }),
            Frame::Archived => json!({ "type": "archived", "archivedSessionIds": state.archived }),
        }
    }
}

fn position(state: &State, id: &str) -> Result<usize, Failure> {
    state
        .workspaces
        .iter()
        .position(|w| w.id == id)
        .ok_or_else(|| not_found(id))
}

fn snapshot(state: &State, sessions: &[SessionRef]) -> Value {
    json!({
        "items": state.workspaces.iter().map(|w| view(w, sessions)).collect::<Vec<_>>(),
        "archivedSessionIds": state.archived,
        "pinnedSessionIds": state.pinned,
    })
}

fn remember(state: &mut State, sessions: &[SessionRef]) {
    let projected: HashMap<String, Vec<String>> = state
        .workspaces
        .iter()
        .map(|w| (w.id.clone(), project(w, sessions)))
        .collect();
    state.reported = projected;
}

fn view(record: &Record, sessions: &[SessionRef]) -> Value {
    json!({
        "workspaceId": record.id,
        "path": record.path,
        "title": record.title,
        "sessionIds": project(record, sessions),
        "createdAt": record.created_at,
        "updatedAt": record.updated_at,
    })
}

/// The sessions whose directory is this workspace's, in manual order, then the
/// rest in the order the registry lists them.
fn project(record: &Record, sessions: &[SessionRef]) -> Vec<String> {
    let members: Vec<&str> = sessions
        .iter()
        .filter(|s| {
            s.cwd
                .as_deref()
                .is_some_and(|cwd| same_directory(cwd, &record.path))
        })
        .map(|s| s.id.as_str())
        .collect();
    let mut ordered: Vec<String> = record
        .order
        .iter()
        .filter(|id| members.contains(&id.as_str()))
        .cloned()
        .collect();
    for id in members {
        if !ordered.iter().any(|o| o == id) {
            ordered.push(id.to_string());
        }
    }
    ordered
}

fn same_directory(a: &str, b: &str) -> bool {
    let resolve = |p: &str| std::fs::canonicalize(p).unwrap_or_else(|_| PathBuf::from(p));
    resolve(a) == resolve(b)
}

fn canonical_directory(path: &str) -> Result<String, Failure> {
    let invalid = |reason: String| {
        fail(
            "workspace/invalid-path",
            format!("cannot create a Workspace at \"{path}\": {reason}"),
            json!({ "path": path }),
        )
    };
    if path.trim().is_empty() {
        return Err(invalid("the path is empty".to_string()));
    }
    let canonical = std::fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?;
    if !canonical.is_dir() {
        return Err(invalid("not a directory".to_string()));
    }
    Ok(canonical.display().to_string())
}

/// A record titled after the directory, suffixed when that title is taken.
fn new_record(state: &State, path: &str, title: Option<&str>) -> Record {
    let base = title
        .map(str::to_string)
        .or_else(|| {
            Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| path.to_string());
    let mut title = base.clone();
    let mut n = 2;
    while state.workspaces.iter().any(|w| w.title == title) {
        title = format!("{base} ({n})");
        n += 1;
    }
    let now = now_iso();
    Record {
        id: crate::plugins::unique_id(),
        path: path.to_string(),
        title,
        created_at: now.clone(),
        updated_at: now,
        order: vec![],
    }
}

/// Current UTC time as ISO-8601 with milliseconds (`2026-10-20T12:34:56.789Z`).
pub(crate) fn now_iso() -> String {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    iso_from_millis(elapsed.as_millis() as i64)
}

fn iso_from_millis(millis: i64) -> String {
    let days = millis.div_euclid(86_400_000);
    let rest = millis.rem_euclid(86_400_000);
    // Civil-from-days (Howard Hinnant's algorithm), valid for the whole i64 day range used here.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rest / 3_600_000,
        rest / 60_000 % 60,
        rest / 1_000 % 60,
        rest % 1_000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_matches_known_instants() {
        assert_eq!(iso_from_millis(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso_from_millis(951_782_400_000 + 3_723_004), "2000-02-29T01:02:03.004Z");
        assert_eq!(iso_from_millis(1_800_000_000_000), "2027-01-15T08:00:00.000Z");
        assert_eq!(iso_from_millis(-1), "1969-12-31T23:59:59.999Z");
    }
}
