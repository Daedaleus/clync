import keycloak from './auth';
import { config } from '../config';

const API_BASE = config.apiUrl;

/** Extracts the human-readable message from a non-OK response. */
async function extractError(response: Response): Promise<string> {
  try {
    const body = await response.json();
    if (typeof body.error === 'string' && body.error.length > 0) return body.error;
  } catch { /* response was not JSON */ }
  // Fallback: only the status code — no statusText (can vary by server)
  return `Fehler ${response.status}`;
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  signal?: AbortSignal,
): Promise<T> {
  // Refresh the token if it expires within the next 30 seconds.
  // Throws if the refresh token itself has expired — Keycloak will redirect to login.
  await keycloak.updateToken(30).catch(() => keycloak.login());

  const response = await fetch(`${API_BASE}${path}`, {
    method,
    headers: {
      Authorization: `Bearer ${keycloak.token}`,
      'Content-Type': 'application/json',
    },
    body: body !== undefined ? JSON.stringify(body) : undefined,
    signal,
  });

  if (!response.ok) {
    throw new Error(await extractError(response));
  }

  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

export const api = {
  /** GET request. Pass an AbortSignal to cancel when the component unmounts. */
  get: <T>(path: string, signal?: AbortSignal) => request<T>('GET', path, undefined, signal),
  post: <T>(path: string, body?: unknown) => request<T>('POST', path, body),
  put: <T>(path: string, body?: unknown) => request<T>('PUT', path, body),
  delete: <T>(path: string) => request<T>('DELETE', path),

  upload: async <T>(path: string, file: File): Promise<T> => {
    await keycloak.updateToken(30).catch(() => keycloak.login());
    const form = new FormData();
    form.append('file', file);
    const response = await fetch(`${API_BASE}${path}`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${keycloak.token}` },
      body: form,
    });
    if (!response.ok) throw new Error(await extractError(response));
    return response.json() as Promise<T>;
  },
};
