import { expect, loadHashRoute, test } from './tauri-shim';

test.describe.configure({ mode: 'serial' });

function localDateInputValue(date = new Date()): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

test('loads the dashboard sample workspace and shows the follow-up on the dashboard', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Start your workspace' })).toBeVisible();

  await page.getByRole('button', { name: 'Load sample workspace' }).click();
  await expect(page.getByText('Sample workspace loaded.').first()).toBeVisible();
  await expect(page.getByText('Call Amara about rollout timeline')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Load sample workspace' })).toBeHidden();

  const storedSampleState = async () =>
    page.evaluate(() => window.localStorage.getItem('900crm.browser-smoke.state') ?? '');
  await expect.poll(storedSampleState).toContain('Amara');
  await expect.poll(storedSampleState).toContain('Northstar Cooperative');
  await expect.poll(storedSampleState).toContain('Solar inventory rollout');
  await expect.poll(storedSampleState).toContain('Call Amara about rollout timeline');

  const sampleOrganizationId = await page.evaluate(() => {
    const raw = window.localStorage.getItem('900crm.browser-smoke.state');
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as {
      organizations?: Array<{ id: string; name: string }>;
    };
    return parsed.organizations?.find((organization) => organization.name === 'Northstar Cooperative')
      ?.id ?? null;
  });
  expect(sampleOrganizationId).toBeTruthy();

  await loadHashRoute(page, `/organizations/${sampleOrganizationId}`);
  await expect(page.getByRole('heading', { name: 'Northstar Cooperative' })).toBeVisible();
  const sampleWorkspace = page.locator('.account-workspace');
  await expect(
    sampleWorkspace.locator('.workspace-metric').filter({ hasText: 'People' }).getByText('1'),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'Amara Okafor' })).toBeVisible();

  const sampleDealId = await page.evaluate(() => {
    const raw = window.localStorage.getItem('900crm.browser-smoke.state');
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as {
      deals?: Array<{ id: string; title?: string; name?: string }>;
    };
    return parsed.deals?.find((deal) => (deal.title ?? deal.name) === 'Solar inventory rollout')?.id ?? null;
  });
  expect(sampleDealId).toBeTruthy();

  await loadHashRoute(page, `/pipeline/${sampleDealId}`);
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Solar inventory rollout' })).toBeVisible();

  await page.getByRole('button', { name: 'Open deal' }).click();
  await expect(page).toHaveURL(new RegExp(`#/deals/${sampleDealId}$`));
  await expect(page.getByRole('heading', { name: 'Solar inventory rollout' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Deal Summary' })).toBeVisible();
  await expect(page.getByTestId('next-step')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'This deal has gone quiet' })).toBeVisible();
  await expect(
    page.getByText('Next follow-up is still Call Amara about rollout timeline.'),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'Amara Okafor', exact: true })).toBeVisible();

  await assertNoConsoleErrors();
});

test('shows a dashboard attention queue for overdue work, stuck deals, and waiting leads', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/');

  await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    await invoke('create_contact', {
      contact_type: 'person',
      first_name: 'Zuri',
      last_name: 'Ndlovu',
      org_name: '',
      email: 'zuri@example.test',
      phone: '',
      address: '',
      city: '',
      country: '',
      org_id: '',
      notes: '',
      lifecycle: 'lead',
    });

    await invoke('create_deal', {
      title: 'Unworked Clinic Kit',
      value: 4200,
      currency: 'USD',
      stage: 'Lead',
      probability: 20,
      expected_close: '',
      contact_id: '',
      organization_id: '',
      notes: '',
    });

    await invoke('create_activity', {
      activity_type: 'task',
      title: 'Past due clinic check-in',
      description: '',
      due_date: '2020-01-15',
      contact_id: '',
      deal_id: '',
    });
  });

  await loadHashRoute(page, '/');
  const queue = page.getByTestId('dashboard-attention-strip');
  await expect(queue.getByRole('heading', { name: 'Needs Attention' })).toBeVisible();
  await expect(queue.getByRole('button', { name: /Past due clinic check-in/ })).toBeVisible();
  await expect(queue.getByRole('button', { name: /Unworked Clinic Kit/ })).toBeVisible();
  await expect(queue.getByRole('button', { name: /Zuri Ndlovu/ })).toBeVisible();

  await queue.getByRole('button', { name: /Zuri Ndlovu/ }).click();
  await expect(page).toHaveURL(/#\/contacts\//);
  await expect(page.getByRole('heading', { name: 'Zuri Ndlovu' })).toBeVisible();

  await loadHashRoute(page, '/');
  await page.getByTestId('dashboard-attention-strip').getByRole('button', { name: /Unworked Clinic Kit/ }).click();
  await expect(page).toHaveURL(/#\/deals\//);
  await expect(page.getByRole('heading', { name: 'Unworked Clinic Kit' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('creates a contact through the visible UI and shows it in Contacts and global search', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Contact' });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel('First Name').fill('Ada');
  await dialog.getByLabel('Last Name').fill('Lovelace');
  await dialog.getByLabel('Email').fill('ada@example.test');
  await dialog.getByLabel('Phone').fill('+1 555 0101');
  await dialog.getByLabel('Organization').fill('Analytical Engine Guild');
  await dialog.getByRole('button', { name: 'Save' }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByText('Ada Lovelace')).toBeVisible();
  await expect(page.getByText('ada@example.test')).toBeVisible();

  await page.getByRole('searchbox', { name: 'Search', exact: true }).fill('Ada');
  const searchResults = page.getByRole('listbox', { name: 'Search results' });
  await expect(searchResults).toBeVisible();
  await expect(searchResults.getByText('Ada Lovelace')).toBeVisible();
  await expect(searchResults.getByText('Contacts')).toBeVisible();

  await assertNoConsoleErrors();
});

test('shows contact list health and the next follow-up', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Contact' });
  await dialog.getByLabel('First Name').fill('Zara');
  await dialog.getByLabel('Last Name').fill('Boateng');
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();

  await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    const listed = await invoke('list_contacts', {
      params: {
        page: 1,
        per_page: 50,
        search_query: 'Zara',
      },
    }) as { contacts?: Array<{ id: string; first_name: string; last_name: string }> };
    const contact = listed.contacts?.find((candidate) =>
      candidate.first_name === 'Zara' && candidate.last_name === 'Boateng'
    );
    if (!contact) {
      throw new Error('Seed contact was not created.');
    }

    await invoke('create_deal', {
      title: 'Clinic kit',
      value: 6400,
      currency: 'USD',
      stage: 'Proposal',
      probability: 35,
      expected_close: '',
      contact_id: contact.id,
      organization_id: '',
      notes: '',
    });

    await invoke('create_activity', {
      activity_type: 'task',
      title: 'Past due clinic check-in',
      description: '',
      due_date: '2020-01-15',
      contact_id: contact.id,
      deal_id: '',
    });
  });

  await loadHashRoute(page, '/contacts');
  const row = page.locator('tr').filter({ hasText: 'Zara Boateng' });
  await expect(row.getByText('Overdue')).toBeVisible();
  await expect(row.getByText('Past due clinic check-in')).toBeVisible();

  await assertNoConsoleErrors();
});

test('contact workspace next step completes an overdue follow-up', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  const seed = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    const contact = await invoke('create_contact', {
      contact_type: 'person',
      first_name: 'Imani',
      last_name: 'Diallo',
      org_name: '',
      email: 'imani.next@example.test',
      phone: '',
      address: '',
      city: '',
      country: '',
      org_id: '',
      notes: '',
      lifecycle: 'customer',
    }) as { id: string };

    await invoke('create_deal', {
      title: 'Imani clinic kit',
      value: 2800,
      currency: 'USD',
      stage: 'Proposal',
      probability: 40,
      expected_close: '',
      contact_id: contact.id,
      organization_id: '',
      notes: '',
    });

    await invoke('create_activity', {
      activity_type: 'task',
      title: 'Past due Imani check-in',
      description: '',
      due_date: '2020-01-15',
      contact_id: contact.id,
      deal_id: '',
    });

    return { contactId: contact.id };
  });

  await loadHashRoute(page, `/contacts/${seed.contactId}`);
  const workspace = page.locator('.customer-workspace');
  await expect(workspace.getByRole('heading', { name: 'Complete Past due Imani check-in' })).toBeVisible();
  await workspace.getByTestId('next-step').getByRole('button', { name: 'Mark Complete' }).click();
  await expect(workspace.getByRole('heading', { name: 'Schedule a follow-up' })).toBeVisible();
  await workspace.getByTestId('next-step').getByRole('button', { name: 'Add Follow-Up' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Activity' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: 'Cancel' }).click();

  await assertNoConsoleErrors();
});

test('creates a lead, filters the lead list, and converts it to a customer', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts', exact: true })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Lead' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Contact' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByLabel('Lifecycle')).toHaveValue('lead');
  await dialog.getByLabel('First Name').fill('Kofi');
  await dialog.getByLabel('Last Name').fill('Mensah');
  await dialog.getByLabel('Email').fill('kofi.lead@example.test');
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();

  await page.getByRole('group', { name: 'Lifecycle' }).getByRole('button', { name: 'Leads' }).click();
  await expect(page.getByText('Kofi Mensah')).toBeVisible();
  await expect(page.getByText('Lead').first()).toBeVisible();

  await page.getByText('Kofi Mensah').click();
  await expect(page.getByRole('heading', { name: 'Kofi Mensah' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Lead Summary' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Convert this lead' })).toBeVisible();
  await page.getByTestId('next-step').getByRole('button', { name: 'Convert to customer' }).click();
  await expect(page.getByText('Customer').first()).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Customer 360 Summary' })).toBeVisible();

  await loadHashRoute(page, '/contacts');
  await page.getByRole('group', { name: 'Lifecycle' }).getByRole('button', { name: 'Customers' }).click();
  await expect(page.getByText('Kofi Mensah')).toBeVisible();

  await assertNoConsoleErrors();
});

test('shows only leads on the Leads list and drops them after convert', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/leads');
  await expect(page.getByRole('heading', { name: 'Leads', exact: true })).toBeVisible();
  await expect(page.getByText('People waiting to be worked')).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Lead' }).click();
  const leadDialog = page.getByRole('dialog', { name: 'Add Contact' });
  await expect(leadDialog.getByLabel('Lifecycle')).toHaveValue('lead');
  await leadDialog.getByLabel('First Name').fill('Amina');
  await leadDialog.getByLabel('Last Name').fill('Leadstone');
  await leadDialog.getByRole('button', { name: 'Save' }).click();
  await expect(leadDialog).toBeHidden();
  await expect(page.getByText('Amina Leadstone')).toBeVisible();

  await loadHashRoute(page, '/contacts');
  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const customerDialog = page.getByRole('dialog', { name: 'Add Contact' });
  await customerDialog.getByLabel('First Name').fill('Ibrahim');
  await customerDialog.getByLabel('Last Name').fill('Customerstone');
  await customerDialog.getByRole('button', { name: 'Save' }).click();
  await expect(customerDialog).toBeHidden();

  await loadHashRoute(page, '/leads');
  await expect(page.getByText('Amina Leadstone')).toBeVisible();
  await expect(page.getByText('Ibrahim Customerstone')).toHaveCount(0);

  await page.getByText('Amina Leadstone').click();
  await expect(page.getByRole('heading', { name: 'Amina Leadstone' })).toBeVisible();
  await page.getByTestId('next-step').getByRole('button', { name: 'Convert to customer' }).click();

  await loadHashRoute(page, '/leads');
  await expect(page.getByText('Amina Leadstone')).toHaveCount(0);

  await loadHashRoute(page, '/contacts');
  await page.getByRole('group', { name: 'Lifecycle' }).getByRole('button', { name: 'Customers' }).click();
  await expect(page.getByText('Amina Leadstone')).toBeVisible();
  await expect(page.getByText('Ibrahim Customerstone')).toBeVisible();

  await assertNoConsoleErrors();
});

test('saves the current contact filters as a named view and applies it later', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await page.locator('.page-header').getByRole('button', { name: 'Add Lead' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Contact' });
  await dialog.getByLabel('First Name').fill('Sana');
  await dialog.getByLabel('Last Name').fill('Diallo');
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();

  await page.getByRole('group', { name: 'Lifecycle' }).getByRole('button', { name: 'Leads' }).click();
  await expect(page.getByText('Sana Diallo')).toBeVisible();

  await page.getByLabel('View name').fill('New leads');
  await page.getByRole('button', { name: 'Save view' }).click();
  await expect(page.getByLabel('Saved view', { exact: true })).toHaveValue(/view-/);

  await page.getByRole('group', { name: 'Lifecycle' }).getByRole('button', { name: 'All' }).click();
  await page.getByLabel('Saved view', { exact: true }).selectOption({ label: 'New leads' });
  await expect(page.getByRole('group', { name: 'Lifecycle' }).getByRole('button', { name: 'Leads' })).toHaveClass(/active/);
  await expect(page.getByText('Sana Diallo')).toBeVisible();

  await assertNoConsoleErrors();
});

test('saves a website bookmark on a contact workspace', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Contact' });
  await dialog.getByLabel('First Name').fill('Imani');
  await dialog.getByLabel('Last Name').fill('Okello');
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();

  await page.getByText('Imani Okello').click();
  await expect(page.getByRole('heading', { name: 'Imani Okello' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Links' })).toBeVisible();
  await expect(page.getByText('does not copy or upload the file')).toBeVisible();

  await page.getByLabel('Link title').fill('Clinic map');
  await page.getByLabel('Website URL').fill('https://maps.example/clinic');
  await page.getByRole('button', { name: 'Add website' }).click();
  await expect(page.getByRole('button', { name: 'Clinic map' })).toBeVisible();
  await expect(page.getByText('https://maps.example/clinic')).toBeVisible();

  await assertNoConsoleErrors();
});

test('shows a customer 360 summary for a contact with linked sales work', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const contactDialog = page.getByRole('dialog', { name: 'Add Contact' });
  await expect(contactDialog).toBeVisible();

  await contactDialog.getByLabel('First Name').fill('Maya');
  await contactDialog.getByLabel('Last Name').fill('Chen');
  await contactDialog.getByLabel('Email').fill('maya@example.test');
  await contactDialog.getByLabel('Phone').fill('+1 555 0303');
  await contactDialog.getByLabel('Organization').fill('Greenfield Solar');
  await contactDialog.getByRole('button', { name: 'Save' }).click();

  await expect(contactDialog).toBeHidden();
  await expect(page.getByText('Maya Chen')).toBeVisible();

  const seed = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    const contacts = await invoke('list_contacts', {
      params: {
        page: 1,
        per_page: 50,
        search_query: 'Maya',
      },
    }) as { contacts: Array<{ id: string }> };
    const contact = contacts.contacts[0];
    if (!contact) {
      throw new Error('Seed contact was not created.');
    }
    const futureDate = new Date(Date.now() + 10 * 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 10);
    const linkedOnlyDate = new Date(Date.now() + 11 * 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 10);

    await invoke('create_deal', {
      title: 'Solar upgrade expansion',
      value: 42000,
      currency: 'USD',
      stage: 'Proposal',
      probability: 50,
      expected_close: '2026-07-31',
      contact_id: contact.id,
      organization_id: '',
      notes: 'Expansion opportunity.',
    });

    await invoke('create_activity', {
      activity_type: 'call',
      title: 'Call Maya about implementation timeline',
      description: '',
      due_date: futureDate,
      contact_id: contact.id,
      deal_id: '',
    });

    const linkedOnlyActivity = await invoke('create_activity', {
      activity_type: 'meeting',
      title: 'Linked-only relationship review',
      description: '',
      due_date: linkedOnlyDate,
      contact_id: '',
      deal_id: '',
    }) as { id: string };

    await invoke('add_activity_link', {
      activity_id: linkedOnlyActivity.id,
      entity_type: 'contact',
      entity_id: contact.id,
    });

    return { contactId: contact.id };
  });

  await loadHashRoute(page, `/contacts/${seed.contactId}`);
  await expect(page).toHaveURL(new RegExp(`#/contacts/${seed.contactId}$`));
  await expect(page.getByRole('heading', { name: 'Maya Chen' })).toBeVisible();

  const workspace = page.locator('.customer-workspace');
  await expect(workspace.getByRole('heading', { name: 'Customer 360 Summary' })).toBeVisible();
  await expect(workspace.getByText('On Track')).toBeVisible();
  await expect(
    workspace.getByRole('heading', { name: 'Call Maya about implementation timeline is scheduled' }),
  ).toBeVisible();
  await expect(workspace.locator('.workspace-metric').filter({ hasText: 'Open Deals' }).getByText('1')).toBeVisible();
  await expect(workspace.getByText('$42,000')).toBeVisible();
  await expect(
    workspace.locator('.workspace-metric').filter({ hasText: 'Next Follow-Up' })
      .getByText('Call Maya about implementation timeline'),
  ).toBeVisible();
  const contactTimeline = page.locator('.detail-activity');
  await expect(contactTimeline.getByText('Linked-only relationship review')).toBeVisible();
  await expect(
    contactTimeline.getByRole('button', { name: 'Open Contact Maya Chen' }).first(),
  ).toBeVisible();

  await workspace.getByRole('button', { name: 'Add Follow-Up' }).click();
  let dialog = page.getByRole('dialog', { name: 'Add Activity' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByLabel('Contact')).toHaveValue(seed.contactId);
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(dialog).toBeHidden();

  await workspace.getByRole('button', { name: 'Add Deal' }).click();
  dialog = page.getByRole('dialog', { name: 'Add Deal' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByLabel('Contact')).toHaveValue(seed.contactId);
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(dialog).toBeHidden();

  await assertNoConsoleErrors();
});

test('creates an organization through the visible UI and shows it in Organizations', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/organizations');
  await expect(page.getByRole('heading', { name: 'Organizations', exact: true })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Organization' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Organization' });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel('Name').fill('Atlas Cooperative');
  await dialog.getByLabel('Email').fill('hello@atlas.example');
  await dialog.getByLabel('Phone').fill('+1 555 0202');
  await dialog.getByLabel('Website').fill('https://atlas.example');
  await dialog.getByLabel('City').fill('Lagos');
  await dialog.getByLabel('Country').fill('Nigeria');
  await dialog.getByLabel('Description').fill('Regional partner for solar deployments.');
  await dialog.getByRole('button', { name: 'Create Organization' }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByText('Atlas Cooperative')).toBeVisible();
  await expect(page.getByText('hello@atlas.example')).toBeVisible();
  await expect(page.getByText('Lagos, Nigeria')).toBeVisible();

  await assertNoConsoleErrors();
});

test('shows organization list health and the next account follow-up', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/organizations');
  await page.locator('.page-header').getByRole('button', { name: 'Add Organization' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Organization' });
  await dialog.getByLabel('Name').fill('Sahel Grid');
  await dialog.getByLabel('City').fill('Niamey');
  await dialog.getByLabel('Country').fill('Niger');
  await dialog.getByRole('button', { name: 'Create Organization' }).click();
  await expect(dialog).toBeHidden();

  await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    const organizations = await invoke('list_organizations') as Array<{ id: string; name: string }>;
    const organization = organizations.find((candidate) => candidate.name === 'Sahel Grid');
    if (!organization) {
      throw new Error('Seed organization was not created.');
    }

    await invoke('create_deal', {
      title: 'Grid expansion',
      value: 18000,
      currency: 'USD',
      stage: 'Proposal',
      probability: 40,
      expected_close: '',
      contact_id: '',
      organization_id: organization.id,
      notes: '',
    });

    const activity = await invoke('create_activity', {
      activity_type: 'task',
      title: 'Past due clinic check-in',
      description: '',
      due_date: '2020-01-15',
      contact_id: '',
      deal_id: '',
    }) as { id: string };

    await invoke('add_activity_link', {
      activity_id: activity.id,
      entity_type: 'organization',
      entity_id: organization.id,
    });
  });

  await loadHashRoute(page, '/organizations');
  const row = page.locator('tr').filter({ hasText: 'Sahel Grid' });
  await expect(row.getByText('Overdue')).toBeVisible();
  await expect(row.getByText('Past due clinic check-in')).toBeVisible();

  await assertNoConsoleErrors();
});

test('saves the current organization filters as a named view and applies it later', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/organizations');
  await page.locator('.page-header').getByRole('button', { name: 'Add Organization' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Organization' });
  await dialog.getByLabel('Name').fill('Rift Valley Clinics');
  await dialog.getByLabel('City').fill('Nakuru');
  await dialog.getByLabel('Country').fill('Kenya');
  await dialog.getByRole('button', { name: 'Create Organization' }).click();
  await expect(dialog).toBeHidden();

  await page.locator('.country-filter').selectOption('Kenya');
  await expect(page.getByText('Rift Valley Clinics')).toBeVisible();

  await page.getByLabel('View name').fill('Kenya accounts');
  await page.getByRole('button', { name: 'Save view' }).click();
  await expect(page.getByLabel('Saved view', { exact: true })).toHaveValue(/view-/);

  await page.locator('.country-filter').selectOption('');
  await page.getByLabel('Saved view', { exact: true }).selectOption({ label: 'Kenya accounts' });
  await expect(page.locator('.country-filter')).toHaveValue('Kenya');
  await expect(page.getByText('Rift Valley Clinics')).toBeVisible();

  await assertNoConsoleErrors();
});

test('saves the current pipeline filters as a named view and applies it later', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const firstDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await firstDialog.getByLabel('Deal Name').fill('Rift Clinic Electrification');
  await firstDialog.getByLabel('Value').fill('18000');
  await firstDialog.getByRole('button', { name: 'Save' }).click();
  await expect(firstDialog).toBeHidden();

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const secondDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await secondDialog.getByLabel('Deal Name').fill('Harbor Wind Maintenance');
  await secondDialog.getByLabel('Value').fill('9400');
  await secondDialog.getByRole('button', { name: 'Save' }).click();
  await expect(secondDialog).toBeHidden();

  const dealSearch = page.locator('.deal-search');
  await dealSearch.fill('Rift Clinic');
  await expect(page.getByRole('button', { name: /Rift Clinic Electrification/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Harbor Wind Maintenance/ })).toHaveCount(0);

  await page.getByLabel('View name').fill('Clinic deals');
  await page.getByRole('button', { name: 'Save view' }).click();
  await expect(page.getByLabel('Saved view', { exact: true })).toHaveValue(/view-/);

  await dealSearch.fill('');
  await expect(page.getByRole('button', { name: /Harbor Wind Maintenance/ })).toBeVisible();

  await page.getByLabel('Saved view', { exact: true }).selectOption({ label: 'Clinic deals' });
  await expect(dealSearch).toHaveValue('Rift Clinic');
  await expect(page.getByRole('button', { name: /Rift Clinic Electrification/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Harbor Wind Maintenance/ })).toHaveCount(0);

  await assertNoConsoleErrors();
});

test('filters the pipeline board by deals that need a follow-up', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const quietDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await quietDialog.getByLabel('Deal Name').fill('Needs Follow Clinic Kit');
  await quietDialog.getByLabel('Value').fill('7200');
  await quietDialog.getByRole('button', { name: 'Save' }).click();
  await expect(quietDialog).toBeHidden();

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const scheduledDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await scheduledDialog.getByLabel('Deal Name').fill('Scheduled Harbor Kit');
  await scheduledDialog.getByLabel('Value').fill('4300');
  await scheduledDialog.getByRole('button', { name: 'Save' }).click();
  await expect(scheduledDialog).toBeHidden();

  const laterDate = new Date(Date.now() + 10 * 24 * 60 * 60 * 1000)
    .toISOString()
    .slice(0, 10);
  await page.evaluate(async (dueDate) => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }
    const deals = await invoke('list_deals') as Array<{ id: string; title?: string }>;
    const scheduled = deals.find((deal) => deal.title === 'Scheduled Harbor Kit');
    if (!scheduled) {
      throw new Error('Scheduled deal was not created.');
    }
    await invoke('create_activity', {
      activity_type: 'task',
      title: 'Harbor site visit',
      description: '',
      due_date: dueDate,
      contact_id: '',
      deal_id: scheduled.id,
    });
  }, laterDate);

  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('button', { name: /Needs Follow Clinic Kit/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Scheduled Harbor Kit/ })).toBeVisible();

  await page.getByRole('group', { name: 'Attention' }).getByRole('button', { name: 'Needs Follow-Up' }).click();
  await expect(page.getByRole('button', { name: /Needs Follow Clinic Kit/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Scheduled Harbor Kit/ })).toHaveCount(0);

  await page.getByLabel('View name').fill('Needs follow-up');
  await page.getByRole('button', { name: 'Save view' }).click();
  await expect(page.getByLabel('Saved view', { exact: true })).toHaveValue(/view-/);

  await page.getByRole('group', { name: 'Attention' }).getByRole('button', { name: 'All' }).click();
  await expect(page.getByRole('button', { name: /Scheduled Harbor Kit/ })).toBeVisible();

  await page.getByLabel('Saved view', { exact: true }).selectOption({ label: 'Needs follow-up' });
  await expect(page.getByRole('group', { name: 'Attention' }).getByRole('button', { name: 'Needs Follow-Up' })).toHaveClass(/active/);
  await expect(page.getByRole('button', { name: /Needs Follow Clinic Kit/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Scheduled Harbor Kit/ })).toHaveCount(0);

  await assertNoConsoleErrors();
});

test('shows an account 360 workspace for an organization with linked work', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/organizations');
  await expect(page.getByRole('heading', { name: 'Organizations', exact: true })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Organization' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Organization' });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel('Name').fill('Helios Account');
  await dialog.getByLabel('Email').fill('hello@helios.example');
  await dialog.getByLabel('Website').fill('https://helios.example');
  await dialog.getByLabel('City').fill('Nairobi');
  await dialog.getByLabel('Country').fill('Kenya');
  await dialog.getByLabel('Description').fill('Regional account for off-grid clinics.');
  await dialog.getByRole('button', { name: 'Create Organization' }).click();
  await expect(dialog).toBeHidden();

  const seed = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }
    const futureDate = new Date(Date.now() + 14 * 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 10);

    const organizations = await invoke('list_organizations') as Array<{ id: string; name: string }>;
    const organization = organizations.find((candidate) => candidate.name === 'Helios Account');
    if (!organization) {
      throw new Error('Seed organization was not created.');
    }

    const contact = await invoke('create_contact', {
      contact_type: 'person',
      first_name: 'Nia',
      last_name: 'Mensah',
      org_name: 'Helios Account',
      email: 'nia@helios.example',
      phone: '+254 555 0101',
      address: '',
      city: '',
      country: '',
      org_id: '',
      notes: '',
    }) as { id: string };

    await invoke('link_contact_to_organization', {
      contact_id: contact.id,
      organization_id: organization.id,
    });

    const deal = await invoke('create_deal', {
      title: 'Clinic electrification rollout',
      value: 73000,
      currency: 'USD',
      stage: 'Proposal',
      probability: 50,
      expected_close: '2026-08-15',
      contact_id: '',
      organization_id: organization.id,
      notes: 'Account-level expansion opportunity.',
    }) as { id: string };

    const activity = await invoke('create_activity', {
      activity_type: 'meeting',
      title: 'Review Helios implementation plan',
      description: '',
      due_date: futureDate,
      contact_id: '',
      deal_id: '',
    }) as { id: string };

    await invoke('add_activity_link', {
      activity_id: activity.id,
      entity_type: 'organization',
      entity_id: organization.id,
    });

    await invoke('add_activity_link', {
      activity_id: activity.id,
      entity_type: 'deal',
      entity_id: deal.id,
    });

    return { organizationId: organization.id, contactId: contact.id, dealId: deal.id };
  });

  await loadHashRoute(page, `/organizations/${seed.organizationId}`);
  await expect(page).toHaveURL(new RegExp(`#/organizations/${seed.organizationId}$`));
  await expect(page.getByRole('heading', { name: 'Helios Account' })).toBeVisible();

  const workspace = page.locator('.account-workspace');
  await expect(workspace.getByRole('heading', { name: 'Account 360 Summary' })).toBeVisible();
  await expect(workspace.getByText('On Track')).toBeVisible();
  await expect(
    workspace.getByRole('heading', { name: 'Review Helios implementation plan is scheduled' }),
  ).toBeVisible();
  await expect(workspace.locator('.workspace-metric').filter({ hasText: 'People' }).getByText('1')).toBeVisible();
  await expect(workspace.locator('.workspace-metric').filter({ hasText: 'Open Deals' }).getByText('1')).toBeVisible();
  await expect(workspace.getByText('$73,000')).toBeVisible();
  await expect(
    workspace.locator('.workspace-metric').filter({ hasText: 'Next Follow-Up' })
      .getByText('Review Helios implementation plan'),
  ).toBeVisible();

  await expect(page.getByRole('button', { name: 'Nia Mensah' })).toBeVisible();
  await expect(
    page.getByLabel('Linked Deals').getByText('Clinic electrification rollout'),
  ).toBeVisible();
  await expect(
    page.getByLabel('Account Activity').getByText('Review Helios implementation plan'),
  ).toBeVisible();
  const accountActivity = page.getByLabel('Account Activity');
  await expect(
    accountActivity.getByRole('button', { name: 'Open Organization Helios Account' }).first(),
  ).toBeVisible();
  await expect(accountActivity.getByText('Deal')).toBeVisible();
  await expect(accountActivity.getByText('Clinic electrification rollout')).toBeVisible();

  await workspace.getByRole('button', { name: 'Add Follow-Up' }).click();
  let accountDialog = page.getByRole('dialog', { name: 'Add Activity' });
  await expect(accountDialog).toBeVisible();
  await expect(accountDialog.getByLabel('Organization')).toHaveValue(seed.organizationId);
  await accountDialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(accountDialog).toBeHidden();

  await workspace.getByRole('button', { name: 'Add Deal' }).click();
  accountDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await expect(accountDialog).toBeVisible();
  await expect(accountDialog.getByLabel('Organization')).toHaveValue(seed.organizationId);
  await accountDialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(accountDialog).toBeHidden();

  await loadHashRoute(page, '/');
  await page.getByRole('searchbox', { name: 'Search', exact: true }).fill('Helios');
  const searchResults = page.getByRole('listbox', { name: 'Search results' });
  await expect(searchResults).toBeVisible();
  await searchResults.getByText('Helios Account').click();
  await expect(page).toHaveURL(new RegExp(`#/organizations/${seed.organizationId}$`));

  await assertNoConsoleErrors();
});

test('routes global contact search results into direct contact detail outside the loaded list page', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/');

  const seed = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    for (let index = 0; index < 55; index += 1) {
      await invoke('create_contact', {
        contact_type: 'person',
        first_name: `Contact ${String(index).padStart(2, '0')}`,
        last_name: 'Paged',
        org_name: '',
        email: `contact-${index}@paged.example`,
        phone: '',
        address: '',
        city: '',
        country: '',
        org_id: '',
        notes: '',
      });
    }

    const target = await invoke('create_contact', {
      contact_type: 'person',
      first_name: 'Zzzara',
      last_name: 'Searchonly',
      org_name: '',
      email: 'zzzara.searchonly@example.test',
      phone: '',
      address: '',
      city: '',
      country: '',
      org_id: '',
      notes: '',
    }) as { id: string };

    return { contactId: target.id };
  });

  await page.getByRole('searchbox', { name: 'Search', exact: true }).fill('Zzzara');
  const searchResults = page.getByRole('listbox', { name: 'Search results' });
  await expect(searchResults).toBeVisible();
  await searchResults.getByText('Zzzara Searchonly').click();

  await expect(page).toHaveURL(new RegExp(`#/contacts/${seed.contactId}$`));
  await expect(page.getByRole('heading', { name: 'Zzzara Searchonly' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('filters Pipeline and the Dashboard queue by deal owner', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const ownedDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await ownedDialog.getByLabel('Deal Name').fill('Owned clinic kit');
  await ownedDialog.getByLabel('Owner').fill('Samira');
  await ownedDialog.getByRole('button', { name: 'Save' }).click();
  await expect(ownedDialog).toBeHidden();

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const otherDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await otherDialog.getByLabel('Deal Name').fill('Unowned clinic kit');
  await otherDialog.getByRole('button', { name: 'Save' }).click();
  await expect(otherDialog).toBeHidden();

  await expect(page.getByText('Owned clinic kit', { exact: true })).toBeVisible();
  await expect(page.getByText('Unowned clinic kit', { exact: true })).toBeVisible();

  await page.getByLabel('Filter by owner').fill('samira');
  await expect(page.getByText('Owned clinic kit', { exact: true })).toBeVisible();
  await expect(page.getByText('Unowned clinic kit', { exact: true })).toHaveCount(0);

  await loadHashRoute(page, '/');
  const queue = page.getByTestId('dashboard-attention-strip');
  await expect(queue.getByRole('heading', { name: 'Needs Attention' })).toBeVisible();
  await queue.getByLabel('Filter by owner').fill('samira');
  await expect(queue.getByRole('button', { name: /Owned clinic kit/ })).toBeVisible();
  await expect(queue.getByRole('button', { name: /Unowned clinic kit/ })).toHaveCount(0);

  await assertNoConsoleErrors();
});

test('creates a deal through the visible UI and shows it in Pipeline', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Deal' });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel('Deal Name').fill('Village Solar Rollout');
  await dialog.getByLabel('Value').fill('12500');
  await dialog.getByLabel('Description').fill('Starter pipeline opportunity.');
  await dialog.getByRole('button', { name: 'Save' }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByText('Village Solar Rollout')).toBeVisible();
  await expect(page.getByText('$12,500').first()).toBeVisible();

  const overview = page.getByTestId('pipeline-forecast-overview');
  const forecastSummary = overview.getByLabel('Pipeline forecast summary');
  await expect(overview.getByRole('heading', { name: 'Forecast and Stage Health' })).toBeVisible();
  await expect(overview.getByText('Open Pipeline')).toBeVisible();
  await expect(forecastSummary.getByText('$12,500')).toBeVisible();
  await expect(overview.getByRole('heading', { name: 'Stage Health', exact: true })).toBeVisible();

  await assertNoConsoleErrors();
});

test('drags a pipeline deal into another stage', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Deal' });
  await dialog.getByLabel('Deal Name').fill('Dragged clinic kit');
  await dialog.getByLabel('Value').fill('4100');
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();

  const card = page.getByRole('button', { name: /Dragged clinic kit/ });
  const qualified = page.getByTestId('pipeline-column-qualified');
  await expect(card).toBeVisible();

  await card.evaluate((from) => {
    const transfer = new DataTransfer();
    (window as unknown as { __pipelineDrag?: DataTransfer }).__pipelineDrag = transfer;
    from.dispatchEvent(new DragEvent('dragstart', { bubbles: true, cancelable: true, dataTransfer: transfer }));
  });
  await qualified.evaluate((to) => {
    const transfer = (window as unknown as { __pipelineDrag?: DataTransfer }).__pipelineDrag;
    to.dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: transfer }));
  });
  await expect(page.getByTestId('pipeline-drop-hint-qualified')).toBeVisible();
  await qualified.evaluate((to) => {
    const transfer = (window as unknown as { __pipelineDrag?: DataTransfer }).__pipelineDrag;
    to.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer }));
  });
  await card.evaluate((from) => {
    const transfer = (window as unknown as { __pipelineDrag?: DataTransfer }).__pipelineDrag;
    from.dispatchEvent(new DragEvent('dragend', { bubbles: true, cancelable: true, dataTransfer: transfer }));
    delete (window as unknown as { __pipelineDrag?: DataTransfer }).__pipelineDrag;
  });

  await expect(qualified.getByRole('button', { name: /Dragged clinic kit/ })).toBeVisible();
  await expect(page.getByLabel('Pipeline stage moves')).toHaveText('Moved Dragged clinic kit to Qualified');

  await assertNoConsoleErrors();
});

test('creates a contact with an owner and filters the Contacts list by that name', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();

  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const unownedDialog = page.getByRole('dialog', { name: 'Add Contact' });
  await unownedDialog.getByLabel('First Name').fill('Kofi');
  await unownedDialog.getByLabel('Last Name').fill('Mensah');
  await unownedDialog.getByRole('button', { name: 'Save' }).click();
  await expect(unownedDialog).toBeHidden();

  await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
  const ownedDialog = page.getByRole('dialog', { name: 'Add Contact' });
  await ownedDialog.getByLabel('First Name').fill('Amina');
  await ownedDialog.getByLabel('Last Name').fill('Diallo');
  await ownedDialog.getByLabel('Owner').fill('Samira');
  await ownedDialog.getByRole('button', { name: 'Save' }).click();
  await expect(ownedDialog).toBeHidden();

  await expect(page.getByText('Kofi Mensah')).toBeVisible();
  await expect(page.getByText('Amina Diallo')).toBeVisible();
  await expect(page.getByText('Samira').first()).toBeVisible();

  await page.getByLabel('Filter by owner').fill('samira');
  await expect(page.getByText('Amina Diallo')).toBeVisible();
  await expect(page.getByText('Kofi Mensah')).toHaveCount(0);

  await page.getByText('Amina Diallo').click();
  await expect(page).toHaveURL(/#\/contacts\//);
  await expect(page.getByRole('heading', { name: 'Amina Diallo' })).toBeVisible();
  await expect(page.locator('#contact-owner')).toHaveValue('Samira');

  await assertNoConsoleErrors();
});

test('opens a pipeline deal guidance drawer and refreshes follow-up state', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  const expectedClose = new Date(Date.now() + 21 * 24 * 60 * 60 * 1000)
    .toISOString()
    .slice(0, 10);
  const followUpDate = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)
    .toISOString()
    .slice(0, 10);

  await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
  const dealDialog = page.getByRole('dialog', { name: 'Add Deal' });
  await expect(dealDialog).toBeVisible();
  await dealDialog.getByLabel('Deal Name').fill('Guided pipeline rollout');
  await dealDialog.getByLabel('Value').fill('50000');
  await dealDialog.getByLabel('Probability').fill('50');
  await dealDialog.locator('#modal-deal-close-date').fill(expectedClose);
  await dealDialog.getByLabel('Description').fill('Needs the next sales action.');
  await dealDialog.getByRole('button', { name: 'Save' }).click();
  await expect(dealDialog).toBeHidden();

  const overview = page.getByTestId('pipeline-forecast-overview');
  const forecastSummary = overview.getByLabel('Pipeline forecast summary');
  await expect(overview.getByText('Weighted Forecast')).toBeVisible();
  await expect(forecastSummary.getByText('$25,000').first()).toBeVisible();
  await expect(overview.getByText('Focus Stage')).toBeVisible();

  const needsFollowUpCard = page.getByRole('button', { name: /Guided pipeline rollout/ });
  await expect(needsFollowUpCard).toContainText('Needs Follow-Up');
  await needsFollowUpCard.evaluate((element) => {
    if (element instanceof HTMLButtonElement) {
      element.click();
    }
  });

  let drawer = page.getByRole('dialog', { name: 'Guided pipeline rollout' });
  await expect(drawer).toBeVisible();
  await expect(drawer.getByText('Needs Follow-Up')).toBeVisible();
  await expect(drawer.getByRole('heading', { name: 'Schedule a follow-up' })).toBeVisible();
  await expect(drawer.getByText('$50,000')).toBeVisible();
  await expect(drawer.getByText('$25,000')).toBeVisible();

  await drawer.getByTestId('next-step').getByRole('button', { name: 'Add Follow-Up' }).click();
  const dialog = page.getByRole('dialog', { name: 'Add Activity' });
  await expect(dialog).toBeVisible();
  await expect(dialog.locator('#modal-activity-deal')).not.toHaveValue('');
  await dialog.getByLabel('Subject').fill('Schedule guided rollout call');
  await dialog.locator('#modal-activity-due-date').fill(followUpDate);
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();

  drawer = page.getByRole('dialog', { name: 'Guided pipeline rollout' });
  await expect(drawer.getByText('Stale')).toBeVisible();
  await expect(drawer.getByRole('heading', { name: 'This deal has gone quiet' })).toBeVisible();
  await expect(drawer.getByText('Schedule guided rollout call', { exact: true })).toBeVisible();

  await drawer.getByRole('button', { name: 'Close' }).click();
  await expect(drawer).toBeHidden();

  await assertNoConsoleErrors();
});

test('pipeline drawer next step opens the deal page to set a close date', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/pipeline');
  const followUpDate = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)
    .toISOString()
    .slice(0, 10);

  const seed = await page.evaluate(async (dueDate) => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    const deal = await invoke('create_deal', {
      title: 'Close-date clinic kit',
      value: 9000,
      currency: 'USD',
      stage: 'Proposal',
      probability: 40,
      expected_close: '',
      contact_id: '',
      organization_id: '',
      notes: '',
    }) as { id: string };

    await invoke('create_activity', {
      activity_type: 'task',
      title: 'Confirm clinic install window',
      description: '',
      due_date: dueDate,
      contact_id: '',
      deal_id: deal.id,
    });

    return { dealId: deal.id };
  }, followUpDate);

  await loadHashRoute(page, '/pipeline');
  const card = page.getByRole('button', { name: /Close-date clinic kit/ });
  await expect(card).toBeVisible();
  await card.evaluate((element) => {
    if (element instanceof HTMLButtonElement) {
      element.click();
    }
  });

  const drawer = page.getByRole('dialog', { name: 'Close-date clinic kit' });
  await expect(drawer.getByRole('heading', { name: 'Set an expected close date' })).toBeVisible();
  await drawer.getByTestId('next-step').getByRole('button', { name: 'Set close date' }).click();
  await expect(page).toHaveURL(new RegExp(`#/deals/${seed.dealId}$`));
  await expect(page.getByRole('heading', { name: 'Set an expected close date' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('lists a stale deal on Reports and opens the deal page', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/reports');
  const laterDate = new Date(Date.now() + 21 * 24 * 60 * 60 * 1000)
    .toISOString()
    .slice(0, 10);

  await page.evaluate(async (dueDate) => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) {
      throw new Error('Tauri smoke shim is not installed.');
    }

    const deal = await invoke('create_deal', {
      title: 'Quiet Clinic Rollout',
      value: 18000,
      currency: 'USD',
      stage: 'Proposal',
      probability: 40,
      expected_close: dueDate,
      contact_id: '',
      organization_id: '',
      notes: '',
    }) as { id: string };

    await invoke('create_activity', {
      activity_type: 'task',
      title: 'Later site visit',
      description: '',
      due_date: dueDate,
      contact_id: '',
      deal_id: deal.id,
    });
  }, laterDate);

  await loadHashRoute(page, '/reports');
  const report = page.getByTestId('stale-deal-report');
  await expect(report.getByRole('heading', { name: 'Stale Deals' })).toBeVisible();
  await expect(report.getByRole('button', { name: 'Quiet Clinic Rollout' })).toBeVisible();
  await expect(report.getByText('Later site visit')).toBeVisible();

  await report.getByRole('button', { name: 'Quiet Clinic Rollout' }).click();
  await expect(page).toHaveURL(/#\/deals\//);
  await expect(page.getByRole('heading', { name: 'Quiet Clinic Rollout' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('saves the current report focus as a named view and applies it later', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/reports');
  await expect(page.getByRole('heading', { name: 'Reports' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Pipeline Overview' })).toBeVisible();
  await expect(page.getByTestId('stale-deal-report')).toBeVisible();

  await page.getByRole('group', { name: 'Focus' }).getByRole('button', { name: 'Stale Deals' }).click();
  await expect(page.getByRole('group', { name: 'Focus' }).getByRole('button', { name: 'Stale Deals' })).toHaveClass(/active/);
  await expect(page.getByTestId('stale-deal-report')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Pipeline Overview' })).toHaveCount(0);
  await expect(page.getByRole('heading', { name: 'Activity Overview' })).toHaveCount(0);

  await page.getByLabel('View name').fill('Stale deals');
  await page.getByRole('button', { name: 'Save view' }).click();
  await expect(page.getByLabel('Saved view', { exact: true })).toHaveValue(/view-/);

  await page.getByRole('group', { name: 'Focus' }).getByRole('button', { name: 'All', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Pipeline Overview' })).toBeVisible();

  await page.getByLabel('Saved view', { exact: true }).selectOption({ label: 'Stale deals' });
  await expect(page.getByRole('group', { name: 'Focus' }).getByRole('button', { name: 'Stale Deals' })).toHaveClass(/active/);
  await expect(page.getByTestId('stale-deal-report')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Pipeline Overview' })).toHaveCount(0);

  await assertNoConsoleErrors();
});

test('downloads the current reports snapshot', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/reports');
  const exportButton = page.getByRole('button', { name: 'Download snapshot' });
  await expect(exportButton).toBeEnabled();

  const downloadPromise = page.waitForEvent('download');
  await exportButton.click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/^900crm-reports-\d{4}-\d{2}-\d{2}\.csv$/);

  const downloadPath = await download.path();
  expect(downloadPath).toBeTruthy();
  const text = await import('node:fs/promises').then((fs) => fs.readFile(downloadPath!, 'utf8'));
  expect(text).toContain('Current dataset snapshot');
  expect(text).toContain('win_rate');
  await expect(page.getByText('Downloaded the current report snapshot. The file is unencrypted.')).toBeVisible();

  await assertNoConsoleErrors();
});

test('creates an activity through the visible UI and shows it in Activities', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/activities');
  await expect(page.getByRole('heading', { name: 'Activities' })).toBeVisible();

  const today = localDateInputValue();

  await page.locator('.page-header').getByRole('button', { name: /^Add Activity$/ }).click();
  const quickAddForm = page.getByRole('form', { name: 'Add Activity' });
  await quickAddForm.getByLabel('Subject').fill('Follow up on grant paperwork');
  await quickAddForm.getByLabel('Due Date').fill(today);
  await quickAddForm.getByRole('button', { name: 'Add', exact: true }).click();

  const workbench = page.getByTestId('activity-workbench');
  await expect(workbench.getByRole('heading', { name: 'Due Workbench' })).toBeVisible();
  await expect(workbench.getByText('Open work')).toBeVisible();

  const todaySection = page.locator('.activity-bucket').filter({
    has: page.getByRole('heading', { name: 'Today', exact: true }),
  });
  const activityRow = todaySection.locator('.activity-row').filter({
    hasText: 'Follow up on grant paperwork',
  });
  await expect(activityRow).toBeVisible();
  await expect(activityRow.getByText('Upcoming')).toBeVisible();

  await activityRow.getByRole('button', { name: 'Snooze' }).click();
  const thisWeekSection = page.locator('.activity-bucket').filter({
    has: page.getByRole('heading', { name: 'This Week', exact: true }),
  });
  const snoozedRow = thisWeekSection.locator('.activity-row').filter({
    hasText: 'Follow up on grant paperwork',
  });
  await expect(snoozedRow).toBeVisible();

  await snoozedRow.getByRole('button', { name: 'Mark Complete' }).click();
  const completedSection = page.locator('.activity-bucket').filter({
    has: page.getByRole('heading', { name: 'Completed', exact: true }),
  });
  await expect(completedSection.locator('.activity-row').filter({
    hasText: 'Follow up on grant paperwork',
  })).toBeVisible();

  await assertNoConsoleErrors();
});

test('saves the current activity filters as a named view and applies it later', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await loadHashRoute(page, '/activities');
  await expect(page.getByRole('heading', { name: 'Activities' })).toBeVisible();

  const today = localDateInputValue();

  await page.locator('.page-header').getByRole('button', { name: /^Add Activity$/ }).click();
  const firstForm = page.getByRole('form', { name: 'Add Activity' });
  await firstForm.getByLabel('Type').selectOption('task');
  await firstForm.getByLabel('Subject').fill('Harbor grant paperwork task');
  await firstForm.getByLabel('Due Date').fill(today);
  await firstForm.getByRole('button', { name: 'Add', exact: true }).click();

  await page.locator('.page-header').getByRole('button', { name: /^Add Activity$/ }).click();
  const secondForm = page.getByRole('form', { name: 'Add Activity' });
  await secondForm.getByLabel('Type').selectOption('call');
  await secondForm.getByLabel('Subject').fill('Clinic site call with Amara');
  await secondForm.getByLabel('Due Date').fill(today);
  await secondForm.getByRole('button', { name: 'Add', exact: true }).click();

  await expect(page.getByText('Harbor grant paperwork task')).toBeVisible();
  await expect(page.getByText('Clinic site call with Amara')).toBeVisible();

  await page.getByRole('group', { name: 'Type' }).getByRole('button', { name: 'Call' }).click();
  await expect(page.getByText('Clinic site call with Amara')).toBeVisible();
  await expect(page.getByText('Harbor grant paperwork task')).toHaveCount(0);

  await page.getByLabel('View name').fill('Clinic calls');
  await page.getByRole('button', { name: 'Save view' }).click();
  await expect(page.getByLabel('Saved view', { exact: true })).toHaveValue(/view-/);

  await page.getByRole('group', { name: 'Type' }).getByRole('button', { name: 'All', exact: true }).click();
  await expect(page.getByText('Harbor grant paperwork task')).toBeVisible();

  await page.getByLabel('Saved view', { exact: true }).selectOption({ label: 'Clinic calls' });
  await expect(page.getByRole('group', { name: 'Type' }).getByRole('button', { name: 'Call' })).toHaveClass(/active/);
  await expect(page.getByText('Clinic site call with Amara')).toBeVisible();
  await expect(page.getByText('Harbor grant paperwork task')).toHaveCount(0);

  await assertNoConsoleErrors();
});
