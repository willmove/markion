//! DOCX exporter: converts a Document AST to Word (.docx) format using pandoc.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use markdown::{render_to_markdown, Document};

use crate::engine::{ExportFormat, ExportOptions, Exporter, PageSize};
use crate::error::ExportError;

/// Exporter that produces DOCX (Word) files from a Document AST.
///
/// Uses pandoc as a subprocess to convert Markdown to DOCX format.
/// Requires pandoc to be installed and available in PATH (or specified via custom path).
pub struct DocxExporter {
    pandoc_path: PathBuf,
}

impl DocxExporter {
    /// Creates a new DOCX exporter using the default pandoc path ("pandoc").
    pub fn new() -> Self {
        Self {
            pandoc_path: PathBuf::from("pandoc"),
        }
    }

    /// Creates a new DOCX exporter with a custom pandoc binary path.
    pub fn with_pandoc_path(path: impl Into<PathBuf>) -> Self {
        Self {
            pandoc_path: path.into(),
        }
    }

    /// Checks if pandoc is available at the configured path.
    pub fn check_pandoc_available(&self) -> bool {
        Command::new(&self.pandoc_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Build the pandoc command arguments for DOCX conversion.
    fn build_args(&self, options: &ExportOptions) -> Vec<String> {
        let mut args = vec![
            "--from=markdown".to_string(),
            "--to=docx".to_string(),
            "--output=-".to_string(),
        ];

        // Page size via pandoc variable
        match options.page_size {
            PageSize::A4 => {
                args.push("-V".to_string());
                args.push("papersize=a4".to_string());
            }
            PageSize::Letter => {
                args.push("-V".to_string());
                args.push("papersize=letter".to_string());
            }
            PageSize::Legal => {
                args.push("-V".to_string());
                args.push("papersize=legal".to_string());
            }
            PageSize::Custom { .. } => {
                // Custom sizes not directly supported by pandoc; default to A4
                args.push("-V".to_string());
                args.push("papersize=a4".to_string());
            }
        }

        args
    }

    /// Render the document to Markdown, optionally prepending YAML front matter.
    fn render_markdown_input(&self, document: &Document, options: &ExportOptions) -> String {
        let md = render_to_markdown(document);

        // If title_override is set and no metadata exists in the document,
        // prepend YAML front matter
        if let Some(ref title) = options.title_override {
            if document.metadata.is_none() {
                let mut output = String::new();
                output.push_str("---\n");
                output.push_str(&format!("title: {}\n", title));
                output.push_str("---\n\n");
                output.push_str(&md);
                return output;
            }
        }

        md
    }
}

impl Default for DocxExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for DocxExporter {
    fn export(&self, document: &Document, options: &ExportOptions) -> Result<Vec<u8>, ExportError> {
        let markdown_input = self.render_markdown_input(document, options);
        let args = self.build_args(options);

        let mut child = Command::new(&self.pandoc_path)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ExportError::GenerationError(
                        "pandoc not found. Please install pandoc to export DOCX files.".into(),
                    )
                } else {
                    ExportError::Io(e)
                }
            })?;

        // Write markdown to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(markdown_input.as_bytes()).map_err(|e| {
                ExportError::GenerationError(format!("Failed to write to pandoc stdin: {}", e))
            })?;
        }

        let output = child.wait_with_output().map_err(|e| {
            ExportError::GenerationError(format!("Failed to read pandoc output: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExportError::GenerationError(format!(
                "pandoc exited with error: {}",
                stderr
            )));
        }

        Ok(output.stdout)
    }

    fn supported_format(&self) -> ExportFormat {
        ExportFormat::Docx
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::{Block, Document, Inline};

    fn default_options() -> ExportOptions {
        ExportOptions::default()
    }

    #[test]
    fn test_supported_format() {
        let exporter = DocxExporter::new();
        assert_eq!(exporter.supported_format(), ExportFormat::Docx);
    }

    #[test]
    fn test_build_args_a4() {
        let exporter = DocxExporter::new();
        let options = default_options();
        let args = exporter.build_args(&options);

        assert!(args.contains(&"--from=markdown".to_string()));
        assert!(args.contains(&"--to=docx".to_string()));
        assert!(args.contains(&"--output=-".to_string()));
        assert!(args.contains(&"papersize=a4".to_string()));
    }

    #[test]
    fn test_build_args_letter() {
        let exporter = DocxExporter::new();
        let mut options = default_options();
        options.page_size = PageSize::Letter;
        let args = exporter.build_args(&options);

        assert!(args.contains(&"papersize=letter".to_string()));
    }

    #[test]
    fn test_build_args_legal() {
        let exporter = DocxExporter::new();
        let mut options = default_options();
        options.page_size = PageSize::Legal;
        let args = exporter.build_args(&options);

        assert!(args.contains(&"papersize=legal".to_string()));
    }

    #[test]
    fn test_render_markdown_input_basic() {
        let exporter = DocxExporter::new();
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Text("Hello world".into())],
            id: 0,
        }]);
        let options = default_options();
        let md = exporter.render_markdown_input(&doc, &options);

        assert!(md.contains("Hello world"));
    }

    #[test]
    fn test_render_markdown_with_title_override() {
        let exporter = DocxExporter::new();
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Text("Content".into())],
            id: 0,
        }]);
        let mut options = default_options();
        options.title_override = Some("Custom Title".into());
        let md = exporter.render_markdown_input(&doc, &options);

        assert!(md.starts_with("---\n"));
        assert!(md.contains("title: Custom Title"));
        assert!(md.contains("Content"));
    }

    #[test]
    fn test_error_when_pandoc_not_found() {
        let exporter = DocxExporter::with_pandoc_path("/nonexistent/path/to/pandoc");
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Text("Test".into())],
            id: 0,
        }]);
        let options = default_options();

        let result = exporter.export(&doc, &options);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("pandoc")
                || err_msg.contains("not found")
                || err_msg.contains("No such file"),
            "Expected error about pandoc not found, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_with_pandoc_path() {
        let exporter = DocxExporter::with_pandoc_path("/custom/pandoc");
        assert_eq!(exporter.pandoc_path, PathBuf::from("/custom/pandoc"));
    }

    #[test]
    fn test_check_pandoc_available_nonexistent() {
        let exporter = DocxExporter::with_pandoc_path("/nonexistent/pandoc");
        assert!(!exporter.check_pandoc_available());
    }

    #[test]
    #[ignore] // Integration test: requires pandoc installed
    fn test_docx_export_integration() {
        let exporter = DocxExporter::new();
        if !exporter.check_pandoc_available() {
            return;
        }

        let doc = Document::new(vec![
            Block::Heading {
                level: 1,
                content: vec![Inline::Text("Test Document".into())],
                id: 0,
            },
            Block::Paragraph {
                content: vec![Inline::Text("Hello, World!".into())],
                id: 1,
            },
        ]);
        let options = default_options();

        let result = exporter.export(&doc, &options);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // DOCX files start with the PK zip magic bytes
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..2], b"PK");
    }
}
