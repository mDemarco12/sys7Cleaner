use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// How safe it is to auto-select a target for bulk deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Safety {
    /// Fully regenerable, safe to default-select (Tier A).
    Regenerable,
    /// User data or ambiguous; never bulk-selected, reviewed item by item (Tier B).
    ReviewRequired,
    /// The engine refuses to plan or execute deletion for this target at all (Tier C).
    NeverTouch,
}

/// Whether we trash a target as one unit or let the user pick individual children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Granularity {
    /// Trash the whole root as a single Trash item (fast; use for many-small-file trees).
    WholeRoot,
    /// Each immediate child of the root is a separate selectable/trashable unit.
    Children,
    /// Individual files are selectable (large-files explorer).
    Files,
}

/// A declarative scan/cleanup target. Roots double as the deletion allowlist:
/// the engine will only ever trash a path that canonicalizes under one of these roots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTarget {
    pub id: &'static str,
    pub label: &'static str,
    pub roots: Vec<PathBuf>,
    pub safety: Safety,
    pub granularity: Granularity,
    pub blurb: &'static str,
    /// If true, this target is measured/reported but deletion is always refused (Tier C).
    pub refuse_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub path: PathBuf,
    pub disk_bytes: u64,
    pub modified: Option<SystemTime>,
    pub dev: u64,
    pub ino: u64,
}

/// One deletable unit within a target: either the target's whole root
/// (`Granularity::WholeRoot`) or one immediate child of it
/// (`Granularity::Children`). This is the folder-level grouping shown at the
/// top of the results browser — `Entry` values (individual files) are only
/// surfaced one level down, after the user drills into a specific folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderSummary {
    pub path: PathBuf,
    pub label: String,
    pub disk_bytes: u64,
    pub file_count: u64,
    pub is_dir: bool,
    pub dev: u64,
    pub ino: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetResult {
    pub id: String,
    pub label: String,
    pub apparent_bytes: u64,
    pub disk_bytes: u64,
    pub file_count: u64,
    /// Folder-level breakdown (one per deletable unit) — what the results
    /// browser groups by at the top level.
    pub folders: Vec<FolderSummary>,
    /// Largest individual files across the whole target, capped. Used only
    /// for the drill-down view, filtered by path prefix to one folder.
    pub entries: Vec<Entry>,
    pub truncated: bool,
    pub denied: Vec<PathBuf>,
    pub refuse_delete: bool,
}

impl TargetResult {
    pub fn empty(id: &str, label: &str, refuse_delete: bool) -> Self {
        TargetResult {
            id: id.to_string(),
            label: label.to_string(),
            apparent_bytes: 0,
            disk_bytes: 0,
            file_count: 0,
            folders: Vec::new(),
            entries: Vec::new(),
            truncated: false,
            denied: Vec::new(),
            refuse_delete,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub results: Vec<TargetResult>,
    pub total_disk_bytes: u64,
    pub cancelled: bool,
}

/// A concrete set of paths approved for deletion, produced from scan results.
/// This is the only input `reclaim::execute` accepts — nothing outside a
/// plan built from an actual scan can be deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReclaimPlan {
    pub items: Vec<PlanItem>,
    pub permanent: bool,
    pub created_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub target_id: String,
    pub path: PathBuf,
    pub expected_disk_bytes: u64,
    pub expected_dev: u64,
    pub expected_ino: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReclaimOutcome {
    pub trashed: Vec<PathBuf>,
    pub removed_permanently: Vec<PathBuf>,
    pub skipped_stale: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
    pub bytes_reclaimed_estimate: u64,
}

impl ReclaimOutcome {
    pub fn empty() -> Self {
        ReclaimOutcome {
            trashed: Vec::new(),
            removed_permanently: Vec::new(),
            skipped_stale: Vec::new(),
            failed: Vec::new(),
            bytes_reclaimed_estimate: 0,
        }
    }
}

/// Architecture classification for a Mach-O executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Arch {
    IntelOnly,
    AppleSiliconOnly,
    Universal,
    /// 32-bit only; cannot run on any modern macOS at all.
    Dead,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub bundle_path: PathBuf,
    pub name: String,
    pub arch: Arch,
    pub last_used: Option<SystemTime>,
}
