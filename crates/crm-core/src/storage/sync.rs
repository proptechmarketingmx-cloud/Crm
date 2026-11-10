//! Offline-first sync changelog for 900CRM.
//!
//! The `sync_changelog` table is an append-only log of every mutation. Each
//! row records the entity type, entity ID, changed field, old value, new value,
//! timestamp, and originating device ID.
//!
//! # Design
//!
//! - `record_change` must be called after every successful mutation.
//! - `get_changes_since` is used by the sync engine to fetch changes to push.
//! - Pending means `synced_at IS NULL` once that column is used.
//! - `mark_changes_synced` and `clear_old_changes` keep the table manageable.
//!
//! # Sync Strategy
//!
//! 900CRM uses a last-write-wins CRDT at the field level. The timestamp +
//! device_id pair breaks ties when two devices modify the same field
//! simultaneously.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
};

// ─────────────────────────────────────────────────────────────────────────────
// Domain structs
// ─────────────────────────────────────────────────────────────────────────────

/// A single entry in the sync changelog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncChange {
    /// Auto-increment integer primary key.
    pub id: i64,

    /// Entity type (e.g. `"contact"`, `"deal"`, `"activity"`).
    pub entity_type: String,

    /// UUID of the changed entity.
    pub entity_id: String,

    /// Name of the changed field (e.g. `"email"`, `"stage"`, `"__create__"`).
    pub field_name: String,

    /// Previous value (`None` for new entities).
    pub old_value: Option<String>,

    /// New value (`None` for deletions).
    pub new_value: Option<String>,

    /// ISO 8601 timestamp of the change.
    pub timestamp: String,

    /// UUID of the device that made the change.
    pub device_id: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Records a single field-level change in the sync changelog.
///
/// Call this after every successful database mutation. Pass `"__create__"` as
/// `field_name` when creating a new entity, and `"__delete__"` when
/// soft-deleting.
///
/// # Parameters
///
/// - `conn` — SQLite connection.
/// - `entity_type` — Type of the changed entity (e.g. `"contact"`).
/// - `entity_id` — UUID of the changed entity.
/// - `field_name` — Name of the changed field, or `"__create__"` / `"__delete__"`.
/// - `old_value` — Previous value (`None` for new entities).
/// - `new_value` — New value (`None` for deletions).
/// - `device_id` — UUID of the originating device.
///
/// # Errors
///
/// Returns [`CrmError::Sync`] if the insert fails.
#[allow(clippy::too_many_arguments)]
pub fn record_change(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    field_name: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    device_id: &str,
) -> CrmResult<()> {
    let timestamp = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO sync_changelog
            (entity_type, entity_id, field_name, old_value, new_value, timestamp, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            entity_type,
            entity_id,
            field_name,
            old_value,
            new_value,
            timestamp,
            device_id
        ],
    )
    .map_err(|e| CrmError::Sync(format!("Failed to record change: {}", e)))?;

    log::debug!(
        "sync::record_change entity={}:{} field={}",
        entity_type,
        entity_id,
        field_name
    );
    Ok(())
}

/// Returns all changelog entries since `since_timestamp` (exclusive) that
/// were NOT made by `exclude_device_id`.
///
/// Used to retrieve changes to push to a remote server or pull to another device.
///
/// # Parameters
///
/// - `since_timestamp` — ISO 8601 timestamp; only changes strictly after this
///   are returned. Pass `""` or `"1970-01-01T00:00:00Z"` to get all changes.
/// - `exclude_device_id` — Skip changes originating from this device (to avoid
///   re-applying our own changes).
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn get_changes_since(
    conn: &Connection,
    since_timestamp: &str,
    exclude_device_id: &str,
) -> CrmResult<Vec<SyncChange>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, entity_type, entity_id, field_name, old_value, new_value, timestamp, device_id
        FROM sync_changelog
        WHERE timestamp > ?1 AND device_id != ?2
        ORDER BY timestamp ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![since_timestamp, exclude_device_id], |row| {
        row_to_sync_change(row)
    })?;

    let changes = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!(
        "get_changes_since since={}: {} changes",
        since_timestamp,
        changes.len()
    );
    Ok(changes)
}

/// Returns changelog entries that have not been marked synced.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn get_all_pending_changes(conn: &Connection) -> CrmResult<Vec<SyncChange>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, entity_type, entity_id, field_name, old_value, new_value, timestamp, device_id
        FROM sync_changelog
        WHERE synced_at IS NULL
        ORDER BY timestamp ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map([], row_to_sync_change)?;
    let changes = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!("get_all_pending_changes: {} entries", changes.len());
    Ok(changes)
}

/// Marks pending changelog rows as synced.
pub fn mark_changes_synced(conn: &Connection) -> CrmResult<u64> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE sync_changelog SET synced_at = ?1 WHERE synced_at IS NULL",
        params![now],
    )?;
    Ok(changed as u64)
}

/// Permanently deletes changelog entries older than `before_timestamp`.
///
/// Call this periodically to prevent the changelog from growing indefinitely.
/// Only delete entries that have already been successfully synced.
///
/// # Parameters
///
/// - `before_timestamp` — ISO 8601 cutoff; entries strictly before this are deleted.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn clear_old_changes(conn: &Connection, before_timestamp: &str) -> CrmResult<u64> {
    let deleted = conn.execute(
        "DELETE FROM sync_changelog WHERE timestamp < ?1 AND synced_at IS NOT NULL",
        params![before_timestamp],
    )?;

    log::info!(
        "clear_old_changes: deleted {} entries before {}",
        deleted,
        before_timestamp
    );
    Ok(deleted as u64)
}

/// Elimina cambios sincronizados vencidos y limita el historial por entidad.
///
/// Los cambios pendientes (`synced_at IS NULL`) nunca se eliminan. La poda
/// conserva como máximo `max_per_entity` cambios recientes por entidad,
/// además de eliminar cambios sincronizados anteriores a la fecha límite.
pub fn prune_synced_changes(
    conn: &Connection,
    before_timestamp: &str,
    max_per_entity: u64,
) -> CrmResult<u64> {
    if max_per_entity == 0 {
        return Err(CrmError::Database(
            "The changelog retention limit must be greater than zero".to_string(),
        ));
    }
    let max_per_entity = i64::try_from(max_per_entity).map_err(|_| {
        CrmError::Database("The changelog retention limit is too large".to_string())
    })?;
    let deleted = conn.execute(
        r#"
        DELETE FROM sync_changelog
        WHERE synced_at IS NOT NULL
          AND (
            timestamp < ?1
            OR id IN (
                SELECT id
                FROM (
                    SELECT id,
                           ROW_NUMBER() OVER (
                               PARTITION BY entity_type, entity_id
                               ORDER BY timestamp DESC, id DESC
                           ) AS retention_rank
                    FROM sync_changelog
                    WHERE synced_at IS NOT NULL
                )
                WHERE retention_rank > ?2
            )
          )
        "#,
        params![before_timestamp, max_per_entity],
    )?;
    let remaining: u64 =
        conn.query_row("SELECT COUNT(*) FROM sync_changelog", [], |row| row.get(0))?;
    const CHANGELOG_WARNING_THRESHOLD: u64 = 100_000;
    if remaining > CHANGELOG_WARNING_THRESHOLD {
        log::warn!(
            "sync_changelog contains {} rows, above the {} row threshold",
            remaining,
            CHANGELOG_WARNING_THRESHOLD
        );
    } else {
        log::info!(
            "sync_changelog pruning completed: deleted={}, remaining={}",
            deleted,
            remaining
        );
    }
    Ok(deleted as u64)
}

/// Returns the timestamp of the most recent changelog entry.
///
/// Returns `None` if the changelog is empty.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn get_latest_change_timestamp(conn: &Connection) -> CrmResult<Option<String>> {
    let result = conn.query_row("SELECT MAX(timestamp) FROM sync_changelog", [], |row| {
        row.get::<_, Option<String>>(0)
    });

    match result {
        Ok(ts) => Ok(ts),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(CrmError::Database(e.to_string())),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Maps a `rusqlite::Row` to a [`SyncChange`].
fn row_to_sync_change(row: &rusqlite::Row<'_>) -> rusqlite::Result<SyncChange> {
    Ok(SyncChange {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        entity_id: row.get(2)?,
        field_name: row.get(3)?,
        old_value: row.get(4)?,
        new_value: row.get(5)?,
        timestamp: row.get(6)?,
        device_id: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::prune_synced_changes;

    #[test]
    fn pruning_preserves_pending_rows_and_recent_rows_per_entity() {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        conn.execute_batch(
            r#"
            CREATE TABLE sync_changelog (
                id INTEGER PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                synced_at TEXT
            );
            INSERT INTO sync_changelog VALUES
                (1, 'contact', 'a', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
                (2, 'contact', 'a', '2024-02-01T00:00:00Z', '2024-02-01T00:00:00Z'),
                (3, 'contact', 'a', '2024-03-01T00:00:00Z', '2024-03-01T00:00:00Z'),
                (4, 'contact', 'a', '2024-04-01T00:00:00Z', NULL),
                (5, 'contact', 'b', '2024-01-01T00:00:00Z', NULL),
                (6, 'contact', 'c', '2024-03-10T00:00:00Z', '2024-03-10T00:00:00Z'),
                (7, 'contact', 'c', '2024-03-11T00:00:00Z', '2024-03-11T00:00:00Z'),
                (8, 'contact', 'c', '2024-03-12T00:00:00Z', '2024-03-12T00:00:00Z');
            "#,
        )
        .expect("create test changelog");

        let deleted =
            prune_synced_changes(&conn, "2024-02-15T00:00:00Z", 2).expect("prune changelog");

        assert_eq!(deleted, 3);
        let remaining: u64 = conn
            .query_row("SELECT COUNT(*) FROM sync_changelog", [], |row| row.get(0))
            .expect("count remaining rows");
        assert_eq!(remaining, 5);
        let old_synced: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sync_changelog WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .expect("check deleted old row");
        assert_eq!(old_synced, 0);
        let pending: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sync_changelog WHERE synced_at IS NULL",
                [],
                |row| row.get(0),
            )
            .expect("count pending rows");
        assert_eq!(pending, 2);
        let overflow_synced: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sync_changelog WHERE id = 6",
                [],
                |row| row.get(0),
            )
            .expect("check row beyond per-entity limit");
        assert_eq!(overflow_synced, 0);
    }
}
