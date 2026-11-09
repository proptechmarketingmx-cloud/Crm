import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';
import { validateUrl } from './validators';

const ALLOWED_PROTOCOLS = new Set(['http:', 'https:']);

/**
 * Open a user-supplied URL in the OS handler after an allow-list check.
 * mailto: stays on the email helper. javascript: and file: are rejected.
 */
export async function openExternalUrl(raw: string): Promise<void> {
  const value = raw.trim();
  const result = validateUrl(value);
  if (!result.valid) {
    throw new Error(result.error ?? 'Invalid URL');
  }

  const parsed = new URL(value);
  if (!ALLOWED_PROTOCOLS.has(parsed.protocol)) {
    throw new Error('URL must use http or https');
  }

  await open(parsed.toString());
}

/**
 * Open a stored local file path in the OS handler.
 * This is for user-saved bookmarks only. It rejects URLs.
 */
export async function openLocalPath(raw: string): Promise<void> {
  const value = raw.trim();
  if (!value) {
    throw new Error('File path is required');
  }

  const lower = value.toLowerCase();
  if (
    lower.startsWith('http://')
    || lower.startsWith('https://')
    || lower.startsWith('javascript:')
    || lower.startsWith('data:')
    || lower.startsWith('file:')
  ) {
    throw new Error('File links must be a local path, not a URL');
  }

  if (value.includes('\0') || value.includes('\r') || value.includes('\n')) {
    throw new Error('File path is invalid');
  }

  const validated = await invoke<string>('validate_open_path', { file_path: value });
  await open(validated);
}
