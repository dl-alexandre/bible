const CACHE_NAME = 'bible-shell-v2';
const MAX_DYNAMIC_ENTRIES = 80;
const SHELL = [
  '/bible/',
  '/bible/static/styles.css',
  '/bible/static/verses.js',
  '/bible/static/search.js',
  '/bible/static/search-worker.js',
  '/bible/static/app.webmanifest'
];

self.addEventListener('install', event => {
  event.waitUntil(caches.open(CACHE_NAME).then(cache => cache.addAll(SHELL)));
  self.skipWaiting();
});

self.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(keys => Promise.all(
      keys.filter(key => key !== CACHE_NAME).map(key => caches.delete(key))
    ))
  );
  self.clients.claim();
});

self.addEventListener('fetch', event => {
  if (event.request.method !== 'GET' || !event.request.url.startsWith(self.location.origin)) return;

  event.respondWith(
    fetch(event.request)
      .then(response => {
        const copy = response.clone();
        caches.open(CACHE_NAME).then(async cache => {
          await cache.put(event.request, copy);
          const keys = await cache.keys();
          const dynamicKeys = keys.filter(key => !SHELL.includes(new URL(key.url).pathname));
          for (const key of dynamicKeys.slice(0, Math.max(0, dynamicKeys.length - MAX_DYNAMIC_ENTRIES))) {
            await cache.delete(key);
          }
        });
        return response;
      })
      .catch(() => caches.match(event.request).then(cached => cached || caches.match('/bible/')))
  );
});

self.addEventListener('message', event => {
  if (event.data?.type === 'CLEAR_CACHE') {
    event.waitUntil(caches.delete(CACHE_NAME).then(() => caches.open(CACHE_NAME).then(cache => cache.addAll(SHELL))));
  }
});
