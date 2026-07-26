// Classic Mac boot sequence: "Welcome to Macintosh" shows for 3 seconds,
// then fades to reveal the already-loaded main window underneath.
(function () {
  const SPLASH_DURATION_MS = 3000;
  const splash = document.getElementById("splash");

  setTimeout(() => {
    splash.classList.add("hidden");
    setTimeout(() => splash.remove(), 500); // after the CSS fade completes
  }, SPLASH_DURATION_MS);
})();
