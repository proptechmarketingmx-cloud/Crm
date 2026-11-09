/**
 * src/lib/api/importExport.ts - Tauri IPC wrappers for import/export commands.
 */

import { invoke } from '@tauri-apps/api/core';

export type ImportEntity =
  | 'contacts'
  | 'deals'
  | 'activities'
  | 'organizations'
  | 'notes'
  | 'tag_definitions'
  | 'custom_field_definitions'
  | 'tag_links';
export type ExportEntity =
  | ImportEntity
  | 'audit_log'
  | 'proposed_actions'
  | 'external_clients'
  | 'external_client_permissions';
export type ImportExportEntity = ImportEntity;
export type ImportPreflightEntity = ImportEntity;
export type ImportFormat = 'csv' | 'json';
export type ExportFormat = 'csv' | 'json';
export type CustomFieldImportTargetField = `custom:${string}`;
export type ContactImportTargetField =
  | 'first_name'
  | 'last_name'
  | 'org_name'
  | 'email'
  | 'phone'
  | 'address'
  | 'city'
  | 'country'
  | 'notes'
  | CustomFieldImportTargetField;
export type OrganizationImportTargetField =
  | 'name'
  | 'email'
  | 'phone'
  | 'website'
  | 'address_line1'
  | 'address_line2'
  | 'city'
  | 'region'
  | 'country'
  | 'postal_code'
  | 'description'
  | CustomFieldImportTargetField;
export type DealImportTargetField =
  | 'title'
  | 'value'
  | 'currency'
  | 'stage'
  | 'expected_close'
  | 'notes'
  | CustomFieldImportTargetField;
export type ActivityImportTargetField =
  | 'activity_type'
  | 'title'
  | 'description'
  | 'due_date'
  | 'completed'
  | 'contact_id'
  | 'deal_id'
  | CustomFieldImportTargetField;
export type NoteImportTargetField =
  | 'entity_type'
  | 'entity_id'
  | 'content';
export type TagDefinitionImportTargetField = 'name' | 'color';
export type CustomFieldDefinitionImportTargetField =
  | 'entity_type'
  | 'field_name'
  | 'field_type'
  | 'field_options'
  | 'sort_order';
export type TagLinkImportTargetField = 'entity_type' | 'entity_id' | 'tag_id';
export type ImportTargetField =
  | ContactImportTargetField
  | DealImportTargetField
  | ActivityImportTargetField
  | OrganizationImportTargetField
  | NoteImportTargetField
  | TagDefinitionImportTargetField
  | CustomFieldDefinitionImportTargetField
  | TagLinkImportTargetField;
export type ImportColumnMapping<TTarget extends string = ImportTargetField> = Record<
  string,
  TTarget | null
>;

export interface ImportResult {
  created: number;
  merged?: number;
  skipped: number;
  errors: string[];
  rollback_plan?: ImportRollbackPlan | null;
}

export interface ImportOptions {
  mergeDuplicates?: boolean;
}

export type ImportRollbackOperation = 'created' | 'merged';

export interface ContactImportRollbackSnapshot {
  contact_type: string;
  first_name: string;
  last_name: string;
  org_name: string;
  email: string;
  phone: string;
  address: string;
  city: string;
  country: string;
  org_id?: string | null;
  organization_id?: string | null;
  notes: string;
  updated_at: string;
  custom_fields?: Record<string, string>;
}

export interface DealImportRollbackSnapshot {
  title: string;
  value: number;
  currency: string;
  stage: string;
  probability: number;
  expected_close?: string | null;
  contact_id?: string | null;
  organization_id?: string | null;
  notes: string;
  updated_at: string;
  custom_fields?: Record<string, string>;
}

export interface ActivityImportRollbackSnapshot {
  activity_type: string;
  title: string;
  description: string;
  due_date?: string | null;
  completed: boolean;
  contact_id?: string | null;
  deal_id?: string | null;
  updated_at: string;
  custom_fields?: Record<string, string>;
}

export interface OrganizationImportRollbackSnapshot {
  name: string;
  email?: string | null;
  phone?: string | null;
  website?: string | null;
  address_line1?: string | null;
  address_line2?: string | null;
  city?: string | null;
  region?: string | null;
  country?: string | null;
  postal_code?: string | null;
  source?: string | null;
  description?: string | null;
  updated_at: string;
  custom_fields?: Record<string, string>;
}

export interface NoteImportRollbackSnapshot {
  entity_type: 'contact' | 'organization' | 'deal' | 'activity';
  entity_id: string;
  content: string;
  updated_at: string;
}

export interface TagDefinitionImportRollbackSnapshot {
  name: string;
  color: string;
  updated_at: string;
}

export interface CustomFieldDefinitionImportRollbackSnapshot {
  entity_type: 'contact' | 'organization' | 'deal' | 'activity';
  field_name: string;
  field_type: 'text' | 'number' | 'date' | 'boolean' | 'select';
  field_options?: string | null;
  sort_order: number;
  created_at: string;
}

export interface TagLinkImportRollbackSnapshot {
  link_id: string;
  entity_type: 'contact' | 'organization' | 'deal' | 'activity';
  entity_id: string;
  tag_id: string;
  created_at: string;
}

export type ImportRollbackAction =
  | {
      entity_type: 'contact';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: ContactImportRollbackSnapshot | null;
      post_import: ContactImportRollbackSnapshot;
    }
  | {
      entity_type: 'deal';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: DealImportRollbackSnapshot | null;
      post_import: DealImportRollbackSnapshot;
    }
  | {
      entity_type: 'activity';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: ActivityImportRollbackSnapshot | null;
      post_import: ActivityImportRollbackSnapshot;
    }
  | {
      entity_type: 'organization';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: OrganizationImportRollbackSnapshot | null;
      post_import: OrganizationImportRollbackSnapshot;
    }
  | {
      entity_type: 'note';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: NoteImportRollbackSnapshot | null;
      post_import: NoteImportRollbackSnapshot;
    }
  | {
      entity_type: 'tag_definition';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: TagDefinitionImportRollbackSnapshot | null;
      post_import: TagDefinitionImportRollbackSnapshot;
    }
  | {
      entity_type: 'custom_field_definition';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: CustomFieldDefinitionImportRollbackSnapshot | null;
      post_import: CustomFieldDefinitionImportRollbackSnapshot;
    }
  | {
      entity_type: 'tag_link';
      row_number: number;
      entity_id: string;
      operation: ImportRollbackOperation;
      changed_fields: string[];
      before_import?: TagLinkImportRollbackSnapshot | null;
      post_import: TagLinkImportRollbackSnapshot;
    };

export interface ImportRollbackPlan {
  token: string;
  actions: ImportRollbackAction[];
}

export interface ImportRollbackRowError {
  entity_type: 'contact' | 'deal' | 'activity' | 'organization' | 'note' | 'tag' | 'tag_link';
  entity_id: string;
  row_number: number;
  code: string;
  message: string;
}

export interface ImportRollbackResult {
  token: string;
  rolled_back: number;
  skipped: number;
  errors: ImportRollbackRowError[];
}

export interface LocalBackupMetadata {
  backup_format_version: number;
  created_at: string;
  app_version: string;
  schema_version: number;
  device_id: string;
  database_file: string;
}

export interface LocalBackup {
  backup_dir: string;
  database_path: string;
  metadata_path: string;
  metadata: LocalBackupMetadata;
}

export interface ImportWithBackupResult {
  import: ImportResult;
  backup: LocalBackup;
}

export interface ImportPreflightReport {
  entity_type: ImportPreflightEntity;
  total_rows: number;
  duplicate_warning_count: number;
  warnings: ImportDuplicateWarning[];
}

export interface ImportDuplicateWarning {
  entity_type: ImportPreflightEntity;
  row_number: number;
  match_type: 'email' | 'phone' | 'name' | 'title';
  csv_value: string;
  existing_entity_type: 'contact' | 'deal' | 'organization';
  existing_entity_id: string;
  existing_display_label: string;
  reason: string;
}

export interface JsonImportPreview {
  total_rows: number;
  headers: string[];
  rows: JsonImportPreviewRow[];
}

export interface JsonImportPreviewRow {
  row_number: number;
  values: Record<string, string>;
}

const importCommands: Record<ImportFormat, Record<ImportExportEntity, string>> = {
  csv: {
    contacts: 'import_contacts_csv',
    deals: 'import_deals_csv',
    activities: 'import_activities_csv',
    organizations: 'import_organizations_csv',
    notes: 'import_notes_csv',
    tag_definitions: 'import_tag_definitions_csv',
    custom_field_definitions: 'import_custom_field_definitions_csv',
    tag_links: 'import_tag_links_csv',
  },
  json: {
    contacts: 'import_contacts_json',
    deals: 'import_deals_json',
    activities: 'import_activities_json',
    organizations: 'import_organizations_json',
    notes: 'import_notes_json',
    tag_definitions: 'import_tag_definitions_json',
    custom_field_definitions: 'import_custom_field_definitions_json',
    tag_links: 'import_tag_links_json',
  },
};

const exportCommands: Record<ExportFormat, Record<ExportEntity, string>> = {
  csv: {
    contacts: 'export_contacts_csv',
    deals: 'export_deals_csv',
    activities: 'export_activities_csv',
    organizations: 'export_organizations_csv',
    notes: 'export_notes_csv',
    tag_definitions: 'export_tag_definitions_csv',
    custom_field_definitions: 'export_custom_field_definitions_csv',
    tag_links: 'export_tag_links_csv',
    audit_log: 'export_audit_log_csv',
    proposed_actions: 'export_proposed_actions_csv',
    external_clients: 'export_external_clients_csv',
    external_client_permissions: 'export_external_client_permissions_csv',
  },
  json: {
    contacts: 'export_contacts_json',
    deals: 'export_deals_json',
    activities: 'export_activities_json',
    organizations: 'export_organizations_json',
    notes: 'export_notes_json',
    tag_definitions: 'export_tag_definitions_json',
    custom_field_definitions: 'export_custom_field_definitions_json',
    tag_links: 'export_tag_links_json',
    audit_log: 'export_audit_log_json',
    proposed_actions: 'export_proposed_actions_json',
    external_clients: 'export_external_clients_json',
    external_client_permissions: 'export_external_client_permissions_json',
  },
};

const preflightCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_csv_import',
  deals: 'preflight_deals_csv_import',
  activities: 'preflight_activities_csv_import',
  organizations: 'preflight_organizations_csv_import',
  notes: 'preflight_notes_csv_import',
  tag_definitions: 'preflight_tag_definitions_csv_import',
  custom_field_definitions: 'preflight_custom_field_definitions_csv_import',
  tag_links: 'preflight_tag_links_csv_import',
};

const preflightJsonCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_json_import',
  deals: 'preflight_deals_json_import',
  activities: 'preflight_activities_json_import',
  organizations: 'preflight_organizations_json_import',
  notes: 'preflight_notes_json_import',
  tag_definitions: 'preflight_tag_definitions_json_import',
  custom_field_definitions: 'preflight_custom_field_definitions_json_import',
  tag_links: 'preflight_tag_links_json_import',
};

const previewJsonCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preview_contacts_json_import',
  deals: 'preview_deals_json_import',
  activities: 'preview_activities_json_import',
  organizations: 'preview_organizations_json_import',
  notes: 'preview_notes_json_import',
  tag_definitions: 'preview_tag_definitions_json_import',
  custom_field_definitions: 'preview_custom_field_definitions_json_import',
  tag_links: 'preview_tag_links_json_import',
};

const importWithMappingCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'import_contacts_csv_with_mapping',
  deals: 'import_deals_csv_with_mapping',
  activities: 'import_activities_csv_with_mapping',
  organizations: 'import_organizations_csv_with_mapping',
  notes: 'import_notes_csv_with_mapping',
  tag_definitions: 'import_tag_definitions_csv_with_mapping',
  custom_field_definitions: 'import_custom_field_definitions_csv_with_mapping',
  tag_links: 'import_tag_links_csv_with_mapping',
};

const preflightWithMappingCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_csv_import_with_mapping',
  deals: 'preflight_deals_csv_import_with_mapping',
  activities: 'preflight_activities_csv_import_with_mapping',
  organizations: 'preflight_organizations_csv_import_with_mapping',
  notes: 'preflight_notes_csv_import_with_mapping',
  tag_definitions: 'preflight_tag_definitions_csv_import_with_mapping',
  custom_field_definitions: 'preflight_custom_field_definitions_csv_import_with_mapping',
  tag_links: 'preflight_tag_links_csv_import_with_mapping',
};

const importJsonWithMappingCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'import_contacts_json_with_mapping',
  deals: 'import_deals_json_with_mapping',
  activities: 'import_activities_json_with_mapping',
  organizations: 'import_organizations_json_with_mapping',
  notes: 'import_notes_json_with_mapping',
  tag_definitions: 'import_tag_definitions_json_with_mapping',
  custom_field_definitions: 'import_custom_field_definitions_json_with_mapping',
  tag_links: 'import_tag_links_json_with_mapping',
};

const preflightJsonWithMappingCommands: Record<ImportPreflightEntity, string> = {
  contacts: 'preflight_contacts_json_import_with_mapping',
  deals: 'preflight_deals_json_import_with_mapping',
  activities: 'preflight_activities_json_import_with_mapping',
  organizations: 'preflight_organizations_json_import_with_mapping',
  notes: 'preflight_notes_json_import_with_mapping',
  tag_definitions: 'preflight_tag_definitions_json_import_with_mapping',
  custom_field_definitions: 'preflight_custom_field_definitions_json_import_with_mapping',
  tag_links: 'preflight_tag_links_json_import_with_mapping',
};

function importOptionArgs(options?: ImportOptions) {
  return options?.mergeDuplicates === undefined
    ? {}
    : { merge_duplicates: options.mergeDuplicates };
}

function filePathArgs(filePath: string, options?: ImportOptions) {
  return { file_path: filePath, ...importOptionArgs(options) };
}

function filePathAndMappingArgs<TTarget extends string>(
  filePath: string,
  mapping: ImportColumnMapping<TTarget>,
  options?: ImportOptions,
) {
  return { file_path: filePath, mapping, ...importOptionArgs(options) };
}

export async function importContactsCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.contacts, filePathArgs(filePath, options));
}

export async function importContactsJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.contacts, filePathArgs(filePath, options));
}

export async function exportContactsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.contacts, filePathArgs(filePath));
}

export async function exportContactsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.contacts, filePathArgs(filePath));
}

export async function preflightContactsCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.contacts, filePathArgs(filePath));
}

export async function preflightContactsJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.contacts, filePathArgs(filePath));
}

export async function previewContactsJsonImport(
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.contacts, filePathArgs(filePath));
}

export async function rollbackCompletedImport(
  rollbackPlan: ImportRollbackPlan,
): Promise<ImportRollbackResult> {
  return invoke<ImportRollbackResult>('rollback_completed_import', {
    rollback_plan: rollbackPlan,
  });
}

export async function importContactsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importContactsJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightContactsCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightContactsJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ContactImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.contacts,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importDealsCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.deals, filePathArgs(filePath, options));
}

export async function importDealsJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.deals, filePathArgs(filePath, options));
}

export async function exportDealsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.deals, filePathArgs(filePath));
}

export async function exportDealsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.deals, filePathArgs(filePath));
}

export async function preflightDealsCsvImport(filePath: string): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.deals, filePathArgs(filePath));
}

export async function preflightDealsJsonImport(filePath: string): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.deals, filePathArgs(filePath));
}

export async function previewDealsJsonImport(filePath: string): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.deals, filePathArgs(filePath));
}

export async function importDealsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importDealsJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightDealsCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightDealsJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<DealImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.deals,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importActivitiesCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.activities, filePathArgs(filePath, options));
}

export async function importActivitiesJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.activities, filePathArgs(filePath, options));
}

export async function exportActivitiesCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.activities, filePathArgs(filePath));
}

export async function exportActivitiesJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.activities, filePathArgs(filePath));
}

export async function preflightActivitiesCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.activities, filePathArgs(filePath));
}

export async function preflightActivitiesJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.activities, filePathArgs(filePath));
}

export async function previewActivitiesJsonImport(
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.activities, filePathArgs(filePath));
}

export async function importActivitiesCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ActivityImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.activities,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importActivitiesJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ActivityImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.activities,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightActivitiesCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ActivityImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.activities,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightActivitiesJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<ActivityImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.activities,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importNotesCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.notes, filePathArgs(filePath, options));
}

export async function importNotesJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.notes, filePathArgs(filePath, options));
}

export async function exportNotesCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.notes, filePathArgs(filePath));
}

export async function exportNotesJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.notes, filePathArgs(filePath));
}

export async function preflightNotesCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.notes, filePathArgs(filePath));
}

export async function preflightNotesJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.notes, filePathArgs(filePath));
}

export async function previewNotesJsonImport(
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.notes, filePathArgs(filePath));
}

export async function importNotesCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<NoteImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.notes,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importNotesJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<NoteImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.notes,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightNotesCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<NoteImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.notes,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightNotesJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<NoteImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.notes,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importTagDefinitionsCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.csv.tag_definitions,
    filePathArgs(filePath, options),
  );
}

export async function importTagDefinitionsJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.json.tag_definitions,
    filePathArgs(filePath, options),
  );
}

export async function exportTagDefinitionsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.tag_definitions, filePathArgs(filePath));
}

export async function exportTagDefinitionsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.tag_definitions, filePathArgs(filePath));
}

export async function preflightTagDefinitionsCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightCommands.tag_definitions,
    filePathArgs(filePath),
  );
}

export async function preflightTagDefinitionsJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonCommands.tag_definitions,
    filePathArgs(filePath),
  );
}

export async function previewTagDefinitionsJsonImport(
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.tag_definitions, filePathArgs(filePath));
}

export async function importTagDefinitionsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<TagDefinitionImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.tag_definitions,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importTagDefinitionsJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<TagDefinitionImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.tag_definitions,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightTagDefinitionsCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<TagDefinitionImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.tag_definitions,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightTagDefinitionsJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<TagDefinitionImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.tag_definitions,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importCustomFieldDefinitionsCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.csv.custom_field_definitions,
    filePathArgs(filePath, options),
  );
}

export async function importCustomFieldDefinitionsJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.json.custom_field_definitions,
    filePathArgs(filePath, options),
  );
}

export async function exportCustomFieldDefinitionsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.custom_field_definitions, filePathArgs(filePath));
}

export async function exportCustomFieldDefinitionsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.custom_field_definitions, filePathArgs(filePath));
}

export async function preflightCustomFieldDefinitionsCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightCommands.custom_field_definitions,
    filePathArgs(filePath),
  );
}

export async function preflightCustomFieldDefinitionsJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonCommands.custom_field_definitions,
    filePathArgs(filePath),
  );
}

export async function previewCustomFieldDefinitionsJsonImport(
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(
    previewJsonCommands.custom_field_definitions,
    filePathArgs(filePath),
  );
}

export async function importCustomFieldDefinitionsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<CustomFieldDefinitionImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.custom_field_definitions,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importCustomFieldDefinitionsJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<CustomFieldDefinitionImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.custom_field_definitions,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightCustomFieldDefinitionsCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<CustomFieldDefinitionImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.custom_field_definitions,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightCustomFieldDefinitionsJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<CustomFieldDefinitionImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.custom_field_definitions,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importTagLinksCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.csv.tag_links,
    filePathArgs(filePath, options),
  );
}

export async function importTagLinksJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.json.tag_links,
    filePathArgs(filePath, options),
  );
}

export async function exportTagLinksCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.tag_links, filePathArgs(filePath));
}

export async function exportTagLinksJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.tag_links, filePathArgs(filePath));
}

export async function preflightTagLinksCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.tag_links, filePathArgs(filePath));
}

export async function preflightTagLinksJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.tag_links, filePathArgs(filePath));
}

export async function previewTagLinksJsonImport(filePath: string): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.tag_links, filePathArgs(filePath));
}

export async function importTagLinksCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<TagLinkImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.tag_links,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importTagLinksJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<TagLinkImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.tag_links,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightTagLinksCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<TagLinkImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.tag_links,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightTagLinksJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<TagLinkImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.tag_links,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importOrganizationsCsv(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.csv.organizations, filePathArgs(filePath, options));
}

export async function importOrganizationsJson(
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(importCommands.json.organizations, filePathArgs(filePath, options));
}

export async function exportOrganizationsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.organizations, filePathArgs(filePath));
}

export async function exportOrganizationsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.organizations, filePathArgs(filePath));
}

export async function preflightOrganizationsCsvImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands.organizations, filePathArgs(filePath));
}

export async function preflightOrganizationsJsonImport(
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands.organizations, filePathArgs(filePath));
}

export async function previewOrganizationsJsonImport(
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands.organizations, filePathArgs(filePath));
}

export async function importOrganizationsCsvWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importOrganizationsJsonWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightOrganizationsCsvImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightOrganizationsJsonImportWithMapping(
  filePath: string,
  mapping: ImportColumnMapping<OrganizationImportTargetField>,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands.organizations,
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function importCsv(
  entity: ImportExportEntity,
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.csv[entity],
    filePathArgs(filePath, options),
  );
}

export async function importJson(
  entity: ImportExportEntity,
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands.json[entity],
    filePathArgs(filePath, options),
  );
}

export async function importData(
  entity: ImportExportEntity,
  format: ImportFormat,
  filePath: string,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importCommands[format][entity],
    filePathArgs(filePath, options),
  );
}

export async function exportAuditLogCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.audit_log, filePathArgs(filePath));
}

export async function exportAuditLogJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.audit_log, filePathArgs(filePath));
}

export async function exportProposedActionsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.proposed_actions, filePathArgs(filePath));
}

export async function exportProposedActionsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.proposed_actions, filePathArgs(filePath));
}

export async function exportExternalClientsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.external_clients, filePathArgs(filePath));
}

export async function exportExternalClientsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.external_clients, filePathArgs(filePath));
}

export async function exportExternalClientPermissionsCsv(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv.external_client_permissions, filePathArgs(filePath));
}

export async function exportExternalClientPermissionsJson(filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json.external_client_permissions, filePathArgs(filePath));
}

export async function exportCsv(entity: ExportEntity, filePath: string): Promise<number> {
  return invoke<number>(exportCommands.csv[entity], filePathArgs(filePath));
}

export async function exportJson(entity: ExportEntity, filePath: string): Promise<number> {
  return invoke<number>(exportCommands.json[entity], filePathArgs(filePath));
}

export async function exportData(
  entity: ExportEntity,
  format: ExportFormat,
  filePath: string,
): Promise<number> {
  return invoke<number>(exportCommands[format][entity], filePathArgs(filePath));
}

export async function preflightCsv(
  entity: ImportPreflightEntity,
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightCommands[entity], filePathArgs(filePath));
}

export async function preflightJson(
  entity: ImportPreflightEntity,
  filePath: string,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(preflightJsonCommands[entity], filePathArgs(filePath));
}

export async function previewJson(
  entity: ImportPreflightEntity,
  filePath: string,
): Promise<JsonImportPreview> {
  return invoke<JsonImportPreview>(previewJsonCommands[entity], filePathArgs(filePath));
}

export async function importCsvWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function importJsonWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
  options?: ImportOptions,
): Promise<ImportWithBackupResult> {
  return invoke<ImportWithBackupResult>(
    importJsonWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping, options),
  );
}

export async function preflightCsvWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping),
  );
}

export async function preflightJsonWithMapping(
  entity: ImportPreflightEntity,
  filePath: string,
  mapping: ImportColumnMapping,
): Promise<ImportPreflightReport> {
  return invoke<ImportPreflightReport>(
    preflightJsonWithMappingCommands[entity],
    filePathAndMappingArgs(filePath, mapping),
  );
}
