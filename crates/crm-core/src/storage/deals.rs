//! Deal CRUD operations and pipeline queries for 900CRM.
//!
//! This module provides all database operations for the `deals` table,
//! including pipeline stage management and aggregate summary queries.
//!
//! # Pipeline Stages
//!
//! The default pipeline stages (in order) are:
//! `Lead → Qualified → Proposal → Negotiation → Closed Won → Closed Lost`
//!
//! # Soft Delete
//!
//! Deals are never physically deleted. `soft_delete_deal` sets `deleted_at`
//! and all list queries exclude deleted records by default.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

// ─────────────────────────────────────────────────────────────────────────────
// Domain structs
// ─────────────────────────────────────────────────────────────────────────────

/// A sales deal moving through the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    /// UUID v4 primary key.
    pub id: String,

    /// Human-readable deal title.
    pub title: String,

    /// Monetary value of the deal.
    pub value: f64,

    /// ISO 4217 currency code (e.g. `"USD"`, `"EUR"`).
    pub currency: String,

    /// Current pipeline stage (e.g. `"Lead"`, `"Proposal"`).
    pub stage: String,

    /// Probability of closing (0–100).
    pub probability: i32,

    /// Expected close date in `YYYY-MM-DD` or ISO 8601 format.
    pub expected_close: Option<String>,

    /// Optional associated contact UUID.
    pub contact_id: Option<String>,

    /// Optional associated organization UUID.
    pub organization_id: Option<String>,

    /// Freeform notes about the deal.
    pub notes: String,

    /// Optional local owner name. This is not a user account.
    pub owner: Option<String>,

    /// ISO 8601 creation timestamp.
    pub created_at: String,

    /// ISO 8601 last-update timestamp.
    pub updated_at: String,

    /// ISO 8601 soft-delete timestamp (`None` = active).
    pub deleted_at: Option<String>,

    /// ID of the device that created or last modified this record.
    pub device_id: String,
}

/// Aggregated summary for a single pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSummary {
    /// Stage name (e.g. `"Lead"`, `"Proposal"`).
    pub stage: String,

    /// Number of active deals in this stage.
    pub count: i64,

    /// Sum of `value` across all active deals in this stage.
    pub total_value: f64,

    /// Probability-weighted sum: `SUM(value * probability / 100)`.
    pub weighted_value: f64,
}

/// Join record linking a contact to a deal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealContact {
    /// UUID v4 primary key.
    pub id: String,

    /// Linked deal UUID.
    pub deal_id: String,

    /// Linked contact UUID.
    pub contact_id: String,

    /// Optional relationship role, such as "decision maker".
    pub role: Option<String>,

    /// Whether this contact is the primary deal contact.
    pub is_primary: bool,

    /// ISO 8601 creation timestamp.
    pub created_at: String,

    /// ISO 8601 soft-delete timestamp (`None` = active).
    pub deleted_at: Option<String>,

    /// ID of the device that created or last modified this record.
    pub device_id: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// CRUD
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new deal record.
///
/// Generates a UUID for `id`, sets `created_at` and `updated_at` to now.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn create_deal(
    conn: &Connection,
    title: &str,
    value: f64,
    currency: &str,
    stage: &str,
    probability: i32,
    expected_close: Option<&str>,
    contact_id: Option<&str>,
    organization_id: Option<&str>,
    notes: &str,
    device_id: &str,
) -> CrmResult<Deal> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO deals
            (id, title, value, currency, stage, probability, expected_close,
             contact_id, organization_id, notes, created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        params![
            id,
            title,
            value,
            currency,
            stage,
            probability,
            expected_close,
            contact_id,
            organization_id,
            notes,
            now,
            now,
            device_id
        ],
    )?;

    log::debug!("Created deal id={} title={}", id, title);
    get_deal(conn, &id)
}

/// Retrieves a single active deal by UUID.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Deal does not exist or is soft-deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn get_deal(conn: &Connection, id: &str) -> CrmResult<Deal> {
    conn.query_row(
        r#"
        SELECT id, title, value, currency, stage, probability, expected_close,
               contact_id, organization_id, notes, created_at, updated_at, deleted_at, device_id, owner
        FROM deals
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![id],
        row_to_deal,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Deal '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

/// Lists all active deals, ordered by creation date descending.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_deals(conn: &Connection) -> CrmResult<Vec<Deal>> {
    list_deals_windowed(conn, None, 0)
}

/// Lists active deals, optionally windowed in SQL.
pub fn list_deals_windowed(
    conn: &Connection,
    limit: Option<u32>,
    offset: u32,
) -> CrmResult<Vec<Deal>> {
    let sql = if limit.is_some() {
        r#"
        SELECT id, title, value, currency, stage, probability, expected_close,
               contact_id, organization_id, notes, created_at, updated_at, deleted_at, device_id, owner
        FROM deals
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#
    } else {
        r#"
        SELECT id, title, value, currency, stage, probability, expected_close,
               contact_id, organization_id, notes, created_at, updated_at, deleted_at, device_id, owner
        FROM deals
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        "#
    };

    let mut stmt = conn.prepare(sql)?;
    let deals = if let Some(limit) = limit {
        let rows = stmt.query_map(params![limit as i64, offset as i64], row_to_deal)?;
        rows.collect::<Result<Vec<_>, _>>()?
    } else {
        let rows = stmt.query_map([], row_to_deal)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    log::debug!("list_deals: {} results", deals.len());
    Ok(deals)
}

/// Lists all active deals in a specific pipeline stage.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_deals_by_stage(conn: &Connection, stage: &str) -> CrmResult<Vec<Deal>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, value, currency, stage, probability, expected_close,
               contact_id, organization_id, notes, created_at, updated_at, deleted_at, device_id, owner
        FROM deals
        WHERE stage = ?1 AND deleted_at IS NULL
        ORDER BY updated_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![stage], row_to_deal)?;
    let deals = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!(
        "list_deals_by_stage stage={}: {} results",
        stage,
        deals.len()
    );
    Ok(deals)
}

/// Finds active deals with an exact case-insensitive title match after trimming.
pub fn find_active_deals_by_title(conn: &Connection, title: &str) -> CrmResult<Vec<Deal>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, value, currency, stage, probability, expected_close,
               contact_id, organization_id, notes, created_at, updated_at, deleted_at, device_id, owner
        FROM deals
        WHERE LOWER(TRIM(title)) = LOWER(TRIM(?1)) AND deleted_at IS NULL
        ORDER BY created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![title], row_to_deal)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Updates a deal's fields.
///
/// All `Option` parameters are applied only if `Some`. Sets `updated_at` to now.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Deal does not exist or is deleted.
/// - [`CrmError::Database`] — SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn update_deal(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    value: Option<f64>,
    currency: Option<&str>,
    stage: Option<&str>,
    probability: Option<i32>,
    expected_close: Option<Option<&str>>,
    contact_id: Option<Option<&str>>,
    organization_id: Option<Option<&str>>,
    notes: Option<&str>,
) -> CrmResult<Deal> {
    let current = get_deal(conn, id)?;
    let now = now_iso8601();

    conn.execute(
        r#"
        UPDATE deals SET
            title          = ?1,
            value          = ?2,
            currency       = ?3,
            stage          = ?4,
            probability    = ?5,
            expected_close = ?6,
            contact_id     = ?7,
            organization_id = ?8,
            notes          = ?9,
            updated_at     = ?10
        WHERE id = ?11 AND deleted_at IS NULL
        "#,
        params![
            title.unwrap_or(&current.title),
            value.unwrap_or(current.value),
            currency.unwrap_or(&current.currency),
            stage.unwrap_or(&current.stage),
            probability.unwrap_or(current.probability),
            expected_close.unwrap_or(current.expected_close.as_deref()),
            contact_id.unwrap_or(current.contact_id.as_deref()),
            organization_id.unwrap_or(current.organization_id.as_deref()),
            notes.unwrap_or(&current.notes),
            now,
            id
        ],
    )?;

    log::debug!("Updated deal id={}", id);
    get_deal(conn, id)
}

/// Moves a deal to a new pipeline stage and optionally updates probability.
///
/// If `probability` is `None`, the default probability for the target stage
/// is applied automatically:
/// - Lead → 10
/// - Qualified → 25
/// - Proposal → 50
/// - Negotiation → 75
/// - Closed Won → 100
/// - Closed Lost → 0
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Deal does not exist or is deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn move_deal_stage(
    conn: &Connection,
    id: &str,
    new_stage: &str,
    probability: Option<i32>,
) -> CrmResult<Deal> {
    let prob = probability.unwrap_or_else(|| default_probability_for_stage(new_stage));
    let now = now_iso8601();

    let changed = conn.execute(
        "UPDATE deals SET stage = ?1, probability = ?2, updated_at = ?3 WHERE id = ?4 AND deleted_at IS NULL",
        params![new_stage, prob, now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Deal '{}' not found", id)));
    }

    log::info!("Moved deal id={} to stage={}", id, new_stage);
    get_deal(conn, id)
}

/// Soft-deletes a deal by setting `deleted_at`.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Deal does not exist or is already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn soft_delete_deal(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE deals SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Deal '{}' not found or already deleted",
            id
        )));
    }

    log::info!("Soft-deleted deal id={}", id);
    Ok(())
}

/// Lists active contact links for a deal.
pub fn list_deal_contacts(conn: &Connection, deal_id: &str) -> CrmResult<Vec<DealContact>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, deal_id, contact_id, role, is_primary, created_at, deleted_at, device_id
        FROM deal_contacts
        WHERE deal_id = ?1 AND deleted_at IS NULL
        ORDER BY is_primary DESC, created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![deal_id], row_to_deal_contact)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Adds or updates an active deal-contact link.
pub fn add_deal_contact(
    conn: &Connection,
    deal_id: &str,
    contact_id: &str,
    role: Option<&str>,
    is_primary: bool,
    device_id: &str,
) -> CrmResult<DealContact> {
    if is_primary {
        clear_active_primary_deal_contacts(conn, deal_id)?;
    }

    if let Some(existing_id) = active_deal_contact_id(conn, deal_id, contact_id)? {
        conn.execute(
            r#"
            UPDATE deal_contacts
            SET role = ?1,
                is_primary = CASE WHEN ?2 THEN 1 ELSE is_primary END,
                device_id = ?3
            WHERE id = ?4 AND deleted_at IS NULL
            "#,
            params![role, is_primary, device_id, existing_id],
        )?;

        if is_primary {
            set_deal_primary_contact_mirror(conn, deal_id, Some(contact_id))?;
        }

        return get_deal_contact_including_deleted(conn, &existing_id);
    }

    let id = new_uuid();
    let now = now_iso8601();
    conn.execute(
        r#"
        INSERT INTO deal_contacts
            (id, deal_id, contact_id, role, is_primary, created_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![id, deal_id, contact_id, role, is_primary, now, device_id],
    )?;

    if is_primary {
        set_deal_primary_contact_mirror(conn, deal_id, Some(contact_id))?;
    }

    get_deal_contact_including_deleted(conn, &id)
}

/// Soft-deletes an active deal-contact link.
pub fn remove_deal_contact(
    conn: &Connection,
    deal_id: &str,
    contact_id: &str,
) -> CrmResult<DealContact> {
    let id = active_deal_contact_id(conn, deal_id, contact_id)?.ok_or_else(|| {
        CrmError::NotFound(format!(
            "Deal contact link deal='{}' contact='{}' not found",
            deal_id, contact_id
        ))
    })?;
    let before = get_deal_contact_including_deleted(conn, &id)?;
    let now = now_iso8601();

    conn.execute(
        r#"
        UPDATE deal_contacts
        SET deleted_at = ?1
        WHERE id = ?2 AND deleted_at IS NULL
        "#,
        params![now, id],
    )?;

    let deal = get_deal(conn, deal_id)?;
    if before.is_primary || deal.contact_id.as_deref() == Some(contact_id) {
        let next_primary = next_active_primary_contact_id(conn, deal_id)?;
        set_deal_primary_contact_mirror(conn, deal_id, next_primary.as_deref())?;
    }

    get_deal_contact_including_deleted(conn, &id)
}

/// Links or unlinks a deal to a normalized organization.
pub fn link_deal_to_organization(
    conn: &Connection,
    deal_id: &str,
    organization_id: Option<&str>,
) -> CrmResult<Deal> {
    let now = now_iso8601();
    let changed = conn.execute(
        r#"
        UPDATE deals
        SET organization_id = ?1,
            updated_at = ?2
        WHERE id = ?3 AND deleted_at IS NULL
        "#,
        params![organization_id, now, deal_id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Deal '{}' not found", deal_id)));
    }

    get_deal(conn, deal_id)
}

/// Returns a [`PipelineSummary`] for each pipeline stage.
///
/// Only active (non-deleted) deals are included. Stages with no deals are
/// included with `count = 0` and `total_value = 0.0`.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn get_pipeline_summary(conn: &Connection) -> CrmResult<Vec<PipelineSummary>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            stage,
            COUNT(*)                                AS count,
            COALESCE(SUM(value), 0.0)               AS total_value,
            COALESCE(SUM(value * probability / 100.0), 0.0) AS weighted_value
        FROM deals
        WHERE deleted_at IS NULL
        GROUP BY stage
        ORDER BY CASE stage
            WHEN 'Lead'        THEN 1
            WHEN 'Qualified'   THEN 2
            WHEN 'Proposal'    THEN 3
            WHEN 'Negotiation' THEN 4
            WHEN 'Closed Won'  THEN 5
            WHEN 'Closed Lost' THEN 6
            ELSE 7
        END
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(PipelineSummary {
            stage: row.get(0)?,
            count: row.get(1)?,
            total_value: row.get(2)?,
            weighted_value: row.get(3)?,
        })
    })?;

    let summaries = rows.collect::<Result<Vec<_>, _>>()?;
    log::debug!("get_pipeline_summary: {} stages", summaries.len());
    Ok(summaries)
}

/// Returns the average age in days for active, non-deleted deals.
pub fn get_average_active_deal_age_days(conn: &Connection) -> CrmResult<f64> {
    let average = conn
        .query_row(
            r#"
            SELECT AVG(
                CAST(
                    (julianday('now') - julianday(created_at))
                AS REAL)
            )
            FROM deals
            WHERE deleted_at IS NULL
              AND stage NOT IN ('Closed Won', 'Closed Lost')
            "#,
            [],
            |row| row.get::<_, Option<f64>>(0),
        )?
        .unwrap_or(0.0);

    Ok(average)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Maps a `rusqlite::Row` to a [`Deal`].
fn row_to_deal(row: &rusqlite::Row<'_>) -> rusqlite::Result<Deal> {
    Ok(Deal {
        id: row.get(0)?,
        title: row.get(1)?,
        value: row.get(2)?,
        currency: row.get(3)?,
        stage: row.get(4)?,
        probability: row.get(5)?,
        expected_close: row.get(6)?,
        contact_id: row.get(7)?,
        organization_id: row.get(8)?,
        notes: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        deleted_at: row.get(12)?,
        device_id: row.get(13)?,
        owner: row.get::<_, Option<String>>(14)?.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }),
    })
}

/// Sets or clears a deal's local owner name.
pub fn set_deal_owner(conn: &Connection, id: &str, owner: Option<&str>) -> CrmResult<Deal> {
    let _current = get_deal(conn, id)?;
    let now = now_iso8601();
    let normalized = owner
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    conn.execute(
        "UPDATE deals SET owner = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
        params![normalized, now, id],
    )?;
    get_deal(conn, id)
}

fn row_to_deal_contact(row: &rusqlite::Row<'_>) -> rusqlite::Result<DealContact> {
    Ok(DealContact {
        id: row.get(0)?,
        deal_id: row.get(1)?,
        contact_id: row.get(2)?,
        role: row.get(3)?,
        is_primary: row.get(4)?,
        created_at: row.get(5)?,
        deleted_at: row.get(6)?,
        device_id: row.get(7)?,
    })
}

fn get_deal_contact_including_deleted(conn: &Connection, id: &str) -> CrmResult<DealContact> {
    conn.query_row(
        r#"
        SELECT id, deal_id, contact_id, role, is_primary, created_at, deleted_at, device_id
        FROM deal_contacts
        WHERE id = ?1
        "#,
        params![id],
        row_to_deal_contact,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Deal contact link '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

fn active_deal_contact_id(
    conn: &Connection,
    deal_id: &str,
    contact_id: &str,
) -> CrmResult<Option<String>> {
    conn.query_row(
        r#"
        SELECT id
        FROM deal_contacts
        WHERE deal_id = ?1 AND contact_id = ?2 AND deleted_at IS NULL
        ORDER BY created_at ASC, id ASC
        LIMIT 1
        "#,
        params![deal_id, contact_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

fn clear_active_primary_deal_contacts(conn: &Connection, deal_id: &str) -> CrmResult<()> {
    conn.execute(
        r#"
        UPDATE deal_contacts
        SET is_primary = 0
        WHERE deal_id = ?1 AND deleted_at IS NULL AND is_primary = 1
        "#,
        params![deal_id],
    )?;
    Ok(())
}

fn next_active_primary_contact_id(conn: &Connection, deal_id: &str) -> CrmResult<Option<String>> {
    conn.query_row(
        r#"
        SELECT contact_id
        FROM deal_contacts
        WHERE deal_id = ?1 AND deleted_at IS NULL AND is_primary = 1
        ORDER BY created_at ASC, id ASC
        LIMIT 1
        "#,
        params![deal_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

fn set_deal_primary_contact_mirror(
    conn: &Connection,
    deal_id: &str,
    contact_id: Option<&str>,
) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        r#"
        UPDATE deals
        SET contact_id = ?1,
            updated_at = ?2
        WHERE id = ?3 AND deleted_at IS NULL
        "#,
        params![contact_id, now, deal_id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Deal '{}' not found", deal_id)));
    }

    Ok(())
}

/// Returns the default win probability (0–100) for a given pipeline stage.
fn default_probability_for_stage(stage: &str) -> i32 {
    match stage {
        "Lead" => 10,
        "Qualified" => 25,
        "Proposal" => 50,
        "Negotiation" => 75,
        "Closed Won" => 100,
        "Closed Lost" => 0,
        _ => 20,
    }
}
