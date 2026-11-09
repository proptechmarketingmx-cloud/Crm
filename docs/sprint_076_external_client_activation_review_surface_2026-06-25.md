# Sprint 076: External Client Activation Review Surface

Date: 2026-06-25

Branch: `codex/external-client-activation-review-surface`

## Scope

Sprint 076 adds an explicit local review/edit path for external-client
activation state and supported permission mode.

## Implemented

- Added a `crm-core` service/storage update path for non-deleted
  `external_clients` rows.
- The update path accepts only the current readiness modes: `disabled`,
  `read_only`, and `draft_only`.
- Disabled clients must store `enabled = false` with
  `permission_mode = 'disabled'`.
- Enabled clients must store `enabled = true` with `read_only` or
  `draft_only`.
- Future modes such as `write_with_confirmation` and `write_allowed` are
  rejected by the activation update path.
- Successful activation updates refresh `updated_at` and record local
  audit/sync evidence.
- Added a thin Tauri command, frontend API wrapper, and API mapping test.
- Added Settings Integrations row controls for local activation mode alongside
  the existing per-tool permission editor.

## Deferred

- MCP server/runtime/listener/tool behavior.
- Token or secret creation/storage.
- AI/model-provider behavior.
- Sync server behavior.
- General tool execution.
- Broad new UI routes.
- Destructive schema or data migrations.
- Support for `write_with_confirmation` or `write_allowed`.

## Documentation

- `docs/MCP_READINESS.md` now describes local activation review/edit as an
  active readiness surface while preserving runtime and credential non-goals.
- `docs/PRIVACY.md` clarifies that local activation does not create tokens,
  start listeners, enable sync server behavior, or run MCP/client code.
- `docs/DATA_MODEL.md` documents the supported activation storage pairs and
  audit/sync behavior.
