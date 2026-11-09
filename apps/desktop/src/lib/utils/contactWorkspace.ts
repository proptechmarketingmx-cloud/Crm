import type { Activity } from '$lib/api/activities';
import type { Deal } from '$lib/api/deals';
import {
  filterActivitiesByRelationship,
  type ActivityLinkIndex,
} from '$lib/utils/activityRelationships';
import { activitySortTime, isOpenDeal } from '$lib/utils/organizationWorkspace';

export type ContactHealthState =
  | 'loading'
  | 'overdue'
  | 'needsFollowUp'
  | 'onTrack'
  | 'nurture';

export type ContactHealthTone = 'neutral' | 'danger' | 'warning' | 'success';

export interface ContactHealth {
  state: ContactHealthState;
  tone: ContactHealthTone;
  subject?: string;
}

export interface ContactListInsight {
  health: ContactHealth;
  nextActivity: Activity | null;
}

export function filterContactDeals(
  deals: Pick<Deal, 'contactId'>[],
  contactId: string,
): Deal[] {
  return deals.filter((deal): deal is Deal => deal.contactId === contactId);
}

export function nextContactActivity(activities: Activity[]): Activity | null {
  return [...activities]
    .filter((activity) => activity.status !== 'completed')
    .sort((left, right) => activitySortTime(left) - activitySortTime(right))[0] ?? null;
}

export function deriveContactHealth({
  isLoading,
  openDealCount,
  pendingActivities,
  overdueActivities,
  nextActivity,
}: {
  isLoading: boolean;
  openDealCount: number;
  pendingActivities: Activity[];
  overdueActivities: Activity[];
  nextActivity: Activity | null;
}): ContactHealth {
  if (isLoading) {
    return { state: 'loading', tone: 'neutral' };
  }

  if (overdueActivities.length > 0) {
    return {
      state: 'overdue',
      tone: 'danger',
      subject: overdueActivities[0].subject,
    };
  }

  if (openDealCount > 0 && pendingActivities.length === 0) {
    return { state: 'needsFollowUp', tone: 'warning' };
  }

  if (nextActivity) {
    return {
      state: 'onTrack',
      tone: 'success',
      subject: nextActivity.subject,
    };
  }

  return { state: 'nurture', tone: 'neutral' };
}

export function buildContactListInsight({
  contactId,
  deals,
  activities,
  linkIndex,
  isLoading,
}: {
  contactId: string;
  deals: Pick<Deal, 'contactId' | 'stage'>[];
  activities: Activity[];
  linkIndex: ActivityLinkIndex;
  isLoading: boolean;
}): ContactListInsight {
  const contactActivities = filterActivitiesByRelationship(activities, linkIndex, { contactId });
  const pendingActivities = contactActivities.filter((activity) => activity.status !== 'completed');
  const nextActivity = nextContactActivity(contactActivities);

  return {
    health: deriveContactHealth({
      isLoading,
      openDealCount: filterContactDeals(deals, contactId).filter(isOpenDeal).length,
      pendingActivities,
      overdueActivities: pendingActivities.filter((activity) => activity.status === 'overdue'),
      nextActivity,
    }),
    nextActivity,
  };
}
