import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { globalSearch, type GlobalSearchResult } from './search';

const backendResult = {
  entity_type: 'organization' as const,
  entity_id: 'org-1',
  title: 'Clinic Partners',
  subtitle: 'clinic@example.test',
  match_field: 'name',
};

const result: GlobalSearchResult = {
  entityType: 'organization',
  entityId: 'org-1',
  title: 'Clinic Partners',
  subtitle: 'clinic@example.test',
  matchField: 'name',
};

describe('search API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps globalSearch to global_search with normalized query and limit', async () => {
    invokeMock.mockResolvedValueOnce([backendResult]);

    await expect(globalSearch(' Clinic ', 25.8)).resolves.toEqual([result]);

    expect(invokeMock).toHaveBeenCalledWith('global_search', {
      query: 'Clinic',
      limit: 25,
    });
  });

  it('omits invalid limits so the core default applies', async () => {
    invokeMock.mockResolvedValueOnce([]);

    await expect(globalSearch('Clinic', Number.NaN)).resolves.toEqual([]);

    expect(invokeMock).toHaveBeenCalledWith('global_search', {
      query: 'Clinic',
      limit: undefined,
    });
  });
});
