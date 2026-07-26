// Custom window chrome: decorations are disabled in tauri.conf.json, so this
// draws the entire title bar (drag region, close/minimize/zoom boxes, and
// the active/inactive pinstripe state) ourselves.
(function () {
  const tauriWindow = window.__TAURI__.window;
  const appWindow = tauriWindow.getCurrentWindow();

  const titleBar = document.querySelector(".title-bar");
  const closeBox = document.querySelector(".win-box.close");
  const minimizeBox = document.querySelector(".win-box.minimize");
  const zoomBox = document.querySelector(".win-box.zoom");

  closeBox.addEventListener("click", () => appWindow.close());
  minimizeBox.addEventListener("click", () => appWindow.minimize());
  zoomBox.addEventListener("click", () => appWindow.toggleMaximize());

  function setActive(isFocused) {
    titleBar.classList.toggle("active", isFocused);
  }

  appWindow.isFocused().then(setActive);
  appWindow.onFocusChanged(({ payload: isFocused }) => setActive(isFocused));
})();
