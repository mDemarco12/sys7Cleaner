(function () {
  const modal = document.getElementById("about-modal");
  const aboutBtn = document.getElementById("about-btn");
  const doneBtn = document.getElementById("about-done-btn");

  // Original chibi mascot — one of exactly two documented exceptions to the
  // app's strict 1-bit rule (see system7.css's header comment). Rendered
  // once at load, not per-open: it's a static <svg> that idle-bobs/blinks
  // via CSS the whole time it's in the DOM, whether the modal is visible or
  // not, so there's nothing to (re)trigger on each open.
  document.getElementById("about-sprite").innerHTML = SYS7_MASCOT_SVG;

  makeWindowDraggable(
    modal.querySelector(".modal-window"),
    modal.querySelectorAll(".title-stripes"),
    document.querySelector(".mac-window")
  );

  aboutBtn.addEventListener("click", () => {
    modal.style.display = "flex";
  });
  doneBtn.addEventListener("click", () => {
    modal.style.display = "none";
  });

  // Clicking a normal <a href target="_blank"> inside a Tauri webview
  // navigates the whole app away to that URL instead of opening it in the
  // system browser. Intercept and route through the opener plugin instead.
  modal.querySelectorAll("a[href]").forEach((link) => {
    link.addEventListener("click", (e) => {
      e.preventDefault();
      window.__TAURI__.opener.openUrl(link.href);
    });
  });
})();
