# Sprint 060: JSON Import Preview

Date: 2026-06-25
Branch: `codex/json-import-preview`

## Scope

Add a visible, read-only browser preview for supported JSON imports before the
duplicate preflight and destructive import confirmation steps.

Supported entities remain:

- contacts;
- deals;
- organizations.

## Changes

- Added side-effect-free JSON preview parsing in `crm-core` using the same flat
  field sets and row-number convention as JSON import/export.
- Added thin Tauri commands and typed frontend API wrappers for JSON preview.
- Updated the Import/Export modal so JSON file selection shows preview rows
  first, then keeps the existing duplicate preflight, duplicate-warning review,
  explicit confirmation, automatic pre-import backup, and import-summary restore
  flow.
- Updated import/export documentation so JSON browser preview is implemented
  while JSON field mapping remains not implemented.

## Verification

Planned sprint verification:

- `npm run lint`
- `npm run check`
- `npm run test`
- `npm run build`
- `npm run test:e2e`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo check --workspace`
- `cargo test --workspace`
- raw SQL scans in `apps/desktop/src-tauri/src/commands` and
  `crates/crm-core/src/crm_engine`
- `git diff --check main...HEAD`
- `git fsck --full --no-progress`

## Non-Goals

- No JSON field mapping.
- No duplicate auto-merge.
- No row-level rollback, partial restore, merge-back, or restore behavior
  changes.
- No custom field, separate note record, tag, activity, audit-log,
  proposed-action, external-client, permission, settings, or relationship
  import/export expansion.
- No remote/cloud/scheduled import/export, sync-server upload/download, MCP
  behavior, AI behavior, encryption, release signing/notarization, schema
  normalization, or broad UI redesign.
