/**
 * Synthetic first-run workspace used by the dashboard starter.
 * The person contact must be linked through organizationId so Account 360
 * people lists are not empty.
 */

import {
  createActivity,
  type Activity,
  type CreateActivityPayload,
} from '$lib/api/activities';
import {
  createContact,
  type Contact,
  type CreateContactPayload,
} from '$lib/api/contacts';
import {
  createDeal,
  type CreateDealPayload,
  type Deal,
} from '$lib/api/deals';
import {
  createOrganization,
  type CreateOrganizationPayload,
  type Organization,
} from '$lib/api/organizations';

export const SAMPLE_ORGANIZATION_NAME = 'Northstar Cooperative';
export const SAMPLE_CONTACT_FIRST_NAME = 'Amara';
export const SAMPLE_CONTACT_LAST_NAME = 'Okafor';
export const SAMPLE_DEAL_NAME = 'Solar inventory rollout';
export const SAMPLE_ACTIVITY_SUBJECT = 'Call Amara about rollout timeline';

export interface SampleWorkspaceIds {
  organizationId: string;
  contactId: string;
  dealId: string;
  activityId: string;
}

export interface SampleWorkspaceDeps {
  currency: string;
  now?: Date;
  createOrganization: (data: CreateOrganizationPayload) => Promise<Organization>;
  createContact: (data: CreateContactPayload) => Promise<Contact>;
  createDeal: (data: CreateDealPayload) => Promise<Deal>;
  createActivity: (data: CreateActivityPayload) => Promise<Activity>;
}

function futureIsoDate(daysFromNow: number, now: Date): string {
  const date = new Date(now);
  date.setDate(date.getDate() + daysFromNow);
  return date.toISOString().slice(0, 10);
}

export function buildDefaultSampleWorkspaceDeps(
  currency: string,
): SampleWorkspaceDeps {
  return {
    currency,
    createOrganization,
    createContact,
    createDeal,
    createActivity,
  };
}

export async function seedSampleWorkspace(
  deps: SampleWorkspaceDeps,
): Promise<SampleWorkspaceIds> {
  const now = deps.now ?? new Date();

  const organization = await deps.createOrganization({
    name: SAMPLE_ORGANIZATION_NAME,
    email: 'hello@northstar.example',
    phone: '+1 555 0140',
    website: 'https://northstar.example',
    city: 'Austin',
    region: 'TX',
    country: 'United States',
    description: 'Sample account for reviewing 900CRM workflows.',
  });

  const contact = await deps.createContact({
    firstName: SAMPLE_CONTACT_FIRST_NAME,
    lastName: SAMPLE_CONTACT_LAST_NAME,
    email: 'amara@northstar.example',
    phone: '+1 555 0141',
    organization: organization.name,
    organizationId: organization.id,
    type: 'person',
    lifecycle: 'lead',
    tags: [],
    notes: 'Sample contact created by the dashboard starter.',
    website: null,
    address: '120 Market Street',
  });

  const deal = await deps.createDeal({
    name: SAMPLE_DEAL_NAME,
    value: 18500,
    currency: deps.currency,
    stage: 'proposal',
    probability: 65,
    expectedCloseDate: futureIsoDate(21, now),
    contactId: contact.id,
    organizationId: organization.id,
    description: 'Sample opportunity for a staged inventory rollout.',
    tags: [],
  });

  const activity = await deps.createActivity({
    type: 'call',
    subject: SAMPLE_ACTIVITY_SUBJECT,
    notes: 'Confirm stakeholders, target install dates, and next quote details.',
    dueDate: futureIsoDate(2, now),
    contactId: contact.id,
    dealId: deal.id,
  });

  return {
    organizationId: organization.id,
    contactId: contact.id,
    dealId: deal.id,
    activityId: activity.id,
  };
}
