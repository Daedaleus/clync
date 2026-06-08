import { userManager } from './auth';
import { config } from '../config';
import i18n from '../i18n';

const API_BASE = config.apiUrl;

/** Extracts and translates the error message from a non-OK response. */
async function extractError(response: Response): Promise<string> {
  try {
    const body = await response.json();
    if (typeof body.error === 'string' && body.error.length > 0) {
      const msg = body.error;
      return /^error\.[a-z_]+\.[a-z_]+$/.test(msg) ? i18n.t(msg) : msg;
    }
  } catch { /* response was not JSON */ }
  return i18n.t('error.http', { status: response.status });
}

/**
 * Returns a valid access token, refreshing it if needed.
 * `automaticSilentRenew` keeps the cached token fresh in the background, so
 * this is mostly a safety net — falls back to a silent refresh and finally to
 * a full login redirect if the refresh token itself has expired.
 */
async function getAccessToken(): Promise<string> {
  const user = await userManager.getUser();
  if (user && !user.expired) return user.access_token;

  const renewed = await userManager.signinSilent().catch(() => null);
  if (renewed) return renewed.access_token;

  // Another tab sharing this (localStorage) session may have renewed concurrently —
  // Keycloak rejects a second renewal attempt with the same (now-rotated) refresh
  // token, which would otherwise force a needless redirect to the login page even
  // though the session is still perfectly valid. Re-check before giving up.
  const fresh = await userManager.getUser();
  if (fresh && !fresh.expired) return fresh.access_token;

  await userManager.signinRedirect();
  throw new Error('session expired — redirecting to login');
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  signal?: AbortSignal,
): Promise<T> {
  const token = await getAccessToken();

  const response = await fetch(`${API_BASE}${path}`, {
    method,
    headers: {
      Authorization: `Bearer ${token}`,
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
    const token = await getAccessToken();
    const form = new FormData();
    form.append('file', file);
    const response = await fetch(`${API_BASE}${path}`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
      body: form,
    });
    if (!response.ok) throw new Error(await extractError(response));
    return response.json() as Promise<T>;
  },
};
