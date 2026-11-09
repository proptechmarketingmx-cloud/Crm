<script lang="ts">
  /**
   * DealDetail.svelte — Full deal workspace for 900CRM.
   *
   * The pipeline board stays a kanban. This page is the deal itself:
   * stage, guidance, people, notes, and follow-ups in one place.
   */

  import { t } from '$lib/i18n';
  import { dealStore } from '$lib/stores/deals';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import { getDeal, DEAL_STAGES, type Deal, type DealStage } from '$lib/api/deals';
  import { listActivities, type Activity } from '$lib/api/activities';
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
    deriveDealRelationshipLabels,
    loadDealRelationshipLookups,
  } from '$lib/utils/dealRelationships';
  import {
    derivePipelineGuidance,
    weightedForecastValue,
    type PipelineGuidance,
  } from '$lib/utils/pipelineGuidance';
  import {
    formatCurrency,
    formatDate,
    formatPercent,
    formatRelativeTime,
  } from '$lib/utils/formatters';
  import { navigateHash } from '$lib/utils/hashRouter';
  import ActivityFeed from '$lib/components/ActivityFeed.svelte';
  import LogJustHappened from '$lib/components/LogJustHappened.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import NextStepCard from '$lib/components/NextStepCard.svelte';
  import EntityNotesPanel from '$lib/components/EntityNotesPanel.svelte';
  import EntityLinksPanel from '$lib/components/EntityLinksPanel.svelte';
  import EntityTagsPanel from '$lib/components/EntityTagsPanel.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import {
    deriveRecordNextStep,
    shouldShowSecondaryFollowUp,
  } from '$lib/utils/recordNextStep';

  const { dealId }: { dealId: string } = $props();

  let deal = $state<Deal | null>(null);
  let isLoading = $state(true);
  let isSaving = $state(false);
  let isDeleting = $state(false);
  let showDeleteConfirm = $state(false);
  let loadError = $state<string | null>(null);
  let loadedDealId = '';
  let lastActivityRefreshVersion = -1;

  let dealName = $state('');
  let dealValue = $state(0);
  let dealProbability = $state(0);
  let dealExpectedClose = $state('');
  let dealDescription = $state('');
  let dealOwner = $state('');
  let isDirty = $state(false);

  let activities = $state<Activity[]>([]);
  let activitiesLoading = $state(false);
  let activityContextError = $state<string | null>(null);
  let activityLinkIndex = $state<ActivityLinkIndex>({});
  let activityLookups = $state<ActivityRelationshipLookups>({
    contacts: [],
    organizations: [],
    deals: [],
  });
  let primaryContactName = $state<string | null>(null);
  let organizationName = $state<string | null>(null);

  const formattedValue = $derived(
    deal ? formatCurrency(deal.value, deal.currency, settingsStore.language) : t('common.none'),
  );

  const weightedForecast = $derived(
    deal
      ? formatCurrency(weightedForecastValue(deal), deal.currency, settingsStore.language)
      : t('common.none'),
  );

  const expectedCloseLabel = $derived(
    deal?.expectedCloseDate
      ? formatDate(deal.expectedCloseDate, settingsStore.dateFormat as 'MMM D, YYYY', settingsStore.language)
      : t('common.none'),
  );

  const dealActivities = $derived.by(() =>
    sortActivitiesForDetailTimeline(
      filterActivitiesByRelationship(activities, activityLinkIndex, {
        dealId,
      }),
    ),
  );

  const activityRelationships = $derived.by<Record<string, ActivityRelationshipLabels>>(() =>
    relationshipLabelsByActivityId(dealActivities, activityLinkIndex, activityLookups),
  );

  const guidance = $derived.by<PipelineGuidance | null>(() => {
    if (!deal || activitiesLoading) {
      return null;
    }
    if (activityContextError) {
      return null;
    }
    return derivePipelineGuidance({ deal, activities: dealActivities });
  });

  const guidanceTone = $derived(guidance?.tone ?? 'neutral');
  const nextStep = $derived(
    deriveRecordNextStep({
      recordKind: 'deal',
      isLoading: !deal || activitiesLoading,
      unavailable: Boolean(activityContextError),
      isClosedWon: deal?.stage === 'closedWon',
      isClosedLost: deal?.stage === 'closedLost',
      overdueActivities:
        guidance?.state === 'overdue' && guidance.nextActivity
          ? [guidance.nextActivity]
          : [],
      nextActivity: guidance?.nextActivity ?? null,
      expectedCloseDate: deal?.expectedCloseDate ?? null,
      isStale: guidance?.state === 'stale',
    }),
  );

  $effect(() => {
    if (!dealId || loadedDealId === dealId) {
      return;
    }
    loadedDealId = dealId;
    void loadWorkspace(dealId);
  });

  $effect(() => {
    if (!deal || loadedDealId !== dealId) {
      return;
    }
    const version = activityStore.relationshipRefreshVersion;
    if (lastActivityRefreshVersion === version) {
      return;
    }
    lastActivityRefreshVersion = version;
    if (version > 0) {
      void loadActivities();
    }
  });

  function populateForm(next: Deal) {
    dealName = next.name;
    dealValue = next.value;
    dealProbability = next.probability;
    dealExpectedClose = next.expectedCloseDate ?? '';
    dealDescription = next.description ?? '';
    dealOwner = next.owner ?? '';
    isDirty = false;
  }

  async function loadWorkspace(id: string) {
    isLoading = true;
    loadError = null;
    try {
      const loaded = await getDeal(id);
      deal = loaded;
      dealStore.selectDeal(loaded);
      populateForm(loaded);
      await Promise.all([loadRelationships(loaded), loadActivities()]);
    } catch (err) {
      console.error('[DealDetail] Failed to load deal:', err);
      loadError = t('deals.notFound');
      deal = null;
    } finally {
      isLoading = false;
    }
  }

  async function loadRelationships(current: Deal) {
    try {
      const lookups = await loadDealRelationshipLookups();
      const labels = deriveDealRelationshipLabels(
        current,
        lookups.contacts,
        lookups.organizations,
      );
      primaryContactName = labels.primaryContactName;
      organizationName = labels.organizationName;
    } catch (err) {
      console.error('[DealDetail] Failed to load relationships:', err);
      primaryContactName = null;
      organizationName = null;
    }
  }

  async function loadActivities() {
    activitiesLoading = true;
    activityContextError = null;
    try {
      const allActivities = await listActivities({
        sortBy: 'dueDate',
        sortDir: 'asc',
        pageSize: 500,
      });
      const [linkIndex, lookups] = await Promise.all([
        loadActivityLinkIndex(allActivities.map((activity) => activity.id)),
        loadActivityRelationshipLookups(),
      ]);
      activities = allActivities;
      activityLinkIndex = linkIndex;
      activityLookups = lookups;
    } catch (err) {
      console.error('[DealDetail] Failed to load activities:', err);
      activityContextError = t('deals.guidance.activityContextFailed');
      activities = [];
    } finally {
      activitiesLoading = false;
    }
  }

  function guidanceLabel(): string {
    if (guidance === null) {
      return activityContextError ? t('deals.guidance.unavailable') : t('deals.guidance.loading');
    }
    return t(`deals.guidance.${guidance.state}`);
  }

  function guidanceDetail(): string {
    if (guidance === null) {
      return activityContextError ?? t('deals.guidance.loadingDetail');
    }
    if (guidance.state === 'overdue' && guidance.nextActivity) {
      return t('deals.guidance.overdueDetail', { subject: guidance.nextActivity.subject });
    }
    if (guidance.state === 'onTrack' && guidance.nextActivity) {
      return t('deals.guidance.onTrackDetail', { subject: guidance.nextActivity.subject });
    }
    return t(`deals.guidance.${guidance.state}Detail`);
  }

  function markDirty() {
    isDirty = true;
  }

  async function handleSave() {
    if (!deal || !dealName.trim()) {
      uiStore.toastError(t('deals.nameRequired'));
      return;
    }

    isSaving = true;
    try {
      const updated = await dealStore.updateDeal(deal.id, {
        name: dealName.trim(),
        value: Number.isFinite(dealValue) ? dealValue : 0,
        probability: Number.isFinite(dealProbability) ? dealProbability : 0,
        expectedCloseDate: dealExpectedClose || null,
        description: dealDescription.trim() || null,
        owner: dealOwner.trim() || null,
      });
      deal = updated;
      populateForm(updated);
    } catch (err) {
      console.error('[DealDetail] Save error:', err);
    } finally {
      isSaving = false;
    }
  }

  async function handleStageChange(event: Event) {
    if (!deal) return;
    const stage = (event.target as HTMLSelectElement).value as DealStage;
    if (!DEAL_STAGES.includes(stage) || stage === deal.stage) {
      return;
    }
    isSaving = true;
    try {
      await dealStore.moveDealStage(deal.id, stage);
      const updated = await getDeal(deal.id);
      deal = updated;
      populateForm(updated);
    } catch (err) {
      console.error('[DealDetail] Stage move error:', err);
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete() {
    if (!deal) return;
    isDeleting = true;
    try {
      await dealStore.deleteDeal(deal.id);
      navigateHash('/pipeline');
    } catch (err) {
      console.error('[DealDetail] Delete error:', err);
      uiStore.toastError(t('errors.deleteNamed', { name: t('entities.deal') }));
    } finally {
      isDeleting = false;
      showDeleteConfirm = false;
    }
  }

  function handleBack() {
    navigateHash('/pipeline');
  }

  function openFollowUp() {
    if (!deal) return;
    dealStore.selectDeal(deal);
    uiStore.openModal('addActivity', {
      dealId: deal.id,
      contactId: deal.contactId ?? '',
      organizationId: deal.organizationId ?? '',
    });
  }

  function focusExpectedClose() {
    const input = document.getElementById('deal-close');
    if (!(input instanceof HTMLInputElement)) {
      return;
    }
    input.focus();
    input.scrollIntoView({ block: 'center', behavior: 'smooth' });
  }

  async function handleNextStep() {
    if (nextStep.action === 'complete' && nextStep.activityId) {
      isSaving = true;
      try {
        await activityStore.markComplete(nextStep.activityId);
        await loadActivities();
      } catch (err) {
        console.error('[DealDetail] Complete next step error:', err);
      } finally {
        isSaving = false;
      }
      return;
    }

    if (nextStep.action === 'addFollowUp') {
      openFollowUp();
      return;
    }

    if (nextStep.action === 'setExpectedClose') {
      focusExpectedClose();
    }
  }

  function handleActivityEntityNavigate(entity: { type: 'contact' | 'organization' | 'deal'; id: string }) {
    if (entity.type === 'contact') {
      navigateHash(`/contacts/${entity.id}`);
      return;
    }
    if (entity.type === 'organization') {
      navigateHash(`/organizations/${entity.id}`);
      return;
    }
    navigateHash(`/deals/${entity.id}`);
  }
</script>

<div class="page-content deal-detail-page">
  {#if isLoading}
    <div class="detail-loading" aria-live="polite" aria-label={t('common.loading')}>
      <div class="skeleton skeleton-header"></div>
      <div class="skeleton-fields">
        {#each [1, 2, 3, 4] as item (item)}
          <div class="skeleton skeleton-field"></div>
        {/each}
      </div>
    </div>
  {:else if loadError}
    <div class="detail-error" role="alert">
      <span>{loadError}</span>
      <button class="btn btn-secondary btn-sm" type="button" onclick={() => loadWorkspace(dealId)}>
        {t('common.retry')}
      </button>
    </div>
  {:else if deal}
    <div class="page-header">
      <div class="header-left">
        <button class="btn-back" onclick={handleBack} type="button" aria-label={t('deals.backToPipeline')}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M19 12H5M12 5l-7 7 7 7"/>
          </svg>
          {t('deals.backToPipeline')}
        </button>
        <div class="deal-identity">
          <h1 class="page-title">{deal.name}</h1>
          <span class="stage-badge stage-{deal.stage}">{t(`deals.stages.${deal.stage}`)}</span>
        </div>
      </div>
      <div class="header-actions">
        {#if isDirty}
          <button class="btn btn-primary btn-sm" type="button" onclick={handleSave} disabled={isSaving}>
            {isSaving ? t('common.loading') : t('common.save')}
          </button>
          <button class="btn btn-secondary btn-sm" type="button" onclick={() => { if (deal) populateForm(deal); }} disabled={isSaving}>
            {t('common.cancel')}
          </button>
        {/if}
        <button class="btn btn-danger btn-sm" type="button" onclick={() => showDeleteConfirm = true}>
          {t('deals.deleteDeal')}
        </button>
      </div>
    </div>

    <section class="deal-workspace" aria-labelledby="deal-workspace-heading">
      <div class="deal-workspace-header">
        <div>
          <p class="workspace-eyebrow">{t('deals.workspace.eyebrow')}</p>
          <h2 class="section-title" id="deal-workspace-heading">{t('deals.workspace.title')}</h2>
        </div>
        <span class="health-badge health-{guidanceTone}">{guidanceLabel()}</span>
      </div>
      <p class="workspace-summary">{guidanceDetail()}</p>
      <NextStepCard step={nextStep} busy={isSaving} onaction={handleNextStep} />
      <div class="workspace-metrics" role="list">
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('deals.value')}</span>
          <strong>{formattedValue}</strong>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('deals.guidance.weightedForecast')}</span>
          <strong>{weightedForecast}</strong>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('deals.probability')}</span>
          <strong>{formatPercent(deal.probability)}</strong>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('deals.expectedClose')}</span>
          <strong>{expectedCloseLabel}</strong>
        </div>
      </div>
      <div class="workspace-actions" aria-label={t('deals.workspace.actionsLabel')}>
        {#if shouldShowSecondaryFollowUp(nextStep)}
          <button class="btn btn-secondary btn-sm" type="button" onclick={openFollowUp}>
            {t('deals.guidance.addFollowUp')}
          </button>
        {/if}
      </div>
    </section>

    <div class="detail-grid">
      <div class="detail-main">
        <section class="card" aria-labelledby="deal-fields-heading">
          <div class="card-header">
            <h2 class="section-title" id="deal-fields-heading">{t('deals.editDeal')}</h2>
          </div>
          <div class="card-body fields-grid">
            <div class="field-group field-group--full">
              <label class="field-label" for="deal-name">{t('deals.name')}</label>
              <input id="deal-name" class="input" bind:value={dealName} oninput={markDirty} />
            </div>
            <div class="field-group">
              <label class="field-label" for="deal-value">{t('deals.value')}</label>
              <input id="deal-value" class="input" type="number" min="0" step="0.01" bind:value={dealValue} oninput={markDirty} />
            </div>
            <div class="field-group">
              <label class="field-label" for="deal-probability">{t('deals.probability')}</label>
              <input id="deal-probability" class="input" type="number" min="0" max="100" bind:value={dealProbability} oninput={markDirty} />
            </div>
            <div class="field-group">
              <label class="field-label" for="deal-stage">{t('deals.stage')}</label>
              <select id="deal-stage" class="select" value={deal.stage} onchange={handleStageChange} disabled={isSaving}>
                {#each DEAL_STAGES as stage (stage)}
                  <option value={stage}>{t(`deals.stages.${stage}`)}</option>
                {/each}
              </select>
            </div>
            <div class="field-group">
              <label class="field-label" for="deal-close">{t('deals.expectedClose')}</label>
              <input id="deal-close" class="input" type="date" bind:value={dealExpectedClose} oninput={markDirty} />
            </div>
            <div class="field-group">
              <label class="field-label" for="deal-owner">{t('common.owner')}</label>
              <input id="deal-owner" class="input" bind:value={dealOwner} oninput={markDirty} placeholder={t('common.optional')} />
            </div>
            <div class="field-group field-group--full">
              <label class="field-label" for="deal-description">{t('deals.description')}</label>
              <textarea id="deal-description" class="input" rows="4" bind:value={dealDescription} oninput={markDirty}></textarea>
            </div>
          </div>
        </section>

        <EntityNotesPanel entityType="deal" entityId={deal.id} />
        <EntityTagsPanel entityType="deal" entityId={deal.id} />
        <EntityLinksPanel entityType="deal" entityId={deal.id} />
      </div>

      <div class="detail-sidebar">
        <section class="card" aria-labelledby="deal-people-heading">
          <div class="card-header">
            <h2 class="section-title" id="deal-people-heading">{t('deals.contact')}</h2>
          </div>
          <div class="card-body people-stack">
            {#if deal.contactId}
              <button class="link-button" type="button" onclick={() => { const id = deal?.contactId; if (id) navigateHash(`/contacts/${id}`); }}>
                {primaryContactName ?? t('deals.contact')}
              </button>
            {:else}
              <p>{t('common.none')}</p>
            {/if}
            <div>
              <span class="workspace-metric-label">{t('deals.organization')}</span>
              {#if deal.organizationId}
                <button class="link-button" type="button" onclick={() => { const id = deal?.organizationId; if (id) navigateHash(`/organizations/${id}`); }}>
                  {organizationName ?? t('deals.organization')}
                </button>
              {:else}
                <p>{t('common.none')}</p>
              {/if}
            </div>
            <p class="meta-line">{t('common.updated')}: {formatRelativeTime(deal.updatedAt)}</p>
          </div>
        </section>

        <section class="card" aria-labelledby="deal-activity-heading">
          <div class="card-header">
            <h2 class="section-title" id="deal-activity-heading">{t('deals.guidance.linkedActivities')}</h2>
            <button class="btn btn-secondary btn-xs" type="button" onclick={openFollowUp}>
              {t('activities.addActivity')}
            </button>
          </div>
          <div class="card-body">
            <LogJustHappened
              dealId={dealId}
              contactId={deal?.contactId}
              onlogged={() => void loadActivities()}
            />
            {#if activityContextError}
              <p class="inline-error">{activityContextError}</p>
            {:else if !activitiesLoading && dealActivities.length === 0}
              <EmptyState
                icon="activities"
                title={t('activities.noActivities')}
                description={t('activities.noActivitiesDesc')}
                compact={true}
              />
            {:else}
              <ActivityFeed
                activities={dealActivities}
                loading={activitiesLoading}
                maxItems={12}
                relationshipsByActivityId={activityRelationships}
                showRelationshipBreadcrumbs={true}
                onNavigateEntity={handleActivityEntityNavigate}
              />
            {/if}
          </div>
        </section>
      </div>
    </div>
  {/if}
</div>

{#if showDeleteConfirm}
  <Modal open={true} title={t('deals.deleteDeal')} size="sm" onclose={() => showDeleteConfirm = false}>
    {#snippet body()}
      <p>{t('deals.confirmDelete')}</p>
    {/snippet}
    {#snippet footer()}
      <button class="btn btn-secondary" type="button" onclick={() => showDeleteConfirm = false}>{t('common.cancel')}</button>
      <button class="btn btn-danger" type="button" onclick={handleDelete} disabled={isDeleting}>
        {isDeleting ? t('common.loading') : t('deals.deleteDeal')}
      </button>
    {/snippet}
  </Modal>
{/if}

<style>
  .deal-detail-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-4);
  }

  .header-left,
  .header-actions,
  .deal-identity,
  .deal-workspace-header,
  .workspace-actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .header-left,
  .deal-identity {
    flex-wrap: wrap;
  }

  .btn-back {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    border: 0;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0;
  }

  .deal-workspace {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    background-color: var(--surface-card);
  }

  .deal-workspace-header {
    justify-content: space-between;
  }

  .workspace-eyebrow {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
  }

  .workspace-summary {
    margin: 0;
    color: var(--text-secondary);
  }

  .workspace-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--space-3);
  }

  .workspace-metric {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .workspace-metric-label {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .health-badge {
    border-radius: 9999px;
    padding: 2px var(--space-3);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .health-success { background: #E8F5EC; color: #2D8659; }
  .health-warning { background: #FFF8E1; color: #D4A017; }
  .health-danger { background: #FFF0F0; color: #C0392B; }
  .health-neutral { background: var(--surface-raised); color: var(--text-secondary); }

  .detail-grid {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(240px, 1fr);
    gap: var(--space-5);
  }

  .detail-main,
  .detail-sidebar,
  .people-stack,
  .fields-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .fields-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .field-group--full {
    grid-column: 1 / -1;
  }

  .field-label {
    display: block;
    margin-block-end: var(--space-1);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .link-button {
    border: 0;
    background: transparent;
    color: var(--text-accent);
    cursor: pointer;
    padding: 0;
    text-align: left;
  }

  .meta-line,
  .inline-error {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .inline-error {
    color: var(--text-danger);
  }

  .stage-badge {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    border-radius: 9999px;
    padding: 2px var(--space-2);
  }

  .stage-lead { background: #E8F4F7; color: #20808D; }
  .stage-qualified { background: #E8F5EC; color: #2D8659; }
  .stage-proposal { background: #FFF8E1; color: #D4A017; }
  .stage-negotiation { background: #FEF3E2; color: #A84B2F; }
  .stage-closedWon { background: #E8F5EC; color: #2D8659; }
  .stage-closedLost { background: #FFF0F0; color: #C0392B; }

  .detail-loading,
  .detail-error {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .skeleton-header { height: 64px; border-radius: var(--radius-lg); }
  .skeleton-fields { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-4); }
  .skeleton-field { height: 56px; border-radius: var(--radius-md); }

  @media (max-width: 900px) {
    .detail-grid,
    .fields-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
