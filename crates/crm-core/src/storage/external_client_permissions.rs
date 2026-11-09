use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::permissions::ToolPermissionGrant;
use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalClientPermission {
    pub id: String,
    pub client_id: String,
    pub tool_name: String,
    pub can_read: bool,
    pub can_write: bool,
    pub requires_confirmation: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl ExternalClientPermission {
    pub fn grant(&self) -> ToolPermissionGrant {
        ToolPermissionGrant {
            can_read: self.can_read,
            can_write: self.can_write,
            requires_confirmation: self.requires_confirmation,
        }
    }

    pub fn has_same_rules(
        &self,
        can_read: bool,
        can_write: bool,
        requires_confirmation: bool,
    ) -> bool {
        self.can_read == can_read
            && self.can_write == can_write
            && self.requires_confirmation == requires_confirmation
    }
}

pub fn list_permissions_for_client(
    conn: &Connection,
    client_id: &str,
) -> CrmResult<Vec<ExternalClientPermission>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, client_id, tool_name, can_read, can_write,
               requires_confirmation, created_at, updated_at
        FROM external_client_permissions
        WHERE client_id = ?1
        ORDER BY tool_name ASC, created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![client_id], map_permission_row)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_all_permissions_for_export(
    conn: &Connection,
) -> CrmResult<Vec<ExternalClientPermission>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, client_id, tool_name, can_read, can_write,
               requires_confirmation, created_at, updated_at
        FROM external_client_permissions
        ORDER BY client_id ASC, tool_name ASC, created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map([], map_permission_row)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_permission_for_tool(
    conn: &Connection,
    client_id: &str,
    tool_name: &str,
) -> CrmResult<Option<ExternalClientPermission>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, client_id, tool_name, can_read, can_write,
               requires_confirmation, created_at, updated_at
        FROM external_client_permissions
        WHERE client_id = ?1 AND tool_name = ?2
        "#,
    )?;

    match stmt.query_row(params![client_id, tool_name], map_permission_row) {
        Ok(permission) => Ok(Some(permission)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn upsert_permission_for_tool(
    conn: &Connection,
    client_id: &str,
    tool_name: &str,
    can_read: bool,
    can_write: bool,
    requires_confirmation: bool,
) -> CrmResult<ExternalClientPermission> {
    let id = new_uuid();
    let now = now_iso8601();
    conn.execute(
        r#"
        INSERT INTO external_client_permissions
            (id, client_id, tool_name, can_read, can_write,
             requires_confirmation, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(client_id, tool_name) DO UPDATE SET
            can_read = excluded.can_read,
            can_write = excluded.can_write,
            requires_confirmation = excluded.requires_confirmation,
            updated_at = excluded.updated_at
        WHERE external_client_permissions.can_read != excluded.can_read
           OR external_client_permissions.can_write != excluded.can_write
           OR external_client_permissions.requires_confirmation != excluded.requires_confirmation
        "#,
        params![
            &id,
            client_id,
            tool_name,
            bool_to_sql(can_read),
            bool_to_sql(can_write),
            bool_to_sql(requires_confirmation),
            &now,
            &now
        ],
    )?;

    get_permission_for_tool(conn, client_id, tool_name)?.ok_or_else(|| {
        CrmError::NotFound(format!(
            "External client permission for client '{}' and tool '{}' was not found after upsert",
            client_id, tool_name
        ))
    })
}

fn map_permission_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExternalClientPermission> {
    let can_read: i64 = row.get(3)?;
    let can_write: i64 = row.get(4)?;
    let requires_confirmation: i64 = row.get(5)?;

    Ok(ExternalClientPermission {
        id: row.get(0)?,
        client_id: row.get(1)?,
        tool_name: row.get(2)?,
        can_read: can_read != 0,
        can_write: can_write != 0,
        requires_confirmation: requires_confirmation != 0,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn bool_to_sql(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}
