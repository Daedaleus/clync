/**
 * Returns true when an error is an AbortError thrown by a cancelled fetch.
 * Use this to silently ignore cleanup-driven request cancellations instead of
 * showing spurious error messages.
 */
export function isAbortError(err: unknown): boolean {
  return err instanceof Error && err.name === 'AbortError';
}
