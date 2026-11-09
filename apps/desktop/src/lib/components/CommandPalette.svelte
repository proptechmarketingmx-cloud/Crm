<script lang="ts">
  /**
   * Command palette: jump to workspace routes and search records.
   */

  import { t } from '$lib/i18n';
  import { globalSearch } from '$lib/api/search';
  import { WORKSPACE_NAV_SHORTCUTS } from '$lib/utils/keyboard';
  import { mapGlobalSearchResultToSearchResult } from '$lib/utils/searchResults';
  import type { SearchResult } from '$lib/stores/ui';
  import { navigateHash } from '$lib/utils/hashRouter';

  let {
    open = false,
    onclose,
    onnavigate,
  }: {
    open?: boolean;
    onclose?: () => void;
    onnavigate?: (href: string) => void;
  } = $props();

  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let searching = $state(false);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);
  let requestId = 0;

  const routes = $derived(
    WORKSPACE_NAV_SHORTCUTS.map((item) => ({
      href: item.href,
      label: routeLabel(item.href),
      key: item.key,
    }))
  );

  const filteredRoutes = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) {
      return routes;
    }
    return routes.filter((route) => route.label.toLowerCase().includes(needle));
  });

  function routeLabel(href: string): string {
    switch (href) {
      case '/':
        return t('nav.dashboard');
      case '/leads':
        return t('nav.leads');
      case '/contacts':
        return t('nav.contacts');
      case '/organizations':
        return t('nav.organizations');
      case '/pipeline':
        return t('nav.pipeline');
      case '/activities':
        return t('nav.activities');
      case '/reports':
        return t('nav.reports');
      default:
        return href;
    }
  }

  function recordHref(result: SearchResult): string {
    if (result.type === 'contact') return `/contacts/${result.id}`;
    if (result.type === 'organization') return `/organizations/${result.id}`;
    if (result.type === 'deal') return `/deals/${result.id}`;
    return '/activities';
  }

  function go(href: string) {
    onnavigate?.(href);
    navigateHash(href);
    onclose?.();
  }

  $effect(() => {
    if (!open) {
      return;
    }

    query = '';
    results = [];
    searching = false;
    const frame = requestAnimationFrame(() => inputEl?.focus());
    return () => cancelAnimationFrame(frame);
  });

  $effect(() => {
    if (!open) {
      return;
    }

    const needle = query.trim();
    if (needle.length < 2) {
      results = [];
      searching = false;
      return;
    }

    const id = ++requestId;
    searching = true;
    const timer = setTimeout(() => {
      void (async () => {
        try {
          const found = await globalSearch(needle, 8);
          if (id !== requestId) return;
          results = found.map(mapGlobalSearchResultToSearchResult);
        } catch {
          if (id !== requestId) return;
          results = [];
        } finally {
          if (id === requestId) {
            searching = false;
          }
        }
      })();
    }, 200);

    return () => clearTimeout(timer);
  });

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onclose?.();
    }
  }
</script>

{#if open}
  <div
    class="palette-backdrop"
    role="presentation"
    onclick={(event) => { if (event.target === event.currentTarget) onclose?.(); }}
    onkeydown={handleKeyDown}
  >
    <div
      class="palette"
      role="dialog"
      aria-modal="true"
      aria-labelledby="command-palette-title"
      data-testid="command-palette"
    >
      <h2 id="command-palette-title" class="palette-title">{t('commandPalette.title')}</h2>
      <input
        id="command-palette-input"
        class="input palette-input"
        bind:this={inputEl}
        bind:value={query}
        type="search"
        placeholder={t('commandPalette.placeholder')}
        aria-label={t('commandPalette.placeholder')}
        autocomplete="off"
        spellcheck={false}
      />

      <p class="palette-section">{t('commandPalette.routes')}</p>
      <ul class="palette-list">
        {#each filteredRoutes as route (route.href)}
          <li>
            <button class="palette-item" type="button" onclick={() => go(route.href)}>
              <span>{route.label}</span>
              <kbd>{route.key}</kbd>
            </button>
          </li>
        {/each}
      </ul>

      {#if query.trim().length >= 2}
        <p class="palette-section">{t('commandPalette.records')}</p>
        {#if searching}
          <p class="palette-empty">{t('common.loading')}</p>
        {:else if results.length === 0}
          <p class="palette-empty">{t('commandPalette.noResults')}</p>
        {:else}
          <ul class="palette-list">
            {#each results as result (`${result.type}:${result.id}`)}
              <li>
                <button class="palette-item" type="button" onclick={() => go(recordHref(result))}>
                  <span>
                    <strong>{result.title}</strong>
                    {#if result.subtitle}
                      <span class="palette-sub">{result.subtitle}</span>
                    {/if}
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    inset: 0;
    z-index: 80;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 12vh var(--space-6) 0;
    background: color-mix(in srgb, var(--text-primary) 28%, transparent);
  }

  .palette {
    width: min(520px, 100%);
    background: var(--surface-app);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    box-shadow: 0 12px 40px color-mix(in srgb, var(--text-primary) 18%, transparent);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .palette-title {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .palette-input {
    width: 100%;
  }

  .palette-section {
    margin: var(--space-2) 0 0;
    font-size: var(--text-xs);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .palette-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 240px;
    overflow-y: auto;
  }

  .palette-item {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    text-align: start;
    cursor: pointer;
  }

  .palette-item:hover,
  .palette-item:focus-visible {
    background: var(--surface-active);
  }

  .palette-sub {
    display: block;
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .palette-empty {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  kbd {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 0 var(--space-2);
  }
</style>
