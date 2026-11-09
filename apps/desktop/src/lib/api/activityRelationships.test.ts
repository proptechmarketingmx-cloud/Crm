import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import type { Activity, ActivityLink } from '$lib/api/activities';
import type { Contact } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';
import type { Organization } from '$lib/api/organizations';
import { activityStore } from '$lib/stores/activities';
import {
  ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE,
  addSelectedActivityLinks,
  filterActivitiesByRelationship,
  deriveActivityRelationshipLabels,
  loadActivityRelationshipContacts,
  relationshipLabelsByActivityId,
  sortActivitiesForDetailTimeline,
} from '$lib/utils/activityRelationships';

function contact(overrides: Partial<Contact>): Contact {
  return {
    id: 'contact-1',
    firstName: 'Amina',
    lastName: 'Khan',
    email: 'amina@example.com',
    phone: null,
    organization: null,
    type: 'person',
    lifecycle: 'customer',
    tags: [],
    notes: null,
    website: null,
    address: null,
    createdAt: '2026-06-24T08:00:00Z',
    updatedAt: '2026-06-24T08:00:00Z',
    deletedAt: null,
    ...overrides,
    organizationId: overrides.organizationId ?? null,
  };
}

function backendContact(id: string, firstName: string, lastName: string) {
  return {
    id,
    contact_type: 'person',
    first_name: firstName,
    last_name: lastName,
    org_name: '',
    email: `${id}@example.com`,
    phone: '',
    address: '',
    city: '',
    country: '',
    org_id: null,
    notes: '',
    created_at: '2026-06-24T08:00:00Z',
    updated_at: '2026-06-24T08:00:00Z',
    deleted_at: null,
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
    createdAt: '2026-06-24T08:00:00Z',
    updatedAt: '2026-06-24T08:00:00Z',
    ...overrides,
  };
}

function link(overrides: Partial<ActivityLink>): ActivityLink {
  return {
    id: 'link-1',
    activityId: 'activity-1',
    entityType: 'contact',
    entityId: 'contact-1',
    createdAt: '2026-06-24T08:00:00Z',
    deletedAt: null,
    deviceId: 'device-1',
    ...overrides,
  };
}

const organization: Organization = {
  id: 'org-1',
  name: 'Nairobi Health',
  email: null,
  phone: null,
  website: null,
  addressLine1: null,
  addressLine2: null,
  city: null,
  region: null,
  country: null,
  postalCode: null,
  source: null,
  description: null,
  createdAt: '2026-06-24T08:00:00Z',
  updatedAt: '2026-06-24T08:00:00Z',
  deletedAt: null,
  deviceId: 'device-1',
};

const deal: Deal = {
  id: 'deal-1',
  name: 'Clinic expansion',
  value: 12000,
  currency: 'USD',
  stage: 'proposal',
  probability: 50,
  expectedCloseDate: null,
  contactId: null,
  organizationId: null,
  contactName: null,
  description: null,
  tags: [],
  createdAt: '2026-06-24T08:00:00Z',
  updatedAt: '2026-06-24T08:00:00Z',
};

const backendActivityLink = {
  id: 'activity-link-1',
  activity_id: 'activity-1',
  entity_type: 'organization' as const,
  entity_id: 'org-1',
  created_at: '2026-06-24T08:30:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

describe('activity relationship labels', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('derives contact, organization, and deal labels from activity links and lookups', () => {
    expect(
      deriveActivityRelationshipLabels(
        activity({}),
        [
          link({ entityType: 'contact', entityId: 'contact-1' }),
          link({ entityType: 'organization', entityId: 'org-1' }),
          link({ entityType: 'deal', entityId: 'deal-1' }),
        ],
        {
          contacts: [contact({})],
          organizations: [organization],
          deals: [deal],
        },
      ),
    ).toEqual({
      contacts: [{ id: 'contact-1', label: 'Amina Khan' }],
      organizations: [{ id: 'org-1', label: 'Nairobi Health' }],
      deals: [{ id: 'deal-1', label: 'Clinic expansion' }],
    });
  });

  it('keeps duplicate display labels keyed by distinct entity ids', () => {
    const duplicateOrganization = {
      ...organization,
      id: 'org-2',
      name: organization.name,
    };

    expect(
      deriveActivityRelationshipLabels(
        activity({}),
        [
          link({ id: 'link-org-1', entityType: 'organization', entityId: 'org-1' }),
          link({ id: 'link-org-2', entityType: 'organization', entityId: 'org-2' }),
        ],
        {
          contacts: [],
          organizations: [organization, duplicateOrganization],
          deals: [],
        },
      ).organizations,
    ).toEqual([
      { id: 'org-1', label: 'Nairobi Health' },
      { id: 'org-2', label: 'Nairobi Health' },
    ]);
  });

  it('preserves legacy contact and deal fallback labels when links are absent', () => {
    expect(
      deriveActivityRelationshipLabels(
        activity({
          contactId: 'legacy-contact',
          contactName: 'Legacy Contact',
          dealId: 'legacy-deal',
          dealName: 'Legacy Deal',
        }),
        [],
        {
          contacts: [],
          organizations: [],
          deals: [],
        },
      ),
    ).toEqual({
      contacts: [{ id: 'legacy-contact', label: 'Legacy Contact' }],
      organizations: [],
      deals: [{ id: 'legacy-deal', label: 'Legacy Deal' }],
    });
  });

  it('collapses duplicate active links and excludes deleted relationship links', () => {
    expect(
      deriveActivityRelationshipLabels(
        activity({ contactId: 'contact-1' }),
        [
          link({ id: 'link-contact-1', entityType: 'contact', entityId: 'contact-1' }),
          link({ id: 'link-contact-duplicate', entityType: 'contact', entityId: 'contact-1' }),
          link({
            id: 'link-deleted-org',
            entityType: 'organization',
            entityId: 'org-1',
            deletedAt: '2026-06-24T09:00:00Z',
          }),
        ],
        {
          contacts: [contact({})],
          organizations: [organization],
          deals: [],
        },
      ),
    ).toEqual({
      contacts: [{ id: 'contact-1', label: 'Amina Khan' }],
      organizations: [],
      deals: [],
    });
  });

  it('filters activities by legacy mirrors and explicit active relationship links', () => {
    const legacyContact = activity({
      id: 'legacy-contact',
      contactId: 'contact-1',
      subject: 'Legacy contact follow-up',
    });
    const linkedContactOnly = activity({
      id: 'linked-contact',
      contactId: null,
      subject: 'Linked contact follow-up',
    });
    const deletedLinkedContact = activity({
      id: 'deleted-linked-contact',
      contactId: null,
      subject: 'Deleted linked contact follow-up',
    });
    const linkedDeal = activity({
      id: 'linked-deal',
      dealId: null,
      subject: 'Linked deal follow-up',
    });

    expect(
      filterActivitiesByRelationship(
        [legacyContact, linkedContactOnly, deletedLinkedContact, linkedDeal],
        {
          [linkedContactOnly.id]: [
            link({ activityId: linkedContactOnly.id, entityType: 'contact', entityId: 'contact-1' }),
          ],
          [deletedLinkedContact.id]: [
            link({
              activityId: deletedLinkedContact.id,
              entityType: 'contact',
              entityId: 'contact-1',
              deletedAt: '2026-06-24T09:00:00Z',
            }),
          ],
          [linkedDeal.id]: [
            link({ activityId: linkedDeal.id, entityType: 'deal', entityId: 'deal-1' }),
          ],
        },
        { contactId: 'contact-1' },
      ).map((entry) => entry.id),
    ).toEqual(['legacy-contact', 'linked-contact']);

    expect(
      filterActivitiesByRelationship(
        [legacyContact, linkedContactOnly, deletedLinkedContact, linkedDeal],
        {
          [linkedDeal.id]: [
            link({ activityId: linkedDeal.id, entityType: 'deal', entityId: 'deal-1' }),
          ],
        },
        { dealId: 'deal-1' },
      ).map((entry) => entry.id),
    ).toEqual(['linked-deal']);
  });

  it('sorts detail timelines by due date first and undated recent activity after scheduled work', () => {
    const undatedNewer = activity({
      id: 'undated-newer',
      dueDate: null,
      updatedAt: '2026-06-24T12:00:00Z',
    });
    const scheduledLater = activity({
      id: 'scheduled-later',
      dueDate: '2026-07-12',
      updatedAt: '2026-06-24T09:00:00Z',
    });
    const scheduledEarlier = activity({
      id: 'scheduled-earlier',
      dueDate: '2026-07-10',
      updatedAt: '2026-06-24T09:00:00Z',
    });
    const undatedOlder = activity({
      id: 'undated-older',
      dueDate: null,
      updatedAt: '2026-06-24T08:00:00Z',
    });

    expect(
      sortActivitiesForDetailTimeline([
        undatedOlder,
        scheduledLater,
        undatedNewer,
        scheduledEarlier,
      ]).map((entry) => entry.id),
    ).toEqual(['scheduled-earlier', 'scheduled-later', 'undated-newer', 'undated-older']);
  });

  it('builds relationship labels keyed by activity id for feeds', () => {
    const linkedActivity = activity({ id: 'activity-linked' });

    expect(
      relationshipLabelsByActivityId(
        [linkedActivity],
        {
          [linkedActivity.id]: [
            link({ activityId: linkedActivity.id, entityType: 'organization', entityId: 'org-1' }),
          ],
        },
        {
          contacts: [],
          organizations: [organization],
          deals: [],
        },
      ),
    ).toEqual({
      'activity-linked': {
        contacts: [],
        organizations: [{ id: 'org-1', label: 'Nairobi Health' }],
        deals: [],
      },
    });
  });

  it('loads contact relationship lookups beyond the first contact page', async () => {
    const firstPage = Array.from({ length: ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE }, (_, index) =>
      backendContact(`contact-${index + 1}`, 'Contact', String(index + 1))
    );
    const linkedContact = backendContact('contact-501', 'Zara', 'Ndlovu');

    invokeMock
      .mockResolvedValueOnce({
        contacts: firstPage,
        total: ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE + 1,
        page: 1,
        per_page: ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE,
      })
      .mockResolvedValueOnce({
        contacts: [linkedContact],
        total: ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE + 1,
        page: 2,
        per_page: ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE,
      });

    const contacts = await loadActivityRelationshipContacts();

    expect(contacts).toHaveLength(ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE + 1);
    expect(
      deriveActivityRelationshipLabels(
        activity({ contactId: 'contact-501' }),
        [],
        {
          contacts,
          organizations: [],
          deals: [],
        },
      ),
    ).toMatchObject({
      contacts: [{ id: 'contact-501', label: 'Zara Ndlovu' }],
    });
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'list_contacts', {
      params: expect.objectContaining({
        page: 1,
        per_page: ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE,
        sort_by: 'first_name',
        sort_dir: 'asc',
      }),
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'list_contacts', {
      params: expect.objectContaining({
        page: 2,
        per_page: ACTIVITY_RELATIONSHIP_CONTACT_PAGE_SIZE,
      }),
    });
  });

  it('adds selected activity links through the existing activity relationship wrappers', async () => {
    invokeMock.mockResolvedValue(backendActivityLink);

    await addSelectedActivityLinks(' activity-1 ', {
      contactId: ' contact-1 ',
      organizationId: ' org-1 ',
      dealId: ' deal-1 ',
    });

    expect(invokeMock).toHaveBeenCalledWith('add_activity_link', {
      activity_id: 'activity-1',
      entity_type: 'contact',
      entity_id: 'contact-1',
    });
    expect(invokeMock).toHaveBeenCalledWith('add_activity_link', {
      activity_id: 'activity-1',
      entity_type: 'organization',
      entity_id: 'org-1',
    });
    expect(invokeMock).toHaveBeenCalledWith('add_activity_link', {
      activity_id: 'activity-1',
      entity_type: 'deal',
      entity_id: 'deal-1',
    });
  });
});

describe('activity relationship refresh signal', () => {
  it('increments when activity relationship links change', () => {
    const before = activityStore.relationshipRefreshVersion;

    activityStore.notifyRelationshipLinksChanged();

    expect(activityStore.relationshipRefreshVersion).toBe(before + 1);
  });
});
