use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};

use crate::{
    AskpassBridge, AuthFailure, Authentication, BackgroundFetchResult, CancellationToken,
    ChangeKind, CommandLimits, GitCommand, GitCommandError, GitObjectId, GitRepository,
    HistoryRelation, JournalError, JournalStore, OperationCheckpoint, OperationKind,
    OperationPhase, PathDecision, ProcessTermination, RepositoryError, RepositoryPolicy,
    ReviewPlan, SyncOutcome, SyncPlan,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncOptions {
    pub network_timeout: Duration,
    pub max_preimage_bytes: u64,
    pub allow_one_push_retry: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            network_timeout: Duration::from_secs(300),
            max_preimage_bytes: 128 * 1024 * 1024,
            allow_one_push_retry: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncEngineError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Command(#[from] GitCommandError),
    #[error(transparent)]
    Journal(#[from] JournalError),
    #[error("repository identity or configured target changed")]
    TargetDrift,
    #[error("repository contains unsupported capabilities")]
    UnsupportedRepository,
    #[error("repository has staging, conflicts, an active operation, or unexpected changes")]
    UnsafeRepositoryState,
    #[error("sync plan contains an unapproved or stale path")]
    StalePlan,
    #[error("remote target is missing, rewritten, or unrelated")]
    UnsafeRemoteHistory,
    #[error("incoming paths cannot be materialized safely")]
    UnsafeIncomingPaths,
    #[error("operation identifier is invalid")]
    InvalidOperationId,
    #[error("repository has no commit to synchronize")]
    NoCommit,
    #[error("local version requires a nonblank commit message")]
    InvalidCommitMessage,
}

#[derive(Clone)]
pub struct GitSyncEngine {
    repository: GitRepository,
    journal: JournalStore,
    options: SyncOptions,
    foreground_credentials: bool,
    observer:
        Option<std::sync::Arc<dyn Fn(OperationPhase) -> Result<(), SyncEngineError> + Send + Sync>>,
}

impl GitSyncEngine {
    pub fn new(repository: GitRepository, journal: JournalStore, options: SyncOptions) -> Self {
        Self {
            repository,
            journal,
            options,
            observer: None,
            foreground_credentials: false,
        }
    }

    pub fn repository(&self) -> &GitRepository {
        &self.repository
    }

    pub fn with_foreground_credentials(mut self) -> Self {
        self.foreground_credentials = true;
        self
    }

    pub fn with_observer(
        mut self,
        observer: impl Fn(OperationPhase) -> Result<(), SyncEngineError> + Send + Sync + 'static,
    ) -> Self {
        self.observer = Some(std::sync::Arc::new(observer));
        self
    }

    pub fn sync_now(
        &self,
        plan: &SyncPlan,
        policy: &mut RepositoryPolicy,
        reviewed: Option<&ReviewPlan>,
        askpass: Option<&AskpassBridge>,
        cancellation: &CancellationToken,
    ) -> Result<SyncOutcome, SyncEngineError> {
        validate_operation_id(&plan.operation_id)?;
        let state = self.preflight(plan, policy, reviewed)?;
        let mut checkpoint = OperationCheckpoint {
            operation_id: plan.operation_id.clone(),
            kind: OperationKind::SyncNow,
            phase: OperationPhase::Preparing,
            identity: plan.identity.clone(),
            target: Some(plan.target.clone()),
            expected_head: state.head.clone(),
            resulting_commit: None,
            fetched_tip: None,
            expected_index_fingerprint: self.index_tree(cancellation).ok(),
            owned_paths: plan.content_fingerprints.clone(),
            recovery_refs: Vec::new(),
            preimages: None,
            drafts: Vec::new(),
            updated_unix_seconds: now_seconds(),
            confirmed: false,
        };
        // Failure here is intentionally before any staging or worktree change.
        self.journal.begin(checkpoint.clone())?;

        let mut local_commit = if plan.paths.is_empty() {
            state.head.clone()
        } else {
            checkpoint.phase = OperationPhase::Staging;
            self.persist_checkpoint(&mut checkpoint)?;
            self.stage_exact_paths(plan, cancellation)?;
            checkpoint.expected_index_fingerprint = self.index_tree(cancellation).ok();
            checkpoint.phase = OperationPhase::Committing;
            self.persist_checkpoint(&mut checkpoint)?;
            match self.commit(plan, cancellation) {
                Ok(commit) => {
                    checkpoint.resulting_commit = Some(commit.clone());
                    Some(commit)
                }
                Err(error) => {
                    let _ = self.persist_checkpoint(&mut checkpoint);
                    return Err(error);
                }
            }
        };

        let Some(mut intended_commit) = local_commit.take() else {
            return Err(SyncEngineError::NoCommit);
        };
        checkpoint.resulting_commit = Some(intended_commit.clone());

        for attempt in 0..=usize::from(self.options.allow_one_push_retry) {
            checkpoint.phase = OperationPhase::Fetching;
            self.persist_checkpoint(&mut checkpoint)?;
            let remote_tip = match self.fetch_target(plan, askpass, cancellation) {
                Ok(Some(tip)) => tip,
                Ok(None) => return Err(SyncEngineError::UnsafeRemoteHistory),
                Err(error) => {
                    checkpoint.phase = OperationPhase::NeedsAttention;
                    let _ = self.persist_checkpoint(&mut checkpoint);
                    return Ok(SyncOutcome::AwaitingUpload {
                        commit: intended_commit,
                        reason: error.to_string(),
                    });
                }
            };
            checkpoint.fetched_tip = Some(remote_tip.clone());
            let relation = self.repository.classify_history(
                &intended_commit,
                &remote_tip,
                policy.last_confirmed_remote.as_ref(),
            )?;
            match relation {
                HistoryRelation::Equal | HistoryRelation::Ahead { .. } => {}
                HistoryRelation::Behind { .. } | HistoryRelation::Diverged { .. } => {
                    checkpoint.phase = OperationPhase::Integrating;
                    self.persist_checkpoint(&mut checkpoint)?;
                    match self.integrate(
                        &mut checkpoint,
                        &intended_commit,
                        &remote_tip,
                        relation,
                        cancellation,
                    ) {
                        Ok(commit) => intended_commit = commit,
                        Err(SyncEngineError::Command(error))
                            if self.repository.status()?.worktree.has_conflicts =>
                        {
                            let conflict_paths = self.conflict_paths(cancellation)?;
                            checkpoint.owned_paths = conflict_paths
                                .iter()
                                .cloned()
                                .map(|path| (path, String::new()))
                                .collect();
                            checkpoint.expected_index_fingerprint = Some(
                                self.conflict_state_fingerprint(&conflict_paths, cancellation)?,
                            );
                            checkpoint.phase = OperationPhase::Resolving;
                            self.persist_checkpoint(&mut checkpoint)?;
                            return Ok(SyncOutcome::NeedsAttention {
                                phase: OperationPhase::Resolving,
                                reason: error.to_string(),
                            });
                        }
                        Err(error) => return Err(error),
                    }
                }
                HistoryRelation::Unrelated
                | HistoryRelation::Rewritten
                | HistoryRelation::DeletedTarget
                | HistoryRelation::Unknown => return Err(SyncEngineError::UnsafeRemoteHistory),
            }

            checkpoint.resulting_commit = Some(intended_commit.clone());
            checkpoint.phase = OperationPhase::Pushing;
            self.persist_checkpoint(&mut checkpoint)?;
            match self.push_exact(plan, &intended_commit, askpass, cancellation) {
                Ok(()) => {
                    return self.finish(
                        policy,
                        &mut checkpoint,
                        intended_commit.clone(),
                        Some(intended_commit),
                        cancellation,
                    );
                }
                Err(error) => {
                    checkpoint.phase = OperationPhase::Verifying;
                    self.persist_checkpoint(&mut checkpoint)?;
                    if let Ok(Some(observed)) = self.fetch_target(plan, askpass, cancellation)
                        && self.is_ancestor(&intended_commit, &observed, cancellation)?
                    {
                        return self.finish(
                            policy,
                            &mut checkpoint,
                            intended_commit,
                            Some(observed),
                            cancellation,
                        );
                    }
                    if attempt < usize::from(self.options.allow_one_push_retry)
                        && is_non_fast_forward(&error)
                    {
                        continue;
                    }
                    // Preserve Verifying across restart: delivery remains unknown.
                    self.persist_checkpoint(&mut checkpoint)?;
                    return Ok(SyncOutcome::UncertainDelivery {
                        commit: intended_commit,
                        reason: error.to_string(),
                    });
                }
            }
        }
        unreachable!("bounded retry loop always returns")
    }

    pub fn commit_locally(
        &self,
        plan: &SyncPlan,
        policy: &RepositoryPolicy,
        reviewed: Option<&ReviewPlan>,
        cancellation: &CancellationToken,
    ) -> Result<SyncOutcome, SyncEngineError> {
        validate_operation_id(&plan.operation_id)?;
        let state = self.preflight(plan, policy, reviewed)?;
        if plan.paths.is_empty() {
            return Ok(SyncOutcome::UpToDate);
        }
        let mut checkpoint = self.new_checkpoint(plan, OperationKind::CommitLocally, &state);
        self.journal.begin(checkpoint.clone())?;
        checkpoint.phase = OperationPhase::Staging;
        self.persist_checkpoint(&mut checkpoint)?;
        self.stage_exact_paths(plan, cancellation)?;
        checkpoint.expected_index_fingerprint = self.index_tree(cancellation).ok();
        checkpoint.phase = OperationPhase::Committing;
        self.persist_checkpoint(&mut checkpoint)?;
        let commit = self.commit(plan, cancellation)?;
        checkpoint.phase = OperationPhase::Complete;
        checkpoint.resulting_commit = Some(commit.clone());
        checkpoint.confirmed = true;
        self.persist_checkpoint(&mut checkpoint)?;
        self.journal
            .retire_success(&plan.operation_id, now_seconds())?;
        Ok(SyncOutcome::CommittedLocally { commit })
    }

    /// Create a deliberate local version from a reviewed subset of current
    /// policy-approved paths. Unlike the automatic snapshot path, unselected
    /// worktree changes are allowed and remain untouched.
    pub fn commit_selected(
        &self,
        plan: &SyncPlan,
        policy: &RepositoryPolicy,
        cancellation: &CancellationToken,
    ) -> Result<SyncOutcome, SyncEngineError> {
        validate_operation_id(&plan.operation_id)?;
        if plan.message.trim().is_empty() {
            return Err(SyncEngineError::InvalidCommitMessage);
        }
        let state = self.preflight_selected(plan, policy)?;
        let mut checkpoint = self.new_checkpoint(plan, OperationKind::CommitLocally, &state);
        self.journal.begin(checkpoint.clone())?;
        checkpoint.phase = OperationPhase::Staging;
        self.persist_checkpoint(&mut checkpoint)?;
        self.stage_exact_paths(plan, cancellation)?;
        checkpoint.expected_index_fingerprint = self.index_tree(cancellation).ok();
        checkpoint.phase = OperationPhase::Committing;
        self.persist_checkpoint(&mut checkpoint)?;
        let commit = self.commit(plan, cancellation)?;
        checkpoint.phase = OperationPhase::Complete;
        checkpoint.resulting_commit = Some(commit.clone());
        checkpoint.confirmed = true;
        self.persist_checkpoint(&mut checkpoint)?;
        self.journal
            .retire_success(&plan.operation_id, now_seconds())?;
        Ok(SyncOutcome::CommittedLocally { commit })
    }

    pub fn check_remote(
        &self,
        plan: &SyncPlan,
        policy: &RepositoryPolicy,
        askpass: Option<&AskpassBridge>,
        cancellation: &CancellationToken,
    ) -> Result<SyncOutcome, SyncEngineError> {
        validate_operation_id(&plan.operation_id)?;
        let state = self.target_preflight(plan, policy)?;
        let local = state.head.ok_or(SyncEngineError::NoCommit)?;
        let Some(remote) = self.fetch_target(plan, askpass, cancellation)? else {
            self.remove_fetch_ref(&plan.operation_id);
            return Ok(SyncOutcome::RemoteChecked {
                relation: GitRepository::classify_missing_target(
                    policy.last_confirmed_remote.as_ref(),
                ),
            });
        };
        let relation = self.repository.classify_history(
            &local,
            &remote,
            policy.last_confirmed_remote.as_ref(),
        )?;
        self.remove_fetch_ref(&plan.operation_id);
        Ok(SyncOutcome::RemoteChecked { relation })
    }

    pub fn background_check(
        &self,
        operation_id: &str,
        policy: &RepositoryPolicy,
        cancellation: &CancellationToken,
    ) -> BackgroundFetchResult {
        if validate_operation_id(operation_id).is_err()
            || !policy.validates_identity(self.repository.identity())
            || self.repository.resolve_sync_target().ok().as_ref() != Some(&policy.target)
        {
            return BackgroundFetchResult::ActionableError(
                "repository identity or synchronization target changed".into(),
            );
        }
        let state = match self.repository.status() {
            Ok(state)
                if state.capabilities.supports_write_sync()
                    && !state.worktree.has_conflicts
                    && state.worktree.operation_in_progress.is_none() =>
            {
                state
            }
            Ok(_) => {
                return BackgroundFetchResult::ActionableError(
                    "repository requires attention before checking the remote".into(),
                );
            }
            Err(error) => return BackgroundFetchResult::ActionableError(error.to_string()),
        };
        let Some(local) = state.head else {
            return BackgroundFetchResult::ActionableError("repository has no local commit".into());
        };
        let plan = SyncPlan {
            operation_id: operation_id.to_string(),
            identity: policy.identity.clone(),
            target: policy.target.clone(),
            expected_head: Some(local.clone()),
            paths: Vec::new(),
            content_fingerprints: Vec::new(),
            message: String::new(),
        };
        let remote = match self.fetch_target(&plan, None, cancellation) {
            Ok(Some(remote)) => remote,
            Ok(None) => {
                return BackgroundFetchResult::ActionableError(
                    "configured remote branch is missing".into(),
                );
            }
            Err(SyncEngineError::Command(error)) => {
                return match Authentication::classify_failure(&error) {
                    AuthFailure::AuthenticationNeeded
                    | AuthFailure::EncryptedKey
                    | AuthFailure::UnknownHostKey => BackgroundFetchResult::AuthenticationNeeded,
                    AuthFailure::Other(message)
                        if message.contains("Could not resolve")
                            || message.contains("Failed to connect")
                            || message.contains("Connection timed out") =>
                    {
                        BackgroundFetchResult::Offline
                    }
                    failure => BackgroundFetchResult::ActionableError(format!("{failure:?}")),
                };
            }
            Err(error) => return BackgroundFetchResult::ActionableError(error.to_string()),
        };
        self.remove_fetch_ref(operation_id);
        match self.repository.classify_history(
            &local,
            &remote,
            policy.last_confirmed_remote.as_ref(),
        ) {
            Ok(HistoryRelation::Equal | HistoryRelation::Ahead { .. }) => {
                BackgroundFetchResult::Unchanged
            }
            Ok(HistoryRelation::Behind { commits }) => BackgroundFetchResult::Incoming { commits },
            Ok(HistoryRelation::Diverged { behind, .. }) => {
                BackgroundFetchResult::Incoming { commits: behind }
            }
            Ok(relation) => BackgroundFetchResult::ActionableError(format!("{relation:?}")),
            Err(error) => BackgroundFetchResult::ActionableError(error.to_string()),
        }
    }

    pub fn pull_updates(
        &self,
        plan: &SyncPlan,
        policy: &mut RepositoryPolicy,
        askpass: Option<&AskpassBridge>,
        cancellation: &CancellationToken,
    ) -> Result<SyncOutcome, SyncEngineError> {
        validate_operation_id(&plan.operation_id)?;
        let state = self.preflight(plan, policy, None)?;
        if !plan.paths.is_empty() {
            return Err(SyncEngineError::UnsafeRepositoryState);
        }
        let local = state.head.clone().ok_or(SyncEngineError::NoCommit)?;
        let mut checkpoint = self.new_checkpoint(plan, OperationKind::PullUpdates, &state);
        self.journal.begin(checkpoint.clone())?;
        checkpoint.phase = OperationPhase::Fetching;
        self.persist_checkpoint(&mut checkpoint)?;
        let remote = self
            .fetch_target(plan, askpass, cancellation)?
            .ok_or(SyncEngineError::UnsafeRemoteHistory)?;
        let relation = self.repository.classify_history(
            &local,
            &remote,
            policy.last_confirmed_remote.as_ref(),
        )?;
        if relation == HistoryRelation::Equal || matches!(relation, HistoryRelation::Ahead { .. }) {
            checkpoint.phase = OperationPhase::Complete;
            checkpoint.confirmed = true;
            self.persist_checkpoint(&mut checkpoint)?;
            self.journal
                .retire_success(&plan.operation_id, now_seconds())?;
            return Ok(SyncOutcome::UpToDate);
        }
        if !matches!(
            relation,
            HistoryRelation::Behind { .. } | HistoryRelation::Diverged { .. }
        ) {
            return Err(SyncEngineError::UnsafeRemoteHistory);
        }
        checkpoint.fetched_tip = Some(remote.clone());
        checkpoint.resulting_commit = Some(local.clone());
        checkpoint.phase = OperationPhase::Integrating;
        self.persist_checkpoint(&mut checkpoint)?;
        let commit = match self.integrate(&mut checkpoint, &local, &remote, relation, cancellation)
        {
            Ok(commit) => commit,
            Err(error) if self.repository.status()?.worktree.has_conflicts => {
                // Reconcile even if cancellation interrupted Git's response.
                let reconcile = CancellationToken::new();
                let paths = self.conflict_paths(&reconcile)?;
                checkpoint.owned_paths = paths
                    .iter()
                    .cloned()
                    .map(|path| (path, String::new()))
                    .collect();
                checkpoint.expected_index_fingerprint =
                    Some(self.conflict_state_fingerprint(&paths, &reconcile)?);
                checkpoint.phase = OperationPhase::Resolving;
                self.persist_checkpoint(&mut checkpoint)?;
                return Ok(SyncOutcome::NeedsAttention {
                    phase: OperationPhase::Resolving,
                    reason: error.to_string(),
                });
            }
            Err(error) => return Err(error),
        };
        self.finish(policy, &mut checkpoint, commit, Some(remote), cancellation)
    }

    pub fn push_commits(
        &self,
        plan: &SyncPlan,
        policy: &mut RepositoryPolicy,
        askpass: Option<&AskpassBridge>,
        cancellation: &CancellationToken,
    ) -> Result<SyncOutcome, SyncEngineError> {
        validate_operation_id(&plan.operation_id)?;
        let state = self.target_preflight(plan, policy)?;
        if !plan.paths.is_empty() {
            return Err(SyncEngineError::UnsafeRepositoryState);
        }
        let commit = state.head.clone().ok_or(SyncEngineError::NoCommit)?;
        let mut checkpoint = self.new_checkpoint(plan, OperationKind::PushCommits, &state);
        self.journal.begin(checkpoint.clone())?;
        checkpoint.phase = OperationPhase::Pushing;
        checkpoint.resulting_commit = Some(commit.clone());
        self.persist_checkpoint(&mut checkpoint)?;
        if let Err(error) = self.push_exact(plan, &commit, askpass, cancellation) {
            checkpoint.phase = OperationPhase::Verifying;
            self.persist_checkpoint(&mut checkpoint)?;
            if let Ok(Some(observed)) = self.fetch_target(plan, askpass, cancellation)
                && self.is_ancestor(&commit, &observed, cancellation)?
            {
                return self.finish(
                    policy,
                    &mut checkpoint,
                    commit,
                    Some(observed),
                    cancellation,
                );
            }
            return Ok(SyncOutcome::UncertainDelivery {
                commit,
                reason: error.to_string(),
            });
        }
        self.finish(
            policy,
            &mut checkpoint,
            commit.clone(),
            Some(commit),
            cancellation,
        )
    }

    /// Check a recorded delivery without pushing again or changing local files.
    pub fn verify_delivery(
        &self,
        checkpoint: &OperationCheckpoint,
        policy: &mut RepositoryPolicy,
        cancellation: &CancellationToken,
    ) -> Result<SyncOutcome, SyncEngineError> {
        let target = checkpoint
            .target
            .clone()
            .ok_or(SyncEngineError::TargetDrift)?;
        let commit = checkpoint
            .resulting_commit
            .clone()
            .ok_or(SyncEngineError::NoCommit)?;
        if self.repository.identity() != &checkpoint.identity
            || self.repository.resolve_sync_target()? != target
        {
            return Err(SyncEngineError::TargetDrift);
        }
        let plan = SyncPlan {
            operation_id: checkpoint.operation_id.clone(),
            identity: checkpoint.identity.clone(),
            target,
            expected_head: self.repository.status()?.head,
            paths: Vec::new(),
            content_fingerprints: Vec::new(),
            message: String::new(),
        };
        let observed = self.fetch_target(&plan, None, cancellation)?;
        if let Some(observed) = observed
            && self.is_ancestor(&commit, &observed, cancellation)?
        {
            return self.finish(
                policy,
                &mut checkpoint.clone(),
                commit,
                Some(observed),
                cancellation,
            );
        }
        Ok(SyncOutcome::AwaitingUpload {
            commit,
            reason: "recorded commit is not present in the observed remote history".into(),
        })
    }

    pub fn recover_owned_staging(
        &self,
        expected_paths: &[PathBuf],
        cancellation: &CancellationToken,
    ) -> Result<(), SyncEngineError> {
        if sorted_paths(self.staged_paths(cancellation)?) != sorted_paths(expected_paths.to_vec()) {
            return Err(SyncEngineError::UnsafeRepositoryState);
        }
        let head_exists = self.current_head(cancellation)?.is_some();
        let command = if head_exists {
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["reset", "-q", "HEAD"])
                .literal_paths(expected_paths)
        } else {
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["rm", "--cached", "-q", "-r"])
                .literal_paths(expected_paths)
        };
        self.run(command, CommandLimits::default(), cancellation)?;
        Ok(())
    }

    fn new_checkpoint(
        &self,
        plan: &SyncPlan,
        kind: OperationKind,
        state: &crate::RepositoryState,
    ) -> OperationCheckpoint {
        OperationCheckpoint {
            operation_id: plan.operation_id.clone(),
            kind,
            phase: OperationPhase::Preparing,
            identity: plan.identity.clone(),
            target: Some(plan.target.clone()),
            expected_head: state.head.clone(),
            resulting_commit: None,
            fetched_tip: None,
            expected_index_fingerprint: None,
            owned_paths: plan.content_fingerprints.clone(),
            recovery_refs: Vec::new(),
            preimages: None,
            drafts: Vec::new(),
            updated_unix_seconds: now_seconds(),
            confirmed: false,
        }
    }

    fn target_preflight(
        &self,
        plan: &SyncPlan,
        policy: &RepositoryPolicy,
    ) -> Result<crate::RepositoryState, SyncEngineError> {
        if self.repository.identity() != &plan.identity
            || !policy.validates_identity(self.repository.identity())
            || plan.target != policy.target
            || self.repository.resolve_sync_target()? != policy.target
        {
            return Err(SyncEngineError::TargetDrift);
        }
        let state = self.repository.status()?;
        if state.head != plan.expected_head {
            return Err(SyncEngineError::TargetDrift);
        }
        if !state.capabilities.supports_write_sync() {
            return Err(SyncEngineError::UnsupportedRepository);
        }
        Ok(state)
    }

    fn preflight(
        &self,
        plan: &SyncPlan,
        policy: &RepositoryPolicy,
        reviewed: Option<&ReviewPlan>,
    ) -> Result<crate::RepositoryState, SyncEngineError> {
        if plan.identity != *self.repository.identity()
            || !policy.validates_identity(self.repository.identity())
            || plan.target != policy.target
            || self.repository.resolve_sync_target()? != plan.target
        {
            return Err(SyncEngineError::TargetDrift);
        }
        let state = self.repository.status()?;
        if !state.capabilities.supports_write_sync() {
            return Err(SyncEngineError::UnsupportedRepository);
        }
        if state.head != plan.expected_head
            || state.worktree.has_external_staging
            || state.worktree.has_conflicts
            || state.worktree.operation_in_progress.is_some()
        {
            return Err(SyncEngineError::UnsafeRepositoryState);
        }

        let mut expected = Vec::new();
        for change in &state.worktree.changes {
            let absolute = self.repository.identity().worktree_root.join(&change.path);
            let is_symlink = fs::symlink_metadata(&absolute)
                .map(|metadata| metadata.file_type().is_symlink())
                .unwrap_or(false);
            let ignored = change.kind == ChangeKind::Untracked && self.is_ignored(&change.path);
            match policy.decide_path(&change.path, change.kind, ignored, is_symlink) {
                PathDecision::Automatic => expected.push(change.path.clone()),
                PathDecision::Ignored => {}
                PathDecision::Attention => return Err(SyncEngineError::StalePlan),
            }
            if change.kind == ChangeKind::Renamed
                && change
                    .original_path
                    .as_ref()
                    .is_some_and(|from| policy.rename_needs_review(from, &change.path))
            {
                return Err(SyncEngineError::StalePlan);
            }
        }
        if sorted_paths(expected) != sorted_paths(plan.paths.clone()) {
            return Err(SyncEngineError::StalePlan);
        }
        if let Some(reviewed) = reviewed
            && !reviewed.still_matches(plan.content_fingerprints.clone())
        {
            return Err(SyncEngineError::StalePlan);
        }
        Ok(state)
    }

    fn preflight_selected(
        &self,
        plan: &SyncPlan,
        policy: &RepositoryPolicy,
    ) -> Result<crate::RepositoryState, SyncEngineError> {
        if plan.identity != *self.repository.identity()
            || !policy.validates_identity(self.repository.identity())
            || plan.target != policy.target
            || self.repository.resolve_sync_target()? != plan.target
        {
            return Err(SyncEngineError::TargetDrift);
        }
        if plan.paths.is_empty() || plan.content_fingerprints.len() != plan.paths.len() {
            return Err(SyncEngineError::StalePlan);
        }
        let state = self.repository.status()?;
        if !state.capabilities.supports_write_sync() {
            return Err(SyncEngineError::UnsupportedRepository);
        }
        if state.head != plan.expected_head
            || state.worktree.has_external_staging
            || state.worktree.has_conflicts
            || state.worktree.operation_in_progress.is_some()
        {
            return Err(SyncEngineError::UnsafeRepositoryState);
        }

        let mut selected = plan.paths.clone();
        selected.sort();
        if selected.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(SyncEngineError::StalePlan);
        }
        for path in &selected {
            let change = state
                .worktree
                .changes
                .iter()
                .find(|change| change.path == *path)
                .ok_or(SyncEngineError::StalePlan)?;
            let absolute = self.repository.identity().worktree_root.join(path);
            let is_symlink = fs::symlink_metadata(&absolute)
                .map(|metadata| metadata.file_type().is_symlink())
                .unwrap_or(false);
            let ignored = change.kind == ChangeKind::Untracked && self.is_ignored(path);
            if policy.decide_path(path, change.kind, ignored, is_symlink) != PathDecision::Automatic
                || change
                    .original_path
                    .as_ref()
                    .is_some_and(|from| policy.rename_needs_review(from, path))
            {
                return Err(SyncEngineError::StalePlan);
            }
            let expected = plan
                .content_fingerprints
                .iter()
                .find_map(|(candidate, fingerprint)| {
                    (candidate == path).then_some(fingerprint.as_str())
                })
                .ok_or(SyncEngineError::StalePlan)?;
            if self.repository.content_fingerprint(path)? != expected {
                return Err(SyncEngineError::StalePlan);
            }
        }
        Ok(state)
    }

    fn stage_exact_paths(
        &self,
        plan: &SyncPlan,
        cancellation: &CancellationToken,
    ) -> Result<(), SyncEngineError> {
        self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .arg("add")
                .literal_paths(&plan.paths),
            CommandLimits::default(),
            cancellation,
        )?;
        if sorted_paths(self.staged_paths(cancellation)?) != sorted_paths(plan.paths.clone()) {
            return Err(SyncEngineError::UnsafeRepositoryState);
        }
        Ok(())
    }

    fn commit(
        &self,
        plan: &SyncPlan,
        cancellation: &CancellationToken,
    ) -> Result<GitObjectId, SyncEngineError> {
        self.run(
            GitCommand::new(&self.repository.identity().worktree_root).args([
                "commit",
                "-m",
                &plan.message,
            ]),
            CommandLimits {
                timeout: Duration::from_secs(120),
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        self.current_head(cancellation)?
            .ok_or(SyncEngineError::NoCommit)
    }

    fn remove_fetch_ref(&self, operation_id: &str) {
        let _ = self.run(
            GitCommand::new(&self.repository.identity().worktree_root).args([
                "update-ref",
                "-d",
                &format!("refs/markion/fetched/{operation_id}"),
            ]),
            CommandLimits::default(),
            &CancellationToken::new(),
        );
    }

    fn fetch_target(
        &self,
        plan: &SyncPlan,
        askpass: Option<&AskpassBridge>,
        cancellation: &CancellationToken,
    ) -> Result<Option<GitObjectId>, SyncEngineError> {
        let private_ref = format!("refs/markion/fetched/{}", plan.operation_id);
        let refspec = format!("+{}:{private_ref}", plan.target.destination_ref);
        let request = GitCommand::new(&self.repository.identity().worktree_root).args([
            "fetch",
            "--no-tags",
            "--no-recurse-submodules",
            &plan.target.remote,
            &refspec,
        ]);
        let request = self.network_request(request, askpass);
        match self.run(
            request,
            CommandLimits {
                timeout: self.options.network_timeout,
                ..CommandLimits::default()
            },
            cancellation,
        ) {
            Ok(_) => {
                let tip = self.rev_parse(&private_ref, cancellation)?;
                self.run(
                    GitCommand::new(&self.repository.identity().worktree_root).args([
                        "update-ref",
                        &GitRepository::observation_ref(&plan.target),
                        tip.as_str(),
                    ]),
                    CommandLimits::default(),
                    cancellation,
                )?;
                Ok(Some(tip))
            }
            Err(SyncEngineError::Command(GitCommandError::Failed { output }))
                if output.termination == ProcessTermination::Exited
                    && output.stderr_text().contains("couldn't find remote ref") =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    fn integrate(
        &self,
        checkpoint: &mut OperationCheckpoint,
        expected_head: &GitObjectId,
        remote_tip: &GitObjectId,
        relation: HistoryRelation,
        cancellation: &CancellationToken,
    ) -> Result<GitObjectId, SyncEngineError> {
        let state = self.repository.status()?;
        if state.head.as_ref() != Some(expected_head)
            || !state.worktree.changes.is_empty()
            || state.worktree.has_external_staging
        {
            return Err(SyncEngineError::UnsafeRepositoryState);
        }
        let incoming = self.incoming_paths(expected_head, remote_tip, cancellation)?;
        self.validate_incoming_paths(expected_head, remote_tip, &incoming, cancellation)?;
        let fingerprints = incoming
            .iter()
            .filter(|path| {
                self.repository
                    .identity()
                    .worktree_root
                    .join(path)
                    .is_file()
            })
            .map(|path| {
                self.file_fingerprint(path)
                    .map(|fingerprint| (path.clone(), fingerprint))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let manifest = self.journal.prepare_preimages(
            &checkpoint.operation_id,
            &self.repository.identity().worktree_root,
            &fingerprints,
            self.options.max_preimage_bytes,
        )?;
        let recovery_ref = format!("refs/markion/recovery/{}/baseline", checkpoint.operation_id);
        self.run(
            GitCommand::new(&self.repository.identity().worktree_root).args([
                "update-ref",
                &recovery_ref,
                expected_head.as_str(),
            ]),
            CommandLimits::default(),
            cancellation,
        )?;
        checkpoint.preimages = Some(manifest);
        checkpoint.recovery_refs.push(recovery_ref);
        self.persist_checkpoint(checkpoint)?;

        let command = match relation {
            HistoryRelation::Behind { .. } => {
                GitCommand::new(&self.repository.identity().worktree_root).args([
                    "merge",
                    "--ff-only",
                    remote_tip.as_str(),
                ])
            }
            HistoryRelation::Diverged { .. } => {
                GitCommand::new(&self.repository.identity().worktree_root).args([
                    "merge",
                    "--no-edit",
                    remote_tip.as_str(),
                ])
            }
            _ => return Ok(expected_head.clone()),
        };
        self.run(
            command,
            CommandLimits {
                timeout: Duration::from_secs(180),
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        self.current_head(cancellation)?
            .ok_or(SyncEngineError::NoCommit)
    }

    fn validate_incoming_paths(
        &self,
        local: &GitObjectId,
        remote: &GitObjectId,
        paths: &[PathBuf],
        cancellation: &CancellationToken,
    ) -> Result<(), SyncEngineError> {
        let mut casefolded = BTreeSet::new();
        for path in paths {
            self.repository.validate_write_path(path)?;
            let folded = path.to_string_lossy().to_lowercase();
            if !casefolded.insert(folded) {
                return Err(SyncEngineError::UnsafeIncomingPaths);
            }
            let local_entry = self.tree_entry(local, path, cancellation)?;
            let remote_entry = self.tree_entry(remote, path, cancellation)?;
            if remote_entry
                .as_deref()
                .is_some_and(|entry| entry.starts_with("120000 "))
            {
                return Err(SyncEngineError::UnsafeIncomingPaths);
            }
            if local_entry.is_none()
                && remote_entry.is_some()
                && self.repository.identity().worktree_root.join(path).exists()
            {
                return Err(SyncEngineError::UnsafeIncomingPaths);
            }
        }
        Ok(())
    }

    fn push_exact(
        &self,
        plan: &SyncPlan,
        commit: &GitObjectId,
        askpass: Option<&AskpassBridge>,
        cancellation: &CancellationToken,
    ) -> Result<(), SyncEngineError> {
        if self.repository.resolve_sync_target()? != plan.target
            || self.current_branch(cancellation)? != plan.target.local_branch
        {
            return Err(SyncEngineError::TargetDrift);
        }
        let refspec = format!("{}:{}", commit.as_str(), plan.target.destination_ref);
        let request = GitCommand::new(&self.repository.identity().worktree_root).args([
            "push",
            "--porcelain",
            &plan.target.remote,
            &refspec,
        ]);
        self.run(
            self.network_request(request, askpass),
            CommandLimits {
                timeout: self.options.network_timeout,
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        Ok(())
    }

    fn finish(
        &self,
        policy: &mut RepositoryPolicy,
        checkpoint: &mut OperationCheckpoint,
        commit: GitObjectId,
        observed_remote: Option<GitObjectId>,
        cancellation: &CancellationToken,
    ) -> Result<SyncOutcome, SyncEngineError> {
        let confirmed_remote = observed_remote.unwrap_or_else(|| commit.clone());
        self.run(
            GitCommand::new(&self.repository.identity().worktree_root).args([
                "update-ref",
                &GitRepository::observation_ref(&policy.target),
                confirmed_remote.as_str(),
            ]),
            CommandLimits::default(),
            &CancellationToken::new(),
        )?;
        policy.last_confirmed_remote = Some(confirmed_remote.clone());
        checkpoint.phase = OperationPhase::Complete;
        checkpoint.confirmed = true;
        checkpoint.resulting_commit = Some(commit.clone());
        checkpoint.fetched_tip = Some(confirmed_remote.clone());
        self.persist_checkpoint(checkpoint)?;
        self.journal
            .retire_success(&checkpoint.operation_id, now_seconds())?;
        for recovery_ref in &checkpoint.recovery_refs {
            let _ = self.run(
                GitCommand::new(&self.repository.identity().worktree_root).args([
                    "update-ref",
                    "-d",
                    recovery_ref,
                ]),
                CommandLimits::default(),
                cancellation,
            );
        }
        self.remove_fetch_ref(&checkpoint.operation_id);
        let pending = self.repository.status()?.worktree.changes.len();
        Ok(SyncOutcome::Synchronized {
            commit: Some(commit),
            remote_tip: Some(confirmed_remote),
            pending_local_changes: pending,
        })
    }

    fn incoming_paths(
        &self,
        local: &GitObjectId,
        remote: &GitObjectId,
        cancellation: &CancellationToken,
    ) -> Result<Vec<PathBuf>, SyncEngineError> {
        let output = self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["diff", "--name-only", "-z", local.as_str(), remote.as_str()])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 16 * 1024 * 1024,
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        Ok(output
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| PathBuf::from(String::from_utf8_lossy(path).into_owned()))
            .collect())
    }

    fn tree_entry(
        &self,
        commit: &GitObjectId,
        path: &Path,
        cancellation: &CancellationToken,
    ) -> Result<Option<String>, SyncEngineError> {
        let output = self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["ls-tree", commit.as_str()])
                .literal_paths([path])
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        )?;
        let text = String::from_utf8_lossy(&output).trim().to_string();
        Ok((!text.is_empty()).then_some(text))
    }

    fn staged_paths(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<Vec<PathBuf>, SyncEngineError> {
        let output = self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["diff", "--cached", "--name-status", "-z", "--find-renames"])
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        )?;
        parse_name_status_paths(&output).ok_or(SyncEngineError::UnsafeRepositoryState)
    }

    fn index_tree(&self, cancellation: &CancellationToken) -> Result<String, SyncEngineError> {
        let output = self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .arg("write-tree")
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        )?;
        Ok(String::from_utf8_lossy(&output).trim().to_string())
    }

    fn conflict_paths(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<Vec<PathBuf>, SyncEngineError> {
        let output = self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["diff", "--name-only", "--diff-filter=U", "-z"])
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        )?;
        Ok(output
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| PathBuf::from(String::from_utf8_lossy(path).into_owned()))
            .collect())
    }

    fn conflict_state_fingerprint(
        &self,
        paths: &[PathBuf],
        cancellation: &CancellationToken,
    ) -> Result<String, SyncEngineError> {
        let index = self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["ls-files", "--stage", "-z"])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 32 * 1024 * 1024,
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        let mut digest = Sha256::new();
        digest.update(index);
        for path in paths {
            digest.update(path.to_string_lossy().as_bytes());
            digest.update([0]);
            match fs::read(self.repository.identity().worktree_root.join(path)) {
                Ok(bytes) => digest.update(bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    digest.update(b"<missing>")
                }
                Err(error) => return Err(error.into()),
            }
            digest.update([0]);
        }
        Ok(format!("{:x}", digest.finalize()))
    }

    fn current_head(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<Option<GitObjectId>, SyncEngineError> {
        match self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["rev-parse", "--verify", "HEAD"])
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        ) {
            Ok(output) => GitObjectId::parse(String::from_utf8_lossy(&output).trim().to_string())
                .map(Some)
                .map_err(|_| SyncEngineError::NoCommit),
            Err(SyncEngineError::Command(GitCommandError::Failed { output }))
                if output.termination == ProcessTermination::Exited =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    fn rev_parse(
        &self,
        reference: &str,
        cancellation: &CancellationToken,
    ) -> Result<GitObjectId, SyncEngineError> {
        let output = self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["rev-parse", "--verify", reference])
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        )?;
        GitObjectId::parse(String::from_utf8_lossy(&output).trim().to_string())
            .map_err(|_| SyncEngineError::NoCommit)
    }

    fn current_branch(&self, cancellation: &CancellationToken) -> Result<String, SyncEngineError> {
        let output = self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        )?;
        Ok(String::from_utf8_lossy(&output).trim().to_string())
    }

    fn is_ancestor(
        &self,
        ancestor: &GitObjectId,
        descendant: &GitObjectId,
        cancellation: &CancellationToken,
    ) -> Result<bool, SyncEngineError> {
        match self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args([
                    "merge-base",
                    "--is-ancestor",
                    ancestor.as_str(),
                    descendant.as_str(),
                ])
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        ) {
            Ok(_) => Ok(true),
            Err(SyncEngineError::Command(GitCommandError::Failed { output }))
                if output.status_code == Some(1) =>
            {
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }

    fn is_ignored(&self, path: &Path) -> bool {
        self.run(
            GitCommand::new(&self.repository.identity().worktree_root)
                .args(["check-ignore", "-q"])
                .literal_paths([path])
                .read_only(true),
            CommandLimits::default(),
            &CancellationToken::new(),
        )
        .is_ok()
    }

    fn file_fingerprint(&self, path: &Path) -> Result<String, SyncEngineError> {
        let metadata = fs::metadata(self.repository.identity().worktree_root.join(path))?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| value.as_nanos())
            .unwrap_or_default();
        Ok(format!("{}:{modified}", metadata.len()))
    }

    fn network_request(&self, request: GitCommand, askpass: Option<&AskpassBridge>) -> GitCommand {
        let mut request = request.interactive_credentials(self.foreground_credentials);
        if !self.foreground_credentials && askpass.is_none() {
            request = noninteractive_network_request(request);
        }
        askpass.map_or(request.clone(), |bridge| {
            Authentication::new(self.repository.runner().clone()).with_askpass(request, bridge)
        })
    }

    fn persist_checkpoint(
        &self,
        checkpoint: &mut OperationCheckpoint,
    ) -> Result<(), SyncEngineError> {
        checkpoint.updated_unix_seconds = now_seconds();
        // Once integration preimages/refs exist, make that ownership durable
        // before an adapter can interrupt checkout. Other observer calls retain
        // their before-transition semantics, including unknown push-result
        // simulation after the remote accepted a commit.
        if checkpoint.phase == OperationPhase::Integrating && checkpoint.preimages.is_some() {
            self.journal.update(checkpoint.clone())?;
            if let Some(observer) = &self.observer {
                observer(checkpoint.phase)?;
            }
        } else {
            if let Some(observer) = &self.observer {
                observer(checkpoint.phase)?;
            }
            self.journal.update(checkpoint.clone())?;
        }
        Ok(())
    }

    fn run(
        &self,
        request: GitCommand,
        limits: CommandLimits,
        cancellation: &CancellationToken,
    ) -> Result<Vec<u8>, SyncEngineError> {
        Ok(self
            .repository
            .runner()
            .run(&request, limits, cancellation)?
            .stdout)
    }
}

fn noninteractive_network_request(mut request: GitCommand) -> GitCommand {
    request.arguments.splice(
        0..0,
        [
            std::ffi::OsString::from("-c"),
            std::ffi::OsString::from("core.askPass="),
            std::ffi::OsString::from("-c"),
            std::ffi::OsString::from("credential.interactive=never"),
        ],
    );
    request
        .interactive_credentials(false)
        .env("GCM_INTERACTIVE", "Never")
}

fn parse_name_status_paths(output: &[u8]) -> Option<Vec<PathBuf>> {
    let mut fields = output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty());
    let mut paths = Vec::new();
    while let Some(status_field) = fields.next() {
        let (status, first_path) = status_field
            .iter()
            .position(|byte| *byte == b'\t')
            .map(|tab| (&status_field[..tab], Some(&status_field[tab + 1..])))
            .unwrap_or((status_field, None));
        let status = String::from_utf8_lossy(status);
        let path = first_path.or_else(|| fields.next())?;
        paths.push(PathBuf::from(String::from_utf8_lossy(path).into_owned()));
        if status.starts_with('R') || status.starts_with('C') {
            let target = fields.next()?;
            paths.push(PathBuf::from(String::from_utf8_lossy(target).into_owned()));
        }
    }
    Some(paths)
}

impl From<std::io::Error> for SyncEngineError {
    fn from(error: std::io::Error) -> Self {
        Self::Repository(RepositoryError::Path(error))
    }
}

fn validate_operation_id(id: &str) -> Result<(), SyncEngineError> {
    if id.is_empty()
        || id.len() > 96
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        || id.starts_with('.')
    {
        return Err(SyncEngineError::InvalidOperationId);
    }
    Ok(())
}

fn sorted_paths(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort();
    paths.dedup();
    paths
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn is_non_fast_forward(error: &SyncEngineError) -> bool {
    let SyncEngineError::Command(GitCommandError::Failed { output }) = error else {
        return false;
    };
    let stderr = output.stderr_text().to_ascii_lowercase();
    stderr.contains("non-fast-forward") || stderr.contains("[rejected]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_network_boundary_disables_every_interactive_git_path() {
        let request = noninteractive_network_request(
            GitCommand::new(".")
                .args(["fetch", "origin"])
                .interactive_credentials(true),
        );
        let arguments = request
            .arguments
            .iter()
            .map(|argument| argument.to_string_lossy())
            .collect::<Vec<_>>();
        assert_eq!(
            &arguments[..4],
            ["-c", "core.askPass=", "-c", "credential.interactive=never"]
        );
        assert!(!request.allow_interactive_credentials);
        let debug = format!("{request:?}");
        assert!(debug.contains("GCM_INTERACTIVE"));
        assert!(!debug.contains("Never"));
    }

    #[test]
    fn staged_name_status_expands_both_rename_paths() {
        assert_eq!(
            parse_name_status_paths(b"M\0note.md\0R100\0old.md\0new.md\0").unwrap(),
            [
                PathBuf::from("note.md"),
                PathBuf::from("old.md"),
                PathBuf::from("new.md")
            ]
        );
        assert!(parse_name_status_paths(b"R100\0old.md\0").is_none());
    }
}
