# Sprint 070 - Custom Field Definition Import/Export

Date: 2026-06-25
Branch: `codex/custom-field-definition-import-export`

## Scope

Close the local/offline import/export gap for custom field definitions while
leaving custom field value import/export behavior unchanged.

Implemented behavior:

- Custom field definitions can be exported to CSV and JSON with `entity_type`,
  `field_name`, `field_type`, `field_options`, and `sort_order`.
- Definition imports support CSV and JSON, including mapped source fields in
  the existing Import/Export modal.
- Definition import creates missing definitions through
  `CrmCore::create_custom_field_def`, preserving normal custom field
  validation, audit, and sync changelog behavior.
- Existing exact definitions are skipped. Definitions with the same
  `entity_type` and `field_name` but different type, options, or order are
  reported as row errors rather than updated silently.
- Definition preflight is read-only. It validates required fields, supported
  entity and field types, select options, integer `sort_order`, and conflicting
  existing definition shapes.
- Desktop definition imports use the existing automatic pre-import backup
  guard.
- Completed definition imports return row-level rollback plans for created
  definitions. Rollback deletes a created definition only when the active
  definition still matches the post-import snapshot and no custom field values
  reference it.
- Frontend API command maps and the existing Import/Export modal expose Custom
  Field Definitions as a selectable CSV/JSON import/export entity without a
  broader UI redesign.

## Non-Goals

- No custom field value import/export changes.
- No remote/cloud/scheduled import/export.
- No MCP runtime, AI behavior, sync-server behavior, release packaging, token
  handling, or secret handling.
- No destructive migration, legacy column removal, broad schema normalization,
  audit log import/export, proposed actions import/export, external clients,
  permissions, settings, sync changelog import/export, or backup metadata
  import/export.
- No opt-in mutation of existing custom field definitions.
- No importing or exporting local definition IDs, timestamps, deleted rows, or
  device IDs.

## Verification

Completed locally on the Sprint 070 worktree:

- `npm run lint` - passed.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed: 20 files, 143 tests. Existing test stderr noted
  missing `common.close` i18n keys in `Modal.test.ts`.
- `npm run build` - passed. Vite emitted the existing browser compatibility
  notices for externalized `node:async_hooks`.
- `npm run test:e2e` - passed: 6 Chromium tests. The web server emitted the
  existing `NO_COLOR`/`FORCE_COLOR` warning.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: 133 `crm-core` tests, 12 Tauri library
  tests, and doc-tests with 17 ignored examples.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no keyword matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no keyword matches.
- `git diff --check` - passed with no whitespace errors.
- `git fsck --full --no-progress` - exited 0 with existing output:
  `dangling commit 10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
