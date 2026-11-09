<script lang="ts">
  /**
   * Pipeline.svelte — Kanban pipeline board for 900CRM.
   *
   * Renders deal cards grouped by pipeline stage using the KanbanBoard
   * component. Each column shows deal count and total value. Add deal
   * button per column triggers addDeal modal with the stage pre-selected.
   *
   * Drag-and-drop uses the native HTML5 drag API on this board.
   * Stage moves call dealStore.moveDealStage() with optimistic updates.
   */

  import { t } from '$lib/i18n';
  import { dealStore } from '$lib/stores/deals';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import { listActivitiesForDeals, type Activity } from '$lib/api/activities';
  import type { Contact } from '$lib/api/contacts';
  import { DEAL_STAGES, type Deal, type DealStage } from '$lib/api/deals';
  import VirtualList from '$lib/components/VirtualList.svelte';
  import type { Organization } from '$lib/api/organizations';
  import {
    filterActivitiesByRelationship,
    loadActivityLinkIndex,
    loadActivityRelationshipLookups,
    relationshipLabelsByActivityId,
    sortActivitiesForDetailTimeline,
    type ActivityLinkIndex,
    type ActivityRelationshipLabels,
    type ActivityRelationshipLookups,
  } from '$lib/utils/activityRelationships';
  import {
    listCustomFieldDefinitions,
    listCustomFieldValuesForEntityType,
    type CustomFieldDefinition,
    type EntityTypeCustomFieldValue,
  } from '$lib/api/customFields';
  import { formatCurrency, formatPercent } from '$lib/utils/formatters';
  import { sumByCurrency } from '$lib/utils/currency';
  import {
    deriveDealRelationshipLabels,
    loadDealRelationshipLookups,
  } from '$lib/utils/dealRelationships';
  import {
    derivePipelineGuidance,
    type PipelineGuidance,
    type PipelineGuidanceTone,
  } from '$lib/utils/pipelineGuidance';
  import {
    buildDealStageFollowUpSuggestion,
    type DealStageFollowUpSuggestion,
  } from '$lib/utils/localAutomation';
  import {
    buildPipelineForecastMetrics,
    type StageForecastMetric,
    type StageFocus,
  } from '$lib/utils/pipelineMetrics';
  import { isLeavingDropTarget, pipelineDropKind } from '$lib/utils/pipelineDrag';
  import DealCard from '$lib/components/DealCard.svelte';
  import DealDetailDrawer from '$lib/components/DealDetailDrawer.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { navigateHash } from '$lib/utils/hashRouter';
  import { matchesRecordOwner } from '$lib/utils/recordOwner';
  import {
    createSavedView,
    deleteSavedView,
    filtersMatch,
    listSavedViews,
    type ContactSavedViewFilters,
    type SavedView,
  } from '$lib/api/savedViews';

  let { dealId = null }: { dealId?: string | null } = $props();

  // ── State ────────────────────────────────────────────────────────────────────

  /** ID of the deal being dragged. */
  let draggingId = $state<string | null>(null);

  /** Stage the dragged deal started in. */
  let draggingFromStage = $state<DealStage | null>(null);

  /** Stage column currently being dragged over. */
  let dragOverStage = $state<DealStage | null>(null);
  let moveAnnouncement = $state('');
  let customFieldDefinitions = $state<CustomFieldDefinition[]>([]);
  let selectedCustomFieldDefId = $state('');
  let customFieldQuery = $state('');
  let customFieldsLoading = $state(true);
  let customFieldValuesLoading = $state(false);
  let customFieldFilterError = $state<string | null>(null);
  let customFieldValueIndex = $state<Record<string, Record<string, string>>>({});
  let relationshipContacts = $state<Contact[]>([]);
  let relationshipOrganizations = $state<Organization[]>([]);
  let allActivities = $state<Activity[]>([]);
  let activityLinkIndex = $state<ActivityLinkIndex>({});
  let activityRelationshipLookups = $state<ActivityRelationshipLookups>({
    contacts: [],
    organizations: [],
    deals: [],
  });
  let activityContextReady = $state(false);
  let activityContextError = $state<string | null>(null);
  let dealActivitiesLoading = $state(false);
  let selectedDeal = $state<Deal | null>(null);
  let lastActivityRefreshVersion = $state(-1);
  let stageFollowUpSuggestion = $state<DealStageFollowUpSuggestion | null>(null);
  let pipelineBootstrapped = false;
  let pipelineBootstrapComplete = $state(false);
  let suppressNextCardClick = false;
  let searchQuery = $state('');
  let ownerFilter = $state('');
  let attentionFilter = $state<'' | 'needsFollowUp' | 'stale' | 'overdue'>('');
  let savedViews = $state<SavedView[]>([]);
  let selectedViewId = $state('');
  let viewName = $state('');
  let viewsLoading = $state(false);
  let viewsSaving = $state(false);
  let viewsError = $state<string | null>(null);

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  $effect(() => {
    if (pipelineBootstrapped) {
      return;
    }

    pipelineBootstrapped = true;
    void (async () => {
      lastActivityRefreshVersion = activityStore.relationshipRefreshVersion;
      await dealStore.loadPipelineBoard();
      await Promise.all([
        ensureRelationshipLookups(),
        loadCustomFieldDefinitions(),
        loadPipelineActivityContext(),
        loadSavedViews(),
      ]);
      lastActivityRefreshVersion = activityStore.relationshipRefreshVersion;
      pipelineBootstrapComplete = true;
    })();
  });

  $effect(() => {
    if (!pipelineBootstrapComplete) {
      return;
    }

    const version = activityStore.relationshipRefreshVersion;
    if (lastActivityRefreshVersion === version) {
      return;
    }

    lastActivityRefreshVersion = version;
    if (version > 0) {
      void loadPipelineActivityContext();
    }
  });

  // ── Drag & drop handlers ────────────────────────────────────────────────────

  function handleDragStart(dealId: string, stage: DealStage, event?: DragEvent) {
    draggingId = dealId;
    draggingFromStage = stage;
    if (event?.dataTransfer) {
      event.dataTransfer.setData('text/plain', dealId);
      event.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDragEnd() {
    draggingId = null;
    draggingFromStage = null;
    dragOverStage = null;
    setTimeout(() => {
      suppressNextCardClick = false;
    }, 0);
  }

  function handleDragOver(event: DragEvent, stage: DealStage) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = draggingFromStage === stage ? 'none' : 'move';
    }
    dragOverStage = stage;
  }

  function handleDragLeave(event: DragEvent) {
    if (isLeavingDropTarget(event)) {
      dragOverStage = null;
    }
  }

  async function handleDrop(event: DragEvent, toStage: DealStage) {
    event.preventDefault();
    const fromStage = draggingFromStage;
    const id = draggingId ?? event.dataTransfer?.getData('text/plain')?.trim() ?? '';
    dragOverStage = null;
    draggingId = null;
    draggingFromStage = null;
    if (!id || !fromStage || fromStage === toStage) {
      return;
    }

    suppressNextCardClick = true;
    setTimeout(() => {
      suppressNextCardClick = false;
    }, 0);

    const movingDeal = allDeals.find((deal) => deal.id === id);
    if (!movingDeal) return;

    await dealStore.moveDealStage(id, toStage);

    const announcement = t('deals.board.moved', {
      deal: movingDeal.name,
      stage: t(`deals.stages.${toStage}`),
    });
    moveAnnouncement = announcement;
    uiStore.toastSuccess(announcement);

    const suggestion = buildDealStageFollowUpSuggestion({
      deal: { ...movingDeal, stage: toStage },
      activities: dealActivitiesById[id] ?? [],
      fromStage,
      toStage,
      activityContextReady,
      activityContextError: Boolean(activityContextError),
    });

    if (suggestion) {
      stageFollowUpSuggestion = suggestion;
    } else if (stageFollowUpSuggestion?.dealId === id) {
      stageFollowUpSuggestion = null;
    }
  }

  // ── Add deal ─────────────────────────────────────────────────────────────────

  /**
   * Open the add deal modal pre-selecting a stage.
   * The modal is responsible for reading uiStore.modalContext.stage.
   */
  function openAddDeal(stage: DealStage) {
    uiStore.openModal('addDeal', { stage });
  }

  async function ensureRelationshipLookups() {
    try {
      const lookups = await loadDealRelationshipLookups();
      relationshipContacts = lookups.contacts;
      relationshipOrganizations = lookups.organizations;
    } catch (err) {
      console.error('[Pipeline] Failed to load deal relationship lookups:', err);
      uiStore.toastError(t('errors.loadRelationships', { name: t('entities.deal') }));
    }
  }

  async function loadPipelineActivityContext() {
    dealActivitiesLoading = true;
    activityContextReady = false;
    activityContextError = null;
    try {
      const deals = Object.values(dealStore.dealsByStage).flat();
      const activities = await listActivitiesForDeals(deals.map((deal) => deal.id));
      const linkIndex = await loadActivityLinkIndex(activities.map((activity) => activity.id));

      allActivities = activities;
      activityLinkIndex = linkIndex;
      activityContextReady = true;

      try {
        activityRelationshipLookups = await loadActivityRelationshipLookups();
      } catch (lookupErr) {
        console.error('[Pipeline] Failed to load activity relationship labels:', lookupErr);
      }
    } catch (err) {
      console.error('[Pipeline] Failed to load deal activity context:', err);
      activityContextError = t('deals.guidance.activityContextFailed');
      uiStore.toastError(activityContextError);
    } finally {
      dealActivitiesLoading = false;
    }
  }

  async function loadCustomFieldDefinitions() {
    customFieldsLoading = true;
    try {
      customFieldDefinitions = await listCustomFieldDefinitions('deal');
    } finally {
      customFieldsLoading = false;
    }
  }

  async function ensureCustomFieldValueIndex() {
    customFieldValuesLoading = true;
    customFieldFilterError = null;
    try {
      const values = await listCustomFieldValuesForEntityType('deal');
      customFieldValueIndex = indexCustomFieldValues(values);
    } catch (err) {
      customFieldFilterError = t('common.filterLoadFailed');
      console.error('[Pipeline] Failed to load custom-field filter values:', err);
    } finally {
      customFieldValuesLoading = false;
    }
  }

  function indexCustomFieldValues(values: EntityTypeCustomFieldValue[]): Record<string, Record<string, string>> {
    const index: Record<string, Record<string, string>> = {};
    for (const value of values) {
      if (!index[value.field_def_id]) {
        index[value.field_def_id] = {};
      }
      index[value.field_def_id][value.entity_id] = value.value;
    }
    return index;
  }

  async function handleCustomFieldDefinitionChange(event: Event) {
    selectedCustomFieldDefId = (event.target as HTMLSelectElement).value;
    if (selectedCustomFieldDefId && customFieldQuery.trim()) {
      await ensureCustomFieldValueIndex();
    }
    syncSelectedView();
  }

  async function handleCustomFieldQueryInput(event: Event) {
    customFieldQuery = (event.target as HTMLInputElement).value;
    if (selectedCustomFieldDefId && customFieldQuery.trim()) {
      await ensureCustomFieldValueIndex();
    }
    syncSelectedView();
  }

  function clearCustomFieldFilter() {
    selectedCustomFieldDefId = '';
    customFieldQuery = '';
    syncSelectedView();
  }

  function collectCurrentFilters(): ContactSavedViewFilters {
    return {
      search: searchQuery.trim() || undefined,
      owner: ownerFilter.trim() || undefined,
      customFieldDefId: selectedCustomFieldDefId || undefined,
      customFieldQuery: customFieldQuery.trim() || undefined,
      attention: attentionFilter || undefined,
    };
  }

  const currentViewFilters = $derived(collectCurrentFilters());
  const selectedView = $derived(savedViews.find((view) => view.id === selectedViewId) ?? null);
  const canSaveView = $derived(viewName.trim().length > 0 && !viewsSaving);

  async function loadSavedViews(): Promise<void> {
    viewsLoading = true;
    viewsError = null;
    try {
      savedViews = await listSavedViews('deal');
      if (selectedViewId && !savedViews.some((view) => view.id === selectedViewId)) {
        selectedViewId = '';
      }
    } catch (error) {
      viewsError = error instanceof Error ? error.message : t('savedViews.loadFailed');
    } finally {
      viewsLoading = false;
    }
  }

  async function applyView(view: SavedView): Promise<void> {
    selectedViewId = view.id;
    searchQuery = view.filters.search ?? '';
    ownerFilter = view.filters.owner ?? '';
    selectedCustomFieldDefId = view.filters.customFieldDefId ?? '';
    customFieldQuery = view.filters.customFieldQuery ?? '';
    attentionFilter = view.filters.attention ?? '';
    if (selectedCustomFieldDefId && customFieldQuery.trim()) {
      await ensureCustomFieldValueIndex();
    }
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
      const view = await createSavedView('deal', name, collectCurrentFilters());
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
      void applyView(view);
    }
  }

  function handleSearchInput(event: Event): void {
    searchQuery = (event.target as HTMLInputElement).value;
    syncSelectedView();
  }

  function handleOwnerFilterInput(event: Event): void {
    ownerFilter = (event.target as HTMLInputElement).value;
    syncSelectedView();
  }

  function handleAttentionFilter(next: '' | 'needsFollowUp' | 'stale' | 'overdue'): void {
    attentionFilter = next;
    syncSelectedView();
  }

  function matchesSearch(deal: Deal): boolean {
    const query = searchQuery.trim().toLowerCase();
    if (!query) {
      return true;
    }
    return deal.name.toLowerCase().includes(query);
  }

  function matchesOwner(deal: Deal): boolean {
    return matchesRecordOwner(deal.owner, ownerFilter);
  }

  function matchesCustomField(dealId: string): boolean {
    const query = customFieldQuery.trim().toLowerCase();
    if (!selectedCustomFieldDefId || !query) {
      return true;
    }
    if (customFieldValuesLoading || customFieldFilterError) {
      return true;
    }

    const rawValue = customFieldValueIndex[selectedCustomFieldDefId]?.[dealId] ?? '';
    return rawValue.toLowerCase().includes(query);
  }

  function matchesAttention(deal: Deal): boolean {
    if (!attentionFilter) {
      return true;
    }
    if (!activityContextReady || activityContextError) {
      return true;
    }
    return guidanceByDealId[deal.id]?.state === attentionFilter;
  }

  // ── Derived ─────────────────────────────────────────────────────────────────

  const allDeals = $derived.by(() =>
    DEAL_STAGES.flatMap((stage) => dealStore.dealsByStage[stage] ?? [])
  );

  $effect(() => {
    if (!pipelineBootstrapComplete || !dealId) {
      return;
    }

    const deal = allDeals.find((item) => item.id === dealId);
    if (deal && selectedDeal?.id !== deal.id) {
      selectedDeal = deal;
      dealStore.selectDeal(deal);
    }
  });

  const dealActivitiesById = $derived.by<Record<string, Activity[]>>(() => {
    if (!activityContextReady) {
      return {};
    }

    return Object.fromEntries(
      allDeals.map((deal) => [
        deal.id,
        sortActivitiesForDetailTimeline(
          filterActivitiesByRelationship(allActivities, activityLinkIndex, {
            dealId: deal.id,
          })
        ),
      ])
    );
  });

  const selectedDealActivities = $derived.by(() =>
    selectedDeal && activityContextReady ? dealActivitiesById[selectedDeal.id] ?? [] : []
  );

  const selectedDealRelationshipLabels = $derived.by<Record<string, ActivityRelationshipLabels>>(() =>
    relationshipLabelsByActivityId(
      selectedDealActivities,
      activityLinkIndex,
      activityRelationshipLookups,
    )
  );

  const selectedDealRelationshipNames = $derived.by(() =>
    selectedDeal
      ? deriveDealRelationshipLabels(
          selectedDeal,
          relationshipContacts,
          relationshipOrganizations,
        )
      : { primaryContactName: null, organizationName: null }
  );

  const guidanceByDealId = $derived.by<Record<string, PipelineGuidance>>(() => {
    if (!activityContextReady || activityContextError) {
      return {};
    }

    return Object.fromEntries(
      allDeals.map((deal) => [
        deal.id,
        derivePipelineGuidance({
          deal,
          activities: dealActivitiesById[deal.id] ?? [],
        }),
      ])
    );
  });

  const columns = $derived(
    DEAL_STAGES.map((stage) => {
      const deals = (dealStore.dealsByStage[stage] ?? []).filter(
        (deal) => matchesSearch(deal) && matchesOwner(deal) && matchesCustomField(deal.id) && matchesAttention(deal),
      );
      const currencyTotals = sumByCurrency(
        deals.map((deal) => ({ currency: deal.currency, value: deal.value }))
      );
      return { stage, deals, currencyTotals };
    })
  );

  const visibleDeals = $derived.by(() => columns.flatMap((column) => column.deals));

  const pipelineMetrics = $derived.by(() =>
    buildPipelineForecastMetrics({
      deals: visibleDeals,
      guidanceByDealId,
    })
  );

  const pipelineFocusSummary = $derived.by(() => {
    if (pipelineMetrics.openDealCount === 0) {
      return t('deals.metrics.noOpenDeals');
    }

    if (activityContextError) {
      return t('deals.guidance.unavailable');
    }

    if (!activityContextReady) {
      return t('deals.guidance.loading');
    }

    return pipelineMetrics.focusStage
      ? `${t(`deals.stages.${pipelineMetrics.focusStage.stage}`)} · ${stageFocusLabel(pipelineMetrics.focusStage)}`
      : t('deals.metrics.noOpenDeals');
  });

  const selectedDealGuidance = $derived<PipelineGuidance | null>(
    selectedDeal ? guidanceByDealId[selectedDeal.id] ?? null : null
  );

  function guidanceLabel(guidance: PipelineGuidance): string {
    return t(`deals.guidance.${guidance.state}`);
  }

  function formatCurrencyList(
    totals: { currency: string; total: number }[],
    fallback = t('common.none'),
  ): string {
    if (totals.length === 0) {
      return fallback;
    }

    return totals
      .map((total) => formatCurrency(total.total, total.currency, settingsStore.language))
      .join(' · ');
  }

  function formatNullableRatio(value: number | null): string {
    return value === null ? '—' : formatPercent(value * 100);
  }

  function formatAverageDays(value: number | null): string {
    if (value === null) {
      return '—';
    }

    return t('deals.metrics.averageDays', { count: Math.round(value) });
  }

  function formatAverageProbability(value: number | null): string {
    return value === null ? '—' : formatPercent(value);
  }

  function ratioWidth(value: number): string {
    const bounded = Math.min(Math.max(value, 0), 1);
    return `${Math.round(bounded * 100)}%`;
  }

  function focusLabel(focus: StageFocus): string {
    return t(`deals.metrics.focus.${focus}`);
  }

  function riskCountLabel(count: number): string {
    if (activityContextError) {
      return t('deals.guidance.unavailable');
    }

    if (!activityContextReady) {
      return t('deals.guidance.loading');
    }

    return String(count);
  }

  function stageFocusLabel(metric: StageForecastMetric): string {
    if (metric.stage === 'closedWon' || metric.stage === 'closedLost' || metric.dealCount === 0) {
      return focusLabel(metric.focus);
    }

    if (activityContextError) {
      return t('deals.guidance.unavailable');
    }

    if (!activityContextReady) {
      return t('deals.guidance.loading');
    }

    return focusLabel(metric.focus);
  }

  const guidanceBadgeByDealId = $derived.by<Record<string, { label: string; tone: PipelineGuidanceTone }>>(() =>
    Object.fromEntries(
      allDeals.map((deal) => {
        const guidance = guidanceByDealId[deal.id];
        return [
          deal.id,
          guidance
            ? {
                label: guidanceLabel(guidance),
                tone: guidance.tone,
              }
            : {
                label: activityContextError ? t('deals.guidance.unavailable') : t('deals.guidance.loading'),
                tone: 'neutral',
              },
        ];
      })
    )
  );

  function openDealDrawer(deal: Deal) {
    if (suppressNextCardClick) {
      return;
    }

    selectedDeal = deal;
    dealStore.selectDeal(deal);
    if (dealId !== deal.id) {
      navigateHash(`/pipeline/${deal.id}`);
    }
  }

  function closeDealDrawer() {
    selectedDeal = null;
    dealStore.selectDeal(null);
    if (dealId) {
      navigateHash('/pipeline');
    }
  }

  function openDealFollowUp(deal: Deal) {
    selectedDeal = deal;
    dealStore.selectDeal(deal);
    uiStore.openModal('addActivity', {
      dealId: deal.id,
      contactId: deal.contactId ?? '',
      organizationId: deal.organizationId ?? '',
    });
  }

  function dismissStageFollowUpSuggestion() {
    stageFollowUpSuggestion = null;
  }

  function openSuggestedFollowUp(suggestion: DealStageFollowUpSuggestion) {
    const deal = allDeals.find((item) => item.id === suggestion.dealId);
    if (deal) {
      selectedDeal = deal;
      dealStore.selectDeal(deal);
    }

    uiStore.openModal('addActivity', {
      dealId: suggestion.draft.dealId,
      contactId: suggestion.draft.contactId,
      organizationId: suggestion.draft.organizationId,
      subject: suggestion.draft.subject,
      type: suggestion.draft.type,
      dueDate: suggestion.draft.dueDate,
      notes: suggestion.draft.notes,
    });
    stageFollowUpSuggestion = null;
  }

  function openSelectedDealFollowUp() {
    if (!selectedDeal) {
      return;
    }

    openDealFollowUp(selectedDeal);
  }

  // ── Stage label colors ───────────────────────────────────────────────────────

  const stageColors: Record<DealStage, string> = {
    lead:        '#20808D',
    qualified:   '#2D8659',
    proposal:    '#D4A017',
    negotiation: '#A84B2F',
    closedWon:   '#2D8659',
    closedLost:  '#C0392B',
  };
</script>

<div class="page-content pipeline-page">
  <!-- Header -->
  <div class="page-header">
    <h1 class="page-title">{t('deals.title')}</h1>
    <div class="toolbar">
      <button
        class="btn btn-primary btn-sm"
        onclick={() => openAddDeal('lead')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('deals.addDeal')}
      </button>
    </div>
  </div>

  <section class="saved-views" aria-labelledby="pipeline-saved-views-heading">
    <div class="saved-views-copy">
      <h2 class="saved-views-title" id="pipeline-saved-views-heading">{t('savedViews.title')}</h2>
      <p class="saved-views-help">{t('savedViews.helpPipeline')}</p>
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

  {#if !dealStore.isLoading && allDeals.length === 0}
    <div class="pipeline-first-run" data-testid="pipeline-first-run">
      <EmptyState
        icon="deals"
        title={t('deals.emptyBoardTitle')}
        description={t('deals.emptyBoardDesc')}
        actionLabel={t('deals.addDeal')}
        onaction={() => openAddDeal('lead')}
      />
    </div>
  {/if}

  {#if !dealStore.isLoading}
    <section
      class="pipeline-insights"
      aria-labelledby="pipeline-insights-heading"
      data-testid="pipeline-forecast-overview"
    >
      <div class="pipeline-insights-header">
        <div>
          <p class="pipeline-insights-eyebrow">{t('deals.metrics.eyebrow')}</p>
          <h2 id="pipeline-insights-heading" class="pipeline-insights-title">
            {t('deals.metrics.title')}
          </h2>
        </div>
        <div class="pipeline-focus-chip">
          <span class="pipeline-focus-label">{t('deals.metrics.focusStage')}</span>
          <span class="pipeline-focus-value">{pipelineFocusSummary}</span>
        </div>
      </div>

      <div class="forecast-grid" aria-label={t('deals.metrics.forecastSummary')}>
        <div class="forecast-card">
          <span class="forecast-card-label">{t('deals.metrics.openPipeline')}</span>
          <strong class="forecast-card-value">
            {formatCurrencyList(pipelineMetrics.openPipelineByCurrency)}
          </strong>
          <span class="forecast-card-meta">
            {t('deals.metrics.openDeals', { count: pipelineMetrics.openDealCount })}
          </span>
        </div>
        <div class="forecast-card">
          <span class="forecast-card-label">{t('deals.metrics.weightedForecast')}</span>
          <strong class="forecast-card-value">
            {formatCurrencyList(pipelineMetrics.weightedForecastByCurrency)}
          </strong>
          <span class="forecast-card-meta">
            {t('deals.metrics.atRisk', { count: riskCountLabel(pipelineMetrics.atRiskCount) })}
          </span>
        </div>
        <div class="forecast-card">
          <span class="forecast-card-label">{t('deals.metrics.closingNext30Days')}</span>
          <strong class="forecast-card-value">
            {formatCurrencyList(pipelineMetrics.closingNext30DaysByCurrency)}
          </strong>
          <span class="forecast-card-meta">
            {t('deals.metrics.closeDateGaps', {
              overdue: pipelineMetrics.overdueCloseDateCount,
              missing: pipelineMetrics.noCloseDateCount,
              later: pipelineMetrics.laterCloseDateCount,
            })}
          </span>
        </div>
        <div class="forecast-card">
          <span class="forecast-card-label">{t('deals.metrics.winRate')}</span>
          <strong class="forecast-card-value">
            {formatNullableRatio(pipelineMetrics.winRate)}
          </strong>
          <span class="forecast-card-meta">
            {t('deals.metrics.closedSplit', {
              won: pipelineMetrics.closedWonCount,
              lost: pipelineMetrics.closedLostCount,
            })}
          </span>
        </div>
      </div>

      <div class="stage-health" aria-label={t('deals.metrics.stageHealth')}>
        <div class="stage-health-heading">
          <h3>{t('deals.metrics.stageHealth')}</h3>
          <span>{t('deals.metrics.currentBoard')}</span>
        </div>
        <div class="stage-health-list">
          {#each pipelineMetrics.stageMetrics as metric (metric.stage)}
            <div class="stage-health-row" data-focus={metric.focus}>
              <div class="stage-health-main">
                <div class="stage-health-title">
                  <span class="col-stage-dot" style="background-color: {stageColors[metric.stage]}" aria-hidden="true"></span>
                  <span>{t(`deals.stages.${metric.stage}`)}</span>
                </div>
                <div class="stage-health-bar" aria-hidden="true">
                  <span
                    class="stage-health-bar-fill"
                    style={`width: ${ratioWidth(metric.dealShare)}`}
                  ></span>
                </div>
              </div>
              <dl class="stage-health-metrics">
                <div>
                  <dt>{t('deals.metrics.deals')}</dt>
                  <dd>{metric.dealCount}</dd>
                </div>
                <div>
                  <dt>{t('deals.metrics.value')}</dt>
                  <dd>{formatCurrencyList(metric.pipelineValueByCurrency)}</dd>
                </div>
                <div>
                  <dt>{t('deals.metrics.forecast')}</dt>
                  <dd>{formatCurrencyList(metric.weightedForecastByCurrency, '—')}</dd>
                </div>
                <div>
                  <dt>{t('deals.metrics.averageProbability')}</dt>
                  <dd>{formatAverageProbability(metric.averageProbability)}</dd>
                </div>
                <div>
                  <dt>{t('deals.metrics.averageUpdateAge')}</dt>
                  <dd>{formatAverageDays(metric.averageStageAgeDays)}</dd>
                </div>
                <div>
                  <dt>{t('deals.metrics.focusLabel')}</dt>
                  <dd>{stageFocusLabel(metric)}</dd>
                </div>
              </dl>
            </div>
          {/each}
        </div>
      </div>
    </section>
  {/if}

  <!-- Loading skeleton -->
  <div class="pipeline-filters" role="group" aria-label={t('common.customFieldFilter')}>
    <input
      class="input pipeline-filter-input selectable deal-search"
      type="search"
      value={searchQuery}
      oninput={handleSearchInput}
      placeholder={t('deals.search')}
      aria-label={t('deals.search')}
    />
    <input
      class="input pipeline-filter-input selectable"
      type="search"
      value={ownerFilter}
      oninput={handleOwnerFilterInput}
      placeholder={t('common.filterOwner')}
      aria-label={t('common.filterOwner')}
    />
    <div class="attention-filters" role="group" aria-label={t('deals.attentionFilter')}>
      {#each [
        { value: '', label: t('deals.attentionAll') },
        { value: 'needsFollowUp', label: t('deals.guidance.needsFollowUp') },
        { value: 'stale', label: t('deals.guidance.stale') },
        { value: 'overdue', label: t('deals.guidance.overdue') },
      ] as option (option.value)}
        <button
          class="filter-chip"
          class:active={attentionFilter === option.value}
          type="button"
          onclick={() => handleAttentionFilter(option.value as '' | 'needsFollowUp' | 'stale' | 'overdue')}
        >
          {option.label}
        </button>
      {/each}
    </div>
    <span class="pipeline-filters-label">{t('common.customField')}:</span>
    <select
      class="input pipeline-filter-select"
      value={selectedCustomFieldDefId}
      onchange={handleCustomFieldDefinitionChange}
      aria-label={t('common.customField')}
    >
      <option value="">
        {customFieldsLoading ? t('common.loading') : t('common.selectCustomField')}
      </option>
      {#each customFieldDefinitions as definition (definition.id)}
        <option value={definition.id}>{definition.field_name}</option>
      {/each}
    </select>
    <input
      class="input pipeline-filter-input selectable"
      type="search"
      value={customFieldQuery}
      oninput={handleCustomFieldQueryInput}
      placeholder={t('common.filterValue')}
      aria-label={t('common.filterValue')}
      disabled={!selectedCustomFieldDefId}
    />
    <button
      class="btn btn-ghost btn-sm"
      type="button"
      onclick={clearCustomFieldFilter}
      disabled={!selectedCustomFieldDefId && !customFieldQuery}
    >
      {t('common.clear')}
    </button>
    {#if customFieldValuesLoading}
      <span class="pipeline-filters-label">{t('common.loading')}</span>
    {/if}
  </div>

  {#if customFieldFilterError}
    <div class="filter-error" role="status">{customFieldFilterError}</div>
  {/if}

  {#if stageFollowUpSuggestion}
    <section
      class="local-automation-prompt"
      aria-labelledby="local-automation-prompt-heading"
      data-testid="local-automation-follow-up-prompt"
    >
      <div class="local-automation-copy">
        <p class="local-automation-eyebrow">{t('localAutomation.eyebrow')}</p>
        <h2 id="local-automation-prompt-heading">{t('localAutomation.pipeline.title')}</h2>
        <p>
          {t('localAutomation.pipeline.description', {
            deal: stageFollowUpSuggestion.dealName,
            stage: t(`deals.stages.${stageFollowUpSuggestion.toStage}`),
          })}
        </p>
      </div>
      <div class="local-automation-actions">
        <button
          class="btn btn-primary btn-sm"
          type="button"
          onclick={() => {
            if (stageFollowUpSuggestion) {
              openSuggestedFollowUp(stageFollowUpSuggestion);
            }
          }}
        >
          {t('localAutomation.pipeline.addDraft')}
        </button>
        <button
          class="btn btn-ghost btn-sm"
          type="button"
          onclick={dismissStageFollowUpSuggestion}
        >
          {t('common.dismiss')}
        </button>
      </div>
    </section>
  {/if}

  {#if dealStore.isLoading}
    <div class="kanban-loading" aria-label={t('common.loading')}>
      {#each DEAL_STAGES as stage (stage)}
        <div class="kanban-column-skeleton">
          <div class="skeleton skeleton-col-header"></div>
          {#each [1, 2, 3] as i (i)}
            <div class="skeleton skeleton-card"></div>
          {/each}
        </div>
      {/each}
    </div>

  {:else}
    <!-- ── Kanban board ─────────────────────────────────────────────────────── -->
    <div
      class="kanban-board"
      class:kanban-board--dragging={Boolean(draggingId)}
      role="region"
      aria-label={t('deals.title')}
    >
      <div class="visually-hidden" aria-live="polite" aria-label={t('deals.board.liveRegion')}>
        {moveAnnouncement}
      </div>
      {#each columns as col (col.stage)}
        {@const dropKind = pipelineDropKind(draggingFromStage, dragOverStage)}
        {@const isTarget = dropKind === 'target' && dragOverStage === col.stage}
        {@const isSource = dropKind === 'source' && dragOverStage === col.stage}
        <div
          class="kanban-column"
          class:kanban-column--drop-target={isTarget}
          class:kanban-column--drop-source={isSource && dragOverStage === col.stage}
          data-testid={`pipeline-column-${col.stage}`}
          ondragover={(e) => handleDragOver(e, col.stage)}
          ondragleave={handleDragLeave}
          ondrop={(e) => handleDrop(e, col.stage)}
          role="list"
          aria-label={t(`deals.stages.${col.stage}`)}
          aria-dropeffect={draggingId && draggingFromStage !== col.stage ? 'move' : 'none'}
        >
          <!-- Column header -->
          <div class="col-header" style="--stage-color: {stageColors[col.stage]}">
            <div class="col-header-left">
              <span class="col-stage-dot" style="background-color: {stageColors[col.stage]}" aria-hidden="true"></span>
              <h2 class="col-title">{t(`deals.stages.${col.stage}`)}</h2>
              <span class="col-count" aria-label="{col.deals.length} {t('deals.addDeal')}">
                {col.deals.length}
              </span>
            </div>
            <button
              class="btn-add-deal"
              onclick={() => openAddDeal(col.stage)}
              type="button"
              aria-label="{t('deals.addDeal')} — {t(`deals.stages.${col.stage}`)}"
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                <path d="M6 1v10M1 6h10"/>
              </svg>
            </button>
          </div>

          <!-- Column total value -->
          {#if col.deals.length > 0}
            <div class="col-total">
              <span class="col-total-label">{t('deals.totalValue')}:</span>
              <div class="col-total-values" role="list" aria-label={t('deals.totalValue')}>
                {#each col.currencyTotals as total (total.currency)}
                  <span class="col-total-value" role="listitem">
                    {formatCurrency(total.total, total.currency, settingsStore.language)}
                  </span>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Deal cards -->
          <div class="col-cards">
            {#if isTarget}
              <p class="col-drop-hint" data-testid={`pipeline-drop-hint-${col.stage}`}>
                {t('deals.board.dropToStage', { stage: t(`deals.stages.${col.stage}`) })}
              </p>
            {:else if isSource && dragOverStage === col.stage}
              <p class="col-drop-hint col-drop-hint--source">
                {t('deals.board.alreadyInStage', { stage: t(`deals.stages.${col.stage}`) })}
              </p>
            {/if}
            {#if col.deals.length === 0}
              <div class="col-empty" aria-label={t('deals.noDeals')}>
                <p class="col-empty-text">
                  {isTarget
                    ? t('deals.board.dropToStage', { stage: t(`deals.stages.${col.stage}`) })
                    : t('deals.noDeals')}
                </p>
              </div>
            {:else}
              <VirtualList items={col.deals} itemHeight={108}>
                {#snippet children(deal)}
                  {@const relationships = deriveDealRelationshipLabels(
                    deal,
                    relationshipContacts,
                    relationshipOrganizations,
                  )}
                  {@const guidanceBadge = guidanceBadgeByDealId[deal.id]}
                  <div
                    class="card-wrapper"
                    class:card-wrapper--dragging={draggingId === deal.id}
                    role="listitem"
                  >
                    <DealCard
                      {deal}
                      primaryContactName={relationships.primaryContactName}
                      organizationName={relationships.organizationName}
                      guidanceLabel={guidanceBadge?.label ?? t('deals.guidance.loading')}
                      guidanceTone={guidanceBadge?.tone ?? 'neutral'}
                      onclick={openDealDrawer}
                      ondragstart={(event) => handleDragStart(deal.id, col.stage, event)}
                      ondragend={handleDragEnd}
                    />
                  </div>
                {/snippet}
              </VirtualList>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if selectedDeal && !uiStore.activeModal}
    <DealDetailDrawer
      deal={selectedDeal}
      guidance={selectedDealGuidance}
      activities={selectedDealActivities}
      activitiesLoading={dealActivitiesLoading}
      activityContextError={activityContextError}
      relationshipsByActivityId={selectedDealRelationshipLabels}
      primaryContactName={selectedDealRelationshipNames.primaryContactName}
      organizationName={selectedDealRelationshipNames.organizationName}
      onclose={closeDealDrawer}
      onaddfollowup={openSelectedDealFollowUp}
      onrefresh={() => void loadPipelineActivityContext()}
    />
  {/if}
</div>

<style>
  /* ── Page ─────────────────────────────────────────────────────────────────── */

  .pipeline-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
    min-height: 0;
  }

  .toolbar {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  .pipeline-insights {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    background: var(--surface-default);
  }

  .pipeline-insights-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .pipeline-insights-eyebrow {
    margin: 0 0 var(--space-1);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    text-transform: uppercase;
  }

  .pipeline-insights-title {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .pipeline-focus-chip {
    display: grid;
    gap: var(--space-1);
    min-width: min(280px, 100%);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background: var(--surface-raised);
  }

  .pipeline-focus-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .pipeline-focus-value {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .forecast-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .forecast-card {
    display: grid;
    align-content: start;
    gap: var(--space-2);
    min-height: 116px;
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
  }

  .forecast-card-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .forecast-card-value {
    min-height: 30px;
    font-size: var(--text-xl);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    line-height: 1.25;
  }

  .forecast-card-meta {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    line-height: 1.35;
  }

  .stage-health {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .stage-health-heading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .stage-health-heading h3 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .stage-health-heading span {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .stage-health-list {
    display: grid;
    gap: var(--space-2);
  }

  .stage-health-row {
    display: grid;
    grid-template-columns: minmax(180px, 0.7fr) minmax(0, 2fr);
    gap: var(--space-4);
    align-items: center;
    min-height: 82px;
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
  }

  .stage-health-main {
    display: grid;
    gap: var(--space-2);
    min-width: 0;
  }

  .stage-health-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .stage-health-title span:last-child {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stage-health-bar {
    height: 6px;
    border-radius: 9999px;
    background: var(--surface-active);
    overflow: hidden;
  }

  .stage-health-bar-fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--color-primary);
  }

  .stage-health-metrics {
    display: grid;
    grid-template-columns: repeat(6, minmax(72px, 1fr));
    gap: var(--space-3);
    margin: 0;
  }

  .stage-health-metrics div {
    display: grid;
    gap: var(--space-1);
    min-width: 0;
  }

  .stage-health-metrics dt {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .stage-health-metrics dd {
    margin: 0;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    white-space: nowrap;
  }

  .pipeline-first-run {
    padding: var(--space-6) 0;
  }

  .saved-views,
  .saved-views-copy,
  .saved-views-controls {
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

  .pipeline-filters {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .pipeline-filters-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .attention-filters {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .filter-chip {
    padding: var(--space-2) var(--space-4);
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

  .pipeline-filter-select {
    min-width: 180px;
    height: 32px;
  }

  .pipeline-filter-input {
    min-width: 180px;
    height: 32px;
  }

  .filter-error {
    font-size: var(--text-xs);
    color: var(--text-danger);
  }

  .local-automation-prompt {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4);
    border: var(--border-width) solid var(--color-primary-200);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
  }

  .local-automation-copy {
    display: grid;
    gap: var(--space-1);
    min-width: 0;
  }

  .local-automation-eyebrow {
    margin: 0;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    text-transform: uppercase;
    letter-spacing: 0;
  }

  .local-automation-copy h2,
  .local-automation-copy p {
    margin: 0;
  }

  .local-automation-copy h2 {
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .local-automation-copy p {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .local-automation-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  /* ── Board ───────────────────────────────────────────────────────────────── */

  .kanban-board {
    display: flex;
    gap: var(--space-4);
    overflow-x: auto;
    flex: 1;
    min-height: 0;
    padding-block-end: var(--space-4);
    overscroll-behavior-x: contain;
    -webkit-overflow-scrolling: touch;

    /* Custom scrollbar */
    scrollbar-width: thin;
    scrollbar-color: var(--border-default) transparent;
  }

  .kanban-board::-webkit-scrollbar {
    height: 6px;
  }

  .kanban-board::-webkit-scrollbar-track {
    background: transparent;
  }

  .kanban-board::-webkit-scrollbar-thumb {
    background-color: var(--border-default);
    border-radius: 3px;
  }

  /* ── Column ──────────────────────────────────────────────────────────────── */

  .kanban-column {
    flex: 0 0 268px;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    background-color: var(--surface-raised);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    min-height: 0;
    max-height: 100%;
    border: 2px solid transparent;
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out);
  }

  .kanban-board--dragging .kanban-column {
    border-color: var(--border-default);
  }

  .kanban-column--drop-target {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
    box-shadow: inset 0 0 0 1px var(--color-primary);
  }

  .kanban-column--drop-source {
    border-style: dashed;
    border-color: var(--border-default);
    background-color: var(--surface-raised);
  }

  /* ── Column header ───────────────────────────────────────────────────────── */

  .col-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .col-header-left {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .col-stage-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .col-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .col-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 var(--space-2);
    background-color: var(--surface-default);
    border: var(--border-width) solid var(--border-default);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  /* ── Add deal button ─────────────────────────────────────────────────────── */

  .btn-add-deal {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: var(--border-width) solid var(--border-default);
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .btn-add-deal:hover {
    background-color: var(--surface-active);
    color: var(--text-accent);
    border-color: var(--color-primary-200);
  }

  /* ── Column total ────────────────────────────────────────────────────────── */

  .col-total {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    padding-block-end: var(--space-2);
    border-block-end: var(--border-width) solid var(--border-default);
  }

  .col-total-label {
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .col-total-values {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  .col-total-value {
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
  }

  /* ── Cards area ──────────────────────────────────────────────────────────── */

  .col-cards {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .col-drop-hint {
    margin: 0;
    padding: var(--space-2) var(--space-3);
    border: 1px dashed var(--color-primary);
    border-radius: var(--radius-md);
    background-color: var(--surface-default);
    color: var(--text-accent, var(--color-primary));
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-align: center;
  }

  .col-drop-hint--source {
    border-color: var(--border-default);
    color: var(--text-secondary);
    font-weight: var(--weight-medium);
  }

  .card-wrapper {
    cursor: grab;
    border-radius: var(--radius-md);
    transition: opacity var(--duration-fast) var(--ease-out),
                box-shadow var(--duration-fast) var(--ease-out);
  }

  .card-wrapper:active {
    cursor: grabbing;
  }

  .card-wrapper--dragging {
    opacity: 0.4;
  }

  /* ── Empty column state ──────────────────────────────────────────────────── */

  .col-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 80px;
    border-radius: var(--radius-md);
    border: 1px dashed var(--border-default);
  }

  .col-empty-text {
    font-size: var(--text-xs);
    color: var(--text-muted, var(--text-secondary));
    text-align: center;
    margin: 0;
    opacity: 0.7;
  }

  /* ── Loading skeleton ────────────────────────────────────────────────────── */

  .kanban-loading {
    display: flex;
    gap: var(--space-4);
    overflow-x: hidden;
    padding-block-end: var(--space-4);
  }

  .kanban-column-skeleton {
    flex: 0 0 268px;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    background-color: var(--surface-raised);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .skeleton-col-header {
    height: 28px;
    border-radius: var(--radius-sm);
  }

  .skeleton-card {
    height: 88px;
    border-radius: var(--radius-md);
  }

  @media (max-width: 1180px) {
    .forecast-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .stage-health-row {
      grid-template-columns: 1fr;
    }

    .stage-health-metrics {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }

  @media (max-width: 760px) {
    .pipeline-insights-header,
    .stage-health-heading {
      align-items: stretch;
      flex-direction: column;
    }

    .forecast-grid {
      grid-template-columns: 1fr;
    }

    .stage-health-metrics {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
