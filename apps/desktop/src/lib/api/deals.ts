/**
 * src/lib/api/deals.ts — Tauri IPC wrappers for deals/pipeline.
 */

import { invoke } from '@tauri-apps/api/core';
import { normalizeCurrencyCode } from '$lib/utils/currency';

export type DealStage =
  | 'lead'
  | 'qualified'
  | 'proposal'
  | 'negotiation'
  | 'closedWon'
  | 'closedLost';

export const DEAL_STAGES: DealStage[] = [
  'lead',
  'qualified',
  'proposal',
  'negotiation',
  'closedWon',
  'closedLost',
];

export interface Deal {
  id: string;
  name: string;
  value: number;
  currency: string;
  stage: DealStage;
  probability: number;
  expectedCloseDate: string | null;
  contactId: string | null;
  organizationId: string | null;
  contactName: string | null;
  description: string | null;
  owner?: string | null;
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

export type CreateDealPayload = Omit<
  Deal,
  'id' | 'createdAt' | 'updatedAt' | 'contactName' | 'organizationId'
> & {
  organizationId?: string | null;
};
export type UpdateDealPayload = Partial<CreateDealPayload>;

export interface DealContact {
  id: string;
  dealId: string;
  contactId: string;
  role: string | null;
  isPrimary: boolean;
  createdAt: string;
  deletedAt: string | null;
  deviceId: string;
}

export interface AddDealContactPayload {
  role?: string | null;
  isPrimary?: boolean;
}

export interface ListDealsParams {
  stage?: DealStage;
  contactId?: string;
  organizationId?: string;
  sortBy?: 'name' | 'value' | 'createdAt' | 'expectedCloseDate';
  sortDir?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

export type DealsByStage = Record<DealStage, Deal[]>;

export interface PipelineSummary {
  totalDeals: number;
  totalValue: number;
  byStage: Record<DealStage, { count: number; value: number }>;
}

interface BackendDeal {
  id: string;
  title: string;
  value: number;
  currency: string;
  stage: string;
  probability: number;
  expected_close: string | null;
  contact_id: string | null;
  organization_id?: string | null;
  notes: string;
  owner?: string | null;
  created_at: string;
  updated_at: string;
}

interface BackendDealContact {
  id: string;
  deal_id: string;
  contact_id: string;
  role: string | null;
  is_primary: boolean;
  created_at: string;
  deleted_at: string | null;
  device_id: string;
}

interface BackendPipelineSummary {
  stage: string;
  count: number;
  total_value: number;
  weighted_value: number;
}

const BACKEND_STAGE_MAP: Record<DealStage, string> = {
  lead: 'Lead',
  qualified: 'Qualified',
  proposal: 'Proposal',
  negotiation: 'Negotiation',
  closedWon: 'Closed Won',
  closedLost: 'Closed Lost',
};

const UI_STAGE_MAP: Record<string, DealStage> = {
  lead: 'lead',
  qualified: 'qualified',
  proposal: 'proposal',
  negotiation: 'negotiation',
  'closed won': 'closedWon',
  'closedwon': 'closedWon',
  'closed lost': 'closedLost',
  'closedlost': 'closedLost',
};

function toUiStage(stage: string): DealStage {
  const key = stage.toLowerCase().trim();
  return UI_STAGE_MAP[key] ?? 'lead';
}

function toBackendStage(stage: DealStage | undefined): string | undefined {
  if (!stage) {
    return undefined;
  }
  return BACKEND_STAGE_MAP[stage];
}

function normalizeNullable(value: string | null | undefined): string | null {
  if (value == null) {
    return null;
  }

  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function hasOwn<T extends object, K extends PropertyKey>(
  object: T,
  key: K,
): object is T & Record<K, unknown> {
  return Object.prototype.hasOwnProperty.call(object, key);
}

function assignNullableUpdate(
  args: Record<string, unknown>,
  valueKey: string,
  resetKey: string,
  value: string | null | undefined,
): void {
  if (value === undefined) {
    return;
  }

  const normalized = normalizeNullable(value);
  if (normalized === null) {
    args[resetKey] = true;
  } else {
    args[valueKey] = normalized;
  }
}

function mapDeal(deal: BackendDeal): Deal {
  return {
    id: deal.id,
    name: deal.title,
    value: Number.isFinite(deal.value) ? deal.value : 0,
    currency: normalizeCurrencyCode(deal.currency),
    stage: toUiStage(deal.stage),
    probability: deal.probability ?? 0,
    expectedCloseDate: deal.expected_close,
    contactId: deal.contact_id,
    organizationId: deal.organization_id ?? null,
    contactName: null,
    description: deal.notes?.trim() ? deal.notes : null,
    owner: deal.owner?.trim() ? deal.owner.trim() : null,
    tags: [],
    createdAt: deal.created_at,
    updatedAt: deal.updated_at,
  };
}

function mapDealContact(dealContact: BackendDealContact): DealContact {
  return {
    id: dealContact.id,
    dealId: dealContact.deal_id,
    contactId: dealContact.contact_id,
    role: dealContact.role ?? null,
    isPrimary: dealContact.is_primary,
    createdAt: dealContact.created_at,
    deletedAt: dealContact.deleted_at ?? null,
    deviceId: dealContact.device_id,
  };
}

function sortDeals(items: Deal[], params: ListDealsParams): Deal[] {
  const sorted = [...items];
  const direction = params.sortDir === 'desc' ? -1 : 1;

  if (!params.sortBy) {
    return sorted;
  }

  sorted.sort((a, b) => {
    switch (params.sortBy) {
      case 'value':
        return (a.value - b.value) * direction;
      case 'createdAt':
        return ((Date.parse(a.createdAt) || 0) - (Date.parse(b.createdAt) || 0)) * direction;
      case 'expectedCloseDate':
        return ((Date.parse(a.expectedCloseDate ?? '') || 0) - (Date.parse(b.expectedCloseDate ?? '') || 0)) * direction;
      case 'name':
      default:
        return a.name.localeCompare(b.name) * direction;
    }
  });

  return sorted;
}

export async function createDeal(data: CreateDealPayload): Promise<Deal> {
  const deal = await invoke<BackendDeal>('create_deal', {
    title: data.name,
    value: data.value,
    currency: data.currency,
    stage: toBackendStage(data.stage),
    probability: data.probability,
    expected_close: data.expectedCloseDate,
    contact_id: data.contactId,
    organization_id: normalizeNullable(data.organizationId),
    notes: data.description ?? '',
    owner: data.owner ?? null,
  });

  return mapDeal(deal);
}

export async function getDeal(id: string): Promise<Deal> {
  const deal = await invoke<BackendDeal>('get_deal', { id });
  return mapDeal(deal);
}

export const DEFAULT_LIST_DEALS_PAGE_SIZE = 200;
export const PIPELINE_BOARD_PAGE_SIZE = 500;

export async function listDeals(params: ListDealsParams = {}): Promise<Deal[]> {
  const pageSize = params.pageSize ?? DEFAULT_LIST_DEALS_PAGE_SIZE;
  const page = params.page ?? 1;
  const invokeArgs: Record<string, unknown> = {
    limit: pageSize,
    offset: Math.max(0, (page - 1) * pageSize),
  };

  const deals = await invoke<BackendDeal[]>('list_deals', invokeArgs);
  let mapped = deals.map(mapDeal);

  if (params.stage) {
    mapped = mapped.filter((deal) => deal.stage === params.stage);
  }

  if (params.contactId) {
    mapped = mapped.filter((deal) => deal.contactId === params.contactId);
  }

  if (params.organizationId) {
    mapped = mapped.filter((deal) => deal.organizationId === params.organizationId);
  }

  mapped = sortDeals(mapped, params);

  return mapped;
}

export async function listDealsByStage(): Promise<DealsByStage> {
  const allDeals = await listDeals({ pageSize: PIPELINE_BOARD_PAGE_SIZE, page: 1 });

  const grouped: DealsByStage = {
    lead: [],
    qualified: [],
    proposal: [],
    negotiation: [],
    closedWon: [],
    closedLost: [],
  };

  for (const deal of allDeals) {
    grouped[deal.stage].push(deal);
  }

  return grouped;
}

export async function updateDeal(id: string, data: UpdateDealPayload): Promise<Deal> {
  const args: Record<string, unknown> = {
    id,
    title: data.name,
    value: data.value,
    currency: data.currency,
    stage: toBackendStage(data.stage),
    probability: data.probability,
    notes: data.description,
  };

  if (hasOwn(data, 'owner')) {
    args.owner = data.owner;
    args.reset_owner = !data.owner?.trim();
  }

  if (hasOwn(data, 'expectedCloseDate') && data.expectedCloseDate !== undefined) {
    assignNullableUpdate(
      args,
      'expected_close',
      'reset_expected_close',
      data.expectedCloseDate,
    );
  }

  if (hasOwn(data, 'contactId') && data.contactId !== undefined) {
    assignNullableUpdate(args, 'contact_id', 'reset_contact_id', data.contactId);
  }

  if (hasOwn(data, 'organizationId')) {
    assignNullableUpdate(
      args,
      'organization_id',
      'reset_organization_id',
      data.organizationId,
    );
  }

  const deal = await invoke<BackendDeal>('update_deal', args);

  return mapDeal(deal);
}

export async function moveDealStage(id: string, stage: DealStage): Promise<Deal> {
  const deal = await invoke<BackendDeal>('move_deal_stage', {
    id,
    stage: toBackendStage(stage),
  });

  return mapDeal(deal);
}

export async function deleteDeal(id: string): Promise<void> {
  await invoke<void>('delete_deal', { id });
}

export async function linkDealToOrganization(
  dealId: string,
  organizationId: string | null,
): Promise<Deal> {
  const deal = await invoke<BackendDeal>('link_deal_to_organization', {
    deal_id: dealId,
    organization_id: normalizeNullable(organizationId),
  });

  return mapDeal(deal);
}

export async function addDealContact(
  dealId: string,
  contactId: string,
  data: AddDealContactPayload = {},
): Promise<DealContact> {
  const dealContact = await invoke<BackendDealContact>('add_deal_contact', {
    deal_id: dealId,
    contact_id: contactId,
    role: normalizeNullable(data.role),
    is_primary: data.isPrimary ?? false,
  });

  return mapDealContact(dealContact);
}

export async function removeDealContact(
  dealId: string,
  contactId: string,
): Promise<DealContact> {
  const dealContact = await invoke<BackendDealContact>('remove_deal_contact', {
    deal_id: dealId,
    contact_id: contactId,
  });

  return mapDealContact(dealContact);
}

export async function listDealContacts(dealId: string): Promise<DealContact[]> {
  const dealContacts = await invoke<BackendDealContact[]>('list_deal_contacts', {
    deal_id: dealId,
  });

  return dealContacts.map(mapDealContact);
}

export async function getPipelineSummary(): Promise<PipelineSummary> {
  const summary = await invoke<BackendPipelineSummary[]>('get_pipeline_summary');

  const byStage: PipelineSummary['byStage'] = {
    lead: { count: 0, value: 0 },
    qualified: { count: 0, value: 0 },
    proposal: { count: 0, value: 0 },
    negotiation: { count: 0, value: 0 },
    closedWon: { count: 0, value: 0 },
    closedLost: { count: 0, value: 0 },
  };

  for (const row of summary) {
    const stage = toUiStage(row.stage);
    byStage[stage] = {
      count: row.count ?? 0,
      value: row.total_value ?? 0,
    };
  }

  const totalDeals = Object.values(byStage).reduce((sum, item) => sum + item.count, 0);
  const totalValue = Object.values(byStage).reduce((sum, item) => sum + item.value, 0);

  return {
    totalDeals,
    totalValue,
    byStage,
  };
}
