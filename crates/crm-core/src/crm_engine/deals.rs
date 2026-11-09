//! Deal business logic — stage definitions, transitions, and win-rate.
//!
//! This module provides the domain layer for deals:
//!
//! - [`PIPELINE_STAGES`] — the ordered list of canonical stage names.
//! - [`validate_deal_input`] — validates a deal before create/update.
//! - [`calculate_win_rate`] — ratio of Closed Won to all closed deals.
//! - [`calculate_weighted_pipeline`] — probability-weighted total value.
//! - [`stage_transition_allowed`] — optional guard for stage movement rules.

use serde::{Deserialize, Serialize};

use crate::utils::datetime::parse_date_only;
use crate::utils::errors::{CrmError, CrmResult};

// ─────────────────────────────────────────────────────────────────────────────
// Stage constants
// ─────────────────────────────────────────────────────────────────────────────

/// The canonical pipeline stage names in order from earliest to latest.
///
/// These names are stored as strings in the `deals.stage` column. Custom
/// stages are allowed but won't receive default probability assignments.
pub const PIPELINE_STAGES: &[&str] = &[
    "Lead",
    "Qualified",
    "Proposal",
    "Negotiation",
    "Closed Won",
    "Closed Lost",
];

/// Returns `true` if `stage` is one of the canonical pipeline stage names.
///
/// Custom (non-canonical) stages are allowed by the storage layer but this
/// check can be used to warn users when entering non-standard names.
pub fn is_canonical_stage(stage: &str) -> bool {
    PIPELINE_STAGES.contains(&stage)
}

// ─────────────────────────────────────────────────────────────────────────────
// Deal input and validation
// ─────────────────────────────────────────────────────────────────────────────

/// Input payload for creating or updating a deal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealInput {
    /// Deal title. Required on create.
    pub title: Option<String>,

    /// Monetary value. Must be >= 0.
    pub value: Option<f64>,

    /// ISO 4217 currency code.
    pub currency: Option<String>,

    /// Pipeline stage name.
    pub stage: Option<String>,

    /// Probability of closing (0–100).
    pub probability: Option<i32>,

    /// Expected close date.
    pub expected_close: Option<String>,

    /// Associated contact UUID.
    pub contact_id: Option<String>,

    /// Freeform notes.
    pub notes: Option<String>,
}

/// Validates a [`DealInput`] for creation.
///
/// # Rules
///
/// - `title` must be non-empty.
/// - `value`, if provided, must be >= 0.
/// - `probability`, if provided, must be in [0, 100].
///
/// # Errors
///
/// Returns [`CrmError::InvalidInput`] on the first validation failure.
pub fn validate_deal_for_create(input: &DealInput) -> CrmResult<()> {
    let title = input.title.as_deref().unwrap_or("").trim();
    if title.is_empty() {
        return Err(CrmError::InvalidInput("Deal title is required".to_string()));
    }

    if let Some(v) = input.value {
        if v < 0.0 {
            return Err(CrmError::InvalidInput(format!(
                "Deal value must be >= 0, got {}",
                v
            )));
        }
    }

    if let Some(p) = input.probability {
        if !(0..=100).contains(&p) {
            return Err(CrmError::InvalidInput(format!(
                "Probability must be between 0 and 100, got {}",
                p
            )));
        }
    }

    if let Some(currency) = &input.currency {
        if !currency.is_empty() {
            let is_iso4217 =
                currency.len() == 3 && currency.chars().all(|c| c.is_ascii_uppercase());
            if !is_iso4217 {
                return Err(CrmError::InvalidInput(format!(
                    "Currency must be a 3-letter ISO 4217 code (e.g. USD), got '{}'",
                    currency
                )));
            }
        }
    }

    if let Some(expected_close) = &input.expected_close {
        if !expected_close.is_empty() && parse_date_only(expected_close).is_err() {
            return Err(CrmError::InvalidInput(
                "expected_close must be a valid date (YYYY-MM-DD)".to_string(),
            ));
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Stage transitions
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `true` if moving from `from_stage` to `to_stage` is allowed.
///
/// Current rules:
/// - Any stage → any non-closed stage is allowed (move backward or forward).
/// - `"Closed Won"` → `"Closed Lost"` is allowed (reverse a close decision).
/// - Any stage → `"Closed Won"` or `"Closed Lost"` is allowed.
/// - `"Closed Won"` or `"Closed Lost"` → any active stage is allowed (reopen).
///
/// In practice all transitions are currently allowed. This function exists
/// as a hook for future rule enforcement.
pub fn stage_transition_allowed(_from_stage: &str, _to_stage: &str) -> bool {
    // All transitions allowed in v1.
    true
}

// ─────────────────────────────────────────────────────────────────────────────
// Pipeline metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Calculates the win rate from a pipeline summary.
///
/// Win rate = `closed_won_count / (closed_won_count + closed_lost_count)`.
///
/// Returns `0.0` if there are no closed deals.
///
/// # Parameters
///
/// - `summaries` — slice of [`crate::storage::deals::PipelineSummary`].
///
/// # Example
///
/// ```rust,ignore
/// let win_rate = calculate_win_rate(&summaries);
/// // 0.75 means 75% win rate
/// ```
pub fn calculate_win_rate(summaries: &[crate::storage::deals::PipelineSummary]) -> f64 {
    let won = summaries
        .iter()
        .find(|s| s.stage == "Closed Won")
        .map(|s| s.count)
        .unwrap_or(0);

    let lost = summaries
        .iter()
        .find(|s| s.stage == "Closed Lost")
        .map(|s| s.count)
        .unwrap_or(0);

    let total = won + lost;
    if total == 0 {
        0.0
    } else {
        won as f64 / total as f64
    }
}

/// Calculates the total probability-weighted pipeline value.
///
/// Sums `PipelineSummary.weighted_value` across all stages, excluding
/// `"Closed Lost"` (which has 0 weighted value by definition).
///
/// # Parameters
///
/// - `summaries` — slice of [`crate::storage::deals::PipelineSummary`].
pub fn calculate_weighted_pipeline(summaries: &[crate::storage::deals::PipelineSummary]) -> f64 {
    summaries
        .iter()
        .filter(|s| s.stage != "Closed Lost")
        .map(|s| s.weighted_value)
        .sum()
}

/// Calculates the total (unweighted) active pipeline value.
///
/// Sums `PipelineSummary.total_value` across all stages except `"Closed Lost"`.
pub fn calculate_total_pipeline_value(summaries: &[crate::storage::deals::PipelineSummary]) -> f64 {
    summaries
        .iter()
        .filter(|s| s.stage != "Closed Lost")
        .map(|s| s.total_value)
        .sum()
}
