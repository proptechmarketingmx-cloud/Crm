# Sprint 087 - MCP Stdio Loop Disabled Gate

Date: 2026-06-26
Branch: `codex/mcp-stdio-loop-disabled-gate`

## Goal

Add the first transport-shaped local stdio boundary for `crm-mcp` while keeping
it disabled by default, config-gated, loopback-validated, metadata-only, and
unable to execute tools.

## Changes

- Added a pure newline-delimited JSON-RPC helper that reuses the existing
  metadata-only `handle_jsonrpc_once` behavior and emits no line for
  notifications.
- Added a gated stdio loop runner that rejects disabled configs before reading
  stdin or writing stdout, rejects enabled non-loopback configs, and delegates
  enabled loopback lines to the metadata-only handler.
- Added `--serve-stdio-from-config <path>` as the only stdio CLI entrypoint.
  Missing optional config paths resolve to the disabled default and fail closed.
- Added focused library and CLI coverage for disabled rejection, enabled
  loopback initialize/tools-list processing, notification suppression,
  `tools/call` rejection, non-loopback rejection, and unchanged default CLI
  behavior.
- Updated MCP readiness docs to state the stdio path is gated,
  disabled-by-default, local-only, metadata-only, and does not add SDK,
  database, auth/token/secret, network listener, sync-server, or tool execution
  behavior.

## Boundaries Preserved

- No TCP, HTTP, SSE, socket, or network listener was added.
- No auth, token, secret, SDK dispatch, database access, raw SQL, write path,
  schema, UI, sync-server, or tool execution behavior was added.
- Default `crm-mcp` binary execution still prints the non-runtime status message
  and does not start a loop.
- `tools/call` remains rejected by the existing metadata-only JSON-RPC handler.

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
- [x] CLI disabled stdio rejection probe
- [x] CLI enabled loopback piped initialize/tools-list probe
- [x] `rg` scan proving no TCP/HTTP/SSE/socket/auth/token/SDK/db/tool-execution
      implementation was added in `crates/crm-mcp`
- [x] Raw SQL scan in desktop commands and `crm_engine`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Notes

This sprint intentionally creates only a narrow, disabled-by-default stdio loop
foundation. It is useful for future transport review, but it is not an active
MCP server and does not execute catalog tools.
