<script lang="ts">
  /**
   * Activities.svelte — Activity list view for 900CRM.
   *
   * Features:
   *   - Filter by type (task/call/meeting/email/visit/whatsapp/sms) and status
   *   - Sorted by due date ascending
   *   - Quick-add activity form (subject, type, due date and optional time)
   *   - Mark complete / mark incomplete toggle
   *   - Empty state, loading state, error state
   */

  import { t } from '$lib/i18n';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import { ACTIVITY_TYPES, type ActivityType, type ActivityStatus, type CreateActivityPayload } from '$lib/api/activities';
  import LogJustHappened from '$lib/components/LogJustHappened.svelte';
  import { combineDueDateAndTime, formatActivityDue } from '$lib/utils/activityDue';
  import {
    listCustomFieldDefinitions,
    listCustomFieldValuesForEntityType,
    type CustomFieldDefinition,
    type EntityTypeCustomFieldValue,
  } from '$lib/api/customFields';
  import {
    createSavedView,
    deleteSavedView,
    filtersMatch,
    listSavedViews,
    type ContactSavedViewFilters,
    type SavedView,
  } from '$lib/api/savedViews';
  import { formatDate, formatRelativeTime } from '$lib/utils/formatters';
  import { currentHashPath, navigateHash } from '$lib/utils/hashRouter';
  import {
    ACTIVITY_DUE_BUCKETS,
    addLocalDays,
    buildActivityWorkbench,
    type ActivityDueBucket,
  } from '$lib/utils/activityWorkbench';
  import {
    buildActivityWeek,
    shiftWeek,
    startOfWeek,
  } from '$lib/utils/activityWeek';
  import {
    buildActivityMonth,
    shiftMonth,
    startOfMonth,
  } from '$lib/utils/activityMonth';
  import {
    addSelectedActivityLinks,
    deriveActivityRelationshipLabels,
    loadActivityLinkIndex,
    loadActivityRelationshipLookups,
    type ActivityLinkIndex,
    type ActivityRelationshipLookups,
  } from '$lib/utils/activityRelationships';
  import EmptyState from '$lib/components/EmptyState.svelte';

  // ── State ────────────────────────────────────────────────────────────────────

  let typeFilter   = $state<ActivityType | ''>('');
  let statusFilter = $state<ActivityStatus | ''>('');
  let bucketFilter = $state<ActivityDueBucket | ''>('');
  let reschedulingActivityId = $state<string | null>(null);
  let customFieldDefinitions = $state<CustomFieldDefinition[]>([]);
  let selectedCustomFieldDefId = $state('');
  let customFieldQuery = $state('');
  let customFieldsLoading = $state(true);
  let customFieldValuesLoading = $state(false);
  let customFieldFilterError = $state<string | null>(null);
  let customFieldValueIndex = $state<Record<string, Record<string, string>>>({});
  let activityRelationshipLookups = $state<ActivityRelationshipLookups>({
    contacts: [],
    organizations: [],
    deals: [],
  });
  let activityLinkIndex = $state<ActivityLinkIndex>({});
  let activityRelationshipsLoading = $state(false);
  let loadedActivityLinkKey = $state('');
  let loadedActivityRelationshipVersion = $state(0);
  let savedViews = $state<SavedView[]>([]);
  let selectedViewId = $state('');
  let viewName = $state('');
  let viewsLoading = $state(false);
  let viewsSaving = $state(false);
  let viewsError = $state<string | null>(null);
  let activitiesBootstrapped = false;
  let activitiesHashBound = false;
  type ActivityLayout = 'list' | 'week' | 'month';
  let viewMode = $state<ActivityLayout>('list');
  let weekStart = $state<Date>(startOfWeek(new Date()));
  let monthStart = $state<Date>(startOfMonth(new Date()));
  let weekDragOverDate = $state<string | null>(null);

  // Quick-add form
  let showQuickAdd  = $state(false);
  let qaSubject     = $state('');
  let qaType        = $state<ActivityType>('task');
  let qaDueDate     = $state('');
  let qaDueTime     = $state('');
  let qaContactId   = $state('');
  let qaOrganizationId = $state('');
  let qaDealId      = $state('');
  let qaSubmitting  = $state(false);

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  $effect(() => {
    if (activitiesBootstrapped) {
      return;
    }

    activitiesBootstrapped = true;
    void (async () => {
      await Promise.all([
        activityStore.loadActivities(),
        ensureActivityRelationshipLookups(),
        loadCustomFieldDefinitions(),
        loadSavedViews(),
      ]);
      await refreshActivityLinks();
    })();
  });

  $effect(() => {
    const activityLinkKey = activityStore.activities.map((activity) => activity.id).join('|');
    const relationshipRefreshVersion = activityStore.relationshipRefreshVersion;
    if (
      activityLinkKey === loadedActivityLinkKey
      && relationshipRefreshVersion === loadedActivityRelationshipVersion
    ) {
      return;
    }

    loadedActivityLinkKey = activityLinkKey;
    loadedActivityRelationshipVersion = relationshipRefreshVersion;
    void loadActivityLinksForCurrentActivities();
  });

  $effect(() => {
    if (activitiesHashBound || typeof window === 'undefined') {
      return;
    }

    activitiesHashBound = true;
    applyActivitiesHash(currentHashPath());
    window.addEventListener('hashchange', () => applyActivitiesHash(currentHashPath()));
  });

  // ── Handlers ─────────────────────────────────────────────────────────────────

  function applyActivitiesHash(path: string) {
    const clean = path.replace(/^#/, '').replace(/\/+$/, '') || '/';
    if (clean === '/activities/week') {
      viewMode = 'week';
    } else if (clean === '/activities/month') {
      viewMode = 'month';
    } else {
      viewMode = 'list';
    }
  }

  function selectViewMode(mode: ActivityLayout) {
    viewMode = mode;
    if (mode === 'week') {
      weekStart = startOfWeek(new Date());
    }
    if (mode === 'month') {
      monthStart = startOfMonth(new Date());
    }
    const nextPath = mode === 'week'
      ? '/activities/week'
      : mode === 'month'
        ? '/activities/month'
        : '/activities';
    if (currentHashPath().replace(/\/+$/, '') !== nextPath) {
      navigateHash(nextPath);
    }
  }

  function moveWeek(weeks: number) {
    weekStart = weeks === 0 ? startOfWeek(new Date()) : shiftWeek(weekStart, weeks);
  }

  function moveMonth(months: number) {
    monthStart = months === 0 ? startOfMonth(new Date()) : shiftMonth(monthStart, months);
  }

  function addFollowUpOnDay(date: string) {
    uiStore.openModal('addActivity', { dueDate: date });
  }

  function handleWeekDragStart(activityId: string, event: DragEvent) {
    event.dataTransfer?.setData('text/plain', activityId);
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleWeekDragOver(date: string, event: DragEvent) {
    event.preventDefault();
    weekDragOverDate = date;
  }

  function handleWeekDragLeave(date: string) {
    if (weekDragOverDate === date) {
      weekDragOverDate = null;
    }
  }

  async function handleWeekDrop(date: string, event: DragEvent) {
    event.preventDefault();
    weekDragOverDate = null;
    const activityId = event.dataTransfer?.getData('text/plain')?.trim();
    if (!activityId) {
      return;
    }

    reschedulingActivityId = activityId;
    try {
      await activityStore.updateActivity(activityId, { dueDate: date });
    } finally {
      reschedulingActivityId = null;
    }
  }

  async function handleTypeFilter(type: ActivityType | '') {
    typeFilter = type;
    await activityStore.setFilters({
      type: type || undefined,
      status: statusFilter || undefined,
    });
    syncSelectedView();
    await refreshActivityLinks();
  }

  async function handleStatusFilter(status: ActivityStatus | '') {
    statusFilter = status;
    bucketFilter = '';
    await activityStore.setFilters({
      type: typeFilter || undefined,
      status: status || undefined,
    });
    syncSelectedView();
    await refreshActivityLinks();
  }

  async function handleBucketFilter(bucket: ActivityDueBucket | '') {
    bucketFilter = bucket;
    statusFilter = '';

    await activityStore.setFilters({
      type: typeFilter || undefined,
      status: undefined,
    });
    syncSelectedView();
    await refreshActivityLinks();
  }

  async function ensureActivityRelationshipLookups() {
    activityRelationshipsLoading = true;
    try {
      activityRelationshipLookups = await loadActivityRelationshipLookups();
    } catch (err) {
      console.error('[Activities] Failed to load activity relationship lookups:', err);
      uiStore.toastError(t('errors.loadRelationships', { name: t('entities.activity') }));
    } finally {
      activityRelationshipsLoading = false;
    }
  }

  async function refreshActivityLinks() {
    loadedActivityLinkKey = activityStore.activities.map((activity) => activity.id).join('|');
    loadedActivityRelationshipVersion = activityStore.relationshipRefreshVersion;
    await loadActivityLinksForCurrentActivities();
  }

  async function loadActivityLinksForCurrentActivities() {
    try {
      activityLinkIndex = await loadActivityLinkIndex(activityStore.activities.map((activity) => activity.id));
    } catch (err) {
      console.error('[Activities] Failed to load activity links:', err);
      uiStore.toastError(t('errors.loadRelationships', { name: t('entities.activity') }));
    }
  }

  async function loadCustomFieldDefinitions() {
    customFieldsLoading = true;
    try {
      customFieldDefinitions = await listCustomFieldDefinitions('activity');
    } finally {
      customFieldsLoading = false;
    }
  }

  async function ensureCustomFieldValueIndex() {
    if (Object.keys(customFieldValueIndex).length > 0) return;

    customFieldValuesLoading = true;
    customFieldFilterError = null;
    try {
      const values = await listCustomFieldValuesForEntityType('activity');
      customFieldValueIndex = indexCustomFieldValues(values);
    } catch (err) {
      customFieldFilterError = t('common.filterLoadFailed');
      console.error('[Activities] Failed to load custom-field filter values:', err);
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

  function asActivityType(value: string | undefined): ActivityType | '' {
    return (ACTIVITY_TYPES as readonly string[]).includes(value ?? '')
      ? (value as ActivityType)
      : '';
  }

  function asActivityStatus(value: string | undefined): ActivityStatus | '' {
    return value === 'pending' || value === 'completed' || value === 'overdue' ? value : '';
  }

  function asActivityBucket(value: string | undefined): ActivityDueBucket | '' {
    return ACTIVITY_DUE_BUCKETS.includes(value as ActivityDueBucket)
      ? (value as ActivityDueBucket)
      : '';
  }

  function collectCurrentFilters(): ContactSavedViewFilters {
    return {
      type: typeFilter || undefined,
      status: statusFilter || undefined,
      bucket: bucketFilter || undefined,
      customFieldDefId: selectedCustomFieldDefId || undefined,
      customFieldQuery: customFieldQuery.trim() || undefined,
    };
  }

  const currentViewFilters = $derived(collectCurrentFilters());
  const selectedView = $derived(savedViews.find((view) => view.id === selectedViewId) ?? null);
  const canSaveView = $derived(viewName.trim().length > 0 && !viewsSaving);

  async function loadSavedViews(): Promise<void> {
    viewsLoading = true;
    viewsError = null;
    try {
      savedViews = await listSavedViews('activity');
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
    typeFilter = asActivityType(view.filters.type);
    const nextBucket = asActivityBucket(view.filters.bucket);
    const nextStatus = asActivityStatus(view.filters.status);
    if (nextBucket) {
      bucketFilter = nextBucket;
      statusFilter = '';
    } else {
      bucketFilter = '';
      statusFilter = nextStatus;
    }
    selectedCustomFieldDefId = view.filters.customFieldDefId ?? '';
    customFieldQuery = view.filters.customFieldQuery ?? '';
    await activityStore.setFilters({
      type: typeFilter || undefined,
      status: statusFilter || undefined,
    });
    if (selectedCustomFieldDefId && customFieldQuery.trim()) {
      await ensureCustomFieldValueIndex();
    }
    await refreshActivityLinks();
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
      const view = await createSavedView('activity', name, collectCurrentFilters());
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

  async function handleToggleComplete(activity: { id: string; status: ActivityStatus }) {
    if (activity.status === 'completed') {
      await activityStore.markIncomplete(activity.id);
    } else {
      await activityStore.markComplete(activity.id);
    }
  }

  async function handleSnooze(activityId: string, days = 1) {
    reschedulingActivityId = activityId;
    try {
      await activityStore.updateActivity(activityId, {
        dueDate: addLocalDays(new Date(), days),
      });
    } finally {
      reschedulingActivityId = null;
    }
  }

  async function handleReschedule(activityId: string, event: Event) {
    const dueDate = (event.target as HTMLInputElement).value;
    reschedulingActivityId = activityId;
    try {
      await activityStore.updateActivity(activityId, {
        dueDate: dueDate || null,
      });
    } finally {
      reschedulingActivityId = null;
    }
  }

  async function handleQuickAdd() {
    if (!qaSubject.trim()) return;
    qaSubmitting = true;
    try {
      const payload: CreateActivityPayload = {
        type:      qaType,
        subject:   qaSubject.trim(),
        notes:     null,
        dueDate:   combineDueDateAndTime(qaDueDate, qaDueTime),
        contactId: qaContactId || null,
        dealId:    qaDealId || null,
      };
      const activity = await activityStore.createActivity(payload);
      await addSelectedActivityLinks(activity.id, {
        contactId: qaContactId || null,
        organizationId: qaOrganizationId || null,
        dealId: qaDealId || null,
      });
      activityStore.notifyRelationshipLinksChanged();
      await refreshActivityLinks();

      // Reset form
      qaSubject  = '';
      qaType     = 'task';
      qaDueDate  = '';
      qaContactId = '';
      qaOrganizationId = '';
      qaDealId = '';
      showQuickAdd = false;
    } catch (err) {
      console.error('[Activities] Quick-add error:', err);
    } finally {
      qaSubmitting = false;
    }
  }

  function cancelQuickAdd() {
    showQuickAdd  = false;
    qaSubject     = '';
    qaType        = 'task';
    qaDueDate     = '';
    qaDueTime     = '';
    qaContactId   = '';
    qaOrganizationId = '';
    qaDealId      = '';
  }

  // ── Type icon helper ─────────────────────────────────────────────────────────

  function activityIcon(type: ActivityType): string {
    switch (type) {
      case 'task':    return 'M9 11l3 3L22 4M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11';
      case 'call':    return 'M22 16.92v3a2 2 0 01-2.18 2 19.79 19.79 0 01-8.63-3.07 19.5 19.5 0 01-6-6 19.79 19.79 0 01-3.07-8.67A2 2 0 014.11 2h3a2 2 0 012 1.72 12.84 12.84 0 00.7 2.81 2 2 0 01-.45 2.11L8.09 9.91a16 16 0 006 6l1.27-1.27a2 2 0 012.11-.45 12.84 12.84 0 002.81.7A2 2 0 0122 16.92z';
      case 'meeting': return 'M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2M9 11a4 4 0 100-8 4 4 0 000 8M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75';
      case 'email':     return 'M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2M22 6l-10 7L2 6';
      case 'visit':     return 'M12 21s7-4.5 7-11a7 7 0 10-14 0c0 6.5 7 11 7 11zM12 11a2 2 0 110-4 2 2 0 010 4z';
      case 'whatsapp':  return 'M20 4a10 10 0 00-16.4 11.4L2 22l6.8-1.6A10 10 0 1020 4z';
      case 'sms':       return 'M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z';
      default:          return 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5';
    }
  }

  // ── Status badge class ───────────────────────────────────────────────────────

  function statusClass(status: ActivityStatus): string {
    switch (status) {
      case 'completed': return 'status-completed';
      case 'overdue':   return 'status-overdue';
      default:          return 'status-pending';
    }
  }

  function matchesCustomField(activityId: string): boolean {
    const query = customFieldQuery.trim().toLowerCase();
    if (!selectedCustomFieldDefId || !query) {
      return true;
    }
    if (customFieldValuesLoading || customFieldFilterError) {
      return true;
    }

    const rawValue = customFieldValueIndex[selectedCustomFieldDefId]?.[activityId] ?? '';
    return rawValue.toLowerCase().includes(query);
  }

  const filteredActivities = $derived(
    activityStore.activities.filter((activity) => matchesCustomField(activity.id))
  );

  const workbench = $derived(buildActivityWorkbench(filteredActivities));
  const week = $derived(buildActivityWeek(filteredActivities, weekStart));
  const month = $derived(buildActivityMonth(filteredActivities, monthStart));
  const isCalendarView = $derived(viewMode === 'week' || viewMode === 'month');

  function formatWeekday(dateKey: string): string {
    const [year, month, day] = dateKey.split('-').map(Number);
    return new Intl.DateTimeFormat(settingsStore.language, { weekday: 'short' }).format(
      new Date(year, month - 1, day),
    );
  }

  function formatWeekDayLabel(dateKey: string): string {
    const [year, month, day] = dateKey.split('-').map(Number);
    return new Intl.DateTimeFormat(settingsStore.language, { day: 'numeric', month: 'short' }).format(
      new Date(year, month - 1, day),
    );
  }

  function formatMonthTitle(dateKey: string): string {
    const [year, month] = dateKey.split('-').map(Number);
    return new Intl.DateTimeFormat(settingsStore.language, { month: 'long', year: 'numeric' }).format(
      new Date(year, month - 1, 1),
    );
  }

  function weekdayHeadings(): string[] {
    return (month.weeks[0] ?? week.days).map((day) => formatWeekday(day.date));
  }

  const visibleBuckets = $derived(
    workbench.buckets.filter((bucket) => !bucketFilter || bucket.bucket === bucketFilter)
  );

  const visibleActivityCount = $derived(
    visibleBuckets.reduce((sum, bucket) => sum + bucket.activities.length, 0)
  );

  const hasActiveActivityFilters = $derived(
    Boolean(
      typeFilter
      || statusFilter
      || bucketFilter
      || selectedCustomFieldDefId
      || customFieldQuery.trim(),
    )
  );
  const isFirstRunEmpty = $derived(
    !activityStore.isLoading
    && activityStore.activities.length === 0
    && !hasActiveActivityFilters
  );

  function bucketLabel(bucket: ActivityDueBucket): string {
    return t(`activities.buckets.${bucket}`);
  }

  function bucketDescription(bucket: ActivityDueBucket): string {
    return t(`activities.bucketDescriptions.${bucket}`);
  }
</script>

<div class="page-content activities-page">
  <!-- Header -->
  <div class="page-header">
    <h1 class="page-title">{t('activities.title')}</h1>
    <div class="toolbar">
      <div class="layout-toggle" role="tablist" aria-label={t('activities.viewLabel')}>
        <button
          class="layout-toggle-button"
          class:layout-toggle-button--active={viewMode === 'list'}
          type="button"
          role="tab"
          aria-selected={viewMode === 'list'}
          onclick={() => selectViewMode('list')}
        >
          {t('activities.viewList')}
        </button>
        <button
          class="layout-toggle-button"
          class:layout-toggle-button--active={viewMode === 'week'}
          type="button"
          role="tab"
          aria-selected={viewMode === 'week'}
          onclick={() => selectViewMode('week')}
        >
          {t('activities.viewWeek')}
        </button>
        <button
          class="layout-toggle-button"
          class:layout-toggle-button--active={viewMode === 'month'}
          type="button"
          role="tab"
          aria-selected={viewMode === 'month'}
          onclick={() => selectViewMode('month')}
        >
          {t('activities.viewMonth')}
        </button>
      </div>
      <button
        class="btn btn-secondary btn-sm"
        onclick={() => showQuickAdd = !showQuickAdd}
        type="button"
        aria-expanded={showQuickAdd}
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M6 1v10M1 6h10"/>
        </svg>
        {t('activities.addActivity')}
      </button>
      <button
        class="btn btn-primary btn-sm"
        onclick={() => uiStore.openModal('addActivity')}
        type="button"
      >
        {t('activities.addActivity')} ({t('common.all')})
      </button>
    </div>
  </div>
  <LogJustHappened onlogged={() => void activityStore.loadActivities()} />

  <section class="saved-views" aria-labelledby="activities-saved-views-heading">
    <div class="saved-views-copy">
      <h2 class="saved-views-title" id="activities-saved-views-heading">{t('savedViews.title')}</h2>
      <p class="saved-views-help">{t('savedViews.helpActivities')}</p>
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

  <!-- Quick-add form -->
  {#if showQuickAdd}
    <form
      class="card quick-add-form"
      onsubmit={(e) => { e.preventDefault(); handleQuickAdd(); }}
      aria-label={t('activities.addActivity')}
    >
      <div class="quick-add-row">
        <!-- Type selector -->
        <div class="qa-field qa-field--type">
          <label class="sr-only" for="qa-type">{t('activities.type')}</label>
          <select
            id="qa-type"
            class="input select-input"
            bind:value={qaType}
          >
            {#each ACTIVITY_TYPES as type (type)}
              <option value={type}>{t(`activities.${type}`)}</option>
            {/each}
          </select>
        </div>

        <!-- Subject input -->
        <div class="qa-field qa-field--subject">
          <label class="sr-only" for="qa-subject">{t('activities.subject')}</label>
          <input
            id="qa-subject"
            class="input"
            type="text"
            bind:value={qaSubject}
            placeholder={t('activities.subject')}
            autocomplete="off"
            required
          />
        </div>

        <!-- Due date -->
        <div class="qa-field qa-field--date">
          <label class="sr-only" for="qa-due">{t('activities.dueDate')}</label>
          <input
            id="qa-due"
            class="input"
            type="date"
            bind:value={qaDueDate}
            aria-label={t('activities.dueDate')}
          />
        </div>
        <div class="qa-field qa-field--time">
          <label class="sr-only" for="qa-due-time">{t('activities.dueTime')}</label>
          <input
            id="qa-due-time"
            class="input"
            type="time"
            bind:value={qaDueTime}
            aria-label={t('activities.dueTime')}
          />
        </div>

        <div class="qa-field qa-field--relationship">
          <label class="sr-only" for="qa-contact">{t('deals.contact')}</label>
          <select
            id="qa-contact"
            class="input select-input"
            bind:value={qaContactId}
            disabled={activityRelationshipsLoading || qaSubmitting}
            aria-busy={activityRelationshipsLoading}
          >
            <option value="">{activityRelationshipsLoading ? t('common.loading') : t('common.none')}</option>
            {#each activityRelationshipLookups.contacts as contact (contact.id)}
              <option value={contact.id}>
                {[contact.firstName, contact.lastName].map((part) => part.trim()).filter(Boolean).join(' ') || contact.email || contact.id}
              </option>
            {/each}
          </select>
        </div>

        <div class="qa-field qa-field--relationship">
          <label class="sr-only" for="qa-organization">{t('contacts.organization')}</label>
          <select
            id="qa-organization"
            class="input select-input"
            bind:value={qaOrganizationId}
            disabled={activityRelationshipsLoading || qaSubmitting}
            aria-busy={activityRelationshipsLoading}
          >
            <option value="">{activityRelationshipsLoading ? t('common.loading') : t('common.none')}</option>
            {#each activityRelationshipLookups.organizations as organization (organization.id)}
              <option value={organization.id}>{organization.name}</option>
            {/each}
          </select>
        </div>

        <div class="qa-field qa-field--relationship">
          <label class="sr-only" for="qa-deal">{t('deals.title')}</label>
          <select
            id="qa-deal"
            class="input select-input"
            bind:value={qaDealId}
            disabled={activityRelationshipsLoading || qaSubmitting}
            aria-busy={activityRelationshipsLoading}
          >
            <option value="">{activityRelationshipsLoading ? t('common.loading') : t('common.none')}</option>
            {#each activityRelationshipLookups.deals as deal (deal.id)}
              <option value={deal.id}>{deal.name}</option>
            {/each}
          </select>
        </div>

        <!-- Actions -->
        <div class="qa-actions">
          <button
            class="btn btn-primary btn-sm"
            type="submit"
            disabled={qaSubmitting || !qaSubject.trim()}
          >
            {qaSubmitting ? t('common.loading') : t('common.add')}
          </button>
          <button
            class="btn btn-secondary btn-sm"
            type="button"
            onclick={cancelQuickAdd}
          >
            {t('common.cancel')}
          </button>
        </div>
      </div>
    </form>
  {/if}

  <section class="activity-workbench" aria-labelledby="activity-workbench-heading" data-testid="activity-workbench">
    <div class="activity-workbench-header">
      <div>
        <p class="activity-workbench-eyebrow">{t('activities.workbenchEyebrow')}</p>
        <h2 id="activity-workbench-heading">{t('activities.workbenchTitle')}</h2>
      </div>
      <div class="activity-workbench-open">
        <span>{t('activities.openWork')}</span>
        <strong>{workbench.summary.open}</strong>
      </div>
    </div>

    <div class="activity-summary-grid" aria-label={t('activities.workbenchSummary')}>
      <button
        class="activity-summary-card"
        class:active={bucketFilter === 'overdue'}
        type="button"
        onclick={() => handleBucketFilter(bucketFilter === 'overdue' ? '' : 'overdue')}
      >
        <span>{t('activities.buckets.overdue')}</span>
        <strong>{workbench.summary.overdue}</strong>
      </button>
      <button
        class="activity-summary-card"
        class:active={bucketFilter === 'today'}
        type="button"
        onclick={() => handleBucketFilter(bucketFilter === 'today' ? '' : 'today')}
      >
        <span>{t('activities.buckets.today')}</span>
        <strong>{workbench.summary.today}</strong>
      </button>
      <button
        class="activity-summary-card"
        class:active={bucketFilter === 'thisWeek'}
        type="button"
        onclick={() => handleBucketFilter(bucketFilter === 'thisWeek' ? '' : 'thisWeek')}
      >
        <span>{t('activities.buckets.thisWeek')}</span>
        <strong>{workbench.summary.thisWeek}</strong>
      </button>
      <button
        class="activity-summary-card"
        class:active={bucketFilter === 'unscheduled'}
        type="button"
        onclick={() => handleBucketFilter(bucketFilter === 'unscheduled' ? '' : 'unscheduled')}
      >
        <span>{t('activities.buckets.unscheduled')}</span>
        <strong>{workbench.summary.unscheduled}</strong>
      </button>
    </div>
  </section>

  <!-- Filters -->
  <div class="activities-filters">
    <div class="filter-group" role="group" aria-label={t('activities.dueFocus')}>
      <span class="filter-group-label">{t('activities.dueFocus')}:</span>
      <button
        class="filter-chip"
        class:active={!bucketFilter}
        onclick={() => handleBucketFilter('')}
        type="button"
      >
        {t('common.all')}
      </button>
      {#each ACTIVITY_DUE_BUCKETS as bucket (bucket)}
        <button
          class="filter-chip"
          class:active={bucketFilter === bucket}
          onclick={() => handleBucketFilter(bucket)}
          type="button"
        >
          {bucketLabel(bucket)}
        </button>
      {/each}
    </div>

    <!-- Type filters -->
    <div class="filter-group" role="group" aria-label={t('activities.type')}>
      <span class="filter-group-label">{t('activities.type')}:</span>
      {#each [
        { value: '', label: t('common.all') },
        { value: 'task', label: t('activities.task') },
        { value: 'call', label: t('activities.call') },
        { value: 'meeting', label: t('activities.meeting') },
        { value: 'email', label: t('activities.email') },
        { value: 'visit', label: t('activities.visit') },
        { value: 'whatsapp', label: t('activities.whatsapp') },
        { value: 'sms', label: t('activities.sms') },
      ] as opt (opt.value)}
        <button
          class="filter-chip"
          class:active={typeFilter === opt.value}
          onclick={() => handleTypeFilter(opt.value as ActivityType | '')}
          type="button"
        >
          {opt.label}
        </button>
      {/each}
    </div>

    <!-- Status filters -->
    <div class="filter-group" role="group" aria-label={t('common.status')}>
      <span class="filter-group-label">{t('common.status')}:</span>
      {#each [
        { value: '', label: t('common.all') },
        { value: 'pending', label: t('activities.upcoming') },
        { value: 'completed', label: t('activities.completed') },
        { value: 'overdue', label: t('activities.overdue') },
      ] as opt (opt.value)}
        <button
          class="filter-chip"
          class:active={statusFilter === opt.value}
          onclick={() => handleStatusFilter(opt.value as ActivityStatus | '')}
          type="button"
        >
          {opt.label}
        </button>
      {/each}
    </div>

    <div class="filter-group" role="group" aria-label={t('common.customFieldFilter')}>
      <span class="filter-group-label">{t('common.customField')}:</span>
      <select
        class="input filter-select"
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
        class="input filter-value-input selectable"
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
        <span class="filter-group-label">{t('common.loading')}</span>
      {/if}
    </div>

    {#if customFieldFilterError}
      <div class="filter-error" role="status">{customFieldFilterError}</div>
    {/if}
  </div>

  {#if viewMode === 'week'}
    <section class="week-toolbar" aria-label={t('activities.weekNav')}>
      <button class="btn btn-ghost btn-sm" type="button" onclick={() => moveWeek(-1)}>
        {t('activities.previousWeek')}
      </button>
      <button class="btn btn-secondary btn-sm" type="button" onclick={() => moveWeek(0)}>
        {t('activities.thisWeekNav')}
      </button>
      <button class="btn btn-ghost btn-sm" type="button" onclick={() => moveWeek(1)}>
        {t('activities.nextWeek')}
      </button>
      <strong class="week-range">
        {formatDate(week.weekStart, settingsStore.dateFormat as 'MMM D, YYYY')}
        –
        {formatDate(week.weekEnd, settingsStore.dateFormat as 'MMM D, YYYY')}
      </strong>
    </section>
  {:else if viewMode === 'month'}
    <section class="week-toolbar" aria-label={t('activities.monthNav')}>
      <button class="btn btn-ghost btn-sm" type="button" onclick={() => moveMonth(-1)}>
        {t('activities.previousMonth')}
      </button>
      <button class="btn btn-secondary btn-sm" type="button" onclick={() => moveMonth(0)}>
        {t('activities.thisMonthNav')}
      </button>
      <button class="btn btn-ghost btn-sm" type="button" onclick={() => moveMonth(1)}>
        {t('activities.nextMonth')}
      </button>
      <strong class="week-range">{formatMonthTitle(month.monthStart)}</strong>
    </section>
  {/if}

  <!-- Activity workbench list -->
  <div class="activities-list-card">
    {#if activityStore.isLoading}
      <!-- Skeleton rows -->
      <ul class="activities-list" aria-label={t('common.loading')}>
        {#each [1, 2, 3, 4, 5] as i (i)}
          <li class="activity-row skeleton-row">
            <div class="skeleton skeleton-icon"></div>
            <div class="skeleton-text-block">
              <div class="skeleton skeleton-title"></div>
              <div class="skeleton skeleton-sub"></div>
            </div>
          </li>
        {/each}
      </ul>

    {:else if (viewMode === 'list' && visibleActivityCount === 0) || (isCalendarView && filteredActivities.length === 0 && hasActiveActivityFilters)}
      <EmptyState
        icon="activities"
        title={isFirstRunEmpty ? t('activities.noActivities') : t('activities.noMatchingTitle')}
        description={isFirstRunEmpty ? t('activities.noActivitiesDesc') : t('activities.noMatchingDesc')}
        actionLabel={t('activities.addActivity')}
        onaction={() => uiStore.openModal('addActivity')}
      />

    {:else if viewMode === 'week'}
      <div class="week-layout" data-testid="activity-week">
        <div class="week-grid" aria-label={t('activities.viewWeek')}>
          {#each week.days as day (day.date)}
            <div
              class="week-day"
              class:week-day--today={day.isToday}
              class:week-day--drop={weekDragOverDate === day.date}
              role="group"
              aria-label={formatWeekDayLabel(day.date)}
              ondragover={(event) => handleWeekDragOver(day.date, event)}
              ondragleave={() => handleWeekDragLeave(day.date)}
              ondrop={(event) => void handleWeekDrop(day.date, event)}
            >
              <header class="week-day-header">
                <div>
                  <span class="week-day-name">{formatWeekday(day.date)}</span>
                  <span class="week-day-date">{formatWeekDayLabel(day.date)}</span>
                  {#if day.isToday}
                    <span class="week-today-badge">{t('activities.todayLabel')}</span>
                  {/if}
                </div>
                <button
                  class="btn btn-ghost btn-sm"
                  type="button"
                  onclick={() => addFollowUpOnDay(day.date)}
                  aria-label={t('activities.addOnDay', { date: formatWeekDayLabel(day.date) })}
                >
                  +
                </button>
              </header>
              {#if day.activities.length === 0}
                <p class="week-day-empty">{t('activities.weekEmpty')}</p>
              {:else}
                <ul class="week-day-list">
                  {#each day.activities as activity (activity.id)}
                    <li>
                      <article
                        class="week-card"
                        class:week-card--completed={activity.status === 'completed'}
                        class:week-card--overdue={activity.status === 'overdue'}
                        draggable={activity.status !== 'completed'}
                        ondragstart={(event) => handleWeekDragStart(activity.id, event)}
                      >
                        <span class="week-card-type">{t(`activities.${activity.type}`)}</span>
                        <strong class="week-card-subject">{activity.subject}</strong>
                        <button
                          class="btn-complete"
                          class:btn-complete--done={activity.status === 'completed'}
                          onclick={() => handleToggleComplete(activity)}
                          type="button"
                          aria-label={activity.status === 'completed'
                            ? t('activities.markIncomplete')
                            : t('activities.markComplete')}
                        >
                          {#if activity.status === 'completed'}
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
                              <polyline points="20 6 9 17 4 12"/>
                            </svg>
                          {:else}
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                              <circle cx="12" cy="12" r="9"/>
                            </svg>
                          {/if}
                        </button>
                      </article>
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/each}
        </div>

        <aside class="week-unscheduled" aria-labelledby="week-unscheduled-heading">
          <h2 id="week-unscheduled-heading">{t('activities.buckets.unscheduled')}</h2>
          <p>{t('activities.unscheduledHelp')}</p>
          {#if week.unscheduled.length === 0}
            <p class="week-day-empty">{t('activities.noDueDate')}</p>
          {:else}
            <ul class="week-day-list">
              {#each week.unscheduled as activity (activity.id)}
                <li>
                  <article
                    class="week-card"
                    draggable={true}
                    ondragstart={(event) => handleWeekDragStart(activity.id, event)}
                  >
                    <span class="week-card-type">{t(`activities.${activity.type}`)}</span>
                    <strong class="week-card-subject">{activity.subject}</strong>
                    <button
                      class="btn-complete"
                      onclick={() => handleToggleComplete(activity)}
                      type="button"
                      aria-label={t('activities.markComplete')}
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                        <circle cx="12" cy="12" r="9"/>
                      </svg>
                    </button>
                  </article>
                </li>
              {/each}
            </ul>
          {/if}
        </aside>
      </div>

    {:else if viewMode === 'month'}
      <div class="week-layout" data-testid="activity-month">
        <div class="month-board">
          <div class="month-weekdays" aria-hidden="true">
            {#each weekdayHeadings() as heading, index (index)}
              <span>{heading}</span>
            {/each}
          </div>
          <div class="month-grid" aria-label={t('activities.viewMonth')}>
            {#each month.weeks as monthWeek, weekIndex (weekIndex)}
              {#each monthWeek as day (day.date)}
                {@const extraCount = Math.max(0, day.activities.length - 3)}
                <div
                  class="month-day"
                  class:month-day--outside={!day.inMonth}
                  class:week-day--today={day.isToday}
                  class:week-day--drop={weekDragOverDate === day.date}
                  role="group"
                  aria-label={formatWeekDayLabel(day.date)}
                  ondragover={(event) => handleWeekDragOver(day.date, event)}
                  ondragleave={() => handleWeekDragLeave(day.date)}
                  ondrop={(event) => void handleWeekDrop(day.date, event)}
                >
                  <header class="week-day-header">
                    <span class="month-day-number">{day.date.slice(8)}</span>
                    <button
                      class="btn btn-ghost btn-sm"
                      type="button"
                      onclick={() => addFollowUpOnDay(day.date)}
                      aria-label={t('activities.addOnDay', { date: formatWeekDayLabel(day.date) })}
                    >
                      +
                    </button>
                  </header>
                  <ul class="week-day-list">
                    {#each day.activities.slice(0, 3) as activity (activity.id)}
                      <li>
                        <article
                          class="week-card month-card"
                          class:week-card--completed={activity.status === 'completed'}
                          class:week-card--overdue={activity.status === 'overdue'}
                          draggable={activity.status !== 'completed'}
                          ondragstart={(event) => handleWeekDragStart(activity.id, event)}
                        >
                          <strong class="week-card-subject">{activity.subject}</strong>
                          <button
                            class="btn-complete"
                            class:btn-complete--done={activity.status === 'completed'}
                            onclick={() => handleToggleComplete(activity)}
                            type="button"
                            aria-label={activity.status === 'completed'
                              ? t('activities.markIncomplete')
                              : t('activities.markComplete')}
                          >
                            {#if activity.status === 'completed'}
                              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
                                <polyline points="20 6 9 17 4 12"/>
                              </svg>
                            {:else}
                              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                                <circle cx="12" cy="12" r="9"/>
                              </svg>
                            {/if}
                          </button>
                        </article>
                      </li>
                    {/each}
                  </ul>
                  {#if extraCount > 0}
                    <p class="month-more">{t('activities.monthMore', { count: extraCount })}</p>
                  {/if}
                </div>
              {/each}
            {/each}
          </div>
        </div>

        <aside class="week-unscheduled" aria-labelledby="month-unscheduled-heading">
          <h2 id="month-unscheduled-heading">{t('activities.buckets.unscheduled')}</h2>
          <p>{t('activities.unscheduledHelp')}</p>
          {#if month.unscheduled.length === 0}
            <p class="week-day-empty">{t('activities.noDueDate')}</p>
          {:else}
            <ul class="week-day-list">
              {#each month.unscheduled as activity (activity.id)}
                <li>
                  <article
                    class="week-card"
                    draggable={true}
                    ondragstart={(event) => handleWeekDragStart(activity.id, event)}
                  >
                    <span class="week-card-type">{t(`activities.${activity.type}`)}</span>
                    <strong class="week-card-subject">{activity.subject}</strong>
                    <button
                      class="btn-complete"
                      onclick={() => handleToggleComplete(activity)}
                      type="button"
                      aria-label={t('activities.markComplete')}
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                        <circle cx="12" cy="12" r="9"/>
                      </svg>
                    </button>
                  </article>
                </li>
              {/each}
            </ul>
          {/if}
        </aside>
      </div>

    {:else}
      <div class="activity-buckets">
        {#each visibleBuckets as bucket (bucket.bucket)}
          {#if bucket.activities.length > 0}
            <section class="activity-bucket card" aria-labelledby="activity-bucket-{bucket.bucket}">
              <div class="activity-bucket-header">
                <div>
                  <h2 id="activity-bucket-{bucket.bucket}">{bucketLabel(bucket.bucket)}</h2>
                  <p>{bucketDescription(bucket.bucket)}</p>
                </div>
                <span>{bucket.activities.length}</span>
              </div>
              <ul class="activities-list" role="list">
                {#each bucket.activities as activity (activity.id)}
                  {@const relationships = deriveActivityRelationshipLabels(
                    activity,
                    activityLinkIndex[activity.id] ?? [],
                    activityRelationshipLookups,
                  )}
                  <li class="activity-row" class:activity-row--completed={activity.status === 'completed'}>
                    <!-- Type icon -->
                    <div class="activity-icon-wrap activity-type-{activity.type}" aria-hidden="true">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <path d={activityIcon(activity.type)}/>
                      </svg>
                    </div>

                    <!-- Content -->
                    <div class="activity-content">
                      <div class="activity-header-row">
                        <span class="activity-subject">{activity.subject}</span>
                        <span class="activity-status-badge {statusClass(activity.status)}">
                          {t(`activities.${activity.status === 'pending' ? 'upcoming' : activity.status}`)}
                        </span>
                      </div>
                      <div class="activity-meta">
                        {#if activity.dueDate}
                          <span class="activity-due">
                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                              <rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>
                            </svg>
                            {formatActivityDue(activity.dueDate, settingsStore.language)}
                          </span>
                        {:else}
                          <span class="activity-due">{t('activities.noDueDate')}</span>
                        {/if}
                        {#each relationships.contacts as contact (contact.id)}
                          <span class="activity-linked">
                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                              <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2M12 11a4 4 0 100-8 4 4 0 000 8"/>
                            </svg>
                            {contact.label}
                          </span>
                        {/each}
                        {#each relationships.organizations as organization (organization.id)}
                          <span class="activity-linked">
                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                              <path d="M3 21h18M5 21V7l7-4 7 4v14M9 21v-6h6v6M9 10h.01M15 10h.01"/>
                            </svg>
                            {organization.label}
                          </span>
                        {/each}
                        {#each relationships.deals as deal (deal.id)}
                          <span class="activity-linked">
                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                              <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                            </svg>
                            {deal.label}
                          </span>
                        {/each}
                        <span class="activity-time">{formatRelativeTime(activity.createdAt)}</span>
                      </div>
                    </div>

                    <div class="activity-row-actions">
                      {#if activity.status !== 'completed'}
                        <button
                          class="btn btn-ghost btn-sm"
                          type="button"
                          onclick={() => handleSnooze(activity.id)}
                          disabled={reschedulingActivityId === activity.id}
                        >
                          {t('activities.snoozeTomorrow')}
                        </button>
                      {/if}
                      <label class="reschedule-control">
                        <span class="sr-only">{t('activities.reschedule')}</span>
                        <input
                          class="input reschedule-input"
                          type="date"
                          value={activity.dueDate ?? ''}
                          disabled={reschedulingActivityId === activity.id}
                          onchange={(event) => handleReschedule(activity.id, event)}
                          aria-label={t('activities.reschedule')}
                        />
                      </label>
                      <button
                        class="btn-complete"
                        class:btn-complete--done={activity.status === 'completed'}
                        onclick={() => handleToggleComplete(activity)}
                        type="button"
                        aria-label={activity.status === 'completed'
                          ? t('activities.markIncomplete')
                          : t('activities.markComplete')}
                        title={activity.status === 'completed'
                          ? t('activities.markIncomplete')
                          : t('activities.markComplete')}
                      >
                        {#if activity.status === 'completed'}
                          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
                            <polyline points="20 6 9 17 4 12"/>
                          </svg>
                        {:else}
                          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                            <circle cx="12" cy="12" r="9"/>
                          </svg>
                        {/if}
                      </button>
                    </div>
                  </li>
                {/each}
              </ul>
            </section>
          {/if}
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  /* ── Page ─────────────────────────────────────────────────────────────────── */

  .activities-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .toolbar {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  .layout-toggle {
    display: inline-flex;
    gap: var(--space-1);
    padding: 2px;
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-default);
  }

  .layout-toggle-button {
    min-height: 28px;
    padding: var(--space-1) var(--space-3);
    border: none;
    border-radius: calc(var(--radius-md) - 2px);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    cursor: pointer;
  }

  .layout-toggle-button--active {
    background: var(--surface-active);
    color: var(--text-accent);
  }

  .week-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .week-range {
    margin-inline-start: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-primary);
  }

  .week-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(180px, 220px);
    gap: var(--space-4);
    padding: var(--space-4);
  }

  .week-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: var(--space-2);
    min-width: 0;
  }

  .week-day,
  .week-unscheduled {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-height: 220px;
    padding: var(--space-2);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-default);
  }

  .week-day--today {
    border-color: var(--color-primary);
    background: var(--surface-active);
  }

  .week-day--drop {
    border-color: var(--color-primary);
    box-shadow: inset 0 0 0 1px var(--color-primary);
  }

  .week-day-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-1);
  }

  .week-day-name,
  .week-day-date {
    display: block;
    font-size: var(--text-xs);
  }

  .week-day-name {
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    text-transform: capitalize;
  }

  .week-day-date,
  .week-today-badge,
  .week-day-empty,
  .week-unscheduled p {
    color: var(--text-secondary);
  }

  .week-today-badge {
    display: inline-block;
    margin-top: 2px;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-accent);
  }

  .week-unscheduled h2 {
    margin: 0;
    font-size: var(--text-sm);
  }

  .week-unscheduled p,
  .week-day-empty {
    margin: 0;
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .week-day-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .week-card {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 2px var(--space-2);
    padding: var(--space-2);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--surface-raised);
    cursor: grab;
  }

  .week-card--completed {
    opacity: 0.65;
    cursor: default;
  }

  .week-card--overdue {
    border-color: var(--color-danger-200);
  }

  .week-card-type {
    grid-column: 1;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-tertiary);
  }

  .week-card-subject {
    grid-column: 1;
    font-size: var(--text-xs);
    color: var(--text-primary);
    overflow-wrap: anywhere;
  }

  .week-card .btn-complete {
    grid-row: 1 / span 2;
    grid-column: 2;
    align-self: center;
  }

  .month-board {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
  }

  .month-weekdays,
  .month-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: var(--space-1);
  }

  .month-weekdays span {
    padding: 0 var(--space-1);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
    text-transform: capitalize;
  }

  .month-day {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-height: 112px;
    padding: var(--space-1);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--surface-default);
  }

  .month-day--outside {
    opacity: 0.55;
  }

  .month-day-number {
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .month-card {
    padding: 4px var(--space-1);
  }

  .month-card .week-card-subject {
    font-size: 11px;
  }

  .month-more {
    margin: 0;
    font-size: 10px;
    color: var(--text-secondary);
  }

  @media (max-width: 900px) {
    .week-layout,
    .week-grid,
    .month-weekdays,
    .month-grid {
      grid-template-columns: 1fr;
    }
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

  .activity-workbench {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    background: var(--surface-default);
  }

  .activity-workbench-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .activity-workbench-eyebrow {
    margin: 0 0 var(--space-1);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    text-transform: uppercase;
  }

  .activity-workbench h2 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .activity-workbench-open {
    display: grid;
    gap: var(--space-1);
    min-width: 120px;
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    text-align: end;
  }

  .activity-workbench-open span {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .activity-workbench-open strong {
    font-size: var(--text-xl);
    color: var(--text-primary);
  }

  .activity-summary-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .activity-summary-card {
    display: grid;
    gap: var(--space-2);
    min-height: 86px;
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    color: var(--text-primary);
    text-align: start;
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .activity-summary-card:hover,
  .activity-summary-card.active {
    border-color: var(--color-primary-200);
    background: var(--surface-active);
  }

  .activity-summary-card span {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .activity-summary-card strong {
    font-size: var(--text-2xl);
    line-height: 1;
  }

  /* ── Quick-add form ──────────────────────────────────────────────────────── */

  .quick-add-form {
    padding: var(--space-5);
  }

  .quick-add-row {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    flex-wrap: wrap;
  }

  .qa-field { flex-shrink: 0; }

  .qa-field--type    { min-width: 120px; }
  .qa-field--subject { flex: 1; min-width: 200px; }
  .qa-field--date    { min-width: 148px; }
  .qa-field--relationship { min-width: 172px; }

  .qa-actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .select-input {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%2313343B' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right var(--space-3) center;
    padding-inline-end: var(--space-8);
    cursor: pointer;
  }

  /* ── Filters ─────────────────────────────────────────────────────────────── */

  .activities-filters {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .filter-group-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    white-space: nowrap;
    margin-inline-end: var(--space-1);
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
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .filter-chip.active {
    background-color: var(--surface-active);
    color: var(--text-accent);
    border-color: var(--color-primary-200);
  }

  .filter-select {
    min-width: 180px;
    height: 32px;
  }

  .filter-value-input {
    min-width: 180px;
    height: 32px;
  }

  .filter-error {
    font-size: var(--text-xs);
    color: var(--text-danger);
  }

  /* ── Activity list card ──────────────────────────────────────────────────── */

  .activities-list-card {
    flex: 1;
    min-width: 0;
  }

  .activity-buckets {
    display: grid;
    gap: var(--space-4);
  }

  .activity-bucket {
    overflow: hidden;
  }

  .activity-bucket-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-6);
    border-block-end: var(--border-width) solid var(--border-default);
  }

  .activity-bucket-header h2 {
    margin: 0;
    font-size: var(--text-base);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .activity-bucket-header p {
    margin: var(--space-1) 0 0;
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .activity-bucket-header > span {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 28px;
    height: 24px;
    padding: 0 var(--space-2);
    border-radius: 9999px;
    border: var(--border-width) solid var(--border-default);
    background: var(--surface-default);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
  }

  .activities-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  /* ── Activity row ────────────────────────────────────────────────────────── */

  .activity-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-6);
    border-block-end: var(--border-width) solid var(--border-default);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .activity-row:last-child {
    border-block-end: none;
  }

  .activity-row:hover {
    background-color: var(--surface-raised);
  }

  .activity-row--completed {
    opacity: 0.65;
  }

  /* ── Activity type icon ──────────────────────────────────────────────────── */

  .activity-icon-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-block-start: 1px;
  }

  .activity-type-task    { background: #E8F4F7; color: #20808D; }
  .activity-type-call    { background: #E8F5EC; color: #2D8659; }
  .activity-type-meeting { background: #FFF8E1; color: #D4A017; }
  .activity-type-email   { background: #FEF3E2; color: #A84B2F; }

  /* ── Activity content ────────────────────────────────────────────────────── */

  .activity-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .activity-row-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-2);
    flex-shrink: 0;
    margin-block-start: -1px;
  }

  .reschedule-control {
    display: inline-flex;
    align-items: center;
  }

  .reschedule-input {
    width: 138px;
    height: 28px;
    font-size: var(--text-xs);
  }

  .activity-header-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .activity-subject {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .activity-meta {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    flex-wrap: wrap;
  }

  .activity-due,
  .activity-linked,
  .activity-time {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  /* ── Status badges ───────────────────────────────────────────────────────── */

  .activity-status-badge {
    display: inline-block;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    border-radius: 9999px;
    padding: 1px var(--space-2);
    white-space: nowrap;
  }

  .status-pending   { background: #E8F4F7; color: #20808D; }
  .status-completed { background: #E8F5EC; color: #2D8659; }
  .status-overdue   { background: #FFF0F0; color: #C0392B; }

  /* ── Mark complete button ────────────────────────────────────────────────── */

  .btn-complete {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: transparent;
    border: var(--border-width) solid var(--border-default);
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    margin-block-start: 2px;
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }

  .btn-complete:hover {
    border-color: var(--color-success);
    color: var(--color-success);
  }

  .btn-complete--done {
    background-color: var(--color-success);
    border-color: var(--color-success);
    color: #fff;
  }

  .btn-complete--done:hover {
    background-color: #1f6640;
    border-color: #1f6640;
    color: #fff;
  }

  /* ── Skeleton rows ───────────────────────────────────────────────────────── */

  .skeleton-row {
    align-items: center;
    padding: var(--space-4) var(--space-6);
  }

  .skeleton-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .skeleton-text-block {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .skeleton-title {
    height: 14px;
    width: 60%;
    border-radius: var(--radius-sm);
  }

  .skeleton-sub {
    height: 11px;
    width: 40%;
    border-radius: var(--radius-sm);
  }

  /* ── Screen reader only ──────────────────────────────────────────────────── */

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
  }

  @media (max-width: 1040px) {
    .activity-summary-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .activity-row {
      align-items: stretch;
    }

    .activity-row-actions {
      flex-wrap: wrap;
      max-width: 180px;
    }
  }

  @media (max-width: 720px) {
    .activity-workbench-header {
      flex-direction: column;
      align-items: stretch;
    }

    .activity-workbench-open {
      text-align: start;
    }

    .activity-summary-grid {
      grid-template-columns: 1fr;
    }

    .activity-row {
      flex-wrap: wrap;
    }

    .activity-row-actions {
      width: 100%;
      justify-content: flex-start;
      max-width: none;
      padding-inline-start: 40px;
    }
  }
</style>
