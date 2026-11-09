//! Named list filters that survive app restarts.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub filters_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub device_id: String,
}

pub fn create_saved_view(
    conn: &Connection,
    entity_type: &str,
    name: &str,
    filters_json: &str,
    device_id: &str,
) -> CrmResult<SavedView> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO saved_views
            (id, entity_type, name, filters_json, created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![id, entity_type, name, filters_json, now, now, device_id],
    )?;

    get_saved_view(conn, &id)
}

pub fn get_saved_view(conn: &Connection, id: &str) -> CrmResult<SavedView> {
    conn.query_row(
        r#"
        SELECT id, entity_type, name, filters_json, created_at, updated_at, deleted_at, device_id
        FROM saved_views
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![id],
        row_to_saved_view,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Saved view '{id}' not found"))
        }
        other => CrmError::Database(other.to_string()),
    })
}

pub fn list_saved_views(conn: &Connection, entity_type: &str) -> CrmResult<Vec<SavedView>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, entity_type, name, filters_json, created_at, updated_at, deleted_at, device_id
        FROM saved_views
        WHERE entity_type = ?1 AND deleted_at IS NULL
        ORDER BY name COLLATE NOCASE ASC, created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![entity_type], row_to_saved_view)?;
    Ok(rows.filter_map(|row| row.ok()).collect())
}

pub fn find_active_saved_view_by_name(
    conn: &Connection,
    entity_type: &str,
    name: &str,
) -> CrmResult<Option<SavedView>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, entity_type, name, filters_json, created_at, updated_at, deleted_at, device_id
        FROM saved_views
        WHERE entity_type = ?1
          AND deleted_at IS NULL
          AND LOWER(TRIM(name)) = LOWER(TRIM(?2))
        LIMIT 1
        "#,
    )?;

    let mut rows = stmt.query_map(params![entity_type, name], row_to_saved_view)?;
    Ok(rows.next().transpose()?)
}

pub fn update_saved_view(
    conn: &Connection,
    id: &str,
    name: &str,
    filters_json: &str,
) -> CrmResult<SavedView> {
    let now = now_iso8601();
    let changed = conn.execute(
        r#"
        UPDATE saved_views
        SET name = ?1, filters_json = ?2, updated_at = ?3
        WHERE id = ?4 AND deleted_at IS NULL
        "#,
        params![name, filters_json, now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Saved view '{id}' not found")));
    }

    get_saved_view(conn, id)
}

pub fn soft_delete_saved_view(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE saved_views SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Saved view '{id}' not found or already deleted"
        )));
    }

    Ok(())
}

fn row_to_saved_view(row: &rusqlite::Row<'_>) -> rusqlite::Result<SavedView> {
    Ok(SavedView {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        name: row.get(2)?,
        filters_json: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        deleted_at: row.get(6)?,
        device_id: row.get(7)?,
    })
}
