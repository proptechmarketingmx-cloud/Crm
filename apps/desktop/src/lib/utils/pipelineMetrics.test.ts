import { describe, expect, it } from 'vitest';
import type { Deal } from '$lib/api/deals';
import type { PipelineGuidance } from '$lib/utils/pipelineGuidance';
import { buildPipelineForecastMetrics } from '$lib/utils/pipelineMetrics';

const NOW = new Date('2026-07-08T12:00:00Z');

function deal(overrides: Partial<Deal>): Deal {
  return {
    id: 'deal-1',
    name: 'Solar rollout',
    value: 10000,
    currency: 'USD',
    stage: 'proposal',
    probability: 50,
    expectedCloseDate: '2026-07-20',
    contactId: null,
    organizationId: null,
    contactName: null,
    description: null,
    tags: [],
    createdAt: '2026-06-01T08:00:00Z',
    updatedAt: '2026-07-01T08:00:00Z',
    ...overrides,
  };
}

function guidance(overrides: Partial<PipelineGuidance>): PipelineGuidance {
  return {
    state: 'onTrack',
    tone: 'success',
    stageAgeDays: 7,
    weightedForecastValue: 5000,
    nextActivity: null,
    ...overrides,
  };
}

function totalForCurrency(
  totals: { currency: string; total: number }[],
  currency: string,
): number {
  return totals.find((total) => total.currency === currency)?.total ?? 0;
}

describe('pipeline forecast metrics', () => {
  it('groups open pipeline and weighted forecast by currency while excluding closed deals', () => {
    const metrics = buildPipelineForecastMetrics({
      now: NOW,
      deals: [
        deal({ id: 'usd-open', value: 10000, currency: 'USD', probability: 40, stage: 'proposal' }),
        deal({ id: 'eur-open', value: 8000, currency: 'EUR', probability: 50, stage: 'negotiation' }),
        deal({ id: 'usd-won', value: 30000, currency: 'USD', probability: 100, stage: 'closedWon' }),
      ],
    });

    expect(metrics.openDealCount).toBe(2);
    expect(metrics.closedWonCount).toBe(1);
    expect(totalForCurrency(metrics.openPipelineByCurrency, 'USD')).toBe(10000);
    expect(totalForCurrency(metrics.openPipelineByCurrency, 'EUR')).toBe(8000);
    expect(totalForCurrency(metrics.weightedForecastByCurrency, 'USD')).toBe(4000);
    expect(totalForCurrency(metrics.weightedForecastByCurrency, 'EUR')).toBe(4000);
  });

  it('keeps closed stages visible in stage counts but out of stage forecast', () => {
    const metrics = buildPipelineForecastMetrics({
      now: NOW,
      deals: [
        deal({ id: 'lead-open', stage: 'lead', value: 12000, probability: 25 }),
        deal({ id: 'won', stage: 'closedWon', value: 30000, probability: 100 }),
        deal({ id: 'lost', stage: 'closedLost', value: 5000, probability: 0 }),
      ],
    });

    const wonStage = metrics.stageMetrics.find((metric) => metric.stage === 'closedWon');
    const lostStage = metrics.stageMetrics.find((metric) => metric.stage === 'closedLost');

    expect(wonStage).toMatchObject({ dealCount: 1, focus: 'closedWon' });
    expect(lostStage).toMatchObject({ dealCount: 1, focus: 'closedLost' });
    expect(wonStage?.weightedForecastByCurrency).toEqual([]);
    expect(metrics.winRate).toBe(0.5);
  });

  it('aggregates close-date buckets from visible open deals', () => {
    const metrics = buildPipelineForecastMetrics({
      now: NOW,
      deals: [
        deal({ id: 'overdue', expectedCloseDate: '2026-07-01', probability: 50 }),
        deal({ id: 'next-30', expectedCloseDate: '2026-07-20', probability: 50 }),
        deal({ id: 'later', expectedCloseDate: '2026-09-20', probability: 50 }),
        deal({ id: 'missing', expectedCloseDate: null, probability: 50 }),
      ],
    });

    expect(metrics.overdueCloseDateCount).toBe(1);
    expect(metrics.noCloseDateCount).toBe(1);
    expect(metrics.laterCloseDateCount).toBe(1);
    expect(totalForCurrency(metrics.closingNext30DaysByCurrency, 'USD')).toBe(5000);
  });

  it('aggregates guidance risk counts by stage and selects the riskiest open stage', () => {
    const metrics = buildPipelineForecastMetrics({
      now: NOW,
      deals: [
        deal({ id: 'lead-overdue', stage: 'lead' }),
        deal({ id: 'lead-ok', stage: 'lead' }),
        deal({ id: 'proposal-follow-up', stage: 'proposal' }),
        deal({ id: 'negotiation-stale', stage: 'negotiation' }),
      ],
      guidanceByDealId: {
        'lead-overdue': guidance({ state: 'overdue', tone: 'danger' }),
        'lead-ok': guidance({ state: 'onTrack', tone: 'success' }),
        'proposal-follow-up': guidance({ state: 'needsFollowUp', tone: 'warning' }),
        'negotiation-stale': guidance({ state: 'stale', tone: 'warning' }),
      },
    });

    const lead = metrics.stageMetrics.find((metric) => metric.stage === 'lead');
    const proposal = metrics.stageMetrics.find((metric) => metric.stage === 'proposal');

    expect(metrics.atRiskCount).toBe(3);
    expect(lead).toMatchObject({ overdueCount: 1, onTrackCount: 1, atRiskCount: 1, focus: 'overdue' });
    expect(proposal).toMatchObject({ needsFollowUpCount: 1, focus: 'needsFollowUp' });
    expect(metrics.focusStage?.stage).toBe('lead');
  });

  it('derives stage averages from only the supplied visible deals', () => {
    const metrics = buildPipelineForecastMetrics({
      now: NOW,
      deals: [
        deal({ id: 'visible-a', stage: 'qualified', probability: 40, updatedAt: '2026-07-06T00:00:00Z' }),
        deal({ id: 'visible-b', stage: 'qualified', probability: 80, updatedAt: '2026-07-02T00:00:00Z' }),
      ],
    });

    const qualified = metrics.stageMetrics.find((metric) => metric.stage === 'qualified');

    expect(qualified?.averageProbability).toBe(60);
    expect(qualified?.averageStageAgeDays).toBe(4);
    expect(qualified?.dealShare).toBe(1);
  });
});
