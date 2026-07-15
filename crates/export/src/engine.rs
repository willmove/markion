//! Core export engine: trait, formats, options, and registry.

use std::collections::HashMap;
use std::path::PathBuf;

use markdown::Document;
use tracing;

use crate::error::{ExportError, ExportErrorContext, ExportStep};

// ---------------------------------------------------------------------------
// ImageFormat
// ---------------------------------------------------------------------------

/// Supported image output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::Png => write!(f, "PNG"),
            ImageFormat::Jpeg => write!(f, "JPEG"),
        }
    }
}

// ---------------------------------------------------------------------------
// ExportFormat
// ---------------------------------------------------------------------------

/// All supported export formats.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    Pdf,
    Html,
    Docx,
    Latex,
    Image(ImageFormat),
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Pdf => write!(f, "PDF"),
            ExportFormat::Html => write!(f, "HTML"),
            ExportFormat::Docx => write!(f, "DOCX"),
            ExportFormat::Latex => write!(f, "LaTeX"),
            ExportFormat::Image(fmt) => write!(f, "Image({})", fmt),
        }
    }
}

// ---------------------------------------------------------------------------
// ExportOptions
// ---------------------------------------------------------------------------

/// Configuration options for an export operation.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Optional output file path.
    pub output_path: Option<PathBuf>,
    /// Whether to include default stylesheet/styles in the output.
    pub include_styles: bool,
    /// Whether to include document metadata (title, author, date) in the output.
    pub include_metadata: bool,
    /// Optional custom CSS to embed (primarily for HTML export).
    pub custom_css: Option<String>,
    /// Page size for paged formats (e.g. PDF). Defaults to A4.
    pub page_size: PageSize,
    /// Whether to embed images inline (base64) or reference them externally.
    pub embed_images: bool,
    /// Optional document title override.
    pub title_override: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            output_path: None,
            include_styles: true,
            include_metadata: true,
            custom_css: None,
            page_size: PageSize::A4,
            embed_images: false,
            title_override: None,
        }
    }
}

/// Standard page sizes for paged export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
    Custom { width_mm: u32, height_mm: u32 },
}

// ---------------------------------------------------------------------------
// Exporter trait
// ---------------------------------------------------------------------------

/// Trait that all format-specific exporters must implement.
pub trait Exporter: Send + Sync {
    /// Export the given document to bytes using the provided options.
    fn export(&self, document: &Document, options: &ExportOptions) -> Result<Vec<u8>, ExportError>;

    /// Returns the format this exporter handles.
    fn supported_format(&self) -> ExportFormat;
}

// ---------------------------------------------------------------------------
// ExportWithFallbackResult
// ---------------------------------------------------------------------------

/// Result of an export operation that may have fallen back to HTML.
#[derive(Debug)]
pub struct ExportWithFallbackResult {
    /// The exported data bytes.
    pub data: Vec<u8>,
    /// The actual format that was exported (may differ from requested if fallback was used).
    pub actual_format: ExportFormat,
    /// Whether the HTML fallback was used instead of the requested format.
    pub fallback_used: bool,
    /// The original error if a fallback was triggered.
    pub original_error: Option<ExportErrorContext>,
}

// ---------------------------------------------------------------------------
// ExportEngine
// ---------------------------------------------------------------------------

/// Registry and dispatcher for format-specific exporters.
pub struct ExportEngine {
    exporters: HashMap<ExportFormat, Box<dyn Exporter>>,
}

impl ExportEngine {
    /// Creates a new empty export engine with no registered exporters.
    pub fn new() -> Self {
        Self {
            exporters: HashMap::new(),
        }
    }

    /// Registers an exporter for its supported format.
    /// If an exporter for the same format already exists, it is replaced.
    pub fn register_exporter(&mut self, exporter: Box<dyn Exporter>) {
        let format = exporter.supported_format();
        self.exporters.insert(format, exporter);
    }

    /// Export a document to the specified format.
    ///
    /// Returns `ExportError::UnsupportedFormat` if no exporter is registered
    /// for the requested format.
    pub fn export(
        &self,
        document: &Document,
        format: &ExportFormat,
        options: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        tracing::info!(format = %format, "Starting export");

        let exporter = self
            .exporters
            .get(format)
            .ok_or_else(|| {
                let err = ExportError::UnsupportedFormat(format.to_string());
                tracing::error!(format = %format, error = %err, "Export failed: no exporter registered");
                err
            })?;

        match exporter.export(document, options) {
            Ok(data) => {
                tracing::info!(format = %format, bytes = data.len(), "Export completed successfully");
                Ok(data)
            }
            Err(err) => {
                tracing::error!(format = %format, error = %err, "Export failed during generation");
                Err(err)
            }
        }
    }

    /// Export a document to the specified format, falling back to HTML on failure.
    ///
    /// If the primary export fails, this method automatically attempts to export
    /// as HTML. Returns an `ExportWithFallbackResult` indicating which format was
    /// actually produced and whether a fallback occurred.
    pub fn export_with_fallback(
        &self,
        document: &Document,
        format: &ExportFormat,
        options: &ExportOptions,
    ) -> Result<ExportWithFallbackResult, ExportError> {
        // Try the primary export first
        match self.export(document, format, options) {
            Ok(data) => Ok(ExportWithFallbackResult {
                data,
                actual_format: format.clone(),
                fallback_used: false,
                original_error: None,
            }),
            Err(err) => {
                // If already requesting HTML, no fallback available
                if *format == ExportFormat::Html {
                    return Err(err);
                }

                let error_context =
                    ExportErrorContext::new(err, format.to_string(), ExportStep::OutputGeneration)
                        .with_suggestion("Automatically falling back to HTML export");

                tracing::warn!(
                    original_format = %format,
                    error = %error_context,
                    "Primary export failed, attempting HTML fallback"
                );

                // Attempt HTML fallback
                match self.export(document, &ExportFormat::Html, options) {
                    Ok(data) => {
                        tracing::info!("HTML fallback export succeeded");
                        Ok(ExportWithFallbackResult {
                            data,
                            actual_format: ExportFormat::Html,
                            fallback_used: true,
                            original_error: Some(error_context),
                        })
                    }
                    Err(fallback_err) => {
                        tracing::error!(
                            error = %fallback_err,
                            "HTML fallback export also failed"
                        );
                        Err(fallback_err)
                    }
                }
            }
        }
    }

    /// Returns a list of all currently registered export formats.
    pub fn available_formats(&self) -> Vec<&ExportFormat> {
        self.exporters.keys().collect()
    }
}

impl Default for ExportEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::{Block, Document, Inline};

    /// A trivial test exporter that returns the document text as bytes.
    struct MockExporter {
        format: ExportFormat,
    }

    impl Exporter for MockExporter {
        fn export(
            &self,
            document: &Document,
            _options: &ExportOptions,
        ) -> Result<Vec<u8>, ExportError> {
            // Simple: concatenate all paragraph text
            let mut output = String::new();
            for block in &document.blocks {
                if let Block::Paragraph { content, .. } = block {
                    for inline in content {
                        if let Inline::Text(text) = inline {
                            output.push_str(text);
                        }
                    }
                }
            }
            Ok(output.into_bytes())
        }

        fn supported_format(&self) -> ExportFormat {
            self.format.clone()
        }
    }

    #[test]
    fn engine_new_has_no_exporters() {
        let engine = ExportEngine::new();
        assert!(engine.available_formats().is_empty());
    }

    #[test]
    fn register_exporter_adds_format() {
        let mut engine = ExportEngine::new();
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Html,
        }));
        assert_eq!(engine.available_formats().len(), 1);
        assert!(engine.available_formats().contains(&&ExportFormat::Html));
    }

    #[test]
    fn register_exporter_replaces_existing() {
        let mut engine = ExportEngine::new();
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Html,
        }));
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Html,
        }));
        // Should still be only 1 exporter for Html
        assert_eq!(engine.available_formats().len(), 1);
    }

    #[test]
    fn export_with_no_exporter_returns_unsupported_error() {
        let engine = ExportEngine::new();
        let doc = Document::new(vec![]);
        let options = ExportOptions::default();

        let result = engine.export(&doc, &ExportFormat::Pdf, &options);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ExportError::UnsupportedFormat(_)));
    }

    #[test]
    fn export_with_registered_exporter_succeeds() {
        let mut engine = ExportEngine::new();
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Html,
        }));

        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Text("Hello, world!".into())],
            id: 0,
        }]);
        let options = ExportOptions::default();

        let result = engine.export(&doc, &ExportFormat::Html, &options);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"Hello, world!");
    }

    #[test]
    fn export_format_display() {
        assert_eq!(ExportFormat::Pdf.to_string(), "PDF");
        assert_eq!(ExportFormat::Html.to_string(), "HTML");
        assert_eq!(ExportFormat::Docx.to_string(), "DOCX");
        assert_eq!(ExportFormat::Latex.to_string(), "LaTeX");
        assert_eq!(
            ExportFormat::Image(ImageFormat::Png).to_string(),
            "Image(PNG)"
        );
        assert_eq!(
            ExportFormat::Image(ImageFormat::Jpeg).to_string(),
            "Image(JPEG)"
        );
    }

    #[test]
    fn export_options_default() {
        let opts = ExportOptions::default();
        assert!(opts.output_path.is_none());
        assert!(opts.include_styles);
        assert!(opts.include_metadata);
        assert!(opts.custom_css.is_none());
        assert_eq!(opts.page_size, PageSize::A4);
        assert!(!opts.embed_images);
        assert!(opts.title_override.is_none());
    }

    #[test]
    fn multiple_formats_can_be_registered() {
        let mut engine = ExportEngine::new();
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Html,
        }));
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Pdf,
        }));
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Image(ImageFormat::Png),
        }));

        assert_eq!(engine.available_formats().len(), 3);
    }

    #[test]
    fn image_format_variants_are_distinct() {
        let png = ExportFormat::Image(ImageFormat::Png);
        let jpeg = ExportFormat::Image(ImageFormat::Jpeg);
        assert_ne!(png, jpeg);
    }

    // -----------------------------------------------------------------------
    // export_with_fallback tests
    // -----------------------------------------------------------------------

    /// An exporter that always fails.
    struct FailingExporter {
        format: ExportFormat,
    }

    impl Exporter for FailingExporter {
        fn export(
            &self,
            _document: &Document,
            _options: &ExportOptions,
        ) -> Result<Vec<u8>, ExportError> {
            Err(ExportError::GenerationError(format!(
                "{} export tool not available",
                self.format
            )))
        }

        fn supported_format(&self) -> ExportFormat {
            self.format.clone()
        }
    }

    #[test]
    fn export_with_fallback_success_path() {
        // When primary format works, no fallback is used
        let mut engine = ExportEngine::new();
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Pdf,
        }));
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Html,
        }));

        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Text("content".into())],
            id: 0,
        }]);
        let options = ExportOptions::default();

        let result = engine
            .export_with_fallback(&doc, &ExportFormat::Pdf, &options)
            .unwrap();

        assert_eq!(result.actual_format, ExportFormat::Pdf);
        assert!(!result.fallback_used);
        assert!(result.original_error.is_none());
        assert_eq!(result.data, b"content");
    }

    #[test]
    fn export_with_fallback_uses_html_on_primary_failure() {
        // When primary format fails but HTML works, fallback is used
        let mut engine = ExportEngine::new();
        engine.register_exporter(Box::new(FailingExporter {
            format: ExportFormat::Pdf,
        }));
        engine.register_exporter(Box::new(MockExporter {
            format: ExportFormat::Html,
        }));

        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Text("fallback content".into())],
            id: 0,
        }]);
        let options = ExportOptions::default();

        let result = engine
            .export_with_fallback(&doc, &ExportFormat::Pdf, &options)
            .unwrap();

        assert_eq!(result.actual_format, ExportFormat::Html);
        assert!(result.fallback_used);
        assert!(result.original_error.is_some());
        let ctx = result.original_error.unwrap();
        assert_eq!(ctx.format, "PDF");
        assert_eq!(ctx.step, ExportStep::OutputGeneration);
        assert!(ctx.suggestion.is_some());
        assert_eq!(result.data, b"fallback content");
    }

    #[test]
    fn export_with_fallback_both_fail() {
        // When both primary and HTML fallback fail, returns error
        let mut engine = ExportEngine::new();
        engine.register_exporter(Box::new(FailingExporter {
            format: ExportFormat::Pdf,
        }));
        engine.register_exporter(Box::new(FailingExporter {
            format: ExportFormat::Html,
        }));

        let doc = Document::new(vec![]);
        let options = ExportOptions::default();

        let result = engine.export_with_fallback(&doc, &ExportFormat::Pdf, &options);
        assert!(result.is_err());
    }

    #[test]
    fn export_with_fallback_html_format_no_double_fallback() {
        // When requesting HTML and it fails, should not attempt fallback to itself
        let mut engine = ExportEngine::new();
        engine.register_exporter(Box::new(FailingExporter {
            format: ExportFormat::Html,
        }));

        let doc = Document::new(vec![]);
        let options = ExportOptions::default();

        let result = engine.export_with_fallback(&doc, &ExportFormat::Html, &options);
        assert!(result.is_err());
    }
}
