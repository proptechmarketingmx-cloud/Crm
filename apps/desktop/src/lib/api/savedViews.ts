/**
 * Named list filters stored in the local database.
 */

import { invoke } from '@tauri-apps/api/core';

export type SavedViewEntityType = 'contact' | 'organization' | 'deal' | 'activity' | 'report';

export interface ContactSavedViewFilters {
  search?: string;
  type?: 'person' | 'organization' | 'task' | 'call' | 'meeting' | 'email' | 'visit' | 'whatsapp' | 'sms';
  lifecycle?: 'lead' | 'customer';
  country?: string;
  customFieldDefId?: string;
  customFieldQuery?: string;
  owner?: string;
  sortBy?: string;
  sortDir?: 'asc' | 'desc';
  attention?: 'needsFollowUp' | 'stale' | 'overdue';
  status?: 'pending' | 'completed' | 'overdue';
  bucket?: 'overdue' | 'today' | 'thisWeek' | 'later' | 'unscheduled' | 'completed';
  focus?: 'pipeline' | 'activity' | 'stale';
}

export interface SavedView<TFilters = ContactSavedViewFilters> {
  id: string;
  entityType: SavedViewEntityType;
  name: string;
  filters: TFilters;
  createdAt: string;
  updatedAt: string;
}

interface BackendSavedView {
  id: string;
  entity_type: SavedViewEntityType;
  name: string;
  filters_json: string;
  created_at: string;
  updated_at: string;
}

interface BackendFilters {
  search?: string;
  type?: 'person' | 'organization' | 'task' | 'call' | 'meeting' | 'email' | 'visit' | 'whatsapp' | 'sms';
  lifecycle?: 'lead' | 'customer';
  country?: string;
  custom_field_def_id?: string;
  custom_field_query?: string;
  owner?: string;
  sort_by?: string;
  sort_dir?: 'asc' | 'desc';
  attention?: 'needsFollowUp' | 'stale' | 'overdue';
  status?: 'pending' | 'completed' | 'overdue';
  bucket?: 'overdue' | 'today' | 'thisWeek' | 'later' | 'unscheduled' | 'completed';
  focus?: 'pipeline' | 'activity' | 'stale';
}

function toBackendFilters(filters: ContactSavedViewFilters): BackendFilters {
  return {
    search: filters.search?.trim() || undefined,
    type: filters.type,
    lifecycle: filters.lifecycle,
    country: filters.country?.trim() || undefined,
    custom_field_def_id: filters.customFieldDefId?.trim() || undefined,
    custom_field_query: filters.customFieldQuery?.trim() || undefined,
    owner: filters.owner?.trim() || undefined,
    sort_by: filters.sortBy,
    sort_dir: filters.sortDir,
    attention: filters.attention,
    status: filters.status,
    bucket: filters.bucket,
    focus: filters.focus,
  };
}

function fromBackendFilters(raw: string): ContactSavedViewFilters {
  const parsed = JSON.parse(raw) as BackendFilters;
  return {
    search: parsed.search,
    type: parsed.type,
    lifecycle: parsed.lifecycle,
    country: parsed.country,
    customFieldDefId: parsed.custom_field_def_id,
    customFieldQuery: parsed.custom_field_query,
    owner: parsed.owner,
    sortBy: parsed.sort_by,
    sortDir: parsed.sort_dir,
    attention: parsed.attention,
    status: parsed.status,
    bucket: parsed.bucket,
    focus: parsed.focus,
  };
}

function mapView(view: BackendSavedView): SavedView {
  return {
    id: view.id,
    entityType: view.entity_type,
    name: view.name,
    filters: fromBackendFilters(view.filters_json),
    createdAt: view.created_at,
    updatedAt: view.updated_at,
  };
}

export async function listSavedViews(entityType: SavedViewEntityType): Promise<SavedView[]> {
  const views = await invoke<BackendSavedView[]>('list_saved_views', {
    entity_type: entityType,
  });
  return views.map(mapView);
}

export async function createSavedView(
  entityType: SavedViewEntityType,
  name: string,
  filters: ContactSavedViewFilters,
): Promise<SavedView> {
  const view = await invoke<BackendSavedView>('create_saved_view', {
    entity_type: entityType,
    name: name.trim(),
    filters_json: JSON.stringify(toBackendFilters(filters)),
  });
  return mapView(view);
}

export async function updateSavedView(
  id: string,
  name: string,
  filters: ContactSavedViewFilters,
): Promise<SavedView> {
  const view = await invoke<BackendSavedView>('update_saved_view', {
    id,
    name: name.trim(),
    filters_json: JSON.stringify(toBackendFilters(filters)),
  });
  return mapView(view);
}

export async function deleteSavedView(id: string): Promise<void> {
  await invoke<void>('delete_saved_view', { id });
}

export function filtersMatch(
  left: ContactSavedViewFilters,
  right: ContactSavedViewFilters,
): boolean {
  return JSON.stringify(toBackendFilters(left)) === JSON.stringify(toBackendFilters(right));
}
