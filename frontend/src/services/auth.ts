import { UserManager, WebStorageStateStore } from 'oidc-client-ts';
import { config } from '../config';

export const userManager = new UserManager({
  authority: `${config.keycloakUrl}/realms/${config.keycloakRealm}`,
  client_id: config.keycloakClientId,
  redirect_uri: window.location.origin,
  post_logout_redirect_uri: window.location.origin,
  scope: 'openid profile offline_access',
  // Persist the session in localStorage (survives browser/app restarts) instead
  // of the default sessionStorage — combined with the offline_access scope and
  // Keycloak's sliding "Offline Session Idle" timeout, this keeps users signed
  // in indefinitely as long as they open the app at least once within that window.
  userStore: new WebStorageStateStore({ store: window.localStorage }),
});
