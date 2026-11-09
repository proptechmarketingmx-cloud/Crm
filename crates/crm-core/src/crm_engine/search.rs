//! Unified cross-entity search for 900CRM.
//!
//! This module provides the [`unified_search`] function that searches across
//! contacts, organizations, deals, activities, notes, and tags simultaneously and returns a combined list
//! of [`SearchResult`] items.
//!
//! # Search Strategy
//!
//! - Contacts use the storage full-text repository first.
//! - Organizations, deals, activities, notes, and tags use storage FTS
//!   repositories first with storage-owned text fallback.
//!
//! Results from all sources are combined in stable entity order.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::storage::search::{
    self as search_storage, ActivitySearchRecord, ContactSearchRecord, DealSearchRecord,
    NoteSearchRecord, OrganizationSearchRecord, TagSearchRecord,
};
use crate::utils::errors::CrmResult;

const DEFAULT_SEARCH_LIMIT: u32 = 30;
const MAX_SEARCH_LIMIT: u32 = 100;

// ─────────────────────────────────────────────────────────────────────────────
// Domain structs
// ─────────────────────────────────────────────────────────────────────────────

/// Entity type represented by a global search result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchEntityType {
    Contact,
    Organization,
    Deal,
    Activity,
    Note,
    Tag,
}

/// A single search result from global search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    /// Entity type: contact, organization, deal, activity, note, or tag.
    pub entity_type: SearchEntityType,

    /// UUID of the matching entity.
    pub entity_id: String,

    /// Primary display title (e.g. full name, deal title, activity title).
    pub title: String,

    /// Secondary subtitle (e.g. email, stage, activity type).
    pub subtitle: String,

    /// Which field produced the match (e.g. `"email"`, `"title"`, `"description"`).
    pub match_field: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Unified search
// ─────────────────────────────────────────────────────────────────────────────

/// Searches all supported CRM entity types for `query`.
///
/// Returns at most `limit` results, sorted by stable entity type order:
/// contacts, organizations, deals, activities, notes, then tags.
///
/// # Parameters
///
/// - `conn` — database connection.
/// - `query` — Search string. Trimmed before use.
/// - `limit` — Maximum number of results to return. `0` returns no results,
///   otherwise clamped to [1, 100].
///
/// # Errors
///
/// Returns [`crate::utils::errors::CrmError::Database`] on storage failure.
pub fn unified_search(conn: &Connection, query: &str, limit: u32) -> CrmResult<Vec<SearchResult>> {
    let q = query.trim();
    if q.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }

    let clamped_limit = limit.min(MAX_SEARCH_LIMIT) as i64;
    let mut results: Vec<SearchResult> = Vec::new();

    let contact_results = match search_storage::search_contacts_full_text(conn, q, clamped_limit) {
        Ok(records) if !records.is_empty() => records,
        Ok(_) | Err(_) => search_storage::search_contacts_fallback(conn, q, clamped_limit)?,
    };
    results.extend(contact_results.into_iter().map(contact_record_to_result));

    let organization_results = search_storage::search_organizations(conn, q, clamped_limit)?;
    results.extend(
        organization_results
            .into_iter()
            .map(organization_record_to_result),
    );

    let deal_results = search_storage::search_deals(conn, q, clamped_limit)?;
    results.extend(deal_results.into_iter().map(deal_record_to_result));

    let activity_results = search_storage::search_activities(conn, q, clamped_limit)?;
    results.extend(activity_results.into_iter().map(activity_record_to_result));

    let note_results = search_storage::search_notes(conn, q, clamped_limit)?;
    results.extend(note_results.into_iter().map(note_record_to_result));

    let tag_results = search_storage::search_tags(conn, q, clamped_limit)?;
    results.extend(tag_results.into_iter().map(tag_record_to_result));

    results.truncate(clamped_limit as usize);

    log::debug!("unified_search query='{}' -> {} results", q, results.len());

    Ok(results)
}

pub fn default_limit() -> u32 {
    DEFAULT_SEARCH_LIMIT
}

// ─────────────────────────────────────────────────────────────────────────────
// Result mapping
// ─────────────────────────────────────────────────────────────────────────────

fn contact_record_to_result(record: ContactSearchRecord) -> SearchResult {
    let subtitle = if !record.email.is_empty() {
        record.email
    } else {
        record.org_name
    };

    SearchResult {
        entity_type: SearchEntityType::Contact,
        entity_id: record.id,
        title: record.full_name.trim().to_string(),
        subtitle,
        match_field: record.match_field,
    }
}

fn organization_record_to_result(record: OrganizationSearchRecord) -> SearchResult {
    let location = [record.city, record.country]
        .into_iter()
        .flatten()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    let subtitle = record.email.or(record.website).unwrap_or(location);

    SearchResult {
        entity_type: SearchEntityType::Organization,
        entity_id: record.id,
        title: record.name,
        subtitle,
        match_field: record.match_field,
    }
}

fn deal_record_to_result(record: DealSearchRecord) -> SearchResult {
    SearchResult {
        entity_type: SearchEntityType::Deal,
        entity_id: record.id,
        title: record.title,
        subtitle: format!("{} — {:.0} {}", record.stage, record.value, record.currency),
        match_field: record.match_field,
    }
}

fn activity_record_to_result(record: ActivitySearchRecord) -> SearchResult {
    let subtitle = match record.due_date {
        Some(d) => format!(
            "{} — due {}",
            record.activity_type,
            d.chars().take(10).collect::<String>()
        ),
        None => record.activity_type,
    };

    SearchResult {
        entity_type: SearchEntityType::Activity,
        entity_id: record.id,
        title: record.title,
        subtitle,
        match_field: record.match_field,
    }
}

fn note_record_to_result(record: NoteSearchRecord) -> SearchResult {
    let title = if record.content.chars().count() > 80 {
        format!("{}...", record.content.chars().take(80).collect::<String>())
    } else {
        record.content
    };

    SearchResult {
        entity_type: SearchEntityType::Note,
        entity_id: record.id,
        title,
        subtitle: format!("{}:{}", record.entity_type, record.entity_id),
        match_field: record.match_field,
    }
}

fn tag_record_to_result(record: TagSearchRecord) -> SearchResult {
    SearchResult {
        entity_type: SearchEntityType::Tag,
        entity_id: record.id,
        title: record.name,
        subtitle: record.color,
        match_field: record.match_field,
    }
}
