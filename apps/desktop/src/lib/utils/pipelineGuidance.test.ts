import { describe, expect, it } from 'vitest';
import type { Activity } from '$lib/api/activities';
import type { Deal } from '$lib/api/deals';
import {
  dealStageAgeDays,
  derivePipelineGuidance,
  nextDealActivity,
  weightedForecastValue,
} from '$lib/utils/pipelineGuidance';

const NOW = new Date('2026-07-08T12:00:00Z');

function deal(overrides: Partial<Deal>): Deal {
  return {
    id: 'deal-1',
    name: 'Solar rollout',
    value: 10000,
    currency: 'USD',
    stage: 'proposal',
    probability: 50,
    expectedCloseDate: '2026-08-01',
    contactId: null,
    organizationId: null,
    contactName: null,
    description: null,
    tags: [],
    createdAt: '2026-06-01T08:00:00Z',
    updatedAt: '2026-07-01T08:00:00Z',
    ...overrides,
  };
}

function activity(overrides: Partial<Activity>): Activity {
  return {
    id: 'activity-1',
    type: 'task',
    subject: 'Follow up',
    notes: null,
    dueDate: null,
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

describe('pipeline guidance', () => {
  it('calculates weighted forecast value from deal value and probability', () => {
    expect(weightedForecastValue(deal({ value: 42000, probability: 75 }))).toBe(31500);
    expect(weightedForecastValue(deal({ value: 42000, probability: 150 }))).toBe(42000);
    expect(weightedForecastValue(deal({ value: 42000, probability: -10 }))).toBe(0);
  });

  it('reports days since the deal was last updated as stage-age proxy', () => {
    expect(dealStageAgeDays(deal({ updatedAt: '2026-07-01T23:00:00Z' }), NOW)).toBe(7);
  });

  it('uses the earliest pending due activity as the next deal activity', () => {
    const later = activity({
      id: 'later',
      subject: 'Later call',
      dueDate: '2026-07-15',
    });
    const completed = activity({
      id: 'completed',
      subject: 'Completed call',
      dueDate: '2026-07-05',
      status: 'completed',
      completedAt: '2026-07-05T12:00:00Z',
    });
    const earlier = activity({
      id: 'earlier',
      subject: 'Earlier call',
      dueDate: '2026-07-09',
    });

    expect(nextDealActivity([later, completed, earlier], NOW)?.id).toBe('earlier');
  });

  it('flags open deals without activity as needing follow-up', () => {
    expect(
      derivePipelineGuidance({
        deal: deal({ updatedAt: '2026-07-07T08:00:00Z' }),
        activities: [],
        now: NOW,
      }),
    ).toMatchObject({
      state: 'needsFollowUp',
      tone: 'warning',
      stageAgeDays: 1,
      nextActivity: null,
      weightedForecastValue: 5000,
    });
  });

  it('flags overdue next activity before stale guidance', () => {
    expect(
      derivePipelineGuidance({
        deal: deal({ updatedAt: '2026-06-01T08:00:00Z' }),
        activities: [
          activity({
            id: 'overdue',
            subject: 'Overdue call',
            dueDate: '2026-07-01',
            status: 'pending',
          }),
        ],
        now: NOW,
      }),
    ).toMatchObject({
      state: 'overdue',
      tone: 'danger',
    });
  });

  it('flags stale deals with a future next activity when the deal has not changed recently', () => {
    expect(
      derivePipelineGuidance({
        deal: deal({ updatedAt: '2026-06-01T08:00:00Z' }),
        activities: [
          activity({
            id: 'future',
            subject: 'Future call',
            dueDate: '2026-07-20',
          }),
        ],
        now: NOW,
        staleDays: 14,
      }),
    ).toMatchObject({
      state: 'stale',
      tone: 'warning',
      stageAgeDays: 37,
    });
  });

  it('flags open deals with current follow-up as on track', () => {
    expect(
      derivePipelineGuidance({
        deal: deal({ updatedAt: '2026-07-06T08:00:00Z' }),
        activities: [
          activity({
            id: 'future',
            subject: 'Future call',
            dueDate: '2026-07-20',
          }),
        ],
        now: NOW,
      }),
    ).toMatchObject({
      state: 'onTrack',
      tone: 'success',
      stageAgeDays: 2,
    });
  });

  it('does not show next-step warnings for closed deals', () => {
    expect(
      derivePipelineGuidance({
        deal: deal({ stage: 'closedWon' }),
        activities: [],
        now: NOW,
      }).state,
    ).toBe('closedWon');

    expect(
      derivePipelineGuidance({
        deal: deal({ stage: 'closedLost' }),
        activities: [
          activity({
            id: 'overdue',
            dueDate: '2026-07-01',
            status: 'overdue',
          }),
        ],
        now: NOW,
      }).state,
    ).toBe('closedLost');
  });
});
