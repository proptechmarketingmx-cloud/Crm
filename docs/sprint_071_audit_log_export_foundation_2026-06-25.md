# Sprint 071 - Audit Log Export Foundation

Date: 2026-06-25
Branch: `codex/audit-log-export-foundation`

## Scope

Reduce the remaining local/offline import/export gap for accountability data by
adding export-only audit log support.

Implemented behavior:

- Audit log entries can be exported to CSV and JSON from the existing
  Import/Export export tab.
- Audit log export includes the full existing audit row fields: `id`,
  `actor_type`, `actor_id`, `action`, `entity_type`, `entity_id`,
  `before_json`, `after_json`, `created_at`, and `device_id`.
- Audit rows are exported in stable chronological order: `created_at` ascending,
  then audit row `id` ascending.
- Audit log export is read-only. It does not record a new audit row and does
  not create a pre-import backup.
- SQL for audit export remains in `crm-core` storage; Tauri commands are thin
  wrappers around `crm-core` service methods.
- Frontend API types separate importable entities from export entities so
  `audit_log` is available only for export.

## Non-Goals

- No audit log import.
- No audit log preview, preflight, mapping UI, rollback, or pre-import backup.
- No proposed actions, external clients, permissions, settings, sync changelog,
  backup metadata, or broad relationship import/export.
- No remote/cloud/scheduled import/export.
- No MCP runtime, AI behavior, sync-server behavior, token handling, or secret
  handling.
- No destructive migration, schema rewrite, release packaging, signing,
  notarization, or unrelated UI redesign.

## Verification

Completed locally on the Sprint 071 worktree:

- `npm run lint` - passed after installing missing `node_modules` with
  `npm ci`.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed: 20 files, 145 tests.
- `npm run build` - passed. Vite emitted the existing browser compatibility
  notices for externalized `node:async_hooks`.
- `npm run test:e2e` - passed: 6 Chromium tests. The web server emitted the
  existing `NO_COLOR`/`FORCE_COLOR` warning.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: 134 `crm-core` tests, 12 Tauri library
  tests, and doc-tests with 17 ignored examples.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no keyword matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no keyword matches.
- `git diff --check` - passed with no whitespace errors.
- `git fsck --full --no-progress` - exited 0 with existing output:
  `dangling commit 10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
