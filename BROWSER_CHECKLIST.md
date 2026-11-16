# Browser regression checks

Run `build_and_sync.bat` and serve the repository root for development. The Rust
and Node suites cover learning, persistence, interaction transitions, and cache
policy. Browser layout, native focus, pointer capture, and installed-PWA behavior
also need these checks in desktop and narrow/mobile layouts:

- Start from welcome. Confirm partner typing dots appear for one second before
  partner text, then learner typing dots before the masked reply. Both bubbles
  briefly grow in; dots morph to text size with a short shrink/grow transition.
  Confirm reduced-motion mode skips these transitions. Only learner
  appearance starts recall timing. Refresh during either typing phase and retry.
- Reveal using a horizontal swipe starting on the message or empty background.
  Hints, black underlines, spacing, and wrapping remain in place with blanks filled.
  Speaker labels and per-message rating labels are absent.
- Flip again to show word pairs/explanations inside the learner bubble, below
  the optional English translation and divider. Rating hides these details;
  a flip after rating reopens them without scoring again. Each form/translation
  pair appears once, in line with its explanation, without duplicate headings.
  Notes scroll with touch, mouse dragging, wheel, or keys and do not pull the surrounding history to the bottom.
- Drag up/down from outside the bubble after revealing. The current learner
  bubble should track small movements closely. Reverse direction mid-drag and
  cross the acceptance threshold repeatedly: the footer and message list must
  stay in place while the release hint changes. Check slow and fast drags on a
  real phone, and confirm the glow follows smoothly without a delayed repaint.
  Movement can change freely between sideways/up/down during one drag and settles
  once on release. Release direction decides the action; ambiguous diagonal
  releases do nothing. The glow stays behind this bubble, with green/red feedback
  for ratings and blue for flips.
  On real touch, verify both up and down gestures finish without turning into
  list scrolling, including after opening word explanations. Scroll inside long
  explanations with touch or mouse dragging. Sideways drags starting on notes
  must still flip, and vertical drags on notes that fit must still rate. Short and diagonal drags do not rate or flip. Cancellation does not rate.
- Open/close info repeatedly with sideways swipes. Content should fade/scale
  without repeatedly rewrapping while its width changes, and history should not
  jump to the bottom. Reload after opening info before rating: target, frozen
  recall duration, scores, and completed counts remain valid; details may collapse.
- Drag continuously left/right past the release threshold and keep going. The
  card must not return to centre when the release hint changes, including with
  the dot simultaneously full. Hold the pointer still beyond the threshold:
  position must remain unchanged until release or a real cancellation.
- On real touch and mouse, move the card sideways, then move vertically while
  holding the same horizontal position. The list must scroll and the card must
  keep its sideways offset. Diagonal movement must do both simultaneously.
  Reverse either axis, including one-pixel movements: neither axis may reset
  the other, and the card must settle only on release/cancellation.
- Fill the dot, pull much farther upward, then reverse 8–10 px. The dot must
  shrink a little and history must stay still. Further reversal shrinks the rest
  before scrolling history. Sideways movement must not clear the dot. If both actions are armed, release
  flips info once without advancing.
- Flick history/long notes and release: touch scrolling glides, stops at bounds,
  and can be interrupted by a tap. Momentum must never reveal the next exchange.
  Pausing before release or reversing direction must not reuse stale velocity.
- After rating, scroll history with touch, mouse dragging, or the wheel; pull
  upward at the bottom for the next exchange. Horizontal flips still work in
  this state. A dot grows in the next partner-message position and stops growing
  at the actual release threshold. Releasing expands it into the typing bubble;
  reversing/cancelling shrinks or removes it without starting a new exchange.
  Test fast mouse movement and release: the final scroll displacement must not
  be lost. Verify pull-dot movement does not cause per-pixel conversation redraws. Check that initial mouse drift does not turn a flip into scrolling.
- Test real touch, desktop mouse, and narrow/mobile emulation. Release outside
  the original message; cancel; try a second finger. Tap messages after swiping
  and verify English toggles for every message, without a swallowed first tap.
- Tab through Guide, timeline, and controls. Enter/Space activates buttons.
  Arrows in focused explanations scroll; held arrow events do not repeat actions.
- Open Guide by pointer and keyboard. Close receives initial focus; Tab/Shift+Tab
  stay inside; background controls are inert. Escape, Close, and backdrop close
  the dialog and restore focus to Guide.
- With a screen reader, masked words announce translation prompts without the
  hidden answer. The reverse reads the answer. Only the visible face is announced.
- Check 320 px width, short/landscape viewports, 200% text/zoom, long sentences,
  explanations, loading errors, and save warnings. Controls/content stay reachable.
- Rate, refresh, and confirm progress survives. Block storage/exhaust quota: show
  a warning and retain saved data. Restore storage and retry saving. Corrupt/future
  progress stays read-only.

For offline checks, use HTTPS staging below a repository path; localhost
intentionally retires service workers:

1. Visit online, wait for worker installation, close/reopen, then go offline.
   Refresh, review cards, and open Guide.
2. Publish a changed build with an old tab open. The old version remains consistent
   while the new worker waits. Close every app tab/window, including the installed
   PWA, then reopen and confirm the new version.
3. Confirm new JS/WASM filenames load, obsolete app caches disappear, and progress
   survives. Other apps' caches remain untouched.
4. Interrupt a new installation. The previous complete offline version remains
   usable. Missing JS/WASM must never receive HTML as a fallback.
5. Return to localhost and confirm local edits appear after reload without stale
   service-worker responses.

See `REFACTORING_ROADMAP.md` for automated results. These browser checks remain
pending until performed; this procedure is not a record that they passed.
