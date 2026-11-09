# Sprint 022 - Activity Relationships UI Surface

Date: 2026-06-24
Branch: `codex/activity-relationships-ui-surface`
Scope: Frontend-only surface for Sprint 021 activity relationship APIs in activity creation and activity list labels.

## Summary

- Added contact, organization, and deal selectors to activity creation surfaces.
- Preserved legacy activity `contactId` and `dealId` create behavior while also ensuring selected relationships are represented through the existing activity-link API.
- Kept organization relationships first-class by writing them only through `activity_links`, not through legacy activity columns.
- Displayed linked contact, organization, and deal labels on activity rows using dedicated lookup data that is not limited by the Contacts route page size.
- Added focused unit coverage for activity relationship label derivation, paged contact lookup loading, and activity-link wrapper usage.

## Architecture Decisions

- The sprint uses existing frontend API wrappers from `activities.ts`; Svelte components do not call Tauri `invoke()` directly.
- Contact and deal selectors continue to populate the legacy create payload fields so context routes and existing filters keep their current behavior.
- Selected contact, organization, and deal IDs are also passed through `addActivityLink`; the backend is idempotent for already-created contact/deal mirror links.
- Activity labels are derived from explicit `activity_links` plus dedicated contact, organization, and deal lookups.
- Contact relationship lookups call the existing contact list API with a 500-row page size and continue fetching pages until all contacts are loaded, avoiding mutation of the Contacts route store state.
- Plain TypeScript relationship helpers avoid Svelte runes and keep component changes narrowly focused.
- No backend commands, schema changes, service changes, migrations, MCP behavior, AI behavior, sync behavior, or broad visual redesign were added.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check main...HEAD`
- [x] Raw SQL scan in `apps/desktop/src-tauri/src/commands`
- [x] Raw SQL scan in `crates/crm-core/src/crm_engine`
- [x] Plain `.ts` rune scan
