# Sprint 073 - External Clients Export Foundation

Date: 2026-06-25
Branch: `codex/external-clients-export-foundation`

## Scope

Reduce the remaining local/offline import/export gap for external-client
readiness placeholders by adding export-only external client support.

Implemented behavior:

- External clients can be exported to CSV and JSON from the existing
  Import/Export export tab.
- External client export includes current storage fields: `id`, `name`,
  `client_type`, `permission_mode`, `enabled`, `created_at`, `updated_at`,
  `deleted_at`, and `device_id`.
- Rows are filtered to non-deleted external client placeholders and exported in
  stable chronological order: `created_at` ascending, then external client `id`
  ascending. `enabled` is runtime activation state, not export eligibility, so
  disabled non-deleted readiness placeholders remain exportable for diagnostics.
- External client export is read-only. It does not record audit rows, sync
  changelog rows, or pre-import backups.
- SQL for external client export remains in `crm-core` storage; Tauri commands
  are thin wrappers around `crm-core` service methods.
- Frontend API types keep external clients out of importable entities while
  exposing them as an export entity. The import entity selector shows a disabled
  export-only external clients option.

## Non-Goals

- No external client import, preview, preflight, mapping UI, backup, rollback,
  activation, enable/disable control, token creation, secret handling, or
  permission grant through import/export.
- No `external_client_permissions` import/export.
- No proposed action, audit log, settings, sync changelog, backup metadata, or
  broad relationship import/export behavior changes.
- No remote/cloud/scheduled import/export.
- No MCP runtime, AI behavior, sync-server behavior, client connection behavior,
  or secret handling.
- No destructive migration, schema rewrite, release packaging, signing,
  notarization, or unrelated UI redesign.

## Verification

Completed locally on the Sprint 073 worktree:

- `npm ci` - passed; installed frontend dependencies from the committed
  lockfile and reported existing audit findings.
- `npm run lint` - passed.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed: 20 files, 149 tests.
- `npm run build` - passed. Vite emitted the existing browser compatibility
  notices for externalized `node:async_hooks`; SvelteKit emitted the existing
  fallback overwrite message.
- `npm run test:e2e` - passed: 6 Chromium tests. The web server emitted the
  existing `NO_COLOR`/`FORCE_COLOR` warning.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: 136 `crm-core` tests, 12 Tauri library
  tests, and doc-tests with 17 ignored examples.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no quoted SQL matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no quoted SQL matches.
- `git diff --check` - passed with no whitespace errors.
- `git fsck --full --no-progress` - exited 0 with existing output:
  `dangling commit 10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
