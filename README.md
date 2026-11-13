# Bunny to Zec

A client-side Yew/WebAssembly flashcard app. Edit Rust in `src/`, styles in
`static/style.scss`, and learning content in `assets/`.

Pre-generated conversation drafts are in `assets/conversations.json`. See
[CONVERSATIONS.md](CONVERSATIONS.md) for the JSON format, current coverage gaps,
validation, and authoring workflow. This is a partial draft library; the current
flashcard UI does not consume it yet.

## Rust source layout

- `lib.rs`: UI rendering, browser events, clocks, randomness, and animation timers.
- `guide.rs`: native guide dialog, Escape/backdrop closing, and focus restoration.
- `model.rs`: shared vocabulary and progress structures, retaining the existing JSON schemas.
- `learning.rs`: pure scoring, decay, progress, and card-selection rules. Time is an explicit input.
- `sentences.rs`: pure tokenization, word matching, and sentence selection. Random samples are explicit inputs.
- `storage.rs`: network loading and browser persistence, separated from learning calculations.
- `persistence.rs`: dataset validation, versioned snapshots, and legacy-data migration.
- `review_state.rs`: local review, drag, and animation state.
- `animation.rs`: pure animation ordering, input locking, and transition IDs.
- `tests/`: unit tests attached to their owning modules, plus build/cache checks.

Review words are a plain `Vec<Card>` inside the current presentation; processed
sentence tokens use `Option<String>` for their translation. These are in-memory
UI values and are not part of the persisted snapshot format.

The tokenizer is a small UTF-8 scanner. It preserves punctuation, individual spaces,
apostrophes, and the previous behavior for tabs/newlines and discarded pipe characters.
The regex crate is kept only as a development dependency: compatibility tests compare
the scanner against the original regex on the sentence corpus and generated boundary
inputs. Browser builds do not include the regex engine.

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

Browser local storage uses two snapshots with a `version` and `value` envelope:

- `learning_content_v1`: version 1; vocabulary, sentences, and explanations together. Downloads
  must have successful HTTP responses, parse correctly, and pass reference/order
  validation before a cache write. Any failed resource falls back to a whole saved
  dataset. A valid download remains usable if its cache write fails, with a warning.
- `learning_progress_v1`: version 2; per-word scores, last-review timestamps, and
  the decay-update timestamp in one write, now using `i64`. The existing key is
  retained so previous version-1 clients reject the newer format before writing.
  Readers accept version 1 or 2. A version-1 snapshot is migrated only on the next
  successful save; failed writes preserve the original bytes. Content stays at
  version 1. Timestamp units and normal scoring formulas remain unchanged.

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

The reviewed spellings `covek`, `pomoči`, and `o.k.` are merged into `čovjek`,
`pomoći`, and `okej` in the vocabulary and saved progress. If both spellings have
scores, the most recent review wins; equal review times retain the higher score.
See [CONVERSATIONS.md](CONVERSATIONS.md) for the correction manifest and content status.

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
Build and sync commands share an exclusive file handle under `target/`, so an asset
watcher cannot synchronize partially generated WASM output. A waiting command times
out after two minutes. The OS releases the handle when its process exits; the lock
file may remain and does not need to be deleted. A full build resets only the
generated root `pkg/` directory while holding the lock, preventing obsolete
crate-named files from surviving package renames; it refuses a linked output directory. Use the canonical commands rather
than running a separate `wasm-pack` process against the same output concurrently.

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

The app, guide, styles, and JSON content are served locally and precached.
The guide has no external icon-font dependency. Browser storage can be cleared or evicted, so
offline availability is not a backup of saved progress.

## Checks

- `cargo fmt --check`
- `cargo check --locked`
- `cargo test --locked`
- `cargo clippy --all-targets --all-features --locked`
- `node --test src/tests/service_worker.test.cjs src/tests/build_scripts.test.cjs`
  (offline-cache behavior and Windows build-script checks)
- `build_and_sync.bat` (Sass, WASM, deployment synchronization)

Unit tests live in `src/tests/`, attached to their owning modules so private
functions remain testable. Tests use explicit times/random samples and production
calculations; no browser-only test branch substitutes the learning formulas.
See [BROWSER_CHECKLIST.md](BROWSER_CHECKLIST.md) for browser/device regression
checks and [REFACTORING_ROADMAP.md](REFACTORING_ROADMAP.md) for implementation records.

## Review controls

Use the visible Start/next, Skip/next, Reveal / flip, Correct/next, and Incorrect/next
buttons, or swipe as before. Tab to buttons and use Enter/Space; with the card region
focused, Left/Right reveals/flips, Up advances correct, and Down advances incorrect.
Arrows inside focused explanations keep native scrolling. Controls ignore input
while loading, dragging, advancing, or showing Guide. Guide is a native modal dialog:
Escape/Close/backdrop dismiss it and focus returns to the opener. Current browsers
with HTML dialog support are required. Review buttons retain focus during animation.

## Editing learning content

The runtime reads only these UTF-8 JSON files in `assets/`:

- `croatian.json`: vocabulary object in selection order. Each Croatian key has
  `base_language` and `sentence_indexes` (ascending, zero-based references into
  the sentence array). Order determines selection ties; do not sort casually.
- `croatian_sentences.json`: array of objects with `target_language` (Croatian)
  and `base_language` (English). Reordering it requires updating all indexes.
- `croatian_explanations.json`: English base-word keys mapping Croatian forms to
  their explanation text.

Editable `croatian.xlsx`, `croatian.xlsm`, and the Krita icon source `icon.kra`
are retained. There is no repository script that exports those workbooks to the
three JSON files, so a repeatable spreadsheet export workflow cannot yet be
established from the source. Do not assume editing a workbook updates the app.
Update/verify the JSON explicitly, run `cargo test --locked` (which validates the
full dataset), then `build_and_sync.bat`. No workbook or existing JSON was changed
by the maintenance refactors. Local certificates and root `pkg/` are ignored.

## Score and timestamp ranges

Each word's score, last-review time, and the progress-update time use `i64`.
Timestamps remain Unix seconds; recall timing remains milliseconds. Existing
legacy/version-1 JSON integers are read exactly without rescaling or resetting
scores. Serialization is performed in Rust, so values are not routed through
JavaScript floating-point numbers.

Scores can grow past the old 2,147,483,647 limit. At the i64 representation limit
(9,223,372,036,854,775,807), earned-score addition saturates instead of wrapping or
panicking. Temporary selection priorities use i128 intermediates so adding a
recent-review bonus preserves their order even at that boundary. The learned
threshold and per-word contribution to displayed progress remain 864,000.

Ordinary scoring still uses the existing floating-point weights and divisor.
Negative elapsed intervals caused by a backward clock or future saved timestamp
are treated as zero: decay cannot add points, and a correct review cannot subtract
earned points. This is an explicit boundary correction, separate from the type
widening. Normal forward-clock results are covered by the existing score tests.

The version-2 progress envelope protects these wider values from older app tabs.
The migration needs no manual reset and leaves legacy progress keys intact.
