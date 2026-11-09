/**
 * src/lib/stores/organizations.svelte.ts - Organization state management for 900CRM.
 */

import {
  createOrganization,
  deleteOrganization,
  getOrganization,
  linkContactToOrganization,
  listOrganizations,
  updateOrganization,
} from '$lib/api/organizations';
import { t } from '$lib/i18n';
import type {
  CreateOrganizationPayload,
  LinkedOrganizationContact,
  Organization,
  UpdateOrganizationPayload,
} from '$lib/api/organizations';
import { uiStore } from './ui';

class OrganizationStore {
  organizations = $state<Organization[]>([]);
  selectedOrganization = $state<Organization | null>(null);
  lastLinkedContact = $state<LinkedOrganizationContact | null>(null);
  isLoading = $state<boolean>(false);
  isSaving = $state<boolean>(false);
  isLinkingContact = $state<boolean>(false);

  total = $derived(this.organizations.length);

  async loadOrganizations(): Promise<void> {
    this.isLoading = true;
    try {
      this.organizations = await listOrganizations();
    } catch (err) {
      uiStore.toastError(t('errors.loadNamed', { name: t('entities.organization') }));
      throw err;
    } finally {
      this.isLoading = false;
    }
  }

  async getOrganization(id: string): Promise<Organization> {
    this.isLoading = true;
    try {
      const organization = await getOrganization(id);
      this.selectedOrganization = organization;
      this.organizations = this.upsertOrganization(organization);
      return organization;
    } catch (err) {
      uiStore.toastError(t('errors.loadNamed', { name: t('entities.organization') }));
      throw err;
    } finally {
      this.isLoading = false;
    }
  }

  async createOrganization(data: CreateOrganizationPayload): Promise<Organization> {
    this.isSaving = true;
    try {
      const organization = await createOrganization(data);
      this.organizations = this.upsertOrganization(organization);
      uiStore.toastSuccess(t('toasts.created', { name: t('entities.organization') }));
      return organization;
    } catch (err) {
      uiStore.toastError(t('errors.createNamed', { name: t('entities.organization') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  async updateOrganization(id: string, data: UpdateOrganizationPayload): Promise<Organization> {
    this.isSaving = true;
    try {
      const organization = await updateOrganization(id, data);
      this.organizations = this.upsertOrganization(organization);

      if (this.selectedOrganization?.id === id) {
        this.selectedOrganization = organization;
      }

      uiStore.toastSuccess(t('toasts.updated', { name: t('entities.organization') }));
      return organization;
    } catch (err) {
      uiStore.toastError(t('errors.updateNamed', { name: t('entities.organization') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  async deleteOrganization(id: string): Promise<void> {
    try {
      await deleteOrganization(id);
      this.organizations = this.organizations.filter((organization) => organization.id !== id);

      if (this.selectedOrganization?.id === id) {
        this.selectedOrganization = null;
      }

      uiStore.toastSuccess(t('toasts.deleted', { name: t('entities.organization') }));
    } catch (err) {
      uiStore.toastError(t('errors.deleteNamed', { name: t('entities.organization') }));
      throw err;
    }
  }

  async linkContactToOrganization(
    contactId: string,
    organizationId: string | null,
  ): Promise<LinkedOrganizationContact> {
    this.isLinkingContact = true;
    try {
      const contact = await linkContactToOrganization(contactId, organizationId);
      this.lastLinkedContact = contact;
      uiStore.toastSuccess(
        organizationId ? t('organizations.linkCreated') : t('organizations.linkCleared'),
      );
      return contact;
    } catch (err) {
      uiStore.toastError(t('errors.updateNamed', { name: t('entities.organization') }));
      throw err;
    } finally {
      this.isLinkingContact = false;
    }
  }

  selectOrganization(organization: Organization | null): void {
    this.selectedOrganization = organization;
  }

  private upsertOrganization(organization: Organization): Organization[] {
    const next = this.organizations.some((existing) => existing.id === organization.id)
      ? this.organizations.map((existing) => existing.id === organization.id ? organization : existing)
      : [...this.organizations, organization];

    return next.sort((left, right) => left.name.localeCompare(right.name));
  }
}

export const organizationStore = new OrganizationStore();
