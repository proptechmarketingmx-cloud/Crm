<script lang="ts">
  /**
   * +page.svelte — Hash route renderer for Tauri shell mode.
   */

  import { browser } from '$app/environment';
  import Dashboard from './Dashboard.svelte';
  import Contacts from './Contacts.svelte';
  import Organizations from './Organizations.svelte';
  import Pipeline from './Pipeline.svelte';
  import DealDetail from './DealDetail.svelte';
  import Activities from './Activities.svelte';
  import Reports from './Reports.svelte';
  import Settings from './Settings.svelte';
  import ContactDetail from './ContactDetail.svelte';
  import OrganizationDetail from './OrganizationDetail.svelte';
  import AuditLog from './AuditLog.svelte';
  import PendingActions from './PendingActions.svelte';
  import { currentHashPath, installHashRouteSync } from '$lib/utils/hashRouter';

  interface ParsedRoute {
    route: string;
    contactId: string | null;
    organizationId: string | null;
    dealId: string | null;
  }

  function emptyRoute(route: string): ParsedRoute {
    return { route, contactId: null, organizationId: null, dealId: null };
  }

  function parseRoutePath(path: string): ParsedRoute {
    const clean = path.startsWith('/') ? path : `/${path}`;
    const normalized = clean.replace(/\/+$/, '') || '/';

    if (normalized === '/leads') {
      return emptyRoute('/leads');
    }

    if (normalized === '/contacts') {
      return emptyRoute('/contacts');
    }

    if (normalized.startsWith('/contacts/')) {
      const id = normalized.split('/')[2] ?? '';
      return { route: '/contacts/:id', contactId: id || null, organizationId: null, dealId: null };
    }

    if (normalized === '/organizations') {
      return emptyRoute('/organizations');
    }

    if (normalized.startsWith('/organizations/')) {
      const id = normalized.split('/')[2] ?? '';
      return { route: '/organizations/:id', contactId: null, organizationId: id || null, dealId: null };
    }

    if (normalized === '/deals') {
      return emptyRoute('/pipeline');
    }

    if (normalized.startsWith('/deals/')) {
      const id = normalized.split('/')[2] ?? '';
      return { route: '/deals/:id', contactId: null, organizationId: null, dealId: id || null };
    }

    if (normalized === '/pipeline') {
      return emptyRoute('/pipeline');
    }

    if (normalized.startsWith('/pipeline/')) {
      const id = normalized.split('/')[2] ?? '';
      return { route: '/pipeline/:id', contactId: null, organizationId: null, dealId: id || null };
    }

    if (normalized === '/activities' || normalized.startsWith('/activities/')) {
      return emptyRoute('/activities');
    }

    if (normalized === '/reports') {
      return emptyRoute('/reports');
    }

    if (normalized === '/settings' || normalized.startsWith('/settings/')) {
      return emptyRoute('/settings');
    }

    if (normalized === '/audit-log') {
      return emptyRoute('/audit-log');
    }

    if (normalized === '/pending-actions') {
      return emptyRoute('/pending-actions');
    }

    return emptyRoute('/');
  }

  function readHashRoute(): ParsedRoute {
    if (!browser) {
      return emptyRoute('/');
    }

    return parseRoutePath(currentHashPath());
  }

  const initialRoute = readHashRoute();

  let route = $state(initialRoute.route);
  let contactId = $state<string | null>(initialRoute.contactId);
  let organizationId = $state<string | null>(initialRoute.organizationId);
  let dealId = $state<string | null>(initialRoute.dealId);
  let routeSyncInitialized = false;

  function parseRoute(path: string) {
    const parsed = parseRoutePath(path);
    route = parsed.route;
    contactId = parsed.contactId;
    organizationId = parsed.organizationId;
    dealId = parsed.dealId;
  }

  $effect(() => {
    if (!browser || routeSyncInitialized) {
      return;
    }

    routeSyncInitialized = true;

    const syncFromHash = () => {
      parseRoute(currentHashPath());
    };
    const removeRouteSync = installHashRouteSync(syncFromHash);

    syncFromHash();
    window.addEventListener('hashchange', syncFromHash);

    return () => {
      removeRouteSync();
      window.removeEventListener('hashchange', syncFromHash);
    };
  });
</script>

{#if route === '/contacts/:id' && contactId}
  <ContactDetail {contactId} />
{:else if route === '/organizations/:id' && organizationId}
  <OrganizationDetail {organizationId} />
{:else if route === '/leads'}
  <Contacts mode="leads" />
{:else if route === '/contacts'}
  <Contacts />
{:else if route === '/organizations'}
  <Organizations />
{:else if route === '/deals/:id' && dealId}
  <DealDetail {dealId} />
{:else if route === '/pipeline' || route === '/pipeline/:id'}
  <Pipeline {dealId} />
{:else if route === '/activities'}
  <Activities />
{:else if route === '/reports'}
  <Reports />
{:else if route === '/settings'}
  <Settings />
{:else if route === '/audit-log'}
  <AuditLog />
{:else if route === '/pending-actions'}
  <PendingActions />
{:else}
  <Dashboard />
{/if}
