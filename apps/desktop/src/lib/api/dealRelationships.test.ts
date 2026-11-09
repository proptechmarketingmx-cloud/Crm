import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import type { Contact } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';
import type { Organization } from '$lib/api/organizations';
import {
  DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE,
  contactDisplayName,
  deriveDealRelationshipLabels,
  loadDealRelationshipContacts,
} from '$lib/utils/dealRelationships';

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
    device_id: 'device-1',
  };
}

function deal(overrides: Partial<Deal>): Deal {
  return {
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

describe('deal relationship labels', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('formats contact selector labels with stable fallbacks', () => {
    expect(contactDisplayName(contact({ firstName: ' Amina ', lastName: ' Khan ' }))).toBe('Amina Khan');
    expect(contactDisplayName(contact({ firstName: '', lastName: '', email: 'owner@example.com' }))).toBe('owner@example.com');
    expect(contactDisplayName(contact({ id: 'contact-fallback', firstName: '', lastName: '', email: null }))).toBe('contact-fallback');
  });

  it('derives primary contact and organization names from loaded frontend lookups', () => {
    expect(
      deriveDealRelationshipLabels(
        deal({ contactId: 'contact-1', organizationId: 'org-1' }),
        [contact({})],
        [organization],
      ),
    ).toEqual({
      primaryContactName: 'Amina Khan',
      organizationName: 'Nairobi Health',
    });
  });

  it('falls back to backend contactName when the contact lookup is not loaded', () => {
    expect(
      deriveDealRelationshipLabels(
        deal({ contactId: 'missing-contact', contactName: 'Legacy Name' }),
        [],
        [],
      ),
    ).toEqual({
      primaryContactName: 'Legacy Name',
      organizationName: null,
    });
  });

  it('loads contact relationship lookups beyond the first contact page', async () => {
    const firstPage = Array.from({ length: DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE }, (_, index) =>
      backendContact(`contact-${index + 1}`, 'Contact', String(index + 1))
    );
    const linkedContact = backendContact('contact-501', 'Zara', 'Ndlovu');

    invokeMock
      .mockResolvedValueOnce({
        contacts: firstPage,
        total: DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE + 1,
        page: 1,
        per_page: DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE,
      })
      .mockResolvedValueOnce({
        contacts: [linkedContact],
        total: DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE + 1,
        page: 2,
        per_page: DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE,
      });

    const contacts = await loadDealRelationshipContacts();

    expect(contacts).toHaveLength(DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE + 1);
    expect(
      deriveDealRelationshipLabels(
        deal({ contactId: 'contact-501', contactName: null }),
        contacts,
        [],
      ),
    ).toEqual({
      primaryContactName: 'Zara Ndlovu',
      organizationName: null,
    });
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'list_contacts', {
      params: expect.objectContaining({
        page: 1,
        per_page: DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE,
        sort_by: 'first_name',
        sort_dir: 'asc',
      }),
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'list_contacts', {
      params: expect.objectContaining({
        page: 2,
        per_page: DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE,
      }),
    });
  });
});
