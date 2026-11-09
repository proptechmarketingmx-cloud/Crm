import { mapColumns, type ColumnMapping } from './csv';
import type { CustomFieldDefinition } from '$lib/api/customFields';
import type {
  ActivityImportTargetField,
  ContactImportTargetField,
  CustomFieldDefinitionImportTargetField,
  CustomFieldImportTargetField,
  DealImportTargetField,
  ImportColumnMapping,
  ImportPreflightEntity,
  ImportTargetField,
  NoteImportTargetField,
  OrganizationImportTargetField,
  TagDefinitionImportTargetField,
  TagLinkImportTargetField,
} from '$lib/api/importExport';

export type MappedImportEntity = ImportPreflightEntity;

export interface ImportFieldOption<TTarget extends ImportTargetField = ImportTargetField> {
  value: TTarget;
  label: string;
  required?: boolean;
}

export interface ImportMappingValidation {
  valid: boolean;
  errors: string[];
}

export type ImportMappingGuidanceKey =
  | 'import.mappingGuidance.contacts'
  | 'import.mappingGuidance.tagLinks';

export const CONTACT_IMPORT_FIELDS: ImportFieldOption<ContactImportTargetField>[] = [
  { value: 'first_name', label: 'First name', required: true },
  { value: 'last_name', label: 'Last name' },
  { value: 'org_name', label: 'Organization' },
  { value: 'email', label: 'Email' },
  { value: 'phone', label: 'Phone' },
  { value: 'address', label: 'Address' },
  { value: 'city', label: 'City' },
  { value: 'country', label: 'Country' },
  { value: 'notes', label: 'Notes' },
];

export const ORGANIZATION_IMPORT_FIELDS: ImportFieldOption<OrganizationImportTargetField>[] = [
  { value: 'name', label: 'Name', required: true },
  { value: 'email', label: 'Email' },
  { value: 'phone', label: 'Phone' },
  { value: 'website', label: 'Website' },
  { value: 'address_line1', label: 'Address line 1' },
  { value: 'address_line2', label: 'Address line 2' },
  { value: 'city', label: 'City' },
  { value: 'region', label: 'Region' },
  { value: 'country', label: 'Country' },
  { value: 'postal_code', label: 'Postal code' },
  { value: 'description', label: 'Description' },
];

export const DEAL_IMPORT_FIELDS: ImportFieldOption<DealImportTargetField>[] = [
  { value: 'title', label: 'Title', required: true },
  { value: 'value', label: 'Value' },
  { value: 'currency', label: 'Currency' },
  { value: 'stage', label: 'Stage' },
  { value: 'expected_close', label: 'Expected close' },
  { value: 'notes', label: 'Notes' },
];

export const ACTIVITY_IMPORT_FIELDS: ImportFieldOption<ActivityImportTargetField>[] = [
  { value: 'activity_type', label: 'Activity type', required: true },
  { value: 'title', label: 'Title', required: true },
  { value: 'description', label: 'Description' },
  { value: 'due_date', label: 'Due date' },
  { value: 'completed', label: 'Completed' },
  { value: 'contact_id', label: 'Contact ID' },
  { value: 'deal_id', label: 'Deal ID' },
];

export const NOTE_IMPORT_FIELDS: ImportFieldOption<NoteImportTargetField>[] = [
  { value: 'entity_type', label: 'Entity type', required: true },
  { value: 'entity_id', label: 'Entity ID', required: true },
  { value: 'content', label: 'Content', required: true },
];

export const TAG_DEFINITION_IMPORT_FIELDS: ImportFieldOption<TagDefinitionImportTargetField>[] = [
  { value: 'name', label: 'Name', required: true },
  { value: 'color', label: 'Color' },
];

export const CUSTOM_FIELD_DEFINITION_IMPORT_FIELDS: ImportFieldOption<CustomFieldDefinitionImportTargetField>[] = [
  { value: 'entity_type', label: 'Entity type', required: true },
  { value: 'field_name', label: 'Field name', required: true },
  { value: 'field_type', label: 'Field type', required: true },
  { value: 'field_options', label: 'Field options' },
  { value: 'sort_order', label: 'Sort order' },
];

export const TAG_LINK_IMPORT_FIELDS: ImportFieldOption<TagLinkImportTargetField>[] = [
  { value: 'entity_type', label: 'Entity type', required: true },
  { value: 'entity_id', label: 'Entity ID', required: true },
  { value: 'tag_id', label: 'Tag ID', required: true },
];

export function importMappingGuidanceKey(entity: MappedImportEntity): ImportMappingGuidanceKey | null {
  if (entity === 'contacts') {
    return 'import.mappingGuidance.contacts';
  }

  if (entity === 'tag_links') {
    return 'import.mappingGuidance.tagLinks';
  }

  return null;
}

type CustomImportEntity = 'contacts' | 'deals' | 'activities' | 'organizations';

const CONTACT_ALIASES: Record<string, ContactImportTargetField> = {
  firstname: 'first_name',
  givenname: 'first_name',
  first: 'first_name',
  lastname: 'last_name',
  surname: 'last_name',
  familyname: 'last_name',
  last: 'last_name',
  organization: 'org_name',
  organisation: 'org_name',
  org: 'org_name',
  company: 'org_name',
  companyname: 'org_name',
  emailaddress: 'email',
  email: 'email',
  phone: 'phone',
  phonenumber: 'phone',
  telephone: 'phone',
  mobile: 'phone',
  address: 'address',
  streetaddress: 'address',
  city: 'city',
  country: 'country',
  notes: 'notes',
  note: 'notes',
};

const ORGANIZATION_ALIASES: Record<string, OrganizationImportTargetField> = {
  name: 'name',
  organization: 'name',
  organisation: 'name',
  org: 'name',
  company: 'name',
  companyname: 'name',
  emailaddress: 'email',
  email: 'email',
  phone: 'phone',
  phonenumber: 'phone',
  telephone: 'phone',
  website: 'website',
  web: 'website',
  url: 'website',
  address: 'address_line1',
  address1: 'address_line1',
  addressline1: 'address_line1',
  streetaddress: 'address_line1',
  address2: 'address_line2',
  addressline2: 'address_line2',
  city: 'city',
  region: 'region',
  state: 'region',
  province: 'region',
  country: 'country',
  postcode: 'postal_code',
  zipcode: 'postal_code',
  postalcode: 'postal_code',
  zip: 'postal_code',
  description: 'description',
  notes: 'description',
};

const DEAL_ALIASES: Record<string, DealImportTargetField> = {
  title: 'title',
  deal: 'title',
  dealname: 'title',
  opportunity: 'title',
  opportunityname: 'title',
  value: 'value',
  amount: 'value',
  dealvalue: 'value',
  currency: 'currency',
  curr: 'currency',
  stage: 'stage',
  pipelinestage: 'stage',
  status: 'stage',
  expectedclose: 'expected_close',
  expectedclosedate: 'expected_close',
  closedate: 'expected_close',
  close: 'expected_close',
  notes: 'notes',
  note: 'notes',
  memo: 'notes',
};

const ACTIVITY_ALIASES: Record<string, ActivityImportTargetField> = {
  activitytype: 'activity_type',
  type: 'activity_type',
  kind: 'activity_type',
  title: 'title',
  subject: 'title',
  summary: 'title',
  description: 'description',
  details: 'description',
  notes: 'description',
  note: 'description',
  body: 'description',
  duedate: 'due_date',
  due: 'due_date',
  deadline: 'due_date',
  completed: 'completed',
  complete: 'completed',
  done: 'completed',
  contactid: 'contact_id',
  localcontactid: 'contact_id',
  dealid: 'deal_id',
  localdealid: 'deal_id',
};

const NOTE_ALIASES: Record<string, NoteImportTargetField> = {
  entitytype: 'entity_type',
  type: 'entity_type',
  kind: 'entity_type',
  parenttype: 'entity_type',
  parententitytype: 'entity_type',
  entityid: 'entity_id',
  id: 'entity_id',
  target: 'entity_id',
  targetid: 'entity_id',
  localtargetid: 'entity_id',
  parentid: 'entity_id',
  parententityid: 'entity_id',
  content: 'content',
  body: 'content',
  note: 'content',
  notes: 'content',
  text: 'content',
};

const TAG_DEFINITION_ALIASES: Record<string, TagDefinitionImportTargetField> = {
  name: 'name',
  tag: 'name',
  tagname: 'name',
  label: 'name',
  color: 'color',
  colour: 'color',
  hex: 'color',
  hexcolor: 'color',
  hexcolour: 'color',
};

const CUSTOM_FIELD_DEFINITION_ALIASES: Record<string, CustomFieldDefinitionImportTargetField> = {
  entitytype: 'entity_type',
  type: 'entity_type',
  ownertype: 'entity_type',
  targettype: 'entity_type',
  fieldname: 'field_name',
  name: 'field_name',
  label: 'field_name',
  customfield: 'field_name',
  customfieldname: 'field_name',
  fieldtype: 'field_type',
  datatype: 'field_type',
  kind: 'field_type',
  options: 'field_options',
  fieldoptions: 'field_options',
  choices: 'field_options',
  selectoptions: 'field_options',
  sortorder: 'sort_order',
  order: 'sort_order',
  displayorder: 'sort_order',
};

const TAG_LINK_ALIASES: Record<string, TagLinkImportTargetField> = {
  entitytype: 'entity_type',
  type: 'entity_type',
  parenttype: 'entity_type',
  parententitytype: 'entity_type',
  entityid: 'entity_id',
  id: 'entity_id',
  targetid: 'entity_id',
  localtargetid: 'entity_id',
  parentid: 'entity_id',
  parententityid: 'entity_id',
  tagid: 'tag_id',
  localtagid: 'tag_id',
};

export function getImportFieldOptions(
  entity: MappedImportEntity,
  customFields: CustomFieldDefinition[] = [],
): ImportFieldOption[] {
  if (entity === 'contacts') {
    return [
      ...CONTACT_IMPORT_FIELDS,
      ...customFieldOptions('contacts', customFields),
    ];
  }

  if (entity === 'deals') {
    return [
      ...DEAL_IMPORT_FIELDS,
      ...customFieldOptions('deals', customFields),
    ];
  }

  if (entity === 'activities') {
    return [
      ...ACTIVITY_IMPORT_FIELDS,
      ...customFieldOptions('activities', customFields),
    ];
  }

  if (entity === 'notes') {
    return NOTE_IMPORT_FIELDS;
  }

  if (entity === 'tag_definitions') {
    return TAG_DEFINITION_IMPORT_FIELDS;
  }

  if (entity === 'custom_field_definitions') {
    return CUSTOM_FIELD_DEFINITION_IMPORT_FIELDS;
  }

  if (entity === 'tag_links') {
    return TAG_LINK_IMPORT_FIELDS;
  }

  return [
    ...ORGANIZATION_IMPORT_FIELDS,
    ...customFieldOptions('organizations', customFields),
  ];
}

export function suggestImportMapping(
  entity: MappedImportEntity,
  headers: string[],
  customFields: CustomFieldDefinition[] = [],
): ColumnMapping {
  const fields = getImportFieldOptions(entity, customFields)
    .map((field) => field.value)
    .filter((field) => !field.startsWith('custom:'));
  const suggested = mapColumns(headers, fields);
  const aliases = getImportAliases(entity);
  const customAliases = getCustomImportAliases(entity, customFields);

  for (const header of headers) {
    const normalized = normalizeHeader(header);
    const alias = aliases[normalized];
    if (alias) {
      suggested[header] = alias;
      continue;
    }

    const customAlias = customAliases[normalized];
    if (customAlias && suggested[header] === null) {
      suggested[header] = customAlias;
    }
  }

  return suggested;
}

function getImportAliases(entity: MappedImportEntity): Record<string, ImportTargetField> {
  if (entity === 'contacts') {
    return CONTACT_ALIASES;
  }

  if (entity === 'deals') {
    return DEAL_ALIASES;
  }

  if (entity === 'activities') {
    return ACTIVITY_ALIASES;
  }

  if (entity === 'notes') {
    return NOTE_ALIASES;
  }

  if (entity === 'tag_definitions') {
    return TAG_DEFINITION_ALIASES;
  }

  if (entity === 'custom_field_definitions') {
    return CUSTOM_FIELD_DEFINITION_ALIASES;
  }

  if (entity === 'tag_links') {
    return TAG_LINK_ALIASES;
  }

  return ORGANIZATION_ALIASES;
}

export function validateImportMapping(
  entity: MappedImportEntity,
  mapping: ColumnMapping,
  customFields: CustomFieldDefinition[] = [],
): ImportMappingValidation {
  const fieldOptions = getImportFieldOptions(entity, customFields);
  const allowedFields = new Set<string>(fieldOptions.map((field) => field.value));
  const assigned = new Map<string, string[]>();
  const errors: string[] = [];

  for (const [source, target] of Object.entries(mapping)) {
    if (target === null) {
      continue;
    }

    if (!allowedFields.has(target)) {
      errors.push(`"${source}" maps to an unsupported field.`);
      continue;
    }

    const sources = assigned.get(target) ?? [];
    sources.push(source);
    assigned.set(target, sources);
  }

  for (const field of fieldOptions) {
    if (field.required && !assigned.has(field.value)) {
      errors.push(`${field.label} is required.`);
    }
  }

  for (const [target, sources] of assigned.entries()) {
    if (sources.length > 1) {
      const label = fieldOptions.find((field) => field.value === target)?.label ?? target;
      errors.push(`${label} is mapped more than once: ${sources.join(', ')}.`);
    }
  }

  return { valid: errors.length === 0, errors };
}

export function toBackendMapping<TTarget extends ImportTargetField>(
  mapping: ColumnMapping,
): ImportColumnMapping<TTarget> {
  return Object.fromEntries(
    Object.entries(mapping).map(([source, target]) => [source, target === '' ? null : target]),
  ) as ImportColumnMapping<TTarget>;
}

function normalizeHeader(header: string): string {
  return header.toLowerCase().replace(/[^a-z0-9]/g, '');
}

function customFieldOptions(
  entity: CustomImportEntity,
  customFields: CustomFieldDefinition[],
): ImportFieldOption<CustomFieldImportTargetField>[] {
  const fields = customFieldsForEntity(entity, customFields);
  const duplicateNames = duplicateCustomFieldNames(fields);

  return fields.map((field) => ({
    value: customFieldTarget(field, duplicateNames.has(field.field_name)),
    label: duplicateNames.has(field.field_name)
      ? `Custom: ${field.field_name} (${field.id})`
      : `Custom: ${field.field_name}`,
  }));
}

function getCustomImportAliases(
  entity: MappedImportEntity,
  customFields: CustomFieldDefinition[],
): Record<string, CustomFieldImportTargetField> {
  if (entity === 'notes') {
    return {};
  }

  if (entity === 'tag_definitions' || entity === 'custom_field_definitions' || entity === 'tag_links') {
    return {};
  }

  const fields = customFieldsForEntity(entity, customFields);
  const duplicateNames = duplicateCustomFieldNames(fields);
  const aliases: Record<string, CustomFieldImportTargetField> = {};
  for (const field of fields) {
    const duplicateName = duplicateNames.has(field.field_name);
    const target = customFieldTarget(field, duplicateName);
    aliases[normalizeHeader(target)] = target;

    if (!duplicateName) {
      aliases[normalizeHeader(field.field_name)] = target;
    }
  }
  return aliases;
}

function customFieldsForEntity(
  entity: CustomImportEntity,
  customFields: CustomFieldDefinition[],
): CustomFieldDefinition[] {
  const expectedEntityType =
    entity === 'contacts'
      ? 'contact'
      : entity === 'deals'
        ? 'deal'
        : entity === 'activities'
          ? 'activity'
          : 'organization';
  return customFields.filter((field) => field.entity_type === expectedEntityType);
}

function duplicateCustomFieldNames(customFields: CustomFieldDefinition[]): Set<string> {
  const counts = new Map<string, number>();
  for (const field of customFields) {
    counts.set(field.field_name, (counts.get(field.field_name) ?? 0) + 1);
  }

  return new Set(
    Array.from(counts.entries())
      .filter(([, count]) => count > 1)
      .map(([fieldName]) => fieldName),
  );
}

function customFieldTarget(
  field: CustomFieldDefinition,
  duplicateName: boolean,
): CustomFieldImportTargetField {
  const escapedFieldName = escapeCustomFieldName(field.field_name);
  return duplicateName ? `custom:${escapedFieldName}#${field.id}` : `custom:${escapedFieldName}`;
}

function escapeCustomFieldName(fieldName: string): string {
  return fieldName.replace(/%/g, '%25').replace(/#/g, '%23');
}
