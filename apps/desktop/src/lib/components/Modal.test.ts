// @vitest-environment jsdom

import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import Modal from './Modal.svelte';

describe('Modal component', () => {
  it('renders an accessible dialog when open', () => {
    render(Modal, {
      open: true,
      title: 'Delete contact',
    });

    const dialog = screen.getByRole('dialog', { name: 'Delete contact' });

    expect(dialog).toBeTruthy();
    expect(screen.getByText('Delete contact')).toBeTruthy();
  });

  it('closes from the visible close button and notifies the caller', async () => {
    const onclose = vi.fn();

    render(Modal, {
      open: true,
      title: 'Edit deal',
      onclose,
    });

    await fireEvent.click(screen.getByRole('button'));

    expect(onclose).toHaveBeenCalledOnce();
    expect(screen.queryByRole('dialog', { name: 'Edit deal' })).toBeNull();
  });
});
