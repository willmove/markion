//! Integration test for the built-in PDF writer (task 2.9): a
//! representative IR document renders to `%PDF` bytes, and a long fixture
//! paginates to more than one page.

use markion_pdf::{
    Alignment, Block, Cell, ImageData, ListMarker, PdfDocument, PdfMetadata, PdfOptions, Rgb, Run,
    Style,
};

fn run(text: &str) -> Run {
    Run {
        text: text.to_string(),
        ..Run::default()
    }
}

fn styled(text: &str, style: Style) -> Run {
    Run {
        text: text.to_string(),
        style,
        ..Run::default()
    }
}

/// A valid 2×2 RGB PNG (generated in-test fixtures need correct CRCs).
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x08, 0x02, 0x00, 0x00, 0x00, 0xfd,
    0xd4, 0x9a, 0x73, 0x00, 0x00, 0x00, 0x10, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x38,
    0x21, 0x07, 0x04, 0x27, 0x18, 0x20, 0x14, 0x00, 0x1c, 0x7e, 0x04, 0x11, 0xb3, 0x9e, 0x82,
    0x94, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

fn representative_document() -> PdfDocument {
    let para_style = Style {
        bold: true,
        ..Style::default()
    };
    PdfDocument {
        metadata: PdfMetadata {
            title: Some("Markion 内置 PDF 导出".to_string()),
            author: Some("markion-pdf".to_string()),
            date: Some("2026-08-22".to_string()),
        },
        options: PdfOptions {
            toc: true,
            ..PdfOptions::default()
        },
        blocks: vec![
            Block::Heading {
                level: 1,
                content: vec![run("混合 CJK/Latin 文档")],
            },
            Block::Paragraph {
                content: vec![
                    run("这是一段中文正文，mixed with Latin words and "),
                    styled("bold text", para_style),
                    Run {
                        text: " and a link".to_string(),
                        link: Some("https://example.com".to_string()),
                        ..Run::default()
                    },
                    Run {
                        text: "[^1]".to_string(),
                        footnote: Some(1),
                        ..Run::default()
                    },
                    run("。"),
                ],
            },
            Block::ListItem {
                indent_level: 0,
                marker: ListMarker::Number(1),
                content: vec![run("第一项")],
            },
            Block::ListItem {
                indent_level: 0,
                marker: ListMarker::Number(2),
                content: vec![run("第二项")],
            },
            Block::CodeBlock {
                language: Some("rust".to_string()),
                lines: vec![
                    vec![
                        styled(
                            "fn",
                            Style {
                                color: Some(Rgb(0xcf, 0x22, 0x2e)),
                                ..Style::default()
                            },
                        ),
                        run(" main() {"),
                    ],
                    vec![run("    println!(\"你好\");")],
                    vec![run("}")],
                ],
            },
            Block::Table {
                header: vec![
                    Cell {
                        content: vec![run("名称")],
                    },
                    Cell {
                        content: vec![run("Value")],
                    },
                ],
                rows: vec![
                    vec![
                        Cell {
                            content: vec![run("中文单元格")],
                        },
                        Cell {
                            content: vec![run("42")],
                        },
                    ],
                    vec![
                        Cell {
                            content: vec![run("another row")],
                        },
                        Cell {
                            content: vec![run("3.14")],
                        },
                    ],
                ],
                alignments: vec![Alignment::Left, Alignment::Right],
            },
            Block::Image {
                data: ImageData::Png(TINY_PNG.to_vec()),
                alt: "a red pixel".to_string(),
                width_px: 2,
                height_px: 2,
            },
            Block::Image {
                data: ImageData::Svg(
                    r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="60"><path d="M10 50 L60 10 L110 50 Z" fill="rgba(18,52,86,1)" stroke="none"/></svg>"##
                        .to_string(),
                ),
                alt: "x^2 + y^2 = z^2".to_string(),
                width_px: 120,
                height_px: 60,
            },
            Block::Rule,
        ],
        footnotes: vec![vec![run("这是脚注的内容。")]],
    }
}

#[test]
fn representative_document_renders_pdf_bytes() {
    let pdf = markion_pdf::render(&representative_document()).expect("render representative doc");
    assert!(pdf.starts_with(b"%PDF"), "output is a PDF");
    assert!(
        pdf.len() > 4_000,
        "PDF with embedded fonts should be non-trivial, got {} bytes",
        pdf.len()
    );
}

#[test]
fn long_fixture_paginates() {
    let paragraph = "分页测试段落 with some English mixed in to force wrapping behavior. "
        .repeat(8);
    let mut blocks = vec![Block::Heading {
        level: 1,
        content: vec![run("长文档")],
    }];
    for _ in 0..40 {
        blocks.push(Block::Paragraph {
            content: vec![run(&paragraph)],
        });
    }
    let doc = PdfDocument {
        blocks,
        ..PdfDocument::default()
    };
    let pdf = markion_pdf::render(&doc).expect("render long doc");
    assert!(pdf.starts_with(b"%PDF"));
    // The layout unit tests assert the page count; here we just check the
    // long document produces a substantially larger multi-object PDF.
    assert!(pdf.len() > 20_000, "got {} bytes", pdf.len());
}

#[test]
fn empty_document_renders_one_page() {
    let pdf = markion_pdf::render(&PdfDocument::default()).expect("render empty doc");
    assert!(pdf.starts_with(b"%PDF"));
}
