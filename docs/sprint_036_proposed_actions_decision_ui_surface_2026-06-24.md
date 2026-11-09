# Sprint 036 - Proposed Actions Decision UI Surface

Date: 2026-06-24
Branch: `codex/proposed-actions-decision-ui-surface`
Scope: Narrow `/pending-actions` UI controls for approving or rejecting existing pending proposed actions using the Sprint 035 frontend API wrappers.

## Summary

- Added explicit Approve and Reject controls to each pending proposed-action row.
- Wired decisions through `approveProposedAction` and `rejectProposedAction` from `apps/desktop/src/lib/api/proposedActions.ts`.
- Removed decided actions from the visible pending list immediately after a successful approval or rejection and refreshed pending data afterward.
- Added per-row busy state, inline failure messages, success/failure toasts, and retry by pressing the same decision control again after a failed decision.
- Guarded pending-action reloads with request and decision sequence checks so stale async responses cannot re-add an action after it is decided.
- Kept JSON rendering in text and `<pre>` output only.
- Added i18n keys for all locale JSON files with English source copy and fallback text parity.

## Boundaries

- No proposed-action execution workflow was added.
- No execute, run, apply, MCP, AI, sync server, external-client grant, token, or secret behavior was added.
- No backend, Tauri command, schema, or `crm-core` changes were made.
- The page remains pending-only and only displays actions returned by the pending-action API wrapper.
- Frontend access continues through typed API wrappers; no direct Svelte `invoke()` calls were added.

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
- [x] i18n locale parity script
