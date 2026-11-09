<script lang="ts">
  /**
   * StatCard.svelte — Dashboard KPI stat card for 900CRM.
   *
   * Displays an icon, label, formatted value, and optional trend indicator.
   */

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    label,
    value,
    icon = 'bar-chart',
    trend,
    trendLabel = '',
    loading = false,
    accent = false,
  }: {
    label: string;
    value: string | number;
    /** Icon variant name */
    icon?: string;
    /** Positive = up, negative = down, undefined = no trend */
    trend?: number;
    trendLabel?: string;
    loading?: boolean;
    accent?: boolean;
  } = $props();

  // ── Derived ────────────────────────────────────────────────────────────────

  const trendUp   = $derived(trend !== undefined && trend > 0);
  const trendDown = $derived(trend !== undefined && trend < 0);

  function iconPath(name: string): string {
    const paths: Record<string, string> = {
      'bar-chart':   'M18 20V10M12 20V4M6 20v-6',
      'users':       'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',
      'dollar-sign': 'M12 1v22M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6',
      'activity':    'M22 12h-4l-3 9L9 3l-3 9H2',
      'check-square':'M9 11l3 3L22 4M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11',
      'trending-up': 'M23 6l-9.5 9.5-5-5L1 18',
    };
    return paths[name] ?? paths['bar-chart'];
  }
</script>

<div class="stat-card card" class:accent>
  <div class="stat-header">
    <div class="stat-icon" aria-hidden="true">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d={iconPath(icon)} />
      </svg>
    </div>
    <p class="stat-label">{label}</p>
  </div>

  {#if loading}
    <div class="skeleton stat-value-skeleton"></div>
  {:else}
    <p class="stat-value">{value}</p>
  {/if}

  {#if trend !== undefined && trendLabel}
    <p class="stat-trend" class:up={trendUp} class:down={trendDown}>
      {#if trendUp}
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 10V2M2 6l4-4 4 4"/>
        </svg>
      {:else if trendDown}
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 2v8M2 6l4 4 4-4"/>
        </svg>
      {/if}
      {trendLabel}
    </p>
  {/if}
</div>

<style>
  .stat-card {
    padding: var(--space-6) var(--space-8);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    transition: box-shadow var(--duration-fast) var(--ease-out);
  }

  .stat-card:hover {
    box-shadow: var(--shadow-md);
  }

  .stat-card.accent {
    border-color: var(--color-primary-200);
    background-color: var(--color-primary-50);
  }

  :global([data-theme="dark"]) .stat-card.accent {
    border-color: rgba(32, 128, 141, 0.3);
    background-color: rgba(32, 128, 141, 0.08);
  }

  .stat-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .stat-icon {
    color: var(--color-primary-500);
    flex-shrink: 0;
  }

  .stat-label {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .stat-value {
    font-size: var(--text-2xl);
    font-weight: var(--weight-bold);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
    line-height: var(--leading-tight);
  }

  .stat-value-skeleton {
    height: 32px;
    width: 80px;
    border-radius: var(--border-radius-sm);
  }

  .stat-trend {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  .stat-trend.up   { color: var(--color-success-500); }
  .stat-trend.down { color: var(--color-danger-500); }
</style>
