/**
 * src/lib/services/activityReminders.ts — Lightweight desktop reminders for upcoming activities.
 *
 * Polls upcoming activities on a fixed interval and sends at-most-once reminders
 * per activity due timestamp while the app is running.
 */

import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

import { listUpcoming, type Activity } from '$lib/api/activities';
import { t } from '$lib/i18n';
import { settingsStore } from '$lib/stores/settings';
import { activityDueTimestamp } from '$lib/utils/activityDue';

const POLL_INTERVAL_MS = 60_000;
const MAX_UPCOMING_RESULTS = 100;
const REMINDER_MIN_MINUTES = 1;
const REMINDER_MAX_MINUTES = 1_440;
const NOTIFICATION_TTL_MS = 2 * 60 * 60 * 1000;

let reminderTimer: ReturnType<typeof setInterval> | null = null;
let pollingInFlight = false;
let permissionPermanentlyDenied = false;

// Key: `${activityId}:${dueDateIso}` -> due timestamp
const notifiedReminderKeys = new Map<string, number>();

function clampLeadMinutes(value: number): number {
  if (!Number.isFinite(value)) return 30;
  return Math.min(REMINDER_MAX_MINUTES, Math.max(REMINDER_MIN_MINUTES, Math.trunc(value)));
}

function dueTimestamp(activity: Activity): number | null {
  return activityDueTimestamp(activity.dueDate);
}

function reminderKey(activity: Activity): string | null {
  if (!activity.dueDate) return null;
  return `${activity.id}:${activity.dueDate}`;
}

function pruneExpiredNotifications(now: number): void {
  for (const [key, dueTs] of notifiedReminderKeys) {
    if (dueTs + NOTIFICATION_TTL_MS < now) {
      notifiedReminderKeys.delete(key);
    }
  }
}

async function ensureNotificationPermission(): Promise<boolean> {
  if (permissionPermanentlyDenied) {
    return false;
  }

  try {
    const granted = await isPermissionGranted();
    if (granted) {
      return true;
    }

    const permission = await requestPermission();
    const allowed = permission === 'granted';
    permissionPermanentlyDenied = !allowed;
    return allowed;
  } catch (err) {
    console.error('[activityReminders] Notification permission check failed:', err);
    return false;
  }
}

function buildNotificationBody(activity: Activity, minutesUntilDue: number): string {
  return `${activity.subject} • ${t('activities.dueDate')}: ${minutesUntilDue}m`;
}

async function pollAndNotify(): Promise<void> {
  if (pollingInFlight) {
    return;
  }

  pollingInFlight = true;

  try {
    if (!settingsStore.notificationsEnabled) {
      return;
    }

    const permissionGranted = await ensureNotificationPermission();
    if (!permissionGranted) {
      return;
    }

    const leadMinutes = clampLeadMinutes(settingsStore.reminderLeadMinutes);
    const reminderWindowMs = leadMinutes * 60_000;
    const now = Date.now();

    const upcoming = await listUpcoming();
    const candidates = upcoming.slice(0, MAX_UPCOMING_RESULTS);

    for (const activity of candidates) {
      const key = reminderKey(activity);
      const dueTs = dueTimestamp(activity);
      if (!key || dueTs == null) {
        continue;
      }

      const deltaMs = dueTs - now;
      if (deltaMs <= 0 || deltaMs > reminderWindowMs) {
        continue;
      }

      if (notifiedReminderKeys.has(key)) {
        continue;
      }

      const minutesUntilDue = Math.max(1, Math.ceil(deltaMs / 60_000));

      await sendNotification({
        title: t('activities.upcoming'),
        body: buildNotificationBody(activity, minutesUntilDue),
      });

      notifiedReminderKeys.set(key, dueTs);
    }

    pruneExpiredNotifications(now);
  } catch (err) {
    console.error('[activityReminders] Polling failed:', err);
  } finally {
    pollingInFlight = false;
  }
}

/**
 * Starts the reminder polling loop.
 * Returns a disposer that stops polling.
 */
export function startActivityReminderService(): () => void {
  if (reminderTimer) {
    return () => {
      // no-op; service is already running.
    };
  }

  void pollAndNotify();
  reminderTimer = setInterval(() => {
    void pollAndNotify();
  }, POLL_INTERVAL_MS);

  return () => {
    if (reminderTimer) {
      clearInterval(reminderTimer);
      reminderTimer = null;
    }
  };
}
