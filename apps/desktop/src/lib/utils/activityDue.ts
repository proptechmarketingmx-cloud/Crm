/**
 * Activity due values are either date-only (YYYY-MM-DD) or ISO-8601 datetimes.
 * Date-only rows stay pending for the full local day.
 */

export type ActivityDueKind = 'date' | 'datetime';

export interface ParsedActivityDue {
  kind: ActivityDueKind;
  /** Instant used for time-aware comparisons. Date-only uses local midnight. */
  at: number;
  dateKey: string;
}

function pad(value: number): string {
  return String(value).padStart(2, '0');
}

export function localDayKey(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function localDayStart(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

const DATE_ONLY = /^(\d{4})-(\d{2})-(\d{2})$/;

export function parseActivityDue(value: string | null | undefined): ParsedActivityDue | null {
  if (!value) {
    return null;
  }

  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  const dateOnly = DATE_ONLY.exec(trimmed);
  if (dateOnly) {
    const [, year, month, day] = dateOnly;
    const at = new Date(Number(year), Number(month) - 1, Number(day)).getTime();
    return { kind: 'date', at, dateKey: trimmed };
  }

  const parsed = new Date(trimmed);
  if (Number.isNaN(parsed.getTime())) {
    return null;
  }

  return {
    kind: 'datetime',
    at: parsed.getTime(),
    dateKey: localDayKey(parsed),
  };
}

export function isActivityOverdue(
  dueDate: string | null | undefined,
  completed: boolean,
  now: Date = new Date(),
): boolean {
  if (completed) {
    return false;
  }

  const due = parseActivityDue(dueDate);
  if (!due) {
    return false;
  }

  if (due.kind === 'date') {
    return due.at < localDayStart(now);
  }

  return due.at < now.getTime();
}

export function activityDueTimestamp(dueDate: string | null | undefined): number | null {
  return parseActivityDue(dueDate)?.at ?? null;
}

export function splitDueDateAndTime(dueDate: string | null | undefined): { date: string; time: string } {
  const due = parseActivityDue(dueDate);
  if (!due) {
    return { date: '', time: '' };
  }

  if (due.kind === 'date') {
    return { date: due.dateKey, time: '' };
  }

  const parsed = new Date(due.at);
  return {
    date: localDayKey(parsed),
    time: `${pad(parsed.getHours())}:${pad(parsed.getMinutes())}`,
  };
}

export function combineDueDateAndTime(date: string, time: string): string | null {
  const trimmedDate = date.trim();
  const trimmedTime = time.trim();
  if (!trimmedDate) {
    return null;
  }

  if (!DATE_ONLY.test(trimmedDate)) {
    return null;
  }

  if (!trimmedTime) {
    return trimmedDate;
  }

  const match = /^(\d{2}):(\d{2})$/.exec(trimmedTime);
  if (!match) {
    return trimmedDate;
  }

  const [, hour, minute] = match;
  const [year, month, day] = trimmedDate.split('-').map(Number);
  return new Date(year, month - 1, day, Number(hour), Number(minute), 0, 0).toISOString();
}

export function formatActivityDue(
  dueDate: string | null | undefined,
  locale = 'en-US',
): string {
  const due = parseActivityDue(dueDate);
  if (!due) {
    return '';
  }

  const date = new Date(due.at);
  if (due.kind === 'date') {
    try {
      return date.toLocaleDateString(locale, { month: 'short', day: 'numeric', year: 'numeric' });
    } catch {
      return due.dateKey;
    }
  }

  try {
    return date.toLocaleString(locale, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  } catch {
    return dueDate ?? '';
  }
}

export function nowLocalDateTimeParts(now: Date = new Date()): { date: string; time: string } {
  return {
    date: localDayKey(now),
    time: `${pad(now.getHours())}:${pad(now.getMinutes())}`,
  };
}
