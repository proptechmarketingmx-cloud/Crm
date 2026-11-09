# Sprint 035 - Proposed Actions Decision API Surface

Date: 2026-06-24
Branch: `codex/proposed-actions-decision-api-surface`
Scope: Narrow backend, desktop, and frontend API foundation for approving or rejecting existing pending proposed actions.

## Summary

- Added `crm-core` storage and service methods for approving and rejecting proposed actions.
- Enforced pending-only transitions; unknown, approved, rejected, executed, failed, or cancelled actions are rejected with explicit errors.
- Set only the decision timestamp for the selected transition: `approved_at` for approval and `rejected_at` for rejection.
- Left `executed_at` untouched and did not execute proposed actions.
- Recorded desktop audit entries for approval and rejection decisions with before/after proposed-action payloads.
- Added thin Tauri commands for `approve_proposed_action` and `reject_proposed_action`.
- Extended `apps/desktop/src/lib/api/proposedActions.ts` with typed decision wrappers and invoke-mapping tests.
- Added focused Rust tests for pending-only decisions, timestamp fields, audit entries, unknown IDs, already-approved IDs, and executed IDs.

## Boundaries

- No `/pending-actions` route or UI changes were added; the visible page remains read-only.
- No MCP server behavior, AI behavior, sync server behavior, external-client grant, token, or secret behavior was added.
- No proposed-action execution workflow was added.
- No schema change was required because decision and execution timestamp columns already exist.
- No raw SQL was added to Tauri commands or `crm_engine`.
- Frontend access goes through typed API wrappers; no direct Svelte `invoke()` calls were added.

## Supersession Note

Sprint 051 later added a narrow `crm-core` execution path for supported
`create_activity_draft` proposed actions. The Sprint 035 no-execution behavior
remains historical context for the initial decision-only API surface.

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
