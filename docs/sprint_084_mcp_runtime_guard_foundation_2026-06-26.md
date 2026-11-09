# Sprint 084 - MCP Runtime Guard Foundation

Date: 2026-06-26
Branch: `codex/mcp-runtime-guard-foundation`
Scope: Disabled-by-default MCP runtime guard/config/status foundation without MCP serving behavior.

## Summary

- Added `McpRuntimeConfig` to `crates/crm-mcp` with a disabled default,
  localhost-only bind host, and zero bind port placeholder.
- Added runtime config validation that accepts enabled loopback hosts and
  rejects enabled non-loopback bind hosts without opening sockets or starting a
  listener.
- Added explicit `McpRuntimeStatus` metadata that always reports
  `serving: false` and `tool_execution_enabled: false`, with default reason
  `runtime disabled`.
- Added deterministic `--print-runtime-status` CLI JSON output for the default
  runtime config while preserving the existing `--print-tool-catalog` and
  `--list-tools` catalog behavior.
- Added focused Rust and CLI tests for default config/status, loopback
  validation, deterministic status JSON, and default non-serving CLI wording.
- Updated MCP readiness docs to describe the runtime guard/config readiness
  surface while preserving the explicit non-runtime boundary.

## Changed Files

- `crates/crm-mcp/src/lib.rs`
- `crates/crm-mcp/src/main.rs`
- `crates/crm-mcp/tests/cli.rs`
- `docs/MCP_READINESS.md`
- `docs/sprint_084_mcp_runtime_guard_foundation_2026-06-26.md`
- `docs/sprint_ledger.md`

## Verification

- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo run -p crm-mcp -- --print-tool-catalog`
- [x] `cargo run -p crm-mcp -- --print-runtime-status`
- [x] `cargo run -p crm-mcp`
- [x] `npm run release:notes:sample`
- [x] `npm run release:manifest:sample`
- [x] `npm run check:release-guardrails`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] Raw SQL/runtime scan in `crates/crm-mcp`, `crates/crm-sdk`, Tauri
      commands, and `crates/crm-core/src/crm_engine`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Non-Goals

- No MCP server startup, socket binding, listener, protocol serving, prompts,
  resources, tool serving, tool execution, SDK dispatch, token/secret handling,
  model integration, auth, or network behavior.
- No UI, schema, desktop/Tauri behavior, sync server behavior, SDK write
  behavior, proposed-action behavior, or raw SQL.
- No dependency additions.
