/**
 * src/lib/api/links.ts — Tauri IPC wrappers for entity links.
 *
 * Links are bookmarks. 900CRM stores the URL or file path as text and does
 * not copy or upload the file.
 */

import { invoke } from '@tauri-apps/api/core';
import type { CrmEntityType } from './notes';

export type LinkEntityType = Exclude<CrmEntityType, 'activity'>;
export type EntityLinkKind = 'url' | 'path';

export interface EntityLink {
  id: string;
  entityType: LinkEntityType;
  entityId: string;
  title: string;
  kind: EntityLinkKind;
  target: string;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  deviceId: string;
}

export interface CreateEntityLinkPayload {
  entityType: LinkEntityType;
  entityId: string;
  title?: string | null;
  kind: EntityLinkKind;
  target: string;
}

interface BackendEntityLink {
  id: string;
  entity_type: LinkEntityType;
  entity_id: string;
  title: string;
  kind: string;
  target: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  device_id: string;
}

function mapLink(link: BackendEntityLink): EntityLink {
  return {
    id: link.id,
    entityType: link.entity_type,
    entityId: link.entity_id,
    title: link.title,
    kind: link.kind === 'path' ? 'path' : 'url',
    target: link.target,
    createdAt: link.created_at,
    updatedAt: link.updated_at,
    deletedAt: link.deleted_at ?? null,
    deviceId: link.device_id,
  };
}

function normalizeText(value: string): string {
  return value.trim();
}

export async function createEntityLink(data: CreateEntityLinkPayload): Promise<EntityLink> {
  const link = await invoke<BackendEntityLink>('create_entity_link', {
    entity_type: data.entityType,
    entity_id: normalizeText(data.entityId),
    title: data.title?.trim() || null,
    kind: data.kind,
    target: normalizeText(data.target),
  });
  return mapLink(link);
}

export async function listEntityLinks(
  entityType: LinkEntityType,
  entityId: string,
): Promise<EntityLink[]> {
  const links = await invoke<BackendEntityLink[]>('list_entity_links', {
    entity_type: entityType,
    entity_id: normalizeText(entityId),
  });
  return links.map(mapLink);
}

export async function updateEntityLink(
  id: string,
  data: Pick<CreateEntityLinkPayload, 'title' | 'kind' | 'target'>,
): Promise<EntityLink> {
  const link = await invoke<BackendEntityLink>('update_entity_link', {
    id: normalizeText(id),
    title: data.title?.trim() || null,
    kind: data.kind,
    target: normalizeText(data.target),
  });
  return mapLink(link);
}

export async function deleteEntityLink(id: string): Promise<void> {
  await invoke<void>('delete_entity_link', { id: normalizeText(id) });
}
