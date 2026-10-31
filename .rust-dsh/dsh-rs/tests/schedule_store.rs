//! The reminder store and delivery history, against the TypeScript
//! implementation (`tests/fixtures/schedule_history_ts.json`) and its file format.

use dsh_rs::schedule::store::{append_delivery, history_page, validate_task, Retention, Store};
use serde_json::{json, Value};

#[test]
fn delivery_history_and_its_pages_match_typescript() {
    let scenarios: Vec<Value> = serde_json::from_str(include_str!("fixtures/schedule_history_ts.json")).unwrap();
    assert_eq!(scenarios.len(), 16);
    let mut wrong = Vec::new();
    let mut checked = 0;
    for scenario in &scenarios {
        let bounds = Retention {
            days: scenario["bounds"]["days"].as_u64().unwrap(),
            records: scenario["bounds"]["records"].as_u64().unwrap() as usize,
        };
        // Rebuild the starting task the generator used.
        let record = json!({ "id": "a1", "kind": "every", "title": "T", "prompt": "p", "everySeconds": 60, "scheduledAt": "2026-03-01T12:00:00.000Z" });
        let receipt0 = json!({ "scheduledAt": "2026-03-01T12:00:00.000Z", "deliveredAt": "2026-03-01T12:00:00.000Z", "messageId": "m0" });
        let mut task = json!({ "sessionId": "s", "record": record, "status": "active" });
        match scenario["start"].as_str().unwrap() {
            "last-only" => task["lastDelivery"] = receipt0.clone(),
            "empty-history" => task["deliveryHistory"] = json!({ "records": [], "earlierRecordsUnavailable": false }),
            "pruned" => {
                task["lastDelivery"] = receipt0.clone();
                let mut entry = receipt0.clone();
                entry["prompt"] = json!("p");
                task["deliveryHistory"] = json!({ "records": [entry], "earlierRecordsUnavailable": true, "earlierRecordsPruned": true });
            }
            _ => {}
        }
        for step in scenario["steps"].as_array().unwrap() {
            task = append_delivery(&task, &step["receipt"], bounds);
            checked += 1;
            if task != step["task"] {
                wrong.push(format!("append {} {}\n   ts: {}\n   rs: {task}", scenario["bounds"], scenario["start"], step["task"]));
                break;
            }
            for page in step["pages"].as_array().unwrap() {
                let limit = page["limit"].as_u64().unwrap() as usize;
                let before = page["before"].as_str();
                let got = history_page(&task, "a1", limit, before, bounds);
                checked += 1;
                if got != page["page"] {
                    wrong.push(format!("page limit={limit} before={before:?}\n   ts: {}\n   rs: {got}", page["page"]));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{} of {checked} differ:\n{}", wrong.len(), wrong.iter().take(6).cloned().collect::<Vec<_>>().join("\n"));
}

fn task(id: &str, session: &str) -> Value {
    json!({
        "sessionId": session,
        "record": { "id": id, "kind": "after", "title": "T", "prompt": "p", "afterSeconds": 60, "scheduledAt": "2030-01-01T00:00:00.000Z" },
        "status": "active",
        "deliveryHistory": { "records": [], "earlierRecordsUnavailable": false },
    })
}

#[test]
fn a_store_writes_the_storage_json_layout_and_reads_it_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("storages").join("schedule.json");
    let mut store = Store::open(&path).unwrap();
    assert_eq!(store.tasks().count(), 0);
    assert!(!path.exists(), "opening an absent store creates nothing");
    store.put(task("t1", "s1")).unwrap();
    store.put(task("t2", "s2")).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    let document: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(document["unit"], json!({ "name": "schedule", "version": 1 }));
    assert_eq!(document["global"], Value::Null);
    assert_eq!(document["tables"]["tasks"]["t1"]["sessionId"], "s1");
    assert!(text.ends_with("}\n"), "pretty JSON with one trailing newline");
    assert!(text.contains("\n  \"unit\": {"), "two-space indentation, as storage-json writes it");

    let reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.tasks().count(), 2);
    assert_eq!(reopened.get("t2"), store.get("t2"));

    let mut store = reopened;
    assert_eq!(store.delete_many(&["t1".into(), "missing".into()]).unwrap(), 1);
    assert_eq!(store.delete_many(&["missing".into()]).unwrap(), 0);
    assert_eq!(Store::open(&path).unwrap().tasks().count(), 1);

    use std::os::unix::fs::PermissionsExt;
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap()).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect();
    assert_eq!(leftovers, vec!["schedule.json"], "no temporary file is left behind");
}

#[test]
fn a_file_that_cannot_be_trusted_is_refused_and_left_alone() {
    let cases = [
        ("not json", "{{{", "not valid JSON"),
        ("foreign unit", r#"{"unit":{"name":"other","version":1},"global":null,"tables":{}}"#, "foreign unit header"),
        ("future version", r#"{"unit":{"name":"schedule","version":2},"global":null,"tables":{}}"#, "stored version 2 is not accepted"),
        ("tables array", r#"{"unit":{"name":"schedule","version":1},"global":null,"tables":[]}"#, "tables is not an object"),
        ("table not object", r#"{"unit":{"name":"schedule","version":1},"global":null,"tables":{"tasks":[]}}"#, "table 'tasks' is not an object"),
        ("key mismatch", r#"{"unit":{"name":"schedule","version":1},"global":null,"tables":{"tasks":{"x":{"sessionId":"s","record":{"id":"y","kind":"after","title":"T","prompt":"p","afterSeconds":60,"scheduledAt":"2030-01-01T00:00:00.000Z"}}}}}"#, "stored task key \"x\" differs from record id \"y\""),
        ("bad record", r#"{"unit":{"name":"schedule","version":1},"global":null,"tables":{"tasks":{"x":{"sessionId":"s","record":{"id":"x"}}}}}"#, "is invalid"),
        ("extra task key", r#"{"unit":{"name":"schedule","version":1},"global":null,"tables":{"tasks":{"x":{"sessionId":"s","extra":1,"record":{"id":"x","kind":"after","title":"T","prompt":"p","afterSeconds":60,"scheduledAt":"2030-01-01T00:00:00.000Z"}}}}}"#, "unexpected key"),
    ];
    for (name, text, needle) in cases {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedule.json");
        std::fs::write(&path, text).unwrap();
        let error = Store::open(&path).expect_err(name);
        assert!(error.to_string().contains(needle), "{name}: {error}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text, "{name}: the file is untouched");
    }
}

#[test]
fn a_task_with_a_history_that_disagrees_with_its_last_delivery_is_invalid() {
    let mut t = task("t1", "s1");
    let receipt = json!({ "scheduledAt": "2030-01-01T00:00:00.000Z", "deliveredAt": "2030-01-01T00:00:01.000Z", "messageId": "m1" });
    t["lastDelivery"] = receipt.clone();
    assert!(validate_task("t1", &t).is_err(), "a last delivery with an empty history");
    let mut entry = receipt.clone();
    entry["prompt"] = json!("p");
    t["deliveryHistory"] = json!({ "records": [entry.clone()], "earlierRecordsUnavailable": false });
    assert!(validate_task("t1", &t).is_ok());
    t["lastDelivery"]["messageId"] = json!("other");
    assert!(validate_task("t1", &t).is_err());
    let mut dup = task("t1", "s1");
    dup["lastDelivery"] = receipt;
    dup["deliveryHistory"] = json!({ "records": [entry.clone(), entry], "earlierRecordsUnavailable": false });
    assert!(validate_task("t1", &dup).is_err(), "message identities must be unique");
}

#[test]
fn tasks_keep_their_place_in_the_file_across_writes_and_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("schedule.json");
    let mut store = Store::open(&path).unwrap();
    // Ids that are neither alphabetical nor in creation order.
    for id in ["zeta", "alpha", "mid", "beta"] {
        store.put(task(id, "s")).unwrap();
    }
    let order = |store: &Store| store.tasks().map(|t| t["record"]["id"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    assert_eq!(order(&store), ["zeta", "alpha", "mid", "beta"]);

    // Replacing a task keeps its place.
    let mut changed = task("alpha", "s");
    changed["record"]["title"] = json!("Changed");
    store.put(changed).unwrap();
    assert_eq!(order(&store), ["zeta", "alpha", "mid", "beta"]);

    // The file lists them in that order, and a restart reads them back in it.
    let text = std::fs::read_to_string(&path).unwrap();
    let at = |id: &str| text.find(&format!("\n      \"{id}\": {{")).unwrap();
    assert!(at("zeta") < at("alpha") && at("alpha") < at("mid") && at("mid") < at("beta"));
    assert_eq!(order(&Store::open(&path).unwrap()), ["zeta", "alpha", "mid", "beta"]);

    // Deleting from the middle leaves the rest in order.
    store.delete_many(&["mid".into()]).unwrap();
    assert_eq!(order(&Store::open(&path).unwrap()), ["zeta", "alpha", "beta"]);
}
