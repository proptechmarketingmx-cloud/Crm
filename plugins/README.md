# 900CRM Plugin System

> **Status: Coming in v2.0**
>
> The plugin system is planned for the v2.0 release. The architecture described here is finalised and the API design is in the [discussion phase](https://github.com/900Labs/900CRM/discussions). Implementation has not yet started.
>
> If you are interested in shaping the plugin API before it is built, please open a [discussion](https://github.com/900Labs/900CRM/discussions) tagged `plugin-system`. Community input during the design phase is especially valuable — your use case may determine what gets built first.

---

## Overview

The 900CRM plugin system lets the community extend the application's functionality without modifying or forking the core codebase. Plugins are loaded from the user's plugin directory at startup and can:

- Add new fields or field types to contacts, deals, and activities
- Add new sidebar panels with custom UI
- Add new toolbar actions and menu items
- Integrate with external services (email, calendar, WhatsApp, etc.)
- Add new import/export formats
- Run custom automation rules (e.g., "when a deal moves to Closed Won, create an invoice activity")

Plugins are written in **JavaScript/TypeScript** and communicate with 900CRM through a restricted, documented API. They cannot access the filesystem, database, or network directly — all operations go through the plugin API, which enforces the same security model as the core application.

---

## Architecture

```
900CRM Core (Rust + Svelte)
        │
        ├── Plugin Loader (Rust)
        │     └── Scans plugin directory at startup
        │         Validates manifest.json schema
        │         Registers declared capabilities
        │         Rejects plugins requesting undeclared permissions
        │
        └── Plugin Host (Svelte)
              └── Loads plugin UI bundle in sandboxed iframe
                  Provides 900CRM Plugin API via postMessage bridge
                  Mounts plugin panels, toolbar items, and menu entries
```

### Plugin Loading Sequence

1. At startup, the Rust backend scans the user's plugin directory.
2. For each subdirectory containing a valid `manifest.json`, the plugin is registered.
3. The plugin's declared capabilities are checked against the user's permission settings (users can disable individual plugin permissions in Settings → Plugins).
4. The plugin's main bundle is loaded in a sandboxed iframe context.
5. The plugin registers itself with the Plugin API, and its UI extensions appear in the application.

### Plugin Directory Location

Plugins are stored in the user's application data directory:

| Platform | Path |
|---|---|
| Windows | `%APPDATA%\900CRM\plugins\` |
| macOS | `~/Library/Application Support/900CRM/plugins/` |
| Linux | `~/.local/share/900CRM/plugins/` |

Each plugin lives in its own subdirectory:

```
plugins/
├── whatsapp-integration/
│   ├── manifest.json     ← required
│   ├── index.js          ← main plugin bundle
│   └── icon.svg          ← optional, shown in plugin settings
├── invoice-generator/
│   ├── manifest.json
│   └── index.js
└── ...
```

---

## Plugin Manifest Format

Every plugin must have a `manifest.json` at its root.

### Schema

```json
{
  "$schema": "https://900labs.com/schemas/crm-plugin-manifest/v1.json",

  "id": "com.example.my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "A short description of what this plugin does.",
  "author": "Your Name <you@example.com>",
  "homepage": "https://github.com/example/my-900crm-plugin",
  "license": "Apache-2.0",

  "main": "index.js",
  "icon": "icon.svg",

  "capabilities": [
    "contacts:read",
    "contacts:write",
    "deals:read",
    "activities:write",
    "ui:sidebar-panel",
    "ui:toolbar-action",
    "network:outbound"
  ],

  "min_crm_version": "1.0.0"
}
```

### Capabilities

Plugins must declare every capability they need. The user is shown these declarations when installing a plugin and can revoke individual permissions.

| Capability | Description |
|---|---|
| `contacts:read` | Read contact records |
| `contacts:write` | Create, update, or delete contacts |
| `deals:read` | Read deal and pipeline stage records |
| `deals:write` | Create, update, or delete deals |
| `activities:read` | Read activity records |
| `activities:write` | Create, update, or delete activities |
| `ui:sidebar-panel` | Add a panel to the application sidebar |
| `ui:toolbar-action` | Add a button to the main toolbar |
| `ui:context-menu` | Add items to context menus (right-click menus) |
| `ui:detail-panel` | Add a tab to the contact or deal detail panel |
| `network:outbound` | Make outbound HTTP/HTTPS requests |
| `filesystem:export` | Save files to user-selected locations |

Requesting `network:outbound` will display a prominent warning to the user during installation, as it means the plugin may send data outside the local machine.

---

## Plugin API

The Plugin API is available in the plugin's JavaScript bundle as the global `crm900` object.

### Reading Data

```javascript
// List contacts
const contacts = await crm900.contacts.list({ limit: 50, offset: 0 });

// Get a single contact
const contact = await crm900.contacts.get('contact-uuid');

// List deals
const deals = await crm900.deals.list({ stage: 'closed-won' });

// List activities for a contact
const activities = await crm900.activities.list({ contactId: 'contact-uuid' });
```

### Writing Data

```javascript
// Create a contact
const newContact = await crm900.contacts.create({
  name: 'Fatima Al-Rashid',
  email: 'fatima@example.com',
  phone: '+966501234567',
  tags: ['vip', 'referral']
});

// Create an activity linked to a contact
await crm900.activities.create({
  type: 'call',
  title: 'Follow-up call',
  contactId: newContact.id,
  dueAt: new Date(Date.now() + 86400000).toISOString() // tomorrow
});

// Update a deal
await crm900.deals.update('deal-uuid', {
  value: 75000,
  notes: 'Revised proposal accepted'
});
```

### UI Extensions

```javascript
// Register a sidebar panel
crm900.ui.registerSidebarPanel({
  id: 'my-plugin-panel',
  label: 'WhatsApp',
  icon: 'whatsapp-icon.svg',
  // The panel renders in an iframe pointing to a URL your plugin serves,
  // or as an inline HTML string for simple panels.
  content: { type: 'iframe', url: 'panel.html' }
});

// Register a toolbar action
crm900.ui.registerToolbarAction({
  id: 'generate-invoice',
  label: 'Generate Invoice',
  icon: 'invoice-icon.svg',
  // context: which page(s) the button appears on
  context: ['deal-detail'],
  onClick: async (context) => {
    const deal = context.deal;
    await generateAndSaveInvoice(deal);
  }
});
```

### Events

```javascript
// React to user actions in the core app
crm900.on('deal:stage-changed', (event) => {
  console.log(`Deal ${event.dealId} moved to stage ${event.newStageId}`);
  // e.g., trigger an external notification
});

crm900.on('contact:created', (event) => {
  // e.g., add contact to external mailing list
});
```

---

## Example Plugin Structure

Below is a minimal skeleton for a plugin that adds a sidebar panel and reads contacts.

### Directory Structure

```
my-crm-plugin/
├── manifest.json
├── index.js          ← compiled bundle (or plain JS for simple plugins)
├── panel.html        ← sidebar panel HTML (if using iframe approach)
└── icon.svg
```

### `manifest.json`

```json
{
  "$schema": "https://900labs.com/schemas/crm-plugin-manifest/v1.json",
  "id": "com.example.contact-stats",
  "name": "Contact Stats",
  "version": "1.0.0",
  "description": "Shows a summary of contact activity in the sidebar.",
  "author": "Example Developer",
  "license": "MIT",
  "main": "index.js",
  "icon": "icon.svg",
  "capabilities": [
    "contacts:read",
    "activities:read",
    "ui:sidebar-panel"
  ],
  "min_crm_version": "2.0.0"
}
```

### `index.js`

```javascript
// 900CRM Plugin: Contact Stats
// Adds a sidebar panel showing activity summary for the selected contact.

crm900.ready(async () => {
  // Register sidebar panel
  crm900.ui.registerSidebarPanel({
    id: 'contact-stats-panel',
    label: 'Stats',
    icon: 'icon.svg',
    content: { type: 'inline', render: renderPanel }
  });

  // React to contact selection
  crm900.on('contact:selected', async (event) => {
    const activities = await crm900.activities.list({
      contactId: event.contactId,
      limit: 100
    });

    const stats = {
      total: activities.length,
      completed: activities.filter(a => a.completedAt).length,
      overdue: activities.filter(a => !a.completedAt && a.dueAt && new Date(a.dueAt) < new Date()).length
    };

    crm900.ui.updatePanel('contact-stats-panel', { stats });
  });
});

function renderPanel({ stats }) {
  if (!stats) {
    return '<p>Select a contact to see stats.</p>';
  }
  return `
    <div class="stats-panel">
      <div class="stat">
        <span class="label">Total activities</span>
        <span class="value">${stats.total}</span>
      </div>
      <div class="stat">
        <span class="label">Completed</span>
        <span class="value">${stats.completed}</span>
      </div>
      <div class="stat overdue">
        <span class="label">Overdue</span>
        <span class="value">${stats.overdue}</span>
      </div>
    </div>
  `;
}
```

---

## Plugin Development Workflow (Planned)

When the plugin system is implemented, development will work like this:

1. Create a plugin directory with `manifest.json` and `index.js`
2. Place it in the 900CRM plugins directory (see [Plugin Directory Location](#plugin-directory-location))
3. Launch 900CRM — your plugin will be loaded automatically
4. Edit `index.js`, then reload the plugin from Settings → Plugins → Reload
5. Submit your plugin to the community registry (coming with v2.0)

---

## Get Involved

The plugin system design is open for discussion. If you have ideas for:
- New capabilities the API should support
- A plugin you want to build
- Improvements to the manifest format
- Security concerns with the proposed sandbox model

Please open a [discussion](https://github.com/900Labs/900CRM/discussions) tagged `plugin-system`. The API will be shaped by what the community needs.
