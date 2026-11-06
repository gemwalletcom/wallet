use std::fs::DirBuilder;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new() -> io::Result<Self> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let name = format!("gem-test-{}-{nanos}-{}", std::process::id(), NEXT_ID.fetch_add(1, Ordering::Relaxed));
        let path = std::env::temp_dir().join(name);
        let mut builder = DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder.create(&path)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::TempDir;

    #[test]
    fn test_temp_dir_is_unique_and_removed_on_drop() {
        let first = TempDir::new().unwrap();
        let second = TempDir::new().unwrap();
        assert_ne!(first.path(), second.path());
        assert!(first.path().is_dir());

        std::fs::write(first.path().join("keystore.json"), b"{}").unwrap();
        let path = first.path().to_path_buf();
        drop(first);
        assert!(!path.exists());
        assert!(second.path().is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn test_temp_dir_is_private() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        assert_eq!(std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777, 0o700);
    }
}
