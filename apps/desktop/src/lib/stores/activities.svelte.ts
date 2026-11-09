/**
 * src/lib/stores/activities.svelte.ts — Activity state management for 900CRM.
 *
 * @module stores/activities
 */

import {
  listActivities,
  listUpcoming,
  createActivity,
  updateActivity,
  markComplete,
  markIncomplete,
  deleteActivity,
  logJustHappened,
} from '$lib/api/activities';
import { t } from '$lib/i18n';
import type {
  Activity,
  ActivityType,
  CreateActivityPayload,
  UpdateActivityPayload,
  ListActivitiesParams,
} from '$lib/api/activities';
import { uiStore } from './ui';

// ─────────────────────────────────────────────────────────────────────────────
// ActivityStore
// ─────────────────────────────────────────────────────────────────────────────

class ActivityStore {
  // ── State ───────────────────────────────────────────────────────────────────

  /** Current page of activities. */
  activities = $state<Activity[]>([]);

  /** Upcoming (pending, not overdue) activities. */
  upcoming = $state<Activity[]>([]);

  /** Overdue activities (for dashboard warning). */
  overdue = $derived(
    this.activities.filter((a) => a.status === 'overdue')
  );

  /** Active filters. */
  filters = $state<ListActivitiesParams>({
    sortBy: 'dueDate',
    sortDir: 'asc',
    pageSize: 200,
    page: 1,
  });

  /** Whether the list is loading. */
  isLoading = $state<boolean>(false);

  /** Whether a save is in progress. */
  isSaving = $state<boolean>(false);

  /** Monotonic signal for activity relationship writes outside this route. */
  relationshipRefreshVersion = $state(0);

  // ── Actions ─────────────────────────────────────────────────────────────────

  /**
   * Load activities with the current filter state.
   */
  async loadActivities(): Promise<void> {
    this.isLoading = true;
    try {
      this.activities = await listActivities(this.filters);
    } catch (err) {
      uiStore.toastError(t('errors.loadNamed', { name: t('entities.activity') }));
      throw err;
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Load upcoming activities (for dashboard).
   */
  async loadUpcoming(): Promise<void> {
    try {
      this.upcoming = await listUpcoming();
    } catch (err) {
      console.error('[activities] Failed to load upcoming:', err);
    }
  }

  /**
   * Create a new activity.
   *
   * @param data  Activity creation payload
   * @returns     The created Activity
   */
  async createActivity(data: CreateActivityPayload): Promise<Activity> {
    this.isSaving = true;
    try {
      const activity = await createActivity(data);
      this.activities = [activity, ...this.activities];
      uiStore.toastSuccess(t('toasts.created', { name: t('entities.activity') }));
      return activity;
    } catch (err) {
      uiStore.toastError(t('errors.createNamed', { name: t('entities.activity') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  notifyRelationshipLinksChanged(): void {
    this.relationshipRefreshVersion += 1;
  }

  /**
   * Mark an activity as complete.
   *
   * @param id  Activity UUID
   */
  async markComplete(id: string): Promise<void> {
    try {
      const updated = await markComplete(id);
      this.activities = this.activities.map((a) => (a.id === id ? updated : a));
      this.upcoming = this.upcoming.filter((a) => a.id !== id);
    } catch (err) {
      uiStore.toastError(t('errors.markCompleteFailed'));
      throw err;
    }
  }

  /**
   * Mark an activity as incomplete.
   *
   * @param id  Activity UUID
   */
  async markIncomplete(id: string): Promise<void> {
    try {
      const updated = await markIncomplete(id);
      this.activities = this.activities.map((a) => (a.id === id ? updated : a));
    } catch (err) {
      uiStore.toastError(t('errors.markIncompleteFailed'));
      throw err;
    }
  }

  /**
   * Update an activity.
   *
   * @param id    Activity UUID
   * @param data  Fields to update
   */
  async updateActivity(id: string, data: UpdateActivityPayload): Promise<Activity> {
    this.isSaving = true;
    try {
      const activity = await updateActivity(id, data);
      this.activities = this.activities.map((a) => (a.id === id ? activity : a));
      uiStore.toastSuccess(t('toasts.updated', { name: t('entities.activity') }));
      return activity;
    } catch (err) {
      uiStore.toastError(t('errors.updateNamed', { name: t('entities.activity') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  /**
   * Delete an activity.
   *
   * @param id  Activity UUID
   */
  async deleteActivity(id: string): Promise<void> {
    try {
      await deleteActivity(id);
      this.activities = this.activities.filter((a) => a.id !== id);
      this.upcoming = this.upcoming.filter((a) => a.id !== id);
      uiStore.toastSuccess(t('toasts.deleted', { name: t('entities.activity') }));
    } catch (err) {
      uiStore.toastError(t('errors.deleteNamed', { name: t('entities.activity') }));
      throw err;
    }
  }

  /**
   * Update filter state and reload.
   *
   * @param updates  Partial filter changes
   */
  async setFilters(updates: Partial<ListActivitiesParams>): Promise<void> {
    this.filters = { ...this.filters, ...updates };
    await this.loadActivities();
  }

  async logJustHappened(input: {
    type: ActivityType;
    contactId?: string | null;
    dealId?: string | null;
  }): Promise<Activity> {
    this.isSaving = true;
    try {
      const subject = logJustHappenedSubject(input.type);
      const activity = await logJustHappened({
        type: input.type,
        subject,
        contactId: input.contactId,
        dealId: input.dealId,
      });
      this.activities = [activity, ...this.activities.filter((item) => item.id !== activity.id)];
      uiStore.toastSuccess(t('toasts.created', { name: t('entities.activity') }));
      return activity;
    } catch (err) {
      uiStore.toastError(t('errors.createNamed', { name: t('entities.activity') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }
}

function logJustHappenedSubject(type: ActivityType): string {
  switch (type) {
    case 'visit':
      return t('activities.logVisit');
    case 'whatsapp':
      return t('activities.logWhatsapp');
    case 'sms':
      return t('activities.logSms');
    default:
      return t(`activities.${type}`);
  }
}

/** Singleton activities store. */
export const activityStore = new ActivityStore();
