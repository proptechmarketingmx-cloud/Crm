<script lang="ts">
  /**
   * One-tap log for a visit, WhatsApp ping, or SMS that just happened.
   */

  import { t } from '$lib/i18n';
  import { activityStore } from '$lib/stores/activities';
  import type { ActivityType } from '$lib/api/activities';

  let {
    contactId = null,
    dealId = null,
    onlogged,
  }: {
    contactId?: string | null;
    dealId?: string | null;
    onlogged?: () => void;
  } = $props();

  let saving = $state<ActivityType | null>(null);

  const actions: Array<{ type: ActivityType; label: () => string }> = [
    { type: 'visit', label: () => t('activities.logVisit') },
    { type: 'whatsapp', label: () => t('activities.logWhatsapp') },
    { type: 'sms', label: () => t('activities.logSms') },
  ];

  async function log(type: ActivityType) {
    saving = type;
    try {
      await activityStore.logJustHappened({
        type,
        contactId,
        dealId,
      });
      onlogged?.();
    } finally {
      saving = null;
    }
  }
</script>

<div class="log-now" role="group" aria-label={t('activities.logJustHappened')} data-testid="log-just-happened">
  <span class="log-now-label">{t('activities.logJustHappened')}</span>
  <div class="log-now-actions">
    {#each actions as action (action.type)}
      <button
        class="btn btn-secondary btn-xs"
        type="button"
        disabled={saving !== null}
        onclick={() => void log(action.type)}
      >
        {saving === action.type ? t('common.loading') : action.label()}
      </button>
    {/each}
  </div>
</div>

<style>
  .log-now {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .log-now-label {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .log-now-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
