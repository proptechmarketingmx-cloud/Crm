// @vitest-environment jsdom

import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

vi.mock('$lib/i18n', () => ({
  t: (key: string, params?: Record<string, string | number>) => {
    if (key === 'nextStep.completeTitle') return `Complete ${params?.subject}`;
    if (key === 'nextStep.onTrackTitle') return `${params?.subject} is scheduled`;
    if (key === 'nextStep.completeAction') return 'Mark Complete';
    if (key === 'nextStep.eyebrow') return 'Next step';
    if (key === 'nextStep.completeDetail') return 'This follow-up is overdue.';
    if (key === 'nextStep.onTrackDetail') return 'The next follow-up is already on the calendar.';
    return key;
  },
}));

import NextStepCard from './NextStepCard.svelte';
import type { RecordNextStep } from '$lib/utils/recordNextStep';

function step(overrides: Partial<RecordNextStep> = {}): RecordNextStep {
  return {
    kind: 'completeOverdue',
    tone: 'danger',
    action: 'complete',
    activityId: 'activity-1',
    subject: 'Past due clinic check-in',
    ...overrides,
  };
}

describe('NextStepCard', () => {
  it('names the overdue follow-up and runs the primary action', async () => {
    const onaction = vi.fn();

    render(NextStepCard, {
      step: step(),
      onaction,
    });

    expect(screen.getByRole('heading', { name: 'Complete Past due clinic check-in' })).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: 'Mark Complete' }));
    expect(onaction).toHaveBeenCalledOnce();
  });

  it('hides the button when the record is already on track', () => {
    render(NextStepCard, {
      step: step({
        kind: 'onTrack',
        tone: 'success',
        action: 'none',
        subject: 'Call Maya',
      }),
    });

    expect(screen.getByRole('heading', { name: 'Call Maya is scheduled' })).toBeTruthy();
    expect(screen.queryByRole('button')).toBeNull();
  });
});
