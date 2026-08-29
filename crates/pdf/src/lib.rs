//! markion-pdf — Markion's built-in PDF export engine.
//!
//! The root crate converts its cached preview blocks into the layout IR
//! defined in [`ir`]; this crate lays the IR out with `cosmic-text` (font
//! discovery, shaping, UAX#14 line breaking) and emits PDF through `krilla`
//! (font subsetting/embedding, images, link annotations, outline, metadata).
//! The crate is GPUI-free per the workspace invariant.

pub mod ir;

mod emit;
mod engine;
mod fonts;
mod layout;
mod raster;
mod text;
mod theme;

pub use engine::render;
pub use ir::{
    AlertKind, Alignment, Block, Cell, ImageData, InlineImage, ListMarker, PdfDocument,
    PdfMetadata, PdfOptions, Rgb, Run, Style,
};
pub use raster::{DEFAULT_SCALE, render_snapshot};

/// Error type for the built-in PDF writer. Export failures surface through
/// the root crate's user-facing status reporting.
#[derive(Debug)]
pub enum PdfError {
    /// No usable font could be loaded at all (bundled fonts are corrupted or
    /// the font database is empty).
    Fonts(String),
    /// A block could not be laid out (e.g. an image wider than the page).
    Layout(String),
    /// krilla failed to serialize the document.
    Emit(String),
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fonts(msg) => write!(f, "font setup failed: {msg}"),
            Self::Layout(msg) => write!(f, "layout failed: {msg}"),
            Self::Emit(msg) => write!(f, "PDF serialization failed: {msg}"),
        }
    }
}

impl std::error::Error for PdfError {}
