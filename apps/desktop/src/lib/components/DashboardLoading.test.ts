// @vitest-environment jsdom

import { render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const {
  createActivityMock,
  getDashboardStatsMock,
  listActivitiesMock,
  listContactsMock,
  listDealsMock,
  loadUpcomingMock,
  openModalMock,
} = vi.hoisted(() => ({
  createActivityMock: vi.fn(),
  getDashboardStatsMock: vi.fn(),
  listActivitiesMock: vi.fn(),
  listContactsMock: vi.fn(),
  listDealsMock: vi.fn(),
  loadUpcomingMock: vi.fn(),
  openModalMock: vi.fn(),
}));

vi.mock('$lib/i18n', () => ({
  t: (key: string) => key,
}));

vi.mock('$lib/api/dashboard', () => ({
  getDashboardStats: getDashboardStatsMock,
}));

vi.mock('$lib/api/activities', () => ({
  createActivity: createActivityMock,
  listActivities: listActivitiesMock,
}));

vi.mock('$lib/api/contacts', () => ({
  listContacts: listContactsMock,
}));

vi.mock('$lib/api/deals', () => ({
  listDeals: listDealsMock,
}));

vi.mock('$lib/stores/activities', () => ({
  activityStore: {
    isLoading: false,
    loadUpcoming: loadUpcomingMock,
    upcoming: [],
  },
}));

vi.mock('$lib/stores/ui', () => ({
  uiStore: {
    openModal: openModalMock,
  },
}));

vi.mock('$lib/stores/settings', () => ({
  settingsStore: {
    currency: 'USD',
    language: 'en',
  },
}));

import Dashboard from '../../routes/Dashboard.svelte';

describe('Dashboard loading behavior', () => {
  beforeEach(() => {
    getDashboardStatsMock.mockReset();
    listActivitiesMock.mockReset();
    listActivitiesMock.mockResolvedValue([]);
    listContactsMock.mockReset();
    listContactsMock.mockResolvedValue({ contacts: [], total: 0, page: 1, pageSize: 100 });
    listDealsMock.mockReset();
    listDealsMock.mockResolvedValue([]);
    loadUpcomingMock.mockReset();
    openModalMock.mockReset();
  });

  it('renders KPI stats even when upcoming activities are still pending', async () => {
    getDashboardStatsMock.mockResolvedValueOnce({
      activeDeals: 3,
      overdueActivities: 0,
      pipelineValue: 1200,
      pipelineValueByCurrency: [{ currency: 'USD', dealCount: 3, totalValue: 1200 }],
      totalContacts: 12,
      upcomingTasks: 4,
    });
    loadUpcomingMock.mockReturnValue(new Promise(() => {}));

    render(Dashboard);

    expect(await screen.findByText('12')).toBeTruthy();
    expect(screen.getByText('3')).toBeTruthy();
    expect(screen.getByText('$1,200')).toBeTruthy();
    expect(screen.getByText('4')).toBeTruthy();
    expect(screen.getByText('dashboard.recentActivity')).toBeTruthy();
    expect(screen.getByText('dashboard.quickActions')).toBeTruthy();
  });

  it('renders overdue and today attention without marking today as overdue', async () => {
    getDashboardStatsMock.mockResolvedValueOnce({
      activeDeals: 1,
      overdueActivities: 0,
      pipelineValue: 900,
      pipelineValueByCurrency: [{ currency: 'USD', dealCount: 1, totalValue: 900 }],
      totalContacts: 4,
      upcomingTasks: 2,
    });
    loadUpcomingMock.mockResolvedValueOnce(undefined);
    listActivitiesMock.mockResolvedValueOnce([
      {
        id: 'overdue',
        type: 'task',
        subject: 'Past due follow-up',
        notes: null,
        dueDate: '2000-01-01',
        completedAt: null,
        status: 'pending',
        contactId: null,
        contactName: null,
        dealId: null,
        dealName: null,
        createdAt: '2000-01-01T00:00:00Z',
        updatedAt: '2000-01-01T00:00:00Z',
      },
      {
        id: 'today',
        type: 'call',
        subject: 'Today follow-up',
        notes: null,
        dueDate: new Date().toISOString().slice(0, 10),
        completedAt: null,
        status: 'pending',
        contactId: null,
        contactName: null,
        dealId: null,
        dealName: null,
        createdAt: '2000-01-01T00:00:00Z',
        updatedAt: '2000-01-01T00:00:00Z',
      },
    ]);

    render(Dashboard);

    expect(await screen.findByText('dashboard.attention.title')).toBeTruthy();
    expect(await screen.findByText('Past due follow-up')).toBeTruthy();
    expect(await screen.findByText('Today follow-up')).toBeTruthy();
    expect(screen.getByText('dashboard.attention.overdue')).toBeTruthy();
    expect(screen.getByText('dashboard.attention.today')).toBeTruthy();
  });

  it('points an empty workspace at import and backup', async () => {
    getDashboardStatsMock.mockResolvedValueOnce({
      activeDeals: 0,
      overdueActivities: 0,
      pipelineValue: 0,
      pipelineValueByCurrency: [],
      totalContacts: 0,
      upcomingTasks: 0,
    });
    loadUpcomingMock.mockResolvedValueOnce(undefined);

    render(Dashboard);

    expect(await screen.findByTestId('first-run-data')).toBeTruthy();
    expect(screen.getByText('dashboard.firstRun.dataTitle')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'dashboard.firstRun.importAction' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'dashboard.firstRun.backupAction' })).toBeTruthy();
  });
});
