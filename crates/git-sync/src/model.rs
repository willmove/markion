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

/// Product-facing state derived from the repository's independent technical
/// dimensions. This deliberately contains no localized copy or GUI types so
/// every frontend projects the same safety semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackupSyncState {
    NotConfigured,
    Running,
    PendingLocal,
    Incoming,
    PendingBoth,
    Synchronized,
    Offline,
    AuthenticationNeeded,
    ConflictOrRecovery,
    UncertainDelivery,
    NeedsAttention,
    RemoteUnknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackupSyncAction {
    TurnOn,
    ViewProgress,
    SyncNow,
    ViewStatus,
    Retry,
    Reconnect,
    Resolve,
    CheckStatus,
    Review,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupSyncFacts {
    pub connection: ConnectionState,
    pub documents: DocumentSyncState,
    /// Named paths waiting on disk/history. Callers must avoid counting the
    /// same dirty named document twice; omitted untitled notes are added here.
    pub pending_paths: usize,
    pub history: HistoryRelation,
    pub operation: Option<OperationProgress>,
    pub has_conflicts: bool,
    pub recovery_items: usize,
    pub attention_items: usize,
    pub has_external_staging: bool,
    pub has_external_operation: bool,
    pub delivery_uncertain: bool,
    pub last_confirmed_at: Option<SystemTime>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupSyncPresentation {
    pub state: BackupSyncState,
    pub action: BackupSyncAction,
    pub local_items: usize,
    pub outgoing_commits: usize,
    pub incoming_commits: usize,
    pub last_confirmed_at: Option<SystemTime>,
}

impl BackupSyncPresentation {
    pub fn project(facts: BackupSyncFacts) -> Self {
        let (outgoing_commits, incoming_commits) = match facts.history {
            HistoryRelation::Ahead { commits } => (commits, 0),
            HistoryRelation::Behind { commits } => (0, commits),
            HistoryRelation::Diverged { ahead, behind } => (ahead, behind),
            _ => (0, 0),
        };
        // `pending_paths` is the caller's deduplicated union of named
        // worktree/history and live-buffer paths. `unsaved` remains an
        // independent fact, so use it as a conservative lower bound for
        // callers that have not yet refreshed their path inventory.
        let local_items = facts
            .pending_paths
            .max(facts.documents.unsaved)
            .saturating_add(facts.documents.omitted_untitled);
        let active_phase = facts.operation.as_ref().map(|operation| operation.phase);

        let (state, action) = if facts.recovery_items > 0
            || active_phase == Some(OperationPhase::Resolving)
            || (facts.connection != ConnectionState::NotConfigured && facts.has_conflicts)
        {
            (
                BackupSyncState::ConflictOrRecovery,
                BackupSyncAction::Resolve,
            )
        } else if facts.connection == ConnectionState::NotConfigured {
            (BackupSyncState::NotConfigured, BackupSyncAction::TurnOn)
        } else if facts.delivery_uncertain {
            (
                BackupSyncState::UncertainDelivery,
                BackupSyncAction::CheckStatus,
            )
        } else if facts.connection == ConnectionState::AuthenticationNeeded {
            (
                BackupSyncState::AuthenticationNeeded,
                BackupSyncAction::Reconnect,
            )
        } else if active_phase.is_some_and(|phase| {
            !matches!(
                phase,
                OperationPhase::Idle | OperationPhase::Complete | OperationPhase::NeedsAttention
            )
        }) {
            (BackupSyncState::Running, BackupSyncAction::ViewProgress)
        } else if facts.connection == ConnectionState::Unsupported
            || facts.attention_items > 0
            || facts.documents.external_conflicts > 0
            || facts.has_external_staging
            || facts.has_external_operation
            || active_phase == Some(OperationPhase::NeedsAttention)
            || matches!(
                facts.history,
                HistoryRelation::Unrelated
                    | HistoryRelation::Rewritten
                    | HistoryRelation::DeletedTarget
            )
        {
            (BackupSyncState::NeedsAttention, BackupSyncAction::Review)
        } else if facts.connection == ConnectionState::Offline {
            (BackupSyncState::Offline, BackupSyncAction::Retry)
        } else {
            let has_local = local_items > 0 || outgoing_commits > 0;
            match (has_local, incoming_commits > 0) {
                (true, true) => (BackupSyncState::PendingBoth, BackupSyncAction::SyncNow),
                (true, false) => (BackupSyncState::PendingLocal, BackupSyncAction::SyncNow),
                (false, true) => (BackupSyncState::Incoming, BackupSyncAction::SyncNow),
                (false, false)
                    if facts.history == HistoryRelation::Equal
                        && facts.last_confirmed_at.is_some() =>
                {
                    (BackupSyncState::Synchronized, BackupSyncAction::ViewStatus)
                }
                (false, false) => (BackupSyncState::RemoteUnknown, BackupSyncAction::SyncNow),
            }
        };

        Self {
            state,
            action,
            local_items,
            outgoing_commits,
            incoming_commits,
            last_confirmed_at: facts.last_confirmed_at,
        }
    }

    pub fn remotely_confirmed(&self) -> bool {
        self.state == BackupSyncState::Synchronized
    }
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

    fn backup_facts() -> BackupSyncFacts {
        BackupSyncFacts {
            connection: ConnectionState::Available,
            documents: DocumentSyncState::default(),
            pending_paths: 0,
            history: HistoryRelation::Equal,
            operation: None,
            has_conflicts: false,
            recovery_items: 0,
            attention_items: 0,
            has_external_staging: false,
            has_external_operation: false,
            delivery_uncertain: false,
            last_confirmed_at: Some(SystemTime::UNIX_EPOCH),
        }
    }

    #[test]
    fn backup_sync_projection_has_deterministic_critical_state_precedence() {
        let mut facts = backup_facts();
        facts.has_conflicts = true;
        facts.delivery_uncertain = true;
        facts.connection = ConnectionState::AuthenticationNeeded;
        facts.operation = Some(OperationProgress {
            operation_id: "sync".into(),
            kind: OperationKind::SyncNow,
            phase: OperationPhase::Pushing,
            detail: None,
        });
        let presentation = BackupSyncPresentation::project(facts);
        assert_eq!(presentation.state, BackupSyncState::ConflictOrRecovery);
        assert_eq!(presentation.action, BackupSyncAction::Resolve);

        let mut facts = backup_facts();
        facts.delivery_uncertain = true;
        facts.connection = ConnectionState::AuthenticationNeeded;
        assert_eq!(
            BackupSyncPresentation::project(facts).state,
            BackupSyncState::UncertainDelivery
        );

        let mut facts = backup_facts();
        facts.connection = ConnectionState::AuthenticationNeeded;
        facts.attention_items = 2;
        assert_eq!(
            BackupSyncPresentation::project(facts).action,
            BackupSyncAction::Reconnect
        );

        let mut facts = backup_facts();
        facts.connection = ConnectionState::NotConfigured;
        facts.recovery_items = 1;
        let presentation = BackupSyncPresentation::project(facts);
        assert_eq!(presentation.state, BackupSyncState::ConflictOrRecovery);
        assert_eq!(presentation.action, BackupSyncAction::Resolve);
    }

    #[test]
    fn backup_sync_projection_preserves_pending_and_incoming_qualifiers() {
        let mut facts = backup_facts();
        facts.pending_paths = 3;
        facts.documents.omitted_untitled = 1;
        facts.history = HistoryRelation::Diverged {
            ahead: 2,
            behind: 4,
        };
        let presentation = BackupSyncPresentation::project(facts);
        assert_eq!(presentation.state, BackupSyncState::PendingBoth);
        assert_eq!(presentation.action, BackupSyncAction::SyncNow);
        assert_eq!(presentation.local_items, 4);
        assert_eq!(presentation.outgoing_commits, 2);
        assert_eq!(presentation.incoming_commits, 4);
        assert_eq!(presentation.last_confirmed_at, Some(SystemTime::UNIX_EPOCH));
        assert!(!presentation.remotely_confirmed());
    }

    #[test]
    fn only_equal_confirmed_clean_state_allows_synchronized_copy() {
        let synchronized = BackupSyncPresentation::project(backup_facts());
        assert_eq!(synchronized.state, BackupSyncState::Synchronized);
        assert_eq!(synchronized.action, BackupSyncAction::ViewStatus);
        assert!(synchronized.remotely_confirmed());

        let mut cases = Vec::new();
        let mut local_only = backup_facts();
        local_only.pending_paths = 1;
        cases.push(local_only);
        let mut unsaved_named = backup_facts();
        unsaved_named.documents.unsaved = 1;
        cases.push(unsaved_named);
        let mut outgoing_only = backup_facts();
        outgoing_only.history = HistoryRelation::Ahead { commits: 1 };
        cases.push(outgoing_only);
        let mut stale = backup_facts();
        stale.history = HistoryRelation::Unknown;
        cases.push(stale);
        let mut never_confirmed = backup_facts();
        never_confirmed.last_confirmed_at = None;
        cases.push(never_confirmed);
        let mut offline = backup_facts();
        offline.connection = ConnectionState::Offline;
        cases.push(offline);

        for facts in cases {
            let presentation = BackupSyncPresentation::project(facts);
            assert_ne!(presentation.state, BackupSyncState::Synchronized);
            assert!(!presentation.remotely_confirmed());
        }
    }

    #[test]
    fn backup_sync_projection_maps_remaining_primary_actions() {
        let mut facts = backup_facts();
        facts.connection = ConnectionState::NotConfigured;
        assert_eq!(
            BackupSyncPresentation::project(facts).action,
            BackupSyncAction::TurnOn
        );

        let mut facts = backup_facts();
        facts.operation = Some(OperationProgress {
            operation_id: "sync".into(),
            kind: OperationKind::SyncNow,
            phase: OperationPhase::Fetching,
            detail: None,
        });
        assert_eq!(
            BackupSyncPresentation::project(facts).action,
            BackupSyncAction::ViewProgress
        );

        let mut facts = backup_facts();
        facts.connection = ConnectionState::Offline;
        assert_eq!(
            BackupSyncPresentation::project(facts).action,
            BackupSyncAction::Retry
        );

        let mut facts = backup_facts();
        facts.attention_items = 1;
        assert_eq!(
            BackupSyncPresentation::project(facts).action,
            BackupSyncAction::Review
        );
    }
}
