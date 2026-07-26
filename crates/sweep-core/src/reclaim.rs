use crate::fsops::FileOps;
use crate::model::{PlanItem, ReclaimOutcome, ReclaimPlan};
use crate::safety::{validate_deletion_path, SafetyViolation};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Plans older than this are refused outright — the scan that produced them
/// may no longer reflect reality (TOCTOU guard).
const MAX_PLAN_AGE: Duration = Duration::from_secs(5 * 60);

/// What `execute` is allowed to delete for one target: its allowlist roots,
/// plus whether the exact root itself is a valid deletion target (true for
/// `Granularity::WholeRoot`, false for `Granularity::Children` — see
/// `safety::validate_deletion_path` for why this distinction matters).
#[derive(Debug, Clone)]
pub struct TargetAllowlist {
    pub roots: Vec<PathBuf>,
    pub root_itself_deletable: bool,
}

/// Execute a reclaim plan against the given `FileOps` implementation.
///
/// This is the ONLY function that mutates the filesystem for deletion, and it
/// is identical whether called with `RealFileOps`, `DryRunFileOps`, or
/// `RecordingFileOps` — a dry run and a real run walk the exact same logic,
/// so a dry-run report genuinely proves what the real run would do.
///
/// Every item is re-validated against its target's allowlist roots here, even
/// though the plan was built from a scan — the plan is not trusted input.
pub fn execute(
    plan: &ReclaimPlan,
    allowlist_by_target: &std::collections::HashMap<String, TargetAllowlist>,
    home: &std::path::Path,
    ops: &dyn FileOps,
) -> ReclaimOutcome {
    let mut outcome = ReclaimOutcome::empty();

    let force_dry_run = std::env::var("SWEEP_DRY_RUN").map(|v| v == "1").unwrap_or(false);

    let now = SystemTime::now();
    if now.duration_since(plan.created_at).map(|age| age > MAX_PLAN_AGE).unwrap_or(true) {
        for item in &plan.items {
            outcome.skipped_stale.push(item.path.clone());
        }
        return outcome;
    }

    for item in &plan.items {
        let Some(allow) = allowlist_by_target.get(&item.target_id) else {
            outcome.failed.push((item.path.clone(), "unknown target_id, not in allowlist".into()));
            continue;
        };

        let validated = match validate_deletion_path(&item.path, &allow.roots, home, allow.root_itself_deletable) {
            Ok(p) => p,
            Err(SafetyViolation::Unresolvable(_)) => {
                // Already gone, or moved — treat as stale rather than an error.
                outcome.skipped_stale.push(item.path.clone());
                continue;
            }
            Err(e) => {
                outcome.failed.push((item.path.clone(), e.to_string()));
                continue;
            }
        };

        if toctou_changed(item, ops) {
            outcome.skipped_stale.push(item.path.clone());
            continue;
        }

        let use_permanent = plan.permanent && !force_dry_run;

        let result = if use_permanent {
            ops.remove(&validated)
        } else {
            ops.trash(&validated)
        };

        match result {
            Ok(()) => {
                outcome.bytes_reclaimed_estimate += item.expected_disk_bytes;
                if use_permanent {
                    outcome.removed_permanently.push(validated);
                } else {
                    outcome.trashed.push(validated);
                }
            }
            Err(e) => outcome.failed.push((item.path.clone(), e.to_string())),
        }
    }

    outcome
}

fn toctou_changed(item: &PlanItem, ops: &dyn FileOps) -> bool {
    match ops.metadata(&item.path) {
        Ok(meta) => meta.dev() != item.expected_dev || meta.ino() != item.expected_ino,
        Err(_) => true, // vanished since the scan — treat as changed/stale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsops::RecordingFileOps;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

    fn plan_item_for(target_id: &str, path: &std::path::Path) -> PlanItem {
        let meta = fs::metadata(path).unwrap();
        PlanItem {
            target_id: target_id.to_string(),
            path: path.to_path_buf(),
            expected_disk_bytes: (meta.blocks() as u64) * 512,
            expected_dev: meta.dev(),
            expected_ino: meta.ino(),
        }
    }

    #[test]
    fn trashes_exactly_the_planned_paths() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("SomeApp");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("data.bin"), vec![0u8; 1024]).unwrap();

        let plan = ReclaimPlan {
            items: vec![plan_item_for("app-caches", &target)],
            permanent: false,
            created_at: SystemTime::now(),
        };
        let mut allowlist = HashMap::new();
        allowlist.insert("app-caches".to_string(), TargetAllowlist { roots: vec![root.clone()], root_itself_deletable: false });

        let ops = RecordingFileOps::new();
        let outcome = execute(&plan, &allowlist, home.path(), &ops);

        assert_eq!(outcome.trashed, vec![target.canonicalize().unwrap()]);
        assert!(outcome.failed.is_empty());
        assert!(outcome.skipped_stale.is_empty());
    }

    #[test]
    fn refuses_path_outside_its_targets_allowlist() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();
        let sensitive = home.path().join("Documents/important.txt");
        fs::create_dir_all(sensitive.parent().unwrap()).unwrap();
        fs::write(&sensitive, "keep me").unwrap();

        let plan = ReclaimPlan {
            items: vec![plan_item_for("app-caches", &sensitive)],
            permanent: false,
            created_at: SystemTime::now(),
        };
        let mut allowlist = HashMap::new();
        allowlist.insert("app-caches".to_string(), TargetAllowlist { roots: vec![root], root_itself_deletable: false });

        let ops = RecordingFileOps::new();
        let outcome = execute(&plan, &allowlist, home.path(), &ops);

        assert!(outcome.trashed.is_empty());
        assert_eq!(outcome.failed.len(), 1);
    }

    #[test]
    fn stale_plan_is_entirely_skipped() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("SomeApp");
        fs::create_dir(&target).unwrap();

        let old_time = SystemTime::now() - Duration::from_secs(10 * 60);
        let plan = ReclaimPlan {
            items: vec![plan_item_for("app-caches", &target)],
            permanent: false,
            created_at: old_time,
        };
        let mut allowlist = HashMap::new();
        allowlist.insert("app-caches".to_string(), TargetAllowlist { roots: vec![root], root_itself_deletable: false });

        let ops = RecordingFileOps::new();
        let outcome = execute(&plan, &allowlist, home.path(), &ops);

        assert!(outcome.trashed.is_empty());
        assert_eq!(outcome.skipped_stale.len(), 1);
    }

    #[test]
    fn toctou_detects_path_replaced_since_scan() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("SomeApp");
        fs::create_dir(&target).unwrap();

        let mut item = plan_item_for("app-caches", &target);
        // Simulate the scan having seen a different inode (e.g. dir was
        // replaced between scan and reclaim).
        item.expected_ino = item.expected_ino.wrapping_add(999);

        let plan = ReclaimPlan {
            items: vec![item],
            permanent: false,
            created_at: SystemTime::now(),
        };
        let mut allowlist = HashMap::new();
        allowlist.insert("app-caches".to_string(), TargetAllowlist { roots: vec![root], root_itself_deletable: false });

        let ops = RecordingFileOps::new();
        let outcome = execute(&plan, &allowlist, home.path(), &ops);

        assert!(outcome.trashed.is_empty());
        assert_eq!(outcome.skipped_stale.len(), 1);
    }

    #[test]
    fn dry_run_env_var_forces_trash_even_for_permanent_plan() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("BigThing");
        fs::create_dir(&target).unwrap();

        std::env::set_var("SWEEP_DRY_RUN", "1");
        let plan = ReclaimPlan {
            items: vec![plan_item_for("app-caches", &target)],
            permanent: true, // requests permanent delete...
            created_at: SystemTime::now(),
        };
        let mut allowlist = HashMap::new();
        allowlist.insert("app-caches".to_string(), TargetAllowlist { roots: vec![root], root_itself_deletable: false });

        let ops = RecordingFileOps::new();
        let outcome = execute(&plan, &allowlist, home.path(), &ops);
        std::env::remove_var("SWEEP_DRY_RUN");

        // ...but the global kill switch forces trash instead of permanent removal.
        assert!(outcome.removed_permanently.is_empty());
        assert_eq!(outcome.trashed.len(), 1);
    }

    #[test]
    fn whole_root_target_can_trash_its_own_root() {
        // Regression test: a WholeRoot target (e.g. a custom-added folder)
        // has its own root as the single deletable unit. Before
        // root_itself_deletable existed, this was wrongly refused as
        // "too shallow" — the same rule that correctly protects a
        // Children-granularity container root (like ~/Library/Caches
        // itself) was also blocking WholeRoot targets it was never meant to
        // apply to.
        let home = tempdir().unwrap();
        let root = home.path().join("Desktop/SweepTest");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("junk.bin"), vec![0u8; 1024]).unwrap();

        let plan = ReclaimPlan {
            items: vec![plan_item_for("custom-1", &root)],
            permanent: false,
            created_at: SystemTime::now(),
        };
        let mut allowlist = HashMap::new();
        allowlist.insert("custom-1".to_string(), TargetAllowlist { roots: vec![root.clone()], root_itself_deletable: true });

        let ops = RecordingFileOps::new();
        let outcome = execute(&plan, &allowlist, home.path(), &ops);

        assert_eq!(outcome.trashed, vec![root.canonicalize().unwrap()]);
        assert!(outcome.failed.is_empty());
    }
}
