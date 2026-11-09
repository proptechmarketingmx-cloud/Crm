import { describe, expect, it } from 'vitest';

import {
  combineDueDateAndTime,
  formatActivityDue,
  isActivityOverdue,
  parseActivityDue,
  splitDueDateAndTime,
} from './activityDue';

const AFTERNOON = new Date('2026-07-08T16:30:00');

describe('activity due time', () => {
  it('keeps a date-only due date pending for the full local day', () => {
    expect(isActivityOverdue('2026-07-08', false, AFTERNOON)).toBe(false);
    expect(isActivityOverdue('2026-07-07', false, AFTERNOON)).toBe(true);
    expect(parseActivityDue('2026-07-08')?.kind).toBe('date');
  });

  it('marks a timed due date overdue after that clock time', () => {
    const due = combineDueDateAndTime('2026-07-08', '15:00');
    expect(due).toMatch(/T/);
    expect(isActivityOverdue(due, false, AFTERNOON)).toBe(true);
    expect(isActivityOverdue(due, false, new Date('2026-07-08T14:00:00'))).toBe(false);
    expect(isActivityOverdue(due, true, AFTERNOON)).toBe(false);
  });

  it('round-trips date and optional time without turning date-only rows into datetimes', () => {
    expect(splitDueDateAndTime('2026-07-08')).toEqual({ date: '2026-07-08', time: '' });
    expect(combineDueDateAndTime('2026-07-08', '')).toBe('2026-07-08');
    expect(combineDueDateAndTime('', '15:00')).toBeNull();
  });

  it('formats timed dues with a clock value', () => {
    const due = combineDueDateAndTime('2026-07-08', '15:00');
    expect(formatActivityDue(due, 'en-US')).toMatch(/15|3:00/);
    expect(formatActivityDue('2026-07-08', 'en-US')).not.toMatch(/:/);
  });
});
