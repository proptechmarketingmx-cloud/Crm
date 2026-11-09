# Sprint 077: External Client Access Audit Foundation

Date: 2026-06-25

Branch: `codex/external-client-access-audit-foundation`

## Scope

Sprint 077 adds local audit evidence for external-client permission/access
evaluation attempts. This is readiness/security-evidence work only.

## Implemented

- Explicit external-client read permission evaluations now record
  `audit_log` rows with action
  `evaluate_external_client_read_permission`.
- Explicit external-client draft permission evaluations now record
  `audit_log` rows with action
  `evaluate_external_client_draft_permission`.
- Draft permission checks inside `create_external_proposed_action_stub` record
  the same draft-evaluation audit context before proposed-action insertion.
- Denied proposed-action draft checks commit the audit row and return the
  existing permission denial without creating `proposed_actions` rows.
- Unknown or deleted client evaluation attempts keep their existing not-found
  error and record attempted client/tool context where the caller provided both
  values.
- Audit JSON context includes `client_id`, `tool_name`, access kind, mode when
  available, `allowed`, decision reason, result status, and optional entity
  scope when the current call path already has it.
- Evaluation-only audit entries do not write `sync_changelog` rows.

## Deferred

- MCP server/runtime/listener/tool behavior.
- Token or secret creation/storage.
- AI/model-provider behavior.
- Sync server behavior.
- Arbitrary tool execution or direct write execution.
- Broad UI changes.
- Schema changes or destructive migrations.
- Permission decision semantic changes, including support for reserved future
  modes.

## Documentation

- `docs/MCP_READINESS.md` now treats local external-client permission/access
  evaluation audit entries as readiness evidence while preserving the
  unimplemented MCP runtime boundary.
- `docs/PRIVACY.md` notes that local evaluation audits record
  client/tool/access-kind decision context without sync changelog entries.
- `docs/DATA_MODEL.md` documents the new audit actions, JSON context, and
  proposed-action draft-guard audit behavior.
