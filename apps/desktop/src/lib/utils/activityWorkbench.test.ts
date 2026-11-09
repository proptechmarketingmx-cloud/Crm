import { describe, expect, it } from 'vitest';
import type { Activity } from '$lib/api/activities';
import {
  addLocalDays,
  bucketActivityByDueDate,
  buildActivityWorkbench,
  statusForDueBucket,
} from '$lib/utils/activityWorkbench';

const NOW = new Date('2026-07-08T16:30:00');

function activity(overrides: Partial<Activity>): Activity {
  return {
    id: 'activity-1',
    type: 'task',
    subject: 'Follow up',
    notes: null,
    dueDate: '2026-07-08',
    completedAt: null,
    status: 'pending',
    contactId: null,
    contactName: null,
    dealId: null,
    dealName: null,
    createdAt: '2026-07-01T08:00:00Z',
    updatedAt: '2026-07-01T08:00:00Z',
    ...overrides,
  };
}

describe('activity workbench', () => {
  it('uses local-day boundaries so a date-only due date stays in today all day', () => {
    expect(
      bucketActivityByDueDate(activity({ dueDate: '2026-07-08', status: 'pending' }), NOW),
    ).toBe('today');
  });

  it('classifies overdue, this-week, later, unscheduled, and completed activities', () => {
    expect(bucketActivityByDueDate(activity({ dueDate: '2026-07-07' }), NOW)).toBe('overdue');
    expect(bucketActivityByDueDate(activity({ dueDate: '2026-07-15' }), NOW)).toBe('thisWeek');
    expect(bucketActivityByDueDate(activity({ dueDate: '2026-07-16' }), NOW)).toBe('later');
    expect(bucketActivityByDueDate(activity({ dueDate: null }), NOW)).toBe('unscheduled');
    expect(bucketActivityByDueDate(activity({ dueDate: '2026-07-07', status: 'completed' }), NOW)).toBe('completed');
  });

  it('builds bucket counts and keeps completed separate from open work', () => {
    const workbench = buildActivityWorkbench([
      activity({ id: 'overdue', dueDate: '2026-07-07' }),
      activity({ id: 'today', dueDate: '2026-07-08' }),
      activity({ id: 'week', dueDate: '2026-07-13' }),
      activity({ id: 'later', dueDate: '2026-08-01' }),
      activity({ id: 'unscheduled', dueDate: null }),
      activity({ id: 'done', dueDate: '2026-07-08', status: 'completed' }),
    ], NOW);

    expect(workbench.summary).toMatchObject({
      overdue: 1,
      today: 1,
      thisWeek: 1,
      later: 1,
      unscheduled: 1,
      completed: 1,
      open: 5,
    });
    expect(workbench.buckets.map((bucket) => bucket.bucket)).toEqual([
      'overdue',
      'today',
      'thisWeek',
      'later',
      'unscheduled',
      'completed',
    ]);
  });

  it('sorts due activities before unscheduled and then by due date', () => {
    const workbench = buildActivityWorkbench([
      activity({ id: 'third', dueDate: '2026-07-15' }),
      activity({ id: 'first', dueDate: '2026-07-09' }),
      activity({ id: 'second', dueDate: '2026-07-10' }),
    ], NOW);

    expect(workbench.buckets.find((bucket) => bucket.bucket === 'thisWeek')?.activities.map((item) => item.id))
      .toEqual(['first', 'second', 'third']);
  });

  it('generates local snooze dates and maps bucket filters to status filters', () => {
    expect(addLocalDays(NOW, 1)).toBe('2026-07-09');
    expect(statusForDueBucket('overdue')).toBe('overdue');
    expect(statusForDueBucket('completed')).toBe('completed');
    expect(statusForDueBucket('today')).toBe('');
  });
});
