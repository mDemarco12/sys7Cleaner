use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsOpError {
    #[error("trash failed for {0}: {1}")]
    Trash(PathBuf, String),
    #[error("remove failed for {0}: {1}")]
    Remove(PathBuf, String),
    #[error("metadata read failed for {0}: {1}")]
    Metadata(PathBuf, String),
}

pub type FsOpResult<T> = Result<T, FsOpError>;

/// Every deletion path in the app goes through this trait. Dry run, production,
/// and tests all call `reclaim::execute` with a different implementation, so a
/// dry run genuinely proves what the real run would do.
pub trait FileOps: Send + Sync {
    fn trash(&self, p: &Path) -> FsOpResult<()>;
    fn remove(&self, p: &Path) -> FsOpResult<()>;
    fn metadata(&self, p: &Path) -> FsOpResult<fs::Metadata>;
}

/// Moves to the Finder Trash (`NSFileManager trashItemAtURL:` under the hood)
/// or, for permanent mode, actually removes the file/directory tree.
pub struct RealFileOps;

impl FileOps for RealFileOps {
    fn trash(&self, p: &Path) -> FsOpResult<()> {
        trash::delete(p).map_err(|e| FsOpError::Trash(p.to_path_buf(), e.to_string()))
    }

    fn remove(&self, p: &Path) -> FsOpResult<()> {
        let meta = self.metadata(p)?;
        if meta.is_dir() {
            fs::remove_dir_all(p).map_err(|e| FsOpError::Remove(p.to_path_buf(), e.to_string()))
        } else {
            fs::remove_file(p).map_err(|e| FsOpError::Remove(p.to_path_buf(), e.to_string()))
        }
    }

    fn metadata(&self, p: &Path) -> FsOpResult<fs::Metadata> {
        fs::symlink_metadata(p).map_err(|e| FsOpError::Metadata(p.to_path_buf(), e.to_string()))
    }
}

/// Performs no filesystem mutation. Reads real metadata (so size estimates in
/// a dry-run report are accurate) but logs what it WOULD have done.
pub struct DryRunFileOps {
    pub log: Mutex<Vec<String>>,
}

impl DryRunFileOps {
    pub fn new() -> Self {
        DryRunFileOps { log: Mutex::new(Vec::new()) }
    }

    pub fn take_log(&self) -> Vec<String> {
        std::mem::take(&mut self.log.lock().unwrap())
    }
}

impl Default for DryRunFileOps {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOps for DryRunFileOps {
    fn trash(&self, p: &Path) -> FsOpResult<()> {
        self.log.lock().unwrap().push(format!("TRASH {}", p.display()));
        Ok(())
    }

    fn remove(&self, p: &Path) -> FsOpResult<()> {
        self.log.lock().unwrap().push(format!("REMOVE {}", p.display()));
        Ok(())
    }

    fn metadata(&self, p: &Path) -> FsOpResult<fs::Metadata> {
        fs::symlink_metadata(p).map_err(|e| FsOpError::Metadata(p.to_path_buf(), e.to_string()))
    }
}

/// Test double: records the exact set of paths that would be trashed/removed
/// without touching the filesystem at all (metadata calls also never hit disk).
/// Tests assert set equality against expectations.
pub struct RecordingFileOps {
    pub trashed: Mutex<Vec<PathBuf>>,
    pub removed: Mutex<Vec<PathBuf>>,
    pub fail_on: Vec<PathBuf>,
}

impl RecordingFileOps {
    pub fn new() -> Self {
        RecordingFileOps {
            trashed: Mutex::new(Vec::new()),
            removed: Mutex::new(Vec::new()),
            fail_on: Vec::new(),
        }
    }

    pub fn with_failures(fail_on: Vec<PathBuf>) -> Self {
        RecordingFileOps {
            trashed: Mutex::new(Vec::new()),
            removed: Mutex::new(Vec::new()),
            fail_on,
        }
    }
}

impl Default for RecordingFileOps {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOps for RecordingFileOps {
    fn trash(&self, p: &Path) -> FsOpResult<()> {
        if self.fail_on.contains(&p.to_path_buf()) {
            return Err(FsOpError::Trash(p.to_path_buf(), "simulated failure".into()));
        }
        self.trashed.lock().unwrap().push(p.to_path_buf());
        Ok(())
    }

    fn remove(&self, p: &Path) -> FsOpResult<()> {
        if self.fail_on.contains(&p.to_path_buf()) {
            return Err(FsOpError::Remove(p.to_path_buf(), "simulated failure".into()));
        }
        self.removed.lock().unwrap().push(p.to_path_buf());
        Ok(())
    }

    fn metadata(&self, p: &Path) -> FsOpResult<fs::Metadata> {
        // Fine to hit real metadata in tests — it's a read, not a mutation,
        // and tests operate on real tempdirs.
        fs::symlink_metadata(p).map_err(|e| FsOpError::Metadata(p.to_path_buf(), e.to_string()))
    }
}
