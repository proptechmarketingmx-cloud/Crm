/**
 * src/lib/stores/contacts.svelte.ts — Contact state management for 900CRM.
 *
 * All state uses Svelte 5 $state runes.
 *
 * @module stores/contacts
 */

import {
  listContacts,
  createContact,
  updateContact,
  deleteContact,
  searchContacts,
  listContactDuplicateCandidates,
  mergeContacts,
  setContactLifecycle,
} from '$lib/api/contacts';
import { t } from '$lib/i18n';
import type {
  Contact,
  ContactDuplicateCandidate,
  ContactLifecycle,
  CreateContactPayload,
  UpdateContactPayload,
  ListContactsParams,
} from '$lib/api/contacts';
import { uiStore } from './ui';

// ─────────────────────────────────────────────────────────────────────────────
// ContactStore
// ─────────────────────────────────────────────────────────────────────────────

class ContactStore {
  // ── State ───────────────────────────────────────────────────────────────────

  /** Current page of contacts. */
  contacts = $state<Contact[]>([]);

  /** Currently selected/viewed contact. */
  selectedContact = $state<Contact | null>(null);

  /** Total count (for pagination). */
  total = $state<number>(0);

  /** Current page number (1-based). */
  page = $state<number>(1);

  /** Page size. */
  pageSize = $state<number>(50);

  /** Active filter state. */
  filters = $state<ListContactsParams>({
    sortBy: 'name',
    sortDir: 'asc',
    pageSize: 50,
    page: 1,
  });

  /** Whether a load is in progress. */
  isLoading = $state<boolean>(false);

  /** Whether a save is in progress. */
  isSaving = $state<boolean>(false);

  /** Inline search results. */
  searchResults = $state<Contact[]>([]);

  /** Whether a search is in progress. */
  isSearching = $state<boolean>(false);

  /** Read-only duplicate merge candidates. */
  duplicateCandidates = $state<ContactDuplicateCandidate[]>([]);

  /** Whether duplicate candidates are being checked. */
  duplicateCandidatesLoading = $state<boolean>(false);

  /** Duplicate candidate load error, if any. */
  duplicateCandidatesError = $state<string | null>(null);

  /** Whether a duplicate merge is in progress. */
  isMergingDuplicates = $state<boolean>(false);

  // ── Derived ────────────────────────────────────────────────────────────────

  /** Total number of pages. */
  totalPages = $derived(Math.ceil(this.total / this.pageSize));

  /** Whether there are more pages. */
  hasNextPage = $derived(this.page < this.totalPages);

  /** Whether there is a previous page. */
  hasPrevPage = $derived(this.page > 1);

  // ── Actions ─────────────────────────────────────────────────────────────────

  /**
   * Load the contacts list with the current filter/page state.
   */
  async loadContacts(): Promise<void> {
    this.isLoading = true;
    try {
      const result = await listContacts(this.filters);
      this.contacts = result.contacts;
      this.total = result.total;
      this.page = result.page;
    } catch (err) {
      uiStore.toastError(t('errors.loadNamed', { name: t('entities.contact') }));
      throw err;
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Create a new contact and refresh the list.
   *
   * @param data  Contact creation payload
   * @returns     The created Contact
   */
  async createContact(data: CreateContactPayload): Promise<Contact> {
    this.isSaving = true;
    try {
      const contact = await createContact(data);
      await this.loadContacts();
      uiStore.toastSuccess(t('toasts.created', { name: t('entities.contact') }));
      return contact;
    } catch (err) {
      uiStore.toastError(t('errors.createNamed', { name: t('entities.contact') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  /**
   * Update an existing contact.
   *
   * @param id    Contact UUID
   * @param data  Fields to update
   * @returns     Updated Contact
   */
  async updateContact(id: string, data: UpdateContactPayload): Promise<Contact> {
    this.isSaving = true;
    try {
      const contact = await updateContact(id, data);

      // Optimistic update in the list
      this.contacts = this.contacts.map((c) => (c.id === id ? contact : c));

      if (this.selectedContact?.id === id) {
        this.selectedContact = contact;
      }

      uiStore.toastSuccess(t('toasts.updated', { name: t('entities.contact') }));
      return contact;
    } catch (err) {
      uiStore.toastError(t('errors.updateNamed', { name: t('entities.contact') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  /**
   * Soft-delete a contact and refresh the list.
   *
   * @param id  Contact UUID
   */
  async deleteContact(id: string): Promise<void> {
    try {
      await deleteContact(id);
      this.contacts = this.contacts.filter((c) => c.id !== id);

      if (this.selectedContact?.id === id) {
        this.selectedContact = null;
      }

      this.total = Math.max(0, this.total - 1);
      uiStore.toastSuccess(t('toasts.deleted', { name: t('entities.contact') }));
    } catch (err) {
      uiStore.toastError(t('errors.deleteNamed', { name: t('entities.contact') }));
      throw err;
    }
  }

  /**
   * Search contacts by query string (used for inline search results).
   *
   * @param query  Search string
   */
  async searchContacts(query: string): Promise<void> {
    if (!query.trim()) {
      this.searchResults = [];
      return;
    }

    this.isSearching = true;
    try {
      this.searchResults = await searchContacts(query);
    } catch {
      this.searchResults = [];
    } finally {
      this.isSearching = false;
    }
  }

  /**
   * Load read-only duplicate candidates for active contacts.
   */
  async loadDuplicateCandidates(): Promise<void> {
    this.duplicateCandidatesLoading = true;
    this.duplicateCandidatesError = null;
    try {
      this.duplicateCandidates = await listContactDuplicateCandidates();
    } catch (err) {
      this.duplicateCandidates = [];
      this.duplicateCandidatesError = err instanceof Error ? err.message : 'Failed to check duplicates';
      uiStore.toastError(t('errors.loadNamed', { name: t('entities.contact') }));
      throw err;
    } finally {
      this.duplicateCandidatesLoading = false;
    }
  }

  /**
   * Merge one duplicate contact into another and refresh affected state.
   */
  async setLifecycle(id: string, lifecycle: ContactLifecycle): Promise<Contact> {
    this.isSaving = true;
    try {
      const contact = await setContactLifecycle(id, lifecycle);
      this.contacts = this.contacts.map((item) => (item.id === id ? contact : item));
      if (this.selectedContact?.id === id) {
        this.selectedContact = contact;
      }
      uiStore.toastSuccess(
        lifecycle === 'customer'
          ? t('contacts.convertedToCustomer')
          : t('toasts.updated', { name: t('entities.contact') }),
      );
      return contact;
    } catch (err) {
      uiStore.toastError(t('errors.updateNamed', { name: t('entities.contact') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  async mergeDuplicateContacts(sourceId: string, targetId: string): Promise<Contact> {
    this.isMergingDuplicates = true;
    try {
      const contact = await mergeContacts(sourceId, targetId);
      await this.loadContacts();
      try {
        await this.loadDuplicateCandidates();
      } catch {
        // loadDuplicateCandidates already records inline state and a toast.
      }
      if (this.selectedContact?.id === sourceId) {
        this.selectedContact = contact;
      } else if (this.selectedContact?.id === targetId) {
        this.selectedContact = contact;
      }
      uiStore.toastSuccess(t('toasts.merged', { name: t('nav.contacts') }));
      return contact;
    } catch (err) {
      uiStore.toastError(t('errors.mergeNamed', { name: t('nav.contacts') }));
      throw err;
    } finally {
      this.isMergingDuplicates = false;
    }
  }

  /**
   * Set the selected contact.
   *
   * @param contact  Contact to select, or null to clear
   */
  selectContact(contact: Contact | null): void {
    this.selectedContact = contact;
  }

  /**
   * Update filter/sort/pagination state and reload.
   *
   * @param updates  Partial filter changes
   */
  async setFilters(updates: Partial<ListContactsParams>): Promise<void> {
    this.filters = { ...this.filters, ...updates, page: 1 };
    await this.loadContacts();
  }

  /** Go to the next page. */
  async nextPage(): Promise<void> {
    if (!this.hasNextPage) return;
    this.filters = { ...this.filters, page: this.page + 1 };
    await this.loadContacts();
  }

  /** Go to the previous page. */
  async prevPage(): Promise<void> {
    if (!this.hasPrevPage) return;
    this.filters = { ...this.filters, page: this.page - 1 };
    await this.loadContacts();
  }
}

/** Singleton contacts store. */
export const contactStore = new ContactStore();
