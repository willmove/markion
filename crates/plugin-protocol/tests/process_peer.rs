use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use markion_plugin_protocol::{
    CapabilityDeclaration, HostMessage, PAGED_DOCUMENT_CAPABILITY, PluginMessage, ProcessPeerError,
    ProcessPluginPeer, ResourceLimits, WorkerFailureKind, WorkerLaunch,
};
use tempfile::TempDir;

struct StagedWorker {
    _temporary: TempDir,
    executable: PathBuf,
    version_root: PathBuf,
    scratch: PathBuf,
}

impl StagedWorker {
    fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let version_root = temporary.path().join("插件 worker space");
        let bin = version_root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let source = PathBuf::from(env!("CARGO_BIN_EXE_plugin-fixture-worker"));
        let executable = bin.join(source.file_name().unwrap());
        fs::copy(&source, &executable).unwrap();
        make_executable(&executable);
        let scratch = temporary.path().join("scratch");
        fs::create_dir_all(&scratch).unwrap();
        Self {
            _temporary: temporary,
            executable,
            version_root,
            scratch,
        }
    }

    fn launch(&self, mode: Option<&str>, generation: u64) -> ProcessPluginPeer {
        let mut config = WorkerLaunch::fixture(&self.executable, &self.version_root);
        config.request_timeout = Duration::from_secs(3);
        config.shutdown_timeout = Duration::from_secs(2);
        if let Some(mode) = mode {
            config.environment.push((
                OsString::from("MARKION_PLUGIN_FIXTURE_MODE"),
                OsString::from(mode),
            ));
        }
        config.environment.push((
            OsString::from("MARKION_PLUGIN_FIXTURE_SCRATCH"),
            self.scratch.as_os_str().to_owned(),
        ));
        ProcessPluginPeer::launch(config, generation).unwrap()
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_: &Path) {}

#[test]
fn real_worker_handshake_ping_and_clean_shutdown() {
    let staged = StagedWorker::new();
    let peer = staged.launch(None, 7);
    let response = peer.request(HostMessage::Ping, Vec::new()).unwrap();
    assert_eq!(response.generation, 7);
    assert_eq!(response.message, PluginMessage::Pong);
    assert!(peer.diagnostics().bytes.is_empty());
    peer.shutdown().unwrap();
    assert!(!peer.is_running());
}

#[test]
fn packaged_fixture_capabilities_are_confirmed_during_handshake() {
    let staged = StagedWorker::new();
    let capability = CapabilityDeclaration {
        id: PAGED_DOCUMENT_CAPABILITY.to_owned(),
        limits: ResourceLimits::default(),
    };
    let mut config = WorkerLaunch::fixture(&staged.executable, &staged.version_root);
    config.requested_capabilities = vec![capability.id.clone()];
    config.expected_capabilities = vec![capability];
    let peer = ProcessPluginPeer::launch(config, 8).unwrap();
    peer.shutdown().unwrap();
}

#[test]
fn timeout_cancels_and_backpressure_is_bounded() {
    let staged = StagedWorker::new();
    let mut config = WorkerLaunch::fixture(&staged.executable, &staged.version_root);
    config.max_in_flight_requests = 1;
    config.request_timeout = Duration::from_secs(3);
    config.environment.push((
        OsString::from("MARKION_PLUGIN_FIXTURE_MODE"),
        OsString::from("hang-first-ping"),
    ));
    let peer = ProcessPluginPeer::launch(config, 11).unwrap();
    let pending = peer.begin_request(HostMessage::Ping, Vec::new()).unwrap();
    assert!(matches!(
        peer.begin_request(HostMessage::Ping, Vec::new()),
        Err(ProcessPeerError::Backpressure)
    ));
    assert!(matches!(
        pending.wait(Duration::from_millis(75)),
        Err(ProcessPeerError::Timeout)
    ));
    thread::sleep(Duration::from_millis(2100));
    assert_eq!(
        peer.request(HostMessage::Ping, Vec::new()).unwrap().message,
        PluginMessage::Pong
    );
    peer.shutdown().unwrap();
}

#[test]
fn malformed_output_fails_closed_and_stderr_is_bounded() {
    let staged = StagedWorker::new();
    let mut config = WorkerLaunch::fixture(&staged.executable, &staged.version_root);
    config.stderr_limit_bytes = 1024;
    config.environment.push((
        OsString::from("MARKION_PLUGIN_FIXTURE_MODE"),
        OsString::from("malformed-ping"),
    ));
    let peer = ProcessPluginPeer::launch(config, 13).unwrap();
    let error = peer.request(HostMessage::Ping, Vec::new()).unwrap_err();
    assert!(matches!(
        error,
        ProcessPeerError::Worker(ref failure)
            if failure.kind == WorkerFailureKind::Protocol
    ));
    peer.force_terminate();
    let diagnostics = peer.diagnostics();
    assert_eq!(diagnostics.bytes.len(), 1024);
    assert!(diagnostics.truncated);
    assert!(diagnostics.total_bytes >= 8192);
}

#[test]
fn incompatible_startup_is_rejected() {
    let staged = StagedWorker::new();
    let mut config = WorkerLaunch::fixture(&staged.executable, &staged.version_root);
    config.environment.push((
        OsString::from("MARKION_PLUGIN_FIXTURE_MODE"),
        OsString::from("bad-handshake"),
    ));
    assert!(matches!(
        ProcessPluginPeer::launch(config, 17),
        Err(ProcessPeerError::IncompatibleHandshake)
    ));
}

#[test]
fn forced_shutdown_terminates_the_worker_process_tree() {
    let staged = StagedWorker::new();
    let peer = staged.launch(Some("spawn-child-ping"), 19);
    assert_eq!(
        peer.request(HostMessage::Ping, Vec::new()).unwrap().message,
        PluginMessage::Pong
    );
    let heartbeat = staged.scratch.join("heartbeat");
    wait_for_len(&heartbeat, 3, Duration::from_secs(3));
    peer.force_terminate();
    let stopped_at = fs::metadata(&heartbeat).unwrap().len();
    thread::sleep(Duration::from_millis(250));
    assert_eq!(fs::metadata(&heartbeat).unwrap().len(), stopped_at);
}

#[test]
fn restart_generations_are_explicit_and_do_not_alias() {
    let staged = StagedWorker::new();
    let first = staged.launch(None, 29);
    assert_eq!(
        first
            .request(HostMessage::Ping, Vec::new())
            .unwrap()
            .generation,
        29
    );
    first.shutdown().unwrap();
    let second = staged.launch(None, 30);
    assert_eq!(
        second
            .request(HostMessage::Ping, Vec::new())
            .unwrap()
            .generation,
        30
    );
    second.shutdown().unwrap();
}

fn wait_for_len(path: &Path, minimum: u64, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if fs::metadata(path).is_ok_and(|metadata| metadata.len() >= minimum) {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("{} did not reach {minimum} bytes", path.display());
}
