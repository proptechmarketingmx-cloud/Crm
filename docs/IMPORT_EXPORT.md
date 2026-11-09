# Import And Export

Date: 2026-06-25

This document describes the current import/export behavior in 900CRM. It is
based on the desktop UI, Tauri command layer, and `crm-core` services as they
exist today.

## Current Scope

900CRM currently supports local CSV import/export and local JSON import/export
for:

- contacts;
- deals;
- activities;
- organizations;
- generic notes stored in the `notes` table;
- reusable tag definitions;
- custom field definitions for contacts, deals, activities, and organizations;
- local tag links for active contacts, organizations, deals, and activities.

Audit log entries are export-only. They can be exported to local CSV or JSON,
but they are intentionally not importable.

Proposed actions are also export-only. They can be exported to local CSV or
JSON for diagnostics and portability, but they are intentionally not importable,
replayable, approvable, rejectable, executable, or otherwise mutated through
the import/export surface.

External clients are export-only readiness placeholders. They can be exported
to local CSV or JSON for diagnostics and portability, but they are intentionally
not importable, activatable, tokenized, permission-granting, executable, or
connected to MCP, AI, sync, or remote-client behavior through the import/export
surface.

External client permissions are export-only diagnostic rows. They can be
exported to local CSV or JSON for readiness review, but they are intentionally
not importable and do not grant permissions, activate clients, create tokens or
secrets, or connect to MCP, AI, sync, or remote-client behavior through the
import/export surface.

Import and export use local files selected by the user. There is no cloud import
service, no remote export destination, and no automatic upload.

JSON import is intentionally limited to direct local `.json` files containing a
flat array of objects. Source object keys can match supported target fields
directly or be mapped in the import wizard before duplicate preflight.

## Import Entry Point

The import/export modal is available from the desktop UI. The import tab asks
the user to choose an entity type, import format, and local file.

For CSV contacts, deals, activities, organizations, generic notes, tag
definitions, custom field definitions, and tag links, the current wizard flow
is:

1. Select entity type and CSV file.
2. Preview parsed headers and the first rows.
3. Map source CSV columns to supported CRM fields.
4. Run preflight validation. Contacts, deals, and organizations also run
   duplicate checks.
5. Review duplicate warnings when the selected entity supports duplicate
   preflight.
6. Confirm import.
7. Review the import summary.

Mapped imports require a desktop-selected file path so the Rust backend can read
the same CSV file. The browser file-input fallback can preview text, but mapped
backend import/preflight requires the desktop file picker path.

For JSON contacts, deals, activities, organizations, generic notes, tag
definitions, custom field definitions, and tag links, the current flow reuses
the same mapping and confirmation concepts as CSV:

1. Select entity type and JSON format.
2. Select a local `.json` file.
3. Preview the parsed JSON rows in the browser.
4. Map source JSON fields to supported CRM fields.
5. Run preflight validation. Contacts, deals, and organizations also run
   duplicate checks.
6. Review duplicate warnings if any are found.
7. Confirm import.
8. Review the import summary.

## CSV Import Parsing

Frontend preview parsing handles:

- quoted fields;
- escaped double quotes;
- commas and newlines inside quoted fields;
- Windows, Unix, and older Mac line endings;
- empty fields and trailing commas.

Backend CSV parsing uses the Rust `csv` crate with headers enabled, trimmed
fields, and flexible row widths.

Rows with a blank required field are skipped by the parser for flat entity
imports:

- contacts require `first_name`;
- deals require `title`;
- activities require `activity_type` and `title`;
- organizations require `name`;
- tag definitions require `name`.
- custom field definitions require `entity_type`, `field_name`, and
  `field_type`.

Generic note imports require `entity_type`, `entity_id`, and `content`. Tag link
imports require `entity_type`, `entity_id`, and `tag_id`. Missing or blank note
and tag link fields are reported as row-level validation errors rather than
silently imported.

## JSON Import Parsing

Direct JSON import accepts a top-level array of flat objects using the same
fields as the matching JSON export:

- contacts: `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`,
  `city`, `country`, and `notes`;
- deals: `title`, `value`, `currency`, `stage`, `expected_close`, and `notes`;
- activities: `activity_type`, `title`, `description`, `due_date`,
  `completed`, `contact_id`, and `deal_id`;
- organizations: `name`, `email`, `phone`, `website`, `address_line1`,
  `address_line2`, `city`, `region`, `country`, `postal_code`, and
  `description`;
- generic notes: `entity_type`, `entity_id`, and `content`;
- tag definitions: `name` and `color`;
- custom field definitions: `entity_type`, `field_name`, `field_type`,
  `field_options`, and `sort_order`;
- tag links: `entity_type`, `entity_id`, and `tag_id`.

JSON rows are parsed into the same flat row structs used by CSV import and then
sent through the same `crm-core` create/import paths as CSV rows. When source
keys are nonstandard, the JSON mapping step maps object keys to those same flat
target fields before duplicate preflight or import. Rows with a blank mapped
required field are skipped before create attempts for contacts, deals,
activities, organizations, and tag definitions. Custom field definition, note,
and tag link rows with missing required values are reported as row-level
validation errors.

JSON row numbers are reported with the same data-row offset as CSV imports: the
first JSON array item is row 2.

The desktop UI parses JSON files through a read-only preview command before
mapping, duplicate preflight, or import confirmation. The preview shows source
object keys and up to the first five JSON object rows. Preview parsing does not
create automatic backups, create or update CRM rows, or write audit/sync rows.
Invalid JSON, non-object rows, or an unsupported shape blocks the duplicate
preflight/import path and shows a preview error.

## Contact CSV

Supported contact import target fields:

| Field | Required | Notes |
|---|---|---|
| `first_name` | Yes | Blank rows are skipped. |
| `last_name` | No | Optional person field. |
| `org_name` | No | Legacy organization/company display field. |
| `email` | No | Validated by contact creation logic when present. |
| `phone` | No | Imported as text. |
| `address` | No | Legacy single-line address field. |
| `city` | No | Imported as text. |
| `country` | No | Imported as text. |
| `notes` | No | Imported into the legacy contact notes field. |

Mapped contact import can accept arbitrary source headers as long as each source
header is mapped to a supported target field or skipped. The UI suggests common
aliases such as `firstname`, `givenname`, `company`, `emailaddress`, and
`telephone`.

Contact export writes these same fields with a header row. It exports up to
100,000 active contacts sorted by `first_name` ascending through the contact
listing path.

## Organization CSV

Supported organization import target fields:

| Field | Required | Notes |
|---|---|---|
| `name` | Yes | Blank rows are skipped. |
| `email` | No | Validated by organization creation logic when present. |
| `phone` | No | Imported as text. |
| `website` | No | Imported as text. |
| `address_line1` | No | Optional address line. |
| `address_line2` | No | Optional address line. |
| `city` | No | Imported as text. |
| `region` | No | State, province, or region. |
| `country` | No | Imported as text. |
| `postal_code` | No | Imported as text. |
| `description` | No | Freeform organization description. |

Mapped organization import can accept arbitrary source headers as long as each
source header is mapped to a supported target field or skipped. The UI suggests
common aliases such as `company`, `organisation`, `url`, `state`, `province`,
`zipcode`, and `notes`.

Organization export writes these same fields with a header row. It exports
active organizations sorted by name through the organization listing path.

## Deal CSV

Supported deal import fields:

| Field | Required | Notes |
|---|---|---|
| `title` | Yes | Blank rows are skipped. |
| `value` | No | Parsed as a decimal number; invalid or missing values become `0.0`. |
| `currency` | No | Defaults to `USD` if omitted. |
| `stage` | No | Defaults to `Lead` if omitted. |
| `expected_close` | No | Stored as provided by the create path. |
| `notes` | No | Imported as deal notes. |

Mapped deal import can accept arbitrary source headers as long as each source
header is mapped to a supported target field or skipped. The UI suggests common
aliases such as `opportunity`, `amount`, `pipeline stage`, `close date`, and
`memo`.

Deal import does not map imported deals to contacts or organizations. Imported
deals pass `None` for `contact_id` and `organization_id`.

Deal export writes `title`, `value`, `currency`, `stage`, `expected_close`, and
`notes` with a header row. Values are formatted with two decimal places.

## Activity CSV

Supported activity import fields:

| Field | Required | Notes |
|---|---|---|
| `activity_type` | Yes | Blank rows are skipped. Values use the existing activity create validation. |
| `title` | Yes | Blank rows are skipped. |
| `description` | No | Imported as the activity description/body. |
| `due_date` | No | Stored as provided by the create path. |
| `completed` | No | `true`, `1`, `yes`, or `y` marks the created activity complete after creation. `false`, `0`, `no`, `n`, blank, or unrecognized values leave it incomplete. |
| `contact_id` | No | Existing active local contact ID only. No lookup by name or portable relationship resolution is attempted. |
| `deal_id` | No | Existing active local deal ID only. No lookup by name or portable relationship resolution is attempted. |

Mapped activity import can accept arbitrary source headers as long as each
source header is mapped to a supported target field or skipped. The UI suggests
common aliases such as `type`, `subject`, `details`, `due date`, `done`,
`local contact id`, and `local deal id`.

Activity imports create rows through `CrmCore::create_activity`, so normal
activity validation, audit, sync, and contact/deal mirror-link behavior apply.
If `completed` is truthy, the import then marks the created activity complete
through the normal completion service path.

Activity export writes `activity_type`, `title`, `description`, `due_date`,
`completed`, `contact_id`, and `deal_id` with a header row. `contact_id` and
`deal_id` are local database IDs and are useful only when importing into a
database that already has the same active IDs.

## Generic Note CSV

Generic note import/export covers note records stored in the `notes` table. It
does not import or export the legacy flat `contacts.notes` or `deals.notes`
fields except through the existing contact and deal flat row formats above.

Supported generic note import fields:

| Field | Required | Notes |
|---|---|---|
| `entity_type` | Yes | Must be `contact`, `organization`, `deal`, or `activity`. |
| `entity_id` | Yes | Existing active local database ID for the parent row. |
| `content` | Yes | Note body. Blank content is rejected. |

Mapped note import can accept arbitrary source headers as long as each source
header is mapped to `entity_type`, `entity_id`, `content`, or skipped. The UI
suggests aliases such as `parent type`, `parent id`, `body`, `note`, and
`text`.

Note imports validate that the parent row exists and is active before writing.
They then create note records through `CrmCore::create_note`, so normal generic
note validation, audit, and sync changelog behavior applies. No lookup by name,
portable identifier, timestamp, deleted row, or device ID is attempted.

Generic note export writes `entity_type`, `entity_id`, and `content` with a
header row. `entity_id` values are local database IDs and are useful only when
importing into a database that already has the same active parent IDs.

## Tag Definition CSV

Tag definition import/export covers reusable rows stored in `tags`. It does not
import or export tag IDs, timestamps, soft-delete metadata, device IDs, or tag
links.

Supported tag definition import fields:

| Field | Required | Notes |
|---|---|---|
| `name` | Yes | Blank rows are skipped. Existing active tag names are skipped rather than merged. |
| `color` | No | Optional color value. Blank values use the normal tag default color. |

Mapped tag definition import can accept arbitrary source headers as long as each
source header is mapped to `name`, `color`, or skipped. The UI suggests aliases
such as `tag`, `label`, `colour`, and `hex color`.

Tag definition imports create missing active tag names through
`CrmCore::create_tag`, so normal tag validation, audit, sync changelog, and
default-color behavior applies. They do not update existing tag names or colors
because duplicate tag merge rules are risky without a stronger identity model.

Tag definition export writes `name` and `color` with a header row. It exports
active reusable tags through the tag listing path.

## Custom Field Definition CSV

Custom field definition import/export covers reusable rows stored in
`custom_field_defs` for the supported entity types: `contact`, `deal`,
`activity`, and `organization`. It does not import or export definition IDs,
timestamps, custom field values, deleted rows, device IDs, or sync metadata.

Supported custom field definition import fields:

| Field | Required | Notes |
|---|---|---|
| `entity_type` | Yes | Must be `contact`, `deal`, `activity`, or `organization`. |
| `field_name` | Yes | Custom field label/name. Existing exact definitions are skipped. |
| `field_type` | Yes | Must be `text`, `number`, `date`, `boolean`, or `select`. |
| `field_options` | No | JSON string array. Required for `select`; optional but validated when present for other field types. |
| `sort_order` | No | Integer display order. Blank values default to `0`. |

Mapped custom field definition import can accept arbitrary source headers as
long as each source header is mapped to `entity_type`, `field_name`,
`field_type`, `field_options`, `sort_order`, or skipped. The UI suggests
aliases such as `owner type`, `custom field name`, `data type`,
`select options`, and `display order`.

Custom field definition imports create missing definitions through
`CrmCore::create_custom_field_def`, so normal custom field validation, audit,
and sync changelog behavior applies. Duplicate handling is conservative:
existing definitions with the same `entity_type`, `field_name`, `field_type`,
`field_options`, and `sort_order` are skipped as exact duplicates; definitions
with the same `entity_type` and `field_name` but a different shape are reported
as row errors and are not updated.

Custom field definition export writes `entity_type`, `field_name`,
`field_type`, `field_options`, and `sort_order` with a header row. The format is
portable and intentionally does not require imported rows to preserve local
definition IDs.

## Tag Link CSV

Tag link import/export covers active tag attachments to active local parent
rows. The format is intentionally local and conservative: it uses existing local
database IDs only and does not attempt portable tag-name or parent-name
resolution.

Supported tag link import fields:

| Field | Required | Notes |
|---|---|---|
| `entity_type` | Yes | Must be `contact`, `organization`, `deal`, or `activity`. |
| `entity_id` | Yes | Existing active local database ID for the parent row. |
| `tag_id` | Yes | Existing active local reusable tag ID. |

Mapped tag link import can accept arbitrary source headers as long as each
source header is mapped to `entity_type`, `entity_id`, `tag_id`, or skipped.
The UI suggests aliases such as `parent type`, `parent id`, and `local tag id`.

Tag link imports validate that the parent row exists and is active, and that
the referenced tag exists and is active, before writing. They then create new
links through `CrmCore::apply_tag_to_entity`, preserving the normal tag-link
audit, sync changelog, and compatibility mirror behavior. Existing active local
links are skipped. The format intentionally does not accept `tag_name`, because
tag names are user-editable and do not provide deterministic portable identity
semantics.

Tag link export writes `entity_type`, `entity_id`, and `tag_id` with a header
row. Exported links are filtered to active tags and active contact,
organization, deal, or activity parent rows. The IDs are useful only when
importing into the same database or another database that already contains the
same active local parent and tag IDs.

## Contact, Deal, Activity, And Organization Custom Fields

Contact, deal, activity, and organization import/export supports existing active
custom field values using the stable target convention `custom:<field_name>`
when the active field name is unique for that entity type. If two active custom
fields for the same entity type share a name, those duplicate-name targets use
`custom:<field_name>#<field_id>`.
Literal `%` and `#` characters in the field name portion are escaped as `%25`
and `%23` so the duplicate-name suffix stays unambiguous.

CSV and JSON exports include one `custom:` column or object key for each active
custom field definition on the exported entity type. Rows with no value for that
definition export a blank value. Exports use the existing custom field storage
definitions and values; they do not create, rename, or delete custom field
definitions.

CSV and JSON imports can set contact, deal, activity, and organization custom
values in two ways:

- direct imports can use source columns or JSON object keys named
  with the supported `custom:` target;
- mapped imports can map any source column or JSON object key to a supported
  `custom:` target shown in the import wizard.

Blank custom field source values are ignored. Non-blank values are written
through the existing custom field value upsert path after the contact, deal,
activity, or organization row is created or merged. Activity imports do not
currently merge duplicates, so activity custom values are only set on newly
created activity rows.

Custom field names are user-readable and are not currently schema-unique. If two
active custom field definitions for the same entity type share a field name, the
field-id suffix keeps exports complete and lets imports map back to the intended
existing custom field definition. The import wizard labels duplicate-name
options with the field id. For unusual field names that contain `%` or `#`, use
the exported `custom:` key or the import wizard target rather than typing an
unescaped key by hand.

## JSON Import And Export

JSON import and export are available for contacts, deals, activities,
organizations, generic notes, tag definitions, custom field definitions, and
tag links from the same import/export modal as CSV. Import uses an open dialog
for `.json` files. Export uses a save dialog for `.json` files.

Audit log, proposed action, external client, and external client permission
JSON are export-only from the export tab.

JSON exports are pretty-printed arrays of objects. They use the same flat fields
as the matching CSV export:

- contacts: `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`,
  `city`, `country`, and `notes`;
- deals: `title`, `value`, `currency`, `stage`, `expected_close`, and `notes`;
- activities: `activity_type`, `title`, `description`, `due_date`,
  `completed`, `contact_id`, and `deal_id`;
- organizations: `name`, `email`, `phone`, `website`, `address_line1`,
  `address_line2`, `city`, `region`, `country`, `postal_code`, and
  `description`;
- generic notes: `entity_type`, `entity_id`, and `content`;
- tag definitions: `name` and `color`;
- custom field definitions: `entity_type`, `field_name`, `field_type`,
  `field_options`, and `sort_order`;
- tag links: `entity_type`, `entity_id`, and `tag_id`;
- audit log entries: `id`, `actor_type`, `actor_id`, `action`, `entity_type`,
  `entity_id`, `before_json`, `after_json`, `created_at`, and `device_id`;
- proposed actions: `id`, `external_client_id`, `client_id`, `tool_name`,
  `action_type`, `entity_type`, `entity_id`, `payload_json`, `input_json`,
  `proposed_output_json`, `status`, `created_at`, `decided_at`, `approved_at`,
  `rejected_at`, `executed_at`, `error_message`, and `device_id`;
- external clients: `id`, `name`, `client_type`, `permission_mode`, `enabled`,
  `created_at`, `updated_at`, `deleted_at`, and `device_id`;
- external client permissions: `id`, `client_id`, `tool_name`, `can_read`,
  `can_write`, `requires_confirmation`, `created_at`, and `updated_at`.

JSON export uses the same active-row listing boundaries as CSV export:

- contacts export up to 100,000 active contacts sorted by `first_name`
  ascending through the contact listing path;
- deals export active deals through the deal listing path;
- activities export active activities through the activity listing path;
- organizations export active organizations through the organization listing
  path;
- generic notes export active note records for `contact`, `organization`,
  `deal`, and `activity` parents through the generic notes listing path;
- tag definitions export active reusable tags through the tag listing path;
- custom field definitions export definitions through the custom field
  definition listing path;
- tag links export active links for active `contact`, `organization`, `deal`,
  and `activity` parents through the tag link listing path;
- audit log export includes all existing audit rows sorted by `created_at`
  ascending, then audit row `id` ascending;
- proposed action export includes all existing proposed action rows sorted by
  `created_at` ascending, then proposed action `id` ascending;
- external client export includes non-deleted external client placeholders
  sorted by `created_at` ascending, then external client `id` ascending;
- external client permission export includes all existing permission rows
  sorted by `client_id` ascending, `tool_name` ascending, `created_at`
  ascending, then permission `id` ascending.

JSON export for importable CRM records does not include record IDs, timestamps,
deleted rows, device IDs, relationship rows beyond the optional local activity
`contact_id` and `deal_id` mirror columns, tag relationship rows beyond the
local tag link format, audit log entries, proposed actions, external clients,
external client permissions, settings, or backup metadata. Generic note export
intentionally uses only the parent `entity_type`, parent `entity_id`, and note
`content`. Tag definition export intentionally uses only `name` and `color`;
custom field definition export intentionally uses only `entity_type`,
`field_name`, `field_type`, `field_options`, and `sort_order`; tag link export
intentionally uses only `entity_type`, `entity_id`, and `tag_id`. For contacts,
deals, activities, and organizations, JSON export does include active custom
field values using `custom:` keys as described above.

Audit log export is the exception to the portable flat-row rule: it preserves
the full local audit row exactly so the file can be used for local review,
accountability, and troubleshooting. Exporting the audit log does not create a
new audit row.

Proposed action export is another export-only diagnostic exception. It
preserves local proposed-action fields including `id`, `external_client_id`
(the export alias for stored `client_id`), `client_id`, `tool_name`,
`action_type`, `entity_type`, `entity_id`, `payload_json` (the export alias for
stored `input_json`), `input_json`, `proposed_output_json`, `status`,
`created_at`, `decided_at`, `approved_at`, `rejected_at`, `executed_at`,
`error_message`, and `device_id`. The current storage schema has no dedicated
`error_message` column, so that export field is reserved and blank/null.
Exporting proposed actions does not create audit rows or sync changelog rows.

External client export is another export-only readiness/diagnostic exception.
It preserves current local external-client placeholder fields: `id`, `name`,
`client_type`, `permission_mode`, `enabled`, `created_at`, `updated_at`,
`deleted_at`, and `device_id`. Exporting external clients does not import or
activate clients, grant permissions, create tokens or secrets, start MCP/AI
behavior, start sync behavior, create pre-import backups, or write audit rows or
sync changelog rows. External client export does not include permission rows;
those use the separate external client permissions export. `enabled` is runtime
activation state, not export eligibility; disabled non-deleted readiness
placeholders remain exportable for diagnostics.

External client permission export is another export-only diagnostic exception.
It preserves current local permission row fields: `id`, `client_id`,
`tool_name`, `can_read`, `can_write`, `requires_confirmation`, `created_at`,
and `updated_at`. Exporting external client permissions does not import or
grant permissions, activate clients, create tokens or secrets, start MCP/AI
behavior, start sync behavior, create pre-import backups, export external
client placeholder rows, or write audit rows or sync changelog rows.

JSON import has the same entity scope. It does not import record IDs, timestamps,
deleted rows, device IDs, broad relationship rows, tag data beyond reusable tag
definitions and local tag links, custom field data beyond definition rows and
supported flat custom field values, audit log entries, proposed actions,
external clients, external client permissions, settings, or backup metadata.
Activity `contact_id` and `deal_id`, and generic note `entity_id`, are accepted
only as existing active local database IDs. Tag link `entity_id` and `tag_id`
are also accepted only as existing active local database IDs. For contacts,
deals, activities, and organizations, JSON import can set existing custom field
values using `custom:` keys or mapped custom field targets as described above.
Custom field definition import creates definition rows as described above.
Generic note, tag definition, custom field definition, and tag link import do
not accept `custom:` values.

JSON import preview is read-only and browser-visible in the import/export modal.
It shows source object keys before the mapping step. Matching supported JSON
keys are auto-mapped, and nonstandard source keys can be mapped to the
supported flat import fields before duplicate preflight and confirmed import.

## Duplicate Preflight

Duplicate detection is currently implemented for contacts, deals, and
organizations. Activity imports run the same parse/mapping/custom-field
validation preflight shape, but they return zero duplicate warnings because
there is no safe activity duplicate rule in the current service layer. Generic
note imports run parse, mapping, required-field, supported `entity_type`, and
active parent-row validation, but they also return zero duplicate warnings
because there is no established safe note duplicate rule. Tag definition
imports run parse, mapping, and required-field validation but return zero
duplicate warnings because existing active tag names are skipped during
confirmed import rather than merged. Custom field definition imports run parse,
mapping, required-field, supported `entity_type`, supported `field_type`,
`field_options`, `sort_order`, and duplicate-shape validation, but they return
zero duplicate warnings because exact existing definitions are skipped and
conflicting definitions are row errors rather than merge candidates. Tag link
imports run parse, mapping, required-field, supported `entity_type`, active
parent-row, and active tag validation, but they return zero duplicate warnings
because existing active local links are skipped during confirmed import.

Contact preflight checks active contacts for:

- case-insensitive exact email matches;
- exact phone matches after trimming.

Organization preflight checks active organizations for:

- case-insensitive exact name matches;
- case-insensitive exact email matches;
- exact phone matches after trimming.

Deal preflight checks active deals for:

- case-insensitive exact title matches after trimming the imported title.

Custom field values do not participate in duplicate detection. Preflight parses
and validates supported contact/deal/activity/organization custom field targets,
but duplicate warnings are based only on the flat fields listed above. Generic
notes, tag definitions, custom field definitions, and tag links do not support
custom field targets.

Preflight returns warnings with the source row number, match type, source value,
existing record ID, existing display label, and a human-readable reason. CSV
imports use CSV data row numbers. JSON imports use the same offset as JSON
import errors: the first array item is row 2.

Duplicate preflight is read-only. It does not create contacts, deals,
activities, organizations, notes, tag definitions, custom field definitions, or
tag links, and it does not write audit rows or sync changelog rows.

Preflight warnings do not block import automatically. The UI lets the user
continue despite warnings. The import then attempts to create each row and
reports created, skipped, and error counts.

## Duplicate Auto-Merge

Contact, deal, and organization imports include an explicit opt-in checkbox to
merge duplicate import rows into matching existing records. The option is
available for CSV, mapped CSV, JSON, and mapped JSON imports. It is off by
default.

Activity, note, tag definition, custom field definition, and tag link imports
do not expose duplicate auto-merge. Confirmed activity rows always attempt to
create new activities. Confirmed generic note rows always attempt to create new
note records after validating their active parent row. Confirmed tag definition
rows create missing active tag names and skip existing active names. Confirmed
custom field definition rows create missing definitions, skip exact existing
definitions, and report conflicting existing definitions as row errors.
Confirmed tag link rows create missing active local links and skip existing
active links.

Duplicate preflight still runs before confirmation when auto-merge is enabled.
The confirmation copy states that duplicate warnings will be merged into
matching existing records where safe.

When enabled, duplicate matching uses the same active-record fields as
preflight:

- contacts match by case-insensitive email or trimmed phone;
- deals match active deals by exact title after trimming, case-insensitively;
- organizations match by case-insensitive name, case-insensitive email, or
  trimmed phone.

If all matching rules identify one existing active record, the import row fills
blank fields on that record without overwriting existing non-empty values.
Existing IDs and active rows are preserved, and updates route through the
existing `crm-core` update services so sync and audit behavior stays aligned
with normal edits. Non-duplicate rows continue to create normally.

For contacts, deals, and organizations, duplicate auto-merge also fills missing
or blank custom field values from supported `custom:` import targets. It does
not overwrite existing non-empty custom field values.

Deal auto-merge is intentionally conservative. Existing deal titles are never
overwritten. Imported `expected_close` and `notes` values fill only blank
existing fields. Imported `value` fills only when the existing value is `0.0`
and the imported value parses to a nonzero number. This value rule exists
because the flat CSV/JSON row shape stores deal value as text and cannot
distinguish a blank value from an explicit zero once it reaches the import row.
Imported `currency` and `stage` do not overwrite existing deal values, including
the default `USD` and `Lead` values.

If one import row matches multiple existing records, that row is skipped with a
row-numbered error rather than merged unsafely.

## Row-Level Results

Imports return:

- `created`: number of rows successfully created;
- `merged`: number of duplicate rows folded into existing records;
- `skipped`: number of rows that failed during creation or duplicate
  auto-merge handling after parsing;
- `errors`: row-numbered error strings from failed create attempts or skipped
  ambiguous duplicate auto-merge rows.

Rows skipped during CSV or JSON parsing because the required field is blank are
not included in the import result as row-level errors.

Successful imported rows are written through normal `crm-core` create paths.
Those paths record sync changelog entries and audit evidence. The import
service also records an `import_row` audit entry for each successfully imported
contact, deal, activity, organization, generic note, tag definition, or tag
custom field definition, or tag link. Successful duplicate auto-merge rows are
updated through normal update services and also receive an `import_row_merge`
audit entry.

Desktop CSV, mapped CSV, JSON, and mapped JSON imports for contacts, deals,
activities, organizations, generic notes, tag definitions, custom field
definitions, and tag links first create an automatic local backup through
`CrmCore::create_local_backup`. The backup is created immediately before import
rows are written, and backup failure stops the import. The import summary shown
in the UI includes the created backup path.

When the import summary includes an automatic pre-import backup path, the UI can
restore that backup directly from the summary. The restore action validates the
backup folder first, asks for explicit destructive confirmation, then calls the
same local restore path used by Settings. If validation or restore fails, the
summary remains visible with the failure message.

## Row-Level Import Rollback

Completed desktop imports for contacts, deals, activities, organizations,
generic notes, tag definitions, custom field definitions, and tag links return a
self-contained row-level rollback plan when the import creates rows or changes
fields through duplicate auto-merge. The current Import/Export summary can use
that plan to request rollback of the just-completed import.

Rollback is available for CSV, mapped CSV, JSON, and mapped JSON imports. It is
limited to the same contact, deal, activity, organization, generic note, tag
definition, custom field definition, and tag link rows supported by
import/export:

- created rows are soft-deleted through the existing `crm-core` delete service
  for the matching entity; contact/deal/activity/organization custom field
  values created by that imported row are deleted after the row passes its
  rollback conflict check;
- created generic notes are soft-deleted through the existing `delete_note`
  service after the active note still matches the post-import snapshot;
- created tag definitions are soft-deleted through the existing `delete_tag`
  service only when the active tag still matches the post-import snapshot and no
  active links reference it;
- created custom field definitions are deleted through the existing
  `delete_custom_field_def` service only when the active definition still
  matches the post-import snapshot and no custom field values reference it;
- created tag links are removed only when the exact target `tag_links` row
  created by the import is still active;
- duplicate auto-merge rows restore only the fields that the import merge
  changed, through the existing `crm-core` update service for the matching
  entity; contact/deal/organization custom field values changed by duplicate
  auto-merge are restored to their before-import value or removed if they did
  not exist before import.

Activity rollback covers created activity rows only because activity imports do
not implement duplicate auto-merge. It uses the existing `delete_activity`
service behavior after confirming that the active activity and its activity
custom values still match the post-import snapshot.

Generic note rollback covers created generic note records only because note
imports do not implement duplicate auto-merge. It does not roll back legacy flat
`contacts.notes` or `deals.notes` fields beyond the existing contact/deal row
rollback behavior.

Tag definition, custom field definition, and tag link rollback cover created
rows only because those imports do not implement duplicate auto-merge. A created
tag definition is not deleted if it was edited after import or if any active
link now references it. A created custom field definition is not deleted if it
was edited after import or if any custom field value now references it. A
created tag link is not removed if the exact imported target link row is already
gone, including when a user removed the link and later reapplied the same tag to
the same parent.

Rollback is conflict-safe. Before applying each row action, `crm-core` compares
the current active row to the post-import state recorded in the rollback plan.
If the row was edited, deleted, or otherwise no longer matches the expected
post-import state, that row is skipped and reported as a row-level error. For
contacts, deals, activities, and organizations, the post-import comparison
includes custom field values. For generic notes, the comparison includes the
parent `entity_type`, parent `entity_id`, note `content`, and `updated_at`.
For tag definitions, the comparison includes `name`, `color`, and `updated_at`.
For custom field definitions, the comparison includes `entity_type`,
`field_name`, `field_type`, `field_options`, `sort_order`, and `created_at`.
For tag links, the comparison requires the active target `tag_links` row id,
`created_at`, `entity_type`, `entity_id`, and `tag_id` to still match the
rollback plan. Later rollback rows continue to run.

The rollback result reports:

- `rolled_back`: number of row actions applied;
- `skipped`: number of row actions skipped, including conflicts and rows that
  were already deleted;
- `errors`: row-level error objects with entity type, entity ID, row number,
  code, and message.

The rollback plan is intentionally summary-scoped. It is not persisted across an
app restart and is not available after the current import summary is dismissed.
The UI disables the row-level rollback button after a successful rollback
command response. Reusing the same plan is still safe: already-rolled-back
created rows are reported as skipped, and already-restored merge rows no longer
match the post-import expected state.

Row-level rollback is not full database restore. It does not roll back
relationships beyond the supported imported row behavior, legacy flat note
fields outside contact/deal row rollback, tag relationships beyond created
local tag links, audit logs, sync changelog rows, proposed actions, external
clients, permissions, settings, backup metadata, or schema changes.

## Export Behavior

Exports write local CSV or JSON files selected by the user through the save
dialog.

Exported files are plain text. They may contain personal, customer, sales, and
business data. 900CRM does not encrypt exported files, upload them, or apply
access controls after they are written.

Audit log export writes accountability data, including local audit IDs,
timestamps, optional actor/entity IDs, JSON before/after snapshots, and
`device_id`. It is read-only and export-only; there is no audit log import,
preview, preflight, rollback, mapping flow, or pre-import backup.

Proposed action export writes local diagnostic state, including proposed action
IDs, optional external-client IDs, requested tool/action names, payload JSON,
status, timestamps, and device ID. It is read-only and export-only; there is no
proposed action import, preview, preflight, backup, rollback, replay, approval,
rejection, execution, MCP/client behavior, or AI behavior in the import/export
surface.

External client export writes local readiness placeholder state, including
external client IDs, names, client types, permission modes, enabled flags,
timestamps, and device ID. It is read-only and export-only; there is no external
client import, preview, preflight, backup, rollback, activation, token/secret
creation, permission grant, MCP/client behavior, AI behavior, or sync behavior
in the import/export surface.

External client permission export writes local permission diagnostic state,
including permission IDs, external client IDs, tool names, read/write flags,
confirmation requirement flags, and timestamps. It is read-only and
export-only; there is no permission import, preview, preflight, backup,
rollback, permission grant UI, activation, token/secret creation, MCP/client
behavior, AI behavior, sync-server behavior, audit-log mutation, or sync
changelog mutation in the import/export surface.

The Settings Data Management export action displays this disclosure before the
user opens the export flow. Store exported files containing sensitive data in a
trusted or encrypted location.

## Relationship To Backups

CSV import/export and JSON import/export are not a backup system.

- CSV export is useful for portability, spreadsheet use, and migration.
- JSON import/export is useful for portability and inspection of the supported
  flat entity fields.
- Audit log export is useful for local accountability review and should not be
  treated as an importable backup or migration format.
- Proposed action export is useful for local diagnostics and should not be
  treated as an importable backup, replay log, approval queue, or execution
  format.
- External client export is useful for local readiness diagnostics and should
  not be treated as an importable backup, activation manifest, permission grant,
  token source, MCP runtime configuration, or sync setup.
- External client permission export is useful for local readiness diagnostics
  and should not be treated as an importable backup, permission grant manifest,
  activation manifest, token source, MCP runtime configuration, AI behavior
  input, or sync setup.
- Local backup creates a SQLite snapshot plus metadata and is the safer way to
  preserve full application state before a destructive restore or risky import.
- Supported desktop imports automatically create a local pre-import backup
  before writing rows.
- Automatic pre-import backups are stored under the platform app data directory
  at `pre-import-backups/<timestamp-and-sequence>/`.
- Current import summaries can run row-level rollback for created rows and
  duplicate auto-merge field changes from the just-completed import. Generic
  note rollback covers created note records only; tag rollback covers created
  tag definitions and created local tag links only; custom field definition
  rollback covers created definitions only.
- Import summaries can validate and restore their automatic pre-import backup
  after explicit destructive confirmation.
- Duplicate auto-merge writes use the same automatic pre-import backup guard as
  normal import creates.
- Restore validation applies to local backups, not CSV or JSON exports.

Before large imports, confirm the automatic backup path in the import summary
and move or copy that backup to durable encrypted storage if needed. Row-level
rollback is useful for undoing flat rows from the current summary, while
restoring the pre-import backup replaces the current local database and remains
the full-state escape hatch. See [Backup and Restore](BACKUP_RESTORE.md) for
the validated backup workflow.

## Not Yet Implemented

The following are not implemented in the current import/export surface:

- Remote import/export endpoints.
- Cloud storage export destinations.
- Scheduled export.
- MCP/AI-driven import behavior.
- Sync-server upload or download as part of import/export.
- Relationship import/export beyond optional activity `contact_id` and
  `deal_id` local ID columns, generic note parent `entity_type`/`entity_id`,
  and local tag link `entity_type`/`entity_id`/`tag_id`.
- Audit log import.
- Proposed action import, replay, approval, rejection, or execution through the
  import/export surface.
- External client import, activation, permission grants, tokens/secrets,
  MCP/client behavior, AI behavior, or sync behavior through the import/export
  surface.
- External client permission import, permission-grant UI, activation,
  tokens/secrets, MCP/client behavior, AI behavior, or sync behavior through the
  import/export surface.
- Settings, sync changelog, or backup metadata import/export.
