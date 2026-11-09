# Sprint 106 - Pipeline Stage Metrics and Forecast

Date: 2026-07-08
Branch: `codex/pipeline-stage-metrics-forecast`

## Scope

This sprint continues the Pipeline Depth roadmap by adding a board-level
forecast and stage-health overview directly above the Kanban board. It uses
already-loaded frontend deal data plus Sprint 105 guidance state only.

## Changes

- Added a pure `pipelineMetrics` utility for deriving Pipeline forecast and
  stage-health metrics from visible deals.
- Added board-level forecast cards for:
  - open pipeline value grouped by currency;
  - weighted forecast grouped by currency;
  - expected weighted forecast closing in the next 30 days;
  - win rate from currently visible Closed Won and Closed Lost deals.
- Added close-date health buckets for overdue expected close dates, missing
  expected close dates, and later-dated opportunities.
- Added stage-health rows for every Pipeline stage, including Closed Won and
  Closed Lost counts.
- Kept closed deals out of open forecast totals and weighted open stage
  forecast while still showing closed-stage counts and win-rate context.
- Added per-stage deal count, total value by currency, weighted forecast by
  currency, average probability, average update-age proxy, and focus state.
- Kept guidance-derived risk counts honest:
  - Loading appears while activity context is unresolved.
  - Unavailable appears if activity context fails.
  - Missing context is not counted as `Needs Follow-Up`.
- Made the overview follow the current custom-field filtered board so visible
  metrics match visible cards.
- Added focused unit coverage for multi-currency grouping, close-date buckets,
  closed-stage exclusion from open forecast, risk aggregation, and visible-deal
  stage averages.
- Extended Pipeline browser-smoke coverage to assert the overview renders in
  deal creation and drawer workflows.
- Serialized the stateful workflow smoke file so modal/state interactions in
  the browser Tauri shim do not race under parallel Playwright workers.

## Reviewer Guardrails

The pre-sprint reviewer recommended a frontend-only Pipeline Stage Metrics and
Forecast Overview:

- Derive metrics from existing deal fields and Sprint 105 guidance state.
- Keep metrics scoped to the current filtered board, or explicitly label
  otherwise. This sprint uses the current filtered board.
- Do not add schema, Rust storage, Tauri commands, MCP, AI, sync server,
  release, or packaging changes.
- Avoid claiming true stage conversion history because the data model does not
  store historical stage transitions.
- Keep multi-currency values grouped by currency instead of summing unlike
  currencies.
- Preserve Sprint 105's activity-context loading/error semantics.

## Explicit Non-Goals

- No historical stage conversion or stage-transition analytics.
- No schema migration.
- No Rust storage, service, or Tauri command changes.
- No deal edit form, deal route, drag/drop redesign, reports hub, MCP, AI,
  sync server, release, or packaging work.

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

Focused checks run during implementation:

```sh
npm run test
npm run test:e2e -- -g "Pipeline"
```

## Follow-Up

Continue Pipeline Depth with deal editing/detail routing and stronger drag/drop
confidence cues, or move to Activities Calendar and Follow-up Center if the
Pipeline board is accepted as deep enough for the current alpha.
