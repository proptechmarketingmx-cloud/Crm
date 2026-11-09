import { expect, loadHashRoute, test } from './tauri-shim';

test('renders the browser app shell and dashboard route', async ({ page, assertNoConsoleErrors }) => {
  await page.goto('/');

  await expect(page.getByText('900CRM')).toBeVisible();
  const navigation = page.getByRole('navigation', { name: 'Main navigation' });
  await expect(navigation).toBeVisible();
  await expect(navigation.getByText('Workspace')).toBeVisible();
  await expect(navigation.getByText('Review')).toBeVisible();
  await expect(navigation.getByRole('link', { name: 'Pending Actions' })).toBeVisible();
  await expect(page.getByTestId('nav-pending-count')).toHaveCount(0);
  await expect(navigation.getByText('Admin')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  await expect(page.getByText('Total Contacts')).toBeVisible();
  await expect(navigation.getByRole('link', { name: 'Reports' })).toBeVisible();
  await expect(page.getByTestId('first-run-data')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Import data' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Make a backup' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('renders key hash routes without native Tauri dialogs', async ({ page, assertNoConsoleErrors }) => {
  await loadHashRoute(page, '/leads');
  await expect(page.getByRole('heading', { name: 'Leads', exact: true })).toBeVisible();
  await expect(page.getByText('No leads yet')).toBeVisible();
  await expect(page.getByText('Convert them on the person page')).toBeVisible();

  await loadHashRoute(page, '/contacts');
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();
  await expect(page.getByText('No contacts yet')).toBeVisible();

  await loadHashRoute(page, '/pipeline');
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();
  await expect(page.getByText('Lead').first()).toBeVisible();
  await expect(page.getByTestId('pipeline-first-run')).toBeVisible();
  await expect(page.getByText('Add a deal to start the board')).toBeVisible();

  await loadHashRoute(page, '/activities');
  await expect(page.getByRole('heading', { name: 'Activities' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'List' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'Week' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'Month' })).toBeVisible();
  await expect(page.getByText('No activities yet')).toBeVisible();
  await expect(page.getByText('Add a task, call, meeting, email, visit, WhatsApp ping, or SMS')).toBeVisible();

  await page.getByRole('tab', { name: 'Week' }).click();
  await expect(page.getByTestId('activity-week')).toBeVisible();
  await expect(page.getByRole('button', { name: 'This week', exact: true })).toBeVisible();

  await page.getByRole('tab', { name: 'Month' }).click();
  await expect(page.getByTestId('activity-month')).toBeVisible();
  await expect(page.getByRole('button', { name: 'This month', exact: true })).toBeVisible();

  await loadHashRoute(page, '/reports');
  await expect(page.getByRole('heading', { name: 'Reports' })).toBeVisible();
  await expect(page.getByTestId('reports-first-run')).toBeVisible();
  await expect(page.getByText('No report data yet')).toBeVisible();
  await expect(page.getByText('Reports fill in after you add deals')).toBeVisible();

  await loadHashRoute(page, '/settings');
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'Appearance' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'Data' })).toBeVisible();
  await expect(page.getByRole('tab', { name: 'Integrations' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Language' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'About' })).toBeVisible();
  await expect(page.getByText('Backup & Restore')).toHaveCount(0);
  await expect(page.getByText('Not available yet')).toHaveCount(0);

  await page.getByRole('tab', { name: 'Data' }).click();
  await expect(page.getByText('Backup & Restore')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Restore Backup' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Language' })).toHaveCount(0);

  await page.getByRole('tab', { name: 'Integrations' }).click();
  await expect(page.getByText('Not available yet')).toBeVisible();
  await expect(page.getByText('Multi-device sync is not implemented')).toBeVisible();
  await expect(page.getByText('Backup & Restore')).toHaveCount(0);

  await loadHashRoute(page, '/settings/data');
  await expect(page.getByTestId('settings-data-guidance')).toBeVisible();
  await expect(page.getByText('Backup & Restore')).toBeVisible();
  await expect(page.getByText('Start with import or a local backup')).toBeVisible();

  await assertNoConsoleErrors();
});

test('renders review routes without console errors', async ({ page, assertNoConsoleErrors }) => {
  await loadHashRoute(page, '/audit-log');
  await expect(page.getByRole('heading', { name: 'Audit Log' })).toBeVisible();

  await loadHashRoute(page, '/pending-actions');
  await expect(page.getByRole('heading', { name: 'Pending Actions' })).toBeVisible();

  await assertNoConsoleErrors();
});

test('opens Settings Data from the dashboard first-run import and backup prompts', async ({
  page,
  assertNoConsoleErrors,
}) => {
  await page.goto('/');
  await expect(page.getByTestId('first-run-data')).toBeVisible();

  await page.getByRole('button', { name: 'Make a backup' }).click();
  await expect(page.getByRole('tab', { name: 'Data' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByTestId('settings-data-guidance')).toBeVisible();
  await expect(page.getByText('Backup & Restore')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Choose Folder' })).toBeVisible();

  await loadHashRoute(page, '/');
  await page.getByRole('button', { name: 'Import data' }).click();
  await expect(page.getByRole('tab', { name: 'Data' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText('Import / Export')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Choose File', exact: true })).toBeVisible();

  await assertNoConsoleErrors();
});

test('switches primary workspace routes from the sidebar', async ({ page, assertNoConsoleErrors }) => {
  await page.goto('/');

  const navigation = page.getByRole('navigation', { name: 'Main navigation' });
  await navigation.getByRole('link', { name: 'Leads' }).click();
  await expect(page.getByRole('heading', { name: 'Leads', exact: true })).toBeVisible();

  await navigation.getByRole('link', { name: 'Contacts' }).click();
  await expect(page.getByRole('heading', { name: 'Contacts' })).toBeVisible();

  await navigation.getByRole('link', { name: 'Pipeline' }).click();
  await expect(page.getByRole('heading', { name: 'Pipeline' })).toBeVisible();

  await navigation.getByRole('link', { name: 'Reports' }).click();
  await expect(page.getByRole('heading', { name: 'Reports' })).toBeVisible();

  await navigation.getByRole('link', { name: 'Settings' }).click();
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();

  await assertNoConsoleErrors();
});
