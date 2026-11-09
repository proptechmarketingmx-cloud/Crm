# Changelog

All notable changes to 900CRM are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added — 2026-08-17
- Dashboard, Pipeline, and Reports can filter the current queue by
  owner. Pipeline cards and the stale-deal list show the name when it
  is set.
- Contacts, deals, and organizations can store an optional local owner
  name. Create and edit forms include the field. Contacts and
  Organizations can filter by owner. This is a free-text name, not a
  user account or team.
- The Pipeline deal drawer uses the same Next step strip as the deal
  page, so the primary button matches overdue work, a missing
  follow-up, or a missing close date.
- Reports can download the current snapshot as an unencrypted CSV.
  The file is the numbers on the page, including the current-stage
  honesty note. It is not historical conversion.
- Pipeline columns show a drop target while a deal is dragged, skip
  same-stage drops, and confirm the new stage after a move.
- Contact, deal, and account workspaces show a Next step strip. The
  primary button matches the real next action: complete an overdue
  follow-up, convert a lead, add a follow-up, or set a deal close date.
- The Review sidebar shows how many proposed actions are waiting, so
  Pending Actions is visible only when there is something to decide.
- Activities Month shows the same follow-ups on a Monday-aligned month
  grid. Days outside the month stay visible but dimmed. A crowded day
  shows the first three cards and a leftover count.
- Activities can switch from the due-bucket list to a Monday–Sunday week
  grid. The same follow-ups appear on their due day. Drag a card to
  another day to reschedule. A day + button opens Add Activity for that
  date. There are still no times, recurrence, or external calendars.
- An empty Dashboard now asks if you already have customers. Import data
  opens the import wizard. Make a backup opens Settings → Data. Both
  stay local and unencrypted.

### Fixed — 2026-08-16
- After a restore, the app reloads the local database so restored data
  appears without a manual restart.
- Creating a lead now writes in one step so a failed create cannot leave a
  half-written person.
- Pipeline follow-up guidance loads activities for the deals on the board
  instead of stopping at the first 500 rows, and the custom-field index
  reloads after deal data changes.
- Database row errors that were previously hidden now surface instead of
  failing silently.
- Import rejects unsafe paths and symlink escapes so files stay in the
  intended location. Desktop import reads and local-file bookmarks now
  go through that same Rust path check.
- Passwords and similar secrets are no longer written into application
  logs.
- In-app update checks stay unavailable until signed public releases
  exist.
- Docs now cite schema version 13, describe the audit log as
  application-append-only (not a WORM store), and treat Arabic RTL as
  direction-enabled with layout polish still in progress.
- Contact merge has a regression test so related records stay with the
  surviving person.
- Sidebar collapse labels are translated, leftover English in Arabic,
  Bengali, French, and Hausa is reduced, and rust-cross CI now runs
  `cargo test --workspace`.

### Changed — 2026-08-16
- Settings is now three panes: Appearance, Data, and Integrations. Backup
  and email no longer sit on the same long page.

### Fixed — 2026-08-15
- Desktop launch no longer aborts when the notification plugin reads an empty
  config object.
- Saved views, contact/lead create, and notes/tags now reach the backend. Tauri
  was expecting camelCase argument names while the app sends snake_case.

### Added — 2026-08-15
- Empty Leads, Pipeline, Activities, and Reports pages now say what to do
  first: add a lead, add a deal, schedule a follow-up, or add a deal so
  reports can fill in.
- Reports can save the current focus — Pipeline, Activities, or Stale Deals —
  as a named view and reopen that view later.

### Added — 2026-08-14
- Leads is a dedicated Workspace list of people with lifecycle lead. Converted
  customers stay on Contacts. This uses the existing contact records.
- Activities can save the current type, status, due-bucket, and custom-field
  filters as a named view and reopen that view later.
- Pipeline can filter the board by attention: Needs Follow-Up, Stale, or
  Overdue. That filter can be saved with a named view.
- Reports lists stale deals: open deals that have not changed in 14 days and
  still have a next step. Each row opens the deal. This uses the same local
  rule as the pipeline Stale badge.
- Contacts list shows health and the next follow-up using the same local rules
  as Customer 360.
- Organizations list shows account health and the next follow-up using the same
  local rules as Account 360.
- Dashboard shows a daily attention queue for overdue follow-ups, open deals
  without a next step, and leads waiting to be worked. Each row opens the
  related record.
- Pipeline can save the current deal-name search and custom-field filter as a
  named view and reopen that view later.

### Added — 2026-08-13
- Organizations can save the current search and country filter as a named
  view and reopen that view later.
- Contacts can save the current list filters as a named view and reopen
  that view later.
- Contacts, organizations, and deals can store website and local-file
  bookmarks. 900CRM saves the location as text and does not copy or upload
  the file.
- Person contacts now have a lifecycle: lead or customer. Existing people stay
  customers. New people can be created as leads, filtered on the contacts list,
  and converted to customers from the contact page.
- Deals have a full workspace page at `#/deals/:id` with stage, guidance,
  people, notes, tags, and follow-ups. Search and linked-deal rows open that
  page. The pipeline board still has a quick drawer with an Open deal action.

### Changed — 2026-08-13
- Settings Sync is labeled as not available. The toggle and server URL are
  gone so the app no longer looks like it can sync.
- Organization website links open through the OS handler after an http(s)
  allow-list check.
- Minimum window size is 800×520 so common 1024×600 netbooks can open the app.
- Store and route toasts go through translation keys instead of hardcoded
  English.
- Deal search and `#/pipeline/:id` open the existing deal guidance drawer.
- Replaced the updater public key with a new keypair. No public installer
  depended on the previous key. The private key stays outside the repository.
- Set the application version to `0.9.0` across npm, Cargo, and Tauri
  manifests. This is an honest source-evaluable alpha identity; no public
  `1.0.0` installer has been published.
- First-run sample workspace now links the sample person to the sample
  organization so Account 360 people lists are not empty.
- Settings no longer collects or persists SMTP/IMAP passwords. Leftover
  password values are cleared on settings load. Email remains a TCP
  reachability probe plus local `mailto:` compose.
- Hidden the Settings update-check control until a signed public release
  exists. The updater plugin remains compiled in but is not user-initiated
  from the current UI.

### Security — 2026-08-13
- Email connection tests now reject IPv4-mapped IPv6 forms of loopback,
  private, link-local, and CGNAT addresses.
- Backup validate and restore now use the same path guard as import/export
  before the live database is closed.

### Security — 2026-08-04
- Tightened Tauri capability grants and narrowed the filesystem scope exposed to
  the WebView so IPC and disk access match only what the app actually needs.
- Added SSRF guards (private/loopback/link-local and cloud-metadata address
  blocklist) to the email connection test so the reachability probe cannot be
  used to scan internal networks.
- Added command-boundary path validation (defense-in-depth) for import, export,
  and backup paths to keep operations within the expected data directory.
- Added an import file-size cap (100 MB) and non-regular-file rejection so a
  compromised WebView cannot exhaust memory or hang the backend on huge or
  non-regular inputs.

### Fixed — 2026-08-04
- Contact merge now reassigns related records (deals, activities, notes, tags,
  custom fields, and organization links) to the surviving contact instead of
  orphaning them when the source contact is removed.
- Full-text search no longer aborts on metacharacters such as quotes and
  asterisks; special characters are sanitized before reaching the FTS index.
- Pagination offset no longer overflows on large page values.
- Activity-completion and import-audit errors are no longer silently swallowed;
  failures now surface to the caller.
- Activity completion no longer records a phantom sync/audit transition when the
  requested state already matches the current state, and now records the actual
  prior value.
- External-client permission evaluation and its audit entry now run in a single
  transaction so the audit trail cannot diverge from the decision.

### Changed — 2026-08-04
- Sync status now honestly reports `not_implemented` instead of a fake
  `success` result from `trigger_sync`; the local changelog is still written, but
  no transport runs.
- Added deal currency (ISO 4217) and expected-close date validation, and contact
  update input validation.
- Clamped global-search and upcoming-activity result limits to bounded maximums.

### Build/CI — 2026-08-04
- Added a cross-OS (Windows, macOS, Linux) Rust check matrix.
- Added Rust build caching.
- Added Dependabot configuration and dependency-hygiene cleanup.

### Fixed — 2026-08-07
- Fixed the manual release packaging workflow: the `tauri build` invocation
  was mangling flags through two layers of npm script forwarding (the same
  defect that affected CI), so it never successfully produced a bundle. It now
  invokes the Tauri CLI directly (`npx tauri build --bundles ...`) from the
  desktop workspace.
- Removed the macOS (`.dmg`) build target from the release workflow until Apple
  Developer ID signing + notarization credentials are available; unsigned and
  unnotarized macOS builds would be blocked by Gatekeeper for the non-technical
  target audience.

### Changed — 2026-08-07
- Release/readiness docs (RELEASE.md, ALPHA_READINESS.md) updated: the GitHub
  Actions billing/spending-limit blocker was resolved as of 2026-08-07 and the
  CI suite now runs green across Ubuntu, Windows, and macOS.

### Added — 2026-06-24
- Added release-readiness documentation that states the current verification-only
  CI status, manual release checklist, required future artifacts, and release
  packaging boundaries.
- Added small synthetic contact and organization CSV samples for manual import
  smoke testing.

### Changed — 2026-06-24
- Updated public release wording so README and changelog text no longer claim
  published installers, multi-OS release CI, or automated release builds exist
  today.

### Added — 2026-03-05
- Added a global modal host component that centralizes create flows for contacts, deals, and activities from any route.
- Wired root layout rendering for global modal support so dashboard, pipeline, contacts, and detail-page quick actions execute consistently.
- Added context-aware modal prefill for linked entity creation (`contactId`, `dealId`) and stage-aware deal creation.
- Added custom field foundation APIs:
  - Rust storage layer for custom field definitions and values.
  - Tauri IPC commands for listing/creating/updating/deleting definitions and setting/listing values.
  - Frontend API wrapper (`src/lib/api/customFields.ts`) for typed invoke integration.
- Added reusable dynamic custom-field renderer component for create/edit forms (`src/lib/components/CustomFieldInputs.svelte`).
- Added custom field UI integration for contact/deal/activity create flows in the global modal host.
- Added custom field UI integration for contact detail edit flow, including value loading, editing, save, and cancel reset behavior.
- Added reporting backend contracts for analytics:
  - Pipeline conversion report endpoint (`get_pipeline_conversion_report`).
  - Activity funnel report endpoint (`get_activity_funnel_report`).
  - Typed frontend API wrapper (`src/lib/api/reports.ts`).
- Added dashboard report UI integration for analytics:
  - Pipeline conversion and activity funnel cards on the dashboard.
  - Lightweight stage/type bar visualization with localized labels.
  - Graceful partial-failure fallback so core dashboard stats still load if reports are unavailable.
- Added custom-field filter UX for primary operational views:
  - Contacts list supports backend-filtered custom-field searching (pagination-safe).
  - Pipeline and Activities support lightweight custom-field filtering via local value indexes.
  - Added bulk custom-field value command for per-entity-type filter lookups.
- Added desktop reminder notifications for upcoming activities:
  - Added reminder polling service with at-most-once notifications per activity due timestamp.
  - Added settings for desktop reminders (`notifications_enabled`, `reminder_lead_minutes`).
  - Added schema migration v3 to backfill reminder setting defaults for existing installs.
- Added complete Portuguese (Brazilian) and Vietnamese locale support:
  - Added `src/lib/i18n/pt.json` and `src/lib/i18n/vi.json`.
  - Registered `pt` and `vi` in `availableLocales` and lazy locale loading.
- Added multi-currency display support across dashboard and pipeline:
  - Added grouped pipeline currency totals in dashboard stats (`pipeline_value_by_currency`).
  - Updated dashboard KPI rendering to handle mixed-currency pipelines without forcing a single-currency format.
  - Updated pipeline stage totals to show per-currency totals for mixed-currency columns.
  - Added currency normalization helpers and stricter deal-currency input normalization.
- Added optional email integration (IMAP/SMTP):
  - Added email integration settings for SMTP/IMAP endpoints and local compose workflow.
  - Added lightweight SMTP/IMAP endpoint reachability test command (`test_email_server_connection`).
  - Added schema migration v4 to seed email integration setting defaults for existing installs.
  - Added contact detail action to open local mail composer via `mailto:`.
- Added complete Hausa and Bengali locale support:
  - Added `src/lib/i18n/ha.json` and `src/lib/i18n/bn.json`.
  - Registered `ha` and `bn` in `availableLocales` and lazy locale loading.
- Added schema migration v2 analytics indexes to keep report queries efficient on constrained hardware.
- Added a lightweight open-source and low-resource guardrail checklist and wired it into the PR template/workflow.

### Fixed — 2026-03-05
- Resolved Svelte/Tauri build blockers (invalid event attributes, modal snippet typing, SVG attribute typing, and Tauri config schema compatibility).
- Unified frontend IPC wrappers with backend command contracts (snake_case argument names, response mapping, and stage/type/status normalization).
- Added missing sync IPC commands (`get_sync_status`, `trigger_sync`) and activity revert command (`mark_activity_incomplete`).
- Aligned hash-route rendering so `/contacts`, `/contacts/:id`, `/pipeline`, `/activities`, and `/settings` render correctly from the root page.
- Aligned import/export flows with implemented backend CSV commands (`import_*_csv`, `export_*_csv`) and file-path based operations.
- Removed remaining Svelte/Tauri warning classes (a11y semantics, rune reactivity refs, dark-theme selector scoping, CSS media-token syntax, and Rust unused imports).
- Verified clean warning baseline: `npm run check` now reports 0 errors and 0 warnings; `cargo check` is warning-free.
- Corrected deal modal translation key usage from `deals.dealName` to existing locale key `deals.name`.
- Fixed TypeScript nullability issues in contact-detail quick actions for add-deal/add-activity modal context opening.
- Fixed contact-detail save/cancel consistency by including custom field state in dirty/reset/save handling.

---

## [1.0.0] — Future stable target

This heading is **not** a shipped release. Current manifests use `0.9.0`.
Do not copy the bullets below into GitHub Release notes.

The eventual stable 1.0 should be a free, open-source, offline-first desktop
CRM built with Tauri v2 (Rust), Svelte 5, and SQLite. Public installers have
not been published. See [Release Readiness](docs/RELEASE.md) and
[Release Roadmap](docs/ROADMAP.md).

Implemented alpha behavior lives in `[Unreleased]` and the current app.
The following historical target list is kept for planning only. Several
items are still incomplete or were overstated:

- Pipeline stages are the built-in set; there is no rename/reorder UI.
- Deal stage-transition history is not stored.
- Contacts do not have a job-title field or file attachments.
- Search results do not yet have full keyboard list navigation.
- CSV import is generic mapped CSV, not vendor-specific Salesforce/HubSpot
  connectors.
- There is no Settings "data directory location" control.
- Bulk mark-complete for activities is not implemented.
- Installer size, RAM, and "10,000+ contacts" claims are not release
  evidence.

---

## Version History Format

This project uses [Semantic Versioning](https://semver.org/):

- **MAJOR** version (1.x.x → 2.x.x): incompatible changes such as database schema migrations that cannot be done automatically, or removal of major features
- **MINOR** version (x.1.x → x.2.x): new features added in a backwards-compatible manner
- **PATCH** version (x.x.1 → x.x.2): backwards-compatible bug fixes

Pre-release versions use a suffix: `1.1.0-alpha.1`, `1.1.0-beta.2`, `1.1.0-rc.1`.

[Unreleased]: https://github.com/900Labs/900CRM/commits/main
[1.0.0]: docs/RELEASE.md
