/**
 * src/lib/api/proposedActions.ts - Tauri IPC wrapper for pending proposed actions.
 */

import { invoke } from '@tauri-apps/api/core';

export interface ProposedAction {
  id: string;
  clientId: string | null;
  actionType: string;
  toolName: string;
  entityType: string | null;
  entityId: string | null;
  inputJson: string;
  proposedOutputJson: string | null;
  status: string;
  createdAt: string;
  approvedAt: string | null;
  rejectedAt: string | null;
  executedAt: string | null;
  deviceId: string;
}

interface BackendProposedAction {
  id: string;
  client_id: string | null;
  action_type: string;
  tool_name: string;
  entity_type: string | null;
  entity_id: string | null;
  input_json: string;
  proposed_output_json: string | null;
  status: string;
  created_at: string;
  approved_at: string | null;
  rejected_at: string | null;
  executed_at: string | null;
  device_id: string;
}

function mapProposedAction(action: BackendProposedAction): ProposedAction {
  return {
    id: action.id,
    clientId: action.client_id,
    actionType: action.action_type,
    toolName: action.tool_name,
    entityType: action.entity_type,
    entityId: action.entity_id,
    inputJson: action.input_json,
    proposedOutputJson: action.proposed_output_json,
    status: action.status,
    createdAt: action.created_at,
    approvedAt: action.approved_at,
    rejectedAt: action.rejected_at,
    executedAt: action.executed_at,
    deviceId: action.device_id,
  };
}

export async function listPendingProposedActions(): Promise<ProposedAction[]> {
  const actions = await invoke<BackendProposedAction[]>('list_pending_proposed_actions');
  return actions.map(mapProposedAction);
}

export async function approveProposedAction(id: string): Promise<ProposedAction> {
  const action = await invoke<BackendProposedAction>('approve_proposed_action', { id });
  return mapProposedAction(action);
}

export async function rejectProposedAction(id: string): Promise<ProposedAction> {
  const action = await invoke<BackendProposedAction>('reject_proposed_action', { id });
  return mapProposedAction(action);
}
