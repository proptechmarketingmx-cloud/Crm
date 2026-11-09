import { describe, expect, it } from 'vitest';
import type { Activity } from '$lib/api/activities';
import type { Deal } from '$lib/api/deals';
import { buildStaleDealReport } from './staleDealReport';

const NOW = new Date('2026-07-08T12:00:00Z');

function deal(overrides: Partial<Deal> = {}): Deal {
  return {
    id: 'deal-1',
    name: 'Quiet Clinic Rollout',
    value: 18000,
    currency: 'USD',
    stage: 'proposal',
    probability: 40,
    expectedCloseDate: '2026-08-01',
    contactId: null,
    organizationId: null,
    contactName: null,
    description: null,
    tags: [],
    createdAt: '2026-06-01T08:00:00Z',
    updatedAt: '2026-06-01T08:00:00Z',
    ...overrides,
  };
}

function activity(overrides: Partial<Activity> = {}): Activity {
  return {
    id: 'activity-1',
    type: 'task',
    subject: 'Later site visit',
    notes: null,
    dueDate: '2026-07-20',
    completedAt: null,
    status: 'pending',
    contactId: null,
    contactName: null,
    dealId: 'deal-1',
    dealName: null,
    createdAt: '2026-06-01T08:00:00Z',
    updatedAt: '2026-06-01T08:00:00Z',
    ...overrides,
  };
}

describe('stale deal report', () => {
  it('lists open deals that are stale under the same pipeline rule', () => {
    const report = buildStaleDealReport({
      deals: [
        deal(),
        deal({
          id: 'deal-fresh',
          name: 'Fresh rollout',
          updatedAt: '2026-07-07T08:00:00Z',
        }),
        deal({
          id: 'deal-won',
          name: 'Won rollout',
          stage: 'closedWon',
        }),
      ],
      activities: [
        activity(),
        activity({
          id: 'activity-fresh',
          dealId: 'deal-fresh',
          subject: 'Tomorrow call',
        }),
        activity({
          id: 'activity-won',
          dealId: 'deal-won',
          subject: 'Close-out',
        }),
      ],
      linkIndex: {},
      now: NOW,
    });

    expect(report).toMatchObject({
      count: 1,
      staleDays: 14,
    });
    expect(report.rows).toEqual([
      expect.objectContaining({
        dealId: 'deal-1',
        name: 'Quiet Clinic Rollout',
        stage: 'proposal',
        nextActivitySubject: 'Later site visit',
        href: '/deals/deal-1',
      }),
    ]);
    expect(report.rows[0]?.stageAgeDays).toBeGreaterThanOrEqual(14);
  });

  it('does not list quiet deals that have no next step', () => {
    const report = buildStaleDealReport({
      deals: [deal()],
      activities: [],
      linkIndex: {},
      now: NOW,
    });

    expect(report.count).toBe(0);
    expect(report.rows).toEqual([]);
  });
});
