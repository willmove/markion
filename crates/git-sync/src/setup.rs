use std::{
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    AskpassBridge, Authentication, CancellationToken, CommandLimits, GitCommand, GitCommandError,
    GitCommandRunner, GitObjectId, GitRepository, RemoteUrl, RemoteUrlError, RepositoryError,
    RepositoryState, SyncTarget,
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
    #[error("requested branch is invalid or unavailable")]
    InvalidBranch,
    #[error("repository has no commit to publish")]
    NothingToPublish,
}

#[derive(Clone, Debug)]
pub struct OnboardingService {
    runner: GitCommandRunner,
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
        Self { runner }
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
        let request = if let Some(bridge) = askpass {
            Authentication::new(self.runner.clone()).with_askpass(request, bridge)
        } else {
            request
        };
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
        let refspec = format!("{}:{destination_ref}", commit.as_str());
        let request = GitCommand::new(&repository.identity().worktree_root).args([
            "push",
            "--porcelain",
            remote_name,
            &refspec,
        ]);
        let request = if let Some(bridge) = askpass {
            Authentication::new(self.runner.clone()).with_askpass(request, bridge)
        } else {
            request
        };
        self.run(request, cancellation)?;

        // Bind upstream only after the exact publication succeeded. A failed
        // network operation is therefore safe to resume.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RepositoryAuthor;

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
