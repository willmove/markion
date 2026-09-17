use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use markion_plugin_protocol::{CatalogArtifact, PluginManifest, VerifiedPackage};
use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{default_plugin_data_dir, storage::atomic_write};

const STORE_SCHEMA_VERSION: u16 = 1;
static NEXT_STAGING_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginStorePaths {
    root: PathBuf,
}

impl Default for PluginStorePaths {
    fn default() -> Self {
        Self::new(default_plugin_data_dir())
    }
}

impl PluginStorePaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn staging_root(&self) -> PathBuf {
        self.root.join("staging")
    }

    /// Atomically replaced cache containing both the canonical catalog and
    /// its detached signature. Keeping the pair in one file prevents a crash
    /// between two independent replacements from discarding the last verified
    /// remote snapshot.
    pub fn catalog_cache_path(&self) -> PathBuf {
        self.root.join("catalog.cache")
    }

    pub fn plugin_root(&self, plugin_id: &str) -> Result<PathBuf, PluginStoreError> {
        validate_plugin_id(plugin_id)?;
        Ok(self.root.join(plugin_id))
    }

    pub fn versions_root(&self, plugin_id: &str) -> Result<PathBuf, PluginStoreError> {
        Ok(self.plugin_root(plugin_id)?.join("versions"))
    }

    pub fn version_root(
        &self,
        plugin_id: &str,
        version: &Version,
    ) -> Result<PathBuf, PluginStoreError> {
        Ok(self.versions_root(plugin_id)?.join(version.to_string()))
    }

    pub fn active_state_path(&self, plugin_id: &str) -> Result<PathBuf, PluginStoreError> {
        Ok(self.plugin_root(plugin_id)?.join("active.json"))
    }

    pub fn config_root(&self, plugin_id: &str) -> Result<PathBuf, PluginStoreError> {
        Ok(self.plugin_root(plugin_id)?.join("config"))
    }

    fn new_staging_root(&self, plugin_id: &str) -> Result<PathBuf, PluginStoreError> {
        validate_plugin_id(plugin_id)?;
        let id = NEXT_STAGING_ID.fetch_add(1, Ordering::Relaxed);
        Ok(self
            .staging_root()
            .join(format!("{plugin_id}-{}-{id}", std::process::id())))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivationRecord {
    pub version: Version,
    pub package_sha256: String,
    pub archive_bytes: u64,
    pub installed_bytes: u64,
    pub activated_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuarantineReason {
    HealthCheckFailed,
    StartupFailed,
    CrashLoop,
    ProtocolViolation,
    Timeout,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginQuarantine {
    pub version: Version,
    pub reason: QuarantineReason,
    pub observed_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginStoreState {
    pub schema_version: u16,
    pub plugin_id: String,
    pub enabled: bool,
    pub active: Option<ActivationRecord>,
    pub rollback: Option<ActivationRecord>,
    pub quarantine: BTreeMap<Version, PluginQuarantine>,
}

impl PluginStoreState {
    fn empty(plugin_id: &str) -> Self {
        Self {
            schema_version: STORE_SCHEMA_VERSION,
            plugin_id: plugin_id.to_owned(),
            enabled: false,
            active: None,
            rollback: None,
            quarantine: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PluginStorageUsage {
    pub active_bytes: u64,
    pub rollback_bytes: u64,
    pub other_version_bytes: u64,
    pub staging_bytes: u64,
    pub config_bytes: u64,
    pub metadata_bytes: u64,
}

impl PluginStorageUsage {
    pub fn retained_package_bytes(self) -> u64 {
        self.active_bytes
            .saturating_add(self.rollback_bytes)
            .saturating_add(self.other_version_bytes)
    }

    pub fn total_bytes(self) -> u64 {
        self.retained_package_bytes()
            .saturating_add(self.staging_bytes)
            .saturating_add(self.config_bytes)
            .saturating_add(self.metadata_bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginPackageEstimate {
    pub download_bytes: u64,
    pub installed_bytes: u64,
}

impl From<&PluginManifest> for PluginPackageEstimate {
    fn from(manifest: &PluginManifest) -> Self {
        Self {
            download_bytes: manifest.archive_size_bytes,
            installed_bytes: manifest.installed_size_bytes,
        }
    }
}

impl From<&CatalogArtifact> for PluginPackageEstimate {
    fn from(artifact: &CatalogArtifact) -> Self {
        Self {
            download_bytes: artifact.length,
            installed_bytes: artifact.installed_size_bytes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallOutcome {
    pub active: ActivationRecord,
    pub rollback: Option<ActivationRecord>,
    pub usage: PluginStorageUsage,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecoveryReport {
    pub staging_directories_removed: usize,
    pub orphan_versions_removed: usize,
    pub rollback_versions_promoted: usize,
}

#[derive(Clone, Debug)]
pub struct PluginStore {
    paths: PluginStorePaths,
}

impl Default for PluginStore {
    fn default() -> Self {
        Self::new(PluginStorePaths::default())
    }
}

impl PluginStore {
    pub fn new(paths: PluginStorePaths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &PluginStorePaths {
        &self.paths
    }

    pub fn initialize(&self) -> Result<(), PluginStoreError> {
        fs::create_dir_all(self.paths.staging_root())?;
        Ok(())
    }

    pub fn state(&self, plugin_id: &str) -> Result<PluginStoreState, PluginStoreError> {
        let path = self.paths.active_state_path(plugin_id)?;
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PluginStoreState::empty(plugin_id));
            }
            Err(error) => return Err(error.into()),
        };
        let state: PluginStoreState = serde_json::from_slice(&bytes)?;
        validate_state(plugin_id, &state)?;
        Ok(state)
    }

    pub fn install_verified<F>(
        &self,
        package: &VerifiedPackage,
        health_check: F,
    ) -> Result<InstallOutcome, PluginStoreError>
    where
        F: FnOnce(&Path, &PluginManifest) -> Result<(), PluginStoreError>,
    {
        self.initialize()?;
        let plugin_id = package.manifest.plugin_id.as_str();
        validate_plugin_id(plugin_id)?;
        let staging_root = self.paths.new_staging_root(plugin_id)?;
        let staged_payload = staging_root.join("payload");
        fs::create_dir(&staging_root)?;

        let result = (|| {
            package.extract_to(&staged_payload)?;
            let entry_point = staged_payload.join(&package.manifest.entry_point);
            health_check(&entry_point, &package.manifest)?;

            let version_root = self
                .paths
                .version_root(plugin_id, &package.manifest.version)?;
            if version_root.exists() {
                return Err(PluginStoreError::VersionAlreadyInstalled(
                    package.manifest.version.clone(),
                ));
            }
            let versions_root = self.paths.versions_root(plugin_id)?;
            fs::create_dir_all(&versions_root)?;
            fs::rename(&staged_payload, &version_root)?;

            let mut state = self.state(plugin_id)?;
            let activation = ActivationRecord {
                version: package.manifest.version.clone(),
                package_sha256: package.archive_sha256.clone(),
                archive_bytes: package.archive_size_bytes,
                installed_bytes: package.installed_size_bytes,
                activated_unix_ms: unix_ms(),
            };
            if state.active.as_ref().map(|record| &record.version) != Some(&activation.version) {
                state.rollback = state.active.take();
            }
            state.active = Some(activation.clone());
            state.enabled = true;
            state.quarantine.remove(&activation.version);
            self.write_state(&state)?;
            self.prune_versions(&state)?;
            Ok(InstallOutcome {
                active: activation,
                rollback: state.rollback,
                usage: self.usage(plugin_id)?,
            })
        })();

        if staging_root.exists() {
            let _ = fs::remove_dir_all(&staging_root);
        }
        result
    }

    pub fn set_enabled(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<PluginStoreState, PluginStoreError> {
        let mut state = self.state(plugin_id)?;
        if enabled && state.active.is_none() {
            return Err(PluginStoreError::NoActiveVersion);
        }
        if enabled
            && state
                .active
                .as_ref()
                .is_some_and(|active| state.quarantine.contains_key(&active.version))
        {
            return Err(PluginStoreError::VersionQuarantined);
        }
        state.enabled = enabled;
        self.write_state(&state)?;
        Ok(state)
    }

    pub fn rollback(&self, plugin_id: &str) -> Result<PluginStoreState, PluginStoreError> {
        let mut state = self.state(plugin_id)?;
        let rollback = state
            .rollback
            .take()
            .ok_or(PluginStoreError::NoRollbackVersion)?;
        if state.quarantine.contains_key(&rollback.version) {
            return Err(PluginStoreError::VersionQuarantined);
        }
        let active = state.active.replace(rollback);
        state.rollback = active;
        state.enabled = true;
        self.write_state(&state)?;
        Ok(state)
    }

    pub fn quarantine(
        &self,
        plugin_id: &str,
        version: &Version,
        reason: QuarantineReason,
    ) -> Result<PluginStoreState, PluginStoreError> {
        let mut state = self.state(plugin_id)?;
        state.quarantine.insert(
            version.clone(),
            PluginQuarantine {
                version: version.clone(),
                reason,
                observed_unix_ms: unix_ms(),
            },
        );
        if state.active.as_ref().map(|active| &active.version) == Some(version) {
            state.enabled = false;
        }
        self.write_state(&state)?;
        Ok(state)
    }

    pub fn uninstall(&self, plugin_id: &str, remove_config: bool) -> Result<(), PluginStoreError> {
        let plugin_root = self.paths.plugin_root(plugin_id)?;
        if !plugin_root.exists() {
            return Ok(());
        }
        if remove_config {
            remove_tree_beneath(self.paths.root(), &plugin_root)?;
            return Ok(());
        }

        let config_root = self.paths.config_root(plugin_id)?;
        let preserved_config = if config_root.is_dir() {
            let temp = self.paths.staging_root().join(format!(
                "preserve-config-{plugin_id}-{}",
                NEXT_STAGING_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(self.paths.staging_root())?;
            fs::rename(&config_root, &temp)?;
            Some(temp)
        } else {
            None
        };
        remove_tree_beneath(self.paths.root(), &plugin_root)?;
        if let Some(temp) = preserved_config {
            fs::create_dir_all(&plugin_root)?;
            fs::rename(temp, config_root)?;
        }
        Ok(())
    }

    pub fn usage(&self, plugin_id: &str) -> Result<PluginStorageUsage, PluginStoreError> {
        let state = self.state(plugin_id)?;
        let mut usage = PluginStorageUsage::default();
        let versions_root = self.paths.versions_root(plugin_id)?;
        if versions_root.is_dir() {
            for entry in fs::read_dir(&versions_root)? {
                let entry = entry?;
                if !entry.file_type()?.is_dir() {
                    continue;
                }
                let version = entry.file_name().to_string_lossy().parse::<Version>().ok();
                let bytes = tree_bytes(&entry.path())?;
                match version.as_ref() {
                    Some(version)
                        if state.active.as_ref().map(|record| &record.version) == Some(version) =>
                    {
                        usage.active_bytes = usage.active_bytes.saturating_add(bytes);
                    }
                    Some(version)
                        if state.rollback.as_ref().map(|record| &record.version)
                            == Some(version) =>
                    {
                        usage.rollback_bytes = usage.rollback_bytes.saturating_add(bytes);
                    }
                    _ => {
                        usage.other_version_bytes = usage.other_version_bytes.saturating_add(bytes);
                    }
                }
            }
        }
        let prefix = format!("{plugin_id}-");
        if self.paths.staging_root().is_dir() {
            for entry in fs::read_dir(self.paths.staging_root())? {
                let entry = entry?;
                if entry.file_name().to_string_lossy().starts_with(&prefix) {
                    usage.staging_bytes = usage
                        .staging_bytes
                        .saturating_add(tree_bytes(&entry.path())?);
                }
            }
        }
        usage.config_bytes = tree_bytes_if_exists(&self.paths.config_root(plugin_id)?)?;
        usage.metadata_bytes = fs::metadata(self.paths.active_state_path(plugin_id)?)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        Ok(usage)
    }

    pub fn recover(&self) -> Result<RecoveryReport, PluginStoreError> {
        self.initialize()?;
        let mut report = RecoveryReport::default();
        for entry in fs::read_dir(self.paths.staging_root())? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                remove_tree_beneath(self.paths.root(), &entry.path())?;
                report.staging_directories_removed += 1;
            } else {
                fs::remove_file(entry.path())?;
            }
        }

        for entry in fs::read_dir(self.paths.root())? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() || entry.file_name() == "staging" {
                continue;
            }
            let plugin_id = entry.file_name().to_string_lossy().into_owned();
            if validate_plugin_id(&plugin_id).is_err() {
                continue;
            }
            let mut state = self.state(&plugin_id)?;
            if let Some(active) = state.active.as_ref()
                && !self
                    .paths
                    .version_root(&plugin_id, &active.version)?
                    .is_dir()
            {
                state.active = None;
                state.enabled = false;
            }
            if state.active.is_none()
                && let Some(rollback) = state.rollback.take()
                && self
                    .paths
                    .version_root(&plugin_id, &rollback.version)?
                    .is_dir()
            {
                state.active = Some(rollback);
                state.enabled = true;
                report.rollback_versions_promoted += 1;
            }
            self.write_state(&state)?;
            report.orphan_versions_removed += self.prune_versions(&state)?;
        }
        Ok(report)
    }

    fn write_state(&self, state: &PluginStoreState) -> Result<(), PluginStoreError> {
        validate_state(&state.plugin_id, state)?;
        let bytes = serde_json::to_vec_pretty(state)?;
        atomic_write(self.paths.active_state_path(&state.plugin_id)?, bytes)?;
        Ok(())
    }

    fn prune_versions(&self, state: &PluginStoreState) -> Result<usize, PluginStoreError> {
        let root = self.paths.versions_root(&state.plugin_id)?;
        if !root.is_dir() {
            return Ok(0);
        }
        let retained = state
            .active
            .iter()
            .chain(state.rollback.iter())
            .map(|record| record.version.to_string())
            .collect::<BTreeSet<_>>();
        let mut removed = 0;
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && !retained.contains(entry.file_name().to_string_lossy().as_ref())
            {
                remove_tree_beneath(self.paths.root(), &entry.path())?;
                removed += 1;
            }
        }
        Ok(removed)
    }
}

#[derive(Debug, Error)]
pub enum PluginStoreError {
    #[error("plugin store I/O failure")]
    Io(#[from] io::Error),
    #[error("plugin store JSON is invalid")]
    Json(#[from] serde_json::Error),
    #[error("verified package extraction failed")]
    Package(#[from] markion_plugin_protocol::PackageError),
    #[error("plugin id is invalid: {0}")]
    InvalidPluginId(String),
    #[error("persisted plugin state is invalid")]
    InvalidState,
    #[error("plugin version is already installed: {0}")]
    VersionAlreadyInstalled(Version),
    #[error("plugin has no active version")]
    NoActiveVersion,
    #[error("plugin has no rollback version")]
    NoRollbackVersion,
    #[error("plugin version is quarantined")]
    VersionQuarantined,
    #[error("plugin store path escaped its root")]
    PathEscape,
    #[error("plugin health check failed")]
    HealthCheck,
}

fn validate_plugin_id(plugin_id: &str) -> Result<(), PluginStoreError> {
    let valid = !plugin_id.is_empty()
        && plugin_id.len() <= 128
        && plugin_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
        && plugin_id.as_bytes()[0].is_ascii_alphanumeric()
        && plugin_id.as_bytes()[plugin_id.len() - 1].is_ascii_alphanumeric();
    if valid {
        Ok(())
    } else {
        Err(PluginStoreError::InvalidPluginId(plugin_id.to_owned()))
    }
}

fn validate_state(plugin_id: &str, state: &PluginStoreState) -> Result<(), PluginStoreError> {
    validate_plugin_id(plugin_id)?;
    if state.schema_version != STORE_SCHEMA_VERSION || state.plugin_id != plugin_id {
        return Err(PluginStoreError::InvalidState);
    }
    if state.enabled && state.active.is_none() {
        return Err(PluginStoreError::InvalidState);
    }
    if state.active.as_ref().map(|record| &record.version)
        == state.rollback.as_ref().map(|record| &record.version)
        && state.active.is_some()
    {
        return Err(PluginStoreError::InvalidState);
    }
    Ok(())
}

fn tree_bytes_if_exists(path: &Path) -> Result<u64, io::Error> {
    match fs::symlink_metadata(path) {
        Ok(_) => tree_bytes(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error),
    }
}

fn tree_bytes(path: &Path) -> Result<u64, io::Error> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut total = 0u64;
    for entry in fs::read_dir(path)? {
        total = total.saturating_add(tree_bytes(&entry?.path())?);
    }
    Ok(total)
}

fn remove_tree_beneath(root: &Path, path: &Path) -> Result<(), PluginStoreError> {
    let root = absolute_lexical(root)?;
    let path = absolute_lexical(path)?;
    if path == root || !path.starts_with(&root) {
        return Err(PluginStoreError::PathEscape);
    }
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn absolute_lexical(path: &Path) -> Result<PathBuf, io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
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

    fn verified_package(version: Version, worker: &[u8]) -> VerifiedPackage {
        let manifest = PluginManifest {
            schema_version: 1,
            plugin_id: "dev.markion.fixture".to_owned(),
            version,
            publisher: "Markion".to_owned(),
            host_version: VersionReq::parse(">=0.3.9, <0.4.0").unwrap(),
            protocol: ProtocolRange::V1,
            target: TargetSpec::current(),
            entry_point: "bin/worker".to_owned(),
            identities: BTreeMap::from([(
                "en".to_owned(),
                LocalizedIdentity {
                    name: "Fixture".to_owned(),
                    description: "Fixture plugin".to_owned(),
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
                ("bin/worker", worker),
            ] {
                zip.start_file(path, options).unwrap();
                std::io::Write::write_all(&mut zip, bytes).unwrap();
            }
            zip.finish().unwrap();
        }
        inspect_package_with(
            &cursor.into_inner(),
            |_, _| Ok(()),
            &Version::new(0, 3, 9),
            ProtocolVersion::V1_0,
            &TargetSpec::current(),
            &["en"],
            PackageLimits::default(),
        )
        .unwrap()
    }

    #[test]
    fn store_paths_reject_plugin_id_escape() {
        let store = PluginStore::new(PluginStorePaths::new("plugins"));
        assert!(store.paths().plugin_root("../escape").is_err());
        assert!(store.paths().plugin_root("dev.markion.pdf").is_ok());
    }

    #[test]
    fn install_update_rollback_disable_and_uninstall_are_atomic_at_state_boundary() {
        let root = tempfile::tempdir().unwrap();
        let store = PluginStore::new(PluginStorePaths::new(root.path().join("plugins")));
        let first = verified_package(Version::new(1, 0, 0), b"first");
        let second = verified_package(Version::new(1, 1, 0), b"second-version");

        let installed = store
            .install_verified(&first, |entry, _| {
                assert_eq!(fs::read(entry).unwrap(), b"first");
                Ok(())
            })
            .unwrap();
        assert_eq!(installed.active.version, Version::new(1, 0, 0));
        assert!(installed.rollback.is_none());

        let updated = store.install_verified(&second, |_, _| Ok(())).unwrap();
        assert_eq!(updated.active.version, Version::new(1, 1, 0));
        assert_eq!(updated.rollback.unwrap().version, Version::new(1, 0, 0));
        assert!(updated.usage.active_bytes > 0);
        assert!(updated.usage.rollback_bytes > 0);

        let state = store.rollback("dev.markion.fixture").unwrap();
        assert_eq!(state.active.unwrap().version, Version::new(1, 0, 0));
        assert_eq!(state.rollback.unwrap().version, Version::new(1, 1, 0));
        assert!(
            !store
                .set_enabled("dev.markion.fixture", false)
                .unwrap()
                .enabled
        );

        let config = store.paths().config_root("dev.markion.fixture").unwrap();
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join("settings.json"), b"settings").unwrap();
        store.uninstall("dev.markion.fixture", false).unwrap();
        assert!(config.join("settings.json").is_file());
        assert!(store.state("dev.markion.fixture").unwrap().active.is_none());
        store.uninstall("dev.markion.fixture", true).unwrap();
        assert!(
            !store
                .paths()
                .plugin_root("dev.markion.fixture")
                .unwrap()
                .exists()
        );
    }

    #[test]
    fn failed_health_check_preserves_active_version_and_cleans_staging() {
        let root = tempfile::tempdir().unwrap();
        let store = PluginStore::new(PluginStorePaths::new(root.path().join("plugins")));
        let first = verified_package(Version::new(1, 0, 0), b"first");
        let second = verified_package(Version::new(2, 0, 0), b"second");
        store.install_verified(&first, |_, _| Ok(())).unwrap();
        let error = store.install_verified(&second, |_, _| Err(PluginStoreError::HealthCheck));
        assert!(matches!(error, Err(PluginStoreError::HealthCheck)));
        assert_eq!(
            store
                .state("dev.markion.fixture")
                .unwrap()
                .active
                .unwrap()
                .version,
            Version::new(1, 0, 0)
        );
        assert_eq!(
            fs::read_dir(store.paths().staging_root()).unwrap().count(),
            0
        );
    }

    #[test]
    fn recovery_removes_staging_and_orphans_then_promotes_rollback() {
        let root = tempfile::tempdir().unwrap();
        let store = PluginStore::new(PluginStorePaths::new(root.path().join("plugins")));
        let first = verified_package(Version::new(1, 0, 0), b"first");
        let second = verified_package(Version::new(2, 0, 0), b"second");
        store.install_verified(&first, |_, _| Ok(())).unwrap();
        store.install_verified(&second, |_, _| Ok(())).unwrap();
        fs::remove_dir_all(
            store
                .paths()
                .version_root("dev.markion.fixture", &Version::new(2, 0, 0))
                .unwrap(),
        )
        .unwrap();
        let interrupted = store.paths().staging_root().join("interrupted");
        fs::create_dir_all(&interrupted).unwrap();
        fs::write(interrupted.join("partial"), b"partial").unwrap();

        let report = store.recover().unwrap();
        assert_eq!(report.staging_directories_removed, 1);
        assert_eq!(report.rollback_versions_promoted, 1);
        assert_eq!(
            store
                .state("dev.markion.fixture")
                .unwrap()
                .active
                .unwrap()
                .version,
            Version::new(1, 0, 0)
        );
    }
}
