# Sprint 085 - MCP Config File Foundation

Date: 2026-06-26
Branch: `codex/mcp-config-file-foundation`
Scope: Disabled-by-default `crm-mcp` JSON config-file metadata loader and deterministic status probe without MCP runtime behavior.

## Summary

- Added JSON serialization/deserialization support for `McpRuntimeConfig` while
  keeping `McpRuntimeConfig::default()` disabled and loopback-only.
- Added safe config-file loading helpers for required and optional paths. The
  optional helper treats a missing file as absent config and returns the
  disabled default. Invalid JSON, unreadable non-missing files, and enabled
  non-loopback hosts return explicit errors.
- Added `--print-runtime-status-from-config <path>` to print deterministic
  status JSON for loaded config metadata. Enabled loopback configs still report
  `serving: false`, `tool_execution_enabled: false`, and reason
  `server not implemented`.
- Preserved existing `--print-runtime-status`, `--print-tool-catalog`,
  `--list-tools`, and default CLI behavior.
- Added focused Rust and CLI tests for missing optional config fallback,
  disabled and enabled-loopback config status, invalid JSON, non-loopback
  rejection, deterministic CLI JSON, and non-serving default wording.
- Updated MCP readiness docs to state config files are readiness metadata only.

## Changed Files

- `crates/crm-mcp/src/lib.rs`
- `crates/crm-mcp/src/main.rs`
- `crates/crm-mcp/tests/cli.rs`
- `docs/MCP_READINESS.md`
- `docs/sprint_085_mcp_config_file_foundation_2026-06-26.md`
- `docs/sprint_ledger.md`

## Verification Checklist

- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p crm-mcp`
- [x] `cargo clippy -p crm-mcp -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo run -p crm-mcp -- --print-runtime-status`
- [x] `cargo run -p crm-mcp -- --print-runtime-status-from-config <temp valid config>`
- [x] `rg` scan proving no server/network/auth/tool execution implementation was added in `crates/crm-mcp`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Non-Goals

- No MCP server startup, socket binding, listener, protocol serving, prompts,
  resources, tool serving, tool execution, SDK dispatch, token/secret handling,
  authentication, model integration, or network behavior.
- No UI, schema, desktop/Tauri behavior, sync server behavior, SDK write
  behavior, proposed-action behavior, raw SQL, or dependency additions.
