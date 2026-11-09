# Sprint 008 — Custom Field Filters

Date: 2026-03-05 (UTC)
Branch: `sprint-008-custom-field-filters`

## Scope
- Deliver custom-field filtering across contacts list, pipeline board, and activities list.
- Keep filtering lightweight for low-resource hardware and offline-first behavior.
- Preserve pagination correctness for contact filtering.

## Changes
- Added new backend custom-field bulk lookup command for filter indexing:
  - `list_custom_field_values_for_type` (Tauri command)
  - Implemented storage query for entity-type scoped custom-field values in:
    - `src-tauri/src/storage/custom_fields.rs`
    - `src-tauri/src/commands/custom_field_commands.rs`
    - `src-tauri/src/lib.rs`
- Added backend contact custom-field filtering support in paginated list queries:
  - Extended `ContactListParams` with:
    - `custom_field_def_id`
    - `custom_field_query`
  - Applied filters in both standard and FTS-backed paginated list paths.
  - Updated compatibility initializer in import/export command.
  - Files:
    - `src-tauri/src/storage/contacts.rs`
    - `src-tauri/src/commands/import_export.rs`
- Added frontend API contract updates:
  - `src/lib/api/contacts.ts` (custom-field list params pass-through)
  - `src/lib/api/customFields.ts` (bulk values endpoint wrapper)
- Added custom-field filter controls to route UIs:
  - `src/routes/Contacts.svelte`
    - Backend-filtered custom-field selector + value search integrated with existing list filters.
  - `src/routes/Pipeline.svelte`
    - Custom-field filter selector + value input; stage columns filter deals locally.
  - `src/routes/Activities.svelte`
    - Custom-field filter selector + value input; activity list filters locally.
- Added i18n keys required by filter UI across supported locales:
  - `src/lib/i18n/en.json`
  - `src/lib/i18n/es.json`
  - `src/lib/i18n/fr.json`
  - `src/lib/i18n/ar.json`
  - `src/lib/i18n/sw.json`
  - `src/lib/i18n/hi.json`

## Validation
- `npm run check` -> passed (0 errors, 0 warnings)
- `cargo check --target-dir /tmp/900crm-sprint-008-target` (in `src-tauri/`) -> passed

## Lightweight Guardrail Checklist
- [x] `Offline-first` remains intact: no feature requires internet to function locally.
- [x] `Local-first` remains intact: data stays in SQLite/local storage.
- [x] No mandatory proprietary/cloud dependency was introduced.
- [x] No heavy new background polling/jobs were added.
- [x] Contact filtering remains paginated at DB layer; pipeline/activity filters use lightweight in-memory maps.
- [x] No charting/UI dependency bloat introduced.
- [x] Changelog and sprint ledger updated chronologically with UTC date.

## Outcome
- Users can now filter contacts, deals, and activities by custom-field values with consistent UX.
- The prior v1.1.0 in-progress item for custom-field filtering is now implemented.
