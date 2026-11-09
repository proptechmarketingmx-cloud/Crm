# Sprint 047 - Unencrypted Export And Backup Warning Surface

Date: 2026-06-24
Branch: `codex/unencrypted-export-backup-warning`
Scope: Settings Data Management warning copy for unencrypted local CSV exports
and backup folders, plus matching documentation.

## Summary

- Added visible, pre-action disclosure copy near `Export All Data` in Settings
  Data Management.
- Added visible, pre-action disclosure copy near the Backup & Restore controls
  in Settings Data Management.
- Kept the copy informational and non-blocking; export, backup, validation, and
  restore flows keep their existing behavior.
- Added i18n keys across every locale JSON file. English copy is used
  consistently where verified translations are not available.
- Added focused frontend source coverage for warning key parity and Settings
  placement before the export and create-backup actions.

## Changed Files

- `apps/desktop/src/routes/Settings.svelte`
- `apps/desktop/src/lib/api/settings.test.ts`
- `apps/desktop/src/lib/i18n/ar.json`
- `apps/desktop/src/lib/i18n/bn.json`
- `apps/desktop/src/lib/i18n/en.json`
- `apps/desktop/src/lib/i18n/es.json`
- `apps/desktop/src/lib/i18n/fr.json`
- `apps/desktop/src/lib/i18n/ha.json`
- `apps/desktop/src/lib/i18n/hi.json`
- `apps/desktop/src/lib/i18n/pt.json`
- `apps/desktop/src/lib/i18n/sw.json`
- `apps/desktop/src/lib/i18n/vi.json`
- `docs/PRIVACY.md`
- `docs/BACKUP_RESTORE.md`
- `docs/IMPORT_EXPORT.md`
- `docs/sprint_047_unencrypted_export_backup_warning_2026-06-24.md`
- `docs/sprint_ledger.md`

## Verification

- `npm ci` - completed from the lockfile to populate this sprint worktree's
  missing dependencies. npm reported existing audit findings: 16
  vulnerabilities (1 low, 9 moderate, 5 high, 1 critical). No package
  manifests or lockfiles were changed.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed, 17 test files and 80 tests.
- `npm run test:e2e` - passed, 6 Playwright Chromium smoke tests.
- `git diff --check` - passed with no whitespace errors before commit.
- `git status --short --branch` - reviewed before commit; only sprint files
  were modified or added.
- `git fsck --full --no-progress` - passed before commit; reported known
  dangling commit `10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
- Full Cargo verification skipped because this sprint did not touch Rust,
  Cargo files, schemas, or native Tauri automation.

## Test Gap

- There is no component-render test harness for Settings in the current
  frontend unit test setup. Coverage is source-checked for Settings placement
  and locale key parity, and the existing Playwright smoke suite covers route
  rendering but does not assert this specific copy.

## Non-Goals

- No encryption was added.
- No backend export or backup behavior changed.
- No backup validation or restore messaging changed.
- No Rust source, schemas, Cargo files, MCP, AI, sync server, release packaging,
  CI workflow, or native Tauri automation changed.
