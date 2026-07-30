const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const scanBtn = document.getElementById("scan-btn");
const cancelBtn = document.getElementById("cancel-btn");
const deleteBtn = document.getElementById("delete-btn");
const statusText = document.getElementById("status-text");
const progressBar = document.getElementById("progress-bar");
const iconGrid = document.getElementById("icon-grid");
const logPane = document.getElementById("log-pane");

let currentScanId = null;
let lastSummary = null;
// When set, we're drilled into one folder: { targetId, targetLabel, path, label }.
let drilldown = null;

// Mirrors sweep_core::human_bytes exactly (B/KB/MB/GB/TB/PB, one decimal).
function humanBytes(n) {
  let val = n;
  for (const unit of ["B", "KB", "MB", "GB", "TB"]) {
    if (val < 1024) return `${val.toFixed(1)} ${unit}`;
    val /= 1024;
  }
  return `${val.toFixed(1)} PB`;
}

function basename(path) {
  const parts = path.replace(/\/+$/, "").split("/");
  return parts[parts.length - 1] || path;
}

function setScanningIndicator(isScanning) {
  progressBar.classList.toggle("active", isScanning);
}

function setStatusText(text) {
  statusText.textContent = text;
}

function updateDeleteButtonState() {
  deleteBtn.disabled = iconGrid.querySelectorAll(".icon-item.selected").length === 0;
}

// `planData` is what this icon would become as a PlanItem if deleted:
// { target_id, path, expected_disk_bytes, expected_dev, expected_ino }.
function makeIconItem({ iconSrc, label, sizeText, path, onOpen, planData }) {
  const item = document.createElement("div");
  item.className = "icon-item";
  item.title = path;
  item._planData = planData;

  const img = document.createElement("img");
  img.className = "glyph";
  img.src = iconSrc;

  const labelEl = document.createElement("div");
  labelEl.className = "label";
  labelEl.textContent = label;

  const sizeEl = document.createElement("div");
  sizeEl.className = "size";
  sizeEl.textContent = sizeText;

  item.append(img, labelEl, sizeEl);
  item.addEventListener("click", () => {
    item.classList.toggle("selected");
    updateDeleteButtonState();
  });
  if (onOpen) item.addEventListener("dblclick", onOpen);
  return item;
}

function deniedBanner(summary) {
  const deniedCount = summary.results.reduce((n, t) => n + t.denied.length, 0);
  if (deniedCount === 0) return null;
  const banner = document.createElement("div");
  banner.style.gridColumn = "1 / -1";
  banner.style.fontSize = "11px";
  banner.style.padding = "4px";
  banner.textContent = `${deniedCount} location(s) couldn't be read — grant Full Disk Access to include them.`;
  return banner;
}

function truncatedNotice() {
  const notice = document.createElement("div");
  notice.style.gridColumn = "1 / -1";
  notice.style.fontSize = "11px";
  notice.style.padding = "4px";
  notice.style.color = "#555";
  notice.textContent = "Not every file could be listed here — this folder is very large.";
  return notice;
}

// Top level: one icon per deletable unit (a whole cache/DerivedData folder,
// or a loose file directly in a Children-granularity root), grouped by
// folder rather than flattening every individual file into one list.
function renderFolderLevel(summary) {
  iconGrid.innerHTML = "";
  logPane.style.display = "none";
  iconGrid.style.display = "grid";

  let allFolders = [];
  for (const target of summary.results) {
    for (const folder of target.folders) {
      allFolders.push({ ...folder, target_id: target.id, target_label: target.label });
    }
  }
  allFolders.sort((a, b) => b.disk_bytes - a.disk_bytes);

  setStatusText(
    `${allFolders.length} item${allFolders.length === 1 ? "" : "s"}    ${humanBytes(summary.total_disk_bytes)} reclaimable`
  );

  const banner = deniedBanner(summary);
  if (banner) iconGrid.appendChild(banner);

  for (const folder of allFolders) {
    const item = makeIconItem({
      iconSrc: folder.is_dir ? FOLDER_ICON_SVG : DOCUMENT_ICON_SVG,
      label: folder.label,
      sizeText: humanBytes(folder.disk_bytes),
      path: folder.path,
      onOpen: folder.is_dir ? () => openFolder(folder) : null,
      planData: {
        target_id: folder.target_id,
        path: folder.path,
        expected_disk_bytes: folder.disk_bytes,
        expected_dev: folder.dev,
        expected_ino: folder.ino,
      },
    });
    iconGrid.appendChild(item);
  }
  updateDeleteButtonState();
}

// Drill-down: show the individual files inside one selected folder. Filtered
// from the owning target's capped top-N `entries` list by path prefix — on a
// very large folder this may not include every file (the entries cap is
// shared across the whole target, not per-folder), so we say so rather than
// silently showing an incomplete list as if it were complete.
function openFolder(folder) {
  const target = lastSummary.results.find((t) => t.id === folder.target_id);
  if (!target) return;

  drilldown = { targetId: folder.target_id, targetLabel: folder.target_label, path: folder.path, label: folder.label };

  const prefix = folder.path.endsWith("/") ? folder.path : folder.path + "/";
  const files = target.entries.filter((e) => e.path === folder.path || e.path.startsWith(prefix));
  files.sort((a, b) => b.disk_bytes - a.disk_bytes);

  iconGrid.innerHTML = "";

  const header = document.createElement("div");
  header.style.gridColumn = "1 / -1";
  header.style.display = "flex";
  header.style.alignItems = "center";
  header.style.gap = "8px";
  header.style.padding = "2px 0 6px";

  const backBtn = document.createElement("button");
  backBtn.className = "sys7-btn";
  backBtn.textContent = "‹ Back";
  backBtn.addEventListener("click", closeFolder);

  const crumb = document.createElement("span");
  crumb.style.fontSize = "11px";
  // WholeRoot targets (e.g. a custom-added folder) have the folder AND the
  // target share the same name — collapse "X › X" down to just "X".
  crumb.textContent = folder.target_label === folder.label ? folder.label : `${folder.target_label} › ${folder.label}`;

  header.append(backBtn, crumb);
  iconGrid.appendChild(header);

  if (target.truncated) iconGrid.appendChild(truncatedNotice());

  if (files.length === 0) {
    const empty = document.createElement("div");
    empty.style.gridColumn = "1 / -1";
    empty.style.fontSize = "11px";
    empty.style.padding = "4px";
    empty.textContent = "No individually-listed files inside this folder (it may only contain small files below the listing threshold).";
    iconGrid.appendChild(empty);
    updateDeleteButtonState();
    return;
  }

  for (const entry of files) {
    const item = makeIconItem({
      iconSrc: iconForPath(entry.path),
      label: basename(entry.path),
      sizeText: humanBytes(entry.disk_bytes),
      path: entry.path,
      planData: {
        target_id: folder.target_id,
        path: entry.path,
        expected_disk_bytes: entry.disk_bytes,
        expected_dev: entry.dev,
        expected_ino: entry.ino,
      },
    });
    iconGrid.appendChild(item);
  }
  updateDeleteButtonState();
}

function closeFolder() {
  drilldown = null;
  if (lastSummary) renderFolderLevel(lastSummary);
}

function renderResults(summary) {
  lastSummary = summary;
  drilldown = null;
  renderFolderLevel(summary);
}

function showLog(text) {
  iconGrid.style.display = "none";
  logPane.style.display = "block";
  logPane.textContent = text;
  deleteBtn.disabled = true;
}

// Idle main-menu view: an icon per currently-selected scan target, kept live
// via TargetPicker.onChange. Never runs once a real scan has produced
// results (lastSummary set) — that view must not be clobbered by selection
// changes made afterward via the modal.
function renderIdleTargetSummary() {
  if (lastSummary) return;

  const selectedTargets = TargetPicker.getSelectedTargets();
  const n = selectedTargets.length;
  setStatusText(`${n} target(s) selected — click "Select Targets…" to change`);

  if (n === 0) {
    showLog('No targets selected. Click "Select Targets…" to choose what to scan.');
    return;
  }

  iconGrid.innerHTML = "";
  logPane.style.display = "none";
  iconGrid.style.display = "grid";

  for (const t of selectedTargets) {
    const item = document.createElement("div");
    item.className = "icon-item readonly";
    item.title = t.blurb;

    const img = document.createElement("img");
    img.className = "glyph";
    img.src = FOLDER_ICON_SVG;

    const labelEl = document.createElement("div");
    labelEl.className = "label";
    labelEl.textContent = t.label;

    item.append(img, labelEl);
    iconGrid.appendChild(item);
  }
  deleteBtn.disabled = true;
}

async function loadTargets() {
  await TargetPicker.ensureLoaded();
  renderIdleTargetSummary();
}

async function startScan() {
  const targetIds = TargetPicker.getSelectedIds();
  if (targetIds.length === 0) {
    showLog('No targets selected. Click "Select Targets…" to choose what to scan.');
    return;
  }

  scanBtn.disabled = true;
  cancelBtn.disabled = false;
  setScanningIndicator(true);
  setStatusText("Scanning...");
  showLog("Scanning...");

  currentScanId = await invoke("start_scan", { targetIds });
}

async function cancelScan() {
  if (currentScanId) {
    setStatusText("Cancelling...");
    await invoke("cancel_scan", { scanId: currentScanId });
  }
}

function friendlyFailureReason(reason) {
  if (reason.includes("not in allowlist")) return "protected — this app never deletes this location";
  if (reason.includes("Denylisted")) return "protected system location";
  return reason;
}

async function deleteSelected() {
  const selectedItems = Array.from(iconGrid.querySelectorAll(".icon-item.selected"))
    .map((el) => el._planData)
    .filter(Boolean);
  if (selectedItems.length === 0) return;

  const totalBytes = selectedItems.reduce((n, i) => n + i.expected_disk_bytes, 0);
  const count = selectedItems.length;

  SystemAlert.confirm({
    icon: CAUTION_ICON_SVG,
    body:
      `Move ${count} item${count === 1 ? "" : "s"} (${humanBytes(totalBytes)}) to the Trash?\n\n` +
      "Items stay in the Trash until you empty it. Emptying the Trash is what actually reclaims the space.",
    affirmativeLabel: "Trash",
    onConfirm: async () => {
      const plan = {
        items: selectedItems,
        permanent: false,
        created_at: { secs_since_epoch: Math.floor(Date.now() / 1000), nanos_since_epoch: 0 },
      };

      setStatusText("Moving to Trash...");
      const outcome = await invoke("execute_reclaim", { plan });

      const trashedPaths = new Set(outcome.trashed);
      if (lastSummary) {
        for (const target of lastSummary.results) {
          target.folders = target.folders.filter((f) => !trashedPaths.has(f.path));
          target.entries = target.entries.filter((e) => !trashedPaths.has(e.path));
        }
        // If a whole folder was trashed, any files drilled down under it are gone too.
        drilldown = null;
        renderFolderLevel(lastSummary);
      }

      let message = `Moved ${outcome.trashed.length} item(s) to the Trash. Empty the Trash to reclaim the space.`;
      if (outcome.failed.length > 0) {
        const reasons = outcome.failed.map(([p, r]) => `${basename(p)}: ${friendlyFailureReason(r)}`).join("; ");
        message += ` (${outcome.failed.length} couldn't be deleted — ${reasons})`;
      }
      if (outcome.skipped_stale.length > 0) {
        message += ` (${outcome.skipped_stale.length} skipped — changed since scan, re-scan to retry)`;
      }
      setStatusText(message);
    },
  });
}

listen("scan://progress", (event) => {
  const { files_seen, bytes_seen } = event.payload;
  setStatusText(`Scanning...    ${files_seen} files, ${humanBytes(bytes_seen)} seen`);
});

listen("scan://done", (event) => {
  scanBtn.disabled = false;
  cancelBtn.disabled = true;
  setScanningIndicator(false);
  renderResults(event.payload.summary);
});

scanBtn.addEventListener("click", startScan);
cancelBtn.addEventListener("click", cancelScan);
deleteBtn.addEventListener("click", deleteSelected);
TargetPicker.onChange(renderIdleTargetSummary);
loadTargets();
