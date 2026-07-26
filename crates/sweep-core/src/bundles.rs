use crate::macho::classify_binary;
use crate::model::{Arch, AppInfo};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Enumerate `.app` bundles directly under `/Applications` and classify each
/// by architecture. Deliberately skips `/System/Applications` (SIP-protected,
/// entirely Apple binaries — nothing actionable there).
///
/// Reads `Contents/Info.plist` with the `plist` crate rather than treating it
/// as text: most bundle Info.plists are *binary* plists, so a naive text
/// parse silently fails to extract `CFBundleExecutable` for the majority of
/// real apps.
pub fn scan_applications(applications_dir: &Path) -> Vec<AppInfo> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(applications_dir) {
        Ok(e) => e,
        Err(_) => return out,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("app") {
            continue;
        }
        if let Some(info) = classify_app_bundle(&path) {
            out.push(info);
        }
    }
    out
}

fn classify_app_bundle(bundle_path: &Path) -> Option<AppInfo> {
    let name = bundle_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| bundle_path.display().to_string());

    let exec_name = read_bundle_executable_name(bundle_path).unwrap_or_else(|| name.clone());
    let exec_path = bundle_path.join("Contents/MacOS").join(&exec_name);

    let arch = if exec_path.exists() {
        classify_binary(&exec_path)
    } else {
        Arch::Unknown
    };

    let last_used = std::fs::metadata(bundle_path)
        .ok()
        .and_then(|m| m.accessed().ok())
        .or_else(|| std::fs::metadata(bundle_path).ok().and_then(|m| m.modified().ok()));

    Some(AppInfo {
        bundle_path: bundle_path.to_path_buf(),
        name,
        arch,
        last_used,
    })
}

fn read_bundle_executable_name(bundle_path: &Path) -> Option<String> {
    let plist_path = bundle_path.join("Contents/Info.plist");
    let value: plist::Value = plist::Value::from_file(&plist_path).ok()?;
    value
        .as_dictionary()?
        .get("CFBundleExecutable")?
        .as_string()
        .map(|s| s.to_string())
}

/// Filter to apps worth surfacing as "Rosetta-dependent": Intel-only (or
/// still-launchable-but-legacy) AND not opened in a long time. Intel-only
/// does NOT mean deletable — those apps run fine under Rosetta 2 — so this
/// is framed as a review list, sorted oldest-last-used first, not a
/// bulk-delete candidate list.
pub fn rosetta_candidates(apps: &[AppInfo], staleness_threshold: std::time::Duration, now: SystemTime) -> Vec<AppInfo> {
    let mut candidates: Vec<AppInfo> = apps
        .iter()
        .filter(|a| a.arch == Arch::IntelOnly)
        .filter(|a| match a.last_used {
            Some(t) => now.duration_since(t).map(|d| d >= staleness_threshold).unwrap_or(false),
            None => true, // no signal at all — surface it for review rather than hide it
        })
        .cloned()
        .collect();

    candidates.sort_by_key(|a| a.last_used.unwrap_or(SystemTime::UNIX_EPOCH));
    candidates
}

/// Best-effort residue paths for a bundle identifier — always presented as a
/// SEPARATE, explicitly reviewed step; never auto-deleted alongside the app.
pub fn residue_paths(home: &Path, bundle_id: &str) -> Vec<PathBuf> {
    vec![
        home.join("Library/Application Support").join(bundle_id),
        home.join("Library/Preferences").join(format!("{bundle_id}.plist")),
        home.join("Library/Caches").join(bundle_id),
        home.join("Library/Saved Application State").join(format!("{bundle_id}.savedState")),
    ]
    .into_iter()
    .filter(|p| p.exists())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_fake_app(apps_dir: &Path, name: &str, plist_xml: &str, exec_bytes: &[u8]) -> PathBuf {
        let bundle = apps_dir.join(format!("{name}.app"));
        fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        fs::write(bundle.join("Contents/Info.plist"), plist_xml).unwrap();
        fs::write(bundle.join("Contents/MacOS").join(name), exec_bytes).unwrap();
        bundle
    }

    fn plist_for(exec: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{exec}</string>
</dict>
</plist>"#
        )
    }

    fn thin_arm64_header() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&0xfeed_facfu32.to_le_bytes());
        v.extend_from_slice(&0x0100_000Cu32.to_le_bytes()); // CPU_TYPE_ARM64
        v.extend_from_slice(&[0u8; 24]);
        v
    }

    #[test]
    fn reads_executable_name_from_binary_or_xml_plist() {
        let dir = tempdir().unwrap();
        let bundle = make_fake_app(dir.path(), "TestApp", &plist_for("TestApp"), &thin_arm64_header());
        let name = read_bundle_executable_name(&bundle).unwrap();
        assert_eq!(name, "TestApp");
    }

    #[test]
    fn classifies_apple_silicon_only_app() {
        let dir = tempdir().unwrap();
        make_fake_app(dir.path(), "TestApp", &plist_for("TestApp"), &thin_arm64_header());
        let apps = scan_applications(dir.path());
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].arch, Arch::AppleSiliconOnly);
    }

    #[test]
    fn skips_non_app_entries() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("not-an-app.txt"), "hello").unwrap();
        let apps = scan_applications(dir.path());
        assert_eq!(apps.len(), 0);
    }

    #[test]
    fn rosetta_candidates_excludes_apple_silicon_only() {
        let apps = vec![AppInfo {
            bundle_path: PathBuf::from("/Applications/Native.app"),
            name: "Native".into(),
            arch: Arch::AppleSiliconOnly,
            last_used: Some(SystemTime::UNIX_EPOCH),
        }];
        let now = SystemTime::now();
        let candidates = rosetta_candidates(&apps, std::time::Duration::from_secs(1), now);
        assert!(candidates.is_empty());
    }

    #[test]
    fn rosetta_candidates_includes_stale_intel_only() {
        let apps = vec![AppInfo {
            bundle_path: PathBuf::from("/Applications/OldApp.app"),
            name: "OldApp".into(),
            arch: Arch::IntelOnly,
            last_used: Some(SystemTime::UNIX_EPOCH),
        }];
        let now = SystemTime::now();
        let candidates = rosetta_candidates(&apps, std::time::Duration::from_secs(1), now);
        assert_eq!(candidates.len(), 1);
    }
}
