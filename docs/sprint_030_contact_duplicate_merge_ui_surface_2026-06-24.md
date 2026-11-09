# Sprint 030 - Contact Duplicate Merge UI Surface

Date: 2026-06-24
Branch: `codex/contact-duplicate-merge-ui-surface`
Scope: User-facing contact duplicate warning and merge workflow using existing contact merge semantics.

## Summary

- Added a read-only `CrmCore::list_contact_duplicate_candidates()` service surface.
- Kept duplicate candidate SQL in `crates/crm-core/src/storage/contacts.rs`.
- Reported active contact pairs with exact trimmed email or phone matches, ignoring blank values and soft-deleted contacts.
- Returned source and target ids, display labels, match type, matched value, and reason for UI review.
- Avoided repeated pair rows when the same contacts match by both email and phone.
- Added a thin Tauri command and typed frontend API wrapper for duplicate candidates.
- Added a Contacts route warning panel with candidate selection, source/target direction preview, optional direction swap, explicit confirmation, and merge execution through the existing `mergeContacts` API path.
- Refreshed contacts and duplicate candidates after a successful merge.
- Updated all locale JSON files with contact duplicate merge keys using English fallback strings where localized copy was not practical.

## Boundaries

- Existing contact merge semantics were reused and not rewritten.
- No destructive normalization migration, legacy column removal, MCP, AI, sync server, or import rewrite was added.
- No raw SQL was added to Tauri commands or `crm_engine`.
- Svelte components continue to use frontend API/store layers rather than direct `invoke()` calls.

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
- [x] Locale key parity check
