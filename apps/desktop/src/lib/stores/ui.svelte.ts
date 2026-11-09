/**
 * src/lib/stores/ui.svelte.ts — UI state store for 900CRM.
 *
 * Manages transient UI state: sidebar, modals, toasts, search, loading flags.
 * All state uses Svelte 5 $state runes.
 *
 * @module stores/ui
 */

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/** Names of available modal dialogs. */
export type ActiveModal =
  | 'addContact'
  | 'editContact'
  | 'addDeal'
  | 'editDeal'
  | 'addActivity'
  | 'editActivity'
  | 'importExport'
  | 'confirmDelete'
  | null;

/** A toast notification. */
export interface ToastItem {
  /** Unique identifier. */
  id: string;
  /** Display message. */
  message: string;
  /** Visual variant. */
  type: 'success' | 'error' | 'warning' | 'info';
  /** Auto-dismiss after ms. 0 = never. */
  duration: number;
}

/** Global search result. */
export interface SearchResult {
  id: string;
  type: 'contact' | 'organization' | 'deal' | 'activity' | 'note' | 'tag';
  title: string;
  subtitle: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// UIStore
// ─────────────────────────────────────────────────────────────────────────────

/** Counter for unique ID generation. */
let _idCounter = 0;
function generateId(): string {
  return `ui-${Date.now()}-${++_idCounter}`;
}

/**
 * Reactive UI state store (Svelte 5 class-based runes).
 */
class UIStore {
  // ── Sidebar ────────────────────────────────────────────────────────────────

  /** Whether the sidebar is collapsed to icon-only mode. */
  sidebarCollapsed = $state<boolean>(false);

  // ── Current route / view ──────────────────────────────────────────────────

  /** The currently active navigation route. */
  activeRoute = $state<string>('/');

  // ── Modal ──────────────────────────────────────────────────────────────────

  /** Currently open modal, or null if none. */
  activeModal = $state<ActiveModal>(null);

  /** Optional data payload for the active modal (e.g. entity ID to edit). */
  modalData = $state<Record<string, unknown> | null>(null);

  // ── Loading flags ──────────────────────────────────────────────────────────

  /** Named loading flags. Multiple operations can be in-flight at once. */
  loadingFlags = $state<Set<string>>(new Set());

  /** True when ANY loading flag is set. */
  isLoading = $derived(this.loadingFlags.size > 0);

  // ── Toasts ─────────────────────────────────────────────────────────────────

  /** Active toast notifications. */
  toasts = $state<ToastItem[]>([]);

  // ── Search ─────────────────────────────────────────────────────────────────

  /** Current global search query. */
  searchQuery = $state<string>('');

  /** Global search results. */
  searchResults = $state<SearchResult[]>([]);

  /** Whether the search results dropdown is visible. */
  searchOpen = $state<boolean>(false);

  /** Whether a search is in flight. */
  isSearching = $state<boolean>(false);

  // ── Sidebar methods ────────────────────────────────────────────────────────

  /** Toggle sidebar collapsed state. */
  toggleSidebar(): void {
    this.sidebarCollapsed = !this.sidebarCollapsed;
  }

  /** Explicitly set sidebar collapsed. */
  setSidebarCollapsed(value: boolean): void {
    this.sidebarCollapsed = value;
  }

  // ── Modal methods ──────────────────────────────────────────────────────────

  /**
   * Open a modal with optional payload data.
   *
   * @param modal  Which modal to open
   * @param data   Optional data for the modal (e.g. entity to edit)
   */
  openModal(modal: ActiveModal, data?: Record<string, unknown>): void {
    this.activeModal = modal;
    this.modalData = data ?? null;
  }

  /** Close the active modal and clear payload. */
  closeModal(): void {
    this.activeModal = null;
    this.modalData = null;
  }

  // ── Loading flag methods ───────────────────────────────────────────────────

  /**
   * Set a named loading flag (marks operation in-progress).
   *
   * @param flag  Unique loading identifier
   */
  setLoading(flag: string): void {
    const next = new Set(this.loadingFlags);
    next.add(flag);
    this.loadingFlags = next;
  }

  /**
   * Clear a named loading flag (marks operation done).
   *
   * @param flag  Loading identifier to clear
   */
  clearLoading(flag: string): void {
    const next = new Set(this.loadingFlags);
    next.delete(flag);
    this.loadingFlags = next;
  }

  // ── Toast methods ──────────────────────────────────────────────────────────

  /**
   * Show a toast notification.
   *
   * @param message   The message to display
   * @param type      Visual variant (default: 'info')
   * @param duration  Auto-dismiss ms (default: 3500). 0 = never.
   * @returns         Toast ID for manual dismissal
   */
  toast(message: string, type: ToastItem['type'] = 'info', duration = 3500): string {
    const id = generateId();
    this.toasts = [...this.toasts, { id, message, type, duration }];

    if (duration > 0) {
      setTimeout(() => this.dismissToast(id), duration);
    }

    return id;
  }

  /** Convenience: success toast. */
  toastSuccess(message: string): string {
    return this.toast(message, 'success');
  }

  /** Convenience: error toast (longer duration). */
  toastError(message: string): string {
    return this.toast(message, 'error', 6000);
  }

  /** Convenience: warning toast. */
  toastWarning(message: string): string {
    return this.toast(message, 'warning', 4500);
  }

  /**
   * Dismiss a specific toast by ID.
   *
   * @param id  Toast ID
   */
  dismissToast(id: string): void {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  /** Dismiss all toasts. */
  clearToasts(): void {
    this.toasts = [];
  }

  // ── Search methods ─────────────────────────────────────────────────────────

  /** Update the search query string. */
  setSearchQuery(query: string): void {
    this.searchQuery = query;
  }

  /** Set search results and open the dropdown. */
  setSearchResults(results: SearchResult[]): void {
    this.searchResults = results;
    this.searchOpen = results.length > 0;
    this.isSearching = false;
  }

  /** Close the search results dropdown. */
  closeSearch(): void {
    this.searchOpen = false;
  }

  /** Clear the search query and results. */
  clearSearch(): void {
    this.searchQuery = '';
    this.searchResults = [];
    this.searchOpen = false;
    this.isSearching = false;
  }
}

/** Singleton UI store — import and use directly in components. */
export const uiStore = new UIStore();
