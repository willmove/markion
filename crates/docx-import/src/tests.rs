use std::{
    io::{Cursor, Write},
    time::Duration,
};

use zip::{CompressionMethod, ZipWriter, write::FileOptions};

use crate::{
    CancellationToken, DiagnosticCode, DiagnosticSeverity, ImportError, ImportLimits, import_docx,
};

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

fn package(parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut output);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
        let mut entries = Vec::new();
        if !parts.iter().any(|(name, _)| *name == "[Content_Types].xml") {
            entries.push(("[Content_Types].xml", CONTENT_TYPES.as_bytes()));
        }
        if !parts.iter().any(|(name, _)| *name == "_rels/.rels") {
            entries.push(("_rels/.rels", ROOT_RELS.as_bytes()));
        }
        entries.extend(parts.iter().copied());
        for (name, bytes) in entries {
            zip.start_file(name, options).unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
    }
    output.into_inner()
}

fn document(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
<w:body>{body}<w:sectPr/></w:body></w:document>"#
    )
}

fn convert(parts: &[(&str, &[u8])]) -> crate::PreparedImport {
    import_docx(
        &package(parts),
        ImportLimits::default(),
        CancellationToken::default(),
    )
    .unwrap()
}

#[test]
fn imports_heading_runs_breaks_and_accepted_revisions() {
    let styles = br#"<?xml version="1.0"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="paragraph" w:styleId="Heading2"><w:pPr><w:outlineLvl w:val="1"/></w:pPr></w:style>
</w:styles>"#;
    let xml = document(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr>
<w:r><w:rPr><w:b/></w:rPr><w:t>你好 *world*</w:t><w:br/><w:t>next</w:t></w:r>
<w:del><w:r><w:delText>old</w:delText></w:r></w:del>
<w:ins><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#,
    );
    let result = convert(&[
        ("word/document.xml", xml.as_bytes()),
        ("word/styles.xml", styles),
    ]);
    let markdown = result.render_markdown(|_| None).unwrap();
    assert!(markdown.contains("## **你好 \\*world\\*  \nnext**new"));
    assert!(!markdown.contains("old"));
    assert!(result.summary.revisions_accepted);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::RevisionsAccepted)
    );
}

#[test]
fn converts_headerless_table_without_consuming_first_row() {
    let xml = document(
        r#"<w:tbl>
<w:tr><w:tc><w:p><w:r><w:t>A|B</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>一</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>二</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl>"#,
    );
    let result = convert(&[("word/document.xml", xml.as_bytes())]);
    let markdown = result.render_markdown(|_| None).unwrap();
    assert!(markdown.starts_with("|  |  |\n| --- | --- |\n"));
    assert!(markdown.contains("| A\\|B | 一 |"));
    assert!(markdown.contains("| C | 二 |"));
    let events = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::ENABLE_TABLES)
        .collect::<Vec<_>>();
    assert!(events.iter().any(|event| matches!(
        event,
        pulldown_cmark::Event::Start(pulldown_cmark::Tag::Table(_))
    )));
}

#[test]
fn merged_table_emits_horizontal_and_vertical_spans() {
    let xml = document(
        r#"<w:tbl>
<w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/><w:vMerge w:val="restart"/></w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/><w:vMerge/></w:tcPr><w:p/></w:tc></w:tr>
</w:tbl>"#,
    );
    let result = convert(&[("word/document.xml", xml.as_bytes())]);
    let markdown = result.render_markdown(|_| None).unwrap();
    assert!(markdown.contains("<td colspan=\"2\" rowspan=\"2\">A</td>"));
    assert_eq!(markdown.matches("<table>").count(), 1);
    assert!(!markdown.contains("\n\n<tr>"));
    for line_endings in [markdown.clone(), markdown.replace('\n', "\r\n")] {
        let html = pulldown_cmark::Parser::new(&line_endings)
            .filter_map(|event| match event {
                pulldown_cmark::Event::Html(value) => Some(value.into_string()),
                _ => None,
            })
            .collect::<String>();
        assert!(html.contains("<table>"));
        assert!(html.contains("rowspan=\"2\""));
    }
}

#[test]
fn inherited_run_styles_numbering_links_and_bookmarks_are_semantic() {
    let styles = br#"<?xml version="1.0"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="character" w:styleId="StrongBase"><w:rPr><w:b/></w:rPr></w:style>
<w:style w:type="character" w:styleId="StrongItalic"><w:basedOn w:val="StrongBase"/><w:rPr><w:i/></w:rPr></w:style>
</w:styles>"#;
    let numbering = r#"<?xml version="1.0"?><w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:abstractNum w:abstractNumId="4"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/></w:lvl><w:lvl w:ilvl="1"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="•"/></w:lvl></w:abstractNum>
<w:num w:numId="9"><w:abstractNumId w:val="4"/><w:lvlOverride w:ilvl="0"><w:startOverride w:val="12"/></w:lvlOverride></w:num>
<w:abstractNum w:abstractNumId="5"><w:lvl w:ilvl="0"><w:start w:val="12"/><w:numFmt w:val="upperRoman"/><w:lvlText w:val="%1."/></w:lvl></w:abstractNum>
<w:num w:numId="10"><w:abstractNumId w:val="5"/></w:num>
<w:abstractNum w:abstractNumId="6"><w:lvl w:ilvl="0"><w:start w:val="11"/><w:numFmt w:val="chineseCounting"/><w:lvlText w:val="第%1章"/></w:lvl></w:abstractNum>
<w:num w:numId="11"><w:abstractNumId w:val="6"/></w:num>
</w:numbering>"#
        .as_bytes();
    let rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="safe" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com/a (b)" TargetMode="External"/>
<Relationship Id="bad" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="javascript:alert(1)" TargetMode="External"/>
</Relationships>"#;
    let xml = document(
        r#"<w:p><w:bookmarkStart w:id="1" w:name="章节 一"/><w:r><w:rPr><w:rStyle w:val="StrongItalic"/></w:rPr><w:t>Styled</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="9"/></w:numPr></w:pPr><w:r><w:t>twelve</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="1"/><w:numId w:val="9"/></w:numPr></w:pPr><w:r><w:t>nested</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="10"/></w:numPr></w:pPr><w:r><w:t>roman</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="11"/></w:numPr></w:pPr><w:r><w:t>chapter</w:t></w:r></w:p>
<w:p><w:hyperlink r:id="safe"><w:r><w:t>safe</w:t></w:r></w:hyperlink><w:r><w:t> </w:t></w:r><w:hyperlink r:id="bad"><w:r><w:t>kept</w:t></w:r></w:hyperlink><w:r><w:t> </w:t></w:r><w:hyperlink w:anchor="章节 一"><w:r><w:t>jump</w:t></w:r></w:hyperlink></w:p>"#,
    );
    let result = convert(&[
        ("word/document.xml", xml.as_bytes()),
        ("word/styles.xml", styles),
        ("word/numbering.xml", numbering),
        ("word/_rels/document.xml.rels", rels),
    ]);
    let markdown = result.render_markdown(|_| None).unwrap();
    assert!(markdown.contains("<a id=\"章节-一\"></a>***Styled***"));
    assert!(markdown.contains("12. twelve"));
    assert!(markdown.contains("    - nested"));
    assert!(markdown.contains("1. XII. roman"));
    assert!(markdown.contains("1. 第十一章 chapter"));
    assert!(markdown.contains("[safe](https://example.com/a%20%28b%29)"));
    assert!(markdown.contains("kept"));
    assert!(!markdown.contains("javascript:"));
    assert!(markdown.contains("[jump](#章节-一)"));
}

#[test]
fn equations_cover_supported_subset_and_diagnose_unknown_structures() {
    let supported = document(
        r#"<w:p><m:oMath><m:f><m:num><m:r><m:t>a</m:t></m:r></m:num><m:den><m:rad><m:e><m:r><m:t>b</m:t></m:r></m:e></m:rad></m:den></m:f></m:oMath></w:p>"#,
    );
    let result = convert(&[("word/document.xml", supported.as_bytes())]);
    let inline_tex = "\\frac{a}{\\sqrt{b}}";
    let markdown = result.render_markdown(|_| None).unwrap();
    assert!(markdown.contains(&format!("${inline_tex}$")));
    typune_markdown::MathRenderer::new()
        .validate_syntax(inline_tex)
        .unwrap();

    let display = document(
        r#"<w:p><m:oMathPara><m:oMath><m:sSubSup><m:e><m:r><m:t>x</m:t></m:r></m:e><m:sub><m:r><m:t>i</m:t></m:r></m:sub><m:sup><m:r><m:t>2</m:t></m:r></m:sup></m:sSubSup></m:oMath></m:oMathPara></w:p>"#,
    );
    let result = convert(&[("word/document.xml", display.as_bytes())]);
    let display_tex = "x_{i}^{2}";
    assert_eq!(
        result.render_markdown(|_| None).unwrap(),
        format!("$$\n{display_tex}\n$$\n\n")
    );
    typune_markdown::MathRenderer::new()
        .validate_syntax(display_tex)
        .unwrap();

    let unsupported = document(
        r#"<w:p><m:oMath><m:m><m:mr><m:e><m:r><m:t>x</m:t></m:r></m:e></m:mr></m:m></m:oMath></w:p>"#,
    );
    let result = convert(&[("word/document.xml", unsupported.as_bytes())]);
    assert!(result.has_content_loss());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::UnsupportedEquation)
    );
    assert!(result.render_markdown(|_| None).unwrap().contains('x'));
}

#[test]
fn placeholder_only_input_is_not_a_successful_empty_import() {
    let xml = document(r#"<w:altChunk r:id="external"/>"#);
    assert!(matches!(
        import_docx(
            &package(&[("word/document.xml", xml.as_bytes())]),
            ImportLimits::default(),
            CancellationToken::default(),
        ),
        Err(ImportError::EmptyDocument)
    ));
}

#[test]
fn accepted_paragraph_mark_deletion_joins_paragraphs_and_unbalanced_moves_warn() {
    let xml = document(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="1"/></w:rPr></w:pPr><w:r><w:t>Hello </w:t></w:r></w:p>
<w:p><w:moveToRangeStart w:id="7"/><w:r><w:t>world</w:t></w:r></w:p>"#,
    );
    let result = convert(&[("word/document.xml", xml.as_bytes())]);
    assert!(
        result
            .render_markdown(|_| None)
            .unwrap()
            .contains("Hello world")
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::AmbiguousRevision)
    );
}

#[test]
fn footnotes_keep_references_and_multi_paragraph_definitions() {
    let xml = document(
        r#"<w:p><w:r><w:t>Body</w:t></w:r><w:r><w:footnoteReference w:id="2"/></w:r><w:r><w:t> again</w:t></w:r><w:r><w:footnoteReference w:id="2"/></w:r></w:p>"#,
    );
    let rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rFoot" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>
</Relationships>"#;
    let notes = br#"<?xml version="1.0"?><w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:footnote w:id="-1"><w:p><w:r><w:t>separator</w:t></w:r></w:p></w:footnote>
<w:footnote w:id="2"><w:p><w:r><w:t>First</w:t></w:r></w:p><w:p><w:r><w:t>Second</w:t></w:r></w:p></w:footnote>
</w:footnotes>"#;
    let result = convert(&[
        ("word/document.xml", xml.as_bytes()),
        ("word/_rels/document.xml.rels", rels),
        ("word/footnotes.xml", notes),
    ]);
    let markdown = result.render_markdown(|_| None).unwrap();
    assert!(markdown.contains("Body[^fn2] again[^fn2]"));
    assert_eq!(markdown.matches("[^fn2]:").count(), 1);
    assert!(markdown.contains("[^fn2]: First\n    Second"));
    assert!(!markdown.contains("separator"));
}

#[test]
fn missing_footnote_definition_is_reported_as_content_loss() {
    let xml = document(r#"<w:p><w:r><w:t>Body</w:t><w:footnoteReference w:id="9"/></w:r></w:p>"#);
    let result = convert(&[("word/document.xml", xml.as_bytes())]);
    assert!(result.has_content_loss());
    assert!(result.diagnostics.iter().any(|item| {
        item.location.as_deref() == Some("footnote 9")
            && item.severity == DiagnosticSeverity::ContentLoss
    }));
}

#[test]
fn repeated_embedded_image_uses_one_typed_asset_and_keeps_alt_text() {
    let rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="img" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/pixel.png"/>
</Relationships>"#;
    let drawing = |alt: &str, floating: bool| {
        let container = if floating { "anchor" } else { "inline" };
        format!(
            r#"<w:r><w:drawing><wp:{container}><wp:docPr descr="{alt}"/><a:graphic><a:graphicData><a:blip r:embed="img"/></a:graphicData></a:graphic></wp:{container}></w:drawing></w:r>"#
        )
    };
    let xml = document(&format!(
        "<w:p>{}{}</w:p>",
        drawing("first", false),
        drawing("second", true)
    ));
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(1, 1)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let image_bytes = png.into_inner();
    let result = convert(&[
        ("word/document.xml", xml.as_bytes()),
        ("word/_rels/document.xml.rels", rels),
        ("word/media/pixel.png", &image_bytes),
    ]);
    assert_eq!(result.assets.len(), 1);
    assert_eq!(result.summary.images, 1);
    let markdown = result
        .render_markdown(|id| Some(format!("assets/{}.png", id.0)))
        .unwrap();
    assert!(markdown.contains("![first](assets/0.png)"));
    assert!(markdown.contains("![second](assets/0.png)"));
    assert!(
        result
            .diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::FloatingImageNormalized)
    );
}

#[test]
fn corrupt_and_floating_images_have_visible_diagnostics() {
    let rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="bad" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/bad.png"/>
</Relationships>"#;
    let xml = document(
        r#"<w:p><w:r><w:t>Body</w:t></w:r><w:r><w:drawing><wp:anchor><wp:docPr descr="broken art"/><a:graphic><a:graphicData><a:blip r:embed="bad"/></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>"#,
    );
    let mut broken = b"\x89PNG\r\n\x1a\n".to_vec();
    broken.extend_from_slice(b"not a complete image");
    let result = convert(&[
        ("word/document.xml", xml.as_bytes()),
        ("word/_rels/document.xml.rels", rels),
        ("word/media/bad.png", &broken),
    ]);
    assert!(
        result
            .render_markdown(|_| None)
            .unwrap()
            .contains("[Unsupported image: broken art]")
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::UnsupportedImage)
    );
}

#[test]
fn unsupported_content_is_never_silent() {
    let xml = document(r#"<w:p><w:r><w:t>Kept</w:t></w:r></w:p><w:altChunk r:id="external"/>"#);
    let result = convert(&[("word/document.xml", xml.as_bytes())]);
    assert!(result.has_content_loss());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|item| item.severity == DiagnosticSeverity::ContentLoss)
    );
    assert!(
        result
            .render_markdown(|_| None)
            .unwrap()
            .contains("Unsupported embedded document content")
    );
}

#[test]
fn limits_and_cancellation_stop_before_conversion() {
    let xml = document(r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#);
    let bytes = package(&[("word/document.xml", xml.as_bytes())]);
    let mut limits = ImportLimits::default();
    limits.max_source_bytes = bytes.len() - 1;
    assert!(matches!(
        import_docx(&bytes, limits, CancellationToken::default()),
        Err(ImportError::SourceTooLarge)
    ));

    let token = CancellationToken::default();
    token.cancel();
    assert!(matches!(
        import_docx(&bytes, ImportLimits::default(), token),
        Err(ImportError::Cancelled)
    ));

    let mut deadline = ImportLimits::default();
    deadline.deadline = Duration::ZERO;
    assert!(matches!(
        import_docx(&bytes, deadline, CancellationToken::default()),
        Err(ImportError::DeadlineExceeded)
    ));
}

#[test]
fn forbidden_entities_and_external_main_document_are_rejected() {
    let malicious = br#"<!DOCTYPE x [<!ENTITY e SYSTEM "file:///secret">]><x/>"#;
    let bytes = package(&[("word/document.xml", malicious)]);
    assert!(matches!(
        import_docx(
            &bytes,
            ImportLimits::default(),
            CancellationToken::default()
        ),
        Err(ImportError::InvalidXml { .. })
    ));

    let external_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="file:///secret" TargetMode="External"/>
</Relationships>"#;
    let bytes = package(&[
        ("_rels/.rels", external_rels),
        ("word/document.xml", document("<w:p/>").as_bytes()),
    ]);
    assert!(matches!(
        import_docx(
            &bytes,
            ImportLimits::default(),
            CancellationToken::default()
        ),
        Err(ImportError::InvalidRelationship(_))
    ));
}

#[test]
fn package_kinds_duplicates_malformed_xml_and_boundaries_are_enforced() {
    let xml = document("<w:p><w:r><w:t>ok</w:t></w:r></w:p>");
    let macro_types = CONTENT_TYPES.replace(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        "application/vnd.ms-word.document.macroEnabled.main+xml",
    );
    assert!(matches!(
        import_docx(
            &package(&[
                ("[Content_Types].xml", macro_types.as_bytes()),
                ("word/document.xml", xml.as_bytes()),
            ]),
            ImportLimits::default(),
            CancellationToken::default(),
        ),
        Err(ImportError::MacroEnabled)
    ));
    assert!(matches!(
        import_docx(
            &package(&[
                ("EncryptionInfo", b"encrypted"),
                ("EncryptedPackage", b"bytes")
            ]),
            ImportLimits::default(),
            CancellationToken::default(),
        ),
        Err(ImportError::Encrypted)
    ));
    assert!(matches!(
        import_docx(
            &package(&[
                ("word/document.xml", xml.as_bytes()),
                ("word/document.xml", xml.as_bytes()),
            ]),
            ImportLimits::default(),
            CancellationToken::default(),
        ),
        Err(ImportError::InvalidPackage(_))
    ));

    let bytes = package(&[("word/document.xml", xml.as_bytes())]);
    let uncompressed = CONTENT_TYPES.len() + ROOT_RELS.len() + xml.len();
    let largest_xml = CONTENT_TYPES.len().max(ROOT_RELS.len()).max(xml.len());
    let mut exact = ImportLimits::default();
    exact.max_entries = 3;
    exact.max_decompressed_bytes = uncompressed;
    exact.max_xml_part_bytes = largest_xml;
    assert!(import_docx(&bytes, exact.clone(), CancellationToken::default()).is_ok());
    exact.max_decompressed_bytes -= 1;
    assert!(matches!(
        import_docx(&bytes, exact, CancellationToken::default()),
        Err(ImportError::LimitExceeded("decompressed package bytes"))
    ));
    let mut output_limit = ImportLimits::default();
    output_limit.max_markdown_bytes = 1;
    assert!(matches!(
        import_docx(&bytes, output_limit, CancellationToken::default()),
        Err(ImportError::LimitExceeded("generated Markdown bytes"))
    ));

    let deep = document(&format!(
        "{}<w:r><w:t>x</w:t></w:r>{}",
        "<w:sdt>".repeat(130),
        "</w:sdt>".repeat(130)
    ));
    assert!(matches!(
        import_docx(
            &package(&[("word/document.xml", deep.as_bytes())]),
            ImportLimits::default(),
            CancellationToken::default(),
        ),
        Err(ImportError::LimitExceeded("XML nesting depth"))
    ));
}

#[test]
fn unsupported_content_inventory_and_external_images_are_explicit() {
    let rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="external" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="https://example.com/tracker.png" TargetMode="External"/>
<Relationship Id="header" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>
</Relationships>"#;
    let xml = document(
        r#"<w:p><w:r><w:t>Body</w:t></w:r><w:r><w:drawing><wp:inline><wp:docPr descr="remote"/><a:graphic><a:graphicData><a:blip r:link="external"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>
<w:p><w:fldSimple w:instr="DATE"><w:r><w:t>cached</w:t></w:r></w:fldSimple></w:p>
<w:object/><w:txbxContent><w:p><w:r><w:t>textbox</w:t></w:r></w:p></w:txbxContent>"#,
    );
    let result = convert(&[
        ("word/document.xml", xml.as_bytes()),
        ("word/_rels/document.xml.rels", rels),
    ]);
    let markdown = result.render_markdown(|_| None).unwrap();
    assert!(markdown.contains("[Blocked external image: remote]"));
    assert!(markdown.contains("cached"));
    for code in [
        DiagnosticCode::ExternalResourceBlocked,
        DiagnosticCode::FieldResultPreserved,
        DiagnosticCode::UnsupportedContent,
    ] {
        assert!(result.diagnostics.iter().any(|item| item.code == code));
    }
}
