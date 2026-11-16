const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const template = fs.readFileSync(path.join(__dirname, '../../service-worker.js'), 'utf8');

function worker({ scope = 'https://example.test/learner/', version = 'new', stores = new Map(), failInstall = false } = {}) {
  const listeners = {};
  const stats = { network: [], skippedWaiting: false, claimed: false, unregistered: false };
  const caches = {
    async open(name) {
      if (!stores.has(name)) stores.set(name, new Map());
      const entries = stores.get(name);
      return {
        async addAll(requests) {
          for (const request of requests) {
            assert.equal(request.cache, 'reload');
            entries.set(request.url, `cached:${request.url}`);
            if (failInstall) throw new Error('download failed');
          }
        },
        async match(request) { return entries.get(typeof request === 'string' ? request : request.url); },
      };
    },
    async keys() { return [...stores.keys()]; },
    async delete(name) { return stores.delete(name); },
  };
  const self = {
    registration: { scope, async unregister() { stats.unregistered = true; } },
    clients: { async claim() { stats.claimed = true; } },
    async skipWaiting() { stats.skippedWaiting = true; },
    addEventListener(name, listener) { listeners[name] = listener; },
  };
  vm.runInNewContext(template.replace('__BUILD_VERSION__', version), {
    self, caches, URL, Request,
    fetch: async request => {
      stats.network.push(request.url);
      throw new Error('offline');
    },
  });
  return {
    stores, stats,
    async dispatch(name, request) {
      let result;
      listeners[name]({ request, waitUntil(promise) { result = promise; }, respondWith(promise) { result = promise; } });
      return await result;
    },
  };
}

test('successful first install provides offline navigation and assets below a repository path', async () => {
  const app = worker();
  await app.dispatch('install');
  await app.dispatch('activate');
  assert.equal(app.stats.skippedWaiting, false);
  assert.equal(app.stats.claimed, true);
  assert.match(await app.dispatch('fetch', { method: 'GET', mode: 'navigate', url: 'https://example.test/learner/' }), /index\.html$/);
  assert.match(await app.dispatch('fetch', { method: 'GET', url: 'https://example.test/learner/pkg/bunny_to_zec_bg.wasm' }), /\.wasm$/);
  for (const file of ['croatian.json', 'croatian_explanations.json', 'conversations.json']) {
    const url = `https://example.test/learner/assets/${file}`;
    assert.equal(await app.dispatch('fetch', { method: 'GET', url }), `cached:${url}`);
  }
  assert.equal([...app.stores.values()][0].has('https://example.test/learner/assets/croatian_sentences.json'), false);
  assert.equal(app.stats.network.length, 0);
});

test('failed installation discards the partial new cache and preserves the old one', async () => {
  const oldName = 'language-learner:https://example.test/learner/:old';
  const stores = new Map([[oldName, new Map([['saved', 'old content']])]]);
  const app = worker({ stores, failInstall: true });
  await assert.rejects(app.dispatch('install'), /download failed/);
  assert.deepEqual([...stores.keys()], [oldName]);
  assert.equal(stores.get(oldName).get('saved'), 'old content');
});

test('upgrade cleanup only deletes obsolete caches for the same application scope', async () => {
  const stores = new Map([
    ['language-learner:https://example.test/learner/:old', new Map()],
    ['language-learner:https://example.test/other/:old', new Map()],
    ['another-app', new Map()],
  ]);
  const app = worker({ stores });
  await app.dispatch('install');
  await app.dispatch('activate');
  assert.equal(stores.has('language-learner:https://example.test/learner/:old'), false);
  assert.equal(stores.has('language-learner:https://example.test/other/:old'), true);
  assert.equal(stores.has('another-app'), true);
});

test('non-GET, cross-origin, out-of-scope and development-tool requests are untouched', async () => {
  const app = worker();
  for (const request of [
    { method: 'POST', url: 'https://example.test/learner/action' },
    { method: 'GET', url: 'https://external.test/style.css' },
    { method: 'GET', url: 'https://example.test/other/' },
    { method: 'GET', url: 'https://example.test/learner/browser-sync/socket.io' },
  ]) assert.equal(await app.dispatch('fetch', request), undefined);
  assert.equal(app.stores.size, 0);
});

test('missing assets fail as assets instead of receiving cached HTML or poisoning the cache', async () => {
  const app = worker();
  await app.dispatch('install');
  const before = [...app.stores.values()][0].size;
  await assert.rejects(app.dispatch('fetch', { method: 'GET', url: 'https://example.test/learner/missing.js' }), /offline/);
  assert.equal([...app.stores.values()][0].size, before);
});

test('localhost retires existing workers and never caches or intercepts development requests', async () => {
  const app = worker({ scope: 'http://localhost:3000/' });
  await app.dispatch('install');
  await app.dispatch('activate');
  assert.equal(app.stats.skippedWaiting, true);
  assert.equal(app.stats.unregistered, true);
  assert.equal(await app.dispatch('fetch', { method: 'GET', url: 'http://localhost:3000/pkg/bunny_to_zec.js' }), undefined);
  assert.equal(app.stores.size, 0);
});
