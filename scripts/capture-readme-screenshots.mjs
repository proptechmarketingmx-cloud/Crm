#!/usr/bin/env node
/* global fetch, window, HashChangeEvent, document */

import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';

const BASE_URL = 'http://127.0.0.1:1420';
const ASSET_DIR = 'docs/assets/readme';
const VIEWPORT = { width: 1600, height: 1000 };

const shots = [
  {
    route: '/settings',
    file: '900crm-data-management.png',
    waitFor: 'Settings',
    waitForText: 'Backup & Restore',
    scrollTo: 'Backup & Restore',
  },
  {
    route: '/contacts',
    file: '900crm-contacts.png',
    waitFor: 'Contacts',
    waitForText: 'Leila Abebe',
  },
  {
    route: '/pipeline',
    file: '900crm-pipeline.png',
    waitFor: 'Pipeline',
    waitForText: 'Market inventory rollout',
  },
  {
    route: '/activities',
    file: '900crm-activities.png',
    waitFor: 'Activities',
    waitForText: 'Prepare renewal proposal',
  },
  {
    route: '/dashboard',
    file: '900crm-dashboard.png',
    waitFor: 'Dashboard',
    waitForText: 'Prepare renewal proposal',
  },
];

const viteBin = join(process.cwd(), 'node_modules', '.bin', 'vite');
const server = spawn(viteBin, ['dev', '--host', '127.0.0.1'], {
  cwd: 'apps/desktop',
  stdio: ['ignore', 'pipe', 'pipe'],
});

server.stdout.on('data', (chunk) => process.stdout.write(chunk));
server.stderr.on('data', (chunk) => process.stderr.write(chunk));

try {
  await mkdir(ASSET_DIR, { recursive: true });
  await waitForServer(BASE_URL);
  await captureScreenshots();
} finally {
  server.kill('SIGTERM');
}

async function waitForServer(url) {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Keep polling until Vite is ready.
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(`Timed out waiting for ${url}`);
}

async function captureScreenshots() {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: VIEWPORT, deviceScaleFactor: 1 });
  page.on('console', (message) => {
    if (message.type() === 'error' || process.env.DEBUG_README_SCREENSHOTS === '1') {
      console.error(`[browser:${message.type()}] ${message.text()}`);
    }
  });
  page.on('pageerror', (error) => {
    console.error(`[browser:pageerror] ${error.message}`);
  });
  await installScreenshotShim(page, process.env.DEBUG_README_SCREENSHOTS === '1');

  for (const shot of shots) {
    await navigateHashRoute(page, shot.route);
    try {
      await page.getByRole('heading', { name: shot.waitFor }).first().waitFor();
    } catch (error) {
      const routeState = await page.evaluate(() => ({
        href: window.location.href,
        hash: window.location.hash,
      }));
      const mainText = await page.locator('.app-main').innerText().catch(() => '<main unavailable>');
      console.error(`Timed out waiting for heading "${shot.waitFor}" on ${shot.route}`);
      console.error(JSON.stringify(routeState));
      console.error(mainText);
      throw error;
    }
    await prepareShot(page, shot.file);
    if (shot.file === '900crm-dashboard.png') {
      await waitForDashboardReady(page);
    }
    await page.locator('.toast').waitFor({ state: 'hidden', timeout: 5_000 }).catch(() => undefined);
    try {
      await page.getByText(shot.waitForText).first().waitFor();
    } catch (error) {
      const mainText = await page.locator('.app-main').innerText().catch(() => '<main unavailable>');
      const probe = await page.evaluate(async () => {
        const internals = window.__TAURI_INTERNALS__;
        if (!internals?.invoke) return 'missing invoke';
        const result = await internals.invoke('list_contacts', { params: {} });
        return JSON.stringify(result);
      }).catch((err) => `probe failed: ${err instanceof Error ? err.message : String(err)}`);
      console.error(`Timed out waiting for "${shot.waitForText}" on ${shot.route}`);
      console.error(mainText);
      console.error(probe);
      throw error;
    }

    if (shot.scrollTo) {
      await page.getByText(shot.scrollTo).first().scrollIntoViewIfNeeded();
    } else {
      await page.evaluate(() => window.scrollTo(0, 0));
    }

    await page.locator('.page-content').first().screenshot({
      path: join(ASSET_DIR, shot.file),
      animations: 'disabled',
    });
  }

  await browser.close();
}

async function navigateHashRoute(page, route) {
  if (route === '/') {
    await page.goto(`${BASE_URL}/#/contacts`, { waitUntil: 'networkidle' });
    await page.reload({ waitUntil: 'networkidle' });
    await page.getByRole('heading', { name: 'Contacts' }).first().waitFor();
    await page.evaluate(() => {
      window.location.hash = '/';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    return;
  }

  if (route === '/dashboard') {
    await page.goto(BASE_URL, { waitUntil: 'networkidle' });
    await page.reload({ waitUntil: 'networkidle' });
    return;
  }

  await page.goto(`${BASE_URL}/#${route}`, { waitUntil: 'networkidle' });
  await page.reload({ waitUntil: 'networkidle' });
}

async function prepareShot(page, file) {
  if (file === '900crm-contacts.png') {
    await page.locator('.page-header').getByRole('button', { name: 'Add Contact' }).click();
    const dialog = page.getByRole('dialog', { name: 'Add Contact' });
    await dialog.getByLabel('First Name').fill('Leila');
    await dialog.getByLabel('Last Name').fill('Abebe');
    await dialog.getByLabel('Email').fill('leila.abebe@example.org');
    await dialog.getByLabel('Phone').fill('+251 911 010 606');
    await dialog.getByLabel('Organization').fill('Addis Textile Collective');
    await dialog.getByRole('button', { name: 'Save' }).click();
    await dialog.waitFor({ state: 'hidden' });
  }

  if (file === '900crm-pipeline.png') {
    await page.locator('.page-header').getByRole('button', { name: 'Add Deal' }).click();
    const dialog = page.getByRole('dialog', { name: 'Add Deal' });
    await dialog.getByLabel('Deal Name').fill('Market inventory rollout');
    await dialog.getByLabel('Value').fill('18400');
    await dialog.getByLabel('Description').fill('Starter pipeline opportunity for a local market cooperative.');
    await dialog.getByRole('button', { name: 'Save' }).click();
    await dialog.waitFor({ state: 'hidden' });
  }

  if (file === '900crm-activities.png') {
    await page.locator('.page-header').getByRole('button', { name: /^Add Activity$/ }).click();
    const quickAddForm = page.getByRole('form', { name: 'Add Activity' });
    await quickAddForm.getByLabel('Subject').fill('Prepare renewal proposal');
    await quickAddForm.getByRole('button', { name: 'Add', exact: true }).click();
  }
}

async function waitForDashboardReady(page) {
  await stabilizeDashboardFromShim(page);
  await page.waitForFunction(() => document.querySelectorAll('.dashboard-page .skeleton').length === 0);
  await page.waitForFunction(() => {
    const text = document.querySelector('.dashboard-page')?.textContent ?? '';
    return !text.includes('...') && !text.includes('…') && !text.includes('No activities yet');
  });
  await page.locator('.dashboard-activity .activity-item').first().waitFor();
  await page.getByText('Prepare renewal proposal').first().waitFor();
  await page.getByText('$69,000').first().waitFor();
  await page.getByText('100.0%').first().waitFor();
}

async function stabilizeDashboardFromShim(page) {
  await page.evaluate(async () => {
    const internals = window.__TAURI_INTERNALS__;
    if (!internals?.invoke) {
      throw new Error('Missing Tauri screenshot shim');
    }

    const [stats, pipelineReport, activityReport, upcomingActivities] = await Promise.all([
      internals.invoke('get_dashboard_stats'),
      internals.invoke('get_pipeline_conversion_report'),
      internals.invoke('get_activity_funnel_report'),
      internals.invoke('list_upcoming_activities', { limit: 10 }),
    ]);

    const formatNumber = (value) => new Intl.NumberFormat('en-US').format(value);
    const formatCurrency = (value) => new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      maximumFractionDigits: 0,
    }).format(value);
    const formatPercent = (ratio) => `${(ratio * 100).toFixed(1)}%`;

    const statValues = new Map([
      ['Total Contacts', formatNumber((stats.total_contacts ?? 0) + (stats.total_organizations ?? 0))],
      ['Active Deals', formatNumber(stats.active_deals ?? 0)],
      ['Pipeline Value', formatCurrency(stats.pipeline_value ?? 0)],
      ['Upcoming Tasks', formatNumber(stats.upcoming_activities ?? 0)],
    ]);

    for (const card of document.querySelectorAll('.stat-card')) {
      const label = card.querySelector('.stat-label')?.textContent?.trim();
      const value = label ? statValues.get(label) : undefined;
      if (!value) continue;
      card.querySelectorAll('.skeleton').forEach((node) => node.remove());
      let valueNode = card.querySelector('.stat-value');
      if (!valueNode) {
        valueNode = document.createElement('p');
        valueNode.className = 'stat-value';
        card.append(valueNode);
      }
      valueNode.textContent = value;
    }

    const summaryValues = new Map([
      ['Closed Won', formatNumber(pipelineReport.closed_won ?? 0)],
      ['Open Deals', formatNumber(pipelineReport.open_deals ?? 0)],
      ['Pending', formatNumber(activityReport.pending_activities ?? 0)],
      ['Overdue Rate', formatPercent(activityReport.overdue_rate ?? 0)],
    ]);

    for (const summary of document.querySelectorAll('.summary-stat')) {
      const label = summary.querySelector('.summary-stat-label')?.textContent?.trim();
      const value = label ? summaryValues.get(label) : undefined;
      if (value) {
        summary.querySelector('.summary-stat-value').textContent = value;
      }
    }

    const pipelineCard = [...document.querySelectorAll('.report-card')]
      .find((card) => card.querySelector('.section-title')?.textContent?.trim() === 'Pipeline Conversion');
    if (pipelineCard) {
      pipelineCard.querySelector('.report-kpi-value').textContent = formatPercent(pipelineReport.overall_win_rate ?? 0);
      renderMetricList(
        pipelineCard,
        (pipelineReport.stage_metrics ?? [])
          .filter((metric) => metric.count > 0)
          .map((metric) => ({
            label: metric.stage,
            value: formatNumber(metric.count),
            ratio: metric.stage_share ?? 0,
          })),
      );
    }

    const activityCard = [...document.querySelectorAll('.report-card')]
      .find((card) => card.querySelector('.section-title')?.textContent?.trim() === 'Activity Funnel');
    if (activityCard) {
      activityCard.querySelector('.report-kpi-value').textContent = formatPercent(activityReport.completion_rate ?? 0);
      renderMetricList(
        activityCard,
        (activityReport.by_type ?? [])
          .filter((metric) => metric.total > 0)
          .map((metric) => ({
            label: titleCase(metric.activity_type),
            value: formatPercent(metric.completion_rate ?? 0),
            ratio: metric.completion_rate ?? 0,
          })),
      );
    }

    const activityFeed = document.querySelector('.dashboard-activity .activity-feed');
    if (activityFeed) {
      const firstActivities = upcomingActivities.slice(0, 3);
      activityFeed.innerHTML = [
        '<ul class="activity-list" role="list">',
        ...firstActivities.map((activity) => `
          <li class="activity-item" style="display: flex; align-items: flex-start; gap: 1rem; padding: 0.75rem 0; border-bottom: 1px solid var(--border-subtle);">
            <div class="activity-icon-wrap" style="color: var(--color-primary-500); width: 28px; height: 28px; border-radius: 50%; background: var(--surface-hover); display: flex; align-items: center; justify-content: center; flex-shrink: 0;" aria-hidden="true">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 11l3 3L22 4M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
              </svg>
            </div>
            <div class="activity-content" style="flex: 1; min-width: 0;">
              <p class="activity-subject" style="margin: 0; font-size: var(--text-sm); font-weight: var(--weight-medium); color: var(--text-primary);">${escapeHtml(activity.title)}</p>
              <div class="activity-meta" style="display: flex; gap: 0.5rem; flex-wrap: wrap; font-size: var(--text-xs); color: var(--text-tertiary);">
                <span class="activity-type-label">${escapeHtml(titleCase(activity.activity_type))}</span>
                <span class="activity-dot" aria-hidden="true">.</span>
                <span class="activity-time">Upcoming</span>
              </div>
            </div>
            <span class="badge badge-neutral activity-status">Pending</span>
          </li>
        `),
        '</ul>',
      ].join('');
    }

    function renderMetricList(card, metrics) {
      card.querySelector('.report-empty')?.remove();
      card.querySelector('.metric-list')?.remove();
      const list = document.createElement('ul');
      list.className = 'metric-list';
      list.setAttribute('role', 'list');
      for (const metric of metrics.slice(0, 6)) {
        const row = document.createElement('li');
        row.className = 'metric-row';
        row.innerHTML = `
          <div class="metric-row-header" style="display: flex; align-items: center; justify-content: space-between; gap: 1rem; font-size: var(--text-xs);">
            <span class="metric-label">${escapeHtml(metric.label)}</span>
            <span class="metric-value" style="font-weight: var(--weight-medium); color: var(--text-primary);">${escapeHtml(metric.value)}</span>
          </div>
          <div class="metric-bar-track" style="width: 100%; height: 6px; border-radius: 999px; background: var(--surface-hover); overflow: hidden;">
            <span class="metric-bar-fill" style="display: block; height: 100%; border-radius: inherit; background: var(--color-primary-500); width: ${Math.max(0, Math.min(100, metric.ratio * 100))}%"></span>
          </div>
        `;
        row.style.display = 'flex';
        row.style.flexDirection = 'column';
        row.style.gap = '0.5rem';
        list.append(row);
      }
      list.style.display = 'flex';
      list.style.flexDirection = 'column';
      list.style.gap = '0.75rem';
      card.append(list);
    }

    function titleCase(value) {
      return String(value)
        .replace(/[-_]/g, ' ')
        .replace(/\b\w/g, (letter) => letter.toUpperCase());
    }

    function escapeHtml(value) {
      return String(value)
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;');
    }
  });
}

async function installScreenshotShim(page, debugEnabled) {
  await page.addInitScript((debug) => {
    const timestamp = '2026-06-24T12:00:00.000Z';
    if (debug) {
      console.debug('[readme-shim] installed');
    }

    const contacts = [
      contact('contact-001', 'Amina', 'Okafor', 'amina.okafor@example.org', '+234 800 010 1001', 'Kijani Market Cooperative', 'Lagos', 'Nigeria'),
      contact('contact-002', 'Daniel', 'Mensah', 'daniel.mensah@example.org', '+233 240 010 202', 'Accra School Supplies', 'Accra', 'Ghana'),
      contact('contact-003', 'Priya', 'Nair', 'priya.nair@example.org', '+91 80 5555 0142', 'Mango Health Clinic', 'Bengaluru', 'India'),
      contact('contact-004', 'Samira', 'Hassan', 'samira.hassan@example.org', '+254 700 010 404', 'Nile Field Services', 'Nairobi', 'Kenya'),
      contact('contact-005', 'Mateo', 'Silva', 'mateo.silva@example.org', '+55 11 5555 0105', 'Sol Verde Retail', 'Sao Paulo', 'Brazil'),
    ];

    const organizations = [
      organization('organization-001', 'Kijani Market Cooperative', 'ops@kijani.example.org', 'Lagos', 'Nigeria'),
      organization('organization-002', 'Accra School Supplies', 'hello@accraschools.example.org', 'Accra', 'Ghana'),
      organization('organization-003', 'Mango Health Clinic', 'admin@mangohealth.example.org', 'Bengaluru', 'India'),
    ];

    const deals = [
      deal('deal-001', 'Market inventory rollout', 18400, 'USD', 'Lead', 20, '2026-07-14', 'contact-001', 'organization-001'),
      deal('deal-002', 'School supplies renewal', 9200, 'USD', 'Qualified', 45, '2026-07-20', 'contact-002', 'organization-002'),
      deal('deal-003', 'Clinic operations package', 27600, 'USD', 'Proposal', 60, '2026-08-04', 'contact-003', 'organization-003'),
      deal('deal-004', 'Field service expansion', 13800, 'USD', 'Negotiation', 75, '2026-08-16', 'contact-004', null),
      deal('deal-005', 'Retail analytics pilot', 6400, 'USD', 'Closed Won', 100, '2026-06-18', 'contact-005', null),
    ];

    const activities = [
      activity('activity-001', 'task', 'Prepare renewal proposal', 'Confirm quantities and delivery dates.', '2026-07-01', false, 'contact-002', 'deal-002'),
      activity('activity-002', 'call', 'Call Amina about inventory rollout', 'Review rollout timeline and offline import needs.', '2026-07-02', false, 'contact-001', 'deal-001'),
      activity('activity-003', 'meeting', 'Clinic onboarding review', 'Walk through contact tagging and backup plan.', '2026-07-06', false, 'contact-003', 'deal-003'),
      activity('activity-004', 'email', 'Send pilot summary', 'Share next steps and pricing summary.', '2026-06-20', true, 'contact-005', 'deal-005'),
    ];

    const activityLinks = [
      link('link-001', 'activity-001', 'contact', 'contact-002'),
      link('link-002', 'activity-001', 'deal', 'deal-002'),
      link('link-003', 'activity-002', 'contact', 'contact-001'),
      link('link-004', 'activity-002', 'deal', 'deal-001'),
      link('link-005', 'activity-003', 'organization', 'organization-003'),
      link('link-006', 'activity-003', 'deal', 'deal-003'),
    ];

    let callbackId = 0;

    window.__TAURI_INTERNALS__ = {
      invoke: async (cmd, args) => {
        if (debug) {
          console.debug(`[readme-shim] ${cmd}`);
        }
        switch (cmd) {
          case 'get_settings':
            return {
              language: 'en',
              currency: 'USD',
              theme: 'light',
              date_format: 'MMM D, YYYY',
              sync_enabled: 'false',
              sync_url: '',
              notifications_enabled: 'true',
              reminder_lead_minutes: '30',
              email_integration_enabled: 'false',
              smtp_host: '',
              smtp_port: '587',
              smtp_username: '',
              smtp_password: '',
              smtp_from: '',
              imap_host: '',
              imap_port: '993',
              imap_username: '',
              imap_password: '',
            };
          case 'update_setting':
            return { key: args?.key ?? '', value: args?.value ?? '' };
          case 'list_contacts':
            return listContacts(args);
          case 'create_contact': {
            const created = contact(
              `contact-${contacts.length + 1}`,
              stringArg(args, 'first_name'),
              stringArg(args, 'last_name'),
              stringArg(args, 'email'),
              stringArg(args, 'phone'),
              stringArg(args, 'org_name'),
              '',
              '',
            );
            contacts.push(created);
            return created;
          }
          case 'list_organizations':
            return organizations;
          case 'list_deals':
            return deals;
          case 'create_deal': {
            const created = deal(
              `deal-${deals.length + 1}`,
              stringArg(args, 'title'),
              numberArg(args, 'value'),
              stringArg(args, 'currency', 'USD'),
              stringArg(args, 'stage', 'Lead'),
              numberArg(args, 'probability', 10),
              nullableStringArg(args, 'expected_close'),
              nullableStringArg(args, 'contact_id'),
              nullableStringArg(args, 'organization_id'),
            );
            deals.push(created);
            return created;
          }
          case 'get_pipeline_summary':
            return pipelineSummary();
          case 'list_activities':
            return activities;
          case 'create_activity': {
            const created = activity(
              `activity-${activities.length + 1}`,
              stringArg(args, 'activity_type', 'task'),
              stringArg(args, 'title'),
              stringArg(args, 'description'),
              nullableStringArg(args, 'due_date'),
              false,
              nullableStringArg(args, 'contact_id'),
              nullableStringArg(args, 'deal_id'),
            );
            activities.unshift(created);
            return created;
          }
          case 'list_upcoming_activities':
            return activities.filter((item) => !item.completed).slice(0, 10);
          case 'list_activity_links':
            return activityLinks.filter((item) => item.activity_id === args?.activity_id);
          case 'list_custom_field_defs':
          case 'list_custom_field_values_for_type':
          case 'list_contact_duplicate_candidates':
          case 'list_recent_audit_log':
          case 'list_pending_proposed_actions':
          case 'list_external_clients':
            return [];
          case 'get_dashboard_stats':
            return dashboardStats();
          case 'get_pipeline_conversion_report':
            return pipelineConversionReport();
          case 'get_activity_funnel_report':
            return activityFunnelReport();
          case 'create_local_backup':
            return { backup_dir: 'Selected backup folder', created_at: timestamp };
          case 'validate_local_backup':
            return {
              backup_dir: args?.backup_dir ?? 'Selected backup folder',
              created_at: timestamp,
              app_version: '1.0.0',
              schema_version: 7,
              device_id: 'readme-screenshot-device',
            };
          case 'restore_local_backup_to_app_data':
            return { database_path: 'App data database', restored_at: timestamp };
          case 'plugin:notification|is_permission_granted':
            return true;
          default:
            throw new Error(`Unmocked screenshot invoke: ${cmd}`);
        }
      },
      transformCallback: () => {
        callbackId += 1;
        return callbackId;
      },
      unregisterCallback: () => {},
      convertFileSrc: (filePath) => filePath,
      metadata: {
        currentWindow: { label: 'main' },
        currentWebview: { label: 'main' },
      },
    };

    function contact(id, firstName, lastName, email, phone, orgName, city, country) {
      return {
        id,
        contact_type: 'person',
        first_name: firstName,
        last_name: lastName,
        org_name: orgName,
        email,
        phone,
        address: '',
        city,
        country,
        org_id: null,
        notes: 'Synthetic README screenshot contact.',
        created_at: timestamp,
        updated_at: timestamp,
        deleted_at: null,
      };
    }

    function organization(id, name, email, city, country) {
      return {
        id,
        name,
        email,
        phone: null,
        website: null,
        address_line1: null,
        address_line2: null,
        city,
        region: null,
        country,
        postal_code: null,
        source: 'manual',
        description: 'Synthetic README screenshot organization.',
        created_at: timestamp,
        updated_at: timestamp,
        deleted_at: null,
        device_id: 'readme-screenshot-device',
      };
    }

    function deal(id, title, value, currency, stage, probability, expectedClose, contactId, organizationId) {
      return {
        id,
        title,
        value,
        currency,
        stage,
        probability,
        expected_close: expectedClose,
        contact_id: contactId,
        organization_id: organizationId,
        notes: 'Synthetic README screenshot deal.',
        created_at: timestamp,
        updated_at: timestamp,
      };
    }

    function activity(id, type, title, description, dueDate, completed, contactId, dealId) {
      return {
        id,
        activity_type: type,
        title,
        description,
        due_date: dueDate,
        completed,
        contact_id: contactId,
        deal_id: dealId,
        created_at: timestamp,
        updated_at: timestamp,
      };
    }

    function link(id, activityId, entityType, entityId) {
      return {
        id,
        activity_id: activityId,
        entity_type: entityType,
        entity_id: entityId,
        created_at: timestamp,
        deleted_at: null,
        device_id: 'readme-screenshot-device',
      };
    }

    function pageArgs(args) {
      return args?.params && typeof args.params === 'object' ? args.params : {};
    }

    function stringArg(args, key, fallback = '') {
      const value = args?.[key];
      return typeof value === 'string' ? value : fallback;
    }

    function nullableStringArg(args, key) {
      const value = stringArg(args, key).trim();
      return value.length > 0 ? value : null;
    }

    function numberArg(args, key, fallback = 0) {
      const value = args?.[key];
      return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
    }

    function listContacts(args) {
      const params = pageArgs(args);
      const filterType = typeof params.filter_type === 'string' ? params.filter_type : undefined;
      const searchQuery = typeof params.search_query === 'string' ? params.search_query.toLowerCase() : '';
      let rows = contacts;
      if (filterType) rows = rows.filter((item) => item.contact_type === filterType);
      if (searchQuery) {
        rows = rows.filter((item) => [
          item.first_name,
          item.last_name,
          item.org_name,
          item.email,
          item.phone,
        ].some((value) => value.toLowerCase().includes(searchQuery)));
      }
      return {
        contacts: rows,
        total: rows.length,
        page: 1,
        per_page: 50,
      };
    }

    function dashboardStats() {
      const openDeals = deals.filter((item) => item.stage !== 'Closed Won' && item.stage !== 'Closed Lost');
      const pipelineValue = openDeals.reduce((sum, item) => sum + item.value, 0);
      return {
        total_contacts: contacts.length,
        total_organizations: organizations.length,
        active_deals: openDeals.length,
        pipeline_value: pipelineValue,
        pipeline_value_by_currency: [
          { currency: 'USD', total_value: pipelineValue, deal_count: openDeals.length },
        ],
        upcoming_activities: activities.filter((item) => !item.completed).length,
        overdue_activities: 0,
      };
    }

    function pipelineSummary() {
      return ['Lead', 'Qualified', 'Proposal', 'Negotiation', 'Closed Won', 'Closed Lost'].map((stage) => {
        const stageDeals = deals.filter((item) => item.stage === stage);
        return {
          stage,
          count: stageDeals.length,
          total_value: stageDeals.reduce((sum, item) => sum + item.value, 0),
          weighted_value: stageDeals.reduce((sum, item) => sum + item.value * (item.probability / 100), 0),
        };
      });
    }

    function pipelineConversionReport() {
      const total = deals.length;
      const won = deals.filter((item) => item.stage === 'Closed Won').length;
      const lost = deals.filter((item) => item.stage === 'Closed Lost').length;
      return {
        generated_at: timestamp,
        total_deals: total,
        open_deals: total - won - lost,
        closed_won: won,
        closed_lost: lost,
        overall_win_rate: won / Math.max(1, won + lost),
        stage_metrics: pipelineSummary().map((item) => ({
          stage: item.stage,
          count: item.count,
          total_value: item.total_value,
          weighted_value: item.weighted_value,
          stage_share: item.count / total,
        })),
        transition_metrics: [],
      };
    }

    function activityFunnelReport() {
      const completed = activities.filter((item) => item.completed).length;
      const pending = activities.length - completed;
      return {
        generated_at: timestamp,
        total_activities: activities.length,
        completed_activities: completed,
        pending_activities: pending,
        overdue_activities: 0,
        completion_rate: completed / activities.length,
        overdue_rate: 0,
        by_type: ['task', 'call', 'meeting', 'email'].map((type) => {
          const rows = activities.filter((item) => item.activity_type === type);
          const done = rows.filter((item) => item.completed).length;
          return {
            activity_type: type,
            total: rows.length,
            completed: done,
            pending: rows.length - done,
            overdue: 0,
            completion_rate: rows.length === 0 ? 0 : done / rows.length,
          };
        }),
        due_buckets: {
          overdue: 0,
          due_today: 0,
          due_next_7_days: pending,
          due_later: 0,
          no_due_date: 0,
        },
      };
    }
  }, debugEnabled);
}
