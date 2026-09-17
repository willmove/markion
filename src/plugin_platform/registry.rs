use std::{collections::BTreeMap, path::Path, sync::Arc};

use markion_plugin_protocol::{
    CatalogPlugin, FileHandlerDeclaration, PluginCatalog, ProtocolVersion, TargetSpec,
};
use semver::Version;
use thiserror::Error;

use super::{PluginStore, PluginStoreError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreHandlerKind {
    Markdown,
    Text,
    Image,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreFileHandler {
    pub extension: String,
    pub icon: String,
    pub kind: CoreHandlerKind,
    pub priority: i16,
}

impl CoreFileHandler {
    pub fn new(
        extension: impl Into<String>,
        icon: impl Into<String>,
        kind: CoreHandlerKind,
    ) -> Self {
        Self {
            extension: normalize_extension(&extension.into()),
            icon: icon.into(),
            kind,
            priority: i16::MAX,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileHandlerAvailability {
    Core(CoreHandlerKind),
    Installed,
    UpdateAvailable { installed_version: Version },
    Available,
    Disabled,
    Incompatible,
    Quarantined,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileHandlerCandidate {
    pub extension: String,
    pub icon: String,
    pub priority: i16,
    pub capability: String,
    pub plugin_id: Option<String>,
    pub plugin_version: Option<Version>,
    pub publisher: Option<String>,
    pub availability: FileHandlerAvailability,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileHandlerResolution {
    Handler(FileHandlerCandidate),
    Conflict(Vec<FileHandlerCandidate>),
}

impl FileHandlerResolution {
    pub fn is_visible(&self) -> bool {
        true
    }

    pub fn installed_handler(&self) -> Option<&FileHandlerCandidate> {
        match self {
            Self::Handler(candidate)
                if matches!(
                    candidate.availability,
                    FileHandlerAvailability::Core(_)
                        | FileHandlerAvailability::Installed
                        | FileHandlerAvailability::UpdateAvailable { .. }
                ) =>
            {
                Some(candidate)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileHandlerRegistrySnapshot {
    revision: u64,
    handlers: Arc<BTreeMap<String, FileHandlerResolution>>,
}

impl FileHandlerRegistrySnapshot {
    pub fn build(
        revision: u64,
        core_handlers: impl IntoIterator<Item = CoreFileHandler>,
        catalog: &PluginCatalog,
        store: &PluginStore,
        host_version: &Version,
        protocol: ProtocolVersion,
        target: &TargetSpec,
    ) -> Result<Self, FileHandlerRegistryError> {
        let mut candidates = BTreeMap::<String, Vec<FileHandlerCandidate>>::new();
        for core in core_handlers {
            let extension = normalize_extension(&core.extension);
            validate_extension(&extension)?;
            candidates
                .entry(extension.clone())
                .or_default()
                .push(FileHandlerCandidate {
                    extension,
                    icon: core.icon,
                    priority: core.priority,
                    capability: "core".to_owned(),
                    plugin_id: None,
                    plugin_version: None,
                    publisher: None,
                    availability: FileHandlerAvailability::Core(core.kind),
                });
        }

        for plugin in &catalog.plugins {
            let state = store.state(&plugin.plugin_id)?;
            let compatible = plugin.host_version.matches(host_version)
                && plugin.protocol.supports(protocol)
                && plugin
                    .artifacts
                    .iter()
                    .any(|artifact| &artifact.target == target);
            let availability = plugin_availability(plugin, &state, compatible);
            for handler in &plugin.file_handlers {
                add_plugin_handler(&mut candidates, plugin, handler, availability.clone())?;
            }
        }

        let handlers = candidates
            .into_iter()
            .map(|(extension, mut candidates)| {
                candidates.sort_by(|left, right| {
                    right
                        .priority
                        .cmp(&left.priority)
                        .then_with(|| left.plugin_id.cmp(&right.plugin_id))
                        .then_with(|| left.plugin_version.cmp(&right.plugin_version))
                });
                let highest = candidates[0].priority;
                let mut winners = candidates
                    .into_iter()
                    .take_while(|candidate| candidate.priority == highest)
                    .collect::<Vec<_>>();
                let resolution = if winners.len() == 1 {
                    FileHandlerResolution::Handler(winners.pop().expect("one winner"))
                } else {
                    FileHandlerResolution::Conflict(winners)
                };
                (extension, resolution)
            })
            .collect::<BTreeMap<_, _>>();
        Ok(Self {
            revision,
            handlers: Arc::new(handlers),
        })
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn resolve_extension(&self, extension: &str) -> Option<&FileHandlerResolution> {
        self.handlers.get(&normalize_extension(extension))
    }

    pub fn resolve_path(&self, path: &Path) -> Option<&FileHandlerResolution> {
        let extension = path.extension()?.to_str()?;
        self.resolve_extension(extension)
    }

    pub fn handlers(&self) -> &BTreeMap<String, FileHandlerResolution> {
        &self.handlers
    }
}

fn plugin_availability(
    plugin: &CatalogPlugin,
    state: &super::PluginStoreState,
    compatible: bool,
) -> FileHandlerAvailability {
    if !compatible {
        return FileHandlerAvailability::Incompatible;
    }
    let Some(active) = state.active.as_ref() else {
        return FileHandlerAvailability::Available;
    };
    if state.quarantine.contains_key(&active.version) {
        return FileHandlerAvailability::Quarantined;
    }
    if !state.enabled {
        return FileHandlerAvailability::Disabled;
    }
    if active.version == plugin.version {
        FileHandlerAvailability::Installed
    } else {
        FileHandlerAvailability::UpdateAvailable {
            installed_version: active.version.clone(),
        }
    }
}

fn add_plugin_handler(
    candidates: &mut BTreeMap<String, Vec<FileHandlerCandidate>>,
    plugin: &CatalogPlugin,
    handler: &FileHandlerDeclaration,
    availability: FileHandlerAvailability,
) -> Result<(), FileHandlerRegistryError> {
    if !plugin
        .capabilities
        .iter()
        .any(|capability| capability == &handler.capability)
    {
        return Err(FileHandlerRegistryError::UndeclaredCapability {
            plugin_id: plugin.plugin_id.clone(),
            capability: handler.capability.clone(),
        });
    }
    for extension in &handler.extensions {
        let extension = normalize_extension(extension);
        validate_extension(&extension)?;
        candidates
            .entry(extension.clone())
            .or_default()
            .push(FileHandlerCandidate {
                extension,
                icon: handler.icon.clone(),
                priority: handler.priority,
                capability: handler.capability.clone(),
                plugin_id: Some(plugin.plugin_id.clone()),
                plugin_version: Some(plugin.version.clone()),
                publisher: Some(plugin.publisher.clone()),
                availability: availability.clone(),
            });
    }
    Ok(())
}

fn normalize_extension(extension: &str) -> String {
    let extension = extension.trim().to_ascii_lowercase();
    if extension.starts_with('.') {
        extension
    } else {
        format!(".{extension}")
    }
}

fn validate_extension(extension: &str) -> Result<(), FileHandlerRegistryError> {
    let valid = extension.starts_with('.')
        && extension.len() >= 2
        && extension.len() <= 16
        && extension[1..]
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(FileHandlerRegistryError::InvalidExtension(
            extension.to_owned(),
        ))
    }
}

#[derive(Debug, Error)]
pub enum FileHandlerRegistryError {
    #[error("plugin store state could not be loaded")]
    Store(#[from] PluginStoreError),
    #[error("file extension is invalid: {0}")]
    InvalidExtension(String),
    #[error("plugin {plugin_id} handler references undeclared capability {capability}")]
    UndeclaredCapability {
        plugin_id: String,
        capability: String,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use markion_plugin_protocol::{
        CatalogArtifact, CatalogPlugin, FileHandlerDeclaration, LocalizedIdentity, PluginCatalog,
        ProtocolRange,
    };
    use semver::VersionReq;

    use super::*;
    use crate::plugin_platform::PluginStorePaths;

    fn plugin(id: &str, priority: i16) -> CatalogPlugin {
        CatalogPlugin {
            plugin_id: id.to_owned(),
            version: Version::new(1, 0, 0),
            publisher: "Markion".to_owned(),
            host_version: VersionReq::parse(">=0.3.9, <0.4.0").unwrap(),
            protocol: ProtocolRange::V1,
            identities: BTreeMap::from([(
                "en".to_owned(),
                LocalizedIdentity {
                    name: id.to_owned(),
                    description: id.to_owned(),
                },
            )]),
            permissions: Vec::new(),
            capabilities: vec!["paged-document/v1".to_owned()],
            file_handlers: vec![FileHandlerDeclaration {
                extensions: vec![".pdf".to_owned()],
                capability: "paged-document/v1".to_owned(),
                priority,
                icon: "document-pdf".to_owned(),
            }],
            artifacts: vec![CatalogArtifact {
                target: TargetSpec::current(),
                url: "https://example.invalid/plugin".to_owned(),
                length: 10,
                sha256: "0".repeat(64),
                installed_size_bytes: 20,
            }],
        }
    }

    fn registry(plugins: Vec<CatalogPlugin>) -> FileHandlerRegistrySnapshot {
        let root = tempfile::tempdir().unwrap();
        let store = PluginStore::new(PluginStorePaths::new(root.path()));
        FileHandlerRegistrySnapshot::build(
            7,
            [CoreFileHandler::new(
                "md",
                "document-markdown",
                CoreHandlerKind::Markdown,
            )],
            &PluginCatalog {
                schema_version: 1,
                sequence: 4,
                plugins,
            },
            &store,
            &Version::new(0, 3, 9),
            ProtocolVersion::V1_0,
            &TargetSpec::current(),
        )
        .unwrap()
    }

    #[test]
    fn core_and_uninstalled_plugin_handlers_share_one_immutable_snapshot() {
        let snapshot = registry(vec![plugin("dev.markion.pdf", 100)]);
        assert_eq!(snapshot.revision(), 7);
        assert!(matches!(
            snapshot.resolve_path(Path::new("README.MD")),
            Some(FileHandlerResolution::Handler(FileHandlerCandidate {
                availability: FileHandlerAvailability::Core(CoreHandlerKind::Markdown),
                ..
            }))
        ));
        assert!(matches!(
            snapshot.resolve_path(Path::new("reference.PDF")),
            Some(FileHandlerResolution::Handler(FileHandlerCandidate {
                availability: FileHandlerAvailability::Available,
                plugin_id: Some(plugin_id),
                ..
            })) if plugin_id == "dev.markion.pdf"
        ));
    }

    #[test]
    fn equal_priority_claims_become_a_deterministic_conflict() {
        let snapshot = registry(vec![
            plugin("dev.markion.pdf", 100),
            plugin("dev.markion.second", 100),
        ]);
        let Some(FileHandlerResolution::Conflict(candidates)) = snapshot.resolve_extension("pdf")
        else {
            panic!("expected conflict");
        };
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].plugin_id.as_deref(), Some("dev.markion.pdf"));
    }

    #[test]
    fn higher_priority_claim_wins_independent_of_catalog_order() {
        let snapshot = registry(vec![
            plugin("dev.markion.low", 10),
            plugin("dev.markion.high", 20),
        ]);
        assert!(matches!(
            snapshot.resolve_extension(".pdf"),
            Some(FileHandlerResolution::Handler(FileHandlerCandidate {
                plugin_id: Some(plugin_id),
                ..
            })) if plugin_id == "dev.markion.high"
        ));
    }
}
