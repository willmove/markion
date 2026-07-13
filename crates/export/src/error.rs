use std::fmt;

use thiserror::Error;

/// Unified error type for the export module
#[derive(Error, Debug)]
pub enum ExportError {
    #[error("Export format not supported: {0}")]
    UnsupportedFormat(String),

    #[error("Export generation failed: {0}")]
    GenerationError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Markdown error: {0}")]
    Markdown(#[from] markdown::MarkdownError),

    #[error("Unknown export error: {0}")]
    Other(String),
}

impl ExportError {
    /// Returns a user-friendly message suitable for display in a notification.
    pub fn user_friendly_message(&self) -> &str {
        match self {
            ExportError::UnsupportedFormat(_) => {
                "Export format not available. Try exporting as HTML."
            }
            ExportError::GenerationError(msg) => {
                if msg.to_lowercase().contains("pandoc") {
                    "PDF export failed. Please ensure pandoc is installed."
                } else {
                    "Export generation failed. Try exporting as HTML instead."
                }
            }
            ExportError::TemplateError(_) => {
                "Export template error. Check your template configuration."
            }
            ExportError::Io(_) => {
                "Could not write the exported file. Check disk space and permissions."
            }
            ExportError::Markdown(_) => {
                "Failed to process the Markdown document. Check your document syntax."
            }
            ExportError::Other(_) => "An unexpected error occurred during export.",
        }
    }
}

/// Result type alias for export operations
pub type ExportResult<T> = Result<T, ExportError>;

// ---------------------------------------------------------------------------
// ExportStep
// ---------------------------------------------------------------------------

/// Represents which step of the export pipeline failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStep {
    /// Preparing the document/input for export
    Preparation,
    /// Running an external tool (e.g., pandoc, wkhtmltoimage)
    ToolExecution,
    /// Generating the output bytes
    OutputGeneration,
    /// Any post-processing step
    PostProcessing,
}

impl fmt::Display for ExportStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportStep::Preparation => write!(f, "Preparing input"),
            ExportStep::ToolExecution => write!(f, "Running external tool"),
            ExportStep::OutputGeneration => write!(f, "Generating output"),
            ExportStep::PostProcessing => write!(f, "Post-processing"),
        }
    }
}

// ---------------------------------------------------------------------------
// ExportErrorContext
// ---------------------------------------------------------------------------

/// Wraps an `ExportError` with additional context about the failure.
#[derive(Debug)]
pub struct ExportErrorContext {
    /// The underlying export error.
    pub error: ExportError,
    /// The format that was being exported when the error occurred.
    pub format: String,
    /// Which step of the export pipeline failed.
    pub step: ExportStep,
    /// An optional user-friendly suggestion for recovery.
    pub suggestion: Option<String>,
}

impl ExportErrorContext {
    /// Creates a new `ExportErrorContext`.
    pub fn new(error: ExportError, format: impl Into<String>, step: ExportStep) -> Self {
        Self {
            error,
            format: format.into(),
            step,
            suggestion: None,
        }
    }

    /// Sets the suggestion field and returns self for builder-style usage.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl fmt::Display for ExportErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Export to {} failed at step '{}': {}",
            self.format, self.step, self.error
        )?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " (suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_step_display() {
        assert_eq!(ExportStep::Preparation.to_string(), "Preparing input");
        assert_eq!(
            ExportStep::ToolExecution.to_string(),
            "Running external tool"
        );
        assert_eq!(
            ExportStep::OutputGeneration.to_string(),
            "Generating output"
        );
        assert_eq!(ExportStep::PostProcessing.to_string(), "Post-processing");
    }

    #[test]
    fn export_error_context_display_without_suggestion() {
        let ctx = ExportErrorContext::new(
            ExportError::GenerationError("pandoc not found".into()),
            "PDF",
            ExportStep::ToolExecution,
        );
        let display = ctx.to_string();
        assert!(display.contains("Export to PDF failed"));
        assert!(display.contains("Running external tool"));
        assert!(display.contains("pandoc not found"));
        assert!(!display.contains("suggestion"));
    }

    #[test]
    fn export_error_context_display_with_suggestion() {
        let ctx = ExportErrorContext::new(
            ExportError::GenerationError("pandoc not found".into()),
            "PDF",
            ExportStep::ToolExecution,
        )
        .with_suggestion("Try exporting as HTML instead");

        let display = ctx.to_string();
        assert!(display.contains("Export to PDF failed"));
        assert!(display.contains("Running external tool"));
        assert!(display.contains("pandoc not found"));
        assert!(display.contains("(suggestion: Try exporting as HTML instead)"));
    }

    #[test]
    fn user_friendly_message_unsupported_format() {
        let err = ExportError::UnsupportedFormat("TIFF".into());
        assert_eq!(
            err.user_friendly_message(),
            "Export format not available. Try exporting as HTML."
        );
    }

    #[test]
    fn user_friendly_message_generation_error_pandoc() {
        let err = ExportError::GenerationError("pandoc process failed".into());
        assert_eq!(
            err.user_friendly_message(),
            "PDF export failed. Please ensure pandoc is installed."
        );
    }

    #[test]
    fn user_friendly_message_generation_error_generic() {
        let err = ExportError::GenerationError("unknown failure".into());
        assert_eq!(
            err.user_friendly_message(),
            "Export generation failed. Try exporting as HTML instead."
        );
    }

    #[test]
    fn user_friendly_message_template_error() {
        let err = ExportError::TemplateError("missing variable".into());
        assert_eq!(
            err.user_friendly_message(),
            "Export template error. Check your template configuration."
        );
    }

    #[test]
    fn user_friendly_message_io_error() {
        let err = ExportError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "access denied",
        ));
        assert_eq!(
            err.user_friendly_message(),
            "Could not write the exported file. Check disk space and permissions."
        );
    }

    #[test]
    fn user_friendly_message_other() {
        let err = ExportError::Other("something went wrong".into());
        assert_eq!(
            err.user_friendly_message(),
            "An unexpected error occurred during export."
        );
    }
}
