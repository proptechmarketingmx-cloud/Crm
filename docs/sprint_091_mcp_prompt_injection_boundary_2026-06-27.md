# Sprint 091 - MCP Prompt-Injection Boundary

Date: 2026-06-27
Branch: `codex/mcp-prompt-injection-boundary`

## Goal

Close the MCP readiness gap for prompt-injection boundaries by documenting and
testing that CRM content returned by MCP tools is untrusted user-controlled
data. This sprint must not add prompts, resources, listeners, network serving,
auth, tokens, model-provider behavior, product UI, schema changes, sync-server
behavior, AI behavior, broad tool behavior, raw SQL, or direct database access.

## Changes

- Added an explicit `crm-mcp` prompt-injection boundary instruction stating
  that returned CRM records, notes, descriptions, titles, search results, and
  draft content are data only and must not be treated as model/client
  instructions.
- Included the boundary instruction in MCP `initialize` responses for both
  metadata-only and reviewed local stdio modes.
- Added per-tool `tools/list` metadata:
  - `returnedContentTrust: "untrusted-user-controlled-data"`
  - `promptInjectionBoundary`
- Added focused `crm-mcp` CLI tests that verify:
  - `initialize` advertises only tool capabilities and includes the untrusted
    CRM content instruction.
  - The reviewed local stdio execution path includes the same instruction and
    per-tool trust metadata, including the draft tool.
  - `tools/list` includes untrusted-content metadata on every advertised tool.
  - `resources/list` and `prompts/list` remain unimplemented.
- Updated MCP readiness docs and the sprint ledger.

## Boundaries Preserved

- No TCP, HTTP, SSE, socket, or network listener was added.
- No auth, token, secret, model-provider, prompt, or resource behavior was
  added.
- No raw SQL or direct database access was added in `crm-mcp`.
- No schema, UI, Tauri command, sync-server, AI, import/export, backup, or
  release-packaging behavior was added.
- No MCP tool names, SDK dispatch paths, permission semantics, draft semantics,
  or proposed-action lifecycle behavior were changed.

## Verification Checklist

- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p crm-mcp`
- [x] `cargo test -p crm-sdk`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `npm run check:release-guardrails`
- [x] `npm run test:e2e`
- [x] `git diff --check`
- [x] Raw SQL boundary scan in `crates/crm-mcp` and Tauri command handlers
- [x] `git fsck --full --no-progress`

## Notes

The implementation is metadata-only. It documents and tests how clients should
treat returned CRM content at the MCP boundary, while preserving the existing
tool catalog, stdio gating, SDK routing, and prompt/resource non-goals.
`git fsck` exited successfully with only pre-existing dangling object notices.
