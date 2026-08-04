// Target picker: lets the user select/exclude which catalog items get
// scanned, and add or remove custom folders. Exposes `window.TargetPicker`
// so app.js's startScan can read the current selection without owning any
// of this state itself.
const TargetPicker = (() => {
  const { invoke } = window.__TAURI__.core;

  const modal = document.getElementById("targets-modal");
  const list = document.getElementById("targets-list");
  const targetsBtn = document.getElementById("targets-btn");
  const doneBtn = document.getElementById("targets-done-btn");
  const addFolderBtn = document.getElementById("add-folder-btn");
  const deselectAllBtn = document.getElementById("deselect-all-btn");

  makeWindowDraggable(
    modal.querySelector(".modal-window"),
    modal.querySelectorAll(".title-stripes"),
    document.querySelector(".mac-window")
  );

  const TIER_ORDER = ["Regenerable", "ReviewRequired", "NeverTouch"];
  const TIER_LABEL = {
    Regenerable: "Safe to clean",
    ReviewRequired: "Review before deleting",
    NeverTouch: "Never deleted by this app (informational only)",
  };

  let targets = [];
  let selected = new Set();
  let loaded = false;
  let onChangeCallback = null;
  let onOpenCallback = null;

  function notifyChange() {
    if (onChangeCallback) onChangeCallback();
  }

  function defaultSelection() {
    selected = new Set(targets.filter((t) => t.safety === "Regenerable").map((t) => t.id));
  }

  function render() {
    list.innerHTML = "";
    for (const tier of TIER_ORDER) {
      const group = targets.filter((t) => t.safety === tier);
      if (group.length === 0) continue;

      const label = document.createElement("div");
      label.className = "target-tier-label";
      label.textContent = TIER_LABEL[tier];
      list.appendChild(label);

      for (const t of group) {
        const row = document.createElement("div");
        row.className = "target-row" + (tier === "NeverTouch" ? " never-touch" : "");

        const checkbox = document.createElement("input");
        checkbox.type = "checkbox";
        checkbox.checked = selected.has(t.id);
        checkbox.addEventListener("change", () => {
          if (checkbox.checked) selected.add(t.id);
          else selected.delete(t.id);
          notifyChange();
        });

        const labelWrap = document.createElement("div");
        labelWrap.className = "target-label";
        const nameLine = document.createElement("div");
        nameLine.textContent = t.label;
        const blurbLine = document.createElement("span");
        blurbLine.className = "target-blurb";
        blurbLine.textContent = t.blurb;
        labelWrap.append(nameLine, blurbLine);

        row.append(checkbox, labelWrap);

        if (t.custom) {
          const removeBtn = document.createElement("button");
          removeBtn.className = "remove-btn";
          removeBtn.textContent = "Remove";
          removeBtn.addEventListener("click", async () => {
            await invoke("remove_custom_target", { id: t.id });
            selected.delete(t.id);
            await load(true);
            notifyChange();
          });
          row.appendChild(removeBtn);
        }

        list.appendChild(row);
      }
    }
  }

  async function load(force) {
    if (loaded && !force) return;
    targets = await invoke("list_targets");
    if (!loaded) defaultSelection();
    loaded = true;
    render();
  }

  async function addFolder() {
    const { open } = window.__TAURI__.dialog;
    const picked = await open({ directory: true, multiple: false, title: "Choose a Folder to Scan" });
    if (!picked) return;
    try {
      const dto = await invoke("add_custom_target", { path: picked });
      selected.add(dto.id);
      await load(true);
      notifyChange();
    } catch (err) {
      alert(`Couldn't add that folder: ${err}`);
    }
  }

  function open_() {
    if (onOpenCallback) onOpenCallback();
    load(false).then(() => {
      modal.style.display = "flex";
    });
  }

  function close() {
    modal.style.display = "none";
  }

  function deselectAll() {
    selected.clear();
    render();
    notifyChange();
  }

  targetsBtn.addEventListener("click", open_);
  doneBtn.addEventListener("click", close);
  addFolderBtn.addEventListener("click", addFolder);
  deselectAllBtn.addEventListener("click", deselectAll);

  return {
    getSelectedIds: () => Array.from(selected),
    getSelectedTargets: () => targets.filter((t) => selected.has(t.id)),
    ensureLoaded: () => load(false),
    onChange: (cb) => {
      onChangeCallback = cb;
    },
    onOpen: (cb) => {
      onOpenCallback = cb;
    },
  };
})();
