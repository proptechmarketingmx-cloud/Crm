import { describe, expect, it } from 'vitest';
import type { Activity } from '$lib/api/activities';
import { deriveRecordNextStep, shouldShowSecondaryFollowUp } from './recordNextStep';

function activity(overrides: Partial<Activity> = {}): Activity {
  return {
    id: 'activity-1',
    type: 'task',
    subject: 'Follow up',
    notes: null,
    dueDate: '2026-07-10',
    completedAt: null,
    status: 'pending',
    contactId: 'contact-1',
    contactName: null,
    dealId: null,
    dealName: null,
    createdAt: '2026-07-01T00:00:00Z',
    updatedAt: '2026-07-01T00:00:00Z',
    ...overrides,
  };
}

describe('deriveRecordNextStep', () => {
  it('completes overdue work before any other record action', () => {
    const overdue = activity({
      id: 'overdue',
      subject: 'Past due clinic check-in',
      status: 'overdue',
    });

    expect(
      deriveRecordNextStep({
        recordKind: 'contact',
        isLoading: false,
        isLead: true,
        openDealCount: 1,
        overdueActivities: [overdue],
        nextActivity: overdue,
      }),
    ).toEqual({
      kind: 'completeOverdue',
      tone: 'danger',
      action: 'complete',
      activityId: 'overdue',
      subject: 'Past due clinic check-in',
    });
  });

  it('asks to convert a lead after overdue work is clear', () => {
    expect(
      deriveRecordNextStep({
        recordKind: 'contact',
        isLoading: false,
        isLead: true,
        openDealCount: 1,
        nextActivity: null,
      }),
    ).toMatchObject({
      kind: 'convertLead',
      tone: 'warning',
      action: 'convert',
    });
  });

  it('asks for a follow-up when open work has no next activity', () => {
    expect(
      deriveRecordNextStep({
        recordKind: 'organization',
        isLoading: false,
        openDealCount: 2,
        nextActivity: null,
      }),
    ).toMatchObject({
      kind: 'addFollowUp',
      action: 'addFollowUp',
      tone: 'warning',
    });
  });

  it('asks an open deal to set a close date before treating it as on track', () => {
    const next = activity({ subject: 'Site visit' });

    expect(
      deriveRecordNextStep({
        recordKind: 'deal',
        isLoading: false,
        nextActivity: next,
        expectedCloseDate: null,
      }),
    ).toEqual({
      kind: 'setExpectedClose',
      tone: 'warning',
      action: 'setExpectedClose',
      activityId: next.id,
      subject: 'Site visit',
    });
  });

  it('flags a stale deal after close date and follow-up are present', () => {
    const next = activity({ subject: 'Confirm proposal' });

    expect(
      deriveRecordNextStep({
        recordKind: 'deal',
        isLoading: false,
        nextActivity: next,
        expectedCloseDate: '2026-08-01',
        isStale: true,
      }),
    ).toMatchObject({
      kind: 'stale',
      tone: 'warning',
      action: 'none',
      subject: 'Confirm proposal',
    });
  });

  it('keeps scheduled work quiet when the record is on track', () => {
    const next = activity({ subject: 'Call Maya' });

    expect(
      deriveRecordNextStep({
        recordKind: 'contact',
        isLoading: false,
        nextActivity: next,
      }),
    ).toEqual({
      kind: 'onTrack',
      tone: 'success',
      action: 'none',
      activityId: next.id,
      subject: 'Call Maya',
    });
  });

  it('uses nurture follow-up when there is no open work', () => {
    expect(
      deriveRecordNextStep({
        recordKind: 'contact',
        isLoading: false,
        openDealCount: 0,
        nextActivity: null,
      }),
    ).toMatchObject({
      kind: 'nurture',
      tone: 'neutral',
      action: 'addFollowUp',
    });
  });

  it('pauses guidance on closed deals even if activity is still loading', () => {
    expect(
      deriveRecordNextStep({
        recordKind: 'deal',
        isLoading: true,
        isClosedWon: true,
        nextActivity: activity({ status: 'overdue' }),
      }).kind,
    ).toBe('closedWon');

    expect(
      deriveRecordNextStep({
        recordKind: 'deal',
        isLoading: false,
        isClosedLost: true,
        overdueActivities: [activity({ status: 'overdue' })],
      }).kind,
    ).toBe('closedLost');
  });
});

describe('shouldShowSecondaryFollowUp', () => {
  it('hides the extra follow-up button when the strip already is that action', () => {
    expect(
      shouldShowSecondaryFollowUp(
        deriveRecordNextStep({
          recordKind: 'deal',
          isLoading: false,
          nextActivity: null,
        }),
      ),
    ).toBe(false);
  });

  it('keeps a quieter follow-up when the primary action is something else', () => {
    expect(
      shouldShowSecondaryFollowUp(
        deriveRecordNextStep({
          recordKind: 'contact',
          isLoading: false,
          isLead: true,
        }),
      ),
    ).toBe(true);
  });
});
