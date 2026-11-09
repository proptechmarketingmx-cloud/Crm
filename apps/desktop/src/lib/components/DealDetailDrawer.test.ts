// @vitest-environment jsdom

import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { markCompleteMock, navigateHashMock } = vi.hoisted(() => ({
  markCompleteMock: vi.fn(),
  navigateHashMock: vi.fn(),
}));

vi.mock('$lib/stores/activities', () => ({
  activityStore: {
    markComplete: markCompleteMock,
  },
}));

vi.mock('$lib/utils/hashRouter', () => ({
  navigateHash: navigateHashMock,
}));

import type { Activity } from '$lib/api/activities';
import type { Deal } from '$lib/api/deals';
import type { PipelineGuidance } from '$lib/utils/pipelineGuidance';
import DealDetailDrawer from './DealDetailDrawer.svelte';

function deal(overrides: Partial<Deal> = {}): Deal {
  return {
    id: 'deal-1',
    name: 'Guided rollout',
    value: 50000,
    currency: 'USD',
    stage: 'proposal',
    probability: 50,
    expectedCloseDate: '2026-08-01',
    contactId: null,
    organizationId: null,
    contactName: null,
    description: 'Coordinate rollout with the buying committee.',
    tags: [],
    createdAt: '2026-06-01T08:00:00Z',
    updatedAt: '2026-07-01T08:00:00Z',
    ...overrides,
  };
}

function activity(overrides: Partial<Activity> = {}): Activity {
  return {
    id: 'activity-1',
    type: 'task',
    subject: 'Follow up',
    notes: null,
    dueDate: '2026-07-20',
    completedAt: null,
    status: 'pending',
    contactId: null,
    contactName: null,
    dealId: 'deal-1',
    dealName: null,
    createdAt: '2026-07-01T08:00:00Z',
    updatedAt: '2026-07-01T08:00:00Z',
    ...overrides,
  };
}

function guidance(overrides: Partial<PipelineGuidance> = {}): PipelineGuidance {
  return {
    state: 'onTrack',
    tone: 'success',
    stageAgeDays: 7,
    weightedForecastValue: 25000,
    nextActivity: null,
    ...overrides,
  };
}

describe('DealDetailDrawer component', () => {
  beforeEach(() => {
    markCompleteMock.mockReset();
    navigateHashMock.mockReset();
  });

  it('shows loading guidance while linked activity context is unresolved', () => {
    render(DealDetailDrawer, {
      deal: deal(),
      guidance: null,
      activitiesLoading: true,
    });

    expect(screen.getByRole('dialog', { name: 'Guided rollout' })).toBeTruthy();
    expect(screen.getByText('Loading')).toBeTruthy();
    expect(screen.getByText('Loading linked activity context before showing deal guidance.')).toBeTruthy();
    expect(screen.getByText('$25,000')).toBeTruthy();
  });

  it('shows unavailable guidance instead of deriving from empty activities after load failure', () => {
    render(DealDetailDrawer, {
      deal: deal(),
      guidance: null,
      activityContextError: 'Deal guidance could not load linked activity context.',
    });

    expect(screen.getByText('Unavailable')).toBeTruthy();
    expect(screen.getAllByText('Deal guidance could not load linked activity context.')).toHaveLength(2);
    expect(screen.queryByText('Needs Follow-Up')).toBeNull();
    expect(screen.queryByText('No activities yet')).toBeNull();
  });

  it('closes on Escape from the document-level dialog handler', async () => {
    const onclose = vi.fn();

    render(DealDetailDrawer, {
      deal: deal(),
      guidance: guidance(),
      onclose,
    });

    await fireEvent.keyDown(document, { key: 'Escape' });

    expect(onclose).toHaveBeenCalledOnce();
  });

  it('does not show the primary follow-up CTA for closed deals', () => {
    render(DealDetailDrawer, {
      deal: deal({ stage: 'closedWon' }),
      guidance: guidance({ state: 'closedWon', tone: 'success' }),
    });

    expect(screen.getAllByText('Closed Won')).toHaveLength(2);
    expect(screen.queryByRole('button', { name: 'Add Follow-Up' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Add Activity' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'This deal is closed won' })).toBeTruthy();
  });

  it('uses the shared next-step action to complete overdue work', async () => {
    const overdue = activity({
      id: 'overdue-1',
      subject: 'Past due clinic check-in',
      status: 'overdue',
    });
    markCompleteMock.mockResolvedValueOnce(undefined);
    const onrefresh = vi.fn();

    render(DealDetailDrawer, {
      deal: deal(),
      guidance: guidance({
        state: 'overdue',
        tone: 'danger',
        nextActivity: overdue,
      }),
      onrefresh,
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Mark Complete' }));

    await waitFor(() => {
      expect(markCompleteMock).toHaveBeenCalledWith('overdue-1');
      expect(onrefresh).toHaveBeenCalledOnce();
    });
  });

  it('opens the deal page when the next step is a missing close date', async () => {
    render(DealDetailDrawer, {
      deal: deal({ expectedCloseDate: null }),
      guidance: guidance({
        state: 'onTrack',
        nextActivity: activity({ subject: 'Site visit' }),
      }),
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Set close date' }));

    expect(navigateHashMock).toHaveBeenCalledWith('/deals/deal-1');
  });
});
