//! Same-directory atomic file replacement.

use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock, Mutex, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

/// Document writers share the destination lock from the freshness check
/// through replacement and disk identity capture.
pub(crate) fn with_document_write_lock<T>(path: &Path, write: impl FnOnce() -> T) -> T {
    type WriteLocks = Mutex<HashMap<PathBuf, Weak<Mutex<()>>>>;
    static ACTIVE: LazyLock<WriteLocks> = LazyLock::new(Mutex::default);

    let key = dunce::canonicalize(path).unwrap_or_else(|_| {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        match (dunce::canonicalize(parent), path.file_name()) {
            (Ok(parent), Some(name)) => parent.join(name),
            _ => std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf()),
        }
    });
    let lock = {
        let mut active = ACTIVE.lock().unwrap_or_else(|error| error.into_inner());
        active.retain(|_, lock| lock.strong_count() > 0);
        let entry = active.entry(key).or_default();
        entry.upgrade().unwrap_or_else(|| {
            let lock = Arc::new(Mutex::new(()));
            *entry = Arc::downgrade(&lock);
            lock
        })
    };
    let _guard = lock.lock().unwrap_or_else(|error| error.into_inner());
    write()
}

/// Durably writes `bytes` to a temporary sibling and atomically replaces
/// `path`. The destination is never truncated before the complete replacement
/// is ready, and the temporary file is removed on every error path.
pub fn atomic_write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    atomic_write_with(path.as_ref(), bytes.as_ref(), true)
}

/// Publishes a complete file without replacing an existing destination.
pub(crate) fn atomic_create(path: &Path, bytes: &[u8]) -> io::Result<()> {
    atomic_write_with(path, bytes, false)
}

fn atomic_write_with(path: &Path, bytes: &[u8], replace_existing: bool) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;

    let destination_permissions = if replace_existing {
        match fs::metadata(path) {
            Ok(metadata) => Some(metadata.permissions()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(err),
        }
    } else {
        None
    };

    let temp = unique_temp_path(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        if let Some(permissions) = destination_permissions {
            file.set_permissions(permissions)?;
            // Persist the metadata update before the replacement becomes visible.
            file.sync_all()?;
        }
        drop(file);
        commit_file(&temp, path, replace_existing)?;
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
fn commit_file(temp: &Path, target: &Path, replace_existing: bool) -> io::Result<()> {
    if replace_existing {
        fs::rename(temp, target)
    } else {
        fs::hard_link(temp, target)?;
        let _ = fs::remove_file(temp);
        Ok(())
    }
}

#[cfg(windows)]
fn commit_file(temp: &Path, target: &Path, replace_existing: bool) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::{
            Foundation::{ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS},
            Storage::FileSystem::{MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW},
        },
        core::{HRESULT, PCWSTR},
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
    let mut flags = MOVEFILE_WRITE_THROUGH;
    if replace_existing {
        flags |= MOVEFILE_REPLACE_EXISTING;
    }
    // SAFETY: both slices are NUL-terminated and remain alive for the call.
    unsafe { MoveFileExW(PCWSTR(temp.as_ptr()), PCWSTR(target.as_ptr()), flags) }.map_err(|err| {
        if !replace_existing
            && [ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS]
                .iter()
                .any(|code| HRESULT::from_win32(code.0) == err.code())
        {
            io::Error::new(io::ErrorKind::AlreadyExists, err)
        } else {
            io::Error::other(err)
        }
    })
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
    fn atomic_create_does_not_replace_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("retained.md");
        atomic_create(&path, b"old snapshot").unwrap();
        let error = atomic_create(&path, b"unrelated snapshot").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&path).unwrap(), b"old snapshot");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

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
        atomic_write(&target_dir, b"replacement").unwrap_err();
        assert!(target_dir.is_dir());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn atomic_write_preserves_destination_permission_flags() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("permissions.md");
        fs::write(&path, "old").unwrap();
        let before = fs::metadata(&path).unwrap().permissions();

        atomic_write(&path, b"new").unwrap();

        let after = fs::metadata(&path).unwrap().permissions();
        assert_eq!(after.readonly(), before.readonly());
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_preserves_existing_unix_mode_bits() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("private.md");
        fs::write(&path, "old").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        atomic_write(&path, b"new complete value").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new complete value");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
