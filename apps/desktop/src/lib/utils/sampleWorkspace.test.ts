import { describe, expect, it, vi } from 'vitest';
import {
  SAMPLE_ACTIVITY_SUBJECT,
  SAMPLE_CONTACT_FIRST_NAME,
  SAMPLE_CONTACT_LAST_NAME,
  SAMPLE_DEAL_NAME,
  SAMPLE_ORGANIZATION_NAME,
  seedSampleWorkspace,
} from './sampleWorkspace';

describe('seedSampleWorkspace', () => {
  it('links the sample person to the created organization id', async () => {
    const createOrganization = vi.fn().mockResolvedValue({
      id: 'org-sample',
      name: SAMPLE_ORGANIZATION_NAME,
    });
    const createContact = vi.fn().mockResolvedValue({
      id: 'contact-sample',
      firstName: SAMPLE_CONTACT_FIRST_NAME,
      lastName: SAMPLE_CONTACT_LAST_NAME,
    });
    const createDeal = vi.fn().mockResolvedValue({ id: 'deal-sample' });
    const createActivity = vi.fn().mockResolvedValue({ id: 'activity-sample' });

    await expect(
      seedSampleWorkspace({
        currency: 'KES',
        now: new Date('2026-08-13T12:00:00Z'),
        createOrganization,
        createContact,
        createDeal,
        createActivity,
      }),
    ).resolves.toEqual({
      organizationId: 'org-sample',
      contactId: 'contact-sample',
      dealId: 'deal-sample',
      activityId: 'activity-sample',
    });

    expect(createContact).toHaveBeenCalledWith(
      expect.objectContaining({
        firstName: SAMPLE_CONTACT_FIRST_NAME,
        lastName: SAMPLE_CONTACT_LAST_NAME,
        organization: SAMPLE_ORGANIZATION_NAME,
        organizationId: 'org-sample',
        lifecycle: 'lead',
      }),
    );
    expect(createDeal).toHaveBeenCalledWith(
      expect.objectContaining({
        name: SAMPLE_DEAL_NAME,
        currency: 'KES',
        contactId: 'contact-sample',
        organizationId: 'org-sample',
      }),
    );
    expect(createActivity).toHaveBeenCalledWith(
      expect.objectContaining({
        subject: SAMPLE_ACTIVITY_SUBJECT,
        contactId: 'contact-sample',
        dealId: 'deal-sample',
      }),
    );
  });
});
