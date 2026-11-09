import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createExternalClientPlaceholder,
  evaluateExternalClientDraftPermission,
  evaluateExternalClientToolReadPermission,
  listExternalClientPermissions,
  listExternalClients,
  updateExternalClientActivation,
  upsertExternalClientToolPermission,
  type ExternalClient,
  type ExternalClientPermission,
  type ToolPermissionEvaluation,
} from './externalClients';

const backendClient = {
  id: 'client-1',
  name: 'Claude Desktop',
  client_type: 'mcp',
  permission_mode: 'disabled',
  enabled: false,
  created_at: '2026-06-24T08:00:00Z',
  updated_at: '2026-06-24T08:00:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

const client: ExternalClient = {
  id: 'client-1',
  name: 'Claude Desktop',
  clientType: 'mcp',
  permissionMode: 'disabled',
  enabled: false,
  createdAt: '2026-06-24T08:00:00Z',
  updatedAt: '2026-06-24T08:00:00Z',
  deletedAt: null,
  deviceId: 'device-1',
};

const backendPermission = {
  id: 'permission-1',
  client_id: 'client-1',
  tool_name: 'contacts.search',
  can_read: true,
  can_write: false,
  requires_confirmation: true,
  created_at: '2026-06-24T08:10:00Z',
  updated_at: '2026-06-24T08:11:00Z',
};

const permission: ExternalClientPermission = {
  id: 'permission-1',
  clientId: 'client-1',
  toolName: 'contacts.search',
  canRead: true,
  canWrite: false,
  requiresConfirmation: true,
  createdAt: '2026-06-24T08:10:00Z',
  updatedAt: '2026-06-24T08:11:00Z',
};

const backendEvaluation = {
  allowed: true,
  mode: 'read_only',
  tool_name: 'contacts.search',
  reason: 'allowed',
} as const;

const evaluation: ToolPermissionEvaluation = {
  allowed: true,
  mode: 'read_only',
  toolName: 'contacts.search',
  reason: 'allowed',
};

describe('external clients API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps listExternalClients to list_external_clients', async () => {
    invokeMock.mockResolvedValueOnce([backendClient]);

    await expect(listExternalClients()).resolves.toEqual([client]);

    expect(invokeMock).toHaveBeenCalledWith('list_external_clients');
  });

  it('maps createExternalClientPlaceholder to create_external_client_placeholder', async () => {
    invokeMock.mockResolvedValueOnce(backendClient);

    const created = await createExternalClientPlaceholder('Claude Desktop', 'mcp');

    expect(created).toEqual(client);

    expect(invokeMock).toHaveBeenCalledWith('create_external_client_placeholder', {
      name: 'Claude Desktop',
      client_type: 'mcp',
    });
    expect(created.enabled).toBe(false);
    expect(created.permissionMode).toBe('disabled');
  });

  it('maps updateExternalClientActivation to update_external_client_activation', async () => {
    const backendUpdatedClient = {
      ...backendClient,
      permission_mode: 'read_only',
      enabled: true,
      updated_at: '2026-06-24T08:15:00Z',
    };
    invokeMock.mockResolvedValueOnce(backendUpdatedClient);

    await expect(
      updateExternalClientActivation({
        clientId: 'client-1',
        enabled: true,
        permissionMode: 'read_only',
      }),
    ).resolves.toEqual({
      ...client,
      permissionMode: 'read_only',
      enabled: true,
      updatedAt: '2026-06-24T08:15:00Z',
    });

    expect(invokeMock).toHaveBeenCalledWith('update_external_client_activation', {
      client_id: 'client-1',
      enabled: true,
      permission_mode: 'read_only',
    });
  });

  it('maps listExternalClientPermissions to list_external_client_permissions', async () => {
    invokeMock.mockResolvedValueOnce([backendPermission]);

    await expect(listExternalClientPermissions('client-1')).resolves.toEqual([permission]);

    expect(invokeMock).toHaveBeenCalledWith('list_external_client_permissions', {
      client_id: 'client-1',
    });
  });

  it('maps upsertExternalClientToolPermission to upsert_external_client_tool_permission', async () => {
    invokeMock.mockResolvedValueOnce(backendPermission);

    await expect(
      upsertExternalClientToolPermission({
        clientId: 'client-1',
        toolName: 'contacts.search',
        canRead: true,
        canWrite: false,
        requiresConfirmation: true,
      }),
    ).resolves.toEqual(permission);

    expect(invokeMock).toHaveBeenCalledWith('upsert_external_client_tool_permission', {
      client_id: 'client-1',
      tool_name: 'contacts.search',
      can_read: true,
      can_write: false,
      requires_confirmation: true,
    });
  });

  it('maps evaluateExternalClientToolReadPermission to evaluate_external_client_tool_read_permission', async () => {
    invokeMock.mockResolvedValueOnce(backendEvaluation);

    await expect(
      evaluateExternalClientToolReadPermission('client-1', 'contacts.search'),
    ).resolves.toEqual(evaluation);

    expect(invokeMock).toHaveBeenCalledWith(
      'evaluate_external_client_tool_read_permission',
      {
        client_id: 'client-1',
        tool_name: 'contacts.search',
      },
    );
  });

  it('maps evaluateExternalClientDraftPermission to evaluate_external_client_draft_permission', async () => {
    const backendDraftEvaluation = {
      ...backendEvaluation,
      mode: 'draft_only',
      tool_name: 'create_activity_draft',
    } as const;
    invokeMock.mockResolvedValueOnce(backendDraftEvaluation);

    await expect(
      evaluateExternalClientDraftPermission('client-1', 'create_activity_draft'),
    ).resolves.toEqual({
      ...evaluation,
      mode: 'draft_only',
      toolName: 'create_activity_draft',
    });

    expect(invokeMock).toHaveBeenCalledWith('evaluate_external_client_draft_permission', {
      client_id: 'client-1',
      tool_name: 'create_activity_draft',
    });
  });
});
