//! Bounded, offline DOCX to Markdown conversion.
//!
//! This crate deliberately owns no filesystem or UI side effects. It reads a
//! bounded package into memory, returns typed image assets and diagnostics,
//! and lets the application materialize resource URLs only after a save
//! destination is selected.

mod package;
mod render;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use thiserror::Error;

pub use package::PackageStats;

/// Stable machine-readable diagnostic severity. Display strings belong to
/// Markion's localization layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    Info,
    FormattingLoss,
    ContentLoss,
}

/// Stable diagnostic code independent of display language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    LayoutNormalized,
    HeadingDepthReduced,
    NumberingNormalized,
    UnsafeHyperlink,
    MissingBookmark,
    MergedTableHtml,
    ComplexTableSimplified,
    FloatingImageNormalized,
    MissingImage,
    UnsupportedImage,
    UnsupportedEquation,
    RevisionsAccepted,
    AmbiguousRevision,
    UnsupportedContent,
    ExternalResourceBlocked,
    FieldResultPreserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub part: Option<String>,
    pub location: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAsset {
    pub id: AssetId,
    pub suggested_stem: String,
    pub extension: String,
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownChunk {
    Text(String),
    AssetUrl(AssetId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    pub paragraphs: usize,
    pub tables: usize,
    pub images: usize,
    pub footnotes: usize,
    pub revisions_accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedImport {
    pub chunks: Vec<MarkdownChunk>,
    pub assets: Vec<PreparedAsset>,
    pub diagnostics: Vec<Diagnostic>,
    pub summary: ImportSummary,
    pub package: PackageStats,
}

impl PreparedImport {
    pub fn has_content_loss(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|item| item.severity == DiagnosticSeverity::ContentLoss)
    }

    pub fn render_markdown(
        &self,
        mut asset_url: impl FnMut(AssetId) -> Option<String>,
    ) -> Result<String, ImportError> {
        let mut output = String::new();
        for chunk in &self.chunks {
            match chunk {
                MarkdownChunk::Text(text) => output.push_str(text),
                MarkdownChunk::AssetUrl(id) => output.push_str(
                    asset_url(*id)
                        .ok_or(ImportError::MissingAssetReference(id.0))?
                        .as_str(),
                ),
            }
        }
        if output.len() > self.package.limits.max_markdown_bytes {
            return Err(ImportError::LimitExceeded("generated Markdown bytes"));
        }
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportLimits {
    pub max_source_bytes: usize,
    pub max_entries: usize,
    pub max_decompressed_bytes: usize,
    pub max_xml_part_bytes: usize,
    pub max_xml_depth: usize,
    pub max_style_depth: usize,
    pub max_images: usize,
    pub max_image_bytes: usize,
    pub max_total_image_bytes: usize,
    pub max_image_pixels: u64,
    pub max_markdown_bytes: usize,
    pub deadline: Duration,
}

impl Default for ImportLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 20 * 1024 * 1024,
            max_entries: 4_096,
            max_decompressed_bytes: 128 * 1024 * 1024,
            max_xml_part_bytes: 16 * 1024 * 1024,
            max_xml_depth: 128,
            max_style_depth: 64,
            max_images: 200,
            max_image_bytes: 16 * 1024 * 1024,
            max_total_image_bytes: 64 * 1024 * 1024,
            max_image_pixels: 40_000_000,
            max_markdown_bytes: 16 * 1024 * 1024,
            deadline: Duration::from_secs(30),
        }
    }
}

#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

pub(crate) struct ImportContext {
    pub limits: ImportLimits,
    pub cancellation: CancellationToken,
    started: Instant,
}

impl ImportContext {
    fn new(limits: ImportLimits, cancellation: CancellationToken) -> Self {
        Self {
            limits,
            cancellation,
            started: Instant::now(),
        }
    }

    pub(crate) fn checkpoint(&self) -> Result<(), ImportError> {
        if self.cancellation.is_cancelled() {
            return Err(ImportError::Cancelled);
        }
        if self.started.elapsed() > self.limits.deadline {
            return Err(ImportError::DeadlineExceeded);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("source exceeds DOCX import limit")]
    SourceTooLarge,
    #[error("input is not a readable DOCX package: {0}")]
    InvalidPackage(String),
    #[error("macro-enabled DOCX is not supported")]
    MacroEnabled,
    #[error("encrypted DOCX is not supported")]
    Encrypted,
    #[error("required DOCX part is missing: {0}")]
    MissingPart(String),
    #[error("DOCX relationship is invalid: {0}")]
    InvalidRelationship(String),
    #[error("DOCX XML is invalid in {part}: {message}")]
    InvalidXml { part: String, message: String },
    #[error("DOCX import limit exceeded: {0}")]
    LimitExceeded(&'static str),
    #[error("DOCX import was cancelled")]
    Cancelled,
    #[error("DOCX import deadline was exceeded")]
    DeadlineExceeded,
    #[error("no recoverable document content was found")]
    EmptyDocument,
    #[error("prepared import references missing asset {0}")]
    MissingAssetReference(usize),
}

pub fn import_docx(
    bytes: &[u8],
    limits: ImportLimits,
    cancellation: CancellationToken,
) -> Result<PreparedImport, ImportError> {
    if bytes.len() > limits.max_source_bytes {
        return Err(ImportError::SourceTooLarge);
    }
    let context = ImportContext::new(limits, cancellation);
    let package = package::DocxPackage::read(bytes, &context)?;
    render::convert(package, &context)
}

#[cfg(test)]
mod tests;
