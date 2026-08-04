// Makes an internal floating window (a modal panel like the targets picker
// or About box) draggable within the app's own canvas — distinct from the
// outer OS window, which moves separately via Tauri's data-tauri-drag-region
// on the main title bar. The two mechanisms are deliberately kept off the
// same element: mixing native OS-window dragging with our own position
// tracking on one drag surface would fight over the same mousedown.
//
// `windowEl` starts CSS-centered (via its .modal-scrim parent's flexbox) and
// is left alone until the first drag, at which point its current on-screen
// position is captured into an explicit `position:absolute` + left/top —
// freezing wherever it already was (centered, or a prior drag position) with
// no visual jump. Because that inline position persists on the element even
// while its modal is hidden (display:none), a window naturally reopens
// wherever it was last left, with no separate position bookkeeping needed.
function makeWindowDraggable(windowEl, handleEls, boundsEl) {
  let startX = 0;
  let startY = 0;
  let startLeft = 0;
  let startTop = 0;

  function onMouseMove(e) {
    const bounds = boundsEl.getBoundingClientRect();
    const winRect = windowEl.getBoundingClientRect();
    const left = clamp(startLeft + (e.clientX - startX), bounds.left, bounds.right - winRect.width);
    const top = clamp(startTop + (e.clientY - startY), bounds.top, bounds.bottom - winRect.height);
    windowEl.style.left = `${left}px`;
    windowEl.style.top = `${top}px`;
  }

  function onMouseUp() {
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
  }

  function clamp(value, min, max) {
    // A window bigger than its bounds (shouldn't happen here, but be safe)
    // would otherwise flip min/max and lock the drag.
    return max < min ? min : Math.min(Math.max(value, min), max);
  }

  function onMouseDown(e) {
    if (e.button !== 0) return;
    const rect = windowEl.getBoundingClientRect();
    startX = e.clientX;
    startY = e.clientY;
    startLeft = rect.left;
    startTop = rect.top;
    windowEl.style.position = "absolute";
    windowEl.style.left = `${startLeft}px`;
    windowEl.style.top = `${startTop}px`;
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    e.preventDefault();
  }

  handleEls.forEach((el) => el.addEventListener("mousedown", onMouseDown));
}
