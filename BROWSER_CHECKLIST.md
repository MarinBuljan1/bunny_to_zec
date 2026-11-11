# Browser regression checks

Run `build_and_sync.bat` and serve the repository root for development. The Rust
and Node suites cover learning, persistence, interaction transitions, and cache
policy. Browser layout, native focus, pointer capture, and installed-PWA behavior
also need these checks in desktop and narrow/mobile layouts:

- Start from welcome. Start/next and vertical swipes load a question without
  rating welcome. Loading/errors block review; Try again works with keyboard.
- Reveal by swipe, Reveal / flip, or Left/Right with the card region focused.
  Recall timing stops immediately. Flip again to return to question/explanations;
  repeated input must not restart timing.
- Rate correct/up and incorrect/down, including immediately after reveal. Rapid
  input during advancement must not skip cards. Skipping an unrevealed question
  must not score it.
- Swipe with a mouse in desktop/mobile emulation and with touch on a real device.
  Release beyond the card, cancel, and try a second finger. Cancellation must not
  rate. Short explanations remain swipeable; long explanations scroll vertically.
  Focus explanations and confirm arrows scroll rather than rate.
- Tab through Guide, the card region, and review buttons. Enter/Space activates
  buttons; arrows review only when the card region itself has focus. Holding an
  arrow must not repeat reviews. Temporarily unavailable buttons retain focus.
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
