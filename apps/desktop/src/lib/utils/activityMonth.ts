import type { Activity } from '$lib/api/activities';
import { localDayKey, parseLocalDueDay, sortActivitiesForWorkbench } from './activityWorkbench';
import { startOfWeek } from './activityWeek';

export interface ActivityMonthDay {
  date: string;
  isToday: boolean;
  inMonth: boolean;
  activities: Activity[];
}

export interface ActivityMonth {
  monthStart: string;
  monthEnd: string;
  weeks: ActivityMonthDay[][];
  unscheduled: Activity[];
}

export function startOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

export function shiftMonth(monthStart: Date, months: number): Date {
  return new Date(monthStart.getFullYear(), monthStart.getMonth() + months, 1);
}

export function buildActivityMonth(
  activities: Activity[],
  monthStart: Date,
  now: Date = new Date(),
): ActivityMonth {
  const firstOfMonth = startOfMonth(monthStart);
  const lastOfMonth = new Date(firstOfMonth.getFullYear(), firstOfMonth.getMonth() + 1, 0);
  const gridStart = startOfWeek(firstOfMonth);
  const gridEnd = startOfWeek(lastOfMonth);
  const lastCell = new Date(gridEnd.getFullYear(), gridEnd.getMonth(), gridEnd.getDate() + 6);
  const todayKey = localDayKey(now);
  const monthPrefix = localDayKey(firstOfMonth).slice(0, 7);

  const days: ActivityMonthDay[] = [];
  for (
    let cursor = new Date(gridStart.getFullYear(), gridStart.getMonth(), gridStart.getDate());
    cursor.getTime() <= lastCell.getTime();
    cursor = new Date(cursor.getFullYear(), cursor.getMonth(), cursor.getDate() + 1)
  ) {
    const date = localDayKey(cursor);
    days.push({
      date,
      isToday: date === todayKey,
      inMonth: date.startsWith(monthPrefix),
      activities: [],
    });
  }

  const byDate = new Map(days.map((day) => [day.date, day]));
  const unscheduled: Activity[] = [];

  for (const activity of activities) {
    const dueTime = parseLocalDueDay(activity.dueDate);
    if (dueTime === null) {
      if (activity.status !== 'completed') {
        unscheduled.push(activity);
      }
      continue;
    }

    const dueKey = localDayKey(new Date(dueTime));
    const day = byDate.get(dueKey);
    if (day) {
      day.activities.push(activity);
    }
  }

  for (const day of days) {
    day.activities = sortActivitiesForWorkbench(day.activities);
  }

  const weeks: ActivityMonthDay[][] = [];
  for (let index = 0; index < days.length; index += 7) {
    weeks.push(days.slice(index, index + 7));
  }

  return {
    monthStart: localDayKey(firstOfMonth),
    monthEnd: localDayKey(lastOfMonth),
    weeks,
    unscheduled: sortActivitiesForWorkbench(unscheduled),
  };
}
