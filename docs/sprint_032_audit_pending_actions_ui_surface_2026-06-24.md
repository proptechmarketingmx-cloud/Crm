# Sprint 032: Audit Log and Pending Actions UI Surface

Date: 2026-06-24
Branch: `codex/audit-pending-actions-ui-surface`

## Scope

Sprint 032 adds visible, read-only operational routes for the Sprint 031 audit and pending proposed-action API wrappers:

- `/audit-log` renders recent audit entries with limit selection, refresh, loading, error/retry, empty, and data states.
- `/pending-actions` renders pending proposed actions with refresh, loading, error/retry, empty, and data states.
- Sidebar navigation and the hash router now expose both screens.
- All locale JSON files have matching keys for the new navigation labels and route copy.

## Boundaries Preserved

- No Tauri command, storage, schema, migration, core service, MCP, sync server, AI, or approval workflow changes.
- The Svelte routes use the existing frontend API wrappers only:
  - `listRecentAuditLog(limit?)`
  - `listPendingProposedActions()`
- Pending Actions remains read-only. There are no approve, reject, execute, or mutation controls.
- JSON payloads are shown as compact summaries with opt-in detail expansion to keep the screens scannable.

## Verification Notes

Focused component tests were not added because this repo currently validates Svelte route integration through `npm run check` and frontend API behavior through existing wrapper tests. UI behavior for the new routes is statically verified by Svelte type checking, build output, and the existing API tests from Sprint 031.
