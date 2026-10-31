//! Writes a schedule store with the Rust writer for the TypeScript reader to open.
//! Run by `tests/oracle/schedule_interop.sh`; skipped unless `SCHEDULE_INTEROP_OUT` is set.

use dsh_rs::schedule::domain;
use dsh_rs::schedule::store::{append_delivery, Retention, Store};
use serde_json::json;

#[test]
fn write_a_store_for_the_typescript_reader() {
    let Some(out) = std::env::var_os("SCHEDULE_INTEROP_OUT") else { return };
    let mut store = Store::open(std::path::Path::new(&out)).unwrap();
    let now = domain::parse_scheduled_ms(&json!({ "scheduledAt": "2026-03-01T12:00:00.000Z" })).unwrap();
    let records = [
        domain::create_after("schedule-a", "ping", &json!(600), now, "After").unwrap(),
        domain::create_at("schedule-b", "at", &json!("2030-05-01T09:00:00Z"), now, "At").unwrap(),
        domain::create_every("schedule-c", "tick", &json!(300), now, "Every").unwrap(),
        domain::create_daily("schedule-d", "daily", &json!({"time":"09:00:00","time_zone":"Asia/Kolkata"}), now, "Daily").unwrap(),
        domain::create_weekly("schedule-e", "weekly", &json!({"time":"09:00:00","time_zone":"America/New_York","weekdays":[3,1]}), now, "Weekly").unwrap(),
        domain::create_cron("schedule-f", "cron", &json!({"expression":"0,15,30,45 9-17 * * 1-5","time_zone":"Europe/London"}), now, "Cron").unwrap(),
    ];
    for record in records {
        let mut task = json!({ "sessionId": "session-1", "record": record, "status": "active", "deliveryHistory": { "records": [], "earlierRecordsUnavailable": false } });
        if task["record"]["id"] == "schedule-c" {
            let receipt = json!({ "scheduledAt": "2026-03-01T12:05:00.000Z", "deliveredAt": "2026-03-01T12:05:01.000Z", "messageId": "m-1" });
            task = append_delivery(&task, &receipt, Retention::default());
        }
        store.put(task).unwrap();
    }
}
