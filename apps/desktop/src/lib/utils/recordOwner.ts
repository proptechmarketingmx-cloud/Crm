/** Case-insensitive exact match for the optional local owner name. */
export function matchesRecordOwner(
  owner: string | null | undefined,
  filter: string | null | undefined,
): boolean {
  const normalized = filter?.trim().toLowerCase() ?? '';
  if (!normalized) {
    return true;
  }
  return (owner ?? '').trim().toLowerCase() === normalized;
}
