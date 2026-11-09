/**
 * src/lib/api/settings.ts — Tauri IPC wrappers for app settings.
 */

import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  language: string;
  currency: string;
  theme: 'light' | 'dark' | 'system';
  dateFormat: string;
  syncEnabled: boolean;
  syncUrl: string;
  notificationsEnabled: boolean;
  reminderLeadMinutes: number;
  emailIntegrationEnabled: boolean;
  smtpHost: string;
  smtpPort: number;
  smtpUsername: string;
  smtpPassword: string;
  smtpFrom: string;
  imapHost: string;
  imapPort: number;
  imapUsername: string;
  imapPassword: string;
}

export type SettingKey = keyof AppSettings;

interface BackendSetting {
  key: string;
  value: string;
  updated_at: string;
}

type BackendSettingsMap = Record<string, string>;

function toBackendKey(key: SettingKey): string {
  switch (key) {
    case 'dateFormat':
      return 'date_format';
    case 'syncEnabled':
      return 'sync_enabled';
    case 'syncUrl':
      return 'sync_url';
    case 'notificationsEnabled':
      return 'notifications_enabled';
    case 'reminderLeadMinutes':
      return 'reminder_lead_minutes';
    case 'emailIntegrationEnabled':
      return 'email_integration_enabled';
    case 'smtpHost':
      return 'smtp_host';
    case 'smtpPort':
      return 'smtp_port';
    case 'smtpUsername':
      return 'smtp_username';
    case 'smtpPassword':
      return 'smtp_password';
    case 'smtpFrom':
      return 'smtp_from';
    case 'imapHost':
      return 'imap_host';
    case 'imapPort':
      return 'imap_port';
    case 'imapUsername':
      return 'imap_username';
    case 'imapPassword':
      return 'imap_password';
    default:
      return key;
  }
}

function parseBoolean(value: string | undefined, fallback = false): boolean {
  if (value == null) {
    return fallback;
  }
  return value === 'true' || value === '1';
}

function parseTheme(value: string | undefined): AppSettings['theme'] {
  if (value === 'light' || value === 'dark' || value === 'system') {
    return value;
  }
  return 'system';
}

function parseInteger(value: string | undefined, fallback: number): number {
  if (value == null || value.trim() === '') {
    return fallback;
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function parsePort(value: string | undefined, fallback: number): number {
  const parsed = parseInteger(value, fallback);
  return Math.min(65535, Math.max(1, parsed));
}

function mapSettings(map: BackendSettingsMap): AppSettings {
  return {
    language: map.language || 'en',
    currency: map.currency || 'USD',
    theme: parseTheme(map.theme),
    dateFormat: map.date_format || 'MMM D, YYYY',
    syncEnabled: parseBoolean(map.sync_enabled, false),
    syncUrl: map.sync_url || '',
    notificationsEnabled: parseBoolean(map.notifications_enabled, true),
    reminderLeadMinutes: parseInteger(map.reminder_lead_minutes, 30),
    emailIntegrationEnabled: parseBoolean(map.email_integration_enabled, false),
    smtpHost: map.smtp_host || '',
    smtpPort: parsePort(map.smtp_port, 587),
    smtpUsername: map.smtp_username || '',
    smtpPassword: map.smtp_password || '',
    smtpFrom: map.smtp_from || '',
    imapHost: map.imap_host || '',
    imapPort: parsePort(map.imap_port, 993),
    imapUsername: map.imap_username || '',
    imapPassword: map.imap_password || '',
  };
}

function serializeValue<K extends SettingKey>(key: K, value: AppSettings[K]): string {
  if (key === 'syncEnabled' || key === 'notificationsEnabled' || key === 'emailIntegrationEnabled') {
    return value ? 'true' : 'false';
  }
  if (key === 'reminderLeadMinutes') {
    return String(Math.max(1, Number(value) || 30));
  }
  if (key === 'smtpPort') {
    const parsed = Number.parseInt(String(value), 10);
    return String(Math.min(65535, Math.max(1, Number.isFinite(parsed) ? parsed : 587)));
  }
  if (key === 'imapPort') {
    const parsed = Number.parseInt(String(value), 10);
    return String(Math.min(65535, Math.max(1, Number.isFinite(parsed) ? parsed : 993)));
  }
  return String(value ?? '');
}

function parseSettingValue<K extends SettingKey>(key: K, value: string | undefined): AppSettings[K] {
  switch (key) {
    case 'syncEnabled':
      return parseBoolean(value, false) as AppSettings[K];
    case 'theme':
      return parseTheme(value) as AppSettings[K];
    case 'dateFormat':
      return (value || 'MMM D, YYYY') as AppSettings[K];
    case 'currency':
      return (value || 'USD') as AppSettings[K];
    case 'language':
      return (value || 'en') as AppSettings[K];
    case 'syncUrl':
      return (value || '') as AppSettings[K];
    case 'notificationsEnabled':
      return parseBoolean(value, true) as AppSettings[K];
    case 'reminderLeadMinutes':
      return parseInteger(value, 30) as AppSettings[K];
    case 'emailIntegrationEnabled':
      return parseBoolean(value, false) as AppSettings[K];
    case 'smtpHost':
      return (value || '') as AppSettings[K];
    case 'smtpPort':
      return parsePort(value, 587) as AppSettings[K];
    case 'smtpUsername':
      return (value || '') as AppSettings[K];
    case 'smtpPassword':
      return (value || '') as AppSettings[K];
    case 'smtpFrom':
      return (value || '') as AppSettings[K];
    case 'imapHost':
      return (value || '') as AppSettings[K];
    case 'imapPort':
      return parsePort(value, 993) as AppSettings[K];
    case 'imapUsername':
      return (value || '') as AppSettings[K];
    case 'imapPassword':
      return (value || '') as AppSettings[K];
    default:
      return value as AppSettings[K];
  }
}

export async function getSettings(): Promise<AppSettings> {
  const settings = await invoke<BackendSettingsMap>('get_settings');
  return mapSettings(settings);
}

export async function updateSetting<K extends SettingKey>(
  key: K,
  value: AppSettings[K]
): Promise<void> {
  await invoke<BackendSetting>('update_setting', {
    key: toBackendKey(key),
    value: serializeValue(key, value),
  });
}

export async function getSetting<K extends SettingKey>(key: K): Promise<AppSettings[K]> {
  const setting = await invoke<BackendSetting | null>('get_setting', {
    key: toBackendKey(key),
  });

  return parseSettingValue(key, setting?.value);
}
