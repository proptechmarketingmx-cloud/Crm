<script lang="ts">
  /**
   * ContactDetail.svelte — Contact detail and edit view for 900CRM.
   *
   * Features:
   *   - Editable fields: firstName, lastName, email, phone, organization,
   *     website, address, type
   *   - Inline notes editor via NoteEditor
   *   - Tag management via TagPicker
   *   - Linked deals list (from dealStore)
   *   - Activity timeline (from activityStore)
   *   - Back button (go to /contacts)
   *   - Soft-delete with confirmation
   */

  import { t } from '$lib/i18n';
  import { contactStore } from '$lib/stores/contacts';
  import { organizationStore } from '$lib/stores/organizations';
  import { dealStore } from '$lib/stores/deals';
  import { activityStore } from '$lib/stores/activities';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import { composeEmail } from '$lib/api/email';
  import { getContact } from '$lib/api/contacts';
  import type { Contact, UpdateContactPayload } from '$lib/api/contacts';
  import type { Deal } from '$lib/api/deals';
  import { listActivities, type Activity } from '$lib/api/activities';
  import {
    filterActivitiesByRelationship,
    loadActivityLinkIndex,
    loadActivityRelationshipLookups,
    relationshipLabelsByActivityId,
    sortActivitiesForDetailTimeline,
    type ActivityLinkIndex,
    type ActivityRelationshipLabels,
    type ActivityRelationshipLookups,
  } from '$lib/utils/activityRelationships';
  import {
    listCustomFieldDefinitions,
    listCustomFieldValues,
    setCustomFieldValue,
    type CustomFieldDefinition,
  } from '$lib/api/customFields';
  import { formatFullName, formatDate, formatRelativeTime, formatCurrency, formatInitials } from '$lib/utils/formatters';
  import { navigateHash } from '$lib/utils/hashRouter';
  import { validateEmail, validateUrl } from '$lib/utils/validators';
  import NoteEditor from '$lib/components/NoteEditor.svelte';
  import EntityNotesPanel from '$lib/components/EntityNotesPanel.svelte';
  import EntityLinksPanel from '$lib/components/EntityLinksPanel.svelte';
  import EntityTagsPanel from '$lib/components/EntityTagsPanel.svelte';
  import ActivityFeed from '$lib/components/ActivityFeed.svelte';
  import LogJustHappened from '$lib/components/LogJustHappened.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import NextStepCard from '$lib/components/NextStepCard.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import CustomFieldInputs from '$lib/components/CustomFieldInputs.svelte';
  import {
    deriveRecordNextStep,
    shouldShowSecondaryFollowUp,
  } from '$lib/utils/recordNextStep';

  // ── Props ───────────────────────────────────────────────────────────────────

  /** Contact ID to load — read from hash routing. */
  const { contactId }: { contactId: string } = $props();

  // ── State ───────────────────────────────────────────────────────────────────

  let contact = $state<Contact | null>(null);
  let isLoading = $state(true);
  let isSaving = $state(false);
  let isDeleting = $state(false);
  let showDeleteConfirm = $state(false);
  let loadError = $state<string | null>(null);

  // Editable form fields — kept in sync with contact
  let firstName = $state('');
  let lastName = $state('');
  let email = $state('');
  let phone = $state('');
  let organization = $state('');
  let selectedOrgId = $state('');
  let website = $state('');
  let address = $state('');
  let contactType = $state<'person' | 'org'>('person');
  let notes = $state('');
  let owner = $state('');
  let tags = $state<string[]>([]);

  let isDirty = $state(false);
  let emailError = $state<string | null>(null);
  let websiteError = $state<string | null>(null);

  // Linked data
  let dealsLoading = $state(false);
  let activitiesLoading = $state(false);
  let contactActivities = $state<Activity[]>([]);
  let contactActivityLinkIndex = $state<ActivityLinkIndex>({});
  let contactActivityLookups = $state<ActivityRelationshipLookups>({
    contacts: [],
    organizations: [],
    deals: [],
  });
  let customFieldsLoading = $state(false);
  let customFieldDefinitions = $state<CustomFieldDefinition[]>([]);
  let customFieldValues = $state<Record<string, string>>({});
  let originalCustomFieldValues = $state<Record<string, string>>({});
  let loadedContactId = '';
  let lastActivityRefreshVersion = -1;

  // ── Derived ─────────────────────────────────────────────────────────────────

  const displayName = $derived(
    contact ? formatFullName(contact.firstName, contact.lastName) : t('common.loading')
  );

  const initials = $derived(
    contact ? formatInitials(contact.firstName, contact.lastName) : '?'
  );

  const avatarColor = $derived.by(() => {
    if (!contact) return 0;
    const str = contact.firstName + contact.lastName;
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    return Math.abs(hash) % 8;
  });

  const linkedDeals = $derived.by(() =>
    dealStore.deals.filter((deal) => deal.contactId === contactId)
  );

  const openDeals = $derived.by(() =>
    linkedDeals.filter((deal) => isOpenDeal(deal))
  );

  const openDealCount = $derived(openDeals.length);

  const openPipelineValueByCurrency = $derived.by(() => {
    const buckets = new Map<string, number>();
    for (const deal of openDeals) {
      const currency = deal.currency || settingsStore.currency || 'USD';
      buckets.set(currency, (buckets.get(currency) ?? 0) + deal.value);
    }
    return [...buckets.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([currency, value]) => ({ currency, value }));
  });

  const openPipelineValue = $derived.by(() => {
    if (openPipelineValueByCurrency.length === 0) {
      return t('contacts.workspace.noOpenValue');
    }

    return openPipelineValueByCurrency
      .map(({ currency, value }) => formatCurrency(value, currency, settingsStore.language))
      .join(' + ');
  });

  const pendingActivities = $derived.by(() =>
    [...contactActivities]
      .filter((activity) => activity.status !== 'completed')
      .sort((left, right) => activitySortTime(left) - activitySortTime(right))
  );

  const overdueActivities = $derived.by(() =>
    pendingActivities.filter((activity) => activity.status === 'overdue')
  );

  const nextActivity = $derived<Activity | null>(pendingActivities[0] ?? null);

  const recentActivity = $derived<Activity | null>(
    [...contactActivities]
      .sort((left, right) => activityUpdatedTime(right) - activityUpdatedTime(left))[0] ?? null
  );

  const contactActivityRelationships = $derived.by<Record<string, ActivityRelationshipLabels>>(() =>
    relationshipLabelsByActivityId(
      contactActivities,
      contactActivityLinkIndex,
      contactActivityLookups,
    )
  );

  const customerHealth = $derived.by(() => {
    if (dealsLoading || activitiesLoading) {
      return {
        tone: 'neutral',
        label: t('common.loading'),
        detail: t('contacts.workspace.healthLoadingDetail'),
      };
    }

    if (overdueActivities.length > 0) {
      const activity = overdueActivities[0];
      return {
        tone: 'danger',
        label: t('contacts.workspace.healthOverdue'),
        detail: t('contacts.workspace.healthOverdueDetail', { subject: activity.subject }),
      };
    }

    if (openDealCount > 0 && pendingActivities.length === 0) {
      return {
        tone: 'warning',
        label: t('contacts.workspace.healthNeedsFollowUp'),
        detail: t('contacts.workspace.healthNeedsFollowUpDetail'),
      };
    }

    if (nextActivity) {
      return {
        tone: 'success',
        label: t('contacts.workspace.healthOnTrack'),
        detail: t('contacts.workspace.healthOnTrackDetail', { subject: nextActivity.subject }),
      };
    }

    return {
      tone: 'neutral',
      label: t('contacts.workspace.healthNurture'),
      detail: t('contacts.workspace.healthNurtureDetail'),
    };
  });

  const nextStep = $derived(
    deriveRecordNextStep({
      recordKind: 'contact',
      isLoading: dealsLoading || activitiesLoading,
      isLead: contact?.type === 'person' && contact.lifecycle === 'lead',
      openDealCount,
      overdueActivities,
      nextActivity,
    }),
  );

  // ── Lifecycle ─────────────────────────────────────────────────────────────

  $effect(() => {
    if (!contactId || loadedContactId === contactId) {
      return;
    }

    loadedContactId = contactId;
    void loadContact();
  });

  $effect(() => {
    const version = activityStore.relationshipRefreshVersion;
    if (!contactId || !contact || lastActivityRefreshVersion === version) {
      return;
    }

    lastActivityRefreshVersion = version;
    if (version > 0) {
      void loadContactTimeline(contactId);
    }
  });

  async function loadContact() {
    isLoading = true;
    loadError = null;
    try {
      // Prefer the already-loaded selectedContact if ID matches
      let c = contactStore.selectedContact?.id === contactId
        ? contactStore.selectedContact
        : null;

      if (!c) {
        // Fallback: load from the contacts list
        c = contactStore.contacts.find((x) => x.id === contactId) ?? null;
      }

      if (!c) {
        await contactStore.loadContacts();
        c = contactStore.contacts.find((x) => x.id === contactId) ?? null;
      }

      if (!c) {
        c = await getContact(contactId);
        contactStore.selectContact(c);
      }

      contact = c;
      populateForm(c);

      // Load linked deals and activities in parallel
      dealsLoading = true;
      await Promise.all([
        dealStore.loadDeals({ contactId }),
        loadContactTimeline(contactId),
        loadCustomFields(contactId),
        organizationStore.organizations.length === 0
          ? organizationStore.loadOrganizations()
          : Promise.resolve(),
      ]);
      lastActivityRefreshVersion = activityStore.relationshipRefreshVersion;
    } catch (err) {
      loadError = t('errors.loadFailed');
      console.error('[ContactDetail] Load error:', err);
    } finally {
      isLoading = false;
      dealsLoading = false;
    }
  }

  async function loadContactTimeline(id: string) {
    activitiesLoading = true;
    try {
      const activities = await listActivities({
        sortBy: 'dueDate',
        sortDir: 'asc',
        pageSize: 500,
      });
      const linkIndex = await loadActivityLinkIndex(activities.map((activity) => activity.id));
      const lookups = await loadActivityRelationshipLookups();
      const matchedActivities = filterActivitiesByRelationship(activities, linkIndex, {
        contactId: id,
      });

      contactActivityLinkIndex = linkIndex;
      contactActivityLookups = lookups;
      contactActivities = sortActivitiesForDetailTimeline(matchedActivities);
    } catch (err) {
      console.error('[ContactDetail] Failed to load activity timeline:', err);
      contactActivityLinkIndex = {};
      contactActivityLookups = { contacts: [], organizations: [], deals: [] };
      contactActivities = [];
    } finally {
      activitiesLoading = false;
    }
  }

  function populateForm(c: Contact) {
    firstName    = c.firstName ?? '';
    lastName     = c.lastName ?? '';
    email        = c.email ?? '';
    phone        = c.phone ?? '';
    organization = c.organization ?? '';
    selectedOrgId = c.organizationId ?? '';
    website      = c.website ?? '';
    address      = c.address ?? '';
    contactType  = c.type;
    notes        = c.notes ?? '';
    owner        = c.owner ?? '';
    tags         = [...(c.tags ?? [])];
    isDirty      = false;
  }

  async function loadCustomFields(entityId: string) {
    customFieldsLoading = true;
    try {
      const [definitions, values] = await Promise.all([
        listCustomFieldDefinitions('contact'),
        listCustomFieldValues('contact', entityId),
      ]);

      customFieldDefinitions = definitions;

      const nextValues: Record<string, string> = Object.fromEntries(
        definitions.map((definition) => [definition.id, ''])
      );
      for (const entry of values) {
        nextValues[entry.field_def_id] = entry.value ?? '';
      }

      customFieldValues = nextValues;
      originalCustomFieldValues = { ...nextValues };
    } catch (err) {
      console.error('[ContactDetail] Failed to load custom fields:', err);
      uiStore.toastError(t('errors.loadCustomFields'));
      customFieldDefinitions = [];
      customFieldValues = {};
      originalCustomFieldValues = {};
    } finally {
      customFieldsLoading = false;
    }
  }

  async function persistCustomFields(entityId: string) {
    if (customFieldDefinitions.length === 0) {
      return;
    }

    await Promise.all(
      customFieldDefinitions.map((definition) =>
        setCustomFieldValue({
          fieldDefId: definition.id,
          entityId,
          value: customFieldValues[definition.id] ?? '',
        })
      )
    );
  }

  function handleCustomFieldChange(fieldDefId: string, value: string) {
    customFieldValues = {
      ...customFieldValues,
      [fieldDefId]: value,
    };
    isDirty = true;
  }

  function resetCustomFieldChanges() {
    customFieldValues = { ...originalCustomFieldValues };
  }

  function isOpenDeal(deal: Deal): boolean {
    return deal.stage !== 'closedWon' && deal.stage !== 'closedLost';
  }

  function activitySortTime(activity: Activity): number {
    const dueTime = Date.parse(activity.dueDate ?? '');
    if (Number.isFinite(dueTime)) {
      return dueTime;
    }

    const updatedTime = Date.parse(activity.updatedAt);
    return Number.isFinite(updatedTime) ? updatedTime : Number.MAX_SAFE_INTEGER;
  }

  function activityUpdatedTime(activity: Activity): number {
    const updatedTime = Date.parse(activity.updatedAt);
    if (Number.isFinite(updatedTime)) {
      return updatedTime;
    }

    const createdTime = Date.parse(activity.createdAt);
    return Number.isFinite(createdTime) ? createdTime : 0;
  }

  function formatActivityDue(activity: Activity | null): string {
    if (!activity) {
      return t('contacts.workspace.noneScheduled');
    }

    if (!activity.dueDate) {
      return t('contacts.workspace.noDueDate');
    }

    return formatDate(
      activity.dueDate,
      settingsStore.dateFormat as 'MMM D, YYYY',
      settingsStore.language,
    );
  }

  function openContactDealModal() {
    if (!contact) return;
    contactStore.selectContact(contact);
    uiStore.openModal('addDeal', { contactId: contact.id });
  }

  function openContactActivityModal() {
    if (!contact) return;
    contactStore.selectContact(contact);
    uiStore.openModal('addActivity', { contactId: contact.id });
  }

  // ── Handlers ────────────────────────────────────────────────────────────────

  function markDirty() {
    isDirty = true;
  }

  function handleOrgChange() {
    const org = organizationStore.organizations.find((o) => o.id === selectedOrgId);
    organization = org ? org.name : '';
    markDirty();
  }

  function validateForm(): boolean {
    emailError   = null;
    websiteError = null;

    if (email) {
      const r = validateEmail(email);
      if (!r.valid) { emailError = r.error ?? t('common.invalidEmail'); return false; }
    }
    if (website) {
      const r = validateUrl(website);
      if (!r.valid) { websiteError = r.error ?? t('common.invalidUrl'); return false; }
    }
    return true;
  }

  async function handleSave() {
    if (!contact || !isDirty) return;
    if (!validateForm()) return;

    isSaving = true;
    try {
      const payload: UpdateContactPayload = {
        firstName: firstName.trim() || undefined,
        lastName:  lastName.trim()  || undefined,
        email:     email.trim()     || null,
        phone:     phone.trim()     || null,
        organization: organization.trim() || null,
        website:   website.trim()   || null,
        address:   address.trim()   || null,
        type:      contactType,
        notes:     notes            || null,
        owner:     owner.trim()     || null,
        tags,
      };
      const updated = await contactStore.updateContact(contact.id, payload);
      await persistCustomFields(contact.id);
      contact = updated;
      const nextOrgId = selectedOrgId || null;
      if (nextOrgId !== updated.organizationId) {
        try {
          await organizationStore.linkContactToOrganization(updated.id, nextOrgId);
          contact = { ...updated, organizationId: nextOrgId };
        } catch (linkErr) {
          // error already toasted by store
          console.error('[ContactDetail] Failed to link organization:', linkErr);
        }
      }
      originalCustomFieldValues = { ...customFieldValues };
      isDirty = false;
    } catch (err) {
      // error already toasted by store
      console.error('[ContactDetail] Save error:', err);
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete() {
    if (!contact) return;
    isDeleting = true;
    try {
      await contactStore.deleteContact(contact.id);
      // Navigate back to contacts list
      navigateHash('/contacts');
    } catch (err) {
      console.error('[ContactDetail] Delete error:', err);
    } finally {
      isDeleting = false;
      showDeleteConfirm = false;
    }
  }

  function handleBack() {
    navigateHash('/contacts');
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

  async function handleNextStep() {
    if (nextStep.action === 'complete' && nextStep.activityId) {
      isSaving = true;
      try {
        await activityStore.markComplete(nextStep.activityId);
        await loadContactTimeline(contactId);
      } catch (err) {
        console.error('[ContactDetail] Complete next step error:', err);
      } finally {
        isSaving = false;
      }
      return;
    }

    if (nextStep.action === 'convert') {
      await handleConvertToCustomer();
      return;
    }

    if (nextStep.action === 'addFollowUp') {
      openContactActivityModal();
    }
  }

  async function handleConvertToCustomer() {
    if (!contact || contact.lifecycle !== 'lead') {
      return;
    }

    isSaving = true;
    try {
      const updated = await contactStore.setLifecycle(contact.id, 'customer');
      contact = updated;
      populateForm(updated);
    } catch (err) {
      console.error('[ContactDetail] Convert error:', err);
    } finally {
      isSaving = false;
    }
  }

  function handleNotesSave(newNotes: string) {
    notes = newNotes;
    isDirty = true;
    // Auto-save notes immediately
    handleSave();
  }

  async function handleComposeEmail() {
    const to = contact?.email?.trim();
    if (!to) {
      uiStore.toastError(t('common.invalidEmail'));
      return;
    }

    try {
      await composeEmail({
        to,
        subject: '',
      });
    } catch (err) {
      console.error('[ContactDetail] Failed to open email composer:', err);
      uiStore.toastError(t('settings.emailComposeFailed'));
    }
  }
</script>

<div class="page-content contact-detail-page">
  {#if isLoading}
    <!-- Loading skeleton -->
    <div class="detail-loading" aria-live="polite" aria-label={t('common.loading')}>
      <div class="skeleton skeleton-header"></div>
      <div class="skeleton-fields">
        {#each [1, 2, 3, 4] as i (i)}
          <div class="skeleton skeleton-field"></div>
        {/each}
      </div>
    </div>

  {:else if loadError}
    <div class="detail-error" role="alert">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <span>{loadError}</span>
      <button class="btn btn-secondary btn-sm" onclick={loadContact} type="button">
        {t('common.retry')}
      </button>
    </div>

  {:else if contact}
    <!-- ── Page header ──────────────────────────────────────────────────────── -->
    <div class="page-header">
      <div class="header-left">
        <button class="btn-back" onclick={handleBack} type="button" aria-label={t('common.back')}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M19 12H5M12 5l-7 7 7 7"/>
          </svg>
          {t('common.back')}
        </button>

        <div class="contact-identity">
          <div class="avatar avatar-lg avatar-color-{avatarColor}" aria-hidden="true">
            {initials}
          </div>
          <div class="identity-text">
            <h1 class="page-title">{displayName}</h1>
            <div class="identity-badges">
              <span class="contact-type-badge">{t(`contacts.${contact.type}`)}</span>
              {#if contact.type === 'person'}
                <span class="contact-lifecycle-badge lifecycle-{contact.lifecycle}">
                  {t(`contacts.lifecycle${contact.lifecycle === 'lead' ? 'Lead' : 'Customer'}`)}
                </span>
              {/if}
            </div>
          </div>
        </div>
      </div>

      <div class="header-actions">
        {#if isDirty}
          <button
            class="btn btn-primary btn-sm"
            onclick={handleSave}
            disabled={isSaving}
            type="button"
          >
            {isSaving ? t('common.loading') : t('common.save')}
          </button>
          <button
            class="btn btn-secondary btn-sm"
            onclick={() => {
              if (contact) {
                populateForm(contact);
              }
              resetCustomFieldChanges();
            }}
            disabled={isSaving}
            type="button"
          >
            {t('common.cancel')}
          </button>
        {/if}
        {#if contact.type === 'person' && contact.lifecycle === 'lead'}
          <button
            class="btn btn-primary btn-sm"
            onclick={handleConvertToCustomer}
            disabled={isSaving}
            type="button"
          >
            {t('contacts.convertToCustomer')}
          </button>
        {/if}
        {#if contact.email}
          <button
            class="btn btn-secondary btn-sm"
            onclick={handleComposeEmail}
            type="button"
            aria-label={t('activities.email')}
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <rect x="2" y="4" width="20" height="16" rx="2"/><path d="m22 7-10 7L2 7"/>
            </svg>
            {t('activities.email')}
          </button>
        {/if}
        <button
          class="btn btn-danger btn-sm"
          onclick={() => showDeleteConfirm = true}
          type="button"
          aria-label={t('contacts.deleteContact')}
        >
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6M10 11v6M14 11v6M9 6V4h6v2"/>
          </svg>
          {t('contacts.deleteContact')}
        </button>
      </div>
    </div>

    <!-- ── Customer workspace summary ──────────────────────────────────────── -->
    <section class="customer-workspace" aria-labelledby="customer-workspace-heading">
      <div class="customer-workspace-header">
        <div>
          <p class="workspace-eyebrow">
            {contact.lifecycle === 'lead'
              ? t('contacts.workspace.eyebrowLead')
              : t('contacts.workspace.eyebrow')}
          </p>
          <h2 class="section-title" id="customer-workspace-heading">
            {contact.lifecycle === 'lead'
              ? t('contacts.workspace.titleLead')
              : t('contacts.workspace.title')}
          </h2>
        </div>
        <span class="health-badge health-{customerHealth.tone}">
          {customerHealth.label}
        </span>
      </div>

      <p class="workspace-summary">{customerHealth.detail}</p>

      <NextStepCard step={nextStep} busy={isSaving} onaction={handleNextStep} />

      <div class="workspace-metrics" role="list">
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('contacts.workspace.openDeals')}</span>
          <strong>{openDealCount}</strong>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('contacts.workspace.openPipeline')}</span>
          <strong>{openPipelineValue}</strong>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('contacts.workspace.nextFollowUp')}</span>
          <strong>{nextActivity?.subject ?? t('contacts.workspace.noneScheduled')}</strong>
          <small>{formatActivityDue(nextActivity)}</small>
        </div>
        <div class="workspace-metric" role="listitem">
          <span class="workspace-metric-label">{t('contacts.workspace.recentActivity')}</span>
          <strong>{recentActivity?.subject ?? t('contacts.workspace.noRecentActivity')}</strong>
          <small>{recentActivity ? t(`activities.${recentActivity.type}`) : t('common.none')}</small>
        </div>
      </div>

      <div class="workspace-actions" aria-label={t('contacts.workspace.actionsLabel')}>
        <button class="btn btn-secondary btn-sm" onclick={openContactDealModal} type="button">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M12 5v14M5 12h14"/>
          </svg>
          {t('deals.addDeal')}
        </button>
        {#if shouldShowSecondaryFollowUp(nextStep)}
          <button class="btn btn-secondary btn-sm" onclick={openContactActivityModal} type="button">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <path d="M12 5v14M5 12h14"/>
            </svg>
            {t('contacts.workspace.addFollowUp')}
          </button>
        {/if}
      </div>
    </section>

    <!-- ── Main content grid ─────────────────────────────────────────────────── -->
    <div class="detail-grid">

      <!-- LEFT COLUMN: editable fields + generic notes/tags + legacy notes/tags -->
      <div class="detail-main">

        <!-- Core fields card -->
        <section class="card detail-fields" aria-labelledby="fields-heading">
          <div class="card-header">
            <h2 class="section-title" id="fields-heading">{t('contacts.editContact')}</h2>
          </div>
          <div class="card-body">
            <div class="fields-grid">

              <!-- First / Last name -->
              <div class="field-group">
                <label class="field-label" for="firstName">{t('contacts.firstName')}</label>
                <input
                  id="firstName"
                  class="input"
                  type="text"
                  bind:value={firstName}
                  oninput={markDirty}
                  autocomplete="given-name"
                />
              </div>
              <div class="field-group">
                <label class="field-label" for="lastName">{t('contacts.lastName')}</label>
                <input
                  id="lastName"
                  class="input"
                  type="text"
                  bind:value={lastName}
                  oninput={markDirty}
                  autocomplete="family-name"
                />
              </div>

              <!-- Email -->
              <div class="field-group field-group--full">
                <label class="field-label" for="email">{t('contacts.email')}</label>
                <input
                  id="email"
                  class="input"
                  class:input-error={!!emailError}
                  type="email"
                  bind:value={email}
                  oninput={() => { markDirty(); emailError = null; }}
                  autocomplete="email"
                />
                {#if emailError}
                  <p class="field-error">{emailError}</p>
                {/if}
              </div>

              <!-- Phone -->
              <div class="field-group">
                <label class="field-label" for="phone">{t('contacts.phone')}</label>
                <input
                  id="phone"
                  class="input"
                  type="tel"
                  bind:value={phone}
                  oninput={markDirty}
                  autocomplete="tel"
                />
              </div>

              <!-- Organization -->
              <div class="field-group">
                <label class="field-label" for="organization">{t('contacts.organization')}</label>
                <select
                  id="organization"
                  class="input"
                  bind:value={selectedOrgId}
                  onchange={handleOrgChange}
                  disabled={organizationStore.isLoading}
                >
                  <option value="">{t('common.none')}</option>
                  {#each organizationStore.organizations as org (org.id)}
                    <option value={org.id}>{org.name}</option>
                  {/each}
                </select>
              </div>

              <!-- Website -->
              <div class="field-group">
                <label class="field-label" for="website">{t('contacts.website')}</label>
                <input
                  id="website"
                  class="input"
                  class:input-error={!!websiteError}
                  type="url"
                  bind:value={website}
                  oninput={() => { markDirty(); websiteError = null; }}
                  placeholder="https://"
                  autocomplete="url"
                />
                {#if websiteError}
                  <p class="field-error">{websiteError}</p>
                {/if}
              </div>

              <!-- Address -->
              <div class="field-group field-group--full">
                <label class="field-label" for="address">{t('contacts.address')}</label>
                <textarea
                  id="address"
                  class="input textarea"
                  bind:value={address}
                  oninput={markDirty}
                  rows="2"
                  autocomplete="street-address"
                ></textarea>
              </div>

              <!-- Owner -->
              <div class="field-group">
                <label class="field-label" for="contact-owner">{t('common.owner')}</label>
                <input
                  id="contact-owner"
                  class="input"
                  type="text"
                  bind:value={owner}
                  oninput={markDirty}
                  placeholder={t('common.optional')}
                  autocomplete="name"
                />
              </div>

              <!-- Type -->
              <div class="field-group">
                <span class="field-label" id="type-label">{t('contacts.type')}</span>
                <div class="radio-group" role="radiogroup" aria-labelledby="type-label">
                  <label class="radio-option">
                    <input
                      type="radio"
                      name="contactType"
                      value="person"
                      bind:group={contactType}
                      onchange={markDirty}
                    />
                    <span>{t('contacts.person')}</span>
                  </label>
                  <label class="radio-option">
                    <input
                      type="radio"
                      name="contactType"
                      value="org"
                      bind:group={contactType}
                      onchange={markDirty}
                    />
                    <span>{t('contacts.org')}</span>
                  </label>
                </div>
              </div>
            </div>

            <!-- Metadata footer -->
            <div class="fields-meta">
              <span>
                {t('contacts.created')}:
                {formatDate(contact.createdAt, settingsStore.dateFormat as 'MMM D, YYYY')}
              </span>
              <span>
                {t('contacts.updated')}:
                {formatRelativeTime(contact.updatedAt)}
              </span>
            </div>
          </div>
        </section>

        <!-- Custom fields card -->
        <section class="card detail-custom-fields" aria-labelledby="custom-fields-heading">
          <div class="card-header">
            <h2 class="section-title" id="custom-fields-heading">{t('common.customFields')}</h2>
          </div>
          <div class="card-body">
            {#if customFieldsLoading}
              <p class="custom-fields-placeholder">{t('common.loading')}</p>
            {:else if customFieldDefinitions.length === 0}
              <p class="custom-fields-placeholder">{t('common.noCustomFields')}</p>
            {:else}
              <CustomFieldInputs
                definitions={customFieldDefinitions}
                values={customFieldValues}
                onchange={handleCustomFieldChange}
                disabled={isSaving}
              />
            {/if}
          </div>
        </section>

        <!-- Generic tags card -->
        <section class="card detail-entity-tags" aria-labelledby="entity-tags-heading">
          <div class="card-header">
            <h2 class="section-title" id="entity-tags-heading">{t('contacts.tags')}</h2>
          </div>
          <div class="card-body">
            <EntityTagsPanel entityType="contact" entityId={contact.id} />
          </div>
        </section>

        <!-- Generic notes card -->
        <section class="card detail-entity-notes" aria-labelledby="entity-notes-heading">
          <div class="card-header">
            <h2 class="section-title" id="entity-notes-heading">{t('contacts.notes')}</h2>
          </div>
          <div class="card-body">
            <EntityNotesPanel entityType="contact" entityId={contact.id} />
          </div>
        </section>

        <section class="card detail-entity-links">
          <div class="card-body">
            <EntityLinksPanel entityType="contact" entityId={contact.id} />
          </div>
        </section>

        <!-- Legacy notes card -->
        <section class="card detail-notes" aria-labelledby="legacy-notes-heading">
          <div class="card-header">
            <h2 class="section-title" id="legacy-notes-heading">{t('contacts.legacyNotes')}</h2>
          </div>
          <div class="card-body">
            <NoteEditor
              value={notes}
              onsave={handleNotesSave}
            />
          </div>
        </section>
      </div>

      <!-- RIGHT COLUMN: linked deals + activity timeline -->
      <div class="detail-sidebar">

        <!-- Linked deals -->
        <section class="card detail-deals" aria-labelledby="deals-heading">
          <div class="card-header">
            <h2 class="section-title" id="deals-heading">{t('contacts.linkedDeals')}</h2>
            <button
              class="btn btn-secondary btn-xs"
              onclick={openContactDealModal}
              type="button"
            >
              <svg width="10" height="10" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                <path d="M6 1v10M1 6h10"/>
              </svg>
              {t('deals.addDeal')}
            </button>
          </div>
          <div class="card-body">
            {#if dealsLoading}
              <div class="sidebar-loading" aria-label={t('common.loading')}>
                {#each [1, 2] as i (i)}
                  <div class="skeleton skeleton-deal-row"></div>
                {/each}
              </div>
            {:else if linkedDeals.length === 0}
              <EmptyState
                icon="deals"
                title={t('deals.noDeals')}
                description={t('deals.noDealsDesc')}
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

        <!-- Activity timeline -->
        <section class="card detail-activity" aria-labelledby="activity-heading">
          <div class="card-header">
            <h2 class="section-title" id="activity-heading">{t('contacts.activityTimeline')}</h2>
            <button
              class="btn btn-secondary btn-xs"
              onclick={openContactActivityModal}
              type="button"
            >
              <svg width="10" height="10" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                <path d="M6 1v10M1 6h10"/>
              </svg>
              {t('activities.addActivity')}
            </button>
          </div>
          <div class="card-body">
            <LogJustHappened
              contactId={contactId}
              onlogged={() => void loadContactTimeline(contactId)}
            />
            <ActivityFeed
              activities={contactActivities}
              loading={activitiesLoading}
              maxItems={10}
              relationshipsByActivityId={contactActivityRelationships}
              showRelationshipBreadcrumbs={true}
              onNavigateEntity={handleActivityEntityNavigate}
            />
          </div>
        </section>
      </div>
    </div>
  {/if}
</div>

<!-- ── Delete confirmation modal ─────────────────────────────────────────────── -->
<Modal bind:open={showDeleteConfirm} title={t('contacts.deleteContact')} size="sm">
  {#snippet body()}
    <p class="confirm-message">{t('contacts.confirmDelete')}</p>
  {/snippet}
  {#snippet footer()}
    <div class="modal-footer-actions">
      <button
        class="btn btn-secondary"
        onclick={() => showDeleteConfirm = false}
        disabled={isDeleting}
        type="button"
      >
        {t('common.cancel')}
      </button>
      <button
        class="btn btn-danger"
        onclick={handleDelete}
        disabled={isDeleting}
        type="button"
      >
        {isDeleting ? t('common.loading') : t('common.delete')}
      </button>
    </div>
  {/snippet}
</Modal>

<style>
  /* ── Layout ─────────────────────────────────────────────────────────────── */

  .contact-detail-page {
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

  /* ── Back button ─────────────────────────────────────────────────────────── */

  .btn-back {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-1) 0;
    transition: color var(--duration-fast) var(--ease-out);
  }

  .btn-back:hover {
    color: var(--text-accent);
  }

  /* ── Contact identity ────────────────────────────────────────────────────── */

  .contact-identity {
    display: flex;
    align-items: center;
    gap: var(--space-5);
  }

  .identity-text {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .identity-badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .contact-type-badge,
  .contact-lifecycle-badge {
    display: inline-block;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    background-color: var(--surface-raised);
    border-radius: 9999px;
    padding: 2px var(--space-3);
    border: var(--border-width) solid var(--border-default);
  }

  .lifecycle-lead {
    color: #20808D;
    background: #E8F4F7;
    border-color: #B7DEE4;
  }

  .lifecycle-customer {
    color: #2D8659;
    background: #E8F5EC;
    border-color: #B7DCC6;
  }

  /* ── Avatar ──────────────────────────────────────────────────────────────── */

  .avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: var(--weight-semibold);
    border-radius: 50%;
    flex-shrink: 0;
    user-select: none;
    letter-spacing: 0.02em;
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

  /* ── Detail grid ─────────────────────────────────────────────────────────── */

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 320px;
    gap: var(--space-6);
    align-items: start;
  }

  @media (max-width: 900px) {
    .detail-grid {
      grid-template-columns: 1fr;
    }
  }

  .detail-main,
  .detail-sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  /* ── Customer workspace summary ─────────────────────────────────────────── */

  .customer-workspace {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-lg);
    background-color: var(--surface-card);
  }

  .customer-workspace-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-4);
  }

  .workspace-eyebrow {
    margin: 0 0 var(--space-1);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
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
    text-overflow: ellipsis;
    color: var(--text-primary);
    font-size: var(--text-base);
    line-height: 1.35;
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
    border-radius: 9999px;
    border: var(--border-width) solid var(--border-default);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .health-neutral {
    color: var(--text-secondary);
    background-color: var(--surface-raised);
  }

  .health-success {
    color: #2D8659;
    background-color: #E8F5EC;
    border-color: #BFE4CC;
  }

  .health-warning {
    color: #A84B2F;
    background-color: #FEF3E2;
    border-color: #F4D1A1;
  }

  .health-danger {
    color: #C0392B;
    background-color: #FFF0F0;
    border-color: #F0B8B2;
  }

  @media (max-width: 900px) {
    .workspace-metrics {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 640px) {
    .customer-workspace-header {
      flex-direction: column;
    }

    .workspace-metrics {
      grid-template-columns: 1fr;
    }

    .workspace-actions .btn {
      width: 100%;
      justify-content: center;
    }
  }

  /* ── Fields grid ─────────────────────────────────────────────────────────── */

  .fields-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-5);
  }

  @media (max-width: 640px) {
    .fields-grid {
      grid-template-columns: 1fr;
    }
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field-group--full {
    grid-column: 1 / -1;
  }

  .field-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .textarea {
    resize: vertical;
    min-height: 64px;
    font-family: inherit;
  }

  .custom-fields-placeholder {
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }

  .input-error {
    border-color: var(--color-danger) !important;
  }

  .field-error {
    font-size: var(--text-xs);
    color: var(--color-danger);
    margin: 0;
  }

  .radio-group {
    display: flex;
    gap: var(--space-5);
  }

  .radio-option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    cursor: pointer;
    color: var(--text-secondary);
  }

  .fields-meta {
    display: flex;
    gap: var(--space-6);
    margin-block-start: var(--space-5);
    padding-block-start: var(--space-4);
    border-block-start: var(--border-width) solid var(--border-default);
    font-size: var(--text-xs);
    color: var(--color-text-muted, var(--text-secondary));
  }

  /* ── Deals list ──────────────────────────────────────────────────────────── */

  .deals-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .deal-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: 0;
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-default);
  }

  .deal-row-button {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-3) var(--space-4);
    border: 0;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: inherit;
  }

  .deal-row-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .deal-row-name {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .deal-row-value {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    flex-shrink: 0;
  }

  /* Stage badges */
  .stage-badge {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    border-radius: 9999px;
    padding: 2px var(--space-2);
  }

  .stage-lead         { background: #E8F4F7; color: #20808D; }
  .stage-qualified    { background: #E8F5EC; color: #2D8659; }
  .stage-proposal     { background: #FFF8E1; color: #D4A017; }
  .stage-negotiation  { background: #FEF3E2; color: #A84B2F; }
  .stage-closedWon    { background: #E8F5EC; color: #2D8659; }
  .stage-closedLost   { background: #FFF0F0; color: #C0392B; }

  /* ── Loading states ──────────────────────────────────────────────────────── */

  .detail-loading {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .skeleton-header {
    height: 64px;
    border-radius: var(--radius-lg);
  }

  .skeleton-fields {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  .skeleton-field {
    height: 56px;
    border-radius: var(--radius-md);
  }

  .skeleton-deal-row {
    height: 44px;
    border-radius: var(--radius-md);
    margin-block-end: var(--space-2);
  }

  .sidebar-loading {
    padding: var(--space-2);
  }

  /* ── Error state ─────────────────────────────────────────────────────────── */

  .detail-error {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-5) var(--space-6);
    background-color: var(--color-danger-50, #FFF0F0);
    color: var(--color-danger);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }

  /* ── Delete confirmation ─────────────────────────────────────────────────── */

  .confirm-message {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .modal-footer-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
  }

  /* ── Danger button ───────────────────────────────────────────────────────── */

  :global(.btn-danger) {
    background-color: var(--color-danger);
    color: #fff;
    border-color: var(--color-danger);
  }

  :global(.btn-danger:hover:not(:disabled)) {
    background-color: #a93226;
    border-color: #a93226;
  }

  :global(.btn-xs) {
    padding: var(--space-1) var(--space-3);
    font-size: var(--text-xs);
  }
</style>
