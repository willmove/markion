use std::{fs, sync::Arc};

use markion_plugin_protocol::{
    PluginCatalog, ProtocolVersion, TargetSpec, canonical_json, verify_minisign,
};
use semver::Version;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::storage::atomic_write;

use super::PluginStorePaths;

type SignatureVerifier = dyn Fn(&[u8], &[u8]) -> Result<(), PluginCatalogError> + Send + Sync;

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
        let cached_json = match fs::read(self.paths.catalog_path()) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(bootstrap),
            Err(error) => return Err(error.into()),
        };
        let cached_signature = match fs::read(self.paths.catalog_signature_path()) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(bootstrap),
        };
        match self.verify(
            &cached_json,
            &cached_signature,
            CatalogSnapshotSource::Cached,
        ) {
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
        // Write the signature first and commit the canonical catalog last. A
        // crash between these writes produces a mismatched pair, which load()
        // rejects in favor of the signed bootstrap rather than half-activating.
        atomic_write(self.paths.catalog_signature_path(), signature)?;
        atomic_write(self.paths.catalog_path(), &candidate.canonical_bytes)?;
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
        let catalog: PluginCatalog = serde_json::from_slice(json)?;
        let canonical = canonical_json(&catalog)?;
        (self.verifier)(&canonical, signature)?;
        let locales = self
            .required_locales
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        catalog.validate(&self.host_version, self.protocol, &self.target, &locales)?;
        let sha256 = format!("{:x}", Sha256::digest(&canonical));
        Ok(CatalogSnapshot {
            catalog: Arc::new(catalog),
            canonical_bytes: canonical.into(),
            sha256,
            source,
        })
    }
}

#[derive(Debug, Error)]
pub enum PluginCatalogError {
    #[error("catalog I/O failure")]
    Io(#[from] std::io::Error),
    #[error("catalog JSON is invalid")]
    Json(#[from] serde_json::Error),
    #[error("catalog validation failed")]
    Validation(#[from] markion_plugin_protocol::ValidationError),
    #[error("catalog signature is invalid")]
    InvalidSignature,
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
                    sha256: "0".repeat(64),
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
    fn invalid_cached_pair_falls_back_to_bootstrap() {
        let root = tempfile::tempdir().unwrap();
        let manager = manager(root.path(), catalog(1, "dev.markion.pdf"));
        fs::create_dir_all(root.path()).unwrap();
        fs::write(
            root.path().join("catalog.json"),
            catalog(2, "dev.markion.pdf"),
        )
        .unwrap();
        fs::write(root.path().join("catalog.json.minisig"), b"invalid").unwrap();
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.catalog().sequence, 1);
        assert_eq!(loaded.source(), CatalogSnapshotSource::Bootstrap);
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
}
