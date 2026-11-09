use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{datetime::now_iso8601, errors::CrmResult, uuid::new_uuid};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub before_json: Option<String>,
    pub after_json: Option<String>,
    pub created_at: String,
    pub device_id: String,
}

#[allow(clippy::too_many_arguments)]
pub fn record_audit(
    conn: &Connection,
    actor_type: &str,
    actor_id: Option<&str>,
    action: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    before_json: Option<&str>,
    after_json: Option<&str>,
    device_id: &str,
) -> CrmResult<AuditLogEntry> {
    let id = new_uuid();
    let created_at = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO audit_log
            (id, actor_type, actor_id, action, entity_type, entity_id,
             before_json, after_json, created_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            id,
            actor_type,
            actor_id,
            action,
            entity_type,
            entity_id,
            before_json,
            after_json,
            created_at,
            device_id
        ],
    )?;

    Ok(AuditLogEntry {
        id,
        actor_type: actor_type.to_string(),
        actor_id: actor_id.map(str::to_string),
        action: action.to_string(),
        entity_type: entity_type.map(str::to_string),
        entity_id: entity_id.map(str::to_string),
        before_json: before_json.map(str::to_string),
        after_json: after_json.map(str::to_string),
        created_at,
        device_id: device_id.to_string(),
    })
}

pub fn list_recent_audit_log(conn: &Connection, limit: u32) -> CrmResult<Vec<AuditLogEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, actor_type, actor_id, action, entity_type, entity_id,
               before_json, after_json, created_at, device_id
        FROM audit_log
        ORDER BY created_at DESC, id DESC
        LIMIT ?1
        "#,
    )?;

    let rows = stmt.query_map(params![limit.clamp(1, 500)], |row| {
        Ok(AuditLogEntry {
            id: row.get(0)?,
            actor_type: row.get(1)?,
            actor_id: row.get(2)?,
            action: row.get(3)?,
            entity_type: row.get(4)?,
            entity_id: row.get(5)?,
            before_json: row.get(6)?,
            after_json: row.get(7)?,
            created_at: row.get(8)?,
            device_id: row.get(9)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_all_audit_log(conn: &Connection) -> CrmResult<Vec<AuditLogEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, actor_type, actor_id, action, entity_type, entity_id,
               before_json, after_json, created_at, device_id
        FROM audit_log
        ORDER BY created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(AuditLogEntry {
            id: row.get(0)?,
            actor_type: row.get(1)?,
            actor_id: row.get(2)?,
            action: row.get(3)?,
            entity_type: row.get(4)?,
            entity_id: row.get(5)?,
            before_json: row.get(6)?,
            after_json: row.get(7)?,
            created_at: row.get(8)?,
            device_id: row.get(9)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
