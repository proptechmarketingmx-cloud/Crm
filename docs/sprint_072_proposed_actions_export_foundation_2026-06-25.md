# Sprint 072 - Proposed Actions Export Foundation

Date: 2026-06-25
Branch: `codex/proposed-actions-export-foundation`

## Scope

Reduce the remaining local/offline import/export gap for diagnostic proposed
action state by adding export-only proposed action support.

Implemented behavior:

- Proposed actions can be exported to CSV and JSON from the existing
  Import/Export export tab.
- Proposed action export includes read-only diagnostic fields: `id`,
  `external_client_id`, `client_id`, `tool_name`, `action_type`, `entity_type`,
  `entity_id`, `payload_json`, `input_json`, `proposed_output_json`, `status`,
  `created_at`, `decided_at`, `approved_at`, `rejected_at`, `executed_at`,
  `error_message`, and `device_id`.
- `external_client_id` mirrors the stored `client_id`; `payload_json` mirrors
  the stored `input_json`; `decided_at` is derived from `approved_at` or
  `rejected_at`; `error_message` is reserved blank/null because the current
  schema has no dedicated error-message column.
- Rows are exported in stable chronological order: `created_at` ascending, then
  proposed action `id` ascending.
- Proposed action export is read-only. It does not record audit rows, sync
  changelog rows, or pre-import backups.
- SQL for proposed action export remains in `crm-core` storage; Tauri commands
  are thin wrappers around `crm-core` service methods.
- Frontend API types keep proposed actions out of importable entities while
  exposing them as an export entity. The import entity selector shows a disabled
  export-only proposed actions option.

## Non-Goals

- No proposed action import, preview, preflight, mapping UI, backup, rollback,
  replay, approval, rejection, execution, or mutation through import/export.
- No audit log behavior changes.
- No external clients, permissions, settings, sync changelog, backup metadata,
  or broad relationship import/export.
- No remote/cloud/scheduled import/export.
- No MCP runtime, AI behavior, sync-server behavior, token handling, permission
  grant, or secret handling.
- No destructive migration, schema rewrite, release packaging, signing,
  notarization, or unrelated UI redesign.

## Verification

Completed locally on the Sprint 072 worktree:

- `npm ci` - passed; installed frontend dependencies from the committed
  lockfile and reported existing audit findings.
- `npm run lint` - passed.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed: 20 files, 147 tests.
- `npm run build` - passed. Vite emitted the existing browser compatibility
  notices for externalized `node:async_hooks`.
- `npm run test:e2e` - passed: 6 Chromium tests. The web server emitted the
  existing `NO_COLOR`/`FORCE_COLOR` warning.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: 135 `crm-core` tests, 12 Tauri library
  tests, and doc-tests with 17 ignored examples.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no quoted SQL matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no quoted SQL matches.
- `git diff --check` - passed with no whitespace errors.
- `git fsck --full --no-progress` - exited 0 with existing output:
  `dangling commit 10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
