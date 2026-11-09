# Sprint 089 - MCP Activity Draft Tool

Date: 2026-06-27
Branch: `codex/mcp-activity-draft-tool`

## Goal

Add exactly one reviewed MCP draft tool path, `create_activity_draft`, that
creates a pending proposed action through the existing core draft permission
boundary. The sprint must not add direct activity creation, proposed-action
approval/rejection/execution from MCP, broad write tools, network serving,
authentication, AI behavior, sync-server behavior, schema changes, or UI.

## Changes

- Extended `crm-sdk` from a read-only facade to a reviewed local facade with:
  - existing permission-gated read methods unchanged;
  - exported `CREATE_ACTIVITY_DRAFT_TOOL`;
  - `create_activity_draft` pending proposed-action creation through
    `CrmCore::create_external_proposed_action_stub`.
- Normalized draft input before persistence:
  - required non-empty `title`;
  - optional trimmed `activity_type`, `description`, and `due_at`;
  - optional `linked_entities` limited to `contact`, `organization`, and `deal`.
- Kept activity creation out of the MCP/SDK call. The tool creates only a
  pending proposed action with `tool_name = "create_activity_draft"` and
  `action_type = "create_activity_draft"`.
- Extended the config-gated stdio `tools/call` dispatcher so
  `create_activity_draft` is executable only when local execution context is
  configured.
- Exposed `create_activity_draft` in MCP `tools/list` only for the reviewed
  stdio execution context, with draft metadata:
  `readOnlyHint: false`, `destructiveHint: false`, `idempotentHint: false`,
  `accessKind: "draft"`, `requiresConfirmation: true`,
  `createsPendingAction: true`, and `directExecution: false`.
- Preserved validation-before-open behavior for unknown tools and malformed
  draft arguments so rejected calls do not create app data or open the SDK.
- Added SDK and MCP CLI tests covering allowed draft creation, read-only
  denial, missing permission denial, audit evidence, no direct activity
  creation, metadata gating, and rejected-call no-open behavior.
- Updated README, architecture, MCP readiness docs, and sprint ledger.

## Boundaries Preserved

- No TCP, HTTP, SSE, socket, or network listener was added.
- No auth, token, secret, model-provider, prompt, or resource behavior was
  added.
- No raw SQL or direct database access was added in `crm-mcp` or `crm-sdk`.
- No direct activity creation from MCP was added.
- No proposed-action approve, reject, or execute tool was added.
- No schema, UI, Tauri command, sync-server, AI, import/export, backup, or
  release-packaging behavior was added.
- Unknown, unreviewed, and direct write-like tool names remain rejected before
  SDK/core open.

## Verification Checklist

- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p crm-sdk`
- [x] `cargo test -p crm-mcp`
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

This sprint intentionally adds only draft proposed-action creation for the
existing reviewed `create_activity_draft` flow. User review through Pending
Actions remains required before the proposed activity can be approved and
executed by the existing core path.
