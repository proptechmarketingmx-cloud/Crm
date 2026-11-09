<script lang="ts">
  import { t } from '$lib/i18n';
  import type { CrmEntityType } from '$lib/api/notes';
  import {
    applyTagToEntity,
    createTag,
    listTags,
    listTagsForEntity,
    removeTagFromEntity,
    type Tag,
  } from '$lib/api/tags';

  let {
    entityType,
    entityId,
  }: {
    entityType: CrmEntityType;
    entityId: string;
  } = $props();

  let entityTags = $state<Tag[]>([]);
  let allTags = $state<Tag[]>([]);
  let inputValue = $state('');
  let showSuggestions = $state(false);
  let isLoading = $state(false);
  let isSaving = $state(false);
  let removingTagId = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let loadToken = 0;

  const selectedTagIds = $derived(new Set(entityTags.map((tag) => tag.id)));
  const normalizedInput = $derived(inputValue.trim().toLowerCase());
  const canApply = $derived(inputValue.trim().length > 0 && !isSaving && entityId.trim().length > 0);
  const filteredSuggestions = $derived(
    allTags
      .filter((tag) => !selectedTagIds.has(tag.id))
      .filter((tag) => !normalizedInput || tag.name.toLowerCase().includes(normalizedInput))
      .slice(0, 6),
  );

  $effect(() => {
    const normalizedEntityId = entityId.trim();
    const normalizedEntityType = entityType;

    if (!normalizedEntityId) {
      entityTags = [];
      allTags = [];
      isLoading = false;
      errorMessage = null;
      return;
    }

    void loadTagData(normalizedEntityType, normalizedEntityId);
  });

  function sortTags(tags: Tag[]): Tag[] {
    return [...tags].sort((a, b) => a.name.localeCompare(b.name));
  }

  function messageFromError(error: unknown, fallback: string): string {
    if (typeof error === 'string') return error;
    return error instanceof Error ? error.message : fallback;
  }

  function findExistingTag(name: string): Tag | undefined {
    const normalized = name.trim().toLowerCase();
    return allTags.find((tag) => tag.name.toLowerCase() === normalized);
  }

  function tagTextColor(color: string): string {
    const hex = color.replace('#', '').trim();
    if (!/^[0-9a-fA-F]{6}$/.test(hex)) {
      return '#ffffff';
    }

    const red = parseInt(hex.slice(0, 2), 16);
    const green = parseInt(hex.slice(2, 4), 16);
    const blue = parseInt(hex.slice(4, 6), 16);
    const luminance = (0.299 * red + 0.587 * green + 0.114 * blue) / 255;
    return luminance > 0.68 ? '#13343B' : '#ffffff';
  }

  async function loadTagData(type = entityType, id = entityId.trim()): Promise<void> {
    const normalizedId = id.trim();
    if (!normalizedId) return;

    const token = ++loadToken;
    isLoading = true;
    errorMessage = null;

    try {
      const [loadedAllTags, loadedEntityTags] = await Promise.all([
        listTags(),
        listTagsForEntity(type, normalizedId),
      ]);

      if (token === loadToken) {
        allTags = sortTags(loadedAllTags);
        entityTags = sortTags(loadedEntityTags);
      }
    } catch (error) {
      if (token === loadToken) {
        errorMessage = messageFromError(error, t('entityTags.loadFailed'));
      }
    } finally {
      if (token === loadToken) {
        isLoading = false;
      }
    }
  }

  async function applyTagName(name = inputValue): Promise<void> {
    const normalizedName = name.trim();
    if (!normalizedName || !entityId.trim()) return;

    isSaving = true;
    errorMessage = null;

    try {
      let tag = findExistingTag(normalizedName);
      if (!tag) {
        tag = await createTag({ name: normalizedName });
        allTags = sortTags([...allTags, tag]);
      }

      if (!entityTags.some((item) => item.id === tag.id)) {
        await applyTagToEntity(entityType, entityId, tag.id);
        entityTags = sortTags([...entityTags, tag]);
      }

      inputValue = '';
      showSuggestions = false;
    } catch (error) {
      errorMessage = messageFromError(error, t('entityTags.saveFailed'));
    } finally {
      isSaving = false;
    }
  }

  async function removeTag(tag: Tag): Promise<void> {
    removingTagId = tag.id;
    errorMessage = null;

    try {
      await removeTagFromEntity(entityType, entityId, tag.id);
      entityTags = entityTags.filter((item) => item.id !== tag.id);
    } catch (error) {
      errorMessage = messageFromError(error, t('entityTags.removeFailed'));
    } finally {
      removingTagId = null;
    }
  }

  function applySuggestion(event: MouseEvent, tag: Tag): void {
    event.preventDefault();
    void applyTagName(tag.name);
  }
</script>

<div class="entity-tags-panel">
  <form
    class="tag-add-form"
    onsubmit={(event) => {
      event.preventDefault();
      void applyTagName();
    }}
  >
    <div class="tag-input-wrap">
      <input
        class="input selectable"
        type="text"
        bind:value={inputValue}
        placeholder={t('entityTags.inputPlaceholder')}
        disabled={isSaving || !entityId.trim()}
        autocomplete="off"
        onfocus={() => { showSuggestions = filteredSuggestions.length > 0; }}
        oninput={() => { showSuggestions = filteredSuggestions.length > 0; }}
        onblur={() => {
          window.setTimeout(() => {
            showSuggestions = false;
          }, 120);
        }}
      />

      {#if showSuggestions && filteredSuggestions.length > 0}
        <ul class="tag-suggestions" role="listbox">
          {#each filteredSuggestions as tag (tag.id)}
            <li role="option" aria-selected="false">
              <button
                class="tag-suggestion"
                type="button"
                onmousedown={(event) => applySuggestion(event, tag)}
              >
                <span
                  class="tag-color-dot"
                  style="background-color: {tag.color};"
                  aria-hidden="true"
                ></span>
                {tag.name}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <button class="btn btn-primary btn-sm" type="submit" disabled={!canApply}>
      {isSaving ? t('common.loading') : t('entityTags.add')}
    </button>
  </form>

  {#if errorMessage}
    <div class="panel-error" role="alert">
      <span>{errorMessage}</span>
      <button class="btn btn-ghost btn-sm" type="button" onclick={() => void loadTagData()}>
        {t('common.retry')}
      </button>
    </div>
  {/if}

  {#if isLoading}
    <div class="tag-loading" aria-label={t('common.loading')} aria-live="polite">
      {#each [1, 2] as row (row)}
        <div class="skeleton tag-skeleton"></div>
      {/each}
    </div>
  {:else if entityTags.length === 0}
    <p class="panel-empty">{t('entityTags.empty')}</p>
  {:else}
    <ul class="tag-list" role="list">
      {#each entityTags as tag (tag.id)}
        <li class="tag-pill" role="listitem" style="background-color: {tag.color}; color: {tagTextColor(tag.color)};">
          <span class="tag-name">{tag.name}</span>
          <button
            class="tag-remove"
            type="button"
            onclick={() => void removeTag(tag)}
            disabled={removingTagId === tag.id}
            aria-label="{t('common.remove')} {tag.name}"
          >
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <path d="M7.5 2.5l-5 5M2.5 2.5l5 5"/>
            </svg>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .entity-tags-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .tag-add-form {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
  }

  .tag-input-wrap {
    position: relative;
    flex: 1;
    min-width: 0;
  }

  .tag-suggestions {
    position: absolute;
    inset-block-start: calc(100% + var(--space-2));
    inset-inline-start: 0;
    z-index: var(--z-tooltip);
    min-width: 220px;
    max-width: min(320px, 100%);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    background-color: var(--surface-modal);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .tag-suggestion {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-3) var(--space-4);
    color: var(--text-primary);
    font-size: var(--text-sm);
    text-align: start;
  }

  .tag-suggestion:hover {
    background-color: var(--surface-hover);
  }

  .tag-color-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: var(--border-width) solid rgba(0, 0, 0, 0.12);
    flex-shrink: 0;
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

  .tag-loading {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .tag-skeleton {
    height: 34px;
    border-radius: var(--border-radius-md);
  }

  .tag-list {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    max-width: 100%;
    border-radius: 9999px;
    padding: 3px var(--space-3);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
  }

  .tag-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag-remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    color: currentColor;
    opacity: 0.75;
    flex-shrink: 0;
  }

  .tag-remove:hover:not(:disabled) {
    opacity: 1;
  }

  .tag-remove:disabled {
    cursor: wait;
    opacity: 0.65;
  }

  @media (max-width: 520px) {
    .tag-add-form {
      flex-direction: column;
    }

    .tag-add-form .btn {
      align-self: flex-end;
    }
  }
</style>
