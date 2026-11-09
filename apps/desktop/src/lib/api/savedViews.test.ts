import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { createSavedView, filtersMatch, listSavedViews, updateSavedView } from './savedViews';

describe('saved views API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps create and list payloads', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-1',
      entity_type: 'contact',
      name: 'New leads',
      filters_json: '{"lifecycle":"lead"}',
      created_at: '2026-08-13T10:00:00Z',
      updated_at: '2026-08-13T10:00:00Z',
    });

    await expect(
      createSavedView('contact', 'New leads', { lifecycle: 'lead', search: '  ' }),
    ).resolves.toMatchObject({
      id: 'view-1',
      name: 'New leads',
      filters: { lifecycle: 'lead' },
    });
    expect(invokeMock).toHaveBeenCalledWith('create_saved_view', {
      entity_type: 'contact',
      name: 'New leads',
      filters_json: JSON.stringify({ lifecycle: 'lead' }),
    });

    invokeMock.mockResolvedValueOnce([
      {
        id: 'view-1',
        entity_type: 'contact',
        name: 'New leads',
        filters_json: '{"lifecycle":"lead"}',
        created_at: '2026-08-13T10:00:00Z',
        updated_at: '2026-08-13T10:00:00Z',
      },
    ]);
    await expect(listSavedViews('contact')).resolves.toEqual([
      expect.objectContaining({ id: 'view-1', filters: { lifecycle: 'lead' } }),
    ]);
  });

  it('maps update payloads', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-1',
      entity_type: 'contact',
      name: 'Active leads',
      filters_json: '{"lifecycle":"lead","search":"clinic"}',
      created_at: '2026-08-13T10:00:00Z',
      updated_at: '2026-08-16T10:00:00Z',
    });

    await expect(
      updateSavedView('view-1', ' Active leads ', { lifecycle: 'lead', search: 'clinic' }),
    ).resolves.toMatchObject({
      id: 'view-1',
      name: 'Active leads',
      filters: { lifecycle: 'lead', search: 'clinic' },
    });
    expect(invokeMock).toHaveBeenCalledWith('update_saved_view', {
      id: 'view-1',
      name: 'Active leads',
      filters_json: JSON.stringify({
        search: 'clinic',
        lifecycle: 'lead',
      }),
    });
  });

  it('compares filter snapshots without empty values', () => {
    expect(filtersMatch({ lifecycle: 'lead', search: ' ' }, { lifecycle: 'lead' })).toBe(true);
    expect(filtersMatch({ lifecycle: 'lead' }, { lifecycle: 'customer' })).toBe(false);
  });

  it('maps owner filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-owner',
      entity_type: 'contact',
      name: 'Samira queue',
      filters_json: '{"owner":"Samira"}',
      created_at: '2026-08-17T10:00:00Z',
      updated_at: '2026-08-17T10:00:00Z',
    });

    await expect(
      createSavedView('contact', 'Samira queue', { owner: ' Samira ' }),
    ).resolves.toMatchObject({
      filters: { owner: 'Samira' },
    });
    expect(invokeMock).toHaveBeenCalledWith('create_saved_view', {
      entity_type: 'contact',
      name: 'Samira queue',
      filters_json: JSON.stringify({ owner: 'Samira' }),
    });
  });

  it('maps organization country filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-org',
      entity_type: 'organization',
      name: 'Kenya accounts',
      filters_json: '{"country":"Kenya","search":"clinic"}',
      created_at: '2026-08-14T10:00:00Z',
      updated_at: '2026-08-14T10:00:00Z',
    });

    await expect(
      createSavedView('organization', 'Kenya accounts', { country: 'Kenya', search: 'clinic' }),
    ).resolves.toMatchObject({
      entityType: 'organization',
      filters: { country: 'Kenya', search: 'clinic' },
    });
  });

  it('maps deal search and custom-field filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-deal',
      entity_type: 'deal',
      name: 'Clinic rollouts',
      filters_json: '{"search":"clinic","custom_field_query":"solar"}',
      created_at: '2026-08-14T10:00:00Z',
      updated_at: '2026-08-14T10:00:00Z',
    });

    await expect(
      createSavedView('deal', 'Clinic rollouts', {
        search: 'clinic',
        customFieldQuery: 'solar',
      }),
    ).resolves.toMatchObject({
      entityType: 'deal',
      filters: { search: 'clinic', customFieldQuery: 'solar' },
    });
    expect(invokeMock).toHaveBeenCalledWith('create_saved_view', {
      entity_type: 'deal',
      name: 'Clinic rollouts',
      filters_json: JSON.stringify({
        search: 'clinic',
        custom_field_query: 'solar',
      }),
    });
  });

  it('maps deal attention filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-attention',
      entity_type: 'deal',
      name: 'Needs follow-up',
      filters_json: '{"attention":"needsFollowUp"}',
      created_at: '2026-08-14T10:00:00Z',
      updated_at: '2026-08-14T10:00:00Z',
    });

    await expect(
      createSavedView('deal', 'Needs follow-up', { attention: 'needsFollowUp' }),
    ).resolves.toMatchObject({
      entityType: 'deal',
      filters: { attention: 'needsFollowUp' },
    });
  });

  it('maps activity type, status, and bucket filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-activity',
      entity_type: 'activity',
      name: 'Overdue calls',
      filters_json: '{"type":"call","status":"overdue","bucket":"today"}',
      created_at: '2026-08-14T10:00:00Z',
      updated_at: '2026-08-14T10:00:00Z',
    });

    await expect(
      createSavedView('activity', 'Overdue calls', {
        type: 'call',
        status: 'overdue',
        bucket: 'today',
      }),
    ).resolves.toMatchObject({
      entityType: 'activity',
      filters: { type: 'call', status: 'overdue', bucket: 'today' },
    });
    expect(invokeMock).toHaveBeenCalledWith('create_saved_view', {
      entity_type: 'activity',
      name: 'Overdue calls',
      filters_json: JSON.stringify({
        type: 'call',
        status: 'overdue',
        bucket: 'today',
      }),
    });
  });

  it('maps report focus filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-report',
      entity_type: 'report',
      name: 'Stale deals',
      filters_json: '{"focus":"stale"}',
      created_at: '2026-08-15T10:00:00Z',
      updated_at: '2026-08-15T10:00:00Z',
    });

    await expect(
      createSavedView('report', 'Stale deals', { focus: 'stale' }),
    ).resolves.toMatchObject({
      entityType: 'report',
      filters: { focus: 'stale' },
    });
    expect(invokeMock).toHaveBeenCalledWith('create_saved_view', {
      entity_type: 'report',
      name: 'Stale deals',
      filters_json: JSON.stringify({ focus: 'stale' }),
    });
  });
});
