# Sprint 033 - External Clients API Surface

Date: 2026-06-24
Branch: `codex/external-clients-api-surface`
Scope: Narrow desktop and frontend API bridge for disabled external client placeholder records.

## Summary

- Added thin Tauri commands for `list_external_clients` and `create_external_client_placeholder`.
- Registered both commands in the desktop invoke handler.
- Kept external client storage access in `crm-core`; Tauri commands only lock `CrmCore` and call service methods.
- Added `apps/desktop/src/lib/api/externalClients.ts` with typed camel-cased `ExternalClient` mapping.
- Added invoke-mapping Vitest coverage for external client list and placeholder creation calls.
- Hardened `crm-core` placeholder creation to trim required fields and reject blank names or client types.
- Added focused Rust tests for disabled/default placeholder safety, list ordering, and blank-field rejection.

## Boundaries

- No UI route, navigation, or visible settings surface was added.
- No enable/disable command was added.
- No permission grants, secrets, tokens, MCP server behavior, AI behavior, or sync server behavior was added.
- Created records remain disabled placeholders with `permission_mode = 'disabled'`.
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
