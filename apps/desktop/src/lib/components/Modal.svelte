<script lang="ts">
  /**
   * Modal.svelte — Reusable modal dialog component for 900CRM.
   */

  import { t } from '$lib/i18n';
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';

  type ModalSize = 'sm' | 'md' | 'lg' | 'xl';

  let {
    open = $bindable(false),
    title = '',
    width,
    size = 'md',
    closeOnBackdrop = true,
    closeOnEscape = true,
    onclose,
    header,
    body,
    footer,
    children,
  }: {
    open?: boolean;
    title?: string;
    width?: string;
    size?: ModalSize;
    closeOnBackdrop?: boolean;
    closeOnEscape?: boolean;
    onclose?: () => void;
    header?: Snippet;
    body?: Snippet;
    footer?: Snippet;
    children?: Snippet;
  } = $props();

  let modalEl = $state<HTMLDivElement | undefined>(undefined);
  let previousFocus: Element | null = null;

  const computedWidth = $derived(
    width ?? ({ sm: '420px', md: '560px', lg: '760px', xl: '920px' }[size])
  );

  function close() {
    open = false;
    onclose?.();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (closeOnBackdrop && e.target === e.currentTarget) {
      close();
    }
  }

  function handleBackdropKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && closeOnEscape) {
      close();
    }
  }

  function handleGlobalKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && closeOnEscape && open) {
      close();
      return;
    }

    if (e.key === 'Tab' && modalEl && open) {
      const focusable = modalEl.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
      );

      if (focusable.length === 0) {
        return;
      }

      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }

  $effect(() => {
    if (open) {
      previousFocus = document.activeElement;
      requestAnimationFrame(() => {
        const first = modalEl?.querySelector<HTMLElement>(
          'button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [tabindex="0"]'
        );
        (first ?? modalEl)?.focus();
      });
    } else if (previousFocus instanceof HTMLElement) {
      previousFocus.focus();
      previousFocus = null;
    }
  });

  onMount(() => {
    document.addEventListener('keydown', handleGlobalKeyDown);
    return () => {
      document.removeEventListener('keydown', handleGlobalKeyDown);
    };
  });
</script>

{#if open}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label={title || t('common.loading')}
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={handleBackdropKeyDown}
  >
    <div
      class="modal"
      style="width: {computedWidth}; max-width: calc(100vw - 2rem);"
      bind:this={modalEl}
      tabindex="-1"
    >
      <div class="modal-header">
        {#if title}
          <span class="modal-title">{title}</span>
        {/if}
        {@render header?.()}
        <button
          class="icon-btn modal-close"
          onclick={close}
          aria-label={t('common.close')}
          type="button"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M12 4L4 12M4 4l8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        {#if body}
          {@render body()}
        {:else}
          {@render children?.()}
        {/if}
      </div>

      {#if footer}
        <div class="modal-footer">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-close {
    margin-inline-start: auto;
    flex-shrink: 0;
  }
</style>
