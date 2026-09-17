use std::collections::BTreeSet;

use semver::Version;
use thiserror::Error;

use crate::{
    CapabilityDeclaration, HandshakeAccepted, HostMessage, PluginError, PluginErrorCode,
    PluginMessage, ProtocolRange, TargetSpec,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginPeerResponse {
    pub request_id: u64,
    pub message: PluginMessage,
}

pub trait PluginPeer {
    fn exchange(
        &mut self,
        request_id: u64,
        message: HostMessage,
    ) -> Result<PluginPeerResponse, AdapterError>;
}

#[derive(Clone, Debug)]
pub struct InMemoryPluginAdapter {
    plugin_id: String,
    plugin_version: Version,
    package_sha256: String,
    protocol: ProtocolRange,
    target: TargetSpec,
    capabilities: Vec<CapabilityDeclaration>,
    handshaken: bool,
    shut_down: bool,
    seen_request_ids: BTreeSet<u64>,
}

impl InMemoryPluginAdapter {
    pub fn new(
        plugin_id: impl Into<String>,
        plugin_version: Version,
        package_sha256: impl Into<String>,
        protocol: ProtocolRange,
        capabilities: Vec<CapabilityDeclaration>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            plugin_version,
            package_sha256: package_sha256.into(),
            protocol,
            target: TargetSpec::current(),
            capabilities,
            handshaken: false,
            shut_down: false,
            seen_request_ids: BTreeSet::new(),
        }
    }

    pub fn is_handshaken(&self) -> bool {
        self.handshaken
    }

    pub fn is_shut_down(&self) -> bool {
        self.shut_down
    }
}

impl PluginPeer for InMemoryPluginAdapter {
    fn exchange(
        &mut self,
        request_id: u64,
        message: HostMessage,
    ) -> Result<PluginPeerResponse, AdapterError> {
        if self.shut_down {
            return Err(AdapterError::ShutDown);
        }
        if request_id == 0 || !self.seen_request_ids.insert(request_id) {
            return Err(AdapterError::InvalidRequestId);
        }

        let response = match message {
            HostMessage::Handshake(hello) => {
                if self.handshaken {
                    return Err(AdapterError::DuplicateHandshake);
                }
                if !self.protocol.supports(hello.protocol)
                    || hello.expected_plugin_id != self.plugin_id
                    || hello.expected_plugin_version != self.plugin_version
                    || hello.package_sha256 != self.package_sha256
                {
                    return Err(AdapterError::IncompatibleHandshake);
                }
                let available = self
                    .capabilities
                    .iter()
                    .map(|capability| capability.id.as_str())
                    .collect::<BTreeSet<_>>();
                if hello
                    .requested_capabilities
                    .iter()
                    .any(|capability| !available.contains(capability.as_str()))
                {
                    return Err(AdapterError::UnsupportedCapability);
                }
                self.handshaken = true;
                PluginMessage::HandshakeAccepted(HandshakeAccepted {
                    protocol: hello.protocol,
                    plugin_id: self.plugin_id.clone(),
                    plugin_version: self.plugin_version.clone(),
                    target: self.target.clone(),
                    package_sha256: self.package_sha256.clone(),
                    capabilities: self.capabilities.clone(),
                })
            }
            HostMessage::Ping if self.handshaken => PluginMessage::Pong,
            HostMessage::Cancel {
                request_id: cancelled,
            } if self.handshaken => PluginMessage::Cancelled {
                request_id: cancelled,
            },
            HostMessage::Shutdown if self.handshaken => {
                self.shut_down = true;
                PluginMessage::ShutdownAccepted
            }
            HostMessage::PagedDocument(_) if self.handshaken => PluginMessage::Error(PluginError {
                code: PluginErrorCode::UnsupportedCapability,
                retryable: false,
            }),
            _ => return Err(AdapterError::HandshakeRequired),
        };
        Ok(PluginPeerResponse {
            request_id,
            message: response,
        })
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AdapterError {
    #[error("request id must be non-zero and unique")]
    InvalidRequestId,
    #[error("plugin handshake is required")]
    HandshakeRequired,
    #[error("plugin already completed its handshake")]
    DuplicateHandshake,
    #[error("plugin handshake identity, digest, or version is incompatible")]
    IncompatibleHandshake,
    #[error("plugin does not provide a requested capability")]
    UnsupportedCapability,
    #[error("plugin adapter has shut down")]
    ShutDown,
}

#[cfg(test)]
mod tests {
    use crate::{HandshakeHello, ProtocolVersion, ResourceLimits};

    use super::*;

    fn adapter() -> InMemoryPluginAdapter {
        InMemoryPluginAdapter::new(
            "dev.markion.fixture",
            Version::new(1, 0, 0),
            "digest",
            ProtocolRange::V1,
            vec![CapabilityDeclaration {
                id: "paged-document/v1".to_owned(),
                limits: ResourceLimits::default(),
            }],
        )
    }

    fn hello(protocol: ProtocolVersion) -> HostMessage {
        HostMessage::Handshake(HandshakeHello {
            protocol,
            host_version: Version::new(0, 3, 9),
            expected_plugin_id: "dev.markion.fixture".to_owned(),
            expected_plugin_version: Version::new(1, 0, 0),
            package_sha256: "digest".to_owned(),
            requested_capabilities: vec!["paged-document/v1".to_owned()],
        })
    }

    #[test]
    fn handshake_correlation_cancel_and_shutdown_are_deterministic() {
        let mut adapter = adapter();
        let handshake = adapter.exchange(1, hello(ProtocolVersion::V1_0)).unwrap();
        assert_eq!(handshake.request_id, 1);
        assert!(matches!(
            handshake.message,
            PluginMessage::HandshakeAccepted(_)
        ));
        let cancel = adapter
            .exchange(2, HostMessage::Cancel { request_id: 77 })
            .unwrap();
        assert_eq!(
            cancel,
            PluginPeerResponse {
                request_id: 2,
                message: PluginMessage::Cancelled { request_id: 77 }
            }
        );
        let shutdown = adapter.exchange(3, HostMessage::Shutdown).unwrap();
        assert_eq!(shutdown.message, PluginMessage::ShutdownAccepted);
        assert!(adapter.is_shut_down());
        assert_eq!(
            adapter.exchange(4, HostMessage::Ping),
            Err(AdapterError::ShutDown)
        );
    }

    #[test]
    fn handshake_is_required_and_incompatible_versions_fail_closed() {
        let mut adapter = adapter();
        assert_eq!(
            adapter.exchange(1, HostMessage::Ping),
            Err(AdapterError::HandshakeRequired)
        );
        assert_eq!(
            adapter.exchange(2, hello(ProtocolVersion { major: 2, minor: 0 })),
            Err(AdapterError::IncompatibleHandshake)
        );
    }

    #[test]
    fn duplicate_request_ids_are_rejected() {
        let mut adapter = adapter();
        adapter.exchange(1, hello(ProtocolVersion::V1_0)).unwrap();
        assert_eq!(
            adapter.exchange(1, HostMessage::Ping),
            Err(AdapterError::InvalidRequestId)
        );
    }
}
