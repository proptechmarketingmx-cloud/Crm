/**
 * src/lib/api/reports.ts — Tauri IPC wrappers for reporting metrics.
 */

import { invoke } from '@tauri-apps/api/core';

export interface PipelineStageMetric {
  stage: string;
  count: number;
  total_value: number;
  weighted_value: number;
  stage_share: number;
}

export interface StageTransitionMetric {
  from_stage: string;
  to_stage: string;
  from_count: number;
  to_count: number;
  ratio: number;
}

export interface PipelineConversionReport {
  generated_at: string;
  total_deals: number;
  open_deals: number;
  closed_won: number;
  closed_lost: number;
  overall_win_rate: number;
  stage_metrics: PipelineStageMetric[];
  transition_metrics: StageTransitionMetric[];
}

export interface ActivityTypeMetric {
  activity_type: string;
  total: number;
  completed: number;
  pending: number;
  overdue: number;
  completion_rate: number;
}

export interface ActivityDueBuckets {
  overdue: number;
  due_today: number;
  due_next_7_days: number;
  due_later: number;
  no_due_date: number;
}

export interface ActivityFunnelReport {
  generated_at: string;
  total_activities: number;
  completed_activities: number;
  pending_activities: number;
  overdue_activities: number;
  completion_rate: number;
  overdue_rate: number;
  by_type: ActivityTypeMetric[];
  due_buckets: ActivityDueBuckets;
}

export async function getPipelineConversionReport(): Promise<PipelineConversionReport> {
  return invoke<PipelineConversionReport>('get_pipeline_conversion_report');
}

export async function getActivityFunnelReport(): Promise<ActivityFunnelReport> {
  return invoke<ActivityFunnelReport>('get_activity_funnel_report');
}
