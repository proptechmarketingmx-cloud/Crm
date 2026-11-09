# Sprint 034 - External Clients UI Surface

Date: 2026-06-24
Branch: `codex/external-clients-ui-surface`
Scope: Narrow Settings UI for disabled external client placeholders using the Sprint 033 API wrappers.

## Summary

- Added a Settings/Integrations card that lists external clients through `apps/desktop/src/lib/api/externalClients.ts`.
- Displayed client name, client type, enabled/disabled state, permission mode, and created/updated timestamps.
- Added a compact create action for disabled placeholder clients with required name and client type fields.
- Added loading, retry, empty, load error, disabled submission, create success, and create failure states.
- Guarded in-flight list responses/errors so stale loads cannot hide a newly created placeholder.
- Added i18n keys across every locale file; English copy is canonical and non-English files use fallback parity text for this sprint.

## Boundaries

- No direct Svelte `invoke()` calls were added.
- No enable/disable controls were added.
- No permission grants, tokens, secrets, MCP server behavior, AI behavior, sync server behavior, or approval workflow was added.
- No schema, `crm-core`, Tauri command, or route changes were added.
- Existing external client records remain disabled placeholders unless created otherwise by lower layers outside this UI.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check main...HEAD`
- [x] Raw SQL boundary scan in Tauri commands and `crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct Svelte `invoke()` scan
- [x] Locale parity Node script

## Residual Risk

- No focused Svelte route test was added because this repo currently has practical frontend coverage for API wrappers and utilities, not mounted route/component UI tests.
