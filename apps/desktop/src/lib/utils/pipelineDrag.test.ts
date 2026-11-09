// @vitest-environment jsdom

import { describe, expect, it } from 'vitest';
import { isLeavingDropTarget, pipelineDropKind } from './pipelineDrag';

describe('pipelineDropKind', () => {
  it('treats another stage as a valid drop target', () => {
    expect(pipelineDropKind('lead', 'qualified')).toBe('target');
  });

  it('marks the current stage so it is not advertised as a drop', () => {
    expect(pipelineDropKind('proposal', 'proposal')).toBe('source');
  });

  it('stays idle when nothing is being dragged', () => {
    expect(pipelineDropKind(null, 'qualified')).toBe('none');
    expect(pipelineDropKind('lead', null)).toBe('none');
  });
});

describe('isLeavingDropTarget', () => {
  it('ignores moves into a child of the same column', () => {
    const column = document.createElement('div');
    const card = document.createElement('button');
    column.append(card);

    expect(
      isLeavingDropTarget({
        currentTarget: column,
        relatedTarget: card,
      }),
    ).toBe(false);
  });

  it('clears the highlight when the pointer leaves the column', () => {
    const column = document.createElement('div');
    const other = document.createElement('div');

    expect(
      isLeavingDropTarget({
        currentTarget: column,
        relatedTarget: other,
      }),
    ).toBe(true);
  });
});
