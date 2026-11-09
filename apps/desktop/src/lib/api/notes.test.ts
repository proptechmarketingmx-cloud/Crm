import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createNote,
  deleteNote,
  getNote,
  listNotesForEntity,
  updateNote,
  type Note,
} from './notes';

const backendNote = {
  id: 'note-1',
  content: 'Discuss onboarding',
  entity_type: 'contact' as const,
  entity_id: 'contact-1',
  created_at: '2026-06-24T08:00:00Z',
  updated_at: '2026-06-24T08:00:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

const note: Note = {
  id: 'note-1',
  content: 'Discuss onboarding',
  entityType: 'contact',
  entityId: 'contact-1',
  createdAt: '2026-06-24T08:00:00Z',
  updatedAt: '2026-06-24T08:00:00Z',
  deletedAt: null,
  deviceId: 'device-1',
};

describe('notes API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createNote to create_note with normalized entity id and content', async () => {
    invokeMock.mockResolvedValueOnce(backendNote);

    await expect(
      createNote({
        entityType: 'contact',
        entityId: ' contact-1 ',
        content: ' Discuss onboarding ',
      }),
    ).resolves.toEqual(note);

    expect(invokeMock).toHaveBeenCalledWith('create_note', {
      entity_type: 'contact',
      entity_id: 'contact-1',
      content: 'Discuss onboarding',
    });
  });

  it('maps getNote to get_note and camel-cases response fields', async () => {
    invokeMock.mockResolvedValueOnce(backendNote);

    await expect(getNote(' note-1 ')).resolves.toEqual(note);

    expect(invokeMock).toHaveBeenCalledWith('get_note', { id: 'note-1' });
  });

  it('maps listNotesForEntity to list_notes_for_entity', async () => {
    invokeMock.mockResolvedValueOnce([backendNote]);

    await expect(listNotesForEntity('contact', ' contact-1 ')).resolves.toEqual([note]);

    expect(invokeMock).toHaveBeenCalledWith('list_notes_for_entity', {
      entity_type: 'contact',
      entity_id: 'contact-1',
    });
  });

  it('maps updateNote to update_note', async () => {
    invokeMock.mockResolvedValueOnce({
      ...backendNote,
      content: 'Updated note',
      updated_at: '2026-06-24T09:00:00Z',
    });

    await updateNote(' note-1 ', ' Updated note ');

    expect(invokeMock).toHaveBeenCalledWith('update_note', {
      id: 'note-1',
      content: 'Updated note',
    });
  });

  it('maps deleteNote to delete_note', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteNote(' note-1 ')).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenCalledWith('delete_note', { id: 'note-1' });
  });
});
