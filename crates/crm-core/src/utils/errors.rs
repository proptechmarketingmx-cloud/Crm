//! Centralized error handling for the 900CRM application.
//!
//! This module defines [`CrmError`], a unified error type covering all failure
//! modes that can occur in the application — from database failures to CSV parse
//! errors. All public functions that can fail return [`CrmResult<T>`], which is
//! a type alias for `Result<T, CrmError>`.
//!
//! # Design Decisions
//!
//! - Each variant carries a human-readable `String` message that can be shown
//!   directly in the UI or logged without further processing.
//! - Foreign error types (`rusqlite`, `std::io`, `csv`) are converted via `From`
//!   implementations so that `?` propagation works naturally.
//! - The `Serialize` impl encodes the error as `{ "kind": "...", "message": "..." }`
//!   so the TypeScript frontend can branch on `kind` for typed error handling.
//! - `Into<tauri::InvokeError>` is implemented so commands can return
//!   `Result<T, CrmError>` directly.

use serde::Serialize;
use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// Primary error type
// ─────────────────────────────────────────────────────────────────────────────

/// The primary error type for the 900CRM application.
///
/// Every public function that can fail returns [`CrmResult<T>`], which is
/// `Result<T, CrmError>`. Each variant corresponds to a distinct failure domain.
///
/// # Variants at a Glance
///
/// | Variant        | Typical Cause |
/// |----------------|---------------|
/// | `Database`     | rusqlite query or migration failure |
/// | `NotFound`     | A requested entity (contact, deal, …) does not exist |
/// | `InvalidInput` | Caller-provided data failed validation |
/// | `Sync`         | Changelog / sync-related failure |
/// | `Csv`          | CSV parse or write failure |
/// | `Io`           | `std::io` file-system failure |
///
/// # IPC Serialization
///
/// When serialized through Tauri's IPC layer:
///
/// ```json
/// { "kind": "Database", "message": "UNIQUE constraint failed: contacts.email" }
/// ```
#[derive(Debug, Clone)]
pub enum CrmError {
    /// An error originating from the SQLite storage layer (rusqlite).
    ///
    /// Covers query failures, schema migrations, constraint violations,
    /// and connection errors.
    Database(String),

    /// A requested entity was not found.
    ///
    /// For example: a contact ID that does not exist in the database,
    /// a deal that has already been deleted, or a tag that was never created.
    NotFound(String),

    /// The caller provided data that failed validation.
    ///
    /// For example: an empty required field, an invalid email address,
    /// a negative deal value, or an unknown pipeline stage.
    InvalidInput(String),

    /// An error related to the sync changelog subsystem.
    ///
    /// Covers failures to write sync records, version conflicts,
    /// or malformed change payloads.
    Sync(String),

    /// An error originating from CSV import or export operations.
    ///
    /// Covers parse failures, missing required columns, and encoding issues.
    Csv(String),

    /// An I/O error from the file system.
    ///
    /// Covers missing files, permission errors, disk-full conditions,
    /// and any other `std::io::Error`.
    Io(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Display
// ─────────────────────────────────────────────────────────────────────────────

impl fmt::Display for CrmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrmError::Database(msg) => write!(f, "Database error: {}", msg),
            CrmError::NotFound(msg) => write!(f, "Not found: {}", msg),
            CrmError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CrmError::Sync(msg) => write!(f, "Sync error: {}", msg),
            CrmError::Csv(msg) => write!(f, "CSV error: {}", msg),
            CrmError::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// std::error::Error
// ─────────────────────────────────────────────────────────────────────────────

impl std::error::Error for CrmError {}

// ─────────────────────────────────────────────────────────────────────────────
// Serde serialization for Tauri IPC
// ─────────────────────────────────────────────────────────────────────────────

/// Serializes a [`CrmError`] as `{ "kind": "...", "message": "..." }`.
///
/// The `kind` field matches the variant name (e.g. `"Database"`) so the
/// TypeScript frontend can use it for typed error handling without depending
/// on the exact message text.
impl Serialize for CrmError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let (kind, message) = match self {
            CrmError::Database(m) => ("Database", m.as_str()),
            CrmError::NotFound(m) => ("NotFound", m.as_str()),
            CrmError::InvalidInput(m) => ("InvalidInput", m.as_str()),
            CrmError::Sync(m) => ("Sync", m.as_str()),
            CrmError::Csv(m) => ("Csv", m.as_str()),
            CrmError::Io(m) => ("Io", m.as_str()),
        };

        let mut s = serializer.serialize_struct("CrmError", 2)?;
        s.serialize_field("kind", kind)?;
        s.serialize_field("message", message)?;
        s.end()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// From conversions for foreign error types
// ─────────────────────────────────────────────────────────────────────────────

/// Converts a [`rusqlite::Error`] into a [`CrmError::Database`].
///
/// Allows `rusqlite` operations to be propagated with `?` throughout the
/// storage layer without manual wrapping.
impl From<rusqlite::Error> for CrmError {
    fn from(err: rusqlite::Error) -> Self {
        CrmError::Database(err.to_string())
    }
}

/// Converts a [`std::io::Error`] into a [`CrmError::Io`].
///
/// Allows standard I/O operations to be propagated with `?` throughout
/// the codebase.
impl From<std::io::Error> for CrmError {
    fn from(err: std::io::Error) -> Self {
        CrmError::Io(err.to_string())
    }
}

/// Converts a [`csv::Error`] into a [`CrmError::Csv`].
///
/// Allows CSV parsing and writing operations to be propagated with `?`
/// throughout the import/export layer.
impl From<csv::Error> for CrmError {
    fn from(err: csv::Error) -> Self {
        CrmError::Csv(err.to_string())
    }
}

/// Converts a [`serde_json::Error`] into a [`CrmError::InvalidInput`].
///
/// Used in sync and settings where JSON parsing can fail on malformed input.
impl From<serde_json::Error> for CrmError {
    fn from(err: serde_json::Error) -> Self {
        CrmError::InvalidInput(format!("JSON error: {}", err))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Type alias
// ─────────────────────────────────────────────────────────────────────────────

/// Convenient type alias for `Result<T, CrmError>` used throughout the crate.
///
/// Import in any module that needs to return a fallible result:
///
/// ```rust,ignore
/// use crate::utils::errors::CrmResult;
///
/// fn get_contact(id: &str) -> CrmResult<Contact> {
///     // …
/// }
/// ```
pub type CrmResult<T> = Result<T, CrmError>;

// ─────────────────────────────────────────────────────────────────────────────
// Helper macros
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a [`CrmError::NotFound`] with a formatted message.
///
/// # Example
///
/// ```rust,ignore
/// return Err(not_found!("Contact with ID '{}' does not exist", id));
/// ```
#[macro_export]
macro_rules! not_found {
    ($($arg:tt)*) => {
        $crate::utils::errors::CrmError::NotFound(format!($($arg)*))
    };
}

/// Creates a [`CrmError::InvalidInput`] with a formatted message.
///
/// # Example
///
/// ```rust,ignore
/// return Err(invalid_input!("Email '{}' is not a valid address", email));
/// ```
#[macro_export]
macro_rules! invalid_input {
    ($($arg:tt)*) => {
        $crate::utils::errors::CrmError::InvalidInput(format!($($arg)*))
    };
}
