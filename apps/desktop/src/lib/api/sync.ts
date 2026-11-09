/**
 * src/lib/api/sync.ts — Tauri IPC wrappers for sync status.
 */

import { invoke } from '@tauri-apps/api/core';

export type SyncState = 'idle' | 'syncing' | 'error' | 'success' | 'not_implemented';

export interface SyncStatus {
  state: SyncState;
  lastSyncAt: string | null;
  errorMessage: string | null;
  pendingChanges: number;
}

interface BackendSyncStatus {
  state: SyncState;
  last_sync_at: string | null;
  error_message: string | null;
  pending_changes: number;
}

function mapStatus(status: BackendSyncStatus): SyncStatus {
  return {
    state: status.state,
    lastSyncAt: status.last_sync_at,
    errorMessage: status.error_message,
    pendingChanges: status.pending_changes,
  };
}

export async function triggerSync(): Promise<SyncStatus> {
  const status = await invoke<BackendSyncStatus>('trigger_sync');
  return mapStatus(status);
}

export async function getSyncStatus(): Promise<SyncStatus> {
  const status = await invoke<BackendSyncStatus>('get_sync_status');
  return mapStatus(status);
}
