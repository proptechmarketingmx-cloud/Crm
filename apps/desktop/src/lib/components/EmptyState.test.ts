// @vitest-environment jsdom

import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import EmptyState from './EmptyState.svelte';

describe('EmptyState component', () => {
  it('renders the user-facing title and description', () => {
    render(EmptyState, {
      title: 'No contacts yet',
      description: 'Add your first contact to start tracking relationships.',
      icon: 'contacts',
    });

    expect(screen.getByText('No contacts yet')).toBeTruthy();
    expect(screen.getByText('Add your first contact to start tracking relationships.')).toBeTruthy();
  });

  it('runs the configured action from the CTA button', async () => {
    const onaction = vi.fn();

    render(EmptyState, {
      title: 'No deals yet',
      actionLabel: 'Add deal',
      onaction,
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Add deal' }));

    expect(onaction).toHaveBeenCalledOnce();
  });
});
