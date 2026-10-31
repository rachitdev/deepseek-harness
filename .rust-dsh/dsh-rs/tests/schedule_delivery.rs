//! Delivery: a due reminder reaches its session as a user message, whether or not
//! the session's agent is live, and the task's state follows.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use cordis::Context;
use dsh_rs::api::services::AgentRegistryService;
use dsh_rs::bundle::install_profile;
use dsh_rs::schedule::service::{CreateRequest, ScheduleService};
use dsh_rs::schedule::store::{Retention, Store};
use dsh_rs::types::{AgentOptions, MessageSource, SessionEventData};
use serde_json::{json, Value};

const START: i64 = 1_793_000_000_000;

struct Rig {
    ctx: Context,
    service: Arc<ScheduleService>,
    clock: Arc<AtomicI64>,
}

async fn rig(store: Store) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} })).await.expect("profile installs");
    let clock = Arc::new(AtomicI64::new(START));
    let service = {
        let clock = Arc::clone(&clock);
        ScheduleService::with_clock(store, Retention::default(), Box::new(move || clock.load(Ordering::SeqCst)))
    };
    service.start(ctx.clone());
    Rig { ctx, service, clock }
}

fn agent(rig: &Rig, id: &str) -> Arc<dyn dsh_rs::api::services::AgentView> {
    let agents = rig.ctx.require::<AgentRegistryService>("agents").unwrap();
    agents.create(Some(id.into()), AgentOptions::mock("m"), Some("/tmp".into()), None, None).unwrap()
}

fn after(seconds: i64) -> CreateRequest {
    CreateRequest { prompt: "drink water".into(), title: "Water".into(), after_seconds: Some(json!(seconds)), ..Default::default() }
}

/// Deliver everything due now, as the driver's timer would.
async fn tick(rig: &Rig) {
    rig.service.drive_once(&mut Default::default()).await;
}

fn reminder_texts(agent: &dyn dsh_rs::api::services::AgentView) -> Vec<String> {
    agent
        .session()
        .events()
        .iter()
        .filter_map(|e| match &e.data {
            SessionEventData::InboxSpliced { inserted, .. } => Some(inserted.clone()),
            _ => None,
        })
        .flatten()
        .filter(|m| matches!(&m.source, MessageSource::Plugin { plugin, .. } if plugin == "schedule"))
        .map(|m| m.text())
        .collect()
}

#[tokio::test]
async fn a_due_one_shot_reminder_is_delivered_once_and_then_ends() {
    let rig = rig(Store::in_memory()).await;
    let main = agent(&rig, "main");
    let record = rig.service.create("main", &after(60)).await.unwrap();
    tick(&rig).await;
    assert!(reminder_texts(&*main).is_empty(), "not due yet");

    rig.clock.fetch_add(61_000, Ordering::SeqCst);
    tick(&rig).await;
    let texts = reminder_texts(&*main);
    assert_eq!(texts.len(), 1);
    assert_eq!(
        texts[0],
        format!(
            "[SCHEDULE REMINDER]\nThis is a scheduled message from the user\nschedule_id_json: \"{}\"\noccurrence_at: {}\nreminder_prompt_json: \"drink water\"",
            record["id"].as_str().unwrap(),
            record["scheduledAt"].as_str().unwrap()
        )
    );
    tick(&rig).await;
    assert_eq!(reminder_texts(&*main).len(), 1, "a delivered reminder is not delivered again");

    assert_eq!(rig.service.list("main").await, Vec::<Value>::new(), "it is no longer active");
    let catalog = rig.service.catalog().await;
    assert_eq!(catalog.len(), 1);
    assert_eq!(catalog[0]["status"], "inactive");
    assert_eq!(catalog[0]["lastDelivery"]["messageId"].as_str().is_some(), true);
    let history = rig.service.history("main", record["id"].as_str().unwrap(), &json!(10), None).await.unwrap();
    assert_eq!(history["records"].as_array().unwrap().len(), 1);
    assert_eq!(history["records"][0]["prompt"], "drink water");
    assert_eq!(rig.service.delete_inactive().await.unwrap(), json!({ "deleted": 1 }));
}

#[tokio::test]
async fn a_recurring_reminder_advances_to_its_next_occurrence() {
    let rig = rig(Store::in_memory()).await;
    let main = agent(&rig, "main");
    let request = CreateRequest { prompt: "tick".into(), title: "Tick".into(), every_seconds: Some(json!(60)), ..Default::default() };
    let record = rig.service.create("main", &request).await.unwrap();
    let first = record["scheduledAt"].as_str().unwrap().to_string();

    rig.clock.fetch_add(61_000, Ordering::SeqCst);
    tick(&rig).await;
    let texts = reminder_texts(&*main);
    assert_eq!(texts.len(), 1);
    assert!(texts[0].starts_with("[SCHEDULE REMINDER BATCH]\nThis is a scheduled message from the user\nreminders_json: "), "{}", texts[0]);
    assert!(texts[0].contains(&format!("\"occurrence_at\":\"{first}\"")), "{}", texts[0]);

    let listed = rig.service.list("main").await;
    assert_eq!(listed.len(), 1, "a recurring reminder stays active");
    assert_ne!(listed[0]["scheduledAt"].as_str().unwrap(), first, "and moves to its next occurrence");
}

#[tokio::test]
async fn after_downtime_a_recurring_reminder_delivers_only_its_latest_missed_occurrence() {
    let rig = rig(Store::in_memory()).await;
    let main = agent(&rig, "main");
    let request = CreateRequest { prompt: "tick".into(), title: "Tick".into(), every_seconds: Some(json!(60)), ..Default::default() };
    rig.service.create("main", &request).await.unwrap();
    rig.clock.fetch_add(10 * 60_000 + 5_000, Ordering::SeqCst);
    tick(&rig).await;
    assert_eq!(reminder_texts(&*main).len(), 1, "ten missed minutes are one message");
    assert!(reminder_texts(&*main)[0].contains("T"), "carries the latest occurrence");
}

#[tokio::test]
async fn recurring_reminders_due_together_in_one_session_are_one_message() {
    let rig = rig(Store::in_memory()).await;
    let main = agent(&rig, "main");
    for (prompt, seconds) in [("one", 60), ("two", 120)] {
        let request = CreateRequest { prompt: prompt.into(), title: prompt.into(), every_seconds: Some(json!(seconds)), ..Default::default() };
        rig.service.create("main", &request).await.unwrap();
    }
    rig.clock.fetch_add(130_000, Ordering::SeqCst);
    tick(&rig).await;
    let texts = reminder_texts(&*main);
    assert_eq!(texts.len(), 1);
    assert!(texts[0].contains("\"reminder_prompt\":\"one\"") && texts[0].contains("\"reminder_prompt\":\"two\""), "{}", texts[0]);
}

#[tokio::test]
async fn a_reminder_for_a_session_that_cannot_be_reached_is_kept_and_retried() {
    let rig = rig(Store::in_memory()).await;
    rig.service.create("ghost", &after(60)).await.unwrap();
    rig.clock.fetch_add(61_000, Ordering::SeqCst);
    let mut retries = Default::default();
    rig.service.drive_once(&mut retries).await;
    assert_eq!(rig.service.list("ghost").await.len(), 1, "an undeliverable reminder stays active");
    assert_eq!(retries.len(), 1, "and waits before it is tried again");
}

#[tokio::test]
async fn a_delivery_survives_in_the_session_log_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let rig = rig(Store::open(&dir.path().join("schedule.json")).unwrap()).await;
    let main = agent(&rig, "main");
    rig.service.create("main", &after(60)).await.unwrap();
    rig.clock.fetch_add(61_000, Ordering::SeqCst);
    tick(&rig).await;
    assert_eq!(reminder_texts(&*main).len(), 1);
    // The task's end is on disk too.
    let reopened = Store::open(&dir.path().join("schedule.json")).unwrap();
    let task = reopened.tasks().next().unwrap();
    assert_eq!(task["status"], "inactive");
    assert_eq!(task["deliveryHistory"]["records"].as_array().unwrap().len(), 1);
}
