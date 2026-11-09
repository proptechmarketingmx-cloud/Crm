# Sprint 025 - Import/Export API Organization CSV Foundation

Date: 2026-06-24
Branch: `codex/import-export-api-organization-csv-foundation`
Scope: Organization CSV import/export foundation plus typed frontend Import/Export API wrappers.

## Summary

- Added organization CSV row helpers for import/export with `name` as the required field.
- Aligned organization CSV optional fields with the existing organization service and storage shape: `email`, `phone`, `website`, `address_line1`, `address_line2`, `city`, `region`, `country`, `postal_code`, and `description`.
- Added `CrmCore::import_organizations_csv` and `CrmCore::export_organizations_csv`.
- Routed organization CSV imports through `create_organization(...)` so existing validation, audit, and sync semantics remain the source of truth.
- Added thin Tauri commands for organization CSV import/export and registered them with the existing command handler.
- Added `apps/desktop/src/lib/api/importExport.ts` with typed wrappers for contact, deal, and organization import/export commands.
- Refactored the Import/Export modal away from direct Tauri `invoke()` calls and added organizations to import/export selectors.
- Added focused Rust and Vitest coverage for the new CSV and frontend invoke-mapping behavior.

## Boundaries

- No full field mapping wizard was built in this sprint.
- Richer duplicate detection, duplicate previews, and duplicate warning UX remain follow-up work.
- No destructive migrations, legacy column removals, MCP behavior, AI behavior, sync server behavior, or broad Import/Export UI redesign was changed.
- Tauri import/export commands remain thin orchestration only and contain no SQL or business logic.
- Svelte components continue to call frontend API wrappers rather than direct Tauri `invoke()` calls.
- No Svelte runes were added outside `.svelte.ts` files.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check main...HEAD`
- [x] Raw SQL scan in `apps/desktop/src-tauri/src/commands`
- [x] Raw SQL scan in `crates/crm-core/src/crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct `invoke()` scan in Svelte components/routes
