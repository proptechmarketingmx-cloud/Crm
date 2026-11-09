<script lang="ts">
  import { onMount } from 'svelte';
  import type { Activity } from '$lib/api/activities';
  import type { Deal } from '$lib/api/deals';
  import { t } from '$lib/i18n';
  import type {
    ActivityRelationshipLabels,
  } from '$lib/utils/activityRelationships';
  import type { PipelineGuidance } from '$lib/utils/pipelineGuidance';
  import { weightedForecastValue } from '$lib/utils/pipelineGuidance';
  import { formatCurrency, formatDate, formatPercent, formatRelativeTime } from '$lib/utils/formatters';
  import { settingsStore } from '$lib/stores/settings';
  import { navigateHash } from '$lib/utils/hashRouter';
  import { activityStore } from '$lib/stores/activities';
  import {
    deriveRecordNextStep,
    shouldShowSecondaryFollowUp,
  } from '$lib/utils/recordNextStep';
  import ActivityFeed from './ActivityFeed.svelte';
  import NextStepCard from './NextStepCard.svelte';

  let {
    deal,
    guidance,
    activities = [],
    activitiesLoading = false,
    activityContextError = null,
    relationshipsByActivityId = {},
    primaryContactName = null,
    organizationName = null,
    onclose,
    onaddfollowup,
    onrefresh,
  }: {
    deal: Deal;
    guidance: PipelineGuidance | null;
    activities?: Activity[];
    activitiesLoading?: boolean;
    activityContextError?: string | null;
    relationshipsByActivityId?: Record<string, ActivityRelationshipLabels>;
    primaryContactName?: string | null;
    organizationName?: string | null;
    onclose?: () => void;
    onaddfollowup?: () => void;
    onrefresh?: () => void;
  } = $props();

  let drawerEl = $state<HTMLDivElement | undefined>(undefined);
  let previousFocus: Element | null = null;

  const weightedForecast = $derived(
    formatCurrency(
      guidance?.weightedForecastValue ?? weightedForecastValue(deal),
      deal.currency,
      settingsStore.language,
    )
  );

  const formattedValue = $derived(
    formatCurrency(deal.value, deal.currency, settingsStore.language)
  );

  const expectedClose = $derived(
    deal.expectedCloseDate
      ? formatDate(deal.expectedCloseDate, settingsStore.dateFormat as 'MMM D, YYYY', settingsStore.language)
      : t('common.none')
  );

  const stageAge = $derived(
    guidance === null
      ? activitiesLoading ? t('common.loading') : t('common.unknown')
      : guidance.stageAgeDays === null
      ? t('common.none')
      : t('deals.guidance.daysSinceUpdate', { count: guidance.stageAgeDays })
  );

  const guidanceTone = $derived(guidance?.tone ?? 'neutral');

  let completing = $state(false);

  const nextStep = $derived(
    deriveRecordNextStep({
      recordKind: 'deal',
      isLoading: activitiesLoading,
      unavailable: Boolean(activityContextError),
      isClosedWon: deal.stage === 'closedWon',
      isClosedLost: deal.stage === 'closedLost',
      overdueActivities:
        guidance?.state === 'overdue' && guidance.nextActivity
          ? [guidance.nextActivity]
          : [],
      nextActivity: guidance?.nextActivity ?? null,
      expectedCloseDate: deal.expectedCloseDate ?? null,
      isStale: guidance?.state === 'stale',
    }),
  );

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

  function close() {
    onclose?.();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  function addFollowUp() {
    onaddfollowup?.();
  }

  function openDealPage() {
    navigateHash(`/deals/${deal.id}`);
  }

  async function handleNextStep() {
    if (nextStep.action === 'complete' && nextStep.activityId) {
      completing = true;
      try {
        await activityStore.markComplete(nextStep.activityId);
        onrefresh?.();
      } finally {
        completing = false;
      }
      return;
    }

    if (nextStep.action === 'addFollowUp') {
      addFollowUp();
      return;
    }

    if (nextStep.action === 'setExpectedClose') {
      openDealPage();
    }
  }

  function focusFirstElement() {
    const first = drawerEl?.querySelector<HTMLElement>(
      'button:not([disabled]), a[href], input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [tabindex="0"]'
    );
    (first ?? drawerEl)?.focus();
  }

  function handleGlobalKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      close();
      return;
    }

    if (event.key !== 'Tab' || !drawerEl) {
      return;
    }

    const focusable = drawerEl.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
    );

    if (focusable.length === 0) {
      event.preventDefault();
      drawerEl.focus();
      return;
    }

    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  onMount(() => {
    previousFocus = document.activeElement;
    requestAnimationFrame(focusFirstElement);
    document.addEventListener('keydown', handleGlobalKeyDown);

    return () => {
      document.removeEventListener('keydown', handleGlobalKeyDown);
      if (previousFocus instanceof HTMLElement && document.contains(previousFocus)) {
        previousFocus.focus();
      }
      previousFocus = null;
    };
  });
</script>

<div
  class="drawer-backdrop"
  role="presentation"
  onclick={handleBackdropClick}
>
  <div
    class="deal-drawer"
    role="dialog"
    aria-modal="true"
    aria-labelledby="deal-drawer-title"
    tabindex="-1"
    bind:this={drawerEl}
  >
    <header class="drawer-header">
      <div>
        <p class="drawer-eyebrow">{t('deals.guidance.drawerEyebrow')}</p>
        <h2 id="deal-drawer-title" class="drawer-title">{deal.name}</h2>
      </div>
      <div class="drawer-header-actions">
        <button class="btn btn-secondary btn-sm" type="button" onclick={openDealPage}>
          {t('deals.openDeal')}
        </button>
      <button class="btn btn-ghost btn-sm" type="button" onclick={close} aria-label={t('common.close')}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M18 6 6 18M6 6l12 12"/>
        </svg>
      </button>
      </div>
    </header>

    <section class="guidance-card guidance-{guidanceTone}" aria-labelledby="deal-guidance-heading">
      <div class="guidance-card-header">
        <h3 id="deal-guidance-heading">{guidanceLabel()}</h3>
        <span>{t(`deals.stages.${deal.stage}`)}</span>
      </div>
      <p>{guidanceDetail()}</p>
      <NextStepCard step={nextStep} busy={completing} onaction={handleNextStep} />
      {#if shouldShowSecondaryFollowUp(nextStep)}
        <button class="btn btn-secondary btn-sm" type="button" onclick={addFollowUp}>
          {t('deals.guidance.addFollowUp')}
        </button>
      {/if}
    </section>

    <dl class="deal-facts">
      <div>
        <dt>{t('deals.value')}</dt>
        <dd>{formattedValue}</dd>
      </div>
      <div>
        <dt>{t('deals.guidance.weightedForecast')}</dt>
        <dd>{weightedForecast}</dd>
      </div>
      <div>
        <dt>{t('deals.probability')}</dt>
        <dd>{formatPercent(deal.probability)}</dd>
      </div>
      <div>
        <dt>{t('deals.expectedClose')}</dt>
        <dd>{expectedClose}</dd>
      </div>
      <div>
        <dt>{t('deals.guidance.stageAge')}</dt>
        <dd>{stageAge}</dd>
      </div>
      <div>
        <dt>{t('common.updated')}</dt>
        <dd>{formatRelativeTime(deal.updatedAt)}</dd>
      </div>
      <div>
        <dt>{t('deals.contact')}</dt>
        <dd>{primaryContactName ?? t('common.none')}</dd>
      </div>
      <div>
        <dt>{t('contacts.organization')}</dt>
        <dd>{organizationName ?? t('common.none')}</dd>
      </div>
      <div>
        <dt>{t('common.created')}</dt>
        <dd>{formatDate(deal.createdAt, settingsStore.dateFormat as 'MMM D, YYYY', settingsStore.language)}</dd>
      </div>
    </dl>

    <section class="drawer-section" aria-labelledby="deal-description-heading">
      <h3 id="deal-description-heading">{t('deals.description')}</h3>
      <p>{deal.description ?? t('deals.guidance.noDescription')}</p>
    </section>

    <section class="drawer-section" aria-labelledby="deal-activity-heading">
      <div class="drawer-section-header">
        <h3 id="deal-activity-heading">{t('deals.guidance.linkedActivities')}</h3>
        <button class="btn btn-secondary btn-xs" type="button" onclick={addFollowUp}>
          {t('activities.addActivity')}
        </button>
      </div>
      {#if activityContextError}
        <p class="drawer-inline-error">{activityContextError}</p>
      {:else}
        <ActivityFeed
          activities={activities}
          loading={activitiesLoading}
          maxItems={8}
          relationshipsByActivityId={relationshipsByActivityId}
          showRelationshipBreadcrumbs={true}
        />
      {/if}
    </section>
  </div>
</div>

<style>
  .drawer-backdrop {
    position: fixed;
    inset: 0;
    z-index: 30;
    display: flex;
    justify-content: flex-end;
    background-color: rgb(15 23 42 / 0.36);
  }

  .deal-drawer {
    width: min(520px, 100vw);
    height: 100%;
    overflow-y: auto;
    padding: var(--space-6);
    background-color: var(--surface-card);
    box-shadow: var(--shadow-lg);
  }

  .drawer-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    margin-block-end: var(--space-5);
  }

  .drawer-header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .drawer-eyebrow {
    margin: 0 0 var(--space-1);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
  }

  .drawer-title {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-xl);
    line-height: var(--leading-tight);
  }

  .guidance-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-block-end: var(--space-5);
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    background-color: var(--surface-raised);
  }

  .guidance-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .guidance-card h3,
  .drawer-section h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-base);
  }

  .guidance-card p,
  .drawer-section p {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .drawer-inline-error {
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background-color: var(--color-danger-50);
    color: var(--text-danger);
  }

  .guidance-success {
    border-color: #BFE4CC;
    background-color: #E8F5EC;
  }

  .guidance-warning {
    border-color: #F4D1A1;
    background-color: #FEF3E2;
  }

  .guidance-danger {
    border-color: #F0B8B2;
    background-color: #FFF0F0;
  }

  .deal-facts {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
    margin: 0 0 var(--space-5);
  }

  .deal-facts div {
    min-width: 0;
    padding-block-start: var(--space-2);
    border-block-start: var(--border-width) solid var(--border-default);
  }

  .deal-facts dt {
    margin-block-end: var(--space-1);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
  }

  .deal-facts dd {
    margin: 0;
    overflow: hidden;
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .drawer-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-block-start: var(--space-5);
  }

  .drawer-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  @media (max-width: 640px) {
    .deal-drawer {
      width: 100vw;
      padding: var(--space-5);
    }

    .deal-facts {
      grid-template-columns: 1fr;
    }
  }
</style>
