# Sprint 105 - Pipeline Deal Guidance Drawer

Date: 2026-07-08
Branch: `codex/pipeline-deal-guidance-drawer`

## Scope

This sprint starts the Pipeline Depth roadmap by making deals inspectable and
actionable directly from the Pipeline board. It uses existing deal and activity
data only.

## Changes

- Added a deal guidance drawer opened from Pipeline deal cards.
- Added pipeline card guidance badges for open and closed deal states.
- Added guidance states for:
  - `Needs Follow-Up` when an open deal has no linked next activity.
  - `Overdue` when the next linked activity is overdue.
  - `Stale` when the deal has not changed recently.
  - `On Track` when an open deal has a current next activity.
  - `Closed Won` and `Closed Lost` without next-step warnings.
- Added weighted forecast display from existing value and probability fields.
- Added a stage-age proxy labeled as days since last deal update. This does not
  claim true historical stage dwell time because the current data model does
  not store stage history.
- Added linked activity loading for the drawer using existing legacy
  `activity.dealId` and explicit `activity_links` deal relationships.
- Added guarded loading and unavailable states so card badges and the drawer do
  not infer `Needs Follow-Up` from an unresolved or failed activity-context
  load.
- Split activity/link readiness from display-name lookup readiness so guidance
  can become accurate without waiting on secondary breadcrumb labels.
- Added drawer dialog focus handling: focus moves into the drawer, Escape
  closes from the document handler, Tab wraps within the drawer, and the drawer
  is hidden while the existing global activity modal is open.
- Added an explicit linked-activity unavailable message on context-load failure
  instead of showing the generic empty activity state.
- Removed the primary Add Follow-Up CTA for closed deals while keeping the
  lower linked-activity Add Activity action available.
- Moved Pipeline route bootstrapping to a one-shot rune effect so the hash route
  initializes guidance, filters, and relationship lookups reliably in browser
  smoke runs.
- Added Add Follow-Up action from the drawer through the existing global
  activity modal, prefilled with the selected deal and refreshed through the
  existing relationship refresh signal.
- Preserved current global search deal routing to Pipeline; no deal detail route
  was added.
- Added focused unit coverage for guidance state derivation, weighted forecast,
  stage-age proxy, and next-activity ordering.
- Added component coverage for drawer loading/unavailable states, Escape close,
  and closed-deal CTA behavior.
- Added browser-smoke coverage for visible deal creation, drawer opening,
  guidance state, Add Follow-Up prefill, and linked activity refresh.

## Reviewer Guardrails

The pre-sprint reviewer recommended keeping this as a narrow deal guidance
drawer sprint and deferring conversion summaries. Required boundaries:

- Use existing deal fields and existing activity relationships.
- Include both legacy `dealId` and explicit `activity_links` deal activity.
- Do not mutate global Activities filters.
- Avoid fake deal routes or global search deep-linking.
- Be explicit that stage age is a last-updated proxy, not true stage history.
- Withhold real sales guidance until linked activity context is loaded, and do
  not treat context-load failures as empty activity lists.
- Keep the drawer accessible when used as a modal surface and avoid exposing two
  simultaneous modal dialogs when the global Add Activity modal opens.

## Explicit Non-Goals

- No schema migration.
- No Rust storage, service, or Tauri command changes.
- No deal page route.
- No editable deal form, notes/tags/custom-field management, import/export, or
  reports hub work.
- No stage conversion summaries in this sprint.
- No MCP, AI, sync server, release, packaging, or broad Pipeline redesign.

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

Continue Pipeline Depth with conversion summaries, forecast/close-date views,
and deeper deal editing or detail routing once the drawer interaction has
settled.
