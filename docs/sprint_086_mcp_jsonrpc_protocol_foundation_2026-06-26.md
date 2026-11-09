# Sprint 086 - MCP JSON-RPC Protocol Foundation

Date: 2026-06-26
Branch: `codex/mcp-jsonrpc-protocol-foundation`
Scope: Metadata-only `crm-mcp` JSON-RPC/MCP one-shot protocol handler and CLI probe without MCP runtime behavior.

## Summary

- Added a reusable one-shot JSON-RPC handler for a narrow metadata-only MCP
  subset: `initialize`, `tools/list`, and no-response notifications.
- Added deterministic `initialize` metadata that advertises tools-list
  capability only and explicitly states that no serving loop, transport,
  listener, SDK dispatch, database access, authentication, or tool execution is
  enabled.
- Added deterministic `tools/list` metadata mapped from the existing offline
  read-only catalog in SDK order. Each entry includes MCP-style name,
  description, input schema, read-only annotations, and metadata showing
  `runtimeEnabled: false` and `executionEnabled: false`.
- Added standard JSON-RPC error responses for malformed JSON, invalid request
  shapes, unknown methods, and explicit `tools/call` rejection.
- Added `--handle-jsonrpc-once <json>` for deterministic tests. The flag
  handles one supplied JSON string and prints one response JSON, or prints
  nothing for notifications. It does not add a stdio loop or runtime server.
- Preserved existing default CLI behavior, `--print-tool-catalog`,
  `--list-tools`, `--print-runtime-status`, and
  `--print-runtime-status-from-config <path>`.
- Updated MCP readiness docs to describe the probe as metadata-only and not a
  serving loop, transport, listener, or execution path.

## Changed Files

- `crates/crm-mcp/src/lib.rs`
- `crates/crm-mcp/src/main.rs`
- `crates/crm-mcp/tests/cli.rs`
- `docs/MCP_READINESS.md`
- `docs/sprint_086_mcp_jsonrpc_protocol_foundation_2026-06-26.md`
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
- [x] CLI probe: `initialize`
- [x] CLI probe: `tools/list`
- [x] CLI probe: `tools/call` rejection
- [x] `rg` scan proving no server/network/auth/tool execution implementation
      was added in `crates/crm-mcp`
- [x] Raw SQL scan in desktop commands and `crm_engine`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Non-Goals

- No MCP server startup, socket binding, stdio loop, TCP/HTTP/SSE transport,
  listener, prompt/resource serving, tool execution, SDK dispatch, database
  access, token/secret handling, authentication, model integration, or network
  behavior.
- No UI, schema, desktop/Tauri behavior, sync server behavior, SDK write
  behavior, proposed-action behavior, raw SQL, or dependency additions.
