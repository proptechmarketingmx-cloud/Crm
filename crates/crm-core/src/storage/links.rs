//! Entity link CRUD for 900CRM.
//!
//! Links are bookmarks attached to a contact, organization, or deal. They store
//! either an http(s) URL or a local file path as text. 900CRM does not copy or
//! upload files.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLink {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub kind: String,
    pub target: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub device_id: String,
}

pub fn create_link(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    title: &str,
    kind: &str,
    target: &str,
    device_id: &str,
) -> CrmResult<EntityLink> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO entity_links
            (id, entity_type, entity_id, title, kind, target, created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            id,
            entity_type,
            entity_id,
            title,
            kind,
            target,
            now,
            now,
            device_id
        ],
    )?;

    get_link(conn, &id)
}

pub fn get_link(conn: &Connection, id: &str) -> CrmResult<EntityLink> {
    conn.query_row(
        r#"
        SELECT id, entity_type, entity_id, title, kind, target,
               created_at, updated_at, deleted_at, device_id
        FROM entity_links
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![id],
        row_to_link,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Link '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

pub fn list_links_for_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> CrmResult<Vec<EntityLink>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, entity_type, entity_id, title, kind, target,
               created_at, updated_at, deleted_at, device_id
        FROM entity_links
        WHERE entity_type = ?1 AND entity_id = ?2 AND deleted_at IS NULL
        ORDER BY created_at DESC, id DESC
        "#,
    )?;

    let rows = stmt.query_map(params![entity_type, entity_id], row_to_link)?;
    Ok(rows.filter_map(|row| row.ok()).collect())
}

pub fn update_link(
    conn: &Connection,
    id: &str,
    title: &str,
    kind: &str,
    target: &str,
) -> CrmResult<EntityLink> {
    let now = now_iso8601();
    let changed = conn.execute(
        r#"
        UPDATE entity_links
        SET title = ?1, kind = ?2, target = ?3, updated_at = ?4
        WHERE id = ?5 AND deleted_at IS NULL
        "#,
        params![title, kind, target, now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Link '{}' not found", id)));
    }

    get_link(conn, id)
}

pub fn soft_delete_link(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE entity_links SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Link '{}' not found or already deleted",
            id
        )));
    }

    Ok(())
}

fn row_to_link(row: &rusqlite::Row<'_>) -> rusqlite::Result<EntityLink> {
    Ok(EntityLink {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        entity_id: row.get(2)?,
        title: row.get(3)?,
        kind: row.get(4)?,
        target: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
        device_id: row.get(9)?,
    })
}
