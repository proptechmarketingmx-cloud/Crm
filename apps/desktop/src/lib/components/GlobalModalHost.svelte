<script lang="ts">
  import Modal from '$lib/components/Modal.svelte';
  import { t } from '$lib/i18n';
  import { uiStore } from '$lib/stores/ui';
  import { contactStore } from '$lib/stores/contacts';
  import { organizationStore } from '$lib/stores/organizations';
  import { dealStore } from '$lib/stores/deals';
  import { activityStore } from '$lib/stores/activities';
  import { settingsStore } from '$lib/stores/settings';
  import { DEAL_STAGES } from '$lib/api/deals';
  import type { Deal, DealStage } from '$lib/api/deals';
  import type { Contact, ContactLifecycle } from '$lib/api/contacts';
  import { ACTIVITY_TYPES, type ActivityType } from '$lib/api/activities';
  import { combineDueDateAndTime, splitDueDateAndTime } from '$lib/utils/activityDue';
  import type { Organization } from '$lib/api/organizations';
  import { normalizeCurrencyCode } from '$lib/utils/currency';
  import {
    contactDisplayName,
    loadDealRelationshipLookups,
  } from '$lib/utils/dealRelationships';
  import {
    addSelectedActivityLinks,
    loadActivityRelationshipLookups,
  } from '$lib/utils/activityRelationships';
  import {
    listCustomFieldDefinitions,
    setCustomFieldValue,
    type CustomFieldDefinition,
    type CustomFieldEntityType,
  } from '$lib/api/customFields';
  import CustomFieldInputs from '$lib/components/CustomFieldInputs.svelte';

  let isSavingContact = $state(false);
  let contactFirstName = $state('');
  let contactLastName = $state('');
  let contactEmail = $state('');
  let contactPhone = $state('');
  let contactOrganization = $state('');
  let contactType = $state<'person' | 'org'>('person');
  let contactLifecycle = $state<ContactLifecycle>('lead');
  let contactOwner = $state('');

  let isSavingDeal = $state(false);
  let dealName = $state('');
  let dealValue = $state<number>(0);
  let dealCurrency = $state('USD');
  let dealStage = $state<DealStage>('lead');
  let dealProbability = $state<number>(10);
  let dealExpectedCloseDate = $state('');
  let dealContactId = $state('');
  let dealOrganizationId = $state('');
  let dealDescription = $state('');
  let dealOwner = $state('');
  let dealRelationshipContacts = $state<Contact[]>([]);
  let dealRelationshipOrganizations = $state<Organization[]>([]);
  let loadingDealRelationships = $state(false);

  let isSavingActivity = $state(false);
  let activitySubject = $state('');
  let activityType = $state<ActivityType>('task');
  let activityDueDate = $state('');
  let activityDueTime = $state('');
  let activityNotes = $state('');
  let activityContactId = $state('');
  let activityOrganizationId = $state('');
  let activityDealId = $state('');
  let activityRelationshipContacts = $state<Contact[]>([]);
  let activityRelationshipOrganizations = $state<Organization[]>([]);
  let activityRelationshipDeals = $state<Deal[]>([]);
  let loadingActivityRelationships = $state(false);

  let contactCustomFieldDefs = $state<CustomFieldDefinition[]>([]);
  let contactCustomFieldValues = $state<Record<string, string>>({});
  let loadingContactCustomFields = $state(false);

  let dealCustomFieldDefs = $state<CustomFieldDefinition[]>([]);
  let dealCustomFieldValues = $state<Record<string, string>>({});
  let loadingDealCustomFields = $state(false);

  let activityCustomFieldDefs = $state<CustomFieldDefinition[]>([]);
  let activityCustomFieldValues = $state<Record<string, string>>({});
  let loadingActivityCustomFields = $state(false);

  let lastModal = $state<string | null>(null);

  const dealContactOptions = $derived.by(() => {
    const selected = contactStore.selectedContact;
    if (!selected || selected.id !== dealContactId) {
      return dealRelationshipContacts;
    }
    if (dealRelationshipContacts.some((contact) => contact.id === selected.id)) {
      return dealRelationshipContacts;
    }
    return [selected, ...dealRelationshipContacts];
  });

  const dealOrganizationOptions = $derived.by(() => {
    const selected = organizationStore.selectedOrganization;
    if (!selected || selected.id !== dealOrganizationId) {
      return dealRelationshipOrganizations;
    }
    if (dealRelationshipOrganizations.some((organization) => organization.id === selected.id)) {
      return dealRelationshipOrganizations;
    }
    return [selected, ...dealRelationshipOrganizations];
  });

  const activityContactOptions = $derived.by(() => {
    const selected = contactStore.selectedContact;
    if (!selected || selected.id !== activityContactId) {
      return activityRelationshipContacts;
    }
    if (activityRelationshipContacts.some((contact) => contact.id === selected.id)) {
      return activityRelationshipContacts;
    }
    return [selected, ...activityRelationshipContacts];
  });

  const activityOrganizationOptions = $derived.by(() => {
    const selected = organizationStore.selectedOrganization;
    if (!selected || selected.id !== activityOrganizationId) {
      return activityRelationshipOrganizations;
    }
    if (activityRelationshipOrganizations.some((organization) => organization.id === selected.id)) {
      return activityRelationshipOrganizations;
    }
    return [selected, ...activityRelationshipOrganizations];
  });

  const activityDealOptions = $derived.by(() => {
    const selected = dealStore.selectedDeal;
    if (!selected || selected.id !== activityDealId) {
      return activityRelationshipDeals;
    }
    if (activityRelationshipDeals.some((deal) => deal.id === selected.id)) {
      return activityRelationshipDeals;
    }
    return [selected, ...activityRelationshipDeals];
  });

  function modalDataString(key: string): string {
    const value = uiStore.modalData?.[key];
    return typeof value === 'string' ? value : '';
  }

  function modalDataActivityType(key: string): ActivityType {
    const value = modalDataString(key);
    return (ACTIVITY_TYPES as readonly string[]).includes(value)
      ? (value as ActivityType)
      : 'task';
  }

  function normalizeStage(stage: string): DealStage {
    const map: Record<string, DealStage> = {
      lead: 'lead',
      qualified: 'qualified',
      proposal: 'proposal',
      negotiation: 'negotiation',
      closedwon: 'closedWon',
      'closed won': 'closedWon',
      closedlost: 'closedLost',
      'closed lost': 'closedLost',
    };

    return map[stage.toLowerCase().trim()] ?? 'lead';
  }

  function defaultProbability(stage: DealStage): number {
    const map: Record<DealStage, number> = {
      lead: 10,
      qualified: 25,
      proposal: 50,
      negotiation: 75,
      closedWon: 100,
      closedLost: 0,
    };

    return map[stage];
  }

  function resetContactForm() {
    contactFirstName = '';
    contactLastName = '';
    contactEmail = '';
    contactPhone = '';
    contactOrganization = '';
    contactType = 'person';
    contactLifecycle = modalDataString('lifecycle') === 'lead' ? 'lead' : 'customer';
    contactOwner = '';
  }

  function resetDealForm() {
    dealName = '';
    dealValue = 0;
    dealCurrency = normalizeCurrencyCode(settingsStore.currency || 'USD');
    dealStage = normalizeStage(modalDataString('stage'));
    dealProbability = defaultProbability(dealStage);
    dealExpectedCloseDate = '';
    dealContactId = modalDataString('contactId');
    dealOrganizationId = modalDataString('organizationId');
    dealDescription = '';
    dealOwner = '';
  }

  function handleDealCurrencyInput(event: Event) {
    const value = (event.target as HTMLInputElement).value;
    dealCurrency = value.toUpperCase().slice(0, 3);
  }

  function resetActivityForm() {
    activitySubject = modalDataString('subject');
    activityType = modalDataActivityType('type');
    const dueParts = splitDueDateAndTime(modalDataString('dueDate'));
    activityDueDate = dueParts.date;
    activityDueTime = dueParts.time;
    activityNotes = modalDataString('notes');
    activityContactId = modalDataString('contactId');
    activityOrganizationId = modalDataString('organizationId');
    activityDealId = modalDataString('dealId');
  }

  function updateCustomFieldValue(
    entityType: CustomFieldEntityType,
    fieldDefId: string,
    value: string,
  ) {
    if (entityType === 'contact') {
      contactCustomFieldValues = { ...contactCustomFieldValues, [fieldDefId]: value };
      return;
    }

    if (entityType === 'deal') {
      dealCustomFieldValues = { ...dealCustomFieldValues, [fieldDefId]: value };
      return;
    }

    activityCustomFieldValues = { ...activityCustomFieldValues, [fieldDefId]: value };
  }

  async function loadCustomFieldDefinitions(entityType: CustomFieldEntityType): Promise<CustomFieldDefinition[]> {
    try {
      return await listCustomFieldDefinitions(entityType);
    } catch (err) {
      console.error(`[GlobalModalHost] Failed to load custom fields for ${entityType}:`, err);
      uiStore.toastError(t('errors.loadCustomFields'));
      return [];
    }
  }

  function blankCustomFieldValues(definitions: CustomFieldDefinition[]): Record<string, string> {
    return Object.fromEntries(definitions.map((definition) => [definition.id, '']));
  }

  async function prepareContactCustomFields() {
    loadingContactCustomFields = true;
    try {
      contactCustomFieldDefs = await loadCustomFieldDefinitions('contact');
      contactCustomFieldValues = blankCustomFieldValues(contactCustomFieldDefs);
    } finally {
      loadingContactCustomFields = false;
    }
  }

  async function prepareDealCustomFields() {
    loadingDealCustomFields = true;
    try {
      dealCustomFieldDefs = await loadCustomFieldDefinitions('deal');
      dealCustomFieldValues = blankCustomFieldValues(dealCustomFieldDefs);
    } finally {
      loadingDealCustomFields = false;
    }
  }

  async function prepareDealRelationships() {
    loadingDealRelationships = true;
    try {
      const lookups = await loadDealRelationshipLookups();
      dealRelationshipContacts = lookups.contacts;
      dealRelationshipOrganizations = lookups.organizations;
    } catch (err) {
      console.error('[GlobalModalHost] Failed to load deal relationships:', err);
      uiStore.toastError(t('errors.loadRelationships', { name: t('entities.deal') }));
    } finally {
      loadingDealRelationships = false;
    }
  }

  async function prepareActivityCustomFields() {
    loadingActivityCustomFields = true;
    try {
      activityCustomFieldDefs = await loadCustomFieldDefinitions('activity');
      activityCustomFieldValues = blankCustomFieldValues(activityCustomFieldDefs);
    } finally {
      loadingActivityCustomFields = false;
    }
  }

  async function prepareActivityRelationships() {
    loadingActivityRelationships = true;
    try {
      const lookups = await loadActivityRelationshipLookups();
      activityRelationshipContacts = lookups.contacts;
      activityRelationshipOrganizations = lookups.organizations;
      activityRelationshipDeals = lookups.deals;
    } catch (err) {
      console.error('[GlobalModalHost] Failed to load activity relationships:', err);
      uiStore.toastError(t('errors.loadRelationships', { name: t('entities.activity') }));
    } finally {
      loadingActivityRelationships = false;
    }
  }

  async function persistCustomFields(
    entityId: string,
    values: Record<string, string>,
  ) {
    const updates = Object.entries(values)
      .filter(([, value]) => value.trim().length > 0)
      .map(([fieldDefId, value]) =>
        setCustomFieldValue({
          fieldDefId,
          entityId,
          value,
        })
      );

    if (updates.length === 0) {
      return;
    }

    await Promise.all(updates);
  }

  $effect(() => {
    const modal = uiStore.activeModal;
    if (modal === lastModal) {
      return;
    }

    lastModal = modal;

    if (modal === 'addContact') {
      resetContactForm();
      void prepareContactCustomFields();
    }

    if (modal === 'addDeal') {
      resetDealForm();
      void prepareDealCustomFields();
      void prepareDealRelationships();
    }

    if (modal === 'addActivity') {
      resetActivityForm();
      void prepareActivityCustomFields();
      void prepareActivityRelationships();
    }
  });

  async function submitContact() {
    if (!contactFirstName.trim()) {
      uiStore.toastError(t('contacts.firstNameRequired'));
      return;
    }

    isSavingContact = true;
    try {
      const contact = await contactStore.createContact({
        firstName: contactFirstName.trim(),
        lastName: contactLastName.trim(),
        email: contactEmail.trim() || null,
        phone: contactPhone.trim() || null,
        organization: contactOrganization.trim() || null,
        type: contactType,
        lifecycle: contactType === 'person' ? contactLifecycle : 'customer',
        tags: [],
        notes: null,
        website: null,
        address: null,
        owner: contactOwner.trim() || null,
      });
      uiStore.closeModal();
      try {
        await persistCustomFields(contact.id, contactCustomFieldValues);
      } catch (cfErr) {
        console.error('[GlobalModalHost] Failed to save contact custom fields:', cfErr);
        uiStore.toastError(t('contacts.customFieldsSaveFailed'));
      }
    } catch (err) {
      console.error('[GlobalModalHost] Failed to create contact:', err);
      uiStore.toastError(t('errors.saveFailed'));
    } finally {
      isSavingContact = false;
    }
  }

  async function submitDeal() {
    if (!dealName.trim()) {
      uiStore.toastError(t('deals.nameRequired'));
      return;
    }

    isSavingDeal = true;
    try {
      const deal = await dealStore.createDeal({
        name: dealName.trim(),
        value: Number.isFinite(dealValue) ? dealValue : 0,
        currency: normalizeCurrencyCode(dealCurrency || settingsStore.currency || 'USD'),
        stage: dealStage,
        probability: dealProbability,
        expectedCloseDate: dealExpectedCloseDate || null,
        contactId: dealContactId || null,
        organizationId: dealOrganizationId || null,
        description: dealDescription.trim() || null,
        tags: [],
        owner: dealOwner.trim() || null,
      });
      uiStore.closeModal();
      try {
        await persistCustomFields(deal.id, dealCustomFieldValues);
      } catch (cfErr) {
        console.error('[GlobalModalHost] Failed to save deal custom fields:', cfErr);
        uiStore.toastError(t('deals.customFieldsSaveFailed'));
      }
    } catch (err) {
      console.error('[GlobalModalHost] Failed to create deal:', err);
      uiStore.toastError(t('errors.saveFailed'));
    } finally {
      isSavingDeal = false;
    }
  }

  async function submitActivity() {
    if (!activitySubject.trim()) {
      uiStore.toastError(t('activities.subjectRequired'));
      return;
    }

    isSavingActivity = true;
    try {
      const activity = await activityStore.createActivity({
        subject: activitySubject.trim(),
        type: activityType,
        dueDate: combineDueDateAndTime(activityDueDate, activityDueTime),
        notes: activityNotes.trim() || null,
        contactId: activityContactId || null,
        dealId: activityDealId || null,
      });
      uiStore.closeModal();
      await addSelectedActivityLinks(activity.id, {
        contactId: activityContactId || null,
        organizationId: activityOrganizationId || null,
        dealId: activityDealId || null,
      });
      activityStore.notifyRelationshipLinksChanged();
      try {
        await persistCustomFields(activity.id, activityCustomFieldValues);
      } catch (cfErr) {
        console.error('[GlobalModalHost] Failed to save activity custom fields:', cfErr);
        uiStore.toastError(t('activities.customFieldsSaveFailed'));
      }
    } catch (err) {
      console.error('[GlobalModalHost] Failed to create activity:', err);
      uiStore.toastError(t('errors.saveFailed'));
    } finally {
      isSavingActivity = false;
    }
  }
</script>

{#if uiStore.activeModal === 'addContact'}
  <Modal open={true} title={t('contacts.addContact')} size="md" onclose={() => uiStore.closeModal()}>
    {#snippet body()}
      <div class="form-grid">
        <div class="form-group">
          <label class="form-label" for="modal-contact-first-name">{t('contacts.firstName')}</label>
          <input id="modal-contact-first-name" class="input" bind:value={contactFirstName} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-contact-last-name">{t('contacts.lastName')}</label>
          <input id="modal-contact-last-name" class="input" bind:value={contactLastName} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-contact-email">{t('contacts.email')}</label>
          <input id="modal-contact-email" class="input" type="email" bind:value={contactEmail} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-contact-phone">{t('contacts.phone')}</label>
          <input id="modal-contact-phone" class="input" bind:value={contactPhone} />
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-contact-organization">{t('contacts.organization')}</label>
          <input id="modal-contact-organization" class="input" bind:value={contactOrganization} />
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-contact-type">{t('contacts.type')}</label>
          <select id="modal-contact-type" class="select" bind:value={contactType}>
            <option value="person">{t('contacts.person')}</option>
            <option value="org">{t('contacts.org')}</option>
          </select>
        </div>
        {#if contactType === 'person'}
          <div class="form-group form-group--full">
            <label class="form-label" for="modal-contact-lifecycle">{t('contacts.lifecycle')}</label>
            <select id="modal-contact-lifecycle" class="select" bind:value={contactLifecycle}>
              <option value="lead">{t('contacts.lifecycleLead')}</option>
              <option value="customer">{t('contacts.lifecycleCustomer')}</option>
            </select>
          </div>
        {/if}
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-contact-owner">{t('common.owner')}</label>
          <input id="modal-contact-owner" class="input" bind:value={contactOwner} />
        </div>
      </div>
      {#if loadingContactCustomFields}
        <p class="custom-field-loading">{t('common.loading')}</p>
      {:else}
        <CustomFieldInputs
          definitions={contactCustomFieldDefs}
          values={contactCustomFieldValues}
          onchange={(fieldDefId, value) => updateCustomFieldValue('contact', fieldDefId, value)}
          disabled={isSavingContact}
        />
      {/if}
    {/snippet}
    {#snippet footer()}
      <button class="btn btn-secondary" type="button" onclick={() => uiStore.closeModal()}>{t('common.cancel')}</button>
      <button class="btn btn-primary" type="button" onclick={submitContact} disabled={isSavingContact}>
        {isSavingContact ? t('common.loading') : t('common.save')}
      </button>
    {/snippet}
  </Modal>
{/if}

{#if uiStore.activeModal === 'addDeal'}
  <Modal open={true} title={t('deals.addDeal')} size="md" onclose={() => uiStore.closeModal()}>
    {#snippet body()}
      <div class="form-grid">
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-deal-name">{t('deals.name')}</label>
          <input id="modal-deal-name" class="input" bind:value={dealName} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-deal-value">{t('deals.value')}</label>
          <input id="modal-deal-value" class="input" type="number" min="0" step="0.01" bind:value={dealValue} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-deal-currency">{t('settings.currency')}</label>
          <input
            id="modal-deal-currency"
            class="input"
            value={dealCurrency}
            oninput={handleDealCurrencyInput}
            maxlength="3"
            autocapitalize="characters"
            spellcheck="false"
          />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-deal-stage">{t('deals.stage')}</label>
          <select id="modal-deal-stage" class="select" bind:value={dealStage} onchange={() => dealProbability = defaultProbability(dealStage)}>
            {#each DEAL_STAGES as stage (stage)}
              <option value={stage}>{t(`deals.stages.${stage}`)}</option>
            {/each}
          </select>
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-deal-probability">{t('deals.probability')}</label>
          <input id="modal-deal-probability" class="input" type="number" min="0" max="100" bind:value={dealProbability} />
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-deal-close-date">{t('deals.expectedClose')}</label>
          <input id="modal-deal-close-date" class="input" type="date" bind:value={dealExpectedCloseDate} />
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-deal-owner">{t('common.owner')}</label>
          <input id="modal-deal-owner" class="input" bind:value={dealOwner} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-deal-organization">{t('contacts.organization')}</label>
          <select
            id="modal-deal-organization"
            class="select"
            bind:value={dealOrganizationId}
            disabled={loadingDealRelationships || isSavingDeal}
            aria-busy={loadingDealRelationships}
          >
            {#if loadingDealRelationships && dealOrganizationId}
              <option value={dealOrganizationId}>{t('common.loading')}</option>
            {/if}
            <option value="">{loadingDealRelationships ? t('common.loading') : t('common.none')}</option>
            {#each dealOrganizationOptions as organization (organization.id)}
              <option value={organization.id}>{organization.name}</option>
            {/each}
          </select>
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-deal-contact">{t('deals.contact')}</label>
          <select
            id="modal-deal-contact"
            class="select"
            bind:value={dealContactId}
            disabled={loadingDealRelationships || isSavingDeal}
            aria-busy={loadingDealRelationships}
          >
            {#if loadingDealRelationships && dealContactId}
              <option value={dealContactId}>{t('common.loading')}</option>
            {/if}
            <option value="">{loadingDealRelationships ? t('common.loading') : t('common.none')}</option>
            {#each dealContactOptions as contact (contact.id)}
              <option value={contact.id}>{contactDisplayName(contact)}</option>
            {/each}
          </select>
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-deal-description">{t('deals.description')}</label>
          <textarea id="modal-deal-description" class="input textarea" rows="3" bind:value={dealDescription}></textarea>
        </div>
      </div>
      {#if loadingDealCustomFields}
        <p class="custom-field-loading">{t('common.loading')}</p>
      {:else}
        <CustomFieldInputs
          definitions={dealCustomFieldDefs}
          values={dealCustomFieldValues}
          onchange={(fieldDefId, value) => updateCustomFieldValue('deal', fieldDefId, value)}
          disabled={isSavingDeal}
        />
      {/if}
    {/snippet}
    {#snippet footer()}
      <button class="btn btn-secondary" type="button" onclick={() => uiStore.closeModal()}>{t('common.cancel')}</button>
      <button class="btn btn-primary" type="button" onclick={submitDeal} disabled={isSavingDeal || loadingDealRelationships}>
        {isSavingDeal ? t('common.loading') : t('common.save')}
      </button>
    {/snippet}
  </Modal>
{/if}

{#if uiStore.activeModal === 'addActivity'}
  <Modal open={true} title={t('activities.addActivity')} size="md" onclose={() => uiStore.closeModal()}>
    {#snippet body()}
      <div class="form-grid">
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-activity-subject">{t('activities.subject')}</label>
          <input id="modal-activity-subject" class="input" bind:value={activitySubject} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-activity-type">{t('activities.type')}</label>
          <select id="modal-activity-type" class="select" bind:value={activityType}>
            {#each ACTIVITY_TYPES as type (type)}
              <option value={type}>{t(`activities.${type}`)}</option>
            {/each}
          </select>
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-activity-due-date">{t('activities.dueDate')}</label>
          <input id="modal-activity-due-date" class="input" type="date" bind:value={activityDueDate} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-activity-due-time">{t('activities.dueTime')}</label>
          <input id="modal-activity-due-time" class="input" type="time" bind:value={activityDueTime} />
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-activity-contact">{t('deals.contact')}</label>
          <select
            id="modal-activity-contact"
            class="select"
            bind:value={activityContactId}
            disabled={loadingActivityRelationships || isSavingActivity}
            aria-busy={loadingActivityRelationships}
          >
            {#if loadingActivityRelationships && activityContactId}
              <option value={activityContactId}>{t('common.loading')}</option>
            {/if}
            <option value="">{loadingActivityRelationships ? t('common.loading') : t('common.none')}</option>
            {#each activityContactOptions as contact (contact.id)}
              <option value={contact.id}>{contactDisplayName(contact)}</option>
            {/each}
          </select>
        </div>
        <div class="form-group">
          <label class="form-label" for="modal-activity-organization">{t('contacts.organization')}</label>
          <select
            id="modal-activity-organization"
            class="select"
            bind:value={activityOrganizationId}
            disabled={loadingActivityRelationships || isSavingActivity}
            aria-busy={loadingActivityRelationships}
          >
            {#if loadingActivityRelationships && activityOrganizationId}
              <option value={activityOrganizationId}>{t('common.loading')}</option>
            {/if}
            <option value="">{loadingActivityRelationships ? t('common.loading') : t('common.none')}</option>
            {#each activityOrganizationOptions as organization (organization.id)}
              <option value={organization.id}>{organization.name}</option>
            {/each}
          </select>
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-activity-deal">{t('deals.title')}</label>
          <select
            id="modal-activity-deal"
            class="select"
            bind:value={activityDealId}
            disabled={loadingActivityRelationships || isSavingActivity}
            aria-busy={loadingActivityRelationships}
          >
            {#if loadingActivityRelationships && activityDealId}
              <option value={activityDealId}>{t('common.loading')}</option>
            {/if}
            <option value="">{loadingActivityRelationships ? t('common.loading') : t('common.none')}</option>
            {#each activityDealOptions as deal (deal.id)}
              <option value={deal.id}>{deal.name}</option>
            {/each}
          </select>
        </div>
        <div class="form-group form-group--full">
          <label class="form-label" for="modal-activity-notes">{t('common.notes')}</label>
          <textarea id="modal-activity-notes" class="input textarea" rows="3" bind:value={activityNotes}></textarea>
        </div>
      </div>
      {#if loadingActivityCustomFields}
        <p class="custom-field-loading">{t('common.loading')}</p>
      {:else}
        <CustomFieldInputs
          definitions={activityCustomFieldDefs}
          values={activityCustomFieldValues}
          onchange={(fieldDefId, value) => updateCustomFieldValue('activity', fieldDefId, value)}
          disabled={isSavingActivity}
        />
      {/if}
    {/snippet}
    {#snippet footer()}
      <button class="btn btn-secondary" type="button" onclick={() => uiStore.closeModal()}>{t('common.cancel')}</button>
      <button class="btn btn-primary" type="button" onclick={submitActivity} disabled={isSavingActivity || loadingActivityRelationships}>
        {isSavingActivity ? t('common.loading') : t('common.save')}
      </button>
    {/snippet}
  </Modal>
{/if}

<style>
  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .form-group--full {
    grid-column: 1 / -1;
  }

  .form-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .custom-field-loading {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  @media (max-width: 720px) {
    .form-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
