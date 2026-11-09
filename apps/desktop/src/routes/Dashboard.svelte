<script lang="ts">
  /**
   * Dashboard.svelte — Main dashboard view for 900CRM.
   *
   * Displays:
   *   - 4 KPI stat cards (total contacts, active deals, pipeline value, upcoming tasks)
   *   - Daily needs-attention queue (overdue follow-ups, stuck deals, waiting leads)
   *   - Recent activity feed
   *   - Quick action buttons
   */

  import { t } from '$lib/i18n';
  import { listActivities } from '$lib/api/activities';
  import type { Activity } from '$lib/api/activities';
  import { listContacts } from '$lib/api/contacts';
  import type { Contact } from '$lib/api/contacts';
  import { getDashboardStats } from '$lib/api/dashboard';
  import type { DashboardStats } from '$lib/api/dashboard';
  import { listDeals } from '$lib/api/deals';
  import type { Deal } from '$lib/api/deals';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import { formatCurrency } from '$lib/utils/formatters';
  import { navigateHash } from '$lib/utils/hashRouter';
  import {
    buildDashboardAttentionQueue,
    type DashboardAttentionItem,
  } from '$lib/utils/localAutomation';
  import { buildDefaultSampleWorkspaceDeps, seedSampleWorkspace } from '$lib/utils/sampleWorkspace';
  import StatCard from '$lib/components/StatCard.svelte';
  import ActivityFeed from '$lib/components/ActivityFeed.svelte';

  // ── State ───────────────────────────────────────────────────────────────────

  let stats = $state<DashboardStats | null>(null);
  let statsLoading = $state(true);
  let error = $state<string | null>(null);
  let sampleLoading = $state(false);
  let sampleLoaded = $state(false);
  let sampleMessage = $state<string | null>(null);
  let sampleError = $state<string | null>(null);
  let attentionActivities = $state<Activity[]>([]);
  let attentionDeals = $state<Deal[]>([]);
  let attentionLeads = $state<Contact[]>([]);
  let attentionLoading = $state(true);
  let attentionError = $state<string | null>(null);
  let dashboardBootstrapped = false;

  const DASHBOARD_LOAD_TIMEOUT_MS = 8_000;

  // ── Derived ────────────────────────────────────────────────────────────────

  const pipelineFormatted = $derived(
    (() => {
      if (!stats) return '—';
      const buckets = stats.pipelineValueByCurrency.filter((bucket) => bucket.totalValue !== 0);

      if (buckets.length === 0) {
        return formatCurrency(0, settingsStore.currency, settingsStore.language);
      }

      if (buckets.length === 1) {
        return formatCurrency(buckets[0].totalValue, buckets[0].currency, settingsStore.language);
      }

      const preview = buckets
        .slice(0, 2)
        .map((bucket) => formatCurrency(bucket.totalValue, bucket.currency, settingsStore.language))
        .join(' · ');

      return buckets.length > 2 ? `${preview} +${buckets.length - 2}` : preview;
    })()
  );

  const hasFirstRunContacts = $derived((stats?.totalContacts ?? 0) > 0);
  const hasFirstRunDeals = $derived((stats?.activeDeals ?? 0) > 0);
  const hasFirstRunActivities = $derived((stats?.upcomingTasks ?? 0) > 0);

  const showFirstRun = $derived(
    !sampleLoaded && !error && (!hasFirstRunContacts || !hasFirstRunDeals || !hasFirstRunActivities)
  );

  const sampleDataAvailable = $derived(
    !sampleLoaded && !hasFirstRunContacts && !hasFirstRunDeals && !hasFirstRunActivities
  );
  let ownerFilter = $state('');

  const attentionQueue = $derived(buildDashboardAttentionQueue({
    activities: attentionActivities,
    deals: attentionDeals,
    leads: attentionLeads,
    owner: ownerFilter,
  }));
  const hasWorkspaceRecords = $derived(
    sampleLoaded || hasFirstRunContacts || hasFirstRunDeals || hasFirstRunActivities,
  );
  const showAttentionQueue = $derived(
    attentionLoading
    || Boolean(attentionError)
    || attentionQueue.totalCount > 0
    || hasWorkspaceRecords,
  );

  function withTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
    return new Promise((resolve, reject) => {
      const timeout = window.setTimeout(() => {
        reject(new Error(`${label} timed out after ${DASHBOARD_LOAD_TIMEOUT_MS}ms`));
      }, DASHBOARD_LOAD_TIMEOUT_MS);

      promise.then(
        (value) => {
          window.clearTimeout(timeout);
          resolve(value);
        },
        (err) => {
          window.clearTimeout(timeout);
          reject(err);
        },
      );
    });
  }

  async function loadStats(): Promise<void> {
    statsLoading = true;
    error = null;

    try {
      const dashStats = await withTimeout(getDashboardStats(), 'Dashboard stats');
      stats = dashStats;
    } catch (err) {
      error = t('errors.loadFailed');
      console.error('[Dashboard] Stats load error:', err);
    } finally {
      statsLoading = false;
    }
  }

  async function loadUpcomingActivities(): Promise<void> {
    try {
      await withTimeout(activityStore.loadUpcoming(), 'Upcoming activities');
    } catch (err) {
      console.error('[Dashboard] Upcoming activity load error:', err);
    }
  }

  async function loadAttentionQueue(): Promise<void> {
    attentionLoading = true;
    attentionError = null;

    try {
      const activities = await withTimeout(
        listActivities({ sortBy: 'dueDate', sortDir: 'asc', pageSize: 200 }),
        'Dashboard attention activities',
      );
      attentionActivities = activities;

      const [deals, leadPage] = await Promise.all([
        listDeals({ sortBy: 'createdAt', sortDir: 'asc' }),
        listContacts({
          type: 'person',
          lifecycle: 'lead',
          sortBy: 'createdAt',
          sortDir: 'asc',
          pageSize: 100,
        }),
      ]);
      attentionDeals = deals;
      attentionLeads = leadPage.contacts;
    } catch (err) {
      attentionError = t('dashboard.attention.loadFailed');
      console.error('[Dashboard] Attention queue load error:', err);
    } finally {
      attentionLoading = false;
    }
  }

  function openAttentionItem(item: DashboardAttentionItem): void {
    navigateHash(item.href);
  }

  async function refreshDashboard(): Promise<void> {
    await Promise.allSettled([
      loadStats(),
      loadUpcomingActivities(),
      loadAttentionQueue(),
    ]);
  }

  function openDataSettings(openImport: boolean): void {
    navigateHash(openImport ? '/settings/data/import' : '/settings/data');
  }

  async function loadSampleWorkspace(): Promise<void> {
    if (sampleLoading) return;

    sampleLoading = true;
    sampleMessage = null;
    sampleError = null;

    try {
      await seedSampleWorkspace(
        buildDefaultSampleWorkspaceDeps(settingsStore.currency),
      );

      sampleMessage = t('dashboard.firstRun.sampleLoaded');
      sampleLoaded = true;
      uiStore.toastSuccess(sampleMessage);
      await refreshDashboard();
    } catch (err) {
      sampleError = t('dashboard.firstRun.sampleFailed');
      uiStore.toastError(sampleError);
      console.error('[Dashboard] Sample workspace load error:', err);
    } finally {
      sampleLoading = false;
    }
  }

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  $effect(() => {
    if (dashboardBootstrapped) {
      return;
    }

    dashboardBootstrapped = true;
    void refreshDashboard();
  });
</script>

<div class="page-content dashboard-page">
  <!-- Header -->
  <div class="page-header">
    <h1 class="page-title">{t('dashboard.title')}</h1>
    <div class="dashboard-actions">
      <button
        class="btn btn-secondary btn-sm"
        onclick={() => uiStore.openModal('addContact')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('dashboard.addContact')}
      </button>
      <button
        class="btn btn-primary btn-sm"
        onclick={() => uiStore.openModal('addDeal')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('dashboard.addDeal')}
      </button>
    </div>
  </div>

  {#if error}
    <div class="dashboard-error" role="alert">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      {error}
    </div>
  {:else}
    <!-- KPI Cards -->
    <section class="stat-grid" aria-label={t('dashboard.title')}>
      <StatCard
        label={t('dashboard.totalContacts')}
        value={statsLoading ? '…' : String(stats?.totalContacts ?? 0)}
        icon="users"
        loading={statsLoading}
      />
      <StatCard
        label={t('dashboard.activeDeals')}
        value={statsLoading ? '…' : String(stats?.activeDeals ?? 0)}
        icon="bar-chart"
        loading={statsLoading}
      />
      <StatCard
        label={t('dashboard.pipelineValue')}
        value={statsLoading ? '…' : pipelineFormatted}
        icon="dollar-sign"
        loading={statsLoading}
        accent={true}
      />
      <StatCard
        label={t('dashboard.upcomingTasks')}
        value={statsLoading ? '…' : String(stats?.upcomingTasks ?? 0)}
        icon="check-square"
        loading={statsLoading}
      />
    </section>

    {#if showFirstRun}
      <section class="first-run-panel" aria-labelledby="first-run-heading">
        <div class="first-run-copy">
          <span class="first-run-eyebrow">{t('dashboard.firstRun.eyebrow')}</span>
          <h2 class="first-run-title" id="first-run-heading">{t('dashboard.firstRun.title')}</h2>
          <p>{t('dashboard.firstRun.subtitle')}</p>

          {#if sampleMessage}
            <div class="first-run-status success" role="status">{sampleMessage}</div>
          {/if}
          {#if sampleError}
            <div class="first-run-status error" role="alert">{sampleError}</div>
          {/if}
        </div>

        <div class="first-run-checklist" aria-label={t('dashboard.firstRun.checklistLabel')}>
          <div class:complete={hasFirstRunContacts} class="first-run-step">
            <span class="step-dot" aria-hidden="true"></span>
            <div>
              <span>{t('dashboard.firstRun.addContact')}</span>
              <small>{t('dashboard.firstRun.addContactHint')}</small>
            </div>
          </div>
          <div class:complete={hasFirstRunDeals} class="first-run-step">
            <span class="step-dot" aria-hidden="true"></span>
            <div>
              <span>{t('dashboard.firstRun.addDeal')}</span>
              <small>{t('dashboard.firstRun.addDealHint')}</small>
            </div>
          </div>
          <div class:complete={hasFirstRunActivities} class="first-run-step">
            <span class="step-dot" aria-hidden="true"></span>
            <div>
              <span>{t('dashboard.firstRun.addFollowUp')}</span>
              <small>{t('dashboard.firstRun.addFollowUpHint')}</small>
            </div>
          </div>
        </div>

        <div class="first-run-actions" aria-label={t('dashboard.firstRun.actionsLabel')}>
          {#if sampleDataAvailable}
            <button
              class="btn btn-primary btn-sm"
              disabled={sampleLoading}
              onclick={loadSampleWorkspace}
              type="button"
            >
              {sampleLoading ? t('dashboard.firstRun.loadingSample') : t('dashboard.firstRun.loadSample')}
            </button>
          {/if}
          <button class="btn btn-secondary btn-sm" onclick={() => uiStore.openModal('addContact')} type="button">
            {t('dashboard.firstRun.contactAction')}
          </button>
          <button class="btn btn-secondary btn-sm" onclick={() => uiStore.openModal('addDeal')} type="button">
            {t('dashboard.firstRun.dealAction')}
          </button>
          <button class="btn btn-secondary btn-sm" onclick={() => uiStore.openModal('addActivity')} type="button">
            {t('dashboard.firstRun.followUpAction')}
          </button>
        </div>

        <div class="first-run-data" data-testid="first-run-data">
          <div>
            <span>{t('dashboard.firstRun.dataTitle')}</span>
            <small>{t('dashboard.firstRun.dataHint')}</small>
          </div>
          <div class="first-run-data-actions">
            <button class="btn btn-secondary btn-sm" onclick={() => openDataSettings(true)} type="button">
              {t('dashboard.firstRun.importAction')}
            </button>
            <button class="btn btn-secondary btn-sm" onclick={() => openDataSettings(false)} type="button">
              {t('dashboard.firstRun.backupAction')}
            </button>
          </div>
        </div>
      </section>
    {/if}

    <!-- Content grid: activity feed + quick actions -->
    {#if showAttentionQueue}
      <section
        class="attention-strip"
        aria-labelledby="attention-strip-heading"
        data-testid="dashboard-attention-strip"
      >
        <div class="attention-strip-copy">
          <p class="attention-eyebrow">{t('localAutomation.eyebrow')}</p>
          <h2 id="attention-strip-heading">{t('dashboard.attention.title')}</h2>
          {#if attentionLoading}
            <p>{t('dashboard.attention.loading')}</p>
          {:else if attentionError}
            <p>{attentionError}</p>
          {:else if attentionQueue.totalCount === 0}
            <p>{t('dashboard.attention.empty')}</p>
          {:else}
            <p>{t('dashboard.attention.help')}</p>
            <p>
              {t('dashboard.attention.summary', {
                overdue: attentionQueue.overdueCount,
                today: attentionQueue.todayCount,
                deals: attentionQueue.dealCount,
                leads: attentionQueue.leadCount,
              })}
            </p>
          {/if}
          <input
            class="input attention-owner-filter"
            type="search"
            bind:value={ownerFilter}
            placeholder={t('common.filterOwner')}
            aria-label={t('common.filterOwner')}
          />
        </div>

        {#if !attentionLoading && !attentionError && attentionQueue.items.length > 0}
          <ul class="attention-list" aria-label={t('dashboard.attention.listLabel')}>
            {#each attentionQueue.items as item (item.id)}
              <li>
                <button
                  class="attention-item"
                  class:overdue={item.kind === 'overdue'}
                  type="button"
                  onclick={() => openAttentionItem(item)}
                >
                  <span>{item.title}</span>
                  <small>{t(`dashboard.attention.${item.kind}`)}</small>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {/if}

    <div class="dashboard-grid">
      <!-- Recent Activity -->
      <section class="card dashboard-activity" aria-labelledby="activity-heading">
        <div class="card-header">
          <h2 class="section-title" id="activity-heading">
            {t('dashboard.recentActivity')}
          </h2>
        </div>
        <div class="card-body">
          <ActivityFeed
            activities={activityStore.upcoming}
            loading={activityStore.isLoading}
            maxItems={8}
          />
        </div>
      </section>

      <!-- Quick Actions -->
      <section class="card dashboard-quick" aria-labelledby="quick-heading">
        <div class="card-header">
          <h2 class="section-title" id="quick-heading">
            {t('dashboard.quickActions')}
          </h2>
        </div>
        <div class="card-body quick-actions">
          <button
            class="quick-action-btn"
            onclick={() => uiStore.openModal('addContact')}
            type="button"
          >
            <div class="quick-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true">
                <path d="M16 21v-2a4 4 0 00-4-4H6a4 4 0 00-4 4v2M9 11a4 4 0 100-8 4 4 0 000 8M19 8v6M22 11h-6"/>
              </svg>
            </div>
            <span>{t('contacts.addContact')}</span>
          </button>

          <button
            class="quick-action-btn"
            onclick={() => uiStore.openModal('addDeal')}
            type="button"
          >
            <div class="quick-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
              </svg>
            </div>
            <span>{t('deals.addDeal')}</span>
          </button>

          <button
            class="quick-action-btn"
            onclick={() => uiStore.openModal('addActivity')}
            type="button"
          >
            <div class="quick-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true">
                <rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>
              </svg>
            </div>
            <span>{t('activities.addActivity')}</span>
          </button>
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .dashboard-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .dashboard-actions {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  .stat-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-6);
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: var(--space-6);
    align-items: start;
  }

  .first-run-panel {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(220px, 0.8fr);
    gap: var(--space-6);
    align-items: start;
    padding: var(--space-6);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    background: var(--surface-raised);
  }

  .first-run-copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .first-run-eyebrow {
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    text-transform: uppercase;
    letter-spacing: 0;
  }

  .first-run-title {
    margin: 0;
    font-size: var(--text-xl);
    color: var(--text-primary);
  }

  .first-run-copy p {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .first-run-status {
    padding: var(--space-3) var(--space-4);
    border-radius: var(--border-radius-sm);
    font-size: var(--text-sm);
  }

  .first-run-status.success {
    background: var(--color-success-50);
    color: var(--color-success-600);
  }

  .first-run-status.error {
    background: var(--color-danger-50);
    color: var(--color-danger-600);
  }

  .first-run-checklist,
  .first-run-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .first-run-step {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr);
    gap: var(--space-3);
    align-items: start;
    color: var(--text-secondary);
  }

  .first-run-step span:not(.step-dot) {
    display: block;
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .first-run-step small {
    display: block;
    margin-top: 2px;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    line-height: 1.4;
  }

  .step-dot {
    width: 18px;
    height: 18px;
    border-radius: 999px;
    border: var(--border-width) solid var(--border-default);
    background: var(--surface-base);
    margin-top: 1px;
  }

  .first-run-step.complete .step-dot {
    border-color: var(--color-success-500);
    background: var(--color-success-500);
    box-shadow: inset 0 0 0 4px var(--surface-raised);
  }

  .first-run-actions {
    grid-column: 1 / -1;
    flex-direction: row;
    flex-wrap: wrap;
  }

  .first-run-data {
    grid-column: 1 / -1;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    background: var(--surface-default);
  }

  .first-run-data span {
    display: block;
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .first-run-data small {
    display: block;
    margin-top: 2px;
    font-size: var(--text-xs);
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .first-run-data-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .attention-strip {
    display: grid;
    grid-template-columns: minmax(0, 0.9fr) minmax(260px, 1.1fr);
    gap: var(--space-4);
    align-items: start;
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    background: var(--surface-raised);
  }

  .attention-strip-copy {
    display: grid;
    gap: var(--space-1);
    min-width: 0;
  }

  .attention-eyebrow {
    margin: 0;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    text-transform: uppercase;
    letter-spacing: 0;
  }

  .attention-strip h2,
  .attention-strip p {
    margin: 0;
  }

  .attention-owner-filter {
    max-width: 220px;
    height: 34px;
  }

  .attention-strip h2 {
    font-size: var(--text-md);
    color: var(--text-primary);
  }

  .attention-strip p {
    font-size: var(--text-sm);
    line-height: 1.45;
    color: var(--text-secondary);
  }

  .attention-list {
    display: grid;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .attention-list li {
    min-width: 0;
  }

  .attention-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    min-width: 0;
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-sm);
    background: var(--surface-base);
    color: inherit;
    text-align: start;
    cursor: pointer;
  }

  .attention-item:hover,
  .attention-item:focus-visible {
    border-color: var(--color-primary);
  }

  .attention-item.overdue {
    border-color: var(--color-danger-200);
    background: var(--color-danger-50);
  }

  .attention-item span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .attention-item small {
    flex-shrink: 0;
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  @media (max-width: 900px) {
    .dashboard-grid {
      grid-template-columns: 1fr;
    }

    .first-run-panel {
      grid-template-columns: 1fr;
    }

    .first-run-data {
      flex-direction: column;
    }

    .attention-strip {
      grid-template-columns: 1fr;
    }
  }

  .section-title {
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .dashboard-activity .card-body,
  .dashboard-quick .card-body {
    padding-block-start: var(--space-4);
  }

  .quick-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .quick-action-btn {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-5);
    border-radius: var(--border-radius-md);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    background: transparent;
    border: var(--border-width) solid var(--border-default);
    cursor: pointer;
    width: 100%;
    text-align: start;
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .quick-action-btn:hover {
    background-color: var(--surface-active);
    color: var(--text-accent);
    border-color: var(--color-primary-200);
  }

  .quick-icon {
    color: var(--color-primary-500);
    flex-shrink: 0;
  }

  .dashboard-error {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-5) var(--space-6);
    background-color: var(--color-danger-50);
    color: var(--color-danger-600);
    border-radius: var(--border-radius-md);
    font-size: var(--text-sm);
  }

</style>
