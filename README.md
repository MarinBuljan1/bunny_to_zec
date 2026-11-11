# Bunny to Zec

A client-side Yew/WebAssembly flashcard app. Edit Rust in `src/`, styles in
`static/style.scss`, and learning content in `assets/`.

## Rust source layout

- `lib.rs`: UI rendering, browser events, clocks, randomness, and animation timers.
- `model.rs`: shared vocabulary and progress structures, retaining the existing JSON schemas.
- `learning.rs`: pure scoring, decay, progress, and card-selection rules. Time is an explicit input.
- `sentences.rs`: pure tokenization, word matching, and sentence selection. Random samples are explicit inputs.
- `storage.rs`: network loading and browser persistence, separated from learning calculations.
- `persistence.rs`: dataset validation, versioned snapshots, and legacy-data migration.
- `review_state.rs`: local review, drag, and animation state.
- `animation.rs`: pure animation ordering, input locking, and transition IDs.
- `tests/`: unit tests attached to their owning modules, plus build/cache checks.

Content has explicit loading, ready, and failed states. When there is no usable
network or cached dataset, a **Try again** button restarts loading. Swipes are enabled
only after validated content is ready. Saved progress is restored independently of
network success and is not reloaded on content retries.

Saving failures show a visible warning and **Retry saving** button. Learning can
continue in memory, but unsaved changes may be lost on reload. If existing progress
cannot be read or uses a newer format, it is left unchanged and the session becomes
read-only for persistence; no automatic reset or overwrite is performed.

Mouse, touch, and pen input share one pointer-event path. Only the active primary
pointer can move or finish its drag; cancellation or lost capture resets the drag
without rating the card. Capture keeps releases outside the card connected to the
original gesture. Overflowing explanations keep native vertical scrolling, while
short explanations and the card/background remain swipeable. Overflow is measured
when explanations change and on window resize so the CSS touch policy is set before
the gesture begins, as required by the [Pointer Events standard](https://www.w3.org/TR/pointerevents/#the-touch-action-css-property).

Animations retain the 400 ms reveal/outgoing delay, 50 ms offscreen reposition,
and 100 ms pause before the next recall timer starts. The first horizontal reveal
stops recall timing immediately; flipping back does not restart its reveal delay.
Advancing before explanations appear cancels that pending reveal. Further swipes
are ignored during advancement until the next question is ready, preventing a
second score update or overlapping card replacement.

One phase-owned `gloo-timers` timeout drives the current step. Phase changes or
view teardown drop the handle and cancel it; transition IDs also reject obsolete
callbacks already queued. The review reducer applies timing and swipe acceptance
together, while the UI performs scoring/persistence once on entering the advance
phase and prepares the next card on entering repositioning. Scoring uses the
accepted input timestamp, not a later timer/effect timestamp.

## Saved-data formats and recovery

Browser local storage uses two versioned snapshots, each serialized as
`{"version":1,"value":...}`:

- `learning_content_v1`: vocabulary, sentences, and explanations together. Downloads
  must have successful HTTP responses, parse correctly, and pass reference/order
  validation before a cache write. Any failed resource falls back to a whole saved
  dataset. A valid download remains usable if its cache write fails, with a warning.
- `learning_progress_v1`: scores and their Unix-second update timestamp in one write.
  Failed writes preserve the previous snapshot. Scoring formulas and timestamp units
  remain unchanged.

The legacy `cards`, `sentences`, and `explanations` content keys remain readable.
Once a complete new content snapshot is saved successfully, those old content copies
are removed to free storage space. They are retained if the new write fails.
Migration can temporarily require space for both copies; quota failures are reported
without deleting the good cache. Offline service-worker caching remains separate.

Legacy `card_scores` and `last_data_update` progress keys are read until a new progress
snapshot exists. The first successful save migrates them, leaving the old progress
keys intact. Readable legacy scores with a missing/invalid update time are preserved;
their decay resumes from the current time with a warning. Malformed scores, negative
score/timestamp values, and unsupported snapshot versions are not silently replaced.

Load cleanup invalidates pending work when the view is removed or a load is replaced.
An in-flight request may finish, but cancelled loads cannot write caches or update the
view. Requests remain sequential; this change makes no startup-speed claim.

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
