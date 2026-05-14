import { describe, it, expect } from 'vitest';
import { isPast, fmtDateTime, fmtTime } from './date';

const PAST   = '2020-01-15T10:30:00.000Z';
const FUTURE = '2099-12-31T23:59:59.000Z';

describe('isPast', () => {
  it('returns true for a date in the past', () => {
    expect(isPast(PAST)).toBe(true);
  });

  it('returns false for a date in the future', () => {
    expect(isPast(FUTURE)).toBe(false);
  });
});

describe('fmtDateTime', () => {
  it('returns a non-empty string', () => {
    expect(fmtDateTime(FUTURE).length).toBeGreaterThan(0);
  });

  it('contains the hour portion of the timestamp', () => {
    // 23:59 should appear somewhere in the formatted string
    const result = fmtDateTime(FUTURE);
    expect(result).toMatch(/23|59/);
  });
});

describe('fmtTime', () => {
  it('returns a non-empty string', () => {
    expect(fmtTime(FUTURE).length).toBeGreaterThan(0);
  });

  it('contains the hour portion of the timestamp', () => {
    expect(fmtTime(FUTURE)).toMatch(/23|59/);
  });
});
