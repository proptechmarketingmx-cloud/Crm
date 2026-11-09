<script lang="ts" generics="T">
  /**
   * Fixed-height virtual list for long kanban columns and similar stacks.
   */

  import { visibleWindow } from '$lib/utils/virtualWindow';
  import type { Snippet } from 'svelte';

  let {
    items,
    itemHeight = 108,
    overscan = 6,
    children,
  }: {
    items: T[];
    itemHeight?: number;
    overscan?: number;
    children: Snippet<[T, number]>;
  } = $props();

  let scrollTop = $state(0);
  let viewportHeight = $state(400);
  let root: HTMLDivElement | undefined;

  const windowed = $derived(
    visibleWindow(items.length, scrollTop, viewportHeight, itemHeight, overscan)
  );
  const visibleItems = $derived(items.slice(windowed.start, windowed.end));

  function handleScroll(event: Event) {
    scrollTop = (event.currentTarget as HTMLElement).scrollTop;
  }

  $effect(() => {
    if (!root || typeof ResizeObserver === 'undefined') {
      return;
    }

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        viewportHeight = entry.contentRect.height;
      }
    });
    observer.observe(root);
    viewportHeight = root.clientHeight;
    return () => observer.disconnect();
  });
</script>

<div
  class="virtual-list"
  bind:this={root}
  onscroll={handleScroll}
  data-testid="virtual-list"
>
  <div
    class="virtual-list-spacer"
    style="height: {items.length * itemHeight}px;"
  >
    <div
      class="virtual-list-window"
      style="transform: translateY({windowed.paddingTop}px);"
    >
      {#each visibleItems as item, offset (offset + windowed.start)}
        {@render children(item, windowed.start + offset)}
      {/each}
    </div>
  </div>
</div>

<style>
  .virtual-list {
    flex: 1;
    min-height: 80px;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .virtual-list-spacer {
    position: relative;
    width: 100%;
  }

  .virtual-list-window {
    display: flex;
    flex-direction: column;
  }

  @media (prefers-reduced-motion: reduce) {
    .virtual-list-window {
      transition: none;
    }
  }
</style>
