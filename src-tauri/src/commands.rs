use crate::progress::{spawn_coalescer, ScanProgress};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use sweep_core::model::{Granularity, ReclaimOutcome, ReclaimPlan, Safety, ScanTarget};
use tauri::{AppHandle, Emitter, State};

/// Per-`scan_id` cancellation flags, checked by the scan thread. Kept in
/// Tauri-managed state rather than a global so multiple scans (in principle)
/// never share a flag.
#[derive(Default)]
pub struct ScanRegistry {
    pub cancel_flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

/// User-added scan targets (arbitrary folders picked via the native Open
/// panel), layered on top of the built-in static catalog rather than mixed
/// into it — `sweep_core::catalog` stays a pure, fully-tested static list.
/// Custom targets are always `Safety::ReviewRequired`: an arbitrary
/// user-picked folder gets no default-selected bulk-delete trust.
#[derive(Default)]
pub struct CustomTargets {
    pub targets: Mutex<Vec<ScanTarget>>,
}

#[derive(Clone, Serialize)]
pub struct TargetDto {
    pub id: String,
    pub label: String,
    pub safety: Safety,
    pub granularity: Granularity,
    pub blurb: String,
    pub refuse_delete: bool,
    pub custom: bool,
}

#[derive(Clone, Serialize)]
struct DoneEvent {
    scan_id: String,
    summary: sweep_core::model::ScanSummary,
}

fn home_dir() -> std::path::PathBuf {
    std::env::var_os("HOME").map(std::path::PathBuf::from).expect("HOME must be set")
}

fn to_dto(t: &ScanTarget, custom: bool) -> TargetDto {
    TargetDto {
        id: t.id.to_string(),
        label: t.label.to_string(),
        safety: t.safety,
        granularity: t.granularity,
        blurb: t.blurb.to_string(),
        refuse_delete: t.refuse_delete,
        custom,
    }
}

/// Built-in catalog plus every user-added custom target, in that order.
fn all_targets(custom: &State<'_, CustomTargets>) -> Vec<ScanTarget> {
    let mut targets = sweep_core::catalog::build_catalog(&home_dir());
    targets.extend(custom.targets.lock().unwrap().iter().cloned());
    targets
}

/// Returns the full cleanup target catalog (built-in + custom). Pure data,
/// no scanning — safe to call synchronously from the async command context.
#[tauri::command]
pub fn list_targets(custom: State<'_, CustomTargets>) -> Vec<TargetDto> {
    let custom_ids: std::collections::HashSet<String> =
        custom.targets.lock().unwrap().iter().map(|t| t.id.to_string()).collect();
    all_targets(&custom)
        .iter()
        .map(|t| to_dto(t, custom_ids.contains(t.id)))
        .collect()
}

/// Registers a user-picked folder as a new scan target. The folder must
/// exist and be a directory — this only registers it for scanning; deletion
/// still goes through the same allowlist/denylist guard as everything else,
/// and a custom target's own root is its only allowlisted path.
#[tauri::command]
pub fn add_custom_target(custom: State<'_, CustomTargets>, path: String) -> Result<TargetDto, String> {
    let root = PathBuf::from(&path);
    let canonical = root.canonicalize().map_err(|e| format!("can't access '{path}': {e}"))?;
    if !canonical.is_dir() {
        return Err(format!("'{path}' is not a directory"));
    }

    let label = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    let id = format!("custom-{}", next_custom_id());

    // ScanTarget's id/label/blurb are &'static str by design (the built-in
    // catalog is fully static data) — leaking is the standard way to turn a
    // runtime String into one, and is safe here: custom targets live in
    // managed state for the app's lifetime anyway, so nothing is wasted that
    // wouldn't already be held for as long as the process runs.
    let id_static: &'static str = Box::leak(id.into_boxed_str());
    let label_static: &'static str = Box::leak(label.into_boxed_str());

    let target = ScanTarget {
        id: id_static,
        label: label_static,
        roots: vec![canonical],
        safety: Safety::ReviewRequired,
        granularity: Granularity::WholeRoot,
        blurb: "User-added folder — reviewed individually, never bulk-selected.",
        refuse_delete: false,
    };

    let dto = to_dto(&target, true);
    custom.targets.lock().unwrap().push(target);
    Ok(dto)
}

/// Drops a custom target from the catalog entirely (distinct from just
/// unchecking it for one scan run). Built-in catalog entries can't be
/// removed this way — only ones this session added.
#[tauri::command]
pub fn remove_custom_target(custom: State<'_, CustomTargets>, id: String) {
    custom.targets.lock().unwrap().retain(|t| t.id != id);
}

/// Kicks off a scan and returns its `scan_id` immediately — the scan itself
/// runs on a dedicated `std::thread`, never on the async runtime, since this
/// is CPU/syscall-bound work rather than async I/O.
///
/// Progress is real: the engine bumps the shared `WalkProgress` atomics as it
/// walks, and the coalescer samples them on its own tick, so `scan://progress`
/// reflects genuine per-file counts rather than a simulated ramp.
#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    registry: State<'_, ScanRegistry>,
    custom: State<'_, CustomTargets>,
    target_ids: Vec<String>,
) -> Result<String, String> {
    let scan_id = format!("scan-{}", uuid_like());
    let cancel = Arc::new(AtomicBool::new(false));
    registry.cancel_flags.lock().unwrap().insert(scan_id.clone(), cancel.clone());

    let progress = ScanProgress::new();
    spawn_coalescer(app.clone(), scan_id.clone(), progress.clone());

    let chosen: Vec<ScanTarget> =
        all_targets(&custom).into_iter().filter(|t| target_ids.contains(&t.id.to_string())).collect();
    let scan_id_for_thread = scan_id.clone();

    std::thread::spawn(move || {
        let summary = sweep_core::run_scan_with_progress(&chosen, &cancel, Some(&progress.walk));
        progress.done.store(true, Ordering::Relaxed);

        let _ = app.emit("scan://done", DoneEvent { scan_id: scan_id_for_thread, summary });
    });

    Ok(scan_id)
}

#[tauri::command]
pub fn cancel_scan(registry: State<'_, ScanRegistry>, scan_id: String) {
    if let Some(flag) = registry.cancel_flags.lock().unwrap().get(&scan_id) {
        flag.store(true, Ordering::Relaxed);
    }
}

/// On-demand full listing of one folder's files, for the "show all N files"
/// drill-down action when a folder has more files than the scan's per-target
/// entry cap kept. Read-only, but still scoped to a known scan target's
/// root (canonicalized, `starts_with` check) rather than an arbitrary path,
/// so this can't become a way for the frontend to enumerate any folder on
/// disk — same trust boundary the existing drill-down already relies on,
/// just without `allowlist_map`'s `refuse_delete` filtering, since Tier C
/// targets are already informationally browsable today, just not deletable.
#[tauri::command]
pub fn list_folder_entries(
    custom: State<'_, CustomTargets>,
    path: String,
) -> Result<Vec<sweep_core::model::Entry>, String> {
    let candidate = PathBuf::from(&path);
    let canonical = candidate.canonicalize().map_err(|e| format!("can't access '{path}': {e}"))?;

    let known_roots = all_targets(&custom);
    let in_scope = known_roots.iter().any(|t| {
        t.roots.iter().any(|r| r.canonicalize().map(|cr| canonical.starts_with(&cr)).unwrap_or(false))
    });
    if !in_scope {
        return Err(format!("'{path}' is not inside a known scan target"));
    }

    let cancel = AtomicBool::new(false);
    Ok(sweep_core::walk::list_folder(&canonical, &cancel))
}

/// Executes a reclaim plan built by the frontend from the user's current
/// selection. This is the only path that actually deletes anything — the
/// plan is re-validated here from scratch (allowlist membership per target,
/// staleness, TOCTOU) rather than trusted as-is; the frontend cannot bypass
/// any of `reclaim::execute`'s checks just by sending a well-formed plan.
/// Always moves to the Trash: this app never wires up permanent deletion
/// from the UI, regardless of what a plan's `permanent` field claims.
#[tauri::command]
pub fn execute_reclaim(custom: State<'_, CustomTargets>, mut plan: ReclaimPlan) -> ReclaimOutcome {
    plan.permanent = false;

    let home = home_dir();
    let allowlist = sweep_core::allowlist_map(&all_targets(&custom));
    let ops = sweep_core::fsops::RealFileOps;
    sweep_core::reclaim::execute(&plan, &allowlist, &home, &ops)
}

/// Not a real UUID generator — this app has no need for global uniqueness
/// guarantees, just a fresh id per scan within one process's lifetime.
fn uuid_like() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed).to_string()
}

fn next_custom_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
