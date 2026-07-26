const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const scanBtn = document.getElementById("scan-btn");
const cancelBtn = document.getElementById("cancel-btn");
const statusText = document.getElementById("status-text");
const progressBar = document.getElementById("progress-bar");
const iconGrid = document.getElementById("icon-grid");
const logPane = document.getElementById("log-pane");

let currentScanId = null;

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

// Renders every scanned target's entries as a Finder-icon-view-style grid:
// icon, filename label, and the file's own size directly beneath it — the
// same pairing as the classic Finder/HyperCard icon view.
function renderResults(summary) {
  iconGrid.innerHTML = "";
  logPane.style.display = "none";
  iconGrid.style.display = "grid";

  let allEntries = [];
  for (const target of summary.results) {
    for (const entry of target.entries) {
      allEntries.push({ ...entry, target_label: target.label });
    }
  }
  allEntries.sort((a, b) => b.disk_bytes - a.disk_bytes);

  const itemCount = allEntries.length;
  setStatusText(
    `${itemCount} item${itemCount === 1 ? "" : "s"}    ${humanBytes(summary.total_disk_bytes)} reclaimable`
  );

  for (const entry of allEntries) {
    const item = document.createElement("div");
    item.className = "icon-item";
    item.title = entry.path;

    const img = document.createElement("img");
    img.className = "glyph";
    img.src = iconForPath(entry.path);

    const label = document.createElement("div");
    label.className = "label";
    label.textContent = basename(entry.path);

    const size = document.createElement("div");
    size.className = "size";
    size.textContent = humanBytes(entry.disk_bytes);

    item.append(img, label, size);
    item.addEventListener("click", () => item.classList.toggle("selected"));
    iconGrid.appendChild(item);
  }

  const deniedCount = summary.results.reduce((n, t) => n + t.denied.length, 0);
  if (deniedCount > 0) {
    const banner = document.createElement("div");
    banner.style.gridColumn = "1 / -1";
    banner.style.fontSize = "11px";
    banner.style.padding = "4px";
    banner.textContent = `${deniedCount} location(s) couldn't be read — grant Full Disk Access to include them.`;
    iconGrid.prepend(banner);
  }
}

function showLog(text) {
  iconGrid.style.display = "none";
  logPane.style.display = "block";
  logPane.textContent = text;
}

async function loadTargets() {
  const targets = await invoke("list_targets");
  setStatusText(`${targets.length} cleanup targets available`);
  showLog(
    "Catalog (" + targets.length + " targets):\n" +
      targets.map((t) => `  [${t.safety}] ${t.id} — ${t.label}`).join("\n")
  );
}

async function startScan() {
  scanBtn.disabled = true;
  cancelBtn.disabled = false;
  setScanningIndicator(true);
  setStatusText("Scanning...");
  showLog("Scanning...");

  const targetIds = [
    "xcode-derived-data",
    "xcode-ios-device-support",
    "homebrew-cache",
    "npm-cache",
    "cargo-registry-cache",
    "pip-cache",
    "go-build-cache",
    "gradle-caches",
    "library-logs",
    "app-caches",
  ];
  currentScanId = await invoke("start_scan", { targetIds });
}

async function cancelScan() {
  if (currentScanId) {
    setStatusText("Cancelling...");
    await invoke("cancel_scan", { scanId: currentScanId });
  }
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
loadTargets();
