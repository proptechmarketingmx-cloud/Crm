import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { createEntityLink, listEntityLinks } from './links';

const backendLink = {
  id: 'link-1',
  entity_type: 'contact',
  entity_id: 'contact-1',
  title: 'Quote sheet',
  kind: 'url',
  target: 'https://northstar.example/quote',
  created_at: '2026-08-13T10:00:00Z',
  updated_at: '2026-08-13T10:00:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

describe('entity links API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps create and list payloads', async () => {
    invokeMock.mockResolvedValueOnce(backendLink);
    await expect(
      createEntityLink({
        entityType: 'contact',
        entityId: 'contact-1',
        title: 'Quote sheet',
        kind: 'url',
        target: 'https://northstar.example/quote',
      }),
    ).resolves.toMatchObject({
      id: 'link-1',
      entityType: 'contact',
      kind: 'url',
      target: 'https://northstar.example/quote',
    });

    invokeMock.mockResolvedValueOnce([backendLink]);
    await expect(listEntityLinks('contact', 'contact-1')).resolves.toEqual([
      expect.objectContaining({ id: 'link-1', title: 'Quote sheet' }),
    ]);
    expect(invokeMock).toHaveBeenLastCalledWith('list_entity_links', {
      entity_type: 'contact',
      entity_id: 'contact-1',
    });
  });
});
