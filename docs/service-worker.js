// The build replaces this marker with a hash of the deployed files.
const CACHE_PREFIX = `language-learner:${self.registration.scope}:`;
const CACHE_NAME = `${CACHE_PREFIX}4baff4b1973138746c00f67857d9a9acd8a9796d10fdc974c174030732cbada5`;
const LOCAL_DEVELOPMENT = ['localhost', '127.0.0.1', '[::1]', '::1']
  .includes(new URL(self.registration.scope).hostname);
const APP_FILES = [
  'index.html', 'manifest.json',
  'assets/croatian.json', 'assets/conversations.json', 'assets/croatian_explanations.json',
  'pkg/bunny_to_zec_bg.wasm', 'pkg/bunny_to_zec.js',
  'icons/icon-192x192.png', 'icons/icon-512x512.png', 'icons/icon-1024x1024.png',
  'static/style.css',
];

self.addEventListener('install', event => {
  if (LOCAL_DEVELOPMENT) {
    // Also retire older local workers whose cached HTML predates the page's opt-out.
    event.waitUntil(self.skipWaiting());
    return;
  }
  event.waitUntil((async () => {
    try {
      const cache = await caches.open(CACHE_NAME);
      // Bypass the HTTP cache so a new version cannot accidentally store old assets.
      await cache.addAll(APP_FILES.map(path => new Request(
        new URL(path, self.registration.scope), { cache: 'reload' }
      )));
    } catch (error) {
      await caches.delete(CACHE_NAME);
      throw error;
    }
  })());
  // Wait for old app tabs to close; never mix their WASM/JS with a new version.
});

self.addEventListener('activate', event => {
  event.waitUntil((async () => {
    const names = await caches.keys();
    await Promise.all(names
      .filter(name => name.startsWith(CACHE_PREFIX) && (LOCAL_DEVELOPMENT || name !== CACHE_NAME))
      .map(name => caches.delete(name)));
    await self.clients.claim();
    if (LOCAL_DEVELOPMENT) await self.registration.unregister();
  })());
});

self.addEventListener('fetch', event => {
  const request = event.request;
  const url = new URL(request.url);
  const scope = new URL(self.registration.scope);
  if (LOCAL_DEVELOPMENT || request.method !== 'GET' || url.origin !== scope.origin ||
      !url.pathname.startsWith(scope.pathname) || url.pathname.includes('browser-sync')) {
    return;
  }

  event.respondWith((async () => {
    const cache = await caches.open(CACHE_NAME);
    const cached = request.mode === 'navigate'
      ? await cache.match(new URL('index.html', scope).href)
      : await cache.match(request);
    // Only the successfully installed app shell is cached. Failed requests and
    // non-navigation resources never receive an HTML fallback or replace it.
    return cached || fetch(request);
  })());
});
