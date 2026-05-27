import { useCallback, useEffect, useState } from 'react';
import { api } from '../services/api';

export type NotificationPermission = 'granted' | 'denied' | 'default' | 'unsupported';

function urlBase64ToUint8Array(b: string): Uint8Array {
  const pad = '='.repeat((4 - (b.length % 4)) % 4);
  const base64 = (b + pad).replace(/-/g, '+').replace(/_/g, '/');
  return Uint8Array.from([...window.atob(base64)].map((c) => c.charCodeAt(0)));
}

function toBase64url(buf: ArrayBuffer): string {
  return btoa(String.fromCharCode(...new Uint8Array(buf)))
    .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

async function storeSubscription(sub: PushSubscription): Promise<void> {
  const key = sub.getKey('p256dh');
  const auth = sub.getKey('auth');
  if (!key || !auth) throw new Error('Missing push subscription keys');
  await api.post('/api/v1/push/subscribe', {
    endpoint: sub.endpoint,
    p256dh: toBase64url(key),
    auth: toBase64url(auth),
  });
}

export function useNotifications() {
  const [permission, setPermission] = useState<NotificationPermission>(() => {
    if (!('Notification' in window)) return 'unsupported';
    return Notification.permission;
  });

  // Separate from OS permission: tracks whether a push subscription is active
  const [subscribed, setSubscribed] = useState(false);

  // On mount: ensure subscription is active and backend has the current endpoint.
  // iOS regularly revokes push subscriptions (inactivity, OS updates, memory pressure).
  // When that happens we re-subscribe silently so the user never has to tap the bell again.
  useEffect(() => {
    if (permission !== 'granted' || !('serviceWorker' in navigator)) return;

    navigator.serviceWorker.ready.then(async (registration) => {
      let sub = await registration.pushManager.getSubscription();

      if (!sub) {
        // Subscription was revoked by the browser/OS — re-create it without prompting.
        const { public_key: vapidKey } = await api.get<{ public_key: string }>(
          '/api/v1/push/vapid-public-key',
        );
        sub = await registration.pushManager.subscribe({
          userVisibleOnly: true,
          applicationServerKey: urlBase64ToUint8Array(vapidKey),
        });
      }

      // Always sync the current endpoint to the backend (covers key rotation too).
      await storeSubscription(sub);
      setSubscribed(true);
    }).catch(() => {
      // Re-subscription failed (e.g. network offline on first load) — mark as off
      // so the UI reflects reality; the next app open will retry automatically.
      setSubscribed(false);
    });
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const enable = async (): Promise<void> => {
    if (!('Notification' in window) || !('serviceWorker' in navigator)) return;

    // Step 1: OS permission
    const result = await Notification.requestPermission();
    setPermission(result);
    if (result !== 'granted') return;

    try {
      // Step 2: Service Worker ready
      const registration = await Promise.race([
        navigator.serviceWorker.ready,
        new Promise<never>((_, reject) =>
          setTimeout(() => reject(new Error('SW timeout')), 5000),
        ),
      ]) as ServiceWorkerRegistration;

      // Step 3: Fetch VAPID public key
      const { public_key: vapidKey } = await api.get<{ public_key: string }>(
        '/api/v1/push/vapid-public-key',
      );

      // Step 4: Reuse existing subscription or create a new one
      const sub =
        (await registration.pushManager.getSubscription()) ??
        (await registration.pushManager.subscribe({
          userVisibleOnly: true,
          applicationServerKey: urlBase64ToUint8Array(vapidKey),
        }));

      // Step 5: Store in backend
      await storeSubscription(sub);
      setSubscribed(true);
      console.log('[Push] ✅ Subscribed');
    } catch (err) {
      console.error('[Push] Subscription failed:', err);
    }
  };

  const disable = async (): Promise<void> => {
    try {
      const registration = await navigator.serviceWorker.ready;
      const sub = await registration.pushManager.getSubscription();
      if (sub) await sub.unsubscribe();
      await api.delete('/api/v1/push/subscribe');
      setSubscribed(false);
      console.log('[Push] Unsubscribed');
    } catch (err) {
      console.warn('[Push] Unsubscribe error:', err);
    }
  };

  /** Show an immediate OS notification when the tab is active. */
  const notify = useCallback((title: string, body: string, url = '/') => {
    if (permission !== 'granted') return;
    const n = new Notification(title, { body, icon: '/favicon.svg' });
    n.onclick = () => { window.focus(); window.location.href = url; n.close(); };
  }, [permission]);

  return { permission, subscribed, enable, disable, notify };
}
