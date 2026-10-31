//! `schedule/update`'s rules: how a requested change is checked against the
//! record the caller saw and applied. Mirrors `update.ts`.

use serde_json::{json, Value};

use super::domain::{self, js_trim, schedule_title};
use super::ScheduleError;

/// The outcome of resolving an update against the stored record.
#[derive(Debug, Clone, PartialEq)]
pub enum Resolved {
    /// The record after the change; `updated` is false when nothing changed.
    Record { updated: bool, record: Value },
    /// The caller's `expected` is not the stored record.
    Conflict,
    /// The request is invalid: `{code, message}` as a tool error.
    Invalid(ScheduleError),
}

fn input(code: &'static str, message: impl Into<String>) -> ScheduleError {
    ScheduleError::input(code, message)
}

/// The selector key each timing kind carries.
fn selector(kind: &str) -> Option<&'static str> {
    match kind {
        "at" => Some("at"),
        "every" => Some("every_seconds"),
        "daily" => Some("daily"),
        "weekly" => Some("weekly"),
        "cron" => Some("cron"),
        _ => None,
    }
}

fn validate_change(change: &Value) -> Result<(&str, &Value), ScheduleError> {
    let invalid = || input("invalid_rule", "Timing change must be an object with exactly one supported timing selector.");
    let object = change.as_object().ok_or_else(invalid)?;
    let kind = object.get("kind").and_then(Value::as_str).unwrap_or_default();
    let selector_key = selector(kind);
    match selector_key {
        Some(key) if object.len() == 2 && object.contains_key("kind") && object.contains_key(key) => {
            Ok((kind, &object[key]))
        }
        _ => Err(input("invalid_rule", "Timing change must contain exactly kind and its matching timing selector.")),
    }
}

fn schedule_prompt(prompt: &str) -> Result<String, ScheduleError> {
    let trimmed = js_trim(prompt);
    if trimmed.is_empty() {
        return Err(input("invalid_prompt", "prompt must be non-empty after trimming."));
    }
    Ok(trimmed.to_string())
}

fn with_content(current: &Value, title: &str, prompt: &str) -> Value {
    if current["title"] == title && current["prompt"] == prompt {
        return current.clone();
    }
    let mut next = current.clone();
    next["title"] = json!(title);
    next["prompt"] = json!(prompt);
    next
}

fn changed_record(current: &Value, title: &str, prompt: &str, change: Option<&Value>, now: i64) -> Result<Value, ScheduleError> {
    let Some(change) = change else {
        return Ok(with_content(current, title, prompt));
    };
    let (kind, value) = validate_change(change)?;
    let id = current["id"].as_str().unwrap_or_default();
    let current_kind = current["kind"].as_str().unwrap_or_default();
    let unchanged = || with_content(current, title, prompt);
    match kind {
        "at" => {
            if matches!(current_kind, "after" | "at")
                && domain::parse_at(value)? == domain::parse_scheduled_ms(current)?
            {
                return Ok(unchanged());
            }
            domain::create_at(id, prompt, value, now, title)
        }
        "every" => {
            if current_kind == "every" && domain::safe_integer(value) == current["everySeconds"].as_i64() && value.is_number() {
                return Ok(unchanged());
            }
            domain::create_every(id, prompt, value, now, title)
        }
        "daily" => {
            if current_kind == "daily" {
                let normalized = domain::parse_daily(value)?;
                if normalized.time == current["time"] && normalized.time_zone == domain::canonicalize_zone(current["timeZone"].as_str().unwrap_or_default())? {
                    return Ok(unchanged());
                }
            }
            domain::create_daily(id, prompt, value, now, title)
        }
        "weekly" => {
            if current_kind == "weekly" {
                let normalized = domain::parse_weekly(value)?;
                if normalized.time == current["time"]
                    && normalized.time_zone == domain::canonicalize_zone(current["timeZone"].as_str().unwrap_or_default())?
                    && json!(normalized.weekdays) == current["weekdays"]
                {
                    return Ok(unchanged());
                }
            }
            domain::create_weekly(id, prompt, value, now, title)
        }
        "cron" => {
            if current_kind == "cron" {
                let normalized = domain::parse_cron_input(value)?;
                let current_expression = super::cron::parse(current["expression"].as_str().unwrap_or_default())?.expression;
                if normalized.expression == current_expression
                    && normalized.time_zone == domain::canonicalize_zone(current["timeZone"].as_str().unwrap_or_default())?
                {
                    return Ok(unchanged());
                }
            }
            domain::create_cron(id, prompt, value, now, title)
        }
        _ => Err(input("invalid_rule", "Timing change must contain exactly kind and its matching timing selector.")),
    }
}

/// Decide an update of `current`, which the caller believed to be `expected`.
pub fn resolve(
    current: &Value,
    expected: &Value,
    change: Option<&Value>,
    now: i64,
    title: Option<&str>,
    prompt: Option<&str>,
) -> Resolved {
    if domain::decode_record(expected).is_err() {
        return Resolved::Invalid(input("invalid_rule", "expected must be a complete valid Schedule record."));
    }
    if current != expected {
        return Resolved::Conflict;
    }
    let result = (|| {
        let retained = current["title"].as_str().unwrap_or_default();
        let title = match title {
            None => retained.to_string(),
            Some(title) => schedule_title(title)?,
        };
        let prompt = match prompt {
            None => current["prompt"].as_str().unwrap_or_default().to_string(),
            Some(prompt) => schedule_prompt(prompt)?,
        };
        changed_record(current, &title, &prompt, change, now)
    })();
    match result {
        Ok(record) => Resolved::Record { updated: record != *current, record },
        Err(error) => Resolved::Invalid(error),
    }
}
