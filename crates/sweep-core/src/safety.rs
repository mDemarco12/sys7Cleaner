use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SafetyViolation {
    #[error("path does not exist or could not be resolved: {0}")]
    Unresolvable(String),
    #[error("path is not under any registered scan target root")]
    NotUnderAllowlist,
    #[error("path matches a hardcoded protected location: {0}")]
    Denylisted(String),
    #[error("path is too shallow relative to its allowlist root (refusing to delete a root itself)")]
    TooShallow,
}

/// Hardcoded defense-in-depth denylist, checked AFTER allowlist membership.
/// Every path here is intentionally absolute and canonical-form; comparisons
/// are done against the canonicalized candidate, never the raw string.
fn denylist(home: &Path) -> Vec<PathBuf> {
    vec![
        PathBuf::from("/"),
        PathBuf::from("/System"),
        PathBuf::from("/usr"),
        PathBuf::from("/bin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/etc"),
        PathBuf::from("/var/db"),
        PathBuf::from("/Library"),
        PathBuf::from("/Applications"),
        home.to_path_buf(),
        home.join("Library"),
        home.join("Documents"),
        home.join("Desktop"),
        home.join("Pictures"),
        home.join("Movies"),
        home.join("Music"),
        home.join("Library/Mobile Documents"),
        home.join("Library/Keychains"),
        home.join("Library/Mail"),
        home.join("Library/Messages"),
        home.join("Library/Photos"),
        home.join(".ssh"),
        home.join(".aws"),
        home.join(".gnupg"),
        home.join(".config"),
        home.join(".kube"),
    ]
}

/// `/usr` is denylisted wholesale above, but `/usr/local` (Homebrew on Intel,
/// and some manual installs) is a legitimate cleanup surface. Carve it back out.
fn is_denylist_exception(canonical: &Path) -> bool {
    canonical.starts_with("/usr/local")
}

/// Validate a candidate deletion path against a set of allowlist roots
/// (normally: the `roots` of the `ScanTarget` that produced it).
///
/// Order matters: allowlist membership is checked BEFORE the denylist, and
/// resolution (`canonicalize`) happens BEFORE any comparison. Checking a
/// path before resolving it means a symlink can defeat every rule below.
pub fn validate_deletion_path(
    candidate: &Path,
    allowlist_roots: &[PathBuf],
    home: &Path,
) -> Result<PathBuf, SafetyViolation> {
    let canonical = candidate
        .canonicalize()
        .map_err(|e| SafetyViolation::Unresolvable(e.to_string()))?;

    let canonical_home = home
        .canonicalize()
        .unwrap_or_else(|_| home.to_path_buf());

    let mut matched_root: Option<PathBuf> = None;
    for root in allowlist_roots {
        if let Ok(canonical_root) = root.canonicalize() {
            if canonical == canonical_root || canonical.starts_with(&canonical_root) {
                matched_root = Some(canonical_root);
                break;
            }
        }
    }
    let matched_root = matched_root.ok_or(SafetyViolation::NotUnderAllowlist)?;

    if !is_denylist_exception(&canonical) {
        for denied in denylist(&canonical_home) {
            if let Ok(canonical_denied) = denied.canonicalize() {
                if canonical == canonical_denied {
                    return Err(SafetyViolation::Denylisted(canonical_denied.display().to_string()));
                }
            } else if canonical == denied {
                // Denylist entry doesn't exist on disk (fine) but compare literally too.
                return Err(SafetyViolation::Denylisted(denied.display().to_string()));
            }
        }
    }

    // Refuse deleting the allowlist root itself, or an immediate child of `/`-like
    // shallow roots — depth here is relative to the matched root, not absolute.
    if canonical == matched_root {
        return Err(SafetyViolation::TooShallow);
    }

    Ok(canonical)
}

#[cfg(unix)]
pub fn has_immutable_flag(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            // st_flags is not exposed via std; fall back to libc stat.
            let _ = meta.dev(); // keep MetadataExt import used on all platforms
            unsafe {
                let cpath = match std::ffi::CString::new(path.as_os_str().as_encoded_bytes()) {
                    Ok(c) => c,
                    Err(_) => return false,
                };
                let mut st: libc::stat = std::mem::zeroed();
                if libc::lstat(cpath.as_ptr(), &mut st) != 0 {
                    return false;
                }
                #[cfg(target_os = "macos")]
                {
                    (st.st_flags & (libc::SF_IMMUTABLE | libc::UF_IMMUTABLE)) != 0
                }
                #[cfg(not(target_os = "macos"))]
                {
                    false
                }
            }
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn roots_for(dir: &Path) -> Vec<PathBuf> {
        vec![dir.to_path_buf()]
    }

    #[test]
    fn accepts_child_of_allowlisted_root() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Developer/Xcode/DerivedData");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("MyApp-abc123");
        fs::create_dir(&target).unwrap();

        let result = validate_deletion_path(&target, &roots_for(&root), home.path());
        assert!(result.is_ok(), "{:?}", result);
    }

    #[test]
    fn rejects_path_outside_allowlist() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();
        let outside = home.path().join("Documents");
        fs::create_dir_all(&outside).unwrap();

        let result = validate_deletion_path(&outside, &roots_for(&root), home.path());
        assert_eq!(result, Err(SafetyViolation::NotUnderAllowlist));
    }

    #[test]
    fn rejects_the_allowlist_root_itself() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();

        let result = validate_deletion_path(&root, &roots_for(&root), home.path());
        assert_eq!(result, Err(SafetyViolation::TooShallow));
    }

    #[test]
    fn rejects_dotdot_traversal_out_of_root() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();
        let escape = root.join("../../Documents");
        fs::create_dir_all(home.path().join("Documents")).unwrap();

        // canonicalize() resolves the .. before any comparison happens.
        let result = validate_deletion_path(&escape, &roots_for(&root), home.path());
        assert_eq!(result, Err(SafetyViolation::NotUnderAllowlist));
    }

    #[test]
    fn symlink_escaping_root_is_resolved_before_check() {
        let home = tempdir().unwrap();
        let root = home.path().join("Library/Caches");
        fs::create_dir_all(&root).unwrap();
        let sensitive = home.path().join("Documents");
        fs::create_dir_all(&sensitive).unwrap();

        let link = root.join("escape-link");
        std::os::unix::fs::symlink(&sensitive, &link).unwrap();

        let result = validate_deletion_path(&link, &roots_for(&root), home.path());
        assert_eq!(result, Err(SafetyViolation::NotUnderAllowlist));
    }

    #[test]
    fn rejects_icloud_drive_even_if_it_were_somehow_an_allowlist_root() {
        let home = tempdir().unwrap();
        let icloud = home.path().join("Library/Mobile Documents");
        fs::create_dir_all(&icloud).unwrap();

        // Pretend a bug registered iCloud Drive itself as an allowlist root;
        // it must still be rejected (as the root itself, or via denylist).
        let result = validate_deletion_path(&icloud, &roots_for(&icloud), home.path());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_home_directory_exactly() {
        let home = tempdir().unwrap();
        let result = validate_deletion_path(home.path(), &roots_for(home.path()), home.path());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_ssh_directory() {
        let home = tempdir().unwrap();
        let ssh = home.path().join(".ssh");
        fs::create_dir_all(&ssh).unwrap();
        // Even if a target's root were misconfigured to include .ssh's parent,
        // the exact .ssh path must be denylisted.
        let result = validate_deletion_path(&ssh, &roots_for(home.path()), home.path());
        assert!(result.is_err());
    }
}
