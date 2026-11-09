/**
 * src/lib/api/dashboard.ts — Tauri IPC wrappers for dashboard stats.
 */

import { invoke } from '@tauri-apps/api/core';
import { normalizeCurrencyCode } from '$lib/utils/currency';

export interface DashboardStats {
  totalContacts: number;
  activeDeals: number;
  pipelineValue: number;
  pipelineValueByCurrency: {
    currency: string;
    totalValue: number;
    dealCount: number;
  }[];
  upcomingTasks: number;
  overdueActivities: number;
}

interface BackendDashboardStats {
  total_contacts: number;
  total_organizations: number;
  active_deals: number;
  pipeline_value: number;
  pipeline_value_by_currency: {
    currency: string;
    total_value: number;
    deal_count: number;
  }[];
  upcoming_activities: number;
  overdue_activities: number;
}

export async function getDashboardStats(): Promise<DashboardStats> {
  const stats = await invoke<BackendDashboardStats>('get_dashboard_stats');

  return {
    totalContacts: (stats.total_contacts ?? 0) + (stats.total_organizations ?? 0),
    activeDeals: stats.active_deals ?? 0,
    pipelineValue: stats.pipeline_value ?? 0,
    pipelineValueByCurrency: (stats.pipeline_value_by_currency ?? []).map((bucket) => ({
      currency: normalizeCurrencyCode(bucket.currency),
      totalValue: Number.isFinite(bucket.total_value) ? bucket.total_value : 0,
      dealCount: bucket.deal_count ?? 0,
    })),
    upcomingTasks: stats.upcoming_activities ?? 0,
    overdueActivities: stats.overdue_activities ?? 0,
  };
}
