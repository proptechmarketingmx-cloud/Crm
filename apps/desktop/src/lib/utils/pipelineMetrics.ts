import type { Deal, DealStage } from '$lib/api/deals';
import { DEAL_STAGES } from '$lib/api/deals';
import { sumByCurrency, type CurrencyTotal } from '$lib/utils/currency';
import {
  dealStageAgeDays,
  isDealClosed,
  type PipelineGuidance,
  weightedForecastValue,
} from '$lib/utils/pipelineGuidance';

export type StageFocus =
  | 'empty'
  | 'overdue'
  | 'needsFollowUp'
  | 'stale'
  | 'onTrack'
  | 'closedWon'
  | 'closedLost';

export interface StageForecastMetric {
  stage: DealStage;
  dealCount: number;
  dealShare: number;
  openDealShare: number;
  pipelineValueByCurrency: CurrencyTotal[];
  weightedForecastByCurrency: CurrencyTotal[];
  averageProbability: number | null;
  averageStageAgeDays: number | null;
  overdueCount: number;
  needsFollowUpCount: number;
  staleCount: number;
  onTrackCount: number;
  atRiskCount: number;
  focus: StageFocus;
}

export interface PipelineForecastMetrics {
  openDealCount: number;
  closedWonCount: number;
  closedLostCount: number;
  winRate: number | null;
  openPipelineByCurrency: CurrencyTotal[];
  weightedForecastByCurrency: CurrencyTotal[];
  closingNext30DaysByCurrency: CurrencyTotal[];
  overdueCloseDateCount: number;
  noCloseDateCount: number;
  laterCloseDateCount: number;
  atRiskCount: number;
  overdueCount: number;
  needsFollowUpCount: number;
  staleCount: number;
  onTrackCount: number;
  stageMetrics: StageForecastMetric[];
  focusStage: StageForecastMetric | null;
}

function parseTime(value: string | null | undefined): number | null {
  const time = Date.parse(value ?? '');
  return Number.isFinite(time) ? time : null;
}

function startOfUtcDay(time: number): number {
  const date = new Date(time);
  return Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate());
}

function isExpectedNext30Days(deal: Pick<Deal, 'expectedCloseDate'>, now: Date): boolean {
  const closeTime = parseTime(deal.expectedCloseDate);
  if (closeTime === null) {
    return false;
  }

  const start = startOfUtcDay(now.getTime());
  const end = start + (30 * 24 * 60 * 60 * 1000);
  return closeTime >= start && closeTime < end;
}

function average(values: number[]): number | null {
  if (values.length === 0) {
    return null;
  }

  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function focusForCounts({
  dealCount,
  overdueCount,
  needsFollowUpCount,
  staleCount,
}: Pick<StageForecastMetric, 'dealCount' | 'overdueCount' | 'needsFollowUpCount' | 'staleCount'>): StageFocus {
  if (dealCount === 0) {
    return 'empty';
  }

  if (overdueCount > 0) {
    return 'overdue';
  }

  if (needsFollowUpCount > 0) {
    return 'needsFollowUp';
  }

  if (staleCount > 0) {
    return 'stale';
  }

  return 'onTrack';
}

function rankStageForFocus(stage: StageForecastMetric): number {
  if (stage.dealCount === 0) {
    return -1;
  }

  return (stage.overdueCount * 1000)
    + (stage.needsFollowUpCount * 100)
    + (stage.staleCount * 10)
    + stage.dealCount;
}

export function buildPipelineForecastMetrics({
  deals,
  guidanceByDealId = {},
  now = new Date(),
}: {
  deals: Deal[];
  guidanceByDealId?: Record<string, PipelineGuidance | undefined>;
  now?: Date;
}): PipelineForecastMetrics {
  const openDeals = deals.filter((deal) => !isDealClosed(deal));
  const overdueCloseDateCount = openDeals.filter((deal) => {
    const closeTime = parseTime(deal.expectedCloseDate);
    return closeTime !== null && closeTime < startOfUtcDay(now.getTime());
  }).length;
  const noCloseDateCount = openDeals.filter((deal) => parseTime(deal.expectedCloseDate) === null).length;
  const closedWonCount = deals.filter((deal) => deal.stage === 'closedWon').length;
  const closedLostCount = deals.filter((deal) => deal.stage === 'closedLost').length;
  const closedCount = closedWonCount + closedLostCount;
  const winRate = closedCount > 0 ? closedWonCount / closedCount : null;

  let overdueCount = 0;
  let needsFollowUpCount = 0;
  let staleCount = 0;
  let onTrackCount = 0;

  for (const deal of openDeals) {
    const guidance = guidanceByDealId[deal.id];
    if (guidance?.state === 'overdue') overdueCount += 1;
    if (guidance?.state === 'needsFollowUp') needsFollowUpCount += 1;
    if (guidance?.state === 'stale') staleCount += 1;
    if (guidance?.state === 'onTrack') onTrackCount += 1;
  }

  const stageMetrics = DEAL_STAGES.map((stage) => {
    const stageDeals = deals.filter((deal) => deal.stage === stage);
    const openStageDeals = stageDeals.filter((deal) => !isDealClosed(deal));
    let stageOverdueCount = 0;
    let stageNeedsFollowUpCount = 0;
    let stageStaleCount = 0;
    let stageOnTrackCount = 0;

    for (const deal of openStageDeals) {
      const guidance = guidanceByDealId[deal.id];
      if (guidance?.state === 'overdue') stageOverdueCount += 1;
      if (guidance?.state === 'needsFollowUp') stageNeedsFollowUpCount += 1;
      if (guidance?.state === 'stale') stageStaleCount += 1;
      if (guidance?.state === 'onTrack') stageOnTrackCount += 1;
    }

    const ageValues = stageDeals
      .map((deal) => dealStageAgeDays(deal, now))
      .filter((value): value is number => value !== null);

    const dealCount = stageDeals.length;
    const metric: StageForecastMetric = {
      stage,
      dealCount,
      dealShare: deals.length > 0 ? dealCount / deals.length : 0,
      openDealShare: openDeals.length > 0 ? openStageDeals.length / openDeals.length : 0,
      pipelineValueByCurrency: sumByCurrency(stageDeals),
      weightedForecastByCurrency: sumByCurrency(
        openStageDeals.map((deal) => ({
          currency: deal.currency,
          value: weightedForecastValue(deal),
        })),
      ),
      averageProbability: average(
        openStageDeals
          .map((deal) => deal.probability)
          .filter((value) => Number.isFinite(value)),
      ),
      averageStageAgeDays: average(ageValues),
      overdueCount: stageOverdueCount,
      needsFollowUpCount: stageNeedsFollowUpCount,
      staleCount: stageStaleCount,
      onTrackCount: stageOnTrackCount,
      atRiskCount: stageOverdueCount + stageNeedsFollowUpCount + stageStaleCount,
      focus: stage === 'closedWon' ? 'closedWon' : stage === 'closedLost' ? 'closedLost' : 'empty',
    };

    if (stage !== 'closedWon' && stage !== 'closedLost') {
      metric.focus = focusForCounts(metric);
    }
    return metric;
  });

  const rankedFocusStages = [...stageMetrics]
    .filter((stage) => stage.dealCount > 0 && stage.stage !== 'closedWon' && stage.stage !== 'closedLost')
    .sort((left, right) => {
      const byRisk = rankStageForFocus(right) - rankStageForFocus(left);
      if (byRisk !== 0) return byRisk;
      return right.dealCount - left.dealCount;
    });

  return {
    openDealCount: openDeals.length,
    closedWonCount,
    closedLostCount,
    winRate,
    openPipelineByCurrency: sumByCurrency(openDeals),
    weightedForecastByCurrency: sumByCurrency(
      openDeals.map((deal) => ({
        currency: deal.currency,
        value: weightedForecastValue(deal),
      })),
    ),
    closingNext30DaysByCurrency: sumByCurrency(
      openDeals
        .filter((deal) => isExpectedNext30Days(deal, now))
        .map((deal) => ({
          currency: deal.currency,
          value: weightedForecastValue(deal),
        })),
    ),
    overdueCloseDateCount,
    noCloseDateCount,
    laterCloseDateCount: openDeals.length - overdueCloseDateCount - noCloseDateCount
      - openDeals.filter((deal) => isExpectedNext30Days(deal, now)).length,
    atRiskCount: overdueCount + needsFollowUpCount + staleCount,
    overdueCount,
    needsFollowUpCount,
    staleCount,
    onTrackCount,
    stageMetrics,
    focusStage: rankedFocusStages[0] ?? null,
  };
}
