pub mod bundles;
pub mod catalog;
pub mod fsops;
pub mod macho;
pub mod model;
pub mod planning;
pub mod reclaim;
pub mod safety;
pub mod walk;

use model::{Granularity, ScanSummary, ScanTarget, TargetResult};
use reclaim::TargetAllowlist;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

/// Human-readable byte formatter (B/KB/MB at one decimal, GB/TB/PB at two —
/// one decimal at GB scale quantises to ~107 MB per step, too coarse for
/// deciding what's worth deleting), matching the reference prototype's
/// `human()` but operating on the correct (disk_bytes) number.
pub fn human_bytes(n: u64) -> String {
    let mut val = n as f64;
    for (unit, decimals) in [("B", 1), ("KB", 1), ("MB", 1), ("GB", 2), ("TB", 2)] {
        if val < 1024.0 {
            return format!("{val:.decimals$} {unit}");
        }
        val /= 1024.0;
    }
    format!("{val:.2} PB")
}

/// Run every target in `targets` through `walk::size_tree` and assemble a
/// `ScanSummary`. This is the synchronous, non-cancellable core used by the
/// CLI; the Tauri command layer wraps this on a dedicated `std::thread` with
/// a real `AtomicBool` for cancellation and a throttled progress emitter —
/// this function itself has no knowledge of Tauri, threads, or events.
pub fn run_scan(targets: &[ScanTarget], cancel: &AtomicBool) -> ScanSummary {
    run_scan_with_progress(targets, cancel, None)
}

/// As [`run_scan`], but reports live counters into `progress` so a UI layer
/// can render real scan progress. Each target is processed in two phases —
/// `PHASE_SCANNING` while its trees are sized, then `PHASE_ANALYZING` while
/// it's broken into deletable units — and `phase` is updated accordingly so
/// the consumer can label what's happening rather than showing a counter
/// that appears to stall during the second pass.
pub fn run_scan_with_progress(
    targets: &[ScanTarget],
    cancel: &AtomicBool,
    progress: Option<&walk::WalkProgress>,
) -> ScanSummary {
    use std::sync::atomic::Ordering;

    let limits = walk::WalkLimits::default();
    let mut results = Vec::new();
    let mut total = 0u64;

    for target in targets {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        if let Some(p) = progress {
            p.phase.store(walk::PHASE_SCANNING, Ordering::Relaxed);
        }

        let mut combined = TargetResult::empty(target.id, target.label, target.refuse_delete);
        for root in &target.roots {
            let outcome = walk::size_tree_with_progress(root, cancel, &limits, progress);
            combined.apparent_bytes += outcome.apparent_bytes;
            combined.disk_bytes += outcome.disk_bytes;
            combined.file_count += outcome.file_count;
            combined.entries.extend(outcome.top_entries);
            combined.truncated |= outcome.truncated;
            combined.denied.extend(outcome.denied);
        }
        combined.entries.sort_by(|a, b| b.disk_bytes.cmp(&a.disk_bytes));
        combined.entries.truncate(limits.entry_cap);

        if let Some(p) = progress {
            p.phase.store(walk::PHASE_ANALYZING, Ordering::Relaxed);
        }

        combined.folders = planning::folder_breakdown_with_progress(target, cancel, progress);
        combined.folders.sort_by(|a, b| b.disk_bytes.cmp(&a.disk_bytes));

        total += combined.disk_bytes;
        results.push(combined);
    }

    ScanSummary {
        cancelled: cancel.load(std::sync::atomic::Ordering::Relaxed),
        results,
        total_disk_bytes: total,
    }
}

/// Build the `target_id -> allowlist roots` map that `reclaim::execute`
/// requires, from the same catalog a scan was run against.
///
/// Targets marked `refuse_delete` (Tier C — Docker's disk image, iCloud
/// Drive, Photos Library) are deliberately excluded here, not just skipped
/// upstream in the UI. `reclaim::execute` only checks whether a path falls
/// under a *registered* allowlist root; it has no separate awareness of
/// `refuse_delete`. Leaving a Tier C target's roots in this map would mean a
/// hand-crafted `ReclaimPlan` (or a future UI bug) referencing that
/// target_id could still pass the allowlist check and get deleted. Omitting
/// it here makes the target_id itself unresolvable, so it fails the same
/// "not in allowlist" path — the engine physically cannot delete a path
/// belonging to a refused target, the same guarantee it already gives for
/// paths outside any target entirely.
pub fn allowlist_map(targets: &[ScanTarget]) -> HashMap<String, TargetAllowlist> {
    targets
        .iter()
        .filter(|t| !t.refuse_delete)
        .map(|t| {
            (
                t.id.to_string(),
                TargetAllowlist {
                    roots: t.roots.clone(),
                    root_itself_deletable: matches!(t.granularity, Granularity::WholeRoot),
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsops::RecordingFileOps;
    use crate::model::{PlanItem, ReclaimPlan};
    use std::fs;
    use std::os::unix::fs::MetadataExt;
    use tempfile::tempdir;

    #[test]
    fn human_bytes_formats_expected_units() {
        assert_eq!(human_bytes(512), "512.0 B");
        assert_eq!(human_bytes(2048), "2.0 KB");
        assert_eq!(human_bytes(5 * 1024 * 1024), "5.0 MB");
        // GB and up carry two decimals, not one — one decimal at this scale
        // would quantise to ~107 MB per step.
        assert_eq!(human_bytes(3 * 1024 * 1024 * 1024), "3.00 GB");
    }

    /// End-to-end proof that a refuse_delete (Tier C) target can never be
    /// deleted through the real catalog + allowlist_map + reclaim::execute
    /// path, even given a well-formed plan with a valid, existing path and a
    /// fresh timestamp — the only two things `execute` otherwise checks.
    #[test]
    fn allowlist_map_excludes_refuse_delete_targets_end_to_end() {
        let home = tempdir().unwrap();
        let docker_dir = home.path().join("Library/Containers/com.docker.docker");
        fs::create_dir_all(&docker_dir).unwrap();

        let catalog = catalog::build_catalog(&home.path().to_path_buf());
        let docker_target = catalog.iter().find(|t| t.id == "docker-data").unwrap();
        assert!(docker_target.refuse_delete);

        let allowlist = allowlist_map(&catalog);
        assert!(
            !allowlist.contains_key("docker-data"),
            "refuse_delete target must not appear in the allowlist map at all"
        );

        let meta = fs::metadata(&docker_dir).unwrap();
        let plan = ReclaimPlan {
            items: vec![PlanItem {
                target_id: "docker-data".to_string(),
                path: docker_dir.clone(),
                expected_disk_bytes: 0,
                expected_dev: meta.dev(),
                expected_ino: meta.ino(),
            }],
            permanent: false,
            created_at: std::time::SystemTime::now(),
        };

        let ops = RecordingFileOps::new();
        let outcome = reclaim::execute(&plan, &allowlist, home.path(), &ops);

        assert!(outcome.trashed.is_empty());
        assert_eq!(outcome.failed.len(), 1);
        assert!(docker_dir.exists(), "Docker's directory must still exist on disk");
    }
}
