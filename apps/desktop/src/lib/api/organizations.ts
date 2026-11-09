/**
 * src/lib/api/organizations.ts - Tauri IPC wrappers for organization commands.
 */

import { invoke } from '@tauri-apps/api/core';

export interface Organization {
  id: string;
  name: string;
  email: string | null;
  phone: string | null;
  website: string | null;
  addressLine1: string | null;
  addressLine2: string | null;
  city: string | null;
  region: string | null;
  country: string | null;
  postalCode: string | null;
  source: string | null;
  description: string | null;
  owner?: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  deviceId: string;
}

export interface CreateOrganizationPayload {
  name: string;
  email?: string | null;
  phone?: string | null;
  website?: string | null;
  addressLine1?: string | null;
  addressLine2?: string | null;
  city?: string | null;
  region?: string | null;
  country?: string | null;
  postalCode?: string | null;
  description?: string | null;
  owner?: string | null;
}

export type UpdateOrganizationPayload = Partial<CreateOrganizationPayload>;

export interface LinkedOrganizationContact {
  id: string;
  firstName: string;
  lastName: string;
  organization: string | null;
  organizationId: string | null;
  updatedAt: string;
}

interface BackendOrganization {
  id: string;
  name: string;
  email: string | null;
  phone: string | null;
  website: string | null;
  address_line1: string | null;
  address_line2: string | null;
  city: string | null;
  region: string | null;
  country: string | null;
  postal_code: string | null;
  source: string | null;
  description: string | null;
  owner?: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  device_id: string;
}

interface BackendLinkedContact {
  id: string;
  first_name: string;
  last_name: string;
  org_name: string;
  organization_id?: string | null;
  updated_at: string;
}

type NullableOrganizationField = Exclude<keyof UpdateOrganizationPayload, 'name' | 'owner'>;

const nullableUpdateFieldMap: Record<NullableOrganizationField, string> = {
  email: 'email',
  phone: 'phone',
  website: 'website',
  addressLine1: 'address_line1',
  addressLine2: 'address_line2',
  city: 'city',
  region: 'region',
  country: 'country',
  postalCode: 'postal_code',
  description: 'description',
};

function normalizeRequired(value: string): string {
  return value.trim();
}

function normalizeNullable(value: string | null | undefined): string | null {
  if (value == null) {
    return null;
  }

  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function hasOwn<T extends object, K extends PropertyKey>(
  object: T,
  key: K,
): object is T & Record<K, unknown> {
  return Object.prototype.hasOwnProperty.call(object, key);
}

function mapOrganization(organization: BackendOrganization): Organization {
  return {
    id: organization.id,
    name: organization.name,
    email: organization.email ?? null,
    phone: organization.phone ?? null,
    website: organization.website ?? null,
    addressLine1: organization.address_line1 ?? null,
    addressLine2: organization.address_line2 ?? null,
    city: organization.city ?? null,
    region: organization.region ?? null,
    country: organization.country ?? null,
    postalCode: organization.postal_code ?? null,
    source: organization.source ?? null,
    description: organization.description ?? null,
    owner: organization.owner?.trim() ? organization.owner.trim() : null,
    createdAt: organization.created_at,
    updatedAt: organization.updated_at,
    deletedAt: organization.deleted_at ?? null,
    deviceId: organization.device_id,
  };
}

function mapLinkedContact(contact: BackendLinkedContact): LinkedOrganizationContact {
  const organization = normalizeNullable(contact.org_name);

  return {
    id: contact.id,
    firstName: contact.first_name,
    lastName: contact.last_name,
    organization,
    organizationId: contact.organization_id ?? null,
    updatedAt: contact.updated_at,
  };
}

function createOrganizationArgs(data: CreateOrganizationPayload) {
  return {
    name: normalizeRequired(data.name),
    email: normalizeNullable(data.email),
    phone: normalizeNullable(data.phone),
    website: normalizeNullable(data.website),
    address_line1: normalizeNullable(data.addressLine1),
    address_line2: normalizeNullable(data.addressLine2),
    city: normalizeNullable(data.city),
    region: normalizeNullable(data.region),
    country: normalizeNullable(data.country),
    postal_code: normalizeNullable(data.postalCode),
    description: normalizeNullable(data.description),
    owner: normalizeNullable(data.owner),
  };
}

function updateOrganizationArgs(id: string, data: UpdateOrganizationPayload) {
  const args: Record<string, string | null> = { id };

  if (hasOwn(data, 'name')) {
    args.name = normalizeRequired(String(data.name ?? ''));
  }

  for (const [frontendKey, backendKey] of Object.entries(nullableUpdateFieldMap) as [
    NullableOrganizationField,
    string,
  ][]) {
    if (hasOwn(data, frontendKey)) {
      args[backendKey] = normalizeNullable(data[frontendKey]);
    }
  }

  if (hasOwn(data, 'owner')) {
    args.owner = normalizeNullable(data.owner);
    if (!args.owner) {
      (args as Record<string, string | null | boolean>).reset_owner = true;
    }
  }

  return args;
}

export async function createOrganization(data: CreateOrganizationPayload): Promise<Organization> {
  const organization = await invoke<BackendOrganization>(
    'create_organization',
    createOrganizationArgs(data),
  );

  return mapOrganization(organization);
}

export async function getOrganization(id: string): Promise<Organization> {
  const organization = await invoke<BackendOrganization>('get_organization', { id });
  return mapOrganization(organization);
}

export async function listOrganizations(): Promise<Organization[]> {
  const organizations = await invoke<BackendOrganization[]>('list_organizations');
  return organizations.map(mapOrganization);
}

export async function updateOrganization(
  id: string,
  data: UpdateOrganizationPayload,
): Promise<Organization> {
  const organization = await invoke<BackendOrganization>(
    'update_organization',
    updateOrganizationArgs(id, data),
  );

  return mapOrganization(organization);
}

export async function deleteOrganization(id: string): Promise<void> {
  await invoke<void>('delete_organization', { id });
}

export async function linkContactToOrganization(
  contactId: string,
  organizationId: string | null,
): Promise<LinkedOrganizationContact> {
  const contact = await invoke<BackendLinkedContact>('link_contact_to_organization', {
    contact_id: contactId,
    organization_id: normalizeNullable(organizationId),
  });

  return mapLinkedContact(contact);
}
