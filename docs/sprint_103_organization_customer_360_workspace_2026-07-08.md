# Sprint 103 - Organization Customer 360 Workspace

Date: 2026-07-08
Branch: `codex/organization-customer-360-workspace`

## Scope

This sprint adds the next Customer 360 slice: a first-class organization/account
detail workspace. The goal is to make accounts useful as daily CRM workspaces
instead of only list rows.

## Changes

- Added a direct hash route for `/organizations/:organizationId`.
- Added an organization detail workspace with account profile metadata,
  description, linked people, linked deals, linked account activity, notes, and
  tags.
- Added an Account 360 summary with people count, open deal count, open pipeline
  value by currency, next follow-up, and account health.
- Added contextual account actions for Add Deal and Add Follow-Up using the
  existing global modals with `organizationId` prefilled.
- Added navigation from the Organizations list into account detail.
- Added global search routing into contact and organization detail routes.
- Exposed normalized contact `organizationId` in frontend contact mapping so
  account workspaces can rely on first-class organization links.
- Added focused utility/API tests and browser-smoke coverage for direct account
  route loading, summary metrics, modal prefill, and search-result navigation.

## Explicit Non-Goals

- No schema migration.
- No Rust storage or service behavior changes.
- No deal detail page or drawer.
- No full timeline redesign, files, inbox, email sync, MCP, AI, sync server,
  release, or packaging changes.
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

The next Customer 360 work should deepen interaction timelines and relationship
breadcrumbs. Pipeline detail remains a separate product-depth sprint.
