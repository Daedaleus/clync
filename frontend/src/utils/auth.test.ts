import { describe, it, expect, beforeEach, vi } from 'vitest';
import keycloak from '../services/auth';

vi.mock('../services/auth', () => ({
  default: { tokenParsed: undefined as unknown },
}));

import { isAdmin } from './auth';

describe('isAdmin', () => {
  beforeEach(() => {
    (keycloak as { tokenParsed: unknown }).tokenParsed = undefined;
  });

  it('returns false when tokenParsed is undefined', () => {
    expect(isAdmin()).toBe(false);
  });

  it('returns true when admin role is present', () => {
    (keycloak as { tokenParsed: unknown }).tokenParsed = {
      realm_access: { roles: ['user', 'admin'] },
    };
    expect(isAdmin()).toBe(true);
  });

  it('returns false when roles do not include admin', () => {
    (keycloak as { tokenParsed: unknown }).tokenParsed = {
      realm_access: { roles: ['user'] },
    };
    expect(isAdmin()).toBe(false);
  });

  it('returns false when realm_access is missing', () => {
    (keycloak as { tokenParsed: unknown }).tokenParsed = {};
    expect(isAdmin()).toBe(false);
  });

  it('returns false when roles array is empty', () => {
    (keycloak as { tokenParsed: unknown }).tokenParsed = {
      realm_access: { roles: [] },
    };
    expect(isAdmin()).toBe(false);
  });
});
