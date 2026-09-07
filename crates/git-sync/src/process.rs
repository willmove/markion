use std::{
    ffi::{OsStr, OsString},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

const DEFAULT_MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(20);

#[derive(Clone, Debug)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandLimits {
    pub timeout: Duration,
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
}

impl Default for CommandLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(15),
            max_stdout_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            max_stderr_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessTermination {
    Exited,
    Cancelled,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCommandOutput {
    pub status_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub termination: ProcessTermination,
    pub elapsed: Duration,
}

impl GitCommandOutput {
    pub fn success(&self) -> bool {
        self.termination == ProcessTermination::Exited && self.status_code == Some(0)
    }

    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    pub fn stderr_text(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GitCommandError {
    #[error("failed to start Git: {0}")]
    Spawn(#[source] io::Error),
    #[error("failed while waiting for Git: {0}")]
    Wait(#[source] io::Error),
    #[error("failed to read Git output: {0}")]
    Output(#[source] io::Error),
    #[error("Git exited unsuccessfully: {output:?}")]
    Failed { output: GitCommandOutput },
}

impl GitCommandError {
    pub fn output(&self) -> Option<&GitCommandOutput> {
        match self {
            Self::Failed { output } => Some(output),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct GitCommand {
    pub working_directory: PathBuf,
    pub arguments: Vec<OsString>,
    pub allow_interactive_credentials: bool,
    pub read_only: bool,
    environment: Vec<(OsString, OsString)>,
    standard_input: Option<Vec<u8>>,
}

impl std::fmt::Debug for GitCommand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GitCommand")
            .field("working_directory", &self.working_directory)
            .field("arguments", &self.arguments)
            .field(
                "allow_interactive_credentials",
                &self.allow_interactive_credentials,
            )
            .field("read_only", &self.read_only)
            .field(
                "environment_keys",
                &self
                    .environment
                    .iter()
                    .map(|(key, _)| key)
                    .collect::<Vec<_>>(),
            )
            .field("has_standard_input", &self.standard_input.is_some())
            .finish()
    }
}

impl GitCommand {
    pub fn new(working_directory: impl Into<PathBuf>) -> Self {
        Self {
            working_directory: working_directory.into(),
            arguments: Vec::new(),
            allow_interactive_credentials: false,
            read_only: false,
            environment: Vec::new(),
            standard_input: None,
        }
    }

    pub fn arg(mut self, argument: impl Into<OsString>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub fn args(mut self, arguments: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        self.arguments.extend(arguments.into_iter().map(Into::into));
        self
    }

    /// Add literal paths after `--` so a leading dash can never become an option.
    pub fn literal_paths(mut self, paths: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Self {
        self.arguments.push(OsString::from("--"));
        self.arguments
            .extend(paths.into_iter().map(|path| path.as_ref().to_owned()));
        self
    }

    pub fn interactive_credentials(mut self, enabled: bool) -> Self {
        self.allow_interactive_credentials = enabled;
        self
    }

    pub fn read_only(mut self, enabled: bool) -> Self {
        self.read_only = enabled;
        self
    }

    /// Set a process-only environment value. Debug output includes the key
    /// while always redacting its value.
    pub(crate) fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.environment.push((key.into(), value.into()));
        self
    }

    /// Provide bounded standard input without putting secrets in arguments or
    /// diagnostic output. Intended for Git credential-helper protocols.
    pub fn standard_input(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.standard_input = Some(bytes.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct GitCommandRunner {
    executable: PathBuf,
}

impl GitCommandRunner {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn run(
        &self,
        request: &GitCommand,
        limits: CommandLimits,
        cancellation: &CancellationToken,
    ) -> Result<GitCommandOutput, GitCommandError> {
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&request.working_directory)
            .args(&request.arguments)
            .stdin(if request.standard_input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        sanitize_environment(&mut command, request.allow_interactive_credentials);
        for (key, value) in &request.environment {
            command.env(key, value);
        }
        if request.read_only {
            command.env("GIT_OPTIONAL_LOCKS", "0");
        }
        hide_windows_console(&mut command);

        let start = Instant::now();
        let mut child = command.spawn().map_err(GitCommandError::Spawn)?;
        let stdin_writer = request.standard_input.clone().map(|bytes| {
            let mut stdin = child.stdin.take().expect("stdin configured as piped");
            thread::spawn(move || -> io::Result<()> {
                stdin.write_all(&bytes)?;
                stdin.flush()
            })
        });
        let stdout = child.stdout.take().expect("stdout configured as piped");
        let stderr = child.stderr.take().expect("stderr configured as piped");
        let stdout_reader = thread::spawn(move || read_bounded(stdout, limits.max_stdout_bytes));
        let stderr_reader = thread::spawn(move || read_bounded(stderr, limits.max_stderr_bytes));

        let (termination, status) = loop {
            if cancellation.is_cancelled() {
                let _ = child.kill();
                let status = child.wait().map_err(GitCommandError::Wait)?;
                break (ProcessTermination::Cancelled, status);
            }
            if start.elapsed() >= limits.timeout {
                let _ = child.kill();
                let status = child.wait().map_err(GitCommandError::Wait)?;
                break (ProcessTermination::TimedOut, status);
            }
            match child.try_wait().map_err(GitCommandError::Wait)? {
                Some(status) => break (ProcessTermination::Exited, status),
                None => thread::sleep(POLL_INTERVAL),
            }
        };

        let (stdout, stdout_truncated) = join_reader(stdout_reader)?;
        let (stderr, stderr_truncated) = join_reader(stderr_reader)?;
        if let Some(writer) = stdin_writer {
            writer
                .join()
                .map_err(|_| {
                    GitCommandError::Output(io::Error::other("Git input writer panicked"))
                })?
                .map_err(GitCommandError::Output)?;
        }
        let output = GitCommandOutput {
            status_code: exit_code(status),
            stdout,
            stderr,
            stdout_truncated,
            stderr_truncated,
            termination,
            elapsed: start.elapsed(),
        };
        if output.success() {
            Ok(output)
        } else {
            Err(GitCommandError::Failed { output })
        }
    }
}

fn join_reader(
    reader: thread::JoinHandle<io::Result<(Vec<u8>, bool)>>,
) -> Result<(Vec<u8>, bool), GitCommandError> {
    reader
        .join()
        .map_err(|_| GitCommandError::Output(io::Error::other("Git output reader panicked")))?
        .map_err(GitCommandError::Output)
}

fn read_bounded(mut reader: impl Read, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut output = Vec::with_capacity(limit.min(16 * 1024));
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(output.len());
        let keep = remaining.min(count);
        output.extend_from_slice(&buffer[..keep]);
        truncated |= keep < count;
    }
    Ok((output, truncated))
}

fn sanitize_environment(command: &mut Command, allow_interactive_credentials: bool) {
    const EXACT: &[&str] = &[
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_COMMON_DIR",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG_COUNT",
    ];
    for key in EXACT {
        command.env_remove(key);
    }
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy();
        if name.starts_with("GIT_CONFIG_KEY_") || name.starts_with("GIT_CONFIG_VALUE_") {
            command.env_remove(key);
        }
    }
    if !allow_interactive_credentials {
        command.env("GIT_TERMINAL_PROMPT", "0");
    }
}

#[cfg(windows)]
fn hide_windows_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_windows_console(_: &mut Command) {}

fn exit_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn literal_path_with_spaces_unicode_and_dash_is_not_an_option() {
        let dir = tempdir().unwrap();
        let runner = GitCommandRunner::new("git");
        run_ok(&runner, dir.path(), ["init", "-q"]);
        run_ok(&runner, dir.path(), ["config", "user.name", "Markion Test"]);
        run_ok(
            &runner,
            dir.path(),
            ["config", "user.email", "test@markion.invalid"],
        );
        let name = "- 中文 note.md";
        fs::write(dir.path().join(name), "hello").unwrap();
        let request = GitCommand::new(dir.path()).arg("add").literal_paths([name]);
        runner
            .run(
                &request,
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .unwrap();
        let output = runner
            .run(
                &GitCommand::new(dir.path())
                    .args(["diff", "--cached", "--name-only", "-z"])
                    .read_only(true),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(output.stdout, format!("{name}\0").as_bytes());
    }

    #[test]
    fn output_is_drained_and_bounded() {
        let dir = tempdir().unwrap();
        let runner = GitCommandRunner::new("git");
        run_ok(&runner, dir.path(), ["init", "-q"]);
        for index in 0..200 {
            fs::write(dir.path().join(format!("note-{index}.md")), "x").unwrap();
        }
        let request = GitCommand::new(dir.path())
            .args(["status", "--short", "--untracked-files=all"])
            .read_only(true);
        let output = runner
            .run(
                &request,
                CommandLimits {
                    max_stdout_bytes: 64,
                    ..CommandLimits::default()
                },
                &CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(output.stdout.len(), 64);
        assert!(output.stdout_truncated);
    }

    #[test]
    fn pre_cancelled_command_reports_reconciliation_required_termination() {
        let directory = tempdir().unwrap();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let error = GitCommandRunner::new("git")
            .run(
                &GitCommand::new(directory.path()).arg("--version"),
                CommandLimits::default(),
                &cancellation,
            )
            .unwrap_err();
        assert_eq!(
            error.output().unwrap().termination,
            ProcessTermination::Cancelled
        );
    }

    fn run_ok<const N: usize>(runner: &GitCommandRunner, directory: &Path, args: [&str; N]) {
        runner
            .run(
                &GitCommand::new(directory).args(args),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .unwrap();
    }
}
