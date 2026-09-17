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
mod paged_document;
mod registry;
mod store;
mod supervisor;

pub use catalog::{
    CatalogManager, CatalogSnapshot, CatalogSnapshotSource, CatalogUpdate, PluginCatalogError,
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
const REQUIRED_LOCALES: &[&str] = &["en", "zh-Hans", "zh-Hant", "ja", "fr", "de", "es"];

// This public key/signature pair is the Minisign upstream reference vector. It
// makes the feasibility build retain and exercise the exact verifier that the
// production bootstrap/package path uses. A release-key-signed catalog replaces
// this self-test when release publication is implemented; it is not a trust root.
const VERIFIER_REFERENCE_PUBLIC_KEY: &str = "untrusted comment: minisign public key: A7090F0642B4E81F\nRWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
const VERIFIER_REFERENCE_SIGNATURE: &str = "untrusted comment: signature from minisign secret key\nRUQf6LRCGA9i559r3g7V1qNyJDApGip8MfqcadIgT9CuhV3EMhHoN1mGTkUidF/z7SrlQgXdy8ofjb7bNJJylDOocrCo8KLzZwo=\ntrusted comment: timestamp:1633700835\tfile:test\tprehashed\nwLMDjy9FLAuxZ3q4NlEvkgtyhrr0gtTu6KC4KBJdITbbOeAi1zBIYo0v4iTgt8jJpIidRJnp94ABQkJAgAooBQ==";

static BOOTSTRAP: OnceLock<Result<PluginHostBootstrap, String>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct PluginHostBootstrap {
    catalog: Arc<PluginCatalog>,
    canonical_catalog: Arc<[u8]>,
}

impl PluginHostBootstrap {
    pub fn catalog(&self) -> &PluginCatalog {
        &self.catalog
    }

    pub fn canonical_catalog(&self) -> &[u8] {
        &self.canonical_catalog
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginHostSummary {
    pub catalog_sequence: u64,
    pub official_plugins: usize,
    pub canonical_catalog_bytes: usize,
}

#[derive(Debug, Error)]
pub enum PluginBootstrapError {
    #[error("bootstrap catalog is invalid JSON")]
    Json(#[from] serde_json::Error),
    #[error("bootstrap catalog failed validation")]
    Validation(#[from] markion_plugin_protocol::ValidationError),
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
    verify_minisign(
        VERIFIER_REFERENCE_PUBLIC_KEY,
        VERIFIER_REFERENCE_SIGNATURE,
        b"test",
    )?;
    let catalog: PluginCatalog = serde_json::from_str(BOOTSTRAP_CATALOG)?;
    catalog.validate(
        &Version::parse(env!("CARGO_PKG_VERSION")).expect("package version is semver"),
        ProtocolVersion::V1_0,
        &TargetSpec::current(),
        REQUIRED_LOCALES,
    )?;
    let canonical_catalog = Arc::<[u8]>::from(canonical_json(&catalog)?);
    Ok(PluginHostBootstrap {
        catalog: Arc::new(catalog),
        canonical_catalog,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_catalog_is_valid_and_locale_complete() {
        let bootstrap = load_bootstrap().unwrap();
        assert_eq!(bootstrap.catalog.sequence, 1);
        assert_eq!(bootstrap.catalog.plugins.len(), 1);
        assert_eq!(bootstrap.catalog.plugins[0].plugin_id, "dev.markion.pdf");
        assert!(!bootstrap.canonical_catalog.is_empty());
    }

    #[test]
    fn bootstrap_is_cached_as_one_immutable_snapshot() {
        let first = bootstrap().unwrap() as *const PluginHostBootstrap;
        let second = bootstrap().unwrap() as *const PluginHostBootstrap;
        assert_eq!(first, second);
    }
}
