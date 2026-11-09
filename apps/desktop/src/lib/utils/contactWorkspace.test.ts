import { describe, expect, it } from 'vitest';
import type { Activity, ActivityLink } from '$lib/api/activities';
import type { Deal } from '$lib/api/deals';
import {
  buildContactListInsight,
  deriveContactHealth,
  filterContactDeals,
  nextContactActivity,
} from './contactWorkspace';

function deal(overrides: Partial<Deal> = {}): Deal {
  return {
    id: 'deal-1',
    name: 'Clinic kit',
    value: 100,
    currency: 'USD',
    stage: 'proposal',
    probability: 50,
    expectedCloseDate: null,
    contactId: 'contact-1',
    organizationId: null,
    contactName: null,
    description: null,
    tags: [],
    createdAt: '2026-07-01T00:00:00Z',
    updatedAt: '2026-07-01T00:00:00Z',
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
    contactId: 'contact-1',
    contactName: null,
    dealId: null,
    dealName: null,
    createdAt: '2026-07-01T00:00:00Z',
    updatedAt: '2026-07-01T00:00:00Z',
    ...overrides,
  };
}

function link(overrides: Partial<ActivityLink> = {}): ActivityLink {
  return {
    id: 'link-1',
    activityId: 'activity-1',
    entityType: 'contact',
    entityId: 'contact-1',
    createdAt: '2026-07-01T00:00:00Z',
    deletedAt: null,
    deviceId: 'test-device',
    ...overrides,
  };
}

describe('contact workspace helpers', () => {
  it('filters contact deals and picks the next pending follow-up', () => {
    expect(filterContactDeals([
      deal(),
      deal({ id: 'deal-2', contactId: 'contact-2' }),
    ], 'contact-1')).toHaveLength(1);

    const next = activity({ id: 'next', subject: 'Next call', dueDate: '2026-07-10' });
    const later = activity({ id: 'later', subject: 'Later', dueDate: '2026-07-20' });
    expect(nextContactActivity([later, next])).toEqual(next);
  });

  it('marks overdue work first, then open deals without a next step', () => {
    const overdue = activity({
      id: 'overdue',
      subject: 'Past due clinic check-in',
      dueDate: '2026-06-01',
      status: 'overdue',
    });

    expect(
      deriveContactHealth({
        isLoading: false,
        openDealCount: 1,
        pendingActivities: [overdue],
        overdueActivities: [overdue],
        nextActivity: overdue,
      }),
    ).toEqual({
      state: 'overdue',
      tone: 'danger',
      subject: 'Past due clinic check-in',
    });

    expect(
      deriveContactHealth({
        isLoading: false,
        openDealCount: 1,
        pendingActivities: [],
        overdueActivities: [],
        nextActivity: null,
      }),
    ).toEqual({ state: 'needsFollowUp', tone: 'warning' });
  });

  it('builds list insights from open deals and contact-linked activity', () => {
    const overdue = activity({
      id: 'activity-overdue',
      subject: 'Past due clinic check-in',
      dueDate: '2026-06-01',
      status: 'overdue',
      contactId: null,
    });

    expect(
      buildContactListInsight({
        contactId: 'contact-1',
        deals: [deal()],
        activities: [overdue],
        linkIndex: {
          [overdue.id]: [link({ activityId: overdue.id })],
        },
        isLoading: false,
      }),
    ).toMatchObject({
      health: { state: 'overdue', tone: 'danger', subject: 'Past due clinic check-in' },
      nextActivity: overdue,
    });

    expect(
      buildContactListInsight({
        contactId: 'contact-2',
        deals: [deal({ contactId: 'contact-2', stage: 'qualified' })],
        activities: [overdue],
        linkIndex: { [overdue.id]: [link({ activityId: overdue.id })] },
        isLoading: false,
      }),
    ).toMatchObject({
      health: { state: 'needsFollowUp', tone: 'warning' },
      nextActivity: null,
    });
  });
});
