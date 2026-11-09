/**
 * src/lib/api/audit.ts - Tauri IPC wrapper for read-only audit log access.
 */

import { invoke } from '@tauri-apps/api/core';

export interface AuditLogEntry {
  id: string;
  actorType: string;
  actorId: string | null;
  action: string;
  entityType: string | null;
  entityId: string | null;
  beforeJson: string | null;
  afterJson: string | null;
  createdAt: string;
  deviceId: string;
}

interface BackendAuditLogEntry {
  id: string;
  actor_type: string;
  actor_id: string | null;
  action: string;
  entity_type: string | null;
  entity_id: string | null;
  before_json: string | null;
  after_json: string | null;
  created_at: string;
  device_id: string;
}

function normalizeAuditLimit(limit: number | undefined): number | undefined {
  if (limit === undefined || !Number.isFinite(limit)) {
    return undefined;
  }

  return Math.min(500, Math.max(1, Math.trunc(limit)));
}

function mapAuditLogEntry(entry: BackendAuditLogEntry): AuditLogEntry {
  return {
    id: entry.id,
    actorType: entry.actor_type,
    actorId: entry.actor_id,
    action: entry.action,
    entityType: entry.entity_type,
    entityId: entry.entity_id,
    beforeJson: entry.before_json,
    afterJson: entry.after_json,
    createdAt: entry.created_at,
    deviceId: entry.device_id,
  };
}

export async function listRecentAuditLog(limit?: number): Promise<AuditLogEntry[]> {
  const entries = await invoke<BackendAuditLogEntry[]>('list_recent_audit_log', {
    limit: normalizeAuditLimit(limit),
  });

  return entries.map(mapAuditLogEntry);
}
