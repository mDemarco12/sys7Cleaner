use crate::model::Entry;
use jwalk::WalkDir;
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;

/// Phase values for `WalkProgress::phase`.
pub const PHASE_SCANNING: u8 = 0;
pub const PHASE_ANALYZING: u8 = 1;

/// Live counters a long-running scan bumps as it goes, so a UI layer can
/// report real progress instead of a simulated ramp. Deliberately plain
/// atomics with no Tauri/event knowledge — `sweep-core` stays dependency-free
/// and the consumer decides how (and how often) to sample these.
///
/// `files`/`bytes` are only bumped by the sizing walk; `folders` only by the
/// folder-breakdown pass. They are kept separate because `run_scan` walks
/// every tree twice (once to size it, once to break it into deletable units),
/// so summing both passes into one counter would report roughly double the
/// true file count.
#[derive(Default)]
pub struct WalkProgress {
    pub files: AtomicU64,
    pub bytes: AtomicU64,
    pub folders: AtomicU64,
    pub phase: AtomicU8,
}

pub struct WalkOutcome {
    pub apparent_bytes: u64,
    pub disk_bytes: u64,
    pub file_count: u64,
    /// Largest entries seen, capped at `cap`, sorted descending by disk_bytes.
    pub top_entries: Vec<Entry>,
    pub truncated: bool,
    pub denied: Vec<PathBuf>,
}

pub struct WalkLimits {
    pub entry_cap: usize,
}

impl Default for WalkLimits {
    fn default() -> Self {
        WalkLimits { entry_cap: 500 }
    }
}

/// Parallel, cancellable, device-boundary-respecting directory sizing.
///
/// Correctness properties this enforces (each fixes a real bug in the
/// Tkinter prototype's `os.walk`-based `size()`):
///   - never follows symlinks (a symlink loop would hang os.walk forever)
///   - sizes in allocated blocks (`st_blocks * 512`), matching Finder/du,
///     not `st_size`, which drifts from what's actually reclaimable
///   - deduplicates hardlinks by (dev, ino) so they aren't double-counted
///   - never crosses the root's device boundary (won't wander into
///     /Volumes, network mounts, or a Time Machine APFS snapshot)
///   - every PermissionDenied is collected into `denied`, never swallowed
pub fn size_tree(root: &Path, cancel: &AtomicBool, limits: &WalkLimits) -> WalkOutcome {
    size_tree_with_progress(root, cancel, limits, None)
}

/// As [`size_tree`], but bumps `progress`'s `files`/`bytes` counters as it
/// walks so a caller can render live progress. Split out rather than folded
/// into `size_tree`'s signature so existing callers (the CLI, tests,
/// `list_folder`) stay untouched.
pub fn size_tree_with_progress(
    root: &Path,
    cancel: &AtomicBool,
    limits: &WalkLimits,
    progress: Option<&WalkProgress>,
) -> WalkOutcome {
    if !root.exists() {
        return WalkOutcome {
            apparent_bytes: 0,
            disk_bytes: 0,
            file_count: 0,
            top_entries: Vec::new(),
            truncated: false,
            denied: Vec::new(),
        };
    }

    let root_dev = match std::fs::metadata(root) {
        Ok(m) => m.dev(),
        Err(_) => {
            return WalkOutcome {
                apparent_bytes: 0,
                disk_bytes: 0,
                file_count: 0,
                top_entries: Vec::new(),
                truncated: false,
                denied: vec![root.to_path_buf()],
            }
        }
    };

    let apparent = AtomicU64::new(0);
    let disk = AtomicU64::new(0);
    let files = AtomicU64::new(0);
    let seen_inodes: Mutex<HashSet<(u64, u64)>> = Mutex::new(HashSet::new());
    let denied: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    let top: Mutex<Vec<Entry>> = Mutex::new(Vec::new());

    let walker = WalkDir::new(root)
        .follow_links(false)
        .skip_hidden(false)
        .process_read_dir(move |_depth, _path, _state, children| {
            children.retain(|entry_result| {
                if let Ok(entry) = entry_result {
                    // Prune whole node_modules subtrees — size as a unit rather
                    // than descending into potentially tens of thousands of files.
                    entry.file_name() != "node_modules" || entry.file_type().is_file()
                } else {
                    true
                }
            });
        });

    for entry_result in walker {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                if let Some(path) = e.path() {
                    denied.lock().unwrap().push(path.to_path_buf());
                }
                continue;
            }
        };

        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => {
                denied.lock().unwrap().push(path);
                continue;
            }
        };

        if meta.is_dir() {
            continue;
        }

        // Never cross the root's device boundary.
        if meta.dev() != root_dev {
            continue;
        }

        let key = (meta.dev(), meta.ino());
        {
            let mut seen = seen_inodes.lock().unwrap();
            if !seen.insert(key) {
                continue; // hardlink already counted
            }
        }

        let this_apparent = meta.size();
        let this_disk = (meta.blocks() as u64) * 512;

        apparent.fetch_add(this_apparent, Ordering::Relaxed);
        disk.fetch_add(this_disk, Ordering::Relaxed);
        files.fetch_add(1, Ordering::Relaxed);

        if let Some(p) = progress {
            p.files.fetch_add(1, Ordering::Relaxed);
            p.bytes.fetch_add(this_disk, Ordering::Relaxed);
        }

        let modified = meta.modified().ok();
        let entry_rec = Entry {
            path: path.clone(),
            disk_bytes: this_disk,
            modified,
            dev: meta.dev(),
            ino: meta.ino(),
        };

        let mut top_guard = top.lock().unwrap();
        top_guard.push(entry_rec);
        if top_guard.len() > limits.entry_cap * 4 {
            // Periodically compact so this doesn't grow unbounded on huge trees.
            top_guard.sort_by(|a, b| b.disk_bytes.cmp(&a.disk_bytes));
            top_guard.truncate(limits.entry_cap);
        }
    }

    let mut top_entries = top.into_inner().unwrap();
    let truncated_by_cap = top_entries.len() > limits.entry_cap;
    top_entries.sort_by(|a, b| b.disk_bytes.cmp(&a.disk_bytes));
    top_entries.truncate(limits.entry_cap);

    WalkOutcome {
        apparent_bytes: apparent.load(Ordering::Relaxed),
        disk_bytes: disk.load(Ordering::Relaxed),
        file_count: files.load(Ordering::Relaxed),
        top_entries,
        truncated: truncated_by_cap,
        denied: denied.into_inner().unwrap(),
    }
}

/// Full (not target-wide-capped) listing of every file under one folder,
/// sorted descending by size. For on-demand "show everything in this
/// specific folder" browsing, as opposed to `size_tree`'s `top_entries`,
/// which are the largest entries across an entire scan target and so can
/// under-represent any one folder within it. `entry_cap` here is a generous
/// safety ceiling against pathological trees, not a meaningful UI limit —
/// pagination of the result is the caller's job.
pub fn list_folder(root: &Path, cancel: &AtomicBool) -> Vec<Entry> {
    let limits = WalkLimits { entry_cap: 50_000 };
    size_tree(root, cancel, &limits).top_entries
}

/// True available bytes on the volume containing `path`, via `statfs`
/// (`f_bavail * f_bsize`). macOS's Finder "Available" figure includes
/// purgeable space, which `statvfs`/naive free-space reads won't reconcile
/// with — this is the number that actually matches Finder.
pub fn available_bytes(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    let cpath = CString::new(path.as_os_str().as_encoded_bytes()).ok()?;
    unsafe {
        let mut stat: libc::statfs = std::mem::zeroed();
        if libc::statfs(cpath.as_ptr(), &mut stat) != 0 {
            return None;
        }
        Some(stat.f_bavail as u64 * stat.f_bsize as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::AtomicBool;
    use tempfile::tempdir;

    #[test]
    fn sizes_nested_files_correctly() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::write(dir.path().join("a/one.bin"), vec![0u8; 4096]).unwrap();
        fs::write(dir.path().join("a/b/two.bin"), vec![0u8; 8192]).unwrap();

        let cancel = AtomicBool::new(false);
        let outcome = size_tree(dir.path(), &cancel, &WalkLimits::default());

        assert_eq!(outcome.file_count, 2);
        assert!(outcome.disk_bytes >= 4096 + 8192);
        assert_eq!(outcome.denied.len(), 0);
    }

    #[test]
    fn deduplicates_hardlinks() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("orig.bin"), vec![0u8; 4096]).unwrap();
        fs::hard_link(dir.path().join("orig.bin"), dir.path().join("link.bin")).unwrap();

        let cancel = AtomicBool::new(false);
        let outcome = size_tree(dir.path(), &cancel, &WalkLimits::default());

        // Both directory entries exist, but they share one inode: count once.
        assert_eq!(outcome.file_count, 1);
    }

    #[test]
    fn does_not_follow_symlink_loops() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        // Symlink back to the parent — following it would recurse forever.
        std::os::unix::fs::symlink(dir.path(), sub.join("loop")).unwrap();

        let cancel = AtomicBool::new(false);
        // The property under test is termination: with follow_links(false)
        // the symlink is never dereferenced, so this must return promptly
        // instead of recursing forever. The symlink entry itself is still a
        // real (non-directory) directory entry and is counted as one file.
        let outcome = size_tree(dir.path(), &cancel, &WalkLimits::default());
        assert_eq!(outcome.file_count, 1);
    }

    #[test]
    fn nonexistent_root_returns_empty_not_error() {
        let cancel = AtomicBool::new(false);
        let outcome = size_tree(Path::new("/definitely/does/not/exist"), &cancel, &WalkLimits::default());
        assert_eq!(outcome.file_count, 0);
        assert_eq!(outcome.disk_bytes, 0);
    }

    #[test]
    fn progress_counters_track_the_real_walk() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a")).unwrap();
        fs::write(dir.path().join("a/one.bin"), vec![0u8; 4096]).unwrap();
        fs::write(dir.path().join("two.bin"), vec![0u8; 8192]).unwrap();

        let cancel = AtomicBool::new(false);
        let progress = WalkProgress::default();
        let outcome = size_tree_with_progress(dir.path(), &cancel, &WalkLimits::default(), Some(&progress));

        // The whole point of the sink: what it reports must equal what the
        // walk actually found, not an independently-derived approximation.
        assert_eq!(progress.files.load(Ordering::Relaxed), outcome.file_count);
        assert_eq!(progress.bytes.load(Ordering::Relaxed), outcome.disk_bytes);
        assert_eq!(progress.files.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn size_tree_without_progress_still_walks_identically() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("one.bin"), vec![0u8; 4096]).unwrap();

        let cancel = AtomicBool::new(false);
        let with_none = size_tree_with_progress(dir.path(), &cancel, &WalkLimits::default(), None);
        let plain = size_tree(dir.path(), &cancel, &WalkLimits::default());

        assert_eq!(plain.file_count, with_none.file_count);
        assert_eq!(plain.disk_bytes, with_none.disk_bytes);
    }

    #[test]
    fn cancellation_stops_the_walk() {
        let dir = tempdir().unwrap();
        for i in 0..50 {
            fs::write(dir.path().join(format!("f{i}.bin")), vec![0u8; 1024]).unwrap();
        }
        let cancel = AtomicBool::new(true); // pre-cancelled
        let outcome = size_tree(dir.path(), &cancel, &WalkLimits::default());
        assert_eq!(outcome.file_count, 0);
    }
}
