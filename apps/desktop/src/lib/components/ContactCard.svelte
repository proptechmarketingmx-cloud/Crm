<script lang="ts">
  /**
   * ContactCard.svelte — Compact contact card for 900CRM.
   *
   * Displays avatar (initials), name, org, email, phone.
   * Used in search results, linked entity lists, and contact picker.
   */

  import type { Contact } from '$lib/api/contacts';
  import { formatInitials, formatFullName } from '$lib/utils/formatters';

  // ── Props ──────────────────────────────────────────────────────────────────

  let {
    contact,
    selected = false,
    compact = false,
    onclick,
  }: {
    contact: Contact;
    selected?: boolean;
    compact?: boolean;
    onclick?: (contact: Contact) => void;
  } = $props();

  // ── Derived ────────────────────────────────────────────────────────────────

  const initials = $derived(formatInitials(contact.firstName, contact.lastName));
  const fullName = $derived(formatFullName(contact.firstName, contact.lastName));
  const avatarColor = $derived(stringToColor(contact.id));

  function stringToColor(str: string): string {
    const colors = [
      '#20808D', '#A84B2F', '#2D8659', '#7B5EA7',
      '#1E6BA8', '#C17F24', '#5B6E8C', '#A04060',
    ];
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length];
  }
</script>

<button
  class="contact-card"
  class:selected
  class:compact
  class:clickable={!!onclick}
  disabled={!onclick}
  type="button"
  onclick={() => onclick?.(contact)}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onclick?.(contact); }}
>
  <!-- Avatar -->
  <div
    class="contact-avatar"
    style="background-color: {avatarColor};"
    aria-hidden="true"
  >
    {initials}
  </div>

  <!-- Info -->
  <div class="contact-info">
    <p class="contact-name">{fullName}</p>
    {#if !compact}
      {#if contact.organization}
        <p class="contact-meta">{contact.organization}</p>
      {/if}
      {#if contact.email}
        <p class="contact-meta contact-email">{contact.email}</p>
      {/if}
    {:else}
      <p class="contact-meta">{contact.organization || contact.email || ''}</p>
    {/if}
  </div>

  <!-- Type badge -->
  {#if !compact}
    <span class="badge {contact.type === 'org' ? 'badge-neutral' : 'badge-primary'} contact-type-badge">
      {contact.type}
    </span>
  {/if}
</button>

<style>
  .contact-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    border-radius: var(--border-radius-md);
    border: none;
    width: 100%;
    text-align: start;
    font: inherit;
    color: inherit;
    background: transparent;
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .contact-card.clickable {
    cursor: pointer;
  }

  .contact-card:disabled {
    opacity: 1;
    cursor: default;
  }

  .contact-card.clickable:hover,
  .contact-card.selected {
    background-color: var(--surface-hover);
  }

  .contact-card.selected {
    background-color: var(--surface-active);
  }

  .contact-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: #fff;
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
  }

  .compact .contact-avatar {
    width: 28px;
    height: 28px;
    font-size: var(--text-xs);
  }

  .contact-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .contact-name {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .contact-meta {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .contact-type-badge {
    flex-shrink: 0;
    text-transform: capitalize;
  }
</style>
