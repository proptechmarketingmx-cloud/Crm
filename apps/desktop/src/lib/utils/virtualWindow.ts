/**
 * Compute a virtualized index window for a fixed-height list.
 */

export interface VirtualWindow {
  start: number;
  end: number;
  paddingTop: number;
  paddingBottom: number;
  visibleCount: number;
}

export function visibleWindow(
  itemCount: number,
  scrollTop: number,
  viewportHeight: number,
  itemHeight: number,
  overscan = 6,
): VirtualWindow {
  const count = Math.max(0, Math.trunc(itemCount));
  const height = Math.max(1, itemHeight);
  const view = Math.max(0, viewportHeight);
  const top = Math.max(0, scrollTop);
  const start = Math.max(0, Math.floor(top / height) - overscan);
  const visible = Math.ceil(view / height) + overscan * 2;
  const end = Math.min(count, start + visible);
  const safeStart = Math.min(start, end);

  return {
    start: safeStart,
    end,
    paddingTop: safeStart * height,
    paddingBottom: Math.max(0, count - end) * height,
    visibleCount: Math.max(0, end - safeStart),
  };
}
