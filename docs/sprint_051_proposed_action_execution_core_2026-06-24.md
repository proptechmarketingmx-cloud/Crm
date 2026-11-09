# Sprint 051 - Proposed Action Execution Core

Date: 2026-06-24
Branch: `codex/proposed-action-execution-core`
Scope: First safe `crm-core` execution path for approved `create_activity_draft` proposed actions.

## Summary

- Added core approval execution for pending proposed actions where `tool_name`
  is `create_activity_draft`.
- `action_type` may be `create_activity_draft` or the compatible legacy category
  value `create_activity`; it cannot make an unsupported `tool_name` executable.
- Routed execution through the existing activity creation service/storage
  behavior so activity validation, audit entries, sync changelog rows, and
  legacy contact/deal activity links are preserved.
- Marked successfully approved draft actions as `executed` with both
  `approved_at` and `executed_at` set.
- Kept approval transactional: malformed input, unsupported tool/action types,
  missing linked entities, or other execution failures leave the proposed action
  pending and do not write approval/execution audit entries.
- Kept rejection decision-only and non-executing.

## Supported Draft Input

`input_json` for this sprint is:

```json
{
  "title": "Call Amina",
  "activity_type": "call",
  "description": "Confirm next steps",
  "due_at": "2026-06-25T09:00:00Z",
  "linked_entities": [
    { "entity_type": "contact", "entity_id": "contact-id" },
    { "entity_type": "organization", "entity_id": "organization-id" },
    { "entity_type": "deal", "entity_id": "deal-id" }
  ]
}
```

- `title` is required.
- `activity_type` is optional and defaults to `task` when absent.
- `description` is optional.
- `due_at` is optional and maps to the existing activity `due_date` field.
- `linked_entities` is optional and supports `contact`, `organization`, and
  `deal`.
- The current core path supports at most one linked contact and one linked deal
  because those map to the existing legacy activity mirror fields. Organization
  links use the existing first-class activity link helper.

## Boundaries

- No MCP server, listener, tools, prompts, resources, token handling, secret
  handling, AI behavior, broader permission modes, UI behavior, Tauri command
  behavior, or sync server behavior was added.
- No raw SQL was added to Tauri commands or `crm_engine`.
- Unsupported proposed-action tool/action types stay pending and return an
  explicit invalid-input error when approval is attempted.
- Mismatched tool/action identities stay pending and do not create activities or
  approval/execution audit entries.

## Validation

- Focused `cargo test -p crm-core` passed while implementing the core path.
- Full acceptance-gate results are recorded in the final builder report for this
  sprint.
