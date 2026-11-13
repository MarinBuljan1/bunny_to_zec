# Bunny to Zec

A client-side Yew/WebAssembly app for learning Croatian through conversations. Edit Rust in `src/`, styles in
`static/style.scss`, and learning content in `assets/`.

Pre-generated conversation drafts are in `assets/conversations.json`. See
[CONVERSATIONS.md](CONVERSATIONS.md) for the JSON format, current coverage gaps,
validation, and authoring workflow. The chat screen uses this partial draft library;
forms absent from learner replies are deferred until coverage expands.

## Rust source layout

- `lib.rs`: app entry point and guide integration; the previous flashcard implementation remains as a compatibility reference.
- `chat.rs`: conversation UI, loading/saving, clocks, and typing/feedback timers.
- `conversations.rs`: pure selection, review targets, and persistent conversation transitions.
- `gestures.rs`: review-message swipes and native history scrolling with a bottom pull.
- `guide.rs`: native guide dialog, Escape/backdrop closing, and focus restoration.
- `model.rs`: shared vocabulary and progress structures, retaining the existing JSON schemas.
- `learning.rs`: pure scoring, decay, progress, and card-selection rules. Time is an explicit input.
- `sentences.rs`: pure tokenization, word matching, and sentence selection. Random samples are explicit inputs.
- `storage.rs`: network loading and browser persistence, separated from learning calculations.
- `persistence.rs`: dataset validation, versioned snapshots, and legacy-data migration.
- `review_state.rs`: previous flashcard state, retained with its regression tests.
- `animation.rs`: previous flashcard animation ordering and regression tests.
- `tests/`: unit tests attached to their owning modules, plus build/cache checks.

Conversation review words are a plain `Vec<Card>` and are stored with the current
message so a reload preserves exactly what was being tested. Matching lowercases
Unicode while retaining accents and distinct grammatical forms. Every occurrence
of a selected form is blanked, but each distinct form is scored once.

The tokenizer is a small UTF-8 scanner. It preserves punctuation, individual spaces,
apostrophes, and the previous behavior for tabs/newlines and discarded pipe characters.
The regex crate is kept only as a development dependency: compatibility tests compare
the scanner against the original regex on the sentence corpus and generated boundary
inputs. Browser builds do not include the regex engine.

Content has explicit loading, ready, and failed states. When there is no usable
network or cached dataset, a **Try again** button restarts loading. Swipes are enabled
only after validated content is ready. Saved progress is restored after content validation, including an unfinished
conversation; failed loading attempts cannot write learning content or progress.

Saving failures show a visible warning and **Retry saving** button. Learning can
continue in memory, but unsaved changes may be lost on reload. If existing progress
cannot be read or uses a newer format, it is left unchanged and the session becomes
read-only for persistence; no automatic reset or overwrite is performed.

Partner messages appear first, with three animated learner typing dots for
1.2–4 seconds, based on partner-message length. Recall timing begins when the
learner reply replaces the dots, and stops on the first reveal. Later flips do
not change that duration. Tap any message to toggle English for all messages.

An active learner reply accepts horizontal swipes to flip, and vertical swipes
after revealing to grade. The bubble moves a little and settles in place over
280 ms; it never flies offscreen. Past messages remain in the current conversation.
After rating, native touch scrolling, mouse dragging, or the wheel scroll history.
A 72 px upward pull beyond the bottom (120 px accumulated wheel motion) requests
the next exchange. Buttons and keyboard controls provide alternatives. Replacing
the conversation clears the previous conversation's visible history.

Timers are cancelled when their phase/view changes; queued callbacks check the
conversation and exchange identity. Repeated ratings are rejected by the pure
state machine. A resumed question starts a fresh recall clock, excluding time
away; a resumed answer keeps its already-frozen duration. Completed replies are
never graded again on reload. The guide pauses a pending typing/feedback timeout.

Conversation selection restricts candidates to forms in the learner index, then
uses the existing vocabulary priorities. Among conversations containing the target,
only those with the fewest completed reads are eligible; actual occurrence counts
weight the random choice. A conversation's read count increases only after its last
learner reply is rated/read. No read count is awarded for starting or abandoning it.

For each learner reply, an already-introduced form below 864,000 raw score takes
priority. Otherwise the next unseen form in the reply is introduced using existing
review priorities. All previously introduced forms plus the target are blanked.
When every form is known, all are blanked. Replies with no vocabulary forms are
readable without grading. Scoring/decay formulas and the learning-level display
remain unchanged.

## Saved-data formats and recovery

Browser local storage uses two snapshots with a `version` and `value` envelope:

- `learning_content_v1`: version 1; vocabulary, sentences, and explanations together. Downloads
  must have successful HTTP responses, parse correctly, and pass reference/order
  validation before a cache write. Any failed resource falls back to a whole saved
  dataset. A valid download remains usable if its cache write fails, with a warning.
- `learning_progress_v1`: version 3; i64 per-word scores, last-review timestamps,
  decay-update timestamp, completed conversation read counts, English visibility,
  and the current conversation cursor/masked words/completed replies in one atomic
  write. Readers accept versions 1, 2, and 3; missing conversation state defaults
  to empty. Migration happens only on a successful save. Failed writes preserve
  the original snapshot. Old readers reject version 3 before writing. Content
  stays at version 1, and timestamp units/scoring formulas remain unchanged.

The conversation JSON is fetched as a relative asset and validated against the
vocabulary and its actual learner occurrences before selection. It is covered by
the existing service-worker cache and is not duplicated in local storage. If a
content update invalidates a saved cursor, only that cursor is reset; word scores
and completed conversation counts are kept. Conversation IDs must remain stable
now that persistent counts refer to them.

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
