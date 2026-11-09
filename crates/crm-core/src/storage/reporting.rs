//! Reporting metrics queries for pipeline conversion and activity funnels.
//!
//! This module exposes lightweight aggregate queries used by analytics/reporting
//! endpoints. Queries are read-only and designed to stay efficient on older
//! hardware by relying on indexed columns and grouped scans.

use std::collections::HashMap;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::storage::deals::{self, PipelineSummary};
use crate::utils::{
    datetime::{local_days_from_now_ymd, local_today_ymd, now_iso8601},
    errors::CrmResult,
};

const CANONICAL_STAGES: [&str; 6] = [
    "Lead",
    "Qualified",
    "Proposal",
    "Negotiation",
    "Closed Won",
    "Closed Lost",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStageMetric {
    pub stage: String,
    pub count: i64,
    pub total_value: f64,
    pub weighted_value: f64,
    pub stage_share: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTransitionMetric {
    pub from_stage: String,
    pub to_stage: String,
    pub from_count: i64,
    pub to_count: i64,
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConversionReport {
    pub generated_at: String,
    pub total_deals: i64,
    pub open_deals: i64,
    pub closed_won: i64,
    pub closed_lost: i64,
    pub overall_win_rate: f64,
    pub stage_metrics: Vec<PipelineStageMetric>,
    pub transition_metrics: Vec<StageTransitionMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityTypeMetric {
    pub activity_type: String,
    pub total: i64,
    pub completed: i64,
    pub pending: i64,
    pub overdue: i64,
    pub completion_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDueBuckets {
    pub overdue: i64,
    pub due_today: i64,
    pub due_next_7_days: i64,
    pub due_later: i64,
    pub no_due_date: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityFunnelReport {
    pub generated_at: String,
    pub total_activities: i64,
    pub completed_activities: i64,
    pub pending_activities: i64,
    pub overdue_activities: i64,
    pub completion_rate: f64,
    pub overdue_rate: f64,
    pub by_type: Vec<ActivityTypeMetric>,
    pub due_buckets: ActivityDueBuckets,
}

pub fn get_pipeline_conversion_report(conn: &Connection) -> CrmResult<PipelineConversionReport> {
    let summaries = deals::get_pipeline_summary(conn)?;

    let mut summary_by_stage: HashMap<String, PipelineSummary> = summaries
        .into_iter()
        .map(|summary| (summary.stage.clone(), summary))
        .collect();

    let mut stage_metrics: Vec<PipelineStageMetric> = CANONICAL_STAGES
        .iter()
        .map(|stage| {
            summary_by_stage.remove(*stage).unwrap_or(PipelineSummary {
                stage: (*stage).to_string(),
                count: 0,
                total_value: 0.0,
                weighted_value: 0.0,
            })
        })
        .map(|summary| PipelineStageMetric {
            stage: summary.stage,
            count: summary.count,
            total_value: summary.total_value,
            weighted_value: summary.weighted_value,
            stage_share: 0.0,
        })
        .collect();

    let mut extra_stage_metrics: Vec<PipelineStageMetric> = summary_by_stage
        .into_values()
        .map(|summary| PipelineStageMetric {
            stage: summary.stage,
            count: summary.count,
            total_value: summary.total_value,
            weighted_value: summary.weighted_value,
            stage_share: 0.0,
        })
        .collect();

    extra_stage_metrics.sort_by(|a, b| a.stage.cmp(&b.stage));
    stage_metrics.extend(extra_stage_metrics);

    let total_deals: i64 = stage_metrics.iter().map(|metric| metric.count).sum();
    for metric in &mut stage_metrics {
        metric.stage_share = if total_deals > 0 {
            metric.count as f64 / total_deals as f64
        } else {
            0.0
        };
    }

    let count_for = |stage_name: &str| -> i64 {
        stage_metrics
            .iter()
            .find(|metric| metric.stage == stage_name)
            .map(|metric| metric.count)
            .unwrap_or(0)
    };

    let closed_won = count_for("Closed Won");
    let closed_lost = count_for("Closed Lost");
    let open_deals = (total_deals - closed_won - closed_lost).max(0);

    let overall_win_rate = if closed_won + closed_lost > 0 {
        closed_won as f64 / (closed_won + closed_lost) as f64
    } else {
        0.0
    };

    let transition_pairs = [
        ("Lead", "Qualified"),
        ("Qualified", "Proposal"),
        ("Proposal", "Negotiation"),
        ("Negotiation", "Closed Won"),
        ("Negotiation", "Closed Lost"),
    ];

    let transition_metrics = transition_pairs
        .iter()
        .map(|(from_stage, to_stage)| {
            let from_count = count_for(from_stage);
            let to_count = count_for(to_stage);
            let ratio = if from_count > 0 {
                to_count as f64 / from_count as f64
            } else {
                0.0
            };

            StageTransitionMetric {
                from_stage: (*from_stage).to_string(),
                to_stage: (*to_stage).to_string(),
                from_count,
                to_count,
                ratio,
            }
        })
        .collect();

    Ok(PipelineConversionReport {
        generated_at: now_iso8601(),
        total_deals,
        open_deals,
        closed_won,
        closed_lost,
        overall_win_rate,
        stage_metrics,
        transition_metrics,
    })
}

pub fn get_activity_funnel_report(conn: &Connection) -> CrmResult<ActivityFunnelReport> {
    let now = now_iso8601();
    let local_today = local_today_ymd();
    let local_day_7 = local_days_from_now_ymd(7);
    let utc_today_prefix = format!("{}%", &now[..10.min(now.len())]);

    let row = conn.query_row(
        r#"
        SELECT
            COUNT(*) AS total,
            SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) AS completed,
            SUM(CASE WHEN completed = 0 AND due_date IS NOT NULL AND ((length(due_date) = 10 AND due_date < ?3) OR (length(due_date) > 10 AND due_date < ?1)) THEN 1 ELSE 0 END) AS overdue,
            SUM(CASE WHEN completed = 0 AND ((length(due_date) = 10 AND due_date = ?3) OR (length(due_date) > 10 AND due_date LIKE ?2)) THEN 1 ELSE 0 END) AS due_today,
            SUM(CASE WHEN completed = 0 AND due_date IS NOT NULL AND substr(due_date, 1, 10) > ?3 AND substr(due_date, 1, 10) <= ?4 THEN 1 ELSE 0 END) AS due_next_7_days,
            SUM(CASE WHEN completed = 0 AND due_date IS NOT NULL AND substr(due_date, 1, 10) > ?4 THEN 1 ELSE 0 END) AS due_later,
            SUM(CASE WHEN completed = 0 AND due_date IS NULL THEN 1 ELSE 0 END) AS no_due_date
        FROM activities
        WHERE deleted_at IS NULL
        "#,
        params![now, utc_today_prefix, local_today, local_day_7],
        |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                row.get::<_, Option<i64>>(6)?.unwrap_or(0),
            ))
        },
    )?;

    let total_activities = row.0;
    let completed_activities = row.1;
    let overdue_activities = row.2;
    let pending_activities = (total_activities - completed_activities - overdue_activities).max(0);

    let completion_rate = if total_activities > 0 {
        completed_activities as f64 / total_activities as f64
    } else {
        0.0
    };

    let overdue_rate = if total_activities > 0 {
        overdue_activities as f64 / total_activities as f64
    } else {
        0.0
    };

    let due_buckets = ActivityDueBuckets {
        overdue: row.2,
        due_today: row.3,
        due_next_7_days: row.4,
        due_later: row.5,
        no_due_date: row.6,
    };

    let mut stmt = conn.prepare(
        r#"
        SELECT
            activity_type,
            COUNT(*) AS total,
            SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) AS completed,
            SUM(CASE WHEN completed = 0 AND due_date IS NOT NULL AND ((length(due_date) = 10 AND due_date < ?2) OR (length(due_date) > 10 AND due_date < ?1)) THEN 1 ELSE 0 END) AS overdue
        FROM activities
        WHERE deleted_at IS NULL
        GROUP BY activity_type
        ORDER BY total DESC, activity_type ASC
        "#,
    )?;

    let rows = stmt.query_map(params![now, local_today], |row| {
        let total = row.get::<_, Option<i64>>(1)?.unwrap_or(0);
        let completed = row.get::<_, Option<i64>>(2)?.unwrap_or(0);
        let overdue = row.get::<_, Option<i64>>(3)?.unwrap_or(0);
        let pending = (total - completed - overdue).max(0);

        Ok(ActivityTypeMetric {
            activity_type: row.get(0)?,
            total,
            completed,
            pending,
            overdue,
            completion_rate: if total > 0 {
                completed as f64 / total as f64
            } else {
                0.0
            },
        })
    })?;

    let by_type = rows.filter_map(|row| row.ok()).collect();

    Ok(ActivityFunnelReport {
        generated_at: now_iso8601(),
        total_activities,
        completed_activities,
        pending_activities,
        overdue_activities,
        completion_rate,
        overdue_rate,
        by_type,
        due_buckets,
    })
}
