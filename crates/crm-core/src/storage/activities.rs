//! Activity CRUD operations and scheduling queries for 900CRM.
//!
//! Activities represent tasks, calls, meetings, emails, visits, WhatsApp, and SMS.
//! They can be attached to a contact, a deal, or both.
//!
//! # Activity Types
//!
//! The `activity_type` field is a freeform string. Standard values are:
//! `"task"`, `"call"`, `"meeting"`, `"email"`, `"note"`, `"visit"`, `"whatsapp"`, `"sms"`.
//!
//! # Soft Delete
//!
//! Activities support the same soft-delete pattern as contacts and deals.

use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::utils::{
    datetime::{local_today_ymd, now_iso8601},
    errors::{CrmError, CrmResult},
    uuid::new_uuid,
};

// ─────────────────────────────────────────────────────────────────────────────
// Domain structs
// ─────────────────────────────────────────────────────────────────────────────

/// An activity (task, call, meeting, or email) in the CRM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// UUID v4 primary key.
    pub id: String,

    /// Activity type: `"task"`, `"call"`, `"meeting"`, `"email"`, etc.
    pub activity_type: String,

    /// Short title or subject of the activity.
    pub title: String,

    /// Longer description or body.
    pub description: String,

    /// Optional due date (ISO 8601 or `YYYY-MM-DD`).
    pub due_date: Option<String>,

    /// Whether the activity has been completed (`true`) or is still pending.
    pub completed: bool,

    /// Optional associated contact UUID.
    pub contact_id: Option<String>,

    /// Optional associated deal UUID.
    pub deal_id: Option<String>,

    /// ISO 8601 creation timestamp.
    pub created_at: String,

    /// ISO 8601 last-update timestamp.
    pub updated_at: String,

    /// ISO 8601 soft-delete timestamp (`None` = active).
    pub deleted_at: Option<String>,

    /// ID of the device that created or last modified this record.
    pub device_id: String,
}

/// Aggregate activity counts used by dashboard orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStatsCounts {
    pub total: i64,
    pub completed: i64,
    pub overdue: i64,
    pub due_today: i64,
}

/// Supported first-class relationship target types for activity links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityLinkEntityType {
    Contact,
    Organization,
    Deal,
}

impl ActivityLinkEntityType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contact => "contact",
            Self::Organization => "organization",
            Self::Deal => "deal",
        }
    }
}

impl TryFrom<&str> for ActivityLinkEntityType {
    type Error = CrmError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "contact" => Ok(Self::Contact),
            "organization" => Ok(Self::Organization),
            "deal" => Ok(Self::Deal),
            other => Err(CrmError::InvalidInput(format!(
                "Unsupported activity link entity type '{}'",
                other
            ))),
        }
    }
}

/// A first-class relationship from an activity to a supported CRM entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLink {
    pub id: String,
    pub activity_id: String,
    pub entity_type: ActivityLinkEntityType,
    pub entity_id: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
    pub device_id: String,
}

/// Loads aggregate activity counts for dashboard statistics.
pub fn get_activity_stats_counts(
    conn: &Connection,
    now: &str,
    local_today: &str,
) -> CrmResult<ActivityStatsCounts> {
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM activities WHERE deleted_at IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM activities WHERE deleted_at IS NULL AND completed = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let overdue: i64 = conn
        .query_row(
            r#"
            SELECT COUNT(*) FROM activities
            WHERE deleted_at IS NULL
              AND completed = 0
              AND due_date IS NOT NULL
              AND (
                (length(due_date) = 10 AND due_date < ?2)
                OR (length(due_date) > 10 AND due_date < ?1)
              )
            "#,
            params![now, local_today],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let utc_today_prefix = format!("{}%", &now[..10.min(now.len())]);
    let due_today: i64 = conn
        .query_row(
            r#"
            SELECT COUNT(*) FROM activities
            WHERE deleted_at IS NULL
              AND completed = 0
              AND (
                (length(due_date) = 10 AND due_date = ?1)
                OR (length(due_date) > 10 AND due_date LIKE ?2)
              )
            "#,
            params![local_today, utc_today_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(ActivityStatsCounts {
        total,
        completed,
        overdue,
        due_today,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// CRUD
// ─────────────────────────────────────────────────────────────────────────────

/// Creates a new activity record.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn create_activity(
    conn: &Connection,
    activity_type: &str,
    title: &str,
    description: &str,
    due_date: Option<&str>,
    contact_id: Option<&str>,
    deal_id: Option<&str>,
    device_id: &str,
) -> CrmResult<Activity> {
    let id = new_uuid();
    let now = now_iso8601();

    conn.execute(
        r#"
        INSERT INTO activities
            (id, activity_type, title, description, due_date, completed,
             contact_id, deal_id, created_at, updated_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            id,
            activity_type,
            title,
            description,
            due_date,
            contact_id,
            deal_id,
            now,
            now,
            device_id
        ],
    )?;

    log::debug!("Created activity id={} type={}", id, activity_type);
    get_activity(conn, &id)
}

/// Retrieves a single active activity by UUID.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or soft-deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn get_activity(conn: &Connection, id: &str) -> CrmResult<Activity> {
    conn.query_row(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
        params![id],
        row_to_activity,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CrmError::NotFound(format!("Activity '{}' not found", id))
        }
        other => CrmError::Database(other.to_string()),
    })
}

/// Lists all active activities, ordered by due date ascending (nulls last).
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_activities(conn: &Connection) -> CrmResult<Vec<Activity>> {
    list_activities_windowed(conn, None, 0)
}

/// Lists active activities, optionally windowed in SQL.
pub fn list_activities_windowed(
    conn: &Connection,
    limit: Option<u32>,
    offset: u32,
) -> CrmResult<Vec<Activity>> {
    let sql = if limit.is_some() {
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deleted_at IS NULL
        ORDER BY due_date ASC NULLS LAST, created_at DESC
        LIMIT ?1 OFFSET ?2
        "#
    } else {
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deleted_at IS NULL
        ORDER BY due_date ASC NULLS LAST, created_at DESC
        "#
    };

    let mut stmt = conn.prepare(sql)?;
    let activities = if let Some(limit) = limit {
        let rows = stmt.query_map(params![limit as i64, offset as i64], row_to_activity)?;
        rows.collect::<Result<Vec<_>, _>>()?
    } else {
        let rows = stmt.query_map([], row_to_activity)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    log::debug!("list_activities: {} results", activities.len());
    Ok(activities)
}

/// Lists all active activities for a specific contact.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_activities_for_contact(
    conn: &Connection,
    contact_id: &str,
) -> CrmResult<Vec<Activity>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE contact_id = ?1 AND deleted_at IS NULL
        ORDER BY due_date ASC NULLS LAST, created_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![contact_id], row_to_activity)?;
    let activities = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!(
        "list_activities_for_contact contact_id={}: {} results",
        contact_id,
        activities.len()
    );
    Ok(activities)
}

/// Lists all active activities for a specific deal.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_activities_for_deal(conn: &Connection, deal_id: &str) -> CrmResult<Vec<Activity>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deal_id = ?1 AND deleted_at IS NULL
        ORDER BY due_date ASC NULLS LAST, created_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![deal_id], row_to_activity)?;
    let activities = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!(
        "list_activities_for_deal deal_id={}: {} results",
        deal_id,
        activities.len()
    );
    Ok(activities)
}

/// Lists active activities linked to any of the given deals (legacy deal_id or
/// activity_links).
pub fn list_activities_for_deal_ids(
    conn: &Connection,
    deal_ids: &[String],
) -> CrmResult<Vec<Activity>> {
    if deal_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = deal_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deleted_at IS NULL
          AND (
            deal_id IN ({placeholders})
            OR id IN (
              SELECT activity_id FROM activity_links
              WHERE deleted_at IS NULL
                AND entity_type = 'deal'
                AND entity_id IN ({placeholders})
            )
          )
        ORDER BY due_date ASC NULLS LAST, created_at DESC
        "#
    );
    let mut values: Vec<&str> = deal_ids.iter().map(String::as_str).collect();
    values.extend(deal_ids.iter().map(String::as_str));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(values), row_to_activity)?;
    let activities = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(activities)
}

/// Lists active links for many activities in one query.
pub fn list_activity_links_for_activities(
    conn: &Connection,
    activity_ids: &[String],
) -> CrmResult<Vec<ActivityLink>> {
    if activity_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = activity_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        r#"
        SELECT id, activity_id, entity_type, entity_id, created_at, deleted_at, device_id
        FROM activity_links
        WHERE deleted_at IS NULL AND activity_id IN ({placeholders})
        ORDER BY created_at ASC, id ASC
        "#
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params_from_iter(activity_ids.iter().map(String::as_str)),
        row_to_activity_link,
    )?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Lists upcoming (incomplete, future due date) activities.
///
/// Returns at most `limit` activities ordered by `due_date` ascending.
/// Excludes completed and soft-deleted activities.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_upcoming_activities(conn: &Connection, limit: u32) -> CrmResult<Vec<Activity>> {
    let now = now_iso8601();
    let local_today = local_today_ymd();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deleted_at IS NULL
          AND completed = 0
          AND due_date IS NOT NULL
          AND (
            (length(due_date) = 10 AND due_date >= ?2)
            OR (length(due_date) > 10 AND due_date >= ?1)
          )
        ORDER BY due_date ASC
        LIMIT ?3
        "#,
    )?;

    let rows = stmt.query_map(params![now, local_today, limit as i64], row_to_activity)?;
    let activities = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!("list_upcoming_activities: {} results", activities.len());
    Ok(activities)
}

/// Lists overdue (incomplete, past due date) activities.
///
/// Returns activities ordered by `due_date` descending (most overdue first).
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_overdue_activities(conn: &Connection) -> CrmResult<Vec<Activity>> {
    let now = now_iso8601();
    let local_today = local_today_ymd();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_type, title, description, due_date, completed,
               contact_id, deal_id, created_at, updated_at, deleted_at, device_id
        FROM activities
        WHERE deleted_at IS NULL
          AND completed = 0
          AND due_date IS NOT NULL
          AND (
            (length(due_date) = 10 AND due_date < ?2)
            OR (length(due_date) > 10 AND due_date < ?1)
          )
        ORDER BY due_date ASC
        "#,
    )?;

    let rows = stmt.query_map(params![now, local_today], row_to_activity)?;
    let activities = rows.collect::<Result<Vec<_>, _>>()?;

    log::debug!("list_overdue_activities: {} results", activities.len());
    Ok(activities)
}

/// Marks an activity as completed.
///
/// Sets `completed = 1` and `updated_at` to now.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn mark_complete(conn: &Connection, id: &str) -> CrmResult<Activity> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE activities SET completed = 1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Activity '{}' not found", id)));
    }

    log::info!("Marked activity id={} as complete", id);
    get_activity(conn, id)
}

/// Marks an activity as incomplete.
///
/// Sets `completed = 0` and `updated_at` to now.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn mark_incomplete(conn: &Connection, id: &str) -> CrmResult<Activity> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE activities SET completed = 0, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!("Activity '{}' not found", id)));
    }

    log::info!("Marked activity id={} as incomplete", id);
    get_activity(conn, id)
}

/// Updates an activity's fields.
///
/// All `Option` parameters are applied only if `Some`.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or deleted.
/// - [`CrmError::Database`] — SQL failure.
#[allow(clippy::too_many_arguments)]
pub fn update_activity(
    conn: &Connection,
    id: &str,
    activity_type: Option<&str>,
    title: Option<&str>,
    description: Option<&str>,
    due_date: Option<Option<&str>>,
    completed: Option<bool>,
    contact_id: Option<Option<&str>>,
    deal_id: Option<Option<&str>>,
) -> CrmResult<Activity> {
    let current = get_activity(conn, id)?;
    let now = now_iso8601();

    conn.execute(
        r#"
        UPDATE activities SET
            activity_type = ?1,
            title         = ?2,
            description   = ?3,
            due_date      = ?4,
            completed     = ?5,
            contact_id    = ?6,
            deal_id       = ?7,
            updated_at    = ?8
        WHERE id = ?9 AND deleted_at IS NULL
        "#,
        params![
            activity_type.unwrap_or(&current.activity_type),
            title.unwrap_or(&current.title),
            description.unwrap_or(&current.description),
            due_date.unwrap_or(current.due_date.as_deref()),
            completed
                .map(|c| c as i32)
                .unwrap_or(current.completed as i32),
            contact_id.unwrap_or(current.contact_id.as_deref()),
            deal_id.unwrap_or(current.deal_id.as_deref()),
            now,
            id
        ],
    )?;

    log::debug!("Updated activity id={}", id);
    get_activity(conn, id)
}

/// Lists active first-class links for an activity.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on SQL failure.
pub fn list_activity_links(conn: &Connection, activity_id: &str) -> CrmResult<Vec<ActivityLink>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, activity_id, entity_type, entity_id, created_at, deleted_at, device_id
        FROM activity_links
        WHERE activity_id = ?1 AND deleted_at IS NULL
        ORDER BY created_at ASC, id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![activity_id], row_to_activity_link)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Returns the active link for an exact activity/entity pair, if present.
pub fn get_active_activity_link(
    conn: &Connection,
    activity_id: &str,
    entity_type: ActivityLinkEntityType,
    entity_id: &str,
) -> CrmResult<Option<ActivityLink>> {
    conn.query_row(
        r#"
        SELECT id, activity_id, entity_type, entity_id, created_at, deleted_at, device_id
        FROM activity_links
        WHERE activity_id = ?1
          AND entity_type = ?2
          AND entity_id = ?3
          AND deleted_at IS NULL
        "#,
        params![activity_id, entity_type.as_str(), entity_id],
        row_to_activity_link,
    )
    .optional()
    .map_err(Into::into)
}

/// Creates an activity link, returning the existing active link when present.
///
/// Reference validation is intentionally done in the service layer so the
/// storage function can stay focused on persistence and transaction reuse.
pub fn add_activity_link(
    conn: &Connection,
    activity_id: &str,
    entity_type: ActivityLinkEntityType,
    entity_id: &str,
    device_id: &str,
) -> CrmResult<ActivityLink> {
    if let Some(link) = get_active_activity_link(conn, activity_id, entity_type, entity_id)? {
        return Ok(link);
    }

    let id = new_uuid();
    let now = now_iso8601();
    conn.execute(
        r#"
        INSERT INTO activity_links
            (id, activity_id, entity_type, entity_id, created_at, deleted_at, device_id)
        VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6)
        "#,
        params![
            id,
            activity_id,
            entity_type.as_str(),
            entity_id,
            now,
            device_id
        ],
    )?;

    get_active_activity_link(conn, activity_id, entity_type, entity_id)?.ok_or_else(|| {
        CrmError::Database(format!(
            "Activity link '{}' was inserted but could not be reloaded",
            id
        ))
    })
}

/// Soft-deletes an active activity link.
pub fn remove_activity_link(
    conn: &Connection,
    activity_id: &str,
    entity_type: ActivityLinkEntityType,
    entity_id: &str,
) -> CrmResult<ActivityLink> {
    let link =
        get_active_activity_link(conn, activity_id, entity_type, entity_id)?.ok_or_else(|| {
            CrmError::NotFound(format!(
                "Activity link '{}:{}:{}' not found",
                activity_id,
                entity_type.as_str(),
                entity_id
            ))
        })?;
    let now = now_iso8601();

    conn.execute(
        "UPDATE activity_links SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, link.id],
    )?;

    Ok(ActivityLink {
        deleted_at: Some(now),
        ..link
    })
}

/// Soft-deletes an activity.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Activity not found or already deleted.
/// - [`CrmError::Database`] — SQL failure.
pub fn soft_delete_activity(conn: &Connection, id: &str) -> CrmResult<()> {
    let now = now_iso8601();
    let changed = conn.execute(
        "UPDATE activities SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    )?;

    if changed == 0 {
        return Err(CrmError::NotFound(format!(
            "Activity '{}' not found or already deleted",
            id
        )));
    }

    log::info!("Soft-deleted activity id={}", id);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Maps a `rusqlite::Row` to an [`Activity`].
fn row_to_activity(row: &rusqlite::Row<'_>) -> rusqlite::Result<Activity> {
    let completed_int: i32 = row.get(5)?;
    Ok(Activity {
        id: row.get(0)?,
        activity_type: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        due_date: row.get(4)?,
        completed: completed_int != 0,
        contact_id: row.get(6)?,
        deal_id: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
        device_id: row.get(11)?,
    })
}

fn row_to_activity_link(row: &rusqlite::Row<'_>) -> rusqlite::Result<ActivityLink> {
    let raw_entity_type: String = row.get(2)?;
    let entity_type =
        ActivityLinkEntityType::try_from(raw_entity_type.as_str()).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(err))
        })?;

    Ok(ActivityLink {
        id: row.get(0)?,
        activity_id: row.get(1)?,
        entity_type,
        entity_id: row.get(3)?,
        created_at: row.get(4)?,
        deleted_at: row.get(5)?,
        device_id: row.get(6)?,
    })
}
