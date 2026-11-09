use rusqlite::Connection;

use crate::utils::errors::{CrmError, CrmResult};

use super::db::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationMigrationReadinessCounts {
    pub legacy_organization_contacts: i64,
    pub contacts_with_org_id_missing_organization_id: i64,
    pub contacts_with_invalid_legacy_org_id_links: i64,
    pub contacts_with_invalid_normalized_organization_id_links: i64,
    pub backup_restore_baseline_available: bool,
}

pub fn get_normalization_migration_readiness(
    conn: &Connection,
) -> CrmResult<NormalizationMigrationReadinessCounts> {
    Ok(NormalizationMigrationReadinessCounts {
        legacy_organization_contacts: count_legacy_organization_contacts(conn)?,
        contacts_with_org_id_missing_organization_id:
            count_contacts_with_org_id_missing_organization_id(conn)?,
        contacts_with_invalid_legacy_org_id_links: count_contacts_with_invalid_legacy_org_id_links(
            conn,
        )?,
        contacts_with_invalid_normalized_organization_id_links:
            count_contacts_with_invalid_normalized_organization_id_links(conn)?,
        backup_restore_baseline_available: backup_restore_baseline_available(conn)?,
    })
}

fn count_legacy_organization_contacts(conn: &Connection) -> CrmResult<i64> {
    conn.query_row(
        r#"
        SELECT COUNT(*)
        FROM contacts
        WHERE contact_type = 'organization'
          AND deleted_at IS NULL
        "#,
        [],
        |row| row.get(0),
    )
    .map_err(CrmError::from)
}

fn count_contacts_with_org_id_missing_organization_id(conn: &Connection) -> CrmResult<i64> {
    conn.query_row(
        r#"
        SELECT COUNT(*)
        FROM contacts
        WHERE deleted_at IS NULL
          AND TRIM(COALESCE(org_id, '')) <> ''
          AND TRIM(COALESCE(organization_id, '')) = ''
        "#,
        [],
        |row| row.get(0),
    )
    .map_err(CrmError::from)
}

fn count_contacts_with_invalid_legacy_org_id_links(conn: &Connection) -> CrmResult<i64> {
    conn.query_row(
        r#"
        SELECT COUNT(*)
        FROM contacts AS c
        WHERE c.deleted_at IS NULL
          AND TRIM(COALESCE(c.org_id, '')) <> ''
          AND NOT EXISTS (
            SELECT 1
            FROM contacts AS org
            WHERE org.id = c.org_id
              AND org.contact_type = 'organization'
              AND org.deleted_at IS NULL
          )
        "#,
        [],
        |row| row.get(0),
    )
    .map_err(CrmError::from)
}

fn count_contacts_with_invalid_normalized_organization_id_links(
    conn: &Connection,
) -> CrmResult<i64> {
    conn.query_row(
        r#"
        SELECT COUNT(*)
        FROM contacts AS c
        WHERE c.deleted_at IS NULL
          AND TRIM(COALESCE(c.organization_id, '')) <> ''
          AND NOT EXISTS (
            SELECT 1
            FROM organizations AS o
            WHERE o.id = c.organization_id
              AND o.deleted_at IS NULL
          )
        "#,
        [],
        |row| row.get(0),
    )
    .map_err(CrmError::from)
}

fn backup_restore_baseline_available(conn: &Connection) -> CrmResult<bool> {
    let schema_version: u32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(CrmError::from)?;
    if schema_version < Database::current_schema_version() {
        return Ok(false);
    }

    let required_table_count: i64 = conn
        .query_row(
            r#"
            SELECT COUNT(*)
            FROM sqlite_master
            WHERE type = 'table'
              AND name IN (
                'contacts',
                'settings',
                'sync_changelog',
                'organizations',
                'audit_log'
              )
            "#,
            [],
            |row| row.get(0),
        )
        .map_err(CrmError::from)?;
    if required_table_count != 5 {
        return Ok(false);
    }

    let organization_id_column_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('contacts') WHERE name = 'organization_id'",
            [],
            |row| row.get(0),
        )
        .map_err(CrmError::from)?;

    Ok(organization_id_column_count == 1)
}
