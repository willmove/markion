use std::{
    collections::{BTreeMap, VecDeque},
    ffi::{OsStr, OsString},
    fs,
    io::Read,
    path::PathBuf,
    process::{Child, ChildStderr, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use semver::Version;
use thiserror::Error;

use crate::{
    CapabilityDeclaration, Frame, FrameError, FrameKind, FrameLimits, HandshakeAccepted,
    HandshakeHello, HostMessage, PluginMessage, ProtocolVersion, TargetSpec, read_frame,
    write_frame,
};

const STATE_RUNNING: u8 = 0;
const STATE_SHUTTING_DOWN: u8 = 1;
const STATE_STOPPED: u8 = 2;
const STATE_FAILED: u8 = 3;
const MAX_BUFFERED_EVENTS: usize = 64;

#[derive(Clone, Debug)]
pub struct WorkerLaunch {
    pub executable: PathBuf,
    pub working_directory: PathBuf,
    pub protocol: ProtocolVersion,
    pub host_version: Version,
    pub plugin_id: String,
    pub plugin_version: Version,
    pub package_sha256: String,
    pub requested_capabilities: Vec<String>,
    pub expected_capabilities: Vec<CapabilityDeclaration>,
    pub frame_limits: FrameLimits,
    pub max_in_flight_requests: usize,
    pub request_timeout: Duration,
    pub shutdown_timeout: Duration,
    pub stderr_limit_bytes: usize,
    /// Extra variables are host-selected values. Manifest content never enters
    /// this list; production callers normally leave it empty.
    pub environment: Vec<(OsString, OsString)>,
}

impl WorkerLaunch {
    pub fn fixture(executable: impl Into<PathBuf>, working_directory: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            working_directory: working_directory.into(),
            protocol: ProtocolVersion::V1_0,
            host_version: Version::new(0, 3, 9),
            plugin_id: "dev.markion.fixture".to_owned(),
            plugin_version: Version::new(1, 0, 0),
            package_sha256: "fixture".to_owned(),
            requested_capabilities: Vec::new(),
            expected_capabilities: Vec::new(),
            frame_limits: FrameLimits::default(),
            max_in_flight_requests: 8,
            request_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(2),
            stderr_limit_bytes: 16 * 1024,
            environment: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerFailureKind {
    Startup,
    Exited,
    Io,
    Protocol,
    ForcedShutdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerFailure {
    pub kind: WorkerFailureKind,
    pub detail: String,
}

impl WorkerFailure {
    fn new(kind: WorkerFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessResponse {
    pub request_id: u64,
    pub generation: u64,
    pub message: PluginMessage,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticSnapshot {
    pub bytes: Vec<u8>,
    pub total_bytes: u64,
    pub truncated: bool,
}

#[derive(Debug, Error)]
pub enum ProcessPeerError {
    #[error("worker launch configuration is invalid: {0}")]
    InvalidLaunch(String),
    #[error("unable to start the plugin worker")]
    Spawn(#[source] std::io::Error),
    #[error("plugin process setup failed: {0}")]
    ProcessSetup(String),
    #[error("plugin frame failed")]
    Frame(#[from] FrameError),
    #[error("plugin request table is full")]
    Backpressure,
    #[error("plugin request timed out")]
    Timeout,
    #[error("plugin request was cancelled")]
    Cancelled,
    #[error("plugin worker is not running")]
    NotRunning,
    #[error("plugin worker failed: {0:?}")]
    Worker(WorkerFailure),
    #[error("plugin handshake response is incompatible")]
    IncompatibleHandshake,
}

struct PendingEntry {
    generation: u64,
    sender: SyncSender<Result<ProcessResponse, WorkerFailure>>,
}

#[derive(Default)]
struct DiagnosticRing {
    bytes: VecDeque<u8>,
    limit: usize,
    total_bytes: u64,
}

impl DiagnosticRing {
    fn new(limit: usize) -> Self {
        Self {
            bytes: VecDeque::with_capacity(limit.min(4096)),
            limit,
            total_bytes: 0,
        }
    }

    fn push(&mut self, chunk: &[u8]) {
        self.total_bytes = self
            .total_bytes
            .saturating_add(u64::try_from(chunk.len()).unwrap_or(u64::MAX));
        if self.limit == 0 {
            return;
        }
        if chunk.len() >= self.limit {
            self.bytes.clear();
            self.bytes.extend(
                chunk[chunk.len().saturating_sub(self.limit)..]
                    .iter()
                    .copied(),
            );
            return;
        }
        while self.bytes.len() + chunk.len() > self.limit {
            self.bytes.pop_front();
        }
        self.bytes.extend(chunk.iter().copied());
    }

    fn snapshot(&self) -> DiagnosticSnapshot {
        DiagnosticSnapshot {
            bytes: self.bytes.iter().copied().collect(),
            total_bytes: self.total_bytes,
            truncated: self.total_bytes > self.bytes.len() as u64,
        }
    }
}

struct Shared {
    protocol: ProtocolVersion,
    generation: u64,
    frame_limits: FrameLimits,
    max_in_flight: usize,
    request_timeout: Duration,
    stdin: Mutex<Option<ChildStdin>>,
    pending: Mutex<BTreeMap<u64, PendingEntry>>,
    events: Mutex<VecDeque<ProcessResponse>>,
    diagnostics: Mutex<DiagnosticRing>,
    failure: Mutex<Option<WorkerFailure>>,
    next_request_id: AtomicU64,
    state: AtomicU8,
    process_tree: Arc<ProcessTree>,
}

impl Shared {
    fn begin_request(
        self: &Arc<Self>,
        kind: FrameKind,
        message: HostMessage,
        body: Vec<u8>,
        allow_shutdown: bool,
    ) -> Result<PendingProcessRequest, ProcessPeerError> {
        let state = self.state.load(Ordering::Acquire);
        if state != STATE_RUNNING && !(allow_shutdown && state == STATE_SHUTTING_DOWN) {
            return Err(self
                .current_failure()
                .map_or(ProcessPeerError::NotRunning, ProcessPeerError::Worker));
        }

        let request_id = self.allocate_request_id()?;
        let (sender, receiver) = mpsc::sync_channel(1);
        {
            let mut pending = self.pending.lock().expect("pending request lock poisoned");
            if pending.len() >= self.max_in_flight {
                return Err(ProcessPeerError::Backpressure);
            }
            pending.insert(
                request_id,
                PendingEntry {
                    generation: self.generation,
                    sender,
                },
            );
        }

        let frame = Frame::from_header(self.protocol, kind, request_id, &message, body)?;
        if let Err(error) = self.write(&frame) {
            self.pending
                .lock()
                .expect("pending request lock poisoned")
                .remove(&request_id);
            let failure = WorkerFailure::new(WorkerFailureKind::Io, error.to_string());
            self.fail(failure.clone());
            return Err(ProcessPeerError::Worker(failure));
        }
        Ok(PendingProcessRequest {
            request_id,
            generation: self.generation,
            receiver: Some(receiver),
            shared: Arc::clone(self),
            complete: false,
        })
    }

    fn allocate_request_id(&self) -> Result<u64, ProcessPeerError> {
        let id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        if id == 0 || id == u64::MAX {
            self.fail(WorkerFailure::new(
                WorkerFailureKind::Protocol,
                "request identifier space exhausted",
            ));
            return Err(ProcessPeerError::NotRunning);
        }
        Ok(id)
    }

    fn write(&self, frame: &Frame) -> Result<(), FrameError> {
        let mut stdin = self.stdin.lock().expect("plugin stdin lock poisoned");
        let writer = stdin.as_mut().ok_or_else(|| {
            FrameError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "plugin stdin is closed",
            ))
        })?;
        write_frame(writer, frame, self.frame_limits)
    }

    fn cancel(&self, request_id: u64, generation: u64) -> Result<(), ProcessPeerError> {
        if generation != self.generation {
            return Ok(());
        }
        let removed = self
            .pending
            .lock()
            .expect("pending request lock poisoned")
            .remove(&request_id);
        if removed.is_none() {
            return Ok(());
        }
        if self.state.load(Ordering::Acquire) != STATE_RUNNING {
            return Ok(());
        }
        let cancel_id = self.allocate_request_id()?;
        let frame = Frame::from_header(
            self.protocol,
            FrameKind::Cancel,
            cancel_id,
            &HostMessage::Cancel { request_id },
            Vec::new(),
        )?;
        self.write(&frame)?;
        Ok(())
    }

    fn dispatch(&self, frame: Frame) -> Result<(), WorkerFailure> {
        if frame.protocol != self.protocol {
            return Err(WorkerFailure::new(
                WorkerFailureKind::Protocol,
                "worker returned an unexpected protocol version",
            ));
        }
        let message = frame
            .decode_header::<PluginMessage>()
            .map_err(|error| WorkerFailure::new(WorkerFailureKind::Protocol, error.to_string()))?;
        let response = ProcessResponse {
            request_id: frame.request_id,
            generation: self.generation,
            message,
            body: frame.body,
        };
        match frame.kind {
            FrameKind::Response | FrameKind::Shutdown => {
                let pending = self
                    .pending
                    .lock()
                    .expect("pending request lock poisoned")
                    .remove(&response.request_id);
                if let Some(pending) = pending
                    && pending.generation == self.generation
                {
                    let _ = pending.sender.send(Ok(response));
                }
                Ok(())
            }
            FrameKind::Event => {
                let mut events = self.events.lock().expect("event queue lock poisoned");
                if events.len() == MAX_BUFFERED_EVENTS {
                    events.pop_front();
                }
                events.push_back(response);
                Ok(())
            }
            _ => Err(WorkerFailure::new(
                WorkerFailureKind::Protocol,
                "worker emitted a request-only frame kind",
            )),
        }
    }

    fn fail(&self, failure: WorkerFailure) {
        if self
            .state
            .compare_exchange(
                STATE_RUNNING,
                STATE_FAILED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
            && self.state.load(Ordering::Acquire) != STATE_SHUTTING_DOWN
        {
            return;
        }
        *self.failure.lock().expect("failure lock poisoned") = Some(failure.clone());
        let pending =
            std::mem::take(&mut *self.pending.lock().expect("pending request lock poisoned"));
        for (_, request) in pending {
            let _ = request.sender.send(Err(failure.clone()));
        }
        let _ = self.process_tree.terminate();
    }

    fn current_failure(&self) -> Option<WorkerFailure> {
        self.failure.lock().expect("failure lock poisoned").clone()
    }
}

pub struct PendingProcessRequest {
    request_id: u64,
    generation: u64,
    receiver: Option<Receiver<Result<ProcessResponse, WorkerFailure>>>,
    shared: Arc<Shared>,
    complete: bool,
}

impl PendingProcessRequest {
    pub fn request_id(&self) -> u64 {
        self.request_id
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn wait(mut self, timeout: Duration) -> Result<ProcessResponse, ProcessPeerError> {
        let receiver = self
            .receiver
            .take()
            .expect("request receiver already consumed");
        let result = match receiver.recv_timeout(timeout) {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(failure)) => Err(ProcessPeerError::Worker(failure)),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let _ = self.shared.cancel(self.request_id, self.generation);
                Err(ProcessPeerError::Timeout)
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(self
                .shared
                .current_failure()
                .map_or(ProcessPeerError::NotRunning, ProcessPeerError::Worker)),
        };
        self.complete = true;
        result
    }

    pub fn cancel(mut self) -> Result<(), ProcessPeerError> {
        let result = self.shared.cancel(self.request_id, self.generation);
        self.complete = true;
        result
    }
}

impl Drop for PendingProcessRequest {
    fn drop(&mut self) {
        if !self.complete {
            let _ = self.shared.cancel(self.request_id, self.generation);
        }
    }
}

struct Runtime {
    child: Option<Child>,
    reader: Option<JoinHandle<()>>,
    stderr: Option<JoinHandle<()>>,
}

pub struct ProcessPluginPeer {
    shared: Arc<Shared>,
    runtime: Mutex<Runtime>,
    shutdown_timeout: Duration,
    identity: HandshakeHello,
    expected_capabilities: Vec<CapabilityDeclaration>,
}

impl ProcessPluginPeer {
    pub fn launch(config: WorkerLaunch, generation: u64) -> Result<Self, ProcessPeerError> {
        validate_launch(&config)?;
        let executable = fs::canonicalize(&config.executable).map_err(ProcessPeerError::Spawn)?;
        let working_directory =
            fs::canonicalize(&config.working_directory).map_err(ProcessPeerError::Spawn)?;
        if !executable.starts_with(&working_directory) || !executable.is_file() {
            return Err(ProcessPeerError::InvalidLaunch(
                "entry point must be a file beneath the verified version directory".to_owned(),
            ));
        }

        let mut command = Command::new(&executable);
        command
            .current_dir(&working_directory)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        add_sanitized_environment(&mut command, &config);
        configure_process_group(&mut command)?;

        let mut child = command.spawn().map_err(ProcessPeerError::Spawn)?;
        let process_tree = match ProcessTree::attach(&child) {
            Ok(tree) => Arc::new(tree),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        };
        let stdin = child.stdin.take().ok_or_else(|| {
            ProcessPeerError::ProcessSetup("worker stdin was not piped".to_owned())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            ProcessPeerError::ProcessSetup("worker stdout was not piped".to_owned())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            ProcessPeerError::ProcessSetup("worker stderr was not piped".to_owned())
        })?;

        let shared = Arc::new(Shared {
            protocol: config.protocol,
            generation,
            frame_limits: config.frame_limits,
            max_in_flight: config.max_in_flight_requests,
            request_timeout: config.request_timeout,
            stdin: Mutex::new(Some(stdin)),
            pending: Mutex::new(BTreeMap::new()),
            events: Mutex::new(VecDeque::new()),
            diagnostics: Mutex::new(DiagnosticRing::new(config.stderr_limit_bytes)),
            failure: Mutex::new(None),
            next_request_id: AtomicU64::new(1),
            state: AtomicU8::new(STATE_RUNNING),
            process_tree,
        });
        let reader = spawn_reader(Arc::clone(&shared), stdout);
        let stderr = spawn_stderr_reader(Arc::clone(&shared), stderr);
        let identity = HandshakeHello {
            protocol: config.protocol,
            host_version: config.host_version,
            expected_plugin_id: config.plugin_id,
            expected_plugin_version: config.plugin_version,
            package_sha256: config.package_sha256,
            requested_capabilities: config.requested_capabilities,
        };
        let peer = Self {
            shared,
            runtime: Mutex::new(Runtime {
                child: Some(child),
                reader: Some(reader),
                stderr: Some(stderr),
            }),
            shutdown_timeout: config.shutdown_timeout,
            identity,
            expected_capabilities: config.expected_capabilities,
        };
        if let Err(error) = peer.handshake() {
            peer.force_terminate();
            return Err(error);
        }
        Ok(peer)
    }

    pub fn generation(&self) -> u64 {
        self.shared.generation
    }

    pub fn begin_request(
        &self,
        message: HostMessage,
        body: Vec<u8>,
    ) -> Result<PendingProcessRequest, ProcessPeerError> {
        self.shared
            .begin_request(FrameKind::Request, message, body, false)
    }

    pub fn request(
        &self,
        message: HostMessage,
        body: Vec<u8>,
    ) -> Result<ProcessResponse, ProcessPeerError> {
        self.begin_request(message, body)?
            .wait(self.shared.request_timeout)
    }

    pub fn request_with_timeout(
        &self,
        message: HostMessage,
        body: Vec<u8>,
        timeout: Duration,
    ) -> Result<ProcessResponse, ProcessPeerError> {
        self.begin_request(message, body)?.wait(timeout)
    }

    pub fn try_next_event(&self) -> Option<ProcessResponse> {
        self.shared
            .events
            .lock()
            .expect("event queue lock poisoned")
            .pop_front()
    }

    pub fn diagnostics(&self) -> DiagnosticSnapshot {
        self.shared
            .diagnostics
            .lock()
            .expect("diagnostic lock poisoned")
            .snapshot()
    }

    pub fn failure(&self) -> Option<WorkerFailure> {
        self.shared.current_failure()
    }

    pub fn is_running(&self) -> bool {
        self.shared.state.load(Ordering::Acquire) == STATE_RUNNING
    }

    pub fn shutdown(&self) -> Result<(), ProcessPeerError> {
        if self
            .shared
            .state
            .compare_exchange(
                STATE_RUNNING,
                STATE_SHUTTING_DOWN,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            return if self.shared.state.load(Ordering::Acquire) == STATE_STOPPED {
                Ok(())
            } else {
                Err(self
                    .shared
                    .current_failure()
                    .map_or(ProcessPeerError::NotRunning, ProcessPeerError::Worker))
            };
        }
        let pending = self.shared.begin_request(
            FrameKind::Shutdown,
            HostMessage::Shutdown,
            Vec::new(),
            true,
        )?;
        let response = pending.wait(self.shutdown_timeout)?;
        if response.message != PluginMessage::ShutdownAccepted {
            let failure = WorkerFailure::new(
                WorkerFailureKind::Protocol,
                "worker did not acknowledge shutdown",
            );
            self.shared.fail(failure.clone());
            return Err(ProcessPeerError::Worker(failure));
        }
        self.shared
            .stdin
            .lock()
            .expect("plugin stdin lock poisoned")
            .take();
        let status = {
            let mut runtime = self.runtime.lock().expect("runtime lock poisoned");
            runtime.child.as_mut().map(Child::wait).transpose()
        }
        .map_err(|error| {
            ProcessPeerError::Worker(WorkerFailure::new(WorkerFailureKind::Io, error.to_string()))
        })?;
        if status.is_some_and(|status| !status.success()) {
            return Err(ProcessPeerError::Worker(WorkerFailure::new(
                WorkerFailureKind::Exited,
                "worker exited unsuccessfully after shutdown",
            )));
        }
        self.shared.state.store(STATE_STOPPED, Ordering::Release);
        self.join_threads();
        Ok(())
    }

    pub fn force_terminate(&self) {
        let previous = self.shared.state.swap(STATE_STOPPED, Ordering::AcqRel);
        self.shared
            .stdin
            .lock()
            .expect("plugin stdin lock poisoned")
            .take();
        let _ = self.shared.process_tree.terminate();
        let mut runtime = self.runtime.lock().expect("runtime lock poisoned");
        if let Some(child) = runtime.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        drop(runtime);
        if previous == STATE_RUNNING || previous == STATE_SHUTTING_DOWN {
            let failure = WorkerFailure::new(
                WorkerFailureKind::ForcedShutdown,
                "worker was forcibly terminated",
            );
            *self.shared.failure.lock().expect("failure lock poisoned") = Some(failure.clone());
            let pending = std::mem::take(
                &mut *self
                    .shared
                    .pending
                    .lock()
                    .expect("pending request lock poisoned"),
            );
            for (_, request) in pending {
                let _ = request.sender.send(Err(failure.clone()));
            }
        }
        self.join_threads();
    }

    fn handshake(&self) -> Result<(), ProcessPeerError> {
        let response = self.request(HostMessage::Handshake(self.identity.clone()), Vec::new())?;
        let PluginMessage::HandshakeAccepted(accepted) = response.message else {
            return Err(ProcessPeerError::IncompatibleHandshake);
        };
        self.validate_handshake(&accepted)
    }

    fn validate_handshake(&self, accepted: &HandshakeAccepted) -> Result<(), ProcessPeerError> {
        if accepted.protocol != self.identity.protocol
            || accepted.plugin_id != self.identity.expected_plugin_id
            || accepted.plugin_version != self.identity.expected_plugin_version
            || accepted.target != TargetSpec::current()
            || accepted.package_sha256 != self.identity.package_sha256
            || accepted.capabilities != self.expected_capabilities
        {
            return Err(ProcessPeerError::IncompatibleHandshake);
        }
        Ok(())
    }

    fn join_threads(&self) {
        let current = thread::current().id();
        let mut runtime = self.runtime.lock().expect("runtime lock poisoned");
        let handles = [runtime.reader.take(), runtime.stderr.take()];
        drop(runtime);
        for handle in handles.into_iter().flatten() {
            if handle.thread().id() != current {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for ProcessPluginPeer {
    fn drop(&mut self) {
        if self.is_running() {
            if self.shutdown().is_err() {
                self.force_terminate();
            }
        } else if self.shared.state.load(Ordering::Acquire) != STATE_STOPPED {
            self.force_terminate();
        }
    }
}

fn validate_launch(config: &WorkerLaunch) -> Result<(), ProcessPeerError> {
    if config.plugin_id.is_empty()
        || config.package_sha256.is_empty()
        || config.max_in_flight_requests == 0
        || config.max_in_flight_requests > 1024
        || config.request_timeout.is_zero()
        || config.shutdown_timeout.is_zero()
        || config.stderr_limit_bytes > 1024 * 1024
    {
        return Err(ProcessPeerError::InvalidLaunch(
            "identity, request, timeout, or diagnostic limits are invalid".to_owned(),
        ));
    }
    for (key, _) in &config.environment {
        if !valid_environment_name(key) {
            return Err(ProcessPeerError::InvalidLaunch(
                "environment contains an invalid variable name".to_owned(),
            ));
        }
    }
    Ok(())
}

fn valid_environment_name(name: &OsStr) -> bool {
    let text = name.to_string_lossy();
    !text.is_empty()
        && text.len() <= 128
        && !text.contains('=')
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn add_sanitized_environment(command: &mut Command, config: &WorkerLaunch) {
    const WINDOWS_KEYS: &[&str] = &["SYSTEMROOT", "WINDIR", "TEMP", "TMP"];
    const UNIX_KEYS: &[&str] = &["TMPDIR", "LANG", "LC_ALL"];
    let keys = if cfg!(windows) {
        WINDOWS_KEYS
    } else {
        UNIX_KEYS
    };
    for key in keys {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
        .env("MARKION_PLUGIN_ID", &config.plugin_id)
        .env("MARKION_PLUGIN_VERSION", config.plugin_version.to_string())
        .env(
            "MARKION_PLUGIN_PROTOCOL",
            format!("{}.{}", config.protocol.major, config.protocol.minor),
        );
    for (key, value) in &config.environment {
        command.env(key, value);
    }
}

fn spawn_reader(shared: Arc<Shared>, mut stdout: impl Read + Send + 'static) -> JoinHandle<()> {
    thread::Builder::new()
        .name("markion-plugin-reader".to_owned())
        .spawn(move || {
            loop {
                match read_frame(&mut stdout, shared.frame_limits) {
                    Ok(Some(frame)) => {
                        if let Err(failure) = shared.dispatch(frame) {
                            shared.fail(failure);
                            break;
                        }
                    }
                    Ok(None) => {
                        if shared.state.load(Ordering::Acquire) == STATE_RUNNING {
                            shared.fail(WorkerFailure::new(
                                WorkerFailureKind::Exited,
                                "worker closed stdout",
                            ));
                        }
                        break;
                    }
                    Err(error) => {
                        if shared.state.load(Ordering::Acquire) == STATE_RUNNING {
                            shared.fail(WorkerFailure::new(
                                WorkerFailureKind::Protocol,
                                error.to_string(),
                            ));
                        }
                        break;
                    }
                }
            }
        })
        .expect("unable to spawn plugin reader thread")
}

fn spawn_stderr_reader(shared: Arc<Shared>, mut stderr: ChildStderr) -> JoinHandle<()> {
    thread::Builder::new()
        .name("markion-plugin-stderr".to_owned())
        .spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match stderr.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(length) => shared
                        .diagnostics
                        .lock()
                        .expect("diagnostic lock poisoned")
                        .push(&buffer[..length]),
                    Err(_) => break,
                }
            }
        })
        .expect("unable to spawn plugin stderr thread")
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) -> Result<(), ProcessPeerError> {
    use std::os::unix::process::CommandExt;

    // SAFETY: setpgid is async-signal-safe and touches no memory shared with
    // the parent. It runs after fork and before exec to isolate the worker tree.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        });
    }
    Ok(())
}

#[cfg(windows)]
fn configure_process_group(command: &mut Command) -> Result<(), ProcessPeerError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    command.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP);
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn configure_process_group(_: &mut Command) -> Result<(), ProcessPeerError> {
    Ok(())
}

#[cfg(unix)]
struct ProcessTree {
    process_group: i32,
}

#[cfg(unix)]
impl ProcessTree {
    fn attach(child: &Child) -> Result<Self, ProcessPeerError> {
        let process_group = i32::try_from(child.id()).map_err(|_| {
            ProcessPeerError::ProcessSetup("worker process id overflowed i32".to_owned())
        })?;
        Ok(Self { process_group })
    }

    fn terminate(&self) -> Result<(), ProcessPeerError> {
        // SAFETY: the negative id addresses only the isolated worker process
        // group created in pre_exec. ESRCH means it already exited.
        let result = unsafe { libc::kill(-self.process_group, libc::SIGKILL) };
        if result == 0 {
            return Ok(());
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            Ok(())
        } else {
            Err(ProcessPeerError::ProcessSetup(error.to_string()))
        }
    }
}

#[cfg(windows)]
struct ProcessTree {
    job: std::os::windows::io::OwnedHandle,
}

#[cfg(windows)]
impl ProcessTree {
    fn attach(child: &Child) -> Result<Self, ProcessPeerError> {
        use std::mem::size_of;
        use std::os::windows::io::{AsRawHandle, FromRawHandle};

        use windows::Win32::{
            Foundation::HANDLE,
            System::JobObjects::{
                AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
                SetInformationJobObject,
            },
        };

        // SAFETY: null security/name creates a private job. Ownership of the
        // returned handle is immediately transferred into OwnedHandle.
        let raw_job = unsafe { CreateJobObjectW(None, None) }
            .map_err(|error| ProcessPeerError::ProcessSetup(error.to_string()))?;
        let job = unsafe { std::os::windows::io::OwnedHandle::from_raw_handle(raw_job.0) };
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let job_handle = HANDLE(job.as_raw_handle());
        // SAFETY: limits points to a properly initialized structure for the
        // selected information class and remains alive for the duration call.
        unsafe {
            SetInformationJobObject(
                job_handle,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                u32::try_from(size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>())
                    .expect("job information structure fits u32"),
            )
            .and_then(|()| AssignProcessToJobObject(job_handle, HANDLE(child.as_raw_handle())))
        }
        .map_err(|error| ProcessPeerError::ProcessSetup(error.to_string()))?;
        Ok(Self { job })
    }

    fn terminate(&self) -> Result<(), ProcessPeerError> {
        use std::os::windows::io::AsRawHandle;

        use windows::Win32::{Foundation::HANDLE, System::JobObjects::TerminateJobObject};

        // SAFETY: the owned job handle remains valid for this call.
        unsafe { TerminateJobObject(HANDLE(self.job.as_raw_handle()), 1) }
            .map_err(|error| ProcessPeerError::ProcessSetup(error.to_string()))
    }
}

#[cfg(not(any(unix, windows)))]
struct ProcessTree;

#[cfg(not(any(unix, windows)))]
impl ProcessTree {
    fn attach(_: &Child) -> Result<Self, ProcessPeerError> {
        Ok(Self)
    }

    fn terminate(&self) -> Result<(), ProcessPeerError> {
        Ok(())
    }
}
