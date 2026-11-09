/**
 * src/lib/api/externalClients.ts - Tauri IPC wrappers for external clients and permissions.
 */

import { invoke } from '@tauri-apps/api/core';

export interface ExternalClient {
  id: string;
  name: string;
  clientType: string;
  permissionMode: string;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
  deviceId: string;
}

export type ExternalClientPermissionMode =
  | 'disabled'
  | 'read_only'
  | 'draft_only'
  | 'write_with_confirmation'
  | 'write_allowed';

export type EditableExternalClientPermissionMode = 'disabled' | 'read_only' | 'draft_only';

export type ToolPermissionDecisionReason =
  | 'allowed'
  | 'client_disabled'
  | 'unsupported_client_mode'
  | 'missing_tool_permission'
  | 'read_not_allowed'
  | 'write_not_allowed'
  | 'confirmation_not_required';

export interface ExternalClientPermission {
  id: string;
  clientId: string;
  toolName: string;
  canRead: boolean;
  canWrite: boolean;
  requiresConfirmation: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ToolPermissionEvaluation {
  allowed: boolean;
  mode: ExternalClientPermissionMode;
  toolName: string;
  reason: ToolPermissionDecisionReason;
}

export interface UpsertExternalClientToolPermissionInput {
  clientId: string;
  toolName: string;
  canRead: boolean;
  canWrite: boolean;
  requiresConfirmation: boolean;
}

export interface UpdateExternalClientActivationInput {
  clientId: string;
  enabled: boolean;
  permissionMode: EditableExternalClientPermissionMode;
}

interface BackendExternalClient {
  id: string;
  name: string;
  client_type: string;
  permission_mode: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  device_id: string;
}

interface BackendExternalClientPermission {
  id: string;
  client_id: string;
  tool_name: string;
  can_read: boolean;
  can_write: boolean;
  requires_confirmation: boolean;
  created_at: string;
  updated_at: string;
}

interface BackendToolPermissionEvaluation {
  allowed: boolean;
  mode: ExternalClientPermissionMode;
  tool_name: string;
  reason: ToolPermissionDecisionReason;
}

function mapExternalClient(client: BackendExternalClient): ExternalClient {
  return {
    id: client.id,
    name: client.name,
    clientType: client.client_type,
    permissionMode: client.permission_mode,
    enabled: client.enabled,
    createdAt: client.created_at,
    updatedAt: client.updated_at,
    deletedAt: client.deleted_at,
    deviceId: client.device_id,
  };
}

function mapExternalClientPermission(
  permission: BackendExternalClientPermission,
): ExternalClientPermission {
  return {
    id: permission.id,
    clientId: permission.client_id,
    toolName: permission.tool_name,
    canRead: permission.can_read,
    canWrite: permission.can_write,
    requiresConfirmation: permission.requires_confirmation,
    createdAt: permission.created_at,
    updatedAt: permission.updated_at,
  };
}

function mapToolPermissionEvaluation(
  evaluation: BackendToolPermissionEvaluation,
): ToolPermissionEvaluation {
  return {
    allowed: evaluation.allowed,
    mode: evaluation.mode,
    toolName: evaluation.tool_name,
    reason: evaluation.reason,
  };
}

export async function listExternalClients(): Promise<ExternalClient[]> {
  const clients = await invoke<BackendExternalClient[]>('list_external_clients');
  return clients.map(mapExternalClient);
}

export async function createExternalClientPlaceholder(
  name: string,
  clientType: string,
): Promise<ExternalClient> {
  const client = await invoke<BackendExternalClient>('create_external_client_placeholder', {
    name,
    client_type: clientType,
  });

  return mapExternalClient(client);
}

export async function updateExternalClientActivation(
  input: UpdateExternalClientActivationInput,
): Promise<ExternalClient> {
  const client = await invoke<BackendExternalClient>('update_external_client_activation', {
    client_id: input.clientId,
    enabled: input.enabled,
    permission_mode: input.permissionMode,
  });

  return mapExternalClient(client);
}

export async function listExternalClientPermissions(
  clientId: string,
): Promise<ExternalClientPermission[]> {
  const permissions = await invoke<BackendExternalClientPermission[]>(
    'list_external_client_permissions',
    { client_id: clientId },
  );

  return permissions.map(mapExternalClientPermission);
}

export async function upsertExternalClientToolPermission(
  input: UpsertExternalClientToolPermissionInput,
): Promise<ExternalClientPermission> {
  const permission = await invoke<BackendExternalClientPermission>(
    'upsert_external_client_tool_permission',
    {
      client_id: input.clientId,
      tool_name: input.toolName,
      can_read: input.canRead,
      can_write: input.canWrite,
      requires_confirmation: input.requiresConfirmation,
    },
  );

  return mapExternalClientPermission(permission);
}

export async function evaluateExternalClientToolReadPermission(
  clientId: string,
  toolName: string,
): Promise<ToolPermissionEvaluation> {
  const evaluation = await invoke<BackendToolPermissionEvaluation>(
    'evaluate_external_client_tool_read_permission',
    {
      client_id: clientId,
      tool_name: toolName,
    },
  );

  return mapToolPermissionEvaluation(evaluation);
}

export async function evaluateExternalClientDraftPermission(
  clientId: string,
  toolName: string,
): Promise<ToolPermissionEvaluation> {
  const evaluation = await invoke<BackendToolPermissionEvaluation>(
    'evaluate_external_client_draft_permission',
    {
      client_id: clientId,
      tool_name: toolName,
    },
  );

  return mapToolPermissionEvaluation(evaluation);
}
