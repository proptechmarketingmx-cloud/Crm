# Sprint 014 — Backup UI/API/Docs Surface

Date: 2026-06-23 (UTC)
Branch: `codex/backup-ui-surface-preserve`

## Scope
- Expose the existing local backup foundation through frontend API wrappers.
- Add focused frontend invoke-mapping tests for backup commands.
- Add Settings/Data Management controls for create, validate, and confirmed restore.
- Document backup and restore behavior without adding sync, MCP, AI, or migration behavior.

## Changes
- Added frontend backup API wrappers:
  - `apps/desktop/src/lib/api/backup.ts`
  - Commands: `create_local_backup`, `validate_local_backup`, `restore_local_backup_to_app_data`
  - Restore wrapper requires an explicit destructive confirmation flag.
- Added API mapping coverage:
  - `apps/desktop/src/lib/api/backup.test.ts`
  - Verifies command names and snake_case Tauri argument payloads.
- Extended Settings/Data Management:
  - `apps/desktop/src/routes/Settings.svelte`
  - Uses the existing Tauri dialog plugin for directory selection.
  - Restore button is enabled only after validation for the selected folder.
  - Restore validates again, then asks for explicit user confirmation before invoking the destructive command with `confirm_destructive_restore: true`.
- Added backup UI i18n keys across all registered locale files:
  - `en`, `fr`, `es`, `ar`, `sw`, `hi`, `pt`, `vi`, `ha`, `bn`
- Added backup/restore documentation:
  - `docs/BACKUP_RESTORE.md`
  - README discoverability updates.

## Validation
- `npm run check` -> passed, `svelte-check found 0 errors and 0 warnings`
- `npm run test` -> passed, `src/lib/api/backup.test.ts` 4 tests passed
- `npm run build` -> passed
- `cargo fmt --all -- --check` -> passed
- `cargo check --workspace` -> passed
- `cargo test --workspace` -> passed, 20 unit tests passed

## Preservation Notes
- Email settings remain in `Settings.svelte` and `apps/desktop/src/lib/api/email.ts`.
- Desktop reminders remain in `Settings.svelte` and the activity reminder service remains wired from layout startup.
- Reports/dashboard remain in `Dashboard.svelte` with `apps/desktop/src/lib/api/reports.ts`.
- Custom-field API/UI files remain in place.
- Locale files `pt`, `vi`, `ha`, `bn`, `ar`, `es`, `fr`, `hi`, and `sw` remain registered and present.
- Multi-currency display remains wired through settings currency selection plus dashboard/deal currency helpers.

## Guardrails
- [x] `Offline-first` remains intact: backup and restore operate on local files only.
- [x] `Local-first` remains intact: no cloud storage or telemetry added.
- [x] No MCP server behavior added.
- [x] No AI behavior added.
- [x] No sync server behavior added.
- [x] No destructive migration added.
- [x] Current main Settings sections were preserved while adding backup controls.
