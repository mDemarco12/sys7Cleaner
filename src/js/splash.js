// Classic Macintosh boot sequence, cosmetic and one-time only:
//   1. Plain gray dithered screen (the "gray screen" moment before anything
//      has loaded — no window, no image).
//   2. The "Welcome to Macintosh" box fades in on top of that gray screen.
//   3. It fades out, and the main window's sections build in one at a time
//      (title bar, then toolbar, then status bar, then content) instead of
//      snapping in all at once — mirroring how the real desktop drew in
//      piece by piece rather than appearing instantly.
//
// After this finishes, every element it touched has its boot-only classes
// stripped entirely, so the app behaves exactly as if this sequence had
// never run — no lingering opacity/transform rules, no leftover listeners.
(function () {
  const GRAY_SCREEN_MS = 1100; // plain gray, nothing else, before Welcome appears
  const WELCOME_VISIBLE_MS = 1600; // how long the Welcome box stays up
  const SPLASH_FADE_MS = 400; // matches .splash-overlay's CSS transition
  const SECTION_STAGGER_MS = 150; // gap between each window section revealing

  const splash = document.getElementById("splash");
  const splashBox = splash.querySelector(".splash-box");
  const sections = Array.from(document.querySelectorAll(".boot-section"));

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

  // Phase 1: gray screen only, then fade the Welcome box in.
  setTimeout(() => {
    splashBox.classList.add("visible");
  }, GRAY_SCREEN_MS);

  // Phase 2: fade the whole splash overlay out.
  setTimeout(() => {
    splash.classList.add("hidden");
  }, GRAY_SCREEN_MS + WELCOME_VISIBLE_MS);

  // Phase 3: once the splash is fully gone, remove it and stage the window in.
  setTimeout(() => {
    splash.remove();
    revealSectionsThenFinish();
  }, GRAY_SCREEN_MS + WELCOME_VISIBLE_MS + SPLASH_FADE_MS);
})();
