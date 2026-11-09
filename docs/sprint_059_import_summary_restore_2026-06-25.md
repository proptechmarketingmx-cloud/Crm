# Sprint 059 - Import Summary Restore

Date: 2026-06-25
Branch: `codex/import-summary-restore`

## Scope

Close the documented gap for restoring the automatic pre-import backup created
by Sprint 058 directly from the Import/Export modal summary.

Implemented scope:

- The Import/Export summary now shows a destructive restore control when the
  import result includes an automatic pre-import backup path.
- The control validates the backup path with `validateLocalBackup` before any
  restore attempt.
- Restore requires explicit user confirmation and calls
  `restoreLocalBackupToAppData(backupPath, true)` only after confirmation.
- Cancelled confirmation does not call restore.
- Validation or restore failures leave the import summary visible and show an
  error state/toast.
- Successful restore shows an in-summary success state and success toast.
- CSV wizard behavior and JSON duplicate preflight confirmation behavior remain
  unchanged.

## Non-Goals

- No row-level import rollback engine.
- No partial restore or merge-back logic.
- No duplicate auto-merge.
- No JSON mapping or browser preview.
- No broader import/export entities, custom fields, notes/tags, activities,
  audit, proposed actions, external clients, or permissions.
- No cloud/remote/scheduled import/export.
- No MCP, AI, sync-server behavior, encryption, release signing/notarization,
  or schema normalization.
- No raw SQL in the Tauri command layer or `crm_engine`.

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

- The sprint reused the existing backup API wrappers and added no new Tauri
  commands.
- `crm-core` restore semantics are unchanged.
- Import summary restore performs a full local database restore, not row-level
  import rollback.
- The Import/Export modal keeps the summary visible when restore fails so the
  user can see the backup path and retry or leave the modal intentionally.
