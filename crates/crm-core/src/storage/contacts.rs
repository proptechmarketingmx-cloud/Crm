//! Contact CRUD operations and FTS5 search for 900CRM.
//!
//! This module provides all database operations for the `contacts` table,
//! including full-text search via the `contacts_fts` FTS5 virtual table.
//!
//! # Soft Delete
//!
//! Contacts are never physically deleted. `soft_delete_contact` sets the
//! `deleted_at` column to the current timestamp. All list and get operations
//! exclude soft-deleted records by default. Use `restore_contact` to undo.
//!
//! # FTS5 Sync
//!
//! The `contacts_fts` virtual table is kept in sync manually:
//! - Insert → insert a corresponding FTS row.
//! - Update → delete old FTS row + insert new one.
//! - Delete → delete FTS row.

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

/// A CRM contact representing a person or organization.
///
/// Contacts are the central entity of 900CRM. A contact with
/// `contact_type = "organization"` can be linked to person contacts via
/// `org_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// UUID v4 primary key.
    pub id: String,

    /// Contact type: `"person"` or `"organization"`.
    pub contact_type: String,

    /// Given name / first name.
    pub first_name: String,

    /// Family name / last name.
    pub last_name: String,

    /// Organization or company name.
    pub org_name: String,

    /// Primary email address.
    pub email: String,

    /// Primary phone number.
    pub phone: String,

    /// Street address.
    pub address: String,

    /// City or locality.
    pub city: String,

    /// Country.
    pub country: String,

    /// Optional parent organization contact ID.
    pub org_id: Option<String>,

    /// Optional normalized organization ID.
    pub organization_id: Option<String>,

    /// Freeform notes.
    pub notes: String,

    /// Person lifecycle: `"lead"` or `"customer"`.
    ///
    /// Existing contacts default to `"customer"` so a schema upgrade does not
    /// silently reclassify people as leads. Organizations stay `"customer"`.
    pub lifecycle: String,

    /// Optional local owner name. This is not a user account.
    pub owner: Option<String>,

    /// ISO 8601 creation timestamp.
    pub created_at: String,

    /// ISO 8601 last-update timestamp.
    pub updated_at: String,

    /// ISO 8601 soft-delete timestamp (`None` = not deleted).
    pub deleted_at: Option<String>,

    /// ID of the device that created or last modified this record.
    pub device_id: String,
}

/// Parameters for listing contacts with pagination, sorting, and filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactListParams {
    /// Page number (1-based).
    pub page: u32,

    /// Number of records per page.
    pub per_page: u32,

    /// Column to sort by (e.g. `"first_name"`, `"created_at"`).
    pub sort_by: String,

    /// Sort direction: `"asc"` or `"desc"`.
    pub sort_dir: String,

    /// Optional filter by `contact_type`.
    pub filter_type: Option<String>,

    /// Optional filter by person lifecycle (`"lead"` or `"customer"`).
    pub filter_lifecycle: Option<String>,

    /// Optional case-insensitive exact owner name filter.
    pub filter_owner: Option<String>,

    /// Optional full-text search query.
    pub search_query: Option<String>,

    /// Optional custom field definition ID for value filtering.
    pub custom_field_def_id: Option<String>,

    /// Optional case-insensitive custom field value substring match.
    pub custom_field_query: Option<String>,
}

impl Default for ContactListParams {
    /// Default parameters: page 1, 25 per page, sorted by `first_name` ascending.
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 25,
            sort_by: "first_name".to_string(),
            sort_dir: "asc".to_string(),
            filter_type: None,
            filter_lifecycle: None,
            filter_owner: None,
            search_query: None,
            custom_field_def_id: None,
            custom_field_query: None,
        }
    }
}

/// A paginated result set for contact listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactListResult {
    /// The contacts on this page.
    pub contacts: Vec<Contact>,

    /// Total number of matching contacts (for pagination UI).
    pub total: u32,

    /// Current page number.
    pub page: u32,

    /// Records per page.
    pub per_page: u32,
}

/// A read-only duplicate contact pair candidate for user-confirmed merges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactDuplicateCandidate {
    /// Contact that should be merged into the target by default.
    pub source_id: String,

    /// Human-readable label for the source contact.
    pub source_display_label: String,

    /// Contact that should be kept by default.
    pub target_id: String,

    /// Human-readable label for the target contact.
    pub target_display_label: String,

    /// Duplicate match type: `"email"` or `"phone"`.
    pub match_type: String,

    /// Trimmed value that matched.
    pub matched_value: String,

    /// Human-readable duplicate reason.
    pub reason: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// CRUD
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new contact record and inserts a matching FTS5 row.
///
/// Generates a new UUID for `id`, sets `created_at` and `updated_at` to now.
///
/// # Parameters
///
/// - `conn` — SQLite connection from the locked `Database`.
/// - `contact_type` — `"person"` or `"organization"`.
/// - `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`,
///   `city`, `country`, `org_id`, `notes` — Contact fields.
/// - `device_id` — The originating device UUID.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn create_contact(
    conn: &Connection,
    contact_type: &str,
    first_name: &str,
    last_name: &str,
    org_name: &str,
    email: &str,
    phone: &str,
    address: &str,
    city: &str,
    country: &str,
    org_id: Option<&str>,
    notes: &str,
    device_id: &str,
) -> CrmResult<Contact> {
    let id = new_uuid();
    let now = now_iso8601();
    let (legacy_org_id, organization_id) = resolve_organization_link(conn, org_id)?;

    conn.execute(
        r#"
        INSERT INTO contacts
            (id, contact_type, first_name, last_name, org_name, email, phone,
             address, city, country, org_id, organization_id, notes,
             created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
        "#,
        params![
            id,
            contact_type,
            first_name,
            last_name,
            org_name,
            email,
            phone,
            address,
            city,
            country,
            legacy_org_id,
            organization_id,
            notes,
            now,
            now,
            device_id
        ],
    )?;

    // Keep FTS5 in sync.
    conn.execute(
        r#"
        INSERT INTO contacts_fts (rowid, first_name, last_name, org_name, email, phone)
        SELECT rowid, first_name, last_name, org_name, email, phone
        FROM contacts WHERE id = ?1
        "#,
        params![id],
    )?;

    log::debug!("Created contact id={}", id);

    get_contact(conn, &id)
}

/// Retrieves a single contact by its UUID.
///
/// Returns [`CrmError::NotFound`] if no contact with `id` exists (including
/// soft-deleted ones — use `get_contact_including_deleted` if needed).
///
/// # Errors
///
/// - [`CrmError::NotFound`] — `id` does not match any active contact.
/// - [`CrmError::Database`] — Query failure.
pub fn get_contact(conn: &Connection, id: &str) -> CrmResult<Contact> {
    let contact = conn
        .query_row(
            r#"
        SELECT id, contact_type, first_name, last_name, org_name, email, phone,
               address, city, country, org_id, organization_id, notes, created_at, updated_at,
               deleted_at, device_id, lifecycle, owner
        FROM contacts
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
            params![id],
            row_to_contact,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CrmError::NotFound(format!("Contact '{}' not found", id))
            }
            other => CrmError::Database(other.to_string()),
        })?;

    Ok(contact)
}

/// Lists contacts with optional pagination, sorting, and filtering.
///
/// - Excludes soft-deleted contacts (`deleted_at IS NULL`).
/// - Supports FTS5 search via `params.search_query`.
/// - Sorts by any valid column; falls back to `first_name ASC` on unknown column.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_contacts(
    conn: &Connection,
    params: &ContactListParams,
) -> CrmResult<ContactListResult> {
    let safe_sort_by = sanitize_sort_column(&params.sort_by, "first_name");
    let safe_sort_dir = if params.sort_dir.to_lowercase() == "desc" {
        "DESC"
    } else {
        "ASC"
    };
    let per_page = params.per_page.clamp(1, 200);
    let page = params.page.max(1);
    let offset = ((page as i64) - 1) * (per_page as i64);
    let limit = per_page as i64;

    // If there is a search query, use FTS5.
    if let Some(ref query) = params.search_query {
        if !query.trim().is_empty() {
            return search_contacts_paged(conn, query, params);
        }
    }

    let apply_type_filter = params.filter_type.is_some();
    let apply_lifecycle_filter = params.filter_lifecycle.is_some();
    let apply_owner_filter = params
        .filter_owner
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let apply_custom_filter = params
        .custom_field_def_id
        .as_ref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        && params
            .custom_field_query
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);

    let sql_count = r#"
        SELECT COUNT(*)
        FROM contacts c
        WHERE c.deleted_at IS NULL
          AND (?1 = 0 OR c.contact_type = ?2)
          AND (?3 = 0 OR c.lifecycle = ?4)
          AND (
            ?5 = 0 OR EXISTS (
              SELECT 1
              FROM custom_field_values cfv
              WHERE cfv.entity_id = c.id
                AND cfv.field_def_id = ?6
                AND LOWER(cfv.value) LIKE LOWER(?7)
            )
          )
          AND (?8 = 0 OR LOWER(TRIM(COALESCE(c.owner, ''))) = LOWER(TRIM(?9)))
    "#;

    let sql_list = format!(
        r#"
        SELECT id, contact_type, first_name, last_name, org_name, email, phone,
               address, city, country, org_id, organization_id, notes, created_at, updated_at,
               deleted_at, device_id, lifecycle, owner
        FROM contacts c
        WHERE c.deleted_at IS NULL
          AND (?1 = 0 OR c.contact_type = ?2)
          AND (?3 = 0 OR c.lifecycle = ?4)
          AND (
            ?5 = 0 OR EXISTS (
              SELECT 1
              FROM custom_field_values cfv
              WHERE cfv.entity_id = c.id
                AND cfv.field_def_id = ?6
                AND LOWER(cfv.value) LIKE LOWER(?7)
            )
          )
          AND (?8 = 0 OR LOWER(TRIM(COALESCE(c.owner, ''))) = LOWER(TRIM(?9)))
        ORDER BY c.{} {}
        LIMIT ?10 OFFSET ?11
        "#,
        safe_sort_by, safe_sort_dir
    );

    let filter_type = params.filter_type.as_deref().unwrap_or("");
    let filter_lifecycle = params.filter_lifecycle.as_deref().unwrap_or("");
    let filter_owner = params.filter_owner.as_deref().unwrap_or("");
    let custom_field_def_id = params.custom_field_def_id.as_deref().unwrap_or("");
    let custom_field_query = params
        .custom_field_query
        .as_ref()
        .map(|q| format!("%{}%", q.trim()))
        .unwrap_or_else(|| "%%".to_string());

    let total: u32 = conn
        .query_row(
            sql_count,
            params![
                if apply_type_filter { 1 } else { 0 },
                filter_type,
                if apply_lifecycle_filter { 1 } else { 0 },
                filter_lifecycle,
                if apply_custom_filter { 1 } else { 0 },
                custom_field_def_id,
                custom_field_query,
                if apply_owner_filter { 1 } else { 0 },
                filter_owner,
            ],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn.prepare(&sql_list)?;
    let rows = stmt.query_map(
        params![
            if apply_type_filter { 1 } else { 0 },
            filter_type,
            if apply_lifecycle_filter { 1 } else { 0 },
            filter_lifecycle,
            if apply_custom_filter { 1 } else { 0 },
            custom_field_def_id,
            custom_field_query,
            if apply_owner_filter { 1 } else { 0 },
            filter_owner,
            limit,
            offset
        ],
        row_to_contact,
    )?;
    let contacts: Vec<Contact> = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!(
        "list_contacts: page={}, per_page={}, total={}",
        params.page,
        params.per_page,
        total
    );

    Ok(ContactListResult {
        contacts,
        total,
        page,
        per_page,
    })
}

/// Updates an existing contact's fields.
///
/// Only updates the columns that are `Some`. Sets `updated_at` to now.
/// Also refreshes the FTS5 index row.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Contact does not exist or is deleted.
/// - [`CrmError::Database`] — SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn update_contact(
    conn: &Connection,
    id: &str,
    contact_type: Option<&str>,
    first_name: Option<&str>,
    last_name: Option<&str>,
    org_name: Option<&str>,
    email: Option<&str>,
    phone: Option<&str>,
    address: Option<&str>,
    city: Option<&str>,
    country: Option<&str>,
    org_id: Option<Option<&str>>,
    organization_id: Option<Option<&str>>,
    notes: Option<&str>,
) -> CrmResult<Contact> {
    // Fetch current to apply partial updates.
    let current = get_contact(conn, id)?;
    let now = now_iso8601();

    conn.execute(
        r#"
        UPDATE contacts SET
            contact_type = ?1,
            first_name   = ?2,
            last_name    = ?3,
            org_name     = ?4,
            email        = ?5,
            phone        = ?6,
            address      = ?7,
            city         = ?8,
            country      = ?9,
            org_id       = ?10,
            organization_id = ?11,
            notes        = ?12,
            updated_at   = ?13
        WHERE id = ?14 AND deleted_at IS NULL
        "#,
        params![
            contact_type.unwrap_or(&current.contact_type),
            first_name.unwrap_or(&current.first_name),
            last_name.unwrap_or(&current.last_name),
            org_name.unwrap_or(&current.org_name),
            email.unwrap_or(&current.email),
            phone.unwrap_or(&current.phone),
            address.unwrap_or(&current.address),
            city.unwrap_or(&current.city),
            country.unwrap_or(&current.country),
            org_id.unwrap_or(current.org_id.as_deref()),
            organization_id.unwrap_or(current.organization_id.as_deref()),
            notes.unwrap_or(&current.notes),
            now,
            id
        ],
    )?;

    rebuild_contacts_fts(conn)?;

    log::debug!("Updated contact id={}", id);
    get_contact(conn, id)
}

/// Sets a person contact's lifecycle to `"lead"` or `"customer"`.
///
/// Organizations cannot be leads. Existing rows stay `"customer"` unless this
/// path is called explicitly.
pub fn set_contact_lifecycle(conn: &Connection, id: &str, lifecycle: &str) -> CrmResult<Contact> {
    let current = get_contact(conn, id)?;
    let normalized = normalize_lifecycle(lifecycle)?;

    if current.contact_type != "person" && normalized == "lead" {
        return Err(CrmError::InvalidInput(
            "Only person contacts can be marked as leads".to_string(),
        ));
    }

    if current.lifecycle == normalized {
        return Ok(current);
    }

    let now = now_iso8601();
    let changed = conn.execute(
        r#"
        UPDATE contacts
        SET lifecycle = ?1, updated_at = ?2
        WHERE id = ?3 AND deleted_at IS NULL
        "#,
        params![normalized, now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Contact '{}' not found", id)));
    }

    log::debug!("Set contact {} lifecycle to {}", id, normalized);
    get_contact(conn, id)
}

/// Sets or clears a contact's local owner name.
pub fn set_contact_owner(conn: &Connection, id: &str, owner: Option<&str>) -> CrmResult<Contact> {
    let _current = get_contact(conn, id)?;
    let now = now_iso8601();
    conn.execute(
        "UPDATE contacts SET owner = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
        params![normalize_owner(owner), now, id],
    )?;
    get_contact(conn, id)
}

pub fn normalize_lifecycle(lifecycle: &str) -> CrmResult<String> {
    match lifecycle.trim().to_ascii_lowercase().as_str() {
        "lead" => Ok("lead".to_string()),
        "customer" => Ok("customer".to_string()),
        other => Err(CrmError::InvalidInput(format!(
            "Invalid lifecycle '{}'. Must be 'lead' or 'customer'",
            other
        ))),
    }
}

/// Links or unlinks a contact to a normalized organization.
///
/// The normalized `organization_id` column is updated for all first-class
/// organization links. The legacy `org_id` mirror is only set when the target
/// still exists as a legacy organization contact, because that column has a
/// foreign key back to `contacts(id)`.
pub fn link_contact_to_organization(
    conn: &Connection,
    contact_id: &str,
    organization_id: Option<&str>,
    organization_name: Option<&str>,
) -> CrmResult<Contact> {
    let current = get_contact(conn, contact_id)?;
    let now = now_iso8601();
    let next_org_name = organization_name.unwrap_or(&current.org_name);
    let (legacy_org_id, normalized_organization_id) =
        resolve_organization_link(conn, organization_id)?;

    let changed = conn.execute(
        r#"
        UPDATE contacts SET
            org_id          = ?1,
            organization_id = ?2,
            org_name        = ?3,
            updated_at      = ?4
        WHERE id = ?5 AND deleted_at IS NULL
        "#,
        params![
            legacy_org_id,
            normalized_organization_id,
            next_org_name,
            now,
            contact_id
        ],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Contact '{}' not found",
            contact_id
        )));
    }

    rebuild_contacts_fts(conn)?;

    get_contact(conn, contact_id)
}

fn resolve_organization_link<'a>(
    conn: &Connection,
    organization_id: Option<&'a str>,
) -> CrmResult<(Option<&'a str>, Option<&'a str>)> {
    let Some(organization_id) = organization_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return Ok((None, None));
    };

    let legacy_contact_exists: i64 = conn.query_row(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM contacts
            WHERE id = ?1 AND contact_type = 'organization' AND deleted_at IS NULL
        )
        "#,
        params![organization_id],
        |row| row.get(0),
    )?;
    let organization_exists: i64 = conn.query_row(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM organizations
            WHERE id = ?1 AND deleted_at IS NULL
        )
        "#,
        params![organization_id],
        |row| row.get(0),
    )?;

    if legacy_contact_exists == 0 && organization_exists == 0 {
        return Err(CrmError::NotFound(format!(
            "Organization '{}' not found",
            organization_id
        )));
    }

    let legacy_org_id = (legacy_contact_exists != 0).then_some(organization_id);
    Ok((legacy_org_id, Some(organization_id)))
}

fn rebuild_contacts_fts(conn: &Connection) -> CrmResult<()> {
    conn.execute(
        "INSERT INTO contacts_fts(contacts_fts) VALUES('rebuild')",
        [],
    )?;
    Ok(())
}

/// Soft-deletes a contact by setting `deleted_at` to the current timestamp.
///
/// The contact is excluded from all list/search operations after this call.
/// Use [`restore_contact`] to undo.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Contact does not exist or is already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn soft_delete_contact(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE contacts SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Contact '{}' not found or already deleted",
            id
        )));
    }

    rebuild_contacts_fts(conn)?;

    log::info!("Soft-deleted contact id={}", id);
    Ok(())
}

/// Restores a soft-deleted contact by clearing `deleted_at`.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — No soft-deleted contact with this ID exists.
/// - [`CrmError::Database`] — SQL failure.
pub fn restore_contact(conn: &Connection, id: &str) -> CrmResult<Contact> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE contacts SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NOT NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "No deleted contact '{}' found",
            id
        )));
    }

    rebuild_contacts_fts(conn)?;

    log::info!("Restored contact id={}", id);
    get_contact(conn, id)
}

/// Full-text search over contacts using the FTS5 virtual table.
///
/// The query is matched against `first_name`, `last_name`, `org_name`,
/// `email`, and `phone`. Results are ordered by FTS5 rank (best match first).
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn search_contacts(conn: &Connection, query: &str) -> CrmResult<Vec<Contact>> {
    let fts_query = crate::storage::search::build_fts_match_query(query);

    let mut stmt = conn.prepare(
        r#"
        SELECT c.id, c.contact_type, c.first_name, c.last_name, c.org_name,
               c.email, c.phone, c.address, c.city, c.country, c.org_id,
               c.organization_id, c.notes, c.created_at, c.updated_at,
               c.deleted_at, c.device_id, c.lifecycle, c.owner
        FROM contacts c
        INNER JOIN contacts_fts fts ON c.rowid = fts.rowid
        WHERE contacts_fts MATCH ?1 AND c.deleted_at IS NULL
        ORDER BY rank
        LIMIT 50
        "#,
    )?;

    let rows = stmt.query_map(params![fts_query], row_to_contact)?;
    let contacts: Vec<Contact> = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!(
        "search_contacts query='{}' results={}",
        query,
        contacts.len()
    );
    Ok(contacts)
}

/// Finds active contacts with an email address matching case-insensitively.
pub fn find_active_contacts_by_email(conn: &Connection, email: &str) -> CrmResult<Vec<Contact>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, contact_type, first_name, last_name, org_name, email, phone,
               address, city, country, org_id, organization_id, notes,
               created_at, updated_at, deleted_at, device_id, lifecycle, owner
        FROM contacts
        WHERE LOWER(email) = LOWER(?1) AND deleted_at IS NULL
        "#,
    )?;

    let rows = stmt.query_map(params![email], row_to_contact)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Finds active contacts with a phone number matching exactly after trimming.
pub fn find_active_contacts_by_phone(conn: &Connection, phone: &str) -> CrmResult<Vec<Contact>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, contact_type, first_name, last_name, org_name, email, phone,
               address, city, country, org_id, organization_id, notes,
               created_at, updated_at, deleted_at, device_id, lifecycle, owner
        FROM contacts
        WHERE TRIM(phone) = TRIM(?1) AND deleted_at IS NULL
        "#,
    )?;

    let rows = stmt.query_map(params![phone], row_to_contact)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Finds active contact duplicate pairs by exact trimmed email or phone matches.
///
/// The older contact is returned as the default merge target. If a pair matches
/// by both email and phone, the email candidate is returned once.
pub fn find_active_contact_duplicate_candidates(
    conn: &Connection,
) -> CrmResult<Vec<ContactDuplicateCandidate>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            source.id AS source_id,
            COALESCE(
                NULLIF(TRIM(CASE
                    WHEN source.contact_type = 'organization' THEN source.org_name
                    ELSE source.first_name || ' ' || source.last_name
                END), ''),
                NULLIF(TRIM(source.email), ''),
                NULLIF(TRIM(source.phone), ''),
                source.id
            ) AS source_display_label,
            target.id AS target_id,
            COALESCE(
                NULLIF(TRIM(CASE
                    WHEN target.contact_type = 'organization' THEN target.org_name
                    ELSE target.first_name || ' ' || target.last_name
                END), ''),
                NULLIF(TRIM(target.email), ''),
                NULLIF(TRIM(target.phone), ''),
                target.id
            ) AS target_display_label,
            'email' AS match_type,
            TRIM(source.email) AS matched_value
        FROM contacts source
        INNER JOIN contacts target
            ON TRIM(source.email) = TRIM(target.email)
           AND (
                target.created_at < source.created_at
                OR (target.created_at = source.created_at AND target.id < source.id)
           )
        WHERE source.deleted_at IS NULL
          AND target.deleted_at IS NULL
          AND TRIM(source.email) <> ''

        UNION ALL

        SELECT
            source.id AS source_id,
            COALESCE(
                NULLIF(TRIM(CASE
                    WHEN source.contact_type = 'organization' THEN source.org_name
                    ELSE source.first_name || ' ' || source.last_name
                END), ''),
                NULLIF(TRIM(source.email), ''),
                NULLIF(TRIM(source.phone), ''),
                source.id
            ) AS source_display_label,
            target.id AS target_id,
            COALESCE(
                NULLIF(TRIM(CASE
                    WHEN target.contact_type = 'organization' THEN target.org_name
                    ELSE target.first_name || ' ' || target.last_name
                END), ''),
                NULLIF(TRIM(target.email), ''),
                NULLIF(TRIM(target.phone), ''),
                target.id
            ) AS target_display_label,
            'phone' AS match_type,
            TRIM(source.phone) AS matched_value
        FROM contacts source
        INNER JOIN contacts target
            ON TRIM(source.phone) = TRIM(target.phone)
           AND (
                target.created_at < source.created_at
                OR (target.created_at = source.created_at AND target.id < source.id)
           )
        WHERE source.deleted_at IS NULL
          AND target.deleted_at IS NULL
          AND TRIM(source.phone) <> ''
          AND NOT (
              TRIM(source.email) <> ''
              AND TRIM(source.email) = TRIM(target.email)
          )

        ORDER BY match_type, matched_value, target_display_label, source_display_label
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        let match_type: String = row.get(4)?;
        let matched_value: String = row.get(5)?;
        let reason = match match_type.as_str() {
            "email" => format!("Same email address: {}", matched_value),
            "phone" => format!("Same phone number: {}", matched_value),
            _ => format!("Same contact field: {}", matched_value),
        };

        Ok(ContactDuplicateCandidate {
            source_id: row.get(0)?,
            source_display_label: row.get(1)?,
            target_id: row.get(2)?,
            target_display_label: row.get(3)?,
            match_type,
            matched_value,
            reason,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Finds active contacts with an exact case-insensitive first/last name match.
pub fn find_active_contacts_by_name(
    conn: &Connection,
    first_name: &str,
    last_name: &str,
) -> CrmResult<Vec<Contact>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, contact_type, first_name, last_name, org_name, email, phone,
               address, city, country, org_id, organization_id, notes,
               created_at, updated_at, deleted_at, device_id, lifecycle, owner
        FROM contacts
        WHERE LOWER(first_name) = ?1 AND LOWER(last_name) = ?2
          AND deleted_at IS NULL
        "#,
    )?;

    let rows = stmt.query_map(params![first_name, last_name], row_to_contact)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Performs a paginated FTS5 search over contacts.
fn search_contacts_paged(
    conn: &Connection,
    query: &str,
    params: &ContactListParams,
) -> CrmResult<ContactListResult> {
    let fts_query = crate::storage::search::build_fts_match_query(query);
    let per_page = params.per_page.clamp(1, 200);
    let page = params.page.max(1);
    let offset = ((page as i64) - 1) * (per_page as i64);
    let limit = per_page as i64;
    let apply_type_filter = params.filter_type.is_some();
    let apply_lifecycle_filter = params.filter_lifecycle.is_some();
    let apply_owner_filter = params
        .filter_owner
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let apply_custom_filter = params
        .custom_field_def_id
        .as_ref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        && params
            .custom_field_query
            .as_ref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
    let filter_type = params.filter_type.as_deref().unwrap_or("");
    let filter_lifecycle = params.filter_lifecycle.as_deref().unwrap_or("");
    let filter_owner = params.filter_owner.as_deref().unwrap_or("");
    let custom_field_def_id = params.custom_field_def_id.as_deref().unwrap_or("");
    let custom_field_query = params
        .custom_field_query
        .as_ref()
        .map(|q| format!("%{}%", q.trim()))
        .unwrap_or_else(|| "%%".to_string());

    let total: u32 = conn
        .query_row(
            r#"
            SELECT COUNT(*)
            FROM contacts c
            INNER JOIN contacts_fts fts ON c.rowid = fts.rowid
            WHERE contacts_fts MATCH ?1
              AND c.deleted_at IS NULL
              AND (?2 = 0 OR c.contact_type = ?3)
              AND (?4 = 0 OR c.lifecycle = ?5)
              AND (
                ?6 = 0 OR EXISTS (
                  SELECT 1
                  FROM custom_field_values cfv
                  WHERE cfv.entity_id = c.id
                    AND cfv.field_def_id = ?7
                    AND LOWER(cfv.value) LIKE LOWER(?8)
                )
              )
              AND (?9 = 0 OR LOWER(TRIM(COALESCE(c.owner, ''))) = LOWER(TRIM(?10)))
            "#,
            params![
                fts_query,
                if apply_type_filter { 1 } else { 0 },
                filter_type,
                if apply_lifecycle_filter { 1 } else { 0 },
                filter_lifecycle,
                if apply_custom_filter { 1 } else { 0 },
                custom_field_def_id,
                custom_field_query,
                if apply_owner_filter { 1 } else { 0 },
                filter_owner,
            ],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn.prepare(
        r#"
        SELECT c.id, c.contact_type, c.first_name, c.last_name, c.org_name,
               c.email, c.phone, c.address, c.city, c.country, c.org_id,
               c.organization_id, c.notes, c.created_at, c.updated_at,
               c.deleted_at, c.device_id, c.lifecycle, c.owner
        FROM contacts c
        INNER JOIN contacts_fts fts ON c.rowid = fts.rowid
        WHERE contacts_fts MATCH ?1
          AND c.deleted_at IS NULL
          AND (?2 = 0 OR c.contact_type = ?3)
          AND (?4 = 0 OR c.lifecycle = ?5)
          AND (
            ?6 = 0 OR EXISTS (
              SELECT 1
              FROM custom_field_values cfv
              WHERE cfv.entity_id = c.id
                AND cfv.field_def_id = ?7
                AND LOWER(cfv.value) LIKE LOWER(?8)
            )
          )
          AND (?9 = 0 OR LOWER(TRIM(COALESCE(c.owner, ''))) = LOWER(TRIM(?10)))
        ORDER BY rank
        LIMIT ?11 OFFSET ?12
        "#,
    )?;

    let rows = stmt.query_map(
        params![
            fts_query,
            if apply_type_filter { 1 } else { 0 },
            filter_type,
            if apply_lifecycle_filter { 1 } else { 0 },
            filter_lifecycle,
            if apply_custom_filter { 1 } else { 0 },
            custom_field_def_id,
            custom_field_query,
            if apply_owner_filter { 1 } else { 0 },
            filter_owner,
            limit,
            offset
        ],
        row_to_contact,
    )?;
    let contacts: Vec<Contact> = rows.collect::<Result<Vec<_>, _>>()?;

    Ok(ContactListResult {
        contacts,
        total,
        page,
        per_page,
    })
}

/// Maps a `rusqlite::Row` to a [`Contact`].
///
/// Column order must match the SELECT in all queries above.
fn row_to_contact(row: &rusqlite::Row<'_>) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: row.get(0)?,
        contact_type: row.get(1)?,
        first_name: row.get(2)?,
        last_name: row.get(3)?,
        org_name: row.get(4)?,
        email: row.get(5)?,
        phone: row.get(6)?,
        address: row.get(7)?,
        city: row.get(8)?,
        country: row.get(9)?,
        org_id: row.get(10)?,
        organization_id: row.get(11)?,
        notes: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
        deleted_at: row.get(15)?,
        device_id: row.get(16)?,
        lifecycle: row
            .get::<_, Option<String>>(17)?
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "customer".to_string()),
        owner: row
            .get::<_, Option<String>>(18)?
            .and_then(|value| normalize_owner(Some(&value))),
    })
}

fn normalize_owner(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

/// Returns an allowlisted column name for ORDER BY, preventing SQL injection.
fn sanitize_sort_column<'a>(col: &'a str, default: &'a str) -> &'a str {
    match col {
        "first_name" | "last_name" | "org_name" | "email" | "phone" | "city" | "country"
        | "created_at" | "updated_at" | "contact_type" | "lifecycle" | "owner" => col,
        _ => default,
    }
}
