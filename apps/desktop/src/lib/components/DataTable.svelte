<script lang="ts">
  /**
   * DataTable.svelte — Sortable, filterable data table for 900CRM.
   *
   * Features:
   *   - Column definitions with custom renderers
   *   - Sort indicators, row click handler
   *   - Loading skeleton state
   *   - Empty state
   *   - Pagination controls
   *   - RTL support via CSS logical properties
   */

  import { t } from '$lib/i18n';
  import EmptyState from './EmptyState.svelte';

  // ── Types ──────────────────────────────────────────────────────────────────

  export interface Column<T = unknown> {
    key: string;
    label: string;
    sortable?: boolean;
    width?: string;
    align?: 'start' | 'center' | 'end';
    render?: (row: T) => string;
    cell?: import('svelte').Snippet<[T]>;
  }

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    columns,
    rows,
    loading = false,
    sortKey = $bindable(''),
    sortDir = $bindable<'asc' | 'desc'>('asc'),
    total = 0,
    page = 1,
    pageSize = 50,
    emptyTitle = '',
    emptyDescription = '',
    emptyIcon = 'inbox',
    emptyActionLabel = '',
    onemptyaction,
    onrowclick,
    onnextpage,
    onprevpage,
  }: {
    columns: Column[];
    rows: unknown[];
    loading?: boolean;
    sortKey?: string;
    sortDir?: 'asc' | 'desc';
    total?: number;
    page?: number;
    pageSize?: number;
    emptyTitle?: string;
    emptyDescription?: string;
    emptyIcon?: string;
    emptyActionLabel?: string;
    onemptyaction?: () => void;
    onrowclick?: (row: unknown) => void;
    onnextpage?: () => void;
    onprevpage?: () => void;
  } = $props();

  // ── Derived ────────────────────────────────────────────────────────────────

  const totalPages = $derived(Math.ceil(total / pageSize));
  const hasNext    = $derived(page < totalPages);
  const hasPrev    = $derived(page > 1);
  const startItem  = $derived((page - 1) * pageSize + 1);
  const endItem    = $derived(Math.min(page * pageSize, total));

  // ── Helpers ────────────────────────────────────────────────────────────────

  function handleSort(col: Column) {
    if (!col.sortable) return;
    if (sortKey === col.key) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = col.key;
      sortDir = 'asc';
    }
  }

  function getCellValue(row: unknown, col: Column): string {
    if (col.render) return col.render(row);
    const val = (row as Record<string, unknown>)[col.key];
    return val != null ? String(val) : '';
  }
</script>

<div class="data-table-wrap">
  <!-- Table -->
  <div class="data-table-scroll">
    <table class="data-table" aria-label="data table">
      <thead>
        <tr>
          {#each columns as col (col.key)}
            <th
              style={col.width ? `width: ${col.width}` : undefined}
              class:sortable={col.sortable}
              class:sort-active={sortKey === col.key}
              style:text-align={col.align ?? 'start'}
              onclick={() => handleSort(col)}
              aria-sort={sortKey === col.key ? (sortDir === 'asc' ? 'ascending' : 'descending') : undefined}
            >
              <span class="th-inner">
                {col.label}
                {#if col.sortable}
                  <span class="sort-icon" aria-hidden="true">
                    {#if sortKey === col.key}
                      {sortDir === 'asc' ? '↑' : '↓'}
                    {:else}
                      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                        <path d="M5 1v8M2 4l3-3 3 3M2 6l3 3 3-3" opacity=".4"/>
                      </svg>
                    {/if}
                  </span>
                {/if}
              </span>
            </th>
          {/each}
        </tr>
      </thead>

      <tbody>
        {#if loading}
          {#each Array(6) as _, i (i)}
            <tr class="skeleton-row">
              {#each columns as col (col.key)}
                <td>
                  <div class="skeleton table-skeleton-cell"></div>
                </td>
              {/each}
            </tr>
          {/each}
        {:else if rows.length === 0}
          <tr>
            <td colspan={columns.length} class="empty-cell">
              <EmptyState
                icon={emptyIcon}
                title={emptyTitle || t('common.noResults')}
                description={emptyDescription}
                actionLabel={emptyActionLabel}
                onaction={onemptyaction}
              />
            </td>
          </tr>
        {:else}
          {#each rows as row}
            <tr
              class:clickable={!!onrowclick}
              onclick={() => onrowclick?.(row)}
              onkeydown={(e) => { if (e.key === 'Enter') onrowclick?.(row); }}
              tabindex={onrowclick ? 0 : undefined}
              role={onrowclick ? 'button' : undefined}
            >
              {#each columns as col (col.key)}
                <td style:text-align={col.align ?? 'start'}>
                  {#if col.cell}
                    {@render col.cell(row)}
                  {:else}
                    {getCellValue(row, col)}
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  <!-- Pagination -->
  {#if totalPages > 1 || total > 0}
    <div class="pagination">
      <span class="pagination-info">
        {#if total > 0}
          {startItem}–{endItem} / {total}
        {/if}
      </span>

      <div class="pagination-controls">
        <button
          class="btn btn-ghost btn-sm"
          onclick={onprevpage}
          disabled={!hasPrev}
          aria-label={t('common.previous')}
          type="button"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M9 11L5 7l4-4"/>
          </svg>
          {t('common.previous')}
        </button>

        <span class="pagination-page">
          {page} / {totalPages}
        </span>

        <button
          class="btn btn-ghost btn-sm"
          onclick={onnextpage}
          disabled={!hasNext}
          aria-label={t('common.next')}
          type="button"
        >
          {t('common.next')}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M5 3l4 4-4 4"/>
          </svg>
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .data-table-wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .data-table-scroll {
    flex: 1 1 0;
    overflow: auto;
  }

  th.sortable {
    cursor: pointer;
  }

  th.sort-active {
    color: var(--text-accent);
  }

  .th-inner {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  .sort-icon {
    opacity: 0.6;
    font-size: var(--text-xs);
  }

  .data-table tbody tr.clickable:focus {
    outline: 2px solid var(--border-focus);
    outline-offset: -2px;
  }

  .skeleton-row td {
    padding: var(--space-4);
  }

  .table-skeleton-cell {
    height: 14px;
    width: 80%;
    border-radius: var(--border-radius-sm);
  }

  .empty-cell {
    padding: 0;
    border: none;
  }

  .pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-4);
    border-block-start: var(--border-width) solid var(--border-default);
    flex-shrink: 0;
    gap: var(--space-4);
  }

  .pagination-info {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  .pagination-controls {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .pagination-page {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    min-width: 40px;
    text-align: center;
  }
</style>
