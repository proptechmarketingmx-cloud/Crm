import type { Contact } from '$lib/api/contacts';
import { listContacts } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';
import type { Organization } from '$lib/api/organizations';
import { listOrganizations } from '$lib/api/organizations';

export const DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE = 500;

export interface DealRelationshipLookups {
  contacts: Contact[];
  organizations: Organization[];
}

export interface DealRelationshipLabels {
  primaryContactName: string | null;
  organizationName: string | null;
}

export function contactDisplayName(contact: Pick<Contact, 'id' | 'firstName' | 'lastName' | 'email'>): string {
  const fullName = [contact.firstName, contact.lastName]
    .map((part) => part.trim())
    .filter(Boolean)
    .join(' ');

  return fullName || contact.email?.trim() || contact.id;
}

export function deriveDealRelationshipLabels(
  deal: Pick<Deal, 'contactId' | 'contactName' | 'organizationId'>,
  contacts: Pick<Contact, 'id' | 'firstName' | 'lastName' | 'email'>[],
  organizations: Pick<Organization, 'id' | 'name'>[],
): DealRelationshipLabels {
  const contact = deal.contactId
    ? contacts.find((candidate) => candidate.id === deal.contactId)
    : null;
  const organization = deal.organizationId
    ? organizations.find((candidate) => candidate.id === deal.organizationId)
    : null;

  return {
    primaryContactName: contact ? contactDisplayName(contact) : deal.contactName,
    organizationName: organization?.name.trim() || null,
  };
}

export async function loadDealRelationshipContacts(): Promise<Contact[]> {
  const contacts: Contact[] = [];
  let page = 1;
  let total = 0;

  do {
    const result = await listContacts({
      page,
      pageSize: DEAL_RELATIONSHIP_CONTACT_PAGE_SIZE,
      sortBy: 'name',
      sortDir: 'asc',
    });

    contacts.push(...result.contacts);
    total = result.total;

    if (result.contacts.length === 0) {
      break;
    }

    page += 1;
  } while (contacts.length < total);

  return contacts;
}

export async function loadDealRelationshipLookups(): Promise<DealRelationshipLookups> {
  // Pipeline labels and add-deal selectors need relationship lookup coverage
  // without mutating the paginated Contacts route store state.
  const [contacts, organizations] = await Promise.all([
    loadDealRelationshipContacts(),
    listOrganizations(),
  ]);

  return { contacts, organizations };
}
