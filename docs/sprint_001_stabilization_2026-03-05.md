# Sprint 001 — Stabilization and Ship Readiness

Date: 2026-03-05 (UTC)
Branch: `sprint-001-stabilization-ship-readiness`

## Scope
- Fix frontend and Tauri build blockers.
- Unify frontend API wrappers with backend command contracts.
- Align route rendering, import/export command usage, and settings/sync behavior.
- Validate with local checks.

## Changes
- Fixed Svelte compile blockers in search/tag picker event handlers and modal snippet typing.
- Added size-aware modal support and body/children snippet compatibility.
- Fixed Tauri config schema incompatibility and provided a valid app icon.
- Reworked API wrapper mappings for contacts, deals, activities, dashboard, settings, and sync.
- Added backend commands: `mark_activity_incomplete`, `get_sync_status`, `trigger_sync`.
- Added lightweight sync command module and registration in Tauri invoke handler.
- Updated hash-route rendering in `src/routes/+page.svelte` to support all main views and contact detail deep links.
- Updated import/export modal to use backend CSV commands with file-path workflows.

## Validation
- `npm run check` -> passed (warnings only).
- `npm run build` -> passed.
- `cargo check` (in `src-tauri/`) -> passed (warnings only).

## Known Non-Blocking Warnings
- Existing accessibility/style warnings remain in some Svelte components and CSS selectors.
- Existing unused import warnings remain in backend utility modules.
