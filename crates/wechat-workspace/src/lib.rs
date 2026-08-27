//! Secure, GPUI-free loopback workspace for local WeChat publishing.

mod assets;
mod resource;
mod server;
mod session;

pub use assets::{
    BundleError, BundleFile, BundleManifest, BundleVerification, ThirdPartyComponent,
    discover_workspace_assets, verify_bundle, verify_launch_gate,
};
pub use resource::{PublishingResource, ResourceBytes, ResourceError};
pub use server::{LaunchSession, WorkspaceConfig, WorkspaceError, WorkspaceService};
pub use session::{
    Clock, DocumentPayload, ManualClock, OsTokenSource, PublishingSnapshot, ResourceMetadata,
    SessionLimits, SystemClock, TokenSource,
};
