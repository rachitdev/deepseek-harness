//! The reminder store: `$DSH_HOME/storages/schedule.json`.
//!
//! This is the TypeScript `storage-json` single-unit layout, so both hosts keep
//! their reminders in one file:
//!
//! ```json
//! { "unit": { "name": "schedule", "version": 1 },
//!   "global": null,
//!   "tables": { "tasks": { "<id>": { "sessionId", "record", "status",
//!                                    "lastDelivery"?, "deliveryHistory"? } } } }
//! ```
//!
//! A task is `{sessionId, record, status: "active"|"inactive", lastDelivery?,
//! deliveryHistory?}` with the record as `domain` defines it. Every write
//! publishes the whole file by renaming a synced temporary file over it, and a
//! failed publish restores the previous state in memory.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::domain;

/// A JSON object read with its keys in file order, which `serde_json::Value`
/// does not keep (it sorts them).
struct Ordered(Vec<(String, Value)>);

impl<'de> serde::Deserialize<'de> for Ordered {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visit;
        impl<'de> serde::de::Visitor<'de> for Visit {
            type Value = Ordered;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Ordered, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, Value>()? {
                    entries.push(entry);
                }
                Ok(Ordered(entries))
            }
        }
        deserializer.deserialize_map(Visit)
    }
}

/// The unit's name and version.
const UNIT: &str = "schedule";
const VERSION: u64 = 1;
const TABLE: &str = "tasks";

/// Why the store could not be opened or written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError(pub String);

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StoreError {}

/// An open reminder store.
#[derive(Debug)]
pub struct Store {
    path: Option<PathBuf>,
    /// Tasks in insertion order, which is the order the file lists them.
    tasks: Vec<(String, Value)>,
}

/// Check one stored task, as the TypeScript task schema does.
pub fn validate_task(key: &str, task: &Value) -> Result<(), StoreError> {
    let bad = |message: String| StoreError(format!("schedule: stored task {key:?} is invalid: {message}"));
    let Some(object) = task.as_object() else {
        return Err(bad("not an object".into()));
    };
    let allowed = ["sessionId", "record", "status", "lastDelivery", "deliveryHistory"];
    if let Some(extra) = object.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(bad(format!("unexpected key {extra:?}")));
    }
    if object.get("sessionId").and_then(Value::as_str).is_none_or(str::is_empty) {
        return Err(bad("sessionId must be a non-empty string".into()));
    }
    if let Some(status) = object.get("status") {
        if !matches!(status.as_str(), Some("active" | "inactive")) {
            return Err(bad("status must be active or inactive".into()));
        }
    }
    domain::decode_record(object.get("record").unwrap_or(&Value::Null))
        .map_err(|e| bad(e.message().to_string()))?;
    let id = object["record"]["id"].as_str().unwrap_or_default();
    if key != id {
        return Err(StoreError(format!("schedule: stored task key \"{key}\" differs from record id \"{id}\"")));
    }
    if let Some(receipt) = object.get("lastDelivery") {
        validate_receipt(receipt, false).map_err(bad)?;
    }
    if let Some(history) = object.get("deliveryHistory") {
        validate_history(history, object.get("lastDelivery")).map_err(bad)?;
    }
    Ok(())
}

fn validate_receipt(receipt: &Value, with_prompt: bool) -> Result<(), String> {
    let Some(object) = receipt.as_object() else {
        return Err("a delivery receipt must be an object".into());
    };
    let allowed: &[&str] = if with_prompt { &["scheduledAt", "deliveredAt", "messageId", "prompt"] } else { &["scheduledAt", "deliveredAt", "messageId"] };
    if object.keys().any(|k| !allowed.contains(&k.as_str())) || !["scheduledAt", "deliveredAt", "messageId"].iter().all(|k| object.contains_key(*k)) {
        return Err("a delivery receipt has the wrong keys".into());
    }
    for key in ["scheduledAt", "deliveredAt"] {
        super::instant::parse_stored_instant(object[key].as_str().unwrap_or_default())
            .map_err(|_| "Expected a canonical four-digit-year UTC calendar instant".to_string())?;
    }
    match object["messageId"].as_str() {
        Some(id) if !id.is_empty() && id.trim() == id => {}
        _ => return Err("messageId must be a non-empty trimmed string".into()),
    }
    if with_prompt && object.get("prompt").is_some_and(|p| !p.is_string()) {
        return Err("prompt must be a string".into());
    }
    Ok(())
}

fn validate_history(history: &Value, last: Option<&Value>) -> Result<(), String> {
    let Some(object) = history.as_object() else {
        return Err("deliveryHistory must be an object".into());
    };
    let allowed = ["records", "earlierRecordsUnavailable", "earlierRecordsPruned"];
    if object.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("deliveryHistory has an unexpected key".into());
    }
    let Some(records) = object.get("records").and_then(Value::as_array) else {
        return Err("deliveryHistory.records must be an array".into());
    };
    if !object.get("earlierRecordsUnavailable").is_some_and(Value::is_boolean)
        || object.get("earlierRecordsPruned").is_some_and(|v| !v.is_boolean())
    {
        return Err("deliveryHistory flags must be booleans".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    for record in records {
        validate_receipt(record, true)?;
        if !seen.insert(record["messageId"].as_str().unwrap_or_default().to_string()) {
            return Err("Delivery history message identities must be unique within a task".into());
        }
    }
    let message = "Last delivery must match the latest saved delivery receipt";
    match (records.last(), last) {
        (None, None) => Ok(()),
        (None, Some(_)) | (Some(_), None) => Err(message.into()),
        (Some(latest), Some(last)) => {
            let same = ["scheduledAt", "deliveredAt", "messageId"].iter().all(|k| latest[*k] == last[*k]);
            if same { Ok(()) } else { Err(message.into()) }
        }
    }
}

impl Store {
    /// An empty store that keeps nothing on disk.
    pub fn in_memory() -> Self {
        Store { path: None, tasks: Vec::new() }
    }

    /// Open `path`, reading the tasks it holds. A missing file is an empty
    /// store; a file that cannot be trusted is an error and is left untouched.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => Some(text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(StoreError(format!("schedule: {} could not be read: {}", path.display(), error.kind()))),
        };
        let mut tasks: Vec<(String, Value)> = Vec::new();
        if let Some(text) = text {
            let top: Ordered = serde_json::from_str(&text)
                .map_err(|_| StoreError(format!("unit '{UNIT}': file is not valid JSON")))?;
            let member = |entries: &[(String, Value)], name: &str| entries.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone());
            let unit = member(&top.0, "unit");
            let header = |key: &str| unit.as_ref().and_then(|u| u.get(key));
            if header("name") != Some(&json!(UNIT)) || !header("version").is_some_and(Value::is_number) {
                return Err(StoreError(format!("unit '{UNIT}': missing or foreign unit header")));
            }
            if header("version").and_then(Value::as_u64) != Some(VERSION) {
                return Err(StoreError(format!(
                    "unit '{UNIT}': stored version {} is not accepted by version {VERSION}",
                    header("version").map_or_else(|| "?".to_string(), ToString::to_string)
                )));
            }
            // The tables member is read again, in file order, down to the tasks.
            let tables = member(&top.0, "tables").filter(Value::is_object);
            let Some(tables) = tables else {
                return Err(StoreError(format!("unit '{UNIT}': tables is not an object")));
            };
            if tables.get(TABLE).is_some_and(|rows| !rows.is_object()) {
                return Err(StoreError(format!("unit '{UNIT}': table '{TABLE}' is not an object")));
            }
            if tables.get(TABLE).is_some() {
                tasks = ordered_tasks(&text)?;
                for (key, task) in &mut tasks {
                    validate_task(key, task)?;
                    *task = normalise(task.clone());
                }
            }
        }
        Ok(Store { path: Some(path.to_path_buf()), tasks })
    }

    /// Every task, in the order the file lists them.
    pub fn tasks(&self) -> impl Iterator<Item = &Value> {
        self.tasks.iter().map(|(_, task)| task)
    }

    /// One task by id.
    pub fn get(&self, id: &str) -> Option<&Value> {
        self.tasks.iter().find(|(key, _)| key == id).map(|(_, task)| task)
    }

    fn position(&self, id: &str) -> Option<usize> {
        self.tasks.iter().position(|(key, _)| key == id)
    }

    /// Insert or replace `task`, keyed by its record id.
    pub fn put(&mut self, task: Value) -> Result<(), StoreError> {
        let id = task["record"]["id"].as_str().unwrap_or_default().to_string();
        // A replaced task keeps its place; a new one goes last.
        let previous = match self.position(&id) {
            Some(at) => Some((at, std::mem::replace(&mut self.tasks[at].1, normalise(task)))),
            None => {
                self.tasks.push((id.clone(), normalise(task)));
                None
            }
        };
        self.publish().inspect_err(|_| match previous {
            Some((at, previous)) => self.tasks[at].1 = previous,
            None => {
                self.tasks.pop();
            }
        })
    }

    /// Remove every id that is present; returns how many were.
    pub fn delete_many(&mut self, ids: &[String]) -> Result<usize, StoreError> {
        let before = self.tasks.clone();
        self.tasks.retain(|(key, _)| !ids.contains(key));
        let count = before.len() - self.tasks.len();
        if count == 0 {
            return Ok(0);
        }
        match self.publish() {
            Ok(()) => Ok(count),
            Err(error) => {
                self.tasks = before;
                Err(error)
            }
        }
    }

    fn publish(&self) -> Result<(), StoreError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let text = self.render().map_err(|e| StoreError(e.to_string()))?;
        write_atomic(path, &text).map_err(|error| StoreError(format!("schedule: the store could not be written: {}", error.kind())))
    }

    /// The file's text: the TypeScript layout, two-space indent. `serde_json`
    /// orders object keys alphabetically, and a task keeps its place in the file
    /// (it is the order `list` answers in), so the table is written entry by
    /// entry with each task pretty-printed in place.
    fn render(&self) -> Result<String, serde_json::Error> {
        let mut out = String::from("{\n  \"unit\": {\n    \"name\": \"schedule\",\n    \"version\": 1\n  },\n  \"global\": null,\n  \"tables\": {\n    \"tasks\": ");
        if self.tasks.is_empty() {
            out.push_str("{}");
        } else {
            out.push_str("{\n");
            for (index, (key, task)) in self.tasks.iter().enumerate() {
                let body = serde_json::to_string_pretty(task)?.replace('\n', "\n      ");
                out.push_str(&format!("      {}: {body}", serde_json::to_string(key)?));
                out.push_str(if index + 1 == self.tasks.len() { "\n" } else { ",\n" });
            }
            out.push_str("    }");
        }
        out.push_str("\n  }\n}\n");
        Ok(out)
    }
}

/// A task as the store holds it: `status` is always present.
fn normalise(mut task: Value) -> Value {
    if let Some(object) = task.as_object_mut() {
        object.entry("status").or_insert(json!("active"));
    }
    task
}

fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    let dir = path.parent().ok_or_else(|| std::io::Error::other("the store has no directory"))?;
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)?;
    let temporary = dir.join(format!(".{}.tmp", super::super::plugins::unique_id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut handle = options.open(&temporary)?;
        handle.write_all(text.as_bytes())?;
        handle.sync_all()?;
        std::fs::rename(&temporary, path)?;
        std::fs::File::open(dir)?.sync_all()
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

/// The default file: `$DSH_HOME/storages/schedule.json`.
pub fn default_path() -> Option<PathBuf> {
    crate::gui::harness_home(|name| std::env::var(name).ok()).map(|home| home.join("storages").join("schedule.json"))
}

// --- delivery history ---------------------------------------------------

/// How long and how many delivery receipts are kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retention {
    pub days: u64,
    pub records: usize,
}

impl Default for Retention {
    fn default() -> Self {
        Retention { days: 30, records: 200 }
    }
}

const DAY_MS: i64 = 86_400_000;

/// The task's history, synthesised from `lastDelivery` when it has none.
fn history_of(task: &Value) -> Value {
    if let Some(history) = task.get("deliveryHistory") {
        return history.clone();
    }
    let records: Vec<Value> = task.get("lastDelivery").cloned().into_iter().collect();
    json!({ "records": records, "earlierRecordsUnavailable": true })
}

/// `task` after recording one delivery: `lastDelivery` and a bounded history.
pub fn append_delivery(task: &Value, receipt: &Value, retention: Retention) -> Value {
    let history = history_of(task);
    let mut appended: Vec<Value> = history["records"].as_array().cloned().unwrap_or_default();
    let mut entry = receipt.clone();
    entry["prompt"] = task["record"]["prompt"].clone();
    appended.push(entry);
    let delivered = super::instant::parse_stored_instant(receipt["deliveredAt"].as_str().unwrap_or_default()).unwrap_or(0);
    let floor = delivered - (retention.days as i64) * DAY_MS;
    let mut retained: Vec<Value> = appended
        .iter()
        .filter(|record| super::instant::parse_stored_instant(record["deliveredAt"].as_str().unwrap_or_default()).is_ok_and(|at| at >= floor))
        .cloned()
        .collect();
    if retained.len() > retention.records {
        retained.drain(..retained.len() - retention.records);
    }
    let pruned = retained.len() != appended.len();
    let mut next = task.clone();
    next["lastDelivery"] = receipt.clone();
    next["deliveryHistory"] = json!({
        "records": retained,
        "earlierRecordsUnavailable": history["earlierRecordsUnavailable"] == json!(true) || pruned,
        "earlierRecordsPruned": history["earlierRecordsPruned"] == json!(true) || pruned,
    });
    next
}

/// One page of delivery history, newest first, or the miss code.
pub fn history_page(task: &Value, id: &str, limit: usize, before: Option<&str>, retention: Retention) -> Value {
    let history = history_of(task);
    let records = history["records"].as_array().cloned().unwrap_or_default();
    let end = match before {
        None => records.len(),
        Some(before) => match records.iter().position(|record| record["messageId"] == before) {
            Some(at) => at,
            None => return json!({ "id": id, "code": "delivery_cursor_not_found" }),
        },
    };
    let start = end.saturating_sub(limit);
    let page: Vec<Value> = records[start..end].iter().rev().cloned().collect();
    let mut result = json!({
        "id": id,
        "records": page,
        "earlierRecordsUnavailable": history["earlierRecordsUnavailable"],
        "earlierRecordsPruned": history["earlierRecordsPruned"] == json!(true),
        "retention": { "days": retention.days, "records": retention.records },
    });
    if let (Some(oldest), true) = (page_oldest(&result), start > 0) {
        result["nextBefore"] = oldest;
    }
    result
}

fn page_oldest(result: &Value) -> Option<Value> {
    result["records"].as_array()?.last().map(|record| record["messageId"].clone())
}


/// The `(id, task)` pairs of `tables.tasks`, in the order the file writes them.
fn ordered_tasks(text: &str) -> Result<Vec<(String, Value)>, StoreError> {
    #[derive(serde::Deserialize)]
    struct Tables {
        tasks: Option<Ordered>,
    }
    #[derive(serde::Deserialize)]
    struct Unit {
        tables: Tables,
    }
    let unit: Unit = serde_json::from_str(text).map_err(|_| StoreError(format!("unit '{UNIT}': file is not valid JSON")))?;
    Ok(unit.tables.tasks.map(|ordered| ordered.0).unwrap_or_default())
}
