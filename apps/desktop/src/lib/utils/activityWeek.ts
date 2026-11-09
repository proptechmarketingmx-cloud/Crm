import type { Activity } from '$lib/api/activities';
import {
  localDayKey,
  parseLocalDueDay,
  sortActivitiesForWorkbench,
} from './activityWorkbench';

export interface ActivityWeekDay {
  date: string;
  isToday: boolean;
  activities: Activity[];
}

export interface ActivityWeek {
  weekStart: string;
  weekEnd: string;
  days: ActivityWeekDay[];
  unscheduled: Activity[];
}

/** Monday of the local week that contains `date`. */
export function startOfWeek(date: Date): Date {
  const local = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  const weekday = local.getDay();
  const mondayOffset = weekday === 0 ? -6 : 1 - weekday;
  return new Date(local.getFullYear(), local.getMonth(), local.getDate() + mondayOffset);
}

export function shiftWeek(weekStart: Date, weeks: number): Date {
  return new Date(
    weekStart.getFullYear(),
    weekStart.getMonth(),
    weekStart.getDate() + weeks * 7,
  );
}

export function buildActivityWeek(
  activities: Activity[],
  weekStart: Date,
  now: Date = new Date(),
): ActivityWeek {
  const todayKey = localDayKey(now);
  const days: ActivityWeekDay[] = Array.from({ length: 7 }, (_, index) => {
    const day = new Date(weekStart.getFullYear(), weekStart.getMonth(), weekStart.getDate() + index);
    const date = localDayKey(day);
    return {
      date,
      isToday: date === todayKey,
      activities: [],
    };
  });

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

  return {
    weekStart: days[0]?.date ?? localDayKey(weekStart),
    weekEnd: days[6]?.date ?? localDayKey(weekStart),
    days,
    unscheduled: sortActivitiesForWorkbench(unscheduled),
  };
}
