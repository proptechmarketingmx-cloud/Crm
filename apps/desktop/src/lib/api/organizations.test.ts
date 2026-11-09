import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createOrganization,
  deleteOrganization,
  linkContactToOrganization,
  listOrganizations,
  updateOrganization,
  type Organization,
} from './organizations';

const backendOrganization = {
  id: 'org-1',
  name: 'Acme Foods',
  email: 'hello@acme.test',
  phone: null,
  website: 'https://acme.test',
  address_line1: '12 Market Road',
  address_line2: null,
  city: 'Lagos',
  region: 'Lagos',
  country: 'Nigeria',
  postal_code: '100001',
  source: 'desktop',
  description: 'Regional distributor',
  created_at: '2026-06-24T08:00:00Z',
  updated_at: '2026-06-24T08:00:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

const organization: Organization = {
  id: 'org-1',
  name: 'Acme Foods',
  email: 'hello@acme.test',
  phone: null,
  website: 'https://acme.test',
  addressLine1: '12 Market Road',
  addressLine2: null,
  city: 'Lagos',
  region: 'Lagos',
  country: 'Nigeria',
  postalCode: '100001',
  source: 'desktop',
  description: 'Regional distributor',
  owner: null,
  createdAt: '2026-06-24T08:00:00Z',
  updatedAt: '2026-06-24T08:00:00Z',
  deletedAt: null,
  deviceId: 'device-1',
};

describe('organization API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createOrganization to create_organization with normalized nullable fields', async () => {
    invokeMock.mockResolvedValueOnce(backendOrganization);

    await expect(
      createOrganization({
        name: '  Acme Foods  ',
        email: ' hello@acme.test ',
        phone: '',
        website: null,
        addressLine1: ' 12 Market Road ',
        city: 'Lagos',
        country: ' Nigeria ',
      }),
    ).resolves.toEqual(organization);

    expect(invokeMock).toHaveBeenCalledWith('create_organization', {
      name: 'Acme Foods',
      email: 'hello@acme.test',
      phone: null,
      website: null,
      address_line1: '12 Market Road',
      address_line2: null,
      city: 'Lagos',
      region: null,
      country: 'Nigeria',
      postal_code: null,
      description: null,
      owner: null,
    });
  });

  it('maps listOrganizations to list_organizations and camel-cases response fields', async () => {
    invokeMock.mockResolvedValueOnce([backendOrganization]);

    await expect(listOrganizations()).resolves.toEqual([organization]);

    expect(invokeMock).toHaveBeenCalledWith('list_organizations');
  });

  it('maps updateOrganization omitted fields as omitted and blank/null fields as explicit clears', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendOrganization,
      email: null,
      phone: null,
      website: 'https://new.acme.test',
      updated_at: '2026-06-24T09:00:00Z',
    });

    await updateOrganization('org-1', {
      email: null,
      phone: '   ',
      website: ' https://new.acme.test ',
    });

    expect(invokeMock).toHaveBeenCalledWith('update_organization', {
      id: 'org-1',
      email: null,
      phone: null,
      website: 'https://new.acme.test',
    });
  });

  it('maps deleteOrganization to delete_organization', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteOrganization('org-1')).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenCalledWith('delete_organization', { id: 'org-1' });
  });

  it('maps linkContactToOrganization to link_contact_to_organization', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'contact-1',
      first_name: 'Amina',
      last_name: 'Diallo',
      org_name: 'Acme Foods',
      organization_id: 'org-1',
      updated_at: '2026-06-24T09:10:00Z',
    });

    await expect(linkContactToOrganization('contact-1', ' org-1 ')).resolves.toEqual({
      id: 'contact-1',
      firstName: 'Amina',
      lastName: 'Diallo',
      organization: 'Acme Foods',
      organizationId: 'org-1',
      updatedAt: '2026-06-24T09:10:00Z',
    });

    expect(invokeMock).toHaveBeenCalledWith('link_contact_to_organization', {
      contact_id: 'contact-1',
      organization_id: 'org-1',
    });
  });

  it('maps blank link organization IDs to explicit unlink null', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'contact-1',
      first_name: 'Amina',
      last_name: 'Diallo',
      org_name: '',
      organization_id: null,
      updated_at: '2026-06-24T09:15:00Z',
    });

    await linkContactToOrganization('contact-1', '   ');

    expect(invokeMock).toHaveBeenCalledWith('link_contact_to_organization', {
      contact_id: 'contact-1',
      organization_id: null,
    });
  });
});
