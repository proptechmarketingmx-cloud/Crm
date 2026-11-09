import { describe, expect, it } from 'vitest';
import type { Activity } from '$lib/api/activities';
import { buildActivityWeek, shiftWeek, startOfWeek } from './activityWeek';

const NOW = new Date('2026-08-12T16:30:00'); // Wednesday

function activity(overrides: Partial<Activity>): Activity {
  return {
    id: 'activity-1',
    type: 'task',
    subject: 'Follow up',
    notes: null,
    dueDate: '2026-08-12',
    completedAt: null,
    status: 'pending',
    contactId: null,
    contactName: null,
    dealId: null,
    dealName: null,
    createdAt: '2026-08-01T08:00:00Z',
    updatedAt: '2026-08-01T08:00:00Z',
    ...overrides,
  };
}

describe('activity week', () => {
  it('starts the week on Monday in local time', () => {
    expect(startOfWeek(NOW)).toEqual(new Date(2026, 7, 10));
    expect(startOfWeek(new Date(2026, 7, 16))).toEqual(new Date(2026, 7, 10));
    expect(startOfWeek(new Date(2026, 7, 9))).toEqual(new Date(2026, 7, 3));
  });

  it('places dated work on the matching weekday and keeps undated open work aside', () => {
    const week = buildActivityWeek([
      activity({ id: 'mon', dueDate: '2026-08-10', subject: 'Monday call' }),
      activity({ id: 'wed', dueDate: '2026-08-12', subject: 'Wednesday task' }),
      activity({ id: 'done', dueDate: '2026-08-12', status: 'completed', subject: 'Done' }),
      activity({ id: 'later', dueDate: '2026-08-20', subject: 'Next week' }),
      activity({ id: 'open', dueDate: null, subject: 'No date' }),
      activity({ id: 'done-undated', dueDate: null, status: 'completed', subject: 'Old' }),
    ], startOfWeek(NOW), NOW);

    expect(week.weekStart).toBe('2026-08-10');
    expect(week.weekEnd).toBe('2026-08-16');
    expect(week.days).toHaveLength(7);
    expect(week.days[0]?.activities.map((item) => item.id)).toEqual(['mon']);
    expect(week.days[2]?.isToday).toBe(true);
    expect(week.days[2]?.activities.map((item) => item.id)).toEqual(['done', 'wed']);
    expect(week.unscheduled.map((item) => item.id)).toEqual(['open']);
    expect(week.days.flatMap((day) => day.activities.map((item) => item.id))).not.toContain('later');
  });

  it('shifts by whole weeks', () => {
    const start = startOfWeek(NOW);
    expect(shiftWeek(start, 1)).toEqual(new Date(2026, 7, 17));
    expect(shiftWeek(start, -1)).toEqual(new Date(2026, 7, 3));
  });
});
