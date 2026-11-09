//! ISO 8601 timestamp utilities for the 900CRM application.
//!
//! All timestamps stored in the database are RFC 3339 / ISO 8601 strings
//! (e.g. `"2024-03-15T09:30:00Z"`). This module provides helpers for:
//!
//! - Generating the current UTC timestamp: [`now_iso8601`].
//! - Parsing an ISO 8601 string into a [`chrono::DateTime`]: [`parse_iso8601`].
//! - Formatting a timestamp for display: [`format_date`].
//! - Generating human-readable relative times: [`format_relative`].
//!
//! Using string timestamps (rather than Unix integers) makes the SQLite
//! records human-readable and simplifies cross-platform sync comparison.

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};

use crate::utils::errors::{CrmError, CrmResult};

// ─────────────────────────────────────────────────────────────────────────────
// Timestamp generation
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the current UTC time as an RFC 3339 string.
///
/// The returned string is always in UTC, formatted as
/// `"2024-03-15T09:30:00.123456789Z"`.
///
/// Use this whenever a `created_at`, `updated_at`, or `timestamp` column
/// needs to be set.
///
/// # Example
///
/// ```rust,ignore
/// use crate::utils::datetime::now_iso8601;
///
/// let ts = now_iso8601();
/// // "2024-03-15T09:30:00.123456789Z"
/// ```
pub fn now_iso8601() -> String {
    Utc::now().to_rfc3339()
}

/// Machine-local calendar day (`YYYY-MM-DD`).
///
/// Date-only activity dues are entered as a local calendar date. Compare them
/// to this value, not `now_iso8601()[..10]`, or a UTC+ clinic after midnight
/// (and a UTC- clinic in the evening) will disagree with the desktop UI.
pub fn local_today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn local_today_ymd() -> String {
    local_today().format("%Y-%m-%d").to_string()
}

pub fn local_days_from_now_ymd(days: i64) -> String {
    (local_today() + chrono::Duration::days(days))
        .format("%Y-%m-%d")
        .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsing
// ─────────────────────────────────────────────────────────────────────────────

/// Parses an ISO 8601 / RFC 3339 string into a [`DateTime<Utc>`].
///
/// Accepts any RFC 3339 string including timezone offsets (e.g. `+01:00`),
/// converting to UTC automatically.
///
/// # Errors
///
/// Returns [`CrmError::InvalidInput`] if `s` is not a valid RFC 3339 string.
///
/// # Example
///
/// ```rust,ignore
/// use crate::utils::datetime::parse_iso8601;
///
/// let dt = parse_iso8601("2024-03-15T09:30:00Z").unwrap();
/// ```
pub fn parse_iso8601(s: &str) -> CrmResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| CrmError::InvalidInput(format!("Invalid ISO 8601 timestamp '{}': {}", s, e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// Display formatting
// ─────────────────────────────────────────────────────────────────────────────

/// Formats an ISO 8601 timestamp string for human-readable display.
///
/// The `format` parameter uses [`chrono` format specifiers][chrono-fmt].
/// Common formats:
///
/// - `"%Y-%m-%d"` → `"2024-03-15"`
/// - `"%d %b %Y"` → `"15 Mar 2024"`
/// - `"%Y-%m-%d %H:%M"` → `"2024-03-15 09:30"`
///
/// Returns the original string unchanged if parsing fails, so display
/// code never panics on bad data.
///
/// [chrono-fmt]: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
///
/// # Example
///
/// ```rust,ignore
/// use crate::utils::datetime::format_date;
///
/// let display = format_date("2024-03-15T09:30:00Z", "%d %b %Y");
/// assert_eq!(display, "15 Mar 2024");
/// ```
pub fn format_date(iso8601: &str, format: &str) -> String {
    match parse_iso8601(iso8601) {
        Ok(dt) => dt.format(format).to_string(),
        Err(_) => iso8601.to_string(),
    }
}

/// Parses a date-only string `"YYYY-MM-DD"` and returns a [`NaiveDate`].
///
/// Used when dealing with `expected_close` and `due_date` fields that may be
/// stored as date-only strings rather than full ISO timestamps.
///
/// # Errors
///
/// Returns [`CrmError::InvalidInput`] if the string is not in `YYYY-MM-DD`
/// format.
pub fn parse_date_only(s: &str) -> CrmResult<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| CrmError::InvalidInput(format!("Invalid date '{}': {}", s, e)))
}

/// Parses an activity due value that may be date-only or a datetime.
///
/// Accepts:
/// - `YYYY-MM-DD` (treated as the start of that UTC day for parsing only)
/// - RFC 3339 / ISO 8601 timestamps
/// - naive `YYYY-MM-DDTHH:MM` or `YYYY-MM-DDTHH:MM:SS`
pub fn parse_due_datetime(s: &str) -> CrmResult<DateTime<Utc>> {
    let trimmed = s.trim();
    if trimmed.len() == 10 {
        let date = parse_date_only(trimmed)?;
        return date
            .and_hms_opt(0, 0, 0)
            .map(|naive| naive.and_utc())
            .ok_or_else(|| CrmError::InvalidInput(format!("Invalid date '{}'", trimmed)));
    }

    if let Ok(dt) = parse_iso8601(trimmed) {
        return Ok(dt);
    }

    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Ok(naive.and_utc());
    }

    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M") {
        return Ok(naive.and_utc());
    }

    Err(CrmError::InvalidInput(format!(
        "Invalid due datetime '{}'",
        trimmed
    )))
}

/// Generates a human-readable relative time string from an ISO 8601 timestamp.
///
/// Computes the difference between `iso8601` and the current UTC time, and
/// returns a phrase like:
///
/// - `"just now"` (< 60 seconds)
/// - `"5 minutes ago"` / `"in 5 minutes"`
/// - `"2 hours ago"` / `"in 2 hours"`
/// - `"yesterday"` / `"tomorrow"`
/// - `"3 days ago"` / `"in 3 days"`
/// - `"2 weeks ago"` / `"in 2 weeks"`
/// - `"4 months ago"` / `"in 4 months"`
/// - `"1 year ago"` / `"in 1 year"`
///
/// Falls back to `format_date(iso8601, "%d %b %Y")` if parsing fails.
///
/// # Example
///
/// ```rust,ignore
/// use crate::utils::datetime::format_relative;
///
/// // Assuming "now" is 2024-03-15T09:30:00Z
/// let rel = format_relative("2024-03-15T09:25:00Z");
/// assert_eq!(rel, "5 minutes ago");
/// ```
pub fn format_relative(iso8601: &str) -> String {
    let Ok(dt) = parse_iso8601(iso8601) else {
        return format_date(iso8601, "%d %b %Y");
    };

    let now = Utc::now();
    let diff = now.signed_duration_since(dt);
    let secs = diff.num_seconds();

    let (past, abs_secs) = if secs >= 0 {
        (true, secs)
    } else {
        (false, -secs)
    };

    let phrase = if abs_secs < 60 {
        "just now".to_string()
    } else if abs_secs < 3600 {
        let mins = abs_secs / 60;
        if mins == 1 {
            "1 minute".to_string()
        } else {
            format!("{} minutes", mins)
        }
    } else if abs_secs < 86400 {
        let hours = abs_secs / 3600;
        if hours == 1 {
            "1 hour".to_string()
        } else {
            format!("{} hours", hours)
        }
    } else if abs_secs < 172800 {
        if past {
            return "yesterday".to_string();
        } else {
            return "tomorrow".to_string();
        }
    } else if abs_secs < 604800 {
        let days = abs_secs / 86400;
        format!("{} days", days)
    } else if abs_secs < 2_592_000 {
        let weeks = abs_secs / 604800;
        if weeks == 1 {
            "1 week".to_string()
        } else {
            format!("{} weeks", weeks)
        }
    } else if abs_secs < 31_536_000 {
        let months = abs_secs / 2_592_000;
        if months == 1 {
            "1 month".to_string()
        } else {
            format!("{} months", months)
        }
    } else {
        let years = abs_secs / 31_536_000;
        if years == 1 {
            "1 year".to_string()
        } else {
            format!("{} years", years)
        }
    };

    if abs_secs < 60 {
        phrase
    } else if past {
        format!("{} ago", phrase)
    } else {
        format!("in {}", phrase)
    }
}

/// Returns the ISO 8601 timestamp for N days from now (or in the past).
///
/// - Positive `days` → future timestamp.
/// - Negative `days` → past timestamp.
///
/// Used in tests and default due-date calculations.
pub fn days_from_now(days: i64) -> String {
    let dt = Utc::now() + chrono::Duration::days(days);
    dt.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::{local_today_ymd, parse_date_only, parse_due_datetime, parse_iso8601};

    #[test]
    fn parse_due_datetime_accepts_date_only_rfc3339_and_naive_local() {
        assert!(parse_date_only("2026-07-08").is_ok());
        assert!(parse_due_datetime("2026-07-08").is_ok());
        assert!(parse_iso8601("2026-07-08T15:00:00Z").is_ok());
        assert!(parse_due_datetime("2026-07-08T15:00:00Z").is_ok());
        assert!(parse_due_datetime("2026-07-08T15:00").is_ok());
        assert!(parse_due_datetime("not-a-date").is_err());
    }

    #[test]
    fn local_today_ymd_is_a_date_only_calendar_key() {
        let today = local_today_ymd();
        assert!(parse_date_only(&today).is_ok());
        assert_eq!(today.len(), 10);
        assert!(!today.contains('T'));
    }
}
