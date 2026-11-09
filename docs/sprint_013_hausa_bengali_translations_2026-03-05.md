# Sprint 013 — Hausa and Bengali Translations

Date: 2026-03-05 (UTC)
Branch: `sprint-013-hausa-bengali-translations`

## Scope
- Add complete Hausa (`ha`) and Bengali (`bn`) localization support.
- Register both locales in i18n metadata and lazy-loading flow.
- Close remaining v1.1.0 roadmap translation item with chronological docs updates.

## Changes
- Added full locale dictionaries:
  - `src/lib/i18n/ha.json`
  - `src/lib/i18n/bn.json`
- Registered new locales in i18n runtime:
  - `src/lib/i18n/index.ts`
  - Added `ha` + `bn` to `availableLocales`.
  - Added lazy-loader cases for `./ha.json` and `./bn.json`.
- Updated project docs/roadmap chronologically:
  - `docs/sprint_ledger.md`
  - `CHANGELOG.md`
  - `README.md`
  - `CONTRIBUTING.md`

## Validation
- `npm run check` -> passed (0 errors, 0 warnings)
- Locale key parity check vs `en.json`:
  - `fr/es/ar/sw/hi/pt/vi/ha/bn`: missing=0, extra=0

## Lightweight Guardrail Checklist
- [x] `Offline-first` remains intact: localization uses local JSON dictionaries only.
- [x] `Local-first` remains intact: no cloud translation API/service introduced.
- [x] No mandatory proprietary/cloud dependency introduced.
- [x] Runtime overhead remains lightweight (lazy-loaded locale chunks).
- [x] Changelog and sprint ledger updated chronologically with UTC date.
- [x] Work completed on a dedicated sprint branch.

## Outcome
- 900CRM now includes Hausa and Bengali locale support.
- The remaining v1.1.0 “additional languages” roadmap item is now completed.
