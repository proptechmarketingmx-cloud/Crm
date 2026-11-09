use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub source: Option<String>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub device_id: String,
}

#[allow(clippy::too_many_arguments)]
pub fn create_organization(
    conn: &Connection,
    name: &str,
    email: Option<&str>,
    phone: Option<&str>,
    website: Option<&str>,
    address_line1: Option<&str>,
    address_line2: Option<&str>,
    city: Option<&str>,
    region: Option<&str>,
    country: Option<&str>,
    postal_code: Option<&str>,
    source: Option<&str>,
    description: Option<&str>,
    device_id: &str,
) -> CrmResult<Organization> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO organizations
            (id, name, email, phone, website, address_line1, address_line2,
             city, region, country, postal_code, source, description,
             created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
        "#,
        params![
            id,
            name,
            email,
            phone,
            website,
            address_line1,
            address_line2,
            city,
            region,
            country,
            postal_code,
            source,
            description,
            now,
            now,
            device_id
        ],
    )?;

    get_organization(conn, &id)
}

pub fn get_organization(conn: &Connection, id: &str) -> CrmResult<Organization> {
    conn.query_row(
        r#"
        SELECT id, name, email, phone, website, address_line1, address_line2,
               city, region, country, postal_code, source, description,
               created_at, updated_at, deleted_at, device_id, owner
        FROM organizations
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![id],
        row_to_organization,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Organization '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

pub fn list_organizations(conn: &Connection) -> CrmResult<Vec<Organization>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, email, phone, website, address_line1, address_line2,
               city, region, country, postal_code, source, description,
               created_at, updated_at, deleted_at, device_id, owner
        FROM organizations
        WHERE deleted_at IS NULL
        ORDER BY LOWER(name) ASC, created_at ASC
        "#,
    )?;

    let rows = stmt.query_map([], row_to_organization)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Finds active organizations with an exact case-insensitive name match.
pub fn find_active_organizations_by_name(
    conn: &Connection,
    name: &str,
) -> CrmResult<Vec<Organization>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, email, phone, website, address_line1, address_line2,
               city, region, country, postal_code, source, description,
               created_at, updated_at, deleted_at, device_id, owner
        FROM organizations
        WHERE LOWER(name) = LOWER(?1) AND deleted_at IS NULL
        "#,
    )?;

    let rows = stmt.query_map(params![name], row_to_organization)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Finds active organizations with an exact case-insensitive email match.
pub fn find_active_organizations_by_email(
    conn: &Connection,
    email: &str,
) -> CrmResult<Vec<Organization>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, email, phone, website, address_line1, address_line2,
               city, region, country, postal_code, source, description,
               created_at, updated_at, deleted_at, device_id, owner
        FROM organizations
        WHERE LOWER(email) = LOWER(?1) AND deleted_at IS NULL
        "#,
    )?;

    let rows = stmt.query_map(params![email], row_to_organization)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Finds active organizations with a phone number matching exactly after trimming.
pub fn find_active_organizations_by_phone(
    conn: &Connection,
    phone: &str,
) -> CrmResult<Vec<Organization>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, email, phone, website, address_line1, address_line2,
               city, region, country, postal_code, source, description,
               created_at, updated_at, deleted_at, device_id, owner
        FROM organizations
        WHERE TRIM(phone) = TRIM(?1) AND deleted_at IS NULL
        "#,
    )?;

    let rows = stmt.query_map(params![phone], row_to_organization)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
pub fn update_organization(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    email: Option<Option<&str>>,
    phone: Option<Option<&str>>,
    website: Option<Option<&str>>,
    address_line1: Option<Option<&str>>,
    address_line2: Option<Option<&str>>,
    city: Option<Option<&str>>,
    region: Option<Option<&str>>,
    country: Option<Option<&str>>,
    postal_code: Option<Option<&str>>,
    source: Option<Option<&str>>,
    description: Option<Option<&str>>,
) -> CrmResult<Organization> {
    let current = get_organization(conn, id)?;
    let now = now_iso8601();

    let changed = conn.execute(
        r#"
        UPDATE organizations SET
            name          = ?1,
            email         = ?2,
            phone         = ?3,
            website       = ?4,
            address_line1 = ?5,
            address_line2 = ?6,
            city          = ?7,
            region        = ?8,
            country       = ?9,
            postal_code   = ?10,
            source        = ?11,
            description   = ?12,
            updated_at    = ?13
        WHERE id = ?14 AND deleted_at IS NULL
        "#,
        params![
            name.unwrap_or(&current.name),
            apply_optional_update(email, &current.email),
            apply_optional_update(phone, &current.phone),
            apply_optional_update(website, &current.website),
            apply_optional_update(address_line1, &current.address_line1),
            apply_optional_update(address_line2, &current.address_line2),
            apply_optional_update(city, &current.city),
            apply_optional_update(region, &current.region),
            apply_optional_update(country, &current.country),
            apply_optional_update(postal_code, &current.postal_code),
            apply_optional_update(source, &current.source),
            apply_optional_update(description, &current.description),
            now,
            id
        ],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Organization '{}' not found",
            id
        )));
    }

    get_organization(conn, id)
}

fn apply_optional_update<'a>(
    update: Option<Option<&'a str>>,
    current: &'a Option<String>,
) -> Option<&'a str> {
    match update {
        Some(value) => value,
        None => current.as_deref(),
    }
}

pub fn soft_delete_organization(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE organizations SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Organization '{}' not found",
            id
        )));
    }

    Ok(())
}

fn row_to_organization(row: &rusqlite::Row<'_>) -> rusqlite::Result<Organization> {
    Ok(Organization {
        id: row.get(0)?,
        name: row.get(1)?,
        email: row.get(2)?,
        phone: row.get(3)?,
        website: row.get(4)?,
        address_line1: row.get(5)?,
        address_line2: row.get(6)?,
        city: row.get(7)?,
        region: row.get(8)?,
        country: row.get(9)?,
        postal_code: row.get(10)?,
        source: row.get(11)?,
        description: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
        deleted_at: row.get(15)?,
        device_id: row.get(16)?,
        owner: row.get::<_, Option<String>>(17)?.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }),
    })
}

/// Sets or clears an organization's local owner name.
pub fn set_organization_owner(
    conn: &Connection,
    id: &str,
    owner: Option<&str>,
) -> CrmResult<Organization> {
    let _current = get_organization(conn, id)?;
    let now = now_iso8601();
    let normalized = owner
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    conn.execute(
        "UPDATE organizations SET owner = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
        params![normalized, now, id],
    )?;
    get_organization(conn, id)
}
