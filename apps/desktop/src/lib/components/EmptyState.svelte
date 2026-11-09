<script lang="ts">
  /**
   * EmptyState.svelte — Friendly empty state placeholder for 900CRM.
   *
   * Renders an icon, title, description, and optional CTA button
   * when a list or view has no content.
   *
   * @example
   * <EmptyState
   *   icon="contacts"
   *   title={t('contacts.noContacts')}
   *   description={t('contacts.noContactsDesc')}
   *   actionLabel={t('contacts.addContact')}
   *   onaction={() => openModal('addContact')}
   * />
   */

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    icon = 'inbox',
    title,
    description = '',
    actionLabel = '',
    onaction,
    compact = false,
  }: {
    /** Icon variant: 'contacts' | 'deals' | 'activities' | 'inbox' | 'search' */
    icon?: string;
    title: string;
    description?: string;
    actionLabel?: string;
    onaction?: () => void;
    compact?: boolean;
  } = $props();

  // ── Icon paths ─────────────────────────────────────────────────────────────

  function iconPath(name: string): string {
    const paths: Record<string, string> = {
      contacts:
        'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zm8 2a2 2 0 1 1 0-4 2 2 0 0 1 0 4zm4 8v-2a4 4 0 0 0-3-3.87',
      deals:
        'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5',
      activities:
        'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2m-3 7 2 2 4-4',
      inbox:
        'M4 4h16v2.92L12 13 4 6.92V4zM4 9.08V20h16V9.08l-8 6.08-8-6.08z',
      search:
        'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zm4.95-3.05 4.24 4.24',
    };
    return paths[name] ?? paths.inbox;
  }
</script>

<div class="empty-state" class:compact>
  <div class="empty-icon" aria-hidden="true">
    <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <path d={iconPath(icon)} />
    </svg>
  </div>

  <p class="empty-title">{title}</p>

  {#if description}
    <p class="empty-description">{description}</p>
  {/if}

  {#if actionLabel && onaction}
    <button class="btn btn-primary empty-action" onclick={onaction} type="button">
      {actionLabel}
    </button>
  {/if}
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-20) var(--space-12);
    text-align: center;
    gap: var(--space-4);
  }

  .empty-state.compact {
    padding: var(--space-10) var(--space-8);
    gap: var(--space-3);
  }

  .empty-icon {
    color: var(--text-tertiary);
    margin-block-end: var(--space-2);
  }

  .compact .empty-icon svg {
    width: 28px;
    height: 28px;
  }

  .empty-title {
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
  }

  .compact .empty-title {
    font-size: var(--text-sm);
  }

  .empty-description {
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    max-width: 320px;
    line-height: var(--leading-relaxed);
  }

  .empty-action {
    margin-block-start: var(--space-4);
  }
</style>
