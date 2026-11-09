import type { Activity } from '$lib/api/activities';
import type { Deal, DealStage } from '$lib/api/deals';
import {
  filterActivitiesByRelationship,
  type ActivityLinkIndex,
} from '$lib/utils/activityRelationships';
import {
  derivePipelineGuidance,
  PIPELINE_STALE_DEAL_DAYS,
} from '$lib/utils/pipelineGuidance';

export interface StaleDealReportRow {
  dealId: string;
  name: string;
  stage: DealStage;
  owner: string | null;
  stageAgeDays: number;
  nextActivitySubject: string | null;
  href: string;
}

export interface StaleDealReport {
  count: number;
  staleDays: number;
  rows: StaleDealReportRow[];
}

export function buildStaleDealReport({
  deals,
  activities,
  linkIndex,
  now = new Date(),
  staleDays = PIPELINE_STALE_DEAL_DAYS,
  limit = 25,
}: {
  deals: Deal[];
  activities: Activity[];
  linkIndex: ActivityLinkIndex;
  now?: Date;
  staleDays?: number;
  limit?: number;
}): StaleDealReport {
  const rows = deals
    .map((deal) => {
      const related = filterActivitiesByRelationship(activities, linkIndex, { dealId: deal.id });
      const guidance = derivePipelineGuidance({
        deal,
        activities: related,
        now,
        staleDays,
      });

      if (guidance.state !== 'stale' || guidance.stageAgeDays === null) {
        return null;
      }

      return {
        dealId: deal.id,
        name: deal.name,
        stage: deal.stage,
        owner: deal.owner?.trim() || null,
        stageAgeDays: guidance.stageAgeDays,
        nextActivitySubject: guidance.nextActivity?.subject ?? null,
        href: `/deals/${deal.id}`,
      };
    })
    .filter((row): row is StaleDealReportRow => row !== null)
    .sort((left, right) => right.stageAgeDays - left.stageAgeDays);

  return {
    count: rows.length,
    staleDays,
    rows: rows.slice(0, limit),
  };
}
