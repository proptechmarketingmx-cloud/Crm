import { describe, expect, it } from 'vitest';
import type { Activity } from '$lib/api/activities';
import type { Contact } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';
import {
  buildDashboardAttentionQueue,
  buildDashboardAttentionSummary,
  buildDealStageFollowUpSuggestion,
} from '$lib/utils/localAutomation';

const NOW = new Date('2026-07-08T16:30:00');

function deal(overrides: Partial<Deal> = {}): Deal {
  return {
    id: 'deal-1',
    name: 'Solar rollout',
    value: 12000,
    currency: 'USD',
    stage: 'proposal',
    probability: 60,
    expectedCloseDate: '2026-08-01',
    contactId: 'contact-1',
    organizationId: 'organization-1',
    contactName: null,
    description: null,
    tags: [],
    createdAt: '2026-07-01T08:00:00Z',
    updatedAt: '2026-07-08T08:00:00Z',
    ...overrides,
  };
}

function contact(overrides: Partial<Contact> = {}): Contact {
  return {
    id: 'contact-1',
    firstName: 'Kofi',
    lastName: 'Mensah',
    email: 'kofi@example.com',
    phone: null,
    organization: null,
    organizationId: null,
    type: 'person',
    lifecycle: 'lead',
    tags: [],
    notes: null,
    website: null,
    address: null,
    createdAt: '2026-07-01T08:00:00Z',
    updatedAt: '2026-07-01T08:00:00Z',
    deletedAt: null,
    ...overrides,
  };
}

function activity(overrides: Partial<Activity>): Activity {
  return {
    id: 'activity-1',
    type: 'task',
    subject: 'Follow up',
    notes: null,
    dueDate: '2026-07-08',
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

describe('local automation helpers', () => {
  it('drafts an explicit follow-up suggestion after an open stage move with no linked activity', () => {
    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'qualified' }),
        activities: [],
        fromStage: 'lead',
        toStage: 'qualified',
        now: NOW,
      }),
    ).toMatchObject({
      dealId: 'deal-1',
      dealName: 'Solar rollout',
      fromStage: 'lead',
      toStage: 'qualified',
      draft: {
        subject: 'Follow up on Solar rollout',
        type: 'task',
        dueDate: '2026-07-09',
        contactId: 'contact-1',
        organizationId: 'organization-1',
        dealId: 'deal-1',
      },
    });
  });

  it('does not suggest a draft for closed deals, unchanged stages, loaded activities, or unresolved context', () => {
    const futureActivity = activity({ dueDate: '2026-07-10' });

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'closedWon' }),
        activities: [],
        fromStage: 'proposal',
        toStage: 'closedWon',
        now: NOW,
      }),
    ).toBeNull();

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'qualified' }),
        activities: [],
        fromStage: 'closedWon',
        toStage: 'qualified',
        now: NOW,
      }),
    ).toBeNull();

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'proposal' }),
        activities: [],
        fromStage: 'proposal',
        toStage: 'proposal',
        now: NOW,
      }),
    ).toBeNull();

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'qualified' }),
        activities: [futureActivity],
        fromStage: 'lead',
        toStage: 'qualified',
        now: NOW,
      }),
    ).toBeNull();

    expect(
      buildDealStageFollowUpSuggestion({
        deal: deal({ stage: 'qualified' }),
        activities: [],
        fromStage: 'lead',
        toStage: 'qualified',
        now: NOW,
        activityContextReady: false,
      }),
    ).toBeNull();
  });

  it('builds dashboard attention with local-day semantics', () => {
    const summary = buildDashboardAttentionSummary([
      activity({ id: 'yesterday', subject: 'Past due', dueDate: '2026-07-07' }),
      activity({ id: 'today', subject: 'Today call', dueDate: '2026-07-08' }),
      activity({ id: 'tomorrow', subject: 'Tomorrow email', dueDate: '2026-07-09' }),
      activity({ id: 'done', subject: 'Done', dueDate: '2026-07-07', status: 'completed' }),
    ], NOW);

    expect(summary).toMatchObject({
      overdueCount: 1,
      todayCount: 1,
      totalCount: 2,
    });
    expect(summary.items.map((item) => [item.id, item.bucket])).toEqual([
      ['activity:yesterday', 'overdue'],
      ['activity:today', 'today'],
    ]);
  });

  it('adds waiting leads and deals without a next step to the dashboard queue', () => {
    const waitingLead = contact();
    const workedLead = contact({
      id: 'contact-2',
      firstName: 'Amina',
      lastName: 'Diallo',
      createdAt: '2026-07-02T08:00:00Z',
    });
    const stuckDeal = deal({ id: 'deal-stuck', name: 'Unworked Clinic Kit', updatedAt: '2026-06-01T08:00:00Z' });
    const workedDeal = deal({ id: 'deal-worked', name: 'Guided rollout' });

    const queue = buildDashboardAttentionQueue({
      activities: [
        activity({
          id: 'lead-call',
          subject: 'Call Amina',
          contactId: workedLead.id,
          dealId: null,
          dueDate: '2026-07-10',
        }),
        activity({
          id: 'deal-task',
          subject: 'Advance guided rollout',
          dealId: workedDeal.id,
          dueDate: '2026-07-10',
        }),
      ],
      deals: [workedDeal, stuckDeal],
      leads: [workedLead, waitingLead],
      now: NOW,
    });

    expect(queue).toMatchObject({
      overdueCount: 0,
      todayCount: 0,
      dealCount: 1,
      leadCount: 1,
      totalCount: 2,
    });
    expect(queue.items).toEqual([
      expect.objectContaining({
        id: 'deal:deal-stuck',
        kind: 'dealNeedsFollowUp',
        title: 'Unworked Clinic Kit',
        href: '/deals/deal-stuck',
      }),
      expect.objectContaining({
        id: 'lead:contact-1',
        kind: 'leadWaiting',
        title: 'Kofi Mensah',
        href: '/contacts/contact-1',
      }),
    ]);
  });

  it('keeps overdue follow-ups ahead of stuck deals and waiting leads', () => {
    const queue = buildDashboardAttentionQueue({
      activities: [
        activity({
          id: 'late',
          subject: 'Past due clinic check-in',
          dueDate: '2026-07-07',
          dealId: null,
          contactId: null,
        }),
      ],
      deals: [deal({ id: 'deal-stuck', name: 'Unworked Clinic Kit' })],
      leads: [contact()],
      now: NOW,
    });

    expect(queue.items.map((item) => item.kind)).toEqual([
      'overdue',
      'dealNeedsFollowUp',
      'leadWaiting',
    ]);
    expect(queue.items[0]?.href).toBe('/activities');
  });

  it('filters the dashboard queue by owner name', () => {
    const ownedLead = contact({ owner: 'Samira' });
    const otherLead = contact({
      id: 'contact-2',
      firstName: 'Amina',
      lastName: 'Diallo',
      owner: 'Kofi',
    });
    const ownedDeal = deal({ id: 'deal-owned', name: 'Owned kit', owner: 'Samira' });
    const otherDeal = deal({ id: 'deal-other', name: 'Other kit', owner: 'Kofi' });

    const queue = buildDashboardAttentionQueue({
      activities: [
        activity({
          id: 'owned-call',
          subject: 'Call Samira clinic',
          contactId: ownedLead.id,
          dealId: null,
          dueDate: '2026-07-07',
        }),
        activity({
          id: 'other-call',
          subject: 'Call Kofi clinic',
          contactId: otherLead.id,
          dealId: null,
          dueDate: '2026-07-07',
        }),
      ],
      deals: [ownedDeal, otherDeal],
      leads: [ownedLead, otherLead],
      owner: 'samira',
      now: NOW,
    });

    expect(queue.items.map((item) => item.title)).toEqual([
      'Call Samira clinic',
      'Owned kit',
    ]);
  });
});
