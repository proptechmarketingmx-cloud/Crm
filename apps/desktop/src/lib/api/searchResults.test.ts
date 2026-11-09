import { describe, expect, it, vi } from 'vitest';

import type { GlobalSearchResult } from './search';
import {
  mapGlobalSearchResultToSearchResult,
  searchResultBadgeClass,
  searchResultTypeLabel,
} from '../utils/searchResults';

describe('global search result UI helpers', () => {
  it('maps backend API result shape to the shared UI store result shape', () => {
    const result: GlobalSearchResult = {
      entityType: 'note',
      entityId: 'note-1',
      title: 'Follow up',
      subtitle: 'Clinic Partners',
      matchField: 'content',
    };

    expect(mapGlobalSearchResultToSearchResult(result)).toEqual({
      id: 'note-1',
      type: 'note',
      title: 'Follow up',
      subtitle: 'Clinic Partners',
    });
  });

  it('labels every global search entity type with translation fallback support', () => {
    const translate = vi.fn((key: string) => {
      if (key === 'common.tags') return key;
      return `translated:${key}`;
    });

    expect(searchResultTypeLabel('contact', translate)).toBe('translated:contacts.title');
    expect(searchResultTypeLabel('organization', translate)).toBe('translated:organizations.title');
    expect(searchResultTypeLabel('deal', translate)).toBe('translated:deals.title');
    expect(searchResultTypeLabel('activity', translate)).toBe('translated:activities.title');
    expect(searchResultTypeLabel('note', translate)).toBe('translated:common.notes');
    expect(searchResultTypeLabel('tag', translate)).toBe('Tags');
  });

  it('assigns a stable badge class to every global search entity type', () => {
    expect(searchResultBadgeClass('contact')).toBe('badge-primary');
    expect(searchResultBadgeClass('organization')).toBe('badge-neutral');
    expect(searchResultBadgeClass('deal')).toBe('badge-success');
    expect(searchResultBadgeClass('activity')).toBe('badge-warning');
    expect(searchResultBadgeClass('note')).toBe('badge-neutral');
    expect(searchResultBadgeClass('tag')).toBe('badge-primary');
  });
});
