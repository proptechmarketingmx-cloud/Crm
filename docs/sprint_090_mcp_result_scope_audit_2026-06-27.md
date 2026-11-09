# Sprint 090 - MCP Result-Scope Audit

Date: 2026-06-27
Branch: `codex/mcp-result-scope-audit`

## Goal

Close the MCP runtime read-audit gap by recording successful read result scope
after reviewed SDK read tools return data. The sprint must preserve the existing
permission evaluation audit and must not add network serving, auth, tokens,
raw SQL in MCP/SDK, schema changes, UI, AI behavior, sync-server behavior, or
new tool behavior.

## Changes

- Added shared `mcp_client` audit actor constant.
- Added `CrmCore::record_external_client_tool_result`.
- The result audit action is `record_external_client_tool_result`.
- Result audit context includes:
  - `client_id`
  - `tool_name`
  - `access_kind`
  - `status`
  - `result_count`
  - optional `entity_scope`
- Wired successful SDK reads to record result counts for:
  - `contacts.list`
  - `organizations.list`
  - `deals.list`
  - `activities.list`
  - `search.global`
- Kept denied reads limited to the existing permission-evaluation audit.
- Kept `create_activity_draft` unchanged. It already records draft permission
  audit plus `propose_action`; this sprint avoids returning a post-create audit
  failure after a proposed action has already been created.
- Updated MCP readiness docs and sprint ledger.

## Boundaries Preserved

- No TCP, HTTP, SSE, socket, or network listener was added.
- No auth, token, secret, model-provider, prompt, or resource behavior was
  added.
- No raw SQL or direct database access was added in `crm-mcp` or `crm-sdk`.
- No schema, UI, Tauri command, sync-server, AI, import/export, backup, or
  release-packaging behavior was added.
- No new MCP tool names, direct activity creation, or proposed-action decision
  tools were added.
- Result audit entries do not create sync changelog rows.

## Verification Checklist

- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p crm-core external_client_tool_result_records_audit_context_without_sync_changelog`
- [x] `cargo test -p crm-sdk`
- [x] `cargo test -p crm-mcp`
- [x] `cargo clippy -p crm-core -- -D warnings`
- [x] `cargo clippy -p crm-sdk -- -D warnings`
- [x] `cargo clippy -p crm-mcp -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `npm run check:release-guardrails`
- [x] `npm run test:e2e`
- [x] Raw SQL boundary scans for desktop commands, `crm_engine`, `crm-mcp`,
      and `crm-sdk`
- [x] Network/auth/token/listener scan for `crm-mcp` and `crm-sdk`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Notes

This sprint records successful read result scope after permission checks and
service execution succeed. It intentionally does not alter permission decisions,
returned data, or proposed-action lifecycle behavior.
