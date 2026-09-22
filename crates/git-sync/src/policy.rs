use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::{ChangeKind, GitObjectId, RepositoryIdentity, SyncTarget, persist::atomic_write};

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
        let Some(class) = NewFileClass::classify(path) else {
            return false;
        };
        self.new_file_rules
            .iter()
            .any(|rule| path_in_root(path, &rule.root) && rule.classes.contains(&class))
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
        assert!(message.ends_with(": a.md, b.md, c.md, +4 more"), "{message}");
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
        assert_eq!(message.chars().count(), "Sync notes ".len() + 16, "{message}");
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
