use std::{fs, sync::Arc};

use markion_plugin_protocol::{
    PAGED_DOCUMENT_CAPABILITY, PluginCatalog, ProtocolVersion, TargetSpec, canonical_json,
    verify_minisign,
};
use semver::Version;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::storage::atomic_write;

use super::PluginStorePaths;

type SignatureVerifier = dyn Fn(&[u8], &[u8]) -> Result<(), PluginCatalogError> + Send + Sync;

const CATALOG_CACHE_MAGIC: &[u8; 8] = b"MKCAT01\0";
const MAX_CATALOG_BYTES: usize = 1024 * 1024;
const MAX_CATALOG_SIGNATURE_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogSnapshotSource {
    Bootstrap,
    Cached,
    Refreshed,
}

#[derive(Clone, Debug)]
pub struct CatalogSnapshot {
    catalog: Arc<PluginCatalog>,
    canonical_bytes: Arc<[u8]>,
    sha256: String,
    source: CatalogSnapshotSource,
}

impl CatalogSnapshot {
    pub fn catalog(&self) -> &PluginCatalog {
        &self.catalog
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn source(&self) -> CatalogSnapshotSource {
        self.source
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogUpdate {
    Activated { sequence: u64, sha256: String },
    Unchanged { sequence: u64, sha256: String },
}

#[derive(Clone)]
pub struct CatalogManager {
    paths: PluginStorePaths,
    bootstrap_json: Arc<[u8]>,
    bootstrap_signature: Arc<[u8]>,
    verifier: Arc<SignatureVerifier>,
    host_version: Version,
    protocol: ProtocolVersion,
    target: TargetSpec,
    required_locales: Arc<[String]>,
}

impl CatalogManager {
    #[allow(clippy::too_many_arguments)]
    pub fn first_party(
        paths: PluginStorePaths,
        public_key: impl Into<String>,
        bootstrap_json: impl Into<Arc<[u8]>>,
        bootstrap_signature: impl Into<Arc<[u8]>>,
        host_version: Version,
        protocol: ProtocolVersion,
        target: TargetSpec,
        required_locales: &[&str],
    ) -> Self {
        let public_key = Arc::<str>::from(public_key.into());
        Self::with_verifier(
            paths,
            bootstrap_json,
            bootstrap_signature,
            move |message, signature| {
                let signature = std::str::from_utf8(signature)
                    .map_err(|_| PluginCatalogError::InvalidSignature)?;
                verify_minisign(&public_key, signature, message)
                    .map_err(|_| PluginCatalogError::InvalidSignature)
            },
            host_version,
            protocol,
            target,
            required_locales,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_verifier<F>(
        paths: PluginStorePaths,
        bootstrap_json: impl Into<Arc<[u8]>>,
        bootstrap_signature: impl Into<Arc<[u8]>>,
        verifier: F,
        host_version: Version,
        protocol: ProtocolVersion,
        target: TargetSpec,
        required_locales: &[&str],
    ) -> Self
    where
        F: Fn(&[u8], &[u8]) -> Result<(), PluginCatalogError> + Send + Sync + 'static,
    {
        Self {
            paths,
            bootstrap_json: bootstrap_json.into(),
            bootstrap_signature: bootstrap_signature.into(),
            verifier: Arc::new(verifier),
            host_version,
            protocol,
            target,
            required_locales: required_locales
                .iter()
                .map(|locale| (*locale).to_owned())
                .collect::<Vec<_>>()
                .into(),
        }
    }

    pub fn load(&self) -> Result<CatalogSnapshot, PluginCatalogError> {
        let bootstrap = self.verify(
            &self.bootstrap_json,
            &self.bootstrap_signature,
            CatalogSnapshotSource::Bootstrap,
        )?;
        let cached_bundle = match fs::read(self.paths.catalog_cache_path()) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(bootstrap),
            Err(error) => return Err(error.into()),
        };
        let (cached_json, cached_signature) = match decode_catalog_cache(&cached_bundle) {
            Ok(pair) => pair,
            Err(_) => return Ok(bootstrap),
        };
        match self.verify(cached_json, cached_signature, CatalogSnapshotSource::Cached) {
            Ok(cached) if cached.catalog.sequence >= bootstrap.catalog.sequence => Ok(cached),
            _ => Ok(bootstrap),
        }
    }

    pub fn refresh(
        &self,
        catalog_json: &[u8],
        signature: &[u8],
    ) -> Result<CatalogUpdate, PluginCatalogError> {
        let current = self.load()?;
        let candidate = self.verify(catalog_json, signature, CatalogSnapshotSource::Refreshed)?;
        if candidate.catalog.sequence < current.catalog.sequence {
            return Err(PluginCatalogError::SequenceRollback {
                current: current.catalog.sequence,
                candidate: candidate.catalog.sequence,
            });
        }
        if candidate.catalog.sequence == current.catalog.sequence {
            if candidate.sha256 == current.sha256 {
                return Ok(CatalogUpdate::Unchanged {
                    sequence: current.catalog.sequence,
                    sha256: current.sha256,
                });
            }
            return Err(PluginCatalogError::SequenceReuse(
                candidate.catalog.sequence,
            ));
        }

        fs::create_dir_all(self.paths.root())?;
        let cache = encode_catalog_cache(&candidate.canonical_bytes, signature)?;
        atomic_write(self.paths.catalog_cache_path(), cache)?;
        Ok(CatalogUpdate::Activated {
            sequence: candidate.catalog.sequence,
            sha256: candidate.sha256,
        })
    }

    fn verify(
        &self,
        json: &[u8],
        signature: &[u8],
        source: CatalogSnapshotSource,
    ) -> Result<CatalogSnapshot, PluginCatalogError> {
        if json.len() > MAX_CATALOG_BYTES || signature.len() > MAX_CATALOG_SIGNATURE_BYTES {
            return Err(PluginCatalogError::CacheFormat);
        }
        let catalog: PluginCatalog = serde_json::from_slice(json)?;
        let canonical = canonical_json(&catalog)?;
        if canonical.as_slice() != json {
            return Err(PluginCatalogError::NonCanonical);
        }
        (self.verifier)(&canonical, signature)?;
        let locales = self
            .required_locales
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        catalog.validate(&self.host_version, self.protocol, &self.target, &locales)?;
        if let Some(capability) = unsupported_capability(&catalog) {
            return Err(PluginCatalogError::UnsupportedCapability(capability));
        }
        let sha256 = format!("{:x}", Sha256::digest(&canonical));
        Ok(CatalogSnapshot {
            catalog: Arc::new(catalog),
            canonical_bytes: canonical.into(),
            sha256,
            source,
        })
    }
}

pub(super) fn unsupported_capability(catalog: &PluginCatalog) -> Option<String> {
    catalog
        .plugins
        .iter()
        .flat_map(|plugin| plugin.capabilities.iter())
        .find(|capability| capability.as_str() != PAGED_DOCUMENT_CAPABILITY)
        .cloned()
}

fn encode_catalog_cache(catalog: &[u8], signature: &[u8]) -> Result<Vec<u8>, PluginCatalogError> {
    if catalog.len() > MAX_CATALOG_BYTES || signature.len() > MAX_CATALOG_SIGNATURE_BYTES {
        return Err(PluginCatalogError::CacheFormat);
    }
    let catalog_len = u32::try_from(catalog.len()).map_err(|_| PluginCatalogError::CacheFormat)?;
    let signature_len =
        u32::try_from(signature.len()).map_err(|_| PluginCatalogError::CacheFormat)?;
    let mut bytes = Vec::with_capacity(
        CATALOG_CACHE_MAGIC.len() + 8 + catalog.len().saturating_add(signature.len()),
    );
    bytes.extend_from_slice(CATALOG_CACHE_MAGIC);
    bytes.extend_from_slice(&catalog_len.to_le_bytes());
    bytes.extend_from_slice(&signature_len.to_le_bytes());
    bytes.extend_from_slice(catalog);
    bytes.extend_from_slice(signature);
    Ok(bytes)
}

fn decode_catalog_cache(bytes: &[u8]) -> Result<(&[u8], &[u8]), PluginCatalogError> {
    let header_len = CATALOG_CACHE_MAGIC.len() + 8;
    if bytes.len() < header_len || &bytes[..CATALOG_CACHE_MAGIC.len()] != CATALOG_CACHE_MAGIC {
        return Err(PluginCatalogError::CacheFormat);
    }
    let catalog_len = u32::from_le_bytes(
        bytes[8..12]
            .try_into()
            .map_err(|_| PluginCatalogError::CacheFormat)?,
    ) as usize;
    let signature_len = u32::from_le_bytes(
        bytes[12..16]
            .try_into()
            .map_err(|_| PluginCatalogError::CacheFormat)?,
    ) as usize;
    if catalog_len > MAX_CATALOG_BYTES || signature_len > MAX_CATALOG_SIGNATURE_BYTES {
        return Err(PluginCatalogError::CacheFormat);
    }
    let catalog_end = header_len
        .checked_add(catalog_len)
        .ok_or(PluginCatalogError::CacheFormat)?;
    let signature_end = catalog_end
        .checked_add(signature_len)
        .ok_or(PluginCatalogError::CacheFormat)?;
    if signature_end != bytes.len() {
        return Err(PluginCatalogError::CacheFormat);
    }
    Ok((
        &bytes[header_len..catalog_end],
        &bytes[catalog_end..signature_end],
    ))
}

#[derive(Debug, Error)]
pub enum PluginCatalogError {
    #[error("catalog I/O failure")]
    Io(#[from] std::io::Error),
    #[error("catalog JSON is invalid")]
    Json(#[from] serde_json::Error),
    #[error("catalog JSON is not canonical")]
    NonCanonical,
    #[error("catalog validation failed")]
    Validation(#[from] markion_plugin_protocol::ValidationError),
    #[error("catalog signature is invalid")]
    InvalidSignature,
    #[error("catalog cache has an invalid format")]
    CacheFormat,
    #[error("catalog requires unsupported capability {0}")]
    UnsupportedCapability(String),
    #[error("catalog sequence rolled back from {current} to {candidate}")]
    SequenceRollback { current: u64, candidate: u64 },
    #[error("catalog sequence {0} was reused for different content")]
    SequenceReuse(u64),
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::Path};

    use markion_plugin_protocol::{
        CatalogArtifact, CatalogPlugin, FileHandlerDeclaration, LocalizedIdentity, Permission,
        PluginCatalog, ProtocolRange, canonical_json,
    };
    use semver::VersionReq;

    use super::*;

    fn catalog(sequence: u64, plugin_id: &str) -> Vec<u8> {
        canonical_json(&PluginCatalog {
            schema_version: 1,
            sequence,
            plugins: vec![CatalogPlugin {
                plugin_id: plugin_id.to_owned(),
                version: Version::new(1, 0, 0),
                publisher: "Markion".to_owned(),
                host_version: VersionReq::parse(">=0.3.9, <0.4.0").unwrap(),
                protocol: ProtocolRange::V1,
                identities: BTreeMap::from([(
                    "en".to_owned(),
                    LocalizedIdentity {
                        name: "Fixture".to_owned(),
                        description: "Fixture plugin".to_owned(),
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
                artifacts: vec![CatalogArtifact {
                    target: TargetSpec::current(),
                    url: "https://example.invalid/plugin".to_owned(),
                    length: 1,
                    sha256: "1".repeat(64),
                    installed_size_bytes: 1,
                }],
            }],
        })
        .unwrap()
    }

    fn signature(bytes: &[u8]) -> Vec<u8> {
        format!("{:x}", Sha256::digest(bytes)).into_bytes()
    }

    fn manager(root: &Path, bootstrap: Vec<u8>) -> CatalogManager {
        let bootstrap_signature = signature(&bootstrap);
        CatalogManager::with_verifier(
            PluginStorePaths::new(root),
            bootstrap,
            bootstrap_signature,
            |message, signature_bytes| {
                (signature(message) == signature_bytes)
                    .then_some(())
                    .ok_or(PluginCatalogError::InvalidSignature)
            },
            Version::new(0, 3, 9),
            ProtocolVersion::V1_0,
            TargetSpec::current(),
            &["en"],
        )
    }

    #[test]
    fn valid_refresh_is_cached_and_available_offline() {
        let root = tempfile::tempdir().unwrap();
        let manager = manager(root.path(), catalog(1, "dev.markion.pdf"));
        let next = catalog(2, "dev.markion.pdf");
        assert!(matches!(
            manager.refresh(&next, &signature(&next)).unwrap(),
            CatalogUpdate::Activated { sequence: 2, .. }
        ));
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.catalog().sequence, 2);
        assert_eq!(loaded.source(), CatalogSnapshotSource::Cached);
    }

    #[test]
    fn invalid_cached_bundle_falls_back_to_bootstrap() {
        let root = tempfile::tempdir().unwrap();
        let manager = manager(root.path(), catalog(1, "dev.markion.pdf"));
        fs::create_dir_all(root.path()).unwrap();
        fs::write(root.path().join("catalog.cache"), b"invalid").unwrap();
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.catalog().sequence, 1);
        assert_eq!(loaded.source(), CatalogSnapshotSource::Bootstrap);
    }

    #[test]
    fn rejected_refresh_preserves_the_last_verified_cached_snapshot() {
        let root = tempfile::tempdir().unwrap();
        let manager = manager(root.path(), catalog(1, "dev.markion.pdf"));
        let next = catalog(2, "dev.markion.pdf");
        manager.refresh(&next, &signature(&next)).unwrap();
        let cache_before = fs::read(root.path().join("catalog.cache")).unwrap();

        let invalid = catalog(3, "dev.markion.other");
        assert!(manager.refresh(&invalid, b"invalid").is_err());
        assert_eq!(
            fs::read(root.path().join("catalog.cache")).unwrap(),
            cache_before
        );
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.catalog().sequence, 2);
        assert_eq!(loaded.source(), CatalogSnapshotSource::Cached);
    }

    #[test]
    fn noncanonical_refresh_is_rejected_before_cache_activation() {
        let root = tempfile::tempdir().unwrap();
        let manager = manager(root.path(), catalog(1, "dev.markion.pdf"));
        let canonical = catalog(2, "dev.markion.pdf");
        let value: serde_json::Value = serde_json::from_slice(&canonical).unwrap();
        let noncanonical = serde_json::to_vec_pretty(&value).unwrap();
        assert!(matches!(
            manager.refresh(&noncanonical, &signature(&canonical)),
            Err(PluginCatalogError::NonCanonical)
        ));
        assert!(!root.path().join("catalog.cache").exists());
    }

    #[test]
    fn sequence_rollback_and_sequence_reuse_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        let manager = manager(root.path(), catalog(2, "dev.markion.pdf"));
        let old = catalog(1, "dev.markion.pdf");
        assert!(matches!(
            manager.refresh(&old, &signature(&old)),
            Err(PluginCatalogError::SequenceRollback { .. })
        ));
        let reused = catalog(2, "dev.markion.other");
        assert!(matches!(
            manager.refresh(&reused, &signature(&reused)),
            Err(PluginCatalogError::SequenceReuse(2))
        ));
    }

    #[test]
    fn conflicting_file_handler_claims_fail_validation() {
        let root = tempfile::tempdir().unwrap();
        let bytes = catalog(1, "dev.markion.pdf");
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let duplicate = value["plugins"][0].clone();
        value["plugins"].as_array_mut().unwrap().push(duplicate);
        let conflicting = canonical_json(&value).unwrap();
        let manager = manager(root.path(), conflicting);
        assert!(matches!(
            manager.load(),
            Err(PluginCatalogError::Validation(_))
        ));
    }

    #[test]
    fn unknown_capability_is_rejected_before_cache_activation() {
        let root = tempfile::tempdir().unwrap();
        let manager = manager(root.path(), catalog(1, "dev.markion.pdf"));
        let mut value: serde_json::Value =
            serde_json::from_slice(&catalog(2, "dev.markion.publisher")).unwrap();
        value["plugins"][0]["capabilities"][0] =
            serde_json::Value::String("publisher-job/v1".to_owned());
        value["plugins"][0]["file_handlers"] = serde_json::Value::Array(Vec::new());
        let unsupported = canonical_json(&value).unwrap();

        assert!(matches!(
            manager.refresh(&unsupported, &signature(&unsupported)),
            Err(PluginCatalogError::UnsupportedCapability(capability))
                if capability == "publisher-job/v1"
        ));
        assert!(!root.path().join("catalog.cache").exists());
    }
}
