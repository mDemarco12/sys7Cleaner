// Reusable System 7-style confirmation alert: {icon, body, affirmativeLabel,
// onConfirm}. Default focus/ring is on Cancel for every destructive alert —
// Return/Enter and Esc both cancel, so trashing anything requires a
// deliberate click on the affirmative button.
const SystemAlert = (() => {
  const overlay = document.getElementById("alert-overlay");
  const iconEl = document.getElementById("alert-icon");
  const bodyEl = document.getElementById("alert-body");
  const affirmativeBtn = document.getElementById("alert-affirmative-btn");
  const cancelBtn = document.getElementById("alert-cancel-btn");

  let onConfirmCallback = null;

  function close() {
    overlay.style.display = "none";
    document.removeEventListener("keydown", onKeyDown);
    onConfirmCallback = null;
  }

  function onKeyDown(e) {
    if (e.key === "Escape" || e.key === "Enter") {
      e.preventDefault();
      close();
    }
  }

  affirmativeBtn.addEventListener("click", () => {
    const cb = onConfirmCallback;
    close();
    if (cb) cb();
  });
  cancelBtn.addEventListener("click", close);

  function confirm({ icon, body, affirmativeLabel, onConfirm }) {
    iconEl.src = icon || CAUTION_ICON_SVG;
    bodyEl.textContent = body;
    affirmativeBtn.textContent = affirmativeLabel || "OK";
    onConfirmCallback = onConfirm;
    overlay.style.display = "flex";
    document.addEventListener("keydown", onKeyDown);
    cancelBtn.focus();
  }

  return { confirm };
})();
