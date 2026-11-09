# Sprint 068 - Generic Notes Import/Export Foundation

Date: 2026-06-25
Branch: `codex/notes-import-export-foundation`

## Scope

Close the local/offline import/export gap for generic note records stored in
the `notes` table.

Implemented behavior:

- Generic notes can be exported to CSV and JSON with `entity_type`,
  `entity_id`, and `content`.
- Generic note imports support CSV and JSON, including mapped source fields in
  the existing Import/Export modal.
- Supported note parent types are `contact`, `organization`, `deal`, and
  `activity`.
- Note import preflight is read-only and validates required fields, supported
  `entity_type` values, and active local parent row existence before import.
- Note imports create rows through `CrmCore::create_note`, preserving normal
  note validation, audit, and sync changelog behavior.
- Note duplicate preflight returns zero warnings because there is no established
  safe duplicate rule for generic notes.
- Completed note imports return a row-level rollback plan for created notes.
  Rollback soft-deletes created notes through `delete_note` after confirming the
  active note still matches the post-import snapshot.
- Frontend API command maps and the existing Import/Export modal expose Notes as
  a selectable CSV/JSON import/export entity without adding a notes route or
  broad notes UI redesign.

## Non-Goals

- No legacy flat `contacts.notes` or `deals.notes` import/export changes beyond
  their existing contact/deal flat row behavior.
- No tag definition or tag link import/export.
- No audit log, proposed actions, external clients, permissions, settings,
  backup metadata, sync changelog, or custom field definition import/export.
- No remote/cloud/scheduled import/export.
- No MCP runtime, AI behavior, sync-server behavior, release packaging,
  destructive schema migration, custom field behavior changes, or broad UI
  redesign.
- No importing or exporting note IDs, timestamps, deleted rows, or device IDs.

## Verification

Completed locally on the Sprint 068 worktree:

- `npm run lint` - passed.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed: 20 files, 133 tests.
- `npm run build` - passed. Vite emitted the existing browser compatibility
  notices for externalized `node:async_hooks`.
- `npm run test:e2e` - passed: 6 Chromium tests. The web server emitted the
  existing `NO_COLOR`/`FORCE_COLOR` warning.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: 126 `crm-core` tests, 10 Tauri library
  tests, and doc-tests with 17 ignored examples.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no quoted SQL
  keyword matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no quoted SQL keyword
  matches.
- `git diff --check` - passed with no whitespace errors.
- `git fsck --full --no-progress` - exited 0 with existing output:
  `dangling commit 10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
