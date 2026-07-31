use crate::model::{FolderSummary, Granularity, ScanTarget};
use crate::walk::{self, WalkLimits, WalkProgress};
use std::os::unix::fs::MetadataExt;
use std::sync::atomic::{AtomicBool, Ordering};

/// Breaks a target down into its deletable units, per its `Granularity`:
///
/// - `WholeRoot`: one `FolderSummary` per root, sized as the whole subtree.
/// - `Children`: one `FolderSummary` per immediate child of each root — a
///   directory child is re-walked for its true recursive size (a directory's
///   own metadata only reflects its inode's block usage, not its contents);
///   a file child is sized directly.
/// - `Files`: not broken into folders — callers should use the flat
///   `TargetResult::entries` list directly for this granularity.
///
/// This is the single source of truth for "what are the deletable units of
/// this target" — used both to populate `TargetResult::folders` at scan time
/// (what the results browser groups by) and to build `PlanItem`s for deletion,
/// so the two can never drift apart.
pub fn folder_breakdown(target: &ScanTarget, cancel: &AtomicBool) -> Vec<FolderSummary> {
    folder_breakdown_with_progress(target, cancel, None)
}

/// As [`folder_breakdown`], but bumps `progress.folders` once per deletable
/// unit resolved, so a UI can show movement during this pass too.
///
/// Only the `folders` counter is touched: the sizing walks this performs
/// re-visit files the caller's earlier `size_tree` pass already counted, so
/// feeding them into `files`/`bytes` would double-count them.
pub fn folder_breakdown_with_progress(
    target: &ScanTarget,
    cancel: &AtomicBool,
    progress: Option<&WalkProgress>,
) -> Vec<FolderSummary> {
    let limits = WalkLimits::default();
    let mut out = Vec::new();
    let bump = || {
        if let Some(p) = progress {
            p.folders.fetch_add(1, Ordering::Relaxed);
        }
    };

    match target.granularity {
        Granularity::WholeRoot => {
            for root in &target.roots {
                let Ok(meta) = std::fs::metadata(root) else { continue };
                let outcome = walk::size_tree(root, cancel, &limits);
                out.push(FolderSummary {
                    path: root.clone(),
                    label: target.label.to_string(),
                    disk_bytes: outcome.disk_bytes,
                    file_count: outcome.file_count,
                    is_dir: true,
                    dev: meta.dev(),
                    ino: meta.ino(),
                });
                bump();
            }
        }
        Granularity::Children => {
            for root in &target.roots {
                let Ok(read) = std::fs::read_dir(root) else { continue };
                for child in read.flatten() {
                    let path = child.path();
                    let Ok(meta) = std::fs::metadata(&path) else { continue };
                    let label = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

                    if meta.is_dir() {
                        let outcome = walk::size_tree(&path, cancel, &limits);
                        out.push(FolderSummary {
                            path,
                            label,
                            disk_bytes: outcome.disk_bytes,
                            file_count: outcome.file_count,
                            is_dir: true,
                            dev: meta.dev(),
                            ino: meta.ino(),
                        });
                        bump();
                    } else {
                        out.push(FolderSummary {
                            path,
                            label,
                            disk_bytes: (meta.blocks() as u64) * 512,
                            file_count: 1,
                            is_dir: false,
                            dev: meta.dev(),
                            ino: meta.ino(),
                        });
                        bump();
                    }
                }
            }
        }
        Granularity::Files => {
            // Flat file listing — no folder grouping; callers use `entries` directly.
        }
    }

    out
}
