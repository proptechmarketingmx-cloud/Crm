# Sprint 104 - Customer 360 Relationship Timeline

Date: 2026-07-08
Branch: `codex/customer-360-relationship-timeline`

## Scope

This sprint deepens the Customer 360 layer already started for contact and
organization/account records. The goal is to make detail timelines explain what
each activity is connected to without adding new backend behavior.

## Changes

- Added opt-in relationship breadcrumbs to the shared `ActivityFeed` component.
- Contact and organization breadcrumbs navigate only to existing safe detail
  routes; deal breadcrumbs are display-only until a deal detail route exists.
- Added reusable relationship filtering for detail timelines so contact detail
  includes both legacy `activity.contactId` activities and explicit
  `activity_links` contact activities where no legacy mirror is present.
- Kept account activity based on first-class organization `activity_links`
  rather than free-text organization names.
- Added detail-timeline ordering that keeps scheduled activity first and places
  undated activity afterward by recent update time.
- Added relationship label maps for contact/account detail timelines using the
  existing contact, organization, and deal lookup APIs.
- Added unit coverage for duplicate breadcrumb collapse, deleted-link exclusion,
  link-only contact activity filtering, detail timeline ordering, and feed label
  map generation.
- Added browser-smoke coverage for contact link-only activity visibility and
  account activity relationship breadcrumbs.
- Updated the product benchmark and sprint ledger.

## Reviewer Guardrails

The pre-sprint reviewer flagged these required boundaries:

- Contact detail must not rely only on the legacy contact activity mirror.
- Contact detail must not leak singleton activity filters into the Activities
  page.
- Breadcrumb lookup work must stay opt-in for detail timelines, not Dashboard.
- Breadcrumbs must collapse duplicates, exclude deleted links, and avoid fake
  deal routes.
- Add-follow-up refresh must continue to use existing modal/link refresh
  signaling.

## Explicit Non-Goals

- No schema migration.
- No Rust storage, service, or Tauri command changes.
- No deal detail page or drawer.
- No files, inbox, email sync, MCP, AI, sync server, release, or packaging work.
- No dashboard-wide relationship lookup expansion.
- No destructive data rewrite.

## Verification

Run for sprint acceptance:

```sh
npm run check
npm run test
npm run build
npm run test:e2e
npm run check:release-guardrails
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
git diff --check
git fsck --full --no-progress
```

## Follow-Up

The next product-depth sprint should move to Pipeline Depth: deal detail,
stage-aging/stale-deal guidance, missing-next-step warnings, forecast context,
and conversion summary surfaces.
