# Sprint 102 - Customer 360 Summary Workspace

Date: 2026-07-04
Branch: `codex/customer-360-summary-workspace`

## Scope

This sprint adds the first visible Customer 360 layer to the contact detail
workspace. It improves daily CRM usefulness by putting the contact's open deal
count, open pipeline value, next follow-up, recent activity, and attention
status above the edit form.

## Changes

- Added a contact detail Customer 360 summary section.
- Reused the existing contact-scoped deal and activity store data; no new IPC,
  storage, schema, or backend behavior was added.
- Reused the existing global add-deal and add-activity modals, passing the
  current contact ID so follow-up/deal creation stays connected to the record.
- Made linked deals react to the deal store instead of freezing the sidebar list
  at initial load.
- Added Playwright browser-smoke coverage for a contact with a linked open deal
  and scheduled follow-up.
- Added Playwright browser-smoke coverage for sidebar route switching across
  primary workspace routes.
- Added a small shared hash-route bridge so contact row/sidebar navigation can
  force route-renderer sync immediately instead of relying only on browser
  `hashchange` timing.
- Changed contact detail loading to react to the `contactId` prop so direct
  contact URLs load reliably in browser smoke and Tauri shell mode.
- Updated product benchmark docs to mark Customer 360 work as started, not
  complete.

## Explicit Non-Goals

- No organization detail workspace.
- No deal detail workspace or deal drawer.
- No timeline redesign, files, inbox, email sync, or communication history.
- No schema, storage, Rust, MCP, AI, sync server, release, or packaging changes.

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
git fsck --full --no-progress
```

## Follow-Up

The next Customer 360 work should add an organization/account detail workspace
and a fuller timeline/relationship layout. Pipeline guidance and activity
calendar depth remain separate product-depth sprints.
