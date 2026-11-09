# Sprint 056 - JSON Import Foundation

Date: 2026-06-24
Branch: `codex/json-import-foundation`
Scope: JSON import for the existing Import/Export entities: contacts, deals, and organizations.

## Summary

- Added `crm-core` JSON import methods for contacts, deals, and organizations.
- Reused the flat row shapes produced by Sprint 055 JSON export.
- Routed JSON rows through the same private row import helpers and create paths as CSV import.
- Added thin Tauri commands and typed frontend wrappers for JSON import.
- Added generic frontend import dispatch for `csv` and `json`.
- Updated the Import/Export modal so CSV keeps the preview, mapping, duplicate preflight, confirmation, and summary wizard while JSON uses a direct `.json` file import path with summary results.
- Updated public import/export documentation to describe current JSON import scope and limitations.

## Behavior Decisions

- JSON import accepts a top-level array of flat objects.
- Contact JSON import accepts `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`, `city`, `country`, and `notes`.
- Deal JSON import accepts `title`, `value`, `currency`, `stage`, `expected_close`, and `notes`.
- Organization JSON import accepts `name`, `email`, `phone`, `website`, `address_line1`, `address_line2`, `city`, `region`, `country`, `postal_code`, and `description`.
- Blank required fields are skipped before create attempts, matching CSV import behavior.
- JSON row numbers use the same data-row offset as CSV import: the first JSON array item is reported as row 2.
- JSON import is direct desktop-file import only in this sprint.

## Non-Goals

- No JSON duplicate preflight, mapping, browser preview, or import wizard beyond direct selected-file import.
- No CSV import/export semantic changes.
- No custom field, separate note record, tag, activity, audit log, proposed-action, external-client, permission, backup, settings, relationship, or metadata import/export.
- No duplicate auto-merge, import rollback, automatic backup before import, remote/cloud destinations, scheduled import/export, MCP/AI behavior, sync server behavior, app encryption, release signing/notarization, or schema normalization.
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
