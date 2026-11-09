# Sprint 083 - MCP Tool Catalog Readiness

Date: 2026-06-26
Branch: `codex/mcp-tool-catalog-readiness`
Scope: Offline MCP tool catalog boundary for the initial SDK-backed read tools.

## Summary

- Replaced the `crm-mcp` placeholder-only binary with a small library plus CLI
  that can print an offline read-only tool catalog.
- Derived catalog tool names from `crm-sdk::INITIAL_READ_TOOL_NAMES` so the MCP
  readiness boundary reuses the reviewed SDK constants for `contacts.list`,
  `organizations.list`, `deals.list`, `activities.list`, and `search.global`.
- Added deterministic JSON output via `--print-tool-catalog` and `--list-tools`.
  Each entry is marked `access_kind: "read"`,
  `requires_external_client_permission: true`, `sdk_backed: true`, and
  `runtime_enabled: false`.
- Kept default CLI output explicit that no MCP server, listener, runtime, tool
  execution, token handling, prompt/resource serving, network binding, or model
  integration is implemented.
- Added focused crate tests for catalog order, read-only permission-gated
  metadata, default non-runtime wording, and parseable deterministic CLI JSON.
- Updated MCP readiness docs to describe the offline catalog while preserving
  the non-runtime boundary.

## Changed Files

- `crates/crm-mcp/Cargo.toml`
- `crates/crm-mcp/src/lib.rs`
- `crates/crm-mcp/src/main.rs`
- `crates/crm-mcp/tests/cli.rs`
- `docs/MCP_READINESS.md`
- `docs/sprint_083_mcp_tool_catalog_readiness_2026-06-26.md`
- `docs/sprint_ledger.md`

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo run -p crm-mcp -- --print-tool-catalog`
- [x] `npm run release:notes:sample`
- [x] `npm run release:manifest:sample`
- [x] `npm run check:release-guardrails`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] Raw SQL scan in `crates/crm-mcp`, `crates/crm-sdk`, Tauri commands, and
      `crates/crm-core/src/crm_engine`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Non-Goals

- No MCP server startup, localhost listener, runtime, prompts, resources, tool
  serving, tool execution, token/secret handling, model integration, or network
  binding.
- No UI, schema, desktop/Tauri behavior, sync server behavior, SDK write
  behavior, proposed-action behavior, or raw SQL.
- No duplication of SDK tool-name strings in production catalog code.
- Added only direct `serde`/`serde_json` dependencies to `crm-mcp` for
  deterministic JSON serialization; both libraries were already used elsewhere
  in the workspace.
