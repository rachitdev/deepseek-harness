//! The schedule service: the operations behind `schedule/*` and the
//! `schedule_*` tools, and the driver that delivers due reminders.
//!
//! Every mutation runs on one lane (a mutex), as the TypeScript service
//! serialises its operations, so a delivery and a delete never interleave.
//! A reminder is delivered into its session as a user message with source
//! `{kind: "schedule"}`; the session need not be live, because the delivery
//! resumes a stored session first.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use cordis::Context;
use serde_json::{json, Value};
use tokio::sync::{Mutex, Notify};

use super::domain::{self, js_trim};
use super::instant;
use super::store::{append_delivery, history_page, Retention, Store, StoreError};
use super::update::{self, Resolved};
use super::ScheduleError;
use crate::api::services::{AgentRegistryService, AgentView};
use crate::types::{ContentBlock, Message, MessageSource, Role};

/// The service key.
pub const SCHEDULE_SERVICE: &str = "schedule";

/// The longest a single timer sleeps, as a JavaScript timer: 2^31 - 1 ms.
const MAX_TIMER_MS: u64 = 2_147_483_647;
/// How long a failed delivery waits before it is tried again.
const RETRY_AFTER: Duration = Duration::from_secs(30);

/// A failure that is not the caller's: the store, or a lane that has stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceError(pub String);

impl From<StoreError> for ServiceError {
    fn from(error: StoreError) -> Self {
        ServiceError(error.0)
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why an operation did not complete.
#[derive(Debug, Clone, PartialEq)]
pub enum Failure {
    /// The caller's input was refused.
    Input(ScheduleError),
    /// The service itself failed.
    Service(ServiceError),
}

impl From<ScheduleError> for Failure {
    fn from(error: ScheduleError) -> Self {
        Failure::Input(error)
    }
}

impl From<StoreError> for Failure {
    fn from(error: StoreError) -> Self {
        Failure::Service(error.into())
    }
}

/// The request to create a reminder: exactly one timing selector.
#[derive(Debug, Clone, Default)]
pub struct CreateRequest {
    pub prompt: String,
    pub title: String,
    pub after_seconds: Option<Value>,
    pub at: Option<Value>,
    pub every_seconds: Option<Value>,
    pub daily: Option<Value>,
    pub weekly: Option<Value>,
    pub cron: Option<Value>,
}

/// The reminder service.
pub struct ScheduleService {
    /// The tasks, behind the single operation lane.
    store: Mutex<Store>,
    retention: Retention,
    /// Rings when the next due time may have changed.
    wake: Notify,
    stopping: AtomicBool,
    /// Callers that want to know after a change (the catalog watchers).
    changed: Notify,
    /// The context deliveries run in; set when the driver starts.
    ctx: StdMutex<Option<Context>>,
    /// Time source, replaceable in tests.
    clock: Box<dyn Fn() -> i64 + Send + Sync>,
}

fn system_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl ScheduleService {
    /// A service over `store`.
    pub fn new(store: Store, retention: Retention) -> Arc<Self> {
        Self::with_clock(store, retention, Box::new(system_now))
    }

    /// A service with its own time source.
    pub fn with_clock(store: Store, retention: Retention, clock: Box<dyn Fn() -> i64 + Send + Sync>) -> Arc<Self> {
        Arc::new(ScheduleService {
            store: Mutex::new(store),
            retention,
            wake: Notify::new(),
            stopping: AtomicBool::new(false),
            changed: Notify::new(),
            ctx: StdMutex::new(None),
            clock,
        })
    }

    /// The current time in milliseconds since the epoch, from this service's clock.
    pub fn now(&self) -> i64 {
        (self.clock)()
    }

    fn refuse_if_stopping(&self) -> Result<(), Failure> {
        if self.stopping.load(Ordering::SeqCst) {
            return Err(Failure::Service(ServiceError("Schedule service is stopping".into())));
        }
        Ok(())
    }

    fn changed(&self) {
        self.changed.notify_waiters();
        self.wake.notify_one();
    }

    /// A handle that resolves after the next change to the task set.
    pub fn changes(&self) -> &Notify {
        &self.changed
    }

    /// Whether `session` is a delegated sub-agent, which never receives reminders.
    fn is_subagent(&self, session: &str) -> bool {
        let Some(ctx) = self.ctx.lock().unwrap().clone() else {
            return false;
        };
        ctx.get::<crate::plugins::subagent::Lineage>(crate::plugins::subagent::LINEAGE_SERVICE)
            .is_some_and(|lineage| lineage.is_child(session))
    }

    // --- operations ------------------------------------------------------

    /// Create a reminder in `session_id`.
    pub async fn create(&self, session_id: &str, request: &CreateRequest) -> Result<Value, Failure> {
        let selectors = [&request.at, &request.after_seconds, &request.every_seconds, &request.daily, &request.weekly, &request.cron]
            .iter()
            .filter(|selector| selector.is_some())
            .count();
        if selectors > 1 {
            return Err(ScheduleError::input("invalid_selector", "Exactly one reminder selector is required.").into());
        }
        let title = domain::schedule_title(&request.title)?;
        let id = format!("schedule-{}", uuid_v4());
        let now = self.now();
        let prompt = &request.prompt;
        let record = if let Some(at) = &request.at {
            domain::create_at(&id, prompt, at, now, &title)?
        } else if let Some(after) = &request.after_seconds {
            domain::create_after(&id, prompt, after, now, &title)?
        } else if let Some(every) = &request.every_seconds {
            domain::create_every(&id, prompt, every, now, &title)?
        } else if let Some(daily) = &request.daily {
            domain::create_daily(&id, prompt, daily, now, &title)?
        } else if let Some(weekly) = &request.weekly {
            domain::create_weekly(&id, prompt, weekly, now, &title)?
        } else if let Some(cron) = &request.cron {
            domain::create_cron(&id, prompt, cron, now, &title)?
        } else {
            return Err(ScheduleError::input("invalid_selector", "Exactly one reminder selector is required.").into());
        };
        let mut store = self.store.lock().await;
        self.refuse_if_stopping()?;
        if self.is_subagent(session_id) {
            return Err(ScheduleError::input(
                "subagent_session",
                "This Session belongs to subagent routing, which never receives reminder delivery.",
            )
            .into());
        }
        store.put(json!({
            "sessionId": session_id,
            "record": record,
            "status": "active",
            "deliveryHistory": { "records": [], "earlierRecordsUnavailable": false },
        }))?;
        drop(store);
        self.changed();
        Ok(record)
    }

    /// The session's active reminders, in storage order.
    pub async fn list(&self, session_id: &str) -> Vec<Value> {
        let store = self.store.lock().await;
        store
            .tasks()
            .filter(|task| task["sessionId"] == session_id && task["status"] == "active")
            .map(|task| task["record"].clone())
            .collect()
    }

    /// Every reminder with its session, status and last delivery, by time then id.
    pub async fn catalog(&self) -> Vec<Value> {
        let store = self.store.lock().await;
        let mut rows: Vec<Value> = store
            .tasks()
            .map(|task| {
                let mut row = task["record"].clone();
                row["sessionId"] = task["sessionId"].clone();
                row["status"] = task["status"].clone();
                if let Some(last) = task.get("lastDelivery") {
                    row["lastDelivery"] = last.clone();
                }
                row
            })
            .collect();
        rows.sort_by(|a, b| {
            let key = |row: &Value| (row["scheduledAt"].as_str().unwrap_or_default().to_string(), row["id"].as_str().unwrap_or_default().to_string());
            js_compare(&key(a).0, &key(b).0).then_with(|| js_compare(&key(a).1, &key(b).1))
        });
        rows
    }

    /// A page of saved deliveries.
    pub async fn history(&self, session_id: &str, id: &str, limit: &Value, before: Option<&str>) -> Result<Value, Failure> {
        let limit = domain::safe_integer(limit).filter(|n| (1..=100).contains(n));
        let Some(limit) = limit else {
            return Err(ScheduleError::input("invalid_rule", "Delivery history limit must be a safe integer from 1 through 100.").into());
        };
        let store = self.store.lock().await;
        match store.get(id) {
            Some(task) if task["sessionId"] == session_id => Ok(history_page(task, id, limit as usize, before, self.retention)),
            _ => Ok(json!({ "id": id, "code": "schedule_not_found" })),
        }
    }

    /// Delete one reminder of the session, active or not.
    pub async fn delete(&self, session_id: &str, id: &str) -> Result<Value, Failure> {
        let mut store = self.store.lock().await;
        self.refuse_if_stopping()?;
        match store.get(id) {
            Some(task) if task["sessionId"] == session_id => {}
            _ => return Ok(json!({ "id": id, "deleted": false, "code": "schedule_not_found" })),
        }
        store.delete_many(&[id.to_string()])?;
        drop(store);
        self.changed();
        Ok(json!({ "id": id, "deleted": true }))
    }

    /// Delete every ended reminder.
    pub async fn delete_inactive(&self) -> Result<Value, Failure> {
        let mut store = self.store.lock().await;
        self.refuse_if_stopping()?;
        let ended: Vec<String> = store
            .tasks()
            .filter(|task| task["status"] == "inactive")
            .filter_map(|task| task["record"]["id"].as_str().map(str::to_string))
            .collect();
        let deleted = store.delete_many(&ended);
        // A failed batch may have landed in part; observers re-read either way.
        self.changed();
        Ok(json!({ "deleted": deleted? }))
    }

    /// Update one reminder in place. The reply is the update result as the wire carries it.
    pub async fn update(
        &self,
        session_id: &str,
        id: &str,
        expected: &Value,
        change: Option<&Value>,
        title: Option<&str>,
        prompt: Option<&str>,
    ) -> Result<Value, Failure> {
        let mut store = self.store.lock().await;
        self.refuse_if_stopping()?;
        if self.is_subagent(session_id) {
            return Ok(json!({
                "code": "subagent_session",
                "message": "This Session belongs to subagent routing, which never receives reminder delivery.",
            }));
        }
        let Some(current) = store.get(id).filter(|task| task["sessionId"] == session_id).cloned() else {
            return Ok(json!({ "id": id, "updated": false, "code": "schedule_not_found" }));
        };
        if current["status"] == "inactive" {
            return Ok(json!({ "id": id, "updated": false, "code": "schedule_ended" }));
        }
        match update::resolve(&current["record"], expected, change, self.now(), title, prompt) {
            Resolved::Invalid(error) => Ok(json!({ "code": error.code(), "message": error.message() })),
            Resolved::Conflict => Ok(json!({ "id": id, "updated": false, "code": "schedule_conflict" })),
            Resolved::Record { updated, record } => {
                if !updated {
                    return Ok(json!({ "id": id, "updated": false, "record": record }));
                }
                let mut next = current;
                next["record"] = record.clone();
                store.put(next)?;
                drop(store);
                self.changed();
                Ok(json!({ "id": id, "updated": true, "record": record }))
            }
        }
    }

    /// The `workspace/session-activity` entry for a session: its armed reminders,
    /// or nothing when it has none.
    pub async fn activity_of(&self, session_id: &str) -> Option<Value> {
        let active = self.list(session_id).await;
        if active.is_empty() {
            return None;
        }
        let items: Vec<Value> = active.iter().map(|record| json!({ "id": record["id"], "label": record["title"] })).collect();
        Some(json!({ "kind": "schedule", "items": items }))
    }

    /// Remove a session's active reminders (the session is being archived).
    pub async fn stop_session(&self, session_id: &str) -> Result<usize, Failure> {
        let mut store = self.store.lock().await;
        let active: Vec<String> = store
            .tasks()
            .filter(|task| task["sessionId"] == session_id && task["status"] == "active")
            .filter_map(|task| task["record"]["id"].as_str().map(str::to_string))
            .collect();
        if active.is_empty() {
            return Ok(0);
        }
        let removed = store.delete_many(&active)?;
        drop(store);
        self.changed();
        Ok(removed)
    }

    // --- delivery --------------------------------------------------------

    /// Start the delivery driver in `ctx`. It stops when `stop` is called.
    pub fn start(self: &Arc<Self>, ctx: Context) {
        *self.ctx.lock().unwrap() = Some(ctx);
        let service = Arc::clone(self);
        tokio::spawn(async move { service.drive_forever().await });
    }

    /// Stop the driver and refuse further operations.
    pub fn stop(&self) {
        self.stopping.store(true, Ordering::SeqCst);
        self.wake.notify_one();
    }

    async fn drive_forever(self: Arc<Self>) {
        let mut retry_at: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        while !self.stopping.load(Ordering::SeqCst) {
            let next = self.drive_once(&mut retry_at).await;
            let sleep = match next {
                Some(at) => {
                    let wait = (at - self.now()).max(0) as u64;
                    Duration::from_millis(wait.min(MAX_TIMER_MS))
                }
                None => Duration::from_millis(MAX_TIMER_MS),
            };
            tokio::select! {
                _ = tokio::time::sleep(sleep) => {}
                _ = self.wake.notified() => {}
            }
        }
    }

    /// Deliver everything due now and return the earliest instant that is next due.
    pub async fn drive_once(&self, retry_at: &mut std::collections::HashMap<String, i64>) -> Option<i64> {
        let mut store = self.store.lock().await;
        if self.stopping.load(Ordering::SeqCst) {
            return None;
        }
        let ctx = self.ctx.lock().unwrap().clone();
        let scan_now = self.now();
        let due: Vec<Value> = store
            .tasks()
            .filter(|task| task["status"] == "active")
            .filter(|task| domain::parse_scheduled_ms(&task["record"]).is_ok_and(|at| at <= scan_now))
            .filter(|task| retry_at.get(task["record"]["id"].as_str().unwrap_or_default()).is_none_or(|at| *at <= scan_now))
            .cloned()
            .collect();
        let mut handled = std::collections::HashSet::new();
        for task in &due {
            let id = task["record"]["id"].as_str().unwrap_or_default().to_string();
            if handled.contains(&id) {
                continue;
            }
            let session = task["sessionId"].as_str().unwrap_or_default().to_string();
            // Recurring reminders of one session due together are one message.
            let group: Vec<&Value> = if domain::is_recurring(&task["record"]) {
                due.iter().filter(|c| c["sessionId"] == session.as_str() && domain::is_recurring(&c["record"])).collect()
            } else {
                vec![task]
            };
            for member in &group {
                handled.insert(member["record"]["id"].as_str().unwrap_or_default().to_string());
            }
            let Some(ctx) = &ctx else { continue };
            let delivered = self.deliver(ctx, &mut store, &session, &group, task).await;
            if let Err(reason) = delivered {
                ctx.logger().log_event(
                    cordis::logger::LogLevel::Warn,
                    "schedule".to_string(),
                    None,
                    format!("schedule: reminders {} were not acknowledged: {reason}", json!(group.iter().map(|m| m["record"]["id"].clone()).collect::<Vec<_>>())),
                );
                let retry = self.now() + RETRY_AFTER.as_millis() as i64;
                for member in &group {
                    retry_at.insert(member["record"]["id"].as_str().unwrap_or_default().to_string(), retry);
                }
            }
        }
        retry_at.retain(|id, _| store.get(id).is_some());
        let mut next: Option<i64> = None;
        for task in store.tasks().filter(|task| task["status"] == "active") {
            let id = task["record"]["id"].as_str().unwrap_or_default();
            let Ok(at) = domain::parse_scheduled_ms(&task["record"]) else { continue };
            let at = at.max(retry_at.get(id).copied().unwrap_or(i64::MIN));
            next = Some(next.map_or(at, |best| best.min(at)));
        }
        next
    }

    /// Deliver one due reminder or one batch of recurring ones.
    async fn deliver(&self, ctx: &Context, store: &mut Store, session: &str, group: &[&Value], lead: &Value) -> Result<(), String> {
        let agent = resolve_agent(ctx, session)?;
        let now = self.now();
        let admitted: Vec<&&Value> = group.iter().filter(|m| domain::parse_scheduled_ms(&m["record"]).is_ok_and(|at| at <= now)).collect();
        if admitted.is_empty() {
            return Ok(());
        }
        let recurring: Vec<(&Value, domain::Occurrence)> = admitted
            .iter()
            .filter(|m| domain::is_recurring(&m["record"]))
            .map(|m| domain::resolve_recurring(&m["record"], now).map(|o| (&m["record"], o)).map_err(|e| e.message().to_string()))
            .collect::<Result<_, _>>()?;
        let text = if domain::is_recurring(&lead["record"]) {
            let batch: Vec<(&Value, &str)> = recurring.iter().map(|(r, o)| (*r, o.occurrence_at.as_str())).collect();
            domain::render_recurring_batch(&batch)
        } else {
            domain::render_reminder(&lead["record"])
        };
        let message_id = format!("msg-{}", crate::plugins::unique_id());
        let message = Message {
            id: message_id.clone(),
            role: Role::User,
            content: vec![ContentBlock::text(text)],
            source: MessageSource::Plugin { plugin: "schedule".into(), form: None },
        };
        agent.followup(message);
        let sessions = ctx
            .get::<crate::api::services::SessionService>(crate::api::services::SESSIONS_SERVICE)
            .ok_or_else(|| "the sessions service is missing".to_string())?;
        sessions.flush(session).await.map_err(|e| format!("Session persistence did not acknowledge the reminder: {e}"))?;
        let delivered_at = instant::format_instant(self.now()).ok_or_else(|| "the clock is out of range".to_string())?;
        if !domain::is_recurring(&lead["record"]) {
            let receipt = json!({ "scheduledAt": lead["record"]["scheduledAt"], "deliveredAt": delivered_at, "messageId": message_id });
            let mut next = append_delivery(lead, &receipt, self.retention);
            next["status"] = json!("inactive");
            store.put(next).map_err(|e| e.0)?;
        }
        for (record, occurrence) in &recurring {
            let id = record["id"].as_str().unwrap_or_default();
            let Some(task) = store.get(id).cloned() else { continue };
            let receipt = json!({ "scheduledAt": occurrence.occurrence_at, "deliveredAt": delivered_at, "messageId": message_id });
            let mut next = append_delivery(&task, &receipt, self.retention);
            match &occurrence.next_scheduled_at {
                Some(at) => {
                    next["record"]["scheduledAt"] = json!(at);
                    next["status"] = json!("active");
                }
                None => {
                    next["record"]["scheduledAt"] = json!(occurrence.occurrence_at);
                    next["status"] = json!("inactive");
                }
            }
            store.put(next).map_err(|e| e.0)?;
        }
        self.changed.notify_waiters();
        Ok(())
    }
}

/// The agent for `session`: live, or resumed from its stored log.
fn resolve_agent(ctx: &Context, session: &str) -> Result<Arc<dyn AgentView>, String> {
    let agents = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .ok_or_else(|| "schedule: no Session activation provider".to_string())?;
    if let Some(agent) = agents.get(session) {
        return Ok(agent);
    }
    match crate::session::resume::resume_one(ctx, session)? {
        Some(agent) => Ok(agent),
        None => Err(format!("session {session:?} is not live and has no stored log")),
    }
}

/// A version-4 UUID.
fn uuid_v4() -> String {
    crate::gui::plugin::uuid_v4()
}

/// JavaScript's `localeCompare` for the ASCII ids and ISO times it orders, which is byte order.
fn js_compare(a: &str, b: &str) -> std::cmp::Ordering {
    a.cmp(b)
}

/// Whether text is empty after JavaScript trimming.
pub fn is_blank(text: &str) -> bool {
    js_trim(text).is_empty()
}
