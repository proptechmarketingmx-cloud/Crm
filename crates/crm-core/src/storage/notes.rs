//! Notes CRUD operations for 900CRM.
//!
//! Notes are freeform text records attached to any entity type (`contact`,
//! `organization`, `deal`, `activity`). They use the same entity polymorphism pattern:
//! `entity_type` + `entity_id` identify the parent record.
//!
//! # Soft Delete
//!
//! Notes use the same soft-delete pattern as other entities.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

// ─────────────────────────────────────────────────────────────────────────────
// Domain structs
// ─────────────────────────────────────────────────────────────────────────────

/// A freeform text note attached to a CRM entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// UUID v4 primary key.
    pub id: String,

    /// Markdown or plain-text note content.
    pub content: String,

    /// Type of the parent entity: `"contact"`, `"deal"`, or `"activity"`.
    pub entity_type: String,

    /// UUID of the parent entity.
    pub entity_id: String,

    /// ISO 8601 creation timestamp.
    pub created_at: String,

    /// ISO 8601 last-update timestamp.
    pub updated_at: String,

    /// ISO 8601 soft-delete timestamp (`None` = active).
    pub deleted_at: Option<String>,

    /// ID of the device that created or last modified this record.
    pub device_id: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// CRUD
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new note attached to the specified entity.
///
/// # Parameters
///
/// - `conn` — SQLite connection.
/// - `content` — Note body (markdown or plain text).
/// - `entity_type` — `"contact"`, `"organization"`, `"deal"`, or `"activity"`.
/// - `entity_id` — UUID of the parent entity.
/// - `device_id` — Originating device UUID.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn create_note(
    conn: &Connection,
    content: &str,
    entity_type: &str,
    entity_id: &str,
    device_id: &str,
) -> CrmResult<Note> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO notes
            (id, content, body, entity_type, entity_id, created_at, updated_at, device_id)
        VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![id, content, entity_type, entity_id, now, now, device_id],
    )?;

    log::debug!("Created note id={} for {}:{}", id, entity_type, entity_id);
    get_note(conn, &id)
}

/// Retrieves a single note by UUID.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Note not found or soft-deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn get_note(conn: &Connection, id: &str) -> CrmResult<Note> {
    conn.query_row(
        r#"
        SELECT id, COALESCE(NULLIF(body, ''), content), entity_type, entity_id,
               created_at, updated_at, deleted_at, device_id
        FROM notes
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![id],
        row_to_note,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Note '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

/// Returns all active notes for a given entity (type + id).
///
/// Notes are ordered by `created_at` descending (newest first).
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn get_notes_for_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> CrmResult<Vec<Note>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, COALESCE(NULLIF(body, ''), content), entity_type, entity_id,
               created_at, updated_at, deleted_at, device_id
        FROM notes
        WHERE entity_type = ?1 AND entity_id = ?2 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![entity_type, entity_id], row_to_note)?;
    let notes: Vec<Note> = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!(
        "get_notes_for_entity {}:{}: {} results",
        entity_type,
        entity_id,
        notes.len()
    );
    Ok(notes)
}

/// Returns all active generic notes for supported parent entity types.
///
/// Notes are ordered by entity type, entity id, and creation time so exports are
/// deterministic while staying independent of generated note ids.
pub fn list_active_notes(conn: &Connection) -> CrmResult<Vec<Note>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, COALESCE(NULLIF(body, ''), content), entity_type, entity_id,
               created_at, updated_at, deleted_at, device_id
        FROM notes
        WHERE deleted_at IS NULL
          AND entity_type IN ('contact', 'organization', 'deal', 'activity')
        ORDER BY entity_type ASC, entity_id ASC, created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map([], row_to_note)?;
    let notes: Vec<Note> = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!("list_active_notes: {} results", notes.len());
    Ok(notes)
}

/// Updates a note's content.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Note not found or deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn update_note(conn: &Connection, id: &str, content: &str) -> CrmResult<Note> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE notes SET content = ?1, body = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
        params![content, now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Note '{}' not found", id)));
    }

    log::debug!("Updated note id={}", id);
    get_note(conn, id)
}

/// Soft-deletes a note.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Note not found or already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn soft_delete_note(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE notes SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Note '{}' not found or already deleted",
            id
        )));
    }

    log::info!("Soft-deleted note id={}", id);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Maps a `rusqlite::Row` to a [`Note`].
fn row_to_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        content: row.get(1)?,
        entity_type: row.get(2)?,
        entity_id: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        deleted_at: row.get(6)?,
        device_id: row.get(7)?,
    })
}
