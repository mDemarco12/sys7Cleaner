use crate::model::{Granularity, Safety, ScanTarget};
use std::path::PathBuf;

/// The full cleanup target catalog, rooted at `home` (or a sandbox root in
/// tests / `--root`). This is data, not code: adding a target never touches
/// the walker, the safety guard, or the deletion path.
///
/// Every root listed here is also the deletion allowlist for that target —
/// `safety::validate_deletion_path` will refuse anything not under one of
/// these roots, so a target that shouldn't be deletable (Tier C) simply
/// carries `refuse_delete: true` rather than needing separate enforcement.
pub fn build_catalog(home: &PathBuf) -> Vec<ScanTarget> {
    vec![
        // ---- Tier A: regenerable, safe to default-select ----
        ScanTarget {
            id: "xcode-derived-data",
            label: "Xcode DerivedData",
            roots: vec![home.join("Library/Developer/Xcode/DerivedData")],
            safety: Safety::Regenerable,
            granularity: Granularity::Children,
            blurb: "Xcode regenerates this on next build. Usually the single largest reclaimable target.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "xcode-ios-device-support",
            label: "Xcode iOS DeviceSupport",
            roots: vec![home.join("Library/Developer/Xcode/iOS DeviceSupport")],
            safety: Safety::Regenerable,
            granularity: Granularity::Children,
            blurb: "Per-device debug symbols Xcode re-downloads when needed.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "homebrew-cache",
            label: "Homebrew Cache",
            roots: vec![home.join("Library/Caches/Homebrew")],
            safety: Safety::Regenerable,
            granularity: Granularity::WholeRoot,
            blurb: "Downloaded bottles/formula archives; `brew` re-fetches on demand.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "npm-cache",
            label: "npm Cache",
            roots: vec![home.join(".npm/_cacache")],
            safety: Safety::Regenerable,
            granularity: Granularity::WholeRoot,
            blurb: "npm re-downloads packages into this cache as needed.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "cargo-registry-cache",
            label: "Cargo Registry Cache",
            roots: vec![home.join(".cargo/registry/cache")],
            safety: Safety::Regenerable,
            granularity: Granularity::WholeRoot,
            blurb: "Downloaded crate archives; cargo re-fetches on demand.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "pip-cache",
            label: "pip Cache",
            roots: vec![home.join("Library/Caches/pip")],
            safety: Safety::Regenerable,
            granularity: Granularity::WholeRoot,
            blurb: "Downloaded wheel/sdist cache; pip re-downloads as needed.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "go-build-cache",
            label: "Go Build Cache",
            roots: vec![home.join("Library/Caches/go-build")],
            safety: Safety::Regenerable,
            granularity: Granularity::WholeRoot,
            blurb: "Compiled package cache; go rebuilds as needed.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "gradle-caches",
            label: "Gradle Caches",
            roots: vec![home.join(".gradle/caches")],
            safety: Safety::Regenerable,
            granularity: Granularity::WholeRoot,
            blurb: "Downloaded dependency/build cache; Gradle re-fetches as needed.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "library-logs",
            label: "Application Logs",
            roots: vec![home.join("Library/Logs")],
            safety: Safety::Regenerable,
            granularity: Granularity::Children,
            blurb: "Diagnostic logs from apps and the system; safe to clear.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "app-caches",
            label: "Per-App Caches",
            roots: vec![home.join("Library/Caches")],
            safety: Safety::Regenerable,
            granularity: Granularity::Children,
            blurb: "Clearing a cache logs nobody out; it just slows that app's next launch while it rebuilds.",
            refuse_delete: false,
        },
        // ---- Tier B: review required, never bulk-selected ----
        ScanTarget {
            id: "ios-backups",
            label: "iOS Device Backups",
            roots: vec![home.join("Library/Application Support/MobileSync/Backup")],
            safety: Safety::ReviewRequired,
            granularity: Granularity::Children,
            blurb: "May be the only backup of an iPhone/iPad. Review each backup's device and date before deleting.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "xcode-archives",
            label: "Xcode Archives",
            roots: vec![home.join("Library/Developer/Xcode/Archives")],
            safety: Safety::ReviewRequired,
            granularity: Granularity::Children,
            blurb: "Archived builds you may need for re-submission or crash symbolication.",
            refuse_delete: false,
        },
        ScanTarget {
            id: "downloads-stale",
            label: "Downloads (old & large)",
            roots: vec![home.join("Downloads")],
            safety: Safety::ReviewRequired,
            granularity: Granularity::Children,
            blurb: "User data. Only files older than 90 days and larger than 100MB are surfaced; never a Clear All.",
            refuse_delete: false,
        },
        // ---- Tier C: refused outright, but still measured and explained ----
        ScanTarget {
            id: "docker-data",
            label: "Docker (Docker Desktop)",
            roots: vec![home.join("Library/Containers/com.docker.docker")],
            safety: Safety::NeverTouch,
            granularity: Granularity::WholeRoot,
            blurb: "This is a sparse Docker.raw disk image. Deleting it destroys every image, container, and volume. Run `docker system prune` or use Docker Desktop's own cleanup instead.",
            refuse_delete: true,
        },
        ScanTarget {
            id: "icloud-drive",
            label: "iCloud Drive",
            roots: vec![home.join("Library/Mobile Documents")],
            safety: Safety::NeverTouch,
            granularity: Granularity::WholeRoot,
            blurb: "Deleting here removes files from iCloud for every device, not just locally. Never touched by this app.",
            refuse_delete: true,
        },
        ScanTarget {
            id: "photos-library",
            label: "Photos Library",
            roots: vec![home.join("Pictures/Photos Library.photoslibrary")],
            safety: Safety::NeverTouch,
            granularity: Granularity::WholeRoot,
            blurb: "Your photo library. Never touched by this app.",
            refuse_delete: true,
        },
    ]
}

/// Targets whose `safety` is `Regenerable` — the set that is default-selected
/// in the UI's "select safe items" action.
pub fn default_selected<'a>(targets: &'a [ScanTarget]) -> Vec<&'a ScanTarget> {
    targets.iter().filter(|t| t.safety == Safety::Regenerable).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_c_targets_all_refuse_delete() {
        let home = PathBuf::from("/Users/testuser");
        let catalog = build_catalog(&home);
        for t in catalog.iter().filter(|t| t.safety == Safety::NeverTouch) {
            assert!(t.refuse_delete, "{} must refuse_delete", t.id);
        }
    }

    #[test]
    fn no_duplicate_target_ids() {
        let home = PathBuf::from("/Users/testuser");
        let catalog = build_catalog(&home);
        let mut ids: Vec<&str> = catalog.iter().map(|t| t.id).collect();
        let before = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), before, "duplicate target id in catalog");
    }

    #[test]
    fn default_selected_only_includes_regenerable() {
        let home = PathBuf::from("/Users/testuser");
        let catalog = build_catalog(&home);
        let selected = default_selected(&catalog);
        assert!(selected.iter().all(|t| t.safety == Safety::Regenerable));
        assert!(!selected.is_empty());
    }
}
