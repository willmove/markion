//! Change-driven Git details refresh: what counts as a Git-relevant file event,
//! when a repository handle can be reused, and which content hashes can be
//! reused between refreshes. Everything here is cheap metadata work; Git
//! processes stay in the background refresh task.
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use markion::DocumentInstanceId;
use markion_git_sync::{GitCommandRunner, GitRepository, RepositoryIdentity};

/// Fallback refresh interval while the window is active and the watcher works.
pub(super) const GIT_FALLBACK_REFRESH: Duration = Duration::from_secs(60);
/// Fallback interval while the watcher failed and the file tree polls.
pub(super) const GIT_FALLBACK_REFRESH_POLLING: Duration = Duration::from_secs(15);
/// Granularity of the fallback timer loop.
pub(super) const GIT_FALLBACK_TICK: Duration = Duration::from_secs(5);

/// Whether a watcher event path should trigger a Git details refresh. Worktree
/// content changes count (including data modifications, which the file tree
/// ignores); inside `.git` only the files that describe repository state do.
pub(super) fn git_watch_path_is_relevant(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return true;
    };
    let mut components = relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => name.to_str(),
            _ => None,
        });
    let Some(first) = components.next() else {
        return true;
    };
    if relative
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".lock"))
    {
        return false;
    }
    if first != ".git" {
        return true;
    }
    match components.next() {
        Some("HEAD" | "index" | "MERGE_HEAD" | "packed-refs" | "refs") => true,
        _ => false,
    }
}

/// Size and modification time of the repository control files whose change
/// can alter discovery or the memoized capability probe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ControlFingerprint(Vec<Option<(u64, Option<SystemTime>)>>);

impl ControlFingerprint {
    pub(super) fn of(identity: &RepositoryIdentity) -> Self {
        let paths = [
            identity.git_dir.join("HEAD"),
            identity.git_dir.join("index"),
            identity.git_dir.join("config"),
            identity.common_dir.join("config"),
            identity.git_dir.join("shallow"),
            identity.git_dir.join("info").join("sparse-checkout"),
            identity.worktree_root.join(".gitmodules"),
        ];
        Self(
            paths
                .iter()
                .map(|path| {
                    std::fs::metadata(path)
                        .ok()
                        .map(|metadata| (metadata.len(), metadata.modified().ok()))
                })
                .collect(),
        )
    }
}

struct CachedRepository {
    workspace: PathBuf,
    executable: String,
    repository: GitRepository,
    fingerprint: ControlFingerprint,
}

/// The repository handle reused by consecutive refreshes. Write operations
/// never use it; they always discover a fresh instance.
#[derive(Clone, Default)]
pub(super) struct RepositoryCache(Arc<Mutex<Option<CachedRepository>>>);

impl RepositoryCache {
    /// Reuse the cached handle when the workspace, executable and control
    /// files are unchanged; otherwise discover again and remember the result.
    pub(super) fn get_or_discover(
        &self,
        workspace: &Path,
        executable: &str,
        runner: impl FnOnce() -> GitCommandRunner,
    ) -> Result<GitRepository, String> {
        let mut cached = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = cached.as_ref()
            && entry.workspace == workspace
            && entry.executable == executable
            && ControlFingerprint::of(entry.repository.identity()) == entry.fingerprint
        {
            return Ok(entry.repository.clone());
        }
        *cached = None;
        let repository =
            GitRepository::discover(runner(), workspace).map_err(|error| error.to_string())?;
        *cached = Some(CachedRepository {
            workspace: workspace.to_path_buf(),
            executable: executable.to_owned(),
            fingerprint: ControlFingerprint::of(repository.identity()),
            repository: repository.clone(),
        });
        Ok(repository)
    }
}

/// Content hashes reused between refreshes: on-disk files by size and mtime,
/// dirty buffers by document instance and version.
#[derive(Clone, Default)]
pub(super) struct FingerprintCache(Arc<Mutex<FingerprintEntries>>);

#[derive(Default)]
struct FingerprintEntries {
    files: HashMap<PathBuf, (u64, Option<SystemTime>, String)>,
    buffers: HashMap<(DocumentInstanceId, u64), String>,
}

impl FingerprintCache {
    pub(super) fn file(&self, absolute: &Path, compute: impl FnOnce() -> String) -> String {
        let stamp = std::fs::metadata(absolute)
            .ok()
            .map(|metadata| (metadata.len(), metadata.modified().ok()));
        let entries = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if let (Some((len, modified)), Some((cached_len, cached_modified, fingerprint))) =
            (stamp, entries.files.get(absolute))
            && len == *cached_len
            && modified == *cached_modified
        {
            return fingerprint.clone();
        }
        drop(entries);
        let fingerprint = compute();
        let mut entries = self.0.lock().unwrap_or_else(|error| error.into_inner());
        match stamp {
            Some((len, modified)) => {
                entries
                    .files
                    .insert(absolute.to_path_buf(), (len, modified, fingerprint.clone()));
            }
            None => {
                entries.files.remove(absolute);
            }
        }
        fingerprint
    }

    pub(super) fn buffer(&self, key: (DocumentInstanceId, u64)) -> Option<String> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .buffers
            .get(&key)
            .cloned()
    }

    /// Remember a dirty buffer hash, keeping only the newest version per document.
    pub(super) fn remember_buffer(&self, key: (DocumentInstanceId, u64), fingerprint: String) {
        let mut entries = self.0.lock().unwrap_or_else(|error| error.into_inner());
        entries
            .buffers
            .retain(|(instance, _), _| *instance != key.0);
        entries.buffers.insert(key, fingerprint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_watch_filter_keeps_content_and_state_files_only() {
        let root = Path::new("/notes");
        for relevant in [
            "/notes/a.md",
            "/notes/sub/b.png",
            "/notes/.git/HEAD",
            "/notes/.git/index",
            "/notes/.git/MERGE_HEAD",
            "/notes/.git/packed-refs",
            "/notes/.git/refs/heads/main",
            "/elsewhere/file",
        ] {
            assert!(
                git_watch_path_is_relevant(root, Path::new(relevant)),
                "{relevant}"
            );
        }
        for ignored in [
            "/notes/.git/objects/ab/cdef",
            "/notes/.git/logs/HEAD",
            "/notes/.git/index.lock",
            "/notes/.git/refs/heads/main.lock",
            "/notes/.git/FETCH_HEAD",
            "/notes/draft.md.lock",
        ] {
            assert!(
                !git_watch_path_is_relevant(root, Path::new(ignored)),
                "{ignored}"
            );
        }
    }

    #[test]
    fn file_fingerprints_are_reused_until_size_or_mtime_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        std::fs::write(&path, "one").unwrap();
        let cache = FingerprintCache::default();
        let computed = std::cell::Cell::new(0);
        let hash = |cache: &FingerprintCache| {
            cache.file(&path, || {
                computed.set(computed.get() + 1);
                format!("hash-{}", std::fs::read_to_string(&path).unwrap())
            })
        };
        assert_eq!(hash(&cache), "hash-one");
        assert_eq!(hash(&cache), "hash-one");
        assert_eq!(computed.get(), 1);
        std::fs::write(&path, "three").unwrap();
        assert_eq!(hash(&cache), "hash-three");
        assert_eq!(computed.get(), 2);
    }

    #[test]
    fn buffer_fingerprints_keep_only_the_newest_version() {
        let cache = FingerprintCache::default();
        let document = markion::MarkdownDocument::new().instance_id();
        cache.remember_buffer((document, 1), "v1".into());
        assert_eq!(cache.buffer((document, 1)).as_deref(), Some("v1"));
        cache.remember_buffer((document, 2), "v2".into());
        assert_eq!(cache.buffer((document, 1)), None);
        assert_eq!(cache.buffer((document, 2)).as_deref(), Some("v2"));
    }
}
