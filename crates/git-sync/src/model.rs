use std::{ffi::OsString, fmt, path::PathBuf, time::SystemTime};

use serde::{Deserialize, Serialize};

/// An opaque object identifier emitted by Git. Keeping it as text supports
/// repositories whose object format is not SHA-1 without guessing a length.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GitObjectId(String);

impl GitObjectId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidObjectId> {
        let value = value.into();
        if value.len() < 4 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(InvalidObjectId);
        }
        Ok(Self(value.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> &str {
        &self.0[..self.0.len().min(12)]
    }
}

impl fmt::Display for GitObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid Git object id")]
pub struct InvalidObjectId;

/// Stable local identity for one checked-out worktree.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryIdentity {
    pub worktree_root: PathBuf,
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
}

impl RepositoryIdentity {
    pub fn new(worktree_root: PathBuf, git_dir: PathBuf, common_dir: PathBuf) -> Self {
        Self {
            worktree_root,
            git_dir,
            common_dir,
        }
    }
}

/// One explicit local-branch to remote-branch synchronization destination.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncTarget {
    pub local_branch: String,
    pub remote: String,
    pub remote_branch: String,
    pub destination_ref: String,
    pub fetch_url: String,
    pub push_url: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryCapabilities {
    pub ordinary_worktree: bool,
    pub detached_head: bool,
    pub linked_worktree: bool,
    pub bare: bool,
    pub shallow: bool,
    pub sparse: bool,
    pub partial: bool,
    pub has_submodules: bool,
    pub has_nested_repositories: bool,
    pub uses_lfs: bool,
}

impl RepositoryCapabilities {
    pub fn supports_write_sync(self) -> bool {
        self.ordinary_worktree
            && !self.detached_head
            && !self.linked_worktree
            && !self.bare
            && !self.shallow
            && !self.sparse
            && !self.partial
            && !self.has_submodules
            && !self.has_nested_repositories
            && !self.uses_lfs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Unmerged,
    Untracked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictKind {
    BothModified,
    BothAdded,
    DeletedByLocal,
    DeletedByRemote,
    BothDeleted,
    Rename,
    Other,
}

/// A repository-relative path is kept as an OS string so the core does not
/// silently make a lossy UTF-8 identity. It is not persisted through serde;
/// the persistence layer validates an explicit portable representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileChange {
    pub path: PathBuf,
    pub original_path: Option<PathBuf>,
    pub kind: ChangeKind,
    pub index_status: char,
    pub worktree_status: char,
    pub conflict: Option<ConflictKind>,
}

impl FileChange {
    pub fn native_name(&self) -> OsString {
        self.path.as_os_str().to_owned()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSyncState {
    pub unsaved: usize,
    pub external_conflicts: usize,
    pub omitted_untitled: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    NotConfigured,
    Available,
    AuthenticationNeeded,
    Offline,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryRelation {
    #[default]
    Unknown,
    Equal,
    Ahead {
        commits: usize,
    },
    Behind {
        commits: usize,
    },
    Diverged {
        ahead: usize,
        behind: usize,
    },
    Unrelated,
    Rewritten,
    DeletedTarget,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorktreeState {
    pub changes: Vec<FileChange>,
    pub has_external_staging: bool,
    pub has_conflicts: bool,
    pub operation_in_progress: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationKind {
    SyncNow,
    CommitLocally,
    CheckRemote,
    PullUpdates,
    PushCommits,
    Clone,
    Connect,
    Initialize,
    ResolveConflict,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationPhase {
    #[default]
    Idle,
    Preparing,
    Saving,
    Staging,
    Committing,
    Fetching,
    Integrating,
    Resolving,
    Pushing,
    Verifying,
    NeedsAttention,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationProgress {
    pub operation_id: String,
    pub kind: OperationKind,
    pub phase: OperationPhase,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositorySnapshot {
    pub identity: RepositoryIdentity,
    pub target: Option<SyncTarget>,
    pub connection: ConnectionState,
    pub documents: DocumentSyncState,
    pub worktree: WorktreeState,
    pub history: HistoryRelation,
    pub head: Option<GitObjectId>,
    pub remote_tip: Option<GitObjectId>,
    pub checked_at: SystemTime,
    pub operation: Option<OperationProgress>,
}

/// Content identities are supplied by the app after its write barrier drains.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncPlan {
    pub operation_id: String,
    pub identity: RepositoryIdentity,
    pub target: SyncTarget,
    pub expected_head: Option<GitObjectId>,
    pub paths: Vec<PathBuf>,
    pub content_fingerprints: Vec<(PathBuf, String)>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    RemoteChecked {
        relation: HistoryRelation,
    },
    Synchronized {
        commit: Option<GitObjectId>,
        remote_tip: Option<GitObjectId>,
        pending_local_changes: usize,
    },
    CommittedLocally {
        commit: GitObjectId,
    },
    UpToDate,
    AwaitingUpload {
        commit: GitObjectId,
        reason: String,
    },
    NeedsAttention {
        phase: OperationPhase,
        reason: String,
    },
    UncertainDelivery {
        commit: GitObjectId,
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_ids_are_opaque_lowercase_hex() {
        let id = GitObjectId::parse("A1B2c3d4").unwrap();
        assert_eq!(id.as_str(), "a1b2c3d4");
        assert!(GitObjectId::parse("not-an-object").is_err());
    }

    #[test]
    fn unsupported_repository_capabilities_fail_closed() {
        let ordinary = RepositoryCapabilities {
            ordinary_worktree: true,
            ..RepositoryCapabilities::default()
        };
        assert!(ordinary.supports_write_sync());
        assert!(
            !RepositoryCapabilities {
                uses_lfs: true,
                ..ordinary
            }
            .supports_write_sync()
        );
    }
}
