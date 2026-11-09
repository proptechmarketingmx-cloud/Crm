import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  exportActivitiesCsv,
  exportActivitiesJson,
  exportAuditLogCsv,
  exportAuditLogJson,
  exportContactsCsv,
  exportContactsJson,
  exportCsv,
  exportCustomFieldDefinitionsCsv,
  exportCustomFieldDefinitionsJson,
  exportData,
  exportDealsCsv,
  exportDealsJson,
  exportExternalClientsCsv,
  exportExternalClientsJson,
  exportExternalClientPermissionsCsv,
  exportExternalClientPermissionsJson,
  exportJson,
  exportNotesCsv,
  exportNotesJson,
  exportOrganizationsCsv,
  exportOrganizationsJson,
  exportProposedActionsCsv,
  exportProposedActionsJson,
  exportTagDefinitionsCsv,
  exportTagDefinitionsJson,
  exportTagLinksCsv,
  exportTagLinksJson,
  importContactsCsv,
  importContactsCsvWithMapping,
  importContactsJson,
  importContactsJsonWithMapping,
  importCsv,
  importCsvWithMapping,
  importCustomFieldDefinitionsCsv,
  importCustomFieldDefinitionsCsvWithMapping,
  importCustomFieldDefinitionsJson,
  importCustomFieldDefinitionsJsonWithMapping,
  importData,
  importActivitiesCsv,
  importActivitiesCsvWithMapping,
  importActivitiesJson,
  importActivitiesJsonWithMapping,
  importDealsCsv,
  importDealsCsvWithMapping,
  importDealsJson,
  importDealsJsonWithMapping,
  importJsonWithMapping,
  importJson,
  importNotesCsv,
  importNotesCsvWithMapping,
  importNotesJson,
  importNotesJsonWithMapping,
  importOrganizationsCsv,
  importOrganizationsCsvWithMapping,
  importOrganizationsJson,
  importOrganizationsJsonWithMapping,
  importTagDefinitionsCsv,
  importTagDefinitionsCsvWithMapping,
  importTagDefinitionsJson,
  importTagDefinitionsJsonWithMapping,
  importTagLinksCsv,
  importTagLinksCsvWithMapping,
  importTagLinksJson,
  importTagLinksJsonWithMapping,
  preflightActivitiesCsvImport,
  preflightActivitiesCsvImportWithMapping,
  preflightActivitiesJsonImport,
  preflightActivitiesJsonImportWithMapping,
  preflightContactsCsvImport,
  preflightContactsCsvImportWithMapping,
  preflightContactsJsonImport,
  preflightContactsJsonImportWithMapping,
  preflightCsv,
  preflightCsvWithMapping,
  preflightCustomFieldDefinitionsCsvImport,
  preflightCustomFieldDefinitionsCsvImportWithMapping,
  preflightCustomFieldDefinitionsJsonImport,
  preflightCustomFieldDefinitionsJsonImportWithMapping,
  preflightDealsCsvImport,
  preflightDealsCsvImportWithMapping,
  preflightDealsJsonImport,
  preflightDealsJsonImportWithMapping,
  preflightJson,
  preflightJsonWithMapping,
  preflightNotesCsvImport,
  preflightNotesCsvImportWithMapping,
  preflightNotesJsonImport,
  preflightNotesJsonImportWithMapping,
  preflightOrganizationsCsvImport,
  preflightOrganizationsCsvImportWithMapping,
  preflightOrganizationsJsonImport,
  preflightOrganizationsJsonImportWithMapping,
  preflightTagDefinitionsCsvImport,
  preflightTagDefinitionsCsvImportWithMapping,
  preflightTagDefinitionsJsonImport,
  preflightTagDefinitionsJsonImportWithMapping,
  preflightTagLinksCsvImport,
  preflightTagLinksCsvImportWithMapping,
  preflightTagLinksJsonImport,
  preflightTagLinksJsonImportWithMapping,
  previewActivitiesJsonImport,
  previewContactsJsonImport,
  previewCustomFieldDefinitionsJsonImport,
  previewDealsJsonImport,
  previewJson,
  previewNotesJsonImport,
  previewOrganizationsJsonImport,
  previewTagDefinitionsJsonImport,
  previewTagLinksJsonImport,
  rollbackCompletedImport,
} from './importExport';

const sampleBackup = {
  backup_dir: '/tmp/app-data/pre-import-backups/backup-1',
  database_path: '/tmp/app-data/pre-import-backups/backup-1/900crm.db',
  metadata_path: '/tmp/app-data/pre-import-backups/backup-1/metadata.json',
  metadata: {
    backup_format_version: 1,
    created_at: '2026-06-25T00:00:00Z',
    app_version: '0.1.0',
    schema_version: 1,
    device_id: 'device-1',
    database_file: '900crm.db',
  },
};

function importWithBackup(created: number, skipped = 0, errors: string[] = []) {
  return {
    import: { created, skipped, errors },
    backup: sampleBackup,
  };
}

describe('import/export API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps contact CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(2, 1, ['Row 4']));
    await expect(importContactsCsv('/tmp/contacts.csv')).resolves.toEqual(
      importWithBackup(2, 1, ['Row 4']),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv', {
      file_path: '/tmp/contacts.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(importContactsJson('/tmp/contacts.json')).resolves.toEqual(importWithBackup(2));
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json', {
      file_path: '/tmp/contacts.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportContactsCsv('/tmp/contacts-export.csv')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_contacts_csv', {
      file_path: '/tmp/contacts-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportContactsJson('/tmp/contacts-export.json')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_contacts_json', {
      file_path: '/tmp/contacts-export.json',
    });
  });

  it('maps deal CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(3));
    await expect(importDealsCsv('/tmp/deals.csv')).resolves.toEqual(importWithBackup(3));
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv', {
      file_path: '/tmp/deals.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(3));
    await expect(importDealsJson('/tmp/deals.json')).resolves.toEqual(importWithBackup(3));
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce(3);
    await expect(exportDealsCsv('/tmp/deals-export.csv')).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_deals_csv', {
      file_path: '/tmp/deals-export.csv',
    });

    invokeMock.mockResolvedValueOnce(3);
    await expect(exportDealsJson('/tmp/deals-export.json')).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_deals_json', {
      file_path: '/tmp/deals-export.json',
    });
  });

  it('maps activity CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(importActivitiesCsv('/tmp/activities.csv')).resolves.toEqual(importWithBackup(2));
    expect(invokeMock).toHaveBeenLastCalledWith('import_activities_csv', {
      file_path: '/tmp/activities.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(importActivitiesJson('/tmp/activities.json')).resolves.toEqual(importWithBackup(2));
    expect(invokeMock).toHaveBeenLastCalledWith('import_activities_json', {
      file_path: '/tmp/activities.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportActivitiesCsv('/tmp/activities-export.csv')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_activities_csv', {
      file_path: '/tmp/activities-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportActivitiesJson('/tmp/activities-export.json')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_activities_json', {
      file_path: '/tmp/activities-export.json',
    });
  });

  it('maps organization CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importOrganizationsCsv('/tmp/organizations.csv')).resolves.toEqual(
      importWithBackup(1),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv', {
      file_path: '/tmp/organizations.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importOrganizationsJson('/tmp/organizations.json')).resolves.toEqual(
      importWithBackup(1),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_json', {
      file_path: '/tmp/organizations.json',
    });

    invokeMock.mockResolvedValueOnce(1);
    await expect(exportOrganizationsCsv('/tmp/organizations-export.csv')).resolves.toBe(1);
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_csv', {
      file_path: '/tmp/organizations-export.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await expect(exportOrganizationsJson('/tmp/organizations-export.json')).resolves.toBe(1);
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_json', {
      file_path: '/tmp/organizations-export.json',
    });
  });

  it('maps note CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(importNotesCsv('/tmp/notes.csv')).resolves.toEqual(importWithBackup(2));
    expect(invokeMock).toHaveBeenLastCalledWith('import_notes_csv', {
      file_path: '/tmp/notes.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(importNotesJson('/tmp/notes.json')).resolves.toEqual(importWithBackup(2));
    expect(invokeMock).toHaveBeenLastCalledWith('import_notes_json', {
      file_path: '/tmp/notes.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportNotesCsv('/tmp/notes-export.csv')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_notes_csv', {
      file_path: '/tmp/notes-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportNotesJson('/tmp/notes-export.json')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_notes_json', {
      file_path: '/tmp/notes-export.json',
    });
  });

  it('maps tag definition CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(importTagDefinitionsCsv('/tmp/tag-definitions.csv')).resolves.toEqual(
      importWithBackup(2),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_definitions_csv', {
      file_path: '/tmp/tag-definitions.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(importTagDefinitionsJson('/tmp/tag-definitions.json')).resolves.toEqual(
      importWithBackup(2),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_definitions_json', {
      file_path: '/tmp/tag-definitions.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportTagDefinitionsCsv('/tmp/tag-definitions-export.csv')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_tag_definitions_csv', {
      file_path: '/tmp/tag-definitions-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportTagDefinitionsJson('/tmp/tag-definitions-export.json')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_tag_definitions_json', {
      file_path: '/tmp/tag-definitions-export.json',
    });
  });

  it('maps custom field definition CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(
      importCustomFieldDefinitionsCsv('/tmp/custom-field-definitions.csv'),
    ).resolves.toEqual(importWithBackup(2));
    expect(invokeMock).toHaveBeenLastCalledWith('import_custom_field_definitions_csv', {
      file_path: '/tmp/custom-field-definitions.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await expect(
      importCustomFieldDefinitionsJson('/tmp/custom-field-definitions.json'),
    ).resolves.toEqual(importWithBackup(2));
    expect(invokeMock).toHaveBeenLastCalledWith('import_custom_field_definitions_json', {
      file_path: '/tmp/custom-field-definitions.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(
      exportCustomFieldDefinitionsCsv('/tmp/custom-field-definitions-export.csv'),
    ).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_custom_field_definitions_csv', {
      file_path: '/tmp/custom-field-definitions-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(
      exportCustomFieldDefinitionsJson('/tmp/custom-field-definitions-export.json'),
    ).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_custom_field_definitions_json', {
      file_path: '/tmp/custom-field-definitions-export.json',
    });
  });

  it('maps tag link CSV import/export commands', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importTagLinksCsv('/tmp/tag-links.csv')).resolves.toEqual(importWithBackup(1));
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_links_csv', {
      file_path: '/tmp/tag-links.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importTagLinksJson('/tmp/tag-links.json')).resolves.toEqual(importWithBackup(1));
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_links_json', {
      file_path: '/tmp/tag-links.json',
    });

    invokeMock.mockResolvedValueOnce(1);
    await expect(exportTagLinksCsv('/tmp/tag-links-export.csv')).resolves.toBe(1);
    expect(invokeMock).toHaveBeenLastCalledWith('export_tag_links_csv', {
      file_path: '/tmp/tag-links-export.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await expect(exportTagLinksJson('/tmp/tag-links-export.json')).resolves.toBe(1);
    expect(invokeMock).toHaveBeenLastCalledWith('export_tag_links_json', {
      file_path: '/tmp/tag-links-export.json',
    });
  });

  it('maps audit log export-only commands', async () => {
    invokeMock.mockResolvedValueOnce(4);
    await expect(exportAuditLogCsv('/tmp/audit-log-export.csv')).resolves.toBe(4);
    expect(invokeMock).toHaveBeenLastCalledWith('export_audit_log_csv', {
      file_path: '/tmp/audit-log-export.csv',
    });

    invokeMock.mockResolvedValueOnce(4);
    await expect(exportAuditLogJson('/tmp/audit-log-export.json')).resolves.toBe(4);
    expect(invokeMock).toHaveBeenLastCalledWith('export_audit_log_json', {
      file_path: '/tmp/audit-log-export.json',
    });
  });

  it('maps proposed actions export-only commands', async () => {
    invokeMock.mockResolvedValueOnce(3);
    await expect(exportProposedActionsCsv('/tmp/proposed-actions-export.csv')).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_proposed_actions_csv', {
      file_path: '/tmp/proposed-actions-export.csv',
    });

    invokeMock.mockResolvedValueOnce(3);
    await expect(exportProposedActionsJson('/tmp/proposed-actions-export.json')).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_proposed_actions_json', {
      file_path: '/tmp/proposed-actions-export.json',
    });
  });

  it('maps external clients export-only commands', async () => {
    invokeMock.mockResolvedValueOnce(2);
    await expect(exportExternalClientsCsv('/tmp/external-clients-export.csv')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_external_clients_csv', {
      file_path: '/tmp/external-clients-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await expect(exportExternalClientsJson('/tmp/external-clients-export.json')).resolves.toBe(2);
    expect(invokeMock).toHaveBeenLastCalledWith('export_external_clients_json', {
      file_path: '/tmp/external-clients-export.json',
    });
  });

  it('maps external client permissions export-only commands', async () => {
    invokeMock.mockResolvedValueOnce(3);
    await expect(
      exportExternalClientPermissionsCsv('/tmp/external-client-permissions-export.csv'),
    ).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_external_client_permissions_csv', {
      file_path: '/tmp/external-client-permissions-export.csv',
    });

    invokeMock.mockResolvedValueOnce(3);
    await expect(
      exportExternalClientPermissionsJson('/tmp/external-client-permissions-export.json'),
    ).resolves.toBe(3);
    expect(invokeMock).toHaveBeenLastCalledWith('export_external_client_permissions_json', {
      file_path: '/tmp/external-client-permissions-export.json',
    });
  });

  it('routes generic CSV helpers by entity', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsv('organizations', '/tmp/orgs.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv', {
      file_path: '/tmp/orgs.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await exportCsv('organizations', '/tmp/orgs-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_csv', {
      file_path: '/tmp/orgs-export.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await importJson('contacts', '/tmp/contacts.json');
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json', {
      file_path: '/tmp/contacts.json',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsv('activities', '/tmp/activities.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('import_activities_csv', {
      file_path: '/tmp/activities.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await exportCsv('activities', '/tmp/activities-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_activities_csv', {
      file_path: '/tmp/activities-export.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await importCsv('notes', '/tmp/notes.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('import_notes_csv', {
      file_path: '/tmp/notes.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await exportCsv('notes', '/tmp/notes-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_notes_csv', {
      file_path: '/tmp/notes-export.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsv('tag_definitions', '/tmp/tag-definitions.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_definitions_csv', {
      file_path: '/tmp/tag-definitions.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await exportCsv('tag_links', '/tmp/tag-links-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_tag_links_csv', {
      file_path: '/tmp/tag-links-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await exportCsv('audit_log', '/tmp/audit-log-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_audit_log_csv', {
      file_path: '/tmp/audit-log-export.csv',
    });

    invokeMock.mockResolvedValueOnce(3);
    await exportCsv('proposed_actions', '/tmp/proposed-actions-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_proposed_actions_csv', {
      file_path: '/tmp/proposed-actions-export.csv',
    });

    invokeMock.mockResolvedValueOnce(2);
    await exportCsv('external_clients', '/tmp/external-clients-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_external_clients_csv', {
      file_path: '/tmp/external-clients-export.csv',
    });

    invokeMock.mockResolvedValueOnce(3);
    await exportCsv('external_client_permissions', '/tmp/external-client-permissions-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_external_client_permissions_csv', {
      file_path: '/tmp/external-client-permissions-export.csv',
    });
  });

  it('routes generic import/export helpers by entity and format', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(2));
    await importData('deals', 'json', '/tmp/deals.json');
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(3));
    await importData('contacts', 'csv', '/tmp/contacts.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv', {
      file_path: '/tmp/contacts.csv',
    });

    invokeMock.mockResolvedValueOnce(1);
    await exportJson('organizations', '/tmp/orgs-export.json');
    expect(invokeMock).toHaveBeenLastCalledWith('export_organizations_json', {
      file_path: '/tmp/orgs-export.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await exportData('deals', 'json', '/tmp/deals-export.json');
    expect(invokeMock).toHaveBeenLastCalledWith('export_deals_json', {
      file_path: '/tmp/deals-export.json',
    });

    invokeMock.mockResolvedValueOnce(3);
    await exportData('contacts', 'csv', '/tmp/contacts-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_contacts_csv', {
      file_path: '/tmp/contacts-export.csv',
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importData('activities', 'json', '/tmp/activities.json');
    expect(invokeMock).toHaveBeenLastCalledWith('import_activities_json', {
      file_path: '/tmp/activities.json',
    });

    invokeMock.mockResolvedValueOnce(1);
    await exportData('activities', 'csv', '/tmp/activities-export.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('export_activities_csv', {
      file_path: '/tmp/activities-export.csv',
    });

    invokeMock.mockResolvedValueOnce(4);
    await exportJson('audit_log', '/tmp/audit-log-export.json');
    expect(invokeMock).toHaveBeenLastCalledWith('export_audit_log_json', {
      file_path: '/tmp/audit-log-export.json',
    });

    invokeMock.mockResolvedValueOnce(4);
    await exportData('audit_log', 'json', '/tmp/audit-log-export.json');
    expect(invokeMock).toHaveBeenLastCalledWith('export_audit_log_json', {
      file_path: '/tmp/audit-log-export.json',
    });

    invokeMock.mockResolvedValueOnce(3);
    await exportData('proposed_actions', 'json', '/tmp/proposed-actions-export.json');
    expect(invokeMock).toHaveBeenLastCalledWith('export_proposed_actions_json', {
      file_path: '/tmp/proposed-actions-export.json',
    });

    invokeMock.mockResolvedValueOnce(2);
    await exportData('external_clients', 'json', '/tmp/external-clients-export.json');
    expect(invokeMock).toHaveBeenLastCalledWith('export_external_clients_json', {
      file_path: '/tmp/external-clients-export.json',
    });

    invokeMock.mockResolvedValueOnce(3);
    await exportData(
      'external_client_permissions',
      'json',
      '/tmp/external-client-permissions-export.json',
    );
    expect(invokeMock).toHaveBeenLastCalledWith('export_external_client_permissions_json', {
      file_path: '/tmp/external-client-permissions-export.json',
    });
  });

  it('passes duplicate auto-merge for contact, deal, and organization imports when enabled', async () => {
    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importContactsCsv('/tmp/contacts.csv', { mergeDuplicates: true });
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv', {
      file_path: '/tmp/contacts.csv',
      merge_duplicates: true,
    });

    const contactMapping = { Email: 'email', First: 'first_name' } as const;
    invokeMock.mockResolvedValueOnce(importWithBackup(0));
    await importContactsJsonWithMapping('/tmp/contacts.json', contactMapping, {
      mergeDuplicates: true,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json_with_mapping', {
      file_path: '/tmp/contacts.json',
      mapping: contactMapping,
      merge_duplicates: true,
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importOrganizationsJson('/tmp/organizations.json', { mergeDuplicates: true });
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_json', {
      file_path: '/tmp/organizations.json',
      merge_duplicates: true,
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importDealsCsv('/tmp/deals.csv', { mergeDuplicates: true });
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv', {
      file_path: '/tmp/deals.csv',
      merge_duplicates: true,
    });

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importData('deals', 'json', '/tmp/deals.json', { mergeDuplicates: true });
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json', {
      file_path: '/tmp/deals.json',
      merge_duplicates: true,
    });

    const dealMapping = { Opportunity: 'title', Amount: 'value' } as const;
    invokeMock.mockResolvedValueOnce(importWithBackup(0));
    await importDealsJsonWithMapping('/tmp/deals.json', dealMapping, {
      mergeDuplicates: true,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json_with_mapping', {
      file_path: '/tmp/deals.json',
      mapping: dealMapping,
      merge_duplicates: true,
    });
  });

  it('maps CSV preflight duplicate warning commands', async () => {
    invokeMock.mockResolvedValueOnce({
      entity_type: 'contacts',
      total_rows: 2,
      duplicate_warning_count: 1,
      warnings: [
        {
          entity_type: 'contacts',
          row_number: 2,
          match_type: 'email',
          csv_value: 'ada@example.com',
          existing_entity_type: 'contact',
          existing_entity_id: 'contact-1',
          existing_display_label: 'Ada Lovelace',
          reason: "Email 'ada@example.com' matches existing contact",
        },
      ],
    });
    await expect(preflightContactsCsvImport('/tmp/contacts.csv')).resolves.toMatchObject({
      entity_type: 'contacts',
      duplicate_warning_count: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_contacts_csv_import', {
      file_path: '/tmp/contacts.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [],
    });
    await expect(preflightOrganizationsCsvImport('/tmp/organizations.csv')).resolves.toMatchObject({
      entity_type: 'organizations',
      total_rows: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_organizations_csv_import', {
      file_path: '/tmp/organizations.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [],
    });
    await expect(preflightDealsCsvImport('/tmp/deals.csv')).resolves.toMatchObject({
      entity_type: 'deals',
      duplicate_warning_count: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_deals_csv_import', {
      file_path: '/tmp/deals.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'activities',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await expect(preflightActivitiesCsvImport('/tmp/activities.csv')).resolves.toMatchObject({
      entity_type: 'activities',
      duplicate_warning_count: 0,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_activities_csv_import', {
      file_path: '/tmp/activities.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'notes',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await expect(preflightNotesCsvImport('/tmp/notes.csv')).resolves.toMatchObject({
      entity_type: 'notes',
      duplicate_warning_count: 0,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_notes_csv_import', {
      file_path: '/tmp/notes.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_definitions',
      total_rows: 2,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await expect(preflightTagDefinitionsCsvImport('/tmp/tag-definitions.csv')).resolves.toMatchObject({
      entity_type: 'tag_definitions',
      duplicate_warning_count: 0,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_tag_definitions_csv_import', {
      file_path: '/tmp/tag-definitions.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_links',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await expect(preflightTagLinksCsvImport('/tmp/tag-links.csv')).resolves.toMatchObject({
      entity_type: 'tag_links',
      duplicate_warning_count: 0,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_tag_links_csv_import', {
      file_path: '/tmp/tag-links.csv',
    });
  });

  it('routes generic CSV preflight helpers by entity', async () => {
    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsv('organizations', '/tmp/orgs.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_organizations_csv_import', {
      file_path: '/tmp/orgs.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsv('deals', '/tmp/deals.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_deals_csv_import', {
      file_path: '/tmp/deals.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'activities',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsv('activities', '/tmp/activities.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_activities_csv_import', {
      file_path: '/tmp/activities.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'notes',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsv('notes', '/tmp/notes.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_notes_csv_import', {
      file_path: '/tmp/notes.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_definitions',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsv('tag_definitions', '/tmp/tag-definitions.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_tag_definitions_csv_import', {
      file_path: '/tmp/tag-definitions.csv',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_links',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsv('tag_links', '/tmp/tag-links.csv');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_tag_links_csv_import', {
      file_path: '/tmp/tag-links.csv',
    });
  });

  it('maps JSON preflight duplicate warning commands', async () => {
    invokeMock.mockResolvedValueOnce({
      entity_type: 'contacts',
      total_rows: 2,
      duplicate_warning_count: 1,
      warnings: [
        {
          entity_type: 'contacts',
          row_number: 2,
          match_type: 'email',
          csv_value: 'ada@example.com',
          existing_entity_type: 'contact',
          existing_entity_id: 'contact-1',
          existing_display_label: 'Ada Lovelace',
          reason: "Email 'ada@example.com' matches existing contact",
        },
      ],
    });
    await expect(preflightContactsJsonImport('/tmp/contacts.json')).resolves.toMatchObject({
      entity_type: 'contacts',
      duplicate_warning_count: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_contacts_json_import', {
      file_path: '/tmp/contacts.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [],
    });
    await expect(preflightOrganizationsJsonImport('/tmp/organizations.json')).resolves.toMatchObject({
      entity_type: 'organizations',
      total_rows: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_organizations_json_import', {
      file_path: '/tmp/organizations.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [],
    });
    await expect(preflightDealsJsonImport('/tmp/deals.json')).resolves.toMatchObject({
      entity_type: 'deals',
      duplicate_warning_count: 1,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_deals_json_import', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'activities',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await expect(preflightActivitiesJsonImport('/tmp/activities.json')).resolves.toMatchObject({
      entity_type: 'activities',
      duplicate_warning_count: 0,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_activities_json_import', {
      file_path: '/tmp/activities.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'notes',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await expect(preflightNotesJsonImport('/tmp/notes.json')).resolves.toMatchObject({
      entity_type: 'notes',
      duplicate_warning_count: 0,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_notes_json_import', {
      file_path: '/tmp/notes.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_definitions',
      total_rows: 2,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await expect(preflightTagDefinitionsJsonImport('/tmp/tag-definitions.json')).resolves.toMatchObject({
      entity_type: 'tag_definitions',
      duplicate_warning_count: 0,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_tag_definitions_json_import', {
      file_path: '/tmp/tag-definitions.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_links',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await expect(preflightTagLinksJsonImport('/tmp/tag-links.json')).resolves.toMatchObject({
      entity_type: 'tag_links',
      duplicate_warning_count: 0,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_tag_links_json_import', {
      file_path: '/tmp/tag-links.json',
    });
  });

  it('routes generic JSON preflight helpers by entity', async () => {
    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJson('organizations', '/tmp/orgs.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_organizations_json_import', {
      file_path: '/tmp/orgs.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJson('deals', '/tmp/deals.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_deals_json_import', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'activities',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJson('activities', '/tmp/activities.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_activities_json_import', {
      file_path: '/tmp/activities.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'notes',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJson('notes', '/tmp/notes.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_notes_json_import', {
      file_path: '/tmp/notes.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_definitions',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJson('tag_definitions', '/tmp/tag-definitions.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_tag_definitions_json_import', {
      file_path: '/tmp/tag-definitions.json',
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_links',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJson('tag_links', '/tmp/tag-links.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_tag_links_json_import', {
      file_path: '/tmp/tag-links.json',
    });
  });

  it('maps JSON preview commands', async () => {
    const preview = {
      total_rows: 1,
      headers: ['first_name', 'email'],
      rows: [{ row_number: 2, values: { first_name: 'Ada', email: 'ada@example.com' } }],
    };

    invokeMock.mockResolvedValueOnce(preview);
    await expect(previewContactsJsonImport('/tmp/contacts.json')).resolves.toEqual(preview);
    expect(invokeMock).toHaveBeenLastCalledWith('preview_contacts_json_import', {
      file_path: '/tmp/contacts.json',
    });

    invokeMock.mockResolvedValueOnce({ ...preview, headers: ['title'] });
    await previewDealsJsonImport('/tmp/deals.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_deals_json_import', {
      file_path: '/tmp/deals.json',
    });

    invokeMock.mockResolvedValueOnce({ ...preview, headers: ['activity_type', 'title'] });
    await previewActivitiesJsonImport('/tmp/activities.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_activities_json_import', {
      file_path: '/tmp/activities.json',
    });

    invokeMock.mockResolvedValueOnce({ ...preview, headers: ['name'] });
    await previewOrganizationsJsonImport('/tmp/organizations.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_organizations_json_import', {
      file_path: '/tmp/organizations.json',
    });

    invokeMock.mockResolvedValueOnce({ ...preview, headers: ['entity_type', 'entity_id', 'content'] });
    await previewNotesJsonImport('/tmp/notes.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_notes_json_import', {
      file_path: '/tmp/notes.json',
    });

    invokeMock.mockResolvedValueOnce(preview);
    await previewTagDefinitionsJsonImport('/tmp/tag-definitions.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_tag_definitions_json_import', {
      file_path: '/tmp/tag-definitions.json',
    });

    invokeMock.mockResolvedValueOnce(preview);
    await previewTagLinksJsonImport('/tmp/tag-links.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_tag_links_json_import', {
      file_path: '/tmp/tag-links.json',
    });
  });

  it('routes generic JSON preview helpers by entity', async () => {
    invokeMock.mockResolvedValueOnce({
      total_rows: 1,
      headers: ['name'],
      rows: [{ row_number: 2, values: { name: 'Acme' } }],
    });

    await previewJson('organizations', '/tmp/orgs.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_organizations_json_import', {
      file_path: '/tmp/orgs.json',
    });

    invokeMock.mockResolvedValueOnce({
      total_rows: 1,
      headers: ['activity_type', 'title'],
      rows: [{ row_number: 2, values: { activity_type: 'task', title: 'Follow up' } }],
    });

    await previewJson('activities', '/tmp/activities.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_activities_json_import', {
      file_path: '/tmp/activities.json',
    });

    invokeMock.mockResolvedValueOnce({
      total_rows: 1,
      headers: ['entity_type', 'entity_id', 'content'],
      rows: [{ row_number: 2, values: { entity_type: 'contact', entity_id: 'contact-1', content: 'Note' } }],
    });

    await previewJson('notes', '/tmp/notes.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_notes_json_import', {
      file_path: '/tmp/notes.json',
    });

    invokeMock.mockResolvedValueOnce({
      total_rows: 1,
      headers: ['name', 'color'],
      rows: [{ row_number: 2, values: { name: 'VIP', color: '#ef4444' } }],
    });

    await previewJson('tag_definitions', '/tmp/tag-definitions.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_tag_definitions_json_import', {
      file_path: '/tmp/tag-definitions.json',
    });

    invokeMock.mockResolvedValueOnce({
      total_rows: 1,
      headers: ['entity_type', 'entity_id', 'tag_id'],
      rows: [{ row_number: 2, values: { entity_type: 'contact', entity_id: 'contact-1', tag_id: 'tag-1' } }],
    });

    await previewJson('tag_links', '/tmp/tag-links.json');
    expect(invokeMock).toHaveBeenLastCalledWith('preview_tag_links_json_import', {
      file_path: '/tmp/tag-links.json',
    });
  });

  it('maps contact CSV import/preflight commands with field mappings', async () => {
    const mapping = {
      'Given Name': 'first_name',
      Surname: 'last_name',
      'Email Address': 'email',
      Ignore: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importContactsCsvWithMapping('/tmp/contacts.csv', mapping)).resolves.toEqual(
      importWithBackup(1),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_csv_with_mapping', {
      file_path: '/tmp/contacts.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'contacts',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightContactsCsvImportWithMapping('/tmp/contacts.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_contacts_csv_import_with_mapping',
      {
        file_path: '/tmp/contacts.csv',
        mapping,
      },
    );
  });

  it('maps contact JSON import/preflight commands with field mappings', async () => {
    const mapping = {
      given: 'first_name',
      surname: 'last_name',
      mail: 'email',
      ignore: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await expect(importContactsJsonWithMapping('/tmp/contacts.json', mapping)).resolves.toEqual(
      importWithBackup(1),
    );
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json_with_mapping', {
      file_path: '/tmp/contacts.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'contacts',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightContactsJsonImportWithMapping('/tmp/contacts.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_contacts_json_import_with_mapping',
      {
        file_path: '/tmp/contacts.json',
        mapping,
      },
    );
  });

  it('passes contact custom field mappings through invoke payloads', async () => {
    const mapping = {
      first: 'first_name',
      vip: 'custom:VIP Tier',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importContactsJsonWithMapping('/tmp/contacts.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_contacts_json_with_mapping', {
      file_path: '/tmp/contacts.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'contacts',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightContactsJsonImportWithMapping('/tmp/contacts.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_contacts_json_import_with_mapping',
      {
        file_path: '/tmp/contacts.json',
        mapping,
      },
    );
  });

  it('maps organization CSV import/preflight commands with field mappings', async () => {
    const mapping = {
      Company: 'name',
      Inbox: 'email',
      Telephone: 'phone',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importOrganizationsCsvWithMapping('/tmp/organizations.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv_with_mapping', {
      file_path: '/tmp/organizations.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightOrganizationsCsvImportWithMapping('/tmp/organizations.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_organizations_csv_import_with_mapping',
      {
        file_path: '/tmp/organizations.csv',
        mapping,
      },
    );
  });

  it('maps organization JSON import/preflight commands with field mappings', async () => {
    const mapping = {
      company: 'name',
      inbox: 'email',
      telephone: 'phone',
      skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importOrganizationsJsonWithMapping('/tmp/organizations.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_json_with_mapping', {
      file_path: '/tmp/organizations.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightOrganizationsJsonImportWithMapping('/tmp/organizations.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_organizations_json_import_with_mapping',
      {
        file_path: '/tmp/organizations.json',
        mapping,
      },
    );
  });

  it('maps deal CSV import/preflight commands with field mappings', async () => {
    const mapping = {
      Opportunity: 'title',
      Amount: 'value',
      Phase: 'stage',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importDealsCsvWithMapping('/tmp/deals.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv_with_mapping', {
      file_path: '/tmp/deals.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightDealsCsvImportWithMapping('/tmp/deals.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_deals_csv_import_with_mapping',
      {
        file_path: '/tmp/deals.csv',
        mapping,
      },
    );
  });

  it('maps deal JSON import/preflight commands with field mappings', async () => {
    const mapping = {
      opportunity: 'title',
      amount: 'value',
      phase: 'stage',
      skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importDealsJsonWithMapping('/tmp/deals.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json_with_mapping', {
      file_path: '/tmp/deals.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightDealsJsonImportWithMapping('/tmp/deals.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_deals_json_import_with_mapping',
      {
        file_path: '/tmp/deals.json',
        mapping,
      },
    );
  });

  it('maps activity CSV import/preflight commands with field mappings', async () => {
    const mapping = {
      Kind: 'activity_type',
      Subject: 'title',
      Done: 'completed',
      Outcome: 'custom:Outcome',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importActivitiesCsvWithMapping('/tmp/activities.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_activities_csv_with_mapping', {
      file_path: '/tmp/activities.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'activities',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightActivitiesCsvImportWithMapping('/tmp/activities.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_activities_csv_import_with_mapping',
      {
        file_path: '/tmp/activities.csv',
        mapping,
      },
    );
  });

  it('maps activity JSON import/preflight commands with field mappings', async () => {
    const mapping = {
      kind: 'activity_type',
      subject: 'title',
      done: 'completed',
      outcome: 'custom:Outcome',
      skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importActivitiesJsonWithMapping('/tmp/activities.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_activities_json_with_mapping', {
      file_path: '/tmp/activities.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'activities',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightActivitiesJsonImportWithMapping('/tmp/activities.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_activities_json_import_with_mapping',
      {
        file_path: '/tmp/activities.json',
        mapping,
      },
    );
  });

  it('maps note CSV and JSON import/preflight commands with field mappings', async () => {
    const csvMapping = {
      Kind: 'entity_type',
      Target: 'entity_id',
      Body: 'content',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importNotesCsvWithMapping('/tmp/notes.csv', csvMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_notes_csv_with_mapping', {
      file_path: '/tmp/notes.csv',
      mapping: csvMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'notes',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightNotesCsvImportWithMapping('/tmp/notes.csv', csvMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_notes_csv_import_with_mapping', {
      file_path: '/tmp/notes.csv',
      mapping: csvMapping,
    });

    const jsonMapping = {
      kind: 'entity_type',
      target: 'entity_id',
      body: 'content',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importNotesJsonWithMapping('/tmp/notes.json', jsonMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_notes_json_with_mapping', {
      file_path: '/tmp/notes.json',
      mapping: jsonMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'notes',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightNotesJsonImportWithMapping('/tmp/notes.json', jsonMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_notes_json_import_with_mapping', {
      file_path: '/tmp/notes.json',
      mapping: jsonMapping,
    });
  });

  it('maps tag definition CSV and JSON import/preflight commands with field mappings', async () => {
    const csvMapping = {
      Label: 'name',
      Hex: 'color',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importTagDefinitionsCsvWithMapping('/tmp/tag-definitions.csv', csvMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_definitions_csv_with_mapping', {
      file_path: '/tmp/tag-definitions.csv',
      mapping: csvMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_definitions',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightTagDefinitionsCsvImportWithMapping('/tmp/tag-definitions.csv', csvMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_tag_definitions_csv_import_with_mapping',
      {
        file_path: '/tmp/tag-definitions.csv',
        mapping: csvMapping,
      },
    );

    const jsonMapping = {
      label: 'name',
      hex: 'color',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importTagDefinitionsJsonWithMapping('/tmp/tag-definitions.json', jsonMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_definitions_json_with_mapping', {
      file_path: '/tmp/tag-definitions.json',
      mapping: jsonMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_definitions',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightTagDefinitionsJsonImportWithMapping('/tmp/tag-definitions.json', jsonMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_tag_definitions_json_import_with_mapping',
      {
        file_path: '/tmp/tag-definitions.json',
        mapping: jsonMapping,
      },
    );
  });

  it('maps custom field definition CSV and JSON import/preflight commands with field mappings', async () => {
    const csvMapping = {
      Owner: 'entity_type',
      Label: 'field_name',
      Kind: 'field_type',
      Options: 'field_options',
      Order: 'sort_order',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCustomFieldDefinitionsCsvWithMapping('/tmp/custom-field-definitions.csv', csvMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'import_custom_field_definitions_csv_with_mapping',
      {
        file_path: '/tmp/custom-field-definitions.csv',
        mapping: csvMapping,
      },
    );

    invokeMock.mockResolvedValueOnce({
      entity_type: 'custom_field_definitions',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCustomFieldDefinitionsCsvImportWithMapping(
      '/tmp/custom-field-definitions.csv',
      csvMapping,
    );
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_custom_field_definitions_csv_import_with_mapping',
      {
        file_path: '/tmp/custom-field-definitions.csv',
        mapping: csvMapping,
      },
    );

    const jsonMapping = {
      owner: 'entity_type',
      label: 'field_name',
      kind: 'field_type',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCustomFieldDefinitionsJsonWithMapping(
      '/tmp/custom-field-definitions.json',
      jsonMapping,
    );
    expect(invokeMock).toHaveBeenLastCalledWith(
      'import_custom_field_definitions_json_with_mapping',
      {
        file_path: '/tmp/custom-field-definitions.json',
        mapping: jsonMapping,
      },
    );

    invokeMock.mockResolvedValueOnce({
      entity_type: 'custom_field_definitions',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCustomFieldDefinitionsJsonImportWithMapping(
      '/tmp/custom-field-definitions.json',
      jsonMapping,
    );
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_custom_field_definitions_json_import_with_mapping',
      {
        file_path: '/tmp/custom-field-definitions.json',
        mapping: jsonMapping,
      },
    );

    invokeMock.mockResolvedValueOnce({ total_rows: 1, headers: ['field_name'], rows: [] });
    await previewCustomFieldDefinitionsJsonImport('/tmp/custom-field-definitions.json');
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preview_custom_field_definitions_json_import',
      {
        file_path: '/tmp/custom-field-definitions.json',
      },
    );
  });

  it('maps tag link CSV and JSON import/preflight commands with field mappings', async () => {
    const csvMapping = {
      Type: 'entity_type',
      Parent: 'entity_id',
      Tag: 'tag_id',
      Skip: null,
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importTagLinksCsvWithMapping('/tmp/tag-links.csv', csvMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_links_csv_with_mapping', {
      file_path: '/tmp/tag-links.csv',
      mapping: csvMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_links',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightTagLinksCsvImportWithMapping('/tmp/tag-links.csv', csvMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_tag_links_csv_import_with_mapping',
      {
        file_path: '/tmp/tag-links.csv',
        mapping: csvMapping,
      },
    );

    const jsonMapping = {
      type: 'entity_type',
      parent: 'entity_id',
      tag: 'tag_id',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importTagLinksJsonWithMapping('/tmp/tag-links.json', jsonMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_links_json_with_mapping', {
      file_path: '/tmp/tag-links.json',
      mapping: jsonMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_links',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightTagLinksJsonImportWithMapping('/tmp/tag-links.json', jsonMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_tag_links_json_import_with_mapping',
      {
        file_path: '/tmp/tag-links.json',
        mapping: jsonMapping,
      },
    );
  });

  it('routes generic mapped CSV helpers by entity', async () => {
    const mapping = {
      Company: 'name',
      Inbox: 'email',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsvWithMapping('organizations', '/tmp/orgs.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_csv_with_mapping', {
      file_path: '/tmp/orgs.csv',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsvWithMapping('organizations', '/tmp/orgs.csv', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_organizations_csv_import_with_mapping',
      {
        file_path: '/tmp/orgs.csv',
        mapping,
      },
    );

    const dealMapping = {
      Opportunity: 'title',
      Amount: 'value',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsvWithMapping('deals', '/tmp/deals.csv', dealMapping, {
      mergeDuplicates: true,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_csv_with_mapping', {
      file_path: '/tmp/deals.csv',
      mapping: dealMapping,
      merge_duplicates: true,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsvWithMapping('deals', '/tmp/deals.csv', dealMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_deals_csv_import_with_mapping',
      {
        file_path: '/tmp/deals.csv',
        mapping: dealMapping,
      },
    );

    const activityMapping = {
      Kind: 'activity_type',
      Subject: 'title',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsvWithMapping('activities', '/tmp/activities.csv', activityMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_activities_csv_with_mapping', {
      file_path: '/tmp/activities.csv',
      mapping: activityMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'activities',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsvWithMapping('activities', '/tmp/activities.csv', activityMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_activities_csv_import_with_mapping',
      {
        file_path: '/tmp/activities.csv',
        mapping: activityMapping,
      },
    );

    const noteMapping = {
      Kind: 'entity_type',
      Target: 'entity_id',
      Body: 'content',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsvWithMapping('notes', '/tmp/notes.csv', noteMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_notes_csv_with_mapping', {
      file_path: '/tmp/notes.csv',
      mapping: noteMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'notes',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsvWithMapping('notes', '/tmp/notes.csv', noteMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_notes_csv_import_with_mapping', {
      file_path: '/tmp/notes.csv',
      mapping: noteMapping,
    });

    const tagDefinitionMapping = {
      Label: 'name',
      Hex: 'color',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsvWithMapping('tag_definitions', '/tmp/tag-definitions.csv', tagDefinitionMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_definitions_csv_with_mapping', {
      file_path: '/tmp/tag-definitions.csv',
      mapping: tagDefinitionMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_definitions',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsvWithMapping('tag_definitions', '/tmp/tag-definitions.csv', tagDefinitionMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_tag_definitions_csv_import_with_mapping',
      {
        file_path: '/tmp/tag-definitions.csv',
        mapping: tagDefinitionMapping,
      },
    );

    const tagLinkMapping = {
      Type: 'entity_type',
      Parent: 'entity_id',
      Tag: 'tag_id',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importCsvWithMapping('tag_links', '/tmp/tag-links.csv', tagLinkMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_links_csv_with_mapping', {
      file_path: '/tmp/tag-links.csv',
      mapping: tagLinkMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_links',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightCsvWithMapping('tag_links', '/tmp/tag-links.csv', tagLinkMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_tag_links_csv_import_with_mapping',
      {
        file_path: '/tmp/tag-links.csv',
        mapping: tagLinkMapping,
      },
    );
  });

  it('routes generic mapped JSON helpers by entity', async () => {
    const mapping = {
      company: 'name',
      inbox: 'email',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importJsonWithMapping('organizations', '/tmp/orgs.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_organizations_json_with_mapping', {
      file_path: '/tmp/orgs.json',
      mapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'organizations',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJsonWithMapping('organizations', '/tmp/orgs.json', mapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_organizations_json_import_with_mapping',
      {
        file_path: '/tmp/orgs.json',
        mapping,
      },
    );

    const dealMapping = {
      opportunity: 'title',
      amount: 'value',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importJsonWithMapping('deals', '/tmp/deals.json', dealMapping, {
      mergeDuplicates: true,
    });
    expect(invokeMock).toHaveBeenLastCalledWith('import_deals_json_with_mapping', {
      file_path: '/tmp/deals.json',
      mapping: dealMapping,
      merge_duplicates: true,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'deals',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJsonWithMapping('deals', '/tmp/deals.json', dealMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_deals_json_import_with_mapping',
      {
        file_path: '/tmp/deals.json',
        mapping: dealMapping,
      },
    );

    const activityMapping = {
      kind: 'activity_type',
      subject: 'title',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importJsonWithMapping('activities', '/tmp/activities.json', activityMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_activities_json_with_mapping', {
      file_path: '/tmp/activities.json',
      mapping: activityMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'activities',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJsonWithMapping('activities', '/tmp/activities.json', activityMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_activities_json_import_with_mapping',
      {
        file_path: '/tmp/activities.json',
        mapping: activityMapping,
      },
    );

    const noteMapping = {
      kind: 'entity_type',
      target: 'entity_id',
      body: 'content',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importJsonWithMapping('notes', '/tmp/notes.json', noteMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_notes_json_with_mapping', {
      file_path: '/tmp/notes.json',
      mapping: noteMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'notes',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJsonWithMapping('notes', '/tmp/notes.json', noteMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('preflight_notes_json_import_with_mapping', {
      file_path: '/tmp/notes.json',
      mapping: noteMapping,
    });

    const tagDefinitionMapping = {
      label: 'name',
      hex: 'color',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importJsonWithMapping('tag_definitions', '/tmp/tag-definitions.json', tagDefinitionMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_definitions_json_with_mapping', {
      file_path: '/tmp/tag-definitions.json',
      mapping: tagDefinitionMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_definitions',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJsonWithMapping('tag_definitions', '/tmp/tag-definitions.json', tagDefinitionMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_tag_definitions_json_import_with_mapping',
      {
        file_path: '/tmp/tag-definitions.json',
        mapping: tagDefinitionMapping,
      },
    );

    const tagLinkMapping = {
      type: 'entity_type',
      parent: 'entity_id',
      tag: 'tag_id',
    } as const;

    invokeMock.mockResolvedValueOnce(importWithBackup(1));
    await importJsonWithMapping('tag_links', '/tmp/tag-links.json', tagLinkMapping);
    expect(invokeMock).toHaveBeenLastCalledWith('import_tag_links_json_with_mapping', {
      file_path: '/tmp/tag-links.json',
      mapping: tagLinkMapping,
    });

    invokeMock.mockResolvedValueOnce({
      entity_type: 'tag_links',
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });
    await preflightJsonWithMapping('tag_links', '/tmp/tag-links.json', tagLinkMapping);
    expect(invokeMock).toHaveBeenLastCalledWith(
      'preflight_tag_links_json_import_with_mapping',
      {
        file_path: '/tmp/tag-links.json',
        mapping: tagLinkMapping,
      },
    );
  });

  it('maps row-level import rollback commands', async () => {
    const rollbackPlan = {
      token: 'rollback-token-1',
      actions: [
        {
          entity_type: 'contact' as const,
          row_number: 2,
          entity_id: 'contact-1',
          operation: 'created' as const,
          changed_fields: [],
          before_import: null,
          post_import: {
            contact_type: 'person',
            first_name: 'Ada',
            last_name: '',
            org_name: '',
            email: 'ada@example.com',
            phone: '',
            address: '',
            city: '',
            country: '',
            org_id: null,
            organization_id: null,
            notes: '',
            updated_at: '2026-06-25T00:00:00Z',
          },
        },
      ],
    };
    const rollbackResult = {
      token: 'rollback-token-1',
      rolled_back: 1,
      skipped: 0,
      errors: [],
    };

    invokeMock.mockResolvedValueOnce(rollbackResult);

    await expect(rollbackCompletedImport(rollbackPlan)).resolves.toEqual(rollbackResult);
    expect(invokeMock).toHaveBeenLastCalledWith('rollback_completed_import', {
      rollback_plan: rollbackPlan,
    });
  });
});
