# sys7 Cleaner — Project Map

Read this before touching code. It exists so a feature request can be routed to the 2-3 relevant files instead of re-reading the whole tree. Sections are ordered by system; each names the files, what they own, and the invariants that matter if you change them.

**Keeping this current:** there is no automated process that updates this file — no hook fires on "N requests." The practice is: update the relevant section immediately when a change adds/removes/renames a module, changes a public function signature, or changes an invariant described below. Do a deliberate full-file review every ~2 feature requests as a backstop. Do not let this drift — a stale map is worse than no map, since it will misdirect editing to code that no longer says what this file claims.

---

## System 1: `sweep-core` — the engine (`crates/sweep-core/src/`)

Pure Rust library, **zero Tauri dependency**, fully unit tested (36 tests). This is the only place filesystem safety logic lives — never duplicate a safety check in the Tauri layer or frontend.

| File | Owns |
|---|---|
| `model.rs` | All shared types: `ScanTarget`, `Safety` (enum: Regenerable/ReviewRequired/NeverTouch = Tier A/B/C), `Granularity` (WholeRoot/Children/Files), `FolderSummary`, `Entry`, `TargetResult`, `ScanSummary`, `ReclaimPlan`, `PlanItem`, `ReclaimOutcome`, `Arch`, `AppInfo`. `ScanTarget.id/label/blurb` are `&'static str` by design (catalog is static data; custom targets `Box::leak` a String to satisfy this — see System 3). |
| `catalog.rs` | `build_catalog(home) -> Vec<ScanTarget>` — the declarative Tier A/B/C target list. **Adding a cleanup target = adding one entry here**, nothing else needs to change unless it needs new scanning logic. |
| `walk.rs` | `size_tree()` — parallel (jwalk), cancellable, hardlink-deduped, device-boundary-respecting directory sizing. Sizes in **allocated blocks** (`st_blocks*512`), not `st_size` — matches Finder/du. Prunes into `node_modules` as a unit. |
| `planning.rs` | `folder_breakdown(target, cancel) -> Vec<FolderSummary>` — the **single source of truth** for "what are this target's deletable units." Used both by `run_scan` (populates `TargetResult.folders`, what the UI groups by) and by `sweep-cli`'s `Plan` command. If you change how targets are broken into deletable units, change it here only. |
| `macho.rs` | Mach-O fat/thin header parsing → `Arch` classification (Intel/AppleSilicon/Universal/Dead/Unknown). No shelling out to `lipo`/`file`. |
| `bundles.rs` | `.app` enumeration under `/Applications`, Info.plist parsing (binary or XML), `rosetta_candidates()` — Intel-only AND stale-by-last-used, never "delete this" framing. |
| `safety.rs` | `validate_deletion_path(candidate, allowlist_roots, home, root_itself_deletable) -> Result<PathBuf, SafetyViolation>`. **The single gate every deletion passes through.** Order: canonicalize → allowlist membership → denylist (home, .ssh, iCloud Drive, Keychains, etc.) → root-shallowness check. `root_itself_deletable` must be `true` only for `Granularity::WholeRoot` targets — `false` for `Children` (where the root is a shared container like `~/Library/Caches` that must never itself be deleted). Getting this flag wrong either blocks legitimate whole-folder deletes or (far worse) would need to be wrong in the *permissive* direction to cause harm, which the denylist still catches regardless. |
| `fsops.rs` | `FileOps` trait + `RealFileOps` / `DryRunFileOps` / `RecordingFileOps`. Dry-run and production share the exact same code path in `reclaim.rs` — never special-case dry-run logic anywhere else. |
| `reclaim.rs` | `execute(plan, allowlist_by_target, home, ops) -> ReclaimOutcome`. Re-validates every item from scratch (allowlist, staleness via `MAX_PLAN_AGE` = 5 min, TOCTOU via dev/ino match) — **never trusts the incoming plan**. `TargetAllowlist { roots, root_itself_deletable }` is the map value type. |
| `lib.rs` | `run_scan()` (orchestrates walk + folder_breakdown per target), `allowlist_map()` (builds the `TargetAllowlist` map — **deliberately excludes `refuse_delete` targets entirely**, so Tier C target_ids are simply unresolvable to `execute`, not just skipped upstream), `human_bytes()`. |

**Safety invariant (do not weaken):** a path is deletable *only if* it resolves (after symlink canonicalization) under a target's registered root AND that target isn't `refuse_delete` AND it's not in the hardcoded denylist. All three checks are independent layers — a bug in one must not be able to bypass another.

## System 2: `sweep-cli` (`crates/sweep-cli/src/main.rs`)

Headless CLI over the same engine: `scan`, `plan`, `apply [--dry-run]`, `targets`, all accepting `--root <dir>` to reroot the entire catalog under a sandbox. This is the fastest way to test engine changes — no Tauri rebuild/relaunch needed. Use it first when debugging engine behavior; only go to the GUI to verify the *rendering* of a result.

## System 3: Tauri shell (`src-tauri/src/`)

The IPC layer. Thin — business logic belongs in `sweep-core`, not here.

| File | Owns |
|---|---|
| `commands.rs` | `list_targets`, `add_custom_target`/`remove_custom_target` (custom folders live in `CustomTargets` managed state, separate from the static catalog — merged via `all_targets()`), `start_scan` (spawns a dedicated `std::thread`, never the async runtime — this is CPU/syscall-bound work), `cancel_scan`, `execute_reclaim` (forces `plan.permanent = false` unconditionally — **this app never wires up permanent delete from the UI**, regardless of what a plan claims). |
| `progress.rs` | `ScanProgress` (atomic counters) + `spawn_coalescer()` — emits `scan://progress` at ≤20/sec (50ms tick), never one event per file. **Note:** progress numbers are currently synthetic/simulated (ramps over ~1s); only `scan://done`'s `ScanSummary` payload is real. Wiring real per-directory progress into `walk.rs` is still open. |
| `lib.rs` | Registers plugins (`tauri-plugin-dialog` for folder picking, `tauri-plugin-opener` for external links) and all commands. |
| `capabilities/default.json` | ACL grants for window controls (close/minimize/toggle-maximize — required since `decorations:false` means the hand-drawn title bar owns these), dialog, opener, event. **Custom `#[tauri::command]` functions are NOT gated by this file** — only built-in Tauri/plugin commands are. |
| `tauri.conf.json` | `frontendDist: "../src"`, no `beforeDevCommand`/Node. `bundle.macOS.signingIdentity: "sys7 Cleaner Dev"` (self-signed, local-only — see README's Code Signing section). Window `title` and `decorations:false`. |

## System 4: Frontend (`src/`) — static, no build step, no Node

| File | Owns |
|---|---|
| `index.html` | All markup: main window, targets-picker modal, about-modal, alert-overlay. IDs are the contract JS binds to — check here first when wiring new UI. |
| `css/system7.css` | All chrome: 1-bit dither pattern (shared `.dither-bg`/`body`/`.splash-overlay` rule — the overlay needs its **own opaque fill**, not a transparent view onto body's, since it sits in front of `.mac-window`'s opaque white), title bar (`.title-bar.active` shows stripes+boxes), `.boot-section` (staged reveal), `.icon-grid`/`.icon-item` (folder/file browser), `.modal-*`, `.alert-*` (default-ring is a **separate outline outside** the button, not a thicker border — and goes on Cancel for destructive alerts, never the affirmative button). |
| `js/icons.js` | All inline-SVG glyphs: `FOLDER_ICON_SVG`, `DOCUMENT_ICON_SVG`, `CAUTION_ICON_SVG`, `SYS7_EMBLEM_SVG` (original splash mark — **do not replace with a real OS-vendor or fictional-brand logo**; see README Branding section for why). |
| `js/splash.js` | One-time boot sequence: gray screen → emblem+text box fades in → fades out → `.boot-section` elements reveal staggered → `.booted` class lands on `<body>` and all boot-only classes are stripped. Timing constants at top of file. |
| `js/chrome.js` | Wires the hand-drawn title bar's close/minimize/zoom to real `tauri::window` calls; toggles `.active` on focus change. |
| `js/alert.js` | `SystemAlert.confirm({icon, body, affirmativeLabel, onConfirm})` — the **one** reusable confirmation dialog. Any new destructive action should call this, not hand-roll a new modal. |
| `js/targets.js` | `TargetPicker` — owns the catalog checkbox state (`Set` of selected ids), custom-folder add/remove via the native picker, "Deselect All". Exposes `getSelectedIds()`/`ensureLoaded()` for `app.js`. |
| `js/about.js` | Trivial open/close + external-link interception (must route through `window.__TAURI__.opener.openUrl`, or a plain `<a target="_blank">` navigates the whole app away instead of opening the system browser). |
| `js/app.js` | Everything scan/results/delete: `renderFolderLevel()` (top-level grouped-by-folder icon grid, reads `target.folders`), `openFolder()`/`closeFolder()` (drill-down into `target.entries` filtered by path prefix — **capped at 500 entries per target total, not per-folder**, so a very large folder may show an incomplete file list; `truncatedNotice()` surfaces this), `deleteSelected()` (builds a plan from `.icon-item._planData`, confirms via `SystemAlert`, calls `execute_reclaim`, prunes deleted items from `lastSummary` and re-renders). |

**Key gotcha already hit twice:** a `<a target="_blank">` or unhandled navigation inside the webview replaces the whole app UI instead of opening a browser/window. Anything that looks like "open this externally" must go through `tauri-plugin-opener`.

## Cross-cutting invariants

- **Trash-only from the UI.** Permanent delete exists in the engine (`ReclaimPlan.permanent`) but `execute_reclaim` in `commands.rs` hard-forces it off. Don't wire a "permanent delete" UI button without deliberately revisiting that line.
- **Tier C (`refuse_delete`) targets are still scanned/measured**, just excluded from `allowlist_map` — so they appear in the results grid (informational) but any delete attempt on them fails with "not in allowlist," not a special-cased error.
- **Self-signed code signing** is wired into `tauri.conf.json`; see README for full setup/regeneration steps. Not sufficient for Gatekeeper on other machines — that needs a paid Developer ID (also documented in README).
- **De-branding**: no Apple assets/copy remain (see README's Branding section). Don't reintroduce a real OS vendor's or another franchise's logo/copy when touching the splash or chrome.

## Where things are NOT yet built (don't assume otherwise)

- Real per-directory scan progress (currently synthetic ticks — see System 3).
- Large-files explorer / `node_modules` finder panels (catalog has no `Granularity::Files` targets yet).
- Notarization (needs a paid Developer ID — self-signed cert only covers local stability).
