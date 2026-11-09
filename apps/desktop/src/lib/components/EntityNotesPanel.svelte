<script lang="ts">
  import { t } from '$lib/i18n';
  import {
    createNote,
    deleteNote,
    listNotesForEntity,
    updateNote,
    type CrmEntityType,
    type Note,
  } from '$lib/api/notes';
  import { formatRelativeTime } from '$lib/utils/formatters';

  let {
    entityType,
    entityId,
  }: {
    entityType: CrmEntityType;
    entityId: string;
  } = $props();

  let notes = $state<Note[]>([]);
  let newContent = $state('');
  let editingNoteId = $state<string | null>(null);
  let editContent = $state('');
  let isLoading = $state(false);
  let isCreating = $state(false);
  let savingNoteId = $state<string | null>(null);
  let deletingNoteId = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let loadToken = 0;

  const canCreate = $derived(
    newContent.trim().length > 0 && !isCreating && entityId.trim().length > 0,
  );

  $effect(() => {
    const normalizedEntityId = entityId.trim();
    const normalizedEntityType = entityType;

    if (!normalizedEntityId) {
      notes = [];
      isLoading = false;
      errorMessage = null;
      return;
    }

    void loadNotes(normalizedEntityType, normalizedEntityId);
  });

  function sortNotes(items: Note[]): Note[] {
    return [...items].sort((a, b) => {
      const createdDiff = Date.parse(b.createdAt) - Date.parse(a.createdAt);
      if (createdDiff !== 0) return createdDiff;
      return Date.parse(b.updatedAt) - Date.parse(a.updatedAt);
    });
  }

  function messageFromError(error: unknown, fallback: string): string {
    if (typeof error === 'string') return error;
    return error instanceof Error ? error.message : fallback;
  }

  async function loadNotes(type = entityType, id = entityId.trim()): Promise<void> {
    const normalizedId = id.trim();
    if (!normalizedId) return;

    const token = ++loadToken;
    isLoading = true;
    errorMessage = null;

    try {
      const loadedNotes = await listNotesForEntity(type, normalizedId);
      if (token === loadToken) {
        notes = sortNotes(loadedNotes);
      }
    } catch (error) {
      if (token === loadToken) {
        errorMessage = messageFromError(error, t('entityNotes.loadFailed'));
      }
    } finally {
      if (token === loadToken) {
        isLoading = false;
      }
    }
  }

  async function addNote(): Promise<void> {
    const content = newContent.trim();
    if (!content || !entityId.trim()) return;

    isCreating = true;
    errorMessage = null;

    try {
      const note = await createNote({
        entityType,
        entityId,
        content,
      });
      notes = sortNotes([note, ...notes]);
      newContent = '';
    } catch (error) {
      errorMessage = messageFromError(error, t('entityNotes.saveFailed'));
    } finally {
      isCreating = false;
    }
  }

  function startEdit(note: Note): void {
    editingNoteId = note.id;
    editContent = note.content;
  }

  function cancelEdit(): void {
    editingNoteId = null;
    editContent = '';
  }

  async function saveEdit(note: Note): Promise<void> {
    const content = editContent.trim();
    if (!content) return;

    savingNoteId = note.id;
    errorMessage = null;

    try {
      const updated = await updateNote(note.id, content);
      notes = sortNotes(notes.map((item) => item.id === updated.id ? updated : item));
      cancelEdit();
    } catch (error) {
      errorMessage = messageFromError(error, t('entityNotes.saveFailed'));
    } finally {
      savingNoteId = null;
    }
  }

  async function removeNote(note: Note): Promise<void> {
    const confirmed = window.confirm(t('entityNotes.confirmDelete'));
    if (!confirmed) return;

    deletingNoteId = note.id;
    errorMessage = null;

    try {
      await deleteNote(note.id);
      notes = notes.filter((item) => item.id !== note.id);
      if (editingNoteId === note.id) {
        cancelEdit();
      }
    } catch (error) {
      errorMessage = messageFromError(error, t('entityNotes.deleteFailed'));
    } finally {
      deletingNoteId = null;
    }
  }
</script>

<div class="entity-notes-panel">
  <form
    class="note-create"
    onsubmit={(event) => {
      event.preventDefault();
      void addNote();
    }}
  >
    <textarea
      class="textarea note-input selectable"
      bind:value={newContent}
      placeholder={t('entityNotes.newPlaceholder')}
      rows="3"
      disabled={isCreating || !entityId.trim()}
    ></textarea>
    <div class="panel-actions">
      <button class="btn btn-primary btn-sm" type="submit" disabled={!canCreate}>
        {isCreating ? t('common.loading') : t('entityNotes.add')}
      </button>
    </div>
  </form>

  {#if errorMessage}
    <div class="panel-error" role="alert">
      <span>{errorMessage}</span>
      <button class="btn btn-ghost btn-sm" type="button" onclick={() => void loadNotes()}>
        {t('common.retry')}
      </button>
    </div>
  {/if}

  {#if isLoading}
    <div class="note-loading" aria-label={t('common.loading')} aria-live="polite">
      {#each [1, 2, 3] as row (row)}
        <div class="skeleton note-skeleton"></div>
      {/each}
    </div>
  {:else if notes.length === 0}
    <p class="panel-empty">{t('entityNotes.empty')}</p>
  {:else}
    <ul class="note-list" role="list">
      {#each notes as note (note.id)}
        <li class="note-row">
          {#if editingNoteId === note.id}
            <textarea
              class="textarea note-input selectable"
              bind:value={editContent}
              rows="3"
              disabled={savingNoteId === note.id}
            ></textarea>
            <div class="note-row-actions">
              <button
                class="btn btn-ghost btn-sm"
                type="button"
                onclick={cancelEdit}
                disabled={savingNoteId === note.id}
              >
                {t('common.cancel')}
              </button>
              <button
                class="btn btn-primary btn-sm"
                type="button"
                onclick={() => void saveEdit(note)}
                disabled={savingNoteId === note.id || !editContent.trim()}
              >
                {savingNoteId === note.id ? t('common.loading') : t('common.save')}
              </button>
            </div>
          {:else}
            <div class="note-row-header">
              <span class="note-timestamp">
                {t('common.updated')} {formatRelativeTime(note.updatedAt)}
              </span>
              <div class="note-row-actions">
                <button
                  class="btn btn-ghost btn-sm"
                  type="button"
                  onclick={() => startEdit(note)}
                  disabled={deletingNoteId === note.id}
                >
                  {t('common.edit')}
                </button>
                <button
                  class="btn btn-ghost btn-sm danger-action"
                  type="button"
                  onclick={() => void removeNote(note)}
                  disabled={deletingNoteId === note.id}
                >
                  {deletingNoteId === note.id ? t('common.loading') : t('common.delete')}
                </button>
              </div>
            </div>
            <p class="note-content selectable">{note.content}</p>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .entity-notes-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .note-create {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .note-input {
    min-height: 72px;
    resize: vertical;
  }

  .panel-actions,
  .note-row-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .panel-error {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border: var(--border-width) solid var(--color-danger-500);
    border-radius: var(--border-radius-md);
    color: var(--text-danger);
    font-size: var(--text-sm);
  }

  .panel-empty {
    margin: 0;
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .note-loading {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .note-skeleton {
    height: 58px;
    border-radius: var(--border-radius-md);
  }

  .note-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin: 0;
    padding: 0;
  }

  .note-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding-block-end: var(--space-4);
    border-block-end: var(--border-width) solid var(--border-subtle);
  }

  .note-row:last-child {
    padding-block-end: 0;
    border-block-end: none;
  }

  .note-row-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .note-timestamp {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
  }

  .note-content {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
    line-height: var(--leading-relaxed);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .danger-action {
    color: var(--text-danger);
  }
</style>
