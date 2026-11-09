# Sprint 015 - Organization UI Surface Preserve

Date: 2026-06-24
Branch: `codex/organization-ui-surface-preserve`
Scope: Feature-preserving frontend/API exposure for existing first-class organization backend commands.

## Summary

- Added a frontend organization API wrapper for the existing Tauri commands: create, get, list, update, delete, and link contact to organization.
- Added focused API mapping tests, including explicit update clear semantics where blank or null optional fields map to `null` while omitted fields remain omitted.
- Added a Svelte 5 `.svelte.ts` organization store with a `.ts` compatibility shim, matching the runtime-state pattern used by the existing stores.
- Added an Organizations hash route and sidebar navigation item.
- Added organization i18n keys to all registered locale JSON files: ar, bn, en, es, fr, ha, hi, pt, sw, vi.

## Preservation Notes

- Contacts still keep their existing legacy `organization` text field behavior; this sprint does not replace contact forms or remove legacy columns.
- The organization route exposes first-class records separately, plus a focused contact-ID link action for the existing `link_contact_to_organization` command.
- No backup, email, reminder, report/dashboard, custom-field, multi-currency, MCP, AI, sync server, or migration behavior was intentionally changed.
- Runtime state continues to use `.svelte.ts` stores with `.ts` re-export shims.
