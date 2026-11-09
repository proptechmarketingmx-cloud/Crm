/**
 * src/lib/api/backup.ts — Tauri IPC wrappers for local backup and restore.
 */

import { invoke } from '@tauri-apps/api/core';

export interface LocalBackupMetadata {
  backup_format_version: number;
  created_at: string;
  app_version: string;
  schema_version: number;
  device_id: string;
  database_file: string;
}

export interface LocalBackup {
  backup_dir: string;
  database_path: string;
  metadata_path: string;
  metadata: LocalBackupMetadata;
}

export interface LocalBackupValidation {
  backup_dir: string;
  database_path: string;
  metadata_path: string;
  metadata: LocalBackupMetadata;
}

export interface LocalRestoreResult {
  restored_at: string;
  database_path: string;
  metadata: LocalBackupMetadata;
}

export async function createLocalBackup(backupDir: string): Promise<LocalBackup> {
  return invoke<LocalBackup>('create_local_backup', {
    backup_dir: backupDir,
  });
}

export async function validateLocalBackup(backupDir: string): Promise<LocalBackupValidation> {
  return invoke<LocalBackupValidation>('validate_local_backup', {
    backup_dir: backupDir,
  });
}

export async function restoreLocalBackupToAppData(
  backupDir: string,
  confirmDestructiveRestore: true,
): Promise<LocalRestoreResult> {
  if (confirmDestructiveRestore !== true) {
    throw new Error('Local restore requires explicit destructive confirmation.');
  }

  return invoke<LocalRestoreResult>('restore_local_backup_to_app_data', {
    backup_dir: backupDir,
    confirm_destructive_restore: true,
  });
}
