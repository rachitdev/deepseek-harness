//! The `schedule` plugin: reminders for the model.
//!
//! It registers `schedule_create`, `schedule_list`, `schedule_update` and
//! `schedule_delete`, which call the same [`ScheduleService`] as the
//! `schedule/*` endpoints. Names, descriptions and parameter schemas are the
//! TypeScript package's own (`src/schedule/tool_schemas.json`, extracted from
//! `@deepseek-ai/dsh-tool-schedule`), so a model sees the same tools in either
//! host. Every result is the JSON the TypeScript tool returns: a reminder view,
//! a list of views, a delete/update outcome, or `{code, message}`.
//!
//! A delegated sub-agent never uses reminders: its calls answer
//! `{code: "subagent_session"}`.

use std::sync::Arc;

use cordis::Plugin;
use serde_json::{json, Value};

use crate::plugins::{register_tools, tool_spec_ctx};
use crate::schedule::domain::{self, js_trim, MAX_TITLE_LENGTH, MIN_EVERY_SECONDS, REQUIRED_TITLE_MESSAGE};
use crate::schedule::service::{CreateRequest, Failure, ScheduleService, SCHEDULE_SERVICE};
use crate::types::ToolExecutionResult;

const SCHEMAS: &str = include_str!("../schedule/tool_schemas.json");

fn schemas() -> Value {
    serde_json::from_str(SCHEMAS).expect("tool_schemas.json is JSON")
}

fn error(code: &str, message: impl Into<String>) -> Value {
    json!({ "code": code, "message": message.into() })
}

fn internal() -> Value {
    error("internal_error", "The schedule operation failed.")
}

/// The tool result for a value. The TypeScript tool returns every outcome,
/// failures included, as an ordinary JSON value rendered as text, so the model
/// reads a refusal the same way it reads a reminder.
fn rendered(value: Value) -> ToolExecutionResult {
    ToolExecutionResult::success_text(value.to_string(), value)
}

fn failure_value(failure: Failure) -> Value {
    match failure {
        Failure::Input(error) => json!({ "code": error.code(), "message": error.message() }),
        Failure::Service(_) => internal(),
    }
}

/// Whether the calling agent is a delegated sub-agent.
fn is_subagent(ctx: &cordis::Context, agent: &str) -> bool {
    ctx.get::<crate::plugins::subagent::Lineage>(crate::plugins::subagent::LINEAGE_SERVICE)
        .is_some_and(|lineage| lineage.is_child(agent))
}

/// The selector keys a create call may carry besides `prompt` and `title`.
const CREATE_KEYS: [&str; 8] = ["prompt", "title", "after_seconds", "at", "every_seconds", "daily", "weekly", "cron"];
const UPDATE_KEYS: [&str; 8] = ["id", "title", "prompt", "at", "every_seconds", "daily", "weekly", "cron"];

fn invalid_interval(every: Option<&Value>) -> Option<Value> {
    let every = every?;
    let Some(seconds) = domain::safe_integer(every) else {
        return Some(error("invalid_rule", "every_seconds must be a safe integer."));
    };
    (seconds < MIN_EVERY_SECONDS).then(|| error("frequency_too_high", format!("every_seconds must be at least {MIN_EVERY_SECONDS}.")))
}

fn title_problem(title: &str) -> Option<Value> {
    let trimmed = js_trim(title);
    if trimmed.is_empty() {
        return Some(error("invalid_prompt", REQUIRED_TITLE_MESSAGE));
    }
    (trimmed.encode_utf16().count() > MAX_TITLE_LENGTH)
        .then(|| error("invalid_prompt", format!("title must be at most {MAX_TITLE_LENGTH} characters.")))
}

fn validate_create(args: &Value) -> Option<Value> {
    let object = args.as_object()?;
    let selectors = ["after_seconds", "at", "every_seconds", "daily", "weekly", "cron"].iter().filter(|k| object.contains_key(**k)).count();
    if object.keys().any(|k| !CREATE_KEYS.contains(&k.as_str())) || selectors != 1 {
        return Some(error(
            "invalid_selector",
            "schedule_create accepts exactly one of after_seconds, at, every_seconds, daily, weekly, or cron.",
        ));
    }
    if js_trim(object["prompt"].as_str().unwrap_or_default()).is_empty() {
        return Some(error("invalid_prompt", "prompt must be non-empty after trimming."));
    }
    if let Some(problem) = title_problem(object["title"].as_str().unwrap_or_default()) {
        return Some(problem);
    }
    if let Some(after) = object.get("after_seconds") {
        if domain::safe_integer(after).is_none_or(|n| n <= 0) {
            return Some(error("invalid_rule", "after_seconds must be a positive safe integer."));
        }
    }
    invalid_interval(object.get("every_seconds"))
}

fn validate_update(args: &Value) -> Option<Value> {
    let object = args.as_object()?;
    let selectors = ["at", "every_seconds", "daily", "weekly", "cron"].iter().filter(|k| object.contains_key(**k)).count();
    if object.keys().any(|k| !UPDATE_KEYS.contains(&k.as_str())) || selectors > 1 {
        return Some(error("invalid_selector", "schedule_update accepts at most one of at, every_seconds, daily, weekly, or cron."));
    }
    let id = object["id"].as_str().unwrap_or_default();
    if id.is_empty() || js_trim(id) != id {
        return Some(error("invalid_rule", "schedule_update id must be non-empty without surrounding whitespace."));
    }
    if selectors == 0 && !object.contains_key("title") && !object.contains_key("prompt") {
        return Some(error(
            "invalid_selector",
            "schedule_update needs a new title, prompt, or one of at, every_seconds, daily, weekly, or cron.",
        ));
    }
    if let Some(title) = object.get("title").and_then(Value::as_str) {
        if let Some(problem) = title_problem(title) {
            return Some(problem);
        }
    }
    if object.get("prompt").and_then(Value::as_str).is_some_and(|p| js_trim(p).is_empty()) {
        return Some(error("invalid_prompt", "prompt must be non-empty after trimming."));
    }
    invalid_interval(object.get("every_seconds"))
}

/// The timing change an update call asks for.
fn timing_change(args: &Value) -> Option<Value> {
    let object = args.as_object()?;
    if let Some(at) = object.get("at") {
        return Some(json!({ "kind": "at", "at": at }));
    }
    if let Some(every) = object.get("every_seconds") {
        return Some(json!({ "kind": "every", "every_seconds": every }));
    }
    for kind in ["daily", "weekly", "cron"] {
        if let Some(selector) = object.get(kind) {
            return Some(json!({ "kind": kind, kind: selector }));
        }
    }
    None
}

fn create_request(args: &Value) -> CreateRequest {
    let get = |key: &str| args.get(key).cloned();
    CreateRequest {
        prompt: args["prompt"].as_str().unwrap_or_default().to_string(),
        title: args["title"].as_str().unwrap_or_default().to_string(),
        after_seconds: get("after_seconds"),
        at: get("at"),
        every_seconds: get("every_seconds"),
        daily: get("daily"),
        weekly: get("weekly"),
        cron: get("cron"),
    }
}

/// The plugin.
pub fn plugin() -> Arc<dyn Plugin> {
    cordis::plugin::plugin_with("schedule", &[SCHEDULE_SERVICE][..], |ctx, _config: Value| async move {
        // The gui plugin provides the `Arc<ScheduleService>`; `require` returns it shared again.
        let service: Arc<ScheduleService> = (*ctx.require::<Arc<ScheduleService>>(SCHEDULE_SERVICE)?).clone();
        let schemas = schemas();
        // The arguments are checked against the tool's schema before its body runs,
        // as the TypeScript `defineTool` wrapper does; a malformed call is an
        // `INVALID_ARGS` error that lists every violation.
        let spec = |name: &str, body: Arc<dyn Fn(Value, crate::types::ToolRunContext) -> crate::api::services::BoxFuture<ToolExecutionResult> + Send + Sync>| {
            let schema = &schemas[name];
            let parameters = schema["parameters"].clone();
            let checked = parameters.clone();
            tool_spec_ctx(
                name,
                schema["description"].as_str().unwrap_or_default(),
                parameters,
                move |args, run| {
                    let found = crate::tools::args_schema::violations(&checked, &args);
                    if !found.is_empty() {
                        let message = crate::tools::args_schema::invalid_arguments_message(&found);
                        return Box::pin(async move { ToolExecutionResult::error("INVALID_ARGS", message) }) as crate::api::services::BoxFuture<ToolExecutionResult>;
                    }
                    body(args, run)
                },
            )
        };
        let svc = service.clone();
        let create = spec("schedule_create", Arc::new(move |args, run| {
            let svc = svc.clone();
            Box::pin(async move {
                let Some(agent) = run.agent_id.clone() else { return rendered(internal()) };
                if is_subagent(&run.ctx, &agent) {
                    return rendered(error("subagent_session", "A delegated subagent cannot use reminders."));
                }
                if let Some(problem) = validate_create(&args) {
                    return rendered(problem);
                }
                if run.signal.is_cancelled() {
                    return rendered(internal());
                }
                match svc.create(&agent, &create_request(&args)).await {
                    Ok(record) => rendered(domain::view(&record, svc.now())),
                    Err(failure) => rendered(failure_value(failure)),
                }
            })
        }));
        let svc = service.clone();
        let list = spec("schedule_list", Arc::new(move |_args, run| {
            let svc = svc.clone();
            Box::pin(async move {
                let Some(agent) = run.agent_id.clone() else { return rendered(internal()) };
                if is_subagent(&run.ctx, &agent) {
                    return rendered(error("subagent_session", "A delegated subagent cannot use reminders."));
                }
                if run.signal.is_cancelled() {
                    return rendered(internal());
                }
                let now = svc.now();
                rendered(Value::Array(svc.list(&agent).await.iter().map(|r| domain::view(r, now)).collect()))
            })
        }));
        let svc = service.clone();
        let delete = spec("schedule_delete", Arc::new(move |args, run| {
            let svc = svc.clone();
            Box::pin(async move {
                let id = args["id"].as_str().unwrap_or_default().to_string();
                if id.is_empty() || js_trim(&id) != id {
                    return rendered(error("invalid_rule", "schedule_delete id must be non-empty without surrounding whitespace."));
                }
                let Some(agent) = run.agent_id.clone() else { return rendered(internal()) };
                if is_subagent(&run.ctx, &agent) {
                    return rendered(error("subagent_session", "A delegated subagent cannot use reminders."));
                }
                if run.signal.is_cancelled() {
                    return rendered(internal());
                }
                match svc.delete(&agent, &id).await {
                    Ok(outcome) => rendered(outcome),
                    Err(failure) => rendered(failure_value(failure)),
                }
            })
        }));
        let svc = service.clone();
        let update = spec("schedule_update", Arc::new(move |args, run| {
            let svc = svc.clone();
            Box::pin(async move {
                let Some(agent) = run.agent_id.clone() else { return rendered(internal()) };
                if is_subagent(&run.ctx, &agent) {
                    return rendered(error("subagent_session", "A delegated subagent cannot use reminders."));
                }
                if let Some(problem) = validate_update(&args) {
                    return rendered(problem);
                }
                if run.signal.is_cancelled() {
                    return rendered(internal());
                }
                let id = args["id"].as_str().unwrap_or_default().to_string();
                let Some(expected) = svc.list(&agent).await.into_iter().find(|r| r["id"] == id.as_str()) else {
                    let ended = svc.catalog().await.iter().any(|row| row["sessionId"] == agent.as_str() && row["id"] == id.as_str());
                    return rendered(json!({ "id": id, "updated": false, "code": if ended { "schedule_ended" } else { "schedule_not_found" } }));
                };
                let change = timing_change(&args);
                let outcome = svc
                    .update(&agent, &id, &expected, change.as_ref(), args.get("title").and_then(Value::as_str), args.get("prompt").and_then(Value::as_str))
                    .await;
                match outcome {
                    Ok(result) => match result.get("record") {
                        Some(record) => rendered(domain::view(record, svc.now())),
                        None => rendered(result),
                    },
                    Err(failure) => rendered(failure_value(failure)),
                }
            })
        }));
        register_tools(&ctx, "schedule.tools", vec![create, list, update, delete]).await?;
        Ok(())
    })
}
