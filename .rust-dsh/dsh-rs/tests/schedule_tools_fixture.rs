//! The `schedule_*` tools against the real TypeScript tool bodies.
//!
//! `tests/fixtures/schedule_tools_ts.json` was produced by
//! `tests/oracle/schedule_tools_ts.mjs`, which runs `@deepseek-ai/dsh-tool-schedule`
//! over the real `@deepseek-ai/dsh-schedule` domain with the clock fixed at
//! 2026-03-01T12:00:00.000Z. The same calls, in the same order, run here on the
//! Rust tools with the same clock; every result must be identical, apart from
//! the reminder ids (which each host mints for itself).

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, ToolsService};
use dsh_rs::bundle::install_profile;
use dsh_rs::schedule::service::ScheduleService;
use dsh_rs::schedule::store::{Retention, Store};
use dsh_rs::types::{AgentOptions, CancelToken, ToolExecutionResult, ToolRunContext};
use serde_json::{json, Value};

const NOW: i64 = 1_772_366_400_000; // 2026-03-01T12:00:00.000Z

async fn rig() -> (Context, Arc<ScheduleService>) {
    let ctx = Context::new();
    let clock = Arc::new(AtomicI64::new(NOW));
    let service = {
        let clock = Arc::clone(&clock);
        ScheduleService::with_clock(Store::in_memory(), Retention::default(), Box::new(move || clock.load(Ordering::SeqCst)))
    };
    let provided = Arc::clone(&service);
    let provider = cordis::plugin::plugin("schedule-provider", move |ctx, _config: Value| {
        let provided = Arc::clone(&provided);
        async move { ctx.provide(dsh_rs::schedule::service::SCHEDULE_SERVICE, provided).await }
    });
    install_profile(&ctx, &json!({ "plugins": { "subagent": {} } })).await.unwrap();
    let held = ctx.plugin(provider, Some(json!({})));
    held.join().await.unwrap();
    std::mem::forget(held);
    std::mem::forget(dsh_rs::bundle::install_native_plugins(&ctx, &json!({ "schedule": {} })).await.unwrap());
    service.start(ctx.clone());
    (ctx, service)
}

/// Replace each reminder id with a stable token by order of first appearance.
fn mask(value: &Value, ids: &mut Vec<String>) -> Value {
    match value {
        Value::String(text) if text.starts_with("schedule-") && text != "schedule-none" => {
            let at = ids.iter().position(|seen| seen == text).unwrap_or_else(|| {
                ids.push(text.clone());
                ids.len() - 1
            });
            json!(format!("<id{at}>"))
        }
        Value::Array(items) => Value::Array(items.iter().map(|v| mask(v, ids)).collect()),
        Value::Object(map) => Value::Object(map.iter().map(|(k, v)| (k.clone(), mask(v, ids))).collect()),
        other => other.clone(),
    }
}

#[tokio::test]
async fn every_tool_outcome_matches_the_typescript_tools() {
    let battery: Vec<Value> = serde_json::from_str(include_str!("fixtures/schedule_tools_ts.json")).unwrap();
    assert!(battery.len() >= 60);
    let (ctx, _service) = rig().await;
    let agents = ctx.require::<AgentRegistryService>("agents").unwrap();
    agents.create(Some("s1".into()), AgentOptions::mock("m"), Some("/tmp".into()), None, None).unwrap();
    agents.create(Some("c1".into()), AgentOptions::mock("m"), Some("/tmp".into()), None, None).unwrap();
    ctx.get::<dsh_rs::plugins::subagent::Lineage>(dsh_rs::plugins::subagent::LINEAGE_SERVICE).unwrap().adopt_for_test("c1", "s1");
    let tools = ctx.require::<ToolsService>("tools").unwrap();

    let (mut ts_ids, mut rs_ids) = (Vec::new(), Vec::new());
    let mut first_id: Option<String> = None;
    let mut wrong = Vec::new();
    for step in &battery {
        let tool = step["tool"].as_str().unwrap();
        let agent = if step["subagent"] == true { "c1" } else { "s1" };
        // The TypeScript battery named the first reminder `schedule-1`; here it has a minted id.
        let mut args = step["args"].clone();
        let text = args.to_string();
        if text.contains("schedule-1\"") {
            if let Some(real) = &first_id {
                args = serde_json::from_str(&text.replace("schedule-1\"", &format!("{real}\""))).unwrap();
            }
        }
        let run = ToolRunContext { ctx: ctx.clone(), signal: CancelToken::new(), agent_id: Some(agent.into()), cwd: None };
        let result = tools.execute(format!("call-{}", dsh_rs::plugins::unique_id()), tool.to_string(), args, run).await;
        let got = match result {
            ToolExecutionResult::Success { value, .. } => value,
            ToolExecutionResult::Error { code, message, .. } => json!({ "threw": format!("{code}: {message}") }),
        };
        if first_id.is_none() && step["label"].as_str().unwrap().starts_with("create {\"after_seconds\":600}") {
            first_id = got["id"].as_str().map(str::to_string);
        }
        let expected = &step["result"];
        let (g, e) = (mask(&got, &mut rs_ids), mask(expected, &mut ts_ids));
        // A schema violation is a thrown `ToolArgsError` in TypeScript and an `INVALID_ARGS`
        // error result here; both carry the same violation list.
        let same = match (e.get("threw").and_then(Value::as_str), g.get("threw").and_then(Value::as_str)) {
            (Some(ts), Some(rs)) => ts.strip_prefix("ToolArgsError: ") == rs.strip_prefix("INVALID_ARGS: "),
            _ => g == e,
        };
        if !same {
            wrong.push(format!("{}\n   ts: {e}\n   rs: {g}", step["label"]));
        }
    }
    assert!(wrong.is_empty(), "{} of {} differ:\n{}", wrong.len(), battery.len(), wrong.iter().take(8).cloned().collect::<Vec<_>>().join("\n"));
}
