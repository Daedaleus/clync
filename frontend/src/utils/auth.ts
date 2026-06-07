import type { User } from 'oidc-client-ts';

/** Decodes a JWT's payload without verifying its signature — UI display only. */
function decodeJwtPayload(token: string): Record<string, unknown> | undefined {
  try {
    const [, payload] = token.split('.');
    return JSON.parse(atob(payload.replace(/-/g, '+').replace(/_/g, '/'))) as Record<string, unknown>;
  } catch {
    return undefined;
  }
}

/**
 * UI-only gate — reads the admin role from the local access token's
 * `realm_access.roles` claim (Keycloak-specific, not part of the ID token)
 * to show or hide UI elements.
 *
 * This is NOT a security boundary. Tampering with the token in DevTools
 * grants no actual access — every privileged action is enforced on the
 * backend by verifying the JWT sent with each request.
 */
export function isAdmin(user: User | null | undefined): boolean {
  const claims = user?.access_token ? decodeJwtPayload(user.access_token) : undefined;
  const realmAccess = claims?.realm_access as { roles?: string[] } | undefined;
  return realmAccess?.roles?.includes('admin') ?? false;
}
