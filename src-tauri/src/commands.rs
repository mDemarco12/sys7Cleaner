use crate::progress::{spawn_coalescer, ScanProgress};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use sweep_core::model::{Granularity, Safety};
use tauri::{AppHandle, Emitter, State};

/// Per-`scan_id` cancellation flags, checked by the scan thread. Kept in
/// Tauri-managed state rather than a global so multiple scans (in principle)
/// never share a flag.
#[derive(Default)]
pub struct ScanRegistry {
    pub cancel_flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

#[derive(Clone, Serialize)]
pub struct TargetDto {
    pub id: String,
    pub label: String,
    pub safety: Safety,
    pub granularity: Granularity,
    pub blurb: String,
    pub refuse_delete: bool,
}

#[derive(Clone, Serialize)]
struct DoneEvent {
    scan_id: String,
    summary: sweep_core::model::ScanSummary,
}

fn home_dir() -> std::path::PathBuf {
    std::env::var_os("HOME").map(std::path::PathBuf::from).expect("HOME must be set")
}

/// Returns the full cleanup target catalog. Pure data, no scanning — safe to
/// call synchronously from the async command context.
#[tauri::command]
pub fn list_targets() -> Vec<TargetDto> {
    sweep_core::catalog::build_catalog(&home_dir())
        .into_iter()
        .map(|t| TargetDto {
            id: t.id.to_string(),
            label: t.label.to_string(),
            safety: t.safety,
            granularity: t.granularity,
            blurb: t.blurb.to_string(),
            refuse_delete: t.refuse_delete,
        })
        .collect()
}

/// Kicks off a scan and returns its `scan_id` immediately — the scan itself
/// runs on a dedicated `std::thread`, never on the async runtime, since this
/// is CPU/syscall-bound work rather than async I/O.
///
/// NOTE (M2 scope): the intermediate `scan://progress` ticks emitted here are
/// synthetic — they exercise the exact same throttled-coalescer pipeline
/// that will carry real per-directory counts once `sweep_core::walk` is
/// instrumented with shared atomics in M4. The final `scan://done` payload,
/// by contrast, is already a genuine `ScanSummary` from `sweep_core::run_scan`
/// against the real catalog — only the "in progress" numbers are fake, not
/// the destination or the result.
#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    registry: State<'_, ScanRegistry>,
    target_ids: Vec<String>,
) -> Result<String, String> {
    let scan_id = format!("scan-{}", uuid_like());
    let cancel = Arc::new(AtomicBool::new(false));
    registry.cancel_flags.lock().unwrap().insert(scan_id.clone(), cancel.clone());

    let progress = ScanProgress::new();
    spawn_coalescer(app.clone(), scan_id.clone(), progress.clone());

    let home = home_dir();
    let scan_id_for_thread = scan_id.clone();

    std::thread::spawn(move || {
        let catalog = sweep_core::catalog::build_catalog(&home);
        let chosen: Vec<_> = catalog.into_iter().filter(|t| target_ids.contains(&t.id.to_string())).collect();

        // Synthetic progress: ramps up over ~1s so the UI can prove it
        // receives and renders live ticks, independent of how fast the
        // real scan underneath happens to finish.
        for step in 1..=20u64 {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            progress.files_seen.store(step * 137, Ordering::Relaxed);
            progress.bytes_seen.store(step * 4_194_304, Ordering::Relaxed);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let summary = sweep_core::run_scan(&chosen, &cancel);
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

/// Not a real UUID generator — this app has no need for global uniqueness
/// guarantees, just a fresh id per scan within one process's lifetime.
fn uuid_like() -> String {
    use std::sync::atomic::AtomicU64;
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed).to_string()
}
