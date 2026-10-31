//! The settings store behind `settings/*`.
//!
//! The TypeScript host exposes every volatile plugin field as a namespace of
//! the settings service. This build serves the namespaces it has fields for:
//!
//! - `ui-settings-general`, whose single field `welcomeNoticeVersion` records
//!   which welcome notice the person acknowledged. The browser writes it when
//!   "Continue" is pressed on the first-run notice, so without a writable store
//!   that notice can never be dismissed.
//! - `speech`, whose `providerId` and `language` are the reader's speech
//!   preferences.
//! - `plugin-market`, whose `indexes` field lists plugin index URLs.
//!
//! Writes are checked against the revision the caller read, so a stale editor
//! is refused (`settings/conflict`) instead of overwriting a newer value. Each
//! write is saved before it is applied, so a failed save changes nothing.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use super::wire::Failure;

/// The namespace of the browser's own settings.
pub const NAMESPACE: &str = "ui-settings-general";

/// The namespace of the speech preferences.
pub const SPEECH_NAMESPACE: &str = "speech";

/// The namespace of the plugin market's index list.
pub const MARKET_NAMESPACE: &str = "plugin-market";

/// The field of [`NAMESPACE`].
const FIELD: &str = "welcomeNoticeVersion";

/// The fields of the speech namespace.
const SPEECH_PROVIDER: &str = "providerId";
const SPEECH_LANGUAGE: &str = "language";

/// The field of the market namespace.
const MARKET_INDEXES: &str = "indexes";

/// `schema.toJSON()` of `z.object({ welcomeNoticeVersion: z.string().volatile() })`,
/// produced by the TypeScript `@deepseek-ai/schemastery` the client rehydrates it with.
const SCHEMA: &str = r#"{"uid":2,"refs":{"1":{"type":"string","meta":{"volatile":true}},"2":{"type":"object","meta":{"default":{}},"dict":{"welcomeNoticeVersion":1}}}}"#;

/// The same schema for the speech namespace's two string fields.
const SPEECH_SCHEMA: &str = r#"{"uid":2,"refs":{"1":{"type":"string","meta":{"volatile":true}},"2":{"type":"object","meta":{"default":{}},"dict":{"providerId":1,"language":1}}}}"#;

/// The schema for the market namespace's list of index urls.
const MARKET_SCHEMA: &str = r#"{"uid":2,"refs":{"1":{"type":"array","meta":{"volatile":true}},"2":{"type":"object","meta":{"default":{}},"dict":{"indexes":1}}}}"#;

/// One namespace: its fields, its schema and its current value.
#[derive(Debug)]
struct Namespace {
    /// The fields it serves, and what each holds.
    fields: &'static [(&'static str, FieldKind)],
    /// The schemastery schema the client rehydrates.
    schema: &'static str,
    /// The user section.
    user: Map<String, Value>,
    /// The revision the caller reads.
    revision: u64,
}

/// What a namespace's field holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    /// A string.
    Text,
    /// A list of strings.
    TextList,
}

impl Namespace {
    fn new(fields: &'static [(&'static str, FieldKind)], schema: &'static str) -> Self {
        Namespace {
            fields,
            schema,
            user: Map::new(),
            revision: 0,
        }
    }

    fn kind(&self, field: &str) -> Option<FieldKind> {
        self.fields
            .iter()
            .find(|(name, _)| *name == field)
            .map(|(_, kind)| *kind)
    }

    fn view(&self, ns: &str) -> Value {
        json!({
            "autoGenerate": false,
            "ns": ns,
            "schema": serde_json::from_str::<Value>(self.schema).expect("the schema constant is JSON"),
            "value": self.user,
            "base": {},
            "user": self.user,
            "applies": "live",
            "secrets": [],
            "revision": self.revision,
        })
    }
}

/// The namespaces this store serves, in the order the client reads them.
fn namespaces() -> Vec<(String, Namespace)> {
    vec![
        (
            NAMESPACE.to_string(),
            Namespace::new(&[(FIELD, FieldKind::Text)], SCHEMA),
        ),
        (
            SPEECH_NAMESPACE.to_string(),
            Namespace::new(
                &[
                    (SPEECH_PROVIDER, FieldKind::Text),
                    (SPEECH_LANGUAGE, FieldKind::Text),
                ],
                SPEECH_SCHEMA,
            ),
        ),
        (
            MARKET_NAMESPACE.to_string(),
            Namespace::new(&[(MARKET_INDEXES, FieldKind::TextList)], MARKET_SCHEMA),
        ),
    ]
}

#[derive(Debug)]
struct State {
    /// The namespaces, in the order the client reads them.
    namespaces: Vec<(String, Namespace)>,
}

/// Settings persisted to a JSON file, or held in memory without one.
#[derive(Debug)]
pub struct SettingsStore {
    state: Mutex<State>,
    file: Option<PathBuf>,
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::new(None).expect("an in-memory store cannot fail to load")
    }
}

fn rejected(ns: &str, message: impl Into<String>) -> Failure {
    Failure {
        code: "settings/rejected".to_string(),
        message: message.into(),
        details: json!({ "ns": ns }),
    }
}

/// Load a fresh snapshot, refusing malformed content rather than repairing it.
fn load(file: Option<&Path>) -> Result<State, String> {
    let mut state = State {
        namespaces: namespaces(),
    };
    if let Some(path) = file {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(state),
            Err(error) => return Err(format!("settings {}: {error}", path.display())),
        };
        let stored: Value =
            serde_json::from_str(&text).map_err(|e| format!("settings {}: {e}", path.display()))?;
        let shown = path.display();
        let Value::Object(root) = stored else {
            return Err(format!("settings {shown}: the document must be an object"));
        };
        if let Some(revision) = root.get("revision") {
            // The legacy document's one revision beside the general namespace.
            if revision.as_u64().is_none() {
                return Err(format!(
                    "settings {shown}: revision must be a non-negative integer"
                ));
            }
        }
        for (ns, raw) in &root {
            if ns == "revision" {
                continue;
            }
            let Some((_, entry)) = state.namespaces.iter_mut().find(|(name, _)| name == ns) else {
                return Err(format!("settings {shown}: unknown namespace {ns:?}"));
            };
            // The document holds each namespace as `{ "user": { … }, "revision": n }`.
            // The first version of this store held the general namespace's fields
            // flat, with one revision beside them; that shape is still read.
            let (user, revision) = match raw {
                Value::Object(fields) if fields.contains_key("user") => {
                    let revision = fields.get("revision").and_then(Value::as_u64).ok_or_else(
                        || {
                            format!(
                                "settings {shown}: {ns}.revision must be a non-negative integer"
                            )
                        },
                    )?;
                    let Value::Object(user) =
                        fields.get("user").cloned().unwrap_or_else(|| json!({}))
                    else {
                        return Err(format!("settings {shown}: {ns}.user must be an object"));
                    };
                    (user, revision)
                }
                Value::Object(fields) => {
                    let revision = root.get("revision").and_then(Value::as_u64).unwrap_or(0);
                    (fields.clone(), revision)
                }
                _ => return Err(format!("settings {shown}: {ns} must be an object")),
            };
            for (field, value) in &user {
                let Some(kind) = entry.kind(field) else {
                    return Err(format!("settings {shown}: invalid {ns}.{field}"));
                };
                if !value_matches(kind, value) {
                    return Err(format!("settings {shown}: invalid {ns}.{field}"));
                }
            }
            entry.user = user;
            entry.revision = revision;
        }
    }
    Ok(state)
}

/// Exclusive sidecar lock, shared by independent stores and processes.
struct FileLock(PathBuf);

impl FileLock {
    fn acquire(file: &Path) -> std::io::Result<Self> {
        use std::io::Write;
        let parent = file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        std::fs::create_dir_all(parent)?;
        let mut name = file.as_os_str().to_os_string();
        name.push(".lock");
        let path = PathBuf::from(name);
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut delay = Duration::from_millis(20);
        loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut handle) => {
                    let lock = Self(path);
                    writeln!(handle, "{}", std::process::id())?;
                    return Ok(lock);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "timed out waiting for settings writer lock",
                ));
            }
            std::thread::sleep(delay);
            delay = (delay * 2).min(Duration::from_millis(200));
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

impl SettingsStore {
    /// Open the store. An existing `file` that does not parse is reported, not replaced.
    pub fn new(file: Option<PathBuf>) -> Result<Self, String> {
        let state = load(file.as_deref())?;
        Ok(SettingsStore {
            state: Mutex::new(state),
            file,
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    // Read APIs cannot return I/O failures; retain the last validated snapshot
    // on failure. Writes always propagate reload errors and never repair a file.
    fn refresh(&self, state: &mut State) {
        if self.file.is_some() {
            if let Ok(fresh) = load(self.file.as_deref()) {
                *state = fresh;
            }
        }
    }

    /// The `settings/describe` value.
    pub fn describe(&self) -> Value {
        let mut state = self.lock();
        self.refresh(&mut state);
        let namespaces: Vec<Value> = state
            .namespaces
            .iter()
            .map(|(ns, entry)| entry.view(ns))
            .collect();
        json!({
            "writable": true,
            "hasDocument": false,
            "namespaces": namespaces,
        })
    }

    /// The user section of one namespace.
    pub fn user(&self, ns: &str) -> Option<Map<String, Value>> {
        let mut state = self.lock();
        self.refresh(&mut state);
        state
            .namespaces
            .iter()
            .find(|(name, _)| name == ns)
            .map(|(_, entry)| entry.user.clone())
    }

    /// The revision of one namespace.
    pub fn revision(&self, ns: &str) -> Option<u64> {
        let mut state = self.lock();
        self.refresh(&mut state);
        state
            .namespaces
            .iter()
            .find(|(name, _)| name == ns)
            .map(|(_, entry)| entry.revision)
    }

    /// Apply `change` to a copy of a namespace's user section, save, then swap it in.
    fn write(
        &self,
        ns: &str,
        expected: Option<u64>,
        change: impl FnOnce(&mut Map<String, Value>) -> Result<(), Failure>,
    ) -> Result<Value, Failure> {
        let mut guard = self.lock();
        let io_failure = |message: String| Failure {
            code: "gateway/internal".to_string(),
            message,
            details: json!({ "path": self.file.as_ref().map(|p| p.display().to_string()) }),
        };
        let _file_lock = self
            .file
            .as_ref()
            .map(|path| FileLock::acquire(path))
            .transpose()
            .map_err(|e| io_failure(format!("could not lock settings: {e}")))?;
        if self.file.is_some() {
            *guard = load(self.file.as_deref()).map_err(io_failure)?;
        }
        let Some((_, entry)) = guard.namespaces.iter_mut().find(|(name, _)| name == ns) else {
            return Err(rejected(
                ns,
                format!("No configurable plugin entry \"{ns}\""),
            ));
        };
        if let Some(expected) = expected.filter(|expected| *expected != entry.revision) {
            return Err(Failure {
                code: "settings/conflict".to_string(),
                message: format!(
                    "settings \"{ns}\" changed: expected revision {expected}, found {}",
                    entry.revision
                ),
                details: json!({ "ns": ns, "expected": expected, "actual": entry.revision }),
            });
        }
        let mut next = entry.user.clone();
        change(&mut next)?;
        if next == entry.user {
            return Ok(entry.view(ns));
        }
        let revision = entry
            .revision
            .checked_add(1)
            .ok_or_else(|| rejected(ns, "settings revision is exhausted"))?;
        if let Some(path) = &self.file {
            // The document carries the new values and the new revision of the
            // namespace being written, so a restart hands the next reader what
            // the writer intended.
            let mut document = Map::new();
            for (name, entry) in guard.namespaces.iter() {
                if name == ns {
                    document.insert(name.clone(), json!({ "user": next, "revision": revision }));
                } else {
                    document.insert(
                        name.clone(),
                        json!({ "user": entry.user, "revision": entry.revision }),
                    );
                }
            }
            // The revision is saved with the values so a restart cannot hand a
            // stale client a revision that matches different content.
            super::workspaces::write_atomically(
                path,
                &serde_json::to_vec_pretty(&Value::Object(document)).expect("a map serializes"),
            )
            .map_err(|e| Failure {
                code: "gateway/internal".to_string(),
                message: format!("could not save settings: {e}"),
                details: json!({ "path": path.display().to_string() }),
            })?;
        }
        let entry = guard
            .namespaces
            .iter_mut()
            .find(|(name, _)| name == ns)
            .map(|(_, entry)| entry)
            .expect("the namespace exists");
        entry.user = next;
        entry.revision = revision;
        Ok(entry.view(ns))
    }

    /// `settings/mutate`: ordered `set` and `unset` edits addressed by path.
    pub fn mutate(&self, ns: &str, ops: &[Value], expected: Option<u64>) -> Result<Value, Failure> {
        self.write(ns, expected, |user| {
            for op in ops {
                let kind = op.get("op").and_then(Value::as_str);
                let path = op.get("path").and_then(Value::as_array);
                let field = match path.map(Vec::as_slice) {
                    Some([Value::String(field)]) => field.clone(),
                    _ => {
                        return Err(rejected(
                            ns,
                            format!(
                                "Config field \"{}\" is not volatile",
                                path.map(|p| p
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join("."))
                                    .unwrap_or_default()
                            ),
                        ))
                    }
                };
                match kind {
                    Some("set") => {
                        let value = op.get("value").cloned().unwrap_or(Value::Null);
                        set_field(ns, user, &field, value)?;
                    }
                    Some("unset") => {
                        if field_kind(ns, &field).is_none() {
                            return Err(rejected(
                                ns,
                                format!("Config field \"{field}\" is not volatile"),
                            ));
                        }
                        user.remove(&field);
                    }
                    _ => return Err(rejected(ns, "a settings edit must be a set or an unset")),
                }
            }
            Ok(())
        })
    }

    /// `settings/update`: merge a patch into the user section.
    pub fn update(
        &self,
        ns: &str,
        patch: &Map<String, Value>,
        expected: Option<u64>,
    ) -> Result<Value, Failure> {
        self.write(ns, expected, |user| {
            for (field, value) in patch {
                set_field(ns, user, field, value.clone())?;
            }
            Ok(())
        })
    }

    /// `settings/replace`: the user section becomes exactly `section`.
    pub fn replace(
        &self,
        ns: &str,
        section: &Map<String, Value>,
        expected: Option<u64>,
    ) -> Result<Value, Failure> {
        self.write(ns, expected, |user| {
            let mut next = Map::new();
            for (field, value) in section {
                set_field(ns, &mut next, field, value.clone())?;
            }
            *user = next;
            Ok(())
        })
    }
}

/// Whether a namespace has a field.
fn field_kind(ns: &str, field: &str) -> Option<FieldKind> {
    match ns {
        NAMESPACE => (field == FIELD).then_some(FieldKind::Text),
        SPEECH_NAMESPACE => match field {
            SPEECH_PROVIDER | SPEECH_LANGUAGE => Some(FieldKind::Text),
            _ => None,
        },
        MARKET_NAMESPACE => (field == MARKET_INDEXES).then_some(FieldKind::TextList),
        _ => None,
    }
}

/// Whether a value is what a field holds.
fn value_matches(kind: FieldKind, value: &Value) -> bool {
    match kind {
        FieldKind::Text => value.is_string(),
        FieldKind::TextList => value
            .as_array()
            .is_some_and(|items| items.iter().all(Value::is_string)),
    }
}

fn set_field(
    ns: &str,
    user: &mut Map<String, Value>,
    field: &str,
    value: Value,
) -> Result<(), Failure> {
    let Some(kind) = field_kind(ns, field) else {
        return Err(rejected(
            ns,
            format!("Config field \"{field}\" is not volatile"),
        ));
    };
    if !value_matches(kind, &value) {
        return Err(rejected(
            ns,
            format!(
                "Config field \"{field}\" must be {}",
                match kind {
                    FieldKind::Text => "a string",
                    FieldKind::TextList => "a list of strings",
                }
            ),
        ));
    }
    user.insert(field.to_string(), value);
    Ok(())
}
