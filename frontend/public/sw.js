self.addEventListener('push', (event) => {
  if (!event.data) return;

  let data;
  try {
    data = event.data.json();
  } catch {
    data = { title: 'WhatsUp', body: event.data.text(), url: '/' };
  }

  event.waitUntil(
    self.registration.showNotification(data.title ?? 'WhatsUp', {
      body: data.body ?? '',
      icon: '/favicon.svg',
      badge: '/favicon.svg',
      data: { url: data.url ?? '/' },
      requireInteraction: false,
    }),
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const url = event.notification.data?.url ?? '/';

  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true }).then((clientList) => {
      // Focus existing tab if open
      for (const client of clientList) {
        if ('focus' in client) {
          client.focus();
          client.postMessage({ type: 'navigate', url });
          return;
        }
      }
      // Otherwise open new tab
      if (clients.openWindow) {
        return clients.openWindow(url);
      }
    }),
  );
});
