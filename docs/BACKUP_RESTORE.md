# Backup and Restore

900CRM stores CRM data locally in SQLite. The backup surface creates a portable local backup directory and restores it only after validation plus explicit user confirmation.

## Backup contents

A local backup directory contains:

- `900crm.db` - a standalone SQLite snapshot of the CRM database.
- `metadata.json` - backup format, app version, schema version, device id, creation time, and database file name.

Backup creation refuses to overwrite an existing `900crm.db` or `metadata.json` in the selected folder. Use an empty folder for each new backup.

## Create a backup

1. Open `Settings`.
2. In `Data Management`, choose a backup folder.
3. Select `Create Backup`.
4. Move the completed backup folder to durable storage, such as an encrypted external drive, if the machine is at risk.

Backups are local files. 900CRM does not upload them, sync them, send them to
any 900 Labs service, or encrypt the backup folder.

The Settings Data Management backup panel warns before backup actions that
local backup folders are unencrypted local files. Store backup folders
containing sensitive data in a trusted or encrypted location.

## Automatic pre-import backups

Supported desktop imports for contacts, deals, activities, and organizations
create a local backup automatically before writing CSV, mapped CSV, or JSON rows. These
pre-import backups use the same local backup format and are stored under the
platform app data directory at:

```text
pre-import-backups/<timestamp-and-sequence>/
```

If the automatic backup cannot be created, the import is not run. The
post-import summary displays the created backup path. Preflight duplicate checks
remain read-only and do not create backups. Contact, deal, and organization
imports with duplicate auto-merge enabled use the same automatic pre-import
backup guard before any create or merge writes.

When a post-import summary includes an automatic pre-import backup path, the
summary also offers a destructive restore control. That control validates the
backup path first, asks the user to confirm replacing the current local
database, and then invokes the same local restore API used by Settings with
`confirm_destructive_restore: true`.

The Import/Export summary may also offer row-level rollback for the
just-completed import. That control is separate from backup restore: it uses a
summary-scoped import rollback plan to soft-delete rows created by that import
or restore only fields changed by duplicate auto-merge. It does not validate or
restore a backup folder, and it is not available after app restart or after the
summary is dismissed.

## Validate a backup

Validation is a read-only safety check. It verifies:

- backup metadata is readable JSON;
- backup format version is supported;
- app version is compatible;
- backup schema version matches the database schema;
- required core tables are present;
- SQLite `PRAGMA quick_check` passes on a temporary copy of the backup database.

Validate a backup before attempting restore, especially after copying it across disks or devices.

## Restore a backup

Restore is destructive because it replaces the current local app database.

The desktop UI allows restore only after the selected folder validates. The
Settings restore handler validates again, then asks for explicit confirmation
before passing `confirm_destructive_restore: true` to the Tauri command. The
Import/Export summary restore control follows the same validation and
confirmation rule for the automatic pre-import backup path.

Before restoring:

- create a fresh backup of the current database if you may need to roll back;
- confirm the selected folder is the intended backup;
- close other tools that may be inspecting the app data database.

During restore, the active `CrmCore` instance is closed, the validated backup database is copied into the app data directory, stale SQLite sidecars are removed, and `CrmCore` is reopened.

Import summary backup restore is full database replacement. It is not
row-level import rollback, partial restore, or merge-back. Row-level import
rollback is intentionally narrower: it only applies supported import rows and
supported contact/deal/activity custom field value changes from the current
summary, and skips records that changed after import.

## Developer API surface

Frontend wrapper:

- `apps/desktop/src/lib/api/backup.ts`

Tauri commands:

- `create_local_backup`
- `validate_local_backup`
- `restore_local_backup_to_app_data`

Core service:

- `crates/crm-core/src/services/backup.rs`

The backup surface is intentionally local-only. It does not add MCP behavior, AI behavior, remote sync behavior, or database migrations.
