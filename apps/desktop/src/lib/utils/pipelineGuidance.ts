import type { Activity } from '$lib/api/activities';
import type { Deal } from '$lib/api/deals';

export type PipelineGuidanceState =
  | 'closedWon'
  | 'closedLost'
  | 'overdue'
  | 'needsFollowUp'
  | 'stale'
  | 'onTrack';

export type PipelineGuidanceTone = 'neutral' | 'success' | 'danger' | 'warning';

export interface PipelineGuidance {
  state: PipelineGuidanceState;
  tone: PipelineGuidanceTone;
  stageAgeDays: number | null;
  weightedForecastValue: number;
  nextActivity: Activity | null;
}

export const PIPELINE_STALE_DEAL_DAYS = 14;

function parseTime(value: string | null | undefined): number | null {
  const time = Date.parse(value ?? '');
  return Number.isFinite(time) ? time : null;
}

function startOfUtcDay(time: number): number {
  const date = new Date(time);
  return Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate());
}

function daysBetween(startTime: number, endTime: number): number {
  const msPerDay = 24 * 60 * 60 * 1000;
  return Math.max(0, Math.floor((startOfUtcDay(endTime) - startOfUtcDay(startTime)) / msPerDay));
}

export function weightedForecastValue(deal: Pick<Deal, 'value' | 'probability'>): number {
  const probability = Number.isFinite(deal.probability) ? Math.min(Math.max(deal.probability, 0), 100) : 0;
  const value = Number.isFinite(deal.value) ? deal.value : 0;
  return value * (probability / 100);
}

export function isDealClosed(deal: Pick<Deal, 'stage'>): boolean {
  return deal.stage === 'closedWon' || deal.stage === 'closedLost';
}

export function dealStageAgeDays(
  deal: Pick<Deal, 'updatedAt'>,
  now: Date = new Date(),
): number | null {
  const updatedTime = parseTime(deal.updatedAt);
  if (updatedTime === null) {
    return null;
  }

  return daysBetween(updatedTime, now.getTime());
}

export function nextDealActivity(
  activities: Activity[],
  _now: Date = new Date(),
): Activity | null {
  return [...activities]
    .filter((activity) => activity.status !== 'completed')
    .sort((left, right) => {
      const leftDue = parseTime(left.dueDate);
      const rightDue = parseTime(right.dueDate);

      if (leftDue !== null && rightDue !== null) {
        return leftDue - rightDue;
      }

      if (leftDue !== null) {
        return -1;
      }

      if (rightDue !== null) {
        return 1;
      }

      return (parseTime(right.updatedAt) ?? parseTime(right.createdAt) ?? 0)
        - (parseTime(left.updatedAt) ?? parseTime(left.createdAt) ?? 0);
    })[0] ?? null;
}

export function isActivityOverdue(
  activity: Pick<Activity, 'status' | 'dueDate'> | null,
  now: Date = new Date(),
): boolean {
  if (!activity || activity.status === 'completed') {
    return false;
  }

  if (activity.status === 'overdue') {
    return true;
  }

  const dueTime = parseTime(activity.dueDate);
  return dueTime !== null && dueTime < startOfUtcDay(now.getTime());
}

export function derivePipelineGuidance({
  deal,
  activities,
  now = new Date(),
  staleDays = PIPELINE_STALE_DEAL_DAYS,
}: {
  deal: Deal;
  activities: Activity[];
  now?: Date;
  staleDays?: number;
}): PipelineGuidance {
  const stageAgeDays = dealStageAgeDays(deal, now);
  const nextActivity = nextDealActivity(activities, now);
  const weightedForecast = weightedForecastValue(deal);

  if (deal.stage === 'closedWon') {
    return {
      state: 'closedWon',
      tone: 'success',
      stageAgeDays,
      weightedForecastValue: weightedForecast,
      nextActivity,
    };
  }

  if (deal.stage === 'closedLost') {
    return {
      state: 'closedLost',
      tone: 'neutral',
      stageAgeDays,
      weightedForecastValue: weightedForecast,
      nextActivity,
    };
  }

  if (isActivityOverdue(nextActivity, now)) {
    return {
      state: 'overdue',
      tone: 'danger',
      stageAgeDays,
      weightedForecastValue: weightedForecast,
      nextActivity,
    };
  }

  if (!nextActivity) {
    return {
      state: 'needsFollowUp',
      tone: 'warning',
      stageAgeDays,
      weightedForecastValue: weightedForecast,
      nextActivity,
    };
  }

  if (stageAgeDays !== null && stageAgeDays >= staleDays) {
    return {
      state: 'stale',
      tone: 'warning',
      stageAgeDays,
      weightedForecastValue: weightedForecast,
      nextActivity,
    };
  }

  return {
    state: 'onTrack',
    tone: 'success',
    stageAgeDays,
    weightedForecastValue: weightedForecast,
    nextActivity,
  };
}
