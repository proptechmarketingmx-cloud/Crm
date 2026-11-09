<script lang="ts">
  /**
   * TagPicker.svelte — Tag selection component for 900CRM.
   *
   * Renders existing tags as colored pills, allows adding new tags
   * via text input, and removing existing ones.
   */

  import { t } from '$lib/i18n';

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    tags = $bindable<string[]>([]),
    suggestions = [],
    placeholder = '',
    maxTags = 20,
    readonly = false,
    onchange,
  }: {
    tags?: string[];
    suggestions?: string[];
    placeholder?: string;
    maxTags?: number;
    readonly?: boolean;
    onchange?: (tags: string[]) => void;
  } = $props();

  // ── State ──────────────────────────────────────────────────────────────────

  let inputValue = $state('');
  let inputEl = $state<HTMLInputElement | undefined>(undefined);
  let showSuggestions = $state(false);

  // ── Derived ────────────────────────────────────────────────────────────────

  const filteredSuggestions = $derived(
    suggestions
      .filter((s) => !tags.includes(s) && s.toLowerCase().includes(inputValue.toLowerCase()))
      .slice(0, 6)
  );

  // ── Helpers ────────────────────────────────────────────────────────────────

  function addTag(tag: string) {
    const trimmed = tag.trim();
    if (!trimmed || tags.includes(trimmed) || tags.length >= maxTags) return;

    const next = [...tags, trimmed];
    tags = next;
    onchange?.(next);
    inputValue = '';
    showSuggestions = false;
  }

  function removeTag(tag: string) {
    const next = tags.filter((t) => t !== tag);
    tags = next;
    onchange?.(next);
  }

  function handleSuggestionMouseDown(e: MouseEvent, suggestion: string) {
    e.preventDefault();
    addTag(suggestion);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag(inputValue);
    }
    if (e.key === 'Backspace' && !inputValue && tags.length > 0) {
      removeTag(tags[tags.length - 1]);
    }
    if (e.key === 'Escape') {
      showSuggestions = false;
    }
  }

  // Tag pill color derived from tag text
  const TAG_COLORS = [
    { bg: '#D6F5FA', fg: '#115058' },
    { bg: '#fce4ec', fg: '#a93226' },
    { bg: '#e8f5e9', fg: '#1b5e20' },
    { bg: '#fff8e1', fg: '#b8860b' },
    { bg: '#f3e5f5', fg: '#6a1b9a' },
    { bg: '#e3f2fd', fg: '#1565c0' },
  ];

  function tagColor(tag: string): { bg: string; fg: string } {
    let hash = 0;
    for (let i = 0; i < tag.length; i++) {
      hash = tag.charCodeAt(i) + ((hash << 5) - hash);
    }
    return TAG_COLORS[Math.abs(hash) % TAG_COLORS.length];
  }
</script>

<div class="tag-picker">
  <!-- Tag pills -->
  <div class="tag-list" role="list">
    {#each tags as tag (tag)}
      {@const color = tagColor(tag)}
      <div
        class="tag-pill"
        style="background-color: {color.bg}; color: {color.fg};"
        role="listitem"
      >
        <span class="tag-label">{tag}</span>
        {#if !readonly}
          <button
            class="tag-remove"
            onclick={() => removeTag(tag)}
            aria-label="{t('common.remove')} {tag}"
            type="button"
          >
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <path d="M7.5 2.5l-5 5M2.5 2.5l5 5"/>
            </svg>
          </button>
        {/if}
      </div>
    {/each}

    <!-- Input -->
    {#if !readonly && tags.length < maxTags}
      <div class="tag-input-wrap">
        <input
          class="tag-input selectable"
          bind:this={inputEl}
          bind:value={inputValue}
          placeholder={tags.length === 0 ? (placeholder || t('common.tags')) : ''}
          onkeydown={handleKeyDown}
          onfocus={() => { showSuggestions = filteredSuggestions.length > 0; }}
          onblur={() => { setTimeout(() => { showSuggestions = false; }, 150); }}
          oninput={() => { showSuggestions = filteredSuggestions.length > 0; }}
          type="text"
          autocomplete="off"
        />

        <!-- Suggestions dropdown -->
        {#if showSuggestions && filteredSuggestions.length > 0}
          <ul class="tag-suggestions" role="listbox">
            {#each filteredSuggestions as suggestion (suggestion)}
              <li
                class="tag-suggestion"
                role="option"
                aria-selected="false"
                onmousedown={(e) => handleSuggestionMouseDown(e, suggestion)}
              >
                {suggestion}
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .tag-picker {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    align-items: center;
    min-height: 32px;
    padding: var(--space-2);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    background-color: var(--surface-input);
    cursor: text;
  }

  .tag-list:focus-within {
    border-color: var(--border-focus);
    box-shadow: 0 0 0 3px rgba(32, 128, 141, 0.15);
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 2px var(--space-3);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    white-space: nowrap;
  }

  .tag-label {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tag-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    flex-shrink: 0;
    opacity: 0.7;
    transition: opacity var(--duration-fast) var(--ease-out);
    cursor: pointer;
  }

  .tag-remove:hover {
    opacity: 1;
  }

  .tag-input-wrap {
    position: relative;
    flex: 1;
    min-width: 80px;
  }

  .tag-input {
    border: none;
    background: transparent;
    outline: none;
    font-size: var(--text-sm);
    color: var(--text-primary);
    width: 100%;
    min-width: 60px;
    padding: 0 var(--space-2);
  }

  .tag-input::placeholder {
    color: var(--text-tertiary);
  }

  .tag-suggestions {
    position: absolute;
    top: calc(100% + var(--space-2));
    inset-inline-start: 0;
    min-width: 160px;
    background-color: var(--surface-modal);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    box-shadow: var(--shadow-lg);
    z-index: var(--z-tooltip);
    overflow: hidden;
    list-style: none;
  }

  .tag-suggestion {
    padding: var(--space-3) var(--space-4);
    font-size: var(--text-sm);
    color: var(--text-primary);
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .tag-suggestion:hover {
    background-color: var(--surface-hover);
  }
</style>
