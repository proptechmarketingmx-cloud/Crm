# Sprint 058 - Pre-Import Backup

Date: 2026-06-25
Branch: `codex/pre-import-backup`

## Scope

Close the import/export safety gap by creating an automatic local backup before
supported desktop imports write CRM rows.

Implemented scope:

- Contacts, deals, and organizations desktop import commands now create a local
  backup before calling the existing `crm-core` CSV, mapped CSV, or JSON import
  methods.
- Automatic backup directories are generated under app data at
  `pre-import-backups/<timestamp-and-sequence>/`.
- Backup creation uses `CrmCore::create_local_backup`; no new backup format was
  added.
- Backup failure stops the import before any import closure is run.
- Duplicate preflight commands remain read-only and unchanged.
- Import commands return `ImportWithBackupResult`, containing the existing
  import summary plus the created backup metadata/path.
- The Import/Export modal displays the automatic pre-import backup path in the
  post-import summary.
- Documentation now describes the automatic backup location and the unencrypted
  local-file caveat.

## Non-Goals

- No import rollback.
- No restore button from the import summary.
- No duplicate auto-merge.
- No JSON mapping or browser preview.
- No broader import/export entities beyond contacts, deals, and organizations.
- No custom fields, notes, tags, activities, audit, proposed actions, external
  clients, or permissions import/export.
- No cloud, remote, scheduled import/export, MCP, AI, sync-server behavior,
  encryption, release signing, notarization, or schema normalization.

## Verification Checklist

- `npm run lint`
- `npm run check`
- `npm run test`
- `npm run build`
- `npm run test:e2e`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo check --workspace`
- `cargo test --workspace`
- Raw SQL scans in `apps/desktop/src-tauri/src/commands` and
  `crates/crm-core/src/crm_engine`
- `git diff --check main...HEAD`
- `git fsck --full --no-progress`

## Acceptance Notes

- The Tauri command layer remains free of raw SQL.
- Restore behavior is unchanged.
- The automatic backup path is visible to the UI without requiring users to pick
  a backup folder before import.
- Preflight duplicate detection remains read-only and does not create backup
  directories.
