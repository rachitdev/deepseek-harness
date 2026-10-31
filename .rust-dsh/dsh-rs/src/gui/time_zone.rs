//! Client time zones: `UTC` or an IANA `Area/Location` name.
//!
//! The rule is the TypeScript host's `canonicalClientTimeZone`: the name must be
//! exactly `UTC`, or match `[A-Za-z][A-Za-z0-9_+.-]*(/[A-Za-z0-9_+.-]+)+`, carry
//! no surrounding whitespace, and name a zone the bundled database knows. Names
//! are matched case-insensitively and answered in the spelling Node's ICU gives
//! (`Asia/Kolkata` is `Asia/Calcutta`), because the TypeScript host stores that
//! spelling and both hosts must write the same text.

/// The canonical spelling of `value`, or `None` when it is not an accepted zone.
pub fn canonical(value: &str) -> Option<String> {
    if value == "UTC" {
        return Some("UTC".to_string());
    }
    if !has_area_location_shape(value) {
        return None;
    }
    let database = jiff::tz::TimeZone::get(value).ok()?;
    let name = database.iana_name()?;
    // Etc/UTC and its kin are `UTC`, which is accepted without a slash.
    Some(alias(name).unwrap_or(name).to_string())
}

/// ICU's canonical spelling of a database name, when it differs.
fn alias(name: &str) -> Option<&'static str> {
    super::zone_aliases::ALIASES
        .binary_search_by(|(database, _)| (*database).cmp(name))
        .ok()
        .map(|at| super::zone_aliases::ALIASES[at].1)
}

/// `Area/Location[/More]`: an alphabetic first character, then name characters,
/// then at least one `/`-separated segment of name characters.
pub(crate) fn has_area_location_shape(value: &str) -> bool {
    let name_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '.' | '-');
    let mut segments = value.split('/');
    let Some(first) = segments.next() else {
        return false;
    };
    let mut first_chars = first.chars();
    let starts_alphabetic = first_chars.next().is_some_and(|c| c.is_ascii_alphabetic());
    if !starts_alphabetic || !first_chars.all(name_char) {
        return false;
    }
    let rest: Vec<&str> = segments.collect();
    !rest.is_empty() && rest.iter().all(|segment| !segment.is_empty() && segment.chars().all(name_char))
}

#[cfg(test)]
mod tests {
    use super::canonical;

    #[test]
    fn accepted_names_are_answered_in_icus_spelling() {
        assert_eq!(canonical("UTC").as_deref(), Some("UTC"));
        assert_eq!(canonical("Europe/Paris").as_deref(), Some("Europe/Paris"));
        assert_eq!(canonical("europe/paris").as_deref(), Some("Europe/Paris"));
        assert_eq!(canonical("Etc/GMT+5").as_deref(), Some("Etc/GMT+5"));
        // ICU's spelling, which the TypeScript host stores.
        assert_eq!(canonical("America/Argentina/Buenos_Aires").as_deref(), Some("America/Buenos_Aires"));
        assert_eq!(canonical("Asia/Kolkata").as_deref(), Some("Asia/Calcutta"));
        assert_eq!(canonical("US/Pacific").as_deref(), Some("America/Los_Angeles"));
        assert_eq!(canonical("Etc/UTC").as_deref(), Some("UTC"));
    }

    #[test]
    fn everything_else_is_refused() {
        for bad in [
            "", " UTC", "UTC ", "utc", "Utc", "GMT", "Z", "EST5EDT", "Europe", "Europe/", "/Paris",
            "Europe//Paris", "Nope/Zone", "Europe/Nowhere", "America/New_York ", " Europe/Paris", "1Europe/Paris",
            "Europe/Pa ris", "Europe\\Paris", "é/x",
        ] {
            assert_eq!(canonical(bad), None, "{bad:?}");
        }
    }
}
