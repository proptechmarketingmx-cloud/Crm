/**
 * src/lib/api/updater.ts — Tauri v2 auto-update helpers.
 */

import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

/**
 * Checks the configured update endpoint for a newer release.
 * Returns the `Update` object when one is available, or `null` when the app is
 * already up to date. Returns `null` on network failure so the offline-first
 * app degrades silently instead of surfacing connection errors.
 */
export async function checkForUpdates(): Promise<Update | null> {
  try {
    const update = await check();
    if (update?.available) {
      return update;
    }
    return null;
  } catch (err) {
    console.error('[updater] checkForUpdates failed:', err);
    return null;
  }
}

/**
 * Downloads and installs the given update, then relaunches the app.
 * Throws on installation failure so callers can surface an error state.
 */
export async function downloadAndInstallUpdate(update: Update): Promise<void> {
  await update.downloadAndInstall();
  await relaunch();
}
