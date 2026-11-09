import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

const srcDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const i18nDir = path.join(srcDir, 'lib', 'i18n');
const settingsSourcePath = path.join(srcDir, 'routes', 'Settings.svelte');

const warningCopy = {
  exportUnencryptedWarning:
    'CSV exports are unencrypted local files. Store files containing sensitive data in a trusted or encrypted location.',
  backupUnencryptedWarning:
    'Local backup folders are unencrypted local files. Store folders containing sensitive data in a trusted or encrypted location.',
};

const importDuplicateAutoMergeCopy = {
  confirmDuplicateWarningsWithMerge:
    '{count} duplicate warnings will be merged into matching existing records where safe.',
  confirmAutoMergeEnabled:
    'Duplicate auto-merge is enabled. Safe contact, deal, or organization matches will merge instead of creating duplicate records.',
  mergeDuplicates: 'Merge duplicate rows into existing records',
  mergeDuplicatesDescription:
    'When enabled, matching contact, deal, or organization rows fill safe blank existing fields without overwriting existing values.',
  duplicateAutoMergeEnabled: 'Duplicate auto-merge is enabled for this import.',
  merged: 'Merged',
};

const importMappingGuidanceCopy = {
  mappingGuidance: {
    contacts:
      'Source, owner, and tag columns are not applied automatically to contacts. Map source-like columns to Notes or create a contact custom field first; import tag definitions and tag links with local IDs when you want tags.',
    tagLinks:
      'Tag links require existing local entity IDs and tag IDs. This import does not match tags by name.',
  },
};

function readLocaleSettings(localeFile: string): Record<string, string> {
  const source = readFileSync(path.join(i18nDir, localeFile), 'utf8');
  return JSON.parse(source).settings;
}

function readLocaleImport(localeFile: string): Record<string, unknown> {
  const source = readFileSync(path.join(i18nDir, localeFile), 'utf8');
  return JSON.parse(source).import;
}

describe('Settings unencrypted file warnings', () => {
  it('keeps export and backup warning copy available in every locale', () => {
    const localeFiles = readdirSync(i18nDir)
      .filter((file) => file.endsWith('.json'))
      .sort();

    expect(localeFiles).toContain('en.json');

    for (const localeFile of localeFiles) {
      const settings = readLocaleSettings(localeFile);

      expect(typeof settings.exportUnencryptedWarning).toBe('string');
      expect(settings.exportUnencryptedWarning.length).toBeGreaterThan(0);
      expect(typeof settings.backupUnencryptedWarning).toBe('string');
      expect(settings.backupUnencryptedWarning.length).toBeGreaterThan(0);
    }
  });

  it('keeps duplicate auto-merge import copy available in every locale', () => {
    const localeFiles = readdirSync(i18nDir)
      .filter((file) => file.endsWith('.json'))
      .sort();

    for (const localeFile of localeFiles) {
      const importCopy = readLocaleImport(localeFile);

      for (const key of Object.keys(importDuplicateAutoMergeCopy)) {
        const value = importCopy[key] as unknown;
        expect(typeof value).toBe('string');
        expect((value as string).length).toBeGreaterThan(0);
      }
    }
  });

  it('keeps import mapping guidance available in every locale', () => {
    const localeFiles = readdirSync(i18nDir)
      .filter((file) => file.endsWith('.json'))
      .sort();

    for (const localeFile of localeFiles) {
      const importCopy = readLocaleImport(localeFile);

      expect(importCopy.mappingGuidance).toBeDefined();
      const guidance = importCopy.mappingGuidance as Record<string, unknown>;
      expect(typeof guidance.contacts).toBe('string');
      expect((guidance.contacts as string).length).toBeGreaterThan(0);
      expect(typeof guidance.tagLinks).toBe('string');
      expect((guidance.tagLinks as string).length).toBeGreaterThan(0);
    }
  });

  it('keeps pane labels and help copy available in every locale', () => {
    const localeFiles = readdirSync(i18nDir)
      .filter((file) => file.endsWith('.json'))
      .sort();

    for (const localeFile of localeFiles) {
      const settings = readLocaleSettings(localeFile);

      expect(typeof settings.sectionAppearance).toBe('string');
      expect(settings.sectionAppearance.length).toBeGreaterThan(0);
      expect(typeof settings.paneAppearanceHelp).toBe('string');
      expect(settings.paneAppearanceHelp.length).toBeGreaterThan(0);
      expect(typeof settings.paneDataHelp).toBe('string');
      expect(settings.paneDataHelp.length).toBeGreaterThan(0);
      expect(typeof settings.paneIntegrationsHelp).toBe('string');
      expect(settings.paneIntegrationsHelp.length).toBeGreaterThan(0);
      expect(typeof settings.dataGuidanceTitle).toBe('string');
      expect(settings.dataGuidanceTitle.length).toBeGreaterThan(0);
      expect(typeof settings.dataGuidance).toBe('string');
      expect(settings.dataGuidance.length).toBeGreaterThan(0);
    }
  });

  it('places warnings before the Settings export and backup actions', () => {
    const source = readFileSync(settingsSourcePath, 'utf8');

    const exportWarningIndex = source.indexOf("t('settings.exportUnencryptedWarning')");
    const exportActionIndex = source.indexOf('onclick={handleExportAll}');
    expect(exportWarningIndex).toBeGreaterThan(-1);
    expect(exportActionIndex).toBeGreaterThan(exportWarningIndex);

    const backupWarningIndex = source.indexOf("t('settings.backupUnencryptedWarning')");
    const backupActionIndex = source.indexOf('onclick={handleCreateBackup}');
    expect(backupWarningIndex).toBeGreaterThan(-1);
    expect(backupActionIndex).toBeGreaterThan(backupWarningIndex);
  });
});
