//! Versioned, GPUI-free interfaces shared by Markion and official plugin workers.
//!
//! This crate deliberately contains no application UI types. It owns the signed
//! package vocabulary, strict validation, and bounded pipe framing used at the
//! process boundary.

mod adapter;
mod frame;
mod model;
mod package;

pub use frame::{
    FIXED_FRAME_HEADER_BYTES, Frame, FrameError, FrameKind, FrameLimits, read_frame, write_frame,
};
pub use model::{
    CapabilityDeclaration, CatalogArtifact, CatalogPlugin, FileHandlerDeclaration,
    HandshakeAccepted, HandshakeHello, HostMessage, LocalizedIdentity, MemberManifest,
    PagedDocumentRequest, PagedDocumentResponse, Permission, PixelFormat, PluginCatalog,
    PluginError, PluginErrorCode, PluginManifest, PluginMessage, ProtocolRange, ProtocolVersion,
    RasterDescriptor, ResourceLimits, TargetSpec, ValidationError, canonical_json,
    canonical_json_value, verify_minisign,
};
pub use package::{
    PackageError, PackageLimits, VerifiedMember, VerifiedPackage, inspect_package,
    inspect_package_with,
};

pub const PLUGIN_MANIFEST_NAME: &str = "plugin.json";
pub const PLUGIN_SIGNATURE_NAME: &str = "plugin.json.minisig";
pub const PAGED_DOCUMENT_CAPABILITY: &str = "paged-document/v1";
pub use adapter::{AdapterError, InMemoryPluginAdapter, PluginPeer, PluginPeerResponse};
