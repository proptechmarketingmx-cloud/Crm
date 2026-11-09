<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listExternalClientPermissions,
    upsertExternalClientToolPermission,
    type ExternalClient,
    type ExternalClientPermission,
  } from '$lib/api/externalClients';
  import { t } from '$lib/i18n';

  let { client } = $props<{ client: ExternalClient }>();

  let permissions = $state<ExternalClientPermission[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);
  let toolName = $state('');
  let canRead = $state(true);
  let canWrite = $state(false);
  let requiresConfirmation = $state(true);
  let requestSeq = 0;

  onMount(() => {
    void loadPermissions();
  });

  function permissionErrorMessage(err: unknown, fallback: string): string {
    if (err instanceof Error && err.message.trim()) {
      return err.message;
    }
    if (typeof err === 'string' && err.trim()) {
      return err;
    }
    return fallback;
  }

  async function loadPermissions() {
    const currentSeq = ++requestSeq;
    loading = true;
    error = null;
    success = null;
    try {
      const rows = await listExternalClientPermissions(client.id);
      if (currentSeq === requestSeq) {
        permissions = rows;
      }
    } catch (err) {
      if (currentSeq === requestSeq) {
        error = permissionErrorMessage(err, t('settings.externalClientPermissionsLoadFailed'));
      }
    } finally {
      if (currentSeq === requestSeq) {
        loading = false;
      }
    }
  }

  function editPermission(permission: ExternalClientPermission) {
    toolName = permission.toolName;
    canRead = permission.canRead;
    canWrite = permission.canWrite;
    requiresConfirmation = permission.requiresConfirmation || permission.canWrite;
    error = null;
    success = null;
  }

  function handleToolNameInput(e: Event) {
    toolName = (e.target as HTMLInputElement).value;
    error = null;
    success = null;
  }

  function handleCanReadChange(e: Event) {
    canRead = (e.target as HTMLInputElement).checked;
    success = null;
  }

  function handleCanWriteChange(e: Event) {
    canWrite = (e.target as HTMLInputElement).checked;
    if (canWrite) {
      requiresConfirmation = true;
    }
    success = null;
  }

  function handleRequiresConfirmationChange(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    requiresConfirmation = canWrite ? true : checked;
    success = null;
  }

  async function savePermission() {
    const normalizedToolName = toolName.trim();
    if (!normalizedToolName || saving) return;

    saving = true;
    error = null;
    success = null;
    try {
      const saved = await upsertExternalClientToolPermission({
        clientId: client.id,
        toolName: normalizedToolName,
        canRead,
        canWrite,
        requiresConfirmation,
      });
      permissions = [
        saved,
        ...permissions.filter(
          (permission) => permission.id !== saved.id && permission.toolName !== saved.toolName,
        ),
      ].sort((a, b) => a.toolName.localeCompare(b.toolName));
      toolName = saved.toolName;
      canRead = saved.canRead;
      canWrite = saved.canWrite;
      requiresConfirmation = saved.requiresConfirmation || saved.canWrite;
      success = t('settings.externalClientPermissionsSaveSuccess', { toolName: saved.toolName });
    } catch (err) {
      error = permissionErrorMessage(err, t('settings.externalClientPermissionsSaveFailed'));
    } finally {
      saving = false;
    }
  }
</script>

<div class="external-client-permissions" aria-live="polite">
  <div class="permissions-header">
    <div class="permissions-title">
      <span>{t('settings.externalClientPermissions')}</span>
      <span>{t('settings.externalClientPermissionsDesc')}</span>
    </div>
    <button
      class="btn btn-secondary btn-sm"
      type="button"
      onclick={loadPermissions}
      disabled={loading || saving}
    >
      {loading ? t('common.loading') : t('settings.externalClientPermissionsRefresh')}
    </button>
  </div>

  {#if loading}
    <p class="permissions-empty">{t('settings.externalClientPermissionsLoading')}</p>
  {:else if error}
    <p class="permissions-status permissions-status--error" role="alert">{error}</p>
  {:else if permissions.length === 0}
    <p class="permissions-empty">{t('settings.externalClientPermissionsEmpty')}</p>
  {:else}
    <div class="permissions-list" aria-label={t('settings.externalClientPermissionsRows')}>
      {#each permissions as permission (permission.id)}
        <div class="permission-row">
          <div class="permission-row-main">
            <strong>{permission.toolName}</strong>
            <span>
              {permission.canRead ? t('settings.externalClientPermissionCanRead') : t('settings.externalClientPermissionNoRead')}
              ·
              {permission.canWrite ? t('settings.externalClientPermissionCanWrite') : t('settings.externalClientPermissionNoWrite')}
              ·
              {permission.requiresConfirmation ? t('settings.externalClientPermissionRequiresConfirmation') : t('settings.externalClientPermissionNoConfirmation')}
            </span>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" onclick={() => editPermission(permission)}>
            {t('common.edit')}
          </button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="permission-editor">
    <div class="field-row">
      <label class="field-label" for={`external-client-tool-${client.id}`}>{t('settings.externalClientToolName')}</label>
      <input
        id={`external-client-tool-${client.id}`}
        class="input"
        type="text"
        value={toolName}
        oninput={handleToolNameInput}
        placeholder={t('settings.externalClientToolNamePlaceholder')}
        disabled={saving}
        spellcheck={false}
      />
    </div>

    <div class="permission-checkboxes">
      <label class="permission-checkbox">
        <input type="checkbox" checked={canRead} onchange={handleCanReadChange} disabled={saving} />
        <span>{t('settings.externalClientPermissionCanRead')}</span>
      </label>
      <label class="permission-checkbox">
        <input type="checkbox" checked={canWrite} onchange={handleCanWriteChange} disabled={saving} />
        <span>{t('settings.externalClientPermissionCanWrite')}</span>
      </label>
      <label class="permission-checkbox">
        <input
          type="checkbox"
          checked={requiresConfirmation}
          onchange={handleRequiresConfirmationChange}
          disabled={saving || canWrite}
        />
        <span>{t('settings.externalClientPermissionRequiresConfirmation')}</span>
      </label>
    </div>

    <p class="permissions-note">{t('settings.externalClientPermissionWriteRequiresConfirmation')}</p>

    <button
      class="btn btn-primary btn-sm"
      type="button"
      onclick={savePermission}
      disabled={!toolName.trim() || saving || loading}
    >
      {saving ? t('common.loading') : t('settings.externalClientPermissionsSave')}
    </button>
  </div>

  {#if success}
    <p class="permissions-status permissions-status--success">{success}</p>
  {/if}
</div>

<style>
  .external-client-permissions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding-top: var(--space-3);
    border-top: var(--border-width) solid var(--border-default);
  }

  .permissions-header,
  .permission-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .permissions-title,
  .permission-row-main,
  .permission-editor {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
  }

  .permissions-title span:first-child {
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
  }

  .permissions-title span:last-child,
  .permission-row-main span,
  .permissions-empty,
  .permissions-note,
  .permissions-status {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .permissions-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .permission-row {
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-sm);
    background-color: var(--surface-raised);
  }

  .permission-row-main strong {
    color: var(--text-primary);
    font-size: var(--text-sm);
    overflow-wrap: anywhere;
  }

  .permission-checkboxes {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .permission-checkbox {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-primary);
    font-size: var(--text-xs);
  }

  .permissions-status {
    margin: 0;
  }

  .permissions-status--success {
    color: var(--color-success-600);
  }

  .permissions-status--error {
    color: var(--color-danger-600);
  }
</style>
