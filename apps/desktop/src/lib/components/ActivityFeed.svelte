<script lang="ts">
  /**
   * ActivityFeed.svelte — Chronological activity timeline for 900CRM.
   *
   * Displays a list of activities with type icons, relative timestamps,
   * and linked entity names. Used on Dashboard and ContactDetail.
   */

  import type { Activity } from '$lib/api/activities';
  import { t } from '$lib/i18n';
  import type {
    ActivityRelationshipItem,
    ActivityRelationshipLabels,
  } from '$lib/utils/activityRelationships';
  import { formatActivityDue } from '$lib/utils/activityDue';
  import EmptyState from './EmptyState.svelte';

  type ActivityRelationshipEntityType = 'contact' | 'organization' | 'deal';

  interface ActivityRelationshipGroup {
    type: ActivityRelationshipEntityType;
    label: string;
    items: ActivityRelationshipItem[];
  }

  export interface ActivityFeedRelationshipNavigation {
    type: ActivityRelationshipEntityType;
    id: string;
  }

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    activities = [],
    loading = false,
    maxItems = 10,
    showEmpty = true,
    compact = false,
    relationshipsByActivityId = {},
    showRelationshipBreadcrumbs = false,
    onNavigateEntity = undefined,
  }: {
    activities?: Activity[];
    loading?: boolean;
    maxItems?: number;
    showEmpty?: boolean;
    compact?: boolean;
    relationshipsByActivityId?: Record<string, ActivityRelationshipLabels>;
    showRelationshipBreadcrumbs?: boolean;
    onNavigateEntity?: (entity: ActivityFeedRelationshipNavigation) => void;
  } = $props();

  // ── Derived ────────────────────────────────────────────────────────────────

  const visible = $derived(activities.slice(0, maxItems));

  function iconForType(type: string): string {
    const icons: Record<string, string> = {
      task:    'M9 11l3 3L22 4M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11',
      call:    'M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07A19.5 19.5 0 0 1 4.69 13.1 19.79 19.79 0 0 1 1.61 4.5 2 2 0 0 1 3.6 2.32h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l.76-.76a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z',
      meeting: 'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zm8 4a2 2 0 1 0 0-4 2 2 0 0 0 0 4zm4 2v-2a4 4 0 0 0-3-3.87',
      email:     'M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2zm0 0l8 8 8-8',
      visit:     'M12 21s7-4.5 7-11a7 7 0 10-14 0c0 6.5 7 11 7 11zM12 11a2 2 0 110-4 2 2 0 010 4z',
      whatsapp:  'M20 4a10 10 0 00-16.4 11.4L2 22l6.8-1.6A10 10 0 1020 4z',
      sms:       'M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z',
    };
    return icons[type] ?? icons.task;
  }

  function colorForStatus(status: string): string {
    if (status === 'completed') return 'var(--color-success-500)';
    if (status === 'overdue')   return 'var(--color-danger-500)';
    return 'var(--color-primary-500)';
  }

  function typeLabel(type: string): string {
    const map: Record<string, string> = {
      task:    t('activities.task'),
      call:    t('activities.call'),
      meeting: t('activities.meeting'),
      email:     t('activities.email'),
      visit:     t('activities.visit'),
      whatsapp:  t('activities.whatsapp'),
      sms:       t('activities.sms'),
    };
    return map[type] ?? type;
  }

  function relationshipLabels(activityId: string): ActivityRelationshipLabels {
    return relationshipsByActivityId[activityId] ?? {
      contacts: [],
      organizations: [],
      deals: [],
    };
  }

  function relationshipGroups(labels: ActivityRelationshipLabels): ActivityRelationshipGroup[] {
    const groups: ActivityRelationshipGroup[] = [
      {
        type: 'contact',
        label: t('activities.relationshipContact'),
        items: labels.contacts,
      },
      {
        type: 'organization',
        label: t('activities.relationshipOrganization'),
        items: labels.organizations,
      },
      {
        type: 'deal',
        label: t('activities.relationshipDeal'),
        items: labels.deals,
      },
    ];

    return groups.filter((group) => group.items.length > 0);
  }

  function hasRelationships(labels: ActivityRelationshipLabels): boolean {
    return labels.contacts.length > 0 || labels.organizations.length > 0 || labels.deals.length > 0;
  }

  function canNavigate(type: ActivityRelationshipEntityType): boolean {
    return type !== 'deal' && typeof onNavigateEntity === 'function';
  }

  function navigateRelationship(type: ActivityRelationshipEntityType, id: string) {
    if (!canNavigate(type)) {
      return;
    }

    onNavigateEntity?.({ type, id });
  }
</script>

<div class="activity-feed" class:compact>
  {#if loading}
    {#each Array(3) as _, i (i)}
      <div class="activity-skeleton">
        <div class="skeleton activity-icon-skeleton"></div>
        <div class="activity-skeleton-content">
          <div class="skeleton activity-skeleton-title"></div>
          <div class="skeleton activity-skeleton-meta"></div>
        </div>
      </div>
    {/each}
  {:else if visible.length === 0}
    {#if showEmpty}
      <EmptyState
        icon="activities"
        title={t('activities.noActivities')}
        compact={true}
      />
    {/if}
  {:else}
    <ul class="activity-list" role="list">
      {#each visible as activity (activity.id)}
        {@const labels = relationshipLabels(activity.id)}
        <li class="activity-item" class:completed={activity.status === 'completed'}>
          <!-- Icon -->
          <div
            class="activity-icon-wrap"
            style="color: {colorForStatus(activity.status)};"
            aria-hidden="true"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d={iconForType(activity.type)} />
            </svg>
          </div>

          <!-- Content -->
          <div class="activity-content">
            <p class="activity-subject">{activity.subject}</p>
            <div class="activity-meta">
              <span class="activity-type-label">{typeLabel(activity.type)}</span>
              {#if activity.contactName}
                <span class="activity-dot" aria-hidden="true">·</span>
                <span>{activity.contactName}</span>
              {/if}
              {#if activity.dueDate}
                <span class="activity-dot" aria-hidden="true">·</span>
                <span
                  class="activity-time"
                  class:overdue={activity.status === 'overdue'}
                >
                  {formatActivityDue(activity.dueDate)}
                </span>
              {/if}
            </div>
            {#if showRelationshipBreadcrumbs && hasRelationships(labels)}
              <div class="activity-relationships" aria-label={t('activities.relationshipsLabel')}>
                {#each relationshipGroups(labels) as group (group.type)}
                  {#each group.items as item (group.type + item.id)}
                    {#if canNavigate(group.type)}
                      <button
                        class="activity-breadcrumb"
                        type="button"
                        onclick={() => navigateRelationship(group.type, item.id)}
                        aria-label={t('activities.openRelationship', {
                          type: group.label,
                          label: item.label,
                        })}
                      >
                        <span class="activity-breadcrumb-type">{group.label}</span>
                        <span>{item.label}</span>
                      </button>
                    {:else}
                      <span class="activity-breadcrumb activity-breadcrumb-static">
                        <span class="activity-breadcrumb-type">{group.label}</span>
                        <span>{item.label}</span>
                      </span>
                    {/if}
                  {/each}
                {/each}
              </div>
            {/if}
          </div>

          <!-- Status badge -->
          {#if !compact}
            <span class="badge {activity.status === 'completed' ? 'badge-success' : activity.status === 'overdue' ? 'badge-danger' : 'badge-neutral'} activity-status">
              {t(`activities.${activity.status}`)}
            </span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .activity-feed {
    display: flex;
    flex-direction: column;
  }

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .activity-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
    padding: var(--space-4) 0;
    border-block-end: var(--border-width) solid var(--border-subtle);
    transition: opacity var(--duration-fast) var(--ease-out);
  }

  .activity-item:last-child {
    border-block-end: none;
  }

  .activity-item.completed {
    opacity: 0.6;
  }

  .activity-icon-wrap {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background-color: var(--surface-hover);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-block-start: 2px;
  }

  .activity-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .activity-subject {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .activity-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  .activity-type-label {
    text-transform: capitalize;
  }

  .activity-dot {
    color: var(--border-strong);
  }

  .activity-time.overdue {
    color: var(--text-danger);
  }

  .activity-relationships {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-block-start: var(--space-1);
  }

  .activity-breadcrumb {
    display: inline-flex;
    align-items: center;
    min-width: 0;
    max-width: 100%;
    gap: 4px;
    padding: 2px var(--space-2);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-sm);
    background-color: var(--surface-raised);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    line-height: 1.3;
  }

  button.activity-breadcrumb {
    cursor: pointer;
  }

  button.activity-breadcrumb:hover {
    border-color: var(--border-strong);
    color: var(--text-accent);
  }

  .activity-breadcrumb-static {
    cursor: default;
  }

  .activity-breadcrumb-type {
    flex-shrink: 0;
    color: var(--text-tertiary);
    font-weight: var(--weight-medium);
  }

  .activity-status {
    flex-shrink: 0;
    margin-block-start: 2px;
  }

  /* Skeleton */
  .activity-skeleton {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4) 0;
  }

  .activity-icon-skeleton {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .activity-skeleton-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .activity-skeleton-title {
    height: 13px;
    width: 60%;
    border-radius: var(--border-radius-sm);
  }

  .activity-skeleton-meta {
    height: 11px;
    width: 40%;
    border-radius: var(--border-radius-sm);
  }
</style>
