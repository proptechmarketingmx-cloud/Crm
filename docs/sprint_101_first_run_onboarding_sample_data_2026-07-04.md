# Sprint 101 - First-Run Onboarding and Sample Data

Date: 2026-07-04
Branch: `codex/first-run-onboarding-sample-data`

## Scope

This sprint adds the first visible product-depth onboarding layer from the
post-MVP roadmap. It helps a new local user turn an empty workspace into a
usable CRM flow directly from the dashboard.

It does not add backend behavior, schema changes, MCP behavior, AI behavior,
sync transport, release packaging, or new CRM modules.

## What Changed

- Added a dashboard starter panel that appears while the workspace is missing
  the first contact/account, active deal, or upcoming follow-up.
- Added checklist progress based on existing dashboard stats.
- Added actions for adding a contact, adding a deal, adding a follow-up, and
  opening Settings data tools.
- Added an optional synthetic sample workspace button for empty workspaces.
- The sample workspace creates one organization, one contact, one active deal,
  and one follow-up task through existing frontend API wrappers.
- Dashboard stats, reports, and upcoming activity refresh after sample data is
  created.
- Updated browser E2E shim dashboard responses so created records affect
  dashboard counts during smoke tests.
- Added Playwright workflow coverage for loading sample data, confirming the
  synthetic records are created in the browser shim, and seeing the follow-up on
  the dashboard.

## Product Impact

A first-time user now has a clear next step from the dashboard instead of
starting from four empty KPI cards. The sample workspace is synthetic and local;
it is meant only to demonstrate the existing CRM loop: account, contact, deal,
and follow-up.

## Non-Goals

- No sample data is inserted automatically.
- No new backend seed command was added.
- No import, backup, or restore behavior changed.
- No lead lifecycle, customer 360 workspace, calendar view, reports hub,
  automation, AI, sync, or MCP behavior was added.

## Verification

- `npm run check`
- `npm run test`
- `npm run build`
- `npm run test:e2e`
- `npm run check:release-guardrails`
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
