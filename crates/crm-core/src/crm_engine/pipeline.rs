//! Pipeline configuration and metrics for 900CRM.
//!
//! This module defines the canonical pipeline stage configuration and provides
//! higher-level metrics calculations that aggregate across all stages.
//!
//! # Default Stages
//!
//! | Stage         | Default Probability | Is Closed? |
//! |---------------|---------------------|------------|
//! | Lead          | 10%                 | No         |
//! | Qualified     | 25%                 | No         |
//! | Proposal      | 50%                 | No         |
//! | Negotiation   | 75%                 | No         |
//! | Closed Won    | 100%                | Yes (won)  |
//! | Closed Lost   | 0%                  | Yes (lost) |

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::utils::errors::CrmResult;

// ─────────────────────────────────────────────────────────────────────────────
// Stage definition
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for a single pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDefinition {
    /// Unique stage name displayed in the UI (e.g. `"Proposal"`).
    pub name: String,

    /// Default win probability (0–100) assigned when a deal enters this stage.
    pub default_probability: i32,

    /// Whether this stage represents a closed deal (won or lost).
    pub is_closed: bool,

    /// Whether this is a won close (`true`) or lost close (`false`).
    /// `None` for non-closed stages.
    pub is_won: Option<bool>,

    /// CSS color for the stage indicator (e.g. `"#6366f1"`).
    pub color: String,
}

/// Returns the default list of pipeline stages in order.
///
/// This is the canonical 6-stage CRM pipeline used by 900CRM.
/// The returned `Vec` is stored on [`crate::crm_engine::CrmEngine`] at
/// startup.
pub fn default_stages() -> Vec<StageDefinition> {
    vec![
        StageDefinition {
            name: "Lead".to_string(),
            default_probability: 10,
            is_closed: false,
            is_won: None,
            color: "#94a3b8".to_string(), // slate-400
        },
        StageDefinition {
            name: "Qualified".to_string(),
            default_probability: 25,
            is_closed: false,
            is_won: None,
            color: "#60a5fa".to_string(), // blue-400
        },
        StageDefinition {
            name: "Proposal".to_string(),
            default_probability: 50,
            is_closed: false,
            is_won: None,
            color: "#a78bfa".to_string(), // violet-400
        },
        StageDefinition {
            name: "Negotiation".to_string(),
            default_probability: 75,
            is_closed: false,
            is_won: None,
            color: "#fb923c".to_string(), // orange-400
        },
        StageDefinition {
            name: "Closed Won".to_string(),
            default_probability: 100,
            is_closed: true,
            is_won: Some(true),
            color: "#4ade80".to_string(), // green-400
        },
        StageDefinition {
            name: "Closed Lost".to_string(),
            default_probability: 0,
            is_closed: true,
            is_won: Some(false),
            color: "#f87171".to_string(), // red-400
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Pipeline metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Full pipeline metrics returned to the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetrics {
    /// Total number of active (non-closed, non-deleted) deals.
    pub active_deal_count: i64,

    /// Sum of `value` across all active deals.
    pub total_pipeline_value: f64,

    /// Probability-weighted sum of deal values for active deals.
    pub weighted_pipeline_value: f64,

    /// Win rate: closed_won / (closed_won + closed_lost). Between 0.0 and 1.0.
    pub win_rate: f64,

    /// Average age of active deals in days.
    pub average_deal_age_days: f64,

    /// Per-stage summaries.
    pub stages: Vec<crate::storage::deals::PipelineSummary>,
}

/// Computes full [`PipelineMetrics`] from the database.
///
/// # Errors
///
/// Returns [`CrmError::Database`] on storage failure.
pub fn get_pipeline_metrics(conn: &Connection) -> CrmResult<PipelineMetrics> {
    let stages = crate::storage::deals::get_pipeline_summary(conn)?;

    let total_pipeline_value = crate::crm_engine::deals::calculate_total_pipeline_value(&stages);
    let weighted_pipeline_value = crate::crm_engine::deals::calculate_weighted_pipeline(&stages);
    let win_rate = crate::crm_engine::deals::calculate_win_rate(&stages);

    let active_deal_count: i64 = stages
        .iter()
        .filter(|s| s.stage != "Closed Won" && s.stage != "Closed Lost")
        .map(|s| s.count)
        .sum();

    let average_deal_age_days = crate::storage::deals::get_average_active_deal_age_days(conn)?;

    log::debug!(
        "Pipeline metrics: active={} total_value={:.2} win_rate={:.2}",
        active_deal_count,
        total_pipeline_value,
        win_rate
    );

    Ok(PipelineMetrics {
        active_deal_count,
        total_pipeline_value,
        weighted_pipeline_value,
        win_rate,
        average_deal_age_days,
        stages,
    })
}
