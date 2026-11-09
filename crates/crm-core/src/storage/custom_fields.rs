//! Custom field storage operations for 900CRM.
//!
//! This module provides CRUD operations for `custom_field_defs` and
//! `custom_field_values`, which power user-defined fields on contacts,
//! deals, activities, and organizations.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

const ENTITY_TYPES: [&str; 4] = ["contact", "deal", "activity", "organization"];
const FIELD_TYPES: [&str; 5] = ["text", "number", "date", "boolean", "select"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldDefinition {
    pub id: String,
    pub entity_type: String,
    pub field_name: String,
    pub field_type: String,
    pub field_options: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldValue {
    pub id: String,
    pub field_def_id: String,
    pub entity_id: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCustomFieldValue {
    pub value_id: String,
    pub field_def_id: String,
    pub field_name: String,
    pub field_type: String,
    pub field_options: Option<String>,
    pub sort_order: i32,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTypeCustomFieldValue {
    pub entity_id: String,
    pub field_def_id: String,
    pub value: String,
    pub updated_at: String,
}

pub fn list_definitions(
    conn: &Connection,
    entity_type: Option<&str>,
) -> CrmResult<Vec<CustomFieldDefinition>> {
    if let Some(entity_type) = entity_type {
        validate_entity_type(entity_type)?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, entity_type, field_name, field_type, field_options, sort_order, created_at
            FROM custom_field_defs
            WHERE entity_type = ?1
            ORDER BY sort_order ASC, created_at ASC
            "#,
        )?;

        let rows = stmt.query_map(params![entity_type], row_to_definition)?;
        return Ok(rows.collect::<Result<Vec<_>, _>>()?);
    }

    let mut stmt = conn.prepare(
        r#"
        SELECT id, entity_type, field_name, field_type, field_options, sort_order, created_at
        FROM custom_field_defs
        ORDER BY entity_type ASC, sort_order ASC, created_at ASC
        "#,
    )?;

    let rows = stmt.query_map([], row_to_definition)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_definition(conn: &Connection, id: &str) -> CrmResult<CustomFieldDefinition> {
    conn.query_row(
        r#"
        SELECT id, entity_type, field_name, field_type, field_options, sort_order, created_at
        FROM custom_field_defs
        WHERE id = ?1
        "#,
        params![id],
        row_to_definition,
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Custom field definition '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

pub fn create_definition(
    conn: &Connection,
    entity_type: &str,
    field_name: &str,
    field_type: &str,
    field_options: Option<&str>,
    sort_order: i32,
) -> CrmResult<CustomFieldDefinition> {
    validate_entity_type(entity_type)?;
    validate_field_type(field_type)?;

    let field_name = field_name.trim();
    if field_name.is_empty() {
        return Err(CrmError::InvalidInput("Field name is required".to_string()));
    }

    validate_field_options(field_type, field_options)?;

    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO custom_field_defs
            (id, entity_type, field_name, field_type, field_options, sort_order, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            id,
            entity_type,
            field_name,
            field_type,
            field_options,
            sort_order,
            now
        ],
    )?;

    get_definition(conn, &id)
}

pub fn update_definition(
    conn: &Connection,
    id: &str,
    field_name: Option<&str>,
    field_type: Option<&str>,
    field_options: Option<&str>,
    sort_order: Option<i32>,
) -> CrmResult<CustomFieldDefinition> {
    let current = get_definition(conn, id)?;

    let next_name = field_name
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(&current.field_name)
        .to_string();

    let next_type = field_type.unwrap_or(&current.field_type).to_string();
    validate_field_type(&next_type)?;

    let next_options = if field_options.is_some() {
        field_options.map(ToString::to_string)
    } else {
        current.field_options.clone()
    };

    validate_field_options(&next_type, next_options.as_deref())?;

    let next_sort = sort_order.unwrap_or(current.sort_order);

    conn.execute(
        r#"
        UPDATE custom_field_defs
        SET field_name = ?2,
            field_type = ?3,
            field_options = ?4,
            sort_order = ?5
        WHERE id = ?1
        "#,
        params![id, next_name, next_type, next_options, next_sort],
    )?;

    get_definition(conn, id)
}

pub fn delete_definition(conn: &Connection, id: &str) -> CrmResult<()> {
    let deleted = conn.execute("DELETE FROM custom_field_defs WHERE id = ?1", params![id])?;

    if deleted == 0 {
        return Err(CrmError::NotFound(format!(
            "Custom field definition '{}' not found",
            id
        )));
    }

    Ok(())
}

pub fn definition_has_values(conn: &Connection, id: &str) -> CrmResult<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM custom_field_values WHERE field_def_id = ?1",
        params![id],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

pub fn set_value(
    conn: &Connection,
    field_def_id: &str,
    entity_id: &str,
    value: &str,
) -> CrmResult<CustomFieldValue> {
    let _ = get_definition(conn, field_def_id)?;

    let now = now_iso8601();

    let existing_id: Option<String> = conn
        .query_row(
            r#"
            SELECT id
            FROM custom_field_values
            WHERE field_def_id = ?1 AND entity_id = ?2
            LIMIT 1
            "#,
            params![field_def_id, entity_id],
            |row| row.get(0),
        )
        .ok();

    let value_id = if let Some(existing_id) = existing_id {
        conn.execute(
            r#"
            UPDATE custom_field_values
            SET value = ?2,
                updated_at = ?3
            WHERE id = ?1
            "#,
            params![existing_id, value, now],
        )?;
        existing_id
    } else {
        let new_id = new_uuid();
        conn.execute(
            r#"
            INSERT INTO custom_field_values
                (id, field_def_id, entity_id, value, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![new_id, field_def_id, entity_id, value, now, now],
        )?;
        new_id
    };

    get_value(conn, &value_id)
}

pub fn get_value(conn: &Connection, id: &str) -> CrmResult<CustomFieldValue> {
    conn.query_row(
        r#"
        SELECT id, field_def_id, entity_id, value, created_at, updated_at
        FROM custom_field_values
        WHERE id = ?1
        "#,
        params![id],
        |row| {
            Ok(CustomFieldValue {
                id: row.get(0)?,
                field_def_id: row.get(1)?,
                entity_id: row.get(2)?,
                value: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Custom field value '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

pub fn get_value_for_entity_field(
    conn: &Connection,
    field_def_id: &str,
    entity_id: &str,
) -> CrmResult<Option<CustomFieldValue>> {
    match conn.query_row(
        r#"
        SELECT id, field_def_id, entity_id, value, created_at, updated_at
        FROM custom_field_values
        WHERE field_def_id = ?1 AND entity_id = ?2
        LIMIT 1
        "#,
        params![field_def_id, entity_id],
        |row| {
            Ok(CustomFieldValue {
                id: row.get(0)?,
                field_def_id: row.get(1)?,
                entity_id: row.get(2)?,
                value: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    ) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(CrmError::Database(err.to_string())),
    }
}

pub fn list_values_for_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> CrmResult<Vec<EntityCustomFieldValue>> {
    validate_entity_type(entity_type)?;

    let mut stmt = conn.prepare(
        r#"
        SELECT
            v.id,
            d.id,
            d.field_name,
            d.field_type,
            d.field_options,
            d.sort_order,
            v.value,
            v.updated_at
        FROM custom_field_values v
        INNER JOIN custom_field_defs d ON d.id = v.field_def_id
        WHERE d.entity_type = ?1 AND v.entity_id = ?2
        ORDER BY d.sort_order ASC, d.created_at ASC
        "#,
    )?;

    let rows = stmt.query_map(params![entity_type, entity_id], |row| {
        Ok(EntityCustomFieldValue {
            value_id: row.get(0)?,
            field_def_id: row.get(1)?,
            field_name: row.get(2)?,
            field_type: row.get(3)?,
            field_options: row.get(4)?,
            sort_order: row.get(5)?,
            value: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn list_values_for_entity_type(
    conn: &Connection,
    entity_type: &str,
) -> CrmResult<Vec<EntityTypeCustomFieldValue>> {
    validate_entity_type(entity_type)?;

    let mut stmt = conn.prepare(
        r#"
        SELECT
            v.entity_id,
            v.field_def_id,
            v.value,
            v.updated_at
        FROM custom_field_values v
        INNER JOIN custom_field_defs d ON d.id = v.field_def_id
        WHERE d.entity_type = ?1
          AND trim(v.value) <> ''
        ORDER BY v.updated_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![entity_type], |row| {
        Ok(EntityTypeCustomFieldValue {
            entity_id: row.get(0)?,
            field_def_id: row.get(1)?,
            value: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_value_for_entity_field(
    conn: &Connection,
    field_def_id: &str,
    entity_id: &str,
) -> CrmResult<bool> {
    let deleted = conn.execute(
        "DELETE FROM custom_field_values WHERE field_def_id = ?1 AND entity_id = ?2",
        params![field_def_id, entity_id],
    )?;

    Ok(deleted > 0)
}

pub fn delete_values_for_entity(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> CrmResult<usize> {
    validate_entity_type(entity_type)?;

    let deleted = conn.execute(
        r#"
        DELETE FROM custom_field_values
        WHERE entity_id = ?1
          AND field_def_id IN (
              SELECT id
              FROM custom_field_defs
              WHERE entity_type = ?2
          )
        "#,
        params![entity_id, entity_type],
    )?;

    Ok(deleted)
}

fn row_to_definition(row: &rusqlite::Row<'_>) -> rusqlite::Result<CustomFieldDefinition> {
    Ok(CustomFieldDefinition {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        field_name: row.get(2)?,
        field_type: row.get(3)?,
        field_options: row.get(4)?,
        sort_order: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn validate_entity_type(value: &str) -> CrmResult<()> {
    if ENTITY_TYPES.contains(&value) {
        return Ok(());
    }

    Err(CrmError::InvalidInput(format!(
        "Invalid entity_type '{}'. Expected one of: {}",
        value,
        ENTITY_TYPES.join(", ")
    )))
}

fn validate_field_type(value: &str) -> CrmResult<()> {
    if FIELD_TYPES.contains(&value) {
        return Ok(());
    }

    Err(CrmError::InvalidInput(format!(
        "Invalid field_type '{}'. Expected one of: {}",
        value,
        FIELD_TYPES.join(", ")
    )))
}

fn validate_field_options(field_type: &str, field_options: Option<&str>) -> CrmResult<()> {
    if field_type == "select" {
        let raw = field_options.ok_or_else(|| {
            CrmError::InvalidInput("field_options is required for select custom fields".to_string())
        })?;
        validate_json_string_array(raw)?;
        return Ok(());
    }

    if let Some(raw) = field_options {
        validate_json_string_array(raw)?;
    }

    Ok(())
}

fn validate_json_string_array(raw: &str) -> CrmResult<()> {
    let parsed: serde_json::Value = serde_json::from_str(raw)?;
    let items = parsed
        .as_array()
        .ok_or_else(|| CrmError::InvalidInput("field_options must be a JSON array".to_string()))?;

    if items.is_empty() {
        return Err(CrmError::InvalidInput(
            "field_options must include at least one option".to_string(),
        ));
    }

    if !items.iter().all(|item| item.is_string()) {
        return Err(CrmError::InvalidInput(
            "field_options entries must all be strings".to_string(),
        ));
    }

    Ok(())
}
