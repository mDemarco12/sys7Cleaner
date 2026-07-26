pub mod bundles;
pub mod catalog;
pub mod fsops;
pub mod macho;
pub mod model;
pub mod reclaim;
pub mod safety;
pub mod walk;

use model::{ScanSummary, ScanTarget, TargetResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

/// Human-readable byte formatter (B/KB/MB/GB/TB/PB), matching the reference
/// prototype's `human()` but operating on the correct (disk_bytes) number.
pub fn human_bytes(n: u64) -> String {
    let mut val = n as f64;
    for unit in ["B", "KB", "MB", "GB", "TB"] {
        if val < 1024.0 {
            return format!("{val:.1} {unit}");
        }
        val /= 1024.0;
    }
    format!("{val:.1} PB")
}

/// Run every target in `targets` through `walk::size_tree` and assemble a
/// `ScanSummary`. This is the synchronous, non-cancellable core used by the
/// CLI; the Tauri command layer wraps this on a dedicated `std::thread` with
/// a real `AtomicBool` for cancellation and a throttled progress emitter —
/// this function itself has no knowledge of Tauri, threads, or events.
pub fn run_scan(targets: &[ScanTarget], cancel: &AtomicBool) -> ScanSummary {
    let limits = walk::WalkLimits::default();
    let mut results = Vec::new();
    let mut total = 0u64;

    for target in targets {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let mut combined = TargetResult::empty(target.id, target.label, target.refuse_delete);
        for root in &target.roots {
            let outcome = walk::size_tree(root, cancel, &limits);
            combined.apparent_bytes += outcome.apparent_bytes;
            combined.disk_bytes += outcome.disk_bytes;
            combined.file_count += outcome.file_count;
            combined.entries.extend(outcome.top_entries);
            combined.truncated |= outcome.truncated;
            combined.denied.extend(outcome.denied);
        }
        combined.entries.sort_by(|a, b| b.disk_bytes.cmp(&a.disk_bytes));
        combined.entries.truncate(limits.entry_cap);

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
pub fn allowlist_map(targets: &[ScanTarget]) -> HashMap<String, Vec<PathBuf>> {
    targets.iter().map(|t| (t.id.to_string(), t.roots.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_bytes_formats_expected_units() {
        assert_eq!(human_bytes(512), "512.0 B");
        assert_eq!(human_bytes(2048), "2.0 KB");
        assert_eq!(human_bytes(5 * 1024 * 1024), "5.0 MB");
    }
}
