import { describe, it, expect } from 'vitest';
import type { User } from 'oidc-client-ts';
import { isAdmin } from './auth';

function userWithClaims(claims: Record<string, unknown>): User {
  const payload = btoa(JSON.stringify(claims)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  return { access_token: `header.${payload}.signature` } as User;
}

describe('isAdmin', () => {
  it('returns false when user is missing', () => {
    expect(isAdmin(undefined)).toBe(false);
    expect(isAdmin(null)).toBe(false);
  });

  it('returns true when admin role is present', () => {
    const user = userWithClaims({ realm_access: { roles: ['user', 'admin'] } });
    expect(isAdmin(user)).toBe(true);
  });

  it('returns false when roles do not include admin', () => {
    const user = userWithClaims({ realm_access: { roles: ['user'] } });
    expect(isAdmin(user)).toBe(false);
  });

  it('returns false when realm_access is missing', () => {
    const user = userWithClaims({});
    expect(isAdmin(user)).toBe(false);
  });

  it('returns false when roles array is empty', () => {
    const user = userWithClaims({ realm_access: { roles: [] } });
    expect(isAdmin(user)).toBe(false);
  });

  it('returns false when the access token is malformed', () => {
    expect(isAdmin({ access_token: 'not-a-jwt' } as User)).toBe(false);
  });
});
