import { describe, expect, it } from 'vitest';
import type { Activity } from '$lib/api/activities';
import { buildActivityMonth, shiftMonth, startOfMonth } from './activityMonth';

const NOW = new Date('2026-08-12T16:30:00'); // Wednesday in August

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

describe('activity month', () => {
  it('starts the month on the first local day', () => {
    expect(startOfMonth(NOW)).toEqual(new Date(2026, 7, 1));
    expect(shiftMonth(startOfMonth(NOW), 1)).toEqual(new Date(2026, 8, 1));
    expect(shiftMonth(startOfMonth(NOW), -1)).toEqual(new Date(2026, 6, 1));
  });

  it('builds Monday-aligned weeks and keeps pad days outside the month', () => {
    const month = buildActivityMonth([
      activity({ id: 'first', dueDate: '2026-08-01' }),
      activity({ id: 'today', dueDate: '2026-08-12' }),
      activity({ id: 'last', dueDate: '2026-08-31' }),
      activity({ id: 'july', dueDate: '2026-07-27' }),
      activity({ id: 'open', dueDate: null }),
    ], startOfMonth(NOW), NOW);

    expect(month.monthStart).toBe('2026-08-01');
    expect(month.monthEnd).toBe('2026-08-31');
    expect(month.weeks.length).toBeGreaterThanOrEqual(5);
    expect(month.weeks[0]?.[0]?.date).toBe('2026-07-27');
    expect(month.weeks[0]?.[0]?.inMonth).toBe(false);
    expect(month.weeks[0]?.[0]?.activities.map((item) => item.id)).toEqual(['july']);
    expect(month.weeks.flat().find((day) => day.date === '2026-08-12')?.isToday).toBe(true);
    expect(month.weeks.flat().find((day) => day.date === '2026-08-01')?.inMonth).toBe(true);
    expect(month.unscheduled.map((item) => item.id)).toEqual(['open']);
  });
});
