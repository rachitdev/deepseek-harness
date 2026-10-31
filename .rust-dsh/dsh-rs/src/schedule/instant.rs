//! Instants, local calendar values and time zones, with the TypeScript
//! `util-time` rules.
//!
//! An instant is a count of milliseconds since the Unix epoch. Only the
//! four-digit-year range is representable on the wire: `0001-01-01T00:00:00.000Z`
//! through `9999-12-31T23:59:59.999Z`, always rendered in the UTC form
//! `YYYY-MM-DDTHH:mm:ss.SSSZ`.

use jiff::civil::{Date, DateTime, Time};
use jiff::tz::TimeZone;

/// `0001-01-01T00:00:00.000Z`.
pub const MIN_MS: i64 = -62_135_596_800_000;
/// `9999-12-31T23:59:59.999Z`.
pub const MAX_MS: i64 = 253_402_300_799_999;

/// Whether `ms` can be written as a four-digit-year instant.
pub fn in_range(ms: i64) -> bool {
    (MIN_MS..=MAX_MS).contains(&ms)
}

/// Why an instant string was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstantRejection {
    NotRfc3339,
    OutOfRange,
    NotACalendarDate,
    InvalidNumericOffset,
}

/// Why a local date and time were refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalRejection {
    Malformed,
    OutOfRange,
    NotACalendarDate,
}

/// Why a time zone name was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneRejection {
    Blank,
    Padded,
    NotAreaLocation,
    Unsupported,
}

/// A local wall-clock reading without a zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Local {
    pub year: i16,
    pub month: i8,
    pub day: i8,
    pub hour: i8,
    pub minute: i8,
    pub second: i8,
    pub millisecond: i16,
}

impl Local {
    /// The civil date-time, or `None` when the date does not exist.
    pub fn civil(self) -> Option<DateTime> {
        let date = Date::new(self.year, self.month, self.day).ok()?;
        let time = Time::new(self.hour, self.minute, self.second, i32::from(self.millisecond) * 1_000_000).ok()?;
        Some(date.to_datetime(time))
    }
}

/// Digits only, exactly `len` of them.
fn digits(text: &str, len: usize) -> Option<i32> {
    (text.len() == len && text.bytes().all(|b| b.is_ascii_digit())).then(|| text.parse().ok())?
}

fn fraction_ms(text: Option<&str>) -> Option<i16> {
    match text {
        None => Some(0),
        Some(fraction) if (1..=3).contains(&fraction.len()) && fraction.bytes().all(|b| b.is_ascii_digit()) => {
            format!("{fraction:0<3}").parse().ok()
        }
        Some(_) => None,
    }
}

/// `HH:mm:ss` with an optional one-to-three digit fraction.
fn clock(text: &str) -> Option<(i32, i32, i32, i16)> {
    let (whole, fraction) = match text.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (text, None),
    };
    let mut parts = whole.split(':');
    let (h, m, s) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    Some((digits(h, 2)?, digits(m, 2)?, digits(s, 2)?, fraction_ms(fraction)?))
}

/// `YYYY-MM-DD`.
fn calendar(text: &str) -> Option<(i32, i32, i32)> {
    let mut parts = text.split('-');
    let (y, m, d) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    Some((digits(y, 4)?, digits(m, 2)?, digits(d, 2)?))
}

/// A local date and time as the `at` object carries them.
pub fn parse_local(date: &str, time: &str) -> Result<Local, LocalRejection> {
    let (Some((year, month, day)), Some((hour, minute, second, millisecond))) = (calendar(date), clock(time)) else {
        return Err(LocalRejection::Malformed);
    };
    if year == 0 || hour > 23 || minute > 59 || second > 59 {
        return Err(LocalRejection::OutOfRange);
    }
    let local = Local {
        year: year as i16,
        month: month as i8,
        day: day as i8,
        hour: hour as i8,
        minute: minute as i8,
        second: second as i8,
        millisecond,
    };
    local.civil().map(|_| local).ok_or(LocalRejection::NotACalendarDate)
}

/// An RFC 3339 instant with a mandatory `Z` or numeric offset.
pub fn parse_offset_instant(value: &str) -> Result<i64, InstantRejection> {
    let (stamp, zone) = split_zone(value).ok_or(InstantRejection::NotRfc3339)?;
    let (date, time) = stamp.split_once('T').ok_or(InstantRejection::NotRfc3339)?;
    let (Some((year, month, day)), Some((hour, minute, second, millisecond))) = (calendar(date), clock(time)) else {
        return Err(InstantRejection::NotRfc3339);
    };
    if year == 0 || hour > 23 || minute > 59 || second > 59 {
        return Err(InstantRejection::OutOfRange);
    }
    let local = Local {
        year: year as i16,
        month: month as i8,
        day: day as i8,
        hour: hour as i8,
        minute: minute as i8,
        second: second as i8,
        millisecond,
    };
    let civil = local.civil().ok_or(InstantRejection::NotACalendarDate)?;
    let utc_ms = civil_to_utc_ms(civil);
    match zone {
        Zone::Utc => Ok(utc_ms),
        Zone::Offset { negative, hours, minutes } => {
            if hours > 23 || minutes > 59 || (negative && hours == 0 && minutes == 0) {
                return Err(InstantRejection::InvalidNumericOffset);
            }
            let direction = if negative { -1 } else { 1 };
            Ok(utc_ms - direction * (hours * 60 + minutes) * 60_000)
        }
    }
}

/// Milliseconds since the epoch of a civil reading taken as UTC. Plain
/// arithmetic, because a `Timestamp` stops short of the last day of 9999.
pub fn civil_to_utc_ms(civil: DateTime) -> i64 {
    let epoch_day = i64::from((civil.date() - Date::constant(1970, 1, 1)).get_days());
    let seconds = epoch_day * 86_400
        + i64::from(civil.hour()) * 3_600
        + i64::from(civil.minute()) * 60
        + i64::from(civil.second());
    seconds * 1_000 + i64::from(civil.subsec_nanosecond()) / 1_000_000
}

enum Zone {
    Utc,
    Offset { negative: bool, hours: i64, minutes: i64 },
}

/// Split the trailing `Z` or `±HH:mm` off an instant string.
fn split_zone(value: &str) -> Option<(&str, Zone)> {
    if let Some(stamp) = value.strip_suffix('Z') {
        return Some((stamp, Zone::Utc));
    }
    let at = value.len().checked_sub(6)?;
    if !value.is_char_boundary(at) {
        return None;
    }
    let (stamp, offset) = value.split_at(at);
    let negative = match offset.as_bytes().first()? {
        b'+' => false,
        b'-' => true,
        _ => return None,
    };
    let (h, m) = offset[1..].split_once(':')?;
    Some((stamp, Zone::Offset { negative, hours: i64::from(digits(h, 2)?), minutes: i64::from(digits(m, 2)?) }))
}

/// Parse a stored scheduled-at: exactly `YYYY-MM-DDTHH:mm:ss.SSSZ`, year 0001 to 9999.
///
/// A string of the wrong shape is `NotRfc3339`; one of the right shape that
/// names no real moment (`2026-02-30...`) is `NotACalendarDate`.
pub fn parse_stored_instant(value: &str) -> Result<i64, InstantRejection> {
    let bytes = value.as_bytes();
    let shaped = value.len() == 24
        && value.is_ascii()
        && value.ends_with('Z')
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'.'
        && [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18, 20, 21, 22]
            .iter()
            .all(|at| bytes[*at].is_ascii_digit())
        && &value[..4] != "0000"
        && value[5..7].parse::<u8>().is_ok_and(|month| (1..=12).contains(&month))
        && value[8..10].parse::<u8>().is_ok_and(|day| (1..=31).contains(&day))
        && value[11..13].parse::<u8>().is_ok_and(|hour| hour <= 23)
        && value[14..16].parse::<u8>().is_ok_and(|minute| minute <= 59)
        && value[17..19].parse::<u8>().is_ok_and(|second| second <= 59);
    if !shaped {
        return Err(InstantRejection::NotRfc3339);
    }
    let ms = parse_offset_instant(value).map_err(|_| InstantRejection::NotACalendarDate)?;
    if !in_range(ms) || format_instant(ms).as_deref() != Some(value) {
        return Err(InstantRejection::NotACalendarDate);
    }
    Ok(ms)
}

/// `YYYY-MM-DDTHH:mm:ss.SSSZ` for an in-range instant. Plain arithmetic, because
/// a `Timestamp` stops short of the last day of 9999.
pub fn format_instant(ms: i64) -> Option<String> {
    if !in_range(ms) {
        return None;
    }
    let (seconds, millis) = (ms.div_euclid(1_000), ms.rem_euclid(1_000));
    let (day, second_of_day) = (seconds.div_euclid(86_400), seconds.rem_euclid(86_400));
    let date = Date::constant(1970, 1, 1).checked_add(jiff::Span::new().days(day)).ok()?;
    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        date.year(),
        date.month(),
        date.day(),
        second_of_day / 3_600,
        second_of_day % 3_600 / 60,
        second_of_day % 60,
        millis
    ))
}

/// Resolve a zone name: `UTC` or an IANA `Area/Location`, no padding.
pub fn canonical_zone(value: &str) -> Result<String, ZoneRejection> {
    if value.is_empty() {
        return Err(ZoneRejection::Blank);
    }
    if value.trim() != value {
        return Err(ZoneRejection::Padded);
    }
    if value == "UTC" {
        return Ok("UTC".to_string());
    }
    if !crate::gui::time_zone::has_area_location_shape(value) {
        return Err(ZoneRejection::NotAreaLocation);
    }
    crate::gui::time_zone::canonical(value).ok_or(ZoneRejection::Unsupported)
}

/// The zone object for an already-canonical name.
pub fn zone(name: &str) -> Option<TimeZone> {
    TimeZone::get(name).ok()
}

/// The instant of a local reading in `zone`, taking the earlier of two
/// repeated readings, or `None` when the reading falls in a gap.
pub fn resolve_local(local: DateTime, zone: &TimeZone) -> Option<i64> {
    let zoned = zone.to_ambiguous_zoned(local).earlier().ok()?;
    // A reading in a gap is moved forward by `earlier`; it only exists when it round-trips.
    (zoned.datetime() == local).then(|| zoned.timestamp().as_millisecond())
}

/// The calendar date an instant falls on in `zone`.
pub fn local_date(ms: i64, zone: &TimeZone) -> Option<Date> {
    Some(jiff::Timestamp::from_millisecond(ms).ok()?.to_zoned(zone.clone()).date())
}
