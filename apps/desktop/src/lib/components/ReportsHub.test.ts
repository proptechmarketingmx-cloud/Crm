// @vitest-environment jsdom

import { render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const {
  getActivityFunnelReportMock,
  getPipelineConversionReportMock,
  listActivitiesMock,
  listDealsMock,
  loadActivityLinkIndexMock,
} = vi.hoisted(() => ({
  getActivityFunnelReportMock: vi.fn(),
  getPipelineConversionReportMock: vi.fn(),
  listActivitiesMock: vi.fn(),
  listDealsMock: vi.fn(),
  loadActivityLinkIndexMock: vi.fn(),
}));

vi.mock('$lib/i18n', () => ({
  t: (key: string, params?: Record<string, string | number>) => {
    if (key === 'reports.generatedAt') return `Generated ${params?.value}`;
    return key;
  },
}));

vi.mock('$lib/api/reports', () => ({
  getActivityFunnelReport: getActivityFunnelReportMock,
  getPipelineConversionReport: getPipelineConversionReportMock,
}));

vi.mock('$lib/api/activities', () => ({
  listActivities: listActivitiesMock,
}));

vi.mock('$lib/api/deals', () => ({
  listDeals: listDealsMock,
}));

vi.mock('$lib/utils/activityRelationships', () => ({
  loadActivityLinkIndex: loadActivityLinkIndexMock,
}));

vi.mock('$lib/stores/settings', () => ({
  settingsStore: {
    language: 'en-US',
  },
}));

vi.mock('$lib/api/savedViews', () => ({
  listSavedViews: vi.fn().mockResolvedValue([]),
  createSavedView: vi.fn(),
  deleteSavedView: vi.fn(),
  filtersMatch: () => true,
}));

import Reports from '../../routes/Reports.svelte';

describe('Reports route', () => {
  beforeEach(() => {
    getActivityFunnelReportMock.mockReset();
    getPipelineConversionReportMock.mockReset();
    listActivitiesMock.mockReset();
    listDealsMock.mockReset();
    loadActivityLinkIndexMock.mockReset();
    listActivitiesMock.mockResolvedValue([]);
    listDealsMock.mockResolvedValue([]);
    loadActivityLinkIndexMock.mockResolvedValue({});
  });

  it('renders current pipeline and activity report data', async () => {
    getPipelineConversionReportMock.mockResolvedValueOnce({
      generated_at: '2026-07-08T12:00:00Z',
      total_deals: 4,
      open_deals: 2,
      closed_won: 1,
      closed_lost: 1,
      overall_win_rate: 0.5,
      stage_metrics: [
        { stage: 'Lead', count: 2, total_value: 0, weighted_value: 0, stage_share: 0.5 },
        { stage: 'Proposal', count: 1, total_value: 0, weighted_value: 0, stage_share: 0.25 },
      ],
      transition_metrics: [
        { from_stage: 'Lead', to_stage: 'Qualified', from_count: 2, to_count: 1, ratio: 0.5 },
      ],
    });
    getActivityFunnelReportMock.mockResolvedValueOnce({
      generated_at: '2026-07-08T12:00:00Z',
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
    });

    render(Reports);

    await waitFor(() => {
      expect(getPipelineConversionReportMock).toHaveBeenCalledOnce();
      expect(getActivityFunnelReportMock).toHaveBeenCalledOnce();
    });

    expect((await screen.findAllByText('50.0%')).length).toBeGreaterThan(0);
    expect(screen.getByText('reports.pipeline.title')).toBeTruthy();
    expect(screen.getByText('reports.activity.title')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'reports.exportSnapshot' })).toBeTruthy();
    expect(screen.getByText('deals.stages.lead')).toBeTruthy();
    expect(screen.getByText('deals.stages.lead -> deals.stages.qualified')).toBeTruthy();
    expect(screen.getByText('reports.activity.dueBuckets.title')).toBeTruthy();
    expect(screen.getByText('66.7% / 2/3')).toBeTruthy();
  });

  it('tells a first-run user how to fill empty reports', async () => {
    getPipelineConversionReportMock.mockResolvedValueOnce({
      generated_at: '2026-08-15T12:00:00Z',
      total_deals: 0,
      open_deals: 0,
      closed_won: 0,
      closed_lost: 0,
      overall_win_rate: 0,
      stage_metrics: [],
      transition_metrics: [],
    });
    getActivityFunnelReportMock.mockResolvedValueOnce({
      generated_at: '2026-08-15T12:00:00Z',
      total_activities: 0,
      completed_activities: 0,
      pending_activities: 0,
      overdue_activities: 0,
      completion_rate: 0,
      overdue_rate: 0,
      by_type: [],
      due_buckets: {
        overdue: 0,
        due_today: 0,
        due_next_7_days: 0,
        due_later: 0,
        no_due_date: 0,
      },
    });

    render(Reports);

    expect(await screen.findByText('reports.emptyWorkspaceTitle')).toBeTruthy();
    expect(screen.getByText('reports.emptyWorkspaceDesc')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'reports.emptyWorkspaceAction' })).toBeTruthy();
  });

  it('keeps activity data visible when the pipeline report fails', async () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
    getPipelineConversionReportMock.mockRejectedValueOnce(new Error('pipeline unavailable'));
    getActivityFunnelReportMock.mockResolvedValueOnce({
      generated_at: '2026-07-08T12:00:00Z',
      total_activities: 1,
      completed_activities: 1,
      pending_activities: 0,
      overdue_activities: 0,
      completion_rate: 1,
      overdue_rate: 0,
      by_type: [
        { activity_type: 'call', total: 1, completed: 1, pending: 0, overdue: 0, completion_rate: 1 },
      ],
      due_buckets: {
        overdue: 0,
        due_today: 0,
        due_next_7_days: 0,
        due_later: 1,
        no_due_date: 0,
      },
    });

    render(Reports);

    await waitFor(() => {
      expect(getPipelineConversionReportMock).toHaveBeenCalledOnce();
      expect(getActivityFunnelReportMock).toHaveBeenCalledOnce();
    });

    expect(await screen.findByText('reports.pipeline.loadFailed')).toBeTruthy();
    expect(screen.getByText('reports.activity.title')).toBeTruthy();
    expect(screen.getAllByText('100.0%').length).toBeGreaterThan(0);

    consoleError.mockRestore();
  });

  it('renders activity data while the pipeline report is still loading', async () => {
    let resolvePipelineReport: (value: unknown) => void = () => {};
    const pipelineReport = new Promise((resolve) => {
      resolvePipelineReport = resolve;
    });

    getPipelineConversionReportMock.mockReturnValueOnce(pipelineReport);
    getActivityFunnelReportMock.mockResolvedValueOnce({
      generated_at: '2026-07-08T12:00:00Z',
      total_activities: 1,
      completed_activities: 1,
      pending_activities: 0,
      overdue_activities: 0,
      completion_rate: 1,
      overdue_rate: 0,
      by_type: [
        { activity_type: 'email', total: 1, completed: 1, pending: 0, overdue: 0, completion_rate: 1 },
      ],
      due_buckets: {
        overdue: 0,
        due_today: 0,
        due_next_7_days: 0,
        due_later: 1,
        no_due_date: 0,
      },
    });

    render(Reports);

    await waitFor(() => {
      expect(getPipelineConversionReportMock).toHaveBeenCalledOnce();
      expect(getActivityFunnelReportMock).toHaveBeenCalledOnce();
    });

    expect(await screen.findByText('reports.activity.dueBuckets.title')).toBeTruthy();
    expect(screen.getAllByText('100.0%').length).toBeGreaterThan(0);
    expect(screen.getByText('reports.loading')).toBeTruthy();

    resolvePipelineReport({
      generated_at: '2026-07-08T12:00:00Z',
      total_deals: 0,
      open_deals: 0,
      closed_won: 0,
      closed_lost: 0,
      overall_win_rate: 0,
      stage_metrics: [],
      transition_metrics: [],
    });

    expect(await screen.findByText('reports.pipeline.empty')).toBeTruthy();
  });
});
