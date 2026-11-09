# Sprint 039 - MCP Readiness Docs Baseline

Date: 2026-06-24
Branch: `codex/mcp-readiness-docs-baseline`
Scope: Documentation-only MCP readiness baseline for the current placeholder and readiness surfaces.

## Summary

- Added `docs/MCP_READINESS.md` to document the current MCP boundary and future acceptance checklist.
- Clarified that 900CRM core has no built-in AI agent and no internet, cloud, or model-provider requirement.
- Documented that `crates/crm-mcp` is a placeholder only and does not implement or start an MCP server.
- Documented current external-client records, permission rows, proposed actions, audit log, Pending Actions UI, and approve/reject state.
- Documented the then-current approve/reject decision-only behavior; Sprint 051
  supersedes that baseline for supported `create_activity_draft` actions.
- Added README and architecture links to the readiness baseline.

## Boundaries

- Docs-only sprint.
- No Rust, TypeScript, Svelte, Cargo, npm, package, schema, command-handler, server, token, secret, sync, MCP runtime, AI, or proposed-action execution behavior was changed.
- No localhost listener, prompt/resource/tool implementation, permission grant UI, or direct action execution was added.
- Wording is intended to be public/open-source safe.

## Supersession Note

Sprint 051 later added a narrow `crm-core` execution path for supported
`create_activity_draft` proposed actions. This Sprint 039 note remains
historical context for the MCP readiness baseline before that execution path
existed.

## Validation

Validation was run after the docs-only changes:

- [x] `git diff --check main...HEAD`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] Raw SQL boundary scan in Tauri commands and `crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct Svelte `invoke()` scan
- [x] `git fsck --full --no-progress`
