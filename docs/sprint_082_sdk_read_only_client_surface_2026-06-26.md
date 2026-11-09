# Sprint 082 - SDK Read-Only Client Surface

Date: 2026-06-26
Branch: `codex/sdk-read-only-client-surface`
Scope: Local read-only SDK facade for reviewed external clients.

## Summary

- Replaced the placeholder `crm-sdk` crate with `ReadOnlyCrmSdk`/`CrmSdk`,
  a narrow local facade over `crm_core::CrmCore`.
- Added exported initial read tool constants for `contacts.list`,
  `organizations.list`, `deals.list`, `activities.list`, and `search.global`.
- Added read-only SDK methods for contacts, organizations, deals, activities,
  and global search. Each method evaluates external-client read permission in
  `crm-core` before calling existing core services.
- Added focused SDK tests for implemented behavior, disabled and unpermitted
  denial, allowed read-only access, audit evidence from core permission
  evaluation, and the global-search permission boundary.
- Updated MCP readiness docs to document the SDK readiness surface while
  keeping MCP runtime startup and tool serving explicitly unimplemented.

## Changed Files

- `README.md`
- `crates/crm-sdk/Cargo.toml`
- `crates/crm-sdk/src/lib.rs`
- `docs/MCP_READINESS.md`
- `docs/sprint_082_sdk_read_only_client_surface_2026-06-26.md`
- `docs/sprint_ledger.md`

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `npm run release:notes:sample`
- [x] `npm run release:manifest:sample`
- [x] `npm run check:release-guardrails`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] Raw SQL scan in `crates/crm-sdk`, `apps/desktop/src-tauri/src/commands`,
      and `crates/crm-core/src/crm_engine`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Non-Goals

- No MCP server startup, localhost listener, prompt/resource/tool serving,
  tokens, secrets, model integration, or network behavior.
- No SDK write methods, proposed-action creation, schema changes, sync server
  behavior, desktop/Tauri behavior, or UI changes.
- No raw SQL in `crm-sdk` production code.
- No npm or Rust dependencies were added.
