import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  addDealContact,
  createDeal,
  linkDealToOrganization,
  listDealContacts,
  listDeals,
  removeDealContact,
  updateDeal,
} from './deals';

const backendDeal = {
  id: 'deal-1',
  title: 'Clinic expansion',
  value: 12000,
  currency: 'USD',
  stage: 'Proposal',
  probability: 50,
  expected_close: '2026-07-15',
  contact_id: 'contact-1',
  organization_id: 'org-1',
  notes: 'Regional rollout',
  created_at: '2026-06-24T08:00:00Z',
  updated_at: '2026-06-24T09:00:00Z',
};

const backendDealContact = {
  id: 'deal-contact-1',
  deal_id: 'deal-1',
  contact_id: 'contact-1',
  role: 'Decision maker',
  is_primary: true,
  created_at: '2026-06-24T08:30:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

describe('deal API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createDeal organization_id and backend organization_id', async () => {
    invokeMock.mockResolvedValueOnce(backendDeal);

    await expect(
      createDeal({
        name: 'Clinic expansion',
        value: 12000,
        currency: 'usd',
        stage: 'proposal',
        probability: 50,
        expectedCloseDate: '2026-07-15',
        contactId: 'contact-1',
        organizationId: ' org-1 ',
        description: 'Regional rollout',
        tags: [],
      }),
    ).resolves.toMatchObject({
      id: 'deal-1',
      name: 'Clinic expansion',
      organizationId: 'org-1',
    });

    expect(invokeMock).toHaveBeenCalledWith('create_deal', {
      title: 'Clinic expansion',
      value: 12000,
      currency: 'usd',
      stage: 'Proposal',
      probability: 50,
      expected_close: '2026-07-15',
      contact_id: 'contact-1',
      organization_id: 'org-1',
      notes: 'Regional rollout',
      owner: null,
    });
  });

  it('maps deal owner on create and update', async () => {
    invokeMock.mockResolvedValueOnce({ ...backendDeal, owner: 'Samira' });

    await expect(
      createDeal({
        name: 'Clinic expansion',
        value: 12000,
        currency: 'USD',
        stage: 'proposal',
        probability: 50,
        expectedCloseDate: '2026-07-15',
        contactId: 'contact-1',
        organizationId: 'org-1',
        description: 'Regional rollout',
        tags: [],
        owner: 'Samira',
      }),
    ).resolves.toMatchObject({ owner: 'Samira' });
    expect(invokeMock).toHaveBeenCalledWith(
      'create_deal',
      expect.objectContaining({ owner: 'Samira' }),
    );

    invokeMock.mockResolvedValueOnce({ ...backendDeal, owner: null });
    await updateDeal('deal-1', { owner: '' });
    expect(invokeMock).toHaveBeenCalledWith(
      'update_deal',
      expect.objectContaining({
        id: 'deal-1',
        reset_owner: true,
      }),
    );
  });

  it('maps updateDeal organization_id only when supplied', async () => {
    invokeMock.mockResolvedValueOnce(backendDeal);

    await updateDeal('deal-1', { name: 'Clinic expansion' });

    expect(invokeMock).toHaveBeenCalledWith('update_deal', {
      id: 'deal-1',
      title: 'Clinic expansion',
      value: undefined,
      currency: undefined,
      stage: undefined,
      probability: undefined,
      notes: undefined,
    });
    expect(invokeMock.mock.calls[0][1]).not.toHaveProperty('expected_close');
    expect(invokeMock.mock.calls[0][1]).not.toHaveProperty('contact_id');

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({ ...backendDeal, organization_id: null });

    await updateDeal('deal-1', { organizationId: '   ' });

    expect(invokeMock).toHaveBeenCalledWith(
      'update_deal',
      expect.objectContaining({
        id: 'deal-1',
        reset_organization_id: true,
      }),
    );
  });

  it('omits absent nullable update fields and sends explicit reset flags intentionally', async () => {
    invokeMock.mockResolvedValueOnce(backendDeal);

    await updateDeal('deal-1', { name: 'Clinic expansion' });

    let args = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    expect(args).not.toHaveProperty('expected_close');
    expect(args).not.toHaveProperty('reset_expected_close');
    expect(args).not.toHaveProperty('contact_id');
    expect(args).not.toHaveProperty('reset_contact_id');
    expect(args).not.toHaveProperty('organization_id');
    expect(args).not.toHaveProperty('reset_organization_id');

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({
      ...backendDeal,
      expected_close: null,
      contact_id: null,
      organization_id: null,
    });

    await updateDeal('deal-1', {
      expectedCloseDate: null,
      contactId: null,
      organizationId: null,
    });

    args = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    expect(args).toMatchObject({
      id: 'deal-1',
      reset_expected_close: true,
      reset_contact_id: true,
      reset_organization_id: true,
    });
    expect(args).not.toHaveProperty('expected_close');
    expect(args).not.toHaveProperty('contact_id');
    expect(args).not.toHaveProperty('organization_id');

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({
      ...backendDeal,
      expected_close: null,
      contact_id: null,
      organization_id: null,
    });

    await updateDeal('deal-1', {
      expectedCloseDate: '   ',
      contactId: '   ',
      organizationId: '   ',
    });

    args = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    expect(args).toMatchObject({
      id: 'deal-1',
      reset_expected_close: true,
      reset_contact_id: true,
      reset_organization_id: true,
    });
    expect(args).not.toHaveProperty('expected_close');
    expect(args).not.toHaveProperty('contact_id');
    expect(args).not.toHaveProperty('organization_id');
  });

  it('maps nullable update field sets without reset flags', async () => {
    invokeMock.mockResolvedValueOnce(backendDeal);

    await updateDeal('deal-1', {
      expectedCloseDate: ' 2026-08-01 ',
      contactId: ' contact-2 ',
      organizationId: ' org-2 ',
    });

    const args = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    expect(args).toMatchObject({
      id: 'deal-1',
      expected_close: '2026-08-01',
      contact_id: 'contact-2',
      organization_id: 'org-2',
    });
    expect(args).not.toHaveProperty('reset_expected_close');
    expect(args).not.toHaveProperty('reset_contact_id');
    expect(args).not.toHaveProperty('reset_organization_id');
  });

  it('maps linkDealToOrganization to link_deal_to_organization', async () => {
    invokeMock.mockResolvedValueOnce(backendDeal);

    await expect(linkDealToOrganization('deal-1', ' org-1 ')).resolves.toMatchObject({
      id: 'deal-1',
      organizationId: 'org-1',
    });

    expect(invokeMock).toHaveBeenCalledWith('link_deal_to_organization', {
      deal_id: 'deal-1',
      organization_id: 'org-1',
    });
  });

  it('always sends a windowed list_deals limit', async () => {
    invokeMock.mockResolvedValueOnce([]);

    await listDeals();

    expect(invokeMock).toHaveBeenCalledWith('list_deals', {
      limit: 200,
      offset: 0,
    });
  });

  it('filters listDeals by organization id on the frontend', async () => {
    invokeMock.mockResolvedValueOnce([
      { ...backendDeal, id: 'deal-1', organization_id: 'org-1' },
      { ...backendDeal, id: 'deal-2', organization_id: 'org-2' },
      { ...backendDeal, id: 'deal-3', organization_id: null },
    ]);

    await expect(listDeals({ organizationId: 'org-1' })).resolves.toEqual([
      expect.objectContaining({ id: 'deal-1', organizationId: 'org-1' }),
    ]);
  });

  it('maps add/list/remove deal contact commands', async () => {
    invokeMock.mockResolvedValueOnce(backendDealContact);

    await expect(
      addDealContact('deal-1', 'contact-1', {
        role: ' Decision maker ',
        isPrimary: true,
      }),
    ).resolves.toEqual({
      id: 'deal-contact-1',
      dealId: 'deal-1',
      contactId: 'contact-1',
      role: 'Decision maker',
      isPrimary: true,
      createdAt: '2026-06-24T08:30:00Z',
      deletedAt: null,
      deviceId: 'device-1',
    });

    expect(invokeMock).toHaveBeenCalledWith('add_deal_contact', {
      deal_id: 'deal-1',
      contact_id: 'contact-1',
      role: 'Decision maker',
      is_primary: true,
    });

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce([backendDealContact]);

    await expect(listDealContacts('deal-1')).resolves.toHaveLength(1);
    expect(invokeMock).toHaveBeenCalledWith('list_deal_contacts', {
      deal_id: 'deal-1',
    });

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({ ...backendDealContact, deleted_at: '2026-06-24T10:00:00Z' });

    await expect(removeDealContact('deal-1', 'contact-1')).resolves.toMatchObject({
      deletedAt: '2026-06-24T10:00:00Z',
    });
    expect(invokeMock).toHaveBeenCalledWith('remove_deal_contact', {
      deal_id: 'deal-1',
      contact_id: 'contact-1',
    });
  });
});
