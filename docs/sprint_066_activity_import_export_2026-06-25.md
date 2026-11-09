# Sprint 066 - Activity Import/Export

Date: 2026-06-25
Branch: `codex/activity-import-export`

## Scope

Close the local import/export gap for activities while staying within the
existing local desktop import/export surface.

Implemented behavior:

- Activity CSV and JSON import/export through `crm-core`, thin Tauri commands,
  typed frontend API wrappers, and the existing Import/Export modal.
- Flat activity fields: `activity_type`, `title`, `description`, `due_date`,
  `completed`, `contact_id`, and `deal_id`.
- Activity `contact_id` and `deal_id` are optional local database IDs only.
  Imports validate them as existing active IDs through the existing activity
  create service path. No lookup by name or portable relationship resolution is
  attempted.
- Activity imports create rows through `CrmCore::create_activity`; truthy
  `completed` values then route through the normal activity completion service.
- Activity custom field values use the existing `custom:` target convention and
  existing custom field value storage because that storage supports
  `activity`.
- Mapped CSV/JSON imports reuse the existing mapping wizard and JSON preview.
- Activity preflight validates parsing, mappings, and custom-field targets, but
  returns zero duplicate warnings because no safe activity duplicate rule exists.
- Automatic pre-import backup wraps activity imports like other supported
  desktop imports.
- Row-level rollback supports created activity rows and activity custom field
  values with current-vs-post-import conflict checks, reusing the existing
  `delete_activity` and custom-field rollback service behavior.

## Non-Goals

- No organization custom fields.
- No activity duplicate auto-merge.
- No lookup-by-name relationship import/export.
- No broad relationship import/export beyond optional local activity
  `contact_id` and `deal_id` columns.
- No notes, tags, audit log, proposed actions, external clients, permissions,
  settings, or backup metadata import/export.
- No remote/cloud/scheduled import/export, MCP behavior, AI behavior,
  sync-server transport, schema normalization, encryption, release signing, or
  notarization.
- No broad UI redesign.

## Verification

Completed by the Sprint 066 builder:

- `npm run lint` - passed.
- `npm run check` - passed.
- `npm run test` - passed: 20 files, 127 tests.
- `npm run build` - passed.
- `npm run test:e2e` - passed: 6 Playwright tests.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: `crm-core` 120 tests,
  `ninehundredcrm_lib` 10 tests, doc tests ignored as before.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no matches.
- `git diff --check main...HEAD` - passed.
- `git fsck --full --no-progress` - passed with only the known dangling commit
  `10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
- `git status --short --branch` - branch `codex/activity-import-export` with
  Sprint 066 changes pending commit at verification time.
