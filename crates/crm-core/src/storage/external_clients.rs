use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{datetime::now_iso8601, errors::CrmResult, uuid::new_uuid};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalClient {
    pub id: String,
    pub name: String,
    pub client_type: String,
    pub permission_mode: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub device_id: String,
}

pub fn create_external_client_placeholder(
    conn: &Connection,
    name: &str,
    client_type: &str,
    device_id: &str,
) -> CrmResult<ExternalClient> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO external_clients
            (id, name, client_type, permission_mode, enabled,
             created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, 'disabled', 0, ?4, ?5, ?6)
        "#,
        params![id, name, client_type, now, now, device_id],
    )?;

    Ok(ExternalClient {
        id,
        name: name.to_string(),
        client_type: client_type.to_string(),
        permission_mode: "disabled".to_string(),
        enabled: false,
        created_at: now.clone(),
        updated_at: now,
        deleted_at: None,
        device_id: device_id.to_string(),
    })
}

pub fn list_external_clients(conn: &Connection) -> CrmResult<Vec<ExternalClient>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, client_type, permission_mode, enabled,
               created_at, updated_at, deleted_at, device_id
        FROM external_clients
        WHERE deleted_at IS NULL
        ORDER BY created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map([], map_external_client_row)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_external_clients_for_export(conn: &Connection) -> CrmResult<Vec<ExternalClient>> {
    // enabled is runtime activation state, not export eligibility; disabled
    // non-deleted readiness placeholders export for diagnostics.
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, client_type, permission_mode, enabled,
               created_at, updated_at, deleted_at, device_id
        FROM external_clients
        WHERE deleted_at IS NULL
        ORDER BY created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map([], map_external_client_row)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_active_external_client(
    conn: &Connection,
    client_id: &str,
) -> CrmResult<Option<ExternalClient>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, client_type, permission_mode, enabled,
               created_at, updated_at, deleted_at, device_id
        FROM external_clients
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
    )?;

    match stmt.query_row(params![client_id], map_external_client_row) {
        Ok(client) => Ok(Some(client)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn update_external_client_activation(
    conn: &Connection,
    client_id: &str,
    enabled: bool,
    permission_mode: &str,
) -> CrmResult<Option<ExternalClient>> {
    let now = now_iso8601();
    let changed = conn.execute(
        r#"
        UPDATE external_clients
        SET enabled = ?2,
            permission_mode = ?3,
            updated_at = ?4
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![client_id, bool_to_sql(enabled), permission_mode, now],
    )?;

    if changed == 0 {
        return Ok(None);
    }

    get_active_external_client(conn, client_id)
}

fn map_external_client_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExternalClient> {
    let enabled: i64 = row.get(4)?;
    Ok(ExternalClient {
        id: row.get(0)?,
        name: row.get(1)?,
        client_type: row.get(2)?,
        permission_mode: row.get(3)?,
        enabled: enabled != 0,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        deleted_at: row.get(7)?,
        device_id: row.get(8)?,
    })
}

fn bool_to_sql(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}
