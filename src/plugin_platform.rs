//! First-party plugin host bootstrap.
//!
//! The full lifecycle/store implementation grows behind this module. The core
//! application only needs an immutable, validated catalog snapshot during
//! startup; optional workers and native runtimes remain outside the package.

use std::sync::{Arc, OnceLock};

use markion_plugin_protocol::{
    PluginCatalog, ProtocolVersion, TargetSpec, canonical_json, verify_minisign,
};
use semver::Version;
use thiserror::Error;

mod catalog;
mod manager;
mod paged_document;
mod registry;
mod store;
mod supervisor;

pub use catalog::{
    CatalogManager, CatalogSnapshot, CatalogSnapshotSource, CatalogUpdate, PluginCatalogError,
};
pub use manager::{
    ManagedPluginEntry, ManagedPluginStatus, PluginInstallPlan, PluginManager, PluginManagerError,
};
pub use paged_document::{
    DEFAULT_MAX_PAGES, DEFAULT_MAX_RASTER_BYTES, DEFAULT_MAX_SOURCE_BYTES, PagedDocument,
    PagedDocumentError, PagedDocumentLimits, PagedRender, PagedRenderRequest,
};
pub use registry::{
    CoreFileHandler, CoreHandlerKind, FileHandlerAvailability, FileHandlerCandidate,
    FileHandlerRegistryError, FileHandlerRegistrySnapshot, FileHandlerResolution,
};
pub use store::{
    ActivationRecord, InstallOutcome, PluginPackageEstimate, PluginQuarantine, PluginStorageUsage,
    PluginStore, PluginStoreError, PluginStorePaths, PluginStoreState, QuarantineReason,
    RecoveryReport,
};
pub use supervisor::{
    PluginSession, PluginSupervisor, PluginSupervisorError, PluginSupervisorPolicy,
    SupervisedRequest,
};

const BOOTSTRAP_CATALOG: &str = include_str!("../assets/plugins/catalog.json");
const BOOTSTRAP_CATALOG_SIGNATURE: &str = include_str!("../assets/plugins/catalog.json.minisig");
const REQUIRED_LOCALES: &[&str] = &["en", "zh-Hans", "zh-Hant", "ja", "fr", "de", "es"];

// Debug/test builds deliberately trust only this repository-controlled test
// key. Release builds never contain it: their public half is injected by the
// release job as the single-line Minisign key in MARKION_PLUGIN_PUBLIC_KEY.
#[cfg(debug_assertions)]
const DEVELOPMENT_PUBLIC_KEY: &str =
    include_str!("../crates/plugin-protocol/tests/fixtures/signing/catalog-test.pub");

static BOOTSTRAP: OnceLock<Result<PluginHostBootstrap, String>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct PluginHostBootstrap {
    catalog: Arc<PluginCatalog>,
    canonical_catalog: Arc<[u8]>,
    public_key: Arc<str>,
}

impl PluginHostBootstrap {
    pub fn catalog(&self) -> &PluginCatalog {
        &self.catalog
    }

    pub fn canonical_catalog(&self) -> &[u8] {
        &self.canonical_catalog
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub fn catalog_manager(&self, paths: PluginStorePaths) -> CatalogManager {
        CatalogManager::first_party(
            paths,
            self.public_key.to_string(),
            self.canonical_catalog.clone(),
            Arc::<[u8]>::from(BOOTSTRAP_CATALOG_SIGNATURE.as_bytes()),
            Version::parse(env!("CARGO_PKG_VERSION")).expect("package version is semver"),
            ProtocolVersion::V1_0,
            TargetSpec::current(),
            REQUIRED_LOCALES,
        )
    }

    /// Opens the recovered per-user store and binds the newest locally
    /// verified catalog snapshot to the production supervisor. This is the
    /// only constructor used by application UI, so cached-catalog fallback,
    /// package trust, compatibility, and recovery cannot drift apart.
    pub fn plugin_manager(
        &self,
        paths: PluginStorePaths,
    ) -> Result<PluginManager, PluginRuntimeError> {
        let snapshot = self.catalog_manager(paths.clone()).load()?;
        let store = PluginStore::new(paths);
        store.recover()?;
        let supervisor = PluginSupervisor::new(store.clone(), PluginSupervisorPolicy::default());
        Ok(PluginManager::first_party(
            Arc::new(snapshot.catalog().clone()),
            store,
            supervisor,
            self.public_key.to_string(),
            Version::parse(env!("CARGO_PKG_VERSION")).expect("package version is semver"),
            ProtocolVersion::V1_0,
            TargetSpec::current(),
            REQUIRED_LOCALES,
        ))
    }
}

#[derive(Debug, Error)]
pub enum PluginRuntimeError {
    #[error("plugin catalog is unavailable")]
    Catalog(#[from] PluginCatalogError),
    #[error("plugin store is unavailable")]
    Store(#[from] PluginStoreError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginHostSummary {
    pub catalog_sequence: u64,
    pub official_plugins: usize,
    pub canonical_catalog_bytes: usize,
}

#[derive(Debug, Error)]
pub enum PluginBootstrapError {
    #[error("release plugin trust root is unavailable")]
    TrustRootUnavailable,
    #[error("bootstrap catalog is invalid JSON")]
    Json(#[from] serde_json::Error),
    #[error("bootstrap catalog JSON is not canonical")]
    NonCanonical,
    #[error("bootstrap catalog failed validation")]
    Validation(#[from] markion_plugin_protocol::ValidationError),
    #[error("bootstrap catalog requires unsupported capability {0}")]
    UnsupportedCapability(String),
}

pub fn bootstrap() -> Result<&'static PluginHostBootstrap, PluginBootstrapError> {
    let result = BOOTSTRAP.get_or_init(|| load_bootstrap().map_err(|error| error.to_string()));
    result.as_ref().map_err(|_| {
        // The detailed parser/signature diagnostic stays internal. Host UI maps
        // this to a stable localized category once plugin UI is enabled.
        PluginBootstrapError::Validation(markion_plugin_protocol::ValidationError::InvalidSignature)
    })
}

pub fn bootstrap_summary() -> Result<PluginHostSummary, PluginBootstrapError> {
    let bootstrap = bootstrap()?;
    Ok(PluginHostSummary {
        catalog_sequence: bootstrap.catalog.sequence,
        official_plugins: bootstrap.catalog.plugins.len(),
        canonical_catalog_bytes: bootstrap.canonical_catalog.len(),
    })
}

fn load_bootstrap() -> Result<PluginHostBootstrap, PluginBootstrapError> {
    let catalog: PluginCatalog = serde_json::from_str(BOOTSTRAP_CATALOG)?;
    let canonical_catalog = Arc::<[u8]>::from(canonical_json(&catalog)?);
    if canonical_catalog.as_ref() != BOOTSTRAP_CATALOG.as_bytes() {
        return Err(PluginBootstrapError::NonCanonical);
    }
    let public_key = release_public_key()?;
    verify_minisign(&public_key, BOOTSTRAP_CATALOG_SIGNATURE, &canonical_catalog)?;
    catalog.validate(
        &Version::parse(env!("CARGO_PKG_VERSION")).expect("package version is semver"),
        ProtocolVersion::V1_0,
        &TargetSpec::current(),
        REQUIRED_LOCALES,
    )?;
    if let Some(capability) = catalog::unsupported_capability(&catalog) {
        return Err(PluginBootstrapError::UnsupportedCapability(capability));
    }
    Ok(PluginHostBootstrap {
        catalog: Arc::new(catalog),
        canonical_catalog,
        public_key: public_key.into(),
    })
}

#[cfg(debug_assertions)]
fn release_public_key() -> Result<String, PluginBootstrapError> {
    Ok(DEVELOPMENT_PUBLIC_KEY.to_owned())
}

#[cfg(not(debug_assertions))]
fn release_public_key() -> Result<String, PluginBootstrapError> {
    let encoded = option_env!("MARKION_PLUGIN_PUBLIC_KEY")
        .filter(|value| !value.trim().is_empty())
        .ok_or(PluginBootstrapError::TrustRootUnavailable)?;
    Ok(format!(
        "untrusted comment: Markion official plugin signing key\n{}",
        encoded.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_catalog_is_valid_and_locale_complete() {
        let catalog: PluginCatalog = serde_json::from_str(BOOTSTRAP_CATALOG).unwrap();
        let canonical = canonical_json(&catalog).unwrap();
        assert_eq!(canonical, BOOTSTRAP_CATALOG.as_bytes());
        verify_minisign(
            DEVELOPMENT_PUBLIC_KEY,
            BOOTSTRAP_CATALOG_SIGNATURE,
            &canonical,
        )
        .unwrap();
        let bootstrap = load_bootstrap().unwrap();
        assert_eq!(bootstrap.catalog.sequence, 1);
        assert_eq!(bootstrap.catalog.plugins.len(), 1);
        assert_eq!(bootstrap.catalog.plugins[0].plugin_id, "dev.markion.pdf");
        assert!(!bootstrap.canonical_catalog.is_empty());
        assert!(!bootstrap.public_key().is_empty());
        assert!(
            bootstrap
                .catalog_manager(PluginStorePaths::new("plugins"))
                .load()
                .is_ok()
        );
    }

    #[test]
    fn bootstrap_is_cached_as_one_immutable_snapshot() {
        let first = bootstrap().unwrap() as *const PluginHostBootstrap;
        let second = bootstrap().unwrap() as *const PluginHostBootstrap;
        assert_eq!(first, second);
    }
}
