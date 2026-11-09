import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { listRecentAuditLog, type AuditLogEntry } from './audit';

const backendEntry = {
  id: 'audit-1',
  actor_type: 'desktop_app',
  actor_id: null,
  action: 'create',
  entity_type: 'contact',
  entity_id: 'contact-1',
  before_json: null,
  after_json: '{"id":"contact-1"}',
  created_at: '2026-06-24T08:00:00Z',
  device_id: 'device-1',
};

const entry: AuditLogEntry = {
  id: 'audit-1',
  actorType: 'desktop_app',
  actorId: null,
  action: 'create',
  entityType: 'contact',
  entityId: 'contact-1',
  beforeJson: null,
  afterJson: '{"id":"contact-1"}',
  createdAt: '2026-06-24T08:00:00Z',
  deviceId: 'device-1',
};

describe('audit API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps listRecentAuditLog to list_recent_audit_log with normalized limit', async () => {
    invokeMock.mockResolvedValueOnce([backendEntry]);

    await expect(listRecentAuditLog(42.8)).resolves.toEqual([entry]);

    expect(invokeMock).toHaveBeenCalledWith('list_recent_audit_log', {
      limit: 42,
    });
  });

  it('omits invalid limits so the command default applies', async () => {
    invokeMock.mockResolvedValueOnce([]);

    await expect(listRecentAuditLog(Number.NaN)).resolves.toEqual([]);

    expect(invokeMock).toHaveBeenCalledWith('list_recent_audit_log', {
      limit: undefined,
    });
  });

  it('clamps limits to the command and storage bounds', async () => {
    invokeMock.mockResolvedValueOnce([]);

    await listRecentAuditLog(0);

    expect(invokeMock).toHaveBeenCalledWith('list_recent_audit_log', {
      limit: 1,
    });

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce([]);

    await listRecentAuditLog(900);

    expect(invokeMock).toHaveBeenCalledWith('list_recent_audit_log', {
      limit: 500,
    });
  });
});
