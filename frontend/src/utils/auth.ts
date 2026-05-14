import keycloak from '../services/auth';

/**
 * UI-only gate — reads the admin role from the local Keycloak token
 * to show or hide UI elements.
 *
 * This is NOT a security boundary. Tampering with the token in DevTools
 * grants no actual access — every privileged action is enforced on the
 * backend by verifying the JWT sent with each request.
 */
export function isAdmin(): boolean {
  const roles = (keycloak.tokenParsed as Record<string, unknown> | undefined)
    ?.realm_access as { roles?: string[] } | undefined;
  return roles?.roles?.includes('admin') ?? false;
}
