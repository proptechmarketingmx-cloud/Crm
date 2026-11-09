# Sprint 074 - External Client Permissions Export Foundation

Date: 2026-06-25
Branch: `codex/external-client-permissions-export-foundation`

## Scope

Reduce the remaining local/offline diagnostics gap for external-client
permission rows by adding export-only external client permission support.

Implemented behavior:

- External client permissions can be exported to CSV and JSON from the existing
  Import/Export export tab.
- Permission export includes current storage fields: `id`, `client_id`,
  `tool_name`, `can_read`, `can_write`, `requires_confirmation`, `created_at`,
  and `updated_at`.
- Rows include all existing permission rows and are exported in stable order:
  `client_id` ascending, `tool_name` ascending, `created_at` ascending, then
  permission `id` ascending.
- External client permission export is read-only. It does not record audit
  rows, sync changelog rows, or pre-import backups.
- SQL for permission export remains in `crm-core` storage; Tauri commands are
  thin wrappers around `crm-core` service methods.
- Frontend API types keep permission rows out of importable entities while
  exposing them as an export entity. The import entity selector shows a
  disabled export-only external client permissions option.

## Non-Goals

- No external client permission import, preview, preflight, mapping UI, backup,
  rollback, activation, enable/disable control, token creation, secret handling,
  or permission grant UI through import/export.
- No external client placeholder export behavior changes. External client rows
  remain covered by Sprint 073 and are not included in this permission export.
- No external client activation semantics changes.
- No proposed action, audit log, settings, sync changelog, backup metadata, or
  broad relationship import/export behavior changes.
- No remote/cloud/scheduled import/export.
- No MCP runtime, AI behavior, sync-server behavior, client connection behavior,
  or secret handling.
- No destructive migration, schema rewrite, release packaging, signing,
  notarization, or unrelated UI redesign.

## Verification

Completed locally on the Sprint 074 worktree:

- `npm ci` - passed; installed frontend dependencies from the committed
  lockfile and reported existing audit findings.
- `npm run lint` - passed.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed: 20 files, 151 tests.
- `npm run build` - passed. Vite emitted the existing browser compatibility
  notices for externalized `node:async_hooks`; SvelteKit emitted the existing
  fallback overwrite message.
- `npm run test:e2e` - passed: 6 Chromium tests. The web server emitted the
  existing `NO_COLOR`/`FORCE_COLOR` warning.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: 137 `crm-core` tests, 12 Tauri library
  tests, and doc-tests with 17 ignored examples.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no quoted SQL
  matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no quoted SQL matches.
- `git diff --check origin/main...HEAD` - passed with no whitespace errors.
- `git fsck --full --no-progress` - exited 0 with existing output:
  `dangling commit 10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
