// CRT-power-on-flavored boot sequence, cosmetic and one-time only:
//   1. Plain gray dithered screen (the "gray screen" moment before anything
//      has loaded — no window, no image).
//   2. The Welcome box unfolds from a flat scanline with a one-frame static
//      flash (see `--dither-pattern` / `.splash-box.power-on` in the CSS).
//   3. A brief chromatic-aberration glitch on the emblem and text — the
//      sequence's one deliberate, bounded exception to the app's strict
//      1-bit rule, confined to this overlay and never reaching the window
//      chrome (see the file-header comment in system7.css).
//   4. The Welcome text types itself out with a blinking cursor.
//   5. One more glitch pulse and a scanline sweep, then the whole overlay
//      fades out, and the main window's sections build in one at a time
//      (title bar, then toolbar, then status bar, then content) instead of
//      snapping in all at once.
//
// Total time budget is pinned at TOTAL_MS regardless of how the beats above
// are repacked — this is a utility people open to do one quick task, so
// boot must never get slower than it already was, only better-looking
// within the same time.
//
// After this finishes, every element it touched has its boot-only classes
// stripped entirely, so the app behaves exactly as if this sequence had
// never run — no lingering opacity/transform rules, no leftover listeners.
(function () {
  const TOTAL_MS = 3100; // hard budget for the whole sequence, gray screen through fade-out
  const GRAY_SCREEN_MS = 500; // plain gray, nothing else, before the box appears
  const POWER_ON_MS = 180; // matches .splash-unfold's animation-duration in CSS
  const GLITCH_MS = 320; // matches .splash-glitch-r/-c/.splash-text-glitch's animation-duration
  const CHAR_MS = 38; // typewriter speed, ms per revealed character
  const HOLD_FLOOR_MS = 300; // minimum pause after typing finishes, before fade-out
  const FADE_MS = 400; // matches .splash-overlay's CSS transition
  const SECTION_STAGGER_MS = 150; // gap between each window section revealing

  const splash = document.getElementById("splash");
  const splashBox = splash.querySelector(".splash-box");
  const typeSpan = splash.querySelector(".splash-type");
  const cursor = splash.querySelector(".splash-cursor");
  const scanline = splash.querySelector(".splash-scanline");
  const sections = Array.from(document.querySelectorAll(".boot-section"));

  splash.querySelectorAll(".splash-logo").forEach((img) => {
    img.src = SYS7_EMBLEM_SVG;
  });

  // Toggles `.glitching` on and back off so the CSS animation is free to
  // replay on the next call — re-adding a class that's already present
  // doesn't restart a CSS animation, so this can't just be a one-way add.
  function glitchPulse() {
    splashBox.classList.add("glitching");
    setTimeout(() => splashBox.classList.remove("glitching"), GLITCH_MS);
  }

  // Reads the full string once (the markup is the source of truth for it,
  // so the text is still present even if this script never runs), clears
  // it, then reveals it one character at a time. `.splash-text`'s width is
  // pinned to its current rendered size FIRST — it's `white-space: nowrap`,
  // so an empty-then-filling span would otherwise widen the whole dialog
  // box character by character. Safe to measure synchronously: the project
  // has no @font-face (fonts are system-resolved "Geneva"), so there's no
  // webfont swap that could invalidate this measurement after the fact.
  function typeText(onDone) {
    const fullText = typeSpan.textContent;
    const textEl = splash.querySelector(".splash-text");
    textEl.style.minWidth = `${textEl.getBoundingClientRect().width}px`;
    typeSpan.textContent = "";
    cursor.classList.add("active");

    let i = 0;
    const tick = () => {
      i += 1;
      typeSpan.textContent = fullText.slice(0, i);
      if (i < fullText.length) {
        setTimeout(tick, CHAR_MS);
      } else if (onDone) {
        onDone();
      }
    };
    setTimeout(tick, CHAR_MS);
  }

  function revealSectionsThenFinish() {
    sections.forEach((el, i) => {
      setTimeout(() => el.classList.add("revealed"), i * SECTION_STAGGER_MS);
    });
    const totalRevealTime = sections.length * SECTION_STAGGER_MS + 300;
    setTimeout(() => {
      document.body.classList.add("booted");
      sections.forEach((el) => el.classList.remove("boot-section", "revealed"));
    }, totalRevealTime);
  }

  // Phase 1 (t=0): gray screen only — nothing to schedule, the box starts
  // at opacity:0 and everything else starts inert.

  // Phase 2 (t=GRAY_SCREEN_MS): power on — box fades/unfolds in with a
  // static flash.
  setTimeout(() => {
    splashBox.classList.add("visible", "power-on");
  }, GRAY_SCREEN_MS);

  // Phase 3: glitch-in on the emblem and text, right as the unfold lands.
  const glitchInAt = GRAY_SCREEN_MS + POWER_ON_MS;
  setTimeout(glitchPulse, glitchInAt);

  // Phase 4: typewriter, once the glitch-in settles. The welcome string is
  // static markup (read by typeText, not passed in), so its length — and
  // therefore every downstream phase boundary — is knowable up front rather
  // than only once typing actually starts.
  const typingAt = glitchInAt + GLITCH_MS;
  const typingMs = typeSpan.textContent.length * CHAR_MS;
  const holdAt = typingAt + typingMs;

  setTimeout(() => {
    typeText(() => {
      // Phase 5: hold — one more glitch pulse plus a scanline sweep, timed
      // to land inside whatever hold window is left before the fade.
      scanline.classList.add("sweeping");
      setTimeout(glitchPulse, 500);
    });
  }, typingAt);

  // Phase 6: fade the whole overlay out. Scheduled from TOTAL_MS rather than
  // stacking phase durations, so the budget stays pinned even if the welcome
  // string's length ever changes — with a floor so an unusually long string
  // still gets a visible pause before fading rather than cutting the hold to
  // zero (in that one edge case, total time is allowed to drift past
  // TOTAL_MS rather than truncating the hold to nothing).
  const fadeAt = Math.max(TOTAL_MS - FADE_MS, holdAt + HOLD_FLOOR_MS);

  setTimeout(() => {
    splash.classList.add("hidden");
  }, fadeAt);

  // Phase 7: once the splash is fully gone, remove it and stage the window in.
  setTimeout(() => {
    splash.remove();
    revealSectionsThenFinish();
  }, fadeAt + FADE_MS);
})();
