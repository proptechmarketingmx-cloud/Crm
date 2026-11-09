import type { Activity, ActivityType } from '$lib/api/activities';
import type { Contact } from '$lib/api/contacts';
import type { Deal, DealStage } from '$lib/api/deals';
import { addLocalDays, buildActivityWorkbench } from '$lib/utils/activityWorkbench';
import { contactDisplayName } from '$lib/utils/dealRelationships';
import { isDealClosed, nextDealActivity } from '$lib/utils/pipelineGuidance';
import { matchesRecordOwner } from '$lib/utils/recordOwner';

export interface SuggestedActivityDraft {
  subject: string;
  type: ActivityType;
  dueDate: string;
  notes: string;
  contactId: string;
  organizationId: string;
  dealId: string;
}

export interface DealStageFollowUpSuggestion {
  id: string;
  dealId: string;
  dealName: string;
  fromStage: DealStage;
  toStage: DealStage;
  draft: SuggestedActivityDraft;
}

export type DashboardAttentionKind =
  | 'overdue'
  | 'today'
  | 'dealNeedsFollowUp'
  | 'leadWaiting';

export interface DashboardAttentionItem {
  id: string;
  kind: DashboardAttentionKind;
  title: string;
  href: string;
  subject?: string;
  dueDate?: string | null;
  bucket?: 'overdue' | 'today';
}

export interface DashboardAttentionSummary {
  overdueCount: number;
  todayCount: number;
  dealCount: number;
  leadCount: number;
  totalCount: number;
  items: DashboardAttentionItem[];
}

export interface DashboardAttentionQueue extends DashboardAttentionSummary {}

export const LOCAL_AUTOMATION_FOLLOW_UP_DAYS = 1;

export function buildDealStageFollowUpSuggestion({
  deal,
  activities,
  fromStage,
  toStage,
  now = new Date(),
  activityContextReady = true,
  activityContextError = false,
}: {
  deal: Deal;
  activities: Activity[];
  fromStage: DealStage;
  toStage: DealStage;
  now?: Date;
  activityContextReady?: boolean;
  activityContextError?: boolean;
}): DealStageFollowUpSuggestion | null {
  if (!activityContextReady || activityContextError) {
    return null;
  }

  if (
    fromStage === toStage ||
    isDealClosed({ stage: fromStage }) ||
    isDealClosed({ stage: toStage }) ||
    isDealClosed(deal)
  ) {
    return null;
  }

  if (nextDealActivity(activities, now)) {
    return null;
  }

  const dueDate = addLocalDays(now, LOCAL_AUTOMATION_FOLLOW_UP_DAYS);

  return {
    id: `${deal.id}:${fromStage}:${toStage}:${dueDate}`,
    dealId: deal.id,
    dealName: deal.name,
    fromStage,
    toStage,
    draft: {
      subject: `Follow up on ${deal.name}`,
      type: 'task',
      dueDate,
      notes: `Suggested locally after moving the deal to ${toStage}. Review and save only if this next step is right.`,
      contactId: deal.contactId ?? '',
      organizationId: deal.organizationId ?? '',
      dealId: deal.id,
    },
  };
}

function activityHref(activity: Activity): string {
  if (activity.dealId) {
    return `/deals/${activity.dealId}`;
  }

  if (activity.contactId) {
    return `/contacts/${activity.contactId}`;
  }

  return '/activities';
}

function hasPendingContactActivity(activities: Activity[], contactId: string): boolean {
  return activities.some(
    (activity) => activity.contactId === contactId && activity.status !== 'completed',
  );
}

export function buildDashboardAttentionQueue({
  activities,
  deals = [],
  leads = [],
  owner = '',
  now = new Date(),
  limit = 8,
}: {
  activities: Activity[];
  deals?: Deal[];
  leads?: Contact[];
  owner?: string;
  now?: Date;
  limit?: number;
}): DashboardAttentionQueue {
  const dealById = new Map(deals.map((deal) => [deal.id, deal]));
  const leadById = new Map(leads.map((lead) => [lead.id, lead]));
  const visibleDeals = deals.filter((deal) => matchesRecordOwner(deal.owner, owner));
  const visibleLeads = leads.filter((lead) => matchesRecordOwner(lead.owner, owner));
  const visibleActivities = activities.filter((activity) => {
    if (!owner.trim()) {
      return true;
    }
    const linkedDeal = activity.dealId ? dealById.get(activity.dealId) : undefined;
    if (linkedDeal && matchesRecordOwner(linkedDeal.owner, owner)) {
      return true;
    }
    const linkedLead = activity.contactId ? leadById.get(activity.contactId) : undefined;
    return Boolean(linkedLead && matchesRecordOwner(linkedLead.owner, owner));
  });

  const workbench = buildActivityWorkbench(visibleActivities, now);
  const overdue = workbench.buckets.find((bucket) => bucket.bucket === 'overdue')?.activities ?? [];
  const today = workbench.buckets.find((bucket) => bucket.bucket === 'today')?.activities ?? [];
  const overdueIds = new Set(overdue.map((activity) => activity.id));

  const followUps: DashboardAttentionItem[] = [...overdue, ...today].map((activity) => {
    const kind = overdueIds.has(activity.id) ? 'overdue' as const : 'today' as const;
    return {
      id: `activity:${activity.id}`,
      kind,
      title: activity.subject,
      href: activityHref(activity),
      subject: activity.subject,
      dueDate: activity.dueDate,
      bucket: kind,
    };
  });

  const dealItems = visibleDeals
    .filter((deal) => !isDealClosed(deal))
    .filter((deal) => !nextDealActivity(
      activities.filter((activity) => activity.dealId === deal.id),
      now,
    ))
    .sort((left, right) => Date.parse(left.updatedAt) - Date.parse(right.updatedAt))
    .map((deal) => ({
      id: `deal:${deal.id}`,
      kind: 'dealNeedsFollowUp' as const,
      title: deal.name,
      href: `/deals/${deal.id}`,
    }));

  const leadItems = visibleLeads
    .filter((lead) => lead.type === 'person' && lead.lifecycle === 'lead')
    .filter((lead) => !hasPendingContactActivity(activities, lead.id))
    .sort((left, right) => Date.parse(left.createdAt) - Date.parse(right.createdAt))
    .map((lead) => ({
      id: `lead:${lead.id}`,
      kind: 'leadWaiting' as const,
      title: contactDisplayName(lead),
      href: `/contacts/${lead.id}`,
    }));

  return {
    overdueCount: workbench.summary.overdue,
    todayCount: workbench.summary.today,
    dealCount: dealItems.length,
    leadCount: leadItems.length,
    totalCount:
      workbench.summary.overdue
      + workbench.summary.today
      + dealItems.length
      + leadItems.length,
    items: [...followUps, ...dealItems, ...leadItems].slice(0, limit),
  };
}

export function buildDashboardAttentionSummary(
  activities: Activity[],
  now: Date = new Date(),
  limit = 4,
): DashboardAttentionSummary {
  return buildDashboardAttentionQueue({ activities, now, limit });
}
