import type { Activity, ActivityStatus } from '$lib/api/activities';

export type ActivityDueBucket =
  | 'overdue'
  | 'today'
  | 'thisWeek'
  | 'later'
  | 'unscheduled'
  | 'completed';

export interface ActivityBucket {
  bucket: ActivityDueBucket;
  activities: Activity[];
}

export interface ActivityWorkbenchSummary {
  overdue: number;
  today: number;
  thisWeek: number;
  later: number;
  unscheduled: number;
  completed: number;
  open: number;
}

export interface ActivityWorkbench {
  buckets: ActivityBucket[];
  summary: ActivityWorkbenchSummary;
}

export const ACTIVITY_DUE_BUCKETS: ActivityDueBucket[] = [
  'overdue',
  'today',
  'thisWeek',
  'later',
  'unscheduled',
  'completed',
];

const MS_PER_DAY = 24 * 60 * 60 * 1000;

export function localDayKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

function localDayStart(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

export function parseLocalDueDay(value: string | null | undefined): number | null {
  if (!value) {
    return null;
  }

  const dateOnly = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value.trim());
  if (dateOnly) {
    const [, year, month, day] = dateOnly;
    return new Date(Number(year), Number(month) - 1, Number(day)).getTime();
  }

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return null;
  }

  return localDayStart(parsed);
}

function activityUpdatedTime(activity: Pick<Activity, 'updatedAt' | 'createdAt'>): number {
  const updated = Date.parse(activity.updatedAt);
  if (Number.isFinite(updated)) {
    return updated;
  }

  const created = Date.parse(activity.createdAt);
  return Number.isFinite(created) ? created : 0;
}

function bucketRank(bucket: ActivityDueBucket): number {
  return ACTIVITY_DUE_BUCKETS.indexOf(bucket);
}

export function addLocalDays(now: Date, days: number): string {
  const target = new Date(now.getFullYear(), now.getMonth(), now.getDate() + days);
  return localDayKey(target);
}

export function bucketActivityByDueDate(
  activity: Pick<Activity, 'dueDate' | 'status'>,
  now: Date = new Date(),
): ActivityDueBucket {
  if (activity.status === 'completed') {
    return 'completed';
  }

  const dueDay = parseLocalDueDay(activity.dueDate);
  if (dueDay === null) {
    return 'unscheduled';
  }

  const today = localDayStart(now);
  const diffDays = Math.floor((dueDay - today) / MS_PER_DAY);

  if (diffDays < 0 || activity.status === 'overdue') {
    return 'overdue';
  }

  if (diffDays === 0) {
    return 'today';
  }

  if (diffDays <= 7) {
    return 'thisWeek';
  }

  return 'later';
}

export function sortActivitiesForWorkbench(activities: Activity[]): Activity[] {
  return [...activities].sort((left, right) => {
    const leftDue = parseLocalDueDay(left.dueDate);
    const rightDue = parseLocalDueDay(right.dueDate);

    if (leftDue !== null && rightDue !== null) {
      const byDue = leftDue - rightDue;
      if (byDue !== 0) return byDue;
    }

    if (leftDue !== null) {
      return -1;
    }

    if (rightDue !== null) {
      return 1;
    }

    return activityUpdatedTime(right) - activityUpdatedTime(left);
  });
}

export function buildActivityWorkbench(
  activities: Activity[],
  now: Date = new Date(),
): ActivityWorkbench {
  const grouped: Record<ActivityDueBucket, Activity[]> = {
    overdue: [],
    today: [],
    thisWeek: [],
    later: [],
    unscheduled: [],
    completed: [],
  };

  for (const activity of activities) {
    grouped[bucketActivityByDueDate(activity, now)].push(activity);
  }

  const buckets = ACTIVITY_DUE_BUCKETS.map((bucket) => ({
    bucket,
    activities: sortActivitiesForWorkbench(grouped[bucket]),
  }));

  return {
    buckets,
    summary: {
      overdue: grouped.overdue.length,
      today: grouped.today.length,
      thisWeek: grouped.thisWeek.length,
      later: grouped.later.length,
      unscheduled: grouped.unscheduled.length,
      completed: grouped.completed.length,
      open: activities.filter((activity) => activity.status !== 'completed').length,
    },
  };
}

export function statusForDueBucket(bucket: ActivityDueBucket): ActivityStatus | '' {
  if (bucket === 'overdue' || bucket === 'completed') {
    return bucket;
  }

  return '';
}
