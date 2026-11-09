<script lang="ts">
  import { t } from '$lib/i18n';

  let {
    open = false,
    onclose,
  }: {
    open?: boolean;
    onclose?: () => void;
  } = $props();

  const rows = $derived([
    { keys: 'Ctrl/Cmd+K', label: t('shortcuts.openPalette') },
    { keys: '/', label: t('shortcuts.focusSearch') },
    { keys: '1-7', label: t('shortcuts.switchNav') },
    { keys: '?', label: t('shortcuts.shortcutSheet') },
  ]);

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onclose?.();
    }
  }
</script>

{#if open}
  <div
    class="sheet-backdrop"
    role="presentation"
    onclick={(event) => { if (event.target === event.currentTarget) onclose?.(); }}
    onkeydown={handleKeyDown}
  >
    <div
      class="sheet"
      role="dialog"
      aria-modal="true"
      aria-labelledby="shortcut-sheet-title"
      data-testid="shortcut-sheet"
    >
      <h2 id="shortcut-sheet-title" class="sheet-title">{t('shortcuts.title')}</h2>
      <table class="sheet-table">
        <tbody>
          {#each rows as row (row.keys)}
            <tr>
              <th scope="row"><kbd>{row.keys}</kbd></th>
              <td>{row.label}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <button class="btn btn-secondary btn-sm" type="button" onclick={() => onclose?.()}>
        {t('common.close')}
      </button>
    </div>
  </div>
{/if}

<style>
  .sheet-backdrop {
    position: fixed;
    inset: 0;
    z-index: 80;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
    background: color-mix(in srgb, var(--text-primary) 28%, transparent);
  }

  .sheet {
    width: min(420px, 100%);
    background: var(--surface-app);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .sheet-title {
    margin: 0;
    font-size: var(--text-md);
  }

  .sheet-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  th, td {
    padding: var(--space-2) 0;
    text-align: start;
    font-weight: var(--weight-medium);
  }

  th {
    width: 38%;
    color: var(--text-secondary);
  }

  kbd {
    font-family: inherit;
    font-weight: var(--weight-semibold);
  }
</style>
