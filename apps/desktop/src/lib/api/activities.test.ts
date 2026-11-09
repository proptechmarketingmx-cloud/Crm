import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  addActivityLink,
  listActivities,
  listActivitiesForDeals,
  listActivityLinks,
  listActivityLinksForActivities,
  removeActivityLink,
  updateActivity,
} from './activities';
import { combineDueDateAndTime } from '../utils/activityDue';

const backendActivity = {
  id: 'activity-1',
  activity_type: 'task',
  title: 'Follow up',
  description: 'Call this week',
  due_date: '2026-07-15',
  completed: false,
  contact_id: 'contact-1',
  deal_id: 'deal-1',
  created_at: '2026-06-24T08:00:00Z',
  updated_at: '2026-06-24T09:00:00Z',
};

const backendActivityLink = {
  id: 'activity-link-1',
  activity_id: 'activity-1',
  entity_type: 'organization' as const,
  entity_id: 'org-1',
  created_at: '2026-06-24T08:30:00Z',
  deleted_at: null,
  device_id: 'device-1',
};

describe('activity API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('keeps date-only activities due today pending for the full local day', async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-07-08T18:30:00'));
    invokeMock.mockResolvedValueOnce([
      {
        ...backendActivity,
        id: 'today',
        due_date: '2026-07-08',
        completed: false,
      },
      {
        ...backendActivity,
        id: 'yesterday',
        due_date: '2026-07-07',
        completed: false,
      },
    ]);

    await expect(listActivities({ sortBy: 'dueDate', sortDir: 'asc' })).resolves.toMatchObject([
      { id: 'yesterday', status: 'overdue' },
      { id: 'today', status: 'pending' },
    ]);

    expect(invokeMock).toHaveBeenCalledWith('list_activities', {
      limit: 200,
      offset: 0,
    });
  });

  it('marks a timed due date overdue after that clock time', async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-07-08T16:30:00'));
    invokeMock.mockResolvedValueOnce([
      {
        ...backendActivity,
        id: 'afternoon',
        activity_type: 'visit',
        due_date: combineDueDateAndTime('2026-07-08', '15:00'),
        completed: false,
      },
    ]);

    await expect(listActivities()).resolves.toMatchObject([
      { id: 'afternoon', type: 'visit', status: 'overdue' },
    ]);
  });

  it('omits absent nullable update fields and sends explicit reset flags intentionally', async () => {
    invokeMock.mockResolvedValueOnce(backendActivity);

    await updateActivity('activity-1', { subject: 'Follow up' });

    let args = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    expect(args).toMatchObject({
      id: 'activity-1',
      title: 'Follow up',
    });
    expect(args).not.toHaveProperty('due_date');
    expect(args).not.toHaveProperty('reset_due_date');
    expect(args).not.toHaveProperty('contact_id');
    expect(args).not.toHaveProperty('reset_contact_id');
    expect(args).not.toHaveProperty('deal_id');
    expect(args).not.toHaveProperty('reset_deal_id');

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({
      ...backendActivity,
      due_date: null,
      contact_id: null,
      deal_id: null,
    });

    await updateActivity('activity-1', {
      dueDate: null,
      contactId: null,
      dealId: null,
    });

    args = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    expect(args).toMatchObject({
      id: 'activity-1',
      reset_due_date: true,
      reset_contact_id: true,
      reset_deal_id: true,
    });
    expect(args).not.toHaveProperty('due_date');
    expect(args).not.toHaveProperty('contact_id');
    expect(args).not.toHaveProperty('deal_id');

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({
      ...backendActivity,
      due_date: null,
      contact_id: null,
      deal_id: null,
    });

    await updateActivity('activity-1', {
      dueDate: '   ',
      contactId: '   ',
      dealId: '   ',
    });

    args = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    expect(args).toMatchObject({
      id: 'activity-1',
      reset_due_date: true,
      reset_contact_id: true,
      reset_deal_id: true,
    });
  });

  it('maps nullable activity update field sets without reset flags', async () => {
    invokeMock.mockResolvedValueOnce(backendActivity);

    await updateActivity('activity-1', {
      dueDate: ' 2026-08-01 ',
      contactId: ' contact-2 ',
      dealId: ' deal-2 ',
    });

    const args = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    expect(args).toMatchObject({
      id: 'activity-1',
      due_date: '2026-08-01',
      contact_id: 'contact-2',
      deal_id: 'deal-2',
    });
    expect(args).not.toHaveProperty('reset_due_date');
    expect(args).not.toHaveProperty('reset_contact_id');
    expect(args).not.toHaveProperty('reset_deal_id');
  });

  it('maps listActivitiesForDeals to the batch command', async () => {
    invokeMock.mockResolvedValueOnce([backendActivity]);

    await expect(listActivitiesForDeals([' deal-1 ', 'deal-1', ''])).resolves.toMatchObject([
      { id: 'activity-1', dealId: 'deal-1' },
    ]);
    expect(invokeMock).toHaveBeenCalledWith('list_activities_for_deals', {
      deal_ids: ['deal-1'],
    });

    invokeMock.mockReset();
    await expect(listActivitiesForDeals(['', '  '])).resolves.toEqual([]);
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('maps listActivityLinksForActivities to the batch command', async () => {
    invokeMock.mockResolvedValueOnce([backendActivityLink]);

    await expect(
      listActivityLinksForActivities([' activity-1 ', 'activity-1', '']),
    ).resolves.toEqual([
      {
        id: 'activity-link-1',
        activityId: 'activity-1',
        entityType: 'organization',
        entityId: 'org-1',
        createdAt: '2026-06-24T08:30:00Z',
        deletedAt: null,
        deviceId: 'device-1',
      },
    ]);
    expect(invokeMock).toHaveBeenCalledWith('list_activity_links_for_activities', {
      activity_ids: ['activity-1'],
    });

    invokeMock.mockReset();
    await expect(listActivityLinksForActivities([])).resolves.toEqual([]);
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('maps activity link wrappers to Tauri commands', async () => {
    invokeMock.mockResolvedValueOnce([backendActivityLink]);

    await expect(listActivityLinks(' activity-1 ')).resolves.toEqual([
      {
        id: 'activity-link-1',
        activityId: 'activity-1',
        entityType: 'organization',
        entityId: 'org-1',
        createdAt: '2026-06-24T08:30:00Z',
        deletedAt: null,
        deviceId: 'device-1',
      },
    ]);
    expect(invokeMock).toHaveBeenCalledWith('list_activity_links', {
      activity_id: 'activity-1',
    });

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce(backendActivityLink);

    await expect(addActivityLink(' activity-1 ', 'organization', ' org-1 ')).resolves.toMatchObject({
      id: 'activity-link-1',
      entityType: 'organization',
      entityId: 'org-1',
    });
    expect(invokeMock).toHaveBeenCalledWith('add_activity_link', {
      activity_id: 'activity-1',
      entity_type: 'organization',
      entity_id: 'org-1',
    });

    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce({ ...backendActivityLink, deleted_at: '2026-06-24T10:00:00Z' });

    await expect(removeActivityLink('activity-1', 'organization', 'org-1')).resolves.toMatchObject({
      id: 'activity-link-1',
      deletedAt: '2026-06-24T10:00:00Z',
    });
    expect(invokeMock).toHaveBeenCalledWith('remove_activity_link', {
      activity_id: 'activity-1',
      entity_type: 'organization',
      entity_id: 'org-1',
    });
  });
});
