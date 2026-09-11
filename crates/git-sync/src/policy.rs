use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
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

    pub fn render_message(&self, changed_files: usize, device_name: Option<&str>) -> String {
        let mut message = if self.message_template.trim().is_empty() {
            default_commit_message(changed_files)
        } else {
            self.message_template
                .replace("{count}", &changed_files.to_string())
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

pub fn default_commit_message(changed_files: usize) -> String {
    format!("Sync notes: {changed_files} files")
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
        let normalized_root = dunce::canonicalize(dir.path()).unwrap();
        policies.upsert(policy(&normalized_root));
        store.save(&policies).unwrap();
        assert_eq!(store.load().unwrap(), policies);

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
}
