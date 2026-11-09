import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createLocalBackup,
  restoreLocalBackupToAppData,
  validateLocalBackup,
  type LocalBackup,
  type LocalBackupValidation,
  type LocalRestoreResult,
} from './backup';

const metadata = {
  backup_format_version: 1,
  created_at: '2026-06-23T12:00:00Z',
  app_version: '1.0.0',
  schema_version: 4,
  device_id: 'device-1',
  database_file: '900crm.db',
};

describe('backup API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps createLocalBackup to create_local_backup', async () => {
    const response: LocalBackup = {
      backup_dir: '/tmp/backup',
      database_path: '/tmp/backup/900crm.db',
      metadata_path: '/tmp/backup/metadata.json',
      metadata,
    };
    invokeMock.mockResolvedValueOnce(response);

    await expect(createLocalBackup('/tmp/backup')).resolves.toEqual(response);

    expect(invokeMock).toHaveBeenCalledWith('create_local_backup', {
      backup_dir: '/tmp/backup',
    });
  });

  it('maps validateLocalBackup to validate_local_backup', async () => {
    const response: LocalBackupValidation = {
      backup_dir: '/tmp/backup',
      database_path: '/tmp/backup/900crm.db',
      metadata_path: '/tmp/backup/metadata.json',
      metadata,
    };
    invokeMock.mockResolvedValueOnce(response);

    await expect(validateLocalBackup('/tmp/backup')).resolves.toEqual(response);

    expect(invokeMock).toHaveBeenCalledWith('validate_local_backup', {
      backup_dir: '/tmp/backup',
    });
  });

  it('maps restoreLocalBackupToAppData with explicit destructive confirmation', async () => {
    const response: LocalRestoreResult = {
      restored_at: '2026-06-23T12:05:00Z',
      database_path: '/app-data/900crm.db',
      metadata,
    };
    invokeMock.mockResolvedValueOnce(response);

    await expect(restoreLocalBackupToAppData('/tmp/backup', true)).resolves.toEqual(response);

    expect(invokeMock).toHaveBeenCalledWith('restore_local_backup_to_app_data', {
      backup_dir: '/tmp/backup',
      confirm_destructive_restore: true,
    });
  });

  it('rejects restoreLocalBackupToAppData without exactly true destructive confirmation', async () => {
    const restoreWithRuntimeConfirmation = restoreLocalBackupToAppData as unknown as (
      backupDir: string,
      confirmDestructiveRestore?: boolean,
    ) => Promise<LocalRestoreResult>;

    await expect(restoreWithRuntimeConfirmation('/tmp/backup', false)).rejects.toThrow(
      'Local restore requires explicit destructive confirmation.',
    );
    await expect(restoreWithRuntimeConfirmation('/tmp/backup')).rejects.toThrow(
      'Local restore requires explicit destructive confirmation.',
    );

    expect(invokeMock).not.toHaveBeenCalled();
  });
});
