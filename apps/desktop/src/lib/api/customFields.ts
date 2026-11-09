/**
 * src/lib/api/customFields.ts — Tauri IPC wrappers for custom field foundation APIs.
 */

import { invoke } from '@tauri-apps/api/core';

export type CustomFieldEntityType = 'contact' | 'deal' | 'activity' | 'organization';
export type CustomFieldType = 'text' | 'number' | 'date' | 'boolean' | 'select';

export interface CustomFieldDefinition {
  id: string;
  entity_type: CustomFieldEntityType;
  field_name: string;
  field_type: CustomFieldType;
  field_options: string | null;
  sort_order: number;
  created_at: string;
}

export interface CustomFieldValue {
  id: string;
  field_def_id: string;
  entity_id: string;
  value: string;
  created_at: string;
  updated_at: string;
}

export interface EntityCustomFieldValue {
  value_id: string;
  field_def_id: string;
  field_name: string;
  field_type: CustomFieldType;
  field_options: string | null;
  sort_order: number;
  value: string;
  updated_at: string;
}

export interface EntityTypeCustomFieldValue {
  entity_id: string;
  field_def_id: string;
  value: string;
  updated_at: string;
}

export async function listCustomFieldDefinitions(
  entityType?: CustomFieldEntityType,
): Promise<CustomFieldDefinition[]> {
  return invoke<CustomFieldDefinition[]>('list_custom_field_defs', {
    entity_type: entityType,
  });
}

export async function createCustomFieldDefinition(payload: {
  entityType: CustomFieldEntityType;
  fieldName: string;
  fieldType: CustomFieldType;
  fieldOptions?: string | null;
  sortOrder?: number;
}): Promise<CustomFieldDefinition> {
  return invoke<CustomFieldDefinition>('create_custom_field_def', {
    entity_type: payload.entityType,
    field_name: payload.fieldName,
    field_type: payload.fieldType,
    field_options: payload.fieldOptions ?? null,
    sort_order: payload.sortOrder ?? 0,
  });
}

export async function updateCustomFieldDefinition(payload: {
  id: string;
  fieldName?: string;
  fieldType?: CustomFieldType;
  fieldOptions?: string | null;
  sortOrder?: number;
}): Promise<CustomFieldDefinition> {
  return invoke<CustomFieldDefinition>('update_custom_field_def', {
    id: payload.id,
    field_name: payload.fieldName,
    field_type: payload.fieldType,
    field_options: payload.fieldOptions,
    sort_order: payload.sortOrder,
  });
}

export async function deleteCustomFieldDefinition(id: string): Promise<void> {
  await invoke<void>('delete_custom_field_def', { id });
}

export async function setCustomFieldValue(payload: {
  fieldDefId: string;
  entityId: string;
  value: string;
}): Promise<CustomFieldValue> {
  return invoke<CustomFieldValue>('set_custom_field_value', {
    field_def_id: payload.fieldDefId,
    entity_id: payload.entityId,
    value: payload.value,
  });
}

export async function listCustomFieldValues(
  entityType: CustomFieldEntityType,
  entityId: string,
): Promise<EntityCustomFieldValue[]> {
  return invoke<EntityCustomFieldValue[]>('list_custom_field_values', {
    entity_type: entityType,
    entity_id: entityId,
  });
}

export async function listCustomFieldValuesForEntityType(
  entityType: CustomFieldEntityType,
): Promise<EntityTypeCustomFieldValue[]> {
  return invoke<EntityTypeCustomFieldValue[]>('list_custom_field_values_for_type', {
    entity_type: entityType,
  });
}
