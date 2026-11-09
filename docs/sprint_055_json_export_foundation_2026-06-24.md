# Sprint 055 - JSON Export Foundation

Date: 2026-06-24
Branch: `codex/json-export-foundation`
Scope: JSON export for the existing Import/Export entities: contacts, deals, and organizations.

## Summary

- Added `crm-core` JSON export methods for contacts, deals, and organizations.
- Reused the existing CSV export row shapes and listing paths so JSON export stays limited to the current flat import/export fields.
- Added thin Tauri commands and typed frontend wrappers for JSON export.
- Added generic frontend export dispatch for `csv` and `json`.
- Updated the Import/Export modal so JSON opens a `.json` save dialog and runs the JSON export path.
- Updated public import/export documentation to describe current JSON export scope.

## Behavior Decisions

- JSON export writes local pretty-printed arrays of objects.
- JSON contact export includes `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`, `city`, `country`, and `notes`.
- JSON deal export includes `title`, `value`, `currency`, `stage`, `expected_close`, and `notes`; `value` uses the same two-decimal string formatting as CSV export.
- JSON organization export includes `name`, `email`, `phone`, `website`, `address_line1`, `address_line2`, `city`, `region`, `country`, `postal_code`, and `description`.
- JSON export uses the same active-row boundaries as CSV export.

## Non-Goals

- No JSON import.
- No CSV import/export semantic changes.
- No custom field, separate note record, tag, activity, audit log, proposed-action, external-client, permission, backup, settings, or related-row export.
- No duplicate auto-merge, import rollback, automatic backup before import, remote/cloud destinations, scheduled export, MCP/AI behavior, sync server behavior, app encryption, release signing/notarization, or schema normalization.
- No raw SQL in Tauri commands or `crm_engine`.

## Validation

- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `npm run test:e2e`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] Raw SQL scan in `apps/desktop/src-tauri/src/commands`
- [x] Raw SQL scan in `crates/crm-core/src/crm_engine`
- [x] `git diff --check main...HEAD`
- [x] `git fsck --full --no-progress`
