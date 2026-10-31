//! Five-field Vixie cron expressions, parsed and canonicalised exactly as the
//! TypeScript `parseCronExpression` does, so a stored expression is the same
//! text in both hosts.

use super::ScheduleError;

/// One field's bounds. `canonical_max` differs from `max` only for
/// day-of-week, where 7 is Sunday and is folded to 0.
#[derive(Debug, Clone, Copy)]
struct Spec {
    name: &'static str,
    min: i64,
    max: i64,
    canonical_max: i64,
}

const FIELDS: [Spec; 5] = [
    Spec { name: "minute", min: 0, max: 59, canonical_max: 59 },
    Spec { name: "hour", min: 0, max: 23, canonical_max: 23 },
    Spec { name: "day-of-month", min: 1, max: 31, canonical_max: 31 },
    Spec { name: "month", min: 1, max: 12, canonical_max: 12 },
    Spec { name: "day-of-week", min: 0, max: 7, canonical_max: 6 },
];

/// A parsed expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cron {
    /// The canonical text.
    pub expression: String,
    pub minutes: Vec<i64>,
    pub hours: Vec<i64>,
    pub days_of_month: Vec<i64>,
    pub months: Vec<i64>,
    pub days_of_week: Vec<i64>,
    /// The field started with `*`.
    pub day_of_month_star: bool,
    pub day_of_week_star: bool,
}

struct Field {
    values: Vec<i64>,
    canonical: String,
    star: bool,
}

fn rule(message: String) -> ScheduleError {
    ScheduleError::input("invalid_rule", message)
}

fn invalid_element(spec: Spec, element: &str) -> ScheduleError {
    rule(format!(
        "cron.expression {} field element {} must be *, a value, a-b, */n, a-b/n, or a comma-separated list of those.",
        spec.name,
        json_string(element)
    ))
}

/// `JSON.stringify` of a string.
fn json_string(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

/// A run of ASCII digits.
fn all_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

/// `Number(text)` for digit-only text, as a safe integer.
fn number(text: &str) -> Option<i64> {
    let value: f64 = text.parse().ok()?;
    (value <= 9_007_199_254_740_991.0).then_some(value as i64)
}

fn field_value(text: &str, spec: Spec) -> Result<i64, ScheduleError> {
    match number(text) {
        Some(value) if (spec.min..=spec.max).contains(&value) => Ok(value),
        _ => Err(rule(format!(
            "cron.expression {} field value {} is outside {}-{}.",
            spec.name, text, spec.min, spec.max
        ))),
    }
}

fn field_step(text: &str, spec: Spec) -> Result<i64, ScheduleError> {
    match number(text) {
        Some(step) if step >= 1 => Ok(step),
        _ => Err(rule(format!("cron.expression {} field step must be a positive integer.", spec.name))),
    }
}

fn field_values(raw: &str, spec: Spec) -> Result<Vec<i64>, ScheduleError> {
    let mut matched = std::collections::BTreeSet::new();
    for element in raw.split(',') {
        if element.is_empty() {
            return Err(rule(format!(
                "cron.expression {} field must not contain an empty list element.",
                spec.name
            )));
        }
        if element == "*" {
            matched.extend(spec.min..=spec.max);
            continue;
        }
        if element.starts_with('*') {
            let step = element
                .strip_prefix("*/")
                .filter(|step| all_digits(step))
                .ok_or_else(|| invalid_element(spec, element))?;
            let step = field_step(step, spec)?;
            let mut value = spec.min;
            while value <= spec.max {
                matched.insert(value);
                value += step;
            }
            continue;
        }
        let (range, step) = match element.split_once('/') {
            Some((range, step)) => (range, Some(step)),
            None => (element, None),
        };
        let (start, end) = match range.split_once('-') {
            Some((start, end)) => (start, Some(end)),
            None => (range, None),
        };
        if !all_digits(start) || end.is_some_and(|end| !all_digits(end)) || step.is_some_and(|step| !all_digits(step)) {
            return Err(invalid_element(spec, element));
        }
        let first = field_value(start, spec)?;
        let Some(end) = end else {
            if step.is_some() {
                return Err(invalid_element(spec, element));
            }
            matched.insert(first);
            continue;
        };
        let last = field_value(end, spec)?;
        if first > last {
            return Err(rule(format!("cron.expression {} field range {first}-{last} is inverted.", spec.name)));
        }
        let step = match step {
            Some(step) => field_step(step, spec)?,
            None => 1,
        };
        let mut value = first;
        while value <= last {
            matched.insert(value);
            value += step;
        }
    }
    let folded: std::collections::BTreeSet<i64> = matched
        .into_iter()
        .map(|value| if spec.canonical_max != spec.max && value == spec.max { spec.min } else { value })
        .collect();
    Ok(folded.into_iter().collect())
}

/// Encode a sorted value set as `a`, `a-b`, `a-b/n` or a list of runs.
fn encode(values: &[i64]) -> String {
    let first = values[0];
    if values.len() == 1 {
        return first.to_string();
    }
    let last = values[values.len() - 1];
    let step = values[1] - first;
    if values.windows(2).skip(1).all(|pair| pair[1] - pair[0] == step) {
        return if step == 1 { format!("{first}-{last}") } else { format!("{first}-{last}/{step}") };
    }
    let mut parts = Vec::new();
    let mut run_start = first;
    for index in 1..values.len() {
        let (current, previous) = (values[index], values[index - 1]);
        if current == previous + 1 {
            continue;
        }
        parts.push(if run_start == previous { run_start.to_string() } else { format!("{run_start}-{previous}") });
        run_start = current;
    }
    parts.push(if run_start == last { run_start.to_string() } else { format!("{run_start}-{last}") });
    parts.join(",")
}

/// The values a `*/step` walk visits, with the day-of-week fold applied.
fn star_walk(spec: Spec, step: i64) -> Vec<i64> {
    let mut walked = std::collections::BTreeSet::new();
    let mut value = spec.min;
    while value <= spec.max {
        walked.insert(if spec.canonical_max != spec.max && value == spec.max { spec.min } else { value });
        value += step;
    }
    walked.into_iter().collect()
}

/// Encode a field that began with `*`, keeping the leading `*`.
fn encode_star(values: &[i64], spec: Spec) -> Result<String, ScheduleError> {
    if values.len() as i64 == spec.canonical_max - spec.min + 1 {
        return Ok("*".to_string());
    }
    let present: std::collections::BTreeSet<i64> = values.iter().copied().collect();
    let mut best: Option<(i64, Vec<i64>)> = None;
    for step in 1..=(spec.max - spec.min + 1) {
        let walk = star_walk(spec, step);
        let longer = best.as_ref().is_none_or(|(_, kept)| walk.len() > kept.len());
        if longer && walk.iter().all(|value| present.contains(value)) {
            best = Some((step, walk));
        }
    }
    let Some((step, walk)) = best else {
        return Err(rule(format!("{} cannot keep a leading `*`.", spec.name)));
    };
    let walked: std::collections::BTreeSet<i64> = walk.into_iter().collect();
    let remaining: Vec<i64> = values.iter().copied().filter(|value| !walked.contains(value)).collect();
    Ok(if remaining.is_empty() { format!("*/{step}") } else { format!("*/{step},{}", encode(&remaining)) })
}

fn parse_field(raw: &str, spec: Spec) -> Result<Field, ScheduleError> {
    let values = field_values(raw, spec)?;
    let star = raw.starts_with('*');
    let canonical = if star { encode_star(&values, spec)? } else { encode(&values) };
    Ok(Field { values, canonical, star })
}

/// Parse and canonicalise `expression`.
pub fn parse(expression: &str) -> Result<Cron, ScheduleError> {
    if expression.is_empty() || expression.trim() != expression {
        return Err(rule("cron.expression must be a non-empty trimmed string.".to_string()));
    }
    let fields: Vec<&str> = expression.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(rule(
            "cron.expression must contain exactly five whitespace-separated fields: minute hour day-of-month month day-of-week."
                .to_string(),
        ));
    }
    let mut parsed = Vec::with_capacity(5);
    for (field, spec) in fields.iter().zip(FIELDS) {
        parsed.push(parse_field(field, spec)?);
    }
    let canonical = parsed.iter().map(|field| field.canonical.as_str()).collect::<Vec<_>>().join(" ");
    let [minute, hour, day_of_month, month, day_of_week]: [Field; 5] =
        parsed.try_into().unwrap_or_else(|_| unreachable!("five fields were parsed"));
    Ok(Cron {
        expression: canonical,
        minutes: minute.values,
        hours: hour.values,
        days_of_month: day_of_month.values,
        months: month.values,
        days_of_week: day_of_week.values,
        day_of_month_star: day_of_month.star,
        day_of_week_star: day_of_week.star,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canon(text: &str) -> String {
        parse(text).unwrap_or_else(|e| panic!("{text}: {e:?}")).expression
    }

    #[test]
    fn canonical_text_collapses_runs_steps_and_stars() {
        assert_eq!(canon("* * * * *"), "* * * * *");
        assert_eq!(canon("*/15 9-17 * * 1-5"), "*/15 9-17 * * 1-5");
        assert_eq!(canon("0,1,2,3 * * * *"), "0-3 * * * *");
        assert_eq!(canon("0,15,30,45 * * * *"), "0-45/15 * * * *");
        assert_eq!(canon("5,6,7,20 * * * *"), "5-7,20 * * * *");
        assert_eq!(canon("1 2 3 4 5"), "1 2 3 4 5");
        assert_eq!(canon("0 0 1 1 *"), "0 0 1 1 *");
    }

    #[test]
    fn sunday_is_zero_or_seven_and_folds_to_zero() {
        assert_eq!(canon("0 0 * * 7"), "0 0 * * 0");
        assert_eq!(canon("0 0 * * 0,7"), "0 0 * * 0");
        assert_eq!(canon("0 0 * * 5-7"), "0 0 * * 0,5-6");
        assert_eq!(canon("0 0 * * */2"), "0 0 * * */2");
    }

    #[test]
    fn whitespace_between_fields_may_vary_but_the_edges_may_not() {
        assert_eq!(canon("0   9\t*  * 1"), "0 9 * * 1");
        assert!(parse(" 0 9 * * 1").is_err());
        assert!(parse("0 9 * * 1 ").is_err());
        assert!(parse("").is_err());
    }

    #[test]
    fn field_count_values_ranges_and_steps_are_checked() {
        for bad in [
            "* * * *", "* * * * * *", "60 * * * *", "* 24 * * *", "* * 0 * *", "* * 32 * *", "* * * 0 *", "* * * 13 *",
            "* * * * 8", "5-1 * * * *", "*/0 * * * *", "1/5 * * * *", "a * * * *", "1,,2 * * * *", ",1 * * * *", "1- * * * *",
            "*-5 * * * *", "*/ * * * *", "*/x * * * *", "1-2-3 * * * *", "-1 * * * *",
        ] {
            let failure = parse(bad).expect_err(bad);
            assert_eq!(failure.code(), "invalid_rule", "{bad}");
        }
    }

    #[test]
    fn error_texts_match_the_typescript_host() {
        let message = |text: &str| parse(text).unwrap_err().message().to_string();
        assert_eq!(message("60 * * * *"), "cron.expression minute field value 60 is outside 0-59.");
        assert_eq!(message("5-1 * * * *"), "cron.expression minute field range 5-1 is inverted.");
        assert_eq!(message("*/0 * * * *"), "cron.expression minute field step must be a positive integer.");
        assert_eq!(message("1,,2 * * * *"), "cron.expression minute field must not contain an empty list element.");
        assert_eq!(
            message("x * * * *"),
            "cron.expression minute field element \"x\" must be *, a value, a-b, */n, a-b/n, or a comma-separated list of those."
        );
        assert_eq!(
            message("* * * *"),
            "cron.expression must contain exactly five whitespace-separated fields: minute hour day-of-month month day-of-week."
        );
        assert_eq!(message(" * * * * *"), "cron.expression must be a non-empty trimmed string.");
    }

    #[test]
    fn star_flags_follow_the_raw_text() {
        let both = parse("0 0 * * *").unwrap();
        assert!(both.day_of_month_star && both.day_of_week_star);
        let one = parse("0 0 1 * 1").unwrap();
        assert!(!one.day_of_month_star && !one.day_of_week_star);
        assert!(parse("0 0 */2 * 1-5").unwrap().day_of_month_star);
    }

    #[test]
    fn a_canonical_expression_parses_to_itself() {
        for text in ["*/15 9-17 * * 1-5", "0-45/15 * * * *", "5-7,20 * * 1,3-5 0,5-6", "*/2,7 * * * *"] {
            let once = canon(text);
            assert_eq!(canon(&once), once, "{text}");
        }
    }

    /// Every expression in the corpus gets the TypeScript host's answer: its
    /// canonical text, or its error code and message. The corpus was produced
    /// by `canonicalizeCronExpression` itself.
    #[test]
    fn the_corpus_canonicalises_exactly_as_typescript_does() {
        let corpus: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(include_str!("../../tests/fixtures/cron_canonical_ts.json")).unwrap();
        assert!(corpus.len() > 400);
        let mut wrong = Vec::new();
        for (input, expected) in &corpus {
            let got = parse(input);
            let matches = match (&got, expected.get("ok")) {
                (Ok(cron), Some(ok)) => ok.as_str() == Some(cron.expression.as_str()),
                (Err(e), None) => {
                    expected["code"].as_str() == Some(e.code()) && expected["message"].as_str() == Some(e.message())
                }
                _ => false,
            };
            if !matches {
                wrong.push(format!("{input:?}\n   ts: {expected}\n   rs: {got:?}"));
            }
        }
        assert!(wrong.is_empty(), "{} of {} differ:\n{}", wrong.len(), corpus.len(), wrong.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    }
}
