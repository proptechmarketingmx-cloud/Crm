<script lang="ts">
  import { t } from '$lib/i18n';
  import { organizationStore } from '$lib/stores/organizations';
  import { dealStore } from '$lib/stores/deals';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { openExternalUrl } from '$lib/utils/openExternal';
  import { settingsStore } from '$lib/stores/settings';
  import type { Organization } from '$lib/api/organizations';
  import type { Contact } from '$lib/api/contacts';
  import { listContacts } from '$lib/api/contacts';
  import type { Activity } from '$lib/api/activities';
  import { listActivities } from '$lib/api/activities';
  import {
    loadActivityLinkIndex,
    loadActivityRelationshipLookups,
    relationshipLabelsByActivityId,
    sortActivitiesForDetailTimeline,
    type ActivityLinkIndex,
    type ActivityRelationshipLabels,
    type ActivityRelationshipLookups,
  } from '$lib/utils/activityRelationships';
  import {
    deriveOrganizationHealth,
    filterOrganizationActivities,
    filterOrganizationContacts,
    filterOrganizationDeals,
    nextOrganizationActivity,
    openPipelineByCurrency,
    recentOrganizationActivity,
  } from '$lib/utils/organizationWorkspace';
  import {
    formatCurrency,
    formatDate,
    formatFullName,
    formatInitials,
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
  import {
    deriveRecordNextStep,
    shouldShowSecondaryFollowUp,
  } from '$lib/utils/recordNextStep';

  const { organizationId }: { organizationId: string } = $props();

  let organization = $state<Organization | null>(null);
  let contacts = $state<Contact[]>([]);
  let organizationActivities = $state<Activity[]>([]);
  let organizationActivityLinkIndex = $state<ActivityLinkIndex>({});
  let organizationActivityLookups = $state<ActivityRelationshipLookups>({
    contacts: [],
    organizations: [],
    deals: [],
  });
  let isLoading = $state(true);
  let isSaving = $state(false);
  let contactsLoading = $state(false);
  let activitiesLoading = $state(false);
  let loadError = $state<string | null>(null);
  let loadedOrganizationId = '';
  let lastActivityRefreshVersion = -1;

  const linkedContacts = $derived.by(() =>
    filterOrganizationContacts(contacts, organizationId)
  );

  const linkedDeals = $derived.by(() =>
    filterOrganizationDeals(dealStore.deals, organizationId)
  );

  const openDeals = $derived.by(() =>
    linkedDeals.filter((deal) => deal.stage !== 'closedWon' && deal.stage !== 'closedLost')
  );

  const openDealCount = $derived(openDeals.length);

  const openPipelineValueByCurrency = $derived.by(() =>
    openPipelineByCurrency(openDeals, settingsStore.currency || 'USD')
  );

  const openPipelineValue = $derived.by(() => {
    if (openPipelineValueByCurrency.length === 0) {
      return t('organizations.workspace.noOpenValue');
    }

    return openPipelineValueByCurrency
      .map(({ currency, value }) => formatCurrency(value, currency, settingsStore.language))
      .join(' + ');
  });

  const pendingActivities = $derived.by(() =>
    organizationActivities
      .filter((activity) => activity.status !== 'completed')
      .sort((left, right) => {
        const leftDue = Date.parse(left.dueDate ?? '') || Number.MAX_SAFE_INTEGER;
        const rightDue = Date.parse(right.dueDate ?? '') || Number.MAX_SAFE_INTEGER;
        return leftDue - rightDue;
      })
  );

  const overdueActivities = $derived.by(() =>
    pendingActivities.filter((activity) => activity.status === 'overdue')
  );

  const nextActivity = $derived(nextOrganizationActivity(organizationActivities));
  const recentActivity = $derived(recentOrganizationActivity(organizationActivities));

  const organizationActivityRelationships = $derived.by<Record<string, ActivityRelationshipLabels>>(() =>
    relationshipLabelsByActivityId(
      organizationActivities,
      organizationActivityLinkIndex,
      organizationActivityLookups,
    )
  );

  const accountHealth = $derived.by(() =>
    deriveOrganizationHealth({
      isLoading: isLoading || dealStore.isLoading || activitiesLoading,
      openDealCount,
      pendingActivities,
      overdueActivities,
      nextActivity,
    })
  );

  const nextStep = $derived(
    deriveRecordNextStep({
      recordKind: 'organization',
      isLoading: isLoading || dealStore.isLoading || activitiesLoading,
      openDealCount,
      overdueActivities,
      nextActivity,
    }),
  );

  const accountLocation = $derived(
    organization ? organizationLocation(organization) : t('common.none')
  );

  const accountInitials = $derived(
    organization ? formatInitials(organization.name) : '?'
  );

  const avatarColor = $derived.by(() => {
    if (!organization) return 0;
    let hash = 0;
    for (let i = 0; i < organization.name.length; i++) {
      hash = organization.name.charCodeAt(i) + ((hash << 5) - hash);
    }
    return Math.abs(hash) % 8;
  });

  $effect(() => {
    if (!organizationId || loadedOrganizationId === organizationId) {
      return;
    }

    loadedOrganizationId = organizationId;
    void loadWorkspace(organizationId);
  });

  $effect(() => {
    const version = activityStore.relationshipRefreshVersion;
    if (!organizationId || !organization || lastActivityRefreshVersion === version) {
      return;
    }

    lastActivityRefreshVersion = version;
    if (version > 0) {
      void loadOrganizationActivities(organizationId);
    }
  });

  async function loadWorkspace(id: string) {
    isLoading = true;
    loadError = null;
    try {
      const [loadedOrganization] = await Promise.all([
        organizationStore.getOrganization(id),
        loadOrganizationContacts(id),
        dealStore.loadDeals({
          organizationId: id,
          sortBy: 'name',
          sortDir: 'asc',
        }),
        loadOrganizationActivities(id),
      ]);

      lastActivityRefreshVersion = activityStore.relationshipRefreshVersion;
      organization = loadedOrganization;
      organizationStore.selectOrganization(loadedOrganization);
    } catch (err) {
      console.error('[OrganizationDetail] Load error:', err);
      organization = null;
      loadError = t('errors.loadFailed');
    } finally {
      isLoading = false;
    }
  }

  async function loadOrganizationContacts(id: string) {
    contactsLoading = true;
    try {
      const allContacts: Contact[] = [];
      let page = 1;
      let total = 0;

      do {
        const result = await listContacts({
          page,
          pageSize: 500,
          sortBy: 'name',
          sortDir: 'asc',
        });

        allContacts.push(...result.contacts);
        total = result.total;
        if (result.contacts.length === 0) {
          break;
        }
        page += 1;
      } while (allContacts.length < total);

      contacts = allContacts.filter((contact) => contact.organizationId === id);
    } finally {
      contactsLoading = false;
    }
  }

  async function loadOrganizationActivities(id: string) {
    activitiesLoading = true;
    try {
      const activities = await listActivities({
        sortBy: 'dueDate',
        sortDir: 'asc',
        pageSize: 500,
      });
      const [linkIndex, lookups] = await Promise.all([
        loadActivityLinkIndex(activities.map((activity) => activity.id)),
        loadActivityRelationshipLookups(),
      ]);
      organizationActivityLinkIndex = linkIndex;
      organizationActivityLookups = lookups;
      organizationActivities = sortActivitiesForDetailTimeline(
        filterOrganizationActivities(activities, linkIndex, id),
      );
    } catch (err) {
      console.error('[OrganizationDetail] Failed to load account activity:', err);
      organizationActivityLinkIndex = {};
      organizationActivityLookups = { contacts: [], organizations: [], deals: [] };
      organizationActivities = [];
    } finally {
      activitiesLoading = false;
    }
  }

  function organizationLocation(item: Organization): string {
    const parts = [item.city, item.region, item.country]
      .map((part) => part?.trim())
      .filter((part): part is string => Boolean(part));

    return parts.length > 0 ? parts.join(', ') : t('common.none');
  }

  function formatActivityDue(activity: Activity | null): string {
    if (!activity) {
      return t('organizations.workspace.noneScheduled');
    }

    if (!activity.dueDate) {
      return t('organizations.workspace.noDueDate');
    }

    return formatDate(
      activity.dueDate,
      settingsStore.dateFormat as 'MMM D, YYYY',
      settingsStore.language,
    );
  }

  function healthLabel(): string {
    return t(`organizations.workspace.health.${accountHealth.state}`);
  }

  function healthDetail(): string {
    if (accountHealth.state === 'overdue') {
      return t('organizations.workspace.health.overdueDetail', {
        subject: accountHealth.subject ?? t('activities.title'),
      });
    }

    if (accountHealth.state === 'onTrack') {
      return t('organizations.workspace.health.onTrackDetail', {
        subject: accountHealth.subject ?? t('activities.title'),
      });
    }

    return t(`organizations.workspace.health.${accountHealth.state}Detail`);
  }

  function contactName(contact: Contact): string {
    return formatFullName(contact.firstName, contact.lastName);
  }

  async function openOrganizationWebsite(website: string | null): Promise<void> {
    if (!website) {
      return;
    }

    try {
      await openExternalUrl(website);
    } catch {
      uiStore.toastError(t('common.openLinkFailed'));
    }
  }

  function openOrganizationDealModal() {
    if (!organization) return;
    organizationStore.selectOrganization(organization);
    uiStore.openModal('addDeal', { organizationId: organization.id });
  }

  function openOrganizationActivityModal() {
    if (!organization) return;
    organizationStore.selectOrganization(organization);
    uiStore.openModal('addActivity', { organizationId: organization.id });
  }

  async function handleNextStep() {
    if (nextStep.action === 'complete' && nextStep.activityId) {
      isSaving = true;
      try {
        await activityStore.markComplete(nextStep.activityId);
        await loadOrganizationActivities(organizationId);
      } catch (err) {
        console.error('[OrganizationDetail] Complete next step error:', err);
      } finally {
        isSaving = false;
      }
      return;
    }

    if (nextStep.action === 'addFollowUp') {
      openOrganizationActivityModal();
    }
  }

  function handleBack() {
    navigateHash('/organizations');
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

<div class="page-content organization-detail-page">
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
      <button class="btn btn-secondary btn-sm" type="button" onclick={() => loadWorkspace(organizationId)}>
        {t('common.retry')}
      </button>
    </div>
  {:else if organization}
    <div class="page-header">
      <div class="header-left">
        <button class="btn-back" onclick={handleBack} type="button" aria-label={t('common.back')}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M19 12H5M12 5l-7 7 7 7"/>
          </svg>
          {t('common.back')}
        </button>

        <div class="account-identity">
          <div class="avatar avatar-lg avatar-color-{avatarColor}" aria-hidden="true">
            {accountInitials}
          </div>
          <div class="identity-text">
            <h1 class="page-title">{organization.name}</h1>
            <span class="account-location">{accountLocation}</span>
          </div>
        </div>
      </div>

      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" type="button" onclick={openOrganizationDealModal}>
          {t('deals.addDeal')}
        </button>
        {#if shouldShowSecondaryFollowUp(nextStep)}
          <button class="btn btn-secondary btn-sm" type="button" onclick={openOrganizationActivityModal}>
            {t('organizations.workspace.addFollowUp')}
          </button>
        {/if}
      </div>
    </div>

    <section class="account-workspace" aria-labelledby="account-workspace-heading">
      <div class="account-workspace-header">
        <div>
          <p class="workspace-eyebrow">{t('organizations.workspace.eyebrow')}</p>
          <h2 class="section-title" id="account-workspace-heading">
            {t('organizations.workspace.title')}
          </h2>
        </div>
        <span class="health-badge health-{accountHealth.tone}">
          {healthLabel()}
        </span>
      </div>

      <p class="workspace-summary">{healthDetail()}</p>

      <NextStepCard step={nextStep} busy={isSaving} onaction={handleNextStep} />

      <div class="workspace-metrics" role="list">
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('organizations.workspace.people')}</span>
          <strong>{linkedContacts.length}</strong>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('organizations.workspace.openDeals')}</span>
          <strong>{openDealCount}</strong>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('organizations.workspace.openPipeline')}</span>
          <strong>{openPipelineValue}</strong>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('organizations.workspace.nextFollowUp')}</span>
          <strong>{nextActivity?.subject ?? t('organizations.workspace.noneScheduled')}</strong>
          <small>{formatActivityDue(nextActivity)}</small>
        </div>
      </div>

      <div class="workspace-actions" aria-label={t('organizations.workspace.actionsLabel')}>
        <button class="btn btn-secondary btn-sm" type="button" onclick={openOrganizationDealModal}>
          {t('deals.addDeal')}
        </button>
        {#if shouldShowSecondaryFollowUp(nextStep)}
          <button class="btn btn-secondary btn-sm" type="button" onclick={openOrganizationActivityModal}>
            {t('organizations.workspace.addFollowUp')}
          </button>
        {/if}
      </div>
    </section>

    <div class="detail-grid">
      <div class="detail-main">
        <section class="card account-profile" aria-labelledby="account-profile-heading">
          <div class="card-header">
            <h2 class="section-title" id="account-profile-heading">{t('organizations.workspace.profile')}</h2>
          </div>
          <div class="card-body">
            <dl class="profile-list">
              <div>
                <dt>{t('organizations.email')}</dt>
                <dd>
                  {#if organization.email}
                    <a href={`mailto:${organization.email}`}>{organization.email}</a>
                  {:else}
                    {t('common.none')}
                  {/if}
                </dd>
              </div>
              <div>
                <dt>{t('organizations.phone')}</dt>
                <dd>{organization.phone ?? t('common.none')}</dd>
              </div>
              <div>
                <dt>{t('organizations.website')}</dt>
                <dd>
                  {#if organization.website}
                    <button
                      class="link-button"
                      type="button"
                      onclick={() => void openOrganizationWebsite(organization?.website ?? null)}
                    >
                      {organization.website}
                    </button>
                  {:else}
                    {t('common.none')}
                  {/if}
                </dd>
              </div>
              <div>
                <dt>{t('organizations.location')}</dt>
                <dd>{accountLocation}</dd>
              </div>
              <div>
                <dt>{t('common.owner')}</dt>
                <dd>{organization.owner ?? t('common.none')}</dd>
              </div>
              <div>
                <dt>{t('organizations.created')}</dt>
                <dd>{formatDate(organization.createdAt, settingsStore.dateFormat as 'MMM D, YYYY', settingsStore.language)}</dd>
              </div>
              <div>
                <dt>{t('organizations.updated')}</dt>
                <dd>{formatRelativeTime(organization.updatedAt)}</dd>
              </div>
            </dl>
            <div class="profile-description">
              <h3>{t('organizations.description')}</h3>
              <p>{organization.description ?? t('organizations.workspace.noDescription')}</p>
            </div>
          </div>
        </section>

        <section class="card linked-people" aria-labelledby="linked-people-heading">
          <div class="card-header">
            <h2 class="section-title" id="linked-people-heading">{t('organizations.workspace.linkedPeople')}</h2>
          </div>
          <div class="card-body">
            {#if contactsLoading}
              <div class="sidebar-loading" aria-label={t('common.loading')}>
                {#each [1, 2, 3] as item (item)}
                  <div class="skeleton skeleton-row"></div>
                {/each}
              </div>
            {:else if linkedContacts.length === 0}
              <EmptyState
                icon="contacts"
                title={t('organizations.workspace.noLinkedPeople')}
                description={t('organizations.workspace.noLinkedPeopleDesc')}
                compact={true}
              />
            {:else}
              <ul class="people-list" role="list">
                {#each linkedContacts as contact (contact.id)}
                  <li>
                    <button
                      class="linked-record-button"
                      type="button"
                      onclick={() => navigateHash(`/contacts/${contact.id}`)}
                    >
                      <span>{contactName(contact)}</span>
                      <small>{contact.email ?? contact.phone ?? t('contacts.person')}</small>
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
        </section>

        <section class="card account-tags" aria-labelledby="account-tags-heading">
          <div class="card-header">
            <h2 class="section-title" id="account-tags-heading">{t('entityTags.title')}</h2>
          </div>
          <div class="card-body">
            <EntityTagsPanel entityType="organization" entityId={organization.id} />
          </div>
        </section>

        <section class="card account-notes" aria-labelledby="account-notes-heading">
          <div class="card-header">
            <h2 class="section-title" id="account-notes-heading">{t('entityNotes.title')}</h2>
          </div>
          <div class="card-body">
            <EntityNotesPanel entityType="organization" entityId={organization.id} />
          </div>
        </section>

        <section class="card account-links">
          <div class="card-body">
            <EntityLinksPanel entityType="organization" entityId={organization.id} />
          </div>
        </section>
      </div>

      <div class="detail-sidebar">
        <section class="card account-deals" aria-labelledby="account-deals-heading">
          <div class="card-header">
            <h2 class="section-title" id="account-deals-heading">{t('contacts.linkedDeals')}</h2>
            <button class="btn btn-secondary btn-xs" type="button" onclick={openOrganizationDealModal}>
              {t('deals.addDeal')}
            </button>
          </div>
          <div class="card-body">
            {#if dealStore.isLoading}
              <div class="sidebar-loading" aria-label={t('common.loading')}>
                {#each [1, 2] as item (item)}
                  <div class="skeleton skeleton-row"></div>
                {/each}
              </div>
            {:else if linkedDeals.length === 0}
              <EmptyState
                icon="deals"
                title={t('organizations.workspace.noLinkedDeals')}
                description={t('organizations.workspace.noLinkedDealsDesc')}
                compact={true}
              />
            {:else}
              <ul class="deals-list" role="list">
                {#each linkedDeals as deal (deal.id)}
                  <li class="deal-row">
                    <button
                      class="deal-row-button"
                      type="button"
                      onclick={() => navigateHash(`/deals/${deal.id}`)}
                    >
                      <div class="deal-row-info">
                        <span class="deal-row-name">{deal.name}</span>
                        <span class="deal-row-stage stage-badge stage-{deal.stage}">
                          {t(`deals.stages.${deal.stage}`)}
                        </span>
                      </div>
                      <span class="deal-row-value">
                        {formatCurrency(deal.value, deal.currency, settingsStore.language)}
                      </span>
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
        </section>

        <section class="card account-activity" aria-labelledby="account-activity-heading">
          <div class="card-header">
            <h2 class="section-title" id="account-activity-heading">{t('organizations.workspace.accountActivity')}</h2>
            <button class="btn btn-secondary btn-xs" type="button" onclick={openOrganizationActivityModal}>
              {t('activities.addActivity')}
            </button>
          </div>
          <div class="card-body">
            <LogJustHappened
              onlogged={() => void loadOrganizationActivities(organizationId)}
            />
            {#if organizationActivities.length === 0 && !activitiesLoading}
              <EmptyState
                icon="activities"
                title={t('organizations.workspace.noLinkedActivity')}
                description={t('organizations.workspace.noLinkedActivityDesc')}
                compact={true}
              />
            {:else}
              <ActivityFeed
                activities={organizationActivities}
                loading={activitiesLoading}
                maxItems={10}
                relationshipsByActivityId={organizationActivityRelationships}
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

<style>
  .organization-detail-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .header-left {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .header-actions {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    flex-wrap: wrap;
  }

  .btn-back {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    width: fit-content;
    padding: var(--space-1) 0;
    color: var(--text-secondary);
    border: none;
    background: none;
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
  }

  .btn-back:hover {
    color: var(--text-accent);
  }

  .account-identity {
    display: flex;
    align-items: center;
    gap: var(--space-5);
  }

  .identity-text {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .account-location {
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border-radius: 50%;
    font-weight: var(--weight-semibold);
    user-select: none;
  }

  .avatar-lg {
    width: 56px;
    height: 56px;
    font-size: var(--text-lg);
  }

  .avatar-color-0 { background-color: #E8F4F7; color: #20808D; }
  .avatar-color-1 { background-color: #FEF3E2; color: #A84B2F; }
  .avatar-color-2 { background-color: #E8F5EC; color: #2D8659; }
  .avatar-color-3 { background-color: #F3E8FF; color: #7C3AED; }
  .avatar-color-4 { background-color: #FFF0F0; color: #C0392B; }
  .avatar-color-5 { background-color: #EEF2FF; color: #3B5BDB; }
  .avatar-color-6 { background-color: #FFF8E1; color: #D4A017; }
  .avatar-color-7 { background-color: #F0FFF4; color: #2D8659; }

  .account-workspace {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    background-color: var(--surface-card);
  }

  .account-workspace-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-4);
  }

  .workspace-eyebrow {
    margin: 0 0 var(--space-1);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
  }

  .workspace-summary {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .workspace-metrics {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .workspace-metric {
    min-width: 0;
    padding-block: var(--space-2);
    border-block-start: var(--border-width) solid var(--border-default);
  }

  .workspace-metric-label {
    display: block;
    margin-block-end: var(--space-1);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
  }

  .workspace-metric strong {
    display: block;
    overflow: hidden;
    color: var(--text-primary);
    font-size: var(--text-base);
    line-height: 1.35;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .workspace-metric small {
    display: block;
    overflow: hidden;
    margin-block-start: 2px;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .workspace-actions {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    flex-wrap: wrap;
  }

  .health-badge {
    flex-shrink: 0;
    padding: var(--space-1) var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .health-neutral {
    color: var(--text-secondary);
    background-color: var(--surface-raised);
  }

  .health-success {
    color: #2D8659;
    border-color: #BFE4CC;
    background-color: #E8F5EC;
  }

  .health-warning {
    color: #A84B2F;
    border-color: #F4D1A1;
    background-color: #FEF3E2;
  }

  .health-danger {
    color: #C0392B;
    border-color: #F0B8B2;
    background-color: #FFF0F0;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 340px;
    gap: var(--space-6);
    align-items: start;
  }

  .detail-main,
  .detail-sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .profile-list {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
    margin: 0;
  }

  .profile-list div {
    min-width: 0;
  }

  .profile-list dt {
    margin-block-end: var(--space-1);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    text-transform: uppercase;
  }

  .profile-list dd {
    overflow-wrap: anywhere;
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
  }

  .profile-description {
    margin-block-start: var(--space-5);
    padding-block-start: var(--space-4);
    border-block-start: var(--border-width) solid var(--border-subtle);
  }

  .profile-description h3 {
    margin: 0 0 var(--space-2);
    color: var(--text-primary);
    font-size: var(--text-sm);
  }

  .profile-description p {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .link-button {
    padding: 0;
    border: 0;
    background: none;
    color: var(--color-primary-600);
    font: inherit;
    text-align: start;
    text-decoration: underline;
    cursor: pointer;
  }

  .people-list,
  .deals-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .linked-record-button {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: var(--space-3);
    text-align: start;
    border: var(--border-width) solid var(--border-subtle);
    border-radius: var(--border-radius-md);
    background-color: var(--surface-input);
    cursor: pointer;
  }

  .linked-record-button:hover {
    background-color: var(--surface-hover);
  }

  .linked-record-button span {
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
  }

  .linked-record-button small {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .deal-row {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    padding-block: var(--space-2);
    border-block-end: var(--border-width) solid var(--border-subtle);
  }

  .deal-row-button {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: inherit;
  }

  .deal-row:last-child {
    border-block-end: none;
  }

  .deal-row-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }

  .deal-row-name {
    overflow: hidden;
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .deal-row-value {
    flex-shrink: 0;
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .detail-loading,
  .detail-error {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .skeleton-fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
  }

  .skeleton-header {
    height: 84px;
  }

  .skeleton-field,
  .skeleton-row {
    height: 42px;
  }

  @media (max-width: 980px) {
    .detail-grid {
      grid-template-columns: 1fr;
    }

    .workspace-metrics,
    .profile-list {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 640px) {
    .account-workspace-header,
    .page-header {
      align-items: stretch;
      flex-direction: column;
    }

    .workspace-metrics,
    .profile-list {
      grid-template-columns: 1fr;
    }
  }
</style>
