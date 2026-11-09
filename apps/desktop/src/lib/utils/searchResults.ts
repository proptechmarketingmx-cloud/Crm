import type { GlobalSearchEntityType, GlobalSearchResult } from '$lib/api/search';
import type { SearchResult } from '$lib/stores/ui';

type Translate = (key: string) => string;

const TYPE_LABEL_KEYS: Record<GlobalSearchEntityType, string> = {
  contact: 'contacts.title',
  organization: 'organizations.title',
  deal: 'deals.title',
  activity: 'activities.title',
  note: 'common.notes',
  tag: 'common.tags',
};

const TYPE_LABEL_FALLBACKS: Record<GlobalSearchEntityType, string> = {
  contact: 'Contacts',
  organization: 'Organizations',
  deal: 'Deals',
  activity: 'Activities',
  note: 'Notes',
  tag: 'Tags',
};

const TYPE_BADGE_CLASSES: Record<GlobalSearchEntityType, string> = {
  contact: 'badge-primary',
  organization: 'badge-neutral',
  deal: 'badge-success',
  activity: 'badge-warning',
  note: 'badge-neutral',
  tag: 'badge-primary',
};

export function mapGlobalSearchResultToSearchResult(result: GlobalSearchResult): SearchResult {
  return {
    id: result.entityId,
    type: result.entityType,
    title: result.title,
    subtitle: result.subtitle,
  };
}

export function searchResultTypeLabel(type: GlobalSearchEntityType, translate: Translate): string {
  const key = TYPE_LABEL_KEYS[type];
  const label = translate(key);

  return label === key ? TYPE_LABEL_FALLBACKS[type] : label;
}

export function searchResultBadgeClass(type: GlobalSearchEntityType): string {
  return TYPE_BADGE_CLASSES[type] ?? 'badge-neutral';
}
