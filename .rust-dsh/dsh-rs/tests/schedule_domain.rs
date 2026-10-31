//! The schedule domain against the TypeScript implementation's own answers.
//!
//! `tests/fixtures/schedule_domain_ts.json` was produced by running
//! `@deepseek-ai/dsh-schedule`'s `create*ScheduleRecord` and
//! `resolveRecurringOccurrence` over a grid of zones (including DST gaps and
//! overlaps, half-hour and 45-minute offsets and a 30-minute DST step), times,
//! creation instants and malformed inputs. Every case must give the same record
//! or the same error code and message, and every occurrence the same instants.

use dsh_rs::schedule::domain;
use serde_json::Value;

fn run(kind: &str, input: &Value, now: i64) -> Result<Value, dsh_rs::schedule::ScheduleError> {
    match kind {
        "daily" => domain::create_daily("ID", "p", input, now, "t"),
        "weekly" => domain::create_weekly("ID", "p", input, now, "t"),
        "cron" => domain::create_cron("ID", "p", input, now, "t"),
        "every" => domain::create_every("ID", "p", input, now, "t"),
        "at" => domain::create_at("ID", "p", input, now, "t"),
        "after" => domain::create_after("ID", "p", input, now, "t"),
        other => panic!("unknown kind {other}"),
    }
}

#[test]
fn creation_and_occurrences_match_typescript() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/schedule_domain_ts.json")).unwrap();
    assert!(cases.len() > 2000);
    let (mut created, mut occurrences) = (0usize, 0usize);
    let mut wrong: Vec<String> = Vec::new();
    for case in &cases {
        let kind = case["kind"].as_str().unwrap();
        let now = case["now"].as_i64().unwrap();
        let expected = &case["created"];
        let got = run(kind, &case["input"], now);
        created += 1;
        let same = match (&got, expected.get("ok")) {
            (Ok(record), Some(ok)) => record == ok,
            (Err(e), None) => expected["code"] == e.code() && expected["message"] == e.message(),
            _ => false,
        };
        if !same {
            wrong.push(format!("create {kind} {} @{now}\n   ts: {expected}\n   rs: {got:?}", case["input"]));
            continue;
        }
        let (Ok(record), Some(list)) = (got, case["occurrences"].as_array()) else { continue };
        for occurrence in list {
            let accepted = occurrence["accepted"].as_i64().unwrap();
            occurrences += 1;
            let result = domain::resolve_recurring(&record, accepted);
            let expected = &occurrence["result"];
            let same = match (&result, expected.get("ok")) {
                (Ok(o), Some(ok)) => {
                    ok["occurrenceAt"] == o.occurrence_at.as_str()
                        && ok.get("nextScheduledAt").and_then(Value::as_str) == o.next_scheduled_at.as_deref()
                }
                (Err(e), None) => expected["message"] == e.message(),
                _ => false,
            };
            if !same {
                wrong.push(format!("resolve {kind} {record} @{accepted}\n   ts: {expected}\n   rs: {result:?}"));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {} creations and {} occurrences differ:\n{}",
        wrong.len(),
        created,
        occurrences,
        wrong.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
    assert!(occurrences > 15_000, "{occurrences}");
}
