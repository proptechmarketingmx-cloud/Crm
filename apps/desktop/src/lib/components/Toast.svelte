<script lang="ts">
  /**
   * Toast.svelte — Toast notification system for 900CRM.
   *
   * Renders the active toasts from uiStore in a fixed stack at the
   * bottom-end corner. Supports success/error/warning/info variants.
   *
   * Mount once in the root layout.
   */

  import { t } from '$lib/i18n';
  import { uiStore } from '$lib/stores/ui';

  // ── Helpers ────────────────────────────────────────────────────────────────

  function iconForType(type: string): string {
    switch (type) {
      case 'success': return '✓';
      case 'error':   return '✕';
      case 'warning': return '⚠';
      default:        return 'ℹ';
    }
  }
</script>

<div class="toast-container" aria-live="polite" aria-atomic="false">
  {#each uiStore.toasts as toast (toast.id)}
    <div class="toast {toast.type}" role="status">
      <span class="toast-icon" aria-hidden="true">{iconForType(toast.type)}</span>
      <span class="toast-message">{toast.message}</span>
      <button
        class="toast-dismiss icon-btn"
        onclick={() => uiStore.dismissToast(toast.id)}
        aria-label={t('common.close')}
        type="button"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M9 3L3 9M3 3l6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-icon {
    font-size: 14px;
    line-height: 1;
    flex-shrink: 0;
  }

  .toast-message {
    flex: 1;
    min-width: 0;
    line-height: var(--leading-snug);
  }

  .toast-dismiss {
    color: var(--text-tooltip);
    opacity: 0.7;
    flex-shrink: 0;
    width: 24px;
    height: 24px;
  }

  .toast-dismiss:hover {
    opacity: 1;
    background-color: rgba(255, 255, 255, 0.15);
  }
</style>
