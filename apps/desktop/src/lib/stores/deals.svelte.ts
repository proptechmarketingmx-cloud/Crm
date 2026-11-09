/**
 * src/lib/stores/deals.svelte.ts — Deal/pipeline state management for 900CRM.
 *
 * @module stores/deals
 */

import {
  listDeals,
  listDealsByStage,
  createDeal,
  updateDeal,
  moveDealStage,
  deleteDeal,
  getPipelineSummary,
} from '$lib/api/deals';
import { t } from '$lib/i18n';
import type {
  Deal,
  CreateDealPayload,
  UpdateDealPayload,
  DealsByStage,
  DealStage,
  PipelineSummary,
  ListDealsParams,
} from '$lib/api/deals';
import { uiStore } from './ui';

// ─────────────────────────────────────────────────────────────────────────────
// DealStore
// ─────────────────────────────────────────────────────────────────────────────

class DealStore {
  // ── State ───────────────────────────────────────────────────────────────────

  /** Flat list of all deals (for list views). */
  deals = $state<Deal[]>([]);

  /** Deals grouped by stage (for Kanban). */
  dealsByStage = $state<DealsByStage>({
    lead: [],
    qualified: [],
    proposal: [],
    negotiation: [],
    closedWon: [],
    closedLost: [],
  });

  /** Currently selected deal. */
  selectedDeal = $state<Deal | null>(null);

  /** Pipeline summary stats. */
  summary = $state<PipelineSummary | null>(null);

  /** Whether the pipeline is loading. */
  isLoading = $state<boolean>(false);

  /** Whether a save/move is in progress. */
  isSaving = $state<boolean>(false);

  // ── Actions ─────────────────────────────────────────────────────────────────

  /**
   * Load all deals and group them by stage.
   */
  async loadDeals(params: ListDealsParams = {}): Promise<void> {
    this.isLoading = true;
    try {
      this.deals = await listDeals(params);
    } catch (err) {
      uiStore.toastError(t('errors.loadNamed', { name: t('entities.deal') }));
      throw err;
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Load deals grouped by pipeline stage (for Kanban board).
   */
  async loadPipelineBoard(): Promise<void> {
    this.isLoading = true;
    try {
      this.dealsByStage = await listDealsByStage();
    } catch (err) {
      uiStore.toastError(t('errors.loadNamed', { name: t('deals.title') }));
      throw err;
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Load pipeline summary statistics.
   */
  async loadPipelineSummary(): Promise<void> {
    try {
      this.summary = await getPipelineSummary();
    } catch (err) {
      console.error('[deals] Failed to load summary:', err);
    }
  }

  /**
   * Create a new deal.
   *
   * @param data  Deal creation payload
   * @returns     The created Deal
   */
  async createDeal(data: CreateDealPayload): Promise<Deal> {
    this.isSaving = true;
    try {
      const deal = await createDeal(data);

      // Insert optimistically into the right stage column
      this.dealsByStage = {
        ...this.dealsByStage,
        [deal.stage]: [...this.dealsByStage[deal.stage], deal],
      };
      this.deals = [...this.deals, deal];

      uiStore.toastSuccess(t('toasts.created', { name: t('entities.deal') }));
      return deal;
    } catch (err) {
      uiStore.toastError(t('errors.createNamed', { name: t('entities.deal') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  /**
   * Update an existing deal.
   *
   * @param id    Deal UUID
   * @param data  Fields to update
   */
  async updateDeal(id: string, data: UpdateDealPayload): Promise<Deal> {
    this.isSaving = true;
    try {
      const deal = await updateDeal(id, data);

      this.deals = this.deals.map((d) => (d.id === id ? deal : d));

      if (this.selectedDeal?.id === id) {
        this.selectedDeal = deal;
      }

      // Refresh stage grouping
      await this.loadPipelineBoard();

      uiStore.toastSuccess(t('toasts.updated', { name: t('entities.deal') }));
      return deal;
    } catch (err) {
      uiStore.toastError(t('errors.updateNamed', { name: t('entities.deal') }));
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  /**
   * Move a deal to a different stage (optimistic, then backend sync).
   *
   * @param id       Deal UUID
   * @param toStage  Destination stage
   */
  async moveDealStage(id: string, toStage: DealStage): Promise<void> {
    // Find the deal and its current stage
    const deal = this.deals.find((d) => d.id === id) ??
      Object.values(this.dealsByStage).flat().find((d) => d.id === id);

    if (!deal) return;

    const fromStage = deal.stage;
    if (fromStage === toStage) return;

    // Optimistic update
    const updatedDeal = { ...deal, stage: toStage };
    this.dealsByStage = {
      ...this.dealsByStage,
      [fromStage]: this.dealsByStage[fromStage].filter((d) => d.id !== id),
      [toStage]: [...this.dealsByStage[toStage], updatedDeal],
    };

    try {
      await moveDealStage(id, toStage);
      this.deals = this.deals.map((d) => (d.id === id ? updatedDeal : d));
    } catch (err) {
      // Rollback
      this.dealsByStage = {
        ...this.dealsByStage,
        [fromStage]: [...this.dealsByStage[fromStage], deal],
        [toStage]: this.dealsByStage[toStage].filter((d) => d.id !== id),
      };
      uiStore.toastError(t('errors.moveNamed', { name: t('entities.deal') }));
      throw err;
    }
  }

  /**
   * Delete a deal.
   *
   * @param id  Deal UUID
   */
  async deleteDeal(id: string): Promise<void> {
    try {
      await deleteDeal(id);
      this.deals = this.deals.filter((d) => d.id !== id);

      // Remove from stage grouping
      const updated = { ...this.dealsByStage };
      for (const stage of Object.keys(updated) as DealStage[]) {
        updated[stage] = updated[stage].filter((d) => d.id !== id);
      }
      this.dealsByStage = updated;

      if (this.selectedDeal?.id === id) {
        this.selectedDeal = null;
      }

      uiStore.toastSuccess(t('toasts.deleted', { name: t('entities.deal') }));
    } catch (err) {
      uiStore.toastError(t('errors.deleteNamed', { name: t('entities.deal') }));
      throw err;
    }
  }

  /** Set the selected deal. */
  selectDeal(deal: Deal | null): void {
    this.selectedDeal = deal;
  }
}

/** Singleton deals store. */
export const dealStore = new DealStore();
