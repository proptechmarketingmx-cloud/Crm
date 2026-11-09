/**
 * Keyboard helpers for workspace shortcuts.
 */

const TYPING_TAGS = new Set(['INPUT', 'TEXTAREA', 'SELECT']);

export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  if (target.isContentEditable) {
    return true;
  }

  if (TYPING_TAGS.has(target.tagName)) {
    return true;
  }

  return Boolean(target.closest('input, textarea, select, [contenteditable="true"]'));
}

export function isModKey(event: KeyboardEvent): boolean {
  return event.metaKey || event.ctrlKey;
}

export const WORKSPACE_NAV_SHORTCUTS: ReadonlyArray<{ key: string; href: string }> = [
  { key: '1', href: '/' },
  { key: '2', href: '/leads' },
  { key: '3', href: '/contacts' },
  { key: '4', href: '/organizations' },
  { key: '5', href: '/pipeline' },
  { key: '6', href: '/activities' },
  { key: '7', href: '/reports' },
];

export function workspaceHrefForDigit(key: string): string | null {
  return WORKSPACE_NAV_SHORTCUTS.find((item) => item.key === key)?.href ?? null;
}

export function focusGlobalSearch(): boolean {
  const input = document.getElementById('global-search');
  if (!(input instanceof HTMLInputElement)) {
    return false;
  }
  input.focus();
  input.select();
  return true;
}
