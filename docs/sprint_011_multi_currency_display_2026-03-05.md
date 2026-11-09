# Sprint 011 — Multi-Currency Display

Date: 2026-03-05 (UTC)
Branch: `sprint-011-multi-currency-display`

## Scope
- Implement multi-currency-safe pipeline and dashboard value display.
- Avoid offline FX conversion dependencies (no network rate providers).
- Keep runtime overhead lightweight and suitable for constrained hardware.

## Changes
- Added shared currency helpers:
  - `src/lib/utils/currency.ts`
  - Normalizes currency codes to ISO-style uppercase (`USD` fallback).
  - Aggregates amounts by currency for UI summaries.
- Improved dashboard pipeline value handling:
  - `src-tauri/src/commands/dashboard_commands.rs`
    - Added `pipeline_value_by_currency` in `get_dashboard_stats`.
    - Uses SQL `GROUP BY currency` for active-deal totals.
  - `src/lib/api/dashboard.ts`
    - Maps backend currency buckets into typed frontend contracts.
  - `src/routes/Dashboard.svelte`
    - Shows single-currency total when one currency exists.
    - Shows compact mixed-currency preview when multiple currencies exist.
- Improved pipeline stage totals:
  - `src/routes/Pipeline.svelte`
  - Replaced single-currency stage total rendering with grouped per-currency totals.
- Hardened currency data normalization at deal boundaries:
  - `src/lib/api/deals.ts` normalizes backend currency values.
  - `src/lib/components/GlobalModalHost.svelte`
    - Normalizes deal currency before save.
    - Uppercases and limits manual currency entry to 3 characters.
  - `src/lib/components/DealCard.svelte`
    - Uses stable locale formatting path for all currencies.

## Validation
- `npm run check` -> passed (0 errors, 0 warnings)
- `cargo check --target-dir /tmp/900crm-sprint-011-target` -> passed

## Lightweight Guardrail Checklist
- [x] `Offline-first` remains intact: no external FX/rate service dependency added.
- [x] `Local-first` remains intact: all calculations use local DB + local UI helpers.
- [x] No mandatory proprietary/cloud dependency was introduced.
- [x] Runtime overhead is low: grouped SQL aggregate + lightweight frontend mapping.
- [x] Changelog and sprint ledger updated chronologically with UTC date.
- [x] Work completed on a dedicated sprint branch.

## Outcome
- 900CRM now displays mixed-currency pipeline values without forcing incorrect single-currency formatting.
- The v1.1.0 roadmap item for multi-currency display is now implemented.
