use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use markion_plugin_protocol::{
    CatalogArtifact, CatalogPlugin, FrameLimits, LocalizedIdentity, PLUGIN_MANIFEST_NAME,
    PLUGIN_SIGNATURE_NAME, PackageError, PackageLimits, Permission, PluginCatalog, PluginManifest,
    PluginMessage, ProcessPluginPeer, ProtocolVersion, TargetSpec, VerifiedPackage, WorkerLaunch,
    canonical_json, inspect_package_with, verify_minisign,
};
use semver::Version;
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::{
    CoreFileHandler, FileHandlerRegistryError, FileHandlerRegistrySnapshot, InstallOutcome,
    PluginSession, PluginStorageUsage, PluginStore, PluginStoreError, PluginSupervisor,
    PluginSupervisorError,
};

type SignatureVerifier = dyn Fn(&[u8], &[u8]) -> Result<(), ()> + Send + Sync;
type HealthCheck =
    dyn Fn(&Path, &PluginManifest, &str) -> Result<(), PluginManagerError> + Send + Sync;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManagedPluginStatus {
    Available,
    Installed,
    Disabled,
    UpdateAvailable,
    Incompatible,
    Quarantined,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedPluginEntry {
    pub plugin_id: String,
    pub catalog_version: Version,
    pub active_version: Option<Version>,
    pub rollback_version: Option<Version>,
    pub publisher: String,
    pub identity: LocalizedIdentity,
    pub permissions: Vec<Permission>,
    pub capabilities: Vec<String>,
    pub status: ManagedPluginStatus,
    pub estimate: Option<super::PluginPackageEstimate>,
    pub usage: PluginStorageUsage,
}

impl ManagedPluginEntry {
    pub fn rollback_available(&self) -> bool {
        self.rollback_version.is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginInstallPlan {
    pub plugin_id: String,
    pub version: Version,
    pub publisher: String,
    pub identity: LocalizedIdentity,
    pub permissions: Vec<Permission>,
    pub capabilities: Vec<String>,
    pub artifact: CatalogArtifact,
    pub is_update: bool,
    expected: CatalogPlugin,
}

impl PluginInstallPlan {
    pub fn estimate(&self) -> super::PluginPackageEstimate {
        (&self.artifact).into()
    }
}

#[derive(Clone)]
pub struct PluginManager {
    catalog: Arc<PluginCatalog>,
    store: PluginStore,
    supervisor: PluginSupervisor,
    host_version: Version,
    protocol: ProtocolVersion,
    target: TargetSpec,
    required_locales: Arc<[String]>,
    verifier: Arc<SignatureVerifier>,
    health_check: Arc<HealthCheck>,
}

impl PluginManager {
    #[allow(clippy::too_many_arguments)]
    pub fn first_party(
        catalog: Arc<PluginCatalog>,
        store: PluginStore,
        supervisor: PluginSupervisor,
        public_key: impl Into<String>,
        host_version: Version,
        protocol: ProtocolVersion,
        target: TargetSpec,
        required_locales: &[&str],
    ) -> Self {
        let public_key = Arc::<str>::from(public_key.into());
        Self::with_adapters(
            catalog,
            store,
            supervisor,
            move |message, signature| {
                let signature = std::str::from_utf8(signature).map_err(|_| ())?;
                verify_minisign(&public_key, signature, message).map_err(|_| ())
            },
            production_health_check,
            host_version,
            protocol,
            target,
            required_locales,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_adapters<V, H>(
        catalog: Arc<PluginCatalog>,
        store: PluginStore,
        supervisor: PluginSupervisor,
        verifier: V,
        health_check: H,
        host_version: Version,
        protocol: ProtocolVersion,
        target: TargetSpec,
        required_locales: &[&str],
    ) -> Self
    where
        V: Fn(&[u8], &[u8]) -> Result<(), ()> + Send + Sync + 'static,
        H: Fn(&Path, &PluginManifest, &str) -> Result<(), PluginManagerError>
            + Send
            + Sync
            + 'static,
    {
        Self {
            catalog,
            store,
            supervisor,
            host_version,
            protocol,
            target,
            required_locales: required_locales
                .iter()
                .map(|locale| (*locale).to_owned())
                .collect::<Vec<_>>()
                .into(),
            verifier: Arc::new(verifier),
            health_check: Arc::new(health_check),
        }
    }

    pub fn catalog(&self) -> &PluginCatalog {
        &self.catalog
    }

    pub fn store(&self) -> &PluginStore {
        &self.store
    }

    pub fn supervisor(&self) -> &PluginSupervisor {
        &self.supervisor
    }

    pub fn file_handler_registry(
        &self,
        revision: u64,
        core_handlers: impl IntoIterator<Item = CoreFileHandler>,
    ) -> Result<FileHandlerRegistrySnapshot, PluginManagerError> {
        FileHandlerRegistrySnapshot::build(
            revision,
            core_handlers,
            &self.catalog,
            &self.store,
            &self.host_version,
            self.protocol,
            &self.target,
        )
        .map_err(Into::into)
    }

    /// Re-verifies the active extracted package and starts its exact signed
    /// entry point for one declared capability. No caller constructs process
    /// paths or handshake limits, which keeps file dispatch independent from
    /// package layout and prevents a replaced on-disk worker from launching.
    pub fn start_capability(
        &self,
        plugin_id: &str,
        capability: &str,
    ) -> Result<PluginSession, PluginManagerError> {
        self.plugin(plugin_id)?;
        let state = self.store.state(plugin_id)?;
        let active = state
            .active
            .as_ref()
            .filter(|_| state.enabled)
            .ok_or(PluginManagerError::Incompatible)?;
        if state.quarantine.contains_key(&active.version) {
            return Err(PluginManagerError::Incompatible);
        }
        let version_root = self
            .store
            .paths()
            .version_root(plugin_id, &active.version)?;
        let manifest = self.verify_installed_version(&version_root, plugin_id, &active.version)?;
        if !manifest
            .capabilities
            .iter()
            .any(|declared| declared.id == capability)
        {
            return Err(PluginManagerError::Incompatible);
        }
        let entry_point = version_root.join(&manifest.entry_point);
        let launch = worker_launch(
            entry_point,
            version_root,
            &manifest,
            active.package_sha256.clone(),
            vec![capability.to_owned()],
        )?;
        self.supervisor.start_process(launch).map_err(Into::into)
    }

    pub fn entries(&self, locale: &str) -> Result<Vec<ManagedPluginEntry>, PluginManagerError> {
        let mut entries = self
            .catalog
            .plugins
            .iter()
            .map(|plugin| self.entry(plugin, locale))
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
        Ok(entries)
    }

    pub fn plan_install(
        &self,
        plugin_id: &str,
        locale: &str,
    ) -> Result<PluginInstallPlan, PluginManagerError> {
        let plugin = self.plugin(plugin_id)?;
        let identity = identity(plugin, locale)?.clone();
        if !plugin.host_version.matches(&self.host_version)
            || !plugin.protocol.supports(self.protocol)
        {
            return Err(PluginManagerError::Incompatible);
        }
        let artifact = plugin
            .artifacts
            .iter()
            .find(|artifact| artifact.target == self.target && artifact.is_published())
            .cloned()
            .ok_or(PluginManagerError::ArtifactUnavailable)?;
        let state = self.store.state(plugin_id)?;
        Ok(PluginInstallPlan {
            plugin_id: plugin.plugin_id.clone(),
            version: plugin.version.clone(),
            publisher: plugin.publisher.clone(),
            identity,
            permissions: plugin.permissions.clone(),
            capabilities: plugin.capabilities.clone(),
            artifact,
            is_update: state.active.is_some(),
            expected: plugin.clone(),
        })
    }

    pub fn install_archive(
        &self,
        plan: &PluginInstallPlan,
        archive: &[u8],
    ) -> Result<InstallOutcome, PluginManagerError> {
        let package = self.verify_archive(plan, archive)?;
        self.activate_verified(plan, &package)
    }

    pub fn verify_archive(
        &self,
        plan: &PluginInstallPlan,
        archive: &[u8],
    ) -> Result<VerifiedPackage, PluginManagerError> {
        self.validate_plan(plan)?;
        if archive.len() as u64 != plan.artifact.length {
            return Err(PluginManagerError::ArtifactMismatch);
        }
        let digest = format!("{:x}", Sha256::digest(archive));
        if !digest.eq_ignore_ascii_case(&plan.artifact.sha256) {
            return Err(PluginManagerError::ArtifactMismatch);
        }
        let locales = self
            .required_locales
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let verifier = Arc::clone(&self.verifier);
        let package = inspect_package_with(
            archive,
            move |manifest, signature| verifier(manifest, signature),
            &self.host_version,
            self.protocol,
            &self.target,
            &locales,
            PackageLimits::default(),
        )?;
        if package.installed_size_bytes != plan.artifact.installed_size_bytes
            || package.archive_size_bytes != plan.artifact.length
            || !package
                .archive_sha256
                .eq_ignore_ascii_case(&plan.artifact.sha256)
            || !manifest_matches_catalog(&package.manifest, &plan.expected)
        {
            return Err(PluginManagerError::CatalogManifestMismatch);
        }
        Ok(package)
    }

    pub fn activate_verified(
        &self,
        plan: &PluginInstallPlan,
        package: &VerifiedPackage,
    ) -> Result<InstallOutcome, PluginManagerError> {
        self.validate_plan(plan)?;
        if package.archive_size_bytes != plan.artifact.length
            || package.installed_size_bytes != plan.artifact.installed_size_bytes
            || !package
                .archive_sha256
                .eq_ignore_ascii_case(&plan.artifact.sha256)
            || !manifest_matches_catalog(&package.manifest, &plan.expected)
        {
            return Err(PluginManagerError::CatalogManifestMismatch);
        }
        let health_check = Arc::clone(&self.health_check);
        let package_sha256 = package.archive_sha256.clone();
        self.store
            .install_verified(package, move |entry_point, manifest| {
                health_check(entry_point, manifest, &package_sha256)
                    .map_err(|_| PluginStoreError::HealthCheck)
            })
            .map_err(Into::into)
    }

    pub fn set_enabled(&self, plugin_id: &str, enabled: bool) -> Result<(), PluginManagerError> {
        self.plugin(plugin_id)?;
        if enabled {
            self.store.set_enabled(plugin_id, true)?;
        } else {
            self.supervisor.disable(plugin_id)?;
        }
        Ok(())
    }

    pub fn rollback(&self, plugin_id: &str) -> Result<(), PluginManagerError> {
        self.plugin(plugin_id)?;
        self.supervisor.stop(plugin_id);
        self.store.rollback(plugin_id)?;
        Ok(())
    }

    pub fn uninstall(
        &self,
        plugin_id: &str,
        remove_configuration: bool,
    ) -> Result<(), PluginManagerError> {
        self.plugin(plugin_id)?;
        self.supervisor.stop(plugin_id);
        self.store.uninstall(plugin_id, remove_configuration)?;
        Ok(())
    }

    fn plugin(&self, plugin_id: &str) -> Result<&CatalogPlugin, PluginManagerError> {
        self.catalog
            .plugins
            .iter()
            .find(|plugin| plugin.plugin_id == plugin_id)
            .ok_or_else(|| PluginManagerError::UnknownPlugin(plugin_id.to_owned()))
    }

    fn validate_plan(&self, plan: &PluginInstallPlan) -> Result<(), PluginManagerError> {
        let current = self.plugin(&plan.plugin_id)?;
        if current != &plan.expected
            || current.version != plan.version
            || !current
                .artifacts
                .iter()
                .any(|artifact| artifact == &plan.artifact && artifact.is_published())
        {
            return Err(PluginManagerError::StalePlan);
        }
        Ok(())
    }

    fn verify_installed_version(
        &self,
        root: &Path,
        plugin_id: &str,
        version: &Version,
    ) -> Result<PluginManifest, PluginManagerError> {
        let manifest_bytes = fs::read(root.join(PLUGIN_MANIFEST_NAME))?;
        let signature = fs::read(root.join(PLUGIN_SIGNATURE_NAME))?;
        if manifest_bytes.len() > PackageLimits::default().max_manifest_bytes
            || signature.len() > PackageLimits::default().max_signature_bytes
        {
            return Err(PluginManagerError::InstalledPackageInvalid);
        }
        let manifest: PluginManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|_| PluginManagerError::InstalledPackageInvalid)?;
        if canonical_json(&manifest).map_err(|_| PluginManagerError::InstalledPackageInvalid)?
            != manifest_bytes
            || (self.verifier)(&manifest_bytes, &signature).is_err()
        {
            return Err(PluginManagerError::InstalledPackageInvalid);
        }
        let locales = self
            .required_locales
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        manifest
            .validate(&self.host_version, self.protocol, &self.target, &locales)
            .map_err(|_| PluginManagerError::InstalledPackageInvalid)?;
        if manifest.plugin_id != plugin_id || &manifest.version != version {
            return Err(PluginManagerError::InstalledPackageInvalid);
        }
        verify_installed_members(root, &manifest)?;
        Ok(manifest)
    }

    fn entry(
        &self,
        plugin: &CatalogPlugin,
        locale: &str,
    ) -> Result<ManagedPluginEntry, PluginManagerError> {
        let state = self.store.state(&plugin.plugin_id)?;
        let artifact = plugin
            .artifacts
            .iter()
            .find(|artifact| artifact.target == self.target && artifact.is_published());
        let compatible = plugin.host_version.matches(&self.host_version)
            && plugin.protocol.supports(self.protocol)
            && artifact.is_some();
        let status = if state
            .active
            .as_ref()
            .is_some_and(|active| state.quarantine.contains_key(&active.version))
        {
            ManagedPluginStatus::Quarantined
        } else if state.active.is_some() && !state.enabled {
            ManagedPluginStatus::Disabled
        } else if !compatible {
            ManagedPluginStatus::Incompatible
        } else if let Some(active) = state.active.as_ref() {
            if active.version != plugin.version {
                ManagedPluginStatus::UpdateAvailable
            } else {
                ManagedPluginStatus::Installed
            }
        } else {
            ManagedPluginStatus::Available
        };
        Ok(ManagedPluginEntry {
            plugin_id: plugin.plugin_id.clone(),
            catalog_version: plugin.version.clone(),
            active_version: state.active.as_ref().map(|active| active.version.clone()),
            rollback_version: state
                .rollback
                .as_ref()
                .map(|rollback| rollback.version.clone()),
            publisher: plugin.publisher.clone(),
            identity: identity(plugin, locale)?.clone(),
            permissions: plugin.permissions.clone(),
            capabilities: plugin.capabilities.clone(),
            status,
            estimate: artifact.map(Into::into),
            usage: self.store.usage(&plugin.plugin_id)?,
        })
    }
}

fn identity<'a>(
    plugin: &'a CatalogPlugin,
    locale: &str,
) -> Result<&'a LocalizedIdentity, PluginManagerError> {
    plugin
        .identities
        .get(locale)
        .ok_or_else(|| PluginManagerError::MissingLocale(locale.to_owned()))
}

fn manifest_matches_catalog(manifest: &PluginManifest, catalog: &CatalogPlugin) -> bool {
    let capability_ids = manifest
        .capabilities
        .iter()
        .map(|capability| capability.id.as_str())
        .collect::<Vec<_>>();
    manifest.plugin_id == catalog.plugin_id
        && manifest.version == catalog.version
        && manifest.publisher == catalog.publisher
        && manifest.host_version == catalog.host_version
        && manifest.protocol == catalog.protocol
        && catalog
            .artifacts
            .iter()
            .any(|artifact| artifact.target == manifest.target)
        && manifest.identities == catalog.identities
        && manifest.permissions == catalog.permissions
        && capability_ids
            == catalog
                .capabilities
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        && manifest.file_handlers == catalog.file_handlers
}

fn verify_installed_members(
    root: &Path,
    manifest: &PluginManifest,
) -> Result<(), PluginManagerError> {
    let mut actual = BTreeMap::<String, PathBuf>::new();
    collect_installed_files(root, root, &mut actual)?;
    let expected = manifest
        .members
        .iter()
        .map(|member| member.path.as_str())
        .chain([PLUGIN_MANIFEST_NAME, PLUGIN_SIGNATURE_NAME])
        .collect::<BTreeSet<_>>();
    let actual_names = actual.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual_names != expected {
        return Err(PluginManagerError::InstalledPackageInvalid);
    }
    for member in &manifest.members {
        let path = actual
            .get(&member.path)
            .ok_or(PluginManagerError::InstalledPackageInvalid)?;
        let bytes = fs::read(path)?;
        if bytes.len() as u64 != member.length
            || !format!("{:x}", Sha256::digest(&bytes)).eq_ignore_ascii_case(&member.sha256)
        {
            return Err(PluginManagerError::InstalledPackageInvalid);
        }
    }
    Ok(())
}

fn collect_installed_files(
    root: &Path,
    directory: &Path,
    output: &mut BTreeMap<String, PathBuf>,
) -> Result<(), PluginManagerError> {
    if output.len() > PackageLimits::default().max_members {
        return Err(PluginManagerError::InstalledPackageInvalid);
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            return Err(PluginManagerError::InstalledPackageInvalid);
        }
        if metadata.is_dir() {
            collect_installed_files(root, &entry.path(), output)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(PluginManagerError::InstalledPackageInvalid);
        }
        if output.len() >= PackageLimits::default().max_members {
            return Err(PluginManagerError::InstalledPackageInvalid);
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|_| PluginManagerError::InstalledPackageInvalid)?
            .to_string_lossy()
            .replace('\\', "/");
        if output
            .keys()
            .any(|present| present.eq_ignore_ascii_case(&relative))
            || output.insert(relative, entry.path()).is_some()
        {
            return Err(PluginManagerError::InstalledPackageInvalid);
        }
    }
    Ok(())
}

fn worker_launch(
    entry_point: PathBuf,
    working_directory: PathBuf,
    manifest: &PluginManifest,
    package_sha256: String,
    requested_capabilities: Vec<String>,
) -> Result<WorkerLaunch, PluginManagerError> {
    let max_control_bytes = manifest
        .capabilities
        .iter()
        .map(|capability| capability.limits.max_control_bytes)
        .max()
        .ok_or(PluginManagerError::HealthCheck)?;
    let max_body_bytes = manifest
        .capabilities
        .iter()
        .map(|capability| capability.limits.max_body_bytes)
        .max()
        .ok_or(PluginManagerError::HealthCheck)?;
    let max_in_flight_requests = manifest
        .capabilities
        .iter()
        .map(|capability| usize::from(capability.limits.max_in_flight_requests))
        .min()
        .ok_or(PluginManagerError::HealthCheck)?;
    let request_timeout_ms = manifest
        .capabilities
        .iter()
        .map(|capability| capability.limits.request_timeout_ms)
        .min()
        .ok_or(PluginManagerError::HealthCheck)?;
    Ok(WorkerLaunch {
        executable: entry_point,
        working_directory,
        protocol: ProtocolVersion::V1_0,
        host_version: Version::parse(env!("CARGO_PKG_VERSION")).expect("package version is semver"),
        plugin_id: manifest.plugin_id.clone(),
        plugin_version: manifest.version.clone(),
        package_sha256,
        requested_capabilities,
        expected_capabilities: manifest.capabilities.clone(),
        frame_limits: FrameLimits {
            max_header_bytes: max_control_bytes,
            max_body_bytes,
        },
        max_in_flight_requests,
        request_timeout: Duration::from_millis(u64::from(request_timeout_ms)),
        shutdown_timeout: Duration::from_secs(2),
        stderr_limit_bytes: 16 * 1024,
        environment: Vec::new(),
    })
}

fn production_health_check(
    entry_point: &Path,
    manifest: &PluginManifest,
    package_sha256: &str,
) -> Result<(), PluginManagerError> {
    let component_count = Path::new(&manifest.entry_point).components().count();
    let working_directory = entry_point
        .ancestors()
        .nth(component_count)
        .ok_or(PluginManagerError::HealthCheck)?;
    let requested_capabilities = manifest
        .capabilities
        .iter()
        .map(|capability| capability.id.clone())
        .collect();
    let launch = worker_launch(
        entry_point.to_path_buf(),
        working_directory.to_path_buf(),
        manifest,
        package_sha256.to_owned(),
        requested_capabilities,
    )?;
    let peer = ProcessPluginPeer::launch(launch, 1).map_err(|_| PluginManagerError::HealthCheck)?;
    let healthy = peer
        .request_with_timeout(
            markion_plugin_protocol::HostMessage::Ping,
            Vec::new(),
            Duration::from_secs(2),
        )
        .is_ok_and(|response| response.message == PluginMessage::Pong && response.body.is_empty());
    if !healthy {
        peer.force_terminate();
        return Err(PluginManagerError::HealthCheck);
    }
    peer.shutdown().map_err(|_| PluginManagerError::HealthCheck)
}

#[derive(Debug, Error)]
pub enum PluginManagerError {
    #[error("installed plugin package is invalid or was modified")]
    InstalledPackageInvalid,
    #[error("installed plugin package could not be read")]
    Io(#[from] std::io::Error),
    #[error("unknown plugin {0}")]
    UnknownPlugin(String),
    #[error("plugin is incompatible with this host")]
    Incompatible,
    #[error("plugin artifact is not published for this target")]
    ArtifactUnavailable,
    #[error("plugin installation plan is stale")]
    StalePlan,
    #[error("plugin artifact does not match the signed catalog")]
    ArtifactMismatch,
    #[error("plugin manifest does not match the signed catalog")]
    CatalogManifestMismatch,
    #[error("plugin package verification failed")]
    Package(#[from] PackageError),
    #[error("plugin store operation failed")]
    Store(#[from] PluginStoreError),
    #[error("plugin file-handler registry could not be built")]
    Registry(#[from] FileHandlerRegistryError),
    #[error("plugin supervisor operation failed")]
    Supervisor(#[from] PluginSupervisorError),
    #[error("plugin locale is unavailable: {0}")]
    MissingLocale(String),
    #[error("plugin health check failed")]
    HealthCheck,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use markion_plugin_protocol::{FileHandlerDeclaration, LocalizedIdentity, ProtocolRange};
    use semver::VersionReq;

    use super::*;
    use crate::plugin_platform::{PluginStorePaths, PluginSupervisorPolicy};

    fn catalog(artifact: CatalogArtifact) -> Arc<PluginCatalog> {
        Arc::new(PluginCatalog {
            schema_version: 1,
            sequence: 1,
            plugins: vec![CatalogPlugin {
                plugin_id: "dev.markion.pdf".to_owned(),
                version: Version::new(1, 0, 0),
                publisher: "Markion".to_owned(),
                host_version: VersionReq::parse(">=0.3.9, <0.4.0").unwrap(),
                protocol: ProtocolRange::V1,
                identities: BTreeMap::from([(
                    "en".to_owned(),
                    LocalizedIdentity {
                        name: "PDF Viewer".to_owned(),
                        description: "Read local PDF documents.".to_owned(),
                    },
                )]),
                permissions: vec![Permission::ReadSelectedFiles],
                capabilities: vec!["paged-document/v1".to_owned()],
                file_handlers: vec![FileHandlerDeclaration {
                    extensions: vec![".pdf".to_owned()],
                    capability: "paged-document/v1".to_owned(),
                    priority: 100,
                    icon: "document-pdf".to_owned(),
                }],
                artifacts: vec![artifact],
            }],
        })
    }

    fn manager(root: &Path, published: bool) -> PluginManager {
        let artifact = CatalogArtifact {
            target: TargetSpec::current(),
            url: "https://example.invalid/pdf.markion-plugin".to_owned(),
            length: u64::from(published),
            sha256: if published {
                "1".repeat(64)
            } else {
                "0".repeat(64)
            },
            installed_size_bytes: u64::from(published),
        };
        let store = PluginStore::new(PluginStorePaths::new(root));
        let supervisor = PluginSupervisor::new(store.clone(), PluginSupervisorPolicy::default());
        PluginManager::with_adapters(
            catalog(artifact),
            store,
            supervisor,
            |_, _| Ok(()),
            |_, _, _| Ok(()),
            Version::new(0, 3, 9),
            ProtocolVersion::V1_0,
            TargetSpec::current(),
            &["en"],
        )
    }

    #[test]
    fn entries_distinguish_available_and_unpublished_plugins() {
        let root = tempfile::tempdir().unwrap();
        let available = manager(root.path(), true);
        let entry = available.entries("en").unwrap().pop().unwrap();
        assert_eq!(entry.status, ManagedPluginStatus::Available);
        assert_eq!(entry.estimate.unwrap().download_bytes, 1);

        let root = tempfile::tempdir().unwrap();
        let unpublished = manager(root.path(), false);
        let entry = unpublished.entries("en").unwrap().pop().unwrap();
        assert_eq!(entry.status, ManagedPluginStatus::Incompatible);
        assert!(entry.estimate.is_none());
        assert!(matches!(
            unpublished.plan_install("dev.markion.pdf", "en"),
            Err(PluginManagerError::ArtifactUnavailable)
        ));
    }

    #[test]
    fn install_plan_contains_signed_disclosures_and_rejects_stale_or_wrong_bytes() {
        let root = tempfile::tempdir().unwrap();
        let manager = manager(root.path(), true);
        let mut plan = manager.plan_install("dev.markion.pdf", "en").unwrap();
        assert_eq!(plan.publisher, "Markion");
        assert_eq!(plan.permissions, vec![Permission::ReadSelectedFiles]);
        assert_eq!(plan.estimate().installed_bytes, 1);
        assert!(matches!(
            manager.install_archive(&plan, b"x"),
            Err(PluginManagerError::ArtifactMismatch)
        ));
        plan.version = Version::new(2, 0, 0);
        assert!(matches!(
            manager.install_archive(&plan, b"x"),
            Err(PluginManagerError::StalePlan)
        ));
    }
}
