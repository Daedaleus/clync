export const config = {
  apiUrl: import.meta.env.VITE_API_URL ?? 'http://localhost:3000',
  keycloakUrl: import.meta.env.VITE_KEYCLOAK_URL ?? 'http://localhost:8080',
  keycloakRealm: import.meta.env.VITE_KEYCLOAK_REALM ?? 'WhatsUp',
  keycloakClientId: import.meta.env.VITE_KEYCLOAK_CLIENT_ID ?? 'whatsup',
} as const;
