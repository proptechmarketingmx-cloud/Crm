import type { Activity, ActivityLink } from '$lib/api/activities';
import type { Contact } from '$lib/api/contacts';
import type { Deal } from '$lib/api/deals';

export type OrganizationHealthState =
  | 'loading'
  | 'overdue'
  | 'needsFollowUp'
  | 'onTrack'
  | 'nurture';

export type OrganizationHealthTone = 'neutral' | 'danger' | 'warning' | 'success';

export interface OrganizationHealth {
  state: OrganizationHealthState;
  tone: OrganizationHealthTone;
  subject?: string;
}

export interface PipelineCurrencyBucket {
  currency: string;
  value: number;
}

export function filterOrganizationContacts(
  contacts: Pick<Contact, 'organizationId' | 'type'>[],
  organizationId: string,
): Contact[] {
  return contacts.filter(
    (contact): contact is Contact =>
      contact.type !== 'org' && contact.organizationId === organizationId,
  );
}

export function isOpenDeal(deal: Pick<Deal, 'stage'>): boolean {
  return deal.stage !== 'closedWon' && deal.stage !== 'closedLost';
}

export function filterOrganizationDeals(
  deals: Pick<Deal, 'organizationId'>[],
  organizationId: string,
): Deal[] {
  return deals.filter(
    (deal): deal is Deal => deal.organizationId === organizationId,
  );
}

export function openPipelineByCurrency(
  deals: Pick<Deal, 'stage' | 'currency' | 'value'>[],
  fallbackCurrency: string,
): PipelineCurrencyBucket[] {
  const buckets = new Map<string, number>();

  for (const deal of deals) {
    if (!isOpenDeal(deal)) {
      continue;
    }

    const currency = deal.currency || fallbackCurrency;
    buckets.set(currency, (buckets.get(currency) ?? 0) + deal.value);
  }

  return [...buckets.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([currency, value]) => ({ currency, value }));
}

export function filterOrganizationActivities(
  activities: Activity[],
  linkIndex: Record<string, ActivityLink[]>,
  organizationId: string,
): Activity[] {
  return activities.filter((activity) =>
    (linkIndex[activity.id] ?? []).some(
      (link) =>
        link.entityType === 'organization' &&
        link.entityId === organizationId &&
        !link.deletedAt,
    ),
  );
}

export function activitySortTime(activity: Pick<Activity, 'dueDate' | 'updatedAt'>): number {
  const dueTime = Date.parse(activity.dueDate ?? '');
  if (Number.isFinite(dueTime)) {
    return dueTime;
  }

  return Number.MAX_SAFE_INTEGER;
}

export function activityUpdatedTime(activity: Pick<Activity, 'updatedAt' | 'createdAt'>): number {
  const updatedTime = Date.parse(activity.updatedAt);
  if (Number.isFinite(updatedTime)) {
    return updatedTime;
  }

  const createdTime = Date.parse(activity.createdAt);
  return Number.isFinite(createdTime) ? createdTime : 0;
}

export function nextOrganizationActivity(activities: Activity[]): Activity | null {
  return [...activities]
    .filter((activity) => activity.status !== 'completed')
    .sort((left, right) => activitySortTime(left) - activitySortTime(right))[0] ?? null;
}

export function recentOrganizationActivity(activities: Activity[]): Activity | null {
  return [...activities]
    .sort((left, right) => activityUpdatedTime(right) - activityUpdatedTime(left))[0] ?? null;
}

export interface OrganizationListInsight {
  health: OrganizationHealth;
  nextActivity: Activity | null;
}

export function buildOrganizationListInsight({
  organizationId,
  deals,
  activities,
  linkIndex,
  isLoading,
}: {
  organizationId: string;
  deals: Pick<Deal, 'organizationId' | 'stage'>[];
  activities: Activity[];
  linkIndex: Record<string, ActivityLink[]>;
  isLoading: boolean;
}): OrganizationListInsight {
  const organizationActivities = filterOrganizationActivities(
    activities,
    linkIndex,
    organizationId,
  );
  const pendingActivities = organizationActivities.filter(
    (activity) => activity.status !== 'completed',
  );
  const nextActivity = nextOrganizationActivity(organizationActivities);

  return {
    health: deriveOrganizationHealth({
      isLoading,
      openDealCount: filterOrganizationDeals(deals, organizationId).filter(isOpenDeal).length,
      pendingActivities,
      overdueActivities: pendingActivities.filter((activity) => activity.status === 'overdue'),
      nextActivity,
    }),
    nextActivity,
  };
}

export function deriveOrganizationHealth({
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
}): OrganizationHealth {
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
