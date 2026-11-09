/**
 * src/lib/api/search.ts - Tauri IPC wrapper for global search.
 */

import { invoke } from '@tauri-apps/api/core';

export type GlobalSearchEntityType =
  | 'contact'
  | 'organization'
  | 'deal'
  | 'activity'
  | 'note'
  | 'tag';

export interface GlobalSearchResult {
  entityType: GlobalSearchEntityType;
  entityId: string;
  title: string;
  subtitle: string;
  matchField: string;
}

interface BackendGlobalSearchResult {
  entity_type: GlobalSearchEntityType;
  entity_id: string;
  title: string;
  subtitle: string;
  match_field: string;
}

function mapGlobalSearchResult(result: BackendGlobalSearchResult): GlobalSearchResult {
  return {
    entityType: result.entity_type,
    entityId: result.entity_id,
    title: result.title,
    subtitle: result.subtitle,
    matchField: result.match_field,
  };
}

function normalizeText(value: string): string {
  return value.trim();
}

function normalizeLimit(limit: number | undefined): number | undefined {
  if (limit === undefined || !Number.isFinite(limit)) {
    return undefined;
  }

  return Math.min(100, Math.max(0, Math.trunc(limit)));
}

export async function globalSearch(
  query: string,
  limit?: number,
): Promise<GlobalSearchResult[]> {
  const results = await invoke<BackendGlobalSearchResult[]>('global_search', {
    query: normalizeText(query),
    limit: normalizeLimit(limit),
  });

  return results.map(mapGlobalSearchResult);
}
