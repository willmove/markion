//! PDF export via pandoc subprocess.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use markdown::{render_to_markdown, Document};

use crate::engine::{ExportFormat, ExportOptions, Exporter, PageSize};
use crate::error::ExportError;

// ---------------------------------------------------------------------------
// PdfExporter
// ---------------------------------------------------------------------------

/// Exports a Markdown document to PDF by invoking `pandoc` as a subprocess.
///
/// Pandoc converts the Markdown input (with optional YAML front matter) into
/// a PDF using xelatex as the PDF engine. Syntax highlighting, math rendering,
/// and page-size options are passed through pandoc CLI flags.
pub struct PdfExporter {
    /// Path to the pandoc binary. Defaults to "pandoc" (resolved via PATH).
    pandoc_path: PathBuf,
    /// Pandoc PDF engine passed as `--pdf-engine=`. Defaults to "xelatex".
    pdf_engine: String,
}

impl PdfExporter {
    /// Creates a new `PdfExporter` using the default `pandoc` binary from PATH.
    pub fn new() -> Self {
        Self {
            pandoc_path: PathBuf::from("pandoc"),
            pdf_engine: "xelatex".to_string(),
        }
    }

    /// Creates a new `PdfExporter` with an explicit path to the pandoc binary.
    pub fn with_pandoc_path(path: PathBuf) -> Self {
        Self {
            pandoc_path: path,
            ..Self::new()
        }
    }

    /// Sets the pandoc PDF engine (e.g. "xelatex", "pdfroff", "tectonic",
    /// "wkhtmltopdf"). Blank values keep the default.
    pub fn with_pdf_engine(mut self, engine: impl Into<String>) -> Self {
        let engine = engine.into();
        let engine = engine.trim();
        if !engine.is_empty() {
            self.pdf_engine = engine.to_string();
        }
        self
    }

    /// Builds the pandoc command-line arguments from the given export options.
    fn build_pandoc_args(&self, options: &ExportOptions) -> Vec<String> {
        let mut args = Vec::new();

        // Input/output format
        args.push("--from=markdown".to_string());
        args.push("--to=pdf".to_string());
        args.push(format!("--pdf-engine={}", self.pdf_engine));

        // Page size via geometry variable
        let geometry = match options.page_size {
            PageSize::A4 => "a4paper".to_string(),
            PageSize::Letter => "letterpaper".to_string(),
            PageSize::Legal => "legalpaper".to_string(),
            PageSize::Custom {
                width_mm,
                height_mm,
            } => {
                format!("paperwidth={}mm,paperheight={}mm", width_mm, height_mm)
            }
        };
        args.push(format!("--variable=geometry:{}", geometry));

        // Syntax highlighting
        if options.include_styles {
            args.push("--highlight-style=tango".to_string());
        }

        // Math rendering (katex for pandoc PDF)
        args.push("--katex".to_string());

        // Standalone document (includes preamble for proper PDF)
        args.push("--standalone".to_string());

        // Output to stdout
        args.push("--output=-".to_string());

        args
    }

    /// Prepares the markdown input text for pandoc.
    ///
    /// If `include_metadata` is true and the document has metadata, the full
    /// markdown (including YAML front matter) is returned. Otherwise only the
    /// body content is rendered.
    fn prepare_input(&self, document: &Document, options: &ExportOptions) -> String {
        if options.include_metadata && document.metadata.is_some() {
            // render_to_markdown already includes YAML front matter
            let mut md = render_to_markdown(document);

            // Apply title override if present
            if let Some(ref title) = options.title_override {
                md = replace_or_prepend_title(&md, title);
            }

            md
        } else {
            // Render without metadata: create a temporary doc without metadata
            let doc_no_meta = Document {
                blocks: document.blocks.clone(),
                metadata: None,
                version: document.version,
                footnote_map: document.footnote_map.clone(),
                shared_regions: Vec::new(),
            };

            let mut md = render_to_markdown(&doc_no_meta);

            // If title_override is set, prepend a YAML block with just the title
            if let Some(ref title) = options.title_override {
                let header = format!("---\ntitle: \"{}\"\n---\n\n", escape_yaml_string(title));
                md = format!("{}{}", header, md);
            }

            md
        }
    }
}

impl Default for PdfExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for PdfExporter {
    fn export(&self, document: &Document, options: &ExportOptions) -> Result<Vec<u8>, ExportError> {
        let input = self.prepare_input(document, options);
        let args = self.build_pandoc_args(options);

        // Spawn pandoc process
        let mut child = Command::new(&self.pandoc_path)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ExportError::GenerationError(format!(
                        "pandoc not found at '{}'. Please install pandoc: https://pandoc.org/installing.html",
                        self.pandoc_path.display()
                    ))
                } else {
                    ExportError::Io(e)
                }
            })?;

        // Write markdown to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes()).map_err(ExportError::Io)?;
            // stdin is dropped here, closing the pipe
        }

        // Wait for process and collect output
        let output = child.wait_with_output().map_err(ExportError::Io)?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ExportError::GenerationError(format!(
                "pandoc exited with status {}: {}",
                output.status,
                stderr.trim()
            )))
        }
    }

    fn supported_format(&self) -> ExportFormat {
        ExportFormat::Pdf
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Escapes a string for safe inclusion in YAML values.
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Replaces the title in existing YAML front matter, or prepends a YAML block
/// with the given title if no front matter exists.
fn replace_or_prepend_title(md: &str, title: &str) -> String {
    if md.starts_with("---\n") {
        // Find the closing ---
        if let Some(end_idx) = md[4..].find("\n---\n") {
            let front_matter = &md[4..4 + end_idx];
            let rest = &md[4 + end_idx + 5..]; // skip past "\n---\n"

            // Replace or add title line
            let mut new_fm = String::new();
            let mut title_found = false;
            for line in front_matter.lines() {
                if line.starts_with("title:") {
                    new_fm.push_str(&format!("title: \"{}\"", escape_yaml_string(title)));
                    title_found = true;
                } else {
                    new_fm.push_str(line);
                }
                new_fm.push('\n');
            }
            if !title_found {
                new_fm.push_str(&format!("title: \"{}\"\n", escape_yaml_string(title)));
            }

            format!("---\n{}---\n{}", new_fm, rest)
        } else {
            md.to_string()
        }
    } else {
        format!(
            "---\ntitle: \"{}\"\n---\n\n{}",
            escape_yaml_string(title),
            md
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::{Block, Document, Inline, YamlFrontMatter};
    use std::collections::HashMap;

    #[test]
    fn pdf_engine_is_configurable() {
        let default_args = PdfExporter::new().build_pandoc_args(&ExportOptions::default());
        assert!(default_args.contains(&"--pdf-engine=xelatex".to_string()));

        let custom = PdfExporter::new().with_pdf_engine("pdfroff");
        let custom_args = custom.build_pandoc_args(&ExportOptions::default());
        assert!(custom_args.contains(&"--pdf-engine=pdfroff".to_string()));

        // Blank values keep the default.
        let blank = PdfExporter::new().with_pdf_engine("  ");
        let blank_args = blank.build_pandoc_args(&ExportOptions::default());
        assert!(blank_args.contains(&"--pdf-engine=xelatex".to_string()));
    }

    fn sample_document() -> Document {
        Document::new(vec![
            Block::Heading {
                level: 1,
                content: vec![Inline::Text("Hello PDF".into())],
                id: 1,
            },
            Block::Paragraph {
                content: vec![Inline::Text("This is a test document.".into())],
                id: 2,
            },
        ])
    }

    fn sample_document_with_metadata() -> Document {
        Document::with_metadata(
            vec![Block::Paragraph {
                content: vec![Inline::Text("Content here.".into())],
                id: 1,
            }],
            YamlFrontMatter {
                title: Some("My Document".into()),
                author: Some("Test Author".into()),
                date: Some("2024-06-01".into()),
                tags: vec!["rust".into(), "pdf".into()],
                custom: HashMap::new(),
            },
        )
    }

    #[test]
    fn supported_format_returns_pdf() {
        let exporter = PdfExporter::new();
        assert_eq!(exporter.supported_format(), ExportFormat::Pdf);
    }

    #[test]
    fn implements_exporter_trait() {
        // Verify PdfExporter can be used as a trait object
        let exporter: Box<dyn Exporter> = Box::new(PdfExporter::new());
        assert_eq!(exporter.supported_format(), ExportFormat::Pdf);
    }

    #[test]
    fn build_args_default_options() {
        let exporter = PdfExporter::new();
        let options = ExportOptions::default();
        let args = exporter.build_pandoc_args(&options);

        assert!(args.contains(&"--from=markdown".to_string()));
        assert!(args.contains(&"--to=pdf".to_string()));
        assert!(args.contains(&"--pdf-engine=xelatex".to_string()));
        assert!(args.contains(&"--variable=geometry:a4paper".to_string()));
        assert!(args.contains(&"--highlight-style=tango".to_string()));
        assert!(args.contains(&"--katex".to_string()));
        assert!(args.contains(&"--standalone".to_string()));
        assert!(args.contains(&"--output=-".to_string()));
    }

    #[test]
    fn build_args_letter_page_size() {
        let exporter = PdfExporter::new();
        let options = ExportOptions {
            page_size: PageSize::Letter,
            ..Default::default()
        };
        let args = exporter.build_pandoc_args(&options);

        assert!(args.contains(&"--variable=geometry:letterpaper".to_string()));
    }

    #[test]
    fn build_args_legal_page_size() {
        let exporter = PdfExporter::new();
        let options = ExportOptions {
            page_size: PageSize::Legal,
            ..Default::default()
        };
        let args = exporter.build_pandoc_args(&options);

        assert!(args.contains(&"--variable=geometry:legalpaper".to_string()));
    }

    #[test]
    fn build_args_custom_page_size() {
        let exporter = PdfExporter::new();
        let options = ExportOptions {
            page_size: PageSize::Custom {
                width_mm: 200,
                height_mm: 300,
            },
            ..Default::default()
        };
        let args = exporter.build_pandoc_args(&options);

        assert!(
            args.contains(&"--variable=geometry:paperwidth=200mm,paperheight=300mm".to_string())
        );
    }

    #[test]
    fn build_args_no_highlight_when_styles_disabled() {
        let exporter = PdfExporter::new();
        let options = ExportOptions {
            include_styles: false,
            ..Default::default()
        };
        let args = exporter.build_pandoc_args(&options);

        assert!(!args.iter().any(|a| a.starts_with("--highlight-style")));
    }

    #[test]
    fn prepare_input_includes_metadata_when_enabled() {
        let exporter = PdfExporter::new();
        let doc = sample_document_with_metadata();
        let options = ExportOptions {
            include_metadata: true,
            ..Default::default()
        };

        let input = exporter.prepare_input(&doc, &options);

        assert!(input.starts_with("---\n"));
        assert!(input.contains("title: My Document"));
        assert!(input.contains("author: Test Author"));
        assert!(input.contains("date: 2024-06-01"));
        assert!(input.contains("Content here."));
    }

    #[test]
    fn prepare_input_excludes_metadata_when_disabled() {
        let exporter = PdfExporter::new();
        let doc = sample_document_with_metadata();
        let options = ExportOptions {
            include_metadata: false,
            ..Default::default()
        };

        let input = exporter.prepare_input(&doc, &options);

        assert!(!input.contains("title: My Document"));
        assert!(!input.contains("author: Test Author"));
        assert!(input.contains("Content here."));
    }

    #[test]
    fn prepare_input_title_override_with_metadata() {
        let exporter = PdfExporter::new();
        let doc = sample_document_with_metadata();
        let options = ExportOptions {
            include_metadata: true,
            title_override: Some("Overridden Title".into()),
            ..Default::default()
        };

        let input = exporter.prepare_input(&doc, &options);

        assert!(input.contains("title: \"Overridden Title\""));
        assert!(!input.contains("title: My Document"));
    }

    #[test]
    fn prepare_input_title_override_without_metadata() {
        let exporter = PdfExporter::new();
        let doc = sample_document();
        let options = ExportOptions {
            include_metadata: true,
            title_override: Some("New Title".into()),
            ..Default::default()
        };

        let input = exporter.prepare_input(&doc, &options);

        // Should prepend YAML with title since doc has no metadata
        assert!(input.contains("title: \"New Title\""));
        assert!(input.contains("# Hello PDF"));
    }

    #[test]
    fn export_returns_error_when_pandoc_not_found() {
        let exporter =
            PdfExporter::with_pandoc_path(PathBuf::from("/nonexistent/path/to/pandoc_binary_xyz"));
        let doc = sample_document();
        let options = ExportOptions::default();

        let result = exporter.export(&doc, &options);

        assert!(result.is_err());
        match result.unwrap_err() {
            ExportError::GenerationError(msg) => {
                assert!(msg.contains("pandoc not found"));
            }
            other => panic!("Expected GenerationError, got: {:?}", other),
        }
    }

    #[test]
    fn with_pandoc_path_sets_custom_path() {
        let path = PathBuf::from("/usr/local/bin/pandoc");
        let exporter = PdfExporter::with_pandoc_path(path.clone());
        assert_eq!(exporter.pandoc_path, path);
    }

    #[test]
    fn default_pandoc_path_is_pandoc() {
        let exporter = PdfExporter::new();
        assert_eq!(exporter.pandoc_path, PathBuf::from("pandoc"));
    }

    #[test]
    fn escape_yaml_handles_special_chars() {
        assert_eq!(escape_yaml_string(r#"hello"world"#), r#"hello\"world"#);
        assert_eq!(escape_yaml_string(r"back\slash"), r"back\\slash");
    }

    // Integration test that requires pandoc to be installed
    #[test]
    #[ignore]
    fn export_produces_pdf_bytes_with_pandoc() {
        let exporter = PdfExporter::new();
        let doc = sample_document_with_metadata();
        let options = ExportOptions::default();

        let result = exporter.export(&doc, &options);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // PDF files start with %PDF
        assert!(bytes.starts_with(b"%PDF"));
    }
}
