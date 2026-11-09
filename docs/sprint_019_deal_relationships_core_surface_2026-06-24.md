# Sprint 019 - Deal Relationships Core Surface

Date: 2026-06-24
Branch: `codex/deal-relationships-core-surface`
Scope: Non-destructive deal organization and deal contact relationship foundation without Pipeline/Deal UI changes.

## Summary

- Added schema v7 with `deals.organization_id`, a first-class `deal_contacts` table, relationship lookup indexes, and conservative legacy `deals.contact_id` backfill into primary deal contacts.
- Exposed `Deal.organization_id` while preserving the legacy `contact_id` primary-contact mirror.
- Added `CrmCore` services for listing deal contacts, adding/removing deal contacts, and linking/unlinking a deal to an organization.
- Added thin Tauri commands and frontend API wrappers for the new relationship surface.
- Added focused Rust and frontend invoke-mapping tests for migration, mirror behavior, audit/sync writes, and command payloads.

## Architecture Decisions

- Migration v7 is additive only: it adds columns/tables/indexes and backfills relationship rows without deleting or rewriting `deals.contact_id`.
- `deal_contacts` is the new explicit contact relationship surface, while `deals.contact_id` remains a compatibility mirror for the primary active deal contact.
- New write paths validate active deal, contact, and organization references in `CrmCore` before storage mutation.
- SQL remains in `crates/crm-core/src/storage/**`; Tauri commands and `crm_engine` do not contain raw SQL.
- Existing create/update deal flows may carry `organization_id`, but `deal_contacts` is mutated only through explicit relationship services and commands.
- Primary contact changes record sync/audit for both the `deal_contact` link and any resulting legacy `deal.contact_id` mirror change.

## Preservation Notes

- No Pipeline/Deal UI surface was added in this sprint.
- No MCP server behavior, AI behavior, sync server behavior, or broad schema normalization was changed.
- Legacy `deals.contact_id` remains present and populated for existing compatibility.
- Existing contact, organization, notes/tags, backup, email, reminders, reports, custom fields, locales, and pipeline features were not removed or rewritten.
- Plain `.ts` files do not use Svelte runes.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] Raw SQL scan in `apps/desktop/src-tauri/src/commands`
- [x] Raw SQL scan in `crates/crm-core/src/crm_engine`
- [x] Plain `.ts` rune scan
- [x] `git diff --check origin/main...HEAD`
