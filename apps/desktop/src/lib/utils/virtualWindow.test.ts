import { describe, expect, it } from 'vitest';

import { visibleWindow } from './virtualWindow';

describe('visibleWindow', () => {
  it('returns an empty window for an empty list', () => {
    expect(visibleWindow(0, 0, 400, 100, 4)).toEqual({
      start: 0,
      end: 0,
      paddingTop: 0,
      paddingBottom: 0,
      visibleCount: 0,
    });
  });

  it('windows a long column so only a slice is in the DOM range', () => {
    const window = visibleWindow(2000, 8000, 400, 100, 4);

    expect(window.start).toBe(76);
    expect(window.end).toBe(88);
    expect(window.visibleCount).toBe(12);
    expect(window.paddingTop).toBe(7600);
    expect(window.paddingBottom).toBe((2000 - 88) * 100);
    expect(window.visibleCount).toBeLessThan(2000);
  });

  it('clamps the last page to the item count', () => {
    const window = visibleWindow(10, 900, 400, 100, 2);
    expect(window.end).toBe(10);
    expect(window.start).toBeGreaterThanOrEqual(0);
    expect(window.start).toBeLessThanOrEqual(window.end);
  });
});
