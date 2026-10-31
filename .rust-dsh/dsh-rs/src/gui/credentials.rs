//! `credentials/describe`, `credentials/set` and `credentials/unset`: the
//! file-backed credential store behind the settings pages.
//!
//! The store is the TypeScript `credentials-local` document, `.credentials.yaml`
//! in the harness home, so both hosts read and write one file:
//!
//! ```yaml
//! version: 1
//! refs: { OMNIROUTE_API_KEY: sk-..., OTHER: value }
//! records: { ... }   # written by other components; kept untouched
//! ```
//!
//! A reference resolves from the file first, then from the inherited process
//! environment, so a key saved in the product outranks one the launch carried.
//! `describe` never returns a value, only `{configured, source?, writable}`.
//! Every reference is writable: a `set` writes the file, which outranks the
//! environment, and an `unset` exposes the environment again.
//!
//! **Divergences.** The `.env` fallback layers (`<cwd>/.env`, `$DSH_HOME/.env`)
//! are not read, so a reference only in a `.env` file shows as not configured.
//! A write rewrites the document through the YAML parser, so comments and
//! hand formatting inside `refs:` are not preserved the way TypeScript preserves
//! them; unrelated top-level sections (`records`, foreign keys) are kept. The
//! document is not watched, but every `describe` re-reads it, so an external
//! edit is seen on the next call.
//!
//! Without a harness home (`$DSH_HOME` and `$HOME` unset) nothing is stored:
//! all three calls answer `gateway/internal`, the TypeScript answer for a host
//! with no credential provider.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};

use super::router::{boundary, check_descriptor};
use super::wire::{codes, Failure, RpcResult};

/// File name of the document inside the harness home.
pub const CREDENTIALS_FILE: &str = ".credentials.yaml";
/// Layout version this build reads and writes.
const DOCUMENT_VERSION: u64 = 1;
/// Most references one `describe` accepts.
const MAX_DESCRIBE_REFS: usize = 64;
/// How long a write waits for another process's writer lock.
const LOCK_WAIT: Duration = Duration::from_secs(30);
/// Group and other permission bits a credentials file must not carry.
#[cfg(unix)]
const GROUP_OTHER_BITS: u32 = 0o077;

/// The credential store: one document path, read per call.
#[derive(Debug)]
pub struct CredentialStore {
    file: Option<PathBuf>,
    /// Serialises this process's writes; the `.lock` file serialises processes.
    writes: tokio::sync::Mutex<()>,
}

impl CredentialStore {
    /// A store over `file`, or one with no home when `file` is `None`.
    pub fn new(file: Option<PathBuf>) -> Self {
        CredentialStore {
            file,
            writes: tokio::sync::Mutex::new(()),
        }
    }

    /// The store at `$DSH_HOME/.credentials.yaml` (else `~/.dsh`).
    pub fn at_harness_home() -> Self {
        Self::new(
            super::commands::harness_home(|name| std::env::var(name).ok())
                .map(|home| home.join(CREDENTIALS_FILE)),
        )
    }

    /// Read a provider record: the section other components write.
    ///
    /// The record is the whole value, so a caller reads what it wrote.
    pub async fn read_record(&self, key: &str) -> Result<Option<Value>, Failure> {
        let file = self.file()?;
        let document = load(file).map_err(|message| Failure {
            code: "credential/rejected".to_string(),
            message,
            details: json!({ "key": key }),
        })?;
        Ok(document.record(key))
    }

    /// Write a provider record, atomically with every other writer.
    pub async fn write_record(&self, key: &str, value: Value) -> Result<(), Failure> {
        self.modify_record(key, |_| Some(value)).await.map(|_| ())
    }

    /// Decide a record mutation from the document re-read under both writer locks.
    /// Returning None keeps the existing record; the result is the stored record.
    pub async fn modify_record(
        &self,
        key: &str,
        change: impl FnOnce(Option<Value>) -> Option<Value>,
    ) -> Result<Option<Value>, Failure> {
        let mut result = None;
        let mut invalid = false;
        write_document(self, |document| {
            let current = document.record(key);
            if let Some(value) = change(current.clone()) {
                if !valid_record(key, &value) {
                    invalid = true;
                    return false;
                }
                result = Some(value.clone());
                if current == result {
                    return false;
                }
                document.set_record(key, Some(value));
                true
            } else {
                result = current;
                false
            }
        })
        .await?;
        if invalid {
            return Err(rejected("credentials-local: invalid record"));
        }
        Ok(result)
    }

    /// Remove a provider record, atomically with every other writer.
    pub async fn delete_record(&self, key: &str) -> Result<(), Failure> {
        write_document(self, |document| {
            if document.record(key).is_none() {
                return false;
            }
            document.set_record(key, None);
            true
        })
        .await
    }

    fn file(&self) -> Result<&Path, Failure> {
        self.file.as_deref().ok_or_else(|| {
            Failure::at(
                codes::INTERNAL,
                "credentials service is absent: this deployment has no harness home to keep a credential document in",
                "credentials",
            )
        })
    }
}

/// A reference name: a POSIX identifier.
fn is_reference(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// Mirrors credentials-local parseCredentialKey and parseRecord. JSON values are
// already finite, acyclic JSON here; YAML conversion is checked separately.
fn valid_record(key: &str, value: &Value) -> bool {
    let parts: Vec<_> = key.split('/').collect();
    if parts.len() != 2
        || !parts.iter().all(|part| {
            part.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
    {
        return false;
    }
    let Some(fields) = value.as_object() else {
        return false;
    };
    match fields.get("kind").and_then(Value::as_str) {
        Some("grant") => {
            fields.contains_key("payload")
                && fields
                    .keys()
                    .all(|k| ["kind", "payload"].contains(&k.as_str()))
        }
        Some("api-key") => {
            fields
                .keys()
                .all(|k| ["kind", "key", "env"].contains(&k.as_str()))
                && fields
                    .get("key")
                    .is_none_or(|v| v.as_str().is_some_and(|s| !s.is_empty()))
                && fields.get("env").is_none_or(|v| {
                    v.as_object().is_some_and(|env| {
                        env.iter().all(|(k, v)| {
                            is_reference(k) && v.as_str().is_some_and(|s| !s.is_empty())
                        })
                    })
                })
        }
        _ => false,
    }
}

/// One schema issue in the shape the TypeScript validator reports it.
fn issue(origin: &str, code: &str, path: Value, extra: Value, message: &str) -> Value {
    let mut issue = json!({ "origin": origin, "code": code });
    if let (Some(target), Some(fields)) = (issue.as_object_mut(), extra.as_object()) {
        target.extend(fields.clone());
    }
    issue["path"] = path;
    issue["message"] = json!(message);
    issue
}

fn bad_reference(path: Value) -> Value {
    issue(
        "string",
        "invalid_format",
        path,
        json!({ "format": "regex", "pattern": "/^[A-Za-z_][A-Za-z0-9_]*$/" }),
        "Invalid string: must match pattern /^[A-Za-z_][A-Za-z0-9_]*$/",
    )
}

/// The failure for a request that is well-typed but breaks a value rule.
fn invalid_payload(method: &str, issues: Vec<Value>) -> Failure {
    Failure {
        code: codes::BAD_REQUEST.to_string(),
        message: format!("invalid payload for credentials.{method}"),
        details: json!({ "issues": issues }),
    }
}

// --- the document -------------------------------------------------------

/// The parsed document: the whole mapping, so a write keeps every other section.
struct Document {
    root: Map<String, Value>,
}

impl Document {
    fn empty() -> Self {
        Document { root: Map::new() }
    }

    fn refs(&self) -> Map<String, Value> {
        self.root
            .get("refs")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    }

    fn get(&self, reference: &str) -> Option<String> {
        self.root
            .get("refs")
            .and_then(Value::as_object)
            .and_then(|refs| refs.get(reference))
            .and_then(Value::as_str)
            .map(str::to_string)
    }

    /// The provider records: the section other components write.
    fn records(&self) -> Map<String, Value> {
        self.root
            .get("records")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    }

    /// One provider record, if the document holds it.
    fn record(&self, key: &str) -> Option<Value> {
        self.root
            .get("records")
            .and_then(Value::as_object)
            .and_then(|records| records.get(key))
            .cloned()
    }

    /// Set or remove one provider record.
    fn set_record(&mut self, key: &str, value: Option<Value>) {
        let mut records = self.records();
        match value {
            Some(value) => {
                records.insert(key.to_string(), value);
            }
            None => {
                records.remove(key);
            }
        }
        self.root
            .insert("version".to_string(), json!(DOCUMENT_VERSION));
        self.root
            .insert("records".to_string(), Value::Object(records));
    }

    fn set(&mut self, reference: &str, value: Option<String>) {
        let mut refs = self.refs();
        match value {
            Some(value) => {
                refs.insert(reference.to_string(), Value::String(value));
            }
            None => {
                refs.remove(reference);
            }
        }
        self.root
            .insert("version".to_string(), json!(DOCUMENT_VERSION));
        self.root.insert("refs".to_string(), Value::Object(refs));
    }

    /// The text to store. `version` leads and `refs` precedes the other sections.
    fn render(&self) -> Result<String, Failure> {
        let mut ordered = serde_norway::Mapping::new();
        let mut put = |key: &str, value: &Value| -> Result<(), Failure> {
            let value =
                serde_norway::to_value(value).map_err(|error| rejected(&error.to_string()))?;
            ordered.insert(serde_norway::Value::String(key.to_string()), value);
            Ok(())
        };
        if let Some(version) = self.root.get("version") {
            put("version", version)?;
        }
        if let Some(refs) = self.root.get("refs") {
            put("refs", refs)?;
        }
        for (key, value) in &self.root {
            if key != "version" && key != "refs" {
                put(key, value)?;
            }
        }
        serde_norway::to_string(&ordered).map_err(|error| rejected(&error.to_string()))
    }
}

/// A refusal from the store, reported with the store's own message and no value.
fn rejected(message: &str) -> Failure {
    Failure {
        code: "credential/rejected".to_string(),
        message: message.to_string(),
        details: json!({}),
    }
}

fn json_compatible_yaml(value: &serde_norway::Value) -> bool {
    use serde_norway::Value as Yaml;
    match value {
        Yaml::Number(number) => number.as_f64().is_none_or(f64::is_finite),
        Yaml::Sequence(items) => items.iter().all(json_compatible_yaml),
        Yaml::Mapping(fields) => fields
            .iter()
            .all(|(key, value)| matches!(key, Yaml::String(_)) && json_compatible_yaml(value)),
        Yaml::Tagged(_) => false,
        _ => true,
    }
}

/// Read and validate the document at `file`. An absent file is the empty store;
/// a file that exists but cannot be trusted is an error, never "no credentials".
fn load(file: &Path) -> Result<Document, String> {
    let name = file.display();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(file) {
            let offending = meta.permissions().mode() & GROUP_OTHER_BITS;
            if offending != 0 {
                return Err(format!(
                    "credentials-local: {name} is readable beyond its owner (mode {:o}); run \"chmod 600 {name}\" before starting again",
                    meta.permissions().mode() & 0o777
                ));
            }
        }
    }
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Document::empty()),
        Err(error) => {
            return Err(format!(
                "credentials-local: {name} could not be read: {}",
                error.kind()
            ))
        }
    };
    // Parser messages quote the offending line, which here may be a secret, so
    // only the failure's position is reported.
    let parsed: serde_norway::Value = serde_norway::from_str(&text).map_err(|error| {
        let at = error
            .location()
            .map(|at| format!(" at line {}, column {}", at.line(), at.column()))
            .unwrap_or_default();
        format!("credentials-local: invalid document at {name}{at}")
    })?;
    if !json_compatible_yaml(&parsed) {
        return Err(format!("credentials-local: {name} holds a non-JSON value"));
    }
    let root = match parsed {
        serde_norway::Value::Null => return Ok(Document::empty()),
        other => serde_json::to_value(other).map_err(|_| {
            format!("credentials-local: {name} holds a value no document can carry")
        })?,
    };
    let Value::Object(root) = root else {
        return Err(format!("credentials-local: {name} must be a mapping"));
    };
    if root.is_empty() {
        return Ok(Document::empty());
    }
    match root.get("version") {
        None => {
            return Err(format!(
                "credentials-local: {name} uses the pre-release flat layout. Add `version: {DOCUMENT_VERSION}` and nest the existing {} under `refs:`. No values need to change.",
                if root.len() == 1 { "1 entry".to_string() } else { format!("{} entries", root.len()) }
            ));
        }
        Some(version) if version != &json!(DOCUMENT_VERSION) => {
            return Err(format!(
                "credentials-local: {name} declares version {version}; this build reads version {DOCUMENT_VERSION}"
            ));
        }
        Some(_) => {}
    }
    if let Some(refs) = root.get("refs") {
        let Some(refs) = refs.as_object() else {
            return Err(format!(
                "credentials-local: \"refs\" in {name} must be a mapping"
            ));
        };
        for (key, value) in refs {
            if !is_reference(key) {
                return Err(format!(
                    "credentials-local: \"{key}\" in {name} is not a valid reference name"
                ));
            }
            match value.as_str() {
                None => return Err(format!("credentials-local: the value for \"{key}\" in {name} must be a string")),
                Some("") => return Err(format!("credentials-local: the value for \"{key}\" in {name} is empty; remove the key instead")),
                Some(_) => {}
            }
        }
    }
    if let Some(records) = root.get("records").filter(|v| !v.is_null()) {
        let Some(records) = records.as_object() else {
            return Err(format!(
                "credentials-local: \"records\" in {name} must be a mapping"
            ));
        };
        if records.iter().any(|(key, value)| !valid_record(key, value)) {
            return Err(format!("credentials-local: invalid record in {name}"));
        }
    }
    Ok(Document { root })
}

/// Exclusive create of `<file>.lock`, retried until `LOCK_WAIT`.
struct FileLock {
    path: PathBuf,
}

impl FileLock {
    async fn acquire(file: &Path) -> Result<FileLock, String> {
        let path = PathBuf::from(format!("{}.lock", file.display()));
        let deadline = Instant::now() + LOCK_WAIT;
        let mut delay = Duration::from_millis(20);
        loop {
            match open_exclusive(&path) {
                Ok(()) => return Ok(FileLock { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if takeover_if_holder_exited(&path) {
                        continue;
                    }
                }
                Err(error) => {
                    return Err(format!(
                        "credentials-local: the writer lock could not be taken: {}",
                        error.kind()
                    ))
                }
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "atomic-write: timed out waiting for the writer lock at {}",
                    path.display()
                ));
            }
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(Duration::from_millis(200));
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn open_exclusive(path: &Path) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    writeln!(file, "{}", std::process::id())
}

/// Remove a lock whose recorded process no longer exists. Returns whether the
/// lock was removed. A record that cannot be read or parsed is left alone.
fn takeover_if_holder_exited(path: &Path) -> bool {
    let Ok(record) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(pid) = record.trim().parse::<i32>() else {
        return false;
    };
    if pid <= 0 || !process_exited(pid) {
        return false;
    }
    // Re-read: only remove the lock this check was made against.
    std::fs::read_to_string(path).is_ok_and(|again| again == record)
        && std::fs::remove_file(path).is_ok()
}

fn process_exited(pid: i32) -> bool {
    // Signal 0 checks existence; ESRCH means the process is gone.
    // SAFETY: kill with signal 0 sends nothing.
    let result = unsafe { libc::kill(pid, 0) };
    result != 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
}

/// Create the document's directory, private to the owner when it is new.
fn make_home(file: &Path) -> std::io::Result<()> {
    let dir = file
        .parent()
        .ok_or_else(|| std::io::Error::other("the document has no directory"))?;
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)
}

/// Write `text` to `file` by renaming a private temporary file over it.
fn write_atomic(file: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    let dir = file
        .parent()
        .ok_or_else(|| std::io::Error::other("the document has no directory"))?;
    let temporary = dir.join(format!(
        ".{}.{}.tmp",
        file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("credentials"),
        std::process::id()
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut handle = options.open(&temporary)?;
        handle.write_all(text.as_bytes())?;
        handle.sync_all()?;
        std::fs::rename(&temporary, file)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

// --- resolution ---------------------------------------------------------

/// Where a reference's value comes from, without the value.
fn describe_one(
    document: &Document,
    reference: &str,
    environment: &impl Fn(&str) -> Option<String>,
) -> Value {
    if document.get(reference).is_some() {
        return json!({ "configured": true, "source": "file", "writable": true });
    }
    if environment(reference).is_some_and(|value| !value.is_empty()) {
        return json!({ "configured": true, "source": "env", "writable": true });
    }
    json!({ "configured": false, "writable": true })
}

// --- endpoints ----------------------------------------------------------

/// Serve one of the three credential endpoints, or `None` for another method.
pub(super) async fn dispatch(
    store: &CredentialStore,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    let result = match method {
        "credentials/describe" => describe(store, payload),
        "credentials/set" => set(store, payload).await,
        "credentials/unset" => unset(store, payload).await,
        _ => return None,
    };
    Some(match result {
        Ok(Some(value)) => RpcResult::ok(value),
        Ok(None) => RpcResult::Empty,
        Err(failure) => RpcResult::failed(failure),
    })
}

fn describe(store: &CredentialStore, payload: &Value) -> Result<Option<Value>, Failure> {
    const ENDPOINT: &str = "credentials/describe";
    check_descriptor(ENDPOINT, &["refs"], payload)?;
    let refs = payload["refs"]
        .as_array()
        .ok_or_else(|| boundary(ENDPOINT, "refs"))?;
    let names: Vec<&str> = refs
        .iter()
        .map(|name| name.as_str().ok_or_else(|| boundary(ENDPOINT, "refs")))
        .collect::<Result<_, _>>()?;
    let mut issues: Vec<Value> = names
        .iter()
        .enumerate()
        .filter(|(_, name)| !is_reference(name))
        .map(|(index, _)| bad_reference(json!(["refs", index])))
        .collect();
    if names.len() > MAX_DESCRIBE_REFS {
        issues.push(issue(
            "array",
            "too_big",
            json!(["refs"]),
            json!({ "maximum": MAX_DESCRIBE_REFS, "inclusive": true }),
            &format!("Too big: expected array to have <={MAX_DESCRIBE_REFS} items"),
        ));
    }
    if !issues.is_empty() {
        return Err(invalid_payload("describe", issues));
    }
    let file = store.file()?;
    let document = load(file).map_err(|reason| Failure::at(codes::INTERNAL, reason, ENDPOINT))?;
    let environment = |name: &str| std::env::var(name).ok();
    let mut described = Map::new();
    for name in names {
        described.insert(
            name.to_string(),
            describe_one(&document, name, &environment),
        );
    }
    Ok(Some(Value::Object(described)))
}

/// The wire strings `ref` and `value` (or `ref` alone), after the gateway's type check.
fn text_args<'a>(
    endpoint: &str,
    payload: &'a Value,
    fields: &[&str],
) -> Result<Vec<&'a str>, Failure> {
    check_descriptor(endpoint, fields, payload)?;
    fields
        .iter()
        .map(|field| {
            payload[field]
                .as_str()
                .ok_or_else(|| boundary(endpoint, field))
        })
        .collect()
}

async fn set(store: &CredentialStore, payload: &Value) -> Result<Option<Value>, Failure> {
    const ENDPOINT: &str = "credentials/set";
    let args = text_args(ENDPOINT, payload, &["ref", "value"])?;
    let (reference, value) = (args[0], args[1]);
    let mut issues = Vec::new();
    if !is_reference(reference) {
        issues.push(bad_reference(json!(["ref"])));
    }
    if value.is_empty() {
        issues.push(issue(
            "string",
            "too_small",
            json!(["value"]),
            json!({ "minimum": 1, "inclusive": true }),
            "Too small: expected string to have >=1 characters",
        ));
    }
    if !issues.is_empty() {
        return Err(invalid_payload("set", issues));
    }
    write(store, reference, Some(value.to_string())).await?;
    Ok(None)
}

async fn unset(store: &CredentialStore, payload: &Value) -> Result<Option<Value>, Failure> {
    const ENDPOINT: &str = "credentials/unset";
    let args = text_args(ENDPOINT, payload, &["ref"])?;
    if !is_reference(args[0]) {
        return Err(invalid_payload(
            "unset",
            vec![bad_reference(json!(["ref"]))],
        ));
    }
    write(store, args[0], None).await?;
    Ok(None)
}

/// Apply one change under the writer locks. A refusal carries the store's own
/// message and names only the reference, so no path can return the value.
/// Apply `change` to the document under the store's lock, and save it.
async fn write_document(
    store: &CredentialStore,
    change: impl FnOnce(&mut Document) -> bool,
) -> Result<(), Failure> {
    let file = store.file()?;
    let _in_process = store.writes.lock().await;
    make_home(file).map_err(|error| Failure {
        code: "credential/rejected".to_string(),
        message: format!(
            "credentials-local: the harness home could not be created: {}",
            error.kind()
        ),
        details: json!({}),
    })?;
    let _across_processes = FileLock::acquire(file).await.map_err(|error| Failure {
        code: "credential/rejected".to_string(),
        message: error,
        details: json!({}),
    })?;
    // Re-read under the lock, so a write by another process is not overwritten.
    let mut document = load(file).map_err(|error| Failure {
        code: "credential/rejected".to_string(),
        message: error,
        details: json!({}),
    })?;
    if !change(&mut document) {
        return Ok(());
    }
    let text = document.render().map_err(|failure| Failure {
        code: "credential/rejected".to_string(),
        message: failure.message,
        details: json!({}),
    })?;
    write_atomic(file, &text).map_err(|error| Failure {
        code: "credential/rejected".to_string(),
        message: format!(
            "credentials-local: the document could not be written: {}",
            error.kind()
        ),
        details: json!({}),
    })
}

async fn write(
    store: &CredentialStore,
    reference: &str,
    value: Option<String>,
) -> Result<(), Failure> {
    let refusal = |message: String| Failure {
        code: "credential/rejected".to_string(),
        message,
        details: json!({ "ref": reference }),
    };
    let file = store.file()?;
    let _in_process = store.writes.lock().await;
    // The lock file lives beside the document, so its directory comes first.
    make_home(file).map_err(|error| {
        refusal(format!(
            "credentials-local: the harness home could not be created: {}",
            error.kind()
        ))
    })?;
    let _across_processes = FileLock::acquire(file).await.map_err(refusal)?;
    // Re-read under the lock, so a write by another process is not overwritten.
    let mut document = load(file).map_err(refusal)?;
    if value.is_none() && document.get(reference).is_none() {
        return Ok(());
    }
    document.set(reference, value);
    let text = document
        .render()
        .map_err(|failure| refusal(failure.message))?;
    write_atomic(file, &text).map_err(|error| {
        refusal(format!(
            "credentials-local: the document could not be written: {}",
            error.kind()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_names_follow_the_posix_identifier_grammar() {
        for good in ["A", "_", "a1", "OMNIROUTE_API_KEY", "__proto__"] {
            assert!(is_reference(good), "{good}");
        }
        for bad in ["", "1a", "a-b", "a b", "a.b", "é", "a\n"] {
            assert!(!is_reference(bad), "{bad:?}");
        }
    }
}
