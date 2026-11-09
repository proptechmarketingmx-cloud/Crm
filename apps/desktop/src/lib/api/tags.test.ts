import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  applyTagToEntity,
  createTag,
  deleteTag,
  listTags,
  listTagsForEntity,
  removeTagFromEntity,
  updateTag,
  type Tag,
} from './tags';

const backendTag = {
  id: 'tag-1',
  name: 'VIP',
  color: '#ef4444',
  created_at: '2026-06-24T08:00:00Z',
  updated_at: '2026-06-24T08:00:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

const tag: Tag = {
  id: 'tag-1',
  name: 'VIP',
  color: '#ef4444',
  createdAt: '2026-06-24T08:00:00Z',
  updatedAt: '2026-06-24T08:00:00Z',
  deletedAt: null,
  deviceId: 'device-1',
};

describe('tags API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createTag to create_tag with normalized nullable color', async () => {
    invokeMock.mockResolvedValueOnce(backendTag);

    await expect(createTag({ name: ' VIP ', color: ' #ef4444 ' })).resolves.toEqual(tag);

    expect(invokeMock).toHaveBeenCalledWith('create_tag', {
      name: 'VIP',
      color: '#ef4444',
    });
  });

  it('maps blank createTag color to null for backend defaulting', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendTag,
      color: '#6366f1',
    });

    await createTag({ name: 'Warm', color: '   ' });

    expect(invokeMock).toHaveBeenCalledWith('create_tag', {
      name: 'Warm',
      color: null,
    });
  });

  it('maps listTags to list_tags and camel-cases response fields', async () => {
    invokeMock.mockResolvedValueOnce([backendTag]);

    await expect(listTags()).resolves.toEqual([tag]);

    expect(invokeMock).toHaveBeenCalledWith('list_tags');
  });

  it('maps updateTag omitted color as omitted', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendTag,
      name: 'Priority',
      updated_at: '2026-06-24T09:00:00Z',
    });

    await updateTag(' tag-1 ', {
      name: ' Priority ',
    });

    expect(invokeMock).toHaveBeenCalledWith('update_tag', {
      id: 'tag-1',
      name: 'Priority',
    });
  });

  it('maps updateTag explicit null color to reset request', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendTag,
      color: '#6366f1',
      updated_at: '2026-06-24T09:00:00Z',
    });

    await updateTag('tag-1', {
      color: null,
    });

    expect(invokeMock).toHaveBeenCalledWith('update_tag', {
      id: 'tag-1',
      reset_color: true,
    });
  });

  it('maps updateTag nonblank color without reset flag', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendTag,
      color: '#0f766e',
      updated_at: '2026-06-24T09:00:00Z',
    });

    await updateTag('tag-1', {
      color: ' #0f766e ',
    });

    expect(invokeMock).toHaveBeenCalledWith('update_tag', {
      id: 'tag-1',
      color: '#0f766e',
    });
  });

  it('maps updateTag blank color to reset request', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendTag,
      name: 'Priority',
      color: '#6366f1',
      updated_at: '2026-06-24T09:00:00Z',
    });

    await updateTag(' tag-1 ', {
      name: ' Priority ',
      color: '   ',
    });

    expect(invokeMock).toHaveBeenCalledWith('update_tag', {
      id: 'tag-1',
      name: 'Priority',
      reset_color: true,
    });
  });

  it('maps deleteTag to delete_tag', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteTag(' tag-1 ')).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenCalledWith('delete_tag', { id: 'tag-1' });
  });

  it('maps applyTagToEntity to apply_tag_to_entity', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await applyTagToEntity('organization', ' org-1 ', ' tag-1 ');

    expect(invokeMock).toHaveBeenCalledWith('apply_tag_to_entity', {
      entity_type: 'organization',
      entity_id: 'org-1',
      tag_id: 'tag-1',
    });
  });

  it('maps removeTagFromEntity to remove_tag_from_entity', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await removeTagFromEntity('deal', ' deal-1 ', ' tag-1 ');

    expect(invokeMock).toHaveBeenCalledWith('remove_tag_from_entity', {
      entity_type: 'deal',
      entity_id: 'deal-1',
      tag_id: 'tag-1',
    });
  });

  it('maps listTagsForEntity to list_tags_for_entity', async () => {
    invokeMock.mockResolvedValueOnce([backendTag]);

    await expect(listTagsForEntity('activity', ' activity-1 ')).resolves.toEqual([tag]);

    expect(invokeMock).toHaveBeenCalledWith('list_tags_for_entity', {
      entity_type: 'activity',
      entity_id: 'activity-1',
    });
  });
});
