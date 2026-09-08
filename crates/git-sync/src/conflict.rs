use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    CancellationToken, CommandLimits, ConflictDraft, ConflictKind, GitCommand, GitCommandError,
    GitObjectId, GitRepository, JournalError, JournalStore, OperationCheckpoint, OperationPhase,
    RepositoryError, SyncTarget, persist::atomic_write,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictSide {
    Base,
    ThisComputer,
    Remote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictSource {
    pub oid: GitObjectId,
    pub mode: String,
    pub bytes: Vec<u8>,
    pub binary: bool,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictFile {
    pub path: PathBuf,
    pub kind: ConflictKind,
    pub base: Option<(String, GitObjectId)>,
    pub local: Option<(String, GitObjectId)>,
    pub remote: Option<(String, GitObjectId)>,
}

impl ConflictFile {
    /// Rename/rename and unrecognized index layouts stay owned by Git and are
    /// intentionally left for an external Git tool.
    pub fn supports_in_app_resolution(&self) -> bool {
        !matches!(self.kind, ConflictKind::Rename | ConflictKind::Other)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictSession {
    pub operation_id: String,
    pub expected_head: GitObjectId,
    pub merge_head: GitObjectId,
    pub files: Vec<ConflictFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConflictResolution {
    Content(Vec<u8>),
    Delete,
    Choose(ConflictSide),
    KeepBoth {
        primary: ConflictSide,
        alternate_path: PathBuf,
        alternate: ConflictSide,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ConflictError {
    #[error(transparent)]
    Command(#[from] GitCommandError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Journal(#[from] JournalError),
    #[error("conflict session no longer owns the repository state")]
    StaleSession,
    #[error("requested conflict source does not exist")]
    MissingSource,
    #[error("conflict content is too large")]
    TooLarge,
    #[error("this conflict shape requires an external Git tool")]
    UnsupportedConflict,
    #[error("conflicts remain unresolved or a draft no longer matches disk")]
    IncompleteResolution,
    #[error("conflict filesystem operation failed: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct ConflictManager {
    repository: GitRepository,
    journal: JournalStore,
}

impl ConflictManager {
    pub fn new(repository: GitRepository, journal: JournalStore) -> Self {
        Self {
            repository,
            journal,
        }
    }

    pub fn restore_session(
        &self,
        operation_id: &str,
        cancellation: &CancellationToken,
    ) -> Result<ConflictSession, ConflictError> {
        let journal = self.journal.load()?;
        let checkpoint = journal
            .active
            .iter()
            .find(|checkpoint| {
                checkpoint.operation_id == operation_id
                    && checkpoint.phase == OperationPhase::Resolving
                    && checkpoint.identity == *self.repository.identity()
            })
            .ok_or(ConflictError::StaleSession)?;
        let expected_head = checkpoint
            .resulting_commit
            .clone()
            .or_else(|| checkpoint.expected_head.clone())
            .ok_or(ConflictError::StaleSession)?;
        let actual_head = self.rev_parse("HEAD", cancellation)?;
        if actual_head != expected_head {
            return Err(ConflictError::StaleSession);
        }
        let merge_head = self.rev_parse("MERGE_HEAD", cancellation)?;
        if checkpoint.fetched_tip.as_ref() != Some(&merge_head) {
            return Err(ConflictError::StaleSession);
        }
        let files = self.read_index_conflicts(cancellation)?;
        Ok(ConflictSession {
            operation_id: operation_id.to_string(),
            expected_head,
            merge_head,
            files,
        })
    }

    pub fn source(
        &self,
        file: &ConflictFile,
        side: ConflictSide,
        max_bytes: usize,
        cancellation: &CancellationToken,
    ) -> Result<ConflictSource, ConflictError> {
        let (mode, oid) = source_entry(file, side).ok_or(ConflictError::MissingSource)?;
        let output = self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .args(["cat-file", "blob", oid.as_str()])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: max_bytes,
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        Ok(ConflictSource {
            oid: oid.clone(),
            mode: mode.clone(),
            binary: output.stdout.contains(&0),
            bytes: output.stdout,
            truncated: output.stdout_truncated,
        })
    }

    /// Draft persistence is independent from the worktree and never stages.
    pub fn save_draft(
        &self,
        session: &ConflictSession,
        path: PathBuf,
        content: Vec<u8>,
        cancellation: &CancellationToken,
    ) -> Result<String, ConflictError> {
        self.verify_session(session, cancellation)?;
        let fingerprint = self.hash_bytes(&content, cancellation)?;
        self.journal.save_draft(
            &session.operation_id,
            ConflictDraft {
                relative_path: path,
                content,
                content_fingerprint: fingerprint.clone(),
            },
        )?;
        Ok(fingerprint)
    }

    pub fn mark_resolved(
        &self,
        session: &ConflictSession,
        path: &Path,
        resolution: ConflictResolution,
        cancellation: &CancellationToken,
    ) -> Result<(), ConflictError> {
        self.verify_session(session, cancellation)?;
        let expected = session
            .files
            .iter()
            .find(|file| file.path == path)
            .ok_or(ConflictError::StaleSession)?;
        if !expected.supports_in_app_resolution() {
            return Err(ConflictError::UnsupportedConflict);
        }
        let current = self
            .read_index_conflicts(cancellation)?
            .into_iter()
            .find(|file| file.path == path)
            .ok_or(ConflictError::StaleSession)?;
        if current != *expected {
            return Err(ConflictError::StaleSession);
        }
        const MAX_RESOLUTION_SOURCE_BYTES: usize = 64 * 1024 * 1024;
        match resolution {
            ConflictResolution::Delete => self.delete_and_stage(path, cancellation),
            ConflictResolution::Content(bytes) => self.write_and_stage(path, &bytes, cancellation),
            ConflictResolution::Choose(side) => {
                let source =
                    self.source(expected, side, MAX_RESOLUTION_SOURCE_BYTES, cancellation)?;
                if source.truncated {
                    return Err(ConflictError::TooLarge);
                }
                self.write_and_stage(path, &source.bytes, cancellation)
            }
            ConflictResolution::KeepBoth {
                primary,
                alternate_path,
                alternate,
            } => {
                if alternate_path == path
                    || self
                        .repository
                        .identity()
                        .worktree_root
                        .join(&alternate_path)
                        .exists()
                {
                    return Err(ConflictError::StaleSession);
                }
                self.repository.validate_write_path(&alternate_path)?;
                let primary =
                    self.source(expected, primary, MAX_RESOLUTION_SOURCE_BYTES, cancellation)?;
                let alternate = self.source(
                    expected,
                    alternate,
                    MAX_RESOLUTION_SOURCE_BYTES,
                    cancellation,
                )?;
                if primary.truncated || alternate.truncated {
                    return Err(ConflictError::TooLarge);
                }
                self.write_and_stage(path, &primary.bytes, cancellation)?;
                self.write_and_stage(&alternate_path, &alternate.bytes, cancellation)
            }
        }?;
        self.journal.clear_draft(&session.operation_id, path)?;
        let mut checkpoint = self.checkpoint(&session.operation_id)?;
        checkpoint.expected_index_fingerprint =
            Some(self.conflict_state_fingerprint(session, cancellation)?);
        checkpoint.updated_unix_seconds = now_seconds();
        self.journal.update(checkpoint)?;
        Ok(())
    }

    pub fn finish_merge(
        &self,
        session: &ConflictSession,
        cancellation: &CancellationToken,
    ) -> Result<GitObjectId, ConflictError> {
        self.verify_session(session, cancellation)?;
        if !self.read_index_conflicts(cancellation)?.is_empty() {
            return Err(ConflictError::IncompleteResolution);
        }
        let checkpoint = self.checkpoint(&session.operation_id)?;
        if !checkpoint.drafts.is_empty() {
            return Err(ConflictError::IncompleteResolution);
        }
        self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .args(["commit", "--no-edit"]),
            CommandLimits {
                timeout: std::time::Duration::from_secs(120),
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        let commit = self.rev_parse("HEAD", cancellation)?;
        let mut checkpoint = checkpoint;
        checkpoint.phase = OperationPhase::Fetching;
        checkpoint.resulting_commit = Some(commit.clone());
        checkpoint.updated_unix_seconds = now_seconds();
        self.journal.update(checkpoint)?;
        Ok(commit)
    }

    pub fn finish_and_push(
        &self,
        session: &ConflictSession,
        target: &SyncTarget,
        cancellation: &CancellationToken,
    ) -> Result<GitObjectId, ConflictError> {
        let commit = self.finish_merge(session, cancellation)?;
        if self.repository.resolve_sync_target()? != *target {
            return Err(ConflictError::StaleSession);
        }
        let mut checkpoint = self.checkpoint(&session.operation_id)?;
        checkpoint.phase = OperationPhase::Pushing;
        self.journal.update(checkpoint)?;
        let refspec = format!("{}:{}", commit.as_str(), target.destination_ref);
        self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .args(["push", "--porcelain", &target.remote, &refspec])
                .interactive_credentials(true),
            CommandLimits {
                timeout: std::time::Duration::from_secs(300),
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        let mut checkpoint = self.checkpoint(&session.operation_id)?;
        checkpoint.phase = OperationPhase::Complete;
        checkpoint.confirmed = true;
        checkpoint.resulting_commit = Some(commit.clone());
        checkpoint.updated_unix_seconds = now_seconds();
        self.journal.update(checkpoint)?;
        self.journal
            .retire_success(&session.operation_id, now_seconds())?;
        Ok(commit)
    }

    pub fn abort(
        &self,
        session: &ConflictSession,
        cancellation: &CancellationToken,
    ) -> Result<(), ConflictError> {
        self.verify_session(session, cancellation)?;
        self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root).args(["merge", "--abort"]),
            CommandLimits::default(),
            cancellation,
        )?;
        let mut checkpoint = self.checkpoint(&session.operation_id)?;
        checkpoint.phase = OperationPhase::NeedsAttention;
        checkpoint.updated_unix_seconds = now_seconds();
        self.journal.update(checkpoint)?;
        Ok(())
    }

    fn read_index_conflicts(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<Vec<ConflictFile>, ConflictError> {
        let output = self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .args(["ls-files", "-u", "-z"])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 16 * 1024 * 1024,
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        let mut grouped: BTreeMap<PathBuf, [Option<(String, GitObjectId)>; 3]> = BTreeMap::new();
        for record in output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|value| !value.is_empty())
        {
            let tab = record
                .iter()
                .position(|byte| *byte == b'\t')
                .ok_or(ConflictError::StaleSession)?;
            let header = String::from_utf8_lossy(&record[..tab]);
            let mut fields = header.split_whitespace();
            let mode = fields
                .next()
                .ok_or(ConflictError::StaleSession)?
                .to_string();
            let oid = GitObjectId::parse(
                fields
                    .next()
                    .ok_or(ConflictError::StaleSession)?
                    .to_string(),
            )
            .map_err(|_| ConflictError::StaleSession)?;
            let stage = fields
                .next()
                .and_then(|value| value.parse::<usize>().ok())
                .filter(|stage| (1..=3).contains(stage))
                .ok_or(ConflictError::StaleSession)?;
            let path = PathBuf::from(String::from_utf8_lossy(&record[tab + 1..]).into_owned());
            grouped.entry(path).or_default()[stage - 1] = Some((mode, oid));
        }
        Ok(grouped
            .into_iter()
            .map(|(path, [base, local, remote])| ConflictFile {
                kind: classify_conflict(&base, &local, &remote),
                path,
                base,
                local,
                remote,
            })
            .collect())
    }

    fn verify_session(
        &self,
        session: &ConflictSession,
        cancellation: &CancellationToken,
    ) -> Result<(), ConflictError> {
        let head = self.rev_parse("HEAD", cancellation)?;
        let merge = self.rev_parse("MERGE_HEAD", cancellation)?;
        if head != session.expected_head || merge != session.merge_head {
            return Err(ConflictError::StaleSession);
        }
        let checkpoint = self.checkpoint(&session.operation_id)?;
        if checkpoint.phase != OperationPhase::Resolving
            || checkpoint.identity != *self.repository.identity()
        {
            return Err(ConflictError::StaleSession);
        }
        if let Some(expected) = checkpoint.expected_index_fingerprint.as_deref() {
            let actual = self.conflict_state_fingerprint(session, cancellation)?;
            if actual != expected {
                return Err(ConflictError::StaleSession);
            }
        }
        Ok(())
    }

    fn conflict_state_fingerprint(
        &self,
        session: &ConflictSession,
        cancellation: &CancellationToken,
    ) -> Result<String, ConflictError> {
        let index = self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .args(["ls-files", "--stage", "-z"])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 32 * 1024 * 1024,
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        let mut digest = Sha256::new();
        digest.update(&index.stdout);
        let checkpoint = self.checkpoint(&session.operation_id)?;
        for (path, _) in &checkpoint.owned_paths {
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

    fn checkpoint(&self, operation_id: &str) -> Result<OperationCheckpoint, ConflictError> {
        self.journal
            .load()?
            .active
            .into_iter()
            .find(|checkpoint| checkpoint.operation_id == operation_id)
            .ok_or(ConflictError::StaleSession)
    }

    fn write_and_stage(
        &self,
        relative: &Path,
        bytes: &[u8],
        cancellation: &CancellationToken,
    ) -> Result<(), ConflictError> {
        let path = self.repository.validate_write_path(relative)?;
        atomic_write(&path, bytes)?;
        self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .arg("add")
                .literal_paths([relative]),
            CommandLimits::default(),
            cancellation,
        )?;
        Ok(())
    }

    fn delete_and_stage(
        &self,
        relative: &Path,
        cancellation: &CancellationToken,
    ) -> Result<(), ConflictError> {
        let path = self.repository.validate_write_path(relative)?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .args(["rm", "-q", "--ignore-unmatch"])
                .literal_paths([relative]),
            CommandLimits::default(),
            cancellation,
        )?;
        Ok(())
    }

    fn hash_bytes(
        &self,
        bytes: &[u8],
        cancellation: &CancellationToken,
    ) -> Result<String, ConflictError> {
        let output = self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .args(["hash-object", "--stdin"])
                .standard_input(bytes.to_vec())
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        )?;
        Ok(output.stdout_text().trim().to_string())
    }

    fn rev_parse(
        &self,
        reference: &str,
        cancellation: &CancellationToken,
    ) -> Result<GitObjectId, ConflictError> {
        let output = self.repository.runner().run(
            &GitCommand::new(&self.repository.identity().worktree_root)
                .args(["rev-parse", "--verify", reference])
                .read_only(true),
            CommandLimits::default(),
            cancellation,
        )?;
        GitObjectId::parse(output.stdout_text().trim().to_string())
            .map_err(|_| ConflictError::StaleSession)
    }
}

fn source_entry(file: &ConflictFile, side: ConflictSide) -> Option<&(String, GitObjectId)> {
    match side {
        ConflictSide::Base => file.base.as_ref(),
        ConflictSide::ThisComputer => file.local.as_ref(),
        ConflictSide::Remote => file.remote.as_ref(),
    }
}

fn classify_conflict(
    base: &Option<(String, GitObjectId)>,
    local: &Option<(String, GitObjectId)>,
    remote: &Option<(String, GitObjectId)>,
) -> ConflictKind {
    match (base.is_some(), local.is_some(), remote.is_some()) {
        (true, true, true) => ConflictKind::BothModified,
        (false, true, true) => ConflictKind::BothAdded,
        (true, false, true) => ConflictKind::DeletedByLocal,
        (true, true, false) => ConflictKind::DeletedByRemote,
        (true, false, false) => ConflictKind::BothDeleted,
        _ => ConflictKind::Other,
    }
}

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryAssessment {
    OwnedStaging {
        operation_id: String,
    },
    CommitCompleted {
        operation_id: String,
        commit: GitObjectId,
    },
    ConflictSession {
        operation_id: String,
    },
    IntegrationCompleted {
        operation_id: String,
        head: GitObjectId,
    },
    UnknownPushResult {
        operation_id: String,
        commit: GitObjectId,
    },
    ExternalState {
        operation_id: String,
    },
}

#[derive(Clone, Debug)]
pub struct RecoveryManager {
    repository: GitRepository,
    journal: JournalStore,
}

impl RecoveryManager {
    pub fn new(repository: GitRepository, journal: JournalStore) -> Self {
        Self {
            repository,
            journal,
        }
    }

    fn oid(&self, name: &str) -> Option<GitObjectId> {
        self.repository
            .runner()
            .run(
                &GitCommand::new(&self.repository.identity().worktree_root)
                    .args(["rev-parse", "--verify", name])
                    .read_only(true),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .ok()
            .and_then(|output| GitObjectId::parse(output.stdout_text().trim().to_string()).ok())
    }

    fn index_tree(&self) -> Option<String> {
        self.repository
            .runner()
            .run(
                &GitCommand::new(&self.repository.identity().worktree_root).arg("write-tree"),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .ok()
            .map(|output| output.stdout_text().trim().to_string())
    }

    fn ancestor(&self, from: &GitObjectId, to: &GitObjectId) -> bool {
        self.repository
            .runner()
            .run(
                &GitCommand::new(&self.repository.identity().worktree_root)
                    .args(["merge-base", "--is-ancestor", from.as_str(), to.as_str()])
                    .read_only(true),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .is_ok()
    }

    /// Explicit recovery: only the exact app-staged tree may be unstaged.
    /// Worktree content is retained, so the next operation can capture it anew.
    pub fn recover_staging(&self, operation_id: &str) -> Result<(), ConflictError> {
        if !self.assess()?.iter().any(|assessment| matches!(assessment, RecoveryAssessment::OwnedStaging { operation_id: id } if id == operation_id)) {
            return Err(ConflictError::StaleSession);
        }
        let checkpoint = self
            .journal
            .load()?
            .active
            .into_iter()
            .find(|entry| entry.operation_id == operation_id)
            .ok_or(ConflictError::StaleSession)?;
        let paths: Vec<_> = checkpoint
            .owned_paths
            .iter()
            .map(|(path, _)| path.clone())
            .collect();
        crate::GitSyncEngine::new(
            self.repository.clone(),
            self.journal.clone(),
            crate::SyncOptions::default(),
        )
        .recover_owned_staging(&paths, &CancellationToken::new())
        .map_err(|_| ConflictError::StaleSession)?;
        self.complete_checkpoint(checkpoint, None)
    }

    /// Adopt a completed commit/integration only when its actual topology agrees
    /// with the durable operation. This never writes the index or working files.
    pub fn adopt_completed(&self, operation_id: &str) -> Result<GitObjectId, ConflictError> {
        let commit = self
            .assess()?
            .into_iter()
            .find_map(|assessment| match assessment {
                RecoveryAssessment::CommitCompleted {
                    operation_id: id,
                    commit,
                } if id == operation_id => Some(commit),
                RecoveryAssessment::IntegrationCompleted {
                    operation_id: id,
                    head,
                } if id == operation_id => Some(head),
                _ => None,
            })
            .ok_or(ConflictError::StaleSession)?;
        let checkpoint = self
            .journal
            .load()?
            .active
            .into_iter()
            .find(|entry| entry.operation_id == operation_id)
            .ok_or(ConflictError::StaleSession)?;
        if !checkpoint.drafts.is_empty() {
            return Err(ConflictError::IncompleteResolution);
        }
        self.complete_checkpoint(checkpoint, Some(commit.clone()))?;
        Ok(commit)
    }

    fn complete_checkpoint(
        &self,
        mut checkpoint: OperationCheckpoint,
        commit: Option<GitObjectId>,
    ) -> Result<(), ConflictError> {
        checkpoint.phase = OperationPhase::Complete;
        checkpoint.confirmed = true;
        if commit.is_some() {
            checkpoint.resulting_commit = commit;
        }
        let id = checkpoint.operation_id.clone();
        self.journal.update(checkpoint)?;
        self.journal.retire_success(&id, now_seconds())?;
        Ok(())
    }

    pub fn assess(&self) -> Result<Vec<RecoveryAssessment>, ConflictError> {
        let journal = self.journal.load()?;
        let state = self.repository.status()?;
        let mut assessments = Vec::new();
        for checkpoint in journal
            .active
            .into_iter()
            .filter(|checkpoint| checkpoint.identity == *self.repository.identity())
        {
            let operation_id = checkpoint.operation_id.clone();
            let expected = checkpoint
                .resulting_commit
                .as_ref()
                .or(checkpoint.expected_head.as_ref());
            let merge = self.oid("MERGE_HEAD");
            let assessment = if checkpoint.phase == OperationPhase::Resolving
                && state.head.as_ref() == expected
                && merge.as_ref() == checkpoint.fetched_tip.as_ref()
                && merge.is_some()
            {
                RecoveryAssessment::ConflictSession { operation_id }
            } else if matches!(
                checkpoint.phase,
                OperationPhase::Pushing | OperationPhase::Verifying
            ) {
                checkpoint.resulting_commit.clone().map_or(
                    RecoveryAssessment::ExternalState {
                        operation_id: operation_id.clone(),
                    },
                    |commit| RecoveryAssessment::UnknownPushResult {
                        operation_id,
                        commit,
                    },
                )
            } else if state.worktree.has_external_staging {
                if state.head == checkpoint.expected_head
                    && checkpoint.phase == OperationPhase::Committing
                    && self.index_tree().as_deref()
                        == checkpoint.expected_index_fingerprint.as_deref()
                    && checkpoint.expected_index_fingerprint.is_some()
                {
                    RecoveryAssessment::OwnedStaging { operation_id }
                } else {
                    RecoveryAssessment::ExternalState { operation_id }
                }
            } else if merge.is_none() && !state.worktree.has_conflicts {
                if let Some(head) = state.head.clone() {
                    let committed = checkpoint.resulting_commit.as_ref() == Some(&head)
                        || (checkpoint.phase == OperationPhase::Committing
                            && self.oid("HEAD^") == checkpoint.expected_head
                            && self.oid("HEAD^{tree}").as_ref().map(|oid| oid.as_str())
                                == checkpoint.expected_index_fingerprint.as_deref()
                            && checkpoint.expected_index_fingerprint.is_some());
                    if matches!(
                        checkpoint.phase,
                        OperationPhase::Integrating | OperationPhase::Resolving
                    ) && checkpoint
                        .fetched_tip
                        .as_ref()
                        .is_some_and(|tip| self.ancestor(tip, &head))
                        && expected.is_some_and(|base| self.ancestor(base, &head))
                    {
                        RecoveryAssessment::IntegrationCompleted { operation_id, head }
                    } else if committed {
                        RecoveryAssessment::CommitCompleted {
                            operation_id,
                            commit: head,
                        }
                    } else {
                        RecoveryAssessment::ExternalState { operation_id }
                    }
                } else {
                    RecoveryAssessment::ExternalState { operation_id }
                }
            } else {
                RecoveryAssessment::ExternalState { operation_id }
            };
            assessments.push(assessment);
        }
        Ok(assessments)
    }
}
