# Sprint 037 - External Client Permissions Core Surface

Date: 2026-06-24
Branch: `codex/external-client-permissions-core-surface`
Scope: Narrow `crm-core` permission foundation for existing external clients and proposed-action draft gating.

## Summary

- Added typed external-client permission primitives for current modes (`disabled`, `read_only`, `draft_only`) while treating future schema modes as unsupported in this sprint.
- Added SQL-only storage repository functions for listing, reading, and atomically upserting one `external_client_permissions` row per `(client_id, tool_name)`.
- Added schema v9 cleanup for duplicate permission rows plus a unique `(client_id, tool_name)` index.
- Added `CrmCore` methods to list a client's tool permissions, upsert a tool permission, evaluate read access, and evaluate draft proposed-action access.
- Enforced permission checks in `create_external_proposed_action_stub` when `client_id` is present.
- Preserved `client_id: None` proposed-action creation for internal tests/tools.
- Restricted draft creation to `draft_only` clients with a matching tool permission row where `can_write = true` and `requires_confirmation = true`.
- Rejected direct-write permission upserts where `can_write = true` and `requires_confirmation = false`.
- Rejected grant upserts for future schema modes `write_with_confirmation` and `write_allowed`; those modes also evaluate as no-read/no-draft in this sprint.
- Audited and sync-logged actual permission row insert/update changes.
- Left external-client placeholder creation behavior unchanged; it still creates disabled placeholders without audit/sync rows to preserve the existing Sprint 033 pattern.

## Boundaries

- No UI, frontend API wrappers, Tauri commands, MCP server behavior, AI behavior, sync server behavior, token handling, secret handling, or direct record execution was added.
- No proposed-action execute/run/apply workflow was added.
- The only schema change was v9 uniqueness hardening for `external_client_permissions`; no new product tables were added.
- No raw SQL was added outside `crates/crm-core/src/storage` production modules.
- Future schema values `write_with_confirmation` and `write_allowed` remain disabled by evaluation logic.

## Validation

- [x] `cargo test -p crm-core`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `git diff --check main...HEAD`
- [x] Raw SQL boundary scan in Tauri commands and `crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct Svelte `invoke()` scan
- [x] `git fsck --full --no-progress`
