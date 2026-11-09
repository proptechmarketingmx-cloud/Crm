<script lang="ts">
  /**
   * +layout.svelte — Root application layout for 900CRM.
   *
   * Renders:
   *   - App sidebar with navigation links
   *   - Main content area (slot for child routes)
   *   - Toast notification overlay
   *
   * Applies theme (data-theme) and direction (dir) from settings store.
   * Loads settings and initial data on mount.
   */

  import '../app.css';
  import { onMount } from 'svelte';
  import { t, isLocaleReady, isRtl } from '$lib/i18n';
  import { uiStore } from '$lib/stores/ui';
  import { settingsStore } from '$lib/stores/settings';
  import Toast from '$lib/components/Toast.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import type { SearchResult } from '$lib/stores/ui';
  import GlobalModalHost from '$lib/components/GlobalModalHost.svelte';
  import { startActivityReminderService } from '$lib/services/activityReminders';
  import { currentHashPath, navigateHash, routeHash } from '$lib/utils/hashRouter';
  import { reviewCountsStore } from '$lib/stores/reviewCounts';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import ShortcutSheet from '$lib/components/ShortcutSheet.svelte';
  import {
    focusGlobalSearch,
    isModKey,
    isTypingTarget,
    workspaceHrefForDigit,
  } from '$lib/utils/keyboard';

  // ── Child route ──────────────────────────────────────────────────────────────

  let { children } = $props();

  // ── State ───────────────────────────────────────────────────────────────────

  let currentRoute = $state('/');
  let commandPaletteOpen = $state(false);
  let shortcutSheetOpen = $state(false);

  // ── Navigation ─────────────────────────────────────────────────────────────

  interface NavItem {
    id: string;
    label: () => string;
    href: string;
    icon: string;
    badge?: () => number;
  }

  interface NavSection {
    id: string;
    label: () => string;
    items: NavItem[];
  }

  const workspaceItems: NavItem[] = [
    {
      id: 'dashboard',
      label: () => t('nav.dashboard'),
      href: '/',
      icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6',
    },
    {
      id: 'leads',
      label: () => t('nav.leads'),
      href: '/leads',
      icon: 'M16 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2M8.5 11a4 4 0 100-8 4 4 0 000 8M20 8v6M23 11h-6',
    },
    {
      id: 'contacts',
      label: () => t('nav.contacts'),
      href: '/contacts',
      icon: 'M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2M9 11a4 4 0 100-8 4 4 0 000 8zm8 2a2 2 0 100-4 2 2 0 000 4zm4 8v-2a4 4 0 00-3-3.87',
    },
    {
      id: 'organizations',
      label: () => t('nav.organizations'),
      href: '/organizations',
      icon: 'M3 21h18M5 21V7l8-4v18M19 21V11l-6-3M9 9h1m-1 4h1m-1 4h1m4-6h1m-1 4h1',
    },
    {
      id: 'pipeline',
      label: () => t('nav.pipeline'),
      href: '/pipeline',
      icon: 'M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2',
    },
    {
      id: 'activities',
      label: () => t('nav.activities'),
      href: '/activities',
      icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7l2 2 4-4',
    },
    {
      id: 'reports',
      label: () => t('nav.reports'),
      href: '/reports',
      icon: 'M4 19V5m0 14h16M8 16v-5m4 5V8m4 8v-9',
    },
  ];

  const reviewItems: NavItem[] = [
    {
      id: 'pending-actions',
      label: () => t('nav.pendingActions'),
      href: '/pending-actions',
      icon: 'M12 6v6l4 2M21 12a9 9 0 11-9-9M19 3v5h-5',
      badge: () => reviewCountsStore.pendingCount,
    },
    {
      id: 'audit-log',
      label: () => t('nav.auditLog'),
      href: '/audit-log',
      icon: 'M9 12h6M9 16h6M9 8h6M5 4h14v16H5zM5 8H3m2 4H3m2 4H3',
    },
  ];

  const adminItems: NavItem[] = [
    {
      id: 'settings',
      label: () => t('nav.settings'),
      href: '/settings',
      icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z',
    },
  ];

  const navSections: NavSection[] = [
    {
      id: 'workspace',
      label: () => t('nav.sections.workspace'),
      items: workspaceItems,
    },
    {
      id: 'review',
      label: () => t('nav.sections.review'),
      items: reviewItems,
    },
    {
      id: 'admin',
      label: () => t('nav.sections.admin'),
      items: adminItems,
    },
  ];

  // ── Helpers ─────────────────────────────────────────────────────────────────

  function navigate(href: string) {
    currentRoute = href;
    navigateHash(href);
  }

  function isActive(href: string): boolean {
    if (href === '/') return currentRoute === '/';
    return currentRoute.startsWith(href);
  }

  function itemTitle(section: NavSection, item: NavItem): string | undefined {
    const count = item.badge?.() ?? 0;
    const label = count > 0
      ? t('nav.pendingActionsCount', { count: reviewCountsStore.formatCount() })
      : item.label();
    return uiStore.sidebarCollapsed ? `${section.label()}: ${label}` : undefined;
  }

  function handleWorkspaceKeydown(event: KeyboardEvent) {
    if (isModKey(event) && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      commandPaletteOpen = true;
      shortcutSheetOpen = false;
      return;
    }

    if (isTypingTarget(event.target)) {
      return;
    }

    if (event.key === '/' && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      focusGlobalSearch();
      return;
    }

    if (event.key === '?' && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      shortcutSheetOpen = true;
      return;
    }

    const href = workspaceHrefForDigit(event.key);
    if (href) {
      event.preventDefault();
      navigate(href);
    }
  }

  function handleSearchResult(result: SearchResult) {
    if (result.type === 'contact') {
      navigate(`/contacts/${result.id}`);
      return;
    }

    if (result.type === 'organization') {
      navigate(`/organizations/${result.id}`);
      return;
    }

    if (result.type === 'deal') {
      navigate(`/deals/${result.id}`);
      return;
    }

    if (result.type === 'activity') {
      navigate('/activities');
    }
  }

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  onMount(() => {
    // Detect current route from hash
    currentRoute = currentHashPath();

    // Listen for hash changes
    const handleHashChange = () => {
      currentRoute = currentHashPath();
    };
    window.addEventListener('hashchange', handleHashChange);
    window.addEventListener('keydown', handleWorkspaceKeydown);

    let isActive = true;
    let stopReminderService = () => {};

    void (async () => {
      // Load settings (applies theme + locale)
      await settingsStore.loadSettings();
      if (!isActive) return;
      void reviewCountsStore.refresh();
      stopReminderService = startActivityReminderService();
    })();

    return () => {
      isActive = false;
      window.removeEventListener('hashchange', handleHashChange);
      window.removeEventListener('keydown', handleWorkspaceKeydown);
      stopReminderService();
    };
  });
</script>

<a
  class="skip-link"
  href="#main-content"
  onclick={(event) => {
    event.preventDefault();
    document.getElementById('main-content')?.focus();
  }}
>
  {t('nav.skipToContent')}
</a>

<div class="app-shell">
  <!-- Sidebar -->
  <aside
    class="app-sidebar"
    class:collapsed={uiStore.sidebarCollapsed}
    aria-label={t('nav.dashboard')}
  >
    <!-- Logo -->
    <div class="sidebar-logo">
      <!-- SVG Logo mark -->
      <svg width="28" height="28" viewBox="0 0 28 28" fill="none" aria-hidden="true">
        <rect width="28" height="28" rx="6" fill="var(--color-primary)"/>
        <text x="14" y="19" text-anchor="middle" font-size="13" font-weight="700" fill="white" font-family="system-ui, sans-serif">9C</text>
      </svg>
      {#if !uiStore.sidebarCollapsed}
        <span class="sidebar-logo-text">900CRM</span>
      {/if}
    </div>

    <!-- Navigation -->
    <nav class="sidebar-nav" aria-label="Main navigation">
      {#each navSections as section (section.id)}
        <div class="sidebar-nav-section">
          {#if !uiStore.sidebarCollapsed}
            <span class="nav-section-label">{section.label()}</span>
          {/if}

          {#each section.items as item (item.id)}
            <a
              class="nav-link"
              class:active={isActive(item.href)}
              href={routeHash(item.href)}
              onclick={(e) => { e.preventDefault(); navigate(item.href); }}
              title={itemTitle(section, item)}
              aria-current={isActive(item.href) ? 'page' : undefined}
              aria-label={item.badge?.()
                ? t('nav.pendingActionsCount', { count: reviewCountsStore.formatCount() })
                : undefined}
            >
              <span class="nav-icon-wrap">
                <svg
                  class="nav-icon"
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.75"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                  style="flex-shrink: 0;"
                >
                  <path d={item.icon} />
                </svg>
                {#if uiStore.sidebarCollapsed && (item.badge?.() ?? 0) > 0}
                  <span class="nav-badge-dot" data-testid="nav-pending-dot"></span>
                {/if}
              </span>
              {#if !uiStore.sidebarCollapsed}
                <span class="nav-label">{item.label()}</span>
                {#if (item.badge?.() ?? 0) > 0}
                  <span class="nav-badge" data-testid="nav-pending-count">{reviewCountsStore.formatCount()}</span>
                {/if}
              {/if}
            </a>
          {/each}
        </div>
      {/each}
    </nav>

    <!-- Footer: collapse toggle -->
    <div class="sidebar-footer">
      <button
        class="nav-link"
        onclick={() => uiStore.toggleSidebar()}
        aria-label={uiStore.sidebarCollapsed ? t('nav.expandSidebar') : t('nav.collapseSidebar')}
        type="button"
        style="width: 100%;"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.75"
          stroke-linecap="round"
          class="nav-icon sidebar-collapse-icon"
          aria-hidden="true"
          style="transform: rotate({uiStore.sidebarCollapsed === isRtl() ? 180 : 0}deg); transition: transform var(--duration-normal) var(--ease-out);"
        >
          <path d="M11 19l-7-7 7-7m8 14l-7-7 7-7"/>
        </svg>
        {#if !uiStore.sidebarCollapsed}
          <span class="nav-label">{t('nav.collapse')}</span>
        {/if}
      </button>
    </div>
  </aside>

  <!-- Main content -->
  <main class="app-main" id="main-content" tabindex="-1" aria-label="Main content">
    <!-- Top bar with search -->
    {#if !uiStore.sidebarCollapsed || true}
      <div class="top-bar">
        <SearchBar placeholder={t('common.search')} onselectresult={handleSearchResult} />
      </div>
    {/if}

    <!-- Route content -->
    {#if isLocaleReady()}
      {@render children()}
    {:else}
      <div class="locale-loading">
        <svg class="animate-spin" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--color-primary)" stroke-width="2" aria-hidden="true">
          <path d="M21 12a9 9 0 11-6.219-8.56"/>
        </svg>
      </div>
    {/if}
  </main>
</div>

<!-- Toast overlay (outside app-shell so it always floats above everything) -->
<CommandPalette
  open={commandPaletteOpen}
  onclose={() => { commandPaletteOpen = false; }}
  onnavigate={(href) => navigate(href)}
/>
<ShortcutSheet
  open={shortcutSheetOpen}
  onclose={() => { shortcutSheetOpen = false; }}
/>
<GlobalModalHost />
<Toast />

<style>
  .skip-link {
    position: absolute;
    inset-inline-start: var(--space-3);
    top: var(--space-3);
    z-index: 90;
    padding: var(--space-2) var(--space-3);
    background: var(--surface-app);
    border: var(--border-width) solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    text-decoration: none;
    transform: translateY(-160%);
  }

  .skip-link:focus {
    transform: none;
  }

  .top-bar {
    display: flex;
    align-items: center;
    padding: var(--space-3) var(--space-8);
    border-block-end: var(--border-width) solid var(--border-subtle);
    flex-shrink: 0;
    height: 48px;
    background-color: var(--surface-app);
    gap: var(--space-4);
  }

  .locale-loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
