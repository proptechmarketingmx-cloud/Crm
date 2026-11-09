# Sprint 038 - External Client Permissions API Surface

Date: 2026-06-24
Branch: `codex/external-client-permissions-api-surface`
Scope: Thin desktop command and typed frontend API surface for Sprint 037 external-client permission core.

## Summary

- Added Tauri commands for listing external-client permissions, upserting one client/tool permission, and evaluating read or draft permission for one client/tool.
- Registered the permission commands in the desktop invoke handler.
- Extended `externalClients.ts` with typed permission and evaluation DTOs plus snake_case-to-camelCase mapping functions.
- Added focused Vitest coverage for command names, snake_case invoke arguments, and response mapping.

## Boundaries

- No UI routes, Svelte components, Settings surfaces, or navigation were changed.
- No MCP server behavior, AI behavior, sync server behavior, token/secret handling, grants activation UX, or proposed-action execution path was added.
- No `crm-core`, storage, or schema changes were required.
- Tauri commands remain thin `crm-core` calls and do not contain SQL.

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
- [x] `git fsck --full --no-progress`

Note: `git fsck --full --no-progress` exited 0 and reported dangling commit `10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
