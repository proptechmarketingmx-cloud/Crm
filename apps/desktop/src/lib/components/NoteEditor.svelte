<script lang="ts">
  /**
   * NoteEditor.svelte — Simple auto-resizing note editor for 900CRM.
   *
   * Textarea with auto-resize, save/cancel buttons, and timestamp display.
   * Used in ContactDetail and ActivityDetail for inline note editing.
   */

  import { t } from '$lib/i18n';
  import { formatRelativeTime } from '$lib/utils/formatters';

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    value = $bindable(''),
    placeholder = '',
    lastUpdated = '',
    saving = false,
    onsave,
    oncancel,
  }: {
    value?: string;
    placeholder?: string;
    lastUpdated?: string;
    saving?: boolean;
    onsave?: (value: string) => void;
    oncancel?: () => void;
  } = $props();

  // ── State ──────────────────────────────────────────────────────────────────

  let editValue = $state(value);
  let isEditing = $state(false);
  let textareaEl = $state<HTMLTextAreaElement | undefined>(undefined);

  // ── Helpers ────────────────────────────────────────────────────────────────

  function startEditing() {
    editValue = value;
    isEditing = true;
    requestAnimationFrame(() => {
      textareaEl?.focus();
      autoResize();
    });
  }

  function autoResize() {
    if (!textareaEl) return;
    textareaEl.style.height = 'auto';
    textareaEl.style.height = textareaEl.scrollHeight + 'px';
  }

  function handleSave() {
    onsave?.(editValue);
    isEditing = false;
  }

  function handleCancel() {
    editValue = value;
    isEditing = false;
    oncancel?.();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleCancel();
    }
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      handleSave();
    }
  }

  // Sync value prop changes
  $effect(() => {
    if (!isEditing) {
      editValue = value;
    }
  });
</script>

<div class="note-editor">
  {#if isEditing}
    <textarea
      class="textarea note-textarea selectable"
      bind:this={textareaEl}
      bind:value={editValue}
      {placeholder}
      oninput={autoResize}
      onkeydown={handleKeyDown}
      rows="4"
    ></textarea>

    <div class="note-actions">
      <span class="note-hint">{t('common.save')} Ctrl+Enter</span>
      <div class="note-buttons">
        <button
          class="btn btn-ghost btn-sm"
          onclick={handleCancel}
          type="button"
          disabled={saving}
        >
          {t('common.cancel')}
        </button>
        <button
          class="btn btn-primary btn-sm"
          onclick={handleSave}
          type="button"
          disabled={saving}
        >
          {saving ? t('common.loading') : t('common.save')}
        </button>
      </div>
    </div>
  {:else}
    <div
      class="note-display"
      role="button"
      tabindex="0"
      onclick={startEditing}
      onkeydown={(e) => { if (e.key === 'Enter') startEditing(); }}
      aria-label={t('common.edit') + ' note'}
    >
      {#if value}
        <p class="note-text selectable">{value}</p>
      {:else}
        <p class="note-placeholder">{placeholder || t('common.notes')}</p>
      {/if}
    </div>

    {#if lastUpdated}
      <p class="note-timestamp">
        {t('common.updated')} {formatRelativeTime(lastUpdated)}
      </p>
    {/if}
  {/if}
</div>

<style>
  .note-editor {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .note-textarea {
    min-height: 80px;
    resize: none;
    overflow: hidden;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    line-height: var(--leading-relaxed);
  }

  .note-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .note-hint {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  .note-buttons {
    display: flex;
    gap: var(--space-3);
  }

  .note-display {
    cursor: text;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--border-radius-md);
    border: var(--border-width) solid transparent;
    min-height: 60px;
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out);
  }

  .note-display:hover {
    border-color: var(--border-default);
    background-color: var(--surface-hover);
  }

  .note-text {
    font-size: var(--text-sm);
    color: var(--text-primary);
    line-height: var(--leading-relaxed);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .note-placeholder {
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    font-style: italic;
  }

  .note-timestamp {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }
</style>
