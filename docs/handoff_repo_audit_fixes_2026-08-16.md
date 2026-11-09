# Handoff — Repo audit fixes

Date: 2026-08-16
Branch: `fix/repo-audit-findings` (from `main` at `20e40cb`)
Status: On `fix/repo-audit-findings`. Do not treat this as shipped until it is reviewed and merged.

This pass fixes the verified audit findings and the doc drift on current
`main`. 900CRM is still a source-evaluable 0.9.0 alpha, not an installable
product. Phase 6 (signed public installers) is still incomplete.

## What shipped

### Correctness

- Restore (Settings Data and Import rollback) now reloads in-memory stores
  through `reloadWorkspaceAfterDataReplace()` so the UI shows the restored
  database without a restart.
- Lead create writes the person and lifecycle in one transaction
  (`CrmCore::create_contact_with_lifecycle`). The Tauri `create_contact`
  command already accepted `lifecycle`; it now uses that single path.
- Pipeline follow-up guidance loads activities for the deals on the board
  (`list_activities_for_deals`) instead of the first 500 activity rows.
- The Pipeline custom-field value index reloads instead of returning the
  first cached map forever.
- Storage list helpers now surface row-mapping errors instead of dropping
  them with `filter_map(|r| r.ok())`.
- Contact merge reassignment SQL runs on the caller’s transaction instead of
  opening a nested one.

### Security

- Desktop import reads go through `read_import_text` (Rust path guard + size
  limit). The frontend no longer uses `@tauri-apps/plugin-fs` `readTextFile`.
  Browser FileReader remains only for the e2e/file-input fallback.
- Local-file bookmarks call `validate_open_path` before `plugin-shell` open.
- `path_guard` canonicalizes existing paths, rejects symlink files, and
  re-checks the denylist after resolve.
- `fs:allow-read-text-file`, `updater:default`, and `process:default` were
  removed from the default capability. `shell:allow-open` and
  `notification:default` stay.
- Setting keys are allowlisted. Password-like setting values are redacted in
  application logs.

### Performance / API

- `list_deals` / `list_activities` accept optional `limit`/`offset`. `None`
  still returns the full set (pipeline board and similar callers).
- Batch commands: `list_activities_for_deals`,
  `list_activity_links_for_activities`.
- Activity-link index loading is one invoke, grouped in the frontend.
- Sync pending counts rows with `synced_at IS NULL`. `trigger_sync` reports
  `get_sync_status()`. `clear_old_changes` only deletes already-synced rows.
- Thin wrappers: `restoreContact`, `updateSavedView`,
  `listActivitiesForDeals`, `listActivityLinksForActivities`.

### Tests / CI

- Rust: `merge_contacts_reassigns_related_records_and_soft_deletes_source`
  and `create_contact_with_lifecycle_writes_lead_in_one_step`.
- rust-cross now runs `cargo test --workspace` after clippy.
- Frontend mapping tests for the new wrappers.
- Browser shim covers the new invokes. App-shell smoke covers Review routes
  and the Settings Data restore button (visibility only; no SQLite replace).
- Contact-merge e2e was not added. The shim still returns empty duplicate
  candidates.

### Docs / i18n

- Schema citations say version 13.
- Deal workspace (`#/deals/:id`) and in-app reminders are treated as shipped.
- Audit log is documented as application-append-only, not a WORM store.
- README RTL claim is now: Arabic sets `dir=rtl`; layout polish is unfinished.
- Sidebar collapse uses `nav.collapse` / `nav.expandSidebar` /
  `nav.collapseSidebar` in all 10 locales.
- Leftover English labels in `ar`, `bn`, `fr`, and `ha` were translated.
  Product names, URL placeholders, and French/Hausa loanwords such as
  Pipeline were left as-is.
- `CHANGELOG.md` `[Unreleased]` has a 2026-08-16 Fixed section.

## What was deferred

- Splitting `crates/crm-core/src/services/mod.rs` (~4k) and
  `services/tests.rs` (~12k).
- Making rustsec a hard CI gate (`continue-on-error: true` stays).
- Linux glibc 2.39+ installer floor (cannot fix in-app).
- Audit-log hash chain / tamper-evident schema.
- macOS signing, notarization, and published installers.
- Organization list still loads the full table (no windowed org command).
- Reports / Contacts / Dashboard / Deal detail still use
  `listActivities({ pageSize: 200 })` rather than entity-scoped queries.
- Full calendar grid, recurrence, mailbox, report export, owner/source
  dimensions.
- Native Playwright e2e in this environment (Chromium was not installed in
  the local Playwright cache). CI installs Chromium before `npm run test:e2e`.

## How to verify

```sh
cargo test --workspace
cargo clippy --workspace -- -D warnings
npm test
npm run check
npx playwright install chromium
npm run test:e2e
```

Verified here on 2026-08-16:

- `cargo test --workspace` passed (including the new merge and lifecycle tests).
- `cargo clippy --workspace -- -D warnings` passed.
- `npm test` — 37 files, 231 tests passed.
- `npm run check` — 0 errors, 0 warnings.
- `npm run test:e2e` was not run to completion here (missing Playwright
  browser binary).

Manual checks still worth doing in the desktop app:

1. Settings → Data → restore a backup, then confirm Contacts/Pipeline show
   the restored rows without restarting.
2. Create a person as a lead and confirm it appears on Leads after a failed
   follow-up write cannot leave a half-created contact.
3. Import a CSV through the desktop file picker (not the browser fallback).
4. Open a local-file bookmark and confirm a path outside the intended tree
   is rejected.

## Remaining risk

- Restore reload is best-effort store refresh. Open detail routes may still
  hold a stale selected id until the user navigates away.
- Pipeline still lists all deals for the kanban board. That is intentional
  and unbounded.
- Path guard is stronger than lexical-only, but `plugin-shell` `open` can
  still hand a validated path to the OS handler.
- Setting allowlist will reject unknown keys. That is intended; any future
  setting must be added to `ALLOWED_SETTING_KEYS`.
- Locale files `ar.json`, `bn.json`, `fr.json`, and `ha.json` were rewritten
  with `JSON.stringify` (2-space). Key parity is 836 across all 10 locales.
  Native review of the new Arabic/Bengali/French/Hausa strings is welcome.
- rustsec remains advisory.

## Suggested next work

1. Review and commit this branch (not done here).
2. Run `npm run test:e2e` after `npx playwright install chromium`.
3. If product wants bounded org lists, add the same windowed SQL pattern
   used for deals/activities.
4. Keep Phase 6 / signing claims out of README until artifacts exist.
