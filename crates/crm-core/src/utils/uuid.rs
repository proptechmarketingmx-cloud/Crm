//! UUID v4 generation helpers.
//!
//! This module provides a thin wrapper around the [`uuid`] crate to generate
//! RFC 4122 version-4 (random) UUID strings used as primary keys throughout
//! the 900CRM database schema.
//!
//! All entity IDs (`contacts.id`, `deals.id`, `activities.id`, etc.) are UUID
//! v4 strings formatted as lowercase hyphenated hex, e.g.:
//!
//! ```text
//! "550e8400-e29b-41d4-a716-446655440000"
//! ```
//!
//! Using UUIDs rather than auto-increment integers is critical for the
//! offline-first sync model: records created on different devices can be
//! merged without ID collisions.

use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Generates a new UUID v4 string.
///
/// Returns a randomly-generated UUID formatted as a lowercase, hyphen-separated
/// string (the standard canonical form, e.g. `"a1b2c3d4-…"`).
///
/// # Example
///
/// ```rust,ignore
/// use crate::utils::uuid::new_uuid;
///
/// let id = new_uuid();
/// assert_eq!(id.len(), 36);
/// ```
pub fn new_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Validates that a string is a valid UUID (any version).
///
/// Returns `true` if `s` can be parsed as a UUID, `false` otherwise.
/// Used in validation layers to reject malformed IDs before they reach
/// the database.
///
/// # Example
///
/// ```rust,ignore
/// use crate::utils::uuid::is_valid_uuid;
///
/// assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
/// assert!(!is_valid_uuid("not-a-uuid"));
/// ```
pub fn is_valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}
