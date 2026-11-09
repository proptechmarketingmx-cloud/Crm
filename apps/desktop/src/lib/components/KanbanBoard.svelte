<script lang="ts">
  /**
   * KanbanBoard.svelte — Generic drag-and-drop Kanban board.
   *
   * Uses native HTML5 drag API — no external libraries.
   * Configurable columns, emits a 'columnchange' event when a card is moved.
   *
   * @example
   * <KanbanBoard
   *   columns={[{ id: 'todo', label: 'To Do', items: [...] }]}
   *   oncolumnchange={(itemId, toColumn) => store.moveItem(itemId, toColumn)}
   * >
   *   {#snippet card(item)}
   *     <DealCard deal={item} />
   *   {/snippet}
   * </KanbanBoard>
   */

  import { t } from '$lib/i18n';
  import EmptyState from './EmptyState.svelte';

  // ── Types ──────────────────────────────────────────────────────────────────

  export interface KanbanColumn<T = unknown> {
    id: string;
    label: string;
    count?: number;
    totalValue?: string;
    items: T[];
  }

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    columns,
    oncolumnchange,
    onadddeal,
    card,
  }: {
    columns: KanbanColumn[];
    oncolumnchange?: (itemId: string, toColumnId: string) => void;
    onadddeal?: (columnId: string) => void;
    card?: import('svelte').Snippet<[unknown]>;
  } = $props();

  // ── Drag state ─────────────────────────────────────────────────────────────

  let draggingItemId = $state<string | null>(null);
  let draggingFromColumn = $state<string | null>(null);
  let overColumnId = $state<string | null>(null);

  // ── Drag handlers ──────────────────────────────────────────────────────────

  function handleDragStart(e: DragEvent, itemId: string, columnId: string) {
    draggingItemId = itemId;
    draggingFromColumn = columnId;
    e.dataTransfer?.setData('text/plain', itemId);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
  }

  function handleDragEnd() {
    draggingItemId = null;
    draggingFromColumn = null;
    overColumnId = null;
  }

  function handleDragOver(e: DragEvent, columnId: string) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    overColumnId = columnId;
  }

  function handleDragLeave(e: DragEvent, columnId: string) {
    // Only clear if leaving the column entirely (not entering a child)
    const related = e.relatedTarget as HTMLElement | null;
    const col = (e.currentTarget as HTMLElement);
    if (!col.contains(related)) {
      if (overColumnId === columnId) overColumnId = null;
    }
  }

  function handleDrop(e: DragEvent, toColumnId: string) {
    e.preventDefault();
    overColumnId = null;

    const itemId = e.dataTransfer?.getData('text/plain') ?? draggingItemId;
    if (!itemId) return;

    const fromColumnId = draggingFromColumn;
    if (fromColumnId === toColumnId) return;

    oncolumnchange?.(itemId, toColumnId);
    draggingItemId = null;
    draggingFromColumn = null;
  }
</script>

<div class="kanban-board" role="region" aria-label="Kanban board">
  {#each columns as column (column.id)}
    <div class="kanban-column">
      <!-- Column header -->
      <div class="kanban-column-header">
        <div class="column-title-wrap">
          <span class="column-title">{column.label}</span>
          <span class="badge badge-neutral column-count">
            {column.count ?? column.items.length}
          </span>
        </div>

        {#if column.totalValue}
          <span class="column-value">{column.totalValue}</span>
        {/if}

        {#if onadddeal}
          <button
            class="icon-btn column-add-btn"
            onclick={() => onadddeal?.(column.id)}
            aria-label="{t('deals.addDeal')} to {column.label}"
            title="{t('deals.addDeal')}"
            type="button"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <path d="M7 1v12M1 7h12"/>
            </svg>
          </button>
        {/if}
      </div>

      <!-- Column body (drop zone) -->
      <div
        class="kanban-column-body"
        class:drop-target={overColumnId === column.id && draggingFromColumn !== column.id}
        role="list"
        aria-label="{column.label} items"
        ondragover={(e) => handleDragOver(e, column.id)}
        ondragleave={(e) => handleDragLeave(e, column.id)}
        ondrop={(e) => handleDrop(e, column.id)}
      >
        {#if column.items.length === 0}
          <EmptyState
            title={t('deals.noDeals')}
            description={t('deals.noDealsDesc')}
            compact={true}
          />
        {:else}
          {#each column.items as item}
            {@const itemId = (item as Record<string, unknown>).id as string}
            <div
              class="kanban-item"
              class:is-dragging={draggingItemId === itemId}
              draggable="true"
              role="listitem"
              ondragstart={(e) => handleDragStart(e, itemId, column.id)}
              ondragend={handleDragEnd}
            >
              {#if card}
                {@render card(item)}
              {:else}
                <div class="kanban-default-card">{itemId}</div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .column-title-wrap {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: 1;
    min-width: 0;
  }

  .column-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .column-count {
    flex-shrink: 0;
  }

  .column-value {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    white-space: nowrap;
    margin-inline-end: var(--space-2);
  }

  .column-add-btn {
    flex-shrink: 0;
  }

  .kanban-item.is-dragging {
    opacity: 0.4;
  }

  .kanban-default-card {
    padding: var(--space-4);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }
</style>
