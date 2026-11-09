<script lang="ts">
  import { t } from '$lib/i18n';
  import {
    createEntityLink,
    deleteEntityLink,
    listEntityLinks,
    type EntityLink,
    type LinkEntityType,
  } from '$lib/api/links';
  import { openExternalUrl, openLocalPath } from '$lib/utils/openExternal';

  let {
    entityType,
    entityId,
  }: {
    entityType: LinkEntityType;
    entityId: string;
  } = $props();

  let links = $state<EntityLink[]>([]);
  let title = $state('');
  let target = $state('');
  let isLoading = $state(false);
  let isSaving = $state(false);
  let openingId = $state<string | null>(null);
  let deletingId = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let loadToken = 0;

  const canCreate = $derived(target.trim().length > 0 && !isSaving && entityId.trim().length > 0);

  $effect(() => {
    if (!entityId.trim()) {
      links = [];
      isLoading = false;
      errorMessage = null;
      return;
    }
    void loadLinks(entityType, entityId.trim());
  });

  function messageFromError(error: unknown, fallback: string): string {
    if (typeof error === 'string') return error;
    return error instanceof Error ? error.message : fallback;
  }

  async function loadLinks(type = entityType, id = entityId.trim()): Promise<void> {
    if (!id.trim()) return;
    const token = ++loadToken;
    isLoading = true;
    errorMessage = null;
    try {
      const loaded = await listEntityLinks(type, id);
      if (token === loadToken) {
        links = loaded;
      }
    } catch (error) {
      if (token === loadToken) {
        errorMessage = messageFromError(error, t('entityLinks.loadFailed'));
      }
    } finally {
      if (token === loadToken) {
        isLoading = false;
      }
    }
  }

  async function addUrlLink(): Promise<void> {
    const nextTarget = target.trim();
    if (!nextTarget) return;
    isSaving = true;
    errorMessage = null;
    try {
      const link = await createEntityLink({
        entityType,
        entityId: entityId.trim(),
        title: title.trim() || null,
        kind: 'url',
        target: nextTarget,
      });
      links = [link, ...links];
      title = '';
      target = '';
    } catch (error) {
      errorMessage = messageFromError(error, t('entityLinks.saveFailed'));
    } finally {
      isSaving = false;
    }
  }

  async function addFileLink(): Promise<void> {
    errorMessage = null;
    try {
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
      const selected = await openDialog({
        multiple: false,
        directory: false,
        title: t('entityLinks.chooseFile'),
      });
      if (!selected || Array.isArray(selected)) {
        return;
      }
      isSaving = true;
      const link = await createEntityLink({
        entityType,
        entityId: entityId.trim(),
        title: title.trim() || null,
        kind: 'path',
        target: selected,
      });
      links = [link, ...links];
      title = '';
      target = '';
    } catch (error) {
      errorMessage = messageFromError(error, t('entityLinks.saveFailed'));
    } finally {
      isSaving = false;
    }
  }

  async function openLink(link: EntityLink): Promise<void> {
    openingId = link.id;
    errorMessage = null;
    try {
      if (link.kind === 'path') {
        await openLocalPath(link.target);
      } else {
        await openExternalUrl(link.target);
      }
    } catch (error) {
      errorMessage = messageFromError(error, t('entityLinks.openFailed'));
    } finally {
      openingId = null;
    }
  }

  async function removeLink(link: EntityLink): Promise<void> {
    if (!window.confirm(t('entityLinks.confirmDelete'))) {
      return;
    }
    deletingId = link.id;
    errorMessage = null;
    try {
      await deleteEntityLink(link.id);
      links = links.filter((item) => item.id !== link.id);
    } catch (error) {
      errorMessage = messageFromError(error, t('entityLinks.deleteFailed'));
    } finally {
      deletingId = null;
    }
  }
</script>

<section class="entity-links-panel" aria-labelledby="entity-links-heading">
  <div class="panel-header">
    <h3 class="section-title" id="entity-links-heading">{t('entityLinks.title')}</h3>
    <p class="panel-help">{t('entityLinks.help')}</p>
  </div>

  <form
    class="link-create"
    onsubmit={(event) => {
      event.preventDefault();
      void addUrlLink();
    }}
  >
    <input
      class="input"
      type="text"
      bind:value={title}
      placeholder={t('entityLinks.titlePlaceholder')}
      aria-label={t('entityLinks.titleLabel')}
      disabled={isSaving || !entityId.trim()}
    />
    <input
      class="input"
      type="url"
      bind:value={target}
      placeholder={t('entityLinks.urlPlaceholder')}
      aria-label={t('entityLinks.urlLabel')}
      disabled={isSaving || !entityId.trim()}
    />
    <div class="panel-actions">
      <button class="btn btn-secondary btn-sm" type="button" onclick={() => void addFileLink()} disabled={isSaving || !entityId.trim()}>
        {t('entityLinks.chooseFile')}
      </button>
      <button class="btn btn-primary btn-sm" type="submit" disabled={!canCreate}>
        {isSaving ? t('common.loading') : t('entityLinks.addUrl')}
      </button>
    </div>
  </form>

  {#if errorMessage}
    <div class="panel-error" role="alert">
      <span>{errorMessage}</span>
      <button class="btn btn-ghost btn-sm" type="button" onclick={() => void loadLinks()}>
        {t('common.retry')}
      </button>
    </div>
  {/if}

  {#if isLoading}
    <div class="link-loading" aria-label={t('common.loading')} aria-live="polite">
      {#each [1, 2] as row (row)}
        <div class="skeleton link-skeleton"></div>
      {/each}
    </div>
  {:else if links.length === 0}
    <p class="panel-empty">{t('entityLinks.empty')}</p>
  {:else}
    <ul class="link-list" role="list">
      {#each links as link (link.id)}
        <li class="link-row">
          <div class="link-copy">
            <button class="link-open" type="button" onclick={() => void openLink(link)} disabled={openingId === link.id}>
              {link.title || link.target}
            </button>
            <small>{link.kind === 'path' ? t('entityLinks.filePath') : t('entityLinks.website')} · {link.target}</small>
          </div>
          <button
            class="btn btn-ghost btn-sm"
            type="button"
            onclick={() => void removeLink(link)}
            disabled={deletingId === link.id}
          >
            {deletingId === link.id ? t('common.loading') : t('common.delete')}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .entity-links-panel,
  .link-create,
  .link-copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .entity-links-panel {
    gap: var(--space-4);
  }

  .panel-help,
  .panel-empty,
  .link-copy small {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    overflow-wrap: anywhere;
  }

  .panel-actions,
  .link-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .panel-actions {
    justify-content: flex-end;
  }

  .link-row {
    padding-block: var(--space-2);
    border-block-end: var(--border-width) solid var(--border-subtle);
  }

  .link-open {
    border: 0;
    padding: 0;
    background: transparent;
    color: var(--text-accent);
    text-align: left;
    cursor: pointer;
    font-weight: var(--weight-medium);
  }

  .panel-error {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    color: var(--text-danger);
    font-size: var(--text-sm);
  }

  .link-skeleton {
    height: 42px;
    border-radius: var(--radius-md);
  }
</style>
