<script lang="ts">
  import { t } from '$lib/i18n';
  import type { RecordNextStep } from '$lib/utils/recordNextStep';

  let {
    step,
    busy = false,
    onaction,
  }: {
    step: RecordNextStep;
    busy?: boolean;
    onaction?: () => void;
  } = $props();

  const fallbackSubject = $derived(step.subject ?? t('activities.title'));

  const title = $derived.by(() => {
    switch (step.kind) {
      case 'loading':
        return t('nextStep.loadingTitle');
      case 'unavailable':
        return t('nextStep.unavailableTitle');
      case 'completeOverdue':
        return t('nextStep.completeTitle', { subject: fallbackSubject });
      case 'convertLead':
        return t('nextStep.convertTitle');
      case 'addFollowUp':
        return t('nextStep.addFollowUpTitle');
      case 'setExpectedClose':
        return t('nextStep.setCloseTitle');
      case 'stale':
        return t('nextStep.staleTitle');
      case 'onTrack':
        return t('nextStep.onTrackTitle', { subject: fallbackSubject });
      case 'nurture':
        return t('nextStep.nurtureTitle');
      case 'closedWon':
        return t('nextStep.closedWonTitle');
      case 'closedLost':
        return t('nextStep.closedLostTitle');
    }
  });

  const detail = $derived.by(() => {
    switch (step.kind) {
      case 'loading':
        return t('nextStep.loadingDetail');
      case 'unavailable':
        return t('nextStep.unavailableDetail');
      case 'completeOverdue':
        return t('nextStep.completeDetail');
      case 'convertLead':
        return t('nextStep.convertDetail');
      case 'addFollowUp':
        return t('nextStep.addFollowUpDetail');
      case 'setExpectedClose':
        return t('nextStep.setCloseDetail');
      case 'stale':
        return t('nextStep.staleDetail', { subject: fallbackSubject });
      case 'onTrack':
        return t('nextStep.onTrackDetail');
      case 'nurture':
        return t('nextStep.nurtureDetail');
      case 'closedWon':
        return t('nextStep.closedWonDetail');
      case 'closedLost':
        return t('nextStep.closedLostDetail');
    }
  });

  const actionLabel = $derived.by(() => {
    switch (step.action) {
      case 'complete':
        return t('nextStep.completeAction');
      case 'convert':
        return t('nextStep.convertAction');
      case 'addFollowUp':
        return t('nextStep.addFollowUpAction');
      case 'setExpectedClose':
        return t('nextStep.setCloseAction');
      case 'none':
        return '';
    }
  });
</script>

<section
  class="next-step next-step-{step.tone}"
  data-testid="next-step"
  aria-labelledby="next-step-heading"
>
  <div class="next-step-copy">
    <p class="next-step-eyebrow">{t('nextStep.eyebrow')}</p>
    <h3 class="next-step-title" id="next-step-heading">{title}</h3>
    <p class="next-step-detail">{detail}</p>
  </div>
  {#if actionLabel && onaction}
    <button
      class="btn btn-primary btn-sm next-step-action"
      type="button"
      onclick={onaction}
      disabled={busy}
    >
      {busy ? t('common.loading') : actionLabel}
    </button>
  {/if}
</section>

<style>
  .next-step {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-4);
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    background-color: var(--surface-raised);
  }

  .next-step-copy {
    min-width: 0;
  }

  .next-step-eyebrow {
    margin: 0 0 var(--space-1);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
    text-transform: uppercase;
  }

  .next-step-title {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    line-height: 1.35;
  }

  .next-step-detail {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    line-height: 1.45;
  }

  .next-step-action {
    flex-shrink: 0;
  }

  .next-step-danger {
    border-color: #f0b8b2;
    background-color: #fff0f0;
  }

  .next-step-warning {
    border-color: #f4d1a1;
    background-color: #fef3e2;
  }

  .next-step-success {
    border-color: #bfe4cc;
    background-color: #e8f5ec;
  }

  @media (max-width: 640px) {
    .next-step {
      flex-direction: column;
    }

    .next-step-action {
      width: 100%;
      justify-content: center;
    }
  }
</style>
