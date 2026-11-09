import type { ActivityFunnelReport, PipelineConversionReport } from '$lib/api/reports';
import type { StaleDealReport } from '$lib/utils/staleDealReport';

export type ReportFocus = '' | 'pipeline' | 'activity' | 'stale';

export interface ReportSnapshotRow {
  section: string;
  name: string;
  field: string;
  value: string;
}

export const REPORT_SNAPSHOT_HONESTY_NOTE =
  'Current dataset snapshot. These ratios are not historical stage conversion.';

export function defaultReportSnapshotFilename(now = new Date()): string {
  return `900crm-reports-${now.toISOString().slice(0, 10)}.csv`;
}

function includeSection(focus: ReportFocus, section: 'pipeline' | 'activity' | 'stale'): boolean {
  return focus === '' || focus === section;
}

function ratioValue(value: number): string {
  if (!Number.isFinite(value)) {
    return '';
  }
  return String(value);
}

function countValue(value: number): string {
  if (!Number.isFinite(value)) {
    return '';
  }
  return String(value);
}

export function buildReportSnapshotRows({
  focus,
  pipeline,
  activity,
  stale,
}: {
  focus: ReportFocus;
  pipeline: PipelineConversionReport | null;
  activity: ActivityFunnelReport | null;
  stale: StaleDealReport | null;
}): ReportSnapshotRow[] {
  const rows: ReportSnapshotRow[] = [
    { section: 'meta', name: 'snapshot', field: 'note', value: REPORT_SNAPSHOT_HONESTY_NOTE },
    { section: 'meta', name: 'snapshot', field: 'focus', value: focus || 'all' },
  ];

  if (includeSection(focus, 'pipeline') && pipeline) {
    rows.push(
      { section: 'pipeline', name: 'summary', field: 'generated_at', value: pipeline.generated_at },
      { section: 'pipeline', name: 'summary', field: 'total_deals', value: countValue(pipeline.total_deals) },
      { section: 'pipeline', name: 'summary', field: 'open_deals', value: countValue(pipeline.open_deals) },
      { section: 'pipeline', name: 'summary', field: 'closed_won', value: countValue(pipeline.closed_won) },
      { section: 'pipeline', name: 'summary', field: 'closed_lost', value: countValue(pipeline.closed_lost) },
      { section: 'pipeline', name: 'summary', field: 'win_rate', value: ratioValue(pipeline.overall_win_rate) },
    );

    for (const metric of pipeline.stage_metrics) {
      rows.push(
        { section: 'pipeline_stage', name: metric.stage, field: 'count', value: countValue(metric.count) },
        { section: 'pipeline_stage', name: metric.stage, field: 'stage_share', value: ratioValue(metric.stage_share) },
      );
    }

    for (const metric of pipeline.transition_metrics) {
      const name = `${metric.from_stage}->${metric.to_stage}`;
      rows.push(
        { section: 'pipeline_funnel', name, field: 'current_stage_ratio', value: ratioValue(metric.ratio) },
        { section: 'pipeline_funnel', name, field: 'from_count', value: countValue(metric.from_count) },
        { section: 'pipeline_funnel', name, field: 'to_count', value: countValue(metric.to_count) },
      );
    }
  }

  if (includeSection(focus, 'activity') && activity) {
    rows.push(
      { section: 'activity', name: 'summary', field: 'generated_at', value: activity.generated_at },
      { section: 'activity', name: 'summary', field: 'total_activities', value: countValue(activity.total_activities) },
      { section: 'activity', name: 'summary', field: 'completed_activities', value: countValue(activity.completed_activities) },
      { section: 'activity', name: 'summary', field: 'pending_activities', value: countValue(activity.pending_activities) },
      { section: 'activity', name: 'summary', field: 'overdue_activities', value: countValue(activity.overdue_activities) },
      { section: 'activity', name: 'summary', field: 'completion_rate', value: ratioValue(activity.completion_rate) },
      { section: 'activity', name: 'summary', field: 'overdue_rate', value: ratioValue(activity.overdue_rate) },
      { section: 'activity_due', name: 'overdue', field: 'count', value: countValue(activity.due_buckets.overdue) },
      { section: 'activity_due', name: 'due_today', field: 'count', value: countValue(activity.due_buckets.due_today) },
      { section: 'activity_due', name: 'due_next_7_days', field: 'count', value: countValue(activity.due_buckets.due_next_7_days) },
      { section: 'activity_due', name: 'due_later', field: 'count', value: countValue(activity.due_buckets.due_later) },
      { section: 'activity_due', name: 'no_due_date', field: 'count', value: countValue(activity.due_buckets.no_due_date) },
    );

    for (const metric of activity.by_type) {
      rows.push(
        { section: 'activity_type', name: metric.activity_type, field: 'total', value: countValue(metric.total) },
        { section: 'activity_type', name: metric.activity_type, field: 'completed', value: countValue(metric.completed) },
        { section: 'activity_type', name: metric.activity_type, field: 'completion_rate', value: ratioValue(metric.completion_rate) },
      );
    }
  }

  if (includeSection(focus, 'stale') && stale) {
    rows.push(
      { section: 'stale', name: 'summary', field: 'count', value: countValue(stale.count) },
      { section: 'stale', name: 'summary', field: 'stale_days', value: countValue(stale.staleDays) },
    );

    for (const row of stale.rows) {
      rows.push(
        { section: 'stale_deal', name: row.name, field: 'stage', value: row.stage },
        { section: 'stale_deal', name: row.name, field: 'owner', value: row.owner ?? '' },
        { section: 'stale_deal', name: row.name, field: 'quiet_days', value: countValue(row.stageAgeDays) },
        { section: 'stale_deal', name: row.name, field: 'next_step', value: row.nextActivitySubject ?? '' },
      );
    }
  }

  return rows;
}

function csvCell(value: string): string {
  if (/[",\n\r]/.test(value)) {
    return `"${value.replaceAll('"', '""')}"`;
  }
  return value;
}

export function reportSnapshotToCsv(rows: ReportSnapshotRow[]): string {
  const lines = ['section,name,field,value'];
  for (const row of rows) {
    lines.push([row.section, row.name, row.field, row.value].map(csvCell).join(','));
  }
  return `${lines.join('\n')}\n`;
}
