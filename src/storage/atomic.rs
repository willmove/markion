//! Same-directory atomic file replacement.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

/// Durably writes `bytes` to a temporary sibling and atomically replaces
/// `path`. The destination is never truncated before the complete replacement
/// is ready, and the temporary file is removed on every error path.
pub fn atomic_write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let path = path.as_ref();
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;

    let temp = unique_temp_path(path);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(bytes.as_ref())?;
        file.sync_all()?;
        drop(file);
        replace_file(&temp, path)?;
        sync_parent(parent);
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn unique_temp_path(path: &Path) -> PathBuf {
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("document");
    path.with_file_name(format!(".{name}.markion-{}-{id}.tmp", std::process::id()))
}

#[cfg(not(windows))]
fn replace_file(temp: &Path, target: &Path) -> io::Result<()> {
    fs::rename(temp, target)
}

#[cfg(windows)]
fn replace_file(temp: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        },
        core::PCWSTR,
    };

    let temp = temp
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let target = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // SAFETY: both slices are NUL-terminated and remain alive for the call.
    unsafe {
        MoveFileExW(
            PCWSTR(temp.as_ptr()),
            PCWSTR(target.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(io::Error::other)
}

#[cfg(unix)]
fn sync_parent(parent: &Path) {
    if let Ok(directory) = fs::File::open(parent) {
        let _ = directory.sync_all();
    }
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_creates_and_replaces_without_temp_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        atomic_write(&path, b"first").unwrap();
        atomic_write(&path, b"second complete value").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second complete value");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn atomic_write_keeps_destination_when_temp_cannot_be_created() {
        let dir = tempfile::tempdir().unwrap();
        let target_dir = dir.path().join("target");
        fs::create_dir(&target_dir).unwrap();
        let err = atomic_write(&target_dir, b"replacement").unwrap_err();
        assert!(matches!(
            err.kind(),
            io::ErrorKind::PermissionDenied | io::ErrorKind::Other
        ));
        assert!(target_dir.is_dir());
    }
}
