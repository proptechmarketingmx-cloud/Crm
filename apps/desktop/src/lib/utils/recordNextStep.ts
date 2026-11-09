import type { Activity } from '$lib/api/activities';

export type RecordKind = 'contact' | 'deal' | 'organization';

export type NextStepKind =
  | 'loading'
  | 'unavailable'
  | 'completeOverdue'
  | 'convertLead'
  | 'addFollowUp'
  | 'setExpectedClose'
  | 'stale'
  | 'onTrack'
  | 'nurture'
  | 'closedWon'
  | 'closedLost';

export type NextStepAction =
  | 'complete'
  | 'convert'
  | 'addFollowUp'
  | 'setExpectedClose'
  | 'none';

export type NextStepTone = 'neutral' | 'danger' | 'warning' | 'success';

export interface RecordNextStep {
  kind: NextStepKind;
  tone: NextStepTone;
  action: NextStepAction;
  activityId: string | null;
  subject: string | null;
}

function step(
  kind: NextStepKind,
  tone: NextStepTone,
  action: NextStepAction,
  activity: Activity | null = null,
): RecordNextStep {
  return {
    kind,
    tone,
    action,
    activityId: activity?.id ?? null,
    subject: activity?.subject ?? null,
  };
}

export function deriveRecordNextStep({
  recordKind,
  isLoading,
  unavailable = false,
  isLead = false,
  isClosedWon = false,
  isClosedLost = false,
  openDealCount = 0,
  overdueActivities = [],
  nextActivity = null,
  expectedCloseDate = null,
  isStale = false,
}: {
  recordKind: RecordKind;
  isLoading: boolean;
  unavailable?: boolean;
  isLead?: boolean;
  isClosedWon?: boolean;
  isClosedLost?: boolean;
  openDealCount?: number;
  overdueActivities?: Activity[];
  nextActivity?: Activity | null;
  expectedCloseDate?: string | null;
  isStale?: boolean;
}): RecordNextStep {
  if (recordKind === 'deal' && isClosedWon) {
    return step('closedWon', 'success', 'none');
  }

  if (recordKind === 'deal' && isClosedLost) {
    return step('closedLost', 'neutral', 'none');
  }

  if (isLoading) {
    return step('loading', 'neutral', 'none');
  }

  if (unavailable) {
    return step('unavailable', 'neutral', 'none');
  }

  const overdue = overdueActivities[0] ?? null;
  if (overdue) {
    return step('completeOverdue', 'danger', 'complete', overdue);
  }

  if (recordKind === 'contact' && isLead) {
    return step('convertLead', 'warning', 'convert', nextActivity);
  }

  const hasOpenWork = recordKind === 'deal' || openDealCount > 0;
  if (hasOpenWork && !nextActivity) {
    return step('addFollowUp', 'warning', 'addFollowUp');
  }

  if (recordKind === 'deal' && !expectedCloseDate) {
    return step('setExpectedClose', 'warning', 'setExpectedClose', nextActivity);
  }

  if (recordKind === 'deal' && isStale) {
    return step('stale', 'warning', 'none', nextActivity);
  }

  if (nextActivity) {
    return step('onTrack', 'success', 'none', nextActivity);
  }

  return step('nurture', 'neutral', 'addFollowUp');
}

export function shouldShowSecondaryFollowUp(nextStep: RecordNextStep): boolean {
  return (
    nextStep.action !== 'addFollowUp' &&
    nextStep.kind !== 'loading' &&
    nextStep.kind !== 'unavailable' &&
    nextStep.kind !== 'closedWon' &&
    nextStep.kind !== 'closedLost'
  );
}
