import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { getActivityFunnelReport, getPipelineConversionReport } from './reports';

describe('reports API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('requests the pipeline conversion report through the expected command', async () => {
    const report = {
      generated_at: '2026-07-08T00:00:00Z',
      total_deals: 2,
      open_deals: 1,
      closed_won: 1,
      closed_lost: 0,
      overall_win_rate: 1,
      stage_metrics: [],
      transition_metrics: [],
    };
    invokeMock.mockResolvedValueOnce(report);

    await expect(getPipelineConversionReport()).resolves.toEqual(report);
    expect(invokeMock).toHaveBeenCalledWith('get_pipeline_conversion_report');
  });

  it('requests the activity funnel report through the expected command', async () => {
    const report = {
      generated_at: '2026-07-08T00:00:00Z',
      total_activities: 3,
      completed_activities: 1,
      pending_activities: 2,
      overdue_activities: 1,
      completion_rate: 1 / 3,
      overdue_rate: 1 / 3,
      by_type: [],
      due_buckets: {
        overdue: 1,
        due_today: 0,
        due_next_7_days: 1,
        due_later: 1,
        no_due_date: 0,
      },
    };
    invokeMock.mockResolvedValueOnce(report);

    await expect(getActivityFunnelReport()).resolves.toEqual(report);
    expect(invokeMock).toHaveBeenCalledWith('get_activity_funnel_report');
  });
});
