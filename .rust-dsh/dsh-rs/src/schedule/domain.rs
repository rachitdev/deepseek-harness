//! Reminder records, their creation rules and the arithmetic that finds the
//! next occurrence. Mirrors `@deepseek-ai/dsh-schedule`'s `domain.ts`.
//!
//! A record is a JSON object whose shape depends on its `kind`:
//!
//! | kind | extra fields |
//! |------|--------------|
//! | `after` | `afterSeconds` |
//! | `at` | none |
//! | `every` | `everySeconds` |
//! | `daily` | `time`, `timeZone` |
//! | `weekly` | `time`, `timeZone`, `weekdays` |
//! | `cron` | `expression`, `timeZone` |
//!
//! Every record also has `id`, `title`, `prompt` and `scheduledAt`, the next
//! instant it is due. Records are kept as JSON values so a record this build
//! stores is byte-identical to the one the TypeScript host stores.

use jiff::civil::{Date, Time};
use jiff::ToSpan;
use serde_json::{json, Map, Value};

use super::cron::{self, Cron};
use super::instant::{self, InstantRejection, LocalRejection, ZoneRejection};
use super::ScheduleError;

/// The shortest `every` interval.
pub const MIN_EVERY_SECONDS: i64 = 60;
/// The longest title, in UTF-16 code units, as JavaScript counts a string.
pub const MAX_TITLE_LENGTH: usize = 120;
/// How far a cron search looks ahead or behind, in years.
const CRON_HORIZON_YEARS: i16 = 400;

pub const REQUIRED_TITLE_MESSAGE: &str = "title is required and must be non-empty after trimming.";

fn input(code: &'static str, message: impl Into<String>) -> ScheduleError {
    ScheduleError::input(code, message)
}

/// Length as JavaScript counts it: UTF-16 code units.
fn js_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// JavaScript's `String.prototype.trim` whitespace: Unicode White_Space plus BOM.
fn is_js_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

/// `String.prototype.trim`.
pub fn js_trim(text: &str) -> &str {
    text.trim_matches(is_js_space)
}

/// Normalise a title the way `scheduleTitle` does.
pub fn schedule_title(title: &str) -> Result<String, ScheduleError> {
    let normalized = js_trim(title);
    if normalized.is_empty() {
        return Err(input("invalid_prompt", REQUIRED_TITLE_MESSAGE));
    }
    if js_len(normalized) > MAX_TITLE_LENGTH {
        return Err(input("invalid_prompt", format!("title must be at most {MAX_TITLE_LENGTH} characters.")));
    }
    Ok(normalized.to_string())
}

fn trimmed_prompt(prompt: &str) -> Result<String, ScheduleError> {
    let normalized = js_trim(prompt);
    if normalized.is_empty() {
        return Err(input("invalid_prompt", "prompt must be non-empty after trimming."));
    }
    Ok(normalized.to_string())
}

fn zone_error(reason: ZoneRejection) -> ScheduleError {
    let _ = reason;
    input("invalid_time_zone", "time_zone must be UTC or a valid IANA Area/Location name.")
}

/// Canonicalise a `time_zone` value.
pub fn canonicalize_zone(value: &str) -> Result<String, ScheduleError> {
    instant::canonical_zone(value).map_err(zone_error)
}

fn local_diagnostic(reason: LocalRejection) -> &'static str {
    match reason {
        LocalRejection::Malformed => {
            "Local at requires date YYYY-MM-DD and time HH:mm:ss with optional one-to-three digit milliseconds."
        }
        LocalRejection::NotACalendarDate => "The at value must be a real ISO calendar date and time.",
        LocalRejection::OutOfRange => "The local at value must be a real ISO calendar date and time.",
    }
}

fn offset_instant(value: &str) -> Result<i64, ScheduleError> {
    instant::parse_offset_instant(value).map_err(|reason| match reason {
        InstantRejection::NotRfc3339 => input(
            "invalid_rule",
            "at must use YYYY-MM-DDTHH:mm:ss with optional 1-3 digit fractional seconds and an explicit Z or numeric offset.",
        ),
        InstantRejection::InvalidNumericOffset => input("invalid_rule", "The at numeric offset is invalid."),
        InstantRejection::NotACalendarDate | InstantRejection::OutOfRange => {
            input("invalid_rule", "The at value must be a real ISO calendar date and time.")
        }
    })
}

/// A wire value that must be an object with exactly these keys.
fn exact_keys(value: &Map<String, Value>, expected: &[&str]) -> bool {
    expected.iter().all(|key| value.contains_key(*key)) && value.keys().all(|key| expected.contains(&key.as_str()))
}

/// The future instant `epoch` as its wire text, refusing the past and the unrepresentable.
fn future_instant(epoch: i64, now: i64) -> Result<String, ScheduleError> {
    let Some(text) = instant::format_instant(epoch) else {
        return Err(input(
            "time_out_of_range",
            "The scheduled time must be representable as a four-digit-year RFC 3339 UTC instant.",
        ));
    };
    if epoch <= now {
        return Err(input("not_future", "The scheduled time must be strictly in the future."));
    }
    Ok(text)
}

/// An `at` input: an offset string or a `{date, time, time_zone}` object.
pub fn parse_at(at: &Value) -> Result<i64, ScheduleError> {
    match at {
        Value::String(text) => offset_instant(text),
        Value::Object(object) => {
            if !exact_keys(object, &["date", "time", "time_zone"]) {
                return Err(input("invalid_rule", "Local at must contain exactly date, time, and time_zone."));
            }
            let (Some(date), Some(time)) = (object["date"].as_str(), object["time"].as_str()) else {
                return Err(input("invalid_rule", "Local at date and time must be strings."));
            };
            let Some(zone_name) = object["time_zone"].as_str() else {
                return Err(input("invalid_time_zone", "time_zone must be a string."));
            };
            let local = instant::parse_local(date, time)
                .map_err(|reason| input("invalid_rule", local_diagnostic(reason)))?;
            let zone_name = canonicalize_zone(zone_name)?;
            let zone = instant::zone(&zone_name).ok_or_else(|| zone_error(ZoneRejection::Unsupported))?;
            let civil = local.civil().ok_or_else(|| input("invalid_rule", local_diagnostic(LocalRejection::NotACalendarDate)))?;
            instant::resolve_local(civil, &zone)
                .ok_or_else(|| input("invalid_rule", "The local at time does not exist in the selected time zone."))
        }
        _ => Err(input("invalid_rule", "at must be an explicit-offset string or local calendar object.")),
    }
}

/// `HH:mm:ss` with optional 1-3 fractional digits: no leap second, no 24:00.
fn clock_time(value: &str, selector: &str) -> Result<Time, ScheduleError> {
    let bad = || {
        input(
            "invalid_rule",
            format!("{selector}.time must use HH:mm:ss with optional 1-3 fractional digits, without leap seconds or 24:00."),
        )
    };
    let (whole, fraction) = match value.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (value, None),
    };
    let parts: Vec<&str> = whole.split(':').collect();
    if parts.len() != 3 || parts.iter().any(|part| part.len() != 2 || !part.bytes().all(|b| b.is_ascii_digit())) {
        return Err(bad());
    }
    let (hour, minute, second): (i8, i8, i8) = (parts[0].parse().map_err(|_| bad())?, parts[1].parse().map_err(|_| bad())?, parts[2].parse().map_err(|_| bad())?);
    if hour > 23 || minute > 59 || second > 59 {
        return Err(bad());
    }
    let millis: i32 = match fraction {
        None => 0,
        Some(f) if (1..=3).contains(&f.len()) && f.bytes().all(|b| b.is_ascii_digit()) => {
            format!("{f:0<3}").parse().map_err(|_| bad())?
        }
        Some(_) => return Err(bad()),
    };
    Time::new(hour, minute, second, millis * 1_000_000).map_err(|_| bad())
}

/// `HH:mm:ss.SSS`.
fn clock_text(time: Time) -> String {
    format!("{:02}:{:02}:{:02}.{:03}", time.hour(), time.minute(), time.second(), time.subsec_nanosecond() / 1_000_000)
}

/// A validated `daily` input.
pub struct Daily {
    pub time: String,
    pub time_zone: String,
}

pub fn parse_daily(daily: &Value) -> Result<Daily, ScheduleError> {
    let object = daily.as_object().filter(|o| exact_keys(o, &["time", "time_zone"]) && o["time"].is_string());
    let Some(object) = object else {
        return Err(input(
            "invalid_rule",
            "daily must contain exactly time and time_zone, with time HH:mm:ss and optional 1-3 fractional digits.",
        ));
    };
    let Some(zone) = object["time_zone"].as_str() else {
        return Err(input("invalid_time_zone", "time_zone must be a string."));
    };
    Ok(Daily {
        time: clock_text(clock_time(object["time"].as_str().unwrap_or_default(), "daily")?),
        time_zone: canonicalize_zone(zone)?,
    })
}

/// A validated `weekly` input.
pub struct Weekly {
    pub time: String,
    pub time_zone: String,
    pub weekdays: Vec<i64>,
}

pub fn parse_weekly(weekly: &Value) -> Result<Weekly, ScheduleError> {
    let object = weekly.as_object().filter(|o| exact_keys(o, &["time", "time_zone", "weekdays"]) && o["time"].is_string());
    let Some(object) = object else {
        return Err(input(
            "invalid_rule",
            "weekly must contain exactly time, time_zone, and weekdays, with time HH:mm:ss and optional 1-3 fractional digits.",
        ));
    };
    let Some(zone) = object["time_zone"].as_str() else {
        return Err(input("invalid_time_zone", "time_zone must be a string."));
    };
    Ok(Weekly {
        time: clock_text(clock_time(object["time"].as_str().unwrap_or_default(), "weekly")?),
        time_zone: canonicalize_zone(zone)?,
        weekdays: normalize_weekdays(&object["weekdays"])?,
    })
}

/// A validated `cron` input.
pub struct CronInput {
    pub expression: String,
    pub time_zone: String,
}

pub fn parse_cron_input(value: &Value) -> Result<CronInput, ScheduleError> {
    let object = value.as_object().filter(|o| exact_keys(o, &["expression", "time_zone"]) && o["expression"].is_string());
    let Some(object) = object else {
        return Err(input(
            "invalid_rule",
            "cron must contain exactly expression and time_zone, with a five-field cron expression.",
        ));
    };
    let Some(zone) = object["time_zone"].as_str() else {
        return Err(input("invalid_time_zone", "time_zone must be a string."));
    };
    Ok(CronInput {
        expression: cron::parse(object["expression"].as_str().unwrap_or_default())?.expression,
        time_zone: canonicalize_zone(zone)?,
    })
}

/// Weekday numbers: unique integers 1 (Monday) to 7 (Sunday), returned sorted.
pub fn normalize_weekdays(weekdays: &Value) -> Result<Vec<i64>, ScheduleError> {
    let Some(list) = weekdays.as_array().filter(|list| !list.is_empty()) else {
        return Err(input("invalid_rule", "weekly.weekdays must be a non-empty array of ISO weekday numbers."));
    };
    let mut unique = std::collections::BTreeSet::new();
    for entry in list {
        let weekday = entry.as_f64().filter(|n| n.fract() == 0.0 && (1.0..=7.0).contains(n));
        let Some(weekday) = weekday else {
            return Err(input(
                "invalid_rule",
                "Each weekly.weekdays entry must be an integer from 1 (Monday) through 7 (Sunday).",
            ));
        };
        if !unique.insert(weekday as i64) {
            return Err(input("invalid_rule", format!("weekly.weekdays must not repeat weekday {}.", weekday as i64)));
        }
    }
    Ok(unique.into_iter().collect())
}

// --- occurrences --------------------------------------------------------

/// One due occurrence and the instant after it, when there is one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub occurrence_at: String,
    pub next_scheduled_at: Option<String>,
}

fn log_error(message: impl Into<String>) -> ScheduleError {
    ScheduleError::input("corrupt_schedule_log", message)
}

/// The record's `scheduledAt` in milliseconds.
pub fn parse_scheduled_ms(record: &Value) -> Result<i64, ScheduleError> {
    parse_scheduled(record)
}

fn parse_scheduled(record: &Value) -> Result<i64, ScheduleError> {
    stored_instant(&record["scheduledAt"])
}

const NON_CANONICAL_INSTANT: &str = "scheduledAt must be a canonical four-digit-year RFC 3339 UTC instant";

/// A stored instant, with TypeScript's two diagnostics: a string that is not
/// in the canonical form, and one that is shaped right but names no real moment.
fn stored_instant(value: &Value) -> Result<i64, ScheduleError> {
    let Some(text) = value.as_str() else {
        return Err(log_error(NON_CANONICAL_INSTANT));
    };
    instant::parse_stored_instant(text).map_err(|reason| match reason {
        InstantRejection::NotACalendarDate => log_error("scheduledAt is not a real UTC calendar instant"),
        _ => log_error(NON_CANONICAL_INSTANT),
    })
}

fn str_field<'a>(record: &'a Value, name: &str) -> &'a str {
    record[name].as_str().unwrap_or_default()
}

fn zone_of(record: &Value) -> Result<jiff::tz::TimeZone, ScheduleError> {
    instant::zone(str_field(record, "timeZone")).ok_or_else(|| log_error("timeZone is not a known zone"))
}

/// A date predicate: which local dates an occurrence may fall on.
type DateMatch<'a> = Box<dyn Fn(Date) -> bool + 'a>;

fn every_date<'a>() -> DateMatch<'a> {
    Box::new(|_| true)
}

fn weekday_match<'a>(weekdays: &'a [i64]) -> DateMatch<'a> {
    Box::new(move |date| weekdays.contains(&i64::from(date.weekday().to_monday_one_offset())))
}

/// The day after the last representable date.
fn ceiling_date() -> Date {
    instant::local_date(instant::MAX_MS, &jiff::tz::TimeZone::UTC)
        .and_then(|date| date.tomorrow().ok())
        .unwrap_or(Date::MAX)
}

/// The next local date and time after `now` that matches, scanning forward.
fn next_matching(
    time: Time,
    zone: &jiff::tz::TimeZone,
    now: i64,
    matches: &DateMatch<'_>,
    after: Option<Date>,
) -> Option<String> {
    let mut date = instant::local_date(now, zone)?;
    let last = ceiling_date();
    if let Some(after) = after {
        if date <= after {
            date = after.tomorrow().ok()?;
        }
    }
    while date <= last {
        if matches(date) {
            if let Some(target) = instant::resolve_local(date.to_datetime(time), zone) {
                if target > instant::MAX_MS {
                    return None;
                }
                if target >= instant::MIN_MS && target > now {
                    return instant::format_instant(target);
                }
            }
        }
        date = date.tomorrow().ok()?;
    }
    None
}

/// The latest occurrence at or before `accepted`, never earlier than `saved`.
fn latest_due(zone: &jiff::tz::TimeZone, matches: &DateMatch<'_>, time: Time, accepted: i64, saved: i64) -> i64 {
    let mut date = instant::local_date(accepted, &jiff::tz::TimeZone::UTC)
        .and_then(|d| d.tomorrow().ok())
        .unwrap_or(Date::MAX);
    loop {
        if matches(date) {
            if let Some(candidate) = instant::resolve_local(date.to_datetime(time), zone) {
                if candidate <= accepted {
                    return candidate.max(saved);
                }
            }
        }
        match date.yesterday() {
            Ok(previous) => date = previous,
            Err(_) => return saved,
        }
    }
}

fn accepted_decision(selector: &str, accepted: i64) -> Result<i64, ScheduleError> {
    if instant::in_range(accepted) {
        Ok(accepted)
    } else {
        Err(log_error(format!("{selector} acceptedAt must be a representable four-digit-year instant")))
    }
}

fn occurrence_of(occurrence: i64, next: Option<String>) -> Result<Occurrence, ScheduleError> {
    Ok(Occurrence {
        occurrence_at: instant::format_instant(occurrence).ok_or_else(|| log_error("occurrence is not representable"))?,
        next_scheduled_at: next,
    })
}

pub fn resolve_every(record: &Value, accepted: i64) -> Result<Occurrence, ScheduleError> {
    let target = parse_scheduled(record)?;
    let seconds = record["everySeconds"].as_i64().unwrap_or(0);
    let interval = seconds.checked_mul(1_000).filter(|i| *i > 0);
    if !instant::in_range(accepted) {
        return Err(log_error("every acceptedAt must be a representable four-digit-year instant"));
    }
    let Some(interval) = interval else {
        return Err(log_error("every interval milliseconds must be a positive safe integer"));
    };
    if accepted < target {
        return Err(log_error("every dispatch cannot precede the active scheduledAt"));
    }
    let steps = (accepted - target) / interval;
    let occurrence = target + steps * interval;
    let next = occurrence + interval;
    occurrence_of(occurrence, instant::in_range(next).then(|| instant::format_instant(next)).flatten())
}

fn time_of(record: &Value) -> Result<Time, ScheduleError> {
    clock_time(str_field(record, "time"), str_field(record, "kind"))
}

pub fn resolve_daily(record: &Value, accepted: i64) -> Result<Occurrence, ScheduleError> {
    let decision = accepted_decision("daily", accepted)?;
    let saved = parse_scheduled(record)?;
    if decision < saved {
        return Err(log_error("daily dispatch cannot precede the active scheduledAt"));
    }
    let (time, zone) = (time_of(record)?, zone_of(record)?);
    let occurrence = latest_due(&zone, &every_date(), time, decision, saved);
    let after = instant::local_date(occurrence, &zone);
    occurrence_of(occurrence, next_matching(time, &zone, decision, &every_date(), after))
}

pub fn resolve_weekly(record: &Value, accepted: i64) -> Result<Occurrence, ScheduleError> {
    let decision = accepted_decision("weekly", accepted)?;
    let saved = parse_scheduled(record)?;
    if decision < saved {
        return Err(log_error("weekly dispatch cannot precede the active scheduledAt"));
    }
    let (time, zone) = (time_of(record)?, zone_of(record)?);
    let weekdays = normalize_weekdays(&record["weekdays"])?;
    let matches = weekday_match(&weekdays);
    let occurrence = latest_due(&zone, &matches, time, decision, saved);
    let after = instant::local_date(occurrence, &zone);
    occurrence_of(occurrence, next_matching(time, &zone, decision, &matches, after))
}

/// Whether a local date satisfies the cron date fields. When both day fields
/// are restricted a date matches if either does, as Vixie cron does.
fn cron_date_match(parsed: &Cron) -> DateMatch<'_> {
    Box::new(move |date| {
        if !parsed.months.contains(&i64::from(date.month())) {
            return false;
        }
        let day_of_month = parsed.days_of_month.contains(&i64::from(date.day()));
        let day_of_week = parsed.days_of_week.contains(&i64::from(date.weekday().to_monday_one_offset() % 7));
        if !parsed.day_of_month_star && !parsed.day_of_week_star {
            return day_of_month || day_of_week;
        }
        day_of_month && day_of_week
    })
}

fn cron_times(parsed: &Cron) -> Vec<Time> {
    let mut times = Vec::new();
    for hour in &parsed.hours {
        for minute in &parsed.minutes {
            if let Ok(time) = Time::new(*hour as i8, *minute as i8, 0, 0) {
                times.push(time);
            }
        }
    }
    times
}

fn next_cron(parsed: &Cron, zone: &jiff::tz::TimeZone, floor: i64) -> Option<String> {
    let times = cron_times(parsed);
    let matches = cron_date_match(parsed);
    let first = instant::local_date(floor, zone)?;
    let floor_time = jiff::Timestamp::from_millisecond(floor).ok()?.to_zoned(zone.clone()).time();
    let ceiling = ceiling_date();
    let horizon = first.checked_add(i64::from(CRON_HORIZON_YEARS).years()).ok()?;
    let last = horizon.min(ceiling);
    let mut date = first;
    while date <= last {
        if !parsed.months.contains(&i64::from(date.month())) {
            // Jump to the first of the next month.
            date = date.first_of_month().checked_add(1.month()).ok()?;
            continue;
        }
        if matches(date) {
            for time in &times {
                if date == first && *time < floor_time {
                    continue;
                }
                let Some(target) = instant::resolve_local(date.to_datetime(*time), zone) else {
                    continue;
                };
                if target > instant::MAX_MS {
                    return None;
                }
                if target >= instant::MIN_MS && target > floor {
                    return instant::format_instant(target);
                }
            }
        }
        date = date.tomorrow().ok()?;
    }
    None
}

fn latest_cron(parsed: &Cron, zone: &jiff::tz::TimeZone, decision: i64, saved: i64) -> i64 {
    let times = cron_times(parsed);
    let matches = cron_date_match(parsed);
    let mut date = instant::local_date(decision, &jiff::tz::TimeZone::UTC)
        .and_then(|d| d.tomorrow().ok())
        .unwrap_or(Date::MAX);
    let floor_date = instant::local_date(instant::MIN_MS, zone).unwrap_or(Date::MIN);
    let horizon = date.checked_sub(i64::from(CRON_HORIZON_YEARS).years()).unwrap_or(Date::MIN);
    let first = horizon.max(floor_date);
    while date >= first {
        if !parsed.months.contains(&i64::from(date.month())) {
            // Jump to the last day of the previous month.
            match date.first_of_month().yesterday() {
                Ok(previous) => date = previous,
                Err(_) => return saved,
            }
            continue;
        }
        if matches(date) {
            // The earliest time of the day that exists; if it is already after
            // the decision, nothing on this day is due.
            let earliest = times.iter().find_map(|time| instant::resolve_local(date.to_datetime(*time), zone));
            if earliest.is_some_and(|earliest| earliest > decision) {
                match date.yesterday() {
                    Ok(previous) => date = previous,
                    Err(_) => return saved,
                }
                continue;
            }
            for time in times.iter().rev() {
                let Some(candidate) = instant::resolve_local(date.to_datetime(*time), zone) else {
                    continue;
                };
                if candidate > decision || candidate < instant::MIN_MS {
                    continue;
                }
                return candidate.max(saved);
            }
        }
        match date.yesterday() {
            Ok(previous) => date = previous,
            Err(_) => return saved,
        }
    }
    saved
}

pub fn resolve_cron(record: &Value, accepted: i64) -> Result<Occurrence, ScheduleError> {
    let decision = accepted_decision("cron", accepted)?;
    let saved = parse_scheduled(record)?;
    if decision < saved {
        return Err(log_error("cron dispatch cannot precede the active scheduledAt"));
    }
    let parsed = cron::parse(str_field(record, "expression"))?;
    let zone = zone_of(record)?;
    let occurrence = latest_cron(&parsed, &zone, decision, saved);
    occurrence_of(occurrence, next_cron(&parsed, &zone, decision))
}

/// Whether the record repeats.
pub fn is_recurring(record: &Value) -> bool {
    matches!(record["kind"].as_str(), Some("every" | "daily" | "weekly" | "cron"))
}

/// The due occurrence of a recurring record at `accepted`.
pub fn resolve_recurring(record: &Value, accepted: i64) -> Result<Occurrence, ScheduleError> {
    match record["kind"].as_str() {
        Some("every") => resolve_every(record, accepted),
        Some("daily") => resolve_daily(record, accepted),
        Some("weekly") => resolve_weekly(record, accepted),
        Some("cron") => resolve_cron(record, accepted),
        _ => Err(log_error("record is not recurring")),
    }
}

// --- creation -----------------------------------------------------------

fn record(id: &str, kind: &str, title: &str, prompt: &str, extra: Vec<(&str, Value)>, scheduled_at: String) -> Value {
    let mut object = Map::new();
    object.insert("id".into(), json!(id));
    object.insert("kind".into(), json!(kind));
    object.insert("title".into(), json!(title));
    object.insert("prompt".into(), json!(prompt));
    for (key, value) in extra {
        object.insert(key.to_string(), value);
    }
    object.insert("scheduledAt".into(), json!(scheduled_at));
    Value::Object(object)
}

pub fn create_after(id: &str, prompt: &str, after_seconds: &Value, now: i64, title: &str) -> Result<Value, ScheduleError> {
    let prompt = trimmed_prompt(prompt)?;
    let seconds = safe_integer(after_seconds).filter(|n| *n > 0);
    let Some(seconds) = seconds else {
        return Err(input("invalid_rule", "after_seconds must be a positive safe integer."));
    };
    let target = now.saturating_add(seconds.saturating_mul(1_000));
    let title = schedule_title(title)?;
    Ok(record(id, "after", &title, &prompt, vec![("afterSeconds", json!(seconds))], future_instant(target, now)?))
}

pub fn create_at(id: &str, prompt: &str, at: &Value, now: i64, title: &str) -> Result<Value, ScheduleError> {
    let prompt = trimmed_prompt(prompt)?;
    let target = parse_at(at)?;
    let title = schedule_title(title)?;
    Ok(record(id, "at", &title, &prompt, vec![], future_instant(target, now)?))
}

pub fn create_every(id: &str, prompt: &str, every_seconds: &Value, now: i64, title: &str) -> Result<Value, ScheduleError> {
    let prompt = trimmed_prompt(prompt)?;
    let Some(seconds) = safe_integer(every_seconds) else {
        return Err(input("invalid_rule", "every_seconds must be a safe integer."));
    };
    if seconds < MIN_EVERY_SECONDS {
        return Err(input("frequency_too_high", format!("every_seconds must be at least {MIN_EVERY_SECONDS}.")));
    }
    let title = schedule_title(title)?;
    let target = now.saturating_add(seconds.saturating_mul(1_000));
    Ok(record(id, "every", &title, &prompt, vec![("everySeconds", json!(seconds))], future_instant(target, now)?))
}

pub fn create_daily(id: &str, prompt: &str, daily: &Value, now: i64, title: &str) -> Result<Value, ScheduleError> {
    let prompt = trimmed_prompt(prompt)?;
    let parsed = parse_daily(daily)?;
    if !instant::in_range(now) {
        return Err(input("time_out_of_range", "Daily creation time must be a representable four-digit-year UTC instant."));
    }
    let zone = instant::zone(&parsed.time_zone).ok_or_else(|| zone_error(ZoneRejection::Unsupported))?;
    let time = clock_time(&parsed.time, "daily")?;
    let Some(scheduled) = next_matching(time, &zone, now, &every_date(), None) else {
        return Err(input("time_out_of_range", "No future daily occurrence is representable as a four-digit-year UTC instant."));
    };
    let title = schedule_title(title)?;
    Ok(record(id, "daily", &title, &prompt, vec![("time", json!(parsed.time)), ("timeZone", json!(parsed.time_zone))], scheduled))
}

pub fn create_weekly(id: &str, prompt: &str, weekly: &Value, now: i64, title: &str) -> Result<Value, ScheduleError> {
    let prompt = trimmed_prompt(prompt)?;
    let parsed = parse_weekly(weekly)?;
    if !instant::in_range(now) {
        return Err(input("time_out_of_range", "Weekly creation time must be a representable four-digit-year UTC instant."));
    }
    let zone = instant::zone(&parsed.time_zone).ok_or_else(|| zone_error(ZoneRejection::Unsupported))?;
    let time = clock_time(&parsed.time, "weekly")?;
    let matches = weekday_match(&parsed.weekdays);
    let Some(scheduled) = next_matching(time, &zone, now, &matches, None) else {
        return Err(input("time_out_of_range", "No future weekly occurrence is representable as a four-digit-year UTC instant."));
    };
    let title = schedule_title(title)?;
    Ok(record(
        id,
        "weekly",
        &title,
        &prompt,
        vec![("time", json!(parsed.time)), ("timeZone", json!(parsed.time_zone)), ("weekdays", json!(parsed.weekdays))],
        scheduled,
    ))
}

pub fn create_cron(id: &str, prompt: &str, cron_value: &Value, now: i64, title: &str) -> Result<Value, ScheduleError> {
    let prompt = trimmed_prompt(prompt)?;
    let parsed = parse_cron_input(cron_value)?;
    if !instant::in_range(now) {
        return Err(input("time_out_of_range", "Cron creation time must be a representable four-digit-year UTC instant."));
    }
    let zone = instant::zone(&parsed.time_zone).ok_or_else(|| zone_error(ZoneRejection::Unsupported))?;
    let expression = cron::parse(&parsed.expression)?;
    let Some(scheduled) = next_cron(&expression, &zone, now) else {
        return Err(input("time_out_of_range", "No future cron occurrence is representable as a four-digit-year UTC instant."));
    };
    let title = schedule_title(title)?;
    Ok(record(
        id,
        "cron",
        &title,
        &prompt,
        vec![("expression", json!(parsed.expression)), ("timeZone", json!(parsed.time_zone))],
        scheduled,
    ))
}

/// `Number.isSafeInteger` on a JSON number.
pub fn safe_integer(value: &Value) -> Option<i64> {
    let number = value.as_f64()?;
    (number.fract() == 0.0 && number.abs() <= 9_007_199_254_740_991.0).then_some(number as i64)
}

/// A record as the list tools show it: the record plus its state and delivery mode.
pub fn view(record: &Value, now: i64) -> Value {
    let due = parse_scheduled(record).map(|at| now >= at).unwrap_or(false);
    let mut view = record.clone();
    if let Some(object) = view.as_object_mut() {
        object.insert("state".into(), json!(if due { "overdue" } else { "scheduled" }));
        object.insert("deliveryMode".into(), json!("host"));
    }
    view
}

const FRAMING: &str = "This is a scheduled message from the user";

/// The text delivered for a one-shot reminder.
pub fn render_reminder(record: &Value) -> String {
    [
        "[SCHEDULE REMINDER]".to_string(),
        FRAMING.to_string(),
        format!("schedule_id_json: {}", json!(record["id"])),
        format!("occurrence_at: {}", str_field(record, "scheduledAt")),
        format!("reminder_prompt_json: {}", json!(record["prompt"])),
    ]
    .join("\n")
}

/// The text delivered for a batch of recurring reminders due together.
pub fn render_recurring_batch(reminders: &[(&Value, &str)]) -> String {
    let payload: Vec<Value> = reminders
        .iter()
        .map(|(record, occurrence)| json!({ "schedule_id": record["id"], "occurrence_at": occurrence, "reminder_prompt": record["prompt"] }))
        .collect();
    ["[SCHEDULE REMINDER BATCH]".to_string(), FRAMING.to_string(), format!("reminders_json: {}", Value::Array(payload))].join("\n")
}

// --- decoding a stored record -------------------------------------------

fn corrupt(message: impl Into<String>) -> ScheduleError {
    log_error(message)
}

/// An input refusal met while decoding a stored value: the TypeScript decoder
/// reports it as `ScheduleInputError: <message>` under the log-corruption code.
fn wrapped(error: ScheduleError) -> ScheduleError {
    log_error(format!("ScheduleInputError: {}", error.message()))
}

/// Keys of an object, checking that the required ones are present and nothing
/// outside required and optional appears.
fn keys_ok(object: &Map<String, Value>, required: &[&str], optional: &[&str]) -> bool {
    required.iter().all(|key| object.contains_key(*key))
        && object.keys().all(|key| required.contains(&key.as_str()) || optional.contains(&key.as_str()))
}

fn stored_title(value: &Value) -> Result<String, ScheduleError> {
    let Some(text) = value.as_str().filter(|text| !js_trim(text).is_empty()) else {
        return Err(corrupt(REQUIRED_TITLE_MESSAGE));
    };
    if js_len(text) > MAX_TITLE_LENGTH {
        return Err(corrupt(format!("title must be at most {MAX_TITLE_LENGTH} characters")));
    }
    if js_trim(text) != text {
        return Err(corrupt("title must be a trimmed string"));
    }
    Ok(text.to_string())
}

fn stored_id(value: &Value) -> Result<(), ScheduleError> {
    match value.as_str() {
        Some(id) if !id.is_empty() && js_trim(id) == id => Ok(()),
        _ => Err(corrupt("schedule id must be a non-empty string without surrounding whitespace")),
    }
}

fn stored_prompt(value: &Value, kind: &str) -> Result<(), ScheduleError> {
    match value.as_str() {
        Some(prompt) if !prompt.is_empty() && js_trim(prompt) == prompt => Ok(()),
        _ => Err(corrupt(format!("{kind} prompt must be non-empty and already trimmed"))),
    }
}

/// Check that a stored record is a well-formed, canonical record of its kind.
/// A record failing this is never rewritten: the store reports it.
pub fn decode_record(value: &Value) -> Result<(), ScheduleError> {
    let Some(object) = value.as_object() else {
        return Err(corrupt("schedule record must be an object"));
    };
    let kind = object.get("kind").and_then(Value::as_str).unwrap_or_default();
    let (extra, message): (&[&str], &str) = match kind {
        "after" => (&["afterSeconds"], "after schedule must contain exactly id, kind, title, prompt, afterSeconds, and scheduledAt"),
        "at" => (&[], "at schedule must contain exactly id, kind, title, prompt, and scheduledAt"),
        "every" => (&["everySeconds"], "every schedule must contain exactly id, kind, title, prompt, everySeconds, and scheduledAt"),
        "daily" => (&["time", "timeZone"], "daily schedule must contain exactly id, kind, title, prompt, time, timeZone, and scheduledAt"),
        "weekly" => (
            &["time", "timeZone", "weekdays"],
            "weekly schedule must contain exactly id, kind, title, prompt, time, timeZone, weekdays, and scheduledAt",
        ),
        "cron" => (&["expression", "timeZone"], "cron schedule must contain exactly id, kind, title, prompt, expression, timeZone, and scheduledAt"),
        _ => return Err(corrupt("v1 schedule kind must be \"after\", \"at\", or \"every\"")),
    };
    // The newer kinds read the title before they check the key set, so a record
    // with no title fails on the title; the historical kinds check the key set
    // first (their title is optional there) and are refused for a missing title afterwards.
    let mut required: Vec<&str> = vec!["id", "kind", "prompt", "scheduledAt"];
    required.extend(extra);
    if matches!(kind, "daily" | "weekly" | "cron") {
        stored_title(object.get("title").unwrap_or(&Value::Null))?;
    }
    if !keys_ok(object, &required, &["title"]) {
        return Err(corrupt(message));
    }
    stored_title(object.get("title").unwrap_or(&Value::Null))?;
    stored_prompt(&object["prompt"], kind)?;
    stored_id(&object["id"])?;
    parse_scheduled(value)?;
    match kind {
        "after" => {
            if safe_integer(&object["afterSeconds"]).is_none_or(|n| n <= 0) {
                return Err(corrupt("afterSeconds must be a positive safe integer"));
            }
        }
        "every" => {
            let seconds = safe_integer(&object["everySeconds"]);
            if seconds.is_none_or(|n| n < MIN_EVERY_SECONDS || n.checked_mul(1_000).is_none()) {
                return Err(corrupt(format!("everySeconds must be a safe integer of at least {MIN_EVERY_SECONDS}")));
            }
        }
        "daily" | "weekly" => {
            let (Some(time), Some(zone)) = (object["time"].as_str(), object["timeZone"].as_str()) else {
                return Err(corrupt(format!("{kind} time and timeZone must be strings")));
            };
            canonicalize_zone(zone).map_err(wrapped)?;
            let normalized = clock_time(time, kind).map(clock_text).map_err(wrapped)?;
            if time != normalized {
                return Err(corrupt(format!("{kind} time must be normalized to HH:mm:ss.SSS")));
            }
            if kind == "weekly" {
                let stored = &object["weekdays"];
                let weekdays = normalize_weekdays(stored).map_err(wrapped)?;
                let as_stored: Vec<Option<i64>> =
                    stored.as_array().map(|list| list.iter().map(|v| v.as_i64()).collect()).unwrap_or_default();
                if weekdays.iter().map(|d| Some(*d)).collect::<Vec<_>>() != as_stored {
                    return Err(corrupt("weekly weekdays must be normalized to unique ascending ISO weekday numbers"));
                }
            }
        }
        "cron" => {
            let (Some(expression), Some(zone)) = (object["expression"].as_str(), object["timeZone"].as_str()) else {
                return Err(corrupt("cron expression and timeZone must be strings"));
            };
            canonicalize_zone(zone).map_err(wrapped)?;
            let canonical = cron::parse(expression).map_err(wrapped)?.expression;
            if expression != canonical {
                return Err(corrupt("cron expression must be canonical"));
            }
        }
        _ => {}
    }
    Ok(())
}
