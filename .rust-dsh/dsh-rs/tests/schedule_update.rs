//! Update resolution and stored-record decoding against the TypeScript
//! implementation's own answers (`tests/fixtures/schedule_update_ts.json`).

use dsh_rs::schedule::domain;
use dsh_rs::schedule::update::{resolve, Resolved};
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/schedule_update_ts.json")).unwrap()
}

#[test]
fn every_update_resolves_as_typescript_resolves_it() {
    let fixture = fixture();
    let cases = fixture["cases"].as_array().unwrap();
    assert!(cases.len() > 1000);
    let mut wrong = Vec::new();
    for case in cases {
        // JSON drops `undefined`, so a key that is absent means no change, and a
        // present `null` is a real (invalid) change.
        let change = case.get("change");
        let content = &case["content"];
        let got = resolve(
            &case["current"],
            &case["expected"],
            change,
            case["now"].as_i64().unwrap(),
            content.get("title").and_then(Value::as_str),
            content.get("prompt").and_then(Value::as_str),
        );
        let expected = &case["result"];
        let same = match (&got, expected.get("ok")) {
            (Resolved::Record { updated, record }, Some(ok)) => ok["updated"] == *updated && ok["record"] == *record,
            (Resolved::Conflict, Some(ok)) => ok["code"] == "schedule_conflict" && ok["updated"] == false,
            (Resolved::Invalid(error), Some(ok)) => ok["code"] == error.code() && ok["message"] == error.message(),
            _ => false,
        };
        if !same {
            wrong.push(format!(
                "{} change={} content={}\n   ts: {expected}\n   rs: {got:?}",
                case["kind"], case["change"], case["content"]
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {} differ:\n{}",
        wrong.len(),
        cases.len(),
        wrong.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn stored_records_are_accepted_and_refused_as_typescript_does() {
    let fixture = fixture();
    let decoded = fixture["decoded"].as_array().unwrap();
    assert!(decoded.len() > 200);
    let mut wrong = Vec::new();
    for entry in decoded {
        let got = domain::decode_record(&entry["value"]);
        let expected = &entry["result"];
        let same = match (&got, expected.get("ok")) {
            (Ok(()), Some(_)) => true,
            (Err(error), None) => expected["code"] == error.code() && expected["message"] == error.message(),
            _ => false,
        };
        if !same {
            wrong.push(format!("{}\n   ts: {expected}\n   rs: {got:?}", entry["value"]));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {} differ:\n{}",
        wrong.len(),
        decoded.len(),
        wrong.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
    );
    let _ = json!(null);
}
