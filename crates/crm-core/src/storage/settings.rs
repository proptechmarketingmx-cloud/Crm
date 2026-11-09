//! Key-value settings store for 900CRM.
//!
//! The `settings` table acts as an application-wide preferences dictionary.
//! All keys and values are UTF-8 strings. Type conversion (bool, int, etc.) is
//! the responsibility of the caller.
//!
//! # Default Settings
//!
//! Seeded by the initial schema migration (`migrate_v1`):
//!
//! | Key            | Default Value  | Purpose |
//! |----------------|----------------|---------|
//! | `language`     | `"en"`         | UI language code |
//! | `currency`     | `"USD"`        | Default currency for deals |
//! | `theme`        | `"light"`      | UI color theme |
//! | `date_format`  | `"YYYY-MM-DD"` | Date display format |
//! | `sync_enabled` | `"false"`      | Whether sync is active |
//! | `sync_url`     | `""`           | Remote sync endpoint URL |
//! | `notifications_enabled` | `"true"` | Whether desktop reminders are enabled |
//! | `reminder_lead_minutes` | `"30"`   | Minutes before due time to trigger reminders |
//! | `email_integration_enabled` | `"false"` | Enables optional IMAP/SMTP tooling |
//! | `smtp_host`    | `""`           | SMTP server hostname |
//! | `smtp_port`    | `"587"`        | SMTP server port |
//! | `smtp_username`| `""`           | SMTP username |
//! | `smtp_password`| `""`           | SMTP password (local-only storage) |
//! | `smtp_from`    | `""`           | Default sender email address |
//! | `imap_host`    | `""`           | IMAP server hostname |
//! | `imap_port`    | `"993"`        | IMAP server port |
//! | `imap_username`| `""`           | IMAP username |
//! | `imap_password`| `""`           | IMAP password (local-only storage) |

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{datetime::now_iso8601, errors::CrmResult};

// ─────────────────────────────────────────────────────────────────────────────
// Domain structs
// ─────────────────────────────────────────────────────────────────────────────

/// A single application setting entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    /// The setting key (e.g. `"theme"`, `"language"`).
    pub key: String,

    /// The setting value as a string (e.g. `"dark"`, `"fr"`).
    pub value: String,

    /// ISO 8601 timestamp of the last update. Empty string for seeded defaults.
    pub updated_at: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Retrieves a single setting by key.
///
/// Returns `Ok(None)` if the key does not exist (rather than an error).
///
/// # Errors
///
/// Returns [`crate::utils::errors::CrmError::Database`] on SQL failure.
pub fn get_setting(conn: &Connection, key: &str) -> CrmResult<Option<Setting>> {
    let result = conn.query_row(
        "SELECT key, value, updated_at FROM settings WHERE key = ?1",
        params![key],
        |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        },
    );

    match result {
        Ok(setting) => Ok(Some(setting)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(crate::utils::errors::CrmError::Database(e.to_string())),
    }
}

/// Returns the string value for a setting key, or a fallback if not found.
///
/// Useful for reading settings with a compile-time default:
///
/// ```rust,ignore
/// let theme = get_setting_or_default(&conn, "theme", "light");
/// ```
///
/// # Errors
///
/// Returns an error only on SQL failure, not on missing key.
pub fn get_setting_or_default(conn: &Connection, key: &str, default: &str) -> CrmResult<String> {
    match get_setting(conn, key)? {
        Some(s) => Ok(s.value),
        None => Ok(default.to_string()),
    }
}

/// Inserts or replaces a setting value.
///
/// If the key already exists, the value and `updated_at` are overwritten.
/// If the key is new, a new row is inserted.
///
/// # Errors
///
/// Returns [`crate::utils::errors::CrmError::Database`] on SQL failure.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> CrmResult<Setting> {
    let now = now_iso8601();

    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
        params![key, value, now],
    )?;

    if key.ends_with("password") {
        log::debug!("set_setting key={} value=[redacted]", key);
    } else {
        log::debug!("set_setting key={} value={}", key, value);
    }

    Ok(Setting {
        key: key.to_string(),
        value: value.to_string(),
        updated_at: now,
    })
}

/// Returns all settings as a `Vec<Setting>`, ordered by key.
///
/// # Errors
///
/// Returns [`crate::utils::errors::CrmError::Database`] on SQL failure.
pub fn get_all_settings(conn: &Connection) -> CrmResult<Vec<Setting>> {
    let mut stmt = conn.prepare("SELECT key, value, updated_at FROM settings ORDER BY key ASC")?;

    let rows = stmt.query_map([], |row| {
        Ok(Setting {
            key: row.get(0)?,
            value: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;

    let settings: Vec<Setting> = rows.collect::<Result<Vec<_>, _>>()?;
    log::debug!("get_all_settings: {} keys", settings.len());
    Ok(settings)
}
