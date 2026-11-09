// @vitest-environment jsdom

import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Deal, DealsByStage } from '$lib/api/deals';

const {
  dealFixture,
  dealStoreMock,
  emptyStagesFixture,
  listActivitiesForDealsMock,
  listDealsMock,
  loadActivityLinkIndexMock,
  loadActivityRelationshipLookupsMock,
  loadDealRelationshipLookupsMock,
  listCustomFieldDefinitionsMock,
  listCustomFieldValuesForEntityTypeMock,
  moveDealStageMock,
  openModalMock,
  selectDealMock,
} = vi.hoisted(() => {
  const moveDealStageMock = vi.fn();
  const selectDealMock = vi.fn();
  const dealFixture = {
    id: 'deal-automation',
    name: 'Automation rollout',
    value: 10000,
    currency: 'USD',
    stage: 'lead',
    probability: 25,
    expectedCloseDate: '2026-08-01',
    contactId: 'contact-1',
    organizationId: 'organization-1',
    contactName: null,
    description: null,
    tags: [],
    createdAt: '2026-07-01T00:00:00Z',
    updatedAt: '2026-07-08T00:00:00Z',
  };
  const emptyStagesFixture = {
    lead: [],
    qualified: [],
    proposal: [],
    negotiation: [],
    closedWon: [],
    closedLost: [],
  };

  return {
    dealFixture,
    emptyStagesFixture,
    dealStoreMock: {
      deals: [dealFixture],
      dealsByStage: {
        ...emptyStagesFixture,
        lead: [dealFixture],
      },
      isLoading: false,
      loadPipelineBoard: vi.fn().mockResolvedValue(undefined),
      moveDealStage: moveDealStageMock,
      selectDeal: selectDealMock,
    },
    listActivitiesForDealsMock: vi.fn(),
    listDealsMock: vi.fn(),
    loadActivityLinkIndexMock: vi.fn(),
    loadActivityRelationshipLookupsMock: vi.fn(),
    loadDealRelationshipLookupsMock: vi.fn(),
    listCustomFieldDefinitionsMock: vi.fn(),
    listCustomFieldValuesForEntityTypeMock: vi.fn(),
    moveDealStageMock,
    openModalMock: vi.fn(),
    selectDealMock,
  };
});

const deal = dealFixture as Deal;
const emptyStages = emptyStagesFixture as DealsByStage;
const pipelineDealStore = dealStoreMock as unknown as {
  deals: Deal[];
  dealsByStage: DealsByStage;
  isLoading: boolean;
  loadPipelineBoard: ReturnType<typeof vi.fn>;
  moveDealStage: ReturnType<typeof vi.fn>;
  selectDeal: ReturnType<typeof vi.fn>;
};

vi.mock('$lib/i18n', () => ({
  t: (key: string, params?: Record<string, string | number>) => {
    if (key === 'localAutomation.pipeline.description') {
      return `${params?.deal} moved to ${params?.stage}`;
    }
    if (key === 'deals.board.moved') {
      return `Moved ${params?.deal} to ${params?.stage}`;
    }
    if (key === 'deals.board.dropToStage') {
      return `Drop to move to ${params?.stage}`;
    }
    if (key === 'deals.board.alreadyInStage') {
      return `Already in ${params?.stage}`;
    }
    return key;
  },
}));

vi.mock('$lib/api/activities', () => ({
  listActivitiesForDeals: listActivitiesForDealsMock,
}));

vi.mock('$lib/api/deals', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/api/deals')>();
  return {
    ...actual,
    listDeals: listDealsMock,
  };
});

vi.mock('$lib/api/customFields', () => ({
  listCustomFieldDefinitions: listCustomFieldDefinitionsMock,
  listCustomFieldValuesForEntityType: listCustomFieldValuesForEntityTypeMock,
}));

vi.mock('$lib/api/savedViews', () => ({
  listSavedViews: vi.fn().mockResolvedValue([]),
  createSavedView: vi.fn(),
  deleteSavedView: vi.fn(),
  filtersMatch: () => false,
}));

vi.mock('$lib/stores/deals', () => ({
  dealStore: dealStoreMock,
}));

vi.mock('$lib/stores/activities', () => ({
  activityStore: {
    relationshipRefreshVersion: 0,
  },
}));

vi.mock('$lib/stores/ui', () => ({
  uiStore: {
    activeModal: null,
    openModal: openModalMock,
    toastError: vi.fn(),
    toastSuccess: vi.fn(),
  },
}));

vi.mock('$lib/stores/settings', () => ({
  settingsStore: {
    currency: 'USD',
    dateFormat: 'MMM D, YYYY',
    language: 'en',
  },
}));

vi.mock('$lib/utils/activityRelationships', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/utils/activityRelationships')>();
  return {
    ...actual,
    loadActivityLinkIndex: loadActivityLinkIndexMock,
    loadActivityRelationshipLookups: loadActivityRelationshipLookupsMock,
  };
});

vi.mock('$lib/utils/dealRelationships', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/utils/dealRelationships')>();
  return {
    ...actual,
    loadDealRelationshipLookups: loadDealRelationshipLookupsMock,
  };
});

import Pipeline from '../../routes/Pipeline.svelte';

function dragData() {
  return {
    effectAllowed: '',
    dropEffect: '',
    setData: vi.fn(),
  };
}

describe('Pipeline local automation prompt', () => {
  beforeEach(() => {
    pipelineDealStore.deals = [deal];
    pipelineDealStore.dealsByStage = {
      ...emptyStages,
      lead: [deal],
    };
    pipelineDealStore.isLoading = false;
    pipelineDealStore.loadPipelineBoard.mockClear();
    moveDealStageMock.mockReset();
    moveDealStageMock.mockImplementation(async (id: string, toStage: string) => {
      pipelineDealStore.dealsByStage = {
        ...emptyStages,
        [toStage]: [{ ...deal, stage: toStage as Deal['stage'] }],
      };
    });
    openModalMock.mockReset();
    selectDealMock.mockReset();
    listActivitiesForDealsMock.mockReset();
    listActivitiesForDealsMock.mockResolvedValue([]);
    listDealsMock.mockReset();
    listDealsMock.mockResolvedValue([deal]);
    loadActivityLinkIndexMock.mockReset();
    loadActivityLinkIndexMock.mockResolvedValue({});
    loadActivityRelationshipLookupsMock.mockReset();
    loadActivityRelationshipLookupsMock.mockResolvedValue({ contacts: [], organizations: [], deals: [] });
    loadDealRelationshipLookupsMock.mockReset();
    loadDealRelationshipLookupsMock.mockResolvedValue({ contacts: [], organizations: [] });
    listCustomFieldDefinitionsMock.mockReset();
    listCustomFieldDefinitionsMock.mockResolvedValue([]);
    listCustomFieldValuesForEntityTypeMock.mockReset();
    listCustomFieldValuesForEntityTypeMock.mockResolvedValue([]);
  });

  it('shows a follow-up draft prompt after moving an open deal with no linked activity', async () => {
    const { container } = render(Pipeline);

    await screen.findByRole('button', { name: /Automation rollout/ });
    await waitFor(() => {
      expect(listActivitiesForDealsMock).toHaveBeenCalledWith(['deal-automation']);
    });

    const card = screen.getByRole('button', { name: /Automation rollout/ });
    const qualifiedColumn = container.querySelector('[aria-label="deals.stages.qualified"]');
    expect(qualifiedColumn).toBeTruthy();

    await fireEvent.dragStart(card, { dataTransfer: dragData() });
    await fireEvent.drop(qualifiedColumn as Element, { dataTransfer: dragData() });

    expect(await screen.findByText('localAutomation.pipeline.title')).toBeTruthy();
    expect(screen.getByText('Automation rollout moved to deals.stages.qualified')).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: 'localAutomation.pipeline.addDraft' }));

    expect(openModalMock).toHaveBeenCalledWith('addActivity', expect.objectContaining({
      dealId: 'deal-automation',
      contactId: 'contact-1',
      organizationId: 'organization-1',
      subject: 'Follow up on Automation rollout',
      type: 'task',
    }));
  });

  it('highlights another stage as a drop target and ignores a same-stage drop', async () => {
    const { container } = render(Pipeline);

    await screen.findByRole('button', { name: /Automation rollout/ });
    await waitFor(() => {
      expect(listActivitiesForDealsMock).toHaveBeenCalledWith(['deal-automation']);
    });

    const card = screen.getByRole('button', { name: /Automation rollout/ });
    await fireEvent.dragStart(card, { dataTransfer: dragData() });

    const qualifiedColumn = container.querySelector('[data-testid="pipeline-column-qualified"]');
    const leadColumn = container.querySelector('[data-testid="pipeline-column-lead"]');
    expect(qualifiedColumn).toBeTruthy();
    expect(leadColumn).toBeTruthy();

    await fireEvent.dragOver(qualifiedColumn as Element, { dataTransfer: dragData() });
    expect((qualifiedColumn as HTMLElement).className).toContain('kanban-column--drop-target');
    expect(screen.getByTestId('pipeline-drop-hint-qualified').textContent).toContain('Drop to move to');

    await fireEvent.dragOver(leadColumn as Element, { dataTransfer: dragData() });
    expect((leadColumn as HTMLElement).className).toContain('kanban-column--drop-source');

    moveDealStageMock.mockClear();
    await fireEvent.drop(leadColumn as Element, { dataTransfer: dragData() });
    expect(moveDealStageMock).not.toHaveBeenCalled();
  });
});
