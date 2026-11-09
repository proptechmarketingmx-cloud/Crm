/**
 * Hash helpers for Settings panes.
 *
 * `#/settings`              → Appearance
 * `#/settings/data`         → Data
 * `#/settings/data/import`  → Data and open the import wizard
 * `#/settings/integrations` → Integrations
 */

export type SettingsHashPane = 'appearance' | 'data' | 'integrations';

export interface ParsedSettingsHash {
  pane: SettingsHashPane;
  openImport: boolean;
}

export function parseSettingsHash(path: string): ParsedSettingsHash {
  const clean = path.replace(/^#/, '').split('?')[0].replace(/\/+$/, '') || '/';

  if (clean === '/settings/data/import') {
    return { pane: 'data', openImport: true };
  }

  if (clean === '/settings/data') {
    return { pane: 'data', openImport: false };
  }

  if (clean === '/settings/integrations') {
    return { pane: 'integrations', openImport: false };
  }

  return { pane: 'appearance', openImport: false };
}

export function settingsPathForPane(pane: SettingsHashPane, openImport = false): string {
  if (openImport) {
    return '/settings/data/import';
  }

  if (pane === 'data') {
    return '/settings/data';
  }

  if (pane === 'integrations') {
    return '/settings/integrations';
  }

  return '/settings';
}
