# Sprint 075: External Client Permissions UI Surface

Date: 2026-06-25

Branch: `codex/external-client-permissions-ui-surface`

## Scope

Sprint 075 adds a Settings Integrations surface for reviewing and editing local
external-client per-tool permission rows through the existing frontend API
wrappers.

## Implemented

- Added a reusable `ExternalClientPermissions` component rendered for each
  listed external client in Settings.
- The component loads permission rows, shows empty/loading/error states, and can
  upsert one permission row by `tool_name`.
- The editor exposes `tool_name`, `can_read`, `can_write`, and
  `requires_confirmation` controls.
- The UI keeps write rows confirmation-gated by forcing
  `requires_confirmation` on when `can_write` is checked. Backend validation
  remains authoritative.
- Focused component tests prove permission list/upsert calls and absence of
  activation, token/secret, or MCP runtime controls in the permission editor.

## Deferred

- Client activation.
- Permission mode editing.
- Token or secret creation/storage.
- MCP server/runtime/listener/tool behavior.
- AI/model-provider behavior.
- Sync server behavior.
- Import/export behavior.
- Schema, migration, Rust/core, Tauri command, and raw SQL changes.

## Documentation

- `docs/MCP_READINESS.md` now treats the permission row review/edit UI as an
  active readiness surface while retaining activation/runtime/token non-goals.
- `docs/PRIVACY.md` notes that permission row review/edit is local readiness
  behavior and does not activate clients or run MCP/client code.
