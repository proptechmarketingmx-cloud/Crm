/**
 * Small hash-route bridge for the Tauri shell.
 *
 * Browser hashchange delivery can be inconsistent during SSR hydration and
 * browser-smoke reloads, so navigation updates the hash and asks the route
 * renderer to sync immediately.
 */

type HashRouteWindow = Window & {
  __900crmSyncHashRoute?: () => void;
};

export function currentHashPath(): string {
  if (typeof window === 'undefined') {
    return '/';
  }

  return window.location.hash.replace(/^#/, '') || '/';
}

export function routeHash(path: string): string {
  if (path.startsWith('#')) {
    return path;
  }

  return path === '/' ? '#/' : `#${path.startsWith('/') ? path : `/${path}`}`;
}

export function installHashRouteSync(sync: () => void): () => void {
  if (typeof window === 'undefined') {
    return () => {};
  }

  const routeWindow = window as HashRouteWindow;
  routeWindow.__900crmSyncHashRoute = sync;

  return () => {
    if (routeWindow.__900crmSyncHashRoute === sync) {
      delete routeWindow.__900crmSyncHashRoute;
    }
  };
}

export function notifyHashRouteChanged(): void {
  if (typeof window === 'undefined') {
    return;
  }

  const routeWindow = window as HashRouteWindow;
  routeWindow.__900crmSyncHashRoute?.();
  window.dispatchEvent(new HashChangeEvent('hashchange'));
}

export function navigateHash(path: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.location.hash = routeHash(path);
  notifyHashRouteChanged();
}
