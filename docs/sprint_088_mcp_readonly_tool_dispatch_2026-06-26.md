# Sprint 088 - MCP Read-Only Tool Dispatch

Date: 2026-06-26
Branch: `codex/mcp-readonly-tool-dispatch`

## Goal

Add the first executable MCP `tools/call` path for reviewed read-only tools
while keeping the runtime disabled by default, local-only, config-gated, and
routed through `crm-sdk` permission checks.

## Changes

- Extended MCP runtime config with optional local execution context:
  `app_data_dir` and `external_client_id`.
- Kept default config disabled with no execution context.
- Kept enabled config loopback-only and added validation for partial or blank
  execution context.
- Updated the stdio JSON-RPC loop so `tools/call` remains rejected without
  execution context.
- Added read-only `tools/call` dispatch for:
  - `contacts.list`
  - `organizations.list`
  - `deals.list`
  - `activities.list`
  - `search.global`
- Routed all read execution through `crm_sdk::CrmSdk`, preserving existing
  external-client permission checks and audit evidence.
- Returned JSON-RPC errors for missing context, unknown/write-like tools,
  malformed arguments, missing search query, denied permissions, and SDK
  failures.
- Kept the one-shot `--handle-jsonrpc-once` path metadata-only.
- Updated MCP readiness docs and sprint ledger.

## Boundaries Preserved

- No TCP, HTTP, SSE, socket, or network listener was added.
- No auth, token, secret, model-provider, or cloud behavior was added.
- No raw SQL or direct database access was added in `crm-mcp`.
- No write tools, draft tools, proposed-action creation, schema changes, UI,
  Tauri commands, sync-server behavior, or desktop runtime behavior was added.
- Unknown or write-like tool names remain rejected.

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
- [x] `npm run check:release-guardrails`
- [x] `npm run test:e2e`
- [x] CLI disabled/default rejection probes
- [x] CLI allowed read-only dispatch probe
- [x] `rg` scan proving no TCP/HTTP/SSE/socket listener/auth/token/secret/raw
      SQL/write-tool implementation was added in `crates/crm-mcp`
- [x] Raw SQL scan in desktop commands and `crm_engine`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Notes

This sprint intentionally adds only a local stdio read-only execution path for
reviewed SDK tools. It is still not a network MCP server, does not introduce
credentials, and does not allow write-like CRM operations.
