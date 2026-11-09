<script lang="ts">
  /**
   * Organizations.svelte - First-class organization list and command surface.
   */

  import { t } from '$lib/i18n';
  import { listActivities, type Activity } from '$lib/api/activities';
  import { listDeals, type Deal } from '$lib/api/deals';
  import { organizationStore } from '$lib/stores/organizations';
  import { uiStore } from '$lib/stores/ui';
  import type { Organization } from '$lib/api/organizations';
  import { formatDate } from '$lib/utils/formatters';
  import { settingsStore } from '$lib/stores/settings';
  import {
    loadActivityLinkIndex,
    type ActivityLinkIndex,
  } from '$lib/utils/activityRelationships';
  import { buildOrganizationListInsight } from '$lib/utils/organizationWorkspace';
  import {
    listCustomFieldDefinitions,
    listCustomFieldValues,
    setCustomFieldValue,
    type CustomFieldDefinition,
  } from '$lib/api/customFields';
  import Modal from '$lib/components/Modal.svelte';
  import CustomFieldInputs from '$lib/components/CustomFieldInputs.svelte';
  import EntityNotesPanel from '$lib/components/EntityNotesPanel.svelte';
  import EntityTagsPanel from '$lib/components/EntityTagsPanel.svelte';
  import { navigateHash } from '$lib/utils/hashRouter';
  import { openExternalUrl } from '$lib/utils/openExternal';
  import {
    createSavedView,
    deleteSavedView,
    filtersMatch,
    listSavedViews,
    type ContactSavedViewFilters,
    type SavedView,
  } from '$lib/api/savedViews';

  interface OrganizationFormState {
    name: string;
    email: string;
    phone: string;
    website: string;
    addressLine1: string;
    addressLine2: string;
    city: string;
    region: string;
    country: string;
    postalCode: string;
    description: string;
    owner: string;
  }

  let searchQuery = $state('');
  let countryFilter = $state('');
  let ownerFilter = $state('');
  let formOpen = $state(false);
  let linkOpen = $state(false);
  let notesTagsOpen = $state(false);
  let editingOrganization = $state<Organization | null>(null);
  let linkOrganization = $state<Organization | null>(null);
  let notesTagsOrganization = $state<Organization | null>(null);
  let linkContactId = $state('');
  let form = $state<OrganizationFormState>(emptyOrganizationForm());
  let customFieldDefinitions = $state<CustomFieldDefinition[]>([]);
  let customFieldValues = $state<Record<string, string>>({});
  let originalCustomFieldValues = $state<Record<string, string>>({});
  let customFieldsLoading = $state(false);
  let savedViews = $state<SavedView[]>([]);
  let selectedViewId = $state('');
  let viewName = $state('');
  let viewsLoading = $state(false);
  let viewsSaving = $state(false);
  let viewsError = $state<string | null>(null);
  let insightDeals = $state<Deal[]>([]);
  let insightActivities = $state<Activity[]>([]);
  let insightLinkIndex = $state<ActivityLinkIndex>({});
  let insightsLoading = $state(true);
  let organizationsBootstrapped = false;

  const countryOptions = $derived(
    [...new Set(
      organizationStore.organizations
        .map((organization) => organization.country?.trim())
        .filter((country): country is string => Boolean(country)),
    )].sort((left, right) => left.localeCompare(right)),
  );

  const ownerOptions = $derived(
    [...new Set(
      organizationStore.organizations
        .map((organization) => organization.owner?.trim())
        .filter((owner): owner is string => Boolean(owner)),
    )].sort((left, right) => left.localeCompare(right)),
  );

  const currentViewFilters = $derived(collectCurrentFilters());
  const selectedView = $derived(savedViews.find((view) => view.id === selectedViewId) ?? null);
  const canSaveView = $derived(viewName.trim().length > 0 && !viewsSaving);

  const filteredOrganizations = $derived(
    organizationStore.organizations.filter((organization) =>
      organizationMatches(organization, searchQuery, countryFilter, ownerFilter),
    ),
  );

  const insightsById = $derived.by(() => {
    const insights: Record<string, ReturnType<typeof buildOrganizationListInsight>> = {};
    for (const organization of filteredOrganizations) {
      insights[organization.id] = buildOrganizationListInsight({
        organizationId: organization.id,
        deals: insightDeals,
        activities: insightActivities,
        linkIndex: insightLinkIndex,
        isLoading: insightsLoading,
      });
    }
    return insights;
  });

  $effect(() => {
    if (organizationsBootstrapped) {
      return;
    }

    organizationsBootstrapped = true;
    void Promise.all([
      organizationStore.loadOrganizations(),
      loadOrganizationCustomFieldDefinitions(),
      loadSavedViews(),
      loadListInsights(),
    ]);
  });

  async function loadListInsights(): Promise<void> {
    insightsLoading = true;
    try {
      const [deals, activities] = await Promise.all([
        listDeals({ sortBy: 'createdAt', sortDir: 'asc' }),
        listActivities({ sortBy: 'dueDate', sortDir: 'asc', pageSize: 200 }),
      ]);
      insightLinkIndex = await loadActivityLinkIndex(activities.map((activity) => activity.id));
      insightDeals = deals;
      insightActivities = activities;
    } catch (error) {
      console.error('[Organizations] Failed to load account health:', error);
      insightDeals = [];
      insightActivities = [];
      insightLinkIndex = {};
    } finally {
      insightsLoading = false;
    }
  }

  function emptyOrganizationForm(): OrganizationFormState {
    return {
      name: '',
      email: '',
      phone: '',
      website: '',
      addressLine1: '',
      addressLine2: '',
      city: '',
      region: '',
      country: '',
      postalCode: '',
      description: '',
      owner: '',
    };
  }

  function organizationMatches(
    organization: Organization,
    query: string,
    country: string,
    owner: string,
  ): boolean {
    const normalizedCountry = country.trim().toLowerCase();
    if (normalizedCountry && (organization.country ?? '').trim().toLowerCase() !== normalizedCountry) {
      return false;
    }

    const normalizedOwner = owner.trim().toLowerCase();
    if (normalizedOwner && (organization.owner ?? '').trim().toLowerCase() !== normalizedOwner) {
      return false;
    }

    const normalized = query.trim().toLowerCase();
    if (!normalized) return true;

    return [
      organization.name,
      organization.email,
      organization.phone,
      organization.website,
      organization.city,
      organization.region,
      organization.country,
      organization.description,
      organization.owner,
    ]
      .filter((value): value is string => Boolean(value))
      .some((value) => value.toLowerCase().includes(normalized));
  }

  function collectCurrentFilters(): ContactSavedViewFilters {
    return {
      search: searchQuery.trim() || undefined,
      country: countryFilter.trim() || undefined,
      owner: ownerFilter.trim() || undefined,
    };
  }

  async function loadSavedViews(): Promise<void> {
    viewsLoading = true;
    viewsError = null;
    try {
      savedViews = await listSavedViews('organization');
      if (selectedViewId && !savedViews.some((view) => view.id === selectedViewId)) {
        selectedViewId = '';
      }
    } catch (error) {
      viewsError = error instanceof Error ? error.message : t('savedViews.loadFailed');
    } finally {
      viewsLoading = false;
    }
  }

  function applyView(view: SavedView): void {
    selectedViewId = view.id;
    searchQuery = view.filters.search ?? '';
    countryFilter = view.filters.country ?? '';
    ownerFilter = view.filters.owner ?? '';
  }

  function syncSelectedView(): void {
    if (!selectedView || filtersMatch(selectedView.filters, currentViewFilters)) {
      return;
    }
    selectedViewId = '';
  }

  async function handleSaveView(): Promise<void> {
    const name = viewName.trim();
    if (!name) {
      return;
    }
    viewsSaving = true;
    viewsError = null;
    try {
      const view = await createSavedView('organization', name, collectCurrentFilters());
      savedViews = [...savedViews.filter((item) => item.id !== view.id), view]
        .sort((left, right) => left.name.localeCompare(right.name));
      selectedViewId = view.id;
      viewName = '';
    } catch (error) {
      viewsError = error instanceof Error ? error.message : t('savedViews.saveFailed');
    } finally {
      viewsSaving = false;
    }
  }

  async function handleDeleteView(): Promise<void> {
    if (!selectedView) {
      return;
    }
    if (!window.confirm(t('savedViews.confirmDelete', { name: selectedView.name }))) {
      return;
    }
    viewsSaving = true;
    viewsError = null;
    try {
      await deleteSavedView(selectedView.id);
      savedViews = savedViews.filter((view) => view.id !== selectedView.id);
      selectedViewId = '';
    } catch (error) {
      viewsError = error instanceof Error ? error.message : t('savedViews.deleteFailed');
    } finally {
      viewsSaving = false;
    }
  }

  function handleViewChange(event: Event): void {
    const id = (event.target as HTMLSelectElement).value;
    if (!id) {
      selectedViewId = '';
      return;
    }
    const view = savedViews.find((item) => item.id === id);
    if (view) {
      applyView(view);
    }
  }

  function handleCountryFilter(event: Event): void {
    countryFilter = (event.target as HTMLSelectElement).value;
    syncSelectedView();
  }

  function handleOwnerFilter(event: Event): void {
    ownerFilter = (event.target as HTMLSelectElement).value;
    syncSelectedView();
  }

  function organizationLocation(organization: Organization): string {
    const parts = [organization.city, organization.region, organization.country]
      .map((part) => part?.trim())
      .filter((part): part is string => Boolean(part));

    return parts.length > 0 ? parts.join(', ') : '—';
  }

  function blankToNull(value: string): string | null {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : null;
  }

  function openCreateForm() {
    editingOrganization = null;
    form = emptyOrganizationForm();
    resetCustomFieldsForCreate();
    formOpen = true;
  }

  function openEditForm(organization: Organization) {
    editingOrganization = organization;
    form = {
      name: organization.name,
      email: organization.email ?? '',
      phone: organization.phone ?? '',
      website: organization.website ?? '',
      addressLine1: organization.addressLine1 ?? '',
      addressLine2: organization.addressLine2 ?? '',
      city: organization.city ?? '',
      region: organization.region ?? '',
      country: organization.country ?? '',
      postalCode: organization.postalCode ?? '',
      description: organization.description ?? '',
      owner: organization.owner ?? '',
    };
    void loadOrganizationCustomFieldValues(organization.id);
    formOpen = true;
  }

  function closeForm() {
    formOpen = false;
    editingOrganization = null;
    form = emptyOrganizationForm();
    resetCustomFieldsForCreate();
  }

  async function loadOrganizationCustomFieldDefinitions() {
    customFieldsLoading = true;
    try {
      customFieldDefinitions = await listCustomFieldDefinitions('organization');
      resetCustomFieldsForCreate();
    } catch (err) {
      console.error('[Organizations] Failed to load custom fields:', err);
      uiStore.toastError(t('errors.loadCustomFields'));
      customFieldDefinitions = [];
      customFieldValues = {};
      originalCustomFieldValues = {};
    } finally {
      customFieldsLoading = false;
    }
  }

  async function loadOrganizationCustomFieldValues(entityId: string) {
    customFieldsLoading = true;
    try {
      const values = await listCustomFieldValues('organization', entityId);
      const nextValues = blankCustomFieldValues();
      for (const value of values) {
        nextValues[value.field_def_id] = value.value ?? '';
      }
      customFieldValues = nextValues;
      originalCustomFieldValues = { ...nextValues };
    } catch (err) {
      console.error('[Organizations] Failed to load custom field values:', err);
      uiStore.toastError(t('errors.loadCustomFieldValues'));
      customFieldValues = blankCustomFieldValues();
      originalCustomFieldValues = { ...customFieldValues };
    } finally {
      customFieldsLoading = false;
    }
  }

  function blankCustomFieldValues(): Record<string, string> {
    return Object.fromEntries(customFieldDefinitions.map((definition) => [definition.id, '']));
  }

  function resetCustomFieldsForCreate() {
    customFieldValues = blankCustomFieldValues();
    originalCustomFieldValues = { ...customFieldValues };
  }

  function handleCustomFieldChange(fieldDefId: string, value: string) {
    customFieldValues = { ...customFieldValues, [fieldDefId]: value };
  }

  async function persistOrganizationCustomFields(entityId: string, includeBlankValues: boolean) {
    if (customFieldDefinitions.length === 0) {
      return;
    }

    const updates = customFieldDefinitions
      .filter((definition) => {
        const value = customFieldValues[definition.id] ?? '';
        if (!includeBlankValues) {
          return value.trim().length > 0;
        }
        return value !== (originalCustomFieldValues[definition.id] ?? '');
      })
      .map((definition) =>
        setCustomFieldValue({
          fieldDefId: definition.id,
          entityId,
          value: customFieldValues[definition.id] ?? '',
        })
      );

    await Promise.all(updates);
  }

  async function submitOrganization() {
    if (!form.name.trim()) {
      uiStore.toastError(t('common.fieldRequired', { field: t('organizations.name') }));
      return;
    }

    const payload = {
      name: form.name.trim(),
      email: blankToNull(form.email),
      phone: blankToNull(form.phone),
      website: blankToNull(form.website),
      addressLine1: blankToNull(form.addressLine1),
      addressLine2: blankToNull(form.addressLine2),
      city: blankToNull(form.city),
      region: blankToNull(form.region),
      country: blankToNull(form.country),
      postalCode: blankToNull(form.postalCode),
      description: blankToNull(form.description),
      owner: blankToNull(form.owner),
    };

    try {
      const organization = editingOrganization
        ? await organizationStore.updateOrganization(editingOrganization.id, payload)
        : await organizationStore.createOrganization(payload);

      await persistOrganizationCustomFields(organization.id, Boolean(editingOrganization));
    } catch (err) {
      console.error('[Organizations] Failed to save organization:', err);
      uiStore.toastError(t('errors.updateNamed', { name: t('entities.organization') }));
      return;
    }

    closeForm();
  }

  async function deleteOrganization(organization: Organization) {
    const confirmed = window.confirm(t('organizations.confirmDelete', { name: organization.name }));
    if (!confirmed) return;

    await organizationStore.deleteOrganization(organization.id);
  }

  function openNotesTags(organization: Organization) {
    notesTagsOrganization = organization;
    notesTagsOpen = true;
  }

  function openOrganizationDetail(organization: Organization) {
    organizationStore.selectOrganization(organization);
    navigateHash(`/organizations/${organization.id}`);
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

  function closeNotesTags() {
    notesTagsOpen = false;
    notesTagsOrganization = null;
  }

  function openLinkContact(organization: Organization) {
    linkOrganization = organization;
    linkContactId = '';
    linkOpen = true;
  }

  function closeLinkContact() {
    linkOpen = false;
    linkOrganization = null;
    linkContactId = '';
  }

  async function submitLinkContact(organizationId: string | null) {
    const contactId = linkContactId.trim();
    if (!contactId) {
      uiStore.toastError(t('organizations.linkContactRequired'));
      return;
    }

    await organizationStore.linkContactToOrganization(contactId, organizationId);
    closeLinkContact();
  }
</script>

<div class="page-content organizations-page">
  <div class="page-header">
    <h1 class="page-title">{t('organizations.title')}</h1>
    <button class="btn btn-primary btn-sm" type="button" onclick={openCreateForm}>
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <path d="M6 1v10M1 6h10"/>
      </svg>
      {t('organizations.addOrganization')}
    </button>
  </div>

  <section class="saved-views" aria-labelledby="org-saved-views-heading">
    <div class="saved-views-copy">
      <h2 class="saved-views-title" id="org-saved-views-heading">{t('savedViews.title')}</h2>
      <p class="saved-views-help">{t('savedViews.helpOrganizations')}</p>
    </div>
    <div class="saved-views-controls">
      <select
        class="input saved-views-select"
        value={selectedViewId}
        onchange={handleViewChange}
        aria-label={t('savedViews.selectLabel')}
        disabled={viewsLoading || viewsSaving}
      >
        <option value="">{t('savedViews.none')}</option>
        {#each savedViews as view (view.id)}
          <option value={view.id}>{view.name}</option>
        {/each}
      </select>
      <input
        class="input saved-views-name"
        type="text"
        bind:value={viewName}
        placeholder={t('savedViews.namePlaceholder')}
        aria-label={t('savedViews.nameLabel')}
        disabled={viewsSaving}
      />
      <button
        class="btn btn-secondary btn-sm"
        type="button"
        onclick={() => void handleSaveView()}
        disabled={!canSaveView}
      >
        {viewsSaving ? t('common.loading') : t('savedViews.save')}
      </button>
      <button
        class="btn btn-ghost btn-sm"
        type="button"
        onclick={() => void handleDeleteView()}
        disabled={!selectedView || viewsSaving}
      >
        {t('savedViews.delete')}
      </button>
    </div>
    {#if viewsError}
      <p class="saved-views-error" role="alert">{viewsError}</p>
    {/if}
  </section>

  <div class="organizations-filters">
    <div class="search-wrap">
      <svg class="filter-search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        class="input filter-search selectable"
        type="search"
        bind:value={searchQuery}
        oninput={syncSelectedView}
        placeholder={t('organizations.search')}
        aria-label={t('organizations.search')}
      />
    </div>
    <select
      class="input country-filter"
      value={countryFilter}
      onchange={handleCountryFilter}
      aria-label={t('organizations.country')}
    >
      <option value="">{t('savedViews.allCountries')}</option>
      {#if countryFilter && !countryOptions.includes(countryFilter)}
        <option value={countryFilter}>{countryFilter}</option>
      {/if}
      {#each countryOptions as country (country)}
        <option value={country}>{country}</option>
      {/each}
    </select>
    <select
      class="input owner-filter"
      value={ownerFilter}
      onchange={handleOwnerFilter}
      aria-label={t('common.filterOwner')}
    >
      <option value="">{t('common.all')}</option>
      {#if ownerFilter && !ownerOptions.includes(ownerFilter)}
        <option value={ownerFilter}>{ownerFilter}</option>
      {/if}
      {#each ownerOptions as owner (owner)}
        <option value={owner}>{owner}</option>
      {/each}
    </select>
  </div>

  <div class="organizations-table card">
    <div class="table-scroll">
      <table class="organization-list" aria-label={t('organizations.title')}>
        <thead>
          <tr>
            <th>{t('organizations.name')}</th>
            <th>{t('organizations.email')}</th>
            <th>{t('organizations.phone')}</th>
            <th>{t('organizations.website')}</th>
            <th>{t('organizations.location')}</th>
            <th>{t('common.owner')}</th>
            <th>{t('organizations.health')}</th>
            <th>{t('organizations.nextFollowUp')}</th>
            <th>{t('common.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#if organizationStore.isLoading}
            {#each Array(6) as _, index (index)}
              <tr class="skeleton-row">
                <td colspan="9"><div class="skeleton table-skeleton-cell"></div></td>
              </tr>
            {/each}
          {:else if filteredOrganizations.length === 0}
            <tr>
              <td colspan="9" class="empty-cell">
                <div class="empty-state">
                  <h2>{t('organizations.noOrganizations')}</h2>
                  <p>{t('organizations.noOrganizationsDesc')}</p>
                  <button class="btn btn-primary btn-sm" type="button" onclick={openCreateForm}>
                    {t('organizations.addOrganization')}
                  </button>
                </div>
              </td>
            </tr>
          {:else}
            {#each filteredOrganizations as organization (organization.id)}
              {@const insight = insightsById[organization.id]}
              <tr>
                <td>
                  <div class="organization-name-cell">
                    <button
                      class="organization-name organization-name-button"
                      type="button"
                      onclick={() => openOrganizationDetail(organization)}
                    >
                      {organization.name}
                    </button>
                    {#if organization.description}
                      <span class="organization-description">{organization.description}</span>
                    {/if}
                  </div>
                </td>
                <td>{organization.email ?? '—'}</td>
                <td>{organization.phone ?? '—'}</td>
                <td>
                  {#if organization.website}
                    <button
                      class="link-button"
                      type="button"
                      onclick={() => void openOrganizationWebsite(organization.website)}
                    >
                      {organization.website}
                    </button>
                  {:else}
                    —
                  {/if}
                </td>
                <td>{organizationLocation(organization)}</td>
                <td>{organization.owner ?? '—'}</td>
                <td>
                  <span class="health-badge health-{insight?.health.tone ?? 'neutral'}">
                    {t(`organizations.workspace.health.${insight?.health.state ?? 'loading'}`)}
                  </span>
                </td>
                <td>
                  {#if insight?.nextActivity}
                    <div class="next-follow-up">
                      <span>{insight.nextActivity.subject}</span>
                      <small>
                        {insight.nextActivity.dueDate
                          ? formatDate(insight.nextActivity.dueDate, settingsStore.dateFormat as 'MMM D, YYYY')
                          : t('organizations.workspace.noDueDate')}
                      </small>
                    </div>
                  {:else}
                    {t('organizations.workspace.noneScheduled')}
                  {/if}
                </td>
                <td>
                  <div class="row-actions">
                    <button class="btn btn-primary btn-sm" type="button" onclick={() => openOrganizationDetail(organization)}>
                      {t('organizations.viewAccount')}
                    </button>
                    <button class="btn btn-ghost btn-sm" type="button" onclick={() => openEditForm(organization)}>
                      {t('common.edit')}
                    </button>
                    <button class="btn btn-ghost btn-sm" type="button" onclick={() => openNotesTags(organization)}>
                      {t('organizations.manageNotesTags')}
                    </button>
                    <button class="btn btn-ghost btn-sm" type="button" onclick={() => openLinkContact(organization)}>
                      {t('organizations.linkContact')}
                    </button>
                    <button class="btn btn-ghost btn-sm danger-action" type="button" onclick={() => deleteOrganization(organization)}>
                      {t('common.delete')}
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<Modal
  bind:open={formOpen}
  title={editingOrganization ? t('organizations.editOrganization') : t('organizations.addOrganization')}
  size="lg"
  onclose={closeForm}
>
  <form class="organization-form" onsubmit={(event) => { event.preventDefault(); void submitOrganization(); }}>
    <div class="form-grid">
      <label class="form-group">
        <span class="form-label">{t('organizations.name')}</span>
        <input class="input selectable" bind:value={form.name} required />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.email')}</span>
        <input class="input selectable" type="email" bind:value={form.email} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.phone')}</span>
        <input class="input selectable" bind:value={form.phone} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.website')}</span>
        <input class="input selectable" type="url" bind:value={form.website} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.addressLine1')}</span>
        <input class="input selectable" bind:value={form.addressLine1} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.addressLine2')}</span>
        <input class="input selectable" bind:value={form.addressLine2} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.city')}</span>
        <input class="input selectable" bind:value={form.city} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.region')}</span>
        <input class="input selectable" bind:value={form.region} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.country')}</span>
        <input class="input selectable" bind:value={form.country} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('organizations.postalCode')}</span>
        <input class="input selectable" bind:value={form.postalCode} />
      </label>
      <label class="form-group">
        <span class="form-label">{t('common.owner')}</span>
        <input class="input selectable" bind:value={form.owner} placeholder={t('common.optional')} />
      </label>
      <label class="form-group form-group-full">
        <span class="form-label">{t('organizations.description')}</span>
        <textarea class="textarea selectable" rows="3" bind:value={form.description}></textarea>
      </label>
    </div>
    <div class="organization-form-custom-fields">
      {#if customFieldsLoading}
        <p class="custom-fields-placeholder">{t('common.loading')}</p>
      {:else}
        <CustomFieldInputs
          definitions={customFieldDefinitions}
          values={customFieldValues}
          onchange={handleCustomFieldChange}
          disabled={organizationStore.isSaving}
        />
      {/if}
    </div>

    <div class="modal-form-actions">
      <button class="btn btn-ghost" type="button" onclick={closeForm}>{t('common.cancel')}</button>
      <button class="btn btn-primary" type="submit" disabled={organizationStore.isSaving}>
        {editingOrganization ? t('organizations.updateOrganization') : t('organizations.createOrganization')}
      </button>
    </div>
  </form>
</Modal>

<Modal
  bind:open={notesTagsOpen}
  title={notesTagsOrganization ? t('organizations.notesTagsTitle', { name: notesTagsOrganization.name }) : t('organizations.manageNotesTags')}
  size="lg"
  onclose={closeNotesTags}
>
  {#if notesTagsOrganization}
    <div class="organization-entity-panels">
      <section class="organization-entity-section" aria-labelledby="organization-tags-heading">
        <h2 class="modal-section-title" id="organization-tags-heading">{t('entityTags.title')}</h2>
        <EntityTagsPanel entityType="organization" entityId={notesTagsOrganization.id} />
      </section>

      <section class="organization-entity-section" aria-labelledby="organization-notes-heading">
        <h2 class="modal-section-title" id="organization-notes-heading">{t('entityNotes.title')}</h2>
        <EntityNotesPanel entityType="organization" entityId={notesTagsOrganization.id} />
      </section>
    </div>
  {/if}
</Modal>

<Modal
  bind:open={linkOpen}
  title={t('organizations.contactLink')}
  size="sm"
  onclose={closeLinkContact}
>
  <form class="link-form" onsubmit={(event) => { event.preventDefault(); void submitLinkContact(linkOrganization?.id ?? null); }}>
    <p class="link-description">{t('organizations.linkContactDesc')}</p>
    <label class="form-group">
      <span class="form-label">{t('organizations.contactId')}</span>
      <input class="input selectable" bind:value={linkContactId} required />
    </label>
    <div class="modal-form-actions link-actions">
      <button class="btn btn-ghost" type="button" onclick={closeLinkContact}>{t('common.cancel')}</button>
      <button
        class="btn btn-secondary"
        type="button"
        onclick={() => submitLinkContact(null)}
        disabled={organizationStore.isLinkingContact}
      >
        {t('organizations.clearContactLink')}
      </button>
      <button class="btn btn-primary" type="submit" disabled={organizationStore.isLinkingContact || !linkOrganization}>
        {t('organizations.linkContact')}
      </button>
    </div>
  </form>
</Modal>

<style>
  .organizations-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: 100%;
  }

  .saved-views,
  .saved-views-copy,
  .saved-views-controls,
  .organizations-filters {
    display: flex;
    gap: var(--space-3);
    align-items: center;
    flex-wrap: wrap;
  }

  .saved-views {
    justify-content: space-between;
  }

  .saved-views-title {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .saved-views-help,
  .saved-views-error {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .saved-views-error {
    color: var(--text-danger);
    width: 100%;
  }

  .saved-views-select,
  .saved-views-name,
  .country-filter,
  .owner-filter {
    min-width: 160px;
  }

  .organizations-filters {
    gap: var(--space-4);
  }

  .search-wrap {
    position: relative;
    flex: 1;
    min-width: 220px;
    max-width: 380px;
  }

  .filter-search-icon {
    position: absolute;
    inset-inline-start: var(--space-4);
    top: 50%;
    transform: translateY(-50%);
    color: var(--icon-muted);
    pointer-events: none;
  }

  .filter-search {
    height: 34px;
    padding-inline-start: var(--space-10);
  }

  .organizations-table {
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

  .organization-list {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .organization-list th,
  .organization-list td {
    padding: var(--space-3) var(--space-4);
    border-block-end: var(--border-width) solid var(--border-subtle);
    text-align: start;
    vertical-align: top;
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

  .organization-list th {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    background-color: var(--surface-subtle);
  }

  .organization-name-cell {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 180px;
  }

  .organization-name {
    color: var(--text-primary);
    font-weight: var(--weight-semibold);
  }

  .health-badge {
    display: inline-flex;
    padding: var(--space-1) var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    white-space: nowrap;
  }

  .health-neutral {
    color: var(--text-secondary);
    background-color: var(--surface-raised);
  }

  .health-success {
    color: #2d8659;
    border-color: #bfe4cc;
    background-color: #e8f5ec;
  }

  .health-warning {
    color: #a84b2f;
    border-color: #f4d1a1;
    background-color: #fef3e2;
  }

  .health-danger {
    color: #c0392b;
    border-color: #f0b8b2;
    background-color: #fff0f0;
  }

  .next-follow-up {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 140px;
  }

  .next-follow-up small {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .organization-name-button {
    width: fit-content;
    padding: 0;
    border: none;
    background: none;
    cursor: pointer;
    text-align: start;
  }

  .organization-name-button:hover {
    color: var(--text-accent);
  }

  .organization-description {
    color: var(--text-tertiary);
    font-size: var(--text-xs);
    max-width: 280px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .danger-action {
    color: var(--color-danger-600);
  }

  .empty-cell {
    padding: var(--space-10) var(--space-4);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    min-height: 260px;
    color: var(--text-secondary);
    text-align: center;
  }

  .empty-state h2 {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-lg);
  }

  .empty-state p {
    margin: 0;
  }

  .organization-form,
  .link-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
  }

  .form-group-full {
    grid-column: 1 / -1;
  }

  .organization-form-custom-fields {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .custom-fields-placeholder {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .modal-form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .organization-entity-panels {
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .organization-entity-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding-block-end: var(--space-8);
    border-block-end: var(--border-width) solid var(--border-subtle);
  }

  .organization-entity-section:last-child {
    padding-block-end: 0;
    border-block-end: none;
  }

  .modal-section-title {
    margin: 0;
    color: var(--text-primary);
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
  }

  .link-description {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .link-actions {
    justify-content: space-between;
  }

  @media (max-width: 720px) {
    .form-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
