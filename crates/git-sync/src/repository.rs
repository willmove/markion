use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs, io,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use sha2::{Digest, Sha256};

use crate::{
    CancellationToken, CommandLimits, GitCommand, GitCommandError, GitCommandRunner, GitObjectId,
    HistoryRelation, RepositoryCapabilities, RepositoryIdentity, SyncTarget, WorktreeState,
    parse_porcelain_v2,
};

pub fn content_fingerprint_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("sha256:{:x}", digest.finalize())
}

pub const DEFAULT_DIFF_FILE_LIMIT: usize = 2 * 1024 * 1024;
pub const DEFAULT_HISTORY_PAGE_SIZE: usize = 50;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Command(#[from] GitCommandError),
    #[error("invalid Git output: {0}")]
    InvalidOutput(String),
    #[error("repository path could not be resolved: {0}")]
    Path(#[from] io::Error),
    #[error(transparent)]
    Status(#[from] crate::StatusParseError),
    #[error(transparent)]
    Target(#[from] TargetResolutionError),
    #[error("path is outside the repository worktree: {0}")]
    OutsideWorktree(PathBuf),
    #[error("path contains an unsupported component or symlink escape: {0}")]
    UnsafePath(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum TargetResolutionError {
    #[error("HEAD is detached or unborn")]
    NoNamedBranch,
    #[error("branch has no configured upstream remote")]
    MissingRemote,
    #[error("branch has no configured upstream branch")]
    MissingRemoteBranch,
    #[error("remote has no URL")]
    MissingUrl,
    #[error("remote has multiple push destinations")]
    MultiplePushDestinations,
    #[error("remote or branch name is invalid")]
    InvalidName,
}

#[derive(Clone, Debug)]
pub struct GitRepository {
    runner: GitCommandRunner,
    identity: RepositoryIdentity,
    bare: bool,
    shallow: bool,
    /// Capability facts that do not come from `git status`, probed once per
    /// repository instance (and therefore once per write operation, which
    /// always discovers a fresh instance). Clones share the result.
    probe: Arc<Mutex<Option<CapabilityProbe>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CapabilityProbe {
    sparse: bool,
    partial: bool,
    has_submodules: bool,
    tracked_gitlinks: bool,
    uses_lfs: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryState {
    pub head: Option<GitObjectId>,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub worktree: WorktreeState,
    pub capabilities: RepositoryCapabilities,
    pub history: HistoryRelation,
    pub checked_at: SystemTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffRequest {
    pub staged: bool,
    pub path: Option<PathBuf>,
    pub max_bytes: usize,
}

impl Default for DiffRequest {
    fn default() -> Self {
        Self {
            staged: false,
            path: None,
            max_bytes: DEFAULT_DIFF_FILE_LIMIT,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffContent {
    pub bytes: Vec<u8>,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionComparison {
    pub content: DiffContent,
    pub binary: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub oid: GitObjectId,
    pub parents: Vec<GitObjectId>,
    pub author_name: String,
    pub author_email: String,
    pub authored_unix_seconds: i64,
    pub authored_iso: String,
    pub subject: String,
}

impl GitRepository {
    pub fn discover(
        runner: GitCommandRunner,
        start: impl AsRef<Path>,
    ) -> Result<Self, RepositoryError> {
        let start = start.as_ref();
        // One process answers every discovery question.
        let output = run_text(
            &runner,
            start,
            GitCommand::new(start).args([
                "rev-parse",
                "--show-toplevel",
                "--absolute-git-dir",
                "--git-common-dir",
                "--is-bare-repository",
                "--is-shallow-repository",
            ]),
        )?;
        let mut lines = output.lines().map(str::trim);
        let mut next = |name: &str| {
            lines
                .next()
                .filter(|line| !line.is_empty())
                .ok_or_else(|| RepositoryError::InvalidOutput(format!("missing {name}")))
        };
        let worktree_root = canonical_discovered_path(start, next("worktree root")?)?;
        let git_dir = canonical_discovered_path(start, next("Git directory")?)?;
        let common_dir = canonical_discovered_path(start, next("common Git directory")?)?;
        let bare = next("bare flag")? == "true";
        let shallow = next("shallow flag")? == "true";
        Ok(Self {
            runner,
            identity: RepositoryIdentity::new(worktree_root, git_dir, common_dir),
            bare,
            shallow,
            probe: Arc::default(),
        })
    }

    pub fn identity(&self) -> &RepositoryIdentity {
        &self.identity
    }

    pub fn runner(&self) -> &GitCommandRunner {
        &self.runner
    }

    /// Repository capabilities as of a fresh status read. Detached HEAD and
    /// untracked nested repositories come from that status; everything else
    /// comes from the memoized probe.
    pub fn capabilities(&self) -> Result<RepositoryCapabilities, RepositoryError> {
        Ok(self.status()?.capabilities)
    }

    fn capabilities_from_status(
        &self,
        detached_head: bool,
        untracked_nested: bool,
    ) -> Result<RepositoryCapabilities, RepositoryError> {
        let probe = self.capability_probe()?;
        Ok(RepositoryCapabilities {
            ordinary_worktree: !self.bare,
            detached_head,
            linked_worktree: self.identity.git_dir != self.identity.common_dir,
            bare: self.bare,
            shallow: self.shallow,
            sparse: probe.sparse,
            partial: probe.partial,
            has_submodules: probe.has_submodules,
            has_nested_repositories: untracked_nested || probe.tracked_gitlinks,
            uses_lfs: probe.uses_lfs,
        })
    }

    fn capability_probe(&self) -> Result<CapabilityProbe, RepositoryError> {
        let mut cached = self.probe.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(probe) = *cached {
            return Ok(probe);
        }
        let root = &self.identity.worktree_root;
        let config = self.run_allow_failure(
            GitCommand::new(root)
                .args([
                    "config",
                    "-z",
                    "--get-regexp",
                    r"^(core\.sparsecheckout|extensions\.partialclone|remote\..*\.promisor)$",
                ])
                .read_only(true),
        )?;
        let (mut sparse, mut partial) = (false, false);
        for entry in config.as_deref().unwrap_or_default().split('\0') {
            let (key, value) = entry.split_once('\n').unwrap_or((entry, ""));
            match key {
                "" => {}
                "core.sparsecheckout" => sparse |= value.is_empty() || git_bool(value),
                _ => partial = true,
            }
        }
        sparse |= self.identity.git_dir.join("info/sparse-checkout").is_file();
        let probe = CapabilityProbe {
            sparse,
            partial,
            has_submodules: root.join(".gitmodules").is_file(),
            tracked_gitlinks: self.index_has_gitlinks()?,
            uses_lfs: self.tracked_attributes_use_lfs()?,
        };
        *cached = Some(probe);
        Ok(probe)
    }

    /// Tracked gitlinks (mode 160000) mean submodule-like content even without
    /// `.gitmodules`. An index listing too large to inspect fails closed.
    fn index_has_gitlinks(&self) -> Result<bool, RepositoryError> {
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args(["ls-files", "-z", "--stage"])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 16 * 1024 * 1024,
                ..CommandLimits::default()
            },
        )?;
        Ok(output.stdout_truncated
            || output
                .stdout
                .split(|byte| *byte == 0)
                .any(|record| record.starts_with(b"160000 ")))
    }

    /// Validate a repository-relative write target without following an
    /// existing symlink or junction outside the worktree. The leaf may be new.
    pub fn validate_write_path(&self, relative: &Path) -> Result<PathBuf, RepositoryError> {
        use std::path::Component;

        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(RepositoryError::UnsafePath(relative.to_path_buf()));
        }
        let root = fs::canonicalize(&self.identity.worktree_root)?;
        let candidate = root.join(relative);
        let mut existing = candidate.as_path();
        while !existing.exists() {
            existing = existing
                .parent()
                .ok_or_else(|| RepositoryError::UnsafePath(candidate.clone()))?;
        }
        let resolved_parent = fs::canonicalize(existing)?;
        if !resolved_parent.starts_with(&root) {
            return Err(RepositoryError::OutsideWorktree(candidate));
        }
        Ok(candidate)
    }

    pub fn relative_path(&self, path: &Path) -> Result<PathBuf, RepositoryError> {
        let root = fs::canonicalize(&self.identity.worktree_root)?;
        let resolved = fs::canonicalize(path)?;
        resolved
            .strip_prefix(&root)
            .map(Path::to_path_buf)
            .map_err(|_| RepositoryError::OutsideWorktree(path.to_path_buf()))
    }

    pub fn path_is_tracked(&self, path: &Path) -> bool {
        self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args(["ls-files", "--error-unmatch"])
                .literal_paths([path])
                .read_only(true),
            CommandLimits::default(),
        )
        .is_ok()
    }

    /// Return the complete bounded inventory of paths tracked by the index.
    ///
    /// This intentionally does not infer repository scope from the editor's
    /// filtered file tree: setup needs to notice unrelated tracked content
    /// before it can offer the ordinary dedicated-notes authorization.
    pub fn tracked_paths(&self) -> Result<Vec<PathBuf>, RepositoryError> {
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args(["ls-files", "-z"])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 16 * 1024 * 1024,
                ..CommandLimits::default()
            },
        )?;
        if output.stdout_truncated {
            return Err(RepositoryError::InvalidOutput(
                "tracked path inventory exceeds limit".into(),
            ));
        }
        nul_delimited_paths(&output.stdout)
    }

    /// Which of `paths` Git ignores, in one `check-ignore --stdin` process.
    /// Like the single-path query, tracked paths are never reported.
    pub fn ignored_among(&self, paths: &[PathBuf]) -> Result<HashSet<PathBuf>, RepositoryError> {
        if paths.is_empty() {
            return Ok(HashSet::new());
        }
        let mut input = Vec::new();
        for path in paths {
            input.extend_from_slice(git_path_text(path).as_bytes());
            input.push(0);
        }
        let output = self.run_allow_failure_bytes(
            GitCommand::new(&self.identity.worktree_root)
                .args(["check-ignore", "-z", "--stdin"])
                .standard_input(input)
                .read_only(true),
        )?;
        let ignored: HashSet<String> = output
            .unwrap_or_default()
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty())
            .map(|record| String::from_utf8_lossy(record).into_owned())
            .collect();
        Ok(paths
            .iter()
            .filter(|path| ignored.contains(&git_path_text(path)))
            .cloned()
            .collect())
    }

    /// Which of `paths` are tracked, from one full index listing. Comparing in
    /// memory avoids pathspec glob interpretation entirely.
    pub fn tracked_among(&self, paths: &[PathBuf]) -> Result<HashSet<PathBuf>, RepositoryError> {
        if paths.is_empty() {
            return Ok(HashSet::new());
        }
        let tracked: HashSet<String> = self
            .tracked_paths()?
            .iter()
            .map(|path| git_path_text(path))
            .collect();
        Ok(paths
            .iter()
            .filter(|path| tracked.contains(&git_path_text(path)))
            .cloned()
            .collect())
    }

    pub fn path_is_ignored(&self, path: &Path) -> bool {
        self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args(["check-ignore", "-q"])
                .literal_paths([path])
                .read_only(true),
            CommandLimits::default(),
        )
        .is_ok()
    }

    /// Hash one repository-relative worktree file for a reviewed commit plan.
    /// Missing paths use a stable marker so deletions can be selected. Symlink
    /// content is never followed into a commit draft.
    pub fn content_fingerprint(&self, path: &Path) -> Result<String, RepositoryError> {
        let absolute = self.validate_write_path(path)?;
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok("missing".into()),
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(RepositoryError::UnsafePath(path.to_path_buf()));
        }
        let mut file = fs::File::open(absolute)?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
        }
        Ok(format!("sha256:{:x}", digest.finalize()))
    }

    pub fn status(&self) -> Result<RepositoryState, RepositoryError> {
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args([
                    "-c",
                    "core.quotepath=false",
                    "status",
                    "--porcelain=v2",
                    "-z",
                    "--branch",
                    "--untracked-files=all",
                ])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 16 * 1024 * 1024,
                ..CommandLimits::default()
            },
        )?;
        let mut parsed = parse_porcelain_v2(&output.stdout)?;
        parsed.worktree.operation_in_progress = self.operation_in_progress();
        let capabilities = self.capabilities_from_status(
            parsed.branch.head.is_none(),
            !parsed.untracked_directories.is_empty(),
        )?;
        let history = match (parsed.branch.ahead, parsed.branch.behind) {
            (0, 0) if parsed.branch.upstream.is_some() => HistoryRelation::Equal,
            (ahead, 0) if ahead > 0 => HistoryRelation::Ahead { commits: ahead },
            (0, behind) if behind > 0 => HistoryRelation::Behind { commits: behind },
            (ahead, behind) if ahead > 0 && behind > 0 => {
                HistoryRelation::Diverged { ahead, behind }
            }
            _ => HistoryRelation::Unknown,
        };
        Ok(RepositoryState {
            head: parsed.branch.oid,
            branch: parsed.branch.head,
            upstream: parsed.branch.upstream,
            worktree: parsed.worktree,
            capabilities,
            history,
            checked_at: SystemTime::now(),
        })
    }

    pub fn resolve_sync_target(&self) -> Result<SyncTarget, RepositoryError> {
        let root = &self.identity.worktree_root;
        let local_branch = self.required_text(
            GitCommand::new(root).args(["symbolic-ref", "--quiet", "--short", "HEAD"]),
            TargetResolutionError::NoNamedBranch,
        )?;
        if !self.ref_is_valid(&format!("refs/heads/{local_branch}"))? {
            return Err(TargetResolutionError::InvalidName.into());
        }
        let (remote, merge_ref) = self.branch_upstream_config(&local_branch)?;
        let remote = remote.ok_or(TargetResolutionError::MissingRemote)?;
        if remote == "."
            || remote.is_empty()
            || remote.starts_with('-')
            || remote.chars().any(char::is_control)
        {
            return Err(TargetResolutionError::InvalidName.into());
        }
        let merge_ref = merge_ref.ok_or(TargetResolutionError::MissingRemoteBranch)?;
        let remote_branch = merge_ref
            .strip_prefix("refs/heads/")
            .filter(|branch| !branch.is_empty())
            .ok_or(TargetResolutionError::InvalidName)?
            .to_owned();
        if !self.ref_is_valid(&merge_ref)? {
            return Err(TargetResolutionError::InvalidName.into());
        }
        let fetch_urls = self.remote_urls(&remote, false)?;
        let push_urls = self.remote_urls(&remote, true)?;
        if push_urls.len() > 1 {
            return Err(TargetResolutionError::MultiplePushDestinations.into());
        }
        let fetch_url = fetch_urls
            .first()
            .cloned()
            .ok_or(TargetResolutionError::MissingUrl)?;
        let push_url = push_urls
            .first()
            .cloned()
            .unwrap_or_else(|| fetch_url.clone());
        Ok(SyncTarget {
            local_branch,
            remote,
            remote_branch,
            destination_ref: merge_ref,
            fetch_url,
            push_url,
        })
    }

    pub fn classify_history(
        &self,
        local: &GitObjectId,
        remote: &GitObjectId,
        previously_observed_remote: Option<&GitObjectId>,
    ) -> Result<HistoryRelation, RepositoryError> {
        if let Some(previous) = previously_observed_remote
            && previous != remote
            && !self.is_ancestor(previous, remote)?
        {
            return Ok(HistoryRelation::Rewritten);
        }
        if local == remote {
            return Ok(HistoryRelation::Equal);
        }
        let local_ancestor = self.is_ancestor(local, remote)?;
        let remote_ancestor = self.is_ancestor(remote, local)?;
        if local_ancestor {
            return Ok(HistoryRelation::Behind {
                commits: self.commit_count(&format!("{local}..{remote}"))?,
            });
        }
        if remote_ancestor {
            return Ok(HistoryRelation::Ahead {
                commits: self.commit_count(&format!("{remote}..{local}"))?,
            });
        }
        let merge_base = self.run_allow_failure(
            GitCommand::new(&self.identity.worktree_root)
                .args(["merge-base", local.as_str(), remote.as_str()])
                .read_only(true),
        )?;
        if merge_base.is_none() {
            return Ok(HistoryRelation::Unrelated);
        }
        let counts = self.run_text(
            GitCommand::new(&self.identity.worktree_root)
                .args([
                    "rev-list",
                    "--left-right",
                    "--count",
                    &format!("{local}...{remote}"),
                ])
                .read_only(true),
        )?;
        let mut fields = counts.split_whitespace();
        let ahead = parse_usize(fields.next(), "ahead count")?;
        let behind = parse_usize(fields.next(), "behind count")?;
        Ok(HistoryRelation::Diverged { ahead, behind })
    }

    pub fn classify_missing_target(
        previously_observed_remote: Option<&GitObjectId>,
    ) -> HistoryRelation {
        if previously_observed_remote.is_some() {
            HistoryRelation::DeletedTarget
        } else {
            HistoryRelation::Unknown
        }
    }

    pub fn diff(&self, request: &DiffRequest) -> Result<DiffContent, RepositoryError> {
        if !request.staged
            && let Some(path) = &request.path
            && !self.path_is_tracked(path)
        {
            use std::io::Read;
            let absolute = self.validate_write_path(path)?;
            let file = fs::File::open(absolute)?;
            let mut bytes = Vec::new();
            file.take(request.max_bytes.saturating_add(1) as u64)
                .read_to_end(&mut bytes)?;
            let truncated = bytes.len() > request.max_bytes;
            bytes.truncate(request.max_bytes);
            return Ok(DiffContent { bytes, truncated });
        }
        let mut command = GitCommand::new(&self.identity.worktree_root).args([
            "-c",
            "diff.external=",
            "diff",
            "--no-ext-diff",
            "--no-textconv",
        ]);
        if request.staged {
            command = command.arg("--cached");
        }
        if let Some(path) = &request.path {
            command = command.literal_paths([path.as_os_str()]);
        }
        let output = self.run(
            command.read_only(true),
            CommandLimits {
                max_stdout_bytes: request.max_bytes,
                ..CommandLimits::default()
            },
        )?;
        Ok(DiffContent {
            bytes: output.stdout,
            truncated: output.stdout_truncated,
        })
    }

    pub fn observed_remote_tip(&self, target: &SyncTarget) -> Option<GitObjectId> {
        self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args(["rev-parse", "--verify", &Self::observation_ref(target)])
                .read_only(true),
            CommandLimits::default(),
        )
        .ok()
        .and_then(|output| GitObjectId::parse(output.stdout_text().trim().to_string()).ok())
    }

    pub(crate) fn observation_ref(target: &SyncTarget) -> String {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        for part in [
            &target.remote,
            &target.destination_ref,
            &target.fetch_url,
            &target.push_url,
        ] {
            digest.update(part.as_bytes());
            digest.update([0]);
        }
        format!("refs/markion/observed/{:x}", digest.finalize())
    }

    pub fn history(&self, skip: usize, limit: usize) -> Result<Vec<HistoryEntry>, RepositoryError> {
        self.history_page(skip, limit, false)
    }

    pub fn history_page(
        &self,
        skip: usize,
        limit: usize,
        outgoing: bool,
    ) -> Result<Vec<HistoryEntry>, RepositoryError> {
        let limit = limit.clamp(1, DEFAULT_HISTORY_PAGE_SIZE);
        let mut arguments = vec![
            OsString::from("log"),
            OsString::from("-z"),
            OsString::from(format!("--skip={skip}")),
            OsString::from(format!("-n{limit}")),
            OsString::from("--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%aI%x1f%s"),
        ];
        if outgoing {
            let observed = self
                .resolve_sync_target()
                .ok()
                .and_then(|target| self.observed_remote_tip(&target));
            arguments.push(OsString::from(
                observed
                    .map(|oid| format!("{oid}..HEAD"))
                    .unwrap_or_else(|| "@{upstream}..HEAD".into()),
            ));
        }
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args(arguments)
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 2 * 1024 * 1024,
                ..CommandLimits::default()
            },
        )?;
        output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty())
            .map(parse_history_entry)
            .collect()
    }

    /// Return one bounded page of commits affecting a single literal path.
    /// `--follow` is intentionally limited to one path so Git can trace
    /// ordinary renames without broadening the history query.
    pub fn history_page_for_path(
        &self,
        path: &Path,
        skip: usize,
        limit: usize,
    ) -> Result<Vec<HistoryEntry>, RepositoryError> {
        let limit = limit.clamp(1, DEFAULT_HISTORY_PAGE_SIZE);
        let requested = skip.saturating_add(limit);
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args([
                    "log",
                    "-z",
                    "--follow",
                    &format!("-n{requested}"),
                    "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%aI%x1f%s",
                ])
                .literal_paths([path.as_os_str()])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 2 * 1024 * 1024,
                ..CommandLimits::default()
            },
        )?;
        if output.stdout_truncated {
            return Err(RepositoryError::InvalidOutput(
                "file history page exceeds limit".into(),
            ));
        }
        output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty())
            .map(parse_history_entry)
            .skip(skip)
            .collect()
    }

    pub fn commit_metadata(&self, commit: &GitObjectId) -> Result<HistoryEntry, RepositoryError> {
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args([
                    "show",
                    "-s",
                    "-z",
                    "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%aI%x1f%s",
                    commit.as_str(),
                ])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 64 * 1024,
                ..CommandLimits::default()
            },
        )?;
        let record = output
            .stdout
            .split(|byte| *byte == 0)
            .find(|record| !record.is_empty())
            .ok_or_else(|| RepositoryError::InvalidOutput("missing commit metadata".into()))?;
        parse_history_entry(record)
    }

    /// Compare one historical path with its current working-tree state. The
    /// binary probe and the rendered patch both disable user diff drivers and
    /// text conversion so inspection cannot execute repository-provided code.
    pub fn working_diff_against(
        &self,
        commit: &GitObjectId,
        path: &Path,
        max_bytes: usize,
    ) -> Result<VersionComparison, RepositoryError> {
        let binary_probe = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args([
                    "-c",
                    "diff.external=",
                    "diff",
                    "--numstat",
                    "-z",
                    "--no-ext-diff",
                    "--no-textconv",
                    commit.as_str(),
                ])
                .literal_paths([path.as_os_str()])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 64 * 1024,
                ..CommandLimits::default()
            },
        )?;
        let binary = binary_probe
            .stdout
            .split(|byte| *byte == 0)
            .filter(|field| !field.is_empty())
            .any(|field| field.starts_with(b"-\t-\t"));
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args([
                    "-c",
                    "diff.external=",
                    "diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    commit.as_str(),
                ])
                .literal_paths([path.as_os_str()])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: max_bytes,
                ..CommandLimits::default()
            },
        )?;
        Ok(VersionComparison {
            content: DiffContent {
                bytes: output.stdout,
                truncated: output.stdout_truncated,
            },
            binary,
        })
    }

    pub fn historical_blob(
        &self,
        commit: &GitObjectId,
        path: &Path,
        max_bytes: usize,
    ) -> Result<DiffContent, RepositoryError> {
        let tree = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args(["ls-tree", "-r", "-z", commit.as_str()])
                .literal_paths([path.as_os_str()])
                .read_only(true),
            CommandLimits::default(),
        )?;
        let record = tree
            .stdout
            .split(|byte| *byte == 0)
            .find(|record| !record.is_empty())
            .ok_or_else(|| RepositoryError::InvalidOutput("historical path missing".into()))?;
        let metadata = record
            .split(|byte| *byte == b'\t')
            .next()
            .ok_or_else(|| RepositoryError::InvalidOutput("invalid ls-tree output".into()))?;
        let metadata = std::str::from_utf8(metadata)
            .map_err(|_| RepositoryError::InvalidOutput("invalid ls-tree metadata".into()))?;
        let mut fields = metadata.split_whitespace();
        let mode = fields
            .next()
            .ok_or_else(|| RepositoryError::InvalidOutput("invalid ls-tree mode".into()))?;
        if !matches!(mode, "100644" | "100755") {
            return Err(RepositoryError::InvalidOutput(
                "historical path is not a regular file".into(),
            ));
        }
        let _kind = fields
            .next()
            .ok_or_else(|| RepositoryError::InvalidOutput("invalid ls-tree kind".into()))?;
        let blob_oid = fields
            .next()
            .ok_or_else(|| RepositoryError::InvalidOutput("invalid ls-tree object".into()))?;
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args(["cat-file", "blob", blob_oid])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: max_bytes,
                ..CommandLimits::default()
            },
        )?;
        Ok(DiffContent {
            bytes: output.stdout,
            truncated: output.stdout_truncated,
        })
    }

    /// A bounded, NUL-delimited inventory including paths omitted by the editor tree.
    pub fn commit_paths(&self, commit: &GitObjectId) -> Result<Vec<PathBuf>, RepositoryError> {
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args([
                    "diff-tree",
                    "--root",
                    "-m",
                    "--no-commit-id",
                    "--name-only",
                    "-r",
                    "-z",
                    commit.as_str(),
                ])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 16 * 1024 * 1024,
                ..CommandLimits::default()
            },
        )?;
        if output.stdout_truncated {
            return Err(RepositoryError::InvalidOutput(
                "commit path inventory exceeds limit".into(),
            ));
        }
        nul_delimited_paths(&output.stdout)
    }

    pub fn commit_diff(
        &self,
        commit: &GitObjectId,
        path: &Path,
        max_bytes: usize,
    ) -> Result<DiffContent, RepositoryError> {
        let output = self.run(
            GitCommand::new(&self.identity.worktree_root)
                .args([
                    "show",
                    "--format=",
                    "--first-parent",
                    "--no-ext-diff",
                    "--no-textconv",
                    commit.as_str(),
                ])
                .literal_paths([path.as_os_str()])
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: max_bytes,
                ..CommandLimits::default()
            },
        )?;
        Ok(DiffContent {
            bytes: output.stdout,
            truncated: output.stdout_truncated,
        })
    }

    pub fn renames_between(
        &self,
        from: &GitObjectId,
        to: &GitObjectId,
        cancellation: &CancellationToken,
    ) -> Result<Vec<(PathBuf, PathBuf)>, RepositoryError> {
        let output = self.runner.run(
            &GitCommand::new(&self.identity.worktree_root)
                .args([
                    "diff",
                    "--name-status",
                    "-z",
                    "--find-renames",
                    "--no-ext-diff",
                    from.as_str(),
                    to.as_str(),
                ])
                .env("GIT_EXTERNAL_DIFF", "")
                .read_only(true),
            CommandLimits {
                max_stdout_bytes: 16 * 1024 * 1024,
                ..CommandLimits::default()
            },
            cancellation,
        )?;
        let fields = output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|field| !field.is_empty())
            .collect::<Vec<_>>();
        let mut renames = Vec::new();
        let mut index = 0;
        while index < fields.len() {
            let status = String::from_utf8_lossy(fields[index]);
            index += 1;
            if status.starts_with('R') || status.starts_with('C') {
                if index + 1 >= fields.len() {
                    return Err(RepositoryError::InvalidOutput(
                        "truncated rename record".into(),
                    ));
                }
                let from = PathBuf::from(String::from_utf8_lossy(fields[index]).into_owned());
                let to = PathBuf::from(String::from_utf8_lossy(fields[index + 1]).into_owned());
                self.validate_write_path(&from)?;
                self.validate_write_path(&to)?;
                if status.starts_with('R') {
                    renames.push((from, to));
                }
                index += 2;
            } else {
                if index >= fields.len() {
                    return Err(RepositoryError::InvalidOutput(
                        "truncated path status record".into(),
                    ));
                }
                index += 1;
            }
        }
        Ok(renames)
    }

    fn tracked_attributes_use_lfs(&self) -> Result<bool, RepositoryError> {
        let output = self.run_allow_failure(
            GitCommand::new(&self.identity.worktree_root)
                .args(["grep", "-I", "-l", "filter=lfs", "--", "*.gitattributes"])
                .read_only(true),
        )?;
        Ok(output.is_some_and(|value| !value.trim().is_empty()))
    }

    fn operation_in_progress(&self) -> Option<String> {
        let git = &self.identity.git_dir;
        let candidates = [
            ("MERGE_HEAD", "merge"),
            ("CHERRY_PICK_HEAD", "cherry-pick"),
            ("REVERT_HEAD", "revert"),
            ("rebase-merge", "rebase"),
            ("rebase-apply", "rebase"),
            ("index.lock", "locked"),
        ];
        candidates
            .iter()
            .find(|(path, _)| git.join(path).exists())
            .map(|(_, operation)| (*operation).to_owned())
    }

    /// `branch.<name>.remote` and `branch.<name>.merge`, each the last value
    /// like `git config --get`. Branch names in the conservative safe set are
    /// read with one regexp query; anything else uses per-key queries.
    fn branch_upstream_config(
        &self,
        branch: &str,
    ) -> Result<(Option<String>, Option<String>), RepositoryError> {
        let root = &self.identity.worktree_root;
        if !is_conservative_ref_name(branch) {
            let get = |key: &str| -> Result<Option<String>, RepositoryError> {
                Ok(self
                    .run_allow_failure(
                        GitCommand::new(root)
                            .args(["config", "--get", &format!("branch.{branch}.{key}")])
                            .read_only(true),
                    )?
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty()))
            };
            return Ok((get("remote")?, get("merge")?));
        }
        let pattern = format!(r"^branch\.{}\.(remote|merge)$", branch.replace('.', r"\."));
        let output = self.run_allow_failure(
            GitCommand::new(root)
                .args(["config", "-z", "--get-regexp", &pattern])
                .read_only(true),
        )?;
        let (mut remote, mut merge) = (None, None);
        let prefix = format!("branch.{branch}.");
        for entry in output.as_deref().unwrap_or_default().split('\0') {
            let Some((key, value)) = entry.split_once('\n') else {
                continue;
            };
            let value = value.trim();
            let value = (!value.is_empty()).then(|| value.to_owned());
            match key.strip_prefix(&prefix) {
                Some("remote") => remote = value,
                Some("merge") => merge = value,
                _ => {}
            }
        }
        Ok((remote, merge))
    }

    fn ref_is_valid(&self, reference: &str) -> Result<bool, RepositoryError> {
        // check-ref-format rejects one-level names, so the fast path does too.
        if reference.contains('/') && is_conservative_ref_name(reference) {
            return Ok(true);
        }
        Ok(self
            .run_allow_failure(
                GitCommand::new(&self.identity.worktree_root)
                    .args(["check-ref-format", reference])
                    .read_only(true),
            )?
            .is_some())
    }

    fn remote_urls(&self, remote: &str, push: bool) -> Result<Vec<String>, RepositoryError> {
        let mut command =
            GitCommand::new(&self.identity.worktree_root).args(["remote", "get-url", "--all"]);
        if push {
            command = GitCommand::new(&self.identity.worktree_root)
                .args(["remote", "get-url", "--push", "--all"]);
        }
        command = command.arg(remote);
        Ok(self
            .run_text(command.read_only(true))?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect())
    }

    fn is_ancestor(
        &self,
        ancestor: &GitObjectId,
        descendant: &GitObjectId,
    ) -> Result<bool, RepositoryError> {
        Ok(self
            .run_allow_failure(
                GitCommand::new(&self.identity.worktree_root)
                    .args([
                        "merge-base",
                        "--is-ancestor",
                        ancestor.as_str(),
                        descendant.as_str(),
                    ])
                    .read_only(true),
            )?
            .is_some())
    }

    fn commit_count(&self, revision: &str) -> Result<usize, RepositoryError> {
        let value = self.run_text(
            GitCommand::new(&self.identity.worktree_root)
                .args(["rev-list", "--count", revision])
                .read_only(true),
        )?;
        parse_usize(Some(value.trim()), "commit count")
    }

    fn required_text(
        &self,
        command: GitCommand,
        missing: TargetResolutionError,
    ) -> Result<String, RepositoryError> {
        self.run_allow_failure(command.read_only(true))?
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| missing.into())
    }

    fn run_text(&self, command: GitCommand) -> Result<String, RepositoryError> {
        let output = self.run(command, CommandLimits::default())?;
        String::from_utf8(output.stdout)
            .map_err(|_| RepositoryError::InvalidOutput("expected UTF-8 Git output".into()))
    }

    fn run_allow_failure(&self, command: GitCommand) -> Result<Option<String>, RepositoryError> {
        match self.runner.run(
            &command,
            CommandLimits::default(),
            &CancellationToken::new(),
        ) {
            Ok(output) => Ok(Some(output.stdout_text())),
            Err(GitCommandError::Failed { output }) if output.status_code == Some(1) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn run_allow_failure_bytes(
        &self,
        command: GitCommand,
    ) -> Result<Option<Vec<u8>>, RepositoryError> {
        match self.runner.run(
            &command,
            CommandLimits {
                max_stdout_bytes: 16 * 1024 * 1024,
                ..CommandLimits::default()
            },
            &CancellationToken::new(),
        ) {
            Ok(output) => Ok(Some(output.stdout)),
            Err(GitCommandError::Failed { output }) if output.status_code == Some(1) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn run(
        &self,
        command: GitCommand,
        limits: CommandLimits,
    ) -> Result<crate::GitCommandOutput, RepositoryError> {
        self.runner
            .run(&command, limits, &CancellationToken::new())
            .map_err(Into::into)
    }
}

fn nul_delimited_paths(bytes: &[u8]) -> Result<Vec<PathBuf>, RepositoryError> {
    let mut paths = Vec::new();
    for bytes in bytes
        .split(|byte| *byte == 0)
        .filter(|bytes| !bytes.is_empty())
    {
        #[cfg(unix)]
        let path = {
            use std::os::unix::ffi::OsStringExt;
            PathBuf::from(OsString::from_vec(bytes.to_vec()))
        };
        #[cfg(not(unix))]
        let path = PathBuf::from(
            std::str::from_utf8(bytes)
                .map_err(|_| RepositoryError::InvalidOutput("unrepresentable path".into()))?,
        );
        paths.push(path);
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

impl crate::ScopeFacts for GitRepository {
    fn is_symlink(&self, path: &Path) -> bool {
        fs::symlink_metadata(self.identity.worktree_root.join(path))
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
    }

    fn is_nested_repository(&self, path: &Path) -> bool {
        self.identity.worktree_root.join(path).join(".git").exists()
    }

    fn fingerprint(&self, path: &Path) -> Option<String> {
        self.content_fingerprint(path).ok()
    }

    fn worktree_blob_ids(&self, paths: &[PathBuf]) -> Option<HashMap<PathBuf, String>> {
        let texts: Vec<String> = paths.iter().map(|path| git_path_text(path)).collect();
        if texts.iter().any(|text| text.contains('\n')) {
            return None;
        }
        let mut input = texts.join("\n").into_bytes();
        input.push(b'\n');
        let output = self
            .run(
                GitCommand::new(&self.identity.worktree_root)
                    .args(["hash-object", "--stdin-paths"])
                    .standard_input(input)
                    .read_only(true),
                CommandLimits::default(),
            )
            .ok()?;
        let ids: Vec<String> = output
            .stdout_text()
            .lines()
            .map(|line| line.trim().to_owned())
            .collect();
        (ids.len() == paths.len()).then(|| paths.iter().cloned().zip(ids).collect())
    }

    fn index_blob_ids(&self, paths: &[PathBuf]) -> Option<HashMap<PathBuf, String>> {
        let output = self
            .run(
                GitCommand::new(&self.identity.worktree_root)
                    .args(["ls-files", "-z", "--stage"])
                    .read_only(true),
                CommandLimits {
                    max_stdout_bytes: 16 * 1024 * 1024,
                    ..CommandLimits::default()
                },
            )
            .ok()
            .filter(|output| !output.stdout_truncated)?;
        let wanted: HashMap<String, &PathBuf> = paths
            .iter()
            .map(|path| (git_path_text(path), path))
            .collect();
        let mut ids = HashMap::new();
        for record in output.stdout.split(|byte| *byte == 0) {
            let Some(tab) = record.iter().position(|byte| *byte == b'\t') else {
                continue;
            };
            let path = String::from_utf8_lossy(&record[tab + 1..]);
            if let Some(original) = wanted.get(path.as_ref()) {
                let metadata = String::from_utf8_lossy(&record[..tab]);
                if let Some(oid) = metadata.split(' ').nth(1) {
                    ids.insert((*original).clone(), oid.to_owned());
                }
            }
        }
        Some(ids)
    }
}

fn canonical_discovered_path(start: &Path, raw: &str) -> Result<PathBuf, RepositoryError> {
    let raw = PathBuf::from(raw);
    dunce::canonicalize(if raw.is_absolute() {
        raw
    } else {
        start.join(raw)
    })
    .map_err(Into::into)
}

/// A ref or branch name that `git check-ref-format` always accepts: ASCII
/// letters, digits, `.`, `_`, `-` and `/`-separated components that do not
/// start with `.` or `-` and do not end with `.` or `.lock`.
fn is_conservative_ref_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
        && !name.contains("..")
        && name.split('/').all(|component| {
            !component.is_empty()
                && !component.starts_with(['.', '-'])
                && !component.ends_with('.')
                && !component.ends_with(".lock")
        })
}

/// Repository-relative path in Git's spelling (`/` separators).
fn git_path_text(path: &Path) -> String {
    let text = path.to_string_lossy();
    if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text.into_owned()
    }
}

/// Git's boolean config spelling (`git config --bool`).
fn git_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "on" | "1"
    )
}

fn run_text(
    runner: &GitCommandRunner,
    start: &Path,
    command: GitCommand,
) -> Result<String, RepositoryError> {
    let _ = start;
    let output = runner.run(
        &command.read_only(true),
        CommandLimits::default(),
        &CancellationToken::new(),
    )?;
    String::from_utf8(output.stdout)
        .map_err(|_| RepositoryError::InvalidOutput("expected UTF-8 Git output".into()))
}

fn parse_usize(value: Option<&str>, name: &str) -> Result<usize, RepositoryError> {
    value
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| RepositoryError::InvalidOutput(format!("invalid {name}")))
}

fn parse_history_entry(record: &[u8]) -> Result<HistoryEntry, RepositoryError> {
    let text = std::str::from_utf8(record)
        .map_err(|_| RepositoryError::InvalidOutput("non-UTF-8 history record".into()))?;
    let fields: Vec<_> = text.splitn(7, '\x1f').collect();
    if fields.len() != 7 {
        return Err(RepositoryError::InvalidOutput(
            "malformed history record".into(),
        ));
    }
    Ok(HistoryEntry {
        oid: GitObjectId::parse(fields[0])
            .map_err(|_| RepositoryError::InvalidOutput("invalid history oid".into()))?,
        parents: fields[1]
            .split_whitespace()
            .map(|parent| {
                GitObjectId::parse(parent)
                    .map_err(|_| RepositoryError::InvalidOutput("invalid parent oid".into()))
            })
            .collect::<Result<_, _>>()?,
        author_name: fields[2].to_owned(),
        author_email: fields[3].to_owned(),
        authored_unix_seconds: fields[4]
            .parse()
            .map_err(|_| RepositoryError::InvalidOutput("invalid author time".into()))?,
        authored_iso: fields[5].to_owned(),
        subject: fields[6].to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn discovers_status_target_diff_history_and_blob() {
        let dir = initialized_repository();
        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        #[cfg(windows)]
        assert!(
            !repository
                .identity()
                .worktree_root
                .as_os_str()
                .to_string_lossy()
                .starts_with(r"\\?\"),
            "persisted repository identity must use the same normal path form as the workspace"
        );
        assert_eq!(
            repository.identity().worktree_root,
            dunce::canonicalize(dir.path()).unwrap()
        );
        assert!(repository.capabilities().unwrap().supports_write_sync());
        let target = repository.resolve_sync_target().unwrap();
        assert_eq!(target.local_branch, "main");
        assert_eq!(target.destination_ref, "refs/heads/main");

        fs::write(dir.path().join("notes.md"), "changed\n").unwrap();
        fs::write(dir.path().join("hidden.bin"), b"new").unwrap();
        let state = repository.status().unwrap();
        assert_eq!(state.worktree.changes.len(), 2);
        assert!(
            repository
                .diff(&DiffRequest::default())
                .unwrap()
                .bytes
                .starts_with(b"diff")
        );
        let history = repository.history(0, 50).unwrap();
        assert_eq!(history.len(), 1);
        let blob = repository
            .historical_blob(&history[0].oid, Path::new("notes.md"), 1024)
            .unwrap();
        assert_eq!(blob.bytes, b"initial\n");
    }

    #[test]
    fn follows_literal_file_history_and_compares_working_versions() {
        let dir = initialized_repository();
        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        fs::write(dir.path().join("notes.md"), "second\n").unwrap();
        git(dir.path(), ["add", "--", "notes.md"]);
        git(dir.path(), ["commit", "-q", "-m", "second"]);
        git(dir.path(), ["mv", "notes.md", "renamed note.md"]);
        git(dir.path(), ["commit", "-q", "-m", "rename"]);

        let history = repository
            .history_page_for_path(Path::new("renamed note.md"), 0, 50)
            .unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].subject, "rename");
        assert_eq!(
            repository
                .history_page_for_path(Path::new("renamed note.md"), 1, 1)
                .unwrap()[0]
                .subject,
            "second"
        );

        fs::write(dir.path().join("renamed note.md"), "working\n").unwrap();
        let comparison = repository
            .working_diff_against(&history[0].oid, Path::new("renamed note.md"), 4096)
            .unwrap();
        assert!(!comparison.binary);
        assert!(!comparison.content.truncated);
        let patch = String::from_utf8(comparison.content.bytes).unwrap();
        assert!(patch.contains("-second"));
        assert!(patch.contains("+working"));
    }

    #[test]
    fn inventories_all_tracked_paths_without_absorbing_untracked_files() {
        let dir = initialized_repository();
        fs::write(dir.path().join("project.toml"), "kind = 'unrelated'\n").unwrap();
        git(dir.path(), ["add", "--", "project.toml"]);
        git(dir.path(), ["commit", "-q", "-m", "mixed content"]);
        fs::write(dir.path().join("private.bin"), [0, 1, 2]).unwrap();

        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        assert_eq!(
            repository.tracked_paths().unwrap(),
            vec![PathBuf::from("notes.md"), PathBuf::from("project.toml")]
        );
    }

    #[test]
    fn working_comparison_reports_binary_and_truncation() {
        let dir = initialized_repository();
        fs::write(dir.path().join("asset.bin"), [0, 1, 2, 3]).unwrap();
        git(dir.path(), ["add", "--", "asset.bin"]);
        git(dir.path(), ["commit", "-q", "-m", "binary"]);
        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        let head = repository.status().unwrap().head.unwrap();
        fs::write(dir.path().join("asset.bin"), [0, 9, 8, 7]).unwrap();
        let binary = repository
            .working_diff_against(&head, Path::new("asset.bin"), 4096)
            .unwrap();
        assert!(binary.binary);

        fs::write(dir.path().join("notes.md"), "changed line\n".repeat(1024)).unwrap();
        let truncated = repository
            .working_diff_against(&head, Path::new("notes.md"), 64)
            .unwrap();
        assert!(truncated.content.truncated);
        assert!(truncated.content.bytes.len() <= 64);
    }

    #[test]
    fn detects_repository_operation_and_lfs_attribute() {
        let dir = initialized_repository();
        fs::write(
            dir.path().join(".gitattributes"),
            "*.png filter=lfs diff=lfs\n",
        )
        .unwrap();
        git(dir.path(), ["add", "--", ".gitattributes"]);
        git(dir.path(), ["commit", "-q", "-m", "lfs"]);
        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        assert!(repository.capabilities().unwrap().uses_lfs);
        fs::write(repository.identity().git_dir.join("MERGE_HEAD"), "abc\n").unwrap();
        assert_eq!(
            repository
                .status()
                .unwrap()
                .worktree
                .operation_in_progress
                .as_deref(),
            Some("merge")
        );
        fs::remove_file(repository.identity().git_dir.join("MERGE_HEAD")).unwrap();
        fs::write(
            repository.identity().git_dir.join("index.lock"),
            "unknown owner\n",
        )
        .unwrap();
        assert_eq!(
            repository
                .status()
                .unwrap()
                .worktree
                .operation_in_progress
                .as_deref(),
            Some("locked")
        );
        assert!(repository.identity().git_dir.join("index.lock").is_file());
    }

    #[test]
    fn classifies_ahead_and_rewritten_history() {
        let dir = initialized_repository();
        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        let first = repository.status().unwrap().head.unwrap();
        fs::write(dir.path().join("second.md"), "second\n").unwrap();
        git(dir.path(), ["add", "--", "second.md"]);
        git(dir.path(), ["commit", "-q", "-m", "second"]);
        let second = repository.status().unwrap().head.unwrap();
        assert_eq!(
            repository
                .classify_history(&second, &first, Some(&first))
                .unwrap(),
            HistoryRelation::Ahead { commits: 1 }
        );
        assert_eq!(
            repository
                .classify_history(&second, &first, Some(&second))
                .unwrap(),
            HistoryRelation::Rewritten
        );
    }

    #[test]
    fn reports_clean_rename_mapping_between_commits() {
        let dir = initialized_repository();
        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        let before = repository.status().unwrap().head.unwrap();
        git(dir.path(), ["mv", "notes.md", "renamed.md"]);
        git(dir.path(), ["commit", "-q", "-m", "rename"]);
        let after = repository.status().unwrap().head.unwrap();
        assert_eq!(
            repository
                .renames_between(&before, &after, &CancellationToken::new())
                .unwrap(),
            vec![(PathBuf::from("notes.md"), PathBuf::from("renamed.md"))]
        );
    }

    #[test]
    fn validates_containment_and_detects_nested_repository() {
        let dir = initialized_repository();
        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        assert_eq!(
            repository
                .validate_write_path(Path::new("notes/new.md"))
                .unwrap(),
            fs::canonicalize(dir.path()).unwrap().join("notes/new.md")
        );
        assert!(
            repository
                .validate_write_path(Path::new("../outside.md"))
                .is_err()
        );

        let nested = dir.path().join("nested");
        git(dir.path(), ["init", "-q", nested.to_str().unwrap()]);
        assert!(repository.capabilities().unwrap().has_nested_repositories);
        assert!(!repository.capabilities().unwrap().supports_write_sync());
    }

    fn counted(dir: &Path) -> (GitRepository, Arc<std::sync::atomic::AtomicUsize>) {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let runner = GitCommandRunner::new("git").with_invocation_counter(counter.clone());
        (GitRepository::discover(runner, dir).unwrap(), counter)
    }

    fn spawned(counter: &std::sync::atomic::AtomicUsize) -> usize {
        counter.swap(0, std::sync::atomic::Ordering::Relaxed)
    }

    #[test]
    fn discovery_uses_one_process_and_later_status_reads_use_one_each() {
        let dir = initialized_repository();
        let (repository, counter) = counted(dir.path());
        assert_eq!(spawned(&counter), 1, "discovery");
        assert!(
            repository
                .status()
                .unwrap()
                .capabilities
                .supports_write_sync()
        );
        spawned(&counter);
        for _ in 0..3 {
            repository.status().unwrap();
            assert_eq!(spawned(&counter), 1, "status after the capability probe");
        }
        // Clones share the memoized probe.
        repository.clone().status().unwrap();
        assert_eq!(spawned(&counter), 1);
    }

    #[test]
    fn unsupported_shapes_are_still_detected_without_a_worktree_walk() {
        let detached = initialized_repository();
        git(detached.path(), ["checkout", "-q", "--detach"]);
        let capabilities = discovered(detached.path()).capabilities().unwrap();
        assert!(capabilities.detached_head && !capabilities.supports_write_sync());

        let gitlink = initialized_repository();
        let inner = initialized_repository();
        let head = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(inner.path())
            .output()
            .unwrap();
        let head = String::from_utf8(head.stdout).unwrap();
        git(
            gitlink.path(),
            [
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("160000,{},vendored", head.trim()),
            ],
        );
        assert!(
            discovered(gitlink.path())
                .capabilities()
                .unwrap()
                .has_nested_repositories
        );

        let sparse = initialized_repository();
        git(sparse.path(), ["config", "core.sparseCheckout", "yes"]);
        assert!(discovered(sparse.path()).capabilities().unwrap().sparse);

        let partial = initialized_repository();
        git(
            partial.path(),
            ["config", "extensions.partialClone", "origin"],
        );
        assert!(discovered(partial.path()).capabilities().unwrap().partial);

        let promisor = initialized_repository();
        git(
            promisor.path(),
            ["config", "remote.origin.promisor", "true"],
        );
        assert!(discovered(promisor.path()).capabilities().unwrap().partial);

        let plain = discovered(initialized_repository().path())
            .capabilities()
            .unwrap();
        assert!(plain.supports_write_sync());
    }

    #[test]
    fn target_resolution_uses_at_most_four_processes_and_honors_insteadof() {
        let dir = initialized_repository();
        git(
            dir.path(),
            [
                "config",
                "url.https://mirror.invalid/.insteadOf",
                "https://example.invalid/",
            ],
        );
        let (repository, counter) = counted(dir.path());
        spawned(&counter);
        let target = repository.resolve_sync_target().unwrap();
        assert!(spawned(&counter) <= 4);
        assert_eq!(target.remote, "origin");
        assert_eq!(target.destination_ref, "refs/heads/main");
        assert_eq!(target.fetch_url, "https://mirror.invalid/notes.git");

        git(
            dir.path(),
            [
                "remote",
                "set-url",
                "--add",
                "--push",
                "origin",
                "https://a.invalid/x.git",
            ],
        );
        git(
            dir.path(),
            [
                "remote",
                "set-url",
                "--add",
                "--push",
                "origin",
                "https://b.invalid/x.git",
            ],
        );
        assert!(matches!(
            repository.resolve_sync_target(),
            Err(RepositoryError::Target(
                TargetResolutionError::MultiplePushDestinations
            ))
        ));
    }

    #[test]
    fn unusual_branch_names_fall_back_to_per_key_queries() {
        let dir = initialized_repository();
        git(dir.path(), ["checkout", "-q", "-b", "notes+daily"]);
        git(
            dir.path(),
            ["config", "branch.notes+daily.remote", "origin"],
        );
        git(
            dir.path(),
            [
                "config",
                "branch.notes+daily.merge",
                "refs/heads/notes+daily",
            ],
        );
        let target = discovered(dir.path()).resolve_sync_target().unwrap();
        assert_eq!(target.local_branch, "notes+daily");
        assert_eq!(target.remote_branch, "notes+daily");
        assert!(is_conservative_ref_name("feature/2026.10-notes_a"));
        for unsafe_name in ["", "a..b", ".hidden", "x.lock", "a//b", "-x", "a/", "a b"] {
            assert!(!is_conservative_ref_name(unsafe_name), "{unsafe_name}");
        }
    }

    #[test]
    fn batched_ignore_and_tracked_queries_are_literal_and_single_process() {
        let dir = initialized_repository();
        fs::write(dir.path().join(".gitignore"), "*.log\n[ab].tmp\n").unwrap();
        let names = [
            "- dash.md",
            "中文 note.md",
            "[ab].md",
            "a.md",
            "skip.log",
            "a.tmp",
        ];
        for name in names {
            fs::write(dir.path().join(name), "x").unwrap();
        }
        git(dir.path(), ["add", "--", ".gitignore", "a.md"]);
        git(dir.path(), ["commit", "-q", "-m", "tracked"]);
        let (repository, counter) = counted(dir.path());
        let paths: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();

        spawned(&counter);
        let ignored = repository.ignored_among(&paths).unwrap();
        assert_eq!(spawned(&counter), 1);
        assert_eq!(
            ignored,
            HashSet::from([PathBuf::from("skip.log"), PathBuf::from("a.tmp")])
        );

        let tracked = repository.tracked_among(&paths).unwrap();
        assert_eq!(spawned(&counter), 1);
        assert_eq!(tracked, HashSet::from([PathBuf::from("a.md")]));

        assert!(repository.ignored_among(&[]).unwrap().is_empty());
        assert!(
            repository
                .ignored_among(&[PathBuf::from("- dash.md")])
                .unwrap()
                .is_empty()
        );
    }

    fn discovered(path: &Path) -> GitRepository {
        GitRepository::discover(GitCommandRunner::new("git"), path).unwrap()
    }

    #[test]
    fn missing_previously_observed_target_is_deleted_not_zero() {
        let oid = GitObjectId::parse("01234567").unwrap();
        assert_eq!(
            GitRepository::classify_missing_target(Some(&oid)),
            HistoryRelation::DeletedTarget
        );
        assert_eq!(
            GitRepository::classify_missing_target(None),
            HistoryRelation::Unknown
        );
    }

    fn initialized_repository() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        git(dir.path(), ["init", "-q", "-b", "main"]);
        git(dir.path(), ["config", "user.name", "Markion Test"]);
        git(dir.path(), ["config", "user.email", "test@markion.invalid"]);
        fs::write(dir.path().join("notes.md"), "initial\n").unwrap();
        git(dir.path(), ["add", "--", "notes.md"]);
        git(dir.path(), ["commit", "-q", "-m", "initial"]);
        git(
            dir.path(),
            [
                "remote",
                "add",
                "origin",
                "https://example.invalid/notes.git",
            ],
        );
        git(dir.path(), ["config", "branch.main.remote", "origin"]);
        git(
            dir.path(),
            ["config", "branch.main.merge", "refs/heads/main"],
        );
        dir
    }

    fn git<const N: usize>(directory: &Path, args: [&str; N]) {
        let output = Command::new("git")
            .current_dir(directory)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", directory.join("missing-global-config"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
