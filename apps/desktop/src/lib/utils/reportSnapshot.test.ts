import { describe, expect, it } from 'vitest';
import type { ActivityFunnelReport, PipelineConversionReport } from '$lib/api/reports';
import type { StaleDealReport } from '$lib/utils/staleDealReport';
import {
  REPORT_SNAPSHOT_HONESTY_NOTE,
  buildReportSnapshotRows,
  defaultReportSnapshotFilename,
  reportSnapshotToCsv,
} from './reportSnapshot';

const pipeline: PipelineConversionReport = {
  generated_at: '2026-08-17T12:00:00Z',
  total_deals: 4,
  open_deals: 2,
  closed_won: 1,
  closed_lost: 1,
  overall_win_rate: 0.5,
  stage_metrics: [
    { stage: 'Lead', count: 2, total_value: 0, weighted_value: 0, stage_share: 0.5 },
  ],
  transition_metrics: [
    { from_stage: 'Lead', to_stage: 'Qualified', from_count: 2, to_count: 1, ratio: 0.5 },
  ],
};

const activity: ActivityFunnelReport = {
  generated_at: '2026-08-17T12:00:00Z',
  total_activities: 5,
  completed_activities: 3,
  pending_activities: 2,
  overdue_activities: 1,
  completion_rate: 0.6,
  overdue_rate: 0.2,
  by_type: [
    { activity_type: 'task', total: 3, completed: 2, pending: 1, overdue: 1, completion_rate: 2 / 3 },
  ],
  due_buckets: {
    overdue: 1,
    due_today: 1,
    due_next_7_days: 1,
    due_later: 1,
    no_due_date: 1,
  },
};

const stale: StaleDealReport = {
  count: 1,
  staleDays: 14,
  rows: [
    {
      dealId: 'deal-1',
      name: 'Quiet Clinic Rollout',
      stage: 'proposal',
      owner: 'Samira',
      stageAgeDays: 21,
      nextActivitySubject: 'Site visit',
      href: '/deals/deal-1',
    },
  ],
};

describe('report snapshot', () => {
  it('names a dated local csv file', () => {
    expect(defaultReportSnapshotFilename(new Date('2026-08-17T15:04:00Z'))).toBe(
      '900crm-reports-2026-08-17.csv',
    );
  });

  it('exports the visible current-dataset numbers and an honesty note', () => {
    const rows = buildReportSnapshotRows({
      focus: '',
      pipeline,
      activity,
      stale,
    });

    expect(rows).toContainEqual({
      section: 'meta',
      name: 'snapshot',
      field: 'note',
      value: REPORT_SNAPSHOT_HONESTY_NOTE,
    });
    expect(rows).toContainEqual({
      section: 'pipeline',
      name: 'summary',
      field: 'win_rate',
      value: '0.5',
    });
    expect(rows).toContainEqual({
      section: 'pipeline_funnel',
      name: 'Lead->Qualified',
      field: 'current_stage_ratio',
      value: '0.5',
    });
    expect(rows).toContainEqual({
      section: 'stale_deal',
      name: 'Quiet Clinic Rollout',
      field: 'quiet_days',
      value: '21',
    });
  });

  it('keeps the snapshot limited to the current report focus', () => {
    const rows = buildReportSnapshotRows({
      focus: 'stale',
      pipeline,
      activity,
      stale,
    });

    expect(rows.some((row) => row.section.startsWith('pipeline'))).toBe(false);
    expect(rows.some((row) => row.section.startsWith('activity'))).toBe(false);
    expect(rows).toContainEqual({
      section: 'stale',
      name: 'summary',
      field: 'count',
      value: '1',
    });
  });

  it('quotes commas in the csv snapshot', () => {
    const csv = reportSnapshotToCsv([
      {
        section: 'meta',
        name: 'snapshot',
        field: 'note',
        value: 'Current dataset, not historical conversion',
      },
    ]);

    expect(csv.startsWith('section,name,field,value\n')).toBe(true);
    expect(csv).toContain('"Current dataset, not historical conversion"');
  });
});
