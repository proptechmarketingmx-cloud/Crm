# Sprint 028 - Import Wizard UI Surface

Date: 2026-06-24
Branch: `codex/import-wizard-ui-surface`
Scope: Visible contact and organization CSV import wizard using the mapped import/preflight API from Sprint 027.

## Completed

- Replaced the contact and organization single-step import path with a staged wizard in `ImportExport.svelte`.
- Added CSV selection, preview rows, field mapping, duplicate-warning review, explicit import confirmation, and import summary states.
- Kept export behavior unchanged.
- Kept deal import/export on the legacy direct CSV path; mapped deal wizard support was not added.
- Used existing CSV parse, column suggestion, and mapping application helpers for preview and mapping UI support.
- Routed contact and organization duplicate detection and import through the mapped frontend API wrappers.
- Added UI validation for duplicate non-skip target assignments and required `first_name` / `name` mappings before preflight/import.
- Kept browser file-input fallback preview-only for mapped imports because Tauri import/preflight commands require a real desktop file path.
- Added focused Vitest coverage for import wizard mapping suggestion, validation, and backend mapping normalization helpers.

## Deferred

- Browser fallback execution for mapped preflight/import remains deferred; desktop file picker selection is still required.
- Deal mapping wizard support remains deferred; deals continue to use the legacy direct CSV import/export behavior.
- Richer duplicate resolution and merge actions remain deferred; this sprint only warns and allows explicit continuation.
- Merge contacts implementation remains deferred.

## Boundaries

- No backend, `crm-core`, Tauri command, migration, sync, MCP, or AI behavior was changed.
- No direct `invoke()` calls were added to Svelte components or routes.
- No raw SQL was added to frontend or Tauri UI code.
