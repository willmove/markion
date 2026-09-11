//! GPUI-free PDF metadata and rasterization service for Markion.
//!
//! PDFium is dynamically loaded from one explicit path and is owned by one
//! service thread. Public values contain only owned Rust data; native handles
//! never cross this crate boundary.

mod backend;
mod runtime;
mod service;
mod types;

pub use backend::{NativePdfBackend, PdfRendererBackend};
pub use runtime::{
    DEV_RUNTIME_ENV, PDFIUM_BUILD, RuntimeBuildMode, RuntimeDiscoveryError, discover_runtime,
    packaged_runtime_path, probe_runtime, resolve_runtime_path, runtime_library_name,
};
pub use service::{PdfCommand, PdfEvent, PdfService, SubmitOutcome};
pub use types::{
    DocumentId, Generation, MAX_COMMAND_QUEUE, MAX_PAGE_COUNT, MAX_RASTER_BYTES, MAX_SOURCE_BYTES,
    OpenRequest, OpenedDocument, PageGeometry, PageRaster, PdfErrorKind, RASTER_WIDTH_BUCKET_PX,
    RenderPriority, RenderRequest, RequestId, bounded_raster_dimensions, target_width_bucket,
};
