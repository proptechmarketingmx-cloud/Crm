// @vitest-environment jsdom

import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { listPermissionsMock, upsertPermissionMock } = vi.hoisted(() => ({
  listPermissionsMock: vi.fn(),
  upsertPermissionMock: vi.fn(),
}));

vi.mock('$lib/api/externalClients', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/api/externalClients')>();

  return {
    ...actual,
    listExternalClientPermissions: listPermissionsMock,
    upsertExternalClientToolPermission: upsertPermissionMock,
  };
});

vi.mock('$lib/i18n', () => ({
  t: (key: string, params?: Record<string, string | number>) => {
    const messages: Record<string, string> = {
      'common.edit': 'Edit',
      'common.loading': 'Loading...',
      'settings.externalClientPermissionCanRead': 'Can Read',
      'settings.externalClientPermissionCanWrite': 'Can Write',
      'settings.externalClientPermissionNoWrite': 'No Write',
      'settings.externalClientPermissionRequiresConfirmation': 'Requires Confirmation',
      'settings.externalClientPermissions': 'Tool Permissions',
      'settings.externalClientPermissionsDesc':
        'Review and edit local permission rows. Disabled clients still evaluate as disabled.',
      'settings.externalClientPermissionsEmpty': 'No permission rows yet.',
      'settings.externalClientPermissionsRefresh': 'Refresh Permissions',
      'settings.externalClientPermissionsRows': 'External client permission rows',
      'settings.externalClientPermissionsSave': 'Save Permission Row',
      'settings.externalClientPermissionsSaveSuccess': 'Permission row saved for {toolName}',
      'settings.externalClientToolName': 'Tool Name',
      'settings.externalClientToolNamePlaceholder': 'contacts.search',
      'settings.externalClientPermissionWriteRequiresConfirmation':
        'Write permission rows require confirmation; the backend rejects write rows that do not require confirmation.',
    };
    const template = messages[key] ?? key;
    return template.replace(/\{(\w+)\}/g, (_match, name: string) => String(params?.[name] ?? `{${name}}`));
  },
}));

import ExternalClientPermissions from './ExternalClientPermissions.svelte';
import type { ExternalClient, ExternalClientPermission } from '$lib/api/externalClients';

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

describe('ExternalClientPermissions component', () => {
  beforeEach(() => {
    listPermissionsMock.mockReset();
    upsertPermissionMock.mockReset();
  });

  it('loads and renders permission rows for the selected client', async () => {
    listPermissionsMock.mockResolvedValueOnce([permission]);

    render(ExternalClientPermissions, { client });

    await screen.findByText('contacts.search');

    expect(listPermissionsMock).toHaveBeenCalledWith('client-1');
    const rows = screen.getByLabelText('External client permission rows');
    expect(rows.textContent).toContain('Can Read');
    expect(rows.textContent).toContain('No Write');
    expect(rows.textContent).toContain('Requires Confirmation');
  });

  it('upserts a permission row with read, write, and confirmation controls', async () => {
    listPermissionsMock.mockResolvedValueOnce([]);
    upsertPermissionMock.mockResolvedValueOnce({
      ...permission,
      id: 'permission-2',
      toolName: 'deals.create_draft',
      canWrite: true,
      requiresConfirmation: true,
    });

    render(ExternalClientPermissions, { client });

    await screen.findByText('No permission rows yet.');
    await fireEvent.input(screen.getByLabelText('Tool Name'), {
      target: { value: 'deals.create_draft' },
    });
    await fireEvent.click(screen.getByLabelText('Can Write'));

    const confirmationCheckbox = screen.getByLabelText('Requires Confirmation') as HTMLInputElement;
    expect(confirmationCheckbox.checked).toBe(true);
    expect(confirmationCheckbox.disabled).toBe(true);

    await fireEvent.click(screen.getByRole('button', { name: 'Save Permission Row' }));

    await waitFor(() => {
      expect(upsertPermissionMock).toHaveBeenCalledWith({
        clientId: 'client-1',
        toolName: 'deals.create_draft',
        canRead: true,
        canWrite: true,
        requiresConfirmation: true,
      });
    });
    await screen.findByText('Permission row saved for deals.create_draft');
  });

  it('keeps activation, token, and MCP runtime controls out of the permission editor', async () => {
    listPermissionsMock.mockResolvedValueOnce([permission]);

    const { container } = render(ExternalClientPermissions, { client });
    await screen.findByText('contacts.search');

    const editor = container.querySelector('.external-client-permissions');
    expect(editor).not.toBeNull();
    const scopedQueries = within(editor as HTMLElement);

    expect(scopedQueries.queryByLabelText(/enabled/i)).toBeNull();
    expect(scopedQueries.queryByLabelText(/token|secret/i)).toBeNull();
    expect(scopedQueries.queryByRole('textbox', { name: /server|listener/i })).toBeNull();
    expect(editor?.textContent?.toLowerCase()).not.toContain('runtime');
  });
});
