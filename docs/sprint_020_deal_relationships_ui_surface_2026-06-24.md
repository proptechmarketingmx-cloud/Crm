# Sprint 020 - Deal Relationships UI Surface

Date: 2026-06-24
Branch: `codex/deal-relationships-ui-surface`
Scope: Frontend-only surface for Sprint 019 deal relationship fields in deal creation and pipeline cards.

## Summary

- Added organization and primary-contact selectors to the existing add-deal modal.
- Preserved stage prefill, contact-context prefill, currency normalization, and custom-field persistence in the modal.
- Displayed primary contact and organization labels on pipeline deal cards using a dedicated relationship lookup that is not limited by the Contacts route page size.
- Added compact loading/disabled states for add-deal relationship selectors while contact and organization lookup data is being fetched.
- Added focused unit coverage for relationship label derivation and selector label fallbacks.

## Architecture Decisions

- The sprint uses the existing `createDeal` payload fields: `contactId` for the primary contact mirror and `organizationId` for the first-class organization link.
- No backend commands, schema changes, service changes, migrations, MCP behavior, AI behavior, sync behavior, or broad editor UI were added.
- Pipeline cards keep the backend `Deal` shape unchanged and derive names from a frontend relationship lookup.
- Relationship lookups intentionally call the existing contact list API with a 500-row page size and continue fetching pages until all matching contacts are loaded. This avoids mutating the paginated `contactStore` state used by the Contacts route while keeping pipeline labels reliable for accounts with more than 50 contacts.
- Organization lookups continue to use the existing full-list `listOrganizations` API.
- Relationship label logic is isolated in a plain TypeScript utility so Svelte components stay compact and test coverage does not require a new component-test harness.
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
- [x] Locale key parity check
- [x] `git diff --check main...HEAD`
