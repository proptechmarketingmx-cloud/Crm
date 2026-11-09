# Sprint 067 - Organization Custom Fields Import/Export

Date: 2026-06-25
Branch: `codex/organization-custom-fields-import-export`

## Scope

Close the documented local/offline gap where organizations were excluded from
the custom-field validation and import/export surface.

Implemented behavior:

- Custom field definitions now support the `organization` entity type alongside
  `contact`, `deal`, and `activity`.
- Custom field value writes validate that the referenced entity exists and is
  active for the definition's entity type before writing.
- Organization custom field value writes reuse the existing custom field service
  path, preserving current audit and sync changelog behavior.
- The organization create/edit modal reuses `CustomFieldInputs.svelte` for
  organization custom field values.
- Organization CSV and JSON export include active organization custom field
  values using the existing `custom:<field_name>` and
  `custom:<field_name>#<field_id>` conventions.
- Direct and mapped organization CSV/JSON import accept supported organization
  custom field targets, and organization preflight validates those targets.
- Organization duplicate auto-merge fills missing or blank custom field values
  from incoming rows without overwriting existing non-empty custom values.
- Row-level rollback snapshots include organization custom field values, so
  created-row rollback removes imported organization custom values and
  auto-merge rollback restores or removes values changed by the import.

## Non-Goals

- No custom field definition import/export.
- No notes, tags, audit log, proposed actions, external clients, permissions,
  settings, or backup metadata import/export.
- No broader relationship import/export beyond the already supported activity
  local ID columns.
- No remote import/export endpoints, cloud export destinations, scheduled
  export, MCP runtime, AI behavior, sync-server behavior, release packaging,
  schema-destructive migration, or broad UI redesign.

## Verification

Completed locally by the Sprint 067 builder:

- `npm run lint` - passed.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed: 20 files, 128 tests.
- `npm run build` - passed. Vite emitted existing browser compatibility
  notices for externalized `node:async_hooks`.
- `npm run test:e2e` - passed: 6 Chromium tests. The web server emitted the
  existing `NO_COLOR`/`FORCE_COLOR` warning.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: 123 `crm-core` tests, 10 Tauri library
  tests, and doc-tests with 17 ignored examples.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no quoted SQL
  keyword matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no quoted SQL keyword
  matches.
- `git diff --check` - passed with no whitespace errors before commit.
- `git fsck --full --no-progress` - passed with existing output:
  `dangling commit 10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
