# Sprint 031 - Audit and Pending Actions API Surface

Date: 2026-06-24
Branch: `codex/audit-pending-actions-api-surface`
Scope: Read-only desktop and frontend API foundation for audit log entries and pending proposed actions.

## Summary

- Added thin Tauri commands for `list_recent_audit_log` and `list_pending_proposed_actions`.
- Registered both commands in the desktop invoke handler.
- Kept audit log and proposed-action data access in existing `crm-core` services and storage modules.
- Defaulted audit log command reads to 100 entries and clamped explicit limits to the storage-backed 1-500 range.
- Added `apps/desktop/src/lib/api/audit.ts` with typed camel-cased `AuditLogEntry` mapping and invoke-mapping tests.
- Added `apps/desktop/src/lib/api/proposedActions.ts` with typed camel-cased `ProposedAction` mapping and invoke-mapping tests.
- Added focused core service tests for pending-action filtering/order and recent audit-log limit behavior.

## Boundaries

- No Audit Log route or UI was added.
- No Pending Actions route or UI was added.
- No approval, rejection, execution, AI, MCP, sync server, migration, or schema behavior was changed.
- No raw SQL was added to Tauri commands or `crm_engine`.
- Frontend access goes through typed API wrappers; no direct Svelte `invoke()` calls were added.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check main...HEAD`
- [x] Raw SQL boundary scan in Tauri commands and `crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct Svelte `invoke()` scan
