//! Repository queries for cross-entity search.

use rusqlite::{params, Connection};

use crate::utils::errors::CrmResult;

/// Builds a safe FTS5 MATCH string from arbitrary user input.
///
/// Each whitespace-separated token is wrapped in FTS5 phrase quotes — which
/// neutralizes metacharacters (`"`, `*`, `(`, `)`, `:`) and reserved keywords
/// such as `OR`/`AND`/`NOT` — and suffixed with a prefix `*` operator. Tokens
/// that become empty after stripping embedded quotes are dropped. Returns an
/// empty string for blank input.
pub(crate) fn build_fts_match_query(raw: &str) -> String {
    raw.split_whitespace()
        .filter_map(|token| {
            let cleaned: String = token.chars().filter(|c| *c != '"').collect();
            (!cleaned.is_empty()).then(|| format!("\"{}\"*", cleaned))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone)]
pub struct ContactSearchRecord {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub org_name: String,
    pub match_field: String,
}

#[derive(Debug, Clone)]
pub struct DealSearchRecord {
    pub id: String,
    pub title: String,
    pub stage: String,
    pub value: f64,
    pub currency: String,
    pub match_field: String,
}

#[derive(Debug, Clone)]
pub struct ActivitySearchRecord {
    pub id: String,
    pub title: String,
    pub activity_type: String,
    pub due_date: Option<String>,
    pub match_field: String,
}

#[derive(Debug, Clone)]
pub struct OrganizationSearchRecord {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub website: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub match_field: String,
}

#[derive(Debug, Clone)]
pub struct NoteSearchRecord {
    pub id: String,
    pub content: String,
    pub entity_type: String,
    pub entity_id: String,
    pub match_field: String,
}

#[derive(Debug, Clone)]
pub struct TagSearchRecord {
    pub id: String,
    pub name: String,
    pub color: String,
    pub match_field: String,
}

pub fn search_contacts_full_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<ContactSearchRecord>> {
    let fts_query = build_fts_match_query(query);
    let mut stmt = conn.prepare(
        r#"
        SELECT c.id,
               c.first_name || ' ' || c.last_name AS full_name,
               c.email,
               c.org_name
        FROM contacts c
        INNER JOIN contacts_fts fts ON c.rowid = fts.rowid
        WHERE contacts_fts MATCH ?1 AND c.deleted_at IS NULL
        ORDER BY rank
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![fts_query, limit], |row| {
        Ok(ContactSearchRecord {
            id: row.get(0)?,
            full_name: row.get(1)?,
            email: row.get(2)?,
            org_name: row.get(3)?,
            match_field: "fts".to_string(),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_contacts_fallback(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<ContactSearchRecord>> {
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        r#"
        SELECT id,
               first_name || ' ' || last_name AS full_name,
               email,
               org_name
        FROM contacts
        WHERE deleted_at IS NULL
          AND (first_name LIKE ?1 OR last_name LIKE ?1 OR email LIKE ?1
               OR org_name LIKE ?1 OR phone LIKE ?1)
        ORDER BY first_name ASC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(ContactSearchRecord {
            id: row.get(0)?,
            full_name: row.get(1)?,
            email: row.get(2)?,
            org_name: row.get(3)?,
            match_field: "name_or_email".to_string(),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_deals_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<DealSearchRecord>> {
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, stage, value, currency,
               CASE
                   WHEN title LIKE ?1 THEN 'title'
                   WHEN notes LIKE ?1 THEN 'notes'
                   ELSE 'text'
               END AS match_field
        FROM deals
        WHERE deleted_at IS NULL
          AND (title LIKE ?1 OR notes LIKE ?1)
        ORDER BY updated_at DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(DealSearchRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            stage: row.get(2)?,
            value: row.get(3)?,
            currency: row.get(4)?,
            match_field: row.get(5)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_deals(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<DealSearchRecord>> {
    match search_deals_full_text(conn, query, limit) {
        Ok(records) if !records.is_empty() => Ok(records),
        Ok(_) | Err(_) => search_deals_text(conn, query, limit),
    }
}

fn search_deals_full_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<DealSearchRecord>> {
    let fts_query = build_fts_match_query(query);
    let mut stmt = conn.prepare(
        r#"
        SELECT d.id, d.title, d.stage, d.value, d.currency
        FROM deals d
        INNER JOIN deals_fts fts ON d.rowid = fts.rowid
        WHERE deals_fts MATCH ?1
          AND d.deleted_at IS NULL
        ORDER BY rank, d.updated_at DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![fts_query, limit], |row| {
        Ok(DealSearchRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            stage: row.get(2)?,
            value: row.get(3)?,
            currency: row.get(4)?,
            match_field: "fts".to_string(),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_activities(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<ActivitySearchRecord>> {
    match search_activities_full_text(conn, query, limit) {
        Ok(records) if !records.is_empty() => Ok(records),
        Ok(_) | Err(_) => search_activities_text(conn, query, limit),
    }
}

fn search_activities_full_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<ActivitySearchRecord>> {
    let fts_query = build_fts_match_query(query);
    let mut stmt = conn.prepare(
        r#"
        SELECT a.id, a.title, a.activity_type, a.due_date
        FROM activities a
        INNER JOIN activities_fts fts ON a.rowid = fts.rowid
        WHERE activities_fts MATCH ?1
          AND a.deleted_at IS NULL
        ORDER BY rank, a.due_date ASC NULLS LAST
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![fts_query, limit], |row| {
        Ok(ActivitySearchRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            activity_type: row.get(2)?,
            due_date: row.get(3)?,
            match_field: "fts".to_string(),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_activities_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<ActivitySearchRecord>> {
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, activity_type, due_date,
               CASE
                   WHEN title LIKE ?1 THEN 'title'
                   WHEN activity_type LIKE ?1 THEN 'activity_type'
                   WHEN description LIKE ?1 THEN 'description'
                   ELSE 'text'
               END AS match_field
        FROM activities
        WHERE deleted_at IS NULL
          AND (title LIKE ?1 OR activity_type LIKE ?1 OR description LIKE ?1)
        ORDER BY due_date ASC NULLS LAST
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(ActivitySearchRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            activity_type: row.get(2)?,
            due_date: row.get(3)?,
            match_field: row.get(4)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_organizations(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<OrganizationSearchRecord>> {
    match search_organizations_full_text(conn, query, limit) {
        Ok(records) if !records.is_empty() => Ok(records),
        Ok(_) | Err(_) => search_organizations_text(conn, query, limit),
    }
}

fn search_organizations_full_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<OrganizationSearchRecord>> {
    let fts_query = build_fts_match_query(query);
    let mut stmt = conn.prepare(
        r#"
        SELECT o.id, o.name, o.email, o.website, o.city, o.country
        FROM organizations o
        INNER JOIN organizations_fts fts ON o.rowid = fts.rowid
        WHERE organizations_fts MATCH ?1
          AND o.deleted_at IS NULL
        ORDER BY rank, o.updated_at DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![fts_query, limit], |row| {
        Ok(OrganizationSearchRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            website: row.get(3)?,
            city: row.get(4)?,
            country: row.get(5)?,
            match_field: "fts".to_string(),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_organizations_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<OrganizationSearchRecord>> {
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, email, website, city, country,
               CASE
                   WHEN name LIKE ?1 THEN 'name'
                   WHEN COALESCE(email, '') LIKE ?1 THEN 'email'
                   WHEN COALESCE(website, '') LIKE ?1 THEN 'website'
                   WHEN COALESCE(description, '') LIKE ?1 THEN 'description'
                   WHEN COALESCE(city, '') LIKE ?1 THEN 'city'
                   WHEN COALESCE(country, '') LIKE ?1 THEN 'country'
                   ELSE 'text'
               END AS match_field
        FROM organizations
        WHERE deleted_at IS NULL
          AND (
              name LIKE ?1
              OR COALESCE(email, '') LIKE ?1
              OR COALESCE(phone, '') LIKE ?1
              OR COALESCE(website, '') LIKE ?1
              OR COALESCE(description, '') LIKE ?1
              OR COALESCE(city, '') LIKE ?1
              OR COALESCE(country, '') LIKE ?1
          )
        ORDER BY updated_at DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(OrganizationSearchRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            website: row.get(3)?,
            city: row.get(4)?,
            country: row.get(5)?,
            match_field: row.get(6)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_notes(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<NoteSearchRecord>> {
    match search_notes_full_text(conn, query, limit) {
        Ok(records) if !records.is_empty() => Ok(records),
        Ok(_) | Err(_) => search_notes_text(conn, query, limit),
    }
}

fn search_notes_full_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<NoteSearchRecord>> {
    let fts_query = build_fts_match_query(query);
    let mut stmt = conn.prepare(
        r#"
        SELECT n.id,
               COALESCE(NULLIF(n.body, ''), n.content) AS search_content,
               n.entity_type,
               n.entity_id
        FROM notes n
        INNER JOIN notes_fts fts ON n.rowid = fts.rowid
        WHERE notes_fts MATCH ?1
          AND n.deleted_at IS NULL
        ORDER BY rank, n.updated_at DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![fts_query, limit], |row| {
        Ok(NoteSearchRecord {
            id: row.get(0)?,
            content: row.get(1)?,
            entity_type: row.get(2)?,
            entity_id: row.get(3)?,
            match_field: "fts".to_string(),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_notes_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<NoteSearchRecord>> {
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        r#"
        SELECT id,
               COALESCE(NULLIF(body, ''), content) AS search_content,
               entity_type,
               entity_id,
               CASE
                   WHEN COALESCE(NULLIF(body, ''), content) LIKE ?1 THEN 'content'
                   ELSE 'text'
               END AS match_field
        FROM notes
        WHERE deleted_at IS NULL
          AND COALESCE(NULLIF(body, ''), content) LIKE ?1
        ORDER BY updated_at DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(NoteSearchRecord {
            id: row.get(0)?,
            content: row.get(1)?,
            entity_type: row.get(2)?,
            entity_id: row.get(3)?,
            match_field: row.get(4)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_tags(conn: &Connection, query: &str, limit: i64) -> CrmResult<Vec<TagSearchRecord>> {
    match search_tags_full_text(conn, query, limit) {
        Ok(records) if !records.is_empty() => Ok(records),
        Ok(_) | Err(_) => search_tags_text(conn, query, limit),
    }
}

fn search_tags_full_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<TagSearchRecord>> {
    let fts_query = build_fts_match_query(query);
    let mut stmt = conn.prepare(
        r#"
        SELECT t.id, t.name, t.color
        FROM tags t
        INNER JOIN tags_fts fts ON t.rowid = fts.rowid
        WHERE tags_fts MATCH ?1
          AND t.deleted_at IS NULL
        ORDER BY rank, t.name ASC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![fts_query, limit], |row| {
        Ok(TagSearchRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            match_field: "fts".to_string(),
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_tags_text(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> CrmResult<Vec<TagSearchRecord>> {
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, color,
               CASE
                   WHEN name LIKE ?1 THEN 'name'
                   WHEN color LIKE ?1 THEN 'color'
                   ELSE 'text'
               END AS match_field
        FROM tags
        WHERE deleted_at IS NULL
          AND (name LIKE ?1 OR color LIKE ?1)
        ORDER BY name ASC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(TagSearchRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            match_field: row.get(3)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
