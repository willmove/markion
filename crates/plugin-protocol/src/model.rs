use std::collections::{BTreeMap, BTreeSet};

use minisign_verify::{PublicKey, Signature};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const MANIFEST_SCHEMA_VERSION: u16 = 1;
pub const CATALOG_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    pub const V1_0: Self = Self { major: 1, minor: 0 };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolRange {
    pub major: u16,
    pub min_minor: u16,
    pub max_minor: u16,
}

impl ProtocolRange {
    pub const V1: Self = Self {
        major: 1,
        min_minor: 0,
        max_minor: 0,
    };

    pub fn supports(self, version: ProtocolVersion) -> bool {
        self.major == version.major
            && self.min_minor <= version.minor
            && version.minor <= self.max_minor
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSpec {
    pub os: String,
    pub arch: String,
}

impl TargetSpec {
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_owned(),
            arch: std::env::consts::ARCH.to_owned(),
        }
    }

    pub fn matches_current(&self) -> bool {
        self == &Self::current()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedIdentity {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Permission {
    ReadSelectedFiles,
    Network,
    CredentialBroker,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimits {
    pub max_control_bytes: u32,
    pub max_body_bytes: u64,
    pub max_in_flight_requests: u16,
    pub request_timeout_ms: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_control_bytes: 64 * 1024,
            max_body_bytes: 32 * 1024 * 1024,
            max_in_flight_requests: 8,
            request_timeout_ms: 30_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityDeclaration {
    pub id: String,
    pub limits: ResourceLimits,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileHandlerDeclaration {
    pub extensions: Vec<String>,
    pub capability: String,
    pub priority: i16,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberManifest {
    pub path: String,
    pub length: u64,
    pub sha256: String,
    #[serde(default)]
    pub executable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub schema_version: u16,
    pub plugin_id: String,
    pub version: Version,
    pub publisher: String,
    pub host_version: VersionReq,
    pub protocol: ProtocolRange,
    pub target: TargetSpec,
    pub entry_point: String,
    pub identities: BTreeMap<String, LocalizedIdentity>,
    pub permissions: Vec<Permission>,
    pub capabilities: Vec<CapabilityDeclaration>,
    #[serde(default)]
    pub file_handlers: Vec<FileHandlerDeclaration>,
    pub archive_size_bytes: u64,
    pub installed_size_bytes: u64,
    pub members: Vec<MemberManifest>,
}

impl PluginManifest {
    pub fn validate(
        &self,
        host_version: &Version,
        protocol: ProtocolVersion,
        target: &TargetSpec,
        required_locales: &[&str],
    ) -> Result<(), ValidationError> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(ValidationError::UnsupportedManifestSchema(
                self.schema_version,
            ));
        }
        validate_identifier(&self.plugin_id, "plugin id")?;
        if self.publisher.trim().is_empty() {
            return Err(ValidationError::EmptyField("publisher"));
        }
        if !self.host_version.matches(host_version) {
            return Err(ValidationError::IncompatibleHost);
        }
        if !self.protocol.supports(protocol) {
            return Err(ValidationError::IncompatibleProtocol);
        }
        if &self.target != target {
            return Err(ValidationError::WrongTarget);
        }
        validate_package_path(&self.entry_point)?;

        let mut member_paths = BTreeSet::new();
        for member in &self.members {
            validate_package_path(&member.path)?;
            let folded = member.path.to_ascii_lowercase();
            if !member_paths.insert(folded) {
                return Err(ValidationError::DuplicatePath(member.path.clone()));
            }
            if member.sha256.len() != 64
                || !member.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(ValidationError::InvalidDigest(member.path.clone()));
            }
        }
        let entry = self
            .members
            .iter()
            .find(|member| member.path == self.entry_point)
            .ok_or(ValidationError::MissingEntryPoint)?;
        if !entry.executable {
            return Err(ValidationError::EntryPointNotExecutable);
        }

        let mut capabilities = BTreeSet::new();
        for capability in &self.capabilities {
            validate_identifier(&capability.id, "capability")?;
            if !capabilities.insert(capability.id.as_str()) {
                return Err(ValidationError::DuplicateCapability(capability.id.clone()));
            }
            validate_limits(&capability.limits)?;
        }
        if capabilities.is_empty() {
            return Err(ValidationError::MissingCapability);
        }

        let mut extensions = BTreeSet::new();
        for handler in &self.file_handlers {
            if !capabilities.contains(handler.capability.as_str()) {
                return Err(ValidationError::UndeclaredCapability(
                    handler.capability.clone(),
                ));
            }
            if handler.extensions.is_empty() {
                return Err(ValidationError::EmptyFileHandler);
            }
            for extension in &handler.extensions {
                validate_extension(extension)?;
                if !extensions.insert(extension.to_ascii_lowercase()) {
                    return Err(ValidationError::DuplicateExtension(extension.clone()));
                }
            }
        }

        for locale in required_locales {
            let Some(identity) = self.identities.get(*locale) else {
                return Err(ValidationError::MissingLocale((*locale).to_owned()));
            };
            validate_plain_identity(locale, identity)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogArtifact {
    pub target: TargetSpec,
    pub url: String,
    pub length: u64,
    pub sha256: String,
    pub installed_size_bytes: u64,
}

impl CatalogArtifact {
    /// Bootstrap catalogs may carry a zeroed, non-installable target slot so
    /// file types stay discoverable before that target's first artifact is
    /// published. Install flows must accept only fully published records.
    pub fn is_published(&self) -> bool {
        self.length > 0
            && self.installed_size_bytes > 0
            && valid_sha256(&self.sha256)
            && self.sha256.bytes().any(|byte| byte != b'0')
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogPlugin {
    pub plugin_id: String,
    pub version: Version,
    pub publisher: String,
    pub host_version: VersionReq,
    pub protocol: ProtocolRange,
    pub identities: BTreeMap<String, LocalizedIdentity>,
    pub permissions: Vec<Permission>,
    pub capabilities: Vec<String>,
    pub file_handlers: Vec<FileHandlerDeclaration>,
    pub artifacts: Vec<CatalogArtifact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginCatalog {
    pub schema_version: u16,
    pub sequence: u64,
    pub plugins: Vec<CatalogPlugin>,
}

impl PluginCatalog {
    pub fn validate(
        &self,
        host_version: &Version,
        protocol: ProtocolVersion,
        target: &TargetSpec,
        required_locales: &[&str],
    ) -> Result<(), ValidationError> {
        if self.schema_version != CATALOG_SCHEMA_VERSION {
            return Err(ValidationError::UnsupportedCatalogSchema(
                self.schema_version,
            ));
        }
        if self.sequence == 0 {
            return Err(ValidationError::InvalidCatalogSequence);
        }
        let mut plugin_ids = BTreeSet::new();
        let mut extension_claims = BTreeSet::new();
        for plugin in &self.plugins {
            validate_identifier(&plugin.plugin_id, "plugin id")?;
            if !plugin_ids.insert(plugin.plugin_id.as_str()) {
                return Err(ValidationError::DuplicatePluginId(plugin.plugin_id.clone()));
            }
            if plugin.publisher.trim().is_empty() {
                return Err(ValidationError::EmptyField("publisher"));
            }
            let mut permissions = BTreeSet::new();
            for permission in &plugin.permissions {
                let encoded =
                    serde_json::to_string(permission).map_err(|_| ValidationError::Json)?;
                if !permissions.insert(encoded) {
                    return Err(ValidationError::DuplicatePermission);
                }
            }
            let mut capabilities = BTreeSet::new();
            for capability in &plugin.capabilities {
                validate_identifier(capability, "capability")?;
                if !capabilities.insert(capability.as_str()) {
                    return Err(ValidationError::DuplicateCapability(capability.clone()));
                }
            }
            if capabilities.is_empty() {
                return Err(ValidationError::MissingCapability);
            }
            for locale in required_locales {
                let Some(identity) = plugin.identities.get(*locale) else {
                    return Err(ValidationError::MissingLocale((*locale).to_owned()));
                };
                validate_plain_identity(locale, identity)?;
            }
            let mut plugin_extensions = BTreeSet::new();
            for handler in &plugin.file_handlers {
                if !capabilities.contains(handler.capability.as_str()) {
                    return Err(ValidationError::UndeclaredCapability(
                        handler.capability.clone(),
                    ));
                }
                validate_identifier(&handler.icon, "file handler icon")?;
                if handler.extensions.is_empty() {
                    return Err(ValidationError::EmptyFileHandler);
                }
                for extension in &handler.extensions {
                    validate_extension(extension)?;
                    if !plugin_extensions.insert(extension.to_ascii_lowercase()) {
                        return Err(ValidationError::DuplicateExtension(extension.clone()));
                    }
                    let claim = (extension.to_ascii_lowercase(), handler.priority);
                    if !extension_claims.insert(claim) {
                        return Err(ValidationError::ConflictingFileHandler(extension.clone()));
                    }
                }
            }
            let mut artifact_targets = BTreeSet::new();
            for artifact in &plugin.artifacts {
                validate_target_component(&artifact.target.os, "artifact operating system")?;
                validate_target_component(&artifact.target.arch, "artifact architecture")?;
                let target_key = (&artifact.target.os, &artifact.target.arch);
                if !artifact_targets.insert(target_key) {
                    return Err(ValidationError::DuplicateTargetArtifact(
                        plugin.plugin_id.clone(),
                    ));
                }
                validate_catalog_artifact(artifact)?;
            }
            if plugin.host_version.matches(host_version) && plugin.protocol.supports(protocol) {
                let matches = plugin
                    .artifacts
                    .iter()
                    .filter(|artifact| &artifact.target == target)
                    .count();
                if matches > 1 {
                    return Err(ValidationError::DuplicateTargetArtifact(
                        plugin.plugin_id.clone(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum HostMessage {
    Handshake(HandshakeHello),
    Ping,
    Cancel { request_id: u64 },
    Shutdown,
    PagedDocument(PagedDocumentRequest),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PluginMessage {
    HandshakeAccepted(HandshakeAccepted),
    Pong,
    Cancelled { request_id: u64 },
    ShutdownAccepted,
    PagedDocument(PagedDocumentResponse),
    Error(PluginError),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandshakeHello {
    pub protocol: ProtocolVersion,
    pub host_version: Version,
    pub expected_plugin_id: String,
    pub expected_plugin_version: Version,
    pub package_sha256: String,
    pub requested_capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandshakeAccepted {
    pub protocol: ProtocolVersion,
    pub plugin_id: String,
    pub plugin_version: Version,
    pub target: TargetSpec,
    pub package_sha256: String,
    pub capabilities: Vec<CapabilityDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PagedDocumentRequest {
    Open {
        path: String,
        max_source_bytes: u64,
        max_pages: u32,
    },
    Render {
        document_token: u64,
        generation: u64,
        page_index: u32,
        width_px: u32,
        max_raster_bytes: u64,
    },
    Close {
        document_token: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PagedDocumentResponse {
    Opened {
        document_token: u64,
        pages: Vec<PageGeometry>,
    },
    Rendered {
        document_token: u64,
        generation: u64,
        page_index: u32,
        raster: RasterDescriptor,
    },
    Closed {
        document_token: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageGeometry {
    pub width_points: u32,
    pub height_points: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PixelFormat {
    Rgba8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RasterDescriptor {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixel_format: PixelFormat,
    pub body_length: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginErrorCode {
    InvalidRequest,
    IncompatibleProtocol,
    UnsupportedCapability,
    SourceTooLarge,
    TooManyPages,
    RasterTooLarge,
    CorruptDocument,
    EncryptedDocument,
    RuntimeUnavailable,
    Cancelled,
    Internal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginError {
    pub code: PluginErrorCode,
    pub retryable: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("unsupported manifest schema {0}")]
    UnsupportedManifestSchema(u16),
    #[error("unsupported catalog schema {0}")]
    UnsupportedCatalogSchema(u16),
    #[error("catalog sequence must be greater than zero")]
    InvalidCatalogSequence,
    #[error("{0} is empty")]
    EmptyField(&'static str),
    #[error("invalid {field}: {value}")]
    InvalidIdentifier { field: &'static str, value: String },
    #[error("host version is incompatible")]
    IncompatibleHost,
    #[error("plugin protocol is incompatible")]
    IncompatibleProtocol,
    #[error("plugin target does not match the host")]
    WrongTarget,
    #[error("invalid package path {0}")]
    InvalidPath(String),
    #[error("duplicate or case-colliding path {0}")]
    DuplicatePath(String),
    #[error("invalid digest for {0}")]
    InvalidDigest(String),
    #[error("manifest entry point is absent from members")]
    MissingEntryPoint,
    #[error("manifest entry point is not executable")]
    EntryPointNotExecutable,
    #[error("manifest declares no capability")]
    MissingCapability,
    #[error("duplicate capability {0}")]
    DuplicateCapability(String),
    #[error("duplicate permission")]
    DuplicatePermission,
    #[error("duplicate plugin id {0}")]
    DuplicatePluginId(String),
    #[error("invalid resource limits")]
    InvalidLimits,
    #[error("file handler references undeclared capability {0}")]
    UndeclaredCapability(String),
    #[error("file handler declares no extension")]
    EmptyFileHandler,
    #[error("invalid extension {0}")]
    InvalidExtension(String),
    #[error("duplicate extension {0}")]
    DuplicateExtension(String),
    #[error("conflicting file handler for {0}")]
    ConflictingFileHandler(String),
    #[error("duplicate target artifact for {0}")]
    DuplicateTargetArtifact(String),
    #[error("invalid catalog artifact URL {0}")]
    InvalidArtifactUrl(String),
    #[error("invalid catalog artifact size declaration")]
    InvalidArtifactSize,
    #[error("missing locale {0}")]
    MissingLocale(String),
    #[error("unsafe or empty localized identity for {0}")]
    InvalidLocale(String),
    #[error("canonical JSON cannot contain floating-point numbers")]
    FloatingPointNumber,
    #[error("unable to serialize canonical JSON")]
    Json,
    #[error("signature or public key is invalid")]
    InvalidSignature,
}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ValidationError> {
    let value = serde_json::to_value(value).map_err(|_| ValidationError::Json)?;
    canonical_json_value(&value)
}

pub fn canonical_json_value(value: &Value) -> Result<Vec<u8>, ValidationError> {
    validate_json_numbers(value)?;
    let sorted = sort_json_value(value);
    serde_json::to_vec(&sorted).map_err(|_| ValidationError::Json)
}

fn sort_json_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(sort_json_value).collect()),
        Value::Object(values) => {
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            let mut sorted = serde_json::Map::new();
            for key in keys {
                sorted.insert(key.clone(), sort_json_value(&values[key]));
            }
            Value::Object(sorted)
        }
        _ => value.clone(),
    }
}

pub fn verify_minisign(
    public_key_text: &str,
    signature_text: &str,
    message: &[u8],
) -> Result<(), ValidationError> {
    let key = PublicKey::decode(public_key_text).map_err(|_| ValidationError::InvalidSignature)?;
    let signature =
        Signature::decode(signature_text).map_err(|_| ValidationError::InvalidSignature)?;
    key.verify(message, &signature, false)
        .map_err(|_| ValidationError::InvalidSignature)
}

fn validate_identifier(value: &str, field: &'static str) -> Result<(), ValidationError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'/')
        })
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.as_bytes()[value.len() - 1].is_ascii_alphanumeric();
    if valid {
        Ok(())
    } else {
        Err(ValidationError::InvalidIdentifier {
            field,
            value: value.to_owned(),
        })
    }
}

fn validate_catalog_artifact(artifact: &CatalogArtifact) -> Result<(), ValidationError> {
    let valid_url = artifact.url.starts_with("https://")
        && artifact.url.len() <= 2_048
        && !artifact
            .url
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace());
    if !valid_url {
        return Err(ValidationError::InvalidArtifactUrl(artifact.url.clone()));
    }
    let unpublished = artifact.length == 0
        && artifact.installed_size_bytes == 0
        && artifact.sha256 == "0".repeat(64);
    if !unpublished && !artifact.is_published() {
        if !valid_sha256(&artifact.sha256) {
            return Err(ValidationError::InvalidDigest(artifact.url.clone()));
        }
        return Err(ValidationError::InvalidArtifactSize);
    }
    Ok(())
}

fn validate_target_component(value: &str, field: &'static str) -> Result<(), ValidationError> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        });
    if valid {
        Ok(())
    } else {
        Err(ValidationError::InvalidIdentifier {
            field,
            value: value.to_owned(),
        })
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn validate_package_path(path: &str) -> Result<(), ValidationError> {
    let invalid = path.is_empty()
        || path.len() > 240
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..");
    if invalid {
        Err(ValidationError::InvalidPath(path.to_owned()))
    } else {
        Ok(())
    }
}

fn validate_extension(extension: &str) -> Result<(), ValidationError> {
    let valid = extension.starts_with('.')
        && extension.len() >= 2
        && extension.len() <= 16
        && extension[1..]
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(ValidationError::InvalidExtension(extension.to_owned()))
    }
}

fn validate_limits(limits: &ResourceLimits) -> Result<(), ValidationError> {
    let valid = (256..=1024 * 1024).contains(&limits.max_control_bytes)
        && (1..=64 * 1024 * 1024).contains(&limits.max_body_bytes)
        && (1..=64).contains(&limits.max_in_flight_requests)
        && (100..=5 * 60 * 1000).contains(&limits.request_timeout_ms);
    if valid {
        Ok(())
    } else {
        Err(ValidationError::InvalidLimits)
    }
}

fn validate_plain_identity(
    locale: &str,
    identity: &LocalizedIdentity,
) -> Result<(), ValidationError> {
    fn safe(value: &str, max: usize) -> bool {
        !value.trim().is_empty()
            && value.len() <= max
            && !value.chars().any(char::is_control)
            && !value.contains('<')
            && !value.contains('>')
    }
    if safe(&identity.name, 80) && safe(&identity.description, 500) {
        Ok(())
    } else {
        Err(ValidationError::InvalidLocale(locale.to_owned()))
    }
}

fn validate_json_numbers(value: &Value) -> Result<(), ValidationError> {
    match value {
        Value::Number(number) if number.is_f64() => Err(ValidationError::FloatingPointNumber),
        Value::Array(values) => values.iter().try_for_each(validate_json_numbers),
        Value::Object(values) => values.values().try_for_each(validate_json_numbers),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> PluginManifest {
        PluginManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            plugin_id: "dev.markion.fixture".to_owned(),
            version: Version::new(1, 0, 0),
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
            file_handlers: vec![FileHandlerDeclaration {
                extensions: vec![".pdf".to_owned()],
                capability: "paged-document/v1".to_owned(),
                priority: 100,
                icon: "document-pdf".to_owned(),
            }],
            archive_size_bytes: 1024,
            installed_size_bytes: 2048,
            members: vec![MemberManifest {
                path: "bin/worker".to_owned(),
                length: 1,
                sha256: "0".repeat(64),
                executable: true,
            }],
        }
    }

    fn valid_catalog() -> PluginCatalog {
        PluginCatalog {
            schema_version: CATALOG_SCHEMA_VERSION,
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
                        description: "Read PDF documents.".to_owned(),
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
                    url: "https://example.invalid/plugin.markion-plugin".to_owned(),
                    length: 10,
                    sha256: "1".repeat(64),
                    installed_size_bytes: 20,
                }],
            }],
        }
    }

    #[test]
    fn canonical_json_is_stable_and_sorted() {
        let value = serde_json::json!({"z": 1, "a": {"y": 2, "b": 3}});
        assert_eq!(
            canonical_json_value(&value).unwrap(),
            br#"{"a":{"b":3,"y":2},"z":1}"#
        );
    }

    #[test]
    fn canonical_json_rejects_floats() {
        assert_eq!(
            canonical_json_value(&serde_json::json!({"value": 1.5})),
            Err(ValidationError::FloatingPointNumber)
        );
    }

    #[test]
    fn minisign_reference_signature_verifies_and_tampering_fails() {
        let public_key = "untrusted comment: minisign public key: A7090F0642B4E81F\nRWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
        let signature = "untrusted comment: signature from minisign secret key\nRUQf6LRCGA9i559r3g7V1qNyJDApGip8MfqcadIgT9CuhV3EMhHoN1mGTkUidF/z7SrlQgXdy8ofjb7bNJJylDOocrCo8KLzZwo=\ntrusted comment: timestamp:1633700835\tfile:test\tprehashed\nwLMDjy9FLAuxZ3q4NlEvkgtyhrr0gtTu6KC4KBJdITbbOeAi1zBIYo0v4iTgt8jJpIidRJnp94ABQkJAgAooBQ==";
        verify_minisign(public_key, signature, b"test").unwrap();
        assert_eq!(
            verify_minisign(public_key, signature, b"tampered"),
            Err(ValidationError::InvalidSignature)
        );
        let wrong_key = public_key.replace("GFO3", "GFO4");
        assert_eq!(
            verify_minisign(&wrong_key, signature, b"test"),
            Err(ValidationError::InvalidSignature)
        );
    }

    #[test]
    fn manifest_rejects_incompatible_host_protocol_target_and_missing_locale() {
        let manifest = valid_manifest();
        assert_eq!(
            manifest.validate(
                &Version::new(0, 4, 0),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
            ),
            Err(ValidationError::IncompatibleHost)
        );
        assert_eq!(
            manifest.validate(
                &Version::new(0, 3, 9),
                ProtocolVersion { major: 2, minor: 0 },
                &TargetSpec::current(),
                &["en"],
            ),
            Err(ValidationError::IncompatibleProtocol)
        );
        assert_eq!(
            manifest.validate(
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec {
                    os: "foreign".to_owned(),
                    arch: "foreign".to_owned(),
                },
                &["en"],
            ),
            Err(ValidationError::WrongTarget)
        );
        assert_eq!(
            manifest.validate(
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en", "ja"],
            ),
            Err(ValidationError::MissingLocale("ja".to_owned()))
        );
    }

    #[test]
    fn manifest_and_catalog_canonical_serialization_is_deterministic() {
        let manifest = valid_manifest();
        assert_eq!(
            canonical_json(&manifest).unwrap(),
            canonical_json(&manifest).unwrap()
        );
        let catalog = PluginCatalog {
            schema_version: CATALOG_SCHEMA_VERSION,
            sequence: 1,
            plugins: Vec::new(),
        };
        assert_eq!(
            canonical_json(&catalog).unwrap(),
            canonical_json(&catalog).unwrap()
        );
    }

    #[test]
    fn catalog_rejects_undeclared_handlers_and_partial_artifacts() {
        let mut catalog = valid_catalog();
        catalog.plugins[0].file_handlers[0].capability = "publisher-job/v1".to_owned();
        assert_eq!(
            catalog.validate(
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
            ),
            Err(ValidationError::UndeclaredCapability(
                "publisher-job/v1".to_owned()
            ))
        );

        let mut catalog = valid_catalog();
        catalog.plugins[0].artifacts[0].length = 0;
        assert_eq!(
            catalog.validate(
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
            ),
            Err(ValidationError::InvalidArtifactSize)
        );
    }

    #[test]
    fn catalog_rejects_duplicate_plugin_ids_even_across_versions() {
        let mut catalog = valid_catalog();
        let mut duplicate = catalog.plugins[0].clone();
        duplicate.version = Version::new(2, 0, 0);
        duplicate.file_handlers[0].extensions = vec![".epub".to_owned()];
        catalog.plugins.push(duplicate);
        assert_eq!(
            catalog.validate(
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
            ),
            Err(ValidationError::DuplicatePluginId(
                "dev.markion.pdf".to_owned()
            ))
        );
    }

    #[test]
    fn catalog_allows_explicit_unpublished_target_slots_but_never_marks_them_published() {
        let mut catalog = valid_catalog();
        let artifact = &mut catalog.plugins[0].artifacts[0];
        artifact.length = 0;
        artifact.installed_size_bytes = 0;
        artifact.sha256 = "0".repeat(64);
        assert!(!artifact.is_published());
        catalog
            .validate(
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
            )
            .unwrap();
    }
}
