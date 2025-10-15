const CACHE_NAME = 'yew-pwa-cache-v1.01.03';
const urlsToCache = [
  'index.html',
  'manifest.json',
  'assets/croatian.json',
  'assets/croatian_sentences.json',
  'assets/croatian_explanations.json',
  'pkg/yew_project_bg.wasm',
  'pkg/yew_project.js',
  'icons/icon-192x192.png',
  'icons/icon-512x512.png',
  'icons/icon-1024x1024.png',
  'static/style.css',
];

self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => cache.addAll(urlsToCache))
  );
});

self.addEventListener('fetch', event => {
  // Ignore dev tooling
  if (event.request.url.includes('browser-sync')) {
    return;
  }

  event.respondWith(
    caches.match(event.request)
      .then(response => {
        if (response) {
          return response;
        }
        return fetch(event.request)
          .then(networkResponse => {
            // Cache a copy of the response for next time (best-effort)
            const copy = networkResponse.clone();
            caches.open(CACHE_NAME).then(cache => cache.put(event.request, copy)).catch(() => {});
            return networkResponse;
          })
          .catch(() => caches.match('index.html'));
      })
  );
});

self.addEventListener('activate', event => {
  const whitelist = [CACHE_NAME];
  event.waitUntil(
    caches.keys().then(cacheNames => Promise.all(
      cacheNames.map(name => (whitelist.includes(name) ? undefined : caches.delete(name)))
    ))
  );
  self.clients.claim();
});

