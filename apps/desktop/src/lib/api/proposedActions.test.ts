import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  approveProposedAction,
  listPendingProposedActions,
  rejectProposedAction,
  type ProposedAction,
} from './proposedActions';

const backendAction = {
  id: 'proposed-1',
  client_id: 'client-1',
  action_type: 'create_activity',
  tool_name: 'create_activity_draft',
  entity_type: 'activity',
  entity_id: null,
  input_json: '{"title":"Follow up"}',
  proposed_output_json: '{"status":"draft"}',
  status: 'pending',
  created_at: '2026-06-24T08:00:00Z',
  approved_at: null,
  rejected_at: null,
  executed_at: null,
  device_id: 'device-1',
};

const action: ProposedAction = {
  id: 'proposed-1',
  clientId: 'client-1',
  actionType: 'create_activity',
  toolName: 'create_activity_draft',
  entityType: 'activity',
  entityId: null,
  inputJson: '{"title":"Follow up"}',
  proposedOutputJson: '{"status":"draft"}',
  status: 'pending',
  createdAt: '2026-06-24T08:00:00Z',
  approvedAt: null,
  rejectedAt: null,
  executedAt: null,
  deviceId: 'device-1',
};

describe('proposed actions API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps listPendingProposedActions to list_pending_proposed_actions', async () => {
    invokeMock.mockResolvedValueOnce([backendAction]);

    await expect(listPendingProposedActions()).resolves.toEqual([action]);

    expect(invokeMock).toHaveBeenCalledWith('list_pending_proposed_actions');
  });

  it('maps approveProposedAction to approve_proposed_action', async () => {
    const approvedBackendAction = {
      ...backendAction,
      status: 'executed',
      approved_at: '2026-06-24T08:05:00Z',
      executed_at: '2026-06-24T08:05:00Z',
    };
    invokeMock.mockResolvedValueOnce(approvedBackendAction);

    await expect(approveProposedAction('proposed-1')).resolves.toEqual({
      ...action,
      status: 'executed',
      approvedAt: '2026-06-24T08:05:00Z',
      executedAt: '2026-06-24T08:05:00Z',
    });

    expect(invokeMock).toHaveBeenCalledWith('approve_proposed_action', {
      id: 'proposed-1',
    });
  });

  it('maps rejectProposedAction to reject_proposed_action', async () => {
    const rejectedBackendAction = {
      ...backendAction,
      status: 'rejected',
      rejected_at: '2026-06-24T08:06:00Z',
    };
    invokeMock.mockResolvedValueOnce(rejectedBackendAction);

    await expect(rejectProposedAction('proposed-1')).resolves.toEqual({
      ...action,
      status: 'rejected',
      rejectedAt: '2026-06-24T08:06:00Z',
    });

    expect(invokeMock).toHaveBeenCalledWith('reject_proposed_action', {
      id: 'proposed-1',
    });
  });
});
