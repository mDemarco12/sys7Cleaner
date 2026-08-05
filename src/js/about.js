(function () {
  const modal = document.getElementById("about-modal");
  const aboutBtn = document.getElementById("about-btn");
  const doneBtn = document.getElementById("about-done-btn");

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
