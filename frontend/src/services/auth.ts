import { UserManager } from 'oidc-client-ts';
import { config } from '../config';

export const userManager = new UserManager({
  authority: `${config.keycloakUrl}/realms/${config.keycloakRealm}`,
  client_id: config.keycloakClientId,
  redirect_uri: window.location.origin,
  post_logout_redirect_uri: window.location.origin,
  scope: 'openid profile',
});
