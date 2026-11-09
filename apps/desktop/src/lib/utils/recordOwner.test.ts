import { describe, expect, it } from 'vitest';
import { matchesRecordOwner } from './recordOwner';

describe('matchesRecordOwner', () => {
  it('treats a blank filter as match-all', () => {
    expect(matchesRecordOwner('Samira', '')).toBe(true);
    expect(matchesRecordOwner(null, '  ')).toBe(true);
  });

  it('matches a trimmed case-insensitive name', () => {
    expect(matchesRecordOwner('Samira', ' samira ')).toBe(true);
    expect(matchesRecordOwner('Samira', 'Amina')).toBe(false);
    expect(matchesRecordOwner(null, 'Samira')).toBe(false);
  });
});
