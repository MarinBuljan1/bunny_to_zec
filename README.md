# Bunny to Zec

A client-side Yew/WebAssembly app for learning Croatian through conversations. Edit Rust in `src/`, styles in
`static/style.scss`, and learning content in `assets/`.

Pre-generated conversation drafts are in `assets/conversations.json`. See
[CONVERSATIONS.md](CONVERSATIONS.md) for the JSON format, current coverage gaps,
validation, and authoring workflow. The chat screen uses this partial draft library;
forms absent from learner replies are deferred until coverage expands.

## Rust source layout

- `lib.rs`: app entry point and guide integration.
- `chat.rs`: conversation UI, loading/saving, clocks, and typing/feedback timers.
- `conversation_transition.rs`: inert outgoing timeline snapshots and conversation-change animations.
- `conversations.rs`: pure selection, review targets, and persistent conversation transitions.
- `gestures.rs`: one pointer controller for review swipes, history/notes scrolling, and bottom pulls.
- `scroll_drag.rs`: independent horizontal displacement, vertical scrolling, and release velocity.
- `drag_motion.rs`: frame-scheduled movement/glow updates for the active bubble.
- `scroll_motion.rs`: frame-coalesced mouse scrolling and pull-dot painting.
- `message_bubble.rs`: cancellable bubble entry/resize animations, separate from dragging.
- `word_notes.rs`: distinct tested/introduced forms and compact explanation entries.
- `guide.rs`: native guide dialog, Escape/backdrop closing, and focus restoration.
- `model.rs`: ordered vocabulary, review words, and saved progress schemas.
- `learning.rs`: pure scoring, decay, and progress calculations. Time is an explicit input.
- `sentences.rs`: shared tokenization for conversation matching and rendering.
- `storage.rs`: network loading and browser persistence, separated from learning calculations.
- `persistence.rs`: dataset validation, versioned snapshots, and legacy-data migration.
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

A partner typing bubble appears for one second before each partner message.
The partner message then appears, with three animated learner typing dots for
1.2–4 seconds, based on partner-message length. Recall timing begins when the
learner reply replaces the dots, and stops on the first reveal. Later flips do
not change that duration. Tap any message to toggle English for all messages.

Swipes can start on the conversation background as well as a message. Horizontal
swipes reveal the active reply or toggle its word explanations, including after
rating. Vertical swipes grade after revealing. A soft green/red/blue glow follows the moving learner bubble and builds toward
the acceptance threshold. The message list itself disables
native touch panning during interactive scroll/flip/review gestures, so a browser
cannot cancel pointer delivery at that scroll-container boundary. Word explanations accept sideways flips too. Short notes accept vertical ratings;
overflowing notes scroll vertically with touch or mouse dragging instead. The overflow decision is made when the gesture starts. The bubble follows the pointer directly in both directions without locking an axis, with smooth
resistance limiting its travel. Small drags track close to finger distance before
easing toward the 32 px travel limit. A single 170 ms transition settles it on release;
there is no competing rating bounce. The 280 ms rating input lock remains. Past messages remain in the current conversation.
After rating, touch dragging, mouse dragging, or the wheel scroll history.
A 72 px upward pull beyond the bottom (120 px accumulated wheel motion) requests
the next exchange. A partner-coloured dot grows below the conversation during the pull. It stops
growing at the acceptance threshold; release expands a dot-sized partner bubble
into the typing message. The preview takes no space while idle. Its space grows
and shrinks in proportion to the dot, with scroll compensation preserving the
distance from the bottom as its height changes.
Cancelling/reversing shrinks the preview. The dot announces readiness for assistive
technology, and keyboard controls provide alternatives to swiping. Replacing
the conversation slides the previous visible history upward as the new conversation
enters from below over 360 ms. The outgoing snapshot is inert, hidden from assistive
technology, and removed after the transition; it is not saved as history. Reduced
motion skips the transition, and timers/temporary DOM are cleaned up on interruption.

Drag visuals are coalesced to one `requestAnimationFrame` callback and update only
the active bubble's inline visual properties. They do not put pointer coordinates
in Yew state. Dot readiness and accessibility attributes are also painted directly;
crossing a threshold does not schedule a conversation render. The glow animates
opacity on a separate layer, offset towards the active swipe direction: blue to
the left/right for flips, green above for correct, and red below for incorrect.
`gloo-render`, already pinned in the dependency tree, is now a direct dependency
for cancellable frame callbacks. Pending frames are cancelled on release, pointer
cancellation, mode changes, and unmount; weak callback ownership avoids cycles.

Interactive scrolling and sideways flips share one pointer owner, with independent
axes. Every vertical delta scrolls the list or notes, and horizontal displacement
from the original press moves the card. Both work simultaneously during diagonal
movement. There is no scroll/flip mode switch, turn threshold, or horizontal reset
when vertical movement begins. The card settles only when released or cancelled.
Long notes use the same controller.
Wheel and keyboard scrolling remain native. Touch release adds decaying momentum,
which never triggers Next; touching again or changing modes cancels the glide.

Scroll writes and pull-dot painting are coalesced to browser frames. Reversing a
pull first shrinks its stored distance, capped at the 72 px visible threshold.
Only movement left after shrinking it to zero scrolls history backward. A pull
already in progress survives a transient bottom measurement. Horizontal movement
does not clear the dot. On release, an armed horizontal flip takes priority over
Next; one release performs only one action.
Wheel pulls use the equivalent 120 px threshold. Pointer cancellation, mode
changes, and view teardown cancel queued movement and preview frames.

Typing bubbles grow in and morph to message size over 160 ms with a brief shrink/
grow animation. Bubble size/opacity animate separately from the outer drag
transform. Info changes animate only transform/opacity, with one normal content
layout rather than interpolating width/height. Opening or closing explanations
does not force history to the bottom; automatic following remains for arriving
messages and completing a review. Reduced-motion preferences disable this animation. Obsolete animations
and resize-follow callbacks are cancelled on replacement/unmount. Animation setup
failure falls back to normally rendered content. Readers at the bottom continue
following the resized message; scrolling notes does not change history-follow state.

Timers are cancelled when their phase/view changes; queued callbacks check the
conversation and exchange identity. Repeated ratings are rejected by the pure
state machine. A resumed question starts a fresh recall clock, excluding time
away; a resumed answer keeps its already-frozen duration. Completed replies are
never graded again on reload. The guide pauses a pending typing/feedback timeout.

Conversation selection restricts candidates to forms in the learner index, then
uses the existing vocabulary priorities. Among conversations containing the target,
only those with the fewest completed reads are eligible. Their selection weight
is `target occurrences / average vocabulary rank`. The average uses every Croatian
word occurrence on both sides, lowercased with grammatical forms kept separate.
Ranks start at 1 in vocabulary order; unknown words get the last rank, while
punctuation and English do not contribute. Difficulty is computed once when the
validated library loads. Equal occurrence counts and half the average rank mean
twice the weight. This biases choices toward easier dialogue without overriding
read-count fairness or vocabulary-review priorities. A conversation's read count increases only after its last
learner reply is rated/read. No read count is awarded for starting or abandoning it.

For each learner reply, an already-introduced form below 864,000 raw score takes
priority. Otherwise the next unseen form in the reply is introduced using existing
review priorities. All previously introduced forms plus the target are blanked.
Target selection tokenizes each reply once and examines its distinct vocabulary
forms, using vocabulary positions to preserve priority ties and message order.
When every form is known, all are blanked. Replies with no vocabulary forms are
readable without grading. Scoring/decay formulas and the learning-level display
remain unchanged.

## Saved-data formats and recovery

Browser local storage uses two snapshots with a `version` and `value` envelope:

- `learning_content_v1`: version 2; ordered vocabulary and explanations together.
  Downloads must have successful HTTP responses, parse correctly, and pass content
  and conversation-index validation before a cache write. Any failed vocabulary or
  explanation resource falls back to a whole saved dataset. A valid download remains
  usable if its cache write fails, with a warning. Readers also accept version 1,
  ignoring obsolete sentence arrays/indexes. After validation, a successful atomic
  cache write migrates either downloaded or fallback content to version 2. Old clients
  reject version 2 rather than using vocabulary without their required sentence indexes.
- `learning_progress_v1`: version 4; i64 per-word scores, last-review timestamps,
  decay-update timestamp, completed conversation read counts, English visibility,
  and the current conversation cursor/masked words/completed replies in one atomic
  write. Readers accept versions 1, 2, 3, and 4; missing conversation state defaults
  to empty. Migration happens only on a successful save. Failed writes preserve
  the original snapshot. Old readers reject version 4 before writing. Version 4 adds the partner typing
  phase and the optional answer-explanation face; version-3 unfinished sessions
  retain their phase, words, scores, and completed counts. Timestamp units and
  scoring formulas remain unchanged.

The conversation JSON is fetched as a relative asset and validated against the
vocabulary and its actual learner occurrences before selection. It is covered by
the existing service-worker cache and is not duplicated in local storage. If a
content update invalidates a saved cursor, only that cursor is reset; word scores
and completed conversation counts are kept. Conversation IDs must remain stable
now that persistent counts refer to them.

The legacy `cards` and `explanations` content keys remain readable; the obsolete
`sentences` key is no longer required. Once a complete new content snapshot is saved
successfully, all three old content copies
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

The header has a circular EN toggle on the left (struck through when off), learning
progress in the centre, and Guide on the right. The progress bar fills smoothly
as the existing score approaches the next level's threshold. There is no toolbar or footer.
Start conversation is available in the welcome message; swipe across the conversation
area to reveal, rate, flip, and advance. Tab to controls and press Enter or
Space; with the conversation focused, Left/Right reveals/flips, Up rates correct
or requests the next exchange at the bottom, and Down rates incorrect. Repeated
held-arrow events are ignored. Arrows inside focused word explanations scroll.

The first reveal fills the blanks while retaining each tested word's English hint,
underline, and layout. A further flip shows compact word explanations inside the learner bubble, below
a divider after the optional English translation. Each form appears once, with
Croatian, English, and its explanation flowing in a single paragraph. Introduced
related forms remain available in vocabulary order; untranslated grammar forms
are not introduced merely by opening notes.
Rating hides this section; a later flip can reopen it without changing any scores.
Rated learner replies keep a subtle green (correct) or red (incorrect) border,
including earlier replies and restored sessions. Ungraded/read-only messages stay
neutral. The border does not change the bubble's dimensions.
Info visibility is session-local before and after rating. Extra info flips and
settling share the same immutable progress snapshot without copying scores/read
counts. Persistent actions detach the snapshot; the reducer detects that change
by identity rather than comparing all progress contents. Extra info flips do not
trigger serialization/storage writes; the first reveal still saves the frozen
recall duration. This changes in-memory ownership only, not the saved JSON format. Older saved
`Notes` answer faces remain readable and convert when closed.
Visible speaker labels and per-message rating labels are omitted. Guide remains
a native modal dialog with keyboard focus containment and Escape/backdrop closing.

## Editing learning content

The runtime reads only these UTF-8 JSON files in `assets/`:

- `croatian.json`: vocabulary object in selection order. Each Croatian key has
  `base_language` (English). Order determines selection ties; do not sort casually.
- `croatian_explanations.json`: English base-word keys mapping Croatian forms to
  their explanation text.
- `conversations.json`: partner/learner exchanges, English translations, and the
  exact-form learner occurrence index described in [CONVERSATIONS.md](CONVERSATIONS.md).

Editable `croatian.xlsx`, `croatian.xlsm`, and the Krita icon source `icon.kra`
are retained. There is no repository script that exports those workbooks to the
runtime JSON files, so a repeatable spreadsheet export workflow cannot yet be
established from the source. Do not assume editing a workbook updates the app.
Update/verify the JSON explicitly, run `cargo test --locked` (which validates the
full dataset), then `build_and_sync.bat`. The old standalone sentence file and
vocabulary sentence indexes were retired when the app switched to conversations.
Local certificates and root `pkg/` are ignored.

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
