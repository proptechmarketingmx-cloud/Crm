/**
 * src/lib/api/contacts.ts — Tauri IPC wrappers for contact commands.
 */

import { invoke } from '@tauri-apps/api/core';

export type ContactType = 'person' | 'org';
export type ContactLifecycle = 'lead' | 'customer';

export interface Contact {
  id: string;
  firstName: string;
  lastName: string;
  email: string | null;
  phone: string | null;
  organization: string | null;
  organizationId: string | null;
  type: ContactType;
  lifecycle: ContactLifecycle;
  tags: string[];
  notes: string | null;
  website: string | null;
  address: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  owner?: string | null;
}

export type CreateContactPayload = Omit<
  Contact,
  'id' | 'createdAt' | 'updatedAt' | 'deletedAt' | 'organizationId' | 'lifecycle'
> & {
  organizationId?: string | null;
  lifecycle?: ContactLifecycle;
  owner?: string | null;
};
export type UpdateContactPayload = Partial<CreateContactPayload>;

export interface ListContactsParams {
  search?: string;
  type?: ContactType;
  lifecycle?: ContactLifecycle;
  owner?: string;
  tags?: string[];
  customFieldDefId?: string;
  customFieldQuery?: string;
  sortBy?: 'name' | 'createdAt' | 'updatedAt';
  sortDir?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

export interface ContactListResponse {
  contacts: Contact[];
  total: number;
  page: number;
  pageSize: number;
}

export type ContactDuplicateMatchType = 'email' | 'phone';

export interface ContactDuplicateCandidate {
  sourceId: string;
  sourceDisplayLabel: string;
  targetId: string;
  targetDisplayLabel: string;
  matchType: ContactDuplicateMatchType;
  matchedValue: string;
  reason: string;
}

interface BackendContact {
  id: string;
  contact_type: 'person' | 'organization' | string;
  first_name: string;
  last_name: string;
  org_name: string;
  email: string;
  phone: string;
  address: string;
  city: string;
  country: string;
  org_id: string | null;
  organization_id?: string | null;
  notes: string;
  lifecycle?: string | null;
  owner?: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
}

interface BackendContactListResponse {
  contacts: BackendContact[];
  total: number;
  page: number;
  per_page: number;
}

interface BackendContactDuplicateCandidate {
  source_id: string;
  source_display_label: string;
  target_id: string;
  target_display_label: string;
  match_type: ContactDuplicateMatchType;
  matched_value: string;
  reason: string;
}

function toNullable(value: string | null | undefined): string | null {
  if (!value) {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function mapContact(contact: BackendContact): Contact {
  const addressParts = [contact.address, contact.city, contact.country]
    .map((part) => part?.trim())
    .filter((part): part is string => Boolean(part));

  return {
    id: contact.id,
    firstName: contact.first_name,
    lastName: contact.last_name,
    email: toNullable(contact.email),
    phone: toNullable(contact.phone),
    organization: toNullable(contact.org_name),
    organizationId: toNullable(contact.organization_id),
    type: contact.contact_type === 'organization' ? 'org' : 'person',
    lifecycle: contact.lifecycle === 'lead' ? 'lead' : 'customer',
    tags: [],
    notes: toNullable(contact.notes),
    website: null,
    address: addressParts.length > 0 ? addressParts.join(', ') : null,
    createdAt: contact.created_at,
    updatedAt: contact.updated_at,
    deletedAt: contact.deleted_at,
    owner: toNullable(contact.owner),
  };
}

function mapContactDuplicateCandidate(
  candidate: BackendContactDuplicateCandidate,
): ContactDuplicateCandidate {
  return {
    sourceId: candidate.source_id,
    sourceDisplayLabel: candidate.source_display_label,
    targetId: candidate.target_id,
    targetDisplayLabel: candidate.target_display_label,
    matchType: candidate.match_type,
    matchedValue: candidate.matched_value,
    reason: candidate.reason,
  };
}

function toBackendSortKey(sortBy: ListContactsParams['sortBy']): string {
  switch (sortBy) {
    case 'createdAt':
      return 'created_at';
    case 'updatedAt':
      return 'updated_at';
    case 'name':
    default:
      return 'first_name';
  }
}

function toBackendContactType(type: ContactType | undefined): string | undefined {
  if (!type) {
    return undefined;
  }
  return type === 'org' ? 'organization' : 'person';
}

export async function createContact(data: CreateContactPayload): Promise<Contact> {
  const contact = await invoke<BackendContact>('create_contact', {
    contact_type: data.type === 'org' ? 'organization' : 'person',
    first_name: data.firstName ?? '',
    last_name: data.lastName ?? '',
    org_name: data.organization ?? '',
    email: data.email ?? '',
    phone: data.phone ?? '',
    address: data.address ?? '',
    city: '',
    country: '',
    org_id: data.organizationId ?? null,
    notes: data.notes ?? '',
    lifecycle: data.type === 'person' ? (data.lifecycle ?? 'customer') : 'customer',
    owner: data.owner ?? null,
  });

  return mapContact(contact);
}

export async function getContact(id: string): Promise<Contact> {
  const contact = await invoke<BackendContact>('get_contact', { id });
  return mapContact(contact);
}

export async function listContacts(params: ListContactsParams = {}): Promise<ContactListResponse> {
  const result = await invoke<BackendContactListResponse>('list_contacts', {
    params: {
      page: params.page ?? 1,
      per_page: params.pageSize ?? 50,
      sort_by: toBackendSortKey(params.sortBy),
      sort_dir: params.sortDir ?? 'asc',
      filter_type: toBackendContactType(params.type),
      filter_lifecycle: params.lifecycle,
      filter_owner: params.owner?.trim() ? params.owner.trim() : undefined,
      search_query: params.search?.trim() ? params.search.trim() : undefined,
      custom_field_def_id: params.customFieldDefId?.trim() || undefined,
      custom_field_query: params.customFieldQuery?.trim() || undefined,
    },
  });

  return {
    contacts: result.contacts.map(mapContact),
    total: result.total,
    page: result.page,
    pageSize: result.per_page,
  };
}

export async function updateContact(id: string, data: UpdateContactPayload): Promise<Contact> {
  const contact = await invoke<BackendContact>('update_contact', {
    id,
    contact_type: data.type ? (data.type === 'org' ? 'organization' : 'person') : undefined,
    first_name: data.firstName,
    last_name: data.lastName,
    org_name: data.organization,
    email: data.email ?? undefined,
    phone: data.phone ?? undefined,
    address: data.address ?? undefined,
    city: undefined,
    country: undefined,
    notes: data.notes ?? undefined,
    owner: data.owner ?? undefined,
    reset_owner: Object.prototype.hasOwnProperty.call(data, 'owner') && !data.owner?.trim(),
  });

  return mapContact(contact);
}

export async function deleteContact(id: string): Promise<void> {
  await invoke<void>('delete_contact', { id });
}

export async function restoreContact(id: string): Promise<Contact> {
  const contact = await invoke<BackendContact>('restore_contact', { id });
  return mapContact(contact);
}

export async function searchContacts(query: string): Promise<Contact[]> {
  const contacts = await invoke<BackendContact[]>('search_contacts', { query });
  return contacts.map(mapContact);
}

export async function listContactDuplicateCandidates(): Promise<ContactDuplicateCandidate[]> {
  const candidates = await invoke<BackendContactDuplicateCandidate[]>(
    'list_contact_duplicate_candidates',
  );
  return candidates.map(mapContactDuplicateCandidate);
}

export async function mergeContacts(sourceId: string, targetId: string): Promise<Contact> {
  const contact = await invoke<BackendContact>('merge_contacts', {
    target_id: targetId,
    source_id: sourceId,
  });
  return mapContact(contact);
}

export async function setContactLifecycle(
  id: string,
  lifecycle: ContactLifecycle,
): Promise<Contact> {
  const contact = await invoke<BackendContact>('set_contact_lifecycle', {
    id,
    lifecycle,
  });
  return mapContact(contact);
}
