//! Shell-free, bounded execution for user-configured image upload programs.

use std::{
    ffi::OsString,
    io::{self, Read},
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

const POLL_INTERVAL: Duration = Duration::from_millis(20);
pub const DEFAULT_COMMAND_OUTPUT_LIMIT: usize = 1024 * 1024;

#[derive(Clone, Debug, Default)]
pub struct CommandCancellation(Arc<AtomicBool>);

impl CommandCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExternalCommandLimits {
    pub timeout: Duration,
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
}

impl Default for ExternalCommandLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            max_stdout_bytes: DEFAULT_COMMAND_OUTPUT_LIMIT,
            max_stderr_bytes: DEFAULT_COMMAND_OUTPUT_LIMIT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalCommandTermination {
    Exited,
    Cancelled,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalCommandOutput {
    pub status_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub termination: ExternalCommandTermination,
}

impl ExternalCommandOutput {
    pub fn success(&self) -> bool {
        self.termination == ExternalCommandTermination::Exited && self.status_code == Some(0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalCommandRequest {
    pub program: PathBuf,
    pub arguments: Vec<OsString>,
    pub working_directory: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ExternalCommandError {
    #[error("failed to start external uploader: {0}")]
    Spawn(#[source] io::Error),
    #[error("failed while waiting for external uploader: {0}")]
    Wait(#[source] io::Error),
    #[error("failed to read external uploader output: {0}")]
    Output(#[source] io::Error),
}

pub fn run_external_command(
    request: &ExternalCommandRequest,
    limits: ExternalCommandLimits,
    cancellation: &CommandCancellation,
) -> Result<ExternalCommandOutput, ExternalCommandError> {
    let mut command = Command::new(&request.program);
    command
        .current_dir(&request.working_directory)
        .args(&request.arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_process_group(&mut command);
    hide_windows_console(&mut command);

    let mut child = command.spawn().map_err(ExternalCommandError::Spawn)?;
    let child_id = child.id();
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stdout_reader = thread::spawn(move || read_bounded(stdout, limits.max_stdout_bytes));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, limits.max_stderr_bytes));
    let started = Instant::now();
    let (termination, status) = loop {
        if cancellation.is_cancelled() {
            terminate_process_tree(child_id, &mut child);
            let status = child.wait().map_err(ExternalCommandError::Wait)?;
            break (ExternalCommandTermination::Cancelled, status);
        }
        if started.elapsed() >= limits.timeout {
            terminate_process_tree(child_id, &mut child);
            let status = child.wait().map_err(ExternalCommandError::Wait)?;
            break (ExternalCommandTermination::TimedOut, status);
        }
        match child.try_wait().map_err(ExternalCommandError::Wait)? {
            Some(status) => break (ExternalCommandTermination::Exited, status),
            None => thread::sleep(POLL_INTERVAL),
        }
    };
    let (stdout, stdout_truncated) = join_reader(stdout_reader)?;
    let (stderr, stderr_truncated) = join_reader(stderr_reader)?;
    Ok(ExternalCommandOutput {
        status_code: exit_code(status),
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
        termination,
    })
}

fn join_reader(
    reader: thread::JoinHandle<io::Result<(Vec<u8>, bool)>>,
) -> Result<(Vec<u8>, bool), ExternalCommandError> {
    reader
        .join()
        .map_err(|_| ExternalCommandError::Output(io::Error::other("output reader panicked")))?
        .map_err(ExternalCommandError::Output)
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

#[cfg(windows)]
fn terminate_process_tree(process_id: u32, child: &mut std::process::Child) {
    use std::os::windows::process::CommandExt as _;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let _ = Command::new("taskkill.exe")
        .args(["/PID", &process_id.to_string(), "/T", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

#[cfg(not(windows))]
fn terminate_process_tree(process_id: u32, child: &mut std::process::Child) {
    // The child is started as process-group leader, so this signal owns only
    // the uploader tree created by this request.
    unsafe {
        libc::kill(-(process_id as i32), libc::SIGKILL);
    }
    let _ = child.kill();
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt as _;
    command.process_group(0);
}

#[cfg(windows)]
fn configure_process_group(_command: &mut Command) {}

#[cfg(windows)]
fn hide_windows_console(command: &mut Command) {
    use std::os::windows::process::CommandExt as _;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_windows_console(_command: &mut Command) {}

#[cfg(unix)]
fn exit_code(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt as _;
    status
        .code()
        .or_else(|| status.signal().map(|signal| -signal))
}

#[cfg(windows)]
fn exit_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    fn request(script: &str, working_directory: &std::path::Path) -> ExternalCommandRequest {
        ExternalCommandRequest {
            program: PathBuf::from("pwsh.exe"),
            arguments: vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                script.into(),
            ],
            working_directory: working_directory.to_path_buf(),
        }
    }

    #[cfg(not(windows))]
    fn request(script: &str, working_directory: &std::path::Path) -> ExternalCommandRequest {
        ExternalCommandRequest {
            program: PathBuf::from("sh"),
            arguments: vec!["-c".into(), script.into()],
            working_directory: working_directory.to_path_buf(),
        }
    }

    #[test]
    fn drains_both_outputs_and_reports_nonzero_without_embedding_arguments() {
        let dir = tempfile::tempdir().unwrap();
        #[cfg(windows)]
        let script = "[Console]::Out.Write('out'); [Console]::Error.Write('err'); exit 7";
        #[cfg(not(windows))]
        let script = "printf out; printf err >&2; exit 7";
        let output = run_external_command(
            &request(script, dir.path()),
            ExternalCommandLimits::default(),
            &CommandCancellation::default(),
        )
        .unwrap();
        assert_eq!(output.status_code, Some(7));
        assert_eq!(output.stdout, b"out");
        assert_eq!(output.stderr, b"err");
        assert!(!output.success());
    }

    #[test]
    fn bounds_output_and_times_out_hanging_processes() {
        let dir = tempfile::tempdir().unwrap();
        #[cfg(windows)]
        let noisy = "$s = 'x' * 4096; [Console]::Out.Write($s)";
        #[cfg(not(windows))]
        let noisy = "head -c 4096 /dev/zero | tr '\\0' x";
        let output = run_external_command(
            &request(noisy, dir.path()),
            ExternalCommandLimits {
                max_stdout_bytes: 128,
                ..ExternalCommandLimits::default()
            },
            &CommandCancellation::default(),
        )
        .unwrap();
        assert_eq!(output.stdout.len(), 128);
        assert!(output.stdout_truncated);

        #[cfg(windows)]
        let hanging = "$p = Start-Process pwsh.exe -ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 30' -PassThru; Wait-Process -Id $p.Id";
        #[cfg(not(windows))]
        let hanging = "sleep 30 & wait";
        let output = run_external_command(
            &request(hanging, dir.path()),
            ExternalCommandLimits {
                timeout: Duration::from_millis(100),
                ..ExternalCommandLimits::default()
            },
            &CommandCancellation::default(),
        )
        .unwrap();
        assert_eq!(output.termination, ExternalCommandTermination::TimedOut);
    }

    #[test]
    fn passes_unicode_metacharacter_argument_literally() {
        let dir = tempfile::tempdir().unwrap();
        let value = "图 片;$(not-executed)&.png";
        #[cfg(windows)]
        let request = {
            let script = dir.path().join("echo-argument.ps1");
            std::fs::write(
                &script,
                "param([string]$value) [Console]::OutputEncoding = [Text.UTF8Encoding]::new(); [Console]::Out.Write($value)",
            )
            .unwrap();
            ExternalCommandRequest {
                program: PathBuf::from("pwsh.exe"),
                arguments: vec![
                    "-NoLogo".into(),
                    "-NoProfile".into(),
                    "-NonInteractive".into(),
                    "-File".into(),
                    script.into_os_string(),
                    value.into(),
                ],
                working_directory: dir.path().to_path_buf(),
            }
        };
        #[cfg(not(windows))]
        let request = ExternalCommandRequest {
            program: PathBuf::from("sh"),
            arguments: vec![
                "-c".into(),
                "printf %s \"$1\"".into(),
                "fixture".into(),
                value.into(),
            ],
            working_directory: dir.path().to_path_buf(),
        };
        let output = run_external_command(
            &request,
            ExternalCommandLimits::default(),
            &CommandCancellation::default(),
        )
        .unwrap();
        assert!(output.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), value);
    }
}
