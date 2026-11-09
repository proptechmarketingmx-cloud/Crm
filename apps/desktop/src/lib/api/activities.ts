/**
 * src/lib/api/activities.ts — Tauri IPC wrappers for activity commands.
 */

import { invoke } from '@tauri-apps/api/core';
import { activityDueTimestamp, isActivityOverdue } from '$lib/utils/activityDue';

export const ACTIVITY_TYPES = [
  'task',
  'call',
  'meeting',
  'email',
  'visit',
  'whatsapp',
  'sms',
] as const;

export type ActivityType = (typeof ACTIVITY_TYPES)[number];
export const DEFAULT_LIST_ACTIVITIES_PAGE_SIZE = 200;
export type ActivityStatus = 'pending' | 'completed' | 'overdue';
export type ActivityLinkEntityType = 'contact' | 'organization' | 'deal';

export interface Activity {
  id: string;
  type: ActivityType;
  subject: string;
  notes: string | null;
  dueDate: string | null;
  completedAt: string | null;
  status: ActivityStatus;
  contactId: string | null;
  contactName: string | null;
  dealId: string | null;
  dealName: string | null;
  createdAt: string;
  updatedAt: string;
}

export type CreateActivityPayload = Omit<
  Activity,
  'id' | 'status' | 'completedAt' | 'contactName' | 'dealName' | 'createdAt' | 'updatedAt'
>;

export type UpdateActivityPayload = Partial<CreateActivityPayload>;

export interface ActivityLink {
  id: string;
  activityId: string;
  entityType: ActivityLinkEntityType;
  entityId: string;
  createdAt: string;
  deletedAt: string | null;
  deviceId: string;
}

export interface ListActivitiesParams {
  type?: ActivityType;
  status?: ActivityStatus;
  contactId?: string;
  dealId?: string;
  sortBy?: 'dueDate' | 'createdAt' | 'subject';
  sortDir?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

interface BackendActivity {
  id: string;
  activity_type: string;
  title: string;
  description: string;
  due_date: string | null;
  completed: boolean;
  contact_id: string | null;
  deal_id: string | null;
  created_at: string;
  updated_at: string;
}

interface BackendActivityLink {
  id: string;
  activity_id: string;
  entity_type: ActivityLinkEntityType;
  entity_id: string;
  created_at: string;
  deleted_at: string | null;
  device_id: string;
}

function normalizeNullable(value: string | null | undefined): string | null {
  if (value == null) {
    return null;
  }

  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function hasOwn<T extends object, K extends PropertyKey>(
  object: T,
  key: K,
): object is T & Record<K, unknown> {
  return Object.prototype.hasOwnProperty.call(object, key);
}

function assignNullableUpdate(
  args: Record<string, unknown>,
  valueKey: string,
  resetKey: string,
  value: string | null | undefined,
): void {
  if (value === undefined) {
    return;
  }

  const normalized = normalizeNullable(value);
  if (normalized === null) {
    args[resetKey] = true;
  } else {
    args[valueKey] = normalized;
  }
}

function toActivityType(value: string): ActivityType {
  if ((ACTIVITY_TYPES as readonly string[]).includes(value)) {
    return value as ActivityType;
  }
  return 'task';
}

function toActivityStatus(completed: boolean, dueDate: string | null): ActivityStatus {
  if (completed) {
    return 'completed';
  }

  if (isActivityOverdue(dueDate, completed)) {
    return 'overdue';
  }

  return 'pending';
}

function mapActivity(activity: BackendActivity): Activity {
  const status = toActivityStatus(activity.completed, activity.due_date);

  return {
    id: activity.id,
    type: toActivityType(activity.activity_type),
    subject: activity.title,
    notes: activity.description?.trim() ? activity.description : null,
    dueDate: activity.due_date,
    completedAt: activity.completed ? activity.updated_at : null,
    status,
    contactId: activity.contact_id,
    contactName: null,
    dealId: activity.deal_id,
    dealName: null,
    createdAt: activity.created_at,
    updatedAt: activity.updated_at,
  };
}

function mapActivityLink(link: BackendActivityLink): ActivityLink {
  return {
    id: link.id,
    activityId: link.activity_id,
    entityType: link.entity_type,
    entityId: link.entity_id,
    createdAt: link.created_at,
    deletedAt: link.deleted_at,
    deviceId: link.device_id,
  };
}

function sortActivities(items: Activity[], params: ListActivitiesParams): Activity[] {
  const sorted = [...items];
  const direction = params.sortDir === 'desc' ? -1 : 1;

  if (!params.sortBy) {
    return sorted;
  }

  sorted.sort((a, b) => {
    switch (params.sortBy) {
      case 'createdAt':
        return ((Date.parse(a.createdAt) || 0) - (Date.parse(b.createdAt) || 0)) * direction;
      case 'subject':
        return a.subject.localeCompare(b.subject) * direction;
      case 'dueDate':
      default:
        return ((activityDueTimestamp(a.dueDate) ?? 0) - (activityDueTimestamp(b.dueDate) ?? 0)) * direction;
    }
  });

  return sorted;
}

export async function createActivity(data: CreateActivityPayload): Promise<Activity> {
  const activity = await invoke<BackendActivity>('create_activity', {
    activity_type: data.type,
    title: data.subject,
    description: data.notes ?? '',
    due_date: data.dueDate,
    contact_id: data.contactId,
    deal_id: data.dealId,
  });

  return mapActivity(activity);
}

export async function getActivity(id: string): Promise<Activity> {
  const activity = await invoke<BackendActivity>('get_activity', { id });
  return mapActivity(activity);
}

export async function listActivities(params: ListActivitiesParams = {}): Promise<Activity[]> {
  const pageSize = params.pageSize ?? DEFAULT_LIST_ACTIVITIES_PAGE_SIZE;
  const page = params.page ?? 1;
  const invokeArgs: Record<string, unknown> = {
    limit: pageSize,
    offset: Math.max(0, (page - 1) * pageSize),
  };

  const activities = await invoke<BackendActivity[]>('list_activities', invokeArgs);

  let mapped = activities.map(mapActivity);

  if (params.type) {
    mapped = mapped.filter((activity) => activity.type === params.type);
  }

  if (params.status) {
    mapped = mapped.filter((activity) => activity.status === params.status);
  }

  if (params.contactId) {
    mapped = mapped.filter((activity) => activity.contactId === params.contactId);
  }

  if (params.dealId) {
    mapped = mapped.filter((activity) => activity.dealId === params.dealId);
  }

  mapped = sortActivities(mapped, params);

  return mapped;
}

export async function listActivitiesForDeals(dealIds: string[]): Promise<Activity[]> {
  const uniqueIds = Array.from(new Set(dealIds.map((id) => id.trim()).filter(Boolean)));
  if (uniqueIds.length === 0) {
    return [];
  }

  const activities = await invoke<BackendActivity[]>('list_activities_for_deals', {
    deal_ids: uniqueIds,
  });
  return activities.map(mapActivity);
}

export async function listUpcoming(): Promise<Activity[]> {
  const activities = await invoke<BackendActivity[]>('list_upcoming_activities', {
    limit: 10,
  });

  return activities.map(mapActivity);
}

export async function markComplete(id: string): Promise<Activity> {
  const activity = await invoke<BackendActivity>('mark_activity_complete', { id });
  return mapActivity(activity);
}

export async function markIncomplete(id: string): Promise<Activity> {
  const activity = await invoke<BackendActivity>('mark_activity_incomplete', { id });
  return mapActivity(activity);
}

export async function updateActivity(id: string, data: UpdateActivityPayload): Promise<Activity> {
  const args: Record<string, unknown> = {
    id,
    activity_type: data.type,
    title: data.subject,
    description: data.notes,
  };

  if (hasOwn(data, 'dueDate')) {
    assignNullableUpdate(args, 'due_date', 'reset_due_date', data.dueDate);
  }

  if (hasOwn(data, 'contactId')) {
    assignNullableUpdate(args, 'contact_id', 'reset_contact_id', data.contactId);
  }

  if (hasOwn(data, 'dealId')) {
    assignNullableUpdate(args, 'deal_id', 'reset_deal_id', data.dealId);
  }

  const activity = await invoke<BackendActivity>('update_activity', args);

  return mapActivity(activity);
}

export async function deleteActivity(id: string): Promise<void> {
  await invoke<void>('delete_activity', { id });
}

export async function logJustHappened(input: {
  type: ActivityType;
  subject: string;
  contactId?: string | null;
  dealId?: string | null;
}): Promise<Activity> {
  const created = await createActivity({
    type: input.type,
    subject: input.subject,
    notes: null,
    dueDate: new Date().toISOString(),
    contactId: input.contactId ?? null,
    dealId: input.dealId ?? null,
  });
  return markComplete(created.id);
}

export async function listActivityLinks(activityId: string): Promise<ActivityLink[]> {
  const links = await invoke<BackendActivityLink[]>('list_activity_links', {
    activity_id: activityId.trim(),
  });
  return links.map(mapActivityLink);
}

export async function listActivityLinksForActivities(
  activityIds: string[],
): Promise<ActivityLink[]> {
  const uniqueIds = Array.from(new Set(activityIds.map((id) => id.trim()).filter(Boolean)));
  if (uniqueIds.length === 0) {
    return [];
  }

  const links = await invoke<BackendActivityLink[]>('list_activity_links_for_activities', {
    activity_ids: uniqueIds,
  });
  return links.map(mapActivityLink);
}

export async function addActivityLink(
  activityId: string,
  entityType: ActivityLinkEntityType,
  entityId: string,
): Promise<ActivityLink> {
  const link = await invoke<BackendActivityLink>('add_activity_link', {
    activity_id: activityId.trim(),
    entity_type: entityType,
    entity_id: entityId.trim(),
  });
  return mapActivityLink(link);
}

export async function removeActivityLink(
  activityId: string,
  entityType: ActivityLinkEntityType,
  entityId: string,
): Promise<ActivityLink> {
  const link = await invoke<BackendActivityLink>('remove_activity_link', {
    activity_id: activityId.trim(),
    entity_type: entityType,
    entity_id: entityId.trim(),
  });
  return mapActivityLink(link);
}
