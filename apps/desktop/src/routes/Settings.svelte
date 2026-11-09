<script lang="ts">
  /**
   * Settings.svelte — Application settings page for 900CRM.
   *
   * Three panes:
   *   Appearance — language, currency, theme, date format, reminders, about
   *   Data — import/export and local backup/restore
   *   Integrations — sync honesty, optional email probe, external clients
   *
   * Each setting auto-saves via settingsStore.updateSetting().
   * No separate Save button needed — changes apply immediately.
   */

  import { t, type TranslationKeys } from '$lib/i18n';
  import { settingsStore } from '$lib/stores/settings';
  import { availableLocales } from '$lib/i18n';
  import { uiStore } from '$lib/stores/ui';
  import type { AppSettings } from '$lib/api/settings';
  import {
    createLocalBackup,
    restoreLocalBackupToAppData,
    validateLocalBackup,
    type LocalBackupValidation,
  } from '$lib/api/backup';
  import { reloadWorkspaceAfterDataReplace } from '$lib/stores/reloadWorkspace';
  import {
    createExternalClientPlaceholder,
    listExternalClients,
    updateExternalClientActivation,
    type EditableExternalClientPermissionMode,
    type ExternalClient,
  } from '$lib/api/externalClients';
  import { testEmailServerConnection } from '$lib/api/email';
  import { currentHashPath, navigateHash } from '$lib/utils/hashRouter';
  import { parseSettingsHash, settingsPathForPane } from '$lib/utils/settingsHash';
  import ExternalClientPermissions from '$lib/components/ExternalClientPermissions.svelte';
  import ImportExport from '$lib/components/ImportExport.svelte';

  // ── Types ────────────────────────────────────────────────────────────────────

  type ThemeOption = 'light' | 'dark' | 'system';
  type DateFormat  = 'YYYY-MM-DD' | 'DD/MM/YYYY' | 'MM/DD/YYYY' | 'MMM D, YYYY';
  type ExternalClientActivationMode = EditableExternalClientPermissionMode;

  type SettingsPane = 'appearance' | 'data' | 'integrations';

  interface SettingsShortcut {
    id: SettingsPane;
    labelKey: TranslationKeys;
    helpKey: TranslationKeys;
  }

  // ── Constants ────────────────────────────────────────────────────────────────

  const CURRENCIES = [
    { code: 'USD', label: 'USD — US Dollar' },
    { code: 'EUR', label: 'EUR — Euro' },
    { code: 'GBP', label: 'GBP — British Pound' },
    { code: 'JPY', label: 'JPY — Japanese Yen' },
    { code: 'KES', label: 'KES — Kenyan Shilling' },
    { code: 'NGN', label: 'NGN — Nigerian Naira' },
    { code: 'ZAR', label: 'ZAR — South African Rand' },
    { code: 'INR', label: 'INR — Indian Rupee' },
    { code: 'BRL', label: 'BRL — Brazilian Real' },
    { code: 'MXN', label: 'MXN — Mexican Peso' },
    { code: 'EGP', label: 'EGP — Egyptian Pound' },
    { code: 'SAR', label: 'SAR — Saudi Riyal' },
    { code: 'AED', label: 'AED — UAE Dirham' },
    { code: 'CAD', label: 'CAD — Canadian Dollar' },
    { code: 'AUD', label: 'AUD — Australian Dollar' },
  ];

  const THEME_OPTIONS: { value: ThemeOption; labelKey: string }[] = [
    { value: 'light',  labelKey: 'settings.themeLight' },
    { value: 'dark',   labelKey: 'settings.themeDark' },
    { value: 'system', labelKey: 'settings.themeSystem' },
  ];

  const DATE_FORMATS: { value: DateFormat; example: string }[] = [
    { value: 'MMM D, YYYY',  example: 'Mar 5, 2026' },
    { value: 'MM/DD/YYYY',   example: '03/05/2026' },
    { value: 'DD/MM/YYYY',   example: '05/03/2026' },
    { value: 'YYYY-MM-DD',   example: '2026-03-05' },
  ];

  const EXTERNAL_CLIENT_ACTIVATION_MODES: {
    value: ExternalClientActivationMode;
    labelKey: string;
  }[] = [
    { value: 'disabled', labelKey: 'settings.externalClientActivationModeDisabled' },
    { value: 'read_only', labelKey: 'settings.externalClientActivationModeReadOnly' },
    { value: 'draft_only', labelKey: 'settings.externalClientActivationModeDraftOnly' },
  ];

  const SETTINGS_SHORTCUTS: SettingsShortcut[] = [
    { id: 'appearance', labelKey: 'settings.sectionAppearance', helpKey: 'settings.paneAppearanceHelp' },
    { id: 'data', labelKey: 'settings.sectionData', helpKey: 'settings.paneDataHelp' },
    { id: 'integrations', labelKey: 'settings.sectionIntegrations', helpKey: 'settings.paneIntegrationsHelp' },
  ];

  // ── State ────────────────────────────────────────────────────────────────────

  /** Tracks which setting key is currently being saved (for per-row spinner). */
  let savingKey = $state<keyof AppSettings | null>(null);

  /** URL field local state — only committed on blur or Enter. */

  let reminderLeadMinutesLocal = $state('30');
  let reminderLeadDirty = $state(false);
  let smtpHostLocal = $state('');
  let smtpPortLocal = $state('587');
  let smtpUsernameLocal = $state('');
  let smtpFromLocal = $state('');
  let imapHostLocal = $state('');
  let imapPortLocal = $state('993');
  let imapUsernameLocal = $state('');
  let emailFieldDirty = $state<Record<string, boolean>>({});
  let smtpTestLoading = $state(false);
  let imapTestLoading = $state(false);
  let smtpTestMessage = $state<string | null>(null);
  let imapTestMessage = $state<string | null>(null);

  let showImportExport = $state(false);
  let backupDirLocal = $state('');
  let backupBusy = $state<null | 'select' | 'create' | 'validate' | 'restore'>(null);
  let backupMessage = $state<string | null>(null);
  let backupError = $state<string | null>(null);
  let lastBackupValidation = $state<LocalBackupValidation | null>(null);
  let externalClients = $state<ExternalClient[]>([]);
  let externalClientsLoading = $state(false);
  let externalClientsError = $state<string | null>(null);
  let externalClientCreateMessage = $state<string | null>(null);
  let externalClientCreateError = $state<string | null>(null);
  let externalClientCreateLoading = $state(false);
  let externalClientName = $state('');
  let externalClientType = $state('');
  let externalClientsListRequestSeq = 0;
  let externalClientsMutationSeq = 0;
  let externalClientActivationModes = $state<Record<string, ExternalClientActivationMode>>({});
  let externalClientActivationSaving = $state<Record<string, boolean>>({});
  let externalClientActivationMessages = $state<Record<string, string>>({});
  let externalClientActivationErrors = $state<Record<string, string>>({});
  let activePane = $state<SettingsPane>('appearance');
  let settingsBootstrapped = false;
  let integrationsLoaded = false;

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  $effect(() => {
    if (settingsBootstrapped) {
      return;
    }

    settingsBootstrapped = true;
    reminderLeadMinutesLocal = String(settingsStore.reminderLeadMinutes);
    smtpHostLocal = settingsStore.smtpHost;
    smtpPortLocal = String(settingsStore.smtpPort);
    smtpUsernameLocal = settingsStore.smtpUsername;
    smtpFromLocal = settingsStore.smtpFrom;
    imapHostLocal = settingsStore.imapHost;
    imapPortLocal = String(settingsStore.imapPort);
    imapUsernameLocal = settingsStore.imapUsername;
  });

  let settingsHashBound = false;
  $effect(() => {
    if (settingsHashBound || typeof window === 'undefined') {
      return;
    }

    settingsHashBound = true;
    applySettingsHash(currentHashPath());
    window.addEventListener('hashchange', () => applySettingsHash(currentHashPath()));
  });

  $effect(() => {
    if (activePane !== 'integrations' || integrationsLoaded) {
      return;
    }

    integrationsLoaded = true;
    void loadExternalClients();
  });

  // ── Handlers ─────────────────────────────────────────────────────────────────

  /**
   * Generic setting updater — shows per-key saving indicator.
   */
  async function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    savingKey = key;
    try {
      await settingsStore.updateSetting(key, value);
      uiStore.toastSuccess(t('settings.settingsSaved'));
    } catch {
      // already toasted by store
    } finally {
      savingKey = null;
    }
  }

  function applySettingsHash(path: string) {
    const parsed = parseSettingsHash(path);
    activePane = parsed.pane;

    if (parsed.openImport) {
      showImportExport = true;
      if (currentHashPath().replace(/\/+$/, '') !== settingsPathForPane('data')) {
        navigateHash(settingsPathForPane('data'));
      }
    }
  }

  function selectSettingsPane(pane: SettingsPane) {
    activePane = pane;
    const nextPath = settingsPathForPane(pane);
    if (currentHashPath().replace(/\/+$/, '') !== nextPath) {
      navigateHash(nextPath);
    }
  }

  function handleSettingsTabKeydown(event: KeyboardEvent) {
    if (event.key !== 'ArrowRight' && event.key !== 'ArrowLeft' && event.key !== 'Home' && event.key !== 'End') {
      return;
    }

    const panes = SETTINGS_SHORTCUTS.map((shortcut) => shortcut.id);
    const current = panes.indexOf(activePane);
    let next = current;

    if (event.key === 'ArrowRight') {
      next = (current + 1) % panes.length;
    } else if (event.key === 'ArrowLeft') {
      next = (current - 1 + panes.length) % panes.length;
    } else if (event.key === 'Home') {
      next = 0;
    } else {
      next = panes.length - 1;
    }

    event.preventDefault();
    selectSettingsPane(panes[next]);
    queueMicrotask(() => {
      document.getElementById(`settings-tab-${panes[next]}`)?.focus();
    });
  }

  async function handleLanguageChange(code: string) {
    await updateSetting('language', code);
  }

  async function handleCurrencyChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    await updateSetting('currency', select.value);
  }

  async function handleThemeChange(theme: ThemeOption) {
    await updateSetting('theme', theme);
  }

  async function handleDateFormatChange(format: string) {
    await updateSetting('dateFormat', format);
  }

  async function handleNotificationsToggle() {
    await updateSetting('notificationsEnabled', !settingsStore.notificationsEnabled);
  }

  function normalizeLeadMinutes(raw: string): number {
    const parsed = Number.parseInt(raw, 10);
    if (!Number.isFinite(parsed)) return 30;
    return Math.min(1440, Math.max(1, parsed));
  }

  function handleReminderLeadInput(e: Event) {
    reminderLeadMinutesLocal = (e.target as HTMLInputElement).value;
    reminderLeadDirty = true;
  }

  async function handleReminderLeadCommit() {
    if (!reminderLeadDirty) return;
    reminderLeadDirty = false;
    const next = normalizeLeadMinutes(reminderLeadMinutesLocal);
    reminderLeadMinutesLocal = String(next);
    await updateSetting('reminderLeadMinutes', next);
  }

  async function handleReminderLeadKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      (e.target as HTMLInputElement).blur();
      await handleReminderLeadCommit();
    }
  }

  async function handleEmailIntegrationToggle() {
    await updateSetting('emailIntegrationEnabled', !settingsStore.emailIntegrationEnabled);
  }

  function markEmailFieldDirty(key: string) {
    emailFieldDirty = { ...emailFieldDirty, [key]: true };
  }

  function clearEmailFieldDirty(key: string) {
    emailFieldDirty = { ...emailFieldDirty, [key]: false };
  }

  function parsePort(raw: string, fallback: number): number {
    const parsed = Number.parseInt(raw, 10);
    if (!Number.isFinite(parsed)) return fallback;
    return Math.min(65535, Math.max(1, parsed));
  }

  async function commitEmailStringSetting<K extends keyof AppSettings>(
    key: K,
    value: string,
    trim = true,
  ) {
    if (!emailFieldDirty[String(key)]) return;
    clearEmailFieldDirty(String(key));
    const normalized = trim ? value.trim() : value;
    await updateSetting(key, normalized as AppSettings[K]);
  }

  async function commitEmailPortSetting<K extends keyof AppSettings>(
    key: K,
    value: string,
    fallback: number,
    setLocal: (next: string) => void,
  ) {
    if (!emailFieldDirty[String(key)]) return;
    clearEmailFieldDirty(String(key));
    const normalized = parsePort(value, fallback);
    setLocal(String(normalized));
    await updateSetting(key, normalized as AppSettings[K]);
  }

  async function testSmtpConnection() {
    const host = smtpHostLocal.trim();
    const port = parsePort(smtpPortLocal, 587);
    if (!host) {
      smtpTestMessage = t('settings.emailTestFailed');
      return;
    }

    smtpPortLocal = String(port);
    smtpTestLoading = true;
    smtpTestMessage = null;
    try {
      const result = await testEmailServerConnection({
        protocol: 'smtp',
        host,
        port,
      });
      smtpTestMessage = result.success
        ? `${t('settings.emailTestSuccess')} (${result.latencyMs}ms)`
        : `${t('settings.emailTestFailed')}: ${result.details}`;
    } catch (err) {
      console.error('[Settings] SMTP test failed:', err);
      smtpTestMessage = t('settings.emailTestFailed');
    } finally {
      smtpTestLoading = false;
    }
  }

  async function testImapConnection() {
    const host = imapHostLocal.trim();
    const port = parsePort(imapPortLocal, 993);
    if (!host) {
      imapTestMessage = t('settings.emailTestFailed');
      return;
    }

    imapPortLocal = String(port);
    imapTestLoading = true;
    imapTestMessage = null;
    try {
      const result = await testEmailServerConnection({
        protocol: 'imap',
        host,
        port,
      });
      imapTestMessage = result.success
        ? `${t('settings.emailTestSuccess')} (${result.latencyMs}ms)`
        : `${t('settings.emailTestFailed')}: ${result.details}`;
    } catch (err) {
      console.error('[Settings] IMAP test failed:', err);
      imapTestMessage = t('settings.emailTestFailed');
    } finally {
      imapTestLoading = false;
    }
  }

  // ── Data management ──────────────────────────────────────────────────────────

  async function handleExportAll() {
    showImportExport = true;
  }

  function handleImportData() {
    showImportExport = true;
  }

  function selectedBackupIsValidated() {
    return backupDirLocal.trim() !== '' && lastBackupValidation?.backup_dir === backupDirLocal;
  }

  function clearBackupFeedback() {
    backupMessage = null;
    backupError = null;
  }

  function setSelectedBackupDir(path: string) {
    backupDirLocal = path;
    lastBackupValidation = null;
    clearBackupFeedback();
  }

  function selectedDialogPath(value: string | string[] | null): string | null {
    if (Array.isArray(value)) {
      return value[0] ?? null;
    }
    return value;
  }

  function requireBackupDir(): string | null {
    const path = backupDirLocal.trim();
    if (!path) {
      backupError = t('settings.backupMissingFolder');
      uiStore.toastError(t('settings.backupMissingFolder'));
      return null;
    }
    return path;
  }

  function backupErrorMessage(err: unknown): string {
    if (err instanceof Error && err.message.trim()) {
      return err.message;
    }
    if (typeof err === 'string' && err.trim()) {
      return err;
    }
    return t('settings.backupFailed');
  }

  function externalClientErrorMessage(err: unknown): string {
    if (err instanceof Error && err.message.trim()) {
      return err.message;
    }
    if (typeof err === 'string' && err.trim()) {
      return err;
    }
    return t('settings.externalClientsCreateFailed');
  }

  function formatExternalClientTimestamp(value: string): string {
    if (!value) return t('common.none');
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return value;

    return new Intl.DateTimeFormat(settingsStore.language, {
      dateStyle: 'medium',
      timeStyle: 'short',
    }).format(parsed);
  }

  function activationModeForClient(client: ExternalClient): ExternalClientActivationMode {
    if (!client.enabled) return 'disabled';
    if (client.permissionMode === 'read_only' || client.permissionMode === 'draft_only') {
      return client.permissionMode;
    }
    return 'disabled';
  }

  function activationDraftModeForClient(client: ExternalClient): ExternalClientActivationMode {
    return externalClientActivationModes[client.id] ?? activationModeForClient(client);
  }

  function activationDraftMatchesClient(client: ExternalClient): boolean {
    const mode = activationDraftModeForClient(client);
    return client.enabled === (mode !== 'disabled') && client.permissionMode === mode;
  }

  function resetExternalClientActivationDrafts(clients: ExternalClient[]) {
    externalClientActivationModes = Object.fromEntries(
      clients.map((client) => [client.id, activationModeForClient(client)]),
    );
  }

  function setExternalClientActivationMessage(clientId: string, message: string | null) {
    const next = { ...externalClientActivationMessages };
    delete next[clientId];
    externalClientActivationMessages = message ? { ...next, [clientId]: message } : next;
  }

  function setExternalClientActivationError(clientId: string, message: string | null) {
    const next = { ...externalClientActivationErrors };
    delete next[clientId];
    externalClientActivationErrors = message ? { ...next, [clientId]: message } : next;
  }

  function setExternalClientActivationSaving(clientId: string, saving: boolean) {
    const next = { ...externalClientActivationSaving };
    delete next[clientId];
    externalClientActivationSaving = saving ? { ...next, [clientId]: true } : next;
  }

  function handleExternalClientActivationModeChange(clientId: string, e: Event) {
    externalClientActivationModes = {
      ...externalClientActivationModes,
      [clientId]: (e.target as HTMLSelectElement).value as ExternalClientActivationMode,
    };
    setExternalClientActivationMessage(clientId, null);
    setExternalClientActivationError(clientId, null);
  }

  async function loadExternalClients() {
    const requestSeq = ++externalClientsListRequestSeq;
    const mutationSeqAtStart = externalClientsMutationSeq;
    externalClientsLoading = true;
    externalClientsError = null;
    try {
      const clients = await listExternalClients();
      if (requestSeq === externalClientsListRequestSeq && mutationSeqAtStart === externalClientsMutationSeq) {
        externalClients = clients;
        resetExternalClientActivationDrafts(clients);
      }
    } catch (err) {
      if (requestSeq === externalClientsListRequestSeq && mutationSeqAtStart === externalClientsMutationSeq) {
        externalClientsError = externalClientErrorMessage(err);
      }
    } finally {
      if (requestSeq === externalClientsListRequestSeq) {
        externalClientsLoading = false;
      }
    }
  }

  function handleExternalClientNameInput(e: Event) {
    externalClientName = (e.target as HTMLInputElement).value;
    externalClientCreateMessage = null;
    externalClientCreateError = null;
  }

  function handleExternalClientTypeInput(e: Event) {
    externalClientType = (e.target as HTMLInputElement).value;
    externalClientCreateMessage = null;
    externalClientCreateError = null;
  }

  async function handleCreateExternalClientPlaceholder() {
    const name = externalClientName.trim();
    const clientType = externalClientType.trim();
    if (!name || !clientType || externalClientCreateLoading) return;

    externalClientCreateLoading = true;
    externalClientCreateMessage = null;
    externalClientCreateError = null;
    try {
      const created = await createExternalClientPlaceholder(name, clientType);
      externalClientsMutationSeq += 1;
      externalClients = [created, ...externalClients.filter((client) => client.id !== created.id)];
      resetExternalClientActivationDrafts(externalClients);
      externalClientsError = null;
      externalClientName = '';
      externalClientType = '';
      externalClientCreateMessage = t('settings.externalClientsCreateSuccess', { name: created.name });
      uiStore.toastSuccess(t('settings.externalClientsCreateSuccess', { name: created.name }));
    } catch (err) {
      externalClientCreateError = externalClientErrorMessage(err);
      uiStore.toastError(`${t('settings.externalClientsCreateFailed')}: ${externalClientCreateError}`);
    } finally {
      externalClientCreateLoading = false;
    }
  }

  async function handleUpdateExternalClientActivation(client: ExternalClient) {
    if (externalClientActivationSaving[client.id]) return;

    const permissionMode = activationDraftModeForClient(client);
    setExternalClientActivationSaving(client.id, true);
    setExternalClientActivationMessage(client.id, null);
    setExternalClientActivationError(client.id, null);
    try {
      const updated = await updateExternalClientActivation({
        clientId: client.id,
        enabled: permissionMode !== 'disabled',
        permissionMode,
      });
      externalClientsMutationSeq += 1;
      externalClients = externalClients.map((existing) =>
        existing.id === updated.id ? updated : existing,
      );
      externalClientActivationModes = {
        ...externalClientActivationModes,
        [updated.id]: activationModeForClient(updated),
      };
      const message = t('settings.externalClientActivationSaveSuccess', { name: updated.name });
      setExternalClientActivationMessage(updated.id, message);
      uiStore.toastSuccess(message);
    } catch (err) {
      const error = externalClientErrorMessage(err);
      setExternalClientActivationError(client.id, error);
      uiStore.toastError(`${t('settings.externalClientActivationSaveFailed')}: ${error}`);
    } finally {
      setExternalClientActivationSaving(client.id, false);
    }
  }

  async function handleChooseBackupFolder() {
    backupBusy = 'select';
    try {
      const { open: openDialog } = await import('@tauri-apps/plugin-dialog');
      const selected = selectedDialogPath(await openDialog({
        directory: true,
        multiple: false,
        title: t('settings.backupDirectoryPickerTitle'),
      }));

      if (selected) {
        setSelectedBackupDir(selected);
      }
    } catch (err) {
      backupError = backupErrorMessage(err);
      uiStore.toastError(`${t('settings.backupFailed')}: ${backupError}`);
    } finally {
      backupBusy = null;
    }
  }

  async function handleCreateBackup() {
    const backupDir = requireBackupDir();
    if (!backupDir) return;

    backupBusy = 'create';
    clearBackupFeedback();
    lastBackupValidation = null;
    try {
      const backup = await createLocalBackup(backupDir);
      backupMessage = t('settings.backupCreated', { path: backup.backup_dir });
      uiStore.toastSuccess(t('settings.backupCreated', { path: backup.backup_dir }));
    } catch (err) {
      backupError = backupErrorMessage(err);
      uiStore.toastError(`${t('settings.backupFailed')}: ${backupError}`);
    } finally {
      backupBusy = null;
    }
  }

  async function validateSelectedBackup(backupDir: string): Promise<LocalBackupValidation> {
    const validation = await validateLocalBackup(backupDir);
    lastBackupValidation = validation;
    backupMessage = t('settings.backupValidated', { path: validation.backup_dir });
    return validation;
  }

  async function handleValidateBackup() {
    const backupDir = requireBackupDir();
    if (!backupDir) return;

    backupBusy = 'validate';
    clearBackupFeedback();
    try {
      const validation = await validateSelectedBackup(backupDir);
      uiStore.toastSuccess(t('settings.backupValidated', { path: validation.backup_dir }));
    } catch (err) {
      lastBackupValidation = null;
      backupError = backupErrorMessage(err);
      uiStore.toastError(`${t('settings.backupFailed')}: ${backupError}`);
    } finally {
      backupBusy = null;
    }
  }

  async function handleRestoreBackup() {
    const backupDir = requireBackupDir();
    if (!backupDir) return;

    backupBusy = 'restore';
    clearBackupFeedback();
    let restoreRevalidationComplete = false;
    try {
      const validation = await validateSelectedBackup(backupDir);
      restoreRevalidationComplete = true;
      const confirmed = window.confirm(t('settings.backupRestoreConfirm'));
      if (!confirmed) {
        backupMessage = t('settings.backupRestoreCancelled');
        return;
      }

      const result = await restoreLocalBackupToAppData(validation.backup_dir, true);
      await reloadWorkspaceAfterDataReplace();
      backupMessage = t('settings.backupRestored', { path: result.database_path });
      uiStore.toastSuccess(t('settings.backupRestored', { path: result.database_path }));
    } catch (err) {
      if (!restoreRevalidationComplete) {
        lastBackupValidation = null;
      }
      backupError = backupErrorMessage(err);
      uiStore.toastError(`${t('settings.backupFailed')}: ${backupError}`);
    } finally {
      backupBusy = null;
    }
  }
</script>

<div class="page-content settings-page">
  <div class="page-header">
    <h1 class="page-title">{t('settings.title')}</h1>
  </div>

  <div
    class="settings-section-nav"
    role="tablist"
    aria-label={t('settings.sectionNavLabel')}
  >
    {#each SETTINGS_SHORTCUTS as shortcut (shortcut.id)}
      <button
        class="settings-section-nav-button"
        class:settings-section-nav-button--active={activePane === shortcut.id}
        type="button"
        role="tab"
        id="settings-tab-{shortcut.id}"
        aria-selected={activePane === shortcut.id}
        aria-controls="settings-pane-{shortcut.id}"
        tabindex={activePane === shortcut.id ? 0 : -1}
        onclick={() => selectSettingsPane(shortcut.id)}
        onkeydown={handleSettingsTabKeydown}
      >
        {t(shortcut.labelKey)}
      </button>
    {/each}
  </div>
  <p class="settings-pane-help">
    {t(SETTINGS_SHORTCUTS.find((shortcut) => shortcut.id === activePane)?.helpKey ?? 'settings.paneAppearanceHelp')}
  </p>

  <div
    class="settings-grid"
    class:settings-grid--single={activePane === 'data'}
    id="settings-pane-{activePane}"
    role="tabpanel"
    aria-labelledby="settings-tab-{activePane}"
  >

    <!-- ── LEFT: pane body ──────────────────────────────────────────────────── -->
    <div class="settings-main">

      {#if activePane === 'appearance'}
      <!-- Language -->
      <section class="card settings-section" aria-labelledby="lang-heading">
        <div class="card-header">
          <h2 class="section-title" id="lang-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z"/>
            </svg>
            {t('settings.language')}
          </h2>
          {#if savingKey === 'language'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body">
          <div class="locale-grid" role="radiogroup" aria-labelledby="lang-heading">
            {#each availableLocales as locale (locale.code)}
              <button
                class="locale-option"
                class:locale-option--active={settingsStore.language === locale.code}
                onclick={() => handleLanguageChange(locale.code)}
                role="radio"
                aria-checked={settingsStore.language === locale.code}
                type="button"
              >
                <span class="locale-native">{locale.nativeName}</span>
                <span class="locale-name">{locale.name}</span>
                {#if settingsStore.language === locale.code}
                  <span class="locale-check" aria-hidden="true">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                      <polyline points="20 6 9 17 4 12"/>
                    </svg>
                  </span>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      </section>

      <!-- Currency -->
      <section class="card settings-section" aria-labelledby="currency-heading">
        <div class="card-header">
          <h2 class="section-title" id="currency-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6"/>
            </svg>
            {t('settings.currency')}
          </h2>
          {#if savingKey === 'currency'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body">
          <div class="field-row">
            <label class="field-label" for="currency-select">{t('settings.currency')}</label>
            <select
              id="currency-select"
              class="input select-input"
              value={settingsStore.currency}
              onchange={handleCurrencyChange}
            >
              {#each CURRENCIES as c (c.code)}
                <option value={c.code}>{c.label}</option>
              {/each}
            </select>
          </div>
        </div>
      </section>

      <!-- Theme -->
      <section class="card settings-section" aria-labelledby="theme-heading">
        <div class="card-header">
          <h2 class="section-title" id="theme-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
            </svg>
            {t('settings.theme')}
          </h2>
          {#if savingKey === 'theme'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body">
          <div class="theme-options" role="radiogroup" aria-labelledby="theme-heading">
            {#each THEME_OPTIONS as opt (opt.value)}
              <button
                class="theme-option"
                class:theme-option--active={settingsStore.theme === opt.value}
                onclick={() => handleThemeChange(opt.value)}
                role="radio"
                aria-checked={settingsStore.theme === opt.value}
                type="button"
              >
                <!-- Theme icon -->
                {#if opt.value === 'light'}
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/>
                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                    <line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/>
                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                  </svg>
                {:else if opt.value === 'dark'}
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/>
                  </svg>
                {:else}
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>
                  </svg>
                {/if}
                <span>{t(opt.labelKey)}</span>
              </button>
            {/each}
          </div>
        </div>
      </section>

      <!-- Date format -->
      <section class="card settings-section" aria-labelledby="dateformat-heading">
        <div class="card-header">
          <h2 class="section-title" id="dateformat-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
            {t('settings.dateFormat')}
          </h2>
          {#if savingKey === 'dateFormat'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body">
          <div class="date-format-options" role="radiogroup" aria-labelledby="dateformat-heading">
            {#each DATE_FORMATS as fmt (fmt.value)}
              <button
                class="date-format-option"
                class:date-format-option--active={settingsStore.dateFormat === fmt.value}
                onclick={() => handleDateFormatChange(fmt.value)}
                role="radio"
                aria-checked={settingsStore.dateFormat === fmt.value}
                type="button"
              >
                <code class="format-code">{fmt.value}</code>
                <span class="format-example">{fmt.example}</span>
              </button>
            {/each}
          </div>
        </div>
      </section>

      <!-- Notification reminders -->
      <section class="card settings-section" aria-labelledby="notifications-heading">
        <div class="card-header">
          <h2 class="section-title" id="notifications-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <path d="M15 17h5l-1.4-1.4A2 2 0 0118 14.2V11a6 6 0 10-12 0v3.2a2 2 0 01-.6 1.4L4 17h5m6 0a3 3 0 01-6 0"/>
            </svg>
            {t('settings.notifications')}
          </h2>
          {#if savingKey === 'notificationsEnabled' || savingKey === 'reminderLeadMinutes'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body sync-body">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">{t('settings.notificationsEnabled')}</span>
              <span class="toggle-desc">
                {settingsStore.notificationsEnabled
                  ? t('common.success')
                  : t('common.none')}
              </span>
            </div>
            <button
              class="toggle-switch"
              class:toggle-switch--on={settingsStore.notificationsEnabled}
              onclick={handleNotificationsToggle}
              role="switch"
              aria-checked={settingsStore.notificationsEnabled}
              aria-label={t('settings.notificationsEnabled')}
              type="button"
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          {#if settingsStore.notificationsEnabled}
            <div class="field-row sync-url-row">
              <label class="field-label" for="reminder-lead-minutes">{t('settings.reminderLeadMinutes')}</label>
              <div class="sync-url-input-wrap">
                <input
                  id="reminder-lead-minutes"
                  class="input"
                  type="number"
                  min="1"
                  max="1440"
                  step="1"
                  value={reminderLeadMinutesLocal}
                  oninput={handleReminderLeadInput}
                  onblur={handleReminderLeadCommit}
                  onkeydown={handleReminderLeadKeydown}
                />
              </div>
              <span class="field-hint">{t('settings.reminderLeadHint')}</span>
            </div>
          {/if}
        </div>
      </section>
      {/if}

      {#if activePane === 'integrations'}
      <!-- Sync configuration -->
      <section class="card settings-section" aria-labelledby="sync-heading">
        <div class="card-header">
          <h2 class="section-title" id="sync-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
              <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
            </svg>
            {t('settings.sync')}
          </h2>

        </div>
        <div class="card-body sync-body">
          <div class="toggle-info">
            <span class="toggle-label">{t('settings.syncUnavailable')}</span>
            <span class="toggle-desc">{t('settings.syncUnavailableHint')}</span>
          </div>
        </div>
      </section>

      <!-- Email integration -->
      <section class="card settings-section email-probe-pane" aria-labelledby="email-heading">
        <div class="card-header">
          <h2 class="section-title" id="email-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1L7 17M17 7l2.1-2.1"/>
            </svg>
            {t('settings.emailIntegration')}
            <span class="probe-badge">{t('settings.emailProbeBadge')}</span>
          </h2>
          {#if savingKey === 'emailIntegrationEnabled'
            || savingKey === 'smtpHost'
            || savingKey === 'smtpPort'
            || savingKey === 'smtpUsername'
            || savingKey === 'smtpFrom'
            || savingKey === 'imapHost'
            || savingKey === 'imapPort'
            || savingKey === 'imapUsername'}
            <span class="saving-indicator" aria-live="polite">{t('common.loading')}</span>
          {/if}
        </div>
        <div class="card-body sync-body">
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">{t('settings.emailIntegrationEnabled')}</span>
              <span class="toggle-desc">{t('settings.emailProbeOnly')}</span>
            </div>
            <button
              class="toggle-switch"
              class:toggle-switch--on={settingsStore.emailIntegrationEnabled}
              onclick={handleEmailIntegrationToggle}
              role="switch"
              aria-checked={settingsStore.emailIntegrationEnabled}
              aria-label={t('settings.emailIntegrationEnabled')}
              type="button"
            >
              <span class="toggle-thumb"></span>
            </button>
          </div>

          {#if settingsStore.emailIntegrationEnabled}
            <div class="email-grid">
              <div class="field-row">
                <label class="field-label" for="smtp-host">{t('settings.smtpHost')}</label>
                <input
                  id="smtp-host"
                  class="input"
                  type="text"
                  value={smtpHostLocal}
                  oninput={(e) => { smtpHostLocal = (e.target as HTMLInputElement).value; markEmailFieldDirty('smtpHost'); }}
                  onblur={() => commitEmailStringSetting('smtpHost', smtpHostLocal)}
                  placeholder="smtp.example.com"
                  spellcheck={false}
                />
              </div>
              <div class="field-row">
                <label class="field-label" for="smtp-port">{t('settings.smtpPort')}</label>
                <input
                  id="smtp-port"
                  class="input"
                  type="number"
                  min="1"
                  max="65535"
                  value={smtpPortLocal}
                  oninput={(e) => { smtpPortLocal = (e.target as HTMLInputElement).value; markEmailFieldDirty('smtpPort'); }}
                  onblur={() => commitEmailPortSetting('smtpPort', smtpPortLocal, 587, (v) => smtpPortLocal = v)}
                />
              </div>
              <div class="field-row">
                <label class="field-label" for="smtp-user">{t('settings.smtpUsername')}</label>
                <input
                  id="smtp-user"
                  class="input"
                  type="text"
                  value={smtpUsernameLocal}
                  oninput={(e) => { smtpUsernameLocal = (e.target as HTMLInputElement).value; markEmailFieldDirty('smtpUsername'); }}
                  onblur={() => commitEmailStringSetting('smtpUsername', smtpUsernameLocal)}
                  spellcheck={false}
                />
              </div>
              <div class="field-row field-row--wide">
                <label class="field-label" for="smtp-from">{t('settings.smtpFrom')}</label>
                <input
                  id="smtp-from"
                  class="input"
                  type="email"
                  value={smtpFromLocal}
                  oninput={(e) => { smtpFromLocal = (e.target as HTMLInputElement).value; markEmailFieldDirty('smtpFrom'); }}
                  onblur={() => commitEmailStringSetting('smtpFrom', smtpFromLocal)}
                  placeholder="sales@example.com"
                  autocomplete="email"
                />
              </div>
              <div class="field-row email-test-row">
                <button
                  class="btn btn-secondary btn-sm"
                  type="button"
                  onclick={testSmtpConnection}
                  disabled={smtpTestLoading}
                >
                  {smtpTestLoading ? t('common.loading') : t('settings.emailTestConnection')}
                </button>
                {#if smtpTestMessage}
                  <span class="field-hint">{smtpTestMessage}</span>
                {/if}
              </div>
            </div>

            <div class="email-grid">
              <div class="field-row">
                <label class="field-label" for="imap-host">{t('settings.imapHost')}</label>
                <input
                  id="imap-host"
                  class="input"
                  type="text"
                  value={imapHostLocal}
                  oninput={(e) => { imapHostLocal = (e.target as HTMLInputElement).value; markEmailFieldDirty('imapHost'); }}
                  onblur={() => commitEmailStringSetting('imapHost', imapHostLocal)}
                  placeholder="imap.example.com"
                  spellcheck={false}
                />
              </div>
              <div class="field-row">
                <label class="field-label" for="imap-port">{t('settings.imapPort')}</label>
                <input
                  id="imap-port"
                  class="input"
                  type="number"
                  min="1"
                  max="65535"
                  value={imapPortLocal}
                  oninput={(e) => { imapPortLocal = (e.target as HTMLInputElement).value; markEmailFieldDirty('imapPort'); }}
                  onblur={() => commitEmailPortSetting('imapPort', imapPortLocal, 993, (v) => imapPortLocal = v)}
                />
              </div>
              <div class="field-row">
                <label class="field-label" for="imap-user">{t('settings.imapUsername')}</label>
                <input
                  id="imap-user"
                  class="input"
                  type="text"
                  value={imapUsernameLocal}
                  oninput={(e) => { imapUsernameLocal = (e.target as HTMLInputElement).value; markEmailFieldDirty('imapUsername'); }}
                  onblur={() => commitEmailStringSetting('imapUsername', imapUsernameLocal)}
                  spellcheck={false}
                />
              </div>
              <div class="field-row email-test-row">
                <button
                  class="btn btn-secondary btn-sm"
                  type="button"
                  onclick={testImapConnection}
                  disabled={imapTestLoading}
                >
                  {imapTestLoading ? t('common.loading') : t('settings.emailTestConnection')}
                </button>
                {#if imapTestMessage}
                  <span class="field-hint">{imapTestMessage}</span>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </section>
      {/if}

      {#if activePane === 'data'}
      <!-- Data management -->
      <section class="card settings-section" aria-labelledby="data-heading">
        <div class="card-header">
          <h2 class="section-title" id="data-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
              <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
            </svg>
            {t('settings.dataManagement')}
          </h2>
        </div>
        <div class="card-body data-body">
          <div class="data-guidance" data-testid="settings-data-guidance">
            <span class="data-action-label">{t('settings.dataGuidanceTitle')}</span>
            <span class="data-action-desc">{t('settings.dataGuidance')}</span>
            <span class="data-action-warning">{t('settings.backupUnencryptedWarning')}</span>
          </div>
          <div class="data-action">
            <div class="data-action-info">
              <span class="data-action-label">{t('settings.exportAll')}</span>
              <span class="data-action-desc">{t('export.title')}</span>
              <span class="data-action-warning">{t('settings.exportUnencryptedWarning')}</span>
            </div>
            <button
              class="btn btn-secondary btn-sm"
              onclick={handleExportAll}
              type="button"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
              </svg>
              {t('settings.exportAll')}
            </button>
          </div>

          <div class="data-action">
            <div class="data-action-info">
              <span class="data-action-label">{t('settings.importData')}</span>
              <span class="data-action-desc">{t('import.title')}</span>
            </div>
            <button
              class="btn btn-secondary btn-sm"
              onclick={handleImportData}
              type="button"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
              </svg>
              {t('settings.importData')}
            </button>
          </div>

          <div class="backup-panel" aria-live="polite">
            <div class="backup-panel-header">
              <div class="data-action-info">
                <span class="data-action-label">{t('settings.backupRestore')}</span>
                <span class="data-action-desc">{t('settings.backupRestoreDesc')}</span>
                <span class="data-action-warning">{t('settings.backupUnencryptedWarning')}</span>
              </div>
              <button
                class="btn btn-secondary btn-sm"
                onclick={handleChooseBackupFolder}
                type="button"
                disabled={backupBusy !== null}
              >
                {backupBusy === 'select' ? t('common.loading') : t('settings.backupChooseFolder')}
              </button>
            </div>

            <div class="backup-folder">
              <span class="field-label">{t('settings.backupSelectedFolder')}</span>
              <span class="backup-path">{backupDirLocal || t('settings.backupNoFolder')}</span>
            </div>

            <div class="backup-actions">
              <button
                class="btn btn-secondary btn-sm"
                onclick={handleCreateBackup}
                type="button"
                disabled={!backupDirLocal || backupBusy !== null}
                title={t('settings.backupCreateDesc')}
              >
                {backupBusy === 'create' ? t('common.loading') : t('settings.backupCreate')}
              </button>
              <button
                class="btn btn-secondary btn-sm"
                onclick={handleValidateBackup}
                type="button"
                disabled={!backupDirLocal || backupBusy !== null}
                title={t('settings.backupValidateDesc')}
              >
                {backupBusy === 'validate' ? t('common.loading') : t('settings.backupValidate')}
              </button>
              <button
                class="btn btn-secondary btn-sm btn-danger-soft"
                onclick={handleRestoreBackup}
                type="button"
                disabled={!selectedBackupIsValidated() || backupBusy !== null}
                title={t('settings.backupRestoreActionDesc')}
              >
                {backupBusy === 'restore' ? t('common.loading') : t('settings.backupRestoreAction')}
              </button>
            </div>

            {#if lastBackupValidation}
              <dl class="backup-metadata" aria-label={t('settings.backupMetadata')}>
                <div>
                  <dt>{t('settings.backupCreatedAt')}</dt>
                  <dd>{lastBackupValidation.metadata.created_at}</dd>
                </div>
                <div>
                  <dt>{t('settings.backupSchemaVersion')}</dt>
                  <dd>{lastBackupValidation.metadata.schema_version}</dd>
                </div>
                <div>
                  <dt>{t('settings.backupAppVersion')}</dt>
                  <dd>{lastBackupValidation.metadata.app_version}</dd>
                </div>
                <div>
                  <dt>{t('settings.backupDeviceId')}</dt>
                  <dd>{lastBackupValidation.metadata.device_id}</dd>
                </div>
              </dl>
            {/if}

            {#if backupMessage}
              <p class="backup-status backup-status--success">{backupMessage}</p>
            {/if}
            {#if backupError}
              <p class="backup-status backup-status--error">{backupError}</p>
            {/if}
          </div>
        </div>
      </section>
      {/if}
    </div>

    {#if activePane === 'appearance'}
    <aside class="settings-sidebar">
      <!-- About / Mission -->
      <section class="card settings-section settings-about" aria-labelledby="about-heading">
        <div class="card-header">
          <h2 class="section-title" id="about-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            {t('settings.about')}
          </h2>
        </div>
        <div class="card-body about-body">
          <!-- App logo mark -->
          <div class="about-logo" aria-hidden="true">
            <svg width="36" height="36" viewBox="0 0 36 36" fill="none" aria-hidden="true">
              <rect width="36" height="36" rx="8" fill="#20808D"/>
              <path d="M9 18C9 13.03 13.03 9 18 9C22.97 9 27 13.03 27 18" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
              <path d="M18 27C15.24 27 12.76 25.76 11.1 23.8" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
              <circle cx="18" cy="18" r="3" fill="white"/>
            </svg>
          </div>

          <div class="about-app">
            <strong class="about-appname">{t('app.title')}</strong>
            <span class="about-tagline">{t('app.tagline')}</span>
          </div>

          <!-- Mission -->
          <div class="mission-block">
            <h3 class="mission-title">{t('mission.title')}</h3>
            <p class="mission-desc">{t('mission.description')}</p>
          </div>

          <!-- Version -->
          <div class="about-meta">
            <span class="about-meta-item">v1.0.0</span>
            <span class="about-meta-item">900 Labs</span>
            <a
              class="about-meta-link"
              href="https://github.com/900Labs/900CRM"
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub
            </a>
          </div>
        </div>
      </section>
    </aside>
    {/if}

    {#if activePane === 'integrations'}
    <aside class="settings-sidebar">
      <!-- Integrations -->
      <section class="card settings-section" aria-labelledby="integrations-heading">
        <div class="card-header">
          <h2 class="section-title" id="integrations-heading">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
              <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/>
              <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/>
            </svg>
            {t('settings.integrations')}
          </h2>
          <button
            class="btn btn-secondary btn-sm"
            onclick={loadExternalClients}
            type="button"
            disabled={externalClientsLoading}
          >
            {externalClientsLoading ? t('common.loading') : t('settings.externalClientsRetry')}
          </button>
        </div>
        <div class="card-body integrations-body">
          <div class="external-client-create" aria-live="polite">
            <div class="data-action-info">
              <span class="data-action-label">{t('settings.externalClients')}</span>
              <span class="data-action-desc">{t('settings.externalClientsDesc')}</span>
            </div>

            <div class="external-client-form">
              <div class="field-row">
                <label class="field-label" for="external-client-name">{t('settings.externalClientName')}</label>
                <input
                  id="external-client-name"
                  class="input"
                  type="text"
                  value={externalClientName}
                  oninput={handleExternalClientNameInput}
                  placeholder={t('settings.externalClientNamePlaceholder')}
                  disabled={externalClientCreateLoading}
                />
              </div>
              <div class="field-row">
                <label class="field-label" for="external-client-type">{t('settings.externalClientType')}</label>
                <input
                  id="external-client-type"
                  class="input"
                  type="text"
                  value={externalClientType}
                  oninput={handleExternalClientTypeInput}
                  placeholder={t('settings.externalClientTypePlaceholder')}
                  disabled={externalClientCreateLoading}
                />
              </div>
              <button
                class="btn btn-secondary btn-sm"
                onclick={handleCreateExternalClientPlaceholder}
                type="button"
                disabled={!externalClientName.trim() || !externalClientType.trim() || externalClientCreateLoading}
              >
                {externalClientCreateLoading ? t('common.loading') : t('settings.externalClientsCreate')}
              </button>
            </div>

            {#if externalClientCreateMessage}
              <p class="backup-status backup-status--success">{externalClientCreateMessage}</p>
            {/if}
            {#if externalClientCreateError}
              <p class="backup-status backup-status--error">{externalClientCreateError}</p>
            {/if}
          </div>

          {#if externalClientsLoading && externalClients.length === 0}
            <p class="external-client-empty">{t('settings.externalClientsLoading')}</p>
          {:else if externalClientsError}
            <div class="external-client-error" role="alert">
              <span>{t('settings.externalClientsLoadFailed')}: {externalClientsError}</span>
              <button class="btn btn-secondary btn-sm" onclick={loadExternalClients} type="button">
                {t('settings.externalClientsRetry')}
              </button>
            </div>
          {:else if externalClients.length === 0}
            <p class="external-client-empty">{t('settings.externalClientsEmpty')}</p>
          {:else}
            <div class="external-client-list">
              {#each externalClients as client (client.id)}
                <article class="external-client-row">
                  <div class="external-client-row-header">
                    <div class="external-client-title">
                      <strong>{client.name}</strong>
                      <span>{client.clientType}</span>
                    </div>
                    <span class:external-client-badge--enabled={client.enabled} class="external-client-badge">
                      {client.enabled ? t('settings.externalClientEnabled') : t('settings.externalClientDisabled')}
                    </span>
                  </div>
                  <dl class="external-client-meta">
                    <div>
                      <dt>{t('settings.externalClientPermissionMode')}</dt>
                      <dd>{client.permissionMode}</dd>
                    </div>
                    <div>
                      <dt>{t('common.created')}</dt>
                      <dd>{formatExternalClientTimestamp(client.createdAt)}</dd>
                    </div>
                    <div>
                      <dt>{t('common.updated')}</dt>
                      <dd>{formatExternalClientTimestamp(client.updatedAt)}</dd>
                    </div>
                  </dl>
                  <div class="external-client-activation" aria-live="polite">
                    <div class="external-client-activation-controls">
                      <div class="field-row external-client-activation-field">
                        <label class="field-label" for={`external-client-activation-${client.id}`}>
                          {t('settings.externalClientActivationMode')}
                        </label>
                        <select
                          id={`external-client-activation-${client.id}`}
                          class="input"
                          value={activationDraftModeForClient(client)}
                          onchange={(e) => handleExternalClientActivationModeChange(client.id, e)}
                          disabled={externalClientActivationSaving[client.id]}
                        >
                          {#each EXTERNAL_CLIENT_ACTIVATION_MODES as option}
                            <option value={option.value}>{t(option.labelKey)}</option>
                          {/each}
                        </select>
                      </div>
                      <button
                        class="btn btn-secondary btn-sm"
                        type="button"
                        onclick={() => handleUpdateExternalClientActivation(client)}
                        disabled={externalClientActivationSaving[client.id] || activationDraftMatchesClient(client)}
                      >
                        {externalClientActivationSaving[client.id] ? t('common.loading') : t('settings.externalClientActivationSave')}
                      </button>
                    </div>
                    <p class="external-client-activation-note">{t('settings.externalClientActivationDesc')}</p>
                    {#if externalClientActivationMessages[client.id]}
                      <p class="backup-status backup-status--success">{externalClientActivationMessages[client.id]}</p>
                    {/if}
                    {#if externalClientActivationErrors[client.id]}
                      <p class="backup-status backup-status--error">{externalClientActivationErrors[client.id]}</p>
                    {/if}
                  </div>
                  <ExternalClientPermissions {client} />
                </article>
              {/each}
            </div>
          {/if}
        </div>
      </section>
    </aside>
    {/if}
  </div>
</div>

<ImportExport bind:open={showImportExport} />

<style>
  /* ── Page ─────────────────────────────────────────────────────────────────── */

  .settings-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .settings-page .page-header {
    margin-block-end: 0;
  }

  .settings-section-nav {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .settings-section-nav-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 30px;
    padding: var(--space-2) var(--space-3);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--border-radius-md);
    background-color: var(--surface-default);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    line-height: var(--leading-tight);
    cursor: pointer;
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out);
  }

  .settings-section-nav-button:hover {
    border-color: var(--color-primary-200);
    background-color: var(--surface-raised);
    color: var(--text-primary);
  }

  .settings-section-nav-button:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .settings-section-nav-button--active,
  .settings-section-nav-button--active:hover {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
    color: var(--text-accent);
  }

  .settings-pane-help {
    margin: 0;
    max-width: 52rem;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
  }

  /* ── Two-column grid ─────────────────────────────────────────────────────── */

  .settings-grid {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: var(--space-6);
    align-items: start;
  }

  .settings-grid--single {
    grid-template-columns: 1fr;
  }

  .settings-grid--single .settings-main {
    max-width: 44rem;
  }

  @media (max-width: 900px) {
    .settings-grid {
      grid-template-columns: 1fr;
    }
  }

  /* ── Columns ─────────────────────────────────────────────────────────────── */

  .settings-main,
  .settings-sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  /* ── Sections ────────────────────────────────────────────────────────────── */

  .settings-section .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .saving-indicator {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  /* ── Language grid ───────────────────────────────────────────────────────── */

  .locale-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: var(--space-3);
  }

  .locale-option {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-4) var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-default);
    cursor: pointer;
    text-align: start;
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out);
  }

  .locale-option:hover {
    border-color: var(--color-primary-200);
    background-color: var(--surface-raised);
  }

  .locale-option--active {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
  }

  .locale-native {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
  }

  .locale-name {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .locale-check {
    position: absolute;
    inset-inline-end: var(--space-3);
    top: var(--space-3);
    color: var(--color-primary);
  }

  /* ── Currency selector ───────────────────────────────────────────────────── */

  .field-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field-label {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .select-input {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%2313343B' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right var(--space-3) center;
    padding-inline-end: var(--space-8);
    cursor: pointer;
  }

  :global([data-theme="dark"]) .select-input {
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%23FCFAF6' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E");
  }

  /* ── Theme options ───────────────────────────────────────────────────────── */

  .theme-options {
    display: flex;
    gap: var(--space-3);
  }

  .theme-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-5) var(--space-3);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-default);
    cursor: pointer;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out);
  }

  .theme-option:hover {
    border-color: var(--color-primary-200);
    background-color: var(--surface-raised);
    color: var(--text-primary);
  }

  .theme-option--active {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
    color: var(--text-accent);
  }

  /* ── Date format options ─────────────────────────────────────────────────── */

  .date-format-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .date-format-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-default);
    cursor: pointer;
    text-align: start;
    transition: border-color var(--duration-fast) var(--ease-out),
                background-color var(--duration-fast) var(--ease-out);
  }

  .date-format-option:hover {
    border-color: var(--color-primary-200);
    background-color: var(--surface-raised);
  }

  .date-format-option--active {
    border-color: var(--color-primary);
    background-color: var(--surface-active);
  }

  .format-code {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    background: none;
  }

  .format-example {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  /* ── Sync section ────────────────────────────────────────────────────────── */

  .sync-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-label {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .toggle-desc {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  /* Toggle switch */
  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: 9999px;
    background-color: var(--border-default);
    border: none;
    cursor: pointer;
    flex-shrink: 0;
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .toggle-switch--on {
    background-color: var(--color-primary);
  }

  .toggle-thumb {
    position: absolute;
    top: 3px;
    inset-inline-start: 3px;
    width: 18px;
    height: 18px;
    background-color: white;
    border-radius: 50%;
    transition: inset-inline-start var(--duration-fast) var(--ease-out),
                transform var(--duration-fast) var(--ease-out);
    box-shadow: 0 1px 3px rgba(0,0,0,0.15);
  }

  .toggle-switch--on .toggle-thumb {
    inset-inline-start: calc(100% - 21px);
  }

  .sync-url-row {
    padding-block-start: var(--space-2);
  }

  .field-hint {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  .sync-url-input-wrap {
    width: 100%;
  }

  .email-probe-pane {
    border-style: dashed;
  }

  .probe-badge {
    display: inline-flex;
    align-items: center;
    margin-inline-start: var(--space-2);
    padding: 0 var(--space-2);
    border: var(--border-width) solid var(--border-default);
    border-radius: 9999px;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .email-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
    padding: var(--space-4);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-md);
    background-color: var(--surface-raised);
  }

  .field-row--wide {
    grid-column: span 2;
  }

  .email-test-row {
    grid-column: span 2;
    flex-direction: row;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  /* ── About section ───────────────────────────────────────────────────────── */

  .about-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .about-logo {
    display: flex;
    align-items: center;
  }

  .about-app {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .about-appname {
    font-size: var(--text-md);
    font-weight: var(--weight-bold);
    color: var(--text-primary);
  }

  .about-tagline {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .mission-block {
    padding: var(--space-4) var(--space-5);
    background-color: var(--surface-raised);
    border-radius: var(--radius-md);
    border-inline-start: 3px solid var(--color-primary);
  }

  .mission-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-accent);
    margin: 0 0 var(--space-2) 0;
  }

  .mission-desc {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.6;
  }

  .about-meta {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
    padding-block-start: var(--space-2);
    border-block-start: var(--border-width) solid var(--border-default);
  }

  .about-meta-item {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .about-meta-link {
    font-size: var(--text-xs);
    color: var(--text-accent);
    text-decoration: none;
  }

  .about-meta-link:hover {
    text-decoration: underline;
  }

  /* ── Data management ─────────────────────────────────────────────────────── */

  .data-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .data-guidance {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-raised);
  }

  .data-action {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
  }

  .data-action-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .data-action-label {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
  }

  .data-action-desc {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .data-action-warning {
    font-size: var(--text-xs);
    color: var(--text-warning);
  }

  /* ── Integrations ───────────────────────────────────────────────────────── */

  .integrations-body,
  .external-client-create,
  .external-client-list,
  .external-client-form {
    display: flex;
    flex-direction: column;
  }

  .integrations-body,
  .external-client-list {
    gap: var(--space-4);
  }

  .external-client-create,
  .external-client-form {
    gap: var(--space-3);
  }

  .external-client-create,
  .external-client-row {
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
  }

  .external-client-create {
    background-color: var(--surface-raised);
  }

  .external-client-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    background-color: var(--surface-default);
  }

  .external-client-row-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .external-client-title {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .external-client-title strong {
    font-size: var(--text-sm);
    color: var(--text-primary);
    overflow-wrap: anywhere;
  }

  .external-client-title span {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    overflow-wrap: anywhere;
  }

  .external-client-badge {
    flex-shrink: 0;
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    border: var(--border-width) solid var(--border-default);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
  }

  .external-client-badge--enabled {
    color: var(--color-success-600);
    border-color: var(--color-success-600);
  }

  .external-client-meta {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-2);
    margin: 0;
  }

  .external-client-meta div {
    min-width: 0;
  }

  .external-client-meta dt {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-tertiary);
  }

  .external-client-meta dd {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--text-primary);
    overflow-wrap: anywhere;
  }

  .external-client-activation {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding-top: var(--space-3);
    border-top: var(--border-width) solid var(--border-default);
  }

  .external-client-activation-controls {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--space-3);
  }

  .external-client-activation-field {
    min-width: min(100%, 220px);
  }

  .external-client-activation-note {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .external-client-empty {
    margin: 0;
    color: var(--text-secondary);
    font-size: var(--text-xs);
  }

  .external-client-error {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
    color: var(--color-danger-600);
    font-size: var(--text-xs);
  }

  .backup-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: var(--border-width) solid var(--border-default);
    background-color: var(--surface-raised);
  }

  .backup-panel-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .backup-folder {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .backup-path {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    overflow-wrap: anywhere;
  }

  .backup-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .btn-danger-soft {
    color: var(--color-danger-600);
    border-color: var(--color-danger-500);
  }

  .btn-danger-soft:hover:not(:disabled) {
    color: var(--color-danger);
    border-color: var(--color-danger);
  }

  .backup-metadata {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
    margin: 0;
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    background-color: var(--surface-default);
  }

  .backup-metadata div {
    min-width: 0;
  }

  .backup-metadata dt {
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    color: var(--text-tertiary);
  }

  .backup-metadata dd {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--text-primary);
    overflow-wrap: anywhere;
  }

  .backup-status {
    margin: 0;
    font-size: var(--text-xs);
    overflow-wrap: anywhere;
  }

  .backup-status--success {
    color: var(--color-success-600);
  }

  .backup-status--error {
    color: var(--color-danger-600);
  }

  @media (max-width: 900px) {
    .email-grid {
      grid-template-columns: 1fr;
    }

    .field-row--wide,
    .email-test-row {
      grid-column: span 1;
    }

    .backup-panel-header {
      flex-direction: column;
    }

    .backup-metadata {
      grid-template-columns: 1fr;
    }
  }
</style>
