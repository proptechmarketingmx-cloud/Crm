<script lang="ts">
  /**
   * DealCard.svelte — Deal card for the Kanban pipeline board.
   *
   * Displays deal name, value, contact, expected close date, and probability.
   * Styled for drag-and-drop with native HTML5 drag API.
   */

  import type { Deal } from '$lib/api/deals';
  import { t } from '$lib/i18n';
  import { formatCurrency, formatDate, formatPercent } from '$lib/utils/formatters';
  import { settingsStore } from '$lib/stores/settings';

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    deal,
    draggable = true,
    onclick,
    ondragstart,
    ondragend,
    primaryContactName = deal.contactName,
    organizationName = null,
    guidanceLabel = null,
    guidanceTone = 'neutral',
  }: {
    deal: Deal;
    draggable?: boolean;
    onclick?: (deal: Deal) => void;
    ondragstart?: (e: DragEvent, deal: Deal) => void;
    ondragend?: () => void;
    primaryContactName?: string | null;
    organizationName?: string | null;
    guidanceLabel?: string | null;
    guidanceTone?: 'neutral' | 'success' | 'danger' | 'warning';
  } = $props();

  // ── Derived ────────────────────────────────────────────────────────────────

  const formattedValue = $derived(
    formatCurrency(deal.value, deal.currency, settingsStore.language)
  );
  const formattedDate = $derived(
    deal.expectedCloseDate
      ? formatDate(deal.expectedCloseDate, settingsStore.dateFormat as 'MMM D, YYYY')
      : null
  );

  function probabilityColor(p: number): string {
    if (p >= 75) return 'var(--color-success-500)';
    if (p >= 40) return 'var(--color-warning-500)';
    return 'var(--color-danger-500)';
  }

  // ── Drag ───────────────────────────────────────────────────────────────────

  let isDragging = $state(false);

  function handleDragStart(e: DragEvent) {
    isDragging = true;
    e.dataTransfer?.setData('text/plain', deal.id);
    e.dataTransfer!.effectAllowed = 'move';
    ondragstart?.(e, deal);
  }

  function handleDragEnd() {
    isDragging = false;
    ondragend?.();
  }
</script>

<button
  class="deal-card card"
  class:dragging={isDragging}
  class:clickable={!!onclick}
  type="button"
  draggable={draggable ? 'true' : 'false'}
  ondragstart={handleDragStart}
  ondragend={handleDragEnd}
  onclick={() => onclick?.(deal)}
  onkeydown={(e) => { if (e.key === 'Enter') onclick?.(deal); }}
>
  <!-- Deal name -->
  <div class="deal-card-header">
    <p class="deal-name">{deal.name}</p>
    {#if guidanceLabel}
      <span class="deal-guidance deal-guidance-{guidanceTone}">
        {guidanceLabel}
      </span>
    {/if}
  </div>

  <!-- Value -->
  <p class="deal-value">{formattedValue}</p>

  <!-- Contact -->
  {#if primaryContactName}
    <div class="deal-meta">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"/>
      </svg>
      <span>{primaryContactName}</span>
    </div>
  {/if}

  {#if deal.owner}
    <div class="deal-meta">
      <span>{t('common.owner')}: {deal.owner}</span>
    </div>
  {/if}

  <!-- Organization -->
  {#if organizationName}
    <div class="deal-meta">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <path d="M3 21h18M5 21V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16M9 7h1M14 7h1M9 11h1M14 11h1M9 15h1M14 15h1"/>
      </svg>
      <span>{organizationName}</span>
    </div>
  {/if}

  <!-- Expected close date -->
  {#if formattedDate}
    <div class="deal-meta">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>
      </svg>
      <span>{formattedDate}</span>
    </div>
  {/if}

  <!-- Probability bar -->
  {#if deal.probability > 0}
    <div class="deal-probability">
      <div class="probability-bar">
        <div
          class="probability-fill"
          style="width: {deal.probability}%; background-color: {probabilityColor(deal.probability)};"
        ></div>
      </div>
      <span class="probability-label">{formatPercent(deal.probability)}</span>
    </div>
  {/if}
</button>

<style>
  .deal-card {
    padding: var(--space-5) var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    appearance: none;
    text-align: start;
    font: inherit;
    color: inherit;
    cursor: grab;
    user-select: none;
    -webkit-user-select: none;
    transition:
      box-shadow var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }

  .deal-card.clickable {
    cursor: grab;
  }

  .deal-card:hover {
    box-shadow: var(--shadow-md);
  }

  .deal-card.dragging {
    opacity: 0.4;
    cursor: grabbing;
  }

  .deal-name {
    min-width: 0;
    margin: 0;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    line-height: var(--leading-snug);
  }

  .deal-card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .deal-guidance {
    flex-shrink: 0;
    padding: 2px var(--space-2);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    line-height: 1.3;
  }

  .deal-guidance-neutral {
    color: var(--text-secondary);
    background-color: var(--surface-raised);
  }

  .deal-guidance-success {
    color: #2D8659;
    border-color: #BFE4CC;
    background-color: #E8F5EC;
  }

  .deal-guidance-warning {
    color: #A84B2F;
    border-color: #F4D1A1;
    background-color: #FEF3E2;
  }

  .deal-guidance-danger {
    color: #C0392B;
    border-color: #F0B8B2;
    background-color: #FFF0F0;
  }

  .deal-value {
    font-size: var(--text-md);
    font-weight: var(--weight-bold);
    color: var(--text-accent);
    letter-spacing: var(--tracking-tight);
  }

  .deal-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  .deal-probability {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-block-start: var(--space-1);
  }

  .probability-bar {
    flex: 1;
    height: 3px;
    background-color: var(--border-default);
    border-radius: 2px;
    overflow: hidden;
  }

  .probability-fill {
    height: 100%;
    border-radius: 2px;
    transition: width var(--duration-normal) var(--ease-out);
  }

  .probability-label {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    flex-shrink: 0;
    min-width: 28px;
    text-align: end;
  }
</style>
