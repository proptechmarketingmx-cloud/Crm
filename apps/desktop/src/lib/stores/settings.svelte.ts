/**
 * src/lib/stores/settings.svelte.ts — Application settings store for 900CRM.
 *
 * Manages language, currency, theme, date format, and sync configuration.
 * Persists settings via the Tauri backend (SQLite).
 *
 * @module stores/settings
 */

import { getSettings, updateSetting } from '$lib/api/settings';
import type { AppSettings } from '$lib/api/settings';
import { chooseLocale, t } from '$lib/i18n';
import { uiStore } from './ui';

// ─────────────────────────────────────────────────────────────────────────────
// SettingsStore
// ─────────────────────────────────────────────────────────────────────────────

class SettingsStore {
  // ── State ───────────────────────────────────────────────────────────────────

  /** ISO 639-1 locale code. */
  language = $state<string>('en');

  /** ISO 4217 currency code. */
  currency = $state<string>('USD');

  /** UI theme. */
  theme = $state<'light' | 'dark' | 'system'>('system');

  /** Date display format. */
  dateFormat = $state<string>('MMM D, YYYY');

  /** Whether background sync is enabled. */
  syncEnabled = $state<boolean>(false);

  /** URL of the sync server. */
  syncUrl = $state<string>('');

  /** Whether desktop activity reminders are enabled. */
  notificationsEnabled = $state<boolean>(true);

  /** Minutes before due time to trigger reminders. */
  reminderLeadMinutes = $state<number>(30);

  /** Whether optional email integration settings are enabled. */
  emailIntegrationEnabled = $state<boolean>(false);

  /** SMTP host for optional outbound email integration. */
  smtpHost = $state<string>('');

  /** SMTP port for optional outbound email integration. */
  smtpPort = $state<number>(587);

  /** SMTP username for optional outbound email integration. */
  smtpUsername = $state<string>('');

  /** SMTP password for optional outbound email integration. */
  smtpPassword = $state<string>('');

  /** Default sender address for optional outbound email integration. */
  smtpFrom = $state<string>('');

  /** IMAP host for optional inbound email integration. */
  imapHost = $state<string>('');

  /** IMAP port for optional inbound email integration. */
  imapPort = $state<number>(993);

  /** IMAP username for optional inbound email integration. */
  imapUsername = $state<string>('');

  /** IMAP password for optional inbound email integration. */
  imapPassword = $state<string>('');

  /** Whether settings have been loaded from the backend. */
  loaded = $state<boolean>(false);

  // ── Actions ─────────────────────────────────────────────────────────────────

  /**
   * Load all settings from the backend and apply them to the UI.
   * Call once on app startup.
   */
  async loadSettings(): Promise<void> {
    try {
      const data: AppSettings = await getSettings();
      this.language   = data.language;
      this.currency   = data.currency;
      this.theme      = data.theme;
      this.dateFormat = data.dateFormat;
      this.syncEnabled = data.syncEnabled;
      this.syncUrl    = data.syncUrl;
      this.notificationsEnabled = data.notificationsEnabled;
      this.reminderLeadMinutes = data.reminderLeadMinutes;
      this.emailIntegrationEnabled = data.emailIntegrationEnabled;
      this.smtpHost = data.smtpHost;
      this.smtpPort = data.smtpPort;
      this.smtpUsername = data.smtpUsername;
      this.smtpPassword = '';
      this.smtpFrom = data.smtpFrom;
      this.imapHost = data.imapHost;
      this.imapPort = data.imapPort;
      this.imapUsername = data.imapUsername;
      this.imapPassword = '';
      this.loaded     = true;

      if (data.smtpPassword || data.imapPassword) {
        void this.clearStoredEmailPasswords();
      }

      // Apply immediately
      this._applyTheme(data.theme);
      chooseLocale(data.language);
    } catch (err) {
      console.error('[settings] Failed to load settings:', err);
      // Apply defaults
      this._applyTheme(this.theme);
      chooseLocale(this.language);
      this.loaded = true;
    }
  }

  /**
   * Update a single setting. Persists to backend and applies immediately.
   *
   * @param key    Setting key
   * @param value  New value
   */
  async updateSetting<K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ): Promise<void> {
    const old = this[key as keyof SettingsStore];

    try {
      const nextValue =
        key === 'smtpPassword' || key === 'imapPassword'
          ? ('' as AppSettings[K])
          : value;

      // Optimistic update
      (this as Record<string, unknown>)[key] = nextValue;

      await updateSetting(key, nextValue);

      // Apply side effects
      if (key === 'theme') {
        this._applyTheme(value as AppSettings['theme']);
      }
      if (key === 'language') {
        chooseLocale(value as string);
      }
    } catch (err) {
      // Rollback on failure
      (this as Record<string, unknown>)[key] = old;
      uiStore.toastError(t('errors.saveSetting', { key }));
      throw err;
    }
  }

  /**
   * Apply the theme to the document element.
   * @param theme  'light' | 'dark' | 'system'
   */
  _applyTheme(theme: 'light' | 'dark' | 'system'): void {
    if (typeof document === 'undefined') return;
    document.documentElement.setAttribute('data-theme', theme);
  }

  /**
   * Mail send/receive is not implemented. Wipe leftover plaintext passwords
   * so a previous settings form cannot keep mailbox secrets in SQLite.
   */
  private async clearStoredEmailPasswords(): Promise<void> {
    try {
      await updateSetting('smtpPassword', '');
      await updateSetting('imapPassword', '');
    } catch (err) {
      console.error('[settings] Failed to clear leftover email passwords:', err);
    }
  }
}

/** Singleton settings store. */
export const settingsStore = new SettingsStore();
