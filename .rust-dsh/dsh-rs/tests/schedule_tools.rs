//! The `schedule_*` tools: names and schemas as TypeScript registers them, and
//! each outcome a model can get.

use std::sync::Arc;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, ToolsService};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::schedule::service::ScheduleService;
use dsh_rs::schedule::store::{Retention, Store};
use dsh_rs::types::{AgentOptions, CancelToken, ToolRunContext};
use serde_json::{json, Value};

struct Rig {
    ctx: Context,
    service: Arc<ScheduleService>,
    clock: Arc<std::sync::atomic::AtomicI64>,
}

async fn rig() -> Rig {
    let ctx = Context::new();
    // The tools view a reminder against the wall clock, as the TypeScript tools do.
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    let clock = Arc::new(std::sync::atomic::AtomicI64::new(now));
    let service = {
        let clock = Arc::clone(&clock);
        ScheduleService::with_clock(Store::in_memory(), Retention::default(), Box::new(move || clock.load(std::sync::atomic::Ordering::SeqCst)))
    };
    // The service is provided from a plugin, as the gui plugin provides it.
    let provided = Arc::clone(&service);
    let provider = cordis::plugin::plugin("schedule-provider", move |ctx, _config: Value| {
        let provided = Arc::clone(&provided);
        async move { ctx.provide(dsh_rs::schedule::service::SCHEDULE_SERVICE, provided).await.map(|_| ()) }
    });
    install_profile(&ctx, &json!({ "plugins": { "subagent": {} } })).await.expect("profile installs");
    let _provider = ctx.plugin(provider, Some(json!({})));
    _provider.join().await.unwrap();
    let handles = dsh_rs::bundle::install_native_plugins(&ctx, &json!({ "schedule": {} })).await.expect("schedule installs");
    std::mem::forget(handles);
    std::mem::forget(_provider);
    service.start(ctx.clone());
    Rig { ctx, service, clock }
}

fn agent(rig: &Rig, id: &str) {
    let agents = rig.ctx.require::<AgentRegistryService>("agents").unwrap();
    agents.create(Some(id.into()), AgentOptions::mock("m"), Some("/tmp".into()), None, None).unwrap();
}

/// Run a tool as `agent_id` would and return the JSON value it produced.
async fn run(rig: &Rig, agent_id: &str, tool: &str, args: Value) -> Value {
    let tools = rig.ctx.require::<ToolsService>("tools").unwrap();
    let run = ToolRunContext { ctx: rig.ctx.clone(), signal: CancelToken::new(), agent_id: Some(agent_id.into()), cwd: None };
    let result = tools.execute(format!("call-{}", dsh_rs::plugins::unique_id()), tool.to_string(), args, run).await;
    match result {
        dsh_rs::types::ToolExecutionResult::Success { value, content, .. } => {
            let text: String = content.iter().filter_map(|c| c.as_text()).collect();
            assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), value, "the text is the value");
            value
        }
        other => panic!("{tool} failed: {other:?}"),
    }
}

#[tokio::test]
async fn the_four_tools_carry_the_typescript_names_descriptions_and_schemas() {
    let rig = rig().await;
    let tools = rig.ctx.require::<ToolsService>("tools").unwrap();
    let expected: Value = serde_json::from_str(include_str!("../src/schedule/tool_schemas.json")).unwrap();
    let schemas = tools.schemas();
    for name in ["schedule_create", "schedule_list", "schedule_update", "schedule_delete"] {
        let schema = schemas.iter().find(|s| s.name == name).unwrap_or_else(|| panic!("{name} is registered"));
        assert_eq!(schema.description, expected[name]["description"].as_str().unwrap(), "{name}");
        assert_eq!(schema.parameters, expected[name]["parameters"], "{name}");
    }
    assert!(expected["schedule_create"]["description"].as_str().unwrap().starts_with("Create a reminder in the current session"));
}

#[tokio::test]
async fn a_reminder_is_created_listed_updated_and_deleted_through_the_tools() {
    let rig = rig().await;
    agent(&rig, "main");
    let created = run(&rig, "main", "schedule_create", json!({ "prompt": "  stretch  ", "title": " Stretch ", "after_seconds": 600 })).await;
    assert_eq!(created["kind"], "after");
    assert_eq!(created["prompt"], "stretch", "the prompt is trimmed");
    assert_eq!(created["title"], "Stretch");
    assert_eq!(created["afterSeconds"], 600);
    assert_eq!(created["state"], "scheduled");
    assert_eq!(created["deliveryMode"], "host");
    let due = dsh_rs::schedule::instant::parse_stored_instant(created["scheduledAt"].as_str().unwrap()).unwrap();
    let made_at = rig.clock.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(due, made_at + 600_000, "ten minutes after creation");
    let id = created["id"].as_str().unwrap().to_string();
    assert!(id.starts_with("schedule-"));

    let listed = run(&rig, "main", "schedule_list", json!({})).await;
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["id"], id.as_str());

    let renamed = run(&rig, "main", "schedule_update", json!({ "id": id, "title": "Walk" })).await;
    assert_eq!(renamed["title"], "Walk");
    assert_eq!(renamed["id"], id.as_str(), "an update keeps the id");
    let moved = run(&rig, "main", "schedule_update", json!({ "id": id, "at": "2030-01-01T00:00:00Z" })).await;
    assert_eq!(moved["kind"], "at");
    assert_eq!(moved["scheduledAt"], "2030-01-01T00:00:00.000Z");

    let deleted = run(&rig, "main", "schedule_delete", json!({ "id": id })).await;
    assert_eq!(deleted, json!({ "id": id, "deleted": true }));
    assert_eq!(run(&rig, "main", "schedule_delete", json!({ "id": id })).await, json!({ "id": id, "deleted": false, "code": "schedule_not_found" }));
    assert_eq!(run(&rig, "main", "schedule_list", json!({})).await, json!([]));
}

#[tokio::test]
async fn reminders_belong_to_the_session_that_made_them() {
    let rig = rig().await;
    agent(&rig, "one");
    agent(&rig, "two");
    let made = run(&rig, "one", "schedule_create", json!({ "prompt": "p", "title": "T", "after_seconds": 60 })).await;
    let id = made["id"].as_str().unwrap();
    assert_eq!(run(&rig, "two", "schedule_list", json!({})).await, json!([]));
    assert_eq!(run(&rig, "two", "schedule_delete", json!({ "id": id })).await["code"], "schedule_not_found");
    assert_eq!(run(&rig, "two", "schedule_update", json!({ "id": id, "title": "x" })).await["code"], "schedule_not_found");
}

#[tokio::test]
async fn create_refuses_bad_input_with_the_typescript_codes_and_messages() {
    let rig = rig().await;
    agent(&rig, "main");
    let base = |extra: Value| {
        let mut args = json!({ "prompt": "p", "title": "T" });
        for (key, value) in extra.as_object().unwrap() {
            args[key] = value.clone();
        }
        args
    };
    let cases = [
        (base(json!({})), "invalid_selector", "schedule_create accepts exactly one of after_seconds, at, every_seconds, daily, weekly, or cron."),
        (base(json!({ "after_seconds": 5, "every_seconds": 60 })), "invalid_selector", "schedule_create accepts exactly one of after_seconds, at, every_seconds, daily, weekly, or cron."),
        (base(json!({ "after_seconds": 5, "zzz": 1 })), "invalid_selector", "schedule_create accepts exactly one of after_seconds, at, every_seconds, daily, weekly, or cron."),
        (json!({ "prompt": "  ", "title": "T", "after_seconds": 5 }), "invalid_prompt", "prompt must be non-empty after trimming."),
        (json!({ "prompt": "p", "title": "  ", "after_seconds": 5 }), "invalid_prompt", "title is required and must be non-empty after trimming."),
        (json!({ "prompt": "p", "title": "x".repeat(121), "after_seconds": 5 }), "invalid_prompt", "title must be at most 120 characters."),
        (base(json!({ "after_seconds": 0 })), "invalid_rule", "after_seconds must be a positive safe integer."),
        (base(json!({ "after_seconds": 1.5 })), "invalid_rule", "after_seconds must be a positive safe integer."),
        (base(json!({ "every_seconds": 1.5 })), "invalid_rule", "every_seconds must be a safe integer."),
        (base(json!({ "every_seconds": 59 })), "frequency_too_high", "every_seconds must be at least 60."),
        (base(json!({ "at": "2020-01-01T00:00:00Z" })), "not_future", "The scheduled time must be strictly in the future."),
        (base(json!({ "cron": { "expression": "x", "time_zone": "UTC" } })), "invalid_rule", "cron.expression must contain exactly five whitespace-separated fields: minute hour day-of-month month day-of-week."),
        (base(json!({ "daily": { "time": "09:00:00", "time_zone": "Nope/Zone" } })), "invalid_time_zone", "time_zone must be UTC or a valid IANA Area/Location name."),
    ];
    for (args, code, message) in cases {
        let result = run(&rig, "main", "schedule_create", args.clone()).await;
        assert_eq!(result, json!({ "code": code, "message": message }), "{args}");
    }
    assert_eq!(run(&rig, "main", "schedule_list", json!({})).await, json!([]), "no refusal created anything");
}

#[tokio::test]
async fn update_and_delete_refuse_bad_input_and_say_when_a_reminder_has_ended() {
    let rig = rig().await;
    agent(&rig, "main");
    let one = |args: Value| args;
    let made = run(&rig, "main", "schedule_create", json!({ "prompt": "p", "title": "T", "after_seconds": 60 })).await;
    let id = made["id"].as_str().unwrap().to_string();
    let refused = [
        (one(json!({ "id": id })), "invalid_selector", "schedule_update needs a new title, prompt, or one of at, every_seconds, daily, weekly, or cron."),
        (one(json!({ "id": id, "at": "2031-01-01T00:00:00Z", "every_seconds": 60 })), "invalid_selector", "schedule_update accepts at most one of at, every_seconds, daily, weekly, or cron."),
        (one(json!({ "id": id, "after_seconds": 5 })), "invalid_selector", "schedule_update accepts at most one of at, every_seconds, daily, weekly, or cron."),
        (one(json!({ "id": " x", "title": "y" })), "invalid_rule", "schedule_update id must be non-empty without surrounding whitespace."),
        (one(json!({ "id": "", "title": "y" })), "invalid_rule", "schedule_update id must be non-empty without surrounding whitespace."),
        (one(json!({ "id": id, "title": " " })), "invalid_prompt", "title is required and must be non-empty after trimming."),
        (one(json!({ "id": id, "prompt": " " })), "invalid_prompt", "prompt must be non-empty after trimming."),
        (one(json!({ "id": id, "every_seconds": 30 })), "frequency_too_high", "every_seconds must be at least 60."),
    ];
    for (args, code, message) in refused {
        assert_eq!(run(&rig, "main", "schedule_update", args.clone()).await, json!({ "code": code, "message": message }), "{args}");
    }
    assert_eq!(
        run(&rig, "main", "schedule_delete", json!({ "id": " x" })).await["code"],
        "invalid_rule"
    );
    assert_eq!(
        run(&rig, "main", "schedule_update", json!({ "id": "schedule-nope", "title": "y" })).await,
        json!({ "id": "schedule-nope", "updated": false, "code": "schedule_not_found" })
    );
}

#[tokio::test]
async fn a_delegated_subagent_cannot_use_reminders() {
    let rig = rig().await;
    agent(&rig, "main");
    agent(&rig, "child");
    let made = run(&rig, "main", "schedule_create", json!({ "prompt": "p", "title": "T", "after_seconds": 60 })).await;
    let id = made["id"].as_str().unwrap().to_string();

    let lineage = rig.ctx.get::<dsh_rs::plugins::subagent::Lineage>(dsh_rs::plugins::subagent::LINEAGE_SERVICE).expect("lineage");
    lineage.adopt_for_test("child", "main");

    let refusal = json!({ "code": "subagent_session", "message": "A delegated subagent cannot use reminders." });
    assert_eq!(run(&rig, "child", "schedule_create", json!({ "prompt": "p", "title": "T", "after_seconds": 60 })).await, refusal);
    assert_eq!(run(&rig, "child", "schedule_list", json!({})).await, refusal);
    assert_eq!(run(&rig, "child", "schedule_delete", json!({ "id": id })).await, refusal);
    assert_eq!(run(&rig, "child", "schedule_update", json!({ "id": id, "title": "x" })).await, refusal);
    // The parent keeps its reminder and its access.
    assert_eq!(run(&rig, "main", "schedule_list", json!({})).await.as_array().unwrap().len(), 1);

    // The service refuses a reminder for the child's session too.
    let request = dsh_rs::schedule::service::CreateRequest { prompt: "p".into(), title: "T".into(), after_seconds: Some(json!(60)), ..Default::default() };
    match rig.service.create("child", &request).await {
        Err(dsh_rs::schedule::service::Failure::Input(error)) => assert_eq!(error.code(), "subagent_session"),
        other => panic!("expected a subagent_session refusal, got {other:?}"),
    }
}
