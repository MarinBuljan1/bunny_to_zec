# Bunny to Zec

A client-side Yew/WebAssembly flashcard app. Edit Rust in `src/`, styles in
`static/style.scss`, and learning content in `assets/`.

## Rust source layout

- `lib.rs`: UI rendering, browser events, clocks, randomness, and animation timers.
- `model.rs`: shared vocabulary and progress structures, retaining the existing JSON schemas.
- `learning.rs`: pure scoring, decay, progress, and card-selection rules. Time is an explicit input.
- `sentences.rs`: pure tokenization, word matching, and sentence selection. Random samples are explicit inputs.
- `storage.rs`: network loading and browser persistence, separated from learning calculations.
- `review_state.rs`: local review, drag, and animation state.
- `tests/`: unit tests attached to their owning modules, plus build/cache checks.

Persistence failures during review saving allow the in-memory session to continue;
they are reported in the browser console, and unsaved progress may be lost on reload.
The existing content-loading fallback policy remains unchanged; broader error/retry
handling is tracked under R5 in the roadmap.

## Build and deploy

Required tools: Rust with the `wasm32-unknown-unknown` target, `wasm-pack`, and the
Dart Sass CLI (`sass` on PATH). The scripts use Windows PowerShell and Robocopy.

Run `build_and_sync.bat` from any directory. It compiles Sass and WASM using the
locked Rust dependencies, then regenerates `docs/`. Required build or copy failures
return a nonzero exit code; stale output is never used as a successful fallback.
Commit and push the generated output when ready to deploy, with GitHub Pages
configured to serve `docs/`. Do not edit `docs/` or `pkg/` manually.

`sync_docs.bat` only synchronizes existing output and generates the versioned service
worker; it does not compile Sass or Rust. Use it for asset changes after a build.
The scripts preserve `.nojekyll` and omit Sass sources and source maps from deployment.

## Local development

With `cargo-watch` and BrowserSync also installed, run `watch.bat`. It performs an
initial build, starts background watchers in the same terminal, and serves the root
site with live reload. Stop the development processes when finished. Sass-only edits
compile CSS without rebuilding WASM.

Offline caching is disabled on `localhost`, `127.0.0.1`, and IPv6 loopback. The page
unregisters earlier workers for its own scope and reloads once if needed to release
an existing controller. This prevents service-worker caches from hiding local edits.
Development through another hostname or HTTPS tunnel follows the production policy.

## Offline support and updates

The generated deployment supports offline use after a successful online visit has
installed its service worker and downloaded all required app files. The first visit
must be online. Hosting must support service workers (GitHub Pages uses HTTPS).

Each sync hashes the deployed files into a cache version. Changed files produce a
changed worker; unchanged builds keep the same version. The new worker precaches a
complete app shell before it can activate. If installation fails, the previous
installed version remains available. Close **all** tabs/windows of the app, including
the installed PWA, then reopen it to let a waiting update take over. A refresh while
another old tab remains open may continue using the old version.

Only successful precaching populates the offline cache. Non-GET, cross-origin, and
development-tool requests are not cached; failed asset requests do not become HTML
responses. Cache cleanup is restricted to this app's scope. Saved learning progress
continues to use browser local storage and is not removed during cache upgrades.

The guide's Font Awesome stylesheet/fonts are externally hosted; their icons may be
unavailable offline if the browser has not retained them. The learning app and its
JSON content are precached locally. Browser storage can be cleared or evicted, so
offline availability is not a backup of saved progress.

## Checks

- `cargo fmt --check`
- `cargo check --locked`
- `cargo test --locked`
- `node --test src/tests/service_worker.test.cjs src/tests/build_scripts.test.cjs`
  (offline-cache behavior and Windows build-script checks)
- `build_and_sync.bat` (Sass, WASM, deployment synchronization)

Unit tests live in `src/tests/`. See `REFACTORING_ROADMAP.md` for planned improvements
and completed work. Editable content workbooks (`croatian.xlsx`, `croatian.xlsm`) and
the Krita icon source (`icon.kra`) are retained; their export workflow is not yet
documented. Local certificates and generated root `pkg/` output are ignored by Git.
