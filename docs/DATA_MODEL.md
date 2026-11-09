# Data Model

Date: 2026-06-25
Last updated: 2026-08-17

This document describes the current 900CRM local data model as implemented in
`crates/crm-core` and exposed through the desktop app. It is a public baseline
for contributors and downstream integrators.

900CRM is offline-first. The local SQLite database is the source of truth for
normal desktop use, and `crm-core` is the required boundary for all current and
future data access.

## Storage Boundary

- The desktop shell opens `900crm.db` in the platform app data directory.
- SQLite runs in WAL mode with foreign keys enabled.
- Schema state is tracked with `PRAGMA user_version`; the current schema version
  is `14`.
- Migrations are idempotent and run at startup through
  `crates/crm-core/src/storage/db.rs`.
- Tauri command handlers and future optional integrations should call typed
  `crm-core` services. They should not bypass the core layer with raw SQL.

## Core Entities

### Contacts

Contacts are stored in the legacy `contacts` table. The table still supports
both person contacts and legacy organization contacts through `contact_type`.
Important fields include:

- identity: `id`, `contact_type`;
- person and legacy organization display fields: `first_name`, `last_name`,
  `org_name`;
- communication and address fields: `email`, `phone`, `address`, `city`,
  `country`;
- relationship compatibility fields: legacy `org_id` and normalized
  `organization_id`;
- local metadata: `notes`, `created_at`, `updated_at`, `deleted_at`,
  `device_id`;
- person lifecycle: `lifecycle`, either `lead` or `customer`. Existing rows
  default to `customer`. Organizations keep `customer` and cannot be leads.
  The Leads workspace list is the same `contacts` table filtered to people
  with `lifecycle = lead`. Converted people stay in Contacts as customers;
- optional local owner name: `owner`. This is trimmed free text, not a
  user id. Empty values are stored as NULL. Contact lists can filter by
  owner, including when a search query is also set.

Contacts are soft-deleted by setting `deleted_at`. Active list and search paths
exclude soft-deleted contacts. Contact search uses the `contacts_fts` FTS5
virtual table plus rebuilds when contact records are changed, restored, or
deleted.

### Organizations

First-class organizations live in the `organizations` table. This table was
added after the legacy contact-as-organization model and currently contains:

- `id`, `name`;
- optional `email`, `phone`, `website`;
- address fields: `address_line1`, `address_line2`, `city`, `region`,
  `country`, `postal_code`;
- `source`, `description`;
- optional local owner name: `owner`;
- `created_at`, `updated_at`, `deleted_at`, `device_id`.

Organization creates currently set `source` to `desktop`. Migration v6 bridges
legacy organization contacts into this table without deleting the original
contact rows.

### Deals

Deals are stored in `deals` and represent sales opportunities. Core fields are:

- `id`, `title`;
- `value`, `currency`, `stage`, `probability`, `expected_close`;
- legacy primary contact mirror: `contact_id`;
- normalized organization link: `organization_id`;
- `notes`, optional local owner name `owner`, `created_at`, `updated_at`,
  `deleted_at`, `device_id`.

Deal rows are soft-deleted. The current pipeline model stores the stage name
directly on each deal rather than using a separate pipeline-stage table.

Deal relationship readiness is stored in `deal_contacts`. This join table can
link multiple contacts to one deal with an optional `role` and `is_primary`
flag. Existing `deals.contact_id` values are mirrored into primary
`deal_contacts` rows for compatibility.

### Activities

Activities are stored in `activities` and represent tasks, calls, meetings,
emails, and similar follow-up records. Core fields are:

- `id`, `activity_type`, `title`, `description`;
- `due_date`, `completed`;
- legacy relationship mirrors: `contact_id`, `deal_id`;
- `created_at`, `updated_at`, `deleted_at`, `device_id`.

Activity rows are soft-deleted. First-class relationship readiness is stored in
`activity_links`, which links an activity to a `contact`, `organization`, or
`deal`. Existing `activities.contact_id` and `activities.deal_id` values are
mirrored into activity links for compatibility.

### Notes

Notes are stored in `notes` and use a polymorphic parent reference:
`entity_type` plus `entity_id`. Service and API paths validate note attachment
for contacts, organizations, deals, and activities. The currently visible
reusable notes UI is wired for contacts, organizations, and deals; activity
note support exists at the service/API boundary but is not exposed through a
dedicated visible panel today.

Generic note import/export uses only `entity_type`, `entity_id`, and `content`.
The `entity_id` value is a local active database ID for a contact,
organization, deal, or activity. Import does not accept note IDs, timestamps,
deleted rows, device IDs, tag links, or legacy flat `contacts.notes`/
`deals.notes` values through the generic note format.

The table has both legacy `content` and newer `body` columns. Current storage
writes both columns and reads `body` when present, falling back to `content`.
Notes are soft-deleted with `deleted_at`.

### Entity Links

Links are stored in `entity_links` and attach to a `contact`, `organization`,
or `deal`. Each row has `title`, `kind` (`url` or `path`), and `target`. A
`url` must be `http://` or `https://`. A `path` is a local filesystem path
stored as text. 900CRM does not copy, upload, or encrypt the referenced file.
Moving a local file will break that bookmark. Links are soft-deleted.

### Saved Views

Named list filters live in `saved_views`. Each row has `entity_type`
(`contact`, `organization`, `deal`, `activity`, or `report`), a unique active `name` per type, and
`filters_json`. The Contacts list can save search, person/organization type, lifecycle, and
custom-field filters. The Organizations list can save search and country. The Pipeline
board can save deal-name search, attention (needs follow-up, stale, or overdue),
and custom-field filters. Activities can save type, status, due bucket, and
custom-field filters. Reports can save focus (`pipeline`, `activity`, or
`stale`). Views are soft-deleted.

### Tags

Reusable tags are stored in `tags` with `id`, `name`, `color`, timestamps,
soft-delete metadata, and `device_id`. Tag names are unique. The default color
is `#6366f1`.

The current compatibility model keeps two link stores:

- `entity_tags`, the legacy many-to-many table;
- `tag_links`, the newer link table with IDs, soft-delete metadata, and
  `device_id`.

Apply/remove paths keep both representations aligned where possible. Removing a
tag link physically deletes the legacy `entity_tags` row and soft-deletes the
target `tag_links` row.

Tag definition import/export uses the flat local fields `name` and `color` for
active reusable tags. Tag link import/export uses `entity_type`, `entity_id`,
and `tag_id` only. Those IDs are local active database IDs; the tag link format
does not accept `tag_name` or parent display names because those values are
user-editable and do not provide deterministic portable identity semantics.
Tag link export is filtered to active tags and active contact, organization,
deal, or activity parent rows.

### Custom Fields

Custom field definitions live in `custom_field_defs`; values live in
`custom_field_values`.

Current supported entity types are `contact`, `deal`, `activity`, and
`organization`. Supported field types are `text`, `number`, `date`, `boolean`,
and `select`. Custom field value writes validate that the referenced entity is
an existing active row for the definition's entity type before storing the
value.

Custom field definition import/export uses the flat portable fields
`entity_type`, `field_name`, `field_type`, `field_options`, and `sort_order`.
Imported definition rows create missing definitions through the existing custom
field service. Exact existing definitions are skipped, and definitions with the
same `entity_type` and `field_name` but a different shape are reported instead
of updated silently. Created-definition rollback deletes only the exact created
definition when it still matches the post-import snapshot and has no attached
custom field values.

### Settings

Application settings are stored as string key-value pairs in `settings`.
Seeded keys include language, currency, theme, date format, notification
settings, sync configuration placeholders, optional email settings, and MCP
readiness settings.

Settings values are local data. Email server hostnames, usernames, and
passwords, if entered by the user, are stored in the local SQLite settings table
as plain strings.

### Search And Reports

Contacts, organizations, deals, activities, notes, and tags have FTS5 indexes.
Global search routes each entity type through `storage::search`, using FTS first
and falling back to the legacy text queries if an FTS table is unavailable or
returns no rows. `crm_engine::search` only orchestrates and maps typed storage
records.

Reporting and dashboard paths use aggregate queries and indexes over existing
contacts, organizations, deals, activities, and custom fields. There is no
separate analytics service.

## Audit, Sync, And Accountability

### Audit Log

The `audit_log` table records accountable actions with:

- `actor_type`, constrained to `user`, `desktop_app`, `mcp_client`, `import`,
  or `system`;
- optional `actor_id`;
- action name;
- optional entity type and ID;
- optional JSON before/after snapshots;
- `created_at` and `device_id`.

Current service paths record audit entries for user-visible data changes,
import row creation, proposed-action decisions, external-client permission
changes, explicit external-client read/draft permission evaluations, and draft
permission checks performed before external proposed-action creation.

Audit log entries can be exported locally to CSV or JSON for accountability
review. The export includes every existing `audit_log` row with `id`,
`actor_type`, `actor_id`, `action`, `entity_type`, `entity_id`, `before_json`,
`after_json`, `created_at`, and `device_id`, sorted by `created_at` ascending
and then `id` ascending. Audit log import is intentionally unsupported, and the
export path does not record a new audit row.

The audit log is application-append-only: normal CRM actions insert new rows
and the app does not offer an edit or delete UI. It is not tamper-evident and
is not a compliance WORM store. Anyone with OS-user access to the SQLite file,
or a confirmed restore from a backup, can replace or rewrite history. That is
intentional for a single-user local CRM.

### Sync Changelog

`sync_changelog` is an append-only mutation log with entity type, entity ID,
field name, old value, new value, timestamp, and device ID. It supports the
offline-first design and future sync reconciliation.

The desktop app currently exposes sync status and a lightweight trigger status
path based on local settings. It does not perform real network sync in the
current implementation.

### External Client Readiness

`external_clients` stores optional external client placeholders. Newly created
clients are disabled by default and do not receive tokens or credentials.
Current fields include name, client type, enabled flag, permission mode,
timestamps, soft-delete metadata, and `device_id`.

The current activation update path is local readiness state only. It accepts
only these consistent stored pairs:

- `enabled = false` with `permission_mode = 'disabled'`;
- `enabled = true` with `permission_mode = 'read_only'`;
- `enabled = true` with `permission_mode = 'draft_only'`.

Activation updates refresh `updated_at` and record local audit/sync evidence.
Schema-reserved future modes cannot be written through the activation update
service until a later sprint explicitly supports them.

`external_client_permissions` stores per-client, per-tool permission rows with
`can_read`, `can_write`, and `requires_confirmation`. A unique index enforces
one row per `(client_id, tool_name)`.

Current supported permission modes are:

- `disabled`;
- `read_only`;
- `draft_only`.

Schema-reserved modes such as `write_with_confirmation` and `write_allowed`
exist as stored values but are treated as unsupported by the current evaluation
logic.

Explicit permission evaluation services record local `audit_log` rows named
`evaluate_external_client_read_permission` and
`evaluate_external_client_draft_permission`. The audit entity is
`external_client` with the attempted client ID when available. The JSON context
records `client_id`, `tool_name`, access kind, mode when available, `allowed`,
decision reason, result status, and optional entity scope for call paths that
already have one. These evaluation-only audit entries do not create
`sync_changelog` rows.

### Proposed Actions

`proposed_actions` stores draft external-client actions for review. Important
fields include client ID, action type, tool name, optional target entity, input
JSON, proposed output JSON, status timestamps, and `device_id`.

External proposed-action creation checks draft permission before inserting the
proposed action row. That check records the same draft-evaluation audit context
with actor `mcp_client`, including denied attempts. Denied checks return the
existing permission/not-found error and do not create `proposed_actions` rows.

Reject behavior records only the decision state and audit evidence. Approval of
a supported `create_activity_draft` proposed action creates an activity through
the normal core activity path, sets `approved_at` and `executed_at`, and records
approval/execution audit evidence. Unsupported proposed-action tool/action types
remain pending with an explicit invalid-input error. Approval does not run MCP
code or call a model/provider.

## Backups

Local backups contain:

- `900crm.db`, created with SQLite `VACUUM INTO` for a standalone snapshot;
- `metadata.json`, including backup format version, app version, schema version,
  device ID, creation time, and database filename.

Restore validation checks readable metadata, compatible app and schema versions,
required core tables, and SQLite `PRAGMA quick_check` on a temporary copy of the
backup database. Restore replaces the local app database only after explicit
confirmation.

Backups are local files. They are not encrypted by 900CRM and are not uploaded
by the app.

## Migration Versions

Current migration history:

| Version | Purpose |
|---|---|
| 1 | Initial contacts, deals, activities, notes, tags, custom fields, settings, sync changelog, and contact FTS. |
| 2 | Reporting and performance indexes. |
| 3 | Notification and reminder setting defaults. |
| 4 | Optional email integration setting defaults. |
| 5 | Foundation realignment: first-class organizations, tag links, audit log, external clients, external-client permissions, proposed actions, and MCP readiness settings. |
| 6 | Non-destructive organization normalization bridge from legacy organization contacts. |
| 7 | Deal relationship foundation with `organization_id` and `deal_contacts`. |
| 8 | Activity relationship foundation with `activity_links`. |
| 9 | External-client permission uniqueness cleanup and unique index. |
| 10 | Global search FTS5 parity for organizations, deals, activities, notes, and tags with active-row backfill and maintenance triggers. |
| 11 | Person contact lifecycle (`lead` or `customer`) on `contacts`. |
| 12 | `entity_links` for website and local-file bookmarks on contacts, organizations, and deals. |
| 13 | `saved_views` for named list filters on contacts, organizations, deals, activities, and reports. |
| 14 | Optional local owner name on `contacts`, `deals`, and `organizations`, plus owner indexes. |

## Legacy And Compatibility Caveats

- Organizations are in transition. Legacy organization contacts remain in
  `contacts`, and first-class organizations live in `organizations`.
- `contacts.org_id` and `contacts.organization_id` can both exist. Service code
  preserves legacy links while populating normalized organization references.
- `deals.contact_id` remains the legacy primary-contact mirror even when
  `deal_contacts` is used.
- `activities.contact_id` and `activities.deal_id` remain legacy mirrors even
  when `activity_links` is used.
- Tags are mirrored across `entity_tags` and `tag_links`.
- Notes preserve both `content` and `body` compatibility columns.
- Sync metadata is present, but real multi-device sync is not implemented.
- MCP storage readiness is present, but no MCP server/runtime/listener/token
  behavior is implemented.
- The database is not encrypted by 900CRM. Use operating-system disk encryption
  where local data at rest needs additional protection.
