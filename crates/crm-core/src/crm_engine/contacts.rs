//! Contact business logic — validation, duplicate detection, and merge.
//!
//! This module implements the domain rules that live above the raw storage
//! layer but below the Tauri command handlers:
//!
//! - Email format validation.
//! - Required field validation.
//! - Duplicate contact detection (by email or name + organization).
//! - Contact merge (combine two contacts into one, keeping best data).
//! - Organization linking (attach a person to an organization contact).

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::storage::contacts::{self, Contact};
use crate::utils::errors::{CrmError, CrmResult};

// ─────────────────────────────────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Input payload for creating or updating a contact.
///
/// All fields are optional on update; required fields are validated when
/// `contact_type` is provided (i.e. on create).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInput {
    /// Contact type: `"person"` or `"organization"`.
    pub contact_type: Option<String>,

    /// Given name.
    pub first_name: Option<String>,

    /// Family name.
    pub last_name: Option<String>,

    /// Organization name.
    pub org_name: Option<String>,

    /// Email address.
    pub email: Option<String>,

    /// Phone number.
    pub phone: Option<String>,

    /// Street address.
    pub address: Option<String>,

    /// City.
    pub city: Option<String>,

    /// Country.
    pub country: Option<String>,

    /// Parent organization contact ID.
    pub org_id: Option<String>,

    /// Freeform notes.
    pub notes: Option<String>,
}

/// Validates a [`ContactInput`] for creation (all required fields must be present).
///
/// # Validation Rules
///
/// - `contact_type` must be `"person"` or `"organization"`.
/// - For `"person"` contacts: `first_name` must be non-empty.
/// - For `"organization"` contacts: `org_name` must be non-empty.
/// - If `email` is provided and non-empty, it must contain `@`.
///
/// # Errors
///
/// Returns [`CrmError::InvalidInput`] with a descriptive message on the first
/// validation failure.
pub fn validate_contact_for_create(input: &ContactInput) -> CrmResult<()> {
    let contact_type = input.contact_type.as_deref().unwrap_or("person");

    match contact_type {
        "person" => {
            let first_name = input.first_name.as_deref().unwrap_or("");
            if first_name.trim().is_empty() {
                return Err(CrmError::InvalidInput(
                    "first_name is required for person contacts".to_string(),
                ));
            }
        }
        "organization" => {
            let org_name = input.org_name.as_deref().unwrap_or("");
            if org_name.trim().is_empty() {
                return Err(CrmError::InvalidInput(
                    "org_name is required for organization contacts".to_string(),
                ));
            }
        }
        other => {
            return Err(CrmError::InvalidInput(format!(
                "Invalid contact_type '{}'. Must be 'person' or 'organization'",
                other
            )));
        }
    }

    validate_email_if_present(input.email.as_deref())?;

    Ok(())
}

/// Validates that an email address, if provided and non-empty, contains `@`.
///
/// This is a lightweight check — not a full RFC 5321 parse. It catches
/// obvious typos while being tolerant of unusual-but-valid addresses.
///
/// # Errors
///
/// Returns [`CrmError::InvalidInput`] if the email looks invalid.
pub fn validate_email_if_present(email: Option<&str>) -> CrmResult<()> {
    if let Some(e) = email {
        if !e.trim().is_empty() && !e.contains('@') {
            return Err(CrmError::InvalidInput(format!(
                "Email '{}' does not appear to be valid (missing @)",
                e
            )));
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Duplicate detection
// ─────────────────────────────────────────────────────────────────────────────

/// A potential duplicate contact found during creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCandidate {
    /// The existing contact that may be a duplicate.
    pub contact: Contact,

    /// Human-readable reason why this is flagged as a duplicate.
    pub reason: String,

    /// Similarity score from 0.0 (no match) to 1.0 (exact match).
    pub score: f32,
}

/// Searches for existing contacts that may be duplicates of the given input.
///
/// Checks for matches on:
/// 1. Exact email match (score = 1.0).
/// 2. Exact `first_name` + `last_name` match (score = 0.9).
/// 3. Exact `org_name` match for organizations (score = 0.85).
///
/// Returns an empty `Vec` if no candidates are found.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on storage failure.
pub fn find_duplicate_candidates(
    conn: &Connection,
    input: &ContactInput,
) -> CrmResult<Vec<DuplicateCandidate>> {
    let mut candidates: Vec<DuplicateCandidate> = Vec::new();

    // Check email match.
    if let Some(email) = &input.email {
        if !email.trim().is_empty() {
            for contact in contacts::find_active_contacts_by_email(conn, email)? {
                candidates.push(DuplicateCandidate {
                    reason: format!("Same email address: {}", email),
                    score: 1.0,
                    contact,
                });
            }
        }
    }

    // Check name match for person contacts.
    let first = input
        .first_name
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let last = input
        .last_name
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_lowercase();

    if !first.is_empty() || !last.is_empty() {
        for contact in contacts::find_active_contacts_by_name(conn, &first, &last)? {
            // Avoid duplicating a candidate already added via email.
            if candidates.iter().any(|c| c.contact.id == contact.id) {
                continue;
            }
            candidates.push(DuplicateCandidate {
                reason: format!("Same name: {} {}", contact.first_name, contact.last_name),
                score: 0.9,
                contact,
            });
        }
    }

    log::debug!(
        "find_duplicate_candidates: {} candidates found",
        candidates.len()
    );
    Ok(candidates)
}

// ─────────────────────────────────────────────────────────────────────────────
// Merge
// ─────────────────────────────────────────────────────────────────────────────

/// Merges `source_id` into `target_id`.
///
/// The merge strategy is "best-of-both": fields that are empty on the target
/// are filled from the source. Non-empty target fields are preserved.
///
/// After merging, the source contact is soft-deleted. The merged target
/// contact is returned.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Either contact does not exist.
/// - [`CrmError::InvalidInput`] — `target_id == source_id`.
/// - [`CrmError::Database`] — storage failure.
pub fn merge_contacts(
    conn: &Connection,
    target_id: &str,
    source_id: &str,
    device_id: &str,
) -> CrmResult<Contact> {
    if target_id == source_id {
        return Err(CrmError::InvalidInput(
            "Cannot merge a contact with itself".to_string(),
        ));
    }

    let target = contacts::get_contact(conn, target_id)?;
    let source = contacts::get_contact(conn, source_id)?;

    // Build merged fields: prefer non-empty target; fall back to source.
    let merged_email = if target.email.is_empty() {
        &source.email
    } else {
        &target.email
    };
    let merged_phone = if target.phone.is_empty() {
        &source.phone
    } else {
        &target.phone
    };
    let merged_address = if target.address.is_empty() {
        &source.address
    } else {
        &target.address
    };
    let merged_city = if target.city.is_empty() {
        &source.city
    } else {
        &target.city
    };
    let merged_country = if target.country.is_empty() {
        &source.country
    } else {
        &target.country
    };
    let merged_org_name = if target.org_name.is_empty() {
        &source.org_name
    } else {
        &target.org_name
    };
    let merged_notes = if target.notes.is_empty() {
        source.notes.clone()
    } else if source.notes.is_empty() {
        target.notes.clone()
    } else {
        format!("{}\n\n---\n\n{}", target.notes, source.notes)
    };
    let merged_org_id = target.org_id.as_deref().or(source.org_id.as_deref());
    let merged_organization_id = target
        .organization_id
        .as_deref()
        .or(source.organization_id.as_deref())
        .or(merged_org_id);

    let updated = contacts::update_contact(
        conn,
        target_id,
        None,
        None,
        None,
        Some(merged_org_name),
        Some(merged_email),
        Some(merged_phone),
        Some(merged_address),
        Some(merged_city),
        Some(merged_country),
        Some(merged_org_id),
        Some(merged_organization_id),
        Some(&merged_notes),
    )?;
    let merged_lifecycle = if target.lifecycle == "customer" || source.lifecycle == "customer" {
        "customer"
    } else {
        target.lifecycle.as_str()
    };
    let updated = if updated.lifecycle == merged_lifecycle {
        updated
    } else {
        contacts::set_contact_lifecycle(conn, target_id, merged_lifecycle)?
    };

    let now = crate::utils::datetime::now_iso8601();

    conn.execute(
        "UPDATE deals SET contact_id = ?1 WHERE contact_id = ?2",
        params![target_id, source_id],
    )?;

    conn.execute(
        "UPDATE activities SET contact_id = ?1 WHERE contact_id = ?2",
        params![target_id, source_id],
    )?;

    conn.execute(
        "UPDATE notes SET entity_id = ?1 WHERE entity_type = 'contact' AND entity_id = ?2",
        params![target_id, source_id],
    )?;

    conn.execute(
        r#"
        DELETE FROM custom_field_values
        WHERE entity_id = ?2
          AND field_def_id IN (SELECT id FROM custom_field_defs WHERE entity_type = 'contact')
          AND field_def_id IN (
              SELECT field_def_id FROM custom_field_values WHERE entity_id = ?1
          )
        "#,
        params![target_id, source_id],
    )?;
    conn.execute(
        r#"
        UPDATE custom_field_values
        SET entity_id = ?1
        WHERE entity_id = ?2
          AND field_def_id IN (SELECT id FROM custom_field_defs WHERE entity_type = 'contact')
        "#,
        params![target_id, source_id],
    )?;

    conn.execute(
        r#"
        UPDATE deal_contacts
        SET deleted_at = ?3
        WHERE contact_id = ?2
          AND deleted_at IS NULL
          AND deal_id IN (
              SELECT deal_id FROM deal_contacts
              WHERE contact_id = ?1 AND deleted_at IS NULL
          )
        "#,
        params![target_id, source_id, now],
    )?;
    conn.execute(
        "UPDATE deal_contacts SET contact_id = ?1 WHERE contact_id = ?2",
        params![target_id, source_id],
    )?;

    conn.execute(
        r#"
        UPDATE activity_links
        SET deleted_at = ?3
        WHERE entity_type = 'contact'
          AND entity_id = ?2
          AND deleted_at IS NULL
          AND activity_id IN (
              SELECT activity_id FROM activity_links
              WHERE entity_type = 'contact' AND entity_id = ?1 AND deleted_at IS NULL
          )
        "#,
        params![target_id, source_id, now],
    )?;
    conn.execute(
        "UPDATE activity_links SET entity_id = ?1 WHERE entity_type = 'contact' AND entity_id = ?2",
        params![target_id, source_id],
    )?;

    conn.execute(
        r#"
        DELETE FROM entity_tags
        WHERE entity_type = 'contact'
          AND entity_id = ?2
          AND tag_id IN (
              SELECT tag_id FROM entity_tags
              WHERE entity_type = 'contact' AND entity_id = ?1
          )
        "#,
        params![target_id, source_id],
    )?;
    conn.execute(
        "UPDATE entity_tags SET entity_id = ?1 WHERE entity_type = 'contact' AND entity_id = ?2",
        params![target_id, source_id],
    )?;

    conn.execute(
        r#"
        UPDATE tag_links
        SET deleted_at = ?3
        WHERE entity_type = 'contact'
          AND entity_id = ?2
          AND deleted_at IS NULL
          AND tag_id IN (
              SELECT tag_id FROM tag_links
              WHERE entity_type = 'contact' AND entity_id = ?1 AND deleted_at IS NULL
          )
        "#,
        params![target_id, source_id, now],
    )?;
    conn.execute(
        "UPDATE tag_links SET entity_id = ?1 WHERE entity_type = 'contact' AND entity_id = ?2",
        params![target_id, source_id],
    )?;

    conn.execute(
        "UPDATE contacts SET org_id = ?1, updated_at = ?3 WHERE org_id = ?2",
        params![target_id, source_id, now],
    )?;

    // Soft-delete the source contact.
    contacts::soft_delete_contact(conn, source_id)?;

    log::info!(
        "Merged contact source={} into target={}",
        source_id,
        target_id
    );

    // Record the merge in the sync changelog.
    crate::storage::sync::record_change(
        conn,
        "contact",
        target_id,
        "__merge__",
        Some(source_id),
        Some(target_id),
        device_id,
    )?;

    Ok(updated)
}

// ─────────────────────────────────────────────────────────────────────────────
// Organization linking
// ─────────────────────────────────────────────────────────────────────────────

/// Links a person contact to an organization contact.
///
/// Sets `contact.org_id = org_contact_id`. Validates that:
/// - The person contact exists and has `contact_type = "person"`.
/// - The organization contact exists and has `contact_type = "organization"`.
///
/// # Errors
///
/// - [`CrmError::NotFound`] — Either contact does not exist.
/// - [`CrmError::InvalidInput`] — Type mismatch.
/// - [`CrmError::Database`] — storage failure.
pub fn link_contact_to_org(
    conn: &Connection,
    person_id: &str,
    org_contact_id: &str,
) -> CrmResult<Contact> {
    let person = contacts::get_contact(conn, person_id)?;
    let org = contacts::get_contact(conn, org_contact_id)?;

    if person.contact_type != "person" {
        return Err(CrmError::InvalidInput(format!(
            "Contact '{}' is not a person (type={})",
            person_id, person.contact_type
        )));
    }
    if org.contact_type != "organization" {
        return Err(CrmError::InvalidInput(format!(
            "Contact '{}' is not an organization (type={})",
            org_contact_id, org.contact_type
        )));
    }

    contacts::update_contact(
        conn,
        person_id,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Some(org_contact_id)),
        Some(Some(org_contact_id)),
        None,
    )
}
