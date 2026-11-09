/**
 * Pending proposed-action count for the Review sidebar.
 */

import { listPendingProposedActions } from '$lib/api/proposedActions';

class ReviewCountsStore {
  pendingCount = $state(0);

  async refresh(): Promise<void> {
    try {
      const pending = await listPendingProposedActions();
      this.pendingCount = pending.length;
    } catch (err) {
      console.error('[ReviewCounts] Failed to load pending actions:', err);
    }
  }

  formatCount(): string {
    if (this.pendingCount > 99) {
      return '99+';
    }
    return String(this.pendingCount);
  }
}

export const reviewCountsStore = new ReviewCountsStore();
