# Sprint 026 - Import Preflight Duplicate Detection Foundation

Date: 2026-06-24
Branch: `codex/import-preflight-duplicate-detection-foundation`
Scope: Backend/API foundation for CSV import preflight duplicate warnings for contacts and organizations.

## Summary

- Added serializable import preflight report DTOs with entity type, source CSV row number, match type, matched CSV value, existing entity reference, display label, reason, total row count, and duplicate warning count.
- Added read-only `CrmCore::preflight_contacts_csv_import` and `CrmCore::preflight_organizations_csv_import` methods.
- Reused CSV parsing helpers while preserving source row numbers including the header offset.
- Added repository helpers for active contact phone matches and active organization name, email, and phone matches.
- Contact preflight flags existing email and phone duplicates.
- Organization preflight flags existing name, email, and phone duplicates.
- Added thin Tauri commands and typed frontend API wrappers for the new preflight commands.
- Added focused Rust and Vitest coverage for duplicate warning reports and invoke payload mapping.

## Boundaries

- Full field mapping wizard UI remains follow-up work.
- Duplicate warning rendering remains follow-up work.
- Confirm-import flow remains follow-up work.
- No actual import semantics were changed.
- No destructive migrations, legacy column removals, MCP behavior, AI behavior, or sync server behavior were changed.
- Tauri commands remain thin and contain no SQL or business logic.
- No direct `invoke()` calls were added to Svelte components.
- No Svelte runes were added to plain `.ts` files.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check main...HEAD`
- [x] Raw SQL scan in Tauri commands and `crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct `invoke()` scan in Svelte components/routes
