# Privacy

Date: 2026-06-24
Last updated: 2026-08-13

900CRM is designed as an offline-first desktop CRM. This document describes the
current privacy behavior and caveats. It is not a legal privacy policy for a
hosted service; 900CRM does not currently operate a hosted service for normal
desktop use.

## Offline-First Baseline

Normal 900CRM desktop use does not require an account, cloud service, telemetry
service, model provider, or internet connection.

CRM records are stored locally in SQLite. Reads and writes go through the local
desktop app and `crm-core`; they are not sent to 900 Labs by the app.

## Telemetry And Analytics

900CRM currently has no telemetry, analytics, crash-reporting, product-usage
tracking, or license-verification service.

The app does not phone home during normal CRM use. If a future feature adds any
networked service, it should be documented separately and require explicit user
understanding of what leaves the device.

## Local Data Storage

The local database file is named `900crm.db` and is created in the platform app
data directory. It contains CRM data such as contacts, organizations, deals,
activities, notes, tags, custom fields, settings, audit log entries, sync
changelog entries, proposed actions, and external-client readiness records.

Settings are stored in the same local SQLite database. Optional email settings
store host, port, username, and from-address strings only. Mailbox passwords
are not collected in the current UI and leftover password values are cleared
on settings load. 900CRM does not add application-level encryption for
settings today.

## Backups And Exports

Local backups and exports are user-controlled files.

- Local backups contain a SQLite database snapshot and metadata.
- Automatic pre-import backups are local backup folders created under app data
  before supported desktop CSV, mapped CSV, or JSON imports write rows.
- CSV exports contain plain CSV data for the selected entity type.
- 900CRM does not upload backups or exports.
- 900CRM does not encrypt backups or exports.
- Anyone with filesystem access to those files may be able to read CRM data.

Store backups and exports only in locations appropriate for the sensitivity of
the data. Use operating-system disk encryption or encrypted external storage
when local data at rest needs stronger protection.

The desktop Settings Data Management surface displays this disclosure before
export and backup actions. The warning is informational only; it does not add
encryption or block the local workflows.

## Optional Network Touchpoints

The core CRM workflow has no cloud dependency. A few optional or development
paths can use networking when the user or developer initiates them:

- The development server uses a local dev URL during development builds.
- Optional email connection tests can make DNS and TCP connection attempts to
  user-entered SMTP or IMAP endpoints. Private, loopback, link-local, CGNAT,
  and IPv4-mapped forms of those ranges are rejected.
- Local email compose opens the system mail client with a `mailto:` URL; the
  system mail client is outside 900CRM.
- The updater plugin is compiled in and configured against the public GitHub
  Releases `latest.json` URL. The Settings update control is hidden until a
  signed public release exists. No launch-time update poll runs.
- Sync settings and sync status placeholders exist, but real sync transport is
  not implemented in the current app.

No optional email test performs credential login. The current test checks
reachability and may read a short server banner from plaintext ports.

## MCP And AI Non-Behavior

900CRM currently has no built-in AI agent.

`crates/crm-mcp` is an optional, disabled-by-default local stdio boundary. It
can expose reviewed read tools and `create_activity_draft` as a pending
proposed action when a user starts the `crm-mcp` process with an explicit
config. The desktop app and `crm-core` do not start that process, bind a
localhost listener, expose prompts/resources, manage MCP tokens, or call a
model provider.

External-client records, permissions, proposed actions, and audit entries are
local readiness primitives. Settings can review and edit local activation mode
and per-tool permission rows for external-client records, but local activation
does not create tokens or secrets, start an MCP server/listener, enable sync
server behavior, or run MCP/client code.

Explicit local external-client read/draft permission evaluations and draft
permission checks before external proposed-action creation write `audit_log`
entries with client/tool/access-kind decision context. These audit-only
evaluation entries do not create sync changelog rows. Approving a supported
`create_activity_draft` proposed action can create a local activity through
`crm-core` and record local audit evidence. Rejection remains decision-only,
unsupported proposed-action types remain pending with an explicit error, and
approval does not run MCP/client code.

See [MCP Readiness Baseline](MCP_READINESS.md) for the detailed current MCP
boundary.

## Security Caveats

900CRM does not currently provide:

- application-level database encryption;
- an app lock or per-user app password;
- encrypted backups;
- encrypted CSV exports;
- token or secret storage for MCP;
- active sync credential management.

Local privacy therefore depends heavily on the operating system account,
filesystem permissions, full-disk encryption, physical device security, and the
locations where users save backups and exports.

For vulnerability reporting and supported-version details, see
[SECURITY.md](../SECURITY.md).
