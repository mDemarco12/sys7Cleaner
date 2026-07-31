use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use sweep_core::walk::WalkProgress;
use tauri::{AppHandle, Emitter};

/// Shared counters a scan's worker(s) bump as they go. A single coalescer
/// thread (see `spawn_coalescer`) wakes on a fixed tick and emits at most one
/// event per tick — never one event per file. Serializing a JSON message per
/// file across the Tauri IPC bridge (hundreds of thousands of them, on a big
/// home directory) would stutter the webview far worse than any GC pause;
/// this is the single highest-leverage performance decision in the IPC layer.
///
/// The counters themselves live in `sweep_core`'s `WalkProgress` so the
/// engine can bump them without taking any dependency on Tauri; this struct
/// just pairs them with the completion flag the coalescer watches.
pub struct ScanProgress {
    pub walk: WalkProgress,
    pub done: AtomicBool,
}

impl ScanProgress {
    pub fn new() -> Arc<Self> {
        Arc::new(ScanProgress {
            walk: WalkProgress::default(),
            done: AtomicBool::new(false),
        })
    }
}

#[derive(Clone, Serialize)]
struct ProgressEvent {
    scan_id: String,
    files_seen: u64,
    bytes_seen: u64,
    folders_seen: u64,
    phase: u8,
}

/// Ticks every 50ms (≤20 events/sec) emitting `scan://progress` until
/// `progress.done` is set, at which point it emits one final tick and exits.
/// Runs on its own thread so it never competes with the scan's own worker(s).
pub fn spawn_coalescer(app: AppHandle, scan_id: String, progress: Arc<ScanProgress>) {
    std::thread::spawn(move || loop {
        let files_seen = progress.walk.files.load(Ordering::Relaxed);
        let bytes_seen = progress.walk.bytes.load(Ordering::Relaxed);
        let folders_seen = progress.walk.folders.load(Ordering::Relaxed);
        let phase = progress.walk.phase.load(Ordering::Relaxed);
        let is_done = progress.done.load(Ordering::Relaxed);

        let _ = app.emit(
            "scan://progress",
            ProgressEvent { scan_id: scan_id.clone(), files_seen, bytes_seen, folders_seen, phase },
        );

        if is_done {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    });
}
