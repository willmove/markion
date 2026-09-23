use std::{
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    AskpassBridge, Authentication, CancellationToken, CommandLimits, GitCommand, GitCommandError,
    GitCommandRunner, GitObjectId, GitRepository, RemoteUrl, RemoteUrlError, RepositoryError,
    RepositoryIdentity, RepositoryState, SyncTarget, tracked_path_is_notes_like,
};

static NEXT_CLONE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, thiserror::Error)]
pub enum OnboardingError {
    #[error(transparent)]
    Remote(#[from] RemoteUrlError),
    #[error(transparent)]
    Command(#[from] GitCommandError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("onboarding filesystem operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("clone destination already exists")]
    DestinationExists,
    #[error("repository already has a remote named {0}; it was not replaced")]
    RemoteExists(String),
    #[error("remote {remote} branch {branch} has no common history with this repository; clone it to a separate folder or integrate it manually")]
    UnrelatedRemoteHistory {
        remote: String,
        branch: String,
    },
    #[error("requested branch is invalid or unavailable")]
    InvalidBranch,
    #[error("repository has no commit to publish")]
    NothingToPublish,
    #[error("initial notes snapshot has no eligible files")]
    NoInitialNotes,
    #[error("initial notes snapshot changed while it was being prepared")]
    UnsafeInitialSnapshot,
    #[error("initial notes snapshot requires a nonblank message")]
    InvalidMessage,
}

#[derive(Clone, Debug)]
pub struct OnboardingService {
    runner: GitCommandRunner,
    foreground_credentials: bool,
}

#[derive(Clone, Debug)]
pub struct ConnectReview {
    pub repository: GitRepository,
    pub state: RepositoryState,
    pub target: Option<SyncTarget>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloneResult {
    pub worktree: PathBuf,
    pub selected_branch: String,
    pub empty_remote: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationResult {
    pub commit: GitObjectId,
    pub target: SyncTarget,
}

impl OnboardingService {
    pub fn new(runner: GitCommandRunner) -> Self {
        Self {
            runner,
            foreground_credentials: false,
        }
    }

    /// Explicit user-triggered setup may use the platform credential helper or
    /// SSH agent UI. Background callers deliberately never enable this flag.
    pub fn with_foreground_credentials(mut self) -> Self {
        self.foreground_credentials = true;
        self
    }

    pub fn clone_repository(
        &self,
        remote: &str,
        destination: &Path,
        explicit_branch: Option<&str>,
        askpass: Option<&AskpassBridge>,
        cancellation: &CancellationToken,
    ) -> Result<CloneResult, OnboardingError> {
        let remote = RemoteUrl::parse(remote)?;
        if destination.exists() {
            return Err(OnboardingError::DestinationExists);
        }
        let parent = destination
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let parent = std::path::absolute(parent)?;
        let destination_name = destination
            .file_name()
            .ok_or(OnboardingError::DestinationExists)?;
        let id = NEXT_CLONE_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{}.markion-clone-{}-{id}",
            destination_name.to_string_lossy(),
            std::process::id()
        ));
        if temporary.exists() {
            return Err(OnboardingError::DestinationExists);
        }

        let request = GitCommand::new(&parent).args([
            OsString::from("clone"),
            OsString::from("--no-tags"),
            OsString::from("--origin"),
            OsString::from("origin"),
            OsString::from(remote.as_str()),
            temporary.as_os_str().to_owned(),
        ]);
        let request = self.network_request(request, askpass);
        if let Err(error) = self.runner.run(
            &request,
            CommandLimits {
                timeout: std::time::Duration::from_secs(300),
                ..CommandLimits::default()
            },
            cancellation,
        ) {
            cleanup_owned_clone(&parent, &temporary);
            return Err(error.into());
        }

        let outcome = (|| {
            let empty_remote = self
                .run_optional(
                    GitCommand::new(&temporary)
                        .args(["rev-parse", "--verify", "HEAD"])
                        .read_only(true),
                    cancellation,
                )?
                .is_none();
            let selected_branch = if let Some(branch) = explicit_branch {
                self.validate_branch(&temporary, branch, cancellation)?;
                if empty_remote {
                    self.run(
                        GitCommand::new(&temporary).args([
                            "symbolic-ref",
                            "HEAD",
                            &format!("refs/heads/{branch}"),
                        ]),
                        cancellation,
                    )?;
                } else {
                    self.run(
                        GitCommand::new(&temporary).args(["switch", branch]),
                        cancellation,
                    )?;
                }
                branch.to_string()
            } else if empty_remote {
                let branch = "main";
                self.validate_branch(&temporary, branch, cancellation)?;
                self.run(
                    GitCommand::new(&temporary).args([
                        "symbolic-ref",
                        "HEAD",
                        &format!("refs/heads/{branch}"),
                    ]),
                    cancellation,
                )?;
                branch.to_string()
            } else {
                self.current_branch(&temporary, cancellation)?
            };
            if destination.exists() {
                return Err(OnboardingError::DestinationExists);
            }
            fs::rename(&temporary, destination)?;
            Ok(CloneResult {
                worktree: dunce::canonicalize(destination)?,
                selected_branch,
                empty_remote,
            })
        })();
        if outcome.is_err() {
            cleanup_owned_clone(&parent, &temporary);
        }
        outcome
    }

    pub fn connect_existing(&self, path: &Path) -> Result<ConnectReview, OnboardingError> {
        let repository = GitRepository::discover(self.runner.clone(), path)?;
        let state = repository.status()?;
        let target = repository.resolve_sync_target().ok();
        Ok(ConnectReview {
            repository,
            state,
            target,
        })
    }

    pub fn initialize_notes(
        &self,
        directory: &Path,
        branch: &str,
        remote: Option<&str>,
    ) -> Result<ConnectReview, OnboardingError> {
        fs::create_dir_all(directory)?;
        self.validate_branch(directory, branch, &CancellationToken::new())?;
        if directory.join(".git").exists() {
            return self.connect_existing(directory);
        }
        self.run(
            GitCommand::new(directory).args(["init", "-b", branch]),
            &CancellationToken::new(),
        )?;
        if let Some(remote) = remote {
            let remote = RemoteUrl::parse(remote)?;
            if self
                .run_optional(
                    GitCommand::new(directory)
                        .args(["remote", "get-url", "origin"])
                        .read_only(true),
                    &CancellationToken::new(),
                )?
                .is_some()
            {
                return Err(OnboardingError::RemoteExists("origin".into()));
            }
            self.run(
                GitCommand::new(directory).args(["remote", "add", "origin", remote.as_str()]),
                &CancellationToken::new(),
            )?;
        }
        self.connect_existing(directory)
    }

    /// Attach `origin` only when it is absent. An existing matching remote is
    /// accepted for resumable setup; a different URL is never replaced.
    pub fn attach_origin(
        &self,
        repository: &GitRepository,
        remote: &str,
        allow_replace: bool,
        cancellation: &CancellationToken,
    ) -> Result<(), OnboardingError> {
        let remote = RemoteUrl::parse(remote)?;
        let existing = self.origin_url(repository, cancellation)?;
        if let Some(existing) = existing {
            if existing.trim() == remote.as_str() {
                return Ok(());
            }
            if !allow_replace {
                return Err(OnboardingError::RemoteExists("origin".into()));
            }
            self.run(
                GitCommand::new(&repository.identity().worktree_root).args([
                    "remote",
                    "set-url",
                    "origin",
                    remote.as_str(),
                ]),
                cancellation,
            )?;
            return Ok(());
        }
        self.run(
            GitCommand::new(&repository.identity().worktree_root).args([
                "remote",
                "add",
                "origin",
                remote.as_str(),
            ]),
            cancellation,
        )?;
        Ok(())
    }

    /// Checks that a sync address can be contacted before setup applies it.
    /// Every failure — bad address, missing repository, denied access,
    /// timeout — maps to the same error so callers can offer one
    /// warn-and-confirm action. Runs outside any repository; an empty but
    /// reachable remote succeeds because ls-remote without --exit-code
    /// exits zero when no refs are listed.
    pub fn probe_remote(
        &self,
        remote: &str,
        directory: &Path,
        cancellation: &CancellationToken,
    ) -> Result<(), OnboardingError> {
        let remote = RemoteUrl::parse(remote)?;
        let request = GitCommand::new(directory)
            .args(["ls-remote", remote.as_str()])
            .read_only(true);
        self.run(self.network_request(request, None), cancellation)?;
        Ok(())
    }

    pub fn origin_url(
        &self,
        repository: &GitRepository,
        cancellation: &CancellationToken,
    ) -> Result<Option<String>, OnboardingError> {        Ok(self.run_optional(
            GitCommand::new(&repository.identity().worktree_root)
                .args(["remote", "get-url", "origin"])
                .read_only(true),
            cancellation,
        )?)
    }

    /// Create the first commit for a newly initialized notes repository. Only
    /// the exact reviewed note/resource paths are staged; failure deliberately
    /// leaves that owned staging visible for a safe retry or manual inspection.
    pub fn create_initial_commit(
        &self,
        repository: &GitRepository,
        paths: &[PathBuf],
        message: &str,
        cancellation: &CancellationToken,
    ) -> Result<GitObjectId, OnboardingError> {
        if message.trim().is_empty() {
            return Err(OnboardingError::InvalidMessage);
        }
        let state = repository.status()?;
        if state.head.is_some()
            || state.worktree.has_external_staging
            || state.worktree.has_conflicts
            || state.worktree.operation_in_progress.is_some()
            || !state.capabilities.supports_write_sync()
        {
            return Err(OnboardingError::UnsafeInitialSnapshot);
        }
        let mut expected = paths.to_vec();
        expected.sort();
        expected.dedup();
        if expected.is_empty() {
            return Err(OnboardingError::NoInitialNotes);
        }
        for path in &expected {
            repository.validate_write_path(path)?;
            if !repository.identity().worktree_root.join(path).is_file() {
                return Err(OnboardingError::UnsafeInitialSnapshot);
            }
        }
        self.run(
            GitCommand::new(&repository.identity().worktree_root)
                .arg("add")
                .literal_paths(&expected),
            cancellation,
        )?;
        let staged_state = repository.status()?;
        let mut staged = staged_state
            .worktree
            .changes
            .iter()
            .filter(|change| change.index_status != '.' && change.index_status != '?')
            .map(|change| change.path.clone())
            .collect::<Vec<_>>();
        staged.sort();
        staged.dedup();
        if staged != expected {
            return Err(OnboardingError::UnsafeInitialSnapshot);
        }
        self.run(
            GitCommand::new(&repository.identity().worktree_root).args([
                "commit",
                "-m",
                message.trim(),
            ]),
            cancellation,
        )?;
        let head = self
            .run_optional(
                GitCommand::new(&repository.identity().worktree_root)
                    .args(["rev-parse", "--verify", "HEAD"])
                    .read_only(true),
                cancellation,
            )?
            .ok_or(OnboardingError::NothingToPublish)?;
        GitObjectId::parse(head.trim().to_string()).map_err(|_| OnboardingError::NothingToPublish)
    }

    pub fn current_branch_name(
        &self,
        repository: &GitRepository,
        cancellation: &CancellationToken,
    ) -> Result<String, OnboardingError> {
        self.current_branch(&repository.identity().worktree_root, cancellation)
    }

    pub fn publish_first_branch(
        &self,
        repository: &GitRepository,
        remote_name: &str,
        remote_branch: &str,
        askpass: Option<&AskpassBridge>,
        cancellation: &CancellationToken,
    ) -> Result<PublicationResult, OnboardingError> {
        self.validate_branch(
            repository.identity().worktree_root.as_path(),
            remote_branch,
            cancellation,
        )?;
        let local_branch =
            self.current_branch(&repository.identity().worktree_root, cancellation)?;
        let commit = self
            .run_optional(
                GitCommand::new(&repository.identity().worktree_root)
                    .args(["rev-parse", "--verify", "HEAD"])
                    .read_only(true),
                cancellation,
            )?
            .ok_or(OnboardingError::NothingToPublish)
            .and_then(|text| {
                GitObjectId::parse(text.trim().to_string())
                    .map_err(|_| OnboardingError::NothingToPublish)
            })?;
        let destination_ref = format!("refs/heads/{remote_branch}");

        // Learn the remote's state before pushing so that connecting to a
        // remote which already has work does not dead-end on a rejected
        // non-fast-forward push: absent or strictly-behind remotes take the
        // exact publication push; a remote already at, ahead of, or diverged
        // from the local commit only binds upstream and lets the first sync
        // integrate; unrelated histories are refused outright.
        let remote_tracking_ref = format!("refs/remotes/{remote_name}/{remote_branch}");
        let fetch = GitCommand::new(&repository.identity().worktree_root)
            .args(["fetch", "--no-tags", remote_name]);
        self.run(self.network_request(fetch, askpass), cancellation)?;
        let remote_commit = self
            .run_optional(
                GitCommand::new(&repository.identity().worktree_root)
                    .args(["rev-parse", "--verify", remote_tracking_ref.as_str()])
                    .read_only(true),
                cancellation,
            )?
            .map(|text| GitObjectId::parse(text.trim().to_string()))
            .transpose()
            .map_err(|_| OnboardingError::NothingToPublish)?;
        let needs_push = match remote_commit {
            None => true,
            Some(remote_commit) if remote_commit == commit => false,
            Some(remote_commit) => {
                let merge_base = self
                    .run_optional(
                        GitCommand::new(&repository.identity().worktree_root)
                            .args([
                                "merge-base",
                                commit.as_str(),
                                remote_commit.as_str(),
                            ])
                            .read_only(true),
                        cancellation,
                    )?
                    .map(|text| GitObjectId::parse(text.trim().to_string()))
                    .transpose()
                    .map_err(|_| OnboardingError::NothingToPublish)?;
                match merge_base {
                    None => {
                        return Err(OnboardingError::UnrelatedRemoteHistory {
                            remote: remote_name.to_string(),
                            branch: remote_branch.to_string(),
                        })
                    }
                    // The remote is strictly behind: the exact push
                    // fast-forwards it. Anything else integrates later.
                    Some(base) => base == remote_commit,
                }
            }
        };
        if needs_push {
            let refspec = format!("{}:{destination_ref}", commit.as_str());
            let request = GitCommand::new(&repository.identity().worktree_root).args([
                "push",
                "--porcelain",
                remote_name,
                &refspec,
            ]);
            let request = self.network_request(request, askpass);
            self.run(request, cancellation)?;
        }

        // Bind upstream only after the exact publication succeeded (or was
        // not needed). A failed network operation is therefore safe to resume.
        self.run(
            GitCommand::new(&repository.identity().worktree_root).args([
                "config",
                "--local",
                &format!("branch.{local_branch}.remote"),
                remote_name,
            ]),
            cancellation,
        )?;
        self.run(
            GitCommand::new(&repository.identity().worktree_root).args([
                "config",
                "--local",
                &format!("branch.{local_branch}.merge"),
                &destination_ref,
            ]),
            cancellation,
        )?;
        let target = repository.resolve_sync_target()?;
        Ok(PublicationResult { commit, target })
    }

    fn current_branch(
        &self,
        directory: &Path,
        cancellation: &CancellationToken,
    ) -> Result<String, OnboardingError> {
        self.run_optional(
            GitCommand::new(directory)
                .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
                .read_only(true),
            cancellation,
        )?
        .map(|branch| branch.trim().to_string())
        .filter(|branch| !branch.is_empty())
        .ok_or(OnboardingError::InvalidBranch)
    }

    fn validate_branch(
        &self,
        directory: &Path,
        branch: &str,
        cancellation: &CancellationToken,
    ) -> Result<(), OnboardingError> {
        if branch.trim() != branch || branch.is_empty() || branch.chars().any(char::is_control) {
            return Err(OnboardingError::InvalidBranch);
        }
        self.run(
            GitCommand::new(directory).args(["check-ref-format", "--branch", branch]),
            cancellation,
        )?;
        Ok(())
    }

    fn network_request(&self, request: GitCommand, askpass: Option<&AskpassBridge>) -> GitCommand {
        let request = request.interactive_credentials(self.foreground_credentials);
        askpass.map_or(request.clone(), |bridge| {
            Authentication::new(self.runner.clone()).with_askpass(request, bridge)
        })
    }

    fn run(
        &self,
        request: GitCommand,
        cancellation: &CancellationToken,
    ) -> Result<String, GitCommandError> {
        self.runner
            .run(&request, CommandLimits::default(), cancellation)
            .map(|output| output.stdout_text())
    }

    fn run_optional(
        &self,
        request: GitCommand,
        cancellation: &CancellationToken,
    ) -> Result<Option<String>, GitCommandError> {
        match self
            .runner
            .run(&request, CommandLimits::default(), cancellation)
        {
            Ok(output) => Ok(Some(output.stdout_text())),
            Err(GitCommandError::Failed { output })
                if output.termination == crate::ProcessTermination::Exited
                    && matches!(output.status_code, Some(1 | 2 | 128)) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
}

fn cleanup_owned_clone(parent: &Path, temporary: &Path) {
    if temporary.parent() == Some(parent)
        && temporary
            .file_name()
            .is_some_and(|name| name.to_string_lossy().contains(".markion-clone-"))
    {
        let _ = fs::remove_dir_all(temporary);
    }
}

/// Decides whether an existing repository discovered at the workspace can be
/// adopted for sync without manual setup. All five safety criteria must hold:
/// the workspace is the worktree root (not a subdirectory of a larger
/// worktree), the repository has at least one commit, it supports write
/// synchronization, the origin resolves to exactly one fetch/push target for
/// the current branch, and the tracked content is notes-like — hidden
/// configuration files and standard repository furniture do not count as
/// unrelated content, visible non-notes files do.
pub fn repository_adoptable(
    workspace_root: &Path,
    identity: &RepositoryIdentity,
    state: &RepositoryState,
    tracked_paths: &[PathBuf],
    target: Option<&SyncTarget>,
) -> bool {
    identity.worktree_root == workspace_root
        && state.head.is_some()
        && state.capabilities.supports_write_sync()
        && target.is_some()
        && tracked_paths.iter().all(|path| tracked_path_is_notes_like(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RepositoryAuthor;
    use crate::RepositoryCapabilities;

    #[test]
    fn clones_empty_remote_to_explicit_unborn_branch() {
        let dir = tempfile::tempdir().unwrap();
        let remote = dir.path().join("empty.git");
        run_git(
            dir.path(),
            ["init", "--bare", "-q", remote.to_str().unwrap()],
        );
        let destination = dir.path().join("notes clone");
        let result = OnboardingService::new(GitCommandRunner::new("git"))
            .clone_repository(
                remote.to_str().unwrap(),
                &destination,
                Some("main"),
                None,
                &CancellationToken::new(),
            )
            .unwrap();
        assert!(result.empty_remote);
        assert_eq!(result.selected_branch, "main");
        assert!(destination.join(".git").is_dir());
    }

    #[test]
    fn clones_empty_remote_to_main_when_branch_is_automatic() {
        let dir = tempfile::tempdir().unwrap();
        let remote = dir.path().join("empty.git");
        run_git(
            dir.path(),
            ["init", "--bare", "-q", remote.to_str().unwrap()],
        );
        let destination = dir.path().join("automatic branch");
        let result = OnboardingService::new(GitCommandRunner::new("git"))
            .clone_repository(
                remote.to_str().unwrap(),
                &destination,
                None,
                None,
                &CancellationToken::new(),
            )
            .unwrap();
        assert!(result.empty_remote);
        assert_eq!(result.selected_branch, "main");
        assert_eq!(
            run_git_output(&destination, ["symbolic-ref", "--short", "HEAD"]).trim(),
            "main"
        );
    }

    #[test]
    fn initializes_and_publishes_with_upstream_only_after_success() {
        let dir = tempfile::tempdir().unwrap();
        let remote = dir.path().join("remote.git");
        run_git(
            dir.path(),
            ["init", "--bare", "-q", remote.to_str().unwrap()],
        );
        let notes = dir.path().join("notes");
        let runner = GitCommandRunner::new("git");
        let service = OnboardingService::new(runner.clone());
        let review = service
            .initialize_notes(&notes, "main", Some(remote.to_str().unwrap()))
            .unwrap();
        Authentication::new(runner.clone())
            .configure_author(
                &notes,
                &RepositoryAuthor {
                    name: "Markion Test".into(),
                    email: "test@markion.invalid".into(),
                },
            )
            .unwrap();
        fs::write(notes.join("notes.md"), "hello\n").unwrap();
        run_git(&notes, ["add", "--", "notes.md"]);
        run_git(&notes, ["commit", "-q", "-m", "initial"]);
        let publication = service
            .publish_first_branch(
                &review.repository,
                "origin",
                "main",
                None,
                &CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(publication.target.destination_ref, "refs/heads/main");
        let remote_head = run_git_output(&remote, ["rev-parse", "refs/heads/main"]);
        assert_eq!(remote_head.trim(), publication.commit.as_str());
    }

    #[test]
    fn initial_commit_stages_only_reviewed_note_paths() {
        let dir = tempfile::tempdir().unwrap();
        let notes = dir.path().join("notes");
        let runner = GitCommandRunner::new("git");
        let service = OnboardingService::new(runner.clone());
        let review = service.initialize_notes(&notes, "main", None).unwrap();
        Authentication::new(runner)
            .configure_author(
                &notes,
                &RepositoryAuthor {
                    name: "Markion Test".into(),
                    email: "test@markion.invalid".into(),
                },
            )
            .unwrap();
        fs::write(notes.join("note.md"), "hello\n").unwrap();
        fs::write(notes.join("private.key"), "do not commit\n").unwrap();

        let commit = service
            .create_initial_commit(
                &review.repository,
                &[PathBuf::from("note.md")],
                "Start notes",
                &CancellationToken::new(),
            )
            .unwrap();

        assert_eq!(
            run_git_output(
                &notes,
                ["show", "--format=", "--name-only", commit.as_str()]
            )
            .trim(),
            "note.md"
        );
        assert!(notes.join("private.key").is_file());
        assert!(
            run_git_output(&notes, ["status", "--porcelain"])
                .lines()
                .any(|line| line.ends_with("private.key"))
        );
    }

    #[test]
    fn resumable_origin_attach_never_replaces_an_existing_remote() {
        let dir = tempfile::tempdir().unwrap();
        let notes = dir.path().join("notes");
        let remote_a = dir.path().join("a.git");
        let remote_b = dir.path().join("b.git");
        run_git(
            dir.path(),
            ["init", "--bare", "-q", remote_a.to_str().unwrap()],
        );
        run_git(
            dir.path(),
            ["init", "--bare", "-q", remote_b.to_str().unwrap()],
        );
        let service = OnboardingService::new(GitCommandRunner::new("git"));
        let review = service.initialize_notes(&notes, "main", None).unwrap();
        service
            .attach_origin(
                &review.repository,
                remote_a.to_str().unwrap(),
                false,
                &CancellationToken::new(),
            )
            .unwrap();
        service
            .attach_origin(
                &review.repository,
                remote_a.to_str().unwrap(),
                false,
                &CancellationToken::new(),
            )
            .unwrap();
        assert!(matches!(
            service.attach_origin(
                &review.repository,
                remote_b.to_str().unwrap(),
                false,
                &CancellationToken::new(),
            ),
            Err(OnboardingError::RemoteExists(name)) if name == "origin"
        ));
        assert_eq!(
            run_git_output(&notes, ["remote", "get-url", "origin"]).trim(),
            remote_a.to_str().unwrap()
        );
        // The explicitly confirmed replacement switches the origin URL.
        service
            .attach_origin(
                &review.repository,
                remote_b.to_str().unwrap(),
                true,
                &CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(
            run_git_output(&notes, ["remote", "get-url", "origin"]).trim(),
            remote_b.to_str().unwrap()
        );
    }

    #[test]
    fn probe_remote_contacts_existing_remote_and_fails_for_missing_or_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let remote = dir.path().join("remote.git");
        run_git(
            dir.path(),
            ["init", "--bare", "-q", remote.to_str().unwrap()],
        );
        let service = OnboardingService::new(GitCommandRunner::new("git"));
        service
            .probe_remote(
                remote.to_str().unwrap(),
                dir.path(),
                &CancellationToken::new(),
            )
            .unwrap();
        // An empty remote is still reachable and must succeed.
        service
            .probe_remote(
                remote.to_str().unwrap(),
                dir.path(),
                &CancellationToken::new(),
            )
            .unwrap();
        assert!(
            service
                .probe_remote(
                    dir.path().join("missing.git").to_str().unwrap(),
                    dir.path(),
                    &CancellationToken::new(),
                )
                .is_err()
        );
        assert!(
            service
                .probe_remote("bad\u{1}address", dir.path(), &CancellationToken::new())
                .is_err()
        );
    }

    #[test]
    fn foreground_credentials_are_opt_in_for_explicit_onboarding() {        let request = GitCommand::new(".").args(["fetch", "origin"]);
        let background = OnboardingService::new(GitCommandRunner::new("git"))
            .network_request(request.clone(), None);
        let foreground = OnboardingService::new(GitCommandRunner::new("git"))
            .with_foreground_credentials()
            .network_request(request, None);
        assert!(!background.allow_interactive_credentials);
        assert!(foreground.allow_interactive_credentials);
    }

    #[test]
    fn cancelled_clone_removes_only_its_owned_temporary_directory() {
        let dir = tempfile::tempdir().unwrap();
        let remote = dir.path().join("empty.git");
        run_git(
            dir.path(),
            ["init", "--bare", "-q", remote.to_str().unwrap()],
        );
        let destination = dir.path().join("cancelled");
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(
            OnboardingService::new(GitCommandRunner::new("git"))
                .clone_repository(
                    remote.to_str().unwrap(),
                    &destination,
                    None,
                    None,
                    &cancellation,
                )
                .is_err()
        );
        assert!(!destination.exists());
        assert!(fs::read_dir(dir.path()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".markion-clone-")
        }));
    }

    #[test]
    fn repository_adoptable_accepts_a_fully_eligible_workspace() {
        let root = Path::new("/notes");
        let state = adoptable_state();
        let target = adoptable_target();
        assert!(repository_adoptable(
            root,
            &RepositoryIdentity::new(root.to_path_buf(), root.join(".git"), root.join(".git")),
            &state,
            &[PathBuf::from("a.md"), PathBuf::from("img/pic.png")],
            Some(&target),
        ));
    }

    #[test]
    fn repository_adoptable_tolerates_hidden_and_furniture_files() {
        let root = Path::new("/notes");
        let state = adoptable_state();
        let target = adoptable_target();
        assert!(repository_adoptable(
            root,
            &RepositoryIdentity::new(root.to_path_buf(), root.join(".git"), root.join(".git")),
            &state,
            &[
                PathBuf::from("a.md"),
                PathBuf::from(".gitignore"),
                PathBuf::from("LICENSE"),
                PathBuf::from(".obsidian/app.json"),
            ],
            Some(&target),
        ));
    }

    #[test]
    fn repository_adoptable_rejects_each_failing_criterion() {
        let root = Path::new("/notes");
        let identity =
            RepositoryIdentity::new(root.to_path_buf(), root.join(".git"), root.join(".git"));
        let state = adoptable_state();
        let target = adoptable_target();
        let tracked = [PathBuf::from("a.md")];

        // Workspace is a subdirectory of the discovered worktree.
        assert!(!repository_adoptable(
            root,
            &RepositoryIdentity::new(
                root.parent().unwrap().to_path_buf(),
                root.parent().unwrap().join(".git"),
                root.parent().unwrap().join(".git"),
            ),
            &state,
            &tracked,
            Some(&target),
        ));
        // Repository has no commit yet.
        assert!(!repository_adoptable(
            root,
            &identity,
            &RepositoryState {
                head: None,
                ..state.clone()
            },
            &tracked,
            Some(&target),
        ));
        // Repository cannot be written safely (e.g. bare).
        let mut unsupported = state.clone();
        unsupported.capabilities.bare = true;
        assert!(!repository_adoptable(
            root,
            &identity,
            &unsupported,
            &tracked,
            Some(&target),
        ));
        // Origin does not resolve to one fetch/push destination.
        assert!(!repository_adoptable(root, &identity, &state, &tracked, None));
        // Tracked content includes files outside the notes-like classes.
        assert!(!repository_adoptable(
            root,
            &identity,
            &state,
            &[PathBuf::from("a.md"), PathBuf::from("src/main.rs")],
            Some(&target),
        ));
    }

    fn adoptable_state() -> RepositoryState {
        RepositoryState {
            head: Some(GitObjectId::parse("0123456789abcdef0123456789abcdef01234567").unwrap()),
            branch: Some("main".into()),
            upstream: Some("origin/main".into()),
            worktree: Default::default(),
            capabilities: RepositoryCapabilities {
                ordinary_worktree: true,
                ..Default::default()
            },
            history: Default::default(),
            checked_at: std::time::SystemTime::UNIX_EPOCH,
        }
    }

    fn adoptable_target() -> SyncTarget {
        SyncTarget {
            local_branch: "main".into(),
            remote: "origin".into(),
            remote_branch: "main".into(),
            destination_ref: "refs/heads/main".into(),
            fetch_url: "https://example.invalid/notes.git".into(),
            push_url: "https://example.invalid/notes.git".into(),
        }
    }

    fn run_git<const N: usize>(directory: &Path, arguments: [&str; N]) {
        run_git_output(directory, arguments);
    }

    fn run_git_output<const N: usize>(directory: &Path, arguments: [&str; N]) -> String {
        let output = std::process::Command::new("git")
            .current_dir(directory)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }
}
