<script lang="ts">
  /**
   * AuditLog.svelte - Read-only audit log view.
   */

  import { onMount } from 'svelte';
  import { t } from '$lib/i18n';
  import { listRecentAuditLog, type AuditLogEntry } from '$lib/api/audit';
  import { settingsStore } from '$lib/stores/settings';

  const limitOptions = [25, 50, 100, 250, 500];

  let entries = $state<AuditLogEntry[]>([]);
  let limit = $state(100);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(() => {
    void loadEntries();
  });

  async function loadEntries() {
    isLoading = true;
    error = null;

    try {
      entries = await listRecentAuditLog(limit);
    } catch (err) {
      console.error('[AuditLog] Failed to load audit log:', err);
      error = t('auditLog.loadFailed');
    } finally {
      isLoading = false;
    }
  }

  function handleLimitChange(event: Event) {
    limit = Number((event.target as HTMLSelectElement).value);
    void loadEntries();
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

  function actorLabel(type: string, id: string | null): string {
    return id ? `${type} / ${compactId(id)}` : type;
  }

  function jsonSummary(value: string | null): string {
    if (!value) return t('auditLog.noJson');

    try {
      const parsed = JSON.parse(value) as unknown;
      if (Array.isArray(parsed)) {
        return t('auditLog.jsonArrayCount', { count: String(parsed.length) });
      }
      if (parsed && typeof parsed === 'object') {
        const keys = Object.keys(parsed);
        if (keys.length === 0) return t('auditLog.jsonEmptyObject');
        const preview = keys.slice(0, 3).join(', ');
        return keys.length > 3
          ? t('auditLog.jsonObjectPreviewMore', { keys: preview, count: String(keys.length - 3) })
          : t('auditLog.jsonObjectPreview', { keys: preview });
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

<div class="page-content audit-log-page">
  <div class="page-header audit-header">
    <div>
      <h1 class="page-title">{t('auditLog.title')}</h1>
      <p class="page-subtitle">{t('auditLog.subtitle')}</p>
    </div>

    <div class="audit-actions">
      <label class="limit-control">
        <span>{t('auditLog.limit')}</span>
        <select class="select limit-select" bind:value={limit} onchange={handleLimitChange} disabled={isLoading}>
          {#each limitOptions as option (option)}
            <option value={option}>{option}</option>
          {/each}
        </select>
      </label>

      <button class="btn btn-secondary btn-sm" type="button" onclick={loadEntries} disabled={isLoading}>
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
          <path d="M3 21v-5h5" />
          <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
          <path d="M16 8h5V3" />
        </svg>
        {t('common.refresh')}
      </button>
    </div>
  </div>

  <div class="audit-table card">
    <div class="table-scroll">
      <table class="audit-list" aria-label={t('auditLog.title')}>
        <thead>
          <tr>
            <th>{t('auditLog.timestamp')}</th>
            <th>{t('auditLog.action')}</th>
            <th>{t('auditLog.actor')}</th>
            <th>{t('auditLog.entity')}</th>
            <th>{t('auditLog.deviceId')}</th>
            <th>{t('auditLog.before')}</th>
            <th>{t('auditLog.after')}</th>
          </tr>
        </thead>
        <tbody>
          {#if isLoading}
            {#each Array(6) as _, index (index)}
              <tr class="skeleton-row">
                <td colspan="7"><div class="skeleton table-skeleton-cell"></div></td>
              </tr>
            {/each}
          {:else if error}
            <tr>
              <td colspan="7" class="state-cell">
                <div class="state-panel" role="alert">
                  <h2>{error}</h2>
                  <p>{t('auditLog.loadFailedDesc')}</p>
                  <button class="btn btn-secondary btn-sm" type="button" onclick={loadEntries}>
                    {t('common.retry')}
                  </button>
                </div>
              </td>
            </tr>
          {:else if entries.length === 0}
            <tr>
              <td colspan="7" class="state-cell">
                <div class="state-panel">
                  <h2>{t('auditLog.emptyTitle')}</h2>
                  <p>{t('auditLog.emptyDesc')}</p>
                </div>
              </td>
            </tr>
          {:else}
            {#each entries as entry (entry.id)}
              <tr>
                <td>
                  <time datetime={entry.createdAt}>{formatTimestamp(entry.createdAt)}</time>
                </td>
                <td><span class="mono strong">{entry.action}</span></td>
                <td>{actorLabel(entry.actorType, entry.actorId)}</td>
                <td>{entityLabel(entry.entityType, entry.entityId)}</td>
                <td><span class="mono">{compactId(entry.deviceId)}</span></td>
                <td>
                  {#if entry.beforeJson}
                    <details class="json-detail">
                      <summary>{jsonSummary(entry.beforeJson)}</summary>
                      <pre class="json-block selectable">{jsonBlock(entry.beforeJson)}</pre>
                    </details>
                  {:else}
                    <span class="muted">{t('auditLog.noJson')}</span>
                  {/if}
                </td>
                <td>
                  {#if entry.afterJson}
                    <details class="json-detail">
                      <summary>{jsonSummary(entry.afterJson)}</summary>
                      <pre class="json-block selectable">{jsonBlock(entry.afterJson)}</pre>
                    </details>
                  {:else}
                    <span class="muted">{t('auditLog.noJson')}</span>
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
  .audit-log-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
  }

  .audit-header {
    align-items: flex-start;
    margin-block-end: 0;
  }

  .page-subtitle {
    margin-block-start: var(--space-2);
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .audit-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .limit-control {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
  }

  .limit-select {
    width: 84px;
    height: 28px;
    padding-block: var(--space-1);
    font-size: var(--text-xs);
  }

  .audit-table {
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

  .audit-list {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .audit-list th,
  .audit-list td {
    padding: var(--space-3) var(--space-4);
    border-block-end: var(--border-width) solid var(--border-subtle);
    text-align: start;
    vertical-align: top;
  }

  .audit-list th {
    position: sticky;
    top: 0;
    z-index: 1;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    background-color: var(--surface-card);
  }

  .audit-list td {
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
    max-width: 360px;
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
    .audit-header {
      flex-direction: column;
    }

    .audit-actions {
      width: 100%;
      justify-content: flex-start;
    }
  }
</style>
