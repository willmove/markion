use std::{
    collections::{BTreeSet, HashMap},
    fs, io,
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::{
    ChangeKind, FileChange, GitObjectId, RepositoryIdentity, SyncTarget, persist::atomic_write,
};

pub const POLICY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NewFileClass {
    Markdown,
    Text,
    Image,
}

impl NewFileClass {
    pub fn classify(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "md" | "markdown" | "mdown" => Some(Self::Markdown),
            "txt" | "text" | "log" | "csv" | "tsv" | "org" | "rst" | "adoc" | "asciidoc" => {
                Some(Self::Text)
            }
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "svg" => {
                Some(Self::Image)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewFileRule {
    pub root: PathBuf,
    pub classes: BTreeSet<NewFileClass>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeException {
    pub path: PathBuf,
    pub content_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryPolicy {
    pub identity: RepositoryIdentity,
    pub target: SyncTarget,
    pub tracked_roots: Vec<PathBuf>,
    pub new_file_rules: Vec<NewFileRule>,
    pub message_template: String,
    pub include_device_name: bool,
    pub background_fetch: bool,
    #[serde(default)]
    pub last_confirmed_remote: Option<GitObjectId>,
    #[serde(default)]
    pub acknowledged_resource_omissions: Vec<ScopeException>,
    /// Extra new-file extensions the user chose to always synchronize
    /// (lowercase, without the dot), applied within the new-file roots.
    #[serde(default)]
    pub allowed_extensions: BTreeSet<String>,
    /// Held-back files the user approved once, bound to their content at
    /// approval time.
    #[serde(default)]
    pub included_once: Vec<ScopeException>,
}

impl RepositoryPolicy {
    pub fn validates_identity(&self, actual: &RepositoryIdentity) -> bool {
        self.identity == *actual
    }

    pub fn tracked_path_allowed(&self, path: &Path) -> bool {
        is_safe_relative(path)
            && self
                .tracked_roots
                .iter()
                .any(|root| path_in_root(path, root))
    }

    pub fn new_path_allowed(&self, path: &Path, ignored: bool, is_symlink: bool) -> bool {
        if ignored || is_symlink || !is_safe_relative(path) || contains_dot_component(path) {
            return false;
        }
        let class = NewFileClass::classify(path);
        let extra = file_extension(path)
            .is_some_and(|extension| self.allowed_extensions.contains(&extension));
        self.new_file_rules.iter().any(|rule| {
            path_in_root(path, &rule.root)
                && (extra || class.is_some_and(|class| rule.classes.contains(&class)))
        })
    }

    /// Split the current changes into what Sync Now commits automatically,
    /// what is held back for a decision, and operating-system artifacts that
    /// are ignored. Held-back changes never block the automatic ones.
    pub fn partition(&self, changes: &[FileChange], facts: &dyn ScopeFacts) -> ScopePartition {
        let mut partition = ScopePartition::default();
        for change in changes {
            let path = &change.path;
            if change.kind == ChangeKind::Untracked {
                if is_os_artifact(path) {
                    partition.ignored.push(path.clone());
                    continue;
                }
                let reason = if facts.is_nested_repository(path) {
                    Some(HeldBackReason::NestedRepository)
                } else if facts.is_symlink(path) {
                    Some(HeldBackReason::Symlink)
                } else if self.new_path_allowed(path, false, false) {
                    None
                } else if let Some(record) = self.included_once.iter().find(|r| r.path == *path) {
                    (facts.fingerprint(path).as_deref() != Some(&record.content_fingerprint))
                        .then_some(HeldBackReason::ChangedSinceApproval)
                } else if contains_dot_component(path) {
                    Some(HeldBackReason::Hidden)
                } else if self
                    .new_file_rules
                    .iter()
                    .any(|rule| path_in_root(path, &rule.root))
                {
                    Some(HeldBackReason::FileType {
                        extension: file_extension(path),
                    })
                } else {
                    Some(HeldBackReason::OutsideFolders)
                };
                match reason {
                    None => partition.automatic.push(path.clone()),
                    Some(reason) => partition.held_back.push(HeldBack {
                        path: path.clone(),
                        tracked: false,
                        reason,
                    }),
                }
            } else if change
                .original_path
                .as_ref()
                .is_some_and(|from| self.rename_needs_review(from, path))
            {
                partition.held_back.push(HeldBack {
                    path: path.clone(),
                    tracked: true,
                    reason: HeldBackReason::RenameAcrossScope,
                });
            } else if self.tracked_path_allowed(path) {
                partition.automatic.push(path.clone());
            } else {
                partition.held_back.push(HeldBack {
                    path: path.clone(),
                    tracked: true,
                    reason: HeldBackReason::OutsideFolders,
                });
            }
        }
        self.pair_moves(changes, facts, &mut partition);
        partition
    }

    /// Hold back a tracked deletion whose content exactly matches a held-back
    /// new file, so a move never propagates as a deletion alone.
    fn pair_moves(
        &self,
        changes: &[FileChange],
        facts: &dyn ScopeFacts,
        partition: &mut ScopePartition,
    ) {
        let new_files: Vec<PathBuf> = partition
            .held_back
            .iter()
            .filter(|held| !held.tracked)
            .map(|held| held.path.clone())
            .collect();
        let deletions: Vec<PathBuf> = changes
            .iter()
            .filter(|change| {
                change.kind == ChangeKind::Deleted && partition.automatic.contains(&change.path)
            })
            .map(|change| change.path.clone())
            .collect();
        if new_files.is_empty() || deletions.is_empty() {
            return;
        }
        let paired: Vec<(PathBuf, Option<PathBuf>)> = match (
            facts.worktree_blob_ids(&new_files),
            facts.index_blob_ids(&deletions),
        ) {
            (Some(new_ids), Some(deleted_ids)) => deletions
                .iter()
                .filter_map(|deleted| {
                    let id = deleted_ids.get(deleted)?;
                    let with = new_files
                        .iter()
                        .find(|candidate| new_ids.get(*candidate) == Some(id))?;
                    Some((deleted.clone(), Some(with.clone())))
                })
                .collect(),
            // Content cannot be compared: conservatively hold every deletion.
            _ => deletions
                .iter()
                .map(|deleted| (deleted.clone(), None))
                .collect(),
        };
        for (deleted, with) in paired {
            partition.automatic.retain(|path| *path != deleted);
            partition.held_back.push(HeldBack {
                path: deleted,
                tracked: true,
                reason: HeldBackReason::PairedMove { with },
            });
        }
    }

    /// Drop include-once approvals that no longer apply: the file was
    /// committed (now tracked) or no longer exists.
    pub fn prune_included_once(&mut self, still_pending: impl Fn(&Path) -> bool) {
        self.included_once
            .retain(|record| still_pending(&record.path));
    }

    pub fn decide_path(
        &self,
        path: &Path,
        kind: ChangeKind,
        ignored: bool,
        is_symlink: bool,
    ) -> PathDecision {
        if kind == ChangeKind::Untracked {
            if ignored {
                PathDecision::Ignored
            } else if self.new_path_allowed(path, false, is_symlink) {
                PathDecision::Automatic
            } else {
                PathDecision::Attention
            }
        } else if self.tracked_path_allowed(path) {
            PathDecision::Automatic
        } else {
            PathDecision::Attention
        }
    }

    pub fn rename_needs_review(&self, original: &Path, destination: &Path) -> bool {
        self.tracked_path_allowed(original) != self.tracked_path_allowed(destination)
    }

    pub fn render_message(
        &self,
        paths: &[PathBuf],
        device_name: Option<&str>,
        timestamp: SystemTime,
    ) -> String {
        let mut message = if self.message_template.trim().is_empty() {
            default_commit_message(paths, timestamp)
        } else {
            let (date, time) = local_timestamp_parts(timestamp);
            self.message_template
                .replace("{count}", &paths.len().to_string())
                .replace("{date}", &date)
                .replace("{time}", &time)
                .replace("{files}", &compact_file_list(paths))
        };
        if self.include_device_name
            && let Some(device) = device_name.filter(|device| !device.trim().is_empty())
        {
            message.push_str(" from ");
            message.push_str(device.trim());
        }
        message
    }
}

/// Per-path facts the policy needs but cannot compute itself. Implemented by
/// [`crate::GitRepository`]; tests can supply fakes.
pub trait ScopeFacts {
    fn is_symlink(&self, path: &Path) -> bool;
    fn is_nested_repository(&self, path: &Path) -> bool;
    /// SHA-256 content fingerprint, as recorded by include-once approvals.
    fn fingerprint(&self, path: &Path) -> Option<String>;
    /// Blob ids the worktree files would get if added. `None` when they cannot
    /// be computed, in which case moves are paired conservatively.
    fn worktree_blob_ids(&self, paths: &[PathBuf]) -> Option<HashMap<PathBuf, String>>;
    /// Blob ids recorded in the index for tracked paths.
    fn index_blob_ids(&self, paths: &[PathBuf]) -> Option<HashMap<PathBuf, String>>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScopePartition {
    /// Committed by Sync Now and Commit Locally without further review.
    pub automatic: Vec<PathBuf>,
    /// Listed as not included in sync; never committed automatically.
    pub held_back: Vec<HeldBack>,
    /// Untracked operating-system artifacts that are never proposed.
    pub ignored: Vec<PathBuf>,
}

impl ScopePartition {
    pub fn held_back_new(&self) -> impl Iterator<Item = &HeldBack> {
        self.held_back.iter().filter(|held| !held.tracked)
    }

    pub fn held_back_tracked(&self) -> impl Iterator<Item = &HeldBack> {
        self.held_back.iter().filter(|held| held.tracked)
    }

    pub fn is_held_back(&self, path: &Path) -> bool {
        self.held_back.iter().any(|held| held.path == path)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeldBack {
    pub path: PathBuf,
    /// Whether the path is tracked (a modification, deletion or rename)
    /// rather than a new file.
    pub tracked: bool,
    pub reason: HeldBackReason,
}

impl HeldBack {
    /// Whether the per-file decisions (include once, always allow the type,
    /// exclude on this device) can be offered for this change.
    pub fn decidable(&self) -> bool {
        !self.tracked
            && !matches!(
                self.reason,
                HeldBackReason::Symlink | HeldBackReason::NestedRepository
            )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeldBackReason {
    /// A hidden file or a file under a hidden folder.
    Hidden,
    /// A file type the notes policy does not synchronize by default.
    FileType {
        extension: Option<String>,
    },
    /// Outside the folders approved for synchronization.
    OutsideFolders,
    Symlink,
    NestedRepository,
    /// A tracked deletion whose content moved to a held-back new file
    /// (`None` when the content could not be compared).
    PairedMove {
        with: Option<PathBuf>,
    },
    /// A rename across the boundary of the approved folders.
    RenameAcrossScope,
    /// Approved once, but the file changed after the approval.
    ChangedSinceApproval,
}

/// Untracked files that operating systems and office suites create next to
/// user content. They are never proposed as notes.
pub fn is_os_artifact(path: &Path) -> bool {
    const DIRECTORIES: &[&str] = &[
        ".appledouble",
        ".spotlight-v100",
        ".trashes",
        ".fseventsd",
        ".temporaryitems",
        "$recycle.bin",
    ];
    const FILES: &[&str] = &[
        ".ds_store",
        "thumbs.db",
        "ehthumbs.db",
        "ehthumbs_vista.db",
        "desktop.ini",
        "icon\r",
    ];
    let mut components = path.components().filter_map(|component| match component {
        Component::Normal(name) => Some(name.to_string_lossy().to_lowercase()),
        _ => None,
    });
    let Some(name) = path
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
    else {
        return false;
    };
    components.any(|component| DIRECTORIES.contains(&component.as_str()))
        || FILES.contains(&name.as_str())
        || name.starts_with("._")
        || name.starts_with("~$")
        || (name.starts_with(".~lock.") && name.ends_with('#'))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathDecision {
    Automatic,
    Attention,
    Ignored,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewPlan {
    pub paths: Vec<(PathBuf, String)>,
}

impl ReviewPlan {
    pub fn still_matches(&self, current: impl IntoIterator<Item = (PathBuf, String)>) -> bool {
        let mut expected = self.paths.clone();
        let mut current: Vec<_> = current.into_iter().collect();
        expected.sort();
        current.sort();
        expected == current
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPolicies {
    pub schema_version: u32,
    #[serde(default)]
    pub repositories: Vec<RepositoryPolicy>,
}

impl Default for SyncPolicies {
    fn default() -> Self {
        Self::empty()
    }
}

impl SyncPolicies {
    pub fn empty() -> Self {
        Self {
            schema_version: POLICY_SCHEMA_VERSION,
            repositories: Vec::new(),
        }
    }

    pub fn upsert(&mut self, policy: RepositoryPolicy) {
        self.repositories
            .retain(|current| current.identity != policy.identity);
        self.repositories.push(policy);
    }

    pub fn disconnect(&mut self, identity: &RepositoryIdentity) -> Option<RepositoryPolicy> {
        let index = self
            .repositories
            .iter()
            .position(|policy| policy.identity == *identity)?;
        Some(self.repositories.remove(index))
    }

    /// Rebind a policy only after the caller has explicitly selected and
    /// rediscovered the replacement worktree. Scope and target remain visible
    /// for review; no path-based guess silently reconnects a repository.
    pub fn reconnect(
        &mut self,
        expected: &RepositoryIdentity,
        replacement: RepositoryIdentity,
    ) -> bool {
        let Some(policy) = self
            .repositories
            .iter_mut()
            .find(|policy| policy.identity == *expected)
        else {
            return false;
        };
        policy.identity = replacement;
        true
    }
}

#[derive(Clone, Debug)]
pub struct PolicyStore {
    path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("failed to read/write Git sync policy: {0}")]
    Io(#[from] io::Error),
    #[error("invalid Git sync policy: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize Git sync policy: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("unsupported Git sync policy schema {0}")]
    UnsupportedVersion(u32),
}

impl PolicyStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Result<SyncPolicies, PolicyError> {
        match fs::read_to_string(&self.path) {
            Ok(text) => {
                let mut policies: SyncPolicies = toml::from_str(&text)?;
                if policies.schema_version != POLICY_SCHEMA_VERSION {
                    return Err(PolicyError::UnsupportedVersion(policies.schema_version));
                }
                for policy in &mut policies.repositories {
                    normalize_identity_paths(&mut policy.identity);
                }
                Ok(policies)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(SyncPolicies::empty()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, policies: &SyncPolicies) -> Result<(), PolicyError> {
        if policies.schema_version != POLICY_SCHEMA_VERSION {
            return Err(PolicyError::UnsupportedVersion(policies.schema_version));
        }
        let text = toml::to_string_pretty(policies)?;
        atomic_write(&self.path, text.as_bytes())?;
        Ok(())
    }

    pub fn update(
        &self,
        edit: impl FnOnce(&mut SyncPolicies),
    ) -> Result<SyncPolicies, PolicyError> {
        let mut policies = self.load()?;
        edit(&mut policies);
        self.save(&policies)?;
        Ok(policies)
    }
}

fn normalize_identity_paths(identity: &mut RepositoryIdentity) {
    for path in [
        &mut identity.worktree_root,
        &mut identity.git_dir,
        &mut identity.common_dir,
    ] {
        if let Ok(normalized) = dunce::canonicalize(&*path) {
            *path = normalized;
        }
    }
}

/// Maximum number of file names listed before the `+N more` tail.
const MESSAGE_FILE_NAME_LIMIT: usize = 3;
/// Subject-length cap the default message enforces by dropping whole names.
const DEFAULT_MESSAGE_SUBJECT_CAP: usize = 72;

pub fn default_commit_message(paths: &[PathBuf], timestamp: SystemTime) -> String {
    let (date, time) = local_timestamp_parts(timestamp);
    let prefix = format!("Sync notes {date} {time}");
    let names = deduped_file_names(paths);
    if names.is_empty() {
        return prefix;
    }
    let mut shown = MESSAGE_FILE_NAME_LIMIT.min(names.len());
    loop {
        let tail = names.len() - shown;
        let mut candidate = format!("{prefix}: ");
        if shown > 0 {
            candidate.push_str(&names[..shown].join(", "));
            if tail > 0 {
                candidate.push_str(", ");
            }
        }
        if tail > 0 {
            candidate.push_str(&format!("+{tail} more"));
        }
        if candidate.chars().count() <= DEFAULT_MESSAGE_SUBJECT_CAP || shown == 0 {
            return candidate;
        }
        shown -= 1;
    }
}

fn compact_file_list(paths: &[PathBuf]) -> String {
    let names = deduped_file_names(paths);
    if names.is_empty() {
        return String::new();
    }
    let shown = MESSAGE_FILE_NAME_LIMIT.min(names.len());
    let tail = names.len() - shown;
    let mut list = names[..shown].join(", ");
    if tail > 0 {
        if !list.is_empty() {
            list.push_str(", ");
        }
        list.push_str(&format!("+{tail} more"));
    }
    list
}

fn deduped_file_names(paths: &[PathBuf]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut names = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        if seen.insert(name.clone()) {
            names.push(name);
        }
    }
    names
}

fn local_timestamp_parts(timestamp: SystemTime) -> (String, String) {
    // `Local` resolves the machine zone and silently falls back to UTC when
    // the offset cannot be determined, keeping sync messages renderable.
    let local: chrono::DateTime<chrono::Local> = timestamp.into();
    (
        local.format("%Y-%m-%d").to_string(),
        local.format("%H:%M").to_string(),
    )
}

/// Lowercase extension without the dot, when the file name has one.
pub fn file_extension(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    (!extension.is_empty()).then_some(extension)
}

/// An extension the user may approve: 1–16 ASCII letters or digits.
pub fn valid_extension(extension: &str) -> bool {
    (1..=16).contains(&extension.len())
        && extension.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn path_in_root(path: &Path, root: &Path) -> bool {
    root.as_os_str().is_empty() || path.starts_with(root)
}

fn is_safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && !path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

fn contains_dot_component(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(name) => name.to_string_lossy().starts_with('.'),
        _ => false,
    })
}

/// Tracked-content file names that are repository furniture rather than
/// notes or code; they never make a repository look "mixed".
const TRACKED_FURNITURE_FILE_NAMES: &[&str] = &["LICENSE", "COPYING", "NOTICE", "README"];

/// A tracked path counts as notes-like when it is a notes/text/image file,
/// a hidden configuration path (`.gitignore`, `.obsidian/…`), or standard
/// repository furniture (`LICENSE`, …). Only visible non-notes content such
/// as source files makes a repository mixed.
pub fn tracked_path_is_notes_like(path: &Path) -> bool {
    NewFileClass::classify(path).is_some()
        || contains_dot_component(path)
        || path.file_name().is_some_and(|name| {
            TRACKED_FURNITURE_FILE_NAMES.contains(&name.to_string_lossy().as_ref())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeFacts {
        symlinks: Vec<PathBuf>,
        nested: Vec<PathBuf>,
        fingerprints: HashMap<PathBuf, String>,
        worktree_blobs: Option<HashMap<PathBuf, String>>,
        index_blobs: HashMap<PathBuf, String>,
    }

    impl ScopeFacts for FakeFacts {
        fn is_symlink(&self, path: &Path) -> bool {
            self.symlinks.iter().any(|candidate| candidate == path)
        }
        fn is_nested_repository(&self, path: &Path) -> bool {
            self.nested.iter().any(|candidate| candidate == path)
        }
        fn fingerprint(&self, path: &Path) -> Option<String> {
            self.fingerprints.get(path).cloned()
        }
        fn worktree_blob_ids(&self, _: &[PathBuf]) -> Option<HashMap<PathBuf, String>> {
            self.worktree_blobs.clone()
        }
        fn index_blob_ids(&self, _: &[PathBuf]) -> Option<HashMap<PathBuf, String>> {
            Some(self.index_blobs.clone())
        }
    }

    fn change(path: &str, kind: ChangeKind) -> FileChange {
        FileChange {
            path: PathBuf::from(path),
            original_path: None,
            kind,
            index_status: if kind == ChangeKind::Untracked {
                '?'
            } else {
                '.'
            },
            worktree_status: match kind {
                ChangeKind::Untracked => '?',
                ChangeKind::Deleted => 'D',
                _ => 'M',
            },
            conflict: None,
        }
    }

    fn reason_of(partition: &ScopePartition, path: &str) -> Option<HeldBackReason> {
        partition
            .held_back
            .iter()
            .find(|held| held.path == Path::new(path))
            .map(|held| held.reason.clone())
    }

    fn whole_repository_policy() -> RepositoryPolicy {
        let mut policy = policy(Path::new("repo"));
        policy.tracked_roots = vec![PathBuf::new()];
        policy.new_file_rules[0].root = PathBuf::new();
        policy
    }

    #[test]
    fn os_artifacts_are_recognized_case_insensitively() {
        for path in [
            ".DS_Store",
            "notes/.ds_store",
            "notes/._photo.png",
            "Thumbs.db",
            "notes/ehthumbs_vista.db",
            "Desktop.ini",
            "Icon\r",
            ".Spotlight-V100/Store-V2/x",
            "$RECYCLE.BIN/S-1/x",
            "notes/~$report.docx",
            "notes/.~lock.report.odt#",
        ] {
            assert!(is_os_artifact(Path::new(path)), "{path:?}");
        }
        for path in [
            "DS_Store.md",
            "notes/thumbs.db.md",
            "notes/report.docx",
            ".~lock.x",
        ] {
            assert!(!is_os_artifact(Path::new(path)), "{path:?}");
        }
    }

    #[test]
    fn partition_holds_back_exceptions_and_ignores_os_artifacts() {
        let policy = policy(Path::new("repo"));
        let facts = FakeFacts {
            symlinks: vec![PathBuf::from("notes/link.md")],
            nested: vec![PathBuf::from("notes/vendored/")],
            ..FakeFacts::default()
        };
        let changes = [
            change("notes/today.md", ChangeKind::Untracked),
            change("notes/edited.md", ChangeKind::Modified),
            change("notes/report.pdf", ChangeKind::Untracked),
            change("notes/.secret.md", ChangeKind::Untracked),
            change("other/new.md", ChangeKind::Untracked),
            change("other/tracked.md", ChangeKind::Modified),
            change("notes/link.md", ChangeKind::Untracked),
            change("notes/vendored/", ChangeKind::Untracked),
            change("notes/.DS_Store", ChangeKind::Untracked),
            change(".DS_Store", ChangeKind::Modified),
        ];
        let partition = policy.partition(&changes, &facts);
        assert_eq!(
            partition.automatic,
            [
                PathBuf::from("notes/today.md"),
                PathBuf::from("notes/edited.md")
            ]
        );
        assert_eq!(partition.ignored, [PathBuf::from("notes/.DS_Store")]);
        assert_eq!(
            reason_of(&partition, "notes/report.pdf"),
            Some(HeldBackReason::FileType {
                extension: Some("pdf".into())
            })
        );
        assert_eq!(
            reason_of(&partition, "notes/.secret.md"),
            Some(HeldBackReason::Hidden)
        );
        assert_eq!(
            reason_of(&partition, "other/new.md"),
            Some(HeldBackReason::OutsideFolders)
        );
        assert_eq!(
            reason_of(&partition, "notes/link.md"),
            Some(HeldBackReason::Symlink)
        );
        assert_eq!(
            reason_of(&partition, "notes/vendored/"),
            Some(HeldBackReason::NestedRepository)
        );
        // Tracked copies follow the tracked-path policy, even for OS artifacts.
        let tracked = partition
            .held_back
            .iter()
            .find(|held| held.path == Path::new("other/tracked.md"))
            .unwrap();
        assert!(tracked.tracked && !tracked.decidable());
        assert_eq!(
            reason_of(&partition, ".DS_Store"),
            Some(HeldBackReason::OutsideFolders)
        );
        assert!(
            partition
                .held_back
                .iter()
                .filter(|held| matches!(
                    held.reason,
                    HeldBackReason::Symlink | HeldBackReason::NestedRepository
                ))
                .all(|held| !held.decidable())
        );
    }

    #[test]
    fn allowed_extensions_and_include_once_approvals() {
        let mut policy = whole_repository_policy();
        let mut facts = FakeFacts::default();
        facts
            .fingerprints
            .insert(PathBuf::from("doc.txt.bin"), "sha256:a".into());
        let changes = [
            change("report.pdf", ChangeKind::Untracked),
            change("doc.txt.bin", ChangeKind::Untracked),
            change(".hidden/report.pdf", ChangeKind::Untracked),
        ];
        policy.allowed_extensions.insert("pdf".into());
        policy.included_once.push(ScopeException {
            path: PathBuf::from("doc.txt.bin"),
            content_fingerprint: "sha256:a".into(),
        });
        let partition = policy.partition(&changes, &facts);
        assert_eq!(
            partition.automatic,
            [PathBuf::from("report.pdf"), PathBuf::from("doc.txt.bin")]
        );
        // An allowed type never admits hidden paths.
        assert_eq!(
            reason_of(&partition, ".hidden/report.pdf"),
            Some(HeldBackReason::Hidden)
        );

        facts
            .fingerprints
            .insert(PathBuf::from("doc.txt.bin"), "sha256:changed".into());
        let partition = policy.partition(&changes, &facts);
        assert_eq!(
            reason_of(&partition, "doc.txt.bin"),
            Some(HeldBackReason::ChangedSinceApproval)
        );

        policy.prune_included_once(|path| path != Path::new("doc.txt.bin"));
        assert!(policy.included_once.is_empty());
        assert!(valid_extension("pdf") && valid_extension("m4a"));
        assert!(!valid_extension("") && !valid_extension(".pdf") && !valid_extension("a b"));
    }

    #[test]
    fn exact_content_moves_are_held_back_together() {
        let policy = whole_repository_policy();
        let changes = [
            change("assets/photo.pdf", ChangeKind::Deleted),
            change(".archive/photo.pdf", ChangeKind::Untracked),
            change("assets/other.pdf", ChangeKind::Deleted),
        ];
        let facts = FakeFacts {
            worktree_blobs: Some(HashMap::from([(
                PathBuf::from(".archive/photo.pdf"),
                "blob-1".into(),
            )])),
            index_blobs: HashMap::from([
                (PathBuf::from("assets/photo.pdf"), "blob-1".into()),
                (PathBuf::from("assets/other.pdf"), "blob-2".into()),
            ]),
            ..FakeFacts::default()
        };
        let partition = policy.partition(&changes, &facts);
        assert_eq!(partition.automatic, [PathBuf::from("assets/other.pdf")]);
        assert_eq!(
            reason_of(&partition, "assets/photo.pdf"),
            Some(HeldBackReason::PairedMove {
                with: Some(PathBuf::from(".archive/photo.pdf"))
            })
        );

        // Without comparable content every deletion is held back.
        let unknown = FakeFacts {
            worktree_blobs: None,
            ..FakeFacts::default()
        };
        let partition = policy.partition(&changes, &unknown);
        assert!(partition.automatic.is_empty());
        assert_eq!(
            reason_of(&partition, "assets/other.pdf"),
            Some(HeldBackReason::PairedMove { with: None })
        );
    }

    #[test]
    fn policies_without_new_fields_still_load() {
        let dir = tempfile::tempdir().unwrap();
        let store = PolicyStore::new(dir.path().join("git-sync.toml"));
        let mut policies = SyncPolicies::empty();
        let mut saved = policy(dir.path());
        saved.allowed_extensions.insert("pdf".into());
        saved.included_once.push(ScopeException {
            path: PathBuf::from("a.bin"),
            content_fingerprint: "sha256:a".into(),
        });
        policies.upsert(saved.clone());
        store.save(&policies).unwrap();
        assert_eq!(store.load().unwrap().repositories[0], saved);

        let text = fs::read_to_string(dir.path().join("git-sync.toml")).unwrap();
        let legacy: String = text
            .lines()
            .filter(|line| !line.starts_with("allowed_extensions"))
            .collect::<Vec<_>>()
            .join("\n");
        let legacy = legacy
            .split("[[repositories.included_once]]")
            .next()
            .unwrap()
            .to_owned();
        fs::write(dir.path().join("git-sync.toml"), legacy).unwrap();
        let loaded = store.load().unwrap().repositories.remove(0);
        assert!(loaded.allowed_extensions.is_empty() && loaded.included_once.is_empty());
    }

    fn policy(root: &Path) -> RepositoryPolicy {
        RepositoryPolicy {
            identity: RepositoryIdentity::new(
                root.to_path_buf(),
                root.join(".git"),
                root.join(".git"),
            ),
            target: SyncTarget {
                local_branch: "main".into(),
                remote: "origin".into(),
                remote_branch: "main".into(),
                destination_ref: "refs/heads/main".into(),
                fetch_url: "https://example.invalid/notes.git".into(),
                push_url: "https://example.invalid/notes.git".into(),
            },
            tracked_roots: vec![PathBuf::from("notes")],
            new_file_rules: vec![NewFileRule {
                root: PathBuf::from("notes"),
                classes: BTreeSet::from([NewFileClass::Markdown, NewFileClass::Image]),
            }],
            message_template: String::new(),
            include_device_name: false,
            background_fetch: false,
            last_confirmed_remote: None,
            acknowledged_resource_omissions: Vec::new(),
            allowed_extensions: Default::default(),
            included_once: Vec::new(),
        }
    }

    #[test]
    fn policy_allows_expected_notes_but_fails_closed_for_hidden_and_binary() {
        let root = Path::new("repo");
        let policy = policy(root);
        assert!(policy.new_path_allowed(Path::new("notes/today.md"), false, false));
        assert!(policy.new_path_allowed(Path::new("notes/image.png"), false, false));
        assert!(!policy.new_path_allowed(Path::new("notes/.secret.md"), false, false));
        assert!(!policy.new_path_allowed(Path::new("notes/secret.key"), false, false));
        assert!(!policy.new_path_allowed(Path::new("notes/today.md"), true, false));
        assert!(policy.rename_needs_review(Path::new("notes/a.md"), Path::new("other/a.md")));
    }

    #[test]
    fn tracked_paths_are_not_removed_by_ignore_rules() {
        let policy = policy(Path::new("repo"));
        assert_eq!(
            policy.decide_path(Path::new("notes/a.md"), ChangeKind::Modified, true, false),
            PathDecision::Automatic
        );
    }

    #[test]
    fn policy_store_roundtrips_and_unknown_schema_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let store = PolicyStore::new(dir.path().join("git-sync.toml"));
        let mut policies = SyncPolicies::empty();
        policies.upsert(policy(dir.path()));
        store.save(&policies).unwrap();
        let mut expected = policies.clone();
        for repository in &mut expected.repositories {
            normalize_identity_paths(&mut repository.identity);
        }
        assert_eq!(store.load().unwrap(), expected);

        let invalid = SyncPolicies {
            schema_version: POLICY_SCHEMA_VERSION + 1,
            repositories: Vec::new(),
        };
        assert!(matches!(
            store.save(&invalid),
            Err(PolicyError::UnsupportedVersion(_))
        ));

        fs::write(dir.path().join("git-sync.toml"), "not = [valid").unwrap();
        assert!(matches!(store.load(), Err(PolicyError::Parse(_))));
    }

    #[cfg(windows)]
    #[test]
    fn policy_store_normalizes_windows_verbatim_repository_identity_paths() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        let policy_path = dir.path().join("git-sync.toml");
        let store = PolicyStore::new(&policy_path);
        let mut persisted = policy(dir.path());
        let verbatim = |path: &Path| PathBuf::from(format!(r"\\?\{}", path.display()));
        persisted.identity = RepositoryIdentity::new(
            verbatim(dir.path()),
            verbatim(&dir.path().join(".git")),
            verbatim(&dir.path().join(".git")),
        );
        let mut policies = SyncPolicies::empty();
        policies.upsert(persisted);
        fs::write(&policy_path, toml::to_string_pretty(&policies).unwrap()).unwrap();

        let loaded = store.load().unwrap();
        let identity = &loaded.repositories[0].identity;
        assert_eq!(
            identity.worktree_root,
            dunce::canonicalize(dir.path()).unwrap()
        );
        assert_eq!(
            identity.git_dir,
            dunce::canonicalize(dir.path().join(".git")).unwrap()
        );
        assert_eq!(identity.common_dir, identity.git_dir);
    }

    #[test]
    fn reconnect_and_disconnect_require_exact_repository_identity() {
        let dir = tempfile::tempdir().unwrap();
        let mut policies = SyncPolicies::empty();
        let original = policy(dir.path());
        let original_identity = original.identity.clone();
        policies.upsert(original);
        let replacement = RepositoryIdentity::new(
            dir.path().join("replacement"),
            dir.path().join("replacement/.git"),
            dir.path().join("replacement/.git"),
        );
        assert!(!policies.reconnect(
            &RepositoryIdentity::new(
                PathBuf::from("wrong"),
                PathBuf::from("x"),
                PathBuf::from("x")
            ),
            replacement.clone()
        ));
        assert!(policies.reconnect(&original_identity, replacement.clone()));
        assert!(policies.disconnect(&replacement).is_some());
        assert!(policies.repositories.is_empty());
    }

    #[test]
    fn reviewed_plan_detects_new_content_identity() {
        let reviewed = ReviewPlan {
            paths: vec![(PathBuf::from("a.md"), "first".into())],
        };
        assert!(reviewed.still_matches([(PathBuf::from("a.md"), "first".into())]));
        assert!(!reviewed.still_matches([(PathBuf::from("a.md"), "second".into())]));
    }

    #[test]
    fn tracked_notes_like_tolerates_hidden_and_furniture_files() {
        assert!(tracked_path_is_notes_like(Path::new("a.md")));
        assert!(tracked_path_is_notes_like(Path::new(".gitignore")));
        assert!(tracked_path_is_notes_like(Path::new(".obsidian/app.json")));
        assert!(tracked_path_is_notes_like(Path::new("LICENSE")));
        assert!(tracked_path_is_notes_like(Path::new("docs/README")));
        assert!(!tracked_path_is_notes_like(Path::new("src/main.rs")));
        assert!(!tracked_path_is_notes_like(Path::new("config.yaml")));
    }

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn default_message_lists_local_datetime_and_single_file_name() {
        let message = default_commit_message(
            &paths(&["notes/a.md"]),
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000),
        );
        assert!(message.starts_with("Sync notes "), "{message}");
        let remainder = message.strip_prefix("Sync notes ").unwrap();
        let (stamp, tail) = remainder.split_once(": ").expect("datetime and tail");
        assert!(
            stamp.len() == 16
                && stamp.as_bytes()[4] == b'-'
                && stamp.as_bytes()[7] == b'-'
                && stamp.as_bytes()[10] == b' '
                && stamp.as_bytes()[13] == b':',
            "expected YYYY-MM-DD HH:MM, got {stamp}"
        );
        assert_eq!(tail, "a.md");
    }

    #[test]
    fn default_message_bounds_file_list_with_more_tail() {
        let message = default_commit_message(
            &paths(&["a.md", "b.md", "c.md", "d.md", "e.md", "f.md", "g.md"]),
            SystemTime::UNIX_EPOCH,
        );
        assert!(
            message.ends_with(": a.md, b.md, c.md, +4 more"),
            "{message}"
        );
    }

    #[test]
    fn default_message_deduplicates_repeated_file_names() {
        let message = default_commit_message(
            &paths(&["one/notes.md", "two/notes.md", "todo.md"]),
            SystemTime::UNIX_EPOCH,
        );
        assert!(message.ends_with(": notes.md, todo.md"), "{message}");
    }

    #[test]
    fn default_message_truncates_to_subject_cap_by_dropping_names() {
        let long: Vec<String> = ["a", "b", "c"]
            .iter()
            .map(|prefix| format!("{prefix}{}", "x".repeat(39)))
            .collect();
        let long = long.iter().map(String::as_str).collect::<Vec<_>>();
        let message = default_commit_message(&paths(&long), SystemTime::UNIX_EPOCH);
        assert!(message.chars().count() <= 72, "{message}");
        assert!(message.ends_with(": +3 more"), "{message}");
        assert!(!message.contains("xxxx"), "{message}");
    }

    #[test]
    fn default_message_without_changes_omits_the_file_tail() {
        let message = default_commit_message(&[], SystemTime::UNIX_EPOCH);
        assert!(message.starts_with("Sync notes "), "{message}");
        // "Sync notes " + "YYYY-MM-DD HH:MM" and nothing after the timestamp.
        assert_eq!(
            message.chars().count(),
            "Sync notes ".len() + 16,
            "{message}"
        );
    }

    #[test]
    fn render_message_substitutes_every_placeholder() {
        let mut custom = policy(Path::new("repo"));
        custom.message_template = "{count}|{files}|{date}|{time}".into();
        let message = custom.render_message(
            &paths(&["a.md", "b.md", "c.md", "d.md"]),
            None,
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000),
        );
        let (count, rest) = message.split_once('|').unwrap();
        let (files, rest) = rest.split_once('|').unwrap();
        let (date, time) = rest.split_once('|').unwrap();
        assert_eq!(count, "4");
        assert_eq!(files, "a.md, b.md, c.md, +1 more");
        assert_eq!(date.len(), 10, "{date}");
        assert_eq!(time.len(), 5, "{time}");
    }

    #[test]
    fn render_message_with_empty_template_uses_the_default_format() {
        let plain = policy(Path::new("repo"));
        let stamp = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        assert_eq!(
            plain.render_message(&paths(&["a.md"]), None, stamp),
            default_commit_message(&paths(&["a.md"]), stamp)
        );
    }

    #[test]
    fn render_message_appends_device_suffix_after_substitution() {
        let mut custom = policy(Path::new("repo"));
        custom.message_template = "{count} notes".into();
        custom.include_device_name = true;
        assert_eq!(
            custom.render_message(&paths(&["a.md"]), Some("Laptop"), SystemTime::UNIX_EPOCH),
            "1 notes from Laptop"
        );
    }
}
