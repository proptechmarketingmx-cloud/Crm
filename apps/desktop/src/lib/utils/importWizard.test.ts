import { describe, expect, it } from 'vitest';

import {
  getImportFieldOptions,
  importMappingGuidanceKey,
  suggestImportMapping,
  toBackendMapping,
  validateImportMapping,
} from './importWizard';

describe('import wizard helpers', () => {
  const contactCustomFields = [
    {
      id: 'field-vip-tier',
      entity_type: 'contact' as const,
      field_name: 'VIP Tier',
      field_type: 'text' as const,
      field_options: null,
      sort_order: 0,
      created_at: '2026-06-25T00:00:00Z',
    },
  ];
  const duplicateContactCustomFields = [
    ...contactCustomFields,
    {
      id: 'field-vip-tier-secondary',
      entity_type: 'contact' as const,
      field_name: 'VIP Tier',
      field_type: 'text' as const,
      field_options: null,
      sort_order: 1,
      created_at: '2026-06-25T00:00:00Z',
    },
  ];
  const activityCustomFields = [
    {
      id: 'field-outcome',
      entity_type: 'activity' as const,
      field_name: 'Outcome',
      field_type: 'text' as const,
      field_options: null,
      sort_order: 0,
      created_at: '2026-06-25T00:00:00Z',
    },
  ];
  const organizationCustomFields = [
    {
      id: 'field-segment',
      entity_type: 'organization' as const,
      field_name: 'Segment',
      field_type: 'text' as const,
      field_options: null,
      sort_order: 0,
      created_at: '2026-06-25T00:00:00Z',
    },
  ];

  it('suggests contact mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('contacts', ['Given Name', 'Surname', 'Company', 'Email Address', 'Ignore']),
    ).toEqual({
      'Given Name': 'first_name',
      Surname: 'last_name',
      Company: 'org_name',
      'Email Address': 'email',
      Ignore: null,
    });
  });

  it('suggests organization mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('organizations', [
        'Company Name',
        'Telephone',
        'URL',
        'Zip Code',
        'Notes',
      ]),
    ).toEqual({
      'Company Name': 'name',
      Telephone: 'phone',
      URL: 'website',
      'Zip Code': 'postal_code',
      Notes: 'description',
    });
  });

  it('suggests deal mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('deals', [
        'Opportunity Name',
        'Amount',
        'Pipeline Stage',
        'Close Date',
        'Memo',
      ]),
    ).toEqual({
      'Opportunity Name': 'title',
      Amount: 'value',
      'Pipeline Stage': 'stage',
      'Close Date': 'expected_close',
      Memo: 'notes',
    });
  });

  it('suggests activity mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('activities', [
        'Type',
        'Subject',
        'Details',
        'Due Date',
        'Done',
        'Local Contact ID',
      ]),
    ).toEqual({
      Type: 'activity_type',
      Subject: 'title',
      Details: 'description',
      'Due Date': 'due_date',
      Done: 'completed',
      'Local Contact ID': 'contact_id',
    });
  });

  it('suggests note mappings from common CSV headers', () => {
    expect(
      suggestImportMapping('notes', [
        'Parent Type',
        'Parent ID',
        'Kind',
        'Target',
        'Body',
        'Ignore',
      ]),
    ).toEqual({
      'Parent Type': 'entity_type',
      'Parent ID': 'entity_id',
      Kind: 'entity_type',
      Target: 'entity_id',
      Body: 'content',
      Ignore: null,
    });
  });

  it('suggests tag definition and tag link mappings from local CSV headers', () => {
    expect(suggestImportMapping('tag_definitions', ['Tag Name', 'Hex Color', 'Ignore'])).toEqual({
      'Tag Name': 'name',
      'Hex Color': 'color',
      Ignore: null,
    });

    expect(suggestImportMapping('tag_links', ['Parent Type', 'Parent ID', 'Local Tag ID'])).toEqual({
      'Parent Type': 'entity_type',
      'Parent ID': 'entity_id',
      'Local Tag ID': 'tag_id',
    });
  });

  it('suggests custom field definition mappings from local CSV headers', () => {
    expect(
      suggestImportMapping('custom_field_definitions', [
        'Owner Type',
        'Custom Field Name',
        'Data Type',
        'Select Options',
        'Display Order',
        'Ignore',
      ]),
    ).toEqual({
      'Owner Type': 'entity_type',
      'Custom Field Name': 'field_name',
      'Data Type': 'field_type',
      'Select Options': 'field_options',
      'Display Order': 'sort_order',
      Ignore: null,
    });
  });

  it('adds contact custom fields as mapping options and suggestions', () => {
    expect(getImportFieldOptions('contacts', contactCustomFields)).toContainEqual({
      value: 'custom:VIP Tier',
      label: 'Custom: VIP Tier',
    });

    expect(
      suggestImportMapping('contacts', ['First Name', 'VIP Tier'], contactCustomFields),
    ).toEqual({
      'First Name': 'first_name',
      'VIP Tier': 'custom:VIP Tier',
    });
  });

  it('adds activity custom fields as mapping options and suggestions', () => {
    expect(getImportFieldOptions('activities', activityCustomFields)).toContainEqual({
      value: 'custom:Outcome',
      label: 'Custom: Outcome',
    });

    expect(
      suggestImportMapping('activities', ['Type', 'Subject', 'Outcome'], activityCustomFields),
    ).toEqual({
      Type: 'activity_type',
      Subject: 'title',
      Outcome: 'custom:Outcome',
    });
  });

  it('accepts supported custom mappings and rejects unsupported ones', () => {
    expect(
      validateImportMapping(
        'contacts',
        {
          'First Name': 'first_name',
          'VIP Tier': 'custom:VIP Tier',
        },
        contactCustomFields,
      ),
    ).toEqual({ valid: true, errors: [] });

    const result = validateImportMapping('contacts', {
      'First Name': 'first_name',
      'Unknown Custom': 'custom:Unknown',
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('"Unknown Custom" maps to an unsupported field.');
  });

  it('uses deterministic custom field targets when names collide', () => {
    expect(getImportFieldOptions('contacts', duplicateContactCustomFields)).toEqual(
      expect.arrayContaining([
        {
          value: 'custom:VIP Tier#field-vip-tier',
          label: 'Custom: VIP Tier (field-vip-tier)',
        },
        {
          value: 'custom:VIP Tier#field-vip-tier-secondary',
          label: 'Custom: VIP Tier (field-vip-tier-secondary)',
        },
      ]),
    );

    expect(
      suggestImportMapping(
        'contacts',
        ['VIP Tier', 'custom:VIP Tier#field-vip-tier'],
        duplicateContactCustomFields,
      ),
    ).toEqual({
      'VIP Tier': null,
      'custom:VIP Tier#field-vip-tier': 'custom:VIP Tier#field-vip-tier',
    });
  });

  it('escapes custom field target delimiters in field names', () => {
    expect(
      getImportFieldOptions('contacts', [
        {
          id: 'field-plan',
          entity_type: 'contact' as const,
          field_name: 'Plan #',
          field_type: 'text' as const,
          field_options: null,
          sort_order: 0,
          created_at: '2026-06-25T00:00:00Z',
        },
      ]),
    ).toContainEqual({
      value: 'custom:Plan %23',
      label: 'Custom: Plan #',
    });
  });

  it('adds organization custom fields as mapping options and suggestions', () => {
    expect(getImportFieldOptions('organizations', organizationCustomFields)).toContainEqual({
      value: 'custom:Segment',
      label: 'Custom: Segment',
    });

    expect(
      suggestImportMapping('organizations', ['Company Name', 'Segment'], organizationCustomFields),
    ).toEqual({
      'Company Name': 'name',
      Segment: 'custom:Segment',
    });
  });

  it('blocks missing required contact fields and duplicate targets', () => {
    const result = validateImportMapping('contacts', {
      Email: 'email',
      'Email 2': 'email',
      Notes: null,
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('First name is required.');
    expect(result.errors).toContain('Email is mapped more than once: Email, Email 2.');
  });

  it('blocks missing required deal title and duplicate deal targets', () => {
    const result = validateImportMapping('deals', {
      Amount: 'value',
      Total: 'value',
      Notes: null,
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('Title is required.');
    expect(result.errors).toContain('Value is mapped more than once: Amount, Total.');
  });

  it('blocks missing required activity fields and duplicate activity targets', () => {
    const result = validateImportMapping('activities', {
      Subject: 'title',
      Summary: 'title',
      Done: 'completed',
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('Activity type is required.');
    expect(result.errors).toContain('Title is mapped more than once: Subject, Summary.');
  });

  it('blocks missing required note fields and duplicate note targets', () => {
    const result = validateImportMapping('notes', {
      Kind: 'entity_type',
      Type: 'entity_type',
      Body: 'content',
    });

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('Entity ID is required.');
    expect(result.errors).toContain('Entity type is mapped more than once: Kind, Type.');
  });

  it('blocks missing required tag fields and duplicate tag targets', () => {
    const definitionResult = validateImportMapping('tag_definitions', {
      Hex: 'color',
      Color: 'color',
    });

    expect(definitionResult.valid).toBe(false);
    expect(definitionResult.errors).toContain('Name is required.');
    expect(definitionResult.errors).toContain('Color is mapped more than once: Hex, Color.');

    const customFieldDefinitionResult = validateImportMapping('custom_field_definitions', {
      Owner: 'entity_type',
      Type: 'field_type',
      DuplicateType: 'field_type',
    });

    expect(customFieldDefinitionResult.valid).toBe(false);
    expect(customFieldDefinitionResult.errors).toContain('Field name is required.');
    expect(customFieldDefinitionResult.errors).toContain(
      'Field type is mapped more than once: Type, DuplicateType.',
    );

    const linkResult = validateImportMapping('tag_links', {
      Type: 'entity_type',
      Parent: 'entity_id',
      DuplicateParent: 'entity_id',
    });

    expect(linkResult.valid).toBe(false);
    expect(linkResult.errors).toContain('Tag ID is required.');
    expect(linkResult.errors).toContain('Entity ID is mapped more than once: Parent, DuplicateParent.');
  });

  it('accepts valid organization mappings with skipped columns', () => {
    expect(
      validateImportMapping('organizations', {
        Company: 'name',
        Email: 'email',
        Ignore: null,
      }),
    ).toEqual({ valid: true, errors: [] });
  });

  it('normalizes blank UI targets to null for backend mappings', () => {
    expect(
      toBackendMapping({
        Company: 'name',
        Ignore: '',
      }),
    ).toEqual({
      Company: 'name',
      Ignore: null,
    });
  });

  it('exposes explicit source and tag guidance only where imports need it', () => {
    expect(importMappingGuidanceKey('contacts')).toBe('import.mappingGuidance.contacts');
    expect(importMappingGuidanceKey('tag_links')).toBe('import.mappingGuidance.tagLinks');
    expect(importMappingGuidanceKey('organizations')).toBeNull();
  });
});
