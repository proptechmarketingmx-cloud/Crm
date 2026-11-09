import { beforeEach, describe, expect, it, vi } from 'vitest';

const { listPendingMock } = vi.hoisted(() => ({
  listPendingMock: vi.fn(),
}));

vi.mock('$lib/api/proposedActions', () => ({
  listPendingProposedActions: listPendingMock,
}));

import { reviewCountsStore } from './reviewCounts.svelte';

describe('reviewCountsStore', () => {
  beforeEach(() => {
    listPendingMock.mockReset();
    reviewCountsStore.pendingCount = 0;
  });

  it('stores the pending proposed-action count', async () => {
    listPendingMock.mockResolvedValueOnce([{ id: 'a' }, { id: 'b' }]);

    await reviewCountsStore.refresh();

    expect(reviewCountsStore.pendingCount).toBe(2);
    expect(reviewCountsStore.formatCount()).toBe('2');
  });

  it('keeps the last count when the list fails', async () => {
    reviewCountsStore.pendingCount = 4;
    listPendingMock.mockRejectedValueOnce(new Error('offline'));

    await reviewCountsStore.refresh();

    expect(reviewCountsStore.pendingCount).toBe(4);
  });

  it('caps the badge label at 99+', () => {
    reviewCountsStore.pendingCount = 120;
    expect(reviewCountsStore.formatCount()).toBe('99+');
  });
});
