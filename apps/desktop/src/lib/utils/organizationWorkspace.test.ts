import { describe, expect, it } from 'vitest';
import type { Activity, ActivityLink } from '$lib/api/activities';
import type { Contact } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';
import {
  buildOrganizationListInsight,
  deriveOrganizationHealth,
  filterOrganizationActivities,
  filterOrganizationContacts,
  filterOrganizationDeals,
  nextOrganizationActivity,
  openPipelineByCurrency,
  recentOrganizationActivity,
} from './organizationWorkspace';

function contact(overrides: Partial<Contact>): Contact {
  return {
    id: 'contact-1',
    firstName: 'Ada',
    lastName: 'Lovelace',
    email: null,
    phone: null,
    organization: null,
    organizationId: null,
    type: 'person',
    lifecycle: 'customer',
    tags: [],
    notes: null,
    website: null,
    address: null,
    createdAt: '2026-07-01T00:00:00Z',
    updatedAt: '2026-07-01T00:00:00Z',
    deletedAt: null,
    ...overrides,
  };
}

function deal(overrides: Partial<Deal>): Deal {
  return {
    id: 'deal-1',
    name: 'Expansion',
    value: 100,
    currency: 'USD',
    stage: 'proposal',
    probability: 50,
    expectedCloseDate: null,
    contactId: null,
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
    contactId: null,
    contactName: null,
    dealId: null,
    dealName: null,
    createdAt: '2026-07-01T00:00:00Z',
    updatedAt: '2026-07-01T00:00:00Z',
    ...overrides,
  };
}

function link(overrides: Partial<ActivityLink>): ActivityLink {
  return {
    id: 'link-1',
    activityId: 'activity-1',
    entityType: 'organization',
    entityId: 'org-1',
    createdAt: '2026-07-01T00:00:00Z',
    deletedAt: null,
    deviceId: 'device-1',
    ...overrides,
  };
}

describe('organizationWorkspace utilities', () => {
  it('derives linked people from normalized organization ids only', () => {
    const linked = contact({ id: 'contact-1', organizationId: 'org-1' });
    const legacyOnly = contact({ id: 'contact-2', organization: 'Org One', organizationId: null });
    const organizationContact = contact({ id: 'contact-3', organizationId: 'org-1', type: 'org' });

    expect(filterOrganizationContacts([linked, legacyOnly, organizationContact], 'org-1'))
      .toEqual([linked]);
  });

  it('groups open organization pipeline by currency and excludes closed deals', () => {
    const deals = [
      deal({ id: 'deal-1', organizationId: 'org-1', value: 100, currency: 'USD' }),
      deal({ id: 'deal-2', organizationId: 'org-1', value: 200, currency: 'EUR' }),
      deal({ id: 'deal-3', organizationId: 'org-1', value: 300, currency: 'USD', stage: 'closedWon' }),
      deal({ id: 'deal-4', organizationId: 'org-2', value: 400, currency: 'USD' }),
    ];

    const organizationDeals = filterOrganizationDeals(deals, 'org-1');

    expect(organizationDeals.map((item) => item.id)).toEqual(['deal-1', 'deal-2', 'deal-3']);
    expect(openPipelineByCurrency(organizationDeals, 'USD')).toEqual([
      { currency: 'EUR', value: 200 },
      { currency: 'USD', value: 100 },
    ]);
  });

  it('derives organization activities from active activity links', () => {
    const activities = [
      activity({ id: 'activity-1', subject: 'Linked active' }),
      activity({ id: 'activity-2', subject: 'Deleted link' }),
      activity({ id: 'activity-3', subject: 'Other org' }),
    ];

    const linkIndex = {
      'activity-1': [link({ id: 'link-1', activityId: 'activity-1', entityId: 'org-1' })],
      'activity-2': [link({ id: 'link-2', activityId: 'activity-2', entityId: 'org-1', deletedAt: '2026-07-02T00:00:00Z' })],
      'activity-3': [link({ id: 'link-3', activityId: 'activity-3', entityId: 'org-2' })],
    };

    expect(filterOrganizationActivities(activities, linkIndex, 'org-1').map((item) => item.id))
      .toEqual(['activity-1']);
  });

  it('selects next, recent, and health states for organization summaries', () => {
    const next = activity({
      id: 'activity-next',
      subject: 'Next call',
      dueDate: '2026-07-10T09:00:00Z',
      updatedAt: '2026-07-03T00:00:00Z',
    });
    const recent = activity({
      id: 'activity-recent',
      subject: 'Recent note',
      dueDate: null,
      updatedAt: '2026-07-05T00:00:00Z',
    });

    expect(nextOrganizationActivity([recent, next])).toEqual(next);
    expect(recentOrganizationActivity([next, recent])).toEqual(recent);
    expect(
      deriveOrganizationHealth({
        isLoading: false,
        openDealCount: 1,
        pendingActivities: [next],
        overdueActivities: [],
        nextActivity: next,
      }),
    ).toEqual({ state: 'onTrack', tone: 'success', subject: 'Next call' });
  });

  it('builds list insights from open deals and linked account activity', () => {
    const overdue = activity({
      id: 'activity-overdue',
      subject: 'Past due clinic check-in',
      dueDate: '2026-06-01',
      status: 'overdue',
    });
    const orgLink = link({
      activityId: overdue.id,
      entityType: 'organization',
      entityId: 'org-1',
    });

    expect(
      buildOrganizationListInsight({
        organizationId: 'org-1',
        deals: [deal({ organizationId: 'org-1', stage: 'proposal' })],
        activities: [overdue, activity({ id: 'other', subject: 'Other account' })],
        linkIndex: { [overdue.id]: [orgLink] },
        isLoading: false,
      }),
    ).toMatchObject({
      health: { state: 'overdue', tone: 'danger', subject: 'Past due clinic check-in' },
      nextActivity: overdue,
    });

    expect(
      buildOrganizationListInsight({
        organizationId: 'org-2',
        deals: [deal({ organizationId: 'org-2', stage: 'qualified' })],
        activities: [overdue],
        linkIndex: { [overdue.id]: [orgLink] },
        isLoading: false,
      }),
    ).toMatchObject({
      health: { state: 'needsFollowUp', tone: 'warning' },
      nextActivity: null,
    });
  });
});
