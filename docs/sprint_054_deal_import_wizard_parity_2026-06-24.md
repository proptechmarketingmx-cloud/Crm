# Sprint 054 - Deal Import Wizard Parity

Date: 2026-06-24
Branch: `codex/deal-import-wizard-parity`
Scope: Deal CSV import parity with the existing mapped contact and organization import wizard.

## Summary

- Added mapped deal CSV import support for `title`, `value`, `currency`, `stage`, `expected_close`, and `notes`.
- Added read-only deal import preflight warnings for exact case-insensitive duplicate title matches.
- Exposed thin Tauri commands and typed frontend API wrappers for deal mapped import and direct/mapped deal preflight.
- Routed deals through the existing import wizard flow for CSV preview, field mapping, duplicate warnings, confirmation, and summary.
- Updated import/export documentation to remove the previous deal mapping and preflight gap.

## Behavior Decisions

- Deal duplicate preflight is deterministic and title-only. It checks active deals by trimmed imported title using an exact case-insensitive match.
- Duplicate warnings are informational. They do not block import, auto-merge records, or change existing deals.
- Deal imports continue to create records through the normal `crm-core` `create_deal` service path, preserving create validation, sync changelog entries, and audit behavior.
- Relationship fields are intentionally not mapped. CSV deal imports still pass `None` for `contact_id` and `organization_id` because there is no narrow, established CSV lookup contract for resolving those relationships by label.
- JSON export behavior remains unchanged and unimplemented.

## Non-Goals

- No JSON export.
- No duplicate auto-merge.
- No import rollback.
- No automatic backup before import.
- No custom field, notes, tags, activity, audit, proposed-action, external-client, or permission import/export.
- No MCP, AI, sync server, schema normalization, release packaging, encryption, broad UI redesign, or runtime behavior outside deal import wizard parity.

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
