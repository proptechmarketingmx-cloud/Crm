# Product Roadmap

Status: source-only `0.9.0`. Not a daily sales CRM yet.
Companion to the release/signing checklist in [ROADMAP.md](ROADMAP.md). This file does not replace that checklist.

Workstreams own the work. Names below are lanes, not people.

---

## 1. Position

900CRM is a local-first desktop CRM for shops, clinics, schools, and field sellers who cannot assume internet, a server, or a SaaS seat. The architecture is the right bet: Tauri, local SQLite, no account, no telemetry, Apache-2.0. Version `0.9.0` is source-evaluable. It is not yet a CRM people use every day. Stay offline-first. Do not chase Twenty’s cloud stack or SuiteCRM’s module zoo. Apache-2.0 is a trust advantage. Keep it. Signed downloadable installers, including the Apple Developer ID blocker, stay in [ROADMAP.md](ROADMAP.md) and issue #134. This roadmap is everything else: better daily work, faster lists, a lower machine floor, and an honest communication surface.

---

## 2. Support floor

Hard constraint. If a change needs more than this, it does not ship as the default path.

| Platform | Floor | Notes |
|---|---|---|
| Windows | Windows 10 1803, 4 GB RAM, HDD, WebView2 Evergreen | Do not promise Windows 7. |
| macOS | macOS 11, Intel and ARM | macOS 10.15 only if someone tests it and files evidence. |
| Linux | Runtime: Ubuntu 22.04 and Debian 12. ABI: glibc 2.35, webkit2gtk-4.1, 4 GB RAM, HDD | Link against an Ubuntu 22.04 userspace. Debian 12 is glibc 2.36: supported runtime, not the build image. Current artifacts target Ubuntu 24.04 / glibc 2.39 and fail the floor. Do not raise the runtime floor because hosted `ubuntu-22.04` runners are retiring. |
| Form factor | Native desktop (Tauri) | No Internet Explorer. No PWA as the main app. |

Stay on Tauri. Do not move the shell to Electron. No WebGPU. No heavy chart kits. `prefers-reduced-motion` already exists. Keep it.

---

## 3. Now / Next / Later

### Now (P0)

Must land before anyone calls this 1.0 as a CRM people use. Signing is the external packaging gate. The rest is product and compat.

| Item | Workstream | Why | Done means |
|---|---|---|---|
| Signed downloadable installers | Release | Source-only is not a product for the target user. | Windows and Linux installers exist on a GitHub Release. macOS follows the existing signing track in [ROADMAP.md](ROADMAP.md). This file does not expand that work. |
| Linux build floor | Compat | Current glibc 2.39 / Ubuntu 24.04 artifacts do not run on the machines we claim to serve. | Release Linux binaries are linked against Ubuntu 22.04 / glibc 2.35 / webkit2gtk-4.1, even if the GitHub job runs on a newer hosted runner. A `.deb` and AppImage install and launch on Ubuntu 22.04 and Debian 12. Do not raise the runtime floor when hosted `ubuntu-22.04` goes away. |
| Command palette and keyboard | Keyboard | Daily CRM work is keyboard-first in Pipedrive-class tools. No new backend is required. | `Cmd/Ctrl+K` opens a palette that jumps to routes and recent records. `/` focuses search. `1`–`7` switch the main nav. `?` opens a shortcut sheet. |
| Bounded pipeline load | Pipeline | `list_deals` with no limit, plus `Pipeline.svelte` calling `listDeals()` unbounded, stalls on HDD with thousands of deals. `list_deals_windowed` already exists in core. | Pipeline uses a windowed IPC path. Cards virtualize. Opening the board with a large local dataset stays usable on the support floor. |
| Honest communication UX | Communication | People log calls, visits, WhatsApp, and SMS. The email pane is a TCP probe. Pretending IMAP exists trains distrust. | Activity types cover call, visit, WhatsApp, and SMS as log entries. Timeline is history of what happened. Settings email copy says probe, not mailbox. The email pane looks like a probe. |
| Due times on activities | Communication | Today `due_date` is a date string. Field days run on the clock, not the calendar day. | An activity can store and show a due time. Reminders and Today/Overdue buckets honor that time. Week and month grids show it. |

### Next (P1)

After P0, this is what makes a seller stay.

| Item | Workstream | Why | Done means |
|---|---|---|---|
| Stage history and real conversion | Reports | Reports today compare current-stage counts. That is not conversion. | Stage moves write history. Reports show true stage-to-stage conversion. Labels stop calling snapshot ratios a funnel. |
| Inbox-shaped Activities | Communication | Field days need one home: what is due, what just happened, what is next. | Activities is the day view. Pipeline can sort by next activity. Completing work does not require hunting three screens. |
| Five local automation rules | Automation | Users already get a stage-move follow-up prompt and an attention queue. They need saved rules, not a BPM canvas. | Five local rules ship: stale-deal remind, won next step, import-review duplicates, plus two more of the same size. Each rule drafts. Nothing writes until the user confirms. No visual workflow builder. |
| Offline quotes PDF | Quotes | Small businesses close with a paper or PDF quote. They do not need ERP. | A deal can hold line items, tax, and a printable local PDF. Stop before invoices, stock, or accounting. |
| WhatsApp contact field | Communication | Contacts already have an unused `whatsapp` column. P0 adds the WhatsApp log type; this item only surfaces that field. | Contact records show and save the existing WhatsApp column. Channel polish stays on that field. Logging stays the P0 activity type. No cloud API. No session takeover. |
| Lock and encrypted backup | Privacy | The SQLite file is plaintext on disk. | Optional SQLCipher for encryption at rest, plus an optional OS-keychain lock. A keychain lock is not a substitute for encryption at rest. Optional encrypted backup. Default path stays simple for first-run users. |
| Native-speaker translation | i18n | Ten locales have key parity. Non-English strings are machine-assisted. Look/RTL is layout; this item is native-speaker review. | Native speakers review the shipped locales. |
| USB / folder merge sync | Sync | Changelog foundations exist. `trigger_sync` still reports `not_implemented`. A sync server is the wrong first transport. | Two copies of the DB can merge via a USB drive or shared folder. Conflicts are visible and resolvable. No hosted sync service. |

### Later (P2)

Do these after the product is usable offline on the support floor.

| Item | Workstream | Why | Done means |
|---|---|---|---|
| Recurrence, CalDAV, real IMAP/SMTP | Communication | Due times and honest logging come first. Recurrence and real mail are a different product surface. | Recurring activities exist. Optional CalDAV. Optional user-configured IMAP/SMTP. Still no shared inbox and no mass mail. |
| Multi-user | Sync | One machine, one seller is the 1.0 shape. Owner filter already exists. | PIN or local lock plus the existing owner filter. Shared cloud accounts stay out. |
| Plugins, custom objects, webhooks | Extensibility | Custom fields already exist. Custom objects are a platform, not a CRM. | Plugins and webhooks only if a real user need cannot be met in-app. Custom objects stay later or never. |
| Mobile companion subset | Mobile | The desktop app is the product. A phone companion is a subset, not a rewrite. | Read and log a small set of activities/contacts against a local or merged store. Not a PWA replacement. |
| Signed user-initiated updater | Release | The signed updater channel is owned by [ROADMAP.md](ROADMAP.md). This file does not invent that work. | [ROADMAP.md](ROADMAP.md) owns secrets, the signed channel, and user-initiated Check for updates. Silent auto-update can stay later. |
| MCP writes stay draft-gated | Extensibility | Optional local stdio MCP can draft an activity. Direct writes are a trust break. | Reads stay reviewed. Writes stay pending-action drafts. No network MCP server. |

---

## 4. Look and performance workstream

User ask: better looking, faster, more optimized, usable on older machines. This lane runs in parallel with P0. It does not wait for quotes or CalDAV.

### Look (Visual)

- Finish RTL on primary routes (nav, lists, pipeline, activities, record pages). Look/RTL is layout. P1 i18n is native-speaker review.
- Tighten density, spacing, and type on list pages (Contacts, Organizations, Activities). No new UI kit.
- Replace the provisional icon with official 900 Labs branding (issue #140).
- Keep the teal, system-font language. No heavy UI kit.
- Replace provisional icons in the shell.
- Dark theme contrast pass on cards, badges, and muted text.
- Add a skip-link and a shortcut sheet on `?`.
- Split monolithic route files over time (`Pipeline.svelte`, `Activities`, Settings). Do this while touching those files. Do not open a rewrite sprint for its own sake.

### Fast (Performance)

- Virtualize pipeline, contact, and activity lists.
- Cap IPC payloads. Do not pull every deal and every activity to render a board.
- Put Pipeline on `list_deals_windowed` (or the existing `list_deals` limit/offset path used as a real window). Today `listDeals()` with no page size still asks for the full set.
- Activities is also unbounded today (`listActivities` with no page size). Use the existing `list_activities_windowed` path, not only Pipeline.
- Stay on Tauri. Do not adopt Electron.
- No WebGPU. No heavy chart libraries. Reports stay simple tables and light SVG.
- Keep `prefers-reduced-motion`.

### Older systems

Treat the support-floor table as a test matrix, not a slogan. A P0 that only feels fast on 16 GB SSD is not done.

---

## 5. Explicit non-goals

Do not build these. Do not sneak them in under a nearby ticket.

- Custom objects / entity manager
- Cloud AI, enrichment, or always-on assistants
- VoIP, dialers, shared inbox, mass mail
- Cases, customer portal, knowledge base
- ERP, inventory, invoicing, accounting
- LinkedIn capture
- Realtime multiplayer / collaborative presence
- AGPL relicense
- Twenty-style cloud stack
- SuiteCRM module zoo
- Visual BPM / workflow canvas
- Twilio or Meta WhatsApp product
- Sync server as the first sync
- PWA as the main app
- Windows 7, Ubuntu 20.04, Internet Explorer

AI ceiling, if anything ships: local, user-triggered, review-before-write only. MCP stays on that rule.

---

## 6. Suggested sequence

No staffed calendar. These are maintainer-sized slices. One person can run a slice. Two lanes can overlap when they do not touch the same files.

### Slice 1. Compat and load (P0)

**Workstreams:** Compat, Pipeline, Performance.

1. Link Linux release binaries against Ubuntu 22.04 / glibc 2.35 / webkit2gtk-4.1 (container or sysroot). Keep the GitHub job on a current hosted runner. Do not build on Debian 12. Do not raise the runtime floor to 24.04 when hosted `ubuntu-22.04` is retired.
2. Prove a `.deb` and AppImage install and launch on Ubuntu 22.04 and Debian 12.
3. Stop unbounded pipeline loads. Use `list_deals_windowed` / limit+offset from the board.
4. Virtualize the kanban columns.

**Exit:** a clinic Linux box and a Windows 10 HDD box can open a large pipeline without a stall.

### Slice 2. Hands on the keyboard (P0)

**Workstream:** Keyboard, plus a thin Visual pass.

1. Command palette (`Cmd/Ctrl+K`).
2. `/` focuses search. `1`–`7` switch nav.
3. Shortcut sheet (`?`) and skip-link.

**Exit:** a seller can move between Dashboard, Leads, Contacts, Organizations, Pipeline, Activities, and Reports without the mouse.

### Slice 3. Honest day (P0)

**Workstream:** Communication.

1. Add visit, WhatsApp, and SMS as log types. Keep task, call, meeting.
2. Store due time. Teach reminders and buckets to use it.
3. One-tap “log what just happened” from the record and from Activities.
4. Timeline is history. Email settings stay a TCP probe. The email pane looks like a probe.

**Exit:** a field user can log a 3pm visit and a WhatsApp ping, then see both on the timeline. Nobody is told they have mail.

### Slice 4. Look while P0 closes (parallel)

**Workstream:** Visual, i18n.

1. Official icon (#140).
2. Dark contrast pass. Replace leftover provisional icons.
3. RTL finish on the routes Slice 2 and Slice 3 already touch.
4. Split a route file only when that slice already has it open.

**Exit:** Arabic primary routes are usable. The window looks like a shipped 900 Labs app, still teal and system fonts.

### Slice 5. Call it 1.0 only after this gate

**Workstream:** Release.

1. Signed Windows and Linux installers, per [ROADMAP.md](ROADMAP.md).
2. User-initiated updater secrets stay on that same release track.
3. Do not call the product 1.0 until Slices 1–3 plus this gate are true. The `v1.0.0` tag still follows [ROADMAP.md](ROADMAP.md).

**Exit:** a non-technical user can install from a GitHub Release on the support floor.

### Slice 6. Seller depth (P1)

**Workstreams:** Reports, Communication, Automation.

1. Persist stage history. Replace snapshot “funnel” copy with real conversion.
2. Make Activities the field-day home. Sort pipeline by next activity.
3. Ship five local draft-gated rules (stale deal, won next step, import duplicates, two more of the same size).

**Exit:** Reports tell the truth. The day starts on Activities. Rules nag, they do not write.

### Slice 7. Close the loop, still local (P1)

**Workstreams:** Quotes, Privacy, Sync, i18n.

1. Offline quote PDF: line items, tax, print. Stop.
2. Optional SQLCipher (encryption at rest) and optional keychain lock. A keychain lock is not a substitute for encryption at rest. Encrypted backup.
3. USB / folder merge. No sync server.
4. Native-speaker review of the ten locales.

**Exit:** a user can quote, lock the file, and carry a merge folder on a USB stick.

### Slice 8. Later, only if P0/P1 held

**Workstreams:** Communication, Sync, Extensibility, Mobile, Release.

Recurrence and CalDAV. Real IMAP/SMTP. PIN + owner filter. Signed user-initiated updater if not already in Slice 5. Mobile companion subset. Plugins or webhooks only on a proven need. MCP writes stay draft-gated.

---

## How to use this file

- [ROADMAP.md](ROADMAP.md) is the release and signing checklist. The signed updater channel lives there, not as a product-later invention. Silent auto-update can stay later.
- [PRODUCT_REVIEW_AND_BENCHMARK.md](PRODUCT_REVIEW_AND_BENCHMARK.md) is the gap analysis this plan executes.
- If a proposal is not in Now or Next and is listed under non-goals, reject it.
- If a proposal needs cloud, a server, or a machine above the support floor, reject it.
