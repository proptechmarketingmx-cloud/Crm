<script lang="ts">
  /**
   * Reports.svelte - Local reports hub for current pipeline and activity health.
   */

  import { t } from '$lib/i18n';
  import { listActivities, type Activity } from '$lib/api/activities';
  import { listDeals, type Deal } from '$lib/api/deals';
  import {
    getActivityFunnelReport,
    getPipelineConversionReport,
    type ActivityDueBuckets,
    type ActivityFunnelReport,
    type ActivityTypeMetric,
    type PipelineConversionReport,
    type PipelineStageMetric,
    type StageTransitionMetric,
  } from '$lib/api/reports';
  import {
    createSavedView,
    deleteSavedView,
    filtersMatch,
    listSavedViews,
    type ContactSavedViewFilters,
    type SavedView,
  } from '$lib/api/savedViews';
  import { settingsStore } from '$lib/stores/settings';
  import { uiStore } from '$lib/stores/ui';
  import { navigateHash } from '$lib/utils/hashRouter';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import {
    loadActivityLinkIndex,
    type ActivityLinkIndex,
  } from '$lib/utils/activityRelationships';
  import { formatCompactNumber, formatPercent } from '$lib/utils/formatters';
  import {
    buildStaleDealReport,
  } from '$lib/utils/staleDealReport';
  import {
    buildReportSnapshotRows,
    defaultReportSnapshotFilename,
    reportSnapshotToCsv,
    type ReportFocus,
  } from '$lib/utils/reportSnapshot';
  import { saveTextFile } from '$lib/utils/saveTextFile';
  import { matchesRecordOwner } from '$lib/utils/recordOwner';

  let pipelineReport = $state<PipelineConversionReport | null>(null);
  let activityReport = $state<ActivityFunnelReport | null>(null);
  let pipelineLoading = $state(true);
  let activityLoading = $state(true);
  let pipelineError = $state<string | null>(null);
  let activityError = $state<string | null>(null);
  let staleSource = $state<{
    deals: Deal[];
    activities: Activity[];
    linkIndex: ActivityLinkIndex;
  } | null>(null);
  let staleLoading = $state(true);
  let staleError = $state<string | null>(null);
  let ownerFilter = $state('');
  let reportsBootstrapped = false;
  let exporting = $state(false);
  let reportFocus = $state<ReportFocus>('');
  let savedViews = $state<SavedView[]>([]);
  let selectedViewId = $state('');
  let viewName = $state('');
  let viewsLoading = $state(false);
  let viewsSaving = $state(false);
  let viewsError = $state<string | null>(null);

  const REPORT_LOAD_TIMEOUT_MS = 8_000;

  const stageLabelKeyMap: Record<string, string> = {
    Lead: 'lead',
    Qualified: 'qualified',
    Proposal: 'proposal',
    Negotiation: 'negotiation',
    'Closed Won': 'closedWon',
    'Closed Lost': 'closedLost',
  };

  const visibleStageMetrics = $derived(
    (pipelineReport?.stage_metrics ?? []).filter((metric) => hasStageMetricData(metric))
  );

  const visibleTransitionMetrics = $derived(
    (pipelineReport?.transition_metrics ?? []).filter((metric) => metric.from_count > 0 || metric.to_count > 0)
  );

  const visibleActivityTypes = $derived(
    (activityReport?.by_type ?? []).filter((metric) => metric.total > 0)
  );

  const dueBucketRows = $derived(
    activityReport ? buildDueBucketRows(activityReport.due_buckets) : []
  );

  const staleReport = $derived(
    staleSource
      ? buildStaleDealReport({
          deals: staleSource.deals.filter((deal) => matchesRecordOwner(deal.owner, ownerFilter)),
          activities: staleSource.activities,
          linkIndex: staleSource.linkIndex,
        })
      : null
  );

  const reportsLoading = $derived(pipelineLoading || activityLoading || staleLoading);

  const showStaleEmpty = $derived(
    !staleLoading && !staleError && staleReport !== null && staleReport.count === 0
  );

  const showPipelineEmpty = $derived(
    !pipelineLoading && !pipelineError && pipelineReport !== null && pipelineReport.total_deals === 0
  );

  const showActivityEmpty = $derived(
    !activityLoading && !activityError && activityReport !== null && activityReport.total_activities === 0
  );

  const currentViewFilters = $derived(collectCurrentFilters());
  const selectedView = $derived(savedViews.find((view) => view.id === selectedViewId) ?? null);
  const canSaveView = $derived(viewName.trim().length > 0 && !viewsSaving);
  const showPipelineSection = $derived(!reportFocus || reportFocus === 'pipeline');
  const showActivitySection = $derived(!reportFocus || reportFocus === 'activity');
  const showStaleSection = $derived(!reportFocus || reportFocus === 'stale');
  const showWorkspaceEmpty = $derived(
    !pipelineLoading
    && !activityLoading
    && !staleLoading
    && !pipelineError
    && !activityError
    && !staleError
    && (pipelineReport?.total_deals ?? 0) === 0
    && (activityReport?.total_activities ?? 0) === 0
  );

  function hasStageMetricData(metric: PipelineStageMetric): boolean {
    return metric.count > 0 || metric.stage_share > 0;
  }

  function buildDueBucketRows(buckets: ActivityDueBuckets) {
    return [
      { id: 'overdue', label: t('reports.activity.dueBuckets.overdue'), value: buckets.overdue },
      { id: 'dueToday', label: t('reports.activity.dueBuckets.dueToday'), value: buckets.due_today },
      { id: 'dueNext7Days', label: t('reports.activity.dueBuckets.dueNext7Days'), value: buckets.due_next_7_days },
      { id: 'dueLater', label: t('reports.activity.dueBuckets.dueLater'), value: buckets.due_later },
      { id: 'noDueDate', label: t('reports.activity.dueBuckets.noDueDate'), value: buckets.no_due_date },
    ];
  }

  function asPercentRatio(ratio: number, decimals = 1): string {
    return formatPercent(Math.max(0, Math.min(100, ratio * 100)), decimals);
  }

  function ratioWidth(ratio: number): string {
    return `${Math.max(0, Math.min(100, ratio * 100))}%`;
  }

  function stageLabel(stage: string): string {
    const key = stageLabelKeyMap[stage];
    return key ? t(`deals.stages.${key}`) : stage;
  }

  function activityTypeLabel(type: string): string {
    const normalized = type.trim().toLowerCase();
    const map: Record<string, string> = {
      task: t('activities.task'),
      call: t('activities.call'),
      meeting: t('activities.meeting'),
      email: t('activities.email'),
      note: t('activities.notes'),
    };
    return map[normalized] ?? type;
  }

  function formatGeneratedAt(iso: string | null | undefined): string {
    if (!iso) return t('reports.notAvailable');
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return t('reports.notAvailable');

    try {
      return date.toLocaleString(settingsStore.language);
    } catch {
      return date.toISOString();
    }
  }

  function formatActivityTypeDetail(metric: ActivityTypeMetric): string {
    return `${asPercentRatio(metric.completion_rate)} / ${formatCompactNumber(metric.completed)}/${formatCompactNumber(metric.total)}`;
  }

  function withTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
    return new Promise((resolve, reject) => {
      const timeout = window.setTimeout(() => {
        reject(new Error(`${label} timed out after ${REPORT_LOAD_TIMEOUT_MS}ms`));
      }, REPORT_LOAD_TIMEOUT_MS);

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

  async function loadPipelineReport(): Promise<void> {
    pipelineLoading = true;
    pipelineError = null;

    try {
      const report = await withTimeout(getPipelineConversionReport(), 'Pipeline report');
      pipelineReport = report;
    } catch (err) {
      pipelineReport = null;
      pipelineError = t('reports.pipeline.loadFailed');
      console.error('[Reports] Pipeline report load error:', err);
    } finally {
      pipelineLoading = false;
    }
  }

  async function loadActivityReport(): Promise<void> {
    activityLoading = true;
    activityError = null;

    try {
      const report = await withTimeout(getActivityFunnelReport(), 'Activity report');
      activityReport = report;
    } catch (err) {
      activityReport = null;
      activityError = t('reports.activity.loadFailed');
      console.error('[Reports] Activity report load error:', err);
    } finally {
      activityLoading = false;
    }
  }

  async function loadStaleReport(): Promise<void> {
    staleLoading = true;
    staleError = null;

    try {
      const [deals, activities] = await withTimeout(
        Promise.all([
          listDeals({ sortBy: 'createdAt', sortDir: 'asc' }),
          listActivities({ sortBy: 'dueDate', sortDir: 'asc', pageSize: 200 }),
        ]),
        'Stale deal report',
      );
      const linkIndex: ActivityLinkIndex = await loadActivityLinkIndex(
        activities.map((activity) => activity.id),
      );
      staleSource = { deals, activities, linkIndex };
    } catch (err) {
      staleSource = null;
      staleError = t('reports.stale.loadFailed');
      console.error('[Reports] Stale deal report load error:', err);
    } finally {
      staleLoading = false;
    }
  }

  async function loadReports(): Promise<void> {
    await Promise.allSettled([
      loadPipelineReport(),
      loadActivityReport(),
      loadStaleReport(),
    ]);
  }

  function asReportFocus(value: string | undefined): ReportFocus {
    return value === 'pipeline' || value === 'activity' || value === 'stale' ? value : '';
  }

  function collectCurrentFilters(): ContactSavedViewFilters {
    return {
      focus: reportFocus || undefined,
      owner: ownerFilter.trim() || undefined,
    };
  }

  async function loadSavedViews(): Promise<void> {
    viewsLoading = true;
    viewsError = null;
    try {
      savedViews = await listSavedViews('report');
      if (selectedViewId && !savedViews.some((view) => view.id === selectedViewId)) {
        selectedViewId = '';
      }
    } catch (error) {
      viewsError = error instanceof Error ? error.message : t('savedViews.loadFailed');
    } finally {
      viewsLoading = false;
    }
  }

  function applyView(view: SavedView): void {
    selectedViewId = view.id;
    reportFocus = asReportFocus(view.filters.focus);
    ownerFilter = view.filters.owner ?? '';
  }

  function syncSelectedView(): void {
    if (!selectedView || filtersMatch(selectedView.filters, currentViewFilters)) {
      return;
    }
    selectedViewId = '';
  }

  async function handleSaveView(): Promise<void> {
    const name = viewName.trim();
    if (!name) {
      return;
    }
    viewsSaving = true;
    viewsError = null;
    try {
      const view = await createSavedView('report', name, collectCurrentFilters());
      savedViews = [...savedViews.filter((item) => item.id !== view.id), view]
        .sort((left, right) => left.name.localeCompare(right.name));
      selectedViewId = view.id;
      viewName = '';
    } catch (error) {
      viewsError = error instanceof Error ? error.message : t('savedViews.saveFailed');
    } finally {
      viewsSaving = false;
    }
  }

  async function handleDeleteView(): Promise<void> {
    if (!selectedView) {
      return;
    }
    if (!window.confirm(t('savedViews.confirmDelete', { name: selectedView.name }))) {
      return;
    }
    viewsSaving = true;
    viewsError = null;
    try {
      await deleteSavedView(selectedView.id);
      savedViews = savedViews.filter((view) => view.id !== selectedView.id);
      selectedViewId = '';
    } catch (error) {
      viewsError = error instanceof Error ? error.message : t('savedViews.deleteFailed');
    } finally {
      viewsSaving = false;
    }
  }

  function handleViewChange(event: Event): void {
    const id = (event.target as HTMLSelectElement).value;
    if (!id) {
      selectedViewId = '';
      return;
    }
    const view = savedViews.find((item) => item.id === id);
    if (view) {
      applyView(view);
    }
  }

  function handleOwnerFilterInput(event: Event): void {
    ownerFilter = (event.target as HTMLInputElement).value;
    syncSelectedView();
  }

  function handleReportFocus(next: ReportFocus): void {
    reportFocus = next;
    syncSelectedView();
  }

  async function handleExportSnapshot(): Promise<void> {
    if (reportsLoading) {
      return;
    }

    const includePipeline = showPipelineSection && pipelineReport !== null;
    const includeActivity = showActivitySection && activityReport !== null;
    const includeStale = showStaleSection && staleReport !== null;
    if (!includePipeline && !includeActivity && !includeStale) {
      uiStore.toastError(t('reports.exportEmpty'));
      return;
    }

    exporting = true;
    try {
      const csv = reportSnapshotToCsv(
        buildReportSnapshotRows({
          focus: reportFocus,
          pipeline: includePipeline ? pipelineReport : null,
          activity: includeActivity ? activityReport : null,
          stale: includeStale ? staleReport : null,
        }),
      );
      const result = await saveTextFile(defaultReportSnapshotFilename(), csv);
      if (result === 'cancelled') {
        return;
      }
      uiStore.toastSuccess(
        result === 'saved' ? t('reports.exportSaved') : t('reports.exportDownloaded'),
      );
    } catch (err) {
      console.error('[Reports] Snapshot export failed:', err);
      uiStore.toastError(t('reports.exportFailed'));
    } finally {
      exporting = false;
    }
  }

  $effect(() => {
    if (reportsBootstrapped) {
      return;
    }

    reportsBootstrapped = true;
    void Promise.all([loadReports(), loadSavedViews()]);
  });
</script>

<div class="page-content reports-page">
  <div class="page-header">
    <div>
      <h1 class="page-title">{t('reports.title')}</h1>
      <p class="page-subtitle">{t('reports.subtitle')}</p>
    </div>
    <div class="header-actions">
      <button class="btn btn-secondary btn-sm" disabled={reportsLoading} onclick={loadReports} type="button">
        {reportsLoading ? t('reports.refreshing') : t('reports.refresh')}
      </button>
      <button
        class="btn btn-primary btn-sm"
        type="button"
        disabled={reportsLoading || exporting}
        onclick={() => void handleExportSnapshot()}
      >
        {exporting ? t('reports.exporting') : t('reports.exportSnapshot')}
      </button>
    </div>
  </div>

  <section class="saved-views" aria-labelledby="reports-saved-views-heading">
    <div class="saved-views-copy">
      <h2 class="saved-views-title" id="reports-saved-views-heading">{t('savedViews.title')}</h2>
      <p class="saved-views-help">{t('savedViews.helpReports')}</p>
    </div>
    <div class="saved-views-controls">
      <select
        class="input saved-views-select"
        value={selectedViewId}
        onchange={handleViewChange}
        aria-label={t('savedViews.selectLabel')}
        disabled={viewsLoading || viewsSaving}
      >
        <option value="">{t('savedViews.none')}</option>
        {#each savedViews as view (view.id)}
          <option value={view.id}>{view.name}</option>
        {/each}
      </select>
      <input
        class="input saved-views-name"
        type="text"
        bind:value={viewName}
        placeholder={t('savedViews.namePlaceholder')}
        aria-label={t('savedViews.nameLabel')}
        disabled={viewsSaving}
      />
      <button
        class="btn btn-secondary btn-sm"
        type="button"
        onclick={() => void handleSaveView()}
        disabled={!canSaveView}
      >
        {viewsSaving ? t('common.loading') : t('savedViews.save')}
      </button>
      <button
        class="btn btn-ghost btn-sm"
        type="button"
        onclick={() => void handleDeleteView()}
        disabled={!selectedView || viewsSaving}
      >
        {t('savedViews.delete')}
      </button>
    </div>
    {#if viewsError}
      <p class="saved-views-error" role="alert">{viewsError}</p>
    {/if}
  </section>

  <div class="report-focus" role="group" aria-label={t('reports.focus')}>
    <input
      class="input report-owner-filter"
      type="search"
      value={ownerFilter}
      oninput={handleOwnerFilterInput}
      placeholder={t('common.filterOwner')}
      aria-label={t('common.filterOwner')}
    />
    <span class="report-focus-label">{t('reports.focus')}:</span>
    <button
      class="filter-chip"
      class:active={!reportFocus}
      type="button"
      onclick={() => handleReportFocus('')}
    >
      {t('reports.focusAll')}
    </button>
    <button
      class="filter-chip"
      class:active={reportFocus === 'pipeline'}
      type="button"
      onclick={() => handleReportFocus('pipeline')}
    >
      {t('reports.focusPipeline')}
    </button>
    <button
      class="filter-chip"
      class:active={reportFocus === 'activity'}
      type="button"
      onclick={() => handleReportFocus('activity')}
    >
      {t('reports.focusActivity')}
    </button>
    <button
      class="filter-chip"
      class:active={reportFocus === 'stale'}
      type="button"
      onclick={() => handleReportFocus('stale')}
    >
      {t('reports.focusStale')}
    </button>
  </div>

  <section class="report-kpi-grid" aria-label={t('reports.summaryLabel')}>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.pipeline.winRate')}</span>
      <strong>{pipelineLoading || !pipelineReport ? '...' : asPercentRatio(pipelineReport.overall_win_rate)}</strong>
      <small>{t('reports.pipeline.winRateDetail')}</small>
    </article>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.pipeline.openDeals')}</span>
      <strong>{pipelineLoading || !pipelineReport ? '...' : formatCompactNumber(pipelineReport.open_deals)}</strong>
      <small>{t('reports.pipeline.openDealsDetail')}</small>
    </article>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.activity.completionRate')}</span>
      <strong>{activityLoading || !activityReport ? '...' : asPercentRatio(activityReport.completion_rate)}</strong>
      <small>{t('reports.activity.completionRateDetail')}</small>
    </article>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.activity.overdueRate')}</span>
      <strong>{activityLoading || !activityReport ? '...' : asPercentRatio(activityReport.overdue_rate)}</strong>
      <small>{t('reports.activity.overdueRateDetail')}</small>
    </article>
    <article class="card report-kpi">
      <span class="report-kpi-label">{t('reports.stale.count')}</span>
      <strong>{staleLoading || !staleReport ? '...' : formatCompactNumber(staleReport.count)}</strong>
      <small>
        {t('reports.stale.countDetail', { days: staleReport?.staleDays ?? 14 })}
      </small>
    </article>
  </section>

  {#if showWorkspaceEmpty}
    <div data-testid="reports-first-run">
      <EmptyState
        icon="deals"
        title={t('reports.emptyWorkspaceTitle')}
        description={t('reports.emptyWorkspaceDesc')}
        actionLabel={t('reports.emptyWorkspaceAction')}
        onaction={() => uiStore.openModal('addDeal')}
      />
    </div>
  {/if}

  <div class="reports-grid">
    {#if showPipelineSection}
    <section class="card report-section" aria-labelledby="pipeline-report-heading" data-testid="pipeline-report">
      <div class="report-section-header">
        <div>
          <h2 class="section-title" id="pipeline-report-heading">{t('reports.pipeline.title')}</h2>
          <p>{t('reports.pipeline.description')}</p>
        </div>
        <span>{t('reports.generatedAt', { value: formatGeneratedAt(pipelineReport?.generated_at) })}</span>
      </div>

      {#if pipelineError}
        <div class="report-alert" role="status">{pipelineError}</div>
      {:else if pipelineLoading && !pipelineReport}
        <div class="report-loading" role="status">{t('reports.loading')}</div>
      {:else if showPipelineEmpty}
        <p class="report-empty">{t('reports.pipeline.empty')}</p>
      {:else if pipelineReport}
        <div class="summary-row" aria-label={t('reports.pipeline.summaryLabel')}>
          <div>
            <span>{t('reports.pipeline.closedWon')}</span>
            <strong>{formatCompactNumber(pipelineReport.closed_won)}</strong>
          </div>
          <div>
            <span>{t('reports.pipeline.closedLost')}</span>
            <strong>{formatCompactNumber(pipelineReport.closed_lost)}</strong>
          </div>
          <div>
            <span>{t('reports.pipeline.totalDeals')}</span>
            <strong>{formatCompactNumber(pipelineReport.total_deals)}</strong>
          </div>
        </div>

        <div class="report-block">
          <h3>{t('reports.pipeline.stageDistribution')}</h3>
          {#if visibleStageMetrics.length === 0}
            <p class="report-empty">{t('reports.pipeline.emptyStages')}</p>
          {:else}
            <ul class="metric-list" role="list">
              {#each visibleStageMetrics as metric (metric.stage)}
                <li class="metric-row">
                  <div class="metric-row-header">
                    <span class="metric-label">{stageLabel(metric.stage)}</span>
                    <span class="metric-value">
                      {formatCompactNumber(metric.count)} / {asPercentRatio(metric.stage_share)}
                    </span>
                  </div>
                  <div class="metric-bar-track">
                    <span class="metric-bar-fill" style={`width: ${ratioWidth(metric.stage_share)}`}></span>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        </div>

        <div class="report-block">
          <h3>{t('reports.pipeline.currentFunnel')}</h3>
          <p class="report-note">{t('reports.pipeline.currentFunnelNote')}</p>
          {#if visibleTransitionMetrics.length === 0}
            <p class="report-empty">{t('reports.pipeline.emptyFunnel')}</p>
          {:else}
            <ul class="funnel-list" role="list">
              {#each visibleTransitionMetrics as metric (metric.from_stage + metric.to_stage)}
                <li class="funnel-row">
                  <span>{stageLabel(metric.from_stage)} -> {stageLabel(metric.to_stage)}</span>
                  <strong>{asPercentRatio(metric.ratio)}</strong>
                  <small>{formatCompactNumber(metric.to_count)} / {formatCompactNumber(metric.from_count)}</small>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </section>
    {/if}

    {#if showActivitySection}
    <section class="card report-section" aria-labelledby="activity-report-heading" data-testid="activity-report">
      <div class="report-section-header">
        <div>
          <h2 class="section-title" id="activity-report-heading">{t('reports.activity.title')}</h2>
          <p>{t('reports.activity.description')}</p>
        </div>
        <span>{t('reports.generatedAt', { value: formatGeneratedAt(activityReport?.generated_at) })}</span>
      </div>

      {#if activityError}
        <div class="report-alert" role="status">{activityError}</div>
      {:else if activityLoading && !activityReport}
        <div class="report-loading" role="status">{t('reports.loading')}</div>
      {:else if showActivityEmpty}
        <p class="report-empty">{t('reports.activity.empty')}</p>
      {:else if activityReport}
        <div class="summary-row" aria-label={t('reports.activity.summaryLabel')}>
          <div>
            <span>{t('reports.activity.completed')}</span>
            <strong>{formatCompactNumber(activityReport.completed_activities)}</strong>
          </div>
          <div>
            <span>{t('reports.activity.pending')}</span>
            <strong>{formatCompactNumber(activityReport.pending_activities)}</strong>
          </div>
          <div>
            <span>{t('reports.activity.overdue')}</span>
            <strong>{formatCompactNumber(activityReport.overdue_activities)}</strong>
          </div>
        </div>

        <div class="report-block">
          <h3>{t('reports.activity.dueBuckets.title')}</h3>
          <ul class="bucket-grid" role="list">
            {#each dueBucketRows as bucket (bucket.id)}
              <li>
                <span>{bucket.label}</span>
                <strong>{formatCompactNumber(bucket.value)}</strong>
              </li>
            {/each}
          </ul>
        </div>

        <div class="report-block">
          <h3>{t('reports.activity.byType')}</h3>
          {#if visibleActivityTypes.length === 0}
            <p class="report-empty">{t('reports.activity.emptyTypes')}</p>
          {:else}
            <ul class="metric-list" role="list">
              {#each visibleActivityTypes as metric (metric.activity_type)}
                <li class="metric-row">
                  <div class="metric-row-header">
                    <span class="metric-label">{activityTypeLabel(metric.activity_type)}</span>
                    <span class="metric-value">
                      {formatActivityTypeDetail(metric)}
                    </span>
                  </div>
                  <div class="metric-bar-track">
                    <span class="metric-bar-fill" style={`width: ${ratioWidth(metric.completion_rate)}`}></span>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </section>
    {/if}
  </div>

  {#if showStaleSection}
  <section
    class="card report-section stale-report"
    aria-labelledby="stale-report-heading"
    data-testid="stale-deal-report"
  >
    <div class="report-section-header">
      <div>
        <h2 class="section-title" id="stale-report-heading">{t('reports.stale.title')}</h2>
        <p>{t('reports.stale.description', { days: staleReport?.staleDays ?? 14 })}</p>
      </div>
    </div>

    {#if staleError}
      <div class="report-alert" role="status">{staleError}</div>
    {:else if staleLoading && !staleReport}
      <div class="report-loading" role="status">{t('reports.loading')}</div>
    {:else if showStaleEmpty}
      <p class="report-empty">{t('reports.stale.empty')}</p>
    {:else if staleReport}
      <div class="table-scroll">
        <table class="stale-deal-table" aria-label={t('reports.stale.listLabel')}>
          <thead>
            <tr>
              <th>{t('reports.stale.deal')}</th>
              <th>{t('reports.stale.stage')}</th>
              <th>{t('common.owner')}</th>
              <th>{t('reports.stale.quietFor')}</th>
              <th>{t('reports.stale.nextStep')}</th>
            </tr>
          </thead>
          <tbody>
            {#each staleReport.rows as row (row.dealId)}
              <tr>
                <td>
                  <button
                    class="link-button"
                    type="button"
                    onclick={() => navigateHash(row.href)}
                  >
                    {row.name}
                  </button>
                </td>
                <td>{t(`deals.stages.${row.stage}`)}</td>
                <td>{row.owner ?? t('common.none')}</td>
                <td>{t('reports.stale.days', { count: row.stageAgeDays })}</td>
                <td>{row.nextActivitySubject ?? t('common.none')}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>
  {/if}
</div>

<style>
  .reports-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-4);
  }

  .header-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .page-subtitle {
    margin: var(--space-2) 0 0;
    max-width: 760px;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .saved-views,
  .saved-views-copy,
  .saved-views-controls,
  .report-focus {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    flex-wrap: wrap;
  }

  .saved-views {
    justify-content: space-between;
  }

  .saved-views-title {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .report-owner-filter {
    min-width: 160px;
    height: 34px;
  }

  .saved-views-help,
  .saved-views-error {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .saved-views-error {
    color: var(--text-danger);
    width: 100%;
  }

  .saved-views-select,
  .saved-views-name {
    min-width: 160px;
  }

  .report-focus-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .filter-chip {
    padding: var(--space-1) var(--space-3);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    background-color: transparent;
    border: var(--border-width) solid var(--border-default);
    cursor: pointer;
  }

  .filter-chip.active {
    background-color: var(--surface-active);
    color: var(--text-accent);
    border-color: var(--color-primary-200);
  }

  .report-kpi-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: var(--space-4);
  }

  .report-kpi {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-5);
  }

  .report-kpi-label {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    text-transform: uppercase;
    letter-spacing: 0;
  }

  .report-kpi strong {
    color: var(--text-primary);
    font-size: var(--text-2xl);
    line-height: 1;
  }

  .report-kpi small {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .reports-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-6);
    align-items: start;
  }

  .report-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-6);
  }

  .report-section-header {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
    align-items: flex-start;
  }

  .report-section-header p {
    margin: var(--space-2) 0 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .report-section-header > span {
    flex-shrink: 0;
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    text-align: end;
  }

  .section-title {
    margin: 0;
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .summary-row {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .summary-row div,
  .bucket-grid li {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-3);
    border-radius: var(--border-radius-sm);
    background: var(--surface-hover);
  }

  .summary-row span,
  .bucket-grid span {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
  }

  .summary-row strong,
  .bucket-grid strong {
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .report-block {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .report-block h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .report-note {
    margin: 0;
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .metric-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .metric-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .metric-row-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    font-size: var(--text-xs);
  }

  .metric-label {
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .metric-value {
    color: var(--text-primary);
    font-weight: var(--weight-medium);
    text-align: end;
  }

  .metric-bar-track {
    width: 100%;
    height: 6px;
    border-radius: 999px;
    background: var(--surface-hover);
    overflow: hidden;
  }

  .metric-bar-fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--color-primary-500);
  }

  .funnel-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .funnel-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    gap: var(--space-3);
    align-items: center;
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-sm);
    font-size: var(--text-xs);
  }

  .funnel-row span {
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .funnel-row strong {
    color: var(--text-primary);
    font-weight: var(--weight-semibold);
  }

  .funnel-row small {
    color: var(--text-tertiary);
  }

  .bucket-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: var(--space-3);
  }

  .report-alert,
  .report-loading,
  .report-empty {
    margin: 0;
    padding: var(--space-4);
    border-radius: var(--border-radius-sm);
    font-size: var(--text-sm);
  }

  .report-alert {
    background: var(--color-warning-50);
    color: var(--color-warning-600);
  }

  .report-loading,
  .report-empty {
    background: var(--surface-hover);
    color: var(--text-tertiary);
  }

  .stale-deal-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .stale-deal-table th,
  .stale-deal-table td {
    padding: var(--space-3) 0;
    border-block-end: var(--border-width) solid var(--border-subtle);
    text-align: start;
    vertical-align: top;
  }

  .stale-deal-table th {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
  }

  .link-button {
    padding: 0;
    border: 0;
    background: none;
    color: var(--color-primary-600);
    font: inherit;
    font-weight: var(--weight-semibold);
    text-align: start;
    text-decoration: underline;
    cursor: pointer;
  }

  @media (max-width: 1100px) {
    .reports-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .report-section-header,
    .page-header {
      flex-direction: column;
    }

    .report-section-header > span {
      text-align: start;
    }

    .summary-row {
      grid-template-columns: 1fr;
    }

    .funnel-row {
      grid-template-columns: 1fr auto;
    }

    .funnel-row small {
      grid-column: 1 / -1;
    }
  }
</style>
