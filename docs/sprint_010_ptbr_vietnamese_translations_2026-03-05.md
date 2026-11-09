# Sprint 010 — Portuguese (Brazil) and Vietnamese Translations

Date: 2026-03-05 (UTC)
Branch: `sprint-010-ptbr-vietnamese-translations`

## Scope
- Add complete Portuguese (Brazil) and Vietnamese locale support.
- Keep i18n loading lightweight and aligned with existing lazy-load architecture.
- Preserve open-source/offline-first constraints and documentation chronology.

## Changes
- Added full translation dictionaries:
  - `src/lib/i18n/pt.json`
  - `src/lib/i18n/vi.json`
- Registered both locales in i18n locale metadata and lazy loader:
  - `src/lib/i18n/index.ts`
  - Added locale metadata entries for `pt` and `vi`.
  - Added loader cases for `./pt.json` and `./vi.json`.
- Updated project documentation to reflect new language support:
  - `README.md`
  - `CONTRIBUTING.md`
  - `CHANGELOG.md`
  - `docs/sprint_ledger.md`

## Validation
- `npm run check` -> passed (0 errors, 0 warnings).
- Translation parity check against English base locale:
  - `pt`: missing=0, extra=0
  - `vi`: missing=0, extra=0

## Lightweight Guardrail Checklist
- [x] `Offline-first` remains intact: localization runs fully local with static JSON files.
- [x] `Local-first` remains intact: no cloud translation APIs or remote assets were introduced.
- [x] No mandatory proprietary/cloud dependency was introduced.
- [x] No heavy runtime overhead added (lazy-loaded locale chunks, no background jobs).
- [x] Bundle growth is minimal and justified by language expansion.
- [x] Changelog and sprint ledger updated chronologically with UTC date.
- [x] Work completed on a dedicated sprint branch.

## Outcome
- 900CRM now includes complete Portuguese (Brazil) and Vietnamese UI translations.
- The previously in-progress localization item for these two languages is now completed.
