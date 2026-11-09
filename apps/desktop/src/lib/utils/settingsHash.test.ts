import { describe, expect, it } from 'vitest';

import { parseSettingsHash, settingsPathForPane } from './settingsHash';

describe('settingsHash', () => {
  it('opens the Data pane and import wizard from the first-run import path', () => {
    expect(parseSettingsHash('#/settings/data/import')).toEqual({
      pane: 'data',
      openImport: true,
    });
    expect(settingsPathForPane('data', true)).toBe('/settings/data/import');
  });

  it('opens the Data pane for backup guidance without starting a dialog', () => {
    expect(parseSettingsHash('/settings/data')).toEqual({
      pane: 'data',
      openImport: false,
    });
    expect(settingsPathForPane('data')).toBe('/settings/data');
  });

  it('keeps Appearance as the default Settings hash', () => {
    expect(parseSettingsHash('/settings')).toEqual({
      pane: 'appearance',
      openImport: false,
    });
    expect(settingsPathForPane('appearance')).toBe('/settings');
  });
});
