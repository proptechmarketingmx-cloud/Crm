<script lang="ts">
  /**
   * PendingActions.svelte - Proposed-action decision surface for pending items.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import {
    approveProposedAction,
    listPendingProposedActions,
    rejectProposedAction,
    type ProposedAction,
  } from '$lib/api/proposedActions';
  import { settingsStore } from '$lib/stores/settings';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { reviewCountsStore } from '$lib/stores/reviewCounts';

  let actions = $state<ProposedAction[]>([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let loadRequestSeq = 0;
  let decisionSeq = 0;
  let decisionBusyById = $state<Record<string, 'approve' | 'reject'>>({});
  let decisionErrorsById = $state<Record<string, string>>({});
  let pageMessage = $state<{ type: 'success' | 'error'; text: string } | null>(null);
  const decidedActionIds = new Set<string>();

  onMount(() => {
    void loadActions();
  });

  async function loadActions() {
    const requestSeq = ++loadRequestSeq;
    const decisionSeqAtStart = decisionSeq;
    isLoading = true;
    error = null;

    try {
      const pendingActions = await listPendingProposedActions();
      if (requestSeq === loadRequestSeq && decisionSeqAtStart === decisionSeq) {
        actions = pendingActions.filter((action) => !decidedActionIds.has(action.id));
        reviewCountsStore.pendingCount = actions.length;
      }
    } catch (err) {
      if (requestSeq === loadRequestSeq && decisionSeqAtStart === decisionSeq) {
        console.error('[PendingActions] Failed to load pending actions:', err);
        error = t('pendingActions.loadFailed');
      }
    } finally {
      if (requestSeq === loadRequestSeq) {
        isLoading = false;
      }
    }
  }

  function errorMessage(err: unknown): string {
    if (err instanceof Error && err.message.trim()) {
      return err.message;
    }
    if (typeof err === 'string' && err.trim()) {
      return err;
    }
    return t('pendingActions.decisionFailed');
  }

  function setDecisionBusy(id: string, value: 'approve' | 'reject' | null) {
    if (value) {
      decisionBusyById = { ...decisionBusyById, [id]: value };
      return;
    }

    const remaining = { ...decisionBusyById };
    delete remaining[id];
    decisionBusyById = remaining;
  }

  function clearDecisionError(id: string) {
    const remaining = { ...decisionErrorsById };
    delete remaining[id];
    decisionErrorsById = remaining;
  }

  async function decideAction(action: ProposedAction, decision: 'approve' | 'reject') {
    if (decisionBusyById[action.id]) return;

    setDecisionBusy(action.id, decision);
    clearDecisionError(action.id);
    pageMessage = null;

    try {
      if (decision === 'approve') {
        await approveProposedAction(action.id);
      } else {
        await rejectProposedAction(action.id);
      }

      decisionSeq += 1;
      decidedActionIds.add(action.id);
      actions = actions.filter((pendingAction) => pendingAction.id !== action.id);
      reviewCountsStore.pendingCount = actions.length;
      error = null;

      const message = decision === 'approve'
        ? t('pendingActions.approveSuccess', { action: action.actionType })
        : t('pendingActions.rejectSuccess', { action: action.actionType });
      pageMessage = { type: 'success', text: message };
      uiStore.toastSuccess(message);
      void loadActions();
    } catch (err) {
      const message = errorMessage(err);
      decisionErrorsById = { ...decisionErrorsById, [action.id]: message };
      pageMessage = {
        type: 'error',
        text: decision === 'approve'
          ? t('pendingActions.approveFailed')
          : t('pendingActions.rejectFailed'),
      };
      uiStore.toastError(`${pageMessage.text}: ${message}`);
    } finally {
      setDecisionBusy(action.id, null);
    }
  }

  function formatTimestamp(value: string): string {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value || '-';

    try {
      return new Intl.DateTimeFormat(settingsStore.language, {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(date);
    } catch {
      return date.toLocaleString();
    }
  }

  function compactId(value: string | null): string {
    if (!value) return '-';
    return value.length > 18 ? `${value.slice(0, 8)}...${value.slice(-6)}` : value;
  }

  function entityLabel(type: string | null, id: string | null): string {
    if (!type && !id) return '-';
    if (!type) return compactId(id);
    if (!id) return type;
    return `${type} / ${compactId(id)}`;
  }

  function jsonSummary(value: string | null): string {
    if (!value) return t('pendingActions.noJson');

    try {
      const parsed = JSON.parse(value) as unknown;
      if (Array.isArray(parsed)) {
        return t('pendingActions.jsonArrayCount', { count: String(parsed.length) });
      }
      if (parsed && typeof parsed === 'object') {
        const keys = Object.keys(parsed);
        if (keys.length === 0) return t('pendingActions.jsonEmptyObject');
        const preview = keys.slice(0, 3).join(', ');
        return keys.length > 3
          ? t('pendingActions.jsonObjectPreviewMore', { keys: preview, count: String(keys.length - 3) })
          : t('pendingActions.jsonObjectPreview', { keys: preview });
      }
      return String(parsed);
    } catch {
      const trimmed = value.trim();
      return trimmed.length > 48 ? `${trimmed.slice(0, 48)}...` : trimmed;
    }
  }

  function jsonBlock(value: string | null): string {
    if (!value) return '';

    try {
      return JSON.stringify(JSON.parse(value), null, 2);
    } catch {
      return value;
    }
  }
</script>

<div class="page-content pending-actions-page">
  <div class="page-header pending-header">
    <div>
      <h1 class="page-title">{t('pendingActions.title')}</h1>
      <p class="page-subtitle">{t('pendingActions.subtitle')}</p>
    </div>

    <button class="btn btn-secondary btn-sm" type="button" onclick={loadActions} disabled={isLoading}>
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
        <path d="M3 21v-5h5" />
        <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
        <path d="M16 8h5V3" />
      </svg>
      {t('common.refresh')}
    </button>
  </div>

  {#if pageMessage}
    <div class="decision-message {pageMessage.type}" role="status">
      {pageMessage.text}
    </div>
  {/if}

  <div class="pending-table card">
    <div class="table-scroll">
      <table class="pending-list" aria-label={t('pendingActions.title')}>
        <thead>
          <tr>
            <th>{t('pendingActions.actionType')}</th>
            <th>{t('pendingActions.toolName')}</th>
            <th>{t('pendingActions.entity')}</th>
            <th>{t('pendingActions.clientId')}</th>
            <th>{t('pendingActions.createdAt')}</th>
            <th>{t('pendingActions.input')}</th>
            <th>{t('pendingActions.proposedOutput')}</th>
            <th>{t('pendingActions.status')}</th>
            <th>{t('pendingActions.deviceId')}</th>
            <th>{t('pendingActions.decision')}</th>
          </tr>
        </thead>
        <tbody>
          {#if isLoading}
            {#each Array(6) as _, index (index)}
              <tr class="skeleton-row">
                <td colspan="10"><div class="skeleton table-skeleton-cell"></div></td>
              </tr>
            {/each}
          {:else if error}
            <tr>
              <td colspan="10" class="state-cell">
                <div class="state-panel" role="alert">
                  <h2>{error}</h2>
                  <p>{t('pendingActions.loadFailedDesc')}</p>
                  <button class="btn btn-secondary btn-sm" type="button" onclick={loadActions}>
                    {t('common.retry')}
                  </button>
                </div>
              </td>
            </tr>
          {:else if actions.length === 0}
            <tr>
              <td colspan="10" class="state-cell">
                <div class="state-panel">
                  <h2>{t('pendingActions.emptyTitle')}</h2>
                  <p>{t('pendingActions.emptyDesc')}</p>
                </div>
              </td>
            </tr>
          {:else}
            {#each actions as action (action.id)}
              <tr>
                <td><span class="mono strong">{action.actionType}</span></td>
                <td>{action.toolName}</td>
                <td>{entityLabel(action.entityType, action.entityId)}</td>
                <td><span class="mono">{compactId(action.clientId)}</span></td>
                <td>
                  <time datetime={action.createdAt}>{formatTimestamp(action.createdAt)}</time>
                </td>
                <td>
                  <details class="json-detail">
                    <summary>{jsonSummary(action.inputJson)}</summary>
                    <pre class="json-block selectable">{jsonBlock(action.inputJson)}</pre>
                  </details>
                </td>
                <td>
                  {#if action.proposedOutputJson}
                    <details class="json-detail">
                      <summary>{jsonSummary(action.proposedOutputJson)}</summary>
                      <pre class="json-block selectable">{jsonBlock(action.proposedOutputJson)}</pre>
                    </details>
                  {:else}
                    <span class="muted">{t('pendingActions.noJson')}</span>
                  {/if}
                </td>
                <td><span class="status-badge">{action.status}</span></td>
                <td><span class="mono">{compactId(action.deviceId)}</span></td>
                <td>
                  <div class="decision-controls" aria-busy={Boolean(decisionBusyById[action.id])}>
                    <button
                      class="btn btn-primary btn-sm decision-button"
                      type="button"
                      onclick={() => void decideAction(action, 'approve')}
                      disabled={Boolean(decisionBusyById[action.id])}
                    >
                      {decisionBusyById[action.id] === 'approve' ? t('pendingActions.approving') : t('pendingActions.approve')}
                    </button>
                    <button
                      class="btn btn-danger btn-sm decision-button"
                      type="button"
                      onclick={() => void decideAction(action, 'reject')}
                      disabled={Boolean(decisionBusyById[action.id])}
                    >
                      {decisionBusyById[action.id] === 'reject' ? t('pendingActions.rejecting') : t('pendingActions.reject')}
                    </button>
                  </div>
                  {#if decisionErrorsById[action.id]}
                    <p class="decision-error" role="alert">
                      {decisionErrorsById[action.id]}
                    </p>
                  {/if}
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .pending-actions-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
  }

  .pending-header {
    align-items: flex-start;
    margin-block-end: 0;
  }

  .page-subtitle {
    margin-block-start: var(--space-2);
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .pending-table {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .table-scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .pending-list {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .pending-list th,
  .pending-list td {
    padding: var(--space-3) var(--space-4);
    border-block-end: var(--border-width) solid var(--border-subtle);
    text-align: start;
    vertical-align: top;
  }

  .pending-list th {
    position: sticky;
    top: 0;
    z-index: 1;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    background-color: var(--surface-card);
  }

  .pending-list td {
    color: var(--text-secondary);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .strong {
    color: var(--text-primary);
    font-weight: var(--weight-semibold);
  }

  .muted {
    color: var(--text-tertiary);
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    min-height: 20px;
    padding: var(--space-1) var(--space-3);
    border-radius: 9999px;
    background-color: var(--color-warning-50);
    color: var(--color-warning-600);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .decision-message {
    padding: var(--space-3) var(--space-4);
    border: var(--border-width) solid var(--border-subtle);
    border-radius: var(--border-radius-md);
    font-size: var(--text-sm);
  }

  .decision-message.success {
    border-color: var(--color-success-500);
    background-color: var(--color-success-50);
    color: var(--text-success);
  }

  .decision-message.error {
    border-color: var(--color-danger-500);
    background-color: var(--color-danger-50);
    color: var(--text-danger);
  }

  .decision-controls {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    min-width: 176px;
  }

  .decision-button {
    min-width: 82px;
  }

  .decision-error {
    margin: var(--space-2) 0 0;
    max-width: 240px;
    color: var(--color-danger-600);
    font-size: var(--text-xs);
    line-height: var(--leading-relaxed);
  }

  .state-cell {
    padding: var(--space-10) var(--space-4);
  }

  .state-panel {
    min-height: 240px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--text-secondary);
    text-align: center;
  }

  .state-panel h2 {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-lg);
  }

  .state-panel p {
    margin: 0;
    max-width: 380px;
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .json-detail {
    max-width: 260px;
  }

  .json-detail summary {
    cursor: pointer;
    color: var(--text-accent);
    font-size: var(--text-xs);
    line-height: var(--leading-relaxed);
  }

  .json-block {
    margin-block-start: var(--space-2);
    max-width: 360px;
    max-height: 180px;
    overflow: auto;
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--surface-panel);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: var(--leading-relaxed);
    white-space: pre-wrap;
  }

  @media (max-width: 820px) {
    .pending-header {
      flex-direction: column;
    }
  }
</style>
