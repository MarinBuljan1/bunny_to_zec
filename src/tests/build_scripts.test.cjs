const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const root = path.resolve(__dirname, '../..');

function fixture(t) {
  const parent = path.join(root, 'target');
  fs.mkdirSync(parent, { recursive: true });
  const dir = fs.mkdtempSync(path.join(parent, 'build checks '));
  // Keep generated fixtures under ignored target; no user/source files are removed.
  const bin = path.join(dir, 'bin');
  fs.mkdirSync(bin);
  t.after(() => {
    const resolved = fs.realpathSync(dir);
    assert.ok(resolved.startsWith(fs.realpathSync(parent) + path.sep));
    fs.rmSync(resolved, { recursive: true, force: true });
  });
  for (const file of ['build_and_sync.bat', 'sync_docs.bat']) {
    fs.copyFileSync(path.join(root, file), path.join(dir, file));
  }
  return { dir, bin };
}

function runBatch(dir, bin, name) {
  // Execute from a different directory to verify script-relative paths.
  return spawnSync('cmd.exe', ['/d', '/c', `call "${path.join(dir, name)}"`], {
    cwd: root,
    env: { ...process.env, PATH: `${bin};${process.env.PATH}` },
    encoding: 'utf8',
    windowsVerbatimArguments: true,
  });
}

test('Sass failure preserves its exit code and prevents a WASM build', { skip: process.platform !== 'win32' }, t => {
  const { dir, bin } = fixture(t);
  fs.writeFileSync(path.join(bin, 'sass.cmd'), '@exit /b 7\r\n');
  fs.writeFileSync(path.join(bin, 'wasm-pack.cmd'), '@echo unexpected > wasm-was-run\r\n');
  const result = runBatch(dir, bin, 'build_and_sync.bat');
  assert.equal(result.status, 7, result.stdout + result.stderr);
  assert.equal(fs.existsSync(path.join(dir, 'wasm-was-run')), false);
});

test('WASM failure preserves its exit code and prevents synchronization', { skip: process.platform !== 'win32' }, t => {
  const { dir, bin } = fixture(t);
  fs.writeFileSync(path.join(bin, 'sass.cmd'), '@exit /b 0\r\n');
  fs.writeFileSync(path.join(bin, 'wasm-pack.cmd'), '@exit /b 9\r\n');
  const result = runBatch(dir, bin, 'build_and_sync.bat');
  assert.equal(result.status, 9, result.stdout + result.stderr);
  assert.equal(fs.existsSync(path.join(dir, 'docs')), false);
});

test('missing required deployment sources fail instead of reporting a successful sync', { skip: process.platform !== 'win32' }, t => {
  const { dir, bin } = fixture(t);
  const result = runBatch(dir, bin, 'sync_docs.bat');
  assert.notEqual(result.status, 0);
  assert.match(result.stdout, /Required source file/);
});

test('cache fingerprints are stable and change for asset or worker-template changes', { skip: process.platform !== 'win32' }, t => {
  const { dir } = fixture(t);
  const site = path.join(dir, 'site');
  fs.mkdirSync(site);
  const asset = path.join(site, 'index.html');
  fs.writeFileSync(asset, 'first version');
  const template = path.join(dir, 'worker-template.js');
  fs.writeFileSync(template, 'const version = "__BUILD_VERSION__";');
  function generate() {
    const result = spawnSync('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass',
      '-File', path.join(root, 'tools/write_service_worker.ps1'),
      '-SiteDirectory', site, '-WorkerTemplate', template], { encoding: 'utf8' });
    assert.equal(result.status, 0, result.stdout + result.stderr);
    const worker = fs.readFileSync(path.join(site, 'service-worker.js'), 'utf8');
    assert.doesNotMatch(worker, /__BUILD_VERSION__/);
    return worker;
  }
  const first = generate();
  assert.equal(generate(), first);
  fs.writeFileSync(asset, 'second version');
  const second = generate();
  assert.notEqual(second, first);
  fs.appendFileSync(template, '\n// New worker policy');
  assert.notEqual(generate(), second);
});

test('a complete sync copies runtime files and removes generated source-only extras', { skip: process.platform !== 'win32' }, t => {
  const { dir, bin } = fixture(t);
  fs.mkdirSync(path.join(dir, 'tools'));
  fs.copyFileSync(path.join(root, 'tools/write_service_worker.ps1'), path.join(dir, 'tools/write_service_worker.ps1'));
  fs.copyFileSync(path.join(root, 'service-worker.js'), path.join(dir, 'service-worker.js'));
  for (const file of ['index.html', 'manifest.json']) fs.writeFileSync(path.join(dir, file), 'fixture');
  for (const folder of ['static', 'icons', 'assets', 'pkg']) fs.mkdirSync(path.join(dir, folder));
  fs.writeFileSync(path.join(dir, 'static/style.css'), 'body {}');
  fs.writeFileSync(path.join(dir, 'static/style.scss'), 'source only');
  fs.writeFileSync(path.join(dir, 'pkg/.gitignore'), '*');
  fs.writeFileSync(path.join(dir, 'pkg/README.md'), 'source documentation');
  const result = runBatch(dir, bin, 'sync_docs.bat');
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.equal(fs.readFileSync(path.join(dir, 'docs/static/style.css'), 'utf8'), 'body {}');
  assert.equal(fs.existsSync(path.join(dir, 'docs/static/style.scss')), false);
  assert.equal(fs.existsSync(path.join(dir, 'docs/pkg/.gitignore')), false);
  assert.equal(fs.existsSync(path.join(dir, 'docs/pkg/README.md')), false);
  assert.equal(fs.existsSync(path.join(dir, 'docs/.nojekyll')), true);
  assert.doesNotMatch(fs.readFileSync(path.join(dir, 'docs/service-worker.js'), 'utf8'), /__BUILD_VERSION__/);
});
