/**
 * src/lib/api/tags.ts - Tauri IPC wrappers for generic tag commands.
 */

import { invoke } from '@tauri-apps/api/core';
import type { CrmEntityType } from './notes';

export interface Tag {
  id: string;
  name: string;
  color: string;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  deviceId: string;
}

export interface CreateTagPayload {
  name: string;
  color?: string | null;
}

export type UpdateTagPayload = Partial<CreateTagPayload>;

interface BackendTag {
  id: string;
  name: string;
  color: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  device_id: string;
}

function mapTag(tag: BackendTag): Tag {
  return {
    id: tag.id,
    name: tag.name,
    color: tag.color,
    createdAt: tag.created_at,
    updatedAt: tag.updated_at,
    deletedAt: tag.deleted_at ?? null,
    deviceId: tag.device_id,
  };
}

function normalizeText(value: string): string {
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

export async function createTag(data: CreateTagPayload): Promise<Tag> {
  const tag = await invoke<BackendTag>('create_tag', {
    name: normalizeText(data.name),
    color: normalizeNullable(data.color),
  });

  return mapTag(tag);
}

export async function getTag(id: string): Promise<Tag> {
  const tag = await invoke<BackendTag>('get_tag', { id: normalizeText(id) });
  return mapTag(tag);
}

export async function listTags(): Promise<Tag[]> {
  const tags = await invoke<BackendTag[]>('list_tags');
  return tags.map(mapTag);
}

export async function updateTag(id: string, data: UpdateTagPayload): Promise<Tag> {
  const args: {
    id: string;
    name?: string;
    color?: string;
    reset_color?: boolean;
  } = { id: normalizeText(id) };

  if (hasOwn(data, 'name')) {
    args.name = normalizeText(String(data.name ?? ''));
  }

  if (hasOwn(data, 'color') && data.color !== undefined) {
    const color = normalizeNullable(data.color);
    if (color === null) {
      args.reset_color = true;
    } else {
      args.color = color;
    }
  }

  const tag = await invoke<BackendTag>('update_tag', args);
  return mapTag(tag);
}

export async function deleteTag(id: string): Promise<void> {
  await invoke<void>('delete_tag', { id: normalizeText(id) });
}

export async function applyTagToEntity(
  entityType: CrmEntityType,
  entityId: string,
  tagId: string,
): Promise<void> {
  await invoke<void>('apply_tag_to_entity', {
    entity_type: entityType,
    entity_id: normalizeText(entityId),
    tag_id: normalizeText(tagId),
  });
}

export async function removeTagFromEntity(
  entityType: CrmEntityType,
  entityId: string,
  tagId: string,
): Promise<void> {
  await invoke<void>('remove_tag_from_entity', {
    entity_type: entityType,
    entity_id: normalizeText(entityId),
    tag_id: normalizeText(tagId),
  });
}

export async function listTagsForEntity(
  entityType: CrmEntityType,
  entityId: string,
): Promise<Tag[]> {
  const tags = await invoke<BackendTag[]>('list_tags_for_entity', {
    entity_type: entityType,
    entity_id: normalizeText(entityId),
  });

  return tags.map(mapTag);
}
