/** @vitest-environment jsdom */
import { describe, expect, it } from 'vitest';

import { isModKey, isTypingTarget, workspaceHrefForDigit } from './keyboard';

describe('keyboard helpers', () => {
  it('maps digits 1-7 to the seven workspace routes', () => {
    expect(workspaceHrefForDigit('1')).toBe('/');
    expect(workspaceHrefForDigit('2')).toBe('/leads');
    expect(workspaceHrefForDigit('3')).toBe('/contacts');
    expect(workspaceHrefForDigit('4')).toBe('/organizations');
    expect(workspaceHrefForDigit('5')).toBe('/pipeline');
    expect(workspaceHrefForDigit('6')).toBe('/activities');
    expect(workspaceHrefForDigit('7')).toBe('/reports');
    expect(workspaceHrefForDigit('8')).toBeNull();
  });

  it('treats form fields as typing targets', () => {
    const input = document.createElement('input');
    const textarea = document.createElement('textarea');
    const button = document.createElement('button');

    expect(isTypingTarget(input)).toBe(true);
    expect(isTypingTarget(textarea)).toBe(true);
    expect(isTypingTarget(button)).toBe(false);
    expect(isTypingTarget(null)).toBe(false);
  });

  it('treats either Ctrl or Meta as the modifier key', () => {
    expect(isModKey({ ctrlKey: true, metaKey: false } as KeyboardEvent)).toBe(true);
    expect(isModKey({ ctrlKey: false, metaKey: true } as KeyboardEvent)).toBe(true);
    expect(isModKey({ ctrlKey: false, metaKey: false } as KeyboardEvent)).toBe(false);
  });
});
