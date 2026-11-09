# Sprint 100 - Navigation and Settings Reframe

Date: 2026-07-04
Branch: `codex/navigation-settings-reframe`

## Scope

This sprint starts the post-MVP product-depth roadmap by making the existing
menu and Settings surface feel less implementation-led.

It does not add backend behavior, schema changes, MCP behavior, AI behavior,
sync transport, release packaging, or new CRM modules.

## What Changed

- Grouped the left navigation into daily work, review, and admin sections:
  - Workspace: Dashboard, Contacts, Organizations, Pipeline, Activities.
  - Review: Pending Actions, Audit Log.
  - Admin: Settings.
- Added collapsed-sidebar separators so the grouped structure remains visible
  when labels are hidden.
- Added a compact Settings section jump bar for Preferences, Sync, Email,
  Notifications, Integrations, Data, and About.
- Added translation keys for the new navigation groups and Settings section
  labels in English, Hausa, and Arabic.

## Product Impact

The app now presents daily CRM work before safety/admin surfaces. Audit and
pending-action workflows remain available, but they no longer visually compete
with the primary customer and pipeline workflows.

The Settings page remains behaviorally unchanged, but the top-level section
navigation makes data management, integrations, notifications, email, and
preferences easier to find.

## Non-Goals

- No route changes or deep-link changes.
- No settings persistence changes.
- No import/export, backup/restore, external-client, or proposed-action
  behavior changes.
- No customer 360, onboarding, pipeline, calendar, reports, AI, sync, or MCP
  product expansion.

## Verification

- `npm run check`: passed, `svelte-check found 0 errors and 0 warnings`.
- `npm run test`: passed, 22 files and 156 tests.
- `npm run build`: passed with the existing Vite warnings about
  `node:async_hooks` browser externalization.
- `npm run test:e2e`: passed, 6 Playwright smoke tests.
- `npm run check:release-guardrails`: passed, scanned 350 tracked text files
  before staging.
- `git diff --check`: passed, no output.
- `cargo fmt --all -- --check`: passed, no output.
- `cargo check --workspace`: passed.
- `cargo test --workspace`: passed, including 150 `crm-core` tests, 21
  `crm-mcp` tests, 28 `crm-mcp` CLI tests, 7 `crm-sdk` tests, 12 desktop
  command tests, and configured doc tests.
