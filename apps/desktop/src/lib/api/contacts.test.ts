import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createContact,
  listContacts,
  listContactDuplicateCandidates,
  mergeContacts,
  restoreContact,
  setContactLifecycle,
  updateContact,
} from './contacts';

const backendContact = {
  id: 'contact-target',
  contact_type: 'person',
  first_name: 'Ada',
  last_name: 'Lovelace',
  org_name: '',
  email: 'ada@example.com',
  phone: '+15550100',
  address: '',
  city: '',
  country: '',
  org_id: null,
  notes: '',
  created_at: '2026-06-24T10:00:00Z',
  updated_at: '2026-06-24T10:00:00Z',
  deleted_at: null,
};

describe('contacts API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps contact duplicate candidate commands', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        source_id: 'contact-source',
        source_display_label: 'Ada Source',
        target_id: 'contact-target',
        target_display_label: 'Ada Target',
        match_type: 'email',
        matched_value: 'ada@example.com',
        reason: 'Same email address: ada@example.com',
      },
    ]);

    await expect(listContactDuplicateCandidates()).resolves.toEqual([
      {
        sourceId: 'contact-source',
        sourceDisplayLabel: 'Ada Source',
        targetId: 'contact-target',
        targetDisplayLabel: 'Ada Target',
        matchType: 'email',
        matchedValue: 'ada@example.com',
        reason: 'Same email address: ada@example.com',
      },
    ]);
    expect(invokeMock).toHaveBeenCalledWith('list_contact_duplicate_candidates');
  });

  it('maps merge contact source and target ids', async () => {
    invokeMock.mockResolvedValueOnce(backendContact);

    await expect(mergeContacts('contact-source', 'contact-target')).resolves.toMatchObject({
      id: 'contact-target',
      firstName: 'Ada',
      lastName: 'Lovelace',
      email: 'ada@example.com',
    });
    expect(invokeMock).toHaveBeenCalledWith('merge_contacts', {
      source_id: 'contact-source',
      target_id: 'contact-target',
    });
  });

  it('maps normalized organization id from contact list responses', async () => {
    invokeMock.mockResolvedValueOnce({
      contacts: [
        {
          ...backendContact,
          id: 'contact-linked',
          organization_id: 'organization-1',
        },
      ],
      total: 1,
      page: 1,
      per_page: 50,
    });

    await expect(listContacts()).resolves.toMatchObject({
      contacts: [
        {
          id: 'contact-linked',
          organizationId: 'organization-1',
          lifecycle: 'customer',
        },
      ],
    });
  });

  it('maps lead lifecycle and list filters', async () => {
    invokeMock.mockResolvedValueOnce({
      contacts: [
        {
          ...backendContact,
          id: 'contact-lead',
          lifecycle: 'lead',
        },
      ],
      total: 1,
      page: 1,
      per_page: 50,
    });

    await expect(listContacts({ lifecycle: 'lead' })).resolves.toMatchObject({
      contacts: [{ id: 'contact-lead', lifecycle: 'lead' }],
    });
    expect(invokeMock).toHaveBeenCalledWith('list_contacts', {
      params: expect.objectContaining({
        filter_lifecycle: 'lead',
      }),
    });
  });

  it('maps restoreContact to the dedicated command', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendContact,
      deleted_at: null,
    });

    await expect(restoreContact('contact-target')).resolves.toMatchObject({
      id: 'contact-target',
      firstName: 'Ada',
      lastName: 'Lovelace',
      email: 'ada@example.com',
      deletedAt: null,
    });
    expect(invokeMock).toHaveBeenCalledWith('restore_contact', {
      id: 'contact-target',
    });
  });

  it('maps owner on create, list filter, and clear', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendContact,
      id: 'contact-owned',
      owner: 'Samira',
    });

    await expect(
      createContact({
        firstName: 'Ada',
        lastName: 'Lovelace',
        email: 'ada@example.com',
        phone: '+15550100',
        organization: null,
        type: 'person',
        tags: [],
        notes: null,
        website: null,
        address: null,
        owner: 'Samira',
      }),
    ).resolves.toMatchObject({
      id: 'contact-owned',
      owner: 'Samira',
    });
    expect(invokeMock).toHaveBeenCalledWith(
      'create_contact',
      expect.objectContaining({ owner: 'Samira' }),
    );

    invokeMock.mockResolvedValueOnce({
      contacts: [{ ...backendContact, id: 'contact-owned', owner: 'Samira' }],
      total: 1,
      page: 1,
      per_page: 50,
    });

    await expect(listContacts({ owner: ' samira ' })).resolves.toMatchObject({
      contacts: [{ id: 'contact-owned', owner: 'Samira' }],
    });
    expect(invokeMock).toHaveBeenCalledWith('list_contacts', {
      params: expect.objectContaining({
        filter_owner: 'samira',
      }),
    });

    invokeMock.mockResolvedValueOnce({
      ...backendContact,
      owner: null,
    });

    await updateContact('contact-owned', { owner: '' });
    expect(invokeMock).toHaveBeenCalledWith(
      'update_contact',
      expect.objectContaining({
        id: 'contact-owned',
        reset_owner: true,
      }),
    );
  });

  it('maps setContactLifecycle to the dedicated command', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendContact,
      lifecycle: 'customer',
    });

    await expect(setContactLifecycle('contact-target', 'customer')).resolves.toMatchObject({
      id: 'contact-target',
      lifecycle: 'customer',
    });
    expect(invokeMock).toHaveBeenCalledWith('set_contact_lifecycle', {
      id: 'contact-target',
      lifecycle: 'customer',
    });
  });
});
