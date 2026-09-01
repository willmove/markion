//! Document exporters: PDF, DOCX (with a hand-written deflate-compressed ZIP
//! container), and PNG/JPEG snapshots that re-render the layout IR through
//! `markion-pdf` (real fonts, CJK-aware shaping, headings/code/tables).
//!
//! PDF and DOCX follow the persisted backend preference: `pandoc` runs the
//! export engine absorbed from Typune (a pandoc subprocess wrapper,
//! `crates/export`) with the hand-written implementations below as the
//! fallback when pandoc is unavailable or fails; `builtin` (the default)
//! writes through the hand-written implementations directly, so export never
//! requires external tools.

use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use markion_pdf::{
    Block as PdfBlock, Cell as PdfCell, ImageData as PdfImageData, InlineImage as PdfInlineImage,
    ListMarker, PdfDocument, PdfMetadata, PdfOptions, Rgb, Run as PdfRun, Style as PdfStyle,
};
use percent_encoding::percent_decode_str;
use typune_export::{DocxExporter, ExportError, ExportOptions, Exporter, PdfExporter};
use typune_markdown::{MathRenderer, Parser};

use crate::MarkdownDocument;
use crate::escape::escape_xml_text;
use crate::highlight::highlight_code;
use crate::i18n::Msg;
use crate::math::tex_to_omml;
use crate::model::{
    AlertKind, DocxExportOptions, DocxImagePolicy, DocxPageSize, EngineFailureCategory,
    ExportBackend, ExportPreferences, HighlightKind, InlineSpan, InlineStyle, PdfExportOptions,
    PdfPageSize, PreviewBlock, RichText, TableAlignment,
};
use crate::parse::{HtmlPreviewPart, HtmlTableGrid, html_preview_parts};

/// Maps an engine error to the failure category disclosed on the status bar
/// when the built-in writer takes over.
fn engine_failure_category(err: &ExportError) -> EngineFailureCategory {
    match err {
        ExportError::ToolNotFound(_) => EngineFailureCategory::BinaryMissing,
        _ => EngineFailureCategory::ConversionError,
    }
}

/// Runs a Typune exporter over the raw Markdown source. On any failure
/// (pandoc missing, conversion error) the failure category is returned so
/// callers can fall back to the built-in writers and disclose the category.
fn engine_export(
    source: &str,
    exporter: &dyn Exporter,
    options: &ExportOptions,
) -> Result<Vec<u8>, EngineFailureCategory> {
    let document = Parser::default().parse(source).map_err(|err| {
        tracing::warn!(error = %err, "engine parser failed; using built-in exporter");
        EngineFailureCategory::ConversionError
    })?;
    exporter.export(&document, options).map_err(|err| {
        let category = engine_failure_category(&err);
        tracing::info!(
            error = %err,
            format = %exporter.supported_format(),
            "export engine unavailable; using built-in exporter"
        );
        category
    })
}

pub(crate) fn engine_pdf(
    source: &str,
    settings: &ExportPreferences,
    document_dir: Option<&Path>,
) -> Result<Vec<u8>, EngineFailureCategory> {
    let exporter = match settings
        .pandoc_path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        Some(path) => PdfExporter::with_pandoc_path(PathBuf::from(path)),
        None => PdfExporter::new(),
    }
    .with_pdf_engine(&settings.pdf_engine)
    .with_mainfont(settings.pdf_mainfont.as_deref())
    .with_cjk_font(settings.pdf_cjk_font.as_deref());
    let options = ExportOptions {
        page_size: match settings.pdf.page_size {
            PdfPageSize::A4 => typune_export::PageSize::A4,
            PdfPageSize::Letter => typune_export::PageSize::Letter,
            PdfPageSize::Legal => typune_export::PageSize::Legal,
        },
        toc: settings.pdf.toc,
        resource_path: document_dir.map(Path::to_path_buf),
        ..ExportOptions::default()
    };
    engine_export(source, &exporter, &options)
}

// --- Built-in PDF IR builder -------------------------------------------------

/// Builds the `markion_pdf` layout IR from the cached preview blocks.
///
/// Mirrors `render_docx_document_xml`: walks `preview_blocks_shared()` once,
/// resolves images against `base_dir` and the prefetched `remote_images`
/// map, converts every `PreviewBlock` variant into the corresponding
/// `PdfBlock`, and collects footnote bodies into `PdfDocument::footnotes`
/// so inline references can resolve to 1-based ids.
pub fn build_pdf_ir(
    document: &MarkdownDocument,
    options: &PdfExportOptions,
    base_dir: Option<&Path>,
    remote_images: &HashMap<String, Vec<u8>>,
) -> PdfDocument {
    let metadata = document.front_matter().ok().flatten();
    let title = metadata
        .as_ref()
        .and_then(|metadata| metadata.title.as_deref())
        .or_else(|| {
            document
                .path()
                .and_then(Path::file_stem)
                .and_then(|stem| stem.to_str())
        })
        .map(str::to_string);
    let author = metadata
        .as_ref()
        .and_then(|metadata| metadata.author.as_deref())
        .map(str::to_string);
    let date = metadata
        .as_ref()
        .and_then(|metadata| metadata.date.as_deref())
        .map(str::to_string);

    let (page_width_mm, page_height_mm) = options.page_size.dimensions_mm();
    let pdf_options = PdfOptions {
        page_width_mm,
        page_height_mm,
        margin_mm: options.margin_mm as f32,
        toc: options.toc,
        page_numbers: options.page_numbers,
    };

    let blocks = document.preview_blocks_shared();
    let base_dir = base_dir.map(Path::to_path_buf);

    // Collect footnote definition labels in document order so in-text
    // superscript references can map to 1-based ids.
    let mut footnote_labels: Vec<String> = Vec::new();
    for block in blocks.iter() {
        if let PreviewBlock::FootnoteDefinition { label, .. } = block {
            footnote_labels.push(label.clone());
        }
    }

    let mut pdf_blocks: Vec<PdfBlock> = Vec::new();
    let mut footnotes: Vec<Vec<PdfRun>> = Vec::new();
    for block in blocks.iter() {
        match block {
            PreviewBlock::FootnoteDefinition { text, .. } => {
                footnotes.push(pdf_runs(text, &footnote_labels));
            }
            other => {
                pdf_blocks.extend(pdf_blocks_from_preview_block(
                    other,
                    &base_dir,
                    &footnote_labels,
                    remote_images,
                ));
            }
        }
    }

    PdfDocument {
        metadata: PdfMetadata {
            title,
            author,
            date,
        },
        options: pdf_options,
        blocks: pdf_blocks,
        footnotes,
    }
}

fn pdf_blocks_from_preview_block(
    block: &PreviewBlock,
    base_dir: &Option<PathBuf>,
    footnotes: &[String],
    remote_images: &HashMap<String, Vec<u8>>,
) -> Vec<PdfBlock> {
    match block {
        PreviewBlock::Html { html, .. } => {
            pdf_html_blocks(html, base_dir, footnotes, remote_images)
        }
        _ => pdf_single_block(block, base_dir, footnotes, remote_images)
            .into_iter()
            .collect(),
    }
}

fn pdf_single_block(
    block: &PreviewBlock,
    base_dir: &Option<PathBuf>,
    footnotes: &[String],
    remote_images: &HashMap<String, Vec<u8>>,
) -> Option<PdfBlock> {
    match block {
        PreviewBlock::Heading { level, text, .. } => Some(PdfBlock::Heading {
            level: *level,
            content: pdf_runs(text, footnotes),
        }),
        PreviewBlock::Paragraph { text, .. } => Some(PdfBlock::Paragraph {
            content: pdf_runs(text, footnotes),
        }),
        PreviewBlock::ListItem {
            level,
            ordered,
            index,
            checked,
            text,
            ..
        } => {
            let indent_level = (*level).saturating_sub(1).min(u8::MAX as usize) as u8;
            let marker = match checked {
                Some(done) => ListMarker::Task { checked: *done },
                None if *ordered => ListMarker::Number(index.unwrap_or(1)),
                None => ListMarker::Bullet,
            };
            Some(PdfBlock::ListItem {
                indent_level,
                marker,
                content: pdf_runs(text, footnotes),
            })
        }
        PreviewBlock::BlockQuote {
            children, alert, ..
        } => {
            let children: Vec<PdfBlock> = children
                .iter()
                .flat_map(|child| {
                    pdf_blocks_from_preview_block(child, base_dir, footnotes, remote_images)
                })
                .collect();
            if let Some(kind) = alert {
                Some(PdfBlock::Alert {
                    kind: pdf_alert_kind(*kind),
                    children,
                })
            } else {
                Some(PdfBlock::Quote { children })
            }
        }
        PreviewBlock::CodeBlock { language, code, .. } => Some(PdfBlock::CodeBlock {
            language: language.clone(),
            lines: pdf_code_lines(code, language.as_deref()),
        }),
        PreviewBlock::MathBlock {
            latex, authored, ..
        } => Some(pdf_math_block(latex, authored)),
        PreviewBlock::Image { alt, url, .. } => pdf_image_block(alt, url, base_dir, remote_images)
            .or_else(|| Some(pdf_image_fallback_block(alt, url))),
        PreviewBlock::Rule { .. } => Some(PdfBlock::Rule),
        PreviewBlock::Table {
            rows, alignments, ..
        } => pdf_table_block(rows, alignments, footnotes),
        PreviewBlock::FootnoteDefinition { .. } => None,
        PreviewBlock::Html { .. } => {
            unreachable!("Html blocks are handled by pdf_blocks_from_preview_block")
        }
    }
}

fn pdf_html_blocks(
    html: &str,
    base_dir: &Option<PathBuf>,
    footnotes: &[String],
    remote_images: &HashMap<String, Vec<u8>>,
) -> Vec<PdfBlock> {
    let mut blocks = Vec::new();
    for part in html_preview_parts(html) {
        match part {
            HtmlPreviewPart::Text { text, .. } => {
                if !text.is_empty() {
                    blocks.push(PdfBlock::Paragraph {
                        content: pdf_runs(&text, footnotes),
                    });
                }
            }
            HtmlPreviewPart::Image { alt, url, .. } => {
                if let Some(block) = pdf_image_block(&alt, &url, base_dir, remote_images) {
                    blocks.push(block);
                } else {
                    blocks.push(pdf_image_fallback_block(&alt, &url));
                }
            }
            HtmlPreviewPart::Table { grid } => {
                if let Some(block) = pdf_html_table_block(&grid, footnotes) {
                    blocks.push(block);
                }
            }
        }
    }
    blocks
}

fn pdf_image_fallback_block(alt: &str, url: &str) -> PdfBlock {
    let label = if alt.is_empty() { "Image" } else { alt };
    PdfBlock::Paragraph {
        content: vec![PdfRun {
            text: format!("{label}: {url}"),
            ..PdfRun::default()
        }],
    }
}

fn pdf_table_block(
    rows: &[Vec<RichText>],
    alignments: &[TableAlignment],
    footnotes: &[String],
) -> Option<PdfBlock> {
    if rows.is_empty() {
        return None;
    }
    let mut row_iter = rows.iter();
    let header = row_iter
        .next()?
        .iter()
        .map(|cell| PdfCell {
            content: pdf_runs(cell, footnotes),
        })
        .collect();
    let body_rows = row_iter
        .map(|row| {
            row.iter()
                .map(|cell| PdfCell {
                    content: pdf_runs(cell, footnotes),
                })
                .collect()
        })
        .collect();
    Some(PdfBlock::Table {
        header,
        rows: body_rows,
        alignments: alignments.iter().map(pdf_alignment).collect(),
    })
}

fn pdf_html_table_block(grid: &HtmlTableGrid, footnotes: &[String]) -> Option<PdfBlock> {
    if grid.rows.is_empty() {
        return None;
    }
    let columns = grid.columns.max(1);
    let mut row_iter = grid.rows.iter();
    let first_row = row_iter.next().unwrap();
    let first_is_header = first_row.iter().any(|cell| cell.is_header);
    let (header, rows) = if first_is_header {
        let header = first_row
            .iter()
            .map(|cell| PdfCell {
                content: pdf_runs(&cell.content, footnotes),
            })
            .collect();
        let rows = row_iter
            .map(|row| {
                row.iter()
                    .map(|cell| PdfCell {
                        content: pdf_runs(&cell.content, footnotes),
                    })
                    .collect()
            })
            .collect();
        (header, rows)
    } else {
        let header = vec![PdfCell::default(); columns];
        let mut rows: Vec<Vec<PdfCell>> = vec![
            first_row
                .iter()
                .map(|cell| PdfCell {
                    content: pdf_runs(&cell.content, footnotes),
                })
                .collect(),
        ];
        rows.extend(row_iter.map(|row| {
            row.iter()
                .map(|cell| PdfCell {
                    content: pdf_runs(&cell.content, footnotes),
                })
                .collect()
        }));
        (header, rows)
    };
    Some(PdfBlock::Table {
        header,
        rows,
        alignments: vec![markion_pdf::Alignment::Left; columns],
    })
}

fn pdf_alignment(alignment: &TableAlignment) -> markion_pdf::Alignment {
    match alignment {
        TableAlignment::Left => markion_pdf::Alignment::Left,
        TableAlignment::Center => markion_pdf::Alignment::Center,
        TableAlignment::Right => markion_pdf::Alignment::Right,
        TableAlignment::Default => markion_pdf::Alignment::Left,
    }
}

fn pdf_alert_kind(kind: AlertKind) -> markion_pdf::AlertKind {
    match kind {
        AlertKind::Note => markion_pdf::AlertKind::Note,
        AlertKind::Tip => markion_pdf::AlertKind::Tip,
        AlertKind::Important => markion_pdf::AlertKind::Important,
        AlertKind::Warning => markion_pdf::AlertKind::Warning,
        AlertKind::Caution => markion_pdf::AlertKind::Caution,
    }
}

fn pdf_runs(rich: &RichText, footnotes: &[String]) -> Vec<PdfRun> {
    if rich.spans.is_empty() {
        if rich.text.is_empty() {
            return Vec::new();
        }
        return vec![PdfRun {
            text: rich.text.clone(),
            ..PdfRun::default()
        }];
    }
    rich.spans
        .iter()
        .filter_map(|span| pdf_run(span, footnotes))
        .collect()
}

fn pdf_run(span: &InlineSpan, footnotes: &[String]) -> Option<PdfRun> {
    if let Some(math) = &span.math {
        return Some(pdf_inline_math(math, span.link.as_deref()));
    }
    if let Some(image) = &span.image {
        let label = if image.alt.is_empty() {
            image.url.clone()
        } else {
            image.alt.clone()
        };
        return Some(PdfRun {
            text: label,
            ..PdfRun::default()
        });
    }
    if span.text.is_empty() {
        return None;
    }
    if span.style.superscript {
        if let Some(id) = footnote_id(&span.text, footnotes) {
            return Some(PdfRun {
                text: span.text.clone(),
                style: PdfStyle {
                    superscript: true,
                    ..PdfStyle::default()
                },
                footnote: Some(id),
                ..PdfRun::default()
            });
        }
    }
    Some(PdfRun {
        text: span.text.clone(),
        style: pdf_style(&span.style),
        link: span.link.clone(),
        ..PdfRun::default()
    })
}

fn footnote_id(label: &str, footnotes: &[String]) -> Option<u32> {
    footnotes
        .iter()
        .position(|known| known == label)
        .map(|index| index as u32 + 1)
}

fn pdf_style(style: &InlineStyle) -> PdfStyle {
    PdfStyle {
        bold: style.bold,
        italic: style.italic,
        strikethrough: style.strikethrough,
        highlight: style.highlight,
        superscript: style.superscript,
        subscript: style.subscript,
        code: style.code,
        color: None,
    }
}

fn pdf_code_lines(code: &str, language: Option<&str>) -> Vec<Vec<PdfRun>> {
    let highlighted = highlight_code(code, language);
    highlighted
        .into_iter()
        .map(|line| {
            line.into_iter()
                .map(|span| PdfRun {
                    text: span.text,
                    style: PdfStyle {
                        code: true,
                        color: pdf_highlight_color(span.kind),
                        ..PdfStyle::default()
                    },
                    ..PdfRun::default()
                })
                .collect()
        })
        .collect()
}

fn pdf_highlight_color(kind: HighlightKind) -> Option<Rgb> {
    // Light-theme palette so exported code stays readable on white paper.
    match kind {
        HighlightKind::Plain => None,
        HighlightKind::Keyword => Some(Rgb(0xcf, 0x22, 0x2e)),
        HighlightKind::String => Some(Rgb(0x0f, 0x76, 0x2b)),
        HighlightKind::Number => Some(Rgb(0x09, 0x55, 0xa5)),
        HighlightKind::Comment => Some(Rgb(0x6e, 0x77, 0x80)),
        HighlightKind::Type => Some(Rgb(0x00, 0x70, 0x90)),
    }
}

fn pdf_inline_math(math: &crate::model::MathSource, link: Option<&str>) -> PdfRun {
    match MathRenderer::new().render_inline(&math.latex) {
        Ok(rendered) => PdfRun {
            text: String::new(),
            link: link.map(str::to_string),
            inline_image: Some(PdfInlineImage {
                data: PdfImageData::Svg(rendered.svg),
                width_px: rendered.dimensions.width,
                height_px: rendered.dimensions.height,
                ascent_px: rendered.ascent,
                alt: math.authored.clone(),
            }),
            ..PdfRun::default()
        },
        Err(_) => PdfRun {
            text: math.authored.clone(),
            style: PdfStyle {
                code: true,
                ..PdfStyle::default()
            },
            link: link.map(str::to_string),
            ..PdfRun::default()
        },
    }
}

fn pdf_math_block(latex: &str, authored: &str) -> PdfBlock {
    match MathRenderer::new().render_block(latex) {
        Ok(rendered) => {
            let width_px = rendered.dimensions.width.round() as u32;
            let height_px = rendered.dimensions.height.round() as u32;
            PdfBlock::Image {
                data: PdfImageData::Svg(rendered.svg),
                alt: authored.to_string(),
                width_px,
                height_px,
            }
        }
        Err(_) => PdfBlock::CodeBlock {
            language: Some("latex".to_string()),
            lines: vec![vec![PdfRun {
                text: authored.to_string(),
                style: PdfStyle {
                    code: true,
                    ..PdfStyle::default()
                },
                ..PdfRun::default()
            }]],
        },
    }
}

fn pdf_image_block(
    alt: &str,
    url: &str,
    base_dir: &Option<PathBuf>,
    remote_images: &HashMap<String, Vec<u8>>,
) -> Option<PdfBlock> {
    let bytes = resolve_image_bytes(url, base_dir, remote_images)?;
    let (data, width_px, height_px) = match normalize_embedded_image(&bytes)? {
        EmbeddableImage::Png {
            bytes,
            width_px,
            height_px,
        } => (PdfImageData::Png(bytes), width_px, height_px),
        EmbeddableImage::Jpeg {
            bytes,
            width_px,
            height_px,
        } => (PdfImageData::Jpeg(bytes), width_px, height_px),
        EmbeddableImage::Svg(svg) => {
            let (w, h) = svg_dimensions(&svg).unwrap_or((300, 100));
            (PdfImageData::Svg(svg), w, h)
        }
    };
    Some(PdfBlock::Image {
        data,
        alt: alt.to_string(),
        width_px,
        height_px,
    })
}

/// Resolves an image URL to raw bytes for either built-in writer: remote
/// (`http(s)`) URLs resolve from the prefetched map (a missing entry means
/// the fetch failed, so the caller keeps the text fallback), `data:` URIs
/// decode inline with no network access, and anything else reads a local
/// file against the document directory.
fn resolve_image_bytes(
    url: &str,
    base_dir: &Option<PathBuf>,
    remote_images: &HashMap<String, Vec<u8>>,
) -> Option<Vec<u8>> {
    if url.starts_with("http://") || url.starts_with("https://") {
        return remote_images.get(url).cloned();
    }
    if url.starts_with("data:") {
        return decode_data_url_bytes(url);
    }
    let decoded = percent_decode_str(url).decode_utf8().ok()?;
    let path = base_dir.as_ref()?.join(decoded.as_ref());
    fs::read(&path).ok()
}

/// An image payload reduced to one of the embeddable forms shared by the
/// built-in DOCX and PDF writers.
pub(crate) enum EmbeddableImage {
    Png {
        bytes: Vec<u8>,
        width_px: u32,
        height_px: u32,
    },
    Jpeg {
        bytes: Vec<u8>,
        width_px: u32,
        height_px: u32,
    },
    /// Vector markup; consumers either embed it natively (the PDF writer) or
    /// rasterize it (the DOCX writer).
    Svg(String),
}

/// System-font database for export-time SVG rasterization — usvg drops
/// `<text>` nodes when no fonts are loaded, which would strip labels from
/// vector illustrations.
static EXPORT_FONT_DB: LazyLock<Arc<usvg::fontdb::Database>> = LazyLock::new(|| {
    let mut db = usvg::fontdb::Database::new();
    db.load_system_fonts();
    Arc::new(db)
});

/// Supersampling factor for export-time SVG rasterization (DOCX embedding):
/// the raster doubles in resolution while the reported size stays natural.
const EXPORT_SVG_SUPERSAMPLE: f32 = 2.0;
/// Long-edge cap for SVG rasters so pathological vectors cannot exhaust
/// memory during export.
const EXPORT_SVG_MAX_EDGE: u32 = 4096;

/// Normalizes a resolved image payload for embedding: PNG/JPEG pass through
/// with sniffed dimensions, SVG payloads (detected and validated by content)
/// pass through as vector text, and every other decodable raster family
/// (GIF, WebP, …) is decoded and re-encoded as in-memory PNG. `None`
/// (undecodable input) keeps the caller's text fallback.
pub(crate) fn normalize_embedded_image(bytes: &[u8]) -> Option<EmbeddableImage> {
    if let Some((kind, width_px, height_px)) = docx_image_dimensions(bytes) {
        let bytes = bytes.to_vec();
        return Some(match kind {
            DocxImageKind::Png => EmbeddableImage::Png {
                bytes,
                width_px,
                height_px,
            },
            DocxImageKind::Jpeg => EmbeddableImage::Jpeg {
                bytes,
                width_px,
                height_px,
            },
        });
    }
    if looks_like_svg_bytes(bytes)
        && let Ok(svg) = std::str::from_utf8(bytes)
        && usvg::Tree::from_str(svg, &usvg::Options::default()).is_ok()
    {
        return Some(EmbeddableImage::Svg(svg.to_string()));
    }
    let decoded = image::load_from_memory(bytes).ok()?;
    let width_px = decoded.width();
    let height_px = decoded.height();
    let mut png = Vec::new();
    decoded
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    Some(EmbeddableImage::Png {
        bytes: png,
        width_px,
        height_px,
    })
}

/// Cheap leading-bytes heuristic for SVG payloads, mirroring the preview
/// loader's detector but scanning a wider prefix (remote SVGs commonly
/// carry an XML prolog before the root element).
fn looks_like_svg_bytes(bytes: &[u8]) -> bool {
    bytes
        .windows(4)
        .take(512)
        .any(|window| window == b"<svg" || window == b"<SVG")
}

/// Rasterizes one SVG payload to PNG bytes plus its natural pixel size. The
/// raster is supersampled (and capped) for crispness; the returned size is
/// the SVG's intrinsic size, so callers embed at natural dimensions.
fn rasterize_svg_png(svg: &str) -> Option<(Vec<u8>, u32, u32)> {
    let options = usvg::Options {
        fontdb: EXPORT_FONT_DB.clone(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(svg, &options).ok()?;
    let size = tree.size();
    let width = size.width().ceil().max(1.0) as u32;
    let height = size.height().ceil().max(1.0) as u32;
    let scale = if width.max(height) > EXPORT_SVG_MAX_EDGE {
        (EXPORT_SVG_MAX_EDGE as f32) / (width.max(height) as f32)
    } else {
        1.0
    };
    let raster_scale = scale * EXPORT_SVG_SUPERSAMPLE;
    let raster_width = ((width as f32) * raster_scale).ceil().max(1.0) as u32;
    let raster_height = ((height as f32) * raster_scale).ceil().max(1.0) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(raster_width, raster_height)?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(raster_scale, raster_scale),
        &mut pixmap.as_mut(),
    );
    let buffer = image::RgbaImage::from_raw(raster_width, raster_height, pixmap.take())?;
    let mut png = Vec::new();
    buffer
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    Some((png, width, height))
}

fn svg_dimensions(svg: &str) -> Option<(u32, u32)> {
    let start = svg.find("<svg").or_else(|| svg.find("<SVG"))?;
    let tag_end = svg[start..].find('>')? + start;
    let tag = &svg[start..=tag_end];
    let lower = tag.to_ascii_lowercase();

    if let (Some(width), Some(height)) = (
        extract_svg_attr(&lower, "width").and_then(|v| parse_svg_length(&v)),
        extract_svg_attr(&lower, "height").and_then(|v| parse_svg_length(&v)),
    ) {
        if width > 0.0 && height > 0.0 {
            return Some((width as u32, height as u32));
        }
    }

    if let Some(viewbox) = extract_svg_attr(&lower, "viewbox") {
        let parts: Vec<&str> = viewbox.split_whitespace().collect();
        if parts.len() >= 4 {
            let w: f32 = parts[2].parse().ok()?;
            let h: f32 = parts[3].parse().ok()?;
            if w > 0.0 && h > 0.0 {
                return Some((w as u32, h as u32));
            }
        }
    }

    Some((300, 100))
}

fn extract_svg_attr(tag_lower: &str, name: &str) -> Option<String> {
    let prefix = format!("{}=\"", name);
    let start = tag_lower.find(&prefix)? + prefix.len();
    let end = tag_lower[start..].find('"')?;
    Some(tag_lower[start..start + end].to_string())
}

fn parse_svg_length(value: &str) -> Option<f32> {
    let value = value.trim();
    let numeric: String = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    numeric.parse().ok().filter(|v: &f32| *v > 0.0)
}

/// Bundled pandoc reference document used to style engine-produced DOCX files.
/// Packaged builds carry `assets/` next to the binary (`packager.toml`
/// `resources`); development builds resolve the repository copy via the
/// compile-time manifest dir.
fn bundled_reference_doc_path() -> Option<PathBuf> {
    if let Ok(exe) = env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("assets/templates/reference.docx");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let candidate = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/templates/reference.docx");
    candidate.is_file().then_some(candidate)
}

/// Reference document for the DOCX engine: the configured path when it names
/// an existing file, otherwise the bundled default when resolvable.
fn resolve_reference_doc(configured: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = configured.map(str::trim).filter(|path| !path.is_empty()) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
        tracing::warn!(
            path = %path.display(),
            "configured export.reference_doc not found; using the bundled template"
        );
    }
    bundled_reference_doc_path()
}

pub(crate) fn engine_docx(
    source: &str,
    settings: &ExportPreferences,
    document_dir: Option<&Path>,
) -> Result<Vec<u8>, EngineFailureCategory> {
    let exporter = match settings
        .pandoc_path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        Some(path) => DocxExporter::with_pandoc_path(path),
        None => DocxExporter::new(),
    };
    let docx = &settings.docx;
    let options = docx_engine_options(settings, document_dir);
    // Pandoc always embeds resolvable local images; the text-fallback policy
    // is honored by rewriting image syntax to `alt: url` text up front.
    let stripped;
    let source = if docx.image_policy == DocxImagePolicy::TextFallback {
        stripped = strip_local_images(source);
        &stripped
    } else {
        source
    };
    engine_export(source, &exporter, &options)
}

/// Maps the persisted DOCX export options (plus the reference-doc and
/// resource-path resolution) onto the engine's `ExportOptions`.
fn docx_engine_options(settings: &ExportPreferences, document_dir: Option<&Path>) -> ExportOptions {
    ExportOptions {
        page_size: match settings.docx.page_size {
            DocxPageSize::A4 => typune_export::PageSize::A4,
            DocxPageSize::Letter => typune_export::PageSize::Letter,
            DocxPageSize::Legal => typune_export::PageSize::Legal,
        },
        toc: settings.docx.toc,
        reference_doc: resolve_reference_doc(settings.reference_doc.as_deref()),
        resource_path: document_dir.map(Path::to_path_buf),
        ..ExportOptions::default()
    }
}

/// Rewrites local `![alt](url)` image syntax to `alt: url` plain text so the
/// pandoc engine exports images as text instead of embedding them. Remote and
/// data-URI sources are left untouched (they never embed locally anyway).
fn strip_local_images(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("![") {
        output.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        let Some(close) = after_open.find("](") else {
            output.push_str(&rest[start..]);
            return output;
        };
        let alt = &after_open[..close];
        let after_paren = &after_open[close + 2..];
        let Some(end) = after_paren.find(')') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let url = after_paren[..end].trim();
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
            output.push_str(&rest[start..start + 2 + close + 2 + end + 1]);
        } else if alt.is_empty() {
            output.push_str(&format!("Image: {url}"));
        } else {
            output.push_str(&format!("{alt}: {url}"));
        }
        rest = &after_paren[end + 1..];
    }
    output.push_str(rest);
    output
}

/// Whether the pandoc engine binary can be launched (system PATH or the
/// configured explicit path). Drives the availability line on the
/// Preferences panel Export tab.
pub fn pandoc_available(pandoc_path: Option<&str>) -> bool {
    match pandoc_path.map(str::trim).filter(|path| !path.is_empty()) {
        Some(path) => DocxExporter::with_pandoc_path(path).check_pandoc_available(),
        None => DocxExporter::new().check_pandoc_available(),
    }
}

/// Status-bar message for a completed PDF/DOCX export, disclosing the
/// backend and, when the built-in writer took over from the pandoc
/// preference, the engine failure category. An export under the explicit
/// built-in preference reports neutrally — the user already declined pandoc,
/// so no install hint is owed.
pub fn backend_status_msg(
    backend: ExportBackend,
    engine_failure: Option<EngineFailureCategory>,
) -> Msg {
    match (backend, engine_failure) {
        (ExportBackend::PandocEngine, _) => Msg::StatusExportedEngine,
        (ExportBackend::BuiltIn, None) => Msg::StatusExportedBuiltin,
        (ExportBackend::BuiltIn, Some(EngineFailureCategory::BinaryMissing)) => {
            Msg::StatusExportedBuiltinPandocMissing
        }
        (ExportBackend::BuiltIn, Some(EngineFailureCategory::ConversionError)) => {
            Msg::StatusExportedBuiltinConversionFailed
        }
    }
}

// --- Built-in DOCX fallback --------------------------------------------------
// The fallback writer emits a complete OOXML package by hand: static
// styles/settings/fontTable/theme templates plus per-document
// document.xml / document.xml.rels / numbering.xml parts rendered from the
// cached preview blocks. The container is a deflate-compressed ZIP (method 8,
// via the already-vendored miniz_oxide crate).

/// A4 page width in twips (210 mm); the built-in writer's default page size.
const DOCX_PAGE_WIDTH_TWIPS: u32 = 11906;
/// A4 page height in twips (297 mm).
const DOCX_PAGE_HEIGHT_TWIPS: u32 = 16838;
/// Default page margin in twips (2.54 cm).
const DOCX_PAGE_MARGIN_TWIPS: u32 = 1440;

/// `w:numId` shared by every bulleted list item; nesting depth is carried by
/// `w:ilvl` alone.
const DOCX_BULLET_NUM_ID: u32 = 1;
/// First `w:numId` handed out to contiguous ordered-list groups; each group
/// gets its own concrete numbering instance so numbering restarts per list.
const DOCX_FIRST_ORDERED_NUM_ID: u32 = 2;
/// Hyperlink relationship ids start well above the fixed style/settings
/// relationships in `word/_rels/document.xml.rels`.
const DOCX_FIRST_HYPERLINK_RID: u32 = 100;

const DOCX_CODE_FONT: &str = "Consolas";
/// Image relationship ids start above the hyperlink range.
const DOCX_FIRST_IMAGE_RID: u32 = 1000;
/// Fixed relationship id for `word/footnotes.xml` (emitted only when the
/// document has footnote definitions).
const DOCX_FOOTNOTES_RID: u32 = 6;
/// EMU per pixel at the assumed 96 DPI (914400 EMU per inch).
const DOCX_EMU_PER_PIXEL: u64 = 914400 / 96;
const DOCX_CODE_EAST_ASIA_FONT: &str = "DengXian";

pub(crate) fn write_docx(
    path: &Path,
    document: &MarkdownDocument,
    options: &DocxExportOptions,
    remote_images: &HashMap<String, Vec<u8>>,
) -> io::Result<()> {
    fs::write(path, build_docx_bytes(document, options, remote_images)?)
}

pub(crate) fn build_docx_bytes(
    document: &MarkdownDocument,
    options: &DocxExportOptions,
    remote_images: &HashMap<String, Vec<u8>>,
) -> io::Result<Vec<u8>> {
    let metadata = document.front_matter().ok().flatten();
    let title = metadata
        .as_ref()
        .and_then(|metadata| metadata.title.as_deref())
        .or_else(|| {
            document
                .path()
                .and_then(Path::file_stem)
                .and_then(|stem| stem.to_str())
        })
        .unwrap_or("Untitled");
    let author = metadata
        .as_ref()
        .and_then(|metadata| metadata.author.as_deref())
        .unwrap_or("Markion");
    let date = metadata
        .as_ref()
        .and_then(|metadata| metadata.date.as_deref())
        .unwrap_or("1970-01-01T00:00:00Z");

    let front_matter_title = metadata
        .as_ref()
        .and_then(|metadata| metadata.title.as_deref());
    // Relative image paths resolve against the document's directory; image
    // files are read once per export, here in the render pass.
    let base_dir = document
        .path()
        .and_then(Path::parent)
        .map(Path::to_path_buf);
    let rendered = render_docx_document_xml(
        document,
        front_matter_title,
        base_dir,
        options,
        remote_images,
    );

    let mut entries: Vec<(String, Vec<u8>)> = vec![
        (
            "[Content_Types].xml".to_string(),
            docx_content_types(&rendered).into_bytes(),
        ),
        (
            "_rels/.rels".to_string(),
            docx_root_relationships().into_bytes(),
        ),
        (
            "docProps/core.xml".to_string(),
            docx_core_properties(title, author, date).into_bytes(),
        ),
        (
            "word/document.xml".to_string(),
            rendered.document_xml.clone().into_bytes(),
        ),
        (
            "word/_rels/document.xml.rels".to_string(),
            docx_document_relationships(&rendered).into_bytes(),
        ),
        (
            "word/styles.xml".to_string(),
            DOCX_STYLES_XML.as_bytes().to_vec(),
        ),
        (
            "word/numbering.xml".to_string(),
            docx_numbering_xml(rendered.ordered_groups).into_bytes(),
        ),
        (
            "word/settings.xml".to_string(),
            DOCX_SETTINGS_XML.as_bytes().to_vec(),
        ),
        (
            "word/fontTable.xml".to_string(),
            DOCX_FONT_TABLE_XML.as_bytes().to_vec(),
        ),
        (
            "word/theme/theme1.xml".to_string(),
            DOCX_THEME_XML.as_bytes().to_vec(),
        ),
    ];
    if let Some(footnotes) = &rendered.footnotes_xml {
        entries.push((
            "word/footnotes.xml".to_string(),
            footnotes.clone().into_bytes(),
        ));
    }
    for image in &rendered.images {
        entries.push((format!("word/{}", image.part_name), image.bytes.clone()));
    }
    zip_deflate_entries(entries)
}

fn docx_content_types(rendered: &DocxRenderResult) -> String {
    let mut extras = String::new();
    for (extension, mime) in rendered.image_content_types() {
        extras.push_str(&format!(
            "<Default Extension=\"{extension}\" ContentType=\"{mime}\"/>"
        ));
    }
    if rendered.footnotes_xml.is_some() {
        extras.push_str("<Override PartName=\"/word/footnotes.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml\"/>");
    }
    docx_content_types_base().replace("</Types>", &format!("{extras}</Types>"))
}

fn docx_content_types_base() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
<Override PartName="/word/settings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"/>
<Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
<Override PartName="/word/fontTable.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml"/>
<Override PartName="/word/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
</Types>"#
        .to_string()
}

fn docx_root_relationships() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#
        .to_string()
}

fn docx_core_properties(title: &str, author: &str, date: &str) -> String {
    let title = escape_xml_text(title);
    let author = escape_xml_text(author);
    let date = escape_xml_text(&docx_normalized_datetime(date));
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"><dc:title>{title}</dc:title><dc:creator>{author}</dc:creator><cp:lastModifiedBy>{author}</cp:lastModifiedBy><dcterms:created xsi:type=\"dcterms:W3CDTF\">{date}</dcterms:created><dcterms:modified xsi:type=\"dcterms:W3CDTF\">{date}</dcterms:modified></cp:coreProperties>"
    )
}

fn docx_normalized_datetime(date: &str) -> String {
    let trimmed = date.trim();
    if trimmed.len() == 10
        && trimmed.as_bytes().get(4) == Some(&b'-')
        && trimmed.as_bytes().get(7) == Some(&b'-')
    {
        format!("{trimmed}T00:00:00Z")
    } else if trimmed.ends_with('Z') && trimmed.contains('T') {
        trimmed.to_string()
    } else {
        "1970-01-01T00:00:00Z".to_string()
    }
}

/// Builds `word/_rels/document.xml.rels`: fixed style/settings/numbering/
/// fontTable/theme relationships, collected footnote/image relationships, and
/// one external relationship per deduplicated hyperlink target.
fn docx_document_relationships(rendered: &DocxRenderResult) -> String {
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
<Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="fontTable.xml"/>
<Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>"#,
    );
    for (index, target) in rendered.links.iter().enumerate() {
        rels.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\" Target=\"{}\" TargetMode=\"External\"/>",
            DOCX_FIRST_HYPERLINK_RID + index as u32,
            escape_xml_text(target)
        ));
    }
    if rendered.footnotes_xml.is_some() {
        rels.push_str(&format!(
            "<Relationship Id=\"rId{DOCX_FOOTNOTES_RID}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes\" Target=\"footnotes.xml\"/>"
        ));
    }
    for image in &rendered.images {
        rels.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"{}\"/>",
            image.rid, image.part_name
        ));
    }
    rels.push_str("</Relationships>");
    rels
}

/// Everything the package builder needs from one render pass over the
/// preview blocks.
struct DocxRenderResult {
    document_xml: String,
    links: Vec<String>,
    ordered_groups: u32,
    images: Vec<DocxImage>,
    footnotes_xml: Option<String>,
}

impl DocxRenderResult {
    /// Deduplicated (extension, MIME) pairs declared for embedded images.
    fn image_content_types(&self) -> Vec<(&str, &str)> {
        let mut pairs: Vec<(&str, &str)> = Vec::new();
        for image in &self.images {
            if !pairs.iter().any(|(ext, _)| *ext == image.extension) {
                pairs.push((image.extension, image.content_type));
            }
        }
        pairs
    }
}

/// One local image copied into `word/media/` during the render pass.
struct DocxImage {
    rid: u32,
    part_name: String,
    content_type: &'static str,
    extension: &'static str,
    bytes: Vec<u8>,
}

/// Mutable state for the `word/document.xml` render pass: accumulates body
/// XML, the deduplicated external hyperlink targets, ordered-list grouping so
/// each contiguous ordered list gets a fresh numbering instance, embedded
/// images, and footnote definitions collected in a pre-pass.
struct DocxRenderState {
    body: String,
    links: Vec<String>,
    ordered_group: Option<u32>,
    ordered_group_count: u32,
    base_dir: Option<PathBuf>,
    images: Vec<DocxImage>,
    /// Remote (`http(s)`) image bytes prefetched by the export flow, keyed by
    /// the exact source URL. A URL missing from the map failed to fetch and
    /// keeps the text fallback.
    remote_images: HashMap<String, Vec<u8>>,
    /// (label, text) pairs; the `w:footnote` id is the 1-based position.
    footnotes: Vec<(String, RichText)>,
    /// Page geometry from the export options (drives `w:sectPr` and the text
    /// column width tables/images are sized against).
    page_width_twips: u32,
    page_height_twips: u32,
    /// When false, images keep the `alt: url` text fallback.
    embed_images: bool,
}

impl Default for DocxRenderState {
    fn default() -> Self {
        Self {
            body: String::new(),
            links: Vec::new(),
            ordered_group: None,
            ordered_group_count: 0,
            base_dir: None,
            images: Vec::new(),
            remote_images: HashMap::new(),
            footnotes: Vec::new(),
            page_width_twips: DOCX_PAGE_WIDTH_TWIPS,
            page_height_twips: DOCX_PAGE_HEIGHT_TWIPS,
            embed_images: true,
        }
    }
}

impl DocxRenderState {
    /// Text column width (page width minus both margins) in twips.
    fn text_column_twips(&self) -> u32 {
        self.page_width_twips - 2 * DOCX_PAGE_MARGIN_TWIPS
    }

    /// Text column width in EMU — the maximum embedded-image width.
    fn text_column_emu(&self) -> u64 {
        u64::from(self.text_column_twips()) * 635
    }
    fn link_rid(&mut self, target: &str) -> u32 {
        if let Some(index) = self.links.iter().position(|known| known == target) {
            return DOCX_FIRST_HYPERLINK_RID + index as u32;
        }
        self.links.push(target.to_string());
        DOCX_FIRST_HYPERLINK_RID + self.links.len() as u32 - 1
    }

    /// Footnote id (1-based) for a definition label collected in the
    /// pre-pass; in-text references render as superscript spans whose text is
    /// the label.
    fn footnote_id(&self, label: &str) -> Option<u32> {
        self.footnotes
            .iter()
            .position(|(known, _)| known == label)
            .map(|index| index as u32 + 1)
    }

    /// Embeds a normalized image into the package and returns its `w:drawing`
    /// run XML. Bytes resolve from three sources — local files (relative to
    /// the document directory), prefetched remote (`http(s)`) images, and
    /// decoded `data:` URIs — then pass through the shared normalizer: PNG
    /// and JPEG embed as-is, other raster payloads (GIF, WebP, …) re-encode
    /// as PNG, and SVG rasterizes to PNG. Unresolvable or undecodable
    /// sources and the text-fallback image policy return `None` so callers
    /// keep the text fallback.
    fn embed_image(&mut self, alt: &str, url: &str) -> Option<String> {
        if !self.embed_images {
            return None;
        }
        let bytes = self.image_bytes(url)?;
        let (kind, bytes, width_px, height_px) = match normalize_embedded_image(&bytes)? {
            EmbeddableImage::Png {
                bytes,
                width_px,
                height_px,
            } => (DocxImageKind::Png, bytes, width_px, height_px),
            EmbeddableImage::Jpeg {
                bytes,
                width_px,
                height_px,
            } => (DocxImageKind::Jpeg, bytes, width_px, height_px),
            EmbeddableImage::Svg(svg) => {
                let (bytes, width_px, height_px) = rasterize_svg_png(&svg)?;
                (DocxImageKind::Png, bytes, width_px, height_px)
            }
        };
        let mut cx = u64::from(width_px) * DOCX_EMU_PER_PIXEL;
        let mut cy = u64::from(height_px) * DOCX_EMU_PER_PIXEL;
        let column_emu = self.text_column_emu();
        if cx > column_emu {
            cy = cy * column_emu / cx;
            cx = column_emu;
        }
        let doc_pr_id = self.images.len() as u32 + 1;
        let rid = DOCX_FIRST_IMAGE_RID + self.images.len() as u32;
        let part_name = format!("media/image{doc_pr_id}.{}", kind.extension());
        self.images.push(DocxImage {
            rid,
            part_name: part_name.clone(),
            content_type: kind.content_type(),
            extension: kind.extension(),
            bytes,
        });
        Some(docx_drawing_run(
            rid, doc_pr_id, &part_name, cx as u32, cy as u32, alt,
        ))
    }

    /// Source resolution for [`Self::embed_image`] — the shared
    /// three-source resolver over the render state's fields.
    fn image_bytes(&self, url: &str) -> Option<Vec<u8>> {
        resolve_image_bytes(url, &self.base_dir, &self.remote_images)
    }

    /// Ends the current contiguous ordered-list group; the next ordered item
    /// starts a fresh `w:num` so numbering restarts.
    fn end_list_group(&mut self) {
        self.ordered_group = None;
    }

    fn ordered_num_id(&mut self) -> u32 {
        if let Some(num_id) = self.ordered_group {
            return num_id;
        }
        self.ordered_group_count += 1;
        let num_id = DOCX_FIRST_ORDERED_NUM_ID + self.ordered_group_count - 1;
        self.ordered_group = Some(num_id);
        num_id
    }

    fn push_paragraph(
        &mut self,
        style: Option<&str>,
        numbering: Option<(u32, u32)>,
        indent_left: Option<u32>,
        runs: &str,
    ) {
        self.push_paragraph_ex(style, numbering, indent_left, "", runs);
    }

    /// `push_paragraph` plus extra raw `w:pPr` XML (borders for horizontal
    /// rules and alert callouts) appended after the standard properties.
    fn push_paragraph_ex(
        &mut self,
        style: Option<&str>,
        numbering: Option<(u32, u32)>,
        indent_left: Option<u32>,
        extra_ppr: &str,
        runs: &str,
    ) {
        let mut ppr = String::new();
        if let Some(style) = style {
            ppr.push_str(&format!("<w:pStyle w:val=\"{style}\"/>"));
        }
        if let Some((ilvl, num_id)) = numbering {
            ppr.push_str(&format!(
                "<w:numPr><w:ilvl w:val=\"{ilvl}\"/><w:numId w:val=\"{num_id}\"/></w:numPr>"
            ));
        }
        if let Some(left) = indent_left {
            ppr.push_str(&format!("<w:ind w:left=\"{left}\"/>"));
        }
        ppr.push_str(extra_ppr);
        if ppr.is_empty() {
            self.body.push_str(&format!("<w:p>{runs}</w:p>"));
        } else {
            self.body
                .push_str(&format!("<w:p><w:pPr>{ppr}</w:pPr>{runs}</w:p>"));
        }
    }

    /// Emits one run per `RichText` span, mapping inline styles to `w:rPr`
    /// and links to `w:hyperlink` elements backed by external relationships.
    /// Falls back to the concatenated plain text when no spans were captured.
    fn rich_runs(&mut self, rich: &RichText) -> String {
        if rich.spans.is_empty() {
            return docx_plain_runs(&rich.text);
        }
        let mut runs = String::new();
        for span in &rich.spans {
            if let Some(math) = &span.math {
                runs.push_str(&format!("<m:oMath>{}</m:oMath>", omml_inner(&math.latex)));
                continue;
            }
            if let Some(image) = &span.image {
                if let Some(drawing) = self.embed_image(&image.alt, &image.url) {
                    runs.push_str(&drawing);
                } else {
                    let label = if image.alt.is_empty() {
                        image.url.as_str()
                    } else {
                        image.alt.as_str()
                    };
                    runs.push_str(&docx_plain_runs(label));
                }
                continue;
            }
            if span.text.is_empty() {
                continue;
            }
            if span.style.superscript
                && let Some(id) = self.footnote_id(&span.text)
            {
                runs.push_str(&format!(
                    "<w:r><w:rPr><w:vertAlign w:val=\"superscript\"/></w:rPr><w:footnoteReference w:id=\"{id}\"/></w:r>"
                ));
                continue;
            }
            match span.link.as_deref().filter(|link| !link.is_empty()) {
                Some(link) => {
                    let rid = self.link_rid(link);
                    runs.push_str(&format!(
                        "<w:hyperlink r:id=\"rId{rid}\">{}</w:hyperlink>",
                        docx_run(&span.text, &span.style, true)
                    ));
                }
                None => runs.push_str(&docx_run(&span.text, &span.style, false)),
            }
        }
        if runs.is_empty() {
            runs.push_str(&docx_plain_runs(""));
        }
        runs
    }
}

fn docx_plain_runs(text: &str) -> String {
    docx_run(text, &InlineStyle::default(), false)
}

fn docx_run(text: &str, style: &InlineStyle, hyperlink: bool) -> String {
    format!(
        "<w:r>{}<w:t xml:space=\"preserve\">{}</w:t></w:r>",
        docx_run_properties(style, hyperlink),
        escape_xml_text(text)
    )
}

/// Maps `InlineStyle` flags onto a single `w:rPr` so nested inline styles
/// (e.g. bold italic) compose on one run.
fn docx_run_properties(style: &InlineStyle, hyperlink: bool) -> String {
    if style.is_plain() && !hyperlink {
        return String::new();
    }
    let mut props = String::new();
    if hyperlink {
        props.push_str("<w:rStyle w:val=\"Hyperlink\"/>");
    }
    if style.code {
        props.push_str(&format!(
            "<w:rFonts w:ascii=\"{DOCX_CODE_FONT}\" w:hAnsi=\"{DOCX_CODE_FONT}\" w:eastAsia=\"{DOCX_CODE_EAST_ASIA_FONT}\"/>"
        ));
    }
    if style.bold {
        props.push_str("<w:b/>");
    }
    if style.italic {
        props.push_str("<w:i/>");
    }
    if style.strikethrough {
        props.push_str("<w:strike/>");
    }
    if style.highlight {
        props.push_str("<w:highlight w:val=\"yellow\"/>");
    }
    if style.superscript {
        props.push_str("<w:vertAlign w:val=\"superscript\"/>");
    }
    if style.subscript {
        props.push_str("<w:vertAlign w:val=\"subscript\"/>");
    }
    format!("<w:rPr>{props}</w:rPr>")
}

fn render_docx_document_xml(
    document: &MarkdownDocument,
    front_matter_title: Option<&str>,
    base_dir: Option<PathBuf>,
    options: &DocxExportOptions,
    remote_images: &HashMap<String, Vec<u8>>,
) -> DocxRenderResult {
    let (page_width_twips, page_height_twips) = options.page_size.dimensions_twips();
    let mut state = DocxRenderState {
        base_dir,
        page_width_twips,
        page_height_twips,
        embed_images: options.image_policy == DocxImagePolicy::Embed,
        remote_images: remote_images.clone(),
        ..DocxRenderState::default()
    };
    // Collect footnote definitions up front so in-text references resolve to
    // footnote ids during the render pass; the definitions themselves move to
    // `word/footnotes.xml` instead of body paragraphs.
    for block in document.preview_blocks() {
        if let PreviewBlock::FootnoteDefinition { label, text, .. } = block {
            state.footnotes.push((label.clone(), text.clone()));
        }
    }
    if let Some(title) = front_matter_title {
        let runs = docx_plain_runs(title);
        state.push_paragraph(Some("Title"), None, None, &runs);
    }
    for block in document.preview_blocks() {
        render_docx_block(&mut state, &block);
    }
    if state.body.is_empty() {
        let runs = docx_plain_runs("");
        state.push_paragraph(None, None, None, &runs);
    }
    let footnotes_xml = docx_footnotes_xml(&mut state);

    let page_width_twips = state.page_width_twips;
    let page_height_twips = state.page_height_twips;
    let document_xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><w:body>{}<w:sectPr><w:pgSz w:w=\"{page_width_twips}\" w:h=\"{page_height_twips}\"/><w:pgMar w:top=\"{DOCX_PAGE_MARGIN_TWIPS}\" w:right=\"{DOCX_PAGE_MARGIN_TWIPS}\" w:bottom=\"{DOCX_PAGE_MARGIN_TWIPS}\" w:left=\"{DOCX_PAGE_MARGIN_TWIPS}\" w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/></w:sectPr></w:body></w:document>",
        state.body
    );
    DocxRenderResult {
        document_xml,
        links: std::mem::take(&mut state.links),
        ordered_groups: state.ordered_group_count,
        images: std::mem::take(&mut state.images),
        footnotes_xml,
    }
}

/// Renders one list item (top-level or inside a blockquote) as a real Word
/// list paragraph: bullets share `DOCX_BULLET_NUM_ID`, contiguous ordered
/// groups get their own `w:numId`, and task items keep their literal `[x]`/
/// `[ ]` marker prefix on an indented bullet-less paragraph until Word
/// checkbox content controls are adopted.
fn render_docx_list_item(
    state: &mut DocxRenderState,
    style: Option<&str>,
    level: usize,
    ordered: bool,
    checked: Option<bool>,
    text: &RichText,
) {
    // `ListItem.level` is 1-based; `w:ilvl` is 0-based with 9 levels.
    let ilvl = level.saturating_sub(1).min(8) as u32;
    match checked {
        Some(done) => {
            state.end_list_group();
            let prefix = if done { "[x] " } else { "[ ] " };
            let mut runs = docx_plain_runs(prefix);
            runs.push_str(&state.rich_runs(text));
            state.push_paragraph(style, None, Some(720 * (ilvl + 1)), &runs);
        }
        None if ordered => {
            let num_id = state.ordered_num_id();
            let runs = state.rich_runs(text);
            state.push_paragraph(style, Some((ilvl, num_id)), None, &runs);
        }
        None => {
            state.end_list_group();
            let runs = state.rich_runs(text);
            state.push_paragraph(style, Some((ilvl, DOCX_BULLET_NUM_ID)), None, &runs);
        }
    }
}

fn render_docx_block(state: &mut DocxRenderState, block: &PreviewBlock) {
    match block {
        PreviewBlock::Heading { level, text, .. } => {
            state.end_list_group();
            let style = format!("Heading{}", (*level).clamp(1, 6));
            let runs = state.rich_runs(text);
            state.push_paragraph(Some(&style), None, None, &runs);
        }
        PreviewBlock::Paragraph { text, .. } => {
            state.end_list_group();
            let runs = state.rich_runs(text);
            state.push_paragraph(None, None, None, &runs);
        }
        PreviewBlock::ListItem {
            level,
            ordered,
            checked,
            text,
            ..
        } => {
            render_docx_list_item(state, None, *level, *ordered, *checked, text);
        }
        PreviewBlock::BlockQuote {
            children, alert, ..
        } => {
            state.end_list_group();
            if let Some(kind) = alert {
                render_docx_alert(state, *kind, children);
                return;
            }
            for child in children {
                match child {
                    PreviewBlock::ListItem {
                        level,
                        ordered,
                        checked,
                        text,
                        ..
                    } => {
                        render_docx_list_item(
                            state,
                            Some("Quote"),
                            *level,
                            *ordered,
                            *checked,
                            text,
                        );
                    }
                    other => {
                        state.end_list_group();
                        let child_text = other.plain_text();
                        if child_text.is_empty() {
                            continue;
                        }
                        let runs = docx_plain_runs(&child_text);
                        state.push_paragraph(Some("Quote"), None, None, &runs);
                    }
                }
            }
        }
        PreviewBlock::CodeBlock { language, code, .. } => {
            state.end_list_group();
            if let Some(language) = language {
                let runs = docx_plain_runs(&format!("Code: {language}"));
                state.push_paragraph(None, None, None, &runs);
            }
            for line in code.lines() {
                let runs = docx_plain_runs(line);
                state.push_paragraph(Some("CodeBlock"), None, None, &runs);
            }
            if code.is_empty() {
                let runs = docx_plain_runs("");
                state.push_paragraph(Some("CodeBlock"), None, None, &runs);
            }
        }
        PreviewBlock::MathBlock { latex, .. } => {
            state.end_list_group();
            let runs = format!(
                "<m:oMathPara><m:oMath>{}</m:oMath></m:oMathPara>",
                omml_inner(latex)
            );
            state.push_paragraph(None, None, None, &runs);
        }
        PreviewBlock::Html { html, .. } => {
            state.end_list_group();
            for part in html_preview_parts(html) {
                match part {
                    HtmlPreviewPart::Text { text, .. } => {
                        let runs = state.rich_runs(&text);
                        state.push_paragraph(None, None, None, &runs);
                    }
                    HtmlPreviewPart::Image { alt, url, .. } => {
                        render_docx_image(state, &alt, &url);
                    }
                    HtmlPreviewPart::Table { grid } => {
                        render_docx_html_table(state, &grid);
                    }
                }
            }
        }
        PreviewBlock::Image { alt, url, .. } => {
            state.end_list_group();
            render_docx_image(state, alt, url);
        }
        PreviewBlock::Rule { .. } => {
            state.end_list_group();
            state.push_paragraph_ex(
                None,
                None,
                None,
                "<w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"auto\"/></w:pBdr>",
                "",
            );
        }
        // Definitions move to `word/footnotes.xml` (collected in the
        // pre-pass); body paragraphs carry only the `w:footnoteReference`
        // marks emitted by `rich_runs`.
        PreviewBlock::FootnoteDefinition { .. } => {
            state.end_list_group();
        }
        PreviewBlock::Table {
            rows, alignments, ..
        } => {
            state.end_list_group();
            render_docx_table(state, rows, alignments);
        }
    }
}

/// Markdown pipe tables: bold header row with `w:tblHeader` (repeat on page
/// breaks), per-column `w:jc` from the separator row, and a fixed grid
/// spanning the text column with equal column widths.
fn render_docx_table(
    state: &mut DocxRenderState,
    rows: &[Vec<RichText>],
    alignments: &[TableAlignment],
) {
    if rows.is_empty() {
        return;
    }
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
    let column_width = state.text_column_twips() / columns as u32;
    push_table_open(state, columns, column_width);
    for (row_index, row) in rows.iter().enumerate() {
        push_table_row_open(state, row_index == 0);
        for column in 0..columns {
            let props = format!("<w:tcW w:w=\"{column_width}\" w:type=\"dxa\"/>");
            let jc = alignments.get(column).and_then(table_alignment_jc);
            let runs = match row.get(column) {
                Some(cell) if row_index == 0 => state.rich_runs(&bolded(cell)),
                Some(cell) => state.rich_runs(cell),
                None => docx_plain_runs(""),
            };
            push_table_cell(state, &props, jc, &runs);
        }
        state.body.push_str("</w:tr>");
    }
    state.body.push_str("</w:tbl>");
}

/// Raw HTML `<table>` blocks feed the parsed grid (rowspan/colspan resolved
/// upstream) into the same `w:tbl` structure: header cells are bold and the
/// header row gets `w:tblHeader`, `colspan` becomes `w:gridSpan`, and
/// `rowspan` becomes `w:vMerge` restart/continue pairs.
fn render_docx_html_table(state: &mut DocxRenderState, grid: &HtmlTableGrid) {
    let columns = grid.columns.max(1);
    let column_width = state.text_column_twips() / columns as u32;
    push_table_open(state, columns, column_width);
    for row in &grid.rows {
        push_table_row_open(state, row.iter().any(|cell| cell.is_header));
        for cell in row {
            let mut props = format!(
                "<w:tcW w:w=\"{}\" w:type=\"dxa\"/>",
                column_width * cell.colspan.max(1) as u32
            );
            if cell.colspan > 1 {
                props.push_str(&format!("<w:gridSpan w:val=\"{}\"/>", cell.colspan));
            }
            if cell.is_spacer {
                props.push_str("<w:vMerge/>");
                let runs = docx_plain_runs("");
                push_table_cell(state, &props, None, &runs);
                continue;
            }
            if cell.rowspan > 1 {
                props.push_str("<w:vMerge w:val=\"restart\"/>");
            }
            let runs = if cell.is_header {
                state.rich_runs(&bolded(&cell.content))
            } else {
                state.rich_runs(&cell.content)
            };
            push_table_cell(state, &props, None, &runs);
        }
        state.body.push_str("</w:tr>");
    }
    state.body.push_str("</w:tbl>");
}

/// Shared table preamble: full text-column width, single-line borders, and a
/// fixed equal-width grid so Word does not auto-fit columns.
fn push_table_open(state: &mut DocxRenderState, columns: usize, column_width: u32) {
    let mut grid = String::new();
    for _ in 0..columns {
        grid.push_str(&format!("<w:gridCol w:w=\"{column_width}\"/>"));
    }
    let text_column = state.text_column_twips();
    state.body.push_str(&format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"{text_column}\" w:type=\"dxa\"/><w:tblBorders><w:top w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:left w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:right w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:insideH w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:insideV w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/></w:tblBorders></w:tblPr><w:tblGrid>{grid}</w:tblGrid>"
    ));
}

fn push_table_row_open(state: &mut DocxRenderState, header: bool) {
    if header {
        state.body.push_str("<w:tr><w:trPr><w:tblHeader/></w:trPr>");
    } else {
        state.body.push_str("<w:tr>");
    }
}

fn push_table_cell(state: &mut DocxRenderState, props: &str, jc: Option<&str>, runs: &str) {
    state
        .body
        .push_str(&format!("<w:tc><w:tcPr>{props}</w:tcPr>"));
    match jc {
        Some(jc) => state.body.push_str(&format!(
            "<w:p><w:pPr><w:jc w:val=\"{jc}\"/></w:pPr>{runs}</w:p>"
        )),
        None => state.body.push_str(&format!("<w:p>{runs}</w:p>")),
    }
    state.body.push_str("</w:tc>");
}

fn table_alignment_jc(alignment: &TableAlignment) -> Option<&'static str> {
    match alignment {
        TableAlignment::Left => Some("left"),
        TableAlignment::Center => Some("center"),
        TableAlignment::Right => Some("right"),
        TableAlignment::Default => None,
    }
}

/// Clone with `bold` forced on every span (table header cells).
fn bolded(rich: &RichText) -> RichText {
    let mut rich = rich.clone();
    if rich.spans.is_empty() {
        if !rich.text.is_empty() {
            rich.spans.push(InlineSpan {
                text: rich.text.clone(),
                style: InlineStyle {
                    bold: true,
                    ..InlineStyle::default()
                },
                link: None,
                math: None,
                image: None,
            });
        }
        return rich;
    }
    for span in &mut rich.spans {
        span.style.bold = true;
    }
    rich
}

/// Emits an embedded `w:drawing` for a resolvable local PNG/JPEG, or the
/// `alt: url` text fallback for remote, data-URI, or unreadable sources.
fn render_docx_image(state: &mut DocxRenderState, alt: &str, url: &str) {
    if let Some(drawing) = state.embed_image(alt, url) {
        state.push_paragraph(None, None, None, &drawing);
        return;
    }
    let label = if alt.is_empty() { "Image" } else { alt };
    let runs = docx_plain_runs(&format!("{label}: {url}"));
    state.push_paragraph(None, None, None, &runs);
}

/// GFM alert blockquotes become callout paragraphs: a bold kind label plus an
/// accented left border and indentation, instead of `> `-prefixed text.
fn render_docx_alert(state: &mut DocxRenderState, kind: AlertKind, children: &[PreviewBlock]) {
    let (label, color) = match kind {
        AlertKind::Note => ("Note", "0969DA"),
        AlertKind::Tip => ("Tip", "1A7F37"),
        AlertKind::Important => ("Important", "8250DF"),
        AlertKind::Warning => ("Warning", "9A6700"),
        AlertKind::Caution => ("Caution", "D1242F"),
    };
    let border = format!(
        "<w:pBdr><w:left w:val=\"single\" w:sz=\"24\" w:space=\"4\" w:color=\"{color}\"/></w:pBdr>"
    );
    let label_style = InlineStyle {
        bold: true,
        ..InlineStyle::default()
    };
    let runs = docx_run(label, &label_style, false);
    state.push_paragraph_ex(None, None, Some(720), &border, &runs);
    for child in children {
        match child {
            PreviewBlock::ListItem {
                level,
                ordered,
                checked,
                text,
                ..
            } => {
                render_docx_list_item(state, None, *level, *ordered, *checked, text);
            }
            other => {
                state.end_list_group();
                let child_text = other.plain_text();
                if child_text.is_empty() {
                    continue;
                }
                let runs = docx_plain_runs(&child_text);
                state.push_paragraph_ex(None, None, Some(720), &border, &runs);
            }
        }
    }
}

/// OMML for one formula: the supported TeX subset converts to real OMML
/// structures; anything else preserves the authored LaTeX as the math-zone
/// text (never a Unicode approximation, never a `Math: ` prefix).
fn omml_inner(latex: &str) -> String {
    match tex_to_omml(latex) {
        Some(omml) => omml,
        None => format!(
            "<m:r><m:t xml:space=\"preserve\">{}</m:t></m:r>",
            escape_xml_text(latex)
        ),
    }
}

/// Builds `word/footnotes.xml` when the document defines footnotes: separator
/// and continuationSeparator placeholders (ids -1/0) precede the definitions,
/// which start at id 1 in collection order and reuse the inline run builder.
fn docx_footnotes_xml(state: &mut DocxRenderState) -> Option<String> {
    if state.footnotes.is_empty() {
        return None;
    }
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:footnotes xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:footnote w:type=\"separator\" w:id=\"-1\"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>",
    );
    // `rich_runs` borrows the state mutably, so the definitions are taken out
    // for the loop and restored afterwards.
    let footnotes = std::mem::take(&mut state.footnotes);
    for (index, (_, text)) in footnotes.iter().enumerate() {
        let runs = state.rich_runs(text);
        xml.push_str(&format!(
            "<w:footnote w:id=\"{}\"><w:p><w:pPr><w:pStyle w:val=\"FootnoteText\"/></w:pPr><w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr><w:footnoteRef/></w:r>{runs}</w:p></w:footnote>",
            index + 1
        ));
    }
    state.footnotes = footnotes;
    xml.push_str("</w:footnotes>");
    Some(xml)
}

#[derive(Clone, Copy)]
enum DocxImageKind {
    Png,
    Jpeg,
}

impl DocxImageKind {
    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }
}

/// Decodes an RFC 2397 `data:` image URL to raw bytes. Both `;base64` and
/// URL-encoded payloads are supported; the format sniffing stays with
/// `docx_image_dimensions`, so non-embeddable payloads keep the text
/// fallback. Mirrors the preview loader's `data_url`-based decoder.
fn decode_data_url_bytes(url: &str) -> Option<Vec<u8>> {
    let processed = data_url::DataUrl::process(url).ok()?;
    let (bytes, _fragment) = processed.decode_to_vec().ok()?;
    Some(bytes)
}

/// Reads pixel dimensions from PNG (IHDR) or JPEG (SOF0-SOF15) headers without
/// decoding. Other formats (gif/svg/webp) return `None` and keep the text
/// fallback — SVG especially cannot embed without a fallback bitmap.
fn docx_image_dimensions(bytes: &[u8]) -> Option<(DocxImageKind, u32, u32)> {
    if bytes.len() >= 24 && bytes[..8] == *b"\x89PNG\r\n\x1a\n" && bytes[12..16] == *b"IHDR" {
        let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
        let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
        return Some((DocxImageKind::Png, width, height));
    }
    if bytes.len() >= 4 && bytes[0] == 0xff && bytes[1] == 0xd8 {
        let mut cursor = 2usize;
        while cursor + 9 < bytes.len() {
            if bytes[cursor] != 0xff {
                cursor += 1;
                continue;
            }
            let marker = bytes[cursor + 1];
            // SOF markers carry frame dimensions; DHT/DAC/JPG do not.
            if matches!(marker, 0xc0..=0xcf) && !matches!(marker, 0xc4 | 0xc8 | 0xcc) {
                let height = u16::from_be_bytes(bytes[cursor + 5..cursor + 7].try_into().ok()?);
                let width = u16::from_be_bytes(bytes[cursor + 7..cursor + 9].try_into().ok()?);
                return Some((DocxImageKind::Jpeg, u32::from(width), u32::from(height)));
            }
            let length =
                u16::from_be_bytes(bytes[cursor + 2..cursor + 4].try_into().ok()?) as usize;
            if length < 2 {
                return None;
            }
            cursor += 2 + length;
        }
    }
    None
}

/// `w:drawing` run embedding one packaged image inline (`wp:inline`), sized
/// in EMU with the alt text preserved as the drawing description.
fn docx_drawing_run(
    rid: u32,
    doc_pr_id: u32,
    part_name: &str,
    cx: u32,
    cy: u32,
    alt: &str,
) -> String {
    let alt = escape_xml_text(alt);
    let name = escape_xml_text(part_name.rsplit('/').next().unwrap_or(part_name));
    format!(
        "<w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:docPr id=\"{doc_pr_id}\" name=\"Picture {doc_pr_id}\" descr=\"{alt}\"/><a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic><pic:nvPicPr><pic:cNvPr id=\"{doc_pr_id}\" name=\"{name}\"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed=\"rId{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"
    )
}

/// Builds `word/numbering.xml`: one shared bullet abstract definition plus a
/// fresh decimal abstract/concrete pair per contiguous ordered-list group, so
/// each ordered list restarts numbering.
fn docx_numbering_xml(ordered_groups: u32) -> String {
    let mut abstract_nums = docx_bullet_abstract_num(0);
    let mut nums =
        format!("<w:num w:numId=\"{DOCX_BULLET_NUM_ID}\"><w:abstractNumId w:val=\"0\"/></w:num>");
    for group in 0..ordered_groups {
        let abstract_id = group + 1;
        abstract_nums.push_str(&docx_decimal_abstract_num(abstract_id));
        nums.push_str(&format!(
            "<w:num w:numId=\"{}\"><w:abstractNumId w:val=\"{abstract_id}\"/></w:num>",
            DOCX_FIRST_ORDERED_NUM_ID + group
        ));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">{abstract_nums}{nums}</w:numbering>"
    )
}

fn docx_bullet_abstract_num(id: u32) -> String {
    const BULLETS: [&str; 3] = ["•", "◦", "▪"];
    let mut levels = String::new();
    for ilvl in 0..9u32 {
        let bullet = BULLETS[(ilvl as usize) % BULLETS.len()];
        let left = 720 * (ilvl + 1);
        levels.push_str(&format!(
            "<w:lvl w:ilvl=\"{ilvl}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"bullet\"/><w:lvlText w:val=\"{bullet}\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"{left}\" w:hanging=\"360\"/></w:pPr></w:lvl>"
        ));
    }
    format!("<w:abstractNum w:abstractNumId=\"{id}\">{levels}</w:abstractNum>")
}

fn docx_decimal_abstract_num(id: u32) -> String {
    let mut levels = String::new();
    for ilvl in 0..9u32 {
        let left = 720 * (ilvl + 1);
        let text = format!("%{}.", ilvl + 1);
        levels.push_str(&format!(
            "<w:lvl w:ilvl=\"{ilvl}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/><w:lvlText w:val=\"{text}\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"{left}\" w:hanging=\"360\"/></w:pPr></w:lvl>"
        ));
    }
    format!("<w:abstractNum w:abstractNumId=\"{id}\">{levels}</w:abstractNum>")
}

/// `word/styles.xml`: docDefaults (10.5 pt body font with an eastAsia face
/// for CJK text) plus every paragraph/character style referenced from
/// `word/document.xml` — Normal, Title, Heading1-6, Quote, CodeBlock, and
/// Hyperlink, FootnoteText, and FootnoteReference.
const DOCX_STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="DengXian"/><w:sz w:val="21"/><w:szCs w:val="21"/><w:lang w:val="en-US" w:eastAsia="zh-CN"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>
<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:before="240" w:after="240"/></w:pPr><w:rPr><w:rFonts w:ascii="Calibri Light" w:hAnsi="Calibri Light" w:eastAsia="Microsoft YaHei"/><w:b/><w:sz w:val="56"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="120"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:rFonts w:ascii="Calibri Light" w:hAnsi="Calibri Light" w:eastAsia="Microsoft YaHei"/><w:b/><w:sz w:val="32"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="200" w:after="120"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:rFonts w:ascii="Calibri Light" w:hAnsi="Calibri Light" w:eastAsia="Microsoft YaHei"/><w:b/><w:sz w:val="28"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="160" w:after="80"/><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:rFonts w:ascii="Calibri Light" w:hAnsi="Calibri Light" w:eastAsia="Microsoft YaHei"/><w:b/><w:sz w:val="24"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="120" w:after="80"/><w:outlineLvl w:val="3"/></w:pPr><w:rPr><w:rFonts w:ascii="Calibri Light" w:hAnsi="Calibri Light" w:eastAsia="Microsoft YaHei"/><w:b/><w:sz w:val="22"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="120" w:after="80"/><w:outlineLvl w:val="4"/></w:pPr><w:rPr><w:rFonts w:ascii="Calibri Light" w:hAnsi="Calibri Light" w:eastAsia="Microsoft YaHei"/><w:b/><w:sz w:val="21"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="120" w:after="80"/><w:outlineLvl w:val="5"/></w:pPr><w:rPr><w:rFonts w:ascii="Calibri Light" w:hAnsi="Calibri Light" w:eastAsia="Microsoft YaHei"/><w:b/><w:sz w:val="20"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:basedOn w:val="Normal"/><w:qFormat/><w:pPr><w:ind w:left="720"/></w:pPr><w:rPr><w:i/><w:color w:val="595959"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="CodeBlock"><w:name w:val="Code Block"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:before="0" w:after="0"/></w:pPr><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:eastAsia="DengXian"/><w:sz w:val="20"/></w:rPr></w:style>
<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/><w:rPr><w:sz w:val="18"/><w:szCs w:val="18"/></w:rPr></w:style>
<w:style w:type="character" w:styleId="FootnoteReference"><w:name w:val="footnote reference"/><w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>
</w:styles>"#;

const DOCX_SETTINGS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:zoom w:percent="100"/><w:defaultTabStop w:val="720"/></w:settings>"#;

const DOCX_FONT_TABLE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:font w:name="Calibri"><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>
<w:font w:name="Calibri Light"><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>
<w:font w:name="DengXian"><w:family w:val="auto"/><w:pitch w:val="variable"/></w:font>
<w:font w:name="Microsoft YaHei"><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>
<w:font w:name="Consolas"><w:family w:val="modern"/><w:pitch w:val="fixed"/></w:font>
</w:fonts>"#;

/// Minimal valid theme so Word can resolve theme colors/fonts; the fallback
/// never references theme slots directly.
const DOCX_THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
  <a:themeElements>
    <a:clrScheme name="Office">
      <a:dk1>
        <a:sysClr val="windowText" lastClr="000000"/>
      </a:dk1>
      <a:lt1>
        <a:sysClr val="window" lastClr="FFFFFF"/>
      </a:lt1>
      <a:dk2>
        <a:srgbClr val="0E2841"/>
      </a:dk2>
      <a:lt2>
        <a:srgbClr val="E8E8E8"/>
      </a:lt2>
      <a:accent1>
        <a:srgbClr val="156082"/>
      </a:accent1>
      <a:accent2>
        <a:srgbClr val="E97132"/>
      </a:accent2>
      <a:accent3>
        <a:srgbClr val="196B24"/>
      </a:accent3>
      <a:accent4>
        <a:srgbClr val="0F9ED5"/>
      </a:accent4>
      <a:accent5>
        <a:srgbClr val="A02B93"/>
      </a:accent5>
      <a:accent6>
        <a:srgbClr val="4EA72E"/>
      </a:accent6>
      <a:hlink>
        <a:srgbClr val="467886"/>
      </a:hlink>
      <a:folHlink>
        <a:srgbClr val="96607D"/>
      </a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Office">
      <a:majorFont>
        <a:latin typeface="Aptos Display" panose="02110004020202020204"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
        <a:font script="Jpan" typeface="游ゴシック Light"/>
        <a:font script="Hang" typeface="맑은 고딕"/>
        <a:font script="Hans" typeface="等线 Light"/>
        <a:font script="Hant" typeface="新細明體"/>
        <a:font script="Arab" typeface="Times New Roman"/>
        <a:font script="Hebr" typeface="Times New Roman"/>
        <a:font script="Thai" typeface="Angsana New"/>
        <a:font script="Ethi" typeface="Nyala"/>
        <a:font script="Beng" typeface="Vrinda"/>
        <a:font script="Gujr" typeface="Shruti"/>
        <a:font script="Khmr" typeface="MoolBoran"/>
        <a:font script="Knda" typeface="Tunga"/>
        <a:font script="Guru" typeface="Raavi"/>
        <a:font script="Cans" typeface="Euphemia"/>
        <a:font script="Cher" typeface="Plantagenet Cherokee"/>
        <a:font script="Yiii" typeface="Microsoft Yi Baiti"/>
        <a:font script="Tibt" typeface="Microsoft Himalaya"/>
        <a:font script="Thaa" typeface="MV Boli"/>
        <a:font script="Deva" typeface="Mangal"/>
        <a:font script="Telu" typeface="Gautami"/>
        <a:font script="Taml" typeface="Latha"/>
        <a:font script="Syrc" typeface="Estrangelo Edessa"/>
        <a:font script="Orya" typeface="Kalinga"/>
        <a:font script="Mlym" typeface="Kartika"/>
        <a:font script="Laoo" typeface="DokChampa"/>
        <a:font script="Sinh" typeface="Iskoola Pota"/>
        <a:font script="Mong" typeface="Mongolian Baiti"/>
        <a:font script="Viet" typeface="Times New Roman"/>
        <a:font script="Uigh" typeface="Microsoft Uighur"/>
        <a:font script="Geor" typeface="Sylfaen"/>
        <a:font script="Armn" typeface="Arial"/>
        <a:font script="Bugi" typeface="Leelawadee UI"/>
        <a:font script="Bopo" typeface="Microsoft JhengHei"/>
        <a:font script="Java" typeface="Javanese Text"/>
        <a:font script="Lisu" typeface="Segoe UI"/>
        <a:font script="Mymr" typeface="Myanmar Text"/>
        <a:font script="Nkoo" typeface="Ebrima"/>
        <a:font script="Olck" typeface="Nirmala UI"/>
        <a:font script="Osma" typeface="Ebrima"/>
        <a:font script="Phag" typeface="Phagspa"/>
        <a:font script="Syrn" typeface="Estrangelo Edessa"/>
        <a:font script="Syrj" typeface="Estrangelo Edessa"/>
        <a:font script="Syre" typeface="Estrangelo Edessa"/>
        <a:font script="Sora" typeface="Nirmala UI"/>
        <a:font script="Tale" typeface="Microsoft Tai Le"/>
        <a:font script="Talu" typeface="Microsoft New Tai Lue"/>
        <a:font script="Tfng" typeface="Ebrima"/>
      </a:majorFont>
      <a:minorFont>
        <a:latin typeface="Aptos" panose="02110004020202020204"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
        <a:font script="Jpan" typeface="游明朝"/>
        <a:font script="Hang" typeface="맑은 고딕"/>
        <a:font script="Hans" typeface="等线"/>
        <a:font script="Hant" typeface="新細明體"/>
        <a:font script="Arab" typeface="Arial"/>
        <a:font script="Hebr" typeface="Arial"/>
        <a:font script="Thai" typeface="Cordia New"/>
        <a:font script="Ethi" typeface="Nyala"/>
        <a:font script="Beng" typeface="Vrinda"/>
        <a:font script="Gujr" typeface="Shruti"/>
        <a:font script="Khmr" typeface="DaunPenh"/>
        <a:font script="Knda" typeface="Tunga"/>
        <a:font script="Guru" typeface="Raavi"/>
        <a:font script="Cans" typeface="Euphemia"/>
        <a:font script="Cher" typeface="Plantagenet Cherokee"/>
        <a:font script="Yiii" typeface="Microsoft Yi Baiti"/>
        <a:font script="Tibt" typeface="Microsoft Himalaya"/>
        <a:font script="Thaa" typeface="MV Boli"/>
        <a:font script="Deva" typeface="Mangal"/>
        <a:font script="Telu" typeface="Gautami"/>
        <a:font script="Taml" typeface="Latha"/>
        <a:font script="Syrc" typeface="Estrangelo Edessa"/>
        <a:font script="Orya" typeface="Kalinga"/>
        <a:font script="Mlym" typeface="Kartika"/>
        <a:font script="Laoo" typeface="DokChampa"/>
        <a:font script="Sinh" typeface="Iskoola Pota"/>
        <a:font script="Mong" typeface="Mongolian Baiti"/>
        <a:font script="Viet" typeface="Arial"/>
        <a:font script="Uigh" typeface="Microsoft Uighur"/>
        <a:font script="Geor" typeface="Sylfaen"/>
        <a:font script="Armn" typeface="Arial"/>
        <a:font script="Bugi" typeface="Leelawadee UI"/>
        <a:font script="Bopo" typeface="Microsoft JhengHei"/>
        <a:font script="Java" typeface="Javanese Text"/>
        <a:font script="Lisu" typeface="Segoe UI"/>
        <a:font script="Mymr" typeface="Myanmar Text"/>
        <a:font script="Nkoo" typeface="Ebrima"/>
        <a:font script="Olck" typeface="Nirmala UI"/>
        <a:font script="Osma" typeface="Ebrima"/>
        <a:font script="Phag" typeface="Phagspa"/>
        <a:font script="Syrn" typeface="Estrangelo Edessa"/>
        <a:font script="Syrj" typeface="Estrangelo Edessa"/>
        <a:font script="Syre" typeface="Estrangelo Edessa"/>
        <a:font script="Sora" typeface="Nirmala UI"/>
        <a:font script="Tale" typeface="Microsoft Tai Le"/>
        <a:font script="Talu" typeface="Microsoft New Tai Lue"/>
        <a:font script="Tfng" typeface="Ebrima"/>
      </a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Office">
      <a:fillStyleLst>
        <a:solidFill>
          <a:schemeClr val="phClr"/>
        </a:solidFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0">
              <a:schemeClr val="phClr">
                <a:lumMod val="110000"/>
                <a:satMod val="105000"/>
                <a:tint val="67000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="50000">
              <a:schemeClr val="phClr">
                <a:lumMod val="105000"/>
                <a:satMod val="103000"/>
                <a:tint val="73000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="100000">
              <a:schemeClr val="phClr">
                <a:lumMod val="105000"/>
                <a:satMod val="109000"/>
                <a:tint val="81000"/>
              </a:schemeClr>
            </a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0">
              <a:schemeClr val="phClr">
                <a:satMod val="103000"/>
                <a:lumMod val="102000"/>
                <a:tint val="94000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="50000">
              <a:schemeClr val="phClr">
                <a:satMod val="110000"/>
                <a:lumMod val="100000"/>
                <a:shade val="100000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="100000">
              <a:schemeClr val="phClr">
                <a:lumMod val="99000"/>
                <a:satMod val="120000"/>
                <a:shade val="78000"/>
              </a:schemeClr>
            </a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="6350" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill>
            <a:schemeClr val="phClr"/>
          </a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
        <a:ln w="12700" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill>
            <a:schemeClr val="phClr"/>
          </a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
        <a:ln w="19050" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill>
            <a:schemeClr val="phClr"/>
          </a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle>
          <a:effectLst/>
        </a:effectStyle>
        <a:effectStyle>
          <a:effectLst/>
        </a:effectStyle>
        <a:effectStyle>
          <a:effectLst>
            <a:outerShdw blurRad="57150" dist="19050" dir="5400000" algn="ctr" rotWithShape="0">
              <a:srgbClr val="000000">
                <a:alpha val="63000"/>
              </a:srgbClr>
            </a:outerShdw>
          </a:effectLst>
        </a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill>
          <a:schemeClr val="phClr"/>
        </a:solidFill>
        <a:solidFill>
          <a:schemeClr val="phClr">
            <a:tint val="95000"/>
            <a:satMod val="170000"/>
          </a:schemeClr>
        </a:solidFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0">
              <a:schemeClr val="phClr">
                <a:tint val="93000"/>
                <a:satMod val="150000"/>
                <a:shade val="98000"/>
                <a:lumMod val="102000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="50000">
              <a:schemeClr val="phClr">
                <a:tint val="98000"/>
                <a:satMod val="130000"/>
                <a:shade val="90000"/>
                <a:lumMod val="103000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="100000">
              <a:schemeClr val="phClr">
                <a:shade val="63000"/>
                <a:satMod val="120000"/>
              </a:schemeClr>
            </a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
  <a:objectDefaults/>
  <a:extraClrSchemeLst/>
  <a:extLst>
    <a:ext uri="{05A4C25C-085E-4340-85A3-A5531E510DB2}">
      <thm15:themeFamily xmlns:thm15="http://schemas.microsoft.com/office/thememl/2012/main" name="Office Theme" id="{2E142A2C-CD16-42D6-873A-C26D2A0506FA}" vid="{1BDDFF52-6CD6-40A5-AB3C-68EB2F1E4D0A}"/>
    </a:ext>
  </a:extLst>
</a:theme>
"#;

/// Writes the OOXML package as a ZIP with deflate-compressed entries
/// (method 8). Central-directory records carry the same CRC and compressed/
/// uncompressed sizes as their local file headers.
fn zip_deflate_entries(entries: Vec<(String, Vec<u8>)>) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut records = Vec::new();
    for (name, data) in entries {
        let offset = u32::try_from(output.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "DOCX archive too large"))?;
        let name_bytes = name.as_bytes();
        let name_len = u16::try_from(name_bytes.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "DOCX entry name too long"))?;
        let size = u32::try_from(data.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "DOCX entry too large"))?;
        let crc = crc32(&data);
        let compressed = miniz_oxide::deflate::compress_to_vec(&data, 6);
        let compressed_size = u32::try_from(compressed.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "DOCX entry too large"))?;

        push_u32_le(&mut output, 0x0403_4b50);
        push_u16_le(&mut output, 20);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 8); // compression method 8: deflate
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 33);
        push_u32_le(&mut output, crc);
        push_u32_le(&mut output, compressed_size);
        push_u32_le(&mut output, size);
        push_u16_le(&mut output, name_len);
        push_u16_le(&mut output, 0);
        output.extend_from_slice(name_bytes);
        output.extend_from_slice(&compressed);

        records.push(ZipEntryRecord {
            name,
            crc,
            compressed_size,
            size,
            offset,
        });
    }

    let central_directory_offset = u32::try_from(output.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "DOCX central directory offset too large",
        )
    })?;
    for record in &records {
        let name_bytes = record.name.as_bytes();
        let name_len = u16::try_from(name_bytes.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "DOCX entry name too long"))?;
        push_u32_le(&mut output, 0x0201_4b50);
        push_u16_le(&mut output, 20);
        push_u16_le(&mut output, 20);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 8);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 33);
        push_u32_le(&mut output, record.crc);
        push_u32_le(&mut output, record.compressed_size);
        push_u32_le(&mut output, record.size);
        push_u16_le(&mut output, name_len);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 0);
        push_u32_le(&mut output, 0);
        push_u32_le(&mut output, record.offset);
        output.extend_from_slice(name_bytes);
    }
    let central_directory_size = u32::try_from(output.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "DOCX central directory size too large",
        )
    })? - central_directory_offset;
    let entry_count = u16::try_from(records.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "too many DOCX entries"))?;

    push_u32_le(&mut output, 0x0605_4b50);
    push_u16_le(&mut output, 0);
    push_u16_le(&mut output, 0);
    push_u16_le(&mut output, entry_count);
    push_u16_le(&mut output, entry_count);
    push_u32_le(&mut output, central_directory_size);
    push_u32_le(&mut output, central_directory_offset);
    push_u16_le(&mut output, 0);

    Ok(output)
}

struct ZipEntryRecord {
    name: String,
    crc: u32,
    compressed_size: u32,
    size: u32,
    offset: u32,
}

fn push_u16_le(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u32_le(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

/// Exports the rendered Markdown layout as a PNG/JPEG snapshot.
///
/// Unlike the previous ASCII-only 8x8 bitmap renderer, this reuses the PDF
/// layout IR ([`build_pdf_ir`]) and the `markion-pdf` cosmic-text font
/// pipeline (bundled Noto Sans SC for CJK plus system fonts), so CJK text
/// renders as real glyphs and headings/code/tables keep the same typography
/// as PDF export. The document flows into one tall, continuous image.
pub(crate) fn write_image_export(
    path: &Path,
    document: &MarkdownDocument,
    settings: &ExportPreferences,
    format: image::ImageFormat,
) -> io::Result<()> {
    let base_dir = document.path().and_then(Path::parent);
    let ir = build_pdf_ir(document, &settings.pdf, base_dir, &HashMap::new());
    let image =
        markion_pdf::render_snapshot(&ir, markion_pdf::DEFAULT_SCALE).map_err(io::Error::other)?;
    let mut dynamic = image::DynamicImage::ImageRgba8(image);
    // JPEG has no alpha channel; flatten the (already opaque) RGBA snapshot
    // onto RGB8 or the encoder rejects the color type.
    if format == image::ImageFormat::Jpeg {
        dynamic = dynamic.to_rgb8().into();
    }
    dynamic
        .save_with_format(path, format)
        .map_err(io::Error::other)
}

/// Test helper: extracts one entry from a hand-written DOCX ZIP by walking
/// local file headers, inflating deflate (method 8) payloads. Used by the
/// package-structure tests in this module and in `lib.rs`.
#[cfg(test)]
pub(crate) fn read_zip_entry(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    let mut cursor = 0usize;
    while cursor + 30 <= bytes.len() && bytes[cursor..cursor + 4] == [0x50, 0x4b, 0x03, 0x04] {
        let read_u16 = |offset: usize| {
            u16::from_le_bytes([bytes[cursor + offset], bytes[cursor + offset + 1]])
        };
        let read_u32 = |offset: usize| {
            u32::from_le_bytes(
                bytes[cursor + offset..cursor + offset + 4]
                    .try_into()
                    .unwrap(),
            )
        };
        let method = read_u16(8);
        let compressed_size = read_u32(18) as usize;
        let name_len = read_u16(26) as usize;
        let extra_len = read_u16(28) as usize;
        let name_start = cursor + 30;
        let entry_name = std::str::from_utf8(&bytes[name_start..name_start + name_len]).ok()?;
        let data_start = name_start + name_len + extra_len;
        if entry_name == name {
            let payload = &bytes[data_start..data_start + compressed_size];
            return match method {
                0 => Some(payload.to_vec()),
                8 => miniz_oxide::inflate::decompress_to_vec(payload).ok(),
                _ => None,
            };
        }
        cursor = data_start + compressed_size;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarkdownDocument;
    use markion_pdf;

    fn docx_parts(document: &MarkdownDocument) -> Vec<u8> {
        build_docx_bytes(document, &DocxExportOptions::default(), &HashMap::new())
            .expect("DOCX package build")
    }

    fn entry<'a>(bytes: &'a [u8], name: &str) -> String {
        let data = read_zip_entry(bytes, name).unwrap_or_else(|| panic!("missing part {name}"));
        String::from_utf8(data).expect("part is UTF-8")
    }

    /// Package-structure validation shared by the DOCX tests: the ZIP magic,
    /// the core part inventory every producer (built-in writer or pandoc)
    /// must carry, and style-reference resolution — every `w:pStyle` referenced
    /// from `word/document.xml` must be defined in `word/styles.xml`.
    fn assert_openable_docx_package(bytes: &[u8]) {
        assert!(bytes.len() > 4, "package too small");
        assert_eq!(&bytes[0..2], b"PK", "DOCX must be a ZIP package");
        const CORE_PARTS: [&str; 4] = [
            "[Content_Types].xml",
            "_rels/.rels",
            "word/document.xml",
            "word/styles.xml",
        ];
        for part in CORE_PARTS {
            assert!(
                read_zip_entry(bytes, part).is_some(),
                "missing package part {part}"
            );
        }

        let document_xml = entry(bytes, "word/document.xml");
        let styles_xml = entry(bytes, "word/styles.xml");
        let mut rest = document_xml.as_str();
        while let Some(start) = rest.find("<w:pStyle w:val=\"") {
            let value_start = start + "<w:pStyle w:val=\"".len();
            let value_end = rest[value_start..].find('"').unwrap() + value_start;
            let style = &rest[value_start..value_end];
            assert!(
                styles_xml.contains(&format!("w:styleId=\"{style}\"")),
                "style {style} referenced but not defined"
            );
            rest = &rest[value_end..];
        }
    }

    /// Minimal package check tolerant of foreign ZIP layouts (pandoc may set
    /// data-descriptor flags our local-header walker does not parse): PK magic
    /// plus the core part names, which ZIP archives always store uncompressed.
    fn assert_docx_part_names_present(bytes: &[u8]) {
        assert!(bytes.len() > 4, "package too small");
        assert_eq!(&bytes[0..2], b"PK", "DOCX must be a ZIP package");
        for part in [
            "[Content_Types].xml",
            "_rels/.rels",
            "word/document.xml",
            "word/styles.xml",
        ] {
            assert!(
                bytes
                    .windows(part.len())
                    .any(|window| window == part.as_bytes()),
                "missing package part {part}"
            );
        }
    }

    /// Regression: a hand-rolled theme with an under-populated `bgFillStyleLst`
    /// made Word reject every exported package as corrupt (2026-08). The theme
    /// is now pandoc's battle-tested Office theme embedded verbatim; pin the
    /// style-matrix lists Word validates strictly.
    #[test]
    fn theme_part_carries_complete_style_matrix() {
        let doc = MarkdownDocument::from_text("# t\n\nx");
        let theme = entry(&docx_parts(&doc), "word/theme/theme1.xml");
        for list in [
            "<a:fillStyleLst>",
            "<a:lnStyleLst>",
            "<a:effectStyleLst>",
            "<a:bgFillStyleLst>",
        ] {
            assert!(theme.contains(list), "theme missing {list}");
        }
        let bg = theme.split("<a:bgFillStyleLst>").nth(1).unwrap();
        let bg = bg.split("</a:bgFillStyleLst>").next().unwrap();
        let fills = [
            "<a:solidFill>",
            "<a:gradFill",
            "<a:blipFill",
            "<a:pattFill",
            "<a:grpFill",
            "<a:noFill",
        ]
        .iter()
        .map(|tag| bg.matches(tag).count())
        .sum::<usize>();
        assert!(
            fills >= 3,
            "bgFillStyleLst needs at least three fills for Word"
        );
    }

    #[test]
    #[ignore] // requires pandoc; run explicitly with `cargo test -- --ignored`
    fn engine_and_fallback_both_produce_openable_packages() {
        let exporter = DocxExporter::new();
        if !exporter.check_pandoc_available() {
            return;
        }
        let source =
            "# Fixture\n\nBody with **bold** and a [link](https://example.com).\n\n- one\n- two\n";
        let settings = ExportPreferences::default();
        let engine_bytes =
            engine_docx(source, &settings, None).expect("pandoc engine export should succeed");
        assert_docx_part_names_present(&engine_bytes);

        let document = MarkdownDocument::from_text(source);
        let fallback_bytes = docx_parts(&document);
        assert_openable_docx_package(&fallback_bytes);
    }

    #[test]
    fn package_contains_all_required_parts() {
        let document = MarkdownDocument::from_text("# Hi\n\nBody");
        let bytes = docx_parts(&document);
        const PARTS: [&str; 10] = [
            "[Content_Types].xml",
            "_rels/.rels",
            "docProps/core.xml",
            "word/document.xml",
            "word/_rels/document.xml.rels",
            "word/styles.xml",
            "word/numbering.xml",
            "word/settings.xml",
            "word/fontTable.xml",
            "word/theme/theme1.xml",
        ];
        for part in PARTS {
            assert!(
                read_zip_entry(&bytes, part).is_some(),
                "missing package part {part}"
            );
        }
    }

    #[test]
    fn every_referenced_style_resolves() {
        let document = MarkdownDocument::from_text(
            "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6\n\n> quote\n\n```rust\nfn main() {}\n```\n\n[link](https://example.com)",
        );
        let bytes = docx_parts(&document);
        assert_openable_docx_package(&bytes);
        let styles_xml = entry(&bytes, "word/styles.xml");

        for style in ["Normal", "Title", "Quote", "CodeBlock", "Hyperlink"] {
            assert!(
                styles_xml.contains(&format!("w:styleId=\"{style}\"")),
                "style {style} not defined"
            );
        }
    }

    #[test]
    fn headings_keep_all_six_levels() {
        let document =
            MarkdownDocument::from_text("# A\n\n## B\n\n### C\n\n#### D\n\n##### E\n\n###### F\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        for level in 1..=6 {
            assert!(
                document_xml.contains(&format!("<w:pStyle w:val=\"Heading{level}\"/>")),
                "Heading{level} missing"
            );
        }
    }

    #[test]
    fn front_matter_title_uses_title_style() {
        let document = MarkdownDocument::from_text("---\ntitle: My Paper\n---\nBody");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<w:pStyle w:val=\"Title\"/>"));
        assert!(document_xml.contains(">My Paper</w:t>"));
        let core = entry(&bytes, "docProps/core.xml");
        assert!(core.contains("<dc:title>My Paper</dc:title>"));
    }

    #[test]
    fn inline_styles_and_hyperlink_survive() {
        let document = MarkdownDocument::from_text(
            "**bold** *italic* ~~gone~~ `code` ==marked== H~2~O x^2^ [site](https://example.com/?a=1&b=2)",
        );
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<w:b/>"));
        assert!(document_xml.contains("<w:i/>"));
        assert!(document_xml.contains("<w:strike/>"));
        assert!(document_xml.contains("<w:highlight w:val=\"yellow\"/>"));
        assert!(document_xml.contains("<w:vertAlign w:val=\"superscript\"/>"));
        assert!(document_xml.contains("<w:vertAlign w:val=\"subscript\"/>"));
        assert!(document_xml.contains("<w:rFonts w:ascii=\"Consolas\""));
        assert!(document_xml.contains("<w:hyperlink r:id=\"rId100\">"));
        assert!(document_xml.contains("<w:rStyle w:val=\"Hyperlink\"/>"));

        let rels = entry(&bytes, "word/_rels/document.xml.rels");
        assert!(rels.contains("Target=\"https://example.com/?a=1&amp;b=2\""));
        assert!(rels.contains("TargetMode=\"External\""));
    }

    #[test]
    fn nested_inline_styles_compose_on_one_run() {
        let mut state = DocxRenderState::default();
        let rich = RichText {
            text: "bold italic".to_string(),
            spans: vec![crate::model::InlineSpan {
                text: "bold italic".to_string(),
                style: InlineStyle {
                    bold: true,
                    italic: true,
                    ..InlineStyle::default()
                },
                link: None,
                math: None,
                image: None,
            }],
        };
        let runs = state.rich_runs(&rich);
        assert!(runs.contains("<w:b/>"));
        assert!(runs.contains("<w:i/>"));
        // Both flags live in a single run, not split across runs.
        assert_eq!(runs.matches("<w:r>").count(), 1);
    }

    #[test]
    fn empty_link_target_falls_back_to_styled_text() {
        let mut state = DocxRenderState::default();
        let rich = RichText {
            text: "label".to_string(),
            spans: vec![crate::model::InlineSpan {
                text: "label".to_string(),
                style: InlineStyle::default(),
                link: Some(String::new()),
                math: None,
                image: None,
            }],
        };
        let runs = state.rich_runs(&rich);
        assert!(!runs.contains("<w:hyperlink"));
        assert!(runs.contains(">label</w:t>"));
        assert!(state.links.is_empty());
    }

    #[test]
    fn nested_bullet_list_keeps_ilvl_depth() {
        let document = MarkdownDocument::from_text("- a\n  - b\n    - c\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<w:ilvl w:val=\"0\"/>"));
        assert!(document_xml.contains("<w:ilvl w:val=\"1\"/>"));
        assert!(document_xml.contains("<w:ilvl w:val=\"2\"/>"));
        assert!(document_xml.contains(&format!("<w:numId w:val=\"{DOCX_BULLET_NUM_ID}\"/>")));
        assert!(!document_xml.contains(">- a</w:t>"));

        let numbering = entry(&bytes, "word/numbering.xml");
        assert!(numbering.contains("<w:numFmt w:val=\"bullet\"/>"));
        // Nine levels per abstract numbering definition.
        let bullet_abstract = numbering
            .split("</w:abstractNum>")
            .next()
            .expect("bullet abstract numbering");
        assert_eq!(bullet_abstract.matches("<w:lvl ").count(), 9);
    }

    #[test]
    fn ordered_lists_are_auto_numbered_and_restart() {
        let document = MarkdownDocument::from_text("1. one\n2. two\n\nplain\n\n1. uno\n2. dos\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        let numbering = entry(&bytes, "word/numbering.xml");

        assert!(numbering.contains("<w:numFmt w:val=\"decimal\"/>"));
        // No literal "1. " marker text is emitted.
        assert!(!document_xml.contains(">1. one</w:t>"));
        // Two contiguous ordered groups get distinct numbering instances.
        assert!(document_xml.contains("<w:numId w:val=\"2\"/>"));
        assert!(document_xml.contains("<w:numId w:val=\"3\"/>"));
        assert!(numbering.contains("<w:abstractNum w:abstractNumId=\"1\">"));
        assert!(numbering.contains("<w:abstractNum w:abstractNumId=\"2\">"));
    }

    #[test]
    fn task_items_keep_checkbox_prefix() {
        let document = MarkdownDocument::from_text("- [x] done\n- [ ] todo\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains(">[x] </w:t>"));
        assert!(document_xml.contains(">[ ] </w:t>"));
    }

    #[test]
    fn page_setup_is_a4_with_default_margins() {
        let document = MarkdownDocument::from_text("Body");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<w:pgSz w:w=\"11906\" w:h=\"16838\"/>"));
        assert!(document_xml.contains("w:top=\"1440\""));
        assert!(document_xml.contains("w:left=\"1440\""));
    }

    #[test]
    fn page_setup_honors_letter_and_legal_options() {
        let document = MarkdownDocument::from_text("Body");
        for (page_size, dims) in [
            (DocxPageSize::Letter, (12240, 15840)),
            (DocxPageSize::Legal, (12240, 20160)),
        ] {
            let options = DocxExportOptions {
                page_size,
                ..DocxExportOptions::default()
            };
            let bytes =
                build_docx_bytes(&document, &options, &HashMap::new()).expect("DOCX package build");
            let document_xml = entry(&bytes, "word/document.xml");
            assert!(
                document_xml.contains(&format!("<w:pgSz w:w=\"{}\" w:h=\"{}\"/>", dims.0, dims.1)),
                "{page_size:?} should drive w:pgSz"
            );
        }
    }

    #[test]
    fn text_fallback_image_policy_skips_embedding() {
        let dir = tempfile::tempdir().unwrap();
        write_png_fixture(&dir.path().join("diagram.png"), 2, 1);
        let md = dir.path().join("doc.md");
        fs::write(&md, "![diagram](diagram.png)").unwrap();
        let document = MarkdownDocument::open(&md).unwrap();
        let options = DocxExportOptions {
            image_policy: DocxImagePolicy::TextFallback,
            ..DocxExportOptions::default()
        };
        let bytes =
            build_docx_bytes(&document, &options, &HashMap::new()).expect("DOCX package build");
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("diagram: diagram.png"));
        assert!(!document_xml.contains("<w:drawing>"));
        assert!(read_zip_entry(&bytes, "word/media/image1.png").is_none());
    }

    #[test]
    fn docx_options_map_onto_engine_export_options() {
        let mut settings = ExportPreferences::default();
        settings.docx = DocxExportOptions {
            page_size: DocxPageSize::Letter,
            toc: true,
            image_policy: DocxImagePolicy::TextFallback,
        };
        let options = docx_engine_options(&settings, Some(Path::new("/docs")));
        assert_eq!(options.page_size, typune_export::PageSize::Letter);
        assert!(options.toc);
        assert_eq!(options.resource_path.as_deref(), Some(Path::new("/docs")));
        // reference_doc resolves to the bundled template when unset (dev
        // builds) or None (packaged without assets); either way the mapping
        // must not panic.
        let _ = &options.reference_doc;

        let options = docx_engine_options(&ExportPreferences::default(), None);
        assert_eq!(options.page_size, typune_export::PageSize::A4);
        assert!(!options.toc);
        assert!(options.resource_path.is_none());
    }

    #[test]
    fn strip_local_images_rewrites_only_local_sources() {
        let source = "![a](pic.png)\n\n![r](https://example.com/x.png)\n\n![](local.jpg)\n\n![d](data:image/png;base64,AA)";
        let stripped = strip_local_images(source);
        assert!(stripped.contains("a: pic.png"));
        assert!(stripped.contains("![r](https://example.com/x.png)"));
        assert!(stripped.contains("Image: local.jpg"));
        assert!(stripped.contains("![d](data:image/png;base64,AA)"));
    }

    #[test]
    fn cjk_fonts_are_declared() {
        let document = MarkdownDocument::from_text("# 标题\n\n正文");
        let bytes = docx_parts(&document);
        let styles_xml = entry(&bytes, "word/styles.xml");
        assert!(styles_xml.contains("<w:eastAsia w:val") || styles_xml.contains("w:eastAsia="));
        let doc_defaults = styles_xml
            .split("<w:style ")
            .next()
            .expect("docDefaults section");
        assert!(doc_defaults.contains("w:eastAsia=\"DengXian\""));
        assert!(styles_xml.contains("w:eastAsia=\"Microsoft YaHei\""));
        assert!(document_xml(&bytes).contains("标题"));
    }

    fn document_xml(bytes: &[u8]) -> String {
        entry(bytes, "word/document.xml")
    }

    #[test]
    fn text_and_link_targets_are_escaped() {
        // Note: pulldown-cmark smart punctuation rewrites ASCII quotes in
        // prose, so the text side exercises `<`, `>`, `&`; the attribute side
        // exercises quotes and `&` inside the link target.
        let document =
            MarkdownDocument::from_text("a < b & c > d\n\n[x](https://a.com/?q=\"v\"&k=1)");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("a &lt; b &amp; c &gt; d"));
        assert!(!document_xml.contains("&b=2\""));
        let rels = entry(&bytes, "word/_rels/document.xml.rels");
        assert!(rels.contains("Target=\"https://a.com/?q=&quot;v&quot;&amp;k=1\""));
    }

    #[test]
    fn link_targets_are_deduplicated() {
        let document =
            MarkdownDocument::from_text("[a](https://example.com) and [b](https://example.com)");
        let bytes = docx_parts(&document);
        let rels = entry(&bytes, "word/_rels/document.xml.rels");
        assert_eq!(rels.matches("Target=\"https://example.com\"").count(), 1);
    }

    #[test]
    fn entries_are_deflate_compressed() {
        let document = MarkdownDocument::from_text("Body");
        let bytes = docx_parts(&document);
        // Local file header compression-method field (offset 8) is 8=deflate.
        assert_eq!(bytes[8..10], [8, 0]);
        assert!(entry(&bytes, "word/document.xml").contains("<w:body>"));
    }

    /// Minimal PNG header fixture: the embedder only reads IHDR dimensions;
    /// the bytes after the header are irrelevant for packaging.
    fn write_png_fixture(path: &Path, width: u32, height: u32) {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
        bytes.extend_from_slice(&[0; 4]);
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn local_png_is_embedded_with_extent() {
        let dir = tempfile::tempdir().unwrap();
        write_png_fixture(&dir.path().join("diagram.png"), 2, 1);
        let md = dir.path().join("doc.md");
        fs::write(&md, "![diagram](diagram.png)").unwrap();
        let document = MarkdownDocument::open(&md).unwrap();
        let bytes = docx_parts(&document);

        let media = read_zip_entry(&bytes, "word/media/image1.png").expect("media part");
        assert!(media.starts_with(b"\x89PNG\r\n\x1a\n"));
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<w:drawing>"));
        // 2 px at 96 DPI = 19050 EMU; a narrow image keeps its natural size.
        assert!(document_xml.contains("cx=\"19050\""));
        assert!(document_xml.contains("descr=\"diagram\""));
        let rels = entry(&bytes, "word/_rels/document.xml.rels");
        assert!(rels.contains("relationships/image\" Target=\"media/image1.png\""));
        let types = entry(&bytes, "[Content_Types].xml");
        assert!(types.contains("<Default Extension=\"png\" ContentType=\"image/png\"/>"));
    }

    #[test]
    fn oversized_image_scales_to_text_column() {
        let dir = tempfile::tempdir().unwrap();
        write_png_fixture(&dir.path().join("wide.png"), 2000, 1000);
        let md = dir.path().join("doc.md");
        fs::write(&md, "![wide](wide.png)").unwrap();
        let document = MarkdownDocument::open(&md).unwrap();
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        // Natural width 2000 px = 19_050_000 EMU exceeds the 5_731_510 EMU
        // text column; height scales proportionally (1000/2000).
        assert!(document_xml.contains("cx=\"5731510\" cy=\"2865755\""));
    }

    #[test]
    fn missing_remote_and_data_uri_images_keep_text_fallback() {
        let document = MarkdownDocument::from_text(
            "![gone](missing.png)\n\n![remote](https://example.com/a.png)\n\n![inline](data:image/png;base64,AAAA)",
        );
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("gone: missing.png"));
        assert!(document_xml.contains("remote: https://example.com/a.png"));
        assert!(!document_xml.contains("<w:drawing>"));
        assert!(read_zip_entry(&bytes, "word/media/image1.png").is_none());
    }

    #[test]
    fn table_header_repeats_bold_and_alignment_applies() {
        let document =
            MarkdownDocument::from_text("| A | B | C |\n|:---|:---:|---:|\n| 1 | 2 | 3 |\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<w:tblHeader/>"));
        assert!(document_xml.contains("<w:tblW w:w=\"9026\" w:type=\"dxa\"/>"));
        assert!(document_xml.contains("<w:jc w:val=\"left\"/>"));
        assert!(document_xml.contains("<w:jc w:val=\"center\"/>"));
        assert!(document_xml.contains("<w:jc w:val=\"right\"/>"));
        // The header row's runs are bold.
        let header_start = document_xml.find("<w:tblHeader/>").unwrap();
        let header_end = document_xml[header_start..].find("</w:tr>").unwrap() + header_start;
        assert!(document_xml[header_start..header_end].contains("<w:b/>"));
    }

    #[test]
    fn html_table_becomes_one_real_table() {
        let document = MarkdownDocument::from_text(
            "<table><tr><th>H1</th><th>H2</th></tr><tr><td>a</td><td>b</td></tr></table>",
        );
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert_eq!(document_xml.matches("<w:tbl>").count(), 1);
        assert_eq!(document_xml.matches("<w:tc>").count(), 4);
        assert!(document_xml.contains("<w:tblHeader/>"));
    }

    #[test]
    fn display_math_becomes_omml_structures() {
        let document = MarkdownDocument::from_text("$$\n\\frac{a}{b} + \\sqrt{x}\n$$\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<m:oMathPara>"));
        assert!(document_xml.contains("<m:f><m:num>"));
        assert!(document_xml.contains("<m:rad>"));
        assert!(!document_xml.contains("Math:"));
    }

    #[test]
    fn inline_math_becomes_omml() {
        let document = MarkdownDocument::from_text("Energy is $x^2 + \\alpha$ here\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<m:oMath>"));
        assert!(document_xml.contains("<m:sSup>"));
        assert!(document_xml.contains(">α</m:t>"));
    }

    #[test]
    fn unsupported_math_preserves_authored_latex() {
        let document = MarkdownDocument::from_text("$$\n\\begin{matrix} a \\end{matrix}\n$$\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<m:oMathPara>"));
        assert!(document_xml.contains("\\begin{matrix} a \\end{matrix}"));
        assert!(!document_xml.contains("Math:"));
    }

    #[test]
    fn footnotes_link_references_to_definitions() {
        let document =
            MarkdownDocument::from_text("See the note[^a] here.\n\n[^a]: Footnote text\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<w:footnoteReference w:id=\"1\"/>"));
        assert!(!document_xml.contains("[^a]"));
        let footnotes = entry(&bytes, "word/footnotes.xml");
        assert!(footnotes.contains("w:type=\"separator\""));
        assert!(footnotes.contains("<w:footnote w:id=\"1\">"));
        assert!(footnotes.contains("Footnote text"));
        let rels = entry(&bytes, "word/_rels/document.xml.rels");
        assert!(rels.contains("relationships/footnotes\" Target=\"footnotes.xml\""));
        let types = entry(&bytes, "[Content_Types].xml");
        assert!(types.contains("/word/footnotes.xml"));
    }

    #[test]
    fn horizontal_rule_is_a_paragraph_border() {
        let document = MarkdownDocument::from_text("Above\n\n---\n\nBelow\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("<w:pBdr><w:bottom w:val=\"single\""));
        assert!(!document_xml.contains("----------"));
    }

    #[test]
    fn gfm_alert_renders_as_callout() {
        let document = MarkdownDocument::from_text("> [!WARNING]\n> Be careful.\n");
        let bytes = docx_parts(&document);
        let document_xml = entry(&bytes, "word/document.xml");
        assert!(document_xml.contains("w:color=\"9A6700\""));
        assert!(document_xml.contains("<w:ind w:left=\"720\"/>"));
        let label_start = document_xml.find(">Warning</w:t>").expect("bold label");
        assert!(document_xml[..label_start].contains("<w:b/>"));
        assert!(document_xml.contains("Be careful."));
    }

    #[test]
    fn engine_failure_category_maps_tool_not_found_to_binary_missing() {
        let missing = ExportError::ToolNotFound("pandoc not found".to_string());
        assert_eq!(
            engine_failure_category(&missing),
            EngineFailureCategory::BinaryMissing
        );
        let failed = ExportError::GenerationError("pandoc exited with error".to_string());
        assert_eq!(
            engine_failure_category(&failed),
            EngineFailureCategory::ConversionError
        );
    }

    #[test]
    fn backend_status_msg_discloses_backend_and_failure_category() {
        assert_eq!(
            backend_status_msg(ExportBackend::PandocEngine, None),
            Msg::StatusExportedEngine
        );
        // Explicit built-in preference (no engine attempt) reports neutrally
        // for both formats — the user already declined pandoc.
        assert_eq!(
            backend_status_msg(ExportBackend::BuiltIn, None),
            Msg::StatusExportedBuiltin
        );
        assert_eq!(
            backend_status_msg(
                ExportBackend::BuiltIn,
                Some(EngineFailureCategory::BinaryMissing)
            ),
            Msg::StatusExportedBuiltinPandocMissing
        );
        assert_eq!(
            backend_status_msg(
                ExportBackend::BuiltIn,
                Some(EngineFailureCategory::ConversionError)
            ),
            Msg::StatusExportedBuiltinConversionFailed
        );
    }

    #[test]
    fn bundled_reference_doc_resolves_in_dev_builds() {
        // cargo test runs from the repo root, so the compile-time manifest-dir
        // fallback must find the checked-in template.
        let path = bundled_reference_doc_path().expect("bundled reference.docx resolves");
        assert!(path.is_file());
        assert_eq!(path.file_name().unwrap(), "reference.docx");
    }

    #[test]
    fn resolve_reference_doc_prefers_configured_existing_file() {
        let bundled = bundled_reference_doc_path().expect("bundled reference.docx resolves");
        let resolved = resolve_reference_doc(Some(bundled.to_str().unwrap()));
        assert_eq!(resolved.as_deref(), Some(bundled.as_path()));

        // A configured path that does not exist falls back to the bundled one.
        let resolved = resolve_reference_doc(Some("/nonexistent/my-reference.docx"));
        assert_eq!(resolved.as_deref(), Some(bundled.as_path()));

        // Blank values behave like unset.
        let resolved = resolve_reference_doc(Some("   "));
        assert_eq!(resolved.as_deref(), Some(bundled.as_path()));
    }

    #[test]
    fn pdf_fallback_renders_cjk_rich_fixture() {
        // Render the built-in PDF fallback for a mixed CJK/Latin document with
        // headings, lists, a table, a code block, and a math block. This guards
        // the deleted `plain_pdf_text` "?" substitution and confirms the IR
        // builder and markion-pdf layout engine produce a real document.
        let doc = MarkdownDocument::from_text(
            "# 标题 (Title)\n\nMixed CJK and Latin paragraph with **bold** and *italic*.\n\n- 第一项\n- 第二项\n\n| Name | 值 |\n|---|---|\n| Ada | 十 |\n| Bob | 百 |\n\n```rust\nfn main() { println!(\"hello\"); }\n```\n\n$$\nE = mc^2\n$$\n",
        );
        let ir = build_pdf_ir(&doc, &PdfExportOptions::default(), None, &HashMap::new());
        let bytes = markion_pdf::render(&ir).expect("built-in PDF render should succeed");
        assert!(
            bytes.starts_with(b"%PDF-"),
            "rendered output should be a PDF file"
        );
        assert!(
            bytes.len() > 1024,
            "rendered PDF should contain real content, not a minimal placeholder"
        );
        let text = String::from_utf8_lossy(&bytes);
        let page_count = text
            .split("/Count")
            .nth(1)
            .and_then(|s| s.trim_start().split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        assert!(page_count >= 1, "PDF should contain at least one page");
    }

    #[test]
    fn pdf_inline_math_becomes_svg_run() {
        let doc = MarkdownDocument::from_text("The formula $E = mc^2$ is famous.\n");
        let ir = build_pdf_ir(&doc, &PdfExportOptions::default(), None, &HashMap::new());
        let content = match ir.blocks.first() {
            Some(PdfBlock::Paragraph { content }) => content,
            other => panic!("expected a paragraph, got {other:?}"),
        };
        let math = content
            .iter()
            .find(|run| run.inline_image.is_some())
            .expect("inline math should become an IR inline image");
        match &math.inline_image.as_ref().unwrap().data {
            PdfImageData::Svg(svg) => {
                assert!(
                    svg.to_ascii_lowercase().contains("<svg"),
                    "inline math SVG missing, got {svg}"
                )
            }
            other => panic!("expected SVG payload, got {other:?}"),
        }
        assert!(math.inline_image.as_ref().unwrap().width_px > 0.0);
        assert!(
            !content
                .iter()
                .any(|run| run.style.code && run.text.contains("E = mc^2")),
            "valid inline math must not fall back to a code-styled source run"
        );
        markion_pdf::render(&ir).expect("PDF with inline math should render");
    }

    #[test]
    fn pdf_unrenderable_inline_math_keeps_authored_source() {
        let math = crate::model::MathSource {
            latex: String::new(),
            authored: "$ $".to_string(),
            style: crate::model::MathLayoutStyle::Text,
            delimiter: crate::model::MathDelimiter::InlineDollar,
            source_range: 0..3,
        };
        let run = pdf_inline_math(&math, None);
        assert!(run.inline_image.is_none());
        assert!(run.style.code);
        assert_eq!(run.text, "$ $");
    }

    /// Regression (2026-08): JPEG has no alpha channel, so saving the RGBA
    /// snapshot failed with "Jpeg does not support the color type 'Rgba8'";
    /// the exporter must flatten to RGB8 first. Both snapshots must also span
    /// the full A4 page width at the default scale — a content-width canvas
    /// truncated the right margin's worth of text.
    #[test]
    fn image_export_writes_jpeg_and_png_snapshots() {
        let document = MarkdownDocument::from_text("# 标题\n\nMixed CJK and Latin paragraph.\n");
        let settings = ExportPreferences::default();
        let dir = tempfile::tempdir().unwrap();

        let jpg_path = dir.path().join("snapshot.jpg");
        write_image_export(&jpg_path, &document, &settings, image::ImageFormat::Jpeg)
            .expect("JPEG snapshot export");
        let jpg = image::open(&jpg_path).expect("written JPEG decodes");
        assert_eq!(jpg.color(), image::ColorType::Rgb8);

        let png_path = dir.path().join("snapshot.png");
        write_image_export(&png_path, &document, &settings, image::ImageFormat::Png)
            .expect("PNG snapshot export");
        let png = image::open(&png_path).expect("written PNG decodes");
        assert_eq!(png.color(), image::ColorType::Rgba8);

        let page_w = (210.0 * 72.0 / 25.4 * markion_pdf::DEFAULT_SCALE).round() as u32;
        assert_eq!(jpg.width(), page_w, "JPEG must include both margins");
        assert_eq!(png.width(), page_w, "PNG must include both margins");
    }
}
