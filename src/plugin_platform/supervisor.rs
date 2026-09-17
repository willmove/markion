use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};

use markion_plugin_protocol::{
    HostMessage, PendingProcessRequest, ProcessPeerError, ProcessPluginPeer, ProcessResponse,
    WorkerFailureKind, WorkerLaunch,
};
use semver::Version;
use thiserror::Error;

use super::{PluginStore, PluginStoreError, QuarantineReason};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginSupervisorPolicy {
    pub failure_limit: usize,
    pub failure_window: Duration,
}

impl Default for PluginSupervisorPolicy {
    fn default() -> Self {
        Self {
            failure_limit: 3,
            failure_window: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Error)]
pub enum PluginSupervisorError {
    #[error("plugin store operation failed")]
    Store(#[from] PluginStoreError),
    #[error("plugin has no enabled active version")]
    Unavailable,
    #[error("plugin version is quarantined")]
    Quarantined,
    #[error("worker launch does not match the verified active version")]
    IdentityMismatch,
    #[error("plugin capability is not declared: {0}")]
    CapabilityUnavailable(String),
    #[error("plugin worker operation failed")]
    Peer(#[from] ProcessPeerError),
    #[error("plugin response belongs to a stale worker generation")]
    StaleGeneration,
}

struct ActiveSession {
    version: Version,
    generation: u64,
    capabilities: Vec<String>,
    peer: Arc<ProcessPluginPeer>,
}

struct FailureWindow {
    observations: VecDeque<Instant>,
}

struct SupervisorInner {
    store: PluginStore,
    policy: PluginSupervisorPolicy,
    lifecycle: Mutex<()>,
    sessions: Mutex<BTreeMap<String, ActiveSession>>,
    generations: Mutex<BTreeMap<String, u64>>,
    failures: Mutex<BTreeMap<(String, Version), FailureWindow>>,
}

#[derive(Clone)]
pub struct PluginSupervisor {
    inner: Arc<SupervisorInner>,
}

impl PluginSupervisor {
    pub fn new(store: PluginStore, policy: PluginSupervisorPolicy) -> Self {
        Self {
            inner: Arc::new(SupervisorInner {
                store,
                policy,
                lifecycle: Mutex::new(()),
                sessions: Mutex::new(BTreeMap::new()),
                generations: Mutex::new(BTreeMap::new()),
                failures: Mutex::new(BTreeMap::new()),
            }),
        }
    }

    pub fn store(&self) -> &PluginStore {
        &self.inner.store
    }

    pub fn start_process(
        &self,
        launch: WorkerLaunch,
    ) -> Result<PluginSession, PluginSupervisorError> {
        let _lifecycle = self
            .inner
            .lifecycle
            .lock()
            .expect("plugin lifecycle lock poisoned");
        let state = self.inner.store.state(&launch.plugin_id)?;
        let active = state
            .active
            .as_ref()
            .ok_or(PluginSupervisorError::Unavailable)?;
        if !state.enabled {
            return Err(PluginSupervisorError::Unavailable);
        }
        if state.quarantine.contains_key(&active.version) {
            return Err(PluginSupervisorError::Quarantined);
        }
        let expected_root = self
            .inner
            .store
            .paths()
            .version_root(&launch.plugin_id, &active.version)?;
        if active.version != launch.plugin_version
            || active.package_sha256 != launch.package_sha256
            || canonical_or_original(&expected_root)
                != canonical_or_original(&launch.working_directory)
        {
            return Err(PluginSupervisorError::IdentityMismatch);
        }

        if let Some(previous) = self
            .inner
            .sessions
            .lock()
            .expect("plugin sessions lock poisoned")
            .remove(&launch.plugin_id)
        {
            stop_peer(&previous.peer);
        }
        let generation = {
            let mut generations = self
                .inner
                .generations
                .lock()
                .expect("plugin generations lock poisoned");
            let generation = generations.entry(launch.plugin_id.clone()).or_insert(0);
            *generation = generation.saturating_add(1).max(1);
            *generation
        };
        let plugin_id = launch.plugin_id.clone();
        let version = launch.plugin_version.clone();
        let capabilities = launch.requested_capabilities.clone();
        let peer = match ProcessPluginPeer::launch(launch, generation) {
            Ok(peer) => Arc::new(peer),
            Err(error) => {
                let _ = self.inner.store.quarantine(
                    &plugin_id,
                    &version,
                    QuarantineReason::StartupFailed,
                );
                return Err(error.into());
            }
        };
        self.inner
            .sessions
            .lock()
            .expect("plugin sessions lock poisoned")
            .insert(
                plugin_id.clone(),
                ActiveSession {
                    version: version.clone(),
                    generation,
                    capabilities,
                    peer: Arc::clone(&peer),
                },
            );
        Ok(PluginSession {
            plugin_id,
            version,
            generation,
            peer,
            supervisor: Arc::downgrade(&self.inner),
        })
    }

    pub fn acquire(
        &self,
        plugin_id: &str,
        capability: &str,
    ) -> Result<PluginSession, PluginSupervisorError> {
        let sessions = self
            .inner
            .sessions
            .lock()
            .expect("plugin sessions lock poisoned");
        let active = sessions
            .get(plugin_id)
            .ok_or(PluginSupervisorError::Unavailable)?;
        if !active
            .capabilities
            .iter()
            .any(|available| available == capability)
        {
            return Err(PluginSupervisorError::CapabilityUnavailable(
                capability.to_owned(),
            ));
        }
        Ok(PluginSession {
            plugin_id: plugin_id.to_owned(),
            version: active.version.clone(),
            generation: active.generation,
            peer: Arc::clone(&active.peer),
            supervisor: Arc::downgrade(&self.inner),
        })
    }

    pub fn stop(&self, plugin_id: &str) {
        let _lifecycle = self
            .inner
            .lifecycle
            .lock()
            .expect("plugin lifecycle lock poisoned");
        if let Some(session) = self
            .inner
            .sessions
            .lock()
            .expect("plugin sessions lock poisoned")
            .remove(plugin_id)
        {
            stop_peer(&session.peer);
        }
        self.bump_generation(plugin_id);
    }

    pub fn disable(&self, plugin_id: &str) -> Result<(), PluginSupervisorError> {
        self.stop(plugin_id);
        self.inner.store.set_enabled(plugin_id, false)?;
        Ok(())
    }

    pub fn shutdown_all(&self) {
        let _lifecycle = self
            .inner
            .lifecycle
            .lock()
            .expect("plugin lifecycle lock poisoned");
        let sessions = std::mem::take(
            &mut *self
                .inner
                .sessions
                .lock()
                .expect("plugin sessions lock poisoned"),
        );
        for (plugin_id, session) in sessions {
            stop_peer(&session.peer);
            self.bump_generation(&plugin_id);
        }
    }

    pub fn generation(&self, plugin_id: &str) -> u64 {
        self.inner
            .generations
            .lock()
            .expect("plugin generations lock poisoned")
            .get(plugin_id)
            .copied()
            .unwrap_or(0)
    }

    fn bump_generation(&self, plugin_id: &str) {
        let mut generations = self
            .inner
            .generations
            .lock()
            .expect("plugin generations lock poisoned");
        let generation = generations.entry(plugin_id.to_owned()).or_insert(0);
        *generation = generation.saturating_add(1).max(1);
    }

    #[cfg(test)]
    fn observe_failure_for_test(
        &self,
        plugin_id: &str,
        version: &Version,
        reason: QuarantineReason,
        immediate: bool,
    ) -> Result<bool, PluginSupervisorError> {
        self.inner
            .observe_failure(plugin_id, version, reason, immediate)
    }
}

impl Drop for PluginSupervisor {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            self.shutdown_all();
        }
    }
}

#[derive(Clone)]
pub struct PluginSession {
    plugin_id: String,
    version: Version,
    generation: u64,
    peer: Arc<ProcessPluginPeer>,
    supervisor: Weak<SupervisorInner>,
}

impl PluginSession {
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn begin_request(
        &self,
        message: HostMessage,
        body: Vec<u8>,
    ) -> Result<SupervisedRequest, PluginSupervisorError> {
        self.ensure_current()?;
        let pending = self.peer.begin_request(message, body)?;
        Ok(SupervisedRequest {
            pending: Some(pending),
            plugin_id: self.plugin_id.clone(),
            version: self.version.clone(),
            generation: self.generation,
            supervisor: self.supervisor.clone(),
        })
    }

    pub fn request(
        &self,
        message: HostMessage,
        body: Vec<u8>,
        timeout: Duration,
    ) -> Result<ProcessResponse, PluginSupervisorError> {
        self.begin_request(message, body)?.wait(timeout)
    }

    pub fn diagnostics(&self) -> markion_plugin_protocol::DiagnosticSnapshot {
        self.peer.diagnostics()
    }

    fn ensure_current(&self) -> Result<(), PluginSupervisorError> {
        let supervisor = self
            .supervisor
            .upgrade()
            .ok_or(PluginSupervisorError::Unavailable)?;
        let current = supervisor
            .generations
            .lock()
            .expect("plugin generations lock poisoned")
            .get(&self.plugin_id)
            .copied()
            .unwrap_or(0);
        if current != self.generation {
            return Err(PluginSupervisorError::StaleGeneration);
        }
        Ok(())
    }
}

pub struct SupervisedRequest {
    pending: Option<PendingProcessRequest>,
    plugin_id: String,
    version: Version,
    generation: u64,
    supervisor: Weak<SupervisorInner>,
}

impl SupervisedRequest {
    pub fn request_id(&self) -> u64 {
        self.pending
            .as_ref()
            .map(PendingProcessRequest::request_id)
            .unwrap_or(0)
    }

    pub fn cancel(mut self) -> Result<(), PluginSupervisorError> {
        if let Some(pending) = self.pending.take() {
            pending.cancel()?;
        }
        Ok(())
    }

    pub fn wait(mut self, timeout: Duration) -> Result<ProcessResponse, PluginSupervisorError> {
        let pending = self.pending.take().expect("supervised request consumed");
        match pending.wait(timeout) {
            Ok(response) if response.generation == self.generation => Ok(response),
            Ok(_) => Err(PluginSupervisorError::StaleGeneration),
            Err(error) => {
                self.record_failure(&error);
                Err(error.into())
            }
        }
    }

    fn record_failure(&self, error: &ProcessPeerError) {
        let Some(supervisor) = self.supervisor.upgrade() else {
            return;
        };
        let (reason, immediate) = match error {
            ProcessPeerError::Timeout => (QuarantineReason::Timeout, false),
            ProcessPeerError::IncompatibleHandshake => (QuarantineReason::StartupFailed, true),
            ProcessPeerError::Worker(failure) => match failure.kind {
                WorkerFailureKind::Protocol => (QuarantineReason::ProtocolViolation, true),
                WorkerFailureKind::Startup => (QuarantineReason::StartupFailed, true),
                WorkerFailureKind::Exited | WorkerFailureKind::Io => {
                    (QuarantineReason::CrashLoop, false)
                }
                WorkerFailureKind::ForcedShutdown => return,
            },
            _ => return,
        };
        let _ = supervisor.observe_failure(&self.plugin_id, &self.version, reason, immediate);
    }
}

impl SupervisorInner {
    fn observe_failure(
        &self,
        plugin_id: &str,
        version: &Version,
        reason: QuarantineReason,
        immediate: bool,
    ) -> Result<bool, PluginSupervisorError> {
        let now = Instant::now();
        let should_quarantine = if immediate {
            true
        } else {
            let mut failures = self.failures.lock().expect("failure window lock poisoned");
            let window = failures
                .entry((plugin_id.to_owned(), version.clone()))
                .or_insert_with(|| FailureWindow {
                    observations: VecDeque::new(),
                });
            while window.observations.front().is_some_and(|observed| {
                now.saturating_duration_since(*observed) > self.policy.failure_window
            }) {
                window.observations.pop_front();
            }
            window.observations.push_back(now);
            window.observations.len() >= self.policy.failure_limit
        };
        if !should_quarantine {
            return Ok(false);
        }
        self.store.quarantine(plugin_id, version, reason)?;
        if let Some(active) = self
            .sessions
            .lock()
            .expect("plugin sessions lock poisoned")
            .remove(plugin_id)
        {
            stop_peer(&active.peer);
        }
        let mut generations = self
            .generations
            .lock()
            .expect("plugin generations lock poisoned");
        let generation = generations.entry(plugin_id.to_owned()).or_insert(0);
        *generation = generation.saturating_add(1).max(1);
        Ok(true)
    }
}

fn stop_peer(peer: &ProcessPluginPeer) {
    if peer.shutdown().is_err() {
        peer.force_terminate();
    }
}

fn canonical_or_original(path: &std::path::Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, io::Cursor};

    use markion_plugin_protocol::{
        CapabilityDeclaration, LocalizedIdentity, MemberManifest, PackageLimits, PluginManifest,
        ProtocolRange, ProtocolVersion, ResourceLimits, TargetSpec, canonical_json,
        inspect_package_with,
    };
    use semver::VersionReq;
    use sha2::{Digest, Sha256};
    use zip::{CompressionMethod, ZipWriter, write::FileOptions};

    use super::*;
    use crate::plugin_platform::PluginStorePaths;

    fn installed_supervisor() -> (tempfile::TempDir, PluginSupervisor, Version) {
        let root = tempfile::tempdir().unwrap();
        let store = PluginStore::new(PluginStorePaths::new(root.path().join("plugins")));
        let version = Version::new(1, 0, 0);
        let worker = b"worker";
        let manifest = PluginManifest {
            schema_version: 1,
            plugin_id: "dev.markion.fixture".to_owned(),
            version: version.clone(),
            publisher: "Markion".to_owned(),
            host_version: VersionReq::parse(">=0.3.9, <0.4.0").unwrap(),
            protocol: ProtocolRange::V1,
            target: TargetSpec::current(),
            entry_point: "bin/worker".to_owned(),
            identities: BTreeMap::from([(
                "en".to_owned(),
                LocalizedIdentity {
                    name: "Fixture".to_owned(),
                    description: "Fixture".to_owned(),
                },
            )]),
            permissions: Vec::new(),
            capabilities: vec![CapabilityDeclaration {
                id: "paged-document/v1".to_owned(),
                limits: ResourceLimits::default(),
            }],
            file_handlers: Vec::new(),
            archive_size_bytes: 4096,
            installed_size_bytes: worker.len() as u64,
            members: vec![MemberManifest {
                path: "bin/worker".to_owned(),
                length: worker.len() as u64,
                sha256: format!("{:x}", Sha256::digest(worker)),
                executable: true,
            }],
        };
        let manifest_bytes = canonical_json(&manifest).unwrap();
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
            for (path, bytes) in [
                ("plugin.json", manifest_bytes.as_slice()),
                ("plugin.json.minisig", b"signature".as_slice()),
                ("bin/worker", worker.as_slice()),
            ] {
                zip.start_file(path, options).unwrap();
                std::io::Write::write_all(&mut zip, bytes).unwrap();
            }
            zip.finish().unwrap();
        }
        let package = inspect_package_with(
            &cursor.into_inner(),
            |_, _| Ok(()),
            &Version::new(0, 3, 9),
            ProtocolVersion::V1_0,
            &TargetSpec::current(),
            &["en"],
            PackageLimits::default(),
        )
        .unwrap();
        store.install_verified(&package, |_, _| Ok(())).unwrap();
        let supervisor = PluginSupervisor::new(
            store,
            PluginSupervisorPolicy {
                failure_limit: 3,
                failure_window: Duration::from_secs(60),
            },
        );
        (root, supervisor, version)
    }

    #[test]
    fn repeated_failures_quarantine_only_the_active_version() {
        let (_root, supervisor, version) = installed_supervisor();
        for observation in 1..=3 {
            let quarantined = supervisor
                .observe_failure_for_test(
                    "dev.markion.fixture",
                    &version,
                    QuarantineReason::CrashLoop,
                    false,
                )
                .unwrap();
            assert_eq!(quarantined, observation == 3);
        }
        let state = supervisor.store().state("dev.markion.fixture").unwrap();
        assert!(!state.enabled);
        assert_eq!(
            state.quarantine.get(&version).unwrap().reason,
            QuarantineReason::CrashLoop
        );
    }

    #[test]
    fn protocol_failure_quarantines_immediately() {
        let (_root, supervisor, version) = installed_supervisor();
        assert!(
            supervisor
                .observe_failure_for_test(
                    "dev.markion.fixture",
                    &version,
                    QuarantineReason::ProtocolViolation,
                    true,
                )
                .unwrap()
        );
        assert!(
            supervisor
                .store()
                .state("dev.markion.fixture")
                .unwrap()
                .quarantine
                .contains_key(&version)
        );
    }
}
