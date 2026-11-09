/**
 * src/lib/api/notes.ts - Tauri IPC wrappers for generic note commands.
 */

import { invoke } from '@tauri-apps/api/core';

export type CrmEntityType = 'contact' | 'organization' | 'deal' | 'activity';

export interface Note {
  id: string;
  content: string;
  entityType: CrmEntityType;
  entityId: string;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  deviceId: string;
}

export interface CreateNotePayload {
  entityType: CrmEntityType;
  entityId: string;
  content: string;
}

interface BackendNote {
  id: string;
  content: string;
  entity_type: CrmEntityType;
  entity_id: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  device_id: string;
}

function mapNote(note: BackendNote): Note {
  return {
    id: note.id,
    content: note.content,
    entityType: note.entity_type,
    entityId: note.entity_id,
    createdAt: note.created_at,
    updatedAt: note.updated_at,
    deletedAt: note.deleted_at ?? null,
    deviceId: note.device_id,
  };
}

function normalizeText(value: string): string {
  return value.trim();
}

export async function createNote(data: CreateNotePayload): Promise<Note> {
  const note = await invoke<BackendNote>('create_note', {
    entity_type: data.entityType,
    entity_id: normalizeText(data.entityId),
    content: normalizeText(data.content),
  });

  return mapNote(note);
}

export async function getNote(id: string): Promise<Note> {
  const note = await invoke<BackendNote>('get_note', { id: normalizeText(id) });
  return mapNote(note);
}

export async function listNotesForEntity(
  entityType: CrmEntityType,
  entityId: string,
): Promise<Note[]> {
  const notes = await invoke<BackendNote[]>('list_notes_for_entity', {
    entity_type: entityType,
    entity_id: normalizeText(entityId),
  });

  return notes.map(mapNote);
}

export async function updateNote(id: string, content: string): Promise<Note> {
  const note = await invoke<BackendNote>('update_note', {
    id: normalizeText(id),
    content: normalizeText(content),
  });

  return mapNote(note);
}

export async function deleteNote(id: string): Promise<void> {
  await invoke<void>('delete_note', { id: normalizeText(id) });
}
