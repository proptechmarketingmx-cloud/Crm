# Sprint 002 — Warning Hardening

Date: 2026-03-05 (UTC)
Branch: `sprint-002-warning-hardening`

## Scope
- Remove remaining frontend and backend warnings after Sprint 001 stabilization.
- Keep runtime behavior unchanged while improving accessibility semantics and build hygiene.

## Changes
- Reworked card interactions to use semantic button elements where interactive behavior exists (`ContactCard`, `DealCard`).
- Updated local element refs to rune state refs in Svelte components (`NoteEditor`, `TagPicker`).
- Fixed label association warning in `ContactDetail` by using non-label text for radio-group heading.
- Scoped theme-based selectors with `:global(...)` for Svelte component CSS (`StatCard`, `Settings`).
- Fixed invalid dark-mode media/token selector syntax in global `app.css`.
- Removed unused Rust imports in dashboard command and datetime utility modules.

## Validation
- `npm run check` -> passed with 0 errors, 0 warnings.
- `npm run build` -> passed.
- `cargo check` (in `src-tauri/`) -> passed.

## Outcome
- Warning baseline is clean and the project remains buildable in frontend and Rust backends.
