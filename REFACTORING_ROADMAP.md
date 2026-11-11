# Refactoring roadmap

Status: R1–R7 implemented; browser smoke verification remains pending. Other tasks remain proposed except for supporting work explicitly recorded below.

This document records the ten improvements from the project review so they can be implemented incrementally in future sessions. Follow `AGENTS.md` during implementation. The objective is clearer, easier-to-modify Rust while preserving the core flashcard and learning behavior.

The review baseline is approximately 1,364 lines in `src/lib.rs`, including four active unit tests. `tests/tests.rs` contains commented-out tests. The dataset inspected contained 8,577 vocabulary entries and 9,235 sentences, with no out-of-range sentence references. These observations should be rechecked if the source changes.

## Scope and behavior preservation

- Preserve scoring formulas, decay, review timing, card-selection priorities, sentence selection, word hiding, explanations, and swipe meanings during structural refactors.
- Preserve vocabulary insertion order and tie behavior. Replacing `IndexMap` with an unordered map could change which equally scored card is selected.
- Preserve existing saved progress. Storage keys and serialized formats are persistent interfaces; any format changes need a compatible reader or migration.
- Preserve the current visual pace initially. Faster calculations do not require shorter animations.
- Keep root source files authoritative. Never manually edit `pkg` or `docs`; regenerate deployment output using the canonical build workflow.
- Treat suspected bugs separately from structural changes. Document the current behavior and agree on the intended correction before changing a learning rule.
- Avoid a framework rewrite, a general-purpose state architecture, or dependency upgrades bundled into these tasks. Add dependencies only when a concrete requirement justifies them.

## Ranking and estimated impact

The ranking balances maintainability, code reduction, runtime benefit, and effort. It is not the implementation order: behavior tests should precede risky changes.

All estimates are provisional, not benchmark results. Line reductions overlap and must not be added together. Explicit error handling and tests may increase total code while making production code simpler.

| ID | Improvement | Impact | Estimated effort | Estimated production-code reduction |
| --- | --- | --- | --- | --- |
| R1 | Simplify state ownership and transitions | Very high | 2–4 days | 80–180 Rust lines |
| R2 | Separate learning rules from Yew and browser APIs | Very high | 1–2 days | Little initially; may add lines |
| R3 | Remove score-map clones during card selection | High | Under half a day | 5–15 Rust lines |
| R4 | Unify mouse and touch handling | High | Half to one day | 40–70 Rust lines |
| R5 | Consolidate loading, caching, and errors | High | About one day | 30–60 Rust lines |
| R6 | Make animation transitions explicit and cancellable | High | 1–2 days | 20–50 Rust lines, possibly offset by lifecycle handling |
| R7 | Consolidate build and deployment scripts | High | Half to one day | Potentially 25–45% of build-script code |
| R8 | Simplify small types, wrappers, and calculations | Medium | Half to one day | 25–60 Rust lines |
| R9 | Establish behavior-preserving tests | High supporting value | 1–2 days | Adds test code |
| R10 | Clean up rendering, accessibility, styles, and repository maintenance | Medium | Low–medium; split into small tasks | Variable |

A reasonable overall target is roughly 15–25% less production Rust, approximately 180–300 lines relative to the reviewed baseline. Total Rust may remain similar once tests are added. Halving the entire app is not an expectation; halving particular duplicated sections may be realistic. Effort estimates also overlap because some tasks share preparation.

## R1 — Simplify state ownership and transitions

### Current situation

`FlashcardContext` contains 22 separate `UseStateHandle` fields. `app` initializes them individually and clones them into the context. `flashcard_view`, `calculate_swipe`, and `reset_card` then clone handles into callbacks and coordinate related updates manually.

Several values also encode state indirectly: welcome text identifies the initial screen, and equality of `time_start` and `time_end` identifies whether the answer has been revealed. Drag offsets and derived direction/intensity values are stored separately.

### Proposed approach

1. Identify three coherent responsibilities: loaded learning content, current review/progress, and drag/animation state.
2. Keep drag state in the component responsible for interaction, instead of broadcasting it through the application context.
3. Represent review phases explicitly, for example welcome, question, and answer. Keep animation phases distinct where necessary rather than creating one enum covering every possible combination.
4. Group values that must change together. Consider a small reducer for review actions if it makes transitions clearer; do not introduce one merely to replace every state setter.
5. Derive simple values from their source state. Keep expensive sentence preparation outside the frequent drag-update path.
6. Reduce or remove the broad context if focused props and local ownership become sufficient.

### Expected benefit

The main benefit is a shorter, more coherent execution path with fewer inconsistent intermediate states. Roughly 80–180 lines of bookkeeping may disappear. More local state may also avoid unnecessary rendering during dragging, but this needs browser measurement.

### Completion checklist

- [x] Review phases no longer depend on matching instructional strings or timestamp equality.
- [x] Drag updates do not require updating unrelated application state.
- [x] Related review values change through one understandable operation.
- [ ] Welcome, reveal, correct, incorrect, and guide interactions retain their behavior: transition/formula tests pass; visual browser smoke verification remains pending.
- [x] No new shared mutable containers or forwarding layers are introduced without need.

Dependencies: establish R9 coverage and preferably complete the pure-function boundaries in R2. Coordinate with R6.

### R1 implementation record — 2026-11-11

- Removed `FlashcardContext` and its provider. `App` now owns only guide visibility; `FlashcardView` owns five groups: learning content, saved progress, review, drag, and animation.
- Added `src/review_state.rs` with explicit welcome/exercise identity, question/answer phases, grouped review presentation, derived drag visuals, and settled/swiping/repositioning animation states. The reducers use Yew's normal immutable `Rc` ownership; no shared mutable containers were added.
- A presentation action replaces the card, its review words, sentence markup, and explanation markup together. Presentation preparation runs only when advancing, not during drag updates.
- Saved scores and their update timestamp change together. Progress is derived and memoized from saved scores rather than independently updated. Accumulation now uses `i64` throughout to avoid the previous intermediate `i32` overflow; a boundary test covers this supporting correctness fix.
- Kept scoring formulas, vocabulary order, current sentence eligibility and fallback behavior, local-storage keys/formats, and the 400/50/100 ms transition delays. Full timer cancellation remains R6; loading/error-policy changes remain R5; per-candidate score-map cloning remains R3.
- Added 13 tests: ten local state/gesture tests and three presentation/progress tests. The four existing formula/tokenization tests continue to pass. This is supporting coverage for R1, not completion of R9.
- Validation passed: `cargo fmt --check`, `cargo check --offline`, `cargo test --offline` (17 tests), `cargo clippy --offline --all-targets --all-features`, and `wasm-pack build --target web --out-dir target/r1-browser/pkg -- --offline`. Clippy still reports existing warnings in unchanged calculation/rendering code and the older wasm-bindgen macro. Dependencies were fetched with `cargo fetch --locked`; no dependency versions or storage schemas changed.
- No connected browser was available for visual interaction checks. Before deployment, smoke-test welcome/start, left/right reveal and flip-back, up/down advancement, guide opening/closing, explanation scrolling, and persistence after refresh.
- Browser-build output stays under ignored `target/r1-browser/pkg`; `pkg` and `docs` were not regenerated for deployment. `cargo fmt` also normalized existing Rust formatting, including the trailing newline in the commented integration-test file.
- Actual size outcome differs from the initial estimate: with both versions formatted consistently, production Rust is approximately 1,265 lines before and 1,367 after, excluding tests. The main file's production portion fell to approximately 1,083 lines, with 284 lines in the explicit state module. This step improves ownership and testability rather than delivering a net line-count reduction; added tests increase total source further. No speedup was benchmarked.

## R2 — Separate learning rules from Yew and browser APIs

### Current situation

`update_score` calculates decay, updates Yew state, reads browser time, and writes local storage. Other learning functions accept `UseStateHandle` even when they only need a collection. `calculate_raw_score` includes a `mock_last_review` switch that selects a hard-coded test timestamp.

### Proposed approach

1. Extract ordinary Rust operations for scoring, decay, progress, and next-card selection.
2. Pass borrowed domain values rather than Yew handles to read-only functions.
3. Read the clock at the application boundary and pass the current timestamp explicitly. Replace `mock_last_review` with this input so tests and production execute the same rules.
4. Have calculation functions return results; apply UI updates and persistence at clear boundaries.
5. Split the large file into a few focused modules, such as learning, sentence processing, storage/loading, and UI components. Choose the final layout based on responsibilities, not a fixed file-size target.
6. Document meaningful units, formulas, and ordering rules near their implementation.

### Expected benefit

Little immediate speed or line-count improvement is expected. The benefit is being able to understand and test a rule without tracing browser effects, making later simplification much safer.

### Completion checklist

- [x] Learning calculations can be tested without invoking JavaScript or rendering Yew.
- [x] Time-dependent rules accept an explicit timestamp.
- [x] Calculation code performs no browser storage writes.
- [x] Existing formula outputs and selection ordering are preserved by deterministic tests.
- [x] Module boundaries isolate responsibilities without adding trivial wrappers.

Dependencies: start alongside R9; this makes several subsequent tasks easier.

### R2 implementation record — 2026-11-11

- Added `model.rs` for the shared vocabulary/progress structures, `learning.rs` for pure scoring/decay/progress/selection, `sentences.rs` for pure sentence rules, and `storage.rs` for browser loading and persistence. `lib.rs` retains UI rendering, events, clocks, randomness, and timers; `review_state.rs` retains local interaction state.
- `apply_review` accepts borrowed saved progress, review words, a `ReviewAttempt`, and an explicit Unix-second timestamp. It returns new progress or `None` for a skipped review. `ReviewOutcome` replaces positional correctness booleans in production calculations, and the test-only `mock_last_review` path is gone. Production and tests now run the same calculation code.
- All score, decay, and selection calculations use one supplied timestamp per operation. Sentence selection takes a supplied random sample. Native tests no longer need special sentence indices just to avoid invoking browser time/randomness.
- Preserved the formulas, score-key strings, millisecond/second units, combined UTF-8 byte length, reviewed-word pre-decay values, ten-day intervals, insertion-order selection ties, previous-review sentence limit, ordered-prefix eligibility, first-index fallback, and first-occurrence word masking. Existing JSON schemas and browser storage keys are unchanged.
- Calculation and persistence are separate calls. Progress saving returns a typed storage error; the UI intentionally continues in memory if saving fails and logs the failure. Changes may be lost on reload. Content loading retains its existing network/cache policy and failure modes; user-visible recovery, atomic storage writes, and content validation remain R5.
- The read-only selection API also completes R3: score lookups borrow the map, so no full score-map clone occurs for each candidate. Removed the score-name allocation wrappers as part of using direct borrowed keys. No runtime speedup was benchmarked.
- All unit tests remain under `src/tests/`, attached to their owning modules. Added deterministic coverage for decay ordering/flooring, immutable inputs, skipped reviews, unseen words, incorrect multiword reviews, empty review words, zero/empty normalization, selection priorities/ties/exclusions, sentence sampling boundaries/fallbacks, and repeated words. The original eight raw-score expectations and existing normalization/tokenization tests still pass.
- Validation: 27 Rust tests and 11 Node build/cache tests pass; formatting and Cargo check pass; Clippy completes with two existing UI warnings (`repeat().take()` and redundant option matching); the WASM build succeeds. Pure learning/model/sentence modules import no browser or Yew APIs. Dependencies and storage schemas were not changed.
- Real-browser interaction verification remains pending because no browser is connected. Formula/selection tests verify business behavior, but do not replace a visual check of welcome, reveal, advance, explanations, and persistence.

## R3 — Remove unnecessary score-map cloning

### Current situation

`find_key_of_lowest_score` calls `get_score_for_key(key, scores.clone())` inside `min_by_key`. The lookup consumes a map even though it only reads one entry, so the full saved-score map is cloned for each eligible vocabulary candidate.

With about 8,577 vocabulary entries and 1,000 saved scores, selection can copy on the order of eight million score entries. Actual work depends on the excluded cards and saved-score count. This is an estimate of copying work, not a timing measurement.

`reset_card` also clones scores before `update_score` creates another clone. These are different from cloning a Yew handle, which is inexpensive and can be appropriate for callback ownership.

### Proposed approach

1. Change read-only score lookups to borrow `&HashMap<String, ScoreData>`.
2. Remove the per-candidate clone while retaining the selection algorithm and tie behavior.
3. Review the score-update path for redundant full-map copies; retain copies that are actually required for immutable state ownership.
4. Capture representative before/after selection timings with small, medium, and large saved-progress maps, using identical inputs and clock values.

### Expected benefit

This is the clearest performance opportunity. It removes work proportional to candidate count multiplied by saved-score count, leaving the candidate scan and lookups. Selection itself could become many times faster as progress grows; whole-app speed will improve less because animations and other work remain.

### Completion checklist

- [x] No full score-map clone occurs inside candidate comparison/lookup.
- [x] The same selection priorities and insertion-order ties are covered by deterministic tests.
- [x] Empty scores and exclusion of current words retain their behavior.
- [x] No measured speedup is claimed; the structural removal of per-candidate copying is documented.

Dependencies: small selection tests from R9; can be implemented early and independently.

Implemented alongside R2 because the new pure, read-only selection boundary naturally
borrows the score map. The remaining full-map copies create updated immutable state
or share a result with the delayed UI transition; they do not occur per candidate.

## R4 — Unify mouse and touch handling

### Current situation

`flashcard_view` has separate, largely duplicated mouse and touch start/move/end flows. Touch extraction uses string-based JavaScript reflection and unchecked conversions in `get_first_touch`. `set_direction` combines geometry, state mutation, previous direction, and two positional boolean switches.

### Proposed approach

1. Use pointer events for a common interaction path where supported by the target browsers.
2. Track the active pointer, capture it when appropriate, and handle cancellation and release outside the card.
3. Preserve the exception allowing long explanations to scroll rather than start a drag.
4. Extract a pure direction calculation from `set_direction`; apply its result explicitly for drag preview and completed swipes.
5. Preserve thresholds, axis comparison, front/back flip behavior, and the existing displacement/intensity formula during this refactor.
6. If any touch-specific code remains necessary, use typed browser APIs and handle missing touches without panicking.

### Expected benefit

Approximately 40–70 lines may be removed, potentially around half the duplicated handlers. Runtime gains should be small; consistent behavior and simpler maintenance are the main gains.

### Completion checklist

- [x] Mouse and touch use the common action path (pen input also uses it).
- [ ] Pointer cancellation and release outside the interaction region leave no stuck drag: state tests pass and capture is implemented; real-browser verification remains pending.
- [ ] Scrollable explanations remain usable on touch devices: measured overflow and CSS pan-y policy implemented; device verification remains pending.
- [x] Short drags, diagonal drags, flips, correct, and incorrect actions retain the existing direction rules in tests; visual interaction checks remain pending.
- [x] Multiple pointers cannot replace, move, complete, or cancel another pointer's active drag in state tests.

Dependencies: pure direction tests from R9. Coordinate state placement with R1.

### R4 implementation record — 2026-11-11

- Replaced separate touch/mouse handlers with one typed `PointerEvent` handler for down, move, up, cancellation, and lost capture. Removed JavaScript reflection and unchecked touch-list extraction; the direct `web-sys` feature list now requests `PointerEvent` instead of `TouchEvent`/`Touch`.
- Drag state and its actions carry the active pointer ID. Only an idle state accepts a new start, and other pointers cannot change or finish an active gesture. The UI accepts only primary pointers and the primary button. Capture failures leave the drag unstarted; viewport-read failure or movement without the primary button cancels safely.
- The interaction container captures accepted pointers, preserving completion outside the original card/window coordinates. Pointer-up performs the existing swipe action and clears the drag; the browser implicitly releases capture. Pointer cancellation and lost capture only clear the matching drag, without revealing/scoring/advancing a card. Direction, displacement, the axis-difference threshold, flip-back behavior, scoring, and animation timing are unchanged.
- Preserved the overflowing-explanation exception. Before a gesture, the front card measures whether its explanation actually overflows and enables `touch-action: pan-y` when needed. Measurements refresh on explanation changes and window resize; the resize listener is removed with its effect. The existing `gloo-events` dependency is now used for that listener.
- Regular cards and a background interaction surface use `touch-action: none`; the enclosing view stays compatible with native explanation scrolling. Short explanations remain swipeable. Gestures starting inside an overflowing explanation are not captured or prevented by the app. The policy follows the [Pointer Events touch-action rules](https://www.w3.org/TR/pointerevents/#the-touch-action-css-property), which must be established before pointer-down handling.
- Added four pointer-ownership/cancellation regression tests and updated the existing geometry tests. All 31 Rust tests and 11 Node build/cache tests pass. Formatting, Cargo check, Clippy, an isolated web-target WASM build, and `build_and_sync.bat` pass; generated Sass/WASM/worker output is synchronized. One pre-existing UI Clippy warning remains (`repeat().take()`). An initial packaging JSON error did not reproduce in the isolated build or the subsequent canonical build.
- No browser is connected. Before deployment, verify mouse and touch swipes, short/diagonal drags, outside-card release, cancellation/lost capture, secondary pointers/right-click, short versus overflowing explanations, and resize/orientation changes. Automated state tests do not validate actual browser capture or native scrolling.
- The original line-reduction estimate predates R1's shared swipe-completion callback. This change removes input duplication, but capture/cancellation guards and correct pre-gesture scrolling add code; no net line-count or runtime-speedup claim is made.

#### R4 regression fix — capture on the view's node reference

The user reported that desktop and emulated mobile swipes stopped working. The first
implementation captured on `event.current_target()`. In Yew 0.20 that is the delegated
subtree root, not the element whose callback appears in the markup, so subsequent
captured events could bypass the flashcard view entirely. Capture now uses a `NodeRef`
attached directly to `.flashcard-view`. See [Yew's event-delegation documentation](https://yew.rs/docs/0.20/concepts/html/events#event-delegation).

Formatting, Cargo check, and all 31 unit tests pass after the correction. The tests
cover gesture state but cannot validate Yew's DOM event routing. Real-browser swipe
verification remains pending; future pointer changes must explicitly check that
capture remains on the view rather than the delegated application root.

## R5 — Consolidate loading, caching, and error handling

### Current situation

The initialization effect repeats fetch, JSON parsing, storage, and fallback logic for three resources. JSON and storage-write failures use `expect`; HTTP status is not explicitly checked. Missing cached content becomes empty collections without a useful loading/failure state.

### Proposed approach

1. Extract one small typed loading operation for the repeated fetch/parse/cache/fallback policy. Avoid a general-purpose networking framework.
2. Check HTTP success and parsing before replacing valid cached content.
3. Fall back to valid cached content on network, HTTP, or parsing failure. Distinguish no saved value from malformed saved content when it affects recovery.
4. Allow valid downloaded content to remain usable if cache storage fails; report that offline persistence was unavailable.
5. Assemble and validate the content before publishing it to the UI, including sentence index bounds and assumptions required by selection.
6. Represent loading, ready, and failure states explicitly, with a clear recovery/retry action.
7. Load saved progress independently of successful content requests. Keep existing storage formats compatible or provide a migration.
8. Consider concurrent content requests only after the error policy is clear and measurement justifies it.

### Expected benefit

Roughly 30–60 lines may be removed, although recovery UI may offset savings. Concurrent requests could reduce startup latency, but no startup speedup has been measured. The strongest gain is avoiding crashes or blank states when external boundaries fail.

### Completion checklist

- [x] Valid saved datasets provide the offline fallback in automated tests; real-browser startup verification remains pending.
- [x] First startup without usable content produces a failed state with a Try again action; browser interaction verification remains pending.
- [x] Non-successful HTTP responses, malformed JSON, unavailable/full storage, and invalid/unordered sentence references have explicit failure paths.
- [x] Failed or invalid downloads cannot overwrite valid cached content.
- [x] Legacy progress restores and migrates without deleting old progress keys; corrupt/future records are protected from overwrite.
- [x] Superseded/unmounted loads can finish reading but cannot commit caches or update UI state.

Dependencies: R2 boundaries and R9 storage/loading coverage.

### R5 implementation record — 2026-11-11

- Consolidated resource transport/status/JSON checking into one typed fetch function. The three resources are still fetched sequentially. A complete downloaded dataset is validated before use, and any network, HTTP, parsing, or validation failure falls back to a complete valid cached dataset instead of mixing resource versions.
- Added `persistence.rs` for versioned schemas, reference/order validation, and legacy readers. Its small key/value boundary has browser and in-memory implementations so parsing, migration, atomic writes, and failure policy can be tested without a browser. The direct `gloo-storage` dependency was removed because its accessor can throw when storage is unavailable; browser access now uses fallible `web-sys::Storage` operations. No new crates were added; Cargo regenerated the lockfile.
- New snapshots use `learning_content_v1` and `learning_progress_v1`, each with `{"version":1,"value":...}`. Score fields, time units, and learning formulas are unchanged. Content snapshots hold all three resources; progress snapshots hold the scores and update time together. Each snapshot uses one atomic storage write.
- Legacy content keys remain readable. Only after a new complete snapshot has committed are the legacy `cards`, `sentences`, and `explanations` copies removed, freeing space for progress. Failed writes retain the old cache; a cleanup failure leaves the new cache usable and produces a warning. Migration can temporarily need space for both versions, so quota errors are recoverable rather than destructive.
- Legacy `card_scores` and `last_data_update` remain readable and are never deleted or rewritten. The first successful progress save creates the new snapshot. Readable scores with a missing/invalid legacy update time are retained and resume decay from now with a warning. Corrupt/unsupported progress or inaccessible storage causes a visible read-only persistence state, preserving the existing raw records. There is no automatic progress reset.
- Progress is restored once independently of network/content loading. Content retries do not reload or overwrite the current in-memory progress. A failed progress write keeps the session usable, shows a visible warning, and offers Retry saving using the latest in-memory scores. Success clears the warning.
- Content uses Loading/Ready/Failed states, a loading panel, and a Try again action on failure. Pointer-down is ignored until content is ready. Valid downloads remain usable with a notice when caching fails; saved fallback content is identified with a nonblocking notice. Error messages avoid exposing raw stored values or internal storage keys.
- Each load effect owns an `Rc<Cell<bool>>` cancellation flag. Cleanup invalidates that load; the synchronous commit boundary checks it before any cache mutation or UI result is returned. In-flight requests are not aborted, but obsolete results cannot affect the active view/cache. Tests cover that boundary.
- Added 18 persistence/loading tests, including a validation check against the actual project JSON. All 49 Rust tests and 11 Node build/cache tests pass; formatting, Cargo check, Clippy, and web-target WASM checks pass. One pre-existing UI Clippy warning remains (`repeat().take()`). The canonical build regenerates deployment output from sources.
- No browser is connected. Before deployment, check online start, offline cached start, first-load failure/retry, blocked/full storage, save failure/retry, legacy migration with real browser records, and the loading/notice layouts on desktop and mobile. Unit tests verify storage policy and parsing; they do not substitute for these UI checks.
- This adds explicit validation, recovery UI, and migration behavior beyond the original repeated loader. No net line-count reduction or startup-speedup claim is made.

## R6 — Make animation transitions explicit and cancellable

### Current situation

`reset_card` combines scoring, selection, sentence preparation, HTML generation, and nested timeouts of 400, 50, and 100 milliseconds. Answer reveal also schedules a delayed update. These callbacks capture state snapshots and can overlap when actions arrive rapidly. The manual timeout boundary does not expose cancellation ownership.

### Proposed approach

1. Separate completing a review, preparing the next review, and presenting the animation.
2. Define the allowed animation phases and which actions are accepted in each phase.
3. Preserve the current timing initially. Store cancellable timer handles or use an equally explicit lifecycle mechanism.
4. Cancel or invalidate delayed callbacks when a review is superseded or the component is removed.
5. Consider transition-completion events where they simplify coordination, with a fallback for transitions that do not emit an event.
6. Ensure old reveal callbacks cannot expose explanations on a newer card.

### Expected benefit

Potentially 20–50 fewer lines, but correct cancellation may offset this. Animation duration should remain unchanged. The benefit is reliable action ordering and a main workflow that reads clearly from top to bottom.

### Completion checklist

- [x] An accepted advance creates one phase entry for scoring and one next-card preparation; repeated swipes leave that entry unchanged in tests.
- [x] Obsolete timer/presentation IDs cannot modify a later transition or reveal a new question in tests.
- [x] One owned timeout is dropped on phase changes or unmount; no forgotten JS timer closures remain. Browser teardown verification is pending.
- [x] Answer timing stops on the first accepted horizontal swipe, using its input timestamp.
- [ ] The existing 400/50/100 ms intervals and front/back rules are covered by native tests; visual/browser timing verification remains pending.

Dependencies: R1 and R2, supported by R9 transition tests and browser verification.

### R6 implementation record — 2026-11-11

- Added `animation.rs`, a pure controller for reveal, outgoing advance, offscreen reposition, and starting the next question. It retains the existing 400/50/100 ms intervals. Each timer stage has a generation ID; elapsed events and presentation updates are accepted only for the current stage.
- Moved animation ownership into `ReviewState` so one reducer accepts a swipe, freezes recall duration, and changes the animation phase atomically. UI callbacks dispatch the action rather than scheduling nested callbacks or performing scoring directly. A rejected rapid swipe cannot alter the review timing or create another scoring phase.
- The UI has one phase-dependent effect. Entering advance calculates/persists progress once using the accepted input timestamp; entering reposition prepares the next presentation once. Changes to progress, notices, drag position, or card presentation do not rerun an unchanged phase effect. Its timeout dispatches only an ID-tagged elapsed event; stale timer callbacks perform no direct UI/storage work.
- Added `gloo-timers = "0.2"` as a direct dependency for cancellable `Timeout` handles. That version was already locked transitively; no dependency versions were updated. Phase cleanup and view teardown drop the handle, replacing manual `Closure::forget()` and nested `set_timeout` calls. Each interval is scheduled after its phase preparation so the new card retains its 50 ms offscreen step.
- Quick horizontal flip-back keeps the original pending reveal timer. Advancing during reveal replaces/cancels that timer. All new swipes are ignored while leaving, repositioning, or waiting to start the next question; input resumes after the final 100 ms pause. No-eligible-card fallback retains the old card and restarts its timer, as before.
- Removed the full score-map clone formerly needed to capture updated scores in the delayed next-card closure. The reposition phase reads the current in-memory progress instead. Scoring formulas, character-byte measurement, storage/retry policy, pointer capture on the view's `NodeRef`, direction thresholds, and explanation scrolling are unchanged.
- Updated review-state tests to exercise the production swipe/elapsed/presentation actions. Added native animation tests for stage delays, repeated input, flip-back, superseded reveal, duplicate elapsed events, and stale presentation/reveal protection. All 58 Rust tests and 11 Node build/cache tests pass; formatting, Cargo check, Clippy, and isolated web-target WASM checks pass. One existing UI Clippy warning remains (`repeat().take()`).
- The canonical `build_and_sync.bat` also passes and regenerated `docs`; generated JavaScript, WASM, styles, and the versioned worker match the final sources/build.
- No browser is connected. Before deployment, verify welcome/start, reveal/flip-back, immediate advance before reveal completion, repeated up/down swipes during transition, the 400/50/100 ms visual sequence, and view teardown. Native tests validate phase ordering and input timestamps, not actual browser timer cancellation or rendering.
- This favors explicit lifecycle and state transitions over a net line-count reduction. No runtime-speedup claim is made; CSS transition events were not added because the existing fixed intervals are retained.

## R7 — Consolidate build and deployment scripts

### Current situation

`build_and_sync.bat` builds WASM but does not compile Sass. `build_docs.bat` uses another build path, permits Sass failure and existing-output fallbacks, and deploys `service-worker.prod.js`. `sync_docs.bat` instead copies the commented-out root `service-worker.js`, and ignores errors returned by copy subroutines before reporting success. The reviewed generated worker was commented out.

### Proposed approach

1. Make `build_and_sync.bat` the canonical non-watching build, consistent with `AGENTS.md`: compile Sass, compile WASM, then synchronize deployment files.
2. Resolve paths relative to the script location, quote paths, fail fast, and preserve meaningful exit codes.
3. Have watch scripts reuse the canonical steps. Remove or clearly delegate the competing deployment entry point.
4. Fail when required tools or sources are missing rather than silently deploying stale CSS/WASM.
5. Explicitly decide which worker is used for development and production. Regenerate `docs`; do not repair generated files by hand.
6. For enabled offline support, define cache version/update behavior, GET-only handling, successful-response caching, and resource-appropriate fallback behavior.
7. Restrict cache cleanup to caches owned by this app; consider multiple apps hosted under the same origin.
8. Update deployment documentation to match the final workflow.

### Expected benefit

Potentially 25–45% less build-script code after consolidating duplicate paths, though additional error checks may reduce that saving. Build speed is unlikely to change much. The gain is reproducible output and avoiding stale or partially copied deployments.

### Completion checklist

- [x] One documented command builds current Sass and WASM and synchronizes `docs`.
- [x] The command works when invoked from another directory, including paths containing spaces (Windows integration fixtures).
- [x] Build/copy failures return a nonzero result and stop subsequent deployment steps.
- [x] Source and generated service-worker choices agree.
- [ ] Relative paths work under a GitHub Pages repository path: automated cache tests pass; browser verification remains pending.
- [ ] If offline support is enabled, verify first load, refresh, offline load, upgrade, cache invalidation, and failed-resource fallbacks: automated cache-policy tests pass; browser lifecycle verification remains pending.

Dependencies: independent of most Rust work; coordinate the loading fallback policy with R5.

### R7 and repository cleanup implementation record — 2026-11-11

- `build_and_sync.bat` is the single build command: compile Sass without source maps, build locked WASM dependencies, then synchronize deployment output. Failures propagate instead of using stale files as a successful fallback. `watch.bat` reuses the build and sync steps, runs in the existing console, and keeps Sass-only changes from rebuilding WASM.
- Removed obsolete `build_docs.bat`, `http.bat`, `watchHTTPS.bat`, the local certificate/key files, the Sass fallback helper, commented-out `tests/tests.rs`, and unused source maps. Replaced `instructions.txt` with `README.md`. The editable spreadsheets and Krita artwork remain intact; generated root `pkg` and future local certificates are ignored.
- Consolidated the two worker sources into `service-worker.js`. `tools/write_service_worker.ps1` hashes the worker template and deployed files into a cache version. Sync removes deployment-only copies of Sass sources, source maps, package README, and package `.gitignore`, and preserves `.nojekyll`. Changes in `docs` are generated by the build/sync scripts.
- The previously checked-in deployment worker was inactive. Following the user's clarification, deployment builds now enable offline app-shell caching, while loopback development explicitly unregisters workers and bypasses caching. Existing local workers receive a loopback-specific retirement path as well.
- Production updates precache a full version before activation and wait for old app tabs/windows to close. Only this app's obsolete scope-specific caches are removed. Failed installs leave the prior version intact; non-GET, cross-origin, and development-tool requests are not cached; missing assets do not receive HTML fallbacks. Local storage is unaffected.
- The external Font Awesome guide icons are not part of the offline app shell; they may be unavailable without a network connection. See the README for the offline/update policy and remaining source-asset provenance work.
- Validation: 17 Rust tests and 11 Node cache/build-script tests pass; Cargo check and formatting pass; the consolidated Sass/WASM build regenerates `docs` successfully. Tests cover space-containing paths, invocation outside the script directory, failed stages, missing required files, complete synchronization, deterministic cache fingerprints, offline navigation/assets, failed installation, scoped upgrade cleanup, request exclusions, and localhost retirement.
- No connected browser is available. Real-browser first-install, reload, upgrade activation, and offline checks remain pending; automated service-worker tests exercise the policy using an in-memory browser-API fixture rather than an actual browser.

## R8 — Simplify small types, wrappers, and calculations

### Current situation

`CurrentTranslations` only wraps a vector. `get_scorecard_name` delegates to `scorecard_name`, which just allocates a string. `ProcessingData` combines a `found` flag with a translation string. Normalization creates several intermediate vectors. `split_text` constructs and compiles a fixed regex on every call. Several APIs take owned collections, `&Vec`, or `&String` unnecessarily.

### Proposed approach

1. Remove wrappers and pass-through functions that add no domain meaning or invariant.
2. Use descriptive names such as review words and processed tokens where they clarify the domain.
3. Represent a missing translation with `Option<String>` if this removes invalid combinations without complicating rendering.
4. Prefer slices, borrowed strings, and borrowed maps for read-only operations.
5. Simplify normalization into a clear calculation with fewer intermediate allocations while preserving floating-point results within the existing tolerance.
6. Compile the fixed tokenizer once using a target-compatible mechanism, or use a small scanner if it is clearer and demonstrably equivalent. Do not replace it with whitespace splitting, which loses punctuation and spacing behavior.
7. Name repeated learning intervals and constants, and make units explicit.
8. Simplify explanation-header construction without changing which explanations are displayed.

### Expected benefit

Approximately 25–60 lines, overlapping with R2 and R3. Fewer allocations and no per-sentence regex compilation should reduce sentence-preparation work, but no numerical speedup is established.

### Completion checklist

- [ ] Removed wrappers did not encode a storage schema that is now incompatible.
- [ ] Tokenization preserves existing punctuation, apostrophe, spacing, and Unicode behavior.
- [ ] Normalization handles empty and zero-score inputs and retains expected outputs.
- [ ] Important domain concepts remain named; shorter code is still easy to understand locally.

Dependencies: R9 characterization tests; coordinate storage changes with R5.

## R9 — Establish behavior-preserving tests

### Current situation

Four active tests cover related words, tokenization, normalization, and raw scoring. The separate integration test file is commented out. Selection, decay, persistent storage recovery, and interaction transitions have limited or no direct coverage.

### Proposed approach

1. Establish the existing test baseline before changing formulas or structure. Fix test/environment issues separately from behavioral changes.
2. Add focused tests for observable rules, not private helper implementation details.
3. Use explicit timestamps and controlled random inputs where needed so tests run deterministically without browser APIs.
4. Cover score decay, timing boundaries, correct/incorrect reviews, selection ties/exclusions, sentence eligibility/fallbacks, repeated tokens, and empty inputs.
5. Cover storage parsing and any migration introduced by R5 or R8.
6. Add transition coverage for reveal, advance, rapid repeated actions, and superseded timers where the chosen boundaries allow it.
7. Check progress accumulation with enough scores to exceed an `i32` total. `update_progress` currently sums into `i32` before converting to `i64`; widening the accumulator is a separate correctness fix.
8. Remove or replace the stale commented-out integration test file.

### Behaviors to clarify before changing

- `find_related_cards` currently hides only the first eligible occurrence of a repeated word.
- Sentence eligibility currently uses the previous review's translation count as its limit, with an ordered-prefix rule and a first-index fallback.
- The no-sentence branch in `reset_card` does not refresh all the same review fields as the sentence branch; define its intended behavior before correcting it.
- Welcome/reveal timing and skipping a card before revealing it should have an explicit baseline.
- Vocabulary order determines selection ties. Explanation order currently comes from a `HashMap`; making it deterministic would be a separate presentation decision.

### Expected benefit

Tests add code and do not speed up production. They make it possible to verify that shorter implementations still follow the same learning rules. Avoid tests that merely duplicate type guarantees or implementation structure.

### Completion checklist

- [ ] Existing tests have a known pass/fail baseline.
- [ ] Key learning rules and identified edge cases are covered deterministically.
- [ ] Tests use the production calculation path rather than a test-mode branch.
- [ ] Intentional behavior changes are documented separately from refactors.

Dependencies: begin first; develop alongside R2 rather than waiting for the entire extraction.

## R10 — Clean up rendering, accessibility, styles, and repository maintenance

This is a collection of smaller follow-ups. Complete them separately when practical so review and verification remain focused.

### Rendering and interaction

- Consider rendering structured sentence/explanation values through focused components instead of storing prebuilt `Html` in state. Keep expensive preparation cached or memoized when drag updates would otherwise repeat it.
- Use semantic controls for opening the guide and provide keyboard equivalents for revealing and rating cards.
- Add dialog semantics, Escape handling, focus management, and visible focus states to the guide.
- Use stable domain keys for dynamic lists where identity matters.

### Styles

- Keep edits in `static/style.scss` and regenerate CSS through the build.
- Flatten deeply nested selectors and remove stale selectors such as references to absent UI after verifying they are unused.
- Remove commented examples and establish variables for repeated colors/spacing where useful.
- Verify narrow screens, larger text, long explanations, keyboard focus, and loading/error states.

### Dependencies and repository hygiene

- Confirm and remove unused crates/features, including apparently unused logging dependencies, using source inspection and compiler/build validation. `gloo-events` is now used by R4 for resize measurements.
- Align the package name, browser title, manifest identity, and instructions with the app's actual identity. A package rename also affects generated JS/WASM names and service-worker asset lists.
- Remove local certificates/private keys from tracked files and ignore machine-specific artifacts. Verify whether any tracked key needs replacement based on its actual use; do not assume removing a file erases Git history.
- Document how the spreadsheets relate to generated JSON, including the actual export/update workflow if it can be established.
- Replace stale deployment instructions with the final canonical workflow.
- Consider widening Unix-second timestamps currently stored as `i32`; plan compatibility with existing saved progress and preserve units. This is a correctness/schema task, not a code-shortening exercise.

### Expected benefit

Variable line reduction, with little guaranteed runtime improvement. Dependency cleanup may reduce build or bundle size, but must be measured. Accessibility, consistent identity, and accurate maintenance instructions improve usability and future work.

### Completion checklist

- [ ] Rendering changes preserve displayed content and avoid expensive work on every drag.
- [ ] Core review actions and the guide are keyboard accessible.
- [ ] Source styles and generated output remain synchronized.
- [ ] Removed dependencies are verified unused and browser builds still succeed.
- [ ] Local machine artifacts are appropriately excluded.
- [ ] Identity and maintenance instructions match the final source and build.

Dependencies: split by concern; rendering changes fit after R1/R2, build/identity work after R7, schema changes alongside R5/R9.

## Suggested implementation order

1. Establish a test baseline and characterize selection/scoring behavior: R9 with the minimum R2 extraction needed.
2. Remove the per-candidate score-map clone: R3. Measure before and after if performance numbers are wanted.
3. Finish pure learning and sentence boundaries: R2 and selected R8 simplifications.
4. Consolidate loading/storage recovery: R5, with compatibility tests.
5. Simplify state ownership and animation lifecycle: R1 and R6 in small, reviewable steps.
6. Unify pointer interaction: R4, retaining explanation scrolling and swipe semantics.
7. Consolidate build/deployment: R7. This can proceed earlier as an independent task.
8. Complete remaining small cleanup tasks: R8 and R10.

For each task, record what changed, validation performed, any measured benefit, and outstanding decisions here. Do not mark an item complete solely because code moved to another file.

## Validation and measurement

For Rust changes, follow `AGENTS.md`: run `cargo fmt`, `cargo fmt --check`, `cargo check`, and relevant `cargo test` checks. Run `cargo clippy --all-targets --all-features` for substantial Rust work and `wasm-pack build --target web` for browser-affecting changes. Before committing deployment output, run `build_and_sync.bat` and verify the generated files match their sources.

Browser verification should cover welcome/start, horizontal reveal, correct/incorrect advancement, timing, long explanation scrolling, guide opening/closing, and persistence after refresh. Add the offline/upgrade checks from R7 when offline support is enabled.

Measure production-code lines separately from tests and generated output. Measure selection separately from deliberate animation delays. Compare equivalent optimized builds with the same dataset, score-map size, timestamps, and selection inputs. Report timing distributions across repeated runs rather than a single sample; do not infer whole-app speedups from one optimized helper.

### Review-time validation limits

- `cargo fmt --check` reported formatting differences.
- `cargo test --offline` could not complete because `aho-corasick v1.1.3` was unavailable locally.
- Browser behavior, Clippy, and a fresh WASM build were not verified during the review.
- No speedup or bundle-size claim in this roadmap has been benchmarked.

## Progress log

| Item | Status | Notes / validation |
| --- | --- | --- |
| R1 | Implemented; browser smoke pending | Five local state groups replace 22 context handles; 17 tests and Rust/WASM build checks pass. See implementation record above. |
| R2 | Implemented; browser smoke pending | Pure learning/sentence rules, shared models, browser persistence boundary, explicit time/random inputs; 27 Rust tests pass. |
| R3 | Implemented with R2 | Selection borrows saved scores; priorities, insertion-order ties, and exclusions tested. No speedup benchmark claimed. |
| R4 | Implemented; browser/device smoke pending | Unified pointer events, active-pointer ownership, capture/cancellation, and overflow-aware native scrolling; 31 Rust tests pass. |
| R5 | Implemented; browser smoke pending | Validated whole-dataset fallback, loading/retry UI, atomic versioned snapshots, protected legacy progress, save warnings/retry, and cancelled-load guards; 49 Rust tests pass. |
| R6 | Implemented; browser smoke pending | Phase-owned cancellable timers, atomic swipe/reveal transitions, ID-guarded callbacks/presentations, and input locking; 58 Rust tests pass. |
| R7 | Implemented; browser smoke pending | Canonical Sass/WASM build, fail-fast sync, scoped versioned offline cache, localhost bypass, cleanup, and README; 11 Node tests pass. |
| R8 | Proposed | |
| R9 | Partially implemented | R1/R2/R4/R5/R6 added state, learning, selection, storage/migration, cache-failure, and animation-order tests. Browser lifecycle coverage remains. |
| R10 | Partially implemented | Obsolete files, HTTPS certificates, duplicate build/worker paths, stale instructions, and unnecessary deployed sources removed. Rendering/accessibility/dependency work remains. |
