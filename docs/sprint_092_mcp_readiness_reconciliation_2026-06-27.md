# Sprint 092 - MCP Readiness Reconciliation

Date: 2026-06-27
Branch: `codex/mcp-readiness-reconciliation`

## Goal

Reconcile the MCP readiness documentation after Sprint 091 so the checklist
accurately distinguishes the completed current local stdio MCP guarantees from
future listener, credential, prompt, resource, and broader runtime work that is
intentionally not implemented.

This sprint must not add MCP runtime features, prompts, resources, listeners,
network serving, auth, tokens, model-provider behavior, product UI, schema
changes, sync-server behavior, AI behavior, broad tool behavior, raw SQL, direct
database access, or new dependencies.

## Changes

- Split `docs/MCP_READINESS.md` into:
  - a completed current accepted local stdio checklist for the optional
    `crm-mcp` package, metadata probes, reviewed SDK-backed read tools, and
    reviewed `create_activity_draft` pending-action path;
  - a deferred future MCP checklist for listener binding, explicit server
    enablement, credential/token design, prompt/resource surfaces, broader
    write modes, and future verification obligations.
- Preserved the existing future security gates and tightened the acceptance
  wording so unchecked future listener/token work is no longer mixed with
  completed current local stdio guarantees.
- Did not add tests because existing `crm-mcp` coverage already verifies the
  documentation-backed claims that initialize advertises only tool capability,
  `resources/list` and `prompts/list` remain unimplemented, and the stdio path
  is config-gated without listener/token behavior.
- Updated the sprint ledger.

## Boundaries Preserved

- No TCP, HTTP, SSE, socket, or network listener was added.
- No auth, token, secret, model-provider, prompt, or resource behavior was
  added.
- No raw SQL or direct database access was added in `crm-mcp`.
- No schema, UI, Tauri command, sync-server, AI, import/export, backup, or
  release-packaging behavior was added.
- No MCP tool names, SDK dispatch paths, permission semantics, draft semantics,
  proposed-action lifecycle behavior, or runtime behavior were changed.

## Verification Checklist

- [x] `cargo test -p crm-mcp`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run check:release-guardrails`
- [x] `git diff --check`
- [x] Focused no raw SQL/network/auth/token scan for `crates/crm-mcp`

## Notes

This is documentation-only reconciliation. The accepted current scope remains a
local stdio boundary, not a network MCP server. Future listener, token,
credential, prompt, resource, and broader write-mode work remains intentionally
deferred.

`npm run check` and `npm run test` required installing frontend dependencies in
this worktree with `npm ci`; the dependency install reported existing npm audit
findings but did not change tracked files.
