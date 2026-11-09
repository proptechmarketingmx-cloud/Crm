<script lang="ts">
  /**
   * ImportExport.svelte — Import/Export modal for CRM CSV data.
   */

  import { t } from '$lib/i18n';
  import { parseCSV, applyMapping } from '$lib/utils/csv';
  import type { ParseCSVResult, ColumnMapping } from '$lib/utils/csv';
  import { restoreLocalBackupToAppData, validateLocalBackup } from '$lib/api/backup';
  import { reloadWorkspaceAfterDataReplace } from '$lib/stores/reloadWorkspace';
  import { listCustomFieldDefinitions, type CustomFieldDefinition } from '$lib/api/customFields';
  import {
    getImportFieldOptions,
    importMappingGuidanceKey,
    suggestImportMapping,
    toBackendMapping,
    validateImportMapping,
    type MappedImportEntity,
  } from '$lib/utils/importWizard';
  import { uiStore } from '$lib/stores/ui';
  import {
    exportData,
    importCsv,
    importActivitiesCsvWithMapping,
    importContactsCsvWithMapping,
    importDealsCsvWithMapping,
    importJsonWithMapping,
    importOrganizationsCsvWithMapping,
    importCustomFieldDefinitionsCsvWithMapping,
    importTagDefinitionsCsvWithMapping,
    importTagLinksCsvWithMapping,
    rollbackCompletedImport,
    previewJson,
    preflightActivitiesCsvImportWithMapping,
    preflightContactsCsvImportWithMapping,
    preflightDealsCsvImportWithMapping,
    preflightJsonWithMapping,
    preflightOrganizationsCsvImportWithMapping,
    preflightCustomFieldDefinitionsCsvImportWithMapping,
    preflightTagDefinitionsCsvImportWithMapping,
    preflightTagLinksCsvImportWithMapping,
    type ExportFormat,
    type ActivityImportTargetField,
    type ContactImportTargetField,
    type DealImportTargetField,
    type ExportEntity,
    type ImportFormat,
    type ImportEntity,
    type ImportOptions,
    type ImportPreflightReport,
    type ImportResult,
    type ImportRollbackResult,
    type ImportWithBackupResult,
    type JsonImportPreview,
    type CustomFieldDefinitionImportTargetField,
    type NoteImportTargetField,
    type OrganizationImportTargetField,
    type TagDefinitionImportTargetField,
    type TagLinkImportTargetField,
    importNotesCsvWithMapping,
    preflightNotesCsvImportWithMapping,
  } from '$lib/api/importExport';

  let {
    open = $bindable(false),
    onclose,
  }: {
    open?: boolean;
    onclose?: () => void;
  } = $props();

  type ImportStep = 'select' | 'preview' | 'mapping' | 'duplicates' | 'confirm' | 'summary';
  type FileSource = 'desktop' | 'browser';

  let activeTab = $state<'import' | 'export'>('import');

  let csvText = $state('');
  let parseResult = $state<ParseCSVResult | null>(null);
  let jsonPreview = $state<JsonImportPreview | null>(null);
  let selectedImportPath = $state<string | null>(null);
  let selectedImportLabel = $state<string | null>(null);
  let fileSource = $state<FileSource | null>(null);
  let isImporting = $state(false);
  let isPreviewing = $state(false);
  let isPreflighting = $state(false);
  let importEntity = $state<ImportEntity>('contacts');
  let importFormat = $state<ImportFormat>('csv');
  let importStep = $state<ImportStep>('select');
  let columnMapping = $state<ColumnMapping>({});
  let validationErrors = $state<string[]>([]);
  let preflightReport = $state<ImportPreflightReport | null>(null);
  let importSummary = $state<ImportResult | null>(null);
  let importBackupPath = $state<string | null>(null);
  let mergeDuplicateImportRows = $state(false);
  let isRestoringImportBackup = $state(false);
  let importRestoreMessage = $state<string | null>(null);
  let importRestoreError = $state<string | null>(null);
  let isRollingBackImportRows = $state(false);
  let importRollbackMessage = $state<string | null>(null);
  let importRollbackError = $state<string | null>(null);
  let importRollbackResult = $state<ImportRollbackResult | null>(null);
  let importRollbackCompleted = $state(false);
  let importCustomFieldDefinitions = $state<{
    contacts: CustomFieldDefinition[];
    deals: CustomFieldDefinition[];
    activities: CustomFieldDefinition[];
    organizations: CustomFieldDefinition[];
    notes: CustomFieldDefinition[];
    tag_definitions: CustomFieldDefinition[];
    custom_field_definitions: CustomFieldDefinition[];
    tag_links: CustomFieldDefinition[];
  }>({
    contacts: [],
    deals: [],
    activities: [],
    organizations: [],
    notes: [],
    tag_definitions: [],
    custom_field_definitions: [],
    tag_links: [],
  });
  let loadedImportCustomFields = $state<{ contacts: boolean; deals: boolean; activities: boolean; organizations: boolean; notes: boolean; tag_definitions: boolean; custom_field_definitions: boolean; tag_links: boolean }>({
    contacts: false,
    deals: false,
    activities: false,
    organizations: false,
    notes: true,
    tag_definitions: true,
    custom_field_definitions: true,
    tag_links: true,
  });

  let exportEntity = $state<ExportEntity>('contacts');
  let exportFormat = $state<ExportFormat>('csv');
  let isExporting = $state(false);

  const importDialogMetadata: Record<ImportFormat, { extension: string; filterName: string }> = {
    csv: { extension: 'csv', filterName: 'CSV' },
    json: { extension: 'json', filterName: 'JSON' },
  };

  const exportDialogMetadata: Record<ExportFormat, { defaultExtension: string; filterName: string }> = {
    csv: { defaultExtension: 'csv', filterName: 'CSV' },
    json: { defaultExtension: 'json', filterName: 'JSON' },
  };

  const previewRows = $derived(parseResult?.rows.slice(0, 5) ?? []);
  const jsonPreviewRows = $derived(jsonPreview?.rows ?? []);
  const isMappedImport = $derived(
    importEntity === 'contacts' ||
      importEntity === 'deals' ||
      importEntity === 'activities' ||
      importEntity === 'organizations' ||
      importEntity === 'notes' ||
      importEntity === 'tag_definitions' ||
      importEntity === 'custom_field_definitions' ||
      importEntity === 'tag_links',
  );
  const isCsvImport = $derived(importFormat === 'csv');
  const isJsonImport = $derived(importFormat === 'json');
  const showMappingWizard = $derived(isMappedImport && (isCsvImport || isJsonImport));
  const usesDuplicatePreflight = $derived(
    importEntity !== 'activities' &&
      importEntity !== 'notes' &&
      importEntity !== 'tag_definitions' &&
      importEntity !== 'custom_field_definitions' &&
      importEntity !== 'tag_links',
  );
  const showDuplicateReview = $derived(showMappingWizard && usesDuplicatePreflight);
  const mappedEntity = $derived(isMappedImport ? (importEntity as MappedImportEntity) : null);
  const activeImportCustomFields = $derived(
    importEntity === 'contacts'
      ? importCustomFieldDefinitions.contacts
      : importEntity === 'deals'
        ? importCustomFieldDefinitions.deals
        : importEntity === 'activities'
            ? importCustomFieldDefinitions.activities
            : importEntity === 'organizations'
              ? importCustomFieldDefinitions.organizations
              : importEntity === 'notes'
                ? importCustomFieldDefinitions.notes
                : importEntity === 'tag_definitions'
                  ? importCustomFieldDefinitions.tag_definitions
                  : importEntity === 'custom_field_definitions'
                    ? importCustomFieldDefinitions.custom_field_definitions
                    : importCustomFieldDefinitions.tag_links,
  );
  const importFieldOptions = $derived(
    mappedEntity ? getImportFieldOptions(mappedEntity, activeImportCustomFields) : [],
  );
  const mappingGuidanceKey = $derived(mappedEntity ? importMappingGuidanceKey(mappedEntity) : null);
  const sourceHeaders = $derived(isJsonImport ? (jsonPreview?.headers ?? []) : (parseResult?.headers ?? []));
  const sourcePreviewRows = $derived(
    isJsonImport ? jsonPreviewRows.map((row) => row.values) : previewRows,
  );
  const mappedPreviewRows = $derived(applyMapping(sourcePreviewRows, columnMapping));
  const canUseMappedCommands = $derived(Boolean(showMappingWizard && fileSource === 'desktop' && selectedImportPath));
  const fallbackImportBlocked = $derived(showMappingWizard && isCsvImport && fileSource === 'browser');
  const duplicateWarnings = $derived(preflightReport?.warnings ?? []);
  const canAutoMergeDuplicates = $derived(
    importEntity === 'contacts' || importEntity === 'deals' || importEntity === 'organizations',
  );
  const importRollbackPlan = $derived(importSummary?.rollback_plan ?? null);
  const importRollbackActionCount = $derived(importRollbackPlan?.actions.length ?? 0);

  async function ensureImportCustomFields(entity: ImportEntity): Promise<CustomFieldDefinition[]> {
    if (
      entity === 'notes' ||
      entity === 'tag_definitions' ||
      entity === 'custom_field_definitions' ||
      entity === 'tag_links'
    ) {
      return [];
    }

    if (loadedImportCustomFields[entity]) {
      return importCustomFieldDefinitions[entity];
    }

    const entityType =
      entity === 'contacts'
        ? 'contact'
        : entity === 'deals'
          ? 'deal'
          : entity === 'activities'
            ? 'activity'
            : 'organization';
    const definitions = await listCustomFieldDefinitions(entityType);
    importCustomFieldDefinitions = {
      ...importCustomFieldDefinitions,
      [entity]: definitions,
    };
    loadedImportCustomFields = {
      ...loadedImportCustomFields,
      [entity]: true,
    };

    return definitions;
  }

  async function handleFilePick() {
    try {
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
      const dialogMetadata = importDialogMetadata[importFormat];
      const selected = await openDialog({
        filters: [{ name: dialogMetadata.filterName, extensions: [dialogMetadata.extension] }],
        multiple: false,
      });

      if (typeof selected !== 'string') {
        return;
      }

      const customFields = await ensureImportCustomFields(importEntity);

      if (isJsonImport) {
        await loadSelectedJson(selected, selected, customFields);
        return;
      }

      const { invoke } = await import('@tauri-apps/api/core');
      const text = await invoke<string>('read_import_text', { file_path: selected });
      loadSelectedCsv(text, selected, selected, 'desktop', customFields);
    } catch {
      if (isCsvImport) {
        document.getElementById('csv-file-input')?.click();
      } else {
        uiStore.toastError(t('import.failed'));
      }
    }
  }

  function handleFileInputChange(e: Event) {
    if (!isCsvImport) {
      return;
    }

    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) {
      return;
    }

    const reader = new FileReader();
    reader.onload = async (ev) => {
      const text = (ev.target?.result as string) ?? '';
      const customFields = await ensureImportCustomFields(importEntity);
      loadSelectedCsv(text, file.name, file.name, 'browser', customFields);
    };
    reader.readAsText(file);
  }

  function handleImportEntityChange(e: Event) {
    importEntity = (e.target as HTMLSelectElement).value as ImportEntity;
    resetImportState({ keepEntity: true, keepFormat: true });
  }

  function handleImportFormatChange(e: Event) {
    importFormat = (e.target as HTMLSelectElement).value as ImportFormat;
    resetImportState({ keepEntity: true, keepFormat: true });
  }

  function loadSelectedCsv(
    text: string,
    label: string,
    path: string | null,
    source: FileSource,
    customFields: CustomFieldDefinition[] = activeImportCustomFields,
  ) {
    csvText = text;
    selectedImportLabel = label;
    selectedImportPath = path;
    fileSource = source;
    isPreviewing = false;
    parseResult = parseCSV(csvText);
    jsonPreview = null;
    preflightReport = null;
    importSummary = null;
    importBackupPath = null;
    mergeDuplicateImportRows = false;
    importRestoreMessage = null;
    importRestoreError = null;
    validationErrors = [];
    importStep = parseResult.headers.length > 0 ? 'preview' : 'select';

    if (isMappedImport && mappedEntity) {
      columnMapping = suggestImportMapping(mappedEntity, parseResult.headers, customFields);
    } else {
      columnMapping = {};
    }
  }

  async function loadSelectedJson(
    label: string,
    path: string,
    customFields: CustomFieldDefinition[] = activeImportCustomFields,
  ) {
    csvText = '';
    parseResult = null;
    jsonPreview = null;
    selectedImportLabel = label;
    selectedImportPath = path;
    fileSource = 'desktop';
    isPreviewing = true;
    preflightReport = null;
    importSummary = null;
    importBackupPath = null;
    importRestoreMessage = null;
    importRestoreError = null;
    isRollingBackImportRows = false;
    importRollbackMessage = null;
    importRollbackError = null;
    importRollbackResult = null;
    importRollbackCompleted = false;
    validationErrors = [];
    columnMapping = {};
    importStep = 'select';

    try {
      jsonPreview = await previewJson(importEntity, path);
      if (mappedEntity) {
        columnMapping = suggestImportMapping(mappedEntity, jsonPreview.headers, customFields);
      }
      importStep = 'preview';
    } catch {
      selectedImportPath = null;
      validationErrors = [t('import.previewFailed')];
      uiStore.toastError(t('import.previewFailed'));
    } finally {
      isPreviewing = false;
    }
  }

  function resetImportState(options: { keepEntity?: boolean; keepFormat?: boolean } = {}) {
    csvText = '';
    parseResult = null;
    jsonPreview = null;
    selectedImportPath = null;
    selectedImportLabel = null;
    fileSource = null;
    isImporting = false;
    isPreviewing = false;
    isPreflighting = false;
    importStep = 'select';
    columnMapping = {};
    validationErrors = [];
    preflightReport = null;
    importSummary = null;
    importBackupPath = null;
    importRestoreMessage = null;
    importRestoreError = null;

    if (!options.keepEntity) {
      importEntity = 'contacts';
    }

    if (!options.keepFormat) {
      importFormat = 'csv';
    }
  }

  function updateMapping(sourceHeader: string, target: string | null) {
    columnMapping = { ...columnMapping, [sourceHeader]: target };
    validationErrors = [];
    preflightReport = null;
    importSummary = null;
    importBackupPath = null;
    importRestoreMessage = null;
    importRestoreError = null;
    importRollbackMessage = null;
    importRollbackError = null;
    importRollbackResult = null;
    importRollbackCompleted = false;
  }

  function validateCurrentMapping(): boolean {
    if (!mappedEntity) {
      return true;
    }

    const result = validateImportMapping(mappedEntity, columnMapping, activeImportCustomFields);
    validationErrors = result.errors;
    return result.valid;
  }

  function goToPreview() {
    if (!parseResult && !jsonPreview) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }
    importStep = 'preview';
  }

  function goToMapping() {
    if (sourceHeaders.length === 0) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }
    importStep = 'mapping';
  }

  async function handleLegacyImport() {
    if (!selectedImportPath) {
      uiStore.toastError(t('import.chooseFile'));
      return;
    }

    isImporting = true;
    try {
      const result = await importCsv(importEntity, selectedImportPath);
      applyImportResult(result);

      if (result.import.skipped > 0) {
        uiStore.toastWarning(`${t('import.success')} (${result.import.created} created, ${result.import.skipped} skipped)`);
      } else {
        uiStore.toastSuccess(`${t('import.success')} (${result.import.created})`);
      }

      close();
    } catch {
      uiStore.toastError(t('import.failed'));
    } finally {
      isImporting = false;
    }
  }

  async function handlePreflight() {
    if (!mappedEntity || !validateCurrentMapping()) {
      return;
    }

    if (!canUseMappedCommands || !selectedImportPath) {
      validationErrors = [t('import.desktopPickerRequired')];
      return;
    }

    isPreflighting = true;
    validationErrors = [];
    try {
      preflightReport = await runMappedPreflight(mappedEntity, selectedImportPath);
      importStep = preflightReport.duplicate_warning_count > 0 ? 'duplicates' : 'confirm';
    } catch {
      validationErrors = [t('import.preflightFailed')];
    } finally {
      isPreflighting = false;
    }
  }

  async function handleMappedImport() {
    if (!mappedEntity || !validateCurrentMapping()) {
      return;
    }

    if (!canUseMappedCommands || !selectedImportPath) {
      validationErrors = [t('import.desktopPickerRequired')];
      return;
    }

    isImporting = true;
    validationErrors = [];
    try {
      applyImportResult(await runMappedImport(mappedEntity, selectedImportPath));
      importStep = 'summary';
      uiStore.toastSuccess(t('import.success'));
    } catch {
      validationErrors = [t('import.failed')];
    } finally {
      isImporting = false;
    }
  }

  async function runMappedPreflight(
    entity: MappedImportEntity,
    filePath: string,
  ): Promise<ImportPreflightReport> {
    if (isJsonImport) {
      return preflightJsonWithMapping(entity, filePath, toBackendMapping(columnMapping));
    }

    if (entity === 'contacts') {
      return preflightContactsCsvImportWithMapping(
        filePath,
        toBackendMapping<ContactImportTargetField>(columnMapping),
      );
    }

    if (entity === 'deals') {
      return preflightDealsCsvImportWithMapping(
        filePath,
        toBackendMapping<DealImportTargetField>(columnMapping),
      );
    }

    if (entity === 'activities') {
      return preflightActivitiesCsvImportWithMapping(
        filePath,
        toBackendMapping<ActivityImportTargetField>(columnMapping),
      );
    }

    if (entity === 'notes') {
      return preflightNotesCsvImportWithMapping(
        filePath,
        toBackendMapping<NoteImportTargetField>(columnMapping),
      );
    }

    if (entity === 'tag_definitions') {
      return preflightTagDefinitionsCsvImportWithMapping(
        filePath,
        toBackendMapping<TagDefinitionImportTargetField>(columnMapping),
      );
    }

    if (entity === 'custom_field_definitions') {
      return preflightCustomFieldDefinitionsCsvImportWithMapping(
        filePath,
        toBackendMapping<CustomFieldDefinitionImportTargetField>(columnMapping),
      );
    }

    if (entity === 'tag_links') {
      return preflightTagLinksCsvImportWithMapping(
        filePath,
        toBackendMapping<TagLinkImportTargetField>(columnMapping),
      );
    }

    return preflightOrganizationsCsvImportWithMapping(
      filePath,
      toBackendMapping<OrganizationImportTargetField>(columnMapping),
    );
  }

  async function runMappedImport(
    entity: MappedImportEntity,
    filePath: string,
  ): Promise<ImportWithBackupResult> {
    const importOptions = duplicateAutoMergeOptions();

    if (isJsonImport) {
      const mapping = toBackendMapping(columnMapping);
      return importOptions
        ? importJsonWithMapping(entity, filePath, mapping, importOptions)
        : importJsonWithMapping(entity, filePath, mapping);
    }

    if (entity === 'contacts') {
      const mapping = toBackendMapping<ContactImportTargetField>(columnMapping);
      return importOptions
        ? importContactsCsvWithMapping(filePath, mapping, importOptions)
        : importContactsCsvWithMapping(filePath, mapping);
    }

    if (entity === 'deals') {
      const mapping = toBackendMapping<DealImportTargetField>(columnMapping);
      return importOptions
        ? importDealsCsvWithMapping(filePath, mapping, importOptions)
        : importDealsCsvWithMapping(filePath, mapping);
    }

    if (entity === 'activities') {
      const mapping = toBackendMapping<ActivityImportTargetField>(columnMapping);
      return importOptions
        ? importActivitiesCsvWithMapping(filePath, mapping, importOptions)
        : importActivitiesCsvWithMapping(filePath, mapping);
    }

    if (entity === 'notes') {
      const mapping = toBackendMapping<NoteImportTargetField>(columnMapping);
      return importOptions
        ? importNotesCsvWithMapping(filePath, mapping, importOptions)
        : importNotesCsvWithMapping(filePath, mapping);
    }

    if (entity === 'tag_definitions') {
      const mapping = toBackendMapping<TagDefinitionImportTargetField>(columnMapping);
      return importOptions
        ? importTagDefinitionsCsvWithMapping(filePath, mapping, importOptions)
        : importTagDefinitionsCsvWithMapping(filePath, mapping);
    }

    if (entity === 'custom_field_definitions') {
      const mapping = toBackendMapping<CustomFieldDefinitionImportTargetField>(columnMapping);
      return importOptions
        ? importCustomFieldDefinitionsCsvWithMapping(filePath, mapping, importOptions)
        : importCustomFieldDefinitionsCsvWithMapping(filePath, mapping);
    }

    if (entity === 'tag_links') {
      const mapping = toBackendMapping<TagLinkImportTargetField>(columnMapping);
      return importOptions
        ? importTagLinksCsvWithMapping(filePath, mapping, importOptions)
        : importTagLinksCsvWithMapping(filePath, mapping);
    }

    const mapping = toBackendMapping<OrganizationImportTargetField>(columnMapping);
    return importOptions
      ? importOrganizationsCsvWithMapping(filePath, mapping, importOptions)
      : importOrganizationsCsvWithMapping(filePath, mapping);
  }

  function duplicateAutoMergeOptions(): ImportOptions | undefined {
    if (!mergeDuplicateImportRows) {
      return undefined;
    }

    return { mergeDuplicates: mergeDuplicateImportRows };
  }

  function isTargetAssigned(target: string, currentHeader: string): boolean {
    return Object.entries(columnMapping).some(
      ([source, assignedTarget]) => source !== currentHeader && assignedTarget === target,
    );
  }

  function backFromCurrentStep() {
    if (importStep === 'summary') {
      importStep = 'confirm';
    } else if (importStep === 'confirm') {
      importStep =
        usesDuplicatePreflight && (preflightReport?.duplicate_warning_count ?? 0) > 0
          ? 'duplicates'
          : 'mapping';
    } else if (importStep === 'duplicates') {
      importStep = 'mapping';
    } else if (importStep === 'mapping') {
      importStep = 'preview';
    } else {
      importStep = 'select';
    }
  }

  function doneImport() {
    close();
    resetImportState({ keepEntity: true });
  }

  function applyImportResult(result: ImportWithBackupResult) {
    importSummary = result.import;
    importBackupPath = result.backup.backup_dir;
    importRestoreMessage = null;
    importRestoreError = null;
    importRollbackMessage = null;
    importRollbackError = null;
    importRollbackResult = null;
    importRollbackCompleted = false;
  }

  function backupActionErrorMessage(err: unknown): string {
    if (err instanceof Error && err.message.trim()) {
      return err.message;
    }

    if (typeof err === 'string' && err.trim()) {
      return err;
    }

    return t('import.preImportBackupRestoreFailed');
  }

  function rowRollbackErrorMessage(err: unknown): string {
    if (err instanceof Error && err.message.trim()) {
      return err.message;
    }

    if (typeof err === 'string' && err.trim()) {
      return err;
    }

    return t('import.rowRollbackFailed');
  }

  async function rollbackImportRows() {
    if (!importRollbackPlan || importRollbackCompleted) {
      return;
    }

    const confirmed = window.confirm(t('import.rowRollbackConfirm'));
    if (!confirmed) {
      importRollbackMessage = t('import.rowRollbackCancelled');
      importRollbackError = null;
      return;
    }

    isRollingBackImportRows = true;
    importRollbackMessage = null;
    importRollbackError = null;
    importRollbackResult = null;

    try {
      const result = await rollbackCompletedImport(importRollbackPlan);
      importRollbackResult = result;
      importRollbackCompleted = true;
      const message = t('import.rowRollbackComplete', {
        rolledBack: result.rolled_back,
        skipped: result.skipped,
        errors: result.errors.length,
      });
      importRollbackMessage = message;
      uiStore.toastSuccess(message);
    } catch (err) {
      importRollbackError = rowRollbackErrorMessage(err);
      uiStore.toastError(`${t('import.rowRollbackFailed')}: ${importRollbackError}`);
    } finally {
      isRollingBackImportRows = false;
    }
  }

  async function restorePreImportBackup() {
    if (!importBackupPath) {
      return;
    }

    isRestoringImportBackup = true;
    importRestoreMessage = null;
    importRestoreError = null;

    try {
      const validation = await validateLocalBackup(importBackupPath);
      const confirmed = window.confirm(t('import.preImportBackupRestoreConfirm'));

      if (!confirmed) {
        importRestoreMessage = t('import.preImportBackupRestoreCancelled');
        return;
      }

      const result = await restoreLocalBackupToAppData(validation.backup_dir, true);
      await reloadWorkspaceAfterDataReplace();
      const message = t('import.preImportBackupRestored', { path: result.database_path });
      importRestoreMessage = message;
      uiStore.toastSuccess(message);
    } catch (err) {
      importRestoreError = backupActionErrorMessage(err);
      uiStore.toastError(`${t('import.preImportBackupRestoreFailed')}: ${importRestoreError}`);
    } finally {
      isRestoringImportBackup = false;
    }
  }

  async function handleExport() {
    isExporting = true;
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const dialogMetadata = exportDialogMetadata[exportFormat];
      const savePath = await save({
        defaultPath: `${exportEntity}-export.${dialogMetadata.defaultExtension}`,
        filters: [
          {
            name: dialogMetadata.filterName,
            extensions: [dialogMetadata.defaultExtension],
          },
        ],
      });

      if (typeof savePath !== 'string') {
        return;
      }

      const rows = await exportData(exportEntity, exportFormat, savePath);
      uiStore.toastSuccess(`${t('export.success')} (${rows})`);
    } catch {
      uiStore.toastError(t('export.failed'));
    } finally {
      isExporting = false;
    }
  }

  function close() {
    open = false;
    onclose?.();
  }
</script>

{#if open}
  <div class="modal-backdrop" role="button" tabindex="0" onclick={(e) => { if (e.target === e.currentTarget) close(); }} onkeydown={(e) => { if (e.key === 'Escape') close(); }}>
    <div class="modal import-export-modal">
      <div class="modal-header">
        <span class="modal-title">{t('common.import')} / {t('common.export')}</span>
        <button class="icon-btn" onclick={close} type="button" aria-label={t('common.close')}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
            <path d="M12 4L4 12M4 4l8 8"/>
          </svg>
        </button>
      </div>

      <div class="tab-bar">
        <button
          class="tab"
          class:active={activeTab === 'import'}
          onclick={() => activeTab = 'import'}
          type="button"
        >
          {t('common.import')}
        </button>
        <button
          class="tab"
          class:active={activeTab === 'export'}
          onclick={() => activeTab = 'export'}
          type="button"
        >
          {t('common.export')}
        </button>
      </div>

      <div class="modal-body">
        {#if activeTab === 'import'}
          <div class="import-tab">
            <div class="form-group">
              <label class="form-label" for="import-entity">{t('common.type')}</label>
              <select id="import-entity" class="select" value={importEntity} onchange={handleImportEntityChange}>
                <option value="contacts">{t('contacts.title')}</option>
                <option value="deals">{t('deals.title')}</option>
                <option value="activities">{t('activities.title')}</option>
                <option value="organizations">{t('organizations.title')}</option>
                <option value="notes">{t('common.notes')}</option>
                <option value="tag_definitions">Tag definitions</option>
                <option value="custom_field_definitions">Custom field definitions</option>
                <option value="tag_links">Tag links</option>
                <option value="" disabled>Proposed actions (export only)</option>
                <option value="" disabled>External clients (export only)</option>
                <option value="" disabled>External client permissions (export only)</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label" for="import-format">{t('export.format')}</label>
              <select id="import-format" class="select" value={importFormat} onchange={handleImportFormatChange}>
                <option value="csv">CSV</option>
                <option value="json">JSON</option>
              </select>
            </div>

            {#if showMappingWizard}
              <ol
                class="wizard-steps"
                class:no-duplicates={!usesDuplicatePreflight}
                aria-label={t('import.wizardProgress')}
              >
                <li class:active={importStep === 'select'} class:complete={importStep !== 'select'}>{t('import.stepSelect')}</li>
                <li class:active={importStep === 'preview'} class:complete={['mapping', 'duplicates', 'confirm', 'summary'].includes(importStep)}>{t('import.stepPreview')}</li>
                <li class:active={importStep === 'mapping'} class:complete={['duplicates', 'confirm', 'summary'].includes(importStep)}>{t('import.stepMap')}</li>
                {#if usesDuplicatePreflight}
                  <li class:active={importStep === 'duplicates'} class:complete={['confirm', 'summary'].includes(importStep)}>{t('import.stepDuplicates')}</li>
                {/if}
                <li class:active={importStep === 'confirm'} class:complete={importStep === 'summary'}>{t('import.stepConfirm')}</li>
                <li class:active={importStep === 'summary'}>{t('import.stepSummary')}</li>
              </ol>
            {/if}

            {#if !showMappingWizard || importStep === 'select'}
              <div class="form-group">
                <label class="form-label" for="import-file-button">{t('import.chooseFile')}</label>
                <button id="import-file-button" class="btn btn-secondary" onclick={handleFilePick} type="button">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
                  </svg>
                  {t('import.chooseFile')}
                </button>
                <input
                  id="csv-file-input"
                  type="file"
                  accept=".csv"
                  style="display: none;"
                  onchange={handleFileInputChange}
                />
                {#if selectedImportLabel}
                  <p class="import-stats">{selectedImportLabel}</p>
                {/if}
                {#if fallbackImportBlocked}
                  <p class="validation-message">{t('import.desktopPickerRequired')}</p>
                {/if}
                {#if validationErrors.length > 0}
                  <div class="validation-list" role="alert">
                    {#each validationErrors as error (error)}
                      <p>{error}</p>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}

            {#if parseResult && (!showMappingWizard || importStep === 'preview')}
              <p class="import-stats">
                {t('import.rowCount', { count: parseResult.count })}
                {#if parseResult.warnings.length > 0}
                  <span class="import-warnings"> · {parseResult.warnings.length} {t('import.parseWarnings')}</span>
                {/if}
              </p>

              {#if previewRows.length > 0}
                <div class="preview-panel">
                  <div class="preview-title">{t('import.previewRows')}</div>
                  <div class="preview-table-wrap">
                    <table class="data-table preview-table">
                      <thead>
                        <tr>
                          {#each parseResult.headers as h (h)}
                            <th>{h}</th>
                          {/each}
                        </tr>
                      </thead>
                      <tbody>
                        {#each previewRows as row, i (i)}
                          <tr>
                            {#each parseResult.headers as h (h)}
                              <td>{row[h]}</td>
                            {/each}
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                </div>
              {/if}
            {/if}

            {#if isJsonImport && jsonPreview && importStep === 'preview'}
              <p class="import-stats">
                {t('import.rowCount', { count: jsonPreview.total_rows })}
              </p>

              {#if jsonPreviewRows.length > 0}
                <div class="preview-panel">
                  <div class="preview-title">{t('import.previewRows')}</div>
                  <div class="preview-table-wrap">
                    <table class="data-table preview-table">
                      <thead>
                        <tr>
                          <th>{t('import.row')}</th>
                          {#each jsonPreview.headers as h (h)}
                            <th>{h}</th>
                          {/each}
                        </tr>
                      </thead>
                      <tbody>
                        {#each jsonPreviewRows as row (row.row_number)}
                          <tr>
                            <td>{row.row_number}</td>
                            {#each jsonPreview.headers as h (h)}
                              <td>{row.values[h] ?? ''}</td>
                            {/each}
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                </div>
              {:else}
                <p class="empty-message">{t('import.noPreviewRows')}</p>
              {/if}
            {/if}

            {#if showMappingWizard && sourceHeaders.length > 0 && importStep === 'mapping'}
              <div class="mapping-panel">
                {#if mappingGuidanceKey}
                  <div class="mapping-guidance" role="note">
                    {t(mappingGuidanceKey)}
                  </div>
                {/if}
                <div class="mapping-header">
                  <span>{t('import.columnMapping')}</span>
                  <span>{t('import.targetField')}</span>
                </div>
                {#each sourceHeaders as h (h)}
                  <div class="mapping-row">
                    <div class="source-column">
                      <span class="source-label">{h}</span>
                      {#if sourcePreviewRows[0]?.[h]}
                        <span class="source-sample">{sourcePreviewRows[0][h]}</span>
                      {/if}
                    </div>
                    <select
                      class="select"
                      aria-label={`${t('import.mapColumn')}: ${h}`}
                      value={columnMapping[h] ?? ''}
                      onchange={(e) => updateMapping(h, (e.target as HTMLSelectElement).value || null)}
                    >
                      <option value="">{t('import.skip')}</option>
                      {#each importFieldOptions as field (field.value)}
                        <option
                          value={field.value}
                          disabled={isTargetAssigned(field.value, h)}
                        >
                          {field.label}{field.required ? ` (${t('common.required')})` : ''}
                        </option>
                      {/each}
                    </select>
                  </div>
                {/each}

                {#if validationErrors.length > 0}
                  <div class="validation-list" role="alert">
                    {#each validationErrors as error (error)}
                      <p>{error}</p>
                    {/each}
                  </div>
                {/if}

                {#if mappedPreviewRows.length > 0}
                  <div class="mapped-preview">
                    <div class="preview-title">{t('import.mappedPreview')}</div>
                    <div class="mapped-preview-list">
                      {#each mappedPreviewRows.slice(0, 3) as row, i (i)}
                        <div class="mapped-preview-row">
                          <span>{t('import.rowNumber', { number: i + 1 })}</span>
                          <code>{JSON.stringify(row)}</code>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {/if}

            {#if canAutoMergeDuplicates && ['mapping', 'duplicates', 'confirm'].includes(importStep)}
              <div class="merge-option">
                <label class="merge-option-label">
                  <input
                    type="checkbox"
                    bind:checked={mergeDuplicateImportRows}
                  />
                  <span>{t('import.mergeDuplicates')}</span>
                </label>
                <p>{t('import.mergeDuplicatesDescription')}</p>
              </div>
            {/if}

            {#if showDuplicateReview && importStep === 'duplicates'}
              <div class="duplicate-panel">
                <p class="import-stats">
                  {t('import.duplicateWarningCount', { count: preflightReport?.duplicate_warning_count ?? 0 })}
                </p>
                {#if mergeDuplicateImportRows && canAutoMergeDuplicates}
                  <p class="import-stats">{t('import.duplicateAutoMergeEnabled')}</p>
                {/if}
                {#if duplicateWarnings.length > 0}
                  <div class="preview-table-wrap duplicate-table-wrap">
                    <table class="data-table preview-table">
                      <thead>
                        <tr>
                          <th>{t('import.row')}</th>
                          <th>{t('import.matchType')}</th>
                          <th>{t('import.csvValue')}</th>
                          <th>{t('import.existingRecord')}</th>
                          <th>{t('import.reason')}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {#each duplicateWarnings as warning (warning.row_number + warning.match_type + warning.csv_value)}
                          <tr>
                            <td>{warning.row_number}</td>
                            <td>{warning.match_type}</td>
                            <td>{warning.csv_value}</td>
                            <td>{warning.existing_display_label}</td>
                            <td>{warning.reason}</td>
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                {:else}
                  <p class="empty-message">{t('import.noDuplicateWarnings')}</p>
                {/if}
              </div>
            {/if}

            {#if showMappingWizard && importStep === 'confirm'}
              <div class="confirm-panel">
                <p class="import-stats">
                  {t('import.confirmRows', { count: preflightReport?.total_rows ?? (isJsonImport ? jsonPreview?.total_rows : parseResult?.count) ?? 0 })}
                </p>
                {#if mergeDuplicateImportRows && canAutoMergeDuplicates}
                  <p class="import-stats">{t('import.confirmAutoMergeEnabled')}</p>
                {/if}
                {#if (preflightReport?.duplicate_warning_count ?? 0) > 0}
                  <p class="import-warnings">
                    {mergeDuplicateImportRows && canAutoMergeDuplicates
                      ? t('import.confirmDuplicateWarningsWithMerge', { count: preflightReport?.duplicate_warning_count ?? 0 })
                      : t('import.confirmDuplicateWarnings', { count: preflightReport?.duplicate_warning_count ?? 0 })}
                  </p>
                {/if}
                {#if !usesDuplicatePreflight}
                  <p class="import-stats">
                    {importEntity === 'activities'
                      ? t('import.activitiesSkipDuplicates')
                      : importEntity === 'notes'
                        ? t('import.notesSkipDuplicates')
                        : importEntity === 'tag_definitions'
                          ? 'Tag definition imports do not run duplicate detection. Existing tag names are skipped.'
                          : importEntity === 'custom_field_definitions'
                            ? 'Custom field definition imports do not run duplicate detection. Exact existing definitions are skipped; conflicting definitions are reported.'
                            : 'Tag link imports do not run duplicate detection. Existing active local links are skipped.'}
                  </p>
                {/if}
                {#if validationErrors.length > 0}
                  <div class="validation-list" role="alert">
                    {#each validationErrors as error (error)}
                      <p>{error}</p>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}

            {#if importStep === 'summary' && importSummary}
              <div class="summary-panel">
                <div class="summary-grid">
                  <div>
                    <span>{t('import.created')}</span>
                    <strong>{importSummary.created}</strong>
                  </div>
                  <div>
                    <span>{t('import.merged')}</span>
                    <strong>{importSummary.merged ?? 0}</strong>
                  </div>
                  <div>
                    <span>{t('import.skipped')}</span>
                    <strong>{importSummary.skipped}</strong>
                  </div>
                  <div>
                    <span>{t('import.errors')}</span>
                    <strong>{importSummary.errors.length}</strong>
                  </div>
                </div>
                {#if importSummary.errors.length > 0}
                  <div class="validation-list" role="alert">
                    {#each importSummary.errors.slice(0, 8) as error (error)}
                      <p>{error}</p>
                    {/each}
                  </div>
                {/if}
                {#if importRollbackPlan}
                  <div class="rollback-summary">
                    <div class="backup-summary-copy">
                      <span>{t('import.rowRollbackAvailable', { count: importRollbackActionCount })}</span>
                      <span class="rollback-summary-note">{t('import.rowRollbackDesc')}</span>
                    </div>
                    <button
                      class="btn btn-secondary"
                      onclick={rollbackImportRows}
                      type="button"
                      disabled={isRollingBackImportRows || importRollbackCompleted}
                    >
                      {isRollingBackImportRows ? t('import.rollingBackRows') : t('import.rollbackImportedRows')}
                    </button>
                  </div>
                  {#if importRollbackMessage}
                    <p class="backup-restore-status backup-restore-status--success">{importRollbackMessage}</p>
                  {/if}
                  {#if importRollbackError}
                    <p class="backup-restore-status backup-restore-status--error" role="alert">
                      {t('import.rowRollbackFailed')}: {importRollbackError}
                    </p>
                  {/if}
                  {#if importRollbackResult?.errors.length}
                    <div class="validation-list" role="alert">
                      {#each importRollbackResult.errors.slice(0, 6) as error (`${error.entity_type}:${error.entity_id}:${error.row_number}:${error.code}`)}
                        <p>{error.message}</p>
                      {/each}
                    </div>
                  {/if}
                {/if}
                {#if importBackupPath}
                  <div class="backup-summary">
                    <div class="backup-summary-copy">
                      <span>{t('import.preImportBackupCreated', { path: importBackupPath })}</span>
                      <span class="backup-summary-warning">{t('import.preImportBackupRestoreDesc')}</span>
                    </div>
                    <button
                      class="btn btn-danger"
                      onclick={restorePreImportBackup}
                      type="button"
                      disabled={isRestoringImportBackup}
                    >
                      {isRestoringImportBackup ? t('import.restoringBackup') : t('import.restorePreImportBackup')}
                    </button>
                  </div>
                  {#if importRestoreMessage}
                    <p class="backup-restore-status backup-restore-status--success">{importRestoreMessage}</p>
                  {/if}
                  {#if importRestoreError}
                    <p class="backup-restore-status backup-restore-status--error" role="alert">
                      {t('import.preImportBackupRestoreFailed')}: {importRestoreError}
                    </p>
                  {/if}
                {/if}
              </div>
            {/if}
          </div>
        {:else}
          <div class="export-tab">
            <div class="form-group">
              <label class="form-label" for="export-entity">{t('export.entity')}</label>
              <select id="export-entity" class="select" bind:value={exportEntity}>
                <option value="contacts">{t('contacts.title')}</option>
                <option value="deals">{t('deals.title')}</option>
                <option value="activities">{t('activities.title')}</option>
                <option value="organizations">{t('organizations.title')}</option>
                <option value="notes">{t('common.notes')}</option>
                <option value="tag_definitions">Tag definitions</option>
                <option value="custom_field_definitions">Custom field definitions</option>
                <option value="tag_links">Tag links</option>
                <option value="audit_log">Audit log</option>
                <option value="proposed_actions">Proposed actions</option>
                <option value="external_clients">External clients</option>
                <option value="external_client_permissions">External client permissions</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label" for="export-format">{t('export.format')}</label>
              <select id="export-format" class="select" bind:value={exportFormat}>
                <option value="csv">CSV</option>
                <option value="json">JSON</option>
              </select>
            </div>
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        {#if activeTab === 'import' && showMappingWizard && importStep !== 'select' && importStep !== 'summary'}
          <button class="btn btn-secondary" onclick={backFromCurrentStep} type="button" disabled={isImporting || isPreflighting}>
            {t('common.back')}
          </button>
        {:else if activeTab === 'import' && importStep === 'summary'}
          <button class="btn btn-secondary" onclick={doneImport} type="button">
            {t('common.close')}
          </button>
        {:else}
          <button class="btn btn-secondary" onclick={close} type="button">
            {t('common.cancel')}
          </button>
        {/if}

        {#if activeTab === 'import'}
          {#if showMappingWizard}
            {#if importStep === 'select'}
              <button
                class="btn btn-primary"
                onclick={goToPreview}
                disabled={!parseResult && !jsonPreview}
                type="button"
              >
                {t('common.next')}
              </button>
            {:else if importStep === 'preview'}
              <button class="btn btn-primary" onclick={goToMapping} type="button">
                {t('common.next')}
              </button>
            {:else if importStep === 'mapping'}
              <button
                class="btn btn-primary"
                onclick={handlePreflight}
                disabled={isPreviewing || isPreflighting}
                type="button"
              >
                {isPreflighting
                  ? t('import.checking')
                  : usesDuplicatePreflight
                    ? t('import.detectDuplicates')
                    : t('import.reviewImport')}
              </button>
            {:else if importStep === 'duplicates'}
              <button class="btn btn-primary" onclick={() => importStep = 'confirm'} type="button">
                {t('import.continueDespiteWarnings')}
              </button>
            {:else if importStep === 'confirm'}
              <button
                class="btn btn-primary"
                onclick={handleMappedImport}
                disabled={isImporting}
                type="button"
              >
                {isImporting ? t('import.importing') : t('import.confirmImport')}
              </button>
            {/if}
          {:else if !isMappedImport}
            <button
              class="btn btn-primary"
              onclick={handleLegacyImport}
              disabled={!selectedImportPath || isImporting}
              type="button"
            >
              {isImporting ? t('import.importing') : t('import.importButton')}
            </button>
          {/if}
        {:else}
          <button
            class="btn btn-primary"
            onclick={handleExport}
            disabled={isExporting}
            type="button"
          >
            {isExporting ? t('export.exporting') : t('export.exportButton')}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .import-export-modal {
    width: min(760px, calc(100vw - 32px));
  }

  .import-tab,
  .export-tab {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .wizard-steps {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: var(--space-2);
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .wizard-steps.no-duplicates {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }

  .wizard-steps li {
    border-bottom: 2px solid var(--border-default);
    color: var(--text-muted);
    font-size: var(--text-xs);
    line-height: 1.3;
    min-height: 34px;
    padding-bottom: var(--space-2);
  }

  .wizard-steps li.active {
    border-color: var(--border-focus);
    color: var(--text-primary);
    font-weight: var(--weight-medium);
  }

  .wizard-steps li.complete {
    border-color: var(--text-success);
    color: var(--text-secondary);
  }

  .import-stats,
  .empty-message {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .import-warnings {
    color: var(--text-warning);
    font-size: var(--text-sm);
  }

  .merge-option {
    background-color: var(--surface-hover);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
  }

  .merge-option-label {
    align-items: center;
    color: var(--text-primary);
    display: flex;
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    gap: var(--space-2);
  }

  .merge-option p {
    font-size: var(--text-xs);
    margin: 0;
  }

  .backup-summary {
    align-items: flex-start;
    background-color: var(--surface-hover);
    border-left: 3px solid var(--color-primary);
    color: var(--text-secondary);
    display: flex;
    gap: var(--space-3);
    justify-content: space-between;
    font-size: var(--text-sm);
    margin: 0;
    overflow-wrap: anywhere;
    padding: var(--space-3);
  }

  .backup-summary-copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
  }

  .backup-summary-warning {
    color: var(--text-danger);
    font-size: var(--text-xs);
  }

  .rollback-summary {
    align-items: flex-start;
    background-color: var(--surface-hover);
    border-left: 3px solid var(--text-success);
    color: var(--text-secondary);
    display: flex;
    gap: var(--space-3);
    justify-content: space-between;
    font-size: var(--text-sm);
    margin: 0;
    overflow-wrap: anywhere;
    padding: var(--space-3);
  }

  .rollback-summary-note {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .backup-restore-status {
    border-left: 3px solid currentColor;
    font-size: var(--text-sm);
    margin: 0;
    padding: var(--space-3);
  }

  .backup-restore-status--success {
    background-color: var(--surface-hover);
    color: var(--text-success);
  }

  .backup-restore-status--error {
    background-color: var(--surface-hover);
    color: var(--text-danger);
  }

  .validation-message {
    color: var(--text-danger);
    font-size: var(--text-sm);
  }

  .preview-panel,
  .mapping-panel,
  .duplicate-panel,
  .confirm-panel,
  .summary-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .preview-title {
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
  }

  .preview-table-wrap {
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    max-height: 220px;
    overflow: auto;
  }

  .duplicate-table-wrap {
    max-height: 260px;
  }

  .preview-table {
    min-width: 100%;
  }

  .preview-table th,
  .preview-table td {
    font-size: var(--text-xs);
    padding: var(--space-2) var(--space-3);
    white-space: nowrap;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mapping-header,
  .mapping-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(180px, 240px);
    gap: var(--space-4);
    align-items: center;
  }

  .mapping-header {
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    text-transform: uppercase;
  }

  .mapping-guidance {
    padding: var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-sm);
    background: var(--surface-hover);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  .mapping-row {
    border-top: var(--border-width) solid var(--border-default);
    padding-top: var(--space-3);
  }

  .source-column {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }

  .source-label {
    color: var(--text-primary);
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-sample {
    color: var(--text-muted);
    font-size: var(--text-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .validation-list {
    background-color: var(--surface-hover);
    border-left: 3px solid var(--text-danger);
    color: var(--text-danger);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
  }

  .validation-list p {
    font-size: var(--text-sm);
    margin: 0;
  }

  .mapped-preview {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .mapped-preview-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .mapped-preview-row {
    align-items: center;
    display: grid;
    gap: var(--space-3);
    grid-template-columns: 64px minmax(0, 1fr);
  }

  .mapped-preview-row span,
  .mapped-preview-row code {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .mapped-preview-row code {
    background-color: var(--surface-hover);
    border-radius: var(--border-radius-sm);
    overflow: hidden;
    padding: var(--space-2);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .summary-grid {
    display: grid;
    gap: var(--space-3);
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .summary-grid div {
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-3);
  }

  .summary-grid span {
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .summary-grid strong {
    color: var(--text-primary);
    font-size: var(--text-lg);
  }

  @media (max-width: 720px) {
    .wizard-steps {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .mapping-header,
    .mapping-row {
      grid-template-columns: 1fr;
      gap: var(--space-2);
    }

    .summary-grid {
      grid-template-columns: 1fr;
    }

    .backup-summary,
    .rollback-summary {
      flex-direction: column;
    }
  }
</style>
