# Sprint 064 - Import Row-Level Rollback

Date: 2026-06-25
Branch: `codex/import-row-level-rollback`

## Scope

Close the documented import/export gap for row-level rollback of a completed
multi-row local desktop import.

Implemented scope:

- Contact, deal, and organization CSV, mapped CSV, JSON, and mapped JSON imports
  now return an optional summary-scoped rollback plan.
- Created import rows can be rolled back by soft-deleting the created record
  through the existing entity delete service.
- Duplicate auto-merge rows can be rolled back by restoring only the flat fields
  that the import merge actually changed.
- Rollback compares the current active record to the recorded post-import state
  before applying each row action. Mismatches are skipped with row-level error
  details rather than overwritten.
- Rollback reports `rolled_back`, `skipped`, and row-level `errors` back to the
  caller.
- The Tauri command layer remains thin and delegates rollback to `crm-core`.
- The Import/Export summary shows a separate row-level rollback control with
  explicit confirmation and disables it after a successful rollback command
  response.
- The existing destructive pre-import backup restore control remains available
  as a separate full-database escape hatch.

## Semantics

Rollback plans are self-contained and are returned with the import summary.
They are not persisted across app restart and are not available after the
summary is dismissed.

Created-row rollback soft-deletes only the row ID recorded by the import plan.
Merge rollback routes through the normal update services and sends updates only
for fields changed by the import merge.

Conflict checks are intentionally conservative. A row is skipped if the current
active record differs from the recorded post-import snapshot. This prevents
row-level rollback from overwriting post-import user edits or deleting a row
that no longer matches the imported state.

## Non-Goals

- No persistence of rollback plans across app restart.
- No rollback for imports not represented by the current summary.
- No full database restore behavior changes.
- No rollback of relationships, custom fields, notes-as-records, tags,
  activities, audit logs, sync changelog rows, proposed actions, external
  clients, permissions, settings, or backup metadata.
- No remote/cloud/scheduled import/export.
- No MCP, AI, sync-server behavior, encryption, release signing/notarization,
  release packaging, or schema normalization.

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

- Default import behavior stays unchanged when rollback is not requested.
- Backup creation before import remains the first write guard for supported
  desktop imports.
- Row-level rollback is documented as narrower than automatic pre-import backup
  restore and does not replace that full-state recovery path.
