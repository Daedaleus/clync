import { describe, it, expect } from 'vitest';
import { isAbortError } from './abort';

describe('isAbortError', () => {
  it('returns true for an AbortError', () => {
    const err = new Error('aborted');
    err.name = 'AbortError';
    expect(isAbortError(err)).toBe(true);
  });

  it('returns false for a generic Error', () => {
    expect(isAbortError(new Error('something else'))).toBe(false);
  });

  it('returns false for a non-Error value', () => {
    expect(isAbortError('string error')).toBe(false);
    expect(isAbortError(null)).toBe(false);
    expect(isAbortError(42)).toBe(false);
  });

  it('returns false for an Error with a different name', () => {
    const err = new TypeError('type error');
    expect(isAbortError(err)).toBe(false);
  });
});
