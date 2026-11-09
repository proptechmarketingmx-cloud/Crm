export type PipelineDropKind = 'none' | 'source' | 'target';

export function pipelineDropKind(
  draggingFromStage: string | null,
  overStage: string | null,
): PipelineDropKind {
  if (!draggingFromStage || !overStage) {
    return 'none';
  }

  return overStage === draggingFromStage ? 'source' : 'target';
}

export function isLeavingDropTarget(event: {
  currentTarget: EventTarget | null;
  relatedTarget: EventTarget | null;
}): boolean {
  const current = event.currentTarget;
  if (!(current instanceof Element)) {
    return true;
  }

  const related = event.relatedTarget;
  if (!(related instanceof Node)) {
    return true;
  }

  return !current.contains(related);
}
