use crate::model::Arch;
use goblin::mach::{Mach, MachO, MultiArch};
use std::fs;
use std::path::Path;

const CPU_TYPE_X86_64: u32 = 0x0100_0007;
const CPU_TYPE_ARM64: u32 = 0x0100_000C;

/// Classify a single Mach-O (or fat) binary's architecture support.
///
/// Deliberately does NOT shell out to `lipo`/`file`: spawning a process per
/// app is ~5-15ms and, across 200+ installed apps, adds seconds of latency
/// for work that takes microseconds done directly. It also reintroduces the
/// shell-argument class of bug being removed from the reference prototype's
/// `os.system(f'rm -rf "{item}"')`.
pub fn classify_binary(path: &Path) -> Arch {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return Arch::Unknown,
    };
    classify_bytes(&bytes)
}

fn classify_bytes(bytes: &[u8]) -> Arch {
    if bytes.len() < 4 {
        return Arch::Unknown;
    }

    match goblin::mach::Mach::parse(bytes) {
        Ok(Mach::Binary(macho)) => classify_thin(&macho),
        Ok(Mach::Fat(fat)) => classify_fat(&fat, bytes),
        Err(_) => Arch::Unknown,
    }
}

fn classify_thin(macho: &MachO) -> Arch {
    let cputype = macho.header.cputype as u32;
    if !macho.is_64 {
        return Arch::Dead;
    }
    match cputype {
        CPU_TYPE_X86_64 => Arch::IntelOnly,
        CPU_TYPE_ARM64 => Arch::AppleSiliconOnly,
        _ => Arch::Unknown,
    }
}

fn classify_fat(fat: &MultiArch, bytes: &[u8]) -> Arch {
    let mut has_x86_64 = false;
    let mut has_arm64 = false;
    let mut has_any_64 = false;

    for i in 0..fat.narches {
        let arch = match fat.get(i) {
            Ok(a) => a,
            Err(_) => continue,
        };
        match arch {
            goblin::mach::SingleArch::MachO(macho) => {
                has_any_64 = true;
                let cputype = macho.header.cputype as u32;
                if cputype == CPU_TYPE_X86_64 {
                    has_x86_64 = true;
                } else if cputype == CPU_TYPE_ARM64 {
                    has_arm64 = true;
                }
            }
            goblin::mach::SingleArch::Archive(_) => {}
        }
    }
    let _ = bytes;

    match (has_x86_64, has_arm64, has_any_64) {
        (true, true, _) => Arch::Universal,
        (true, false, _) => Arch::IntelOnly,
        (false, true, _) => Arch::AppleSiliconOnly,
        (false, false, false) => Arch::Dead,
        (false, false, true) => Arch::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hand-synthesized headers, big-endian for fat, matching Apple's
    // <mach-o/fat.h> and <mach-o/loader.h> layouts.

    fn thin_header(magic: u32, cputype: u32) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&magic.to_le_bytes()); // MH_MAGIC_64 is native-endian per magic choice
        v.extend_from_slice(&cputype.to_le_bytes());
        v.extend_from_slice(&0u32.to_le_bytes()); // cpusubtype
        v.extend_from_slice(&2u32.to_le_bytes()); // filetype = MH_EXECUTE
        v.extend_from_slice(&0u32.to_le_bytes()); // ncmds
        v.extend_from_slice(&0u32.to_le_bytes()); // sizeofcmds
        v.extend_from_slice(&0u32.to_le_bytes()); // flags
        v.extend_from_slice(&0u32.to_le_bytes()); // reserved (64-bit only)
        v
    }

    #[test]
    fn thin_x86_64_classifies_intel_only() {
        // MH_MAGIC_64 = 0xfeedfacf (little-endian on disk for a little-endian host header)
        let bytes = thin_header(0xfeed_facf, CPU_TYPE_X86_64);
        assert_eq!(classify_bytes(&bytes), Arch::IntelOnly);
    }

    #[test]
    fn thin_arm64_classifies_apple_silicon_only() {
        let bytes = thin_header(0xfeed_facf, CPU_TYPE_ARM64);
        assert_eq!(classify_bytes(&bytes), Arch::AppleSiliconOnly);
    }

    #[test]
    fn too_short_buffer_is_unknown_not_a_panic() {
        assert_eq!(classify_bytes(&[0x01, 0x02]), Arch::Unknown);
    }

    #[test]
    fn java_class_magic_is_not_misparsed_as_fat() {
        // 0xCAFEBABE is FAT_MAGIC big-endian AND the Java .class file magic.
        // A real .class file's next 4 bytes are minor/major version (small
        // integers), which must not be misread as a plausible nfat_arch.
        let mut bytes = vec![0xCA, 0xFE, 0xBA, 0xBE]; // magic
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x3D]); // minor=0, major=61 (Java 17)
        bytes.extend_from_slice(&[0u8; 100]); // padding so goblin has enough to fail on cleanly
        let result = classify_bytes(&bytes);
        // Must not panic and must not report a real architecture.
        assert!(matches!(result, Arch::Unknown | Arch::Dead));
    }

    #[test]
    fn empty_bytes_is_unknown() {
        assert_eq!(classify_bytes(&[]), Arch::Unknown);
    }
}
