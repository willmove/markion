//! Document exporters: PDF, DOCX (with a hand-written stored-only ZIP
//! container), and PNG/JPEG text snapshots rendered with the 8x8 bitmap font.
//!
//! PDF and DOCX first try the export engine absorbed from Typune (a pandoc
//! subprocess wrapper, `crates/export`); the hand-written implementations
//! below are the fallback when pandoc is unavailable or fails, so export
//! never requires external tools.

use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use typune_export::{DocxExporter, ExportOptions, Exporter, PdfExporter};
use typune_markdown::Parser;

use crate::MarkdownDocument;
use crate::escape::escape_xml_text;
use crate::math::render_math;
use crate::model::PreviewBlock;

/// Runs a Typune exporter over the raw Markdown source. Returns `None` on any
/// failure (pandoc missing, conversion error) so callers fall back to the
/// built-in writers.
fn engine_export(source: &str, exporter: &dyn Exporter) -> Option<Vec<u8>> {
    let document = match Parser::default().parse(source) {
        Ok(document) => document,
        Err(err) => {
            tracing::warn!(error = %err, "engine parser failed; using built-in exporter");
            return None;
        }
    };
    match exporter.export(&document, &ExportOptions::default()) {
        Ok(bytes) => Some(bytes),
        Err(err) => {
            tracing::info!(
                error = %err,
                format = %exporter.supported_format(),
                "export engine unavailable; using built-in exporter"
            );
            None
        }
    }
}

pub(crate) fn engine_pdf(source: &str, pdf_engine: &str) -> Option<Vec<u8>> {
    engine_export(source, &PdfExporter::new().with_pdf_engine(pdf_engine))
}

pub(crate) fn engine_docx(source: &str) -> Option<Vec<u8>> {
    engine_export(source, &DocxExporter::new())
}

fn plain_pdf_text(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '(' | ')' | '\\' => format!("\\{ch}"),
            '\n' | '\r' => " ".to_string(),
            ch if ch.is_ascii_graphic() || ch == ' ' => ch.to_string(),
            _ => "?".to_string(),
        })
        .collect::<Vec<_>>()
        .join("")
}

pub(crate) fn write_pdf(mut writer: impl Write, text: &str) -> io::Result<()> {
    let lines = wrap_text(text, 82);
    let mut stream = String::from("BT\n/F1 11 Tf\n50 792 Td\n14 TL\n");
    for line in lines.iter().take(52) {
        stream.push_str(&format!("({}) Tj\nT*\n", plain_pdf_text(line)));
    }
    stream.push_str("ET\n");

    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        format!("<< /Length {} >>\nstream\n{}endstream", stream.len(), stream),
    ];

    let mut buffer = Vec::new();
    buffer.extend_from_slice(b"%PDF-1.4\n");
    let mut offsets = vec![0usize];
    for (index, object) in objects.iter().enumerate() {
        offsets.push(buffer.len());
        buffer.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }
    let xref_start = buffer.len();
    buffer.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets.iter().skip(1) {
        buffer.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    buffer.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_start
        )
        .as_bytes(),
    );
    writer.write_all(&buffer)
}

pub(crate) fn write_docx(path: &Path, document: &MarkdownDocument) -> io::Result<()> {
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

    let entries = vec![
        ("[Content_Types].xml", docx_content_types().into_bytes()),
        ("_rels/.rels", docx_root_relationships().into_bytes()),
        (
            "docProps/core.xml",
            docx_core_properties(title, author, date).into_bytes(),
        ),
        (
            "word/document.xml",
            render_docx_document_xml(document).into_bytes(),
        ),
    ];
    let bytes = zip_stored_entries(entries)?;
    fs::write(path, bytes)
}

fn docx_content_types() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
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

fn render_docx_document_xml(document: &MarkdownDocument) -> String {
    let mut body = String::new();
    for block in document.preview_blocks() {
        body.push_str(&render_docx_block(&block));
    }
    if body.is_empty() {
        body.push_str(&docx_paragraph("", None));
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{body}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/></w:sectPr></w:body></w:document>"
    )
}

fn render_docx_block(block: &PreviewBlock) -> String {
    match block {
        PreviewBlock::Heading { level, text, .. } => {
            let style = match level {
                1 => "Heading1",
                2 => "Heading2",
                3 => "Heading3",
                _ => "Heading4",
            };
            docx_paragraph(&text.text, Some(style))
        }
        PreviewBlock::Paragraph { text, .. } => docx_paragraph(&text.text, None),
        PreviewBlock::ListItem {
            ordered,
            index,
            checked,
            text,
            ..
        } => {
            let marker = match checked {
                Some(true) => "[x] ".to_string(),
                Some(false) => "[ ] ".to_string(),
                None if *ordered => format!("{}. ", index.unwrap_or(1)),
                None => "- ".to_string(),
            };
            docx_paragraph(&format!("{marker}{text}"), None)
        }
        PreviewBlock::BlockQuote { text, .. } => docx_paragraph(&format!("> {text}"), None),
        PreviewBlock::CodeBlock { language, code, .. } => {
            let mut output = String::new();
            if let Some(language) = language {
                output.push_str(&docx_paragraph(&format!("Code: {language}"), None));
            }
            for line in code.lines() {
                output.push_str(&docx_code_paragraph(line));
            }
            if code.is_empty() {
                output.push_str(&docx_code_paragraph(""));
            }
            output
        }
        PreviewBlock::MathBlock { latex, error, .. } => {
            let rendered = render_math(latex, true);
            let prefix = if error.is_some() {
                "Math error: "
            } else {
                "Math: "
            };
            docx_paragraph(&format!("{prefix}{}", rendered.text), None)
        }
        PreviewBlock::Image { alt, url, .. } => {
            let label = if alt.is_empty() { "Image" } else { alt };
            docx_paragraph(&format!("{label}: {url}"), None)
        }
        PreviewBlock::Rule { .. } => docx_paragraph("----------", None),
        PreviewBlock::Table { rows, .. } => render_docx_table(rows),
    }
}

fn docx_paragraph(text: &str, style: Option<&str>) -> String {
    let style = style
        .map(|style| {
            format!(
                "<w:pPr><w:pStyle w:val=\"{}\"/></w:pPr>",
                escape_xml_text(style)
            )
        })
        .unwrap_or_default();
    format!(
        "<w:p>{style}<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        escape_xml_text(text)
    )
}

fn docx_code_paragraph(text: &str) -> String {
    format!(
        "<w:p><w:pPr><w:spacing w:before=\"0\" w:after=\"0\"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/><w:sz w:val=\"20\"/></w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        escape_xml_text(text)
    )
}

fn render_docx_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let columns = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
    let mut output = String::from(
        "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/><w:tblBorders><w:top w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:left w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:right w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:insideH w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/><w:insideV w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/></w:tblBorders></w:tblPr>",
    );
    for row in rows {
        output.push_str("<w:tr>");
        for column in 0..columns {
            let cell = row.get(column).map(String::as_str).unwrap_or("");
            output.push_str("<w:tc><w:tcPr><w:tcW w:w=\"2400\" w:type=\"dxa\"/></w:tcPr>");
            output.push_str(&docx_paragraph(cell, None));
            output.push_str("</w:tc>");
        }
        output.push_str("</w:tr>");
    }
    output.push_str("</w:tbl>");
    output
}

fn zip_stored_entries(entries: Vec<(&str, Vec<u8>)>) -> io::Result<Vec<u8>> {
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

        push_u32_le(&mut output, 0x0403_4b50);
        push_u16_le(&mut output, 20);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 33);
        push_u32_le(&mut output, crc);
        push_u32_le(&mut output, size);
        push_u32_le(&mut output, size);
        push_u16_le(&mut output, name_len);
        push_u16_le(&mut output, 0);
        output.extend_from_slice(name_bytes);
        output.extend_from_slice(&data);

        records.push(ZipEntryRecord {
            name: name.to_string(),
            crc,
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
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 0);
        push_u16_le(&mut output, 33);
        push_u32_le(&mut output, record.crc);
        push_u32_le(&mut output, record.size);
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

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    text.lines()
        .flat_map(|line| {
            let mut lines = Vec::new();
            let mut current = String::new();
            let mut current_width = 0usize;
            for word in line.split_whitespace() {
                // Measure in characters, not bytes, so multi-byte text (CJK,
                // accented Latin) wraps at the intended column rather than
                // breaking early on byte count.
                let word_width = word.chars().count();
                if current_width != 0 && current_width + word_width + 1 > width {
                    lines.push(std::mem::take(&mut current));
                    current_width = 0;
                }
                if current_width != 0 {
                    current.push(' ');
                    current_width += 1;
                }
                current.push_str(word);
                current_width += word_width;
            }
            if current.is_empty() {
                lines.push(String::new());
            } else {
                lines.push(current);
            }
            lines
        })
        .collect()
}

pub(crate) fn write_image_snapshot(
    path: &Path,
    text: &str,
    format: image::ImageFormat,
) -> io::Result<()> {
    let image = render_image_snapshot(text);
    image
        .save_with_format(path, format)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

fn render_image_snapshot(text: &str) -> image::RgbImage {
    let lines = wrap_text(
        if text.trim().is_empty() {
            "Empty document"
        } else {
            text
        },
        96,
    );
    let scale = 2u32;
    let padding = 32u32;
    let glyph_width = 8 * scale;
    let glyph_height = 8 * scale;
    let char_gap = scale;
    let line_gap = 6u32;
    let line_height = glyph_height + line_gap;
    let max_chars = lines
        .iter()
        .map(|line| line.chars().count() as u32)
        .max()
        .unwrap_or(0)
        .min(120);
    let width = (padding * 2 + max_chars * (glyph_width + char_gap)).clamp(640, 2200);
    let height = (padding * 2 + lines.len() as u32 * line_height).clamp(360, 8000);
    let mut image = image::ImageBuffer::from_pixel(width, height, image::Rgb([250, 250, 248]));

    draw_snapshot_rule(&mut image, padding, padding / 2, width - padding * 2);
    for (line_index, line) in lines.iter().enumerate() {
        let y = padding + line_index as u32 * line_height;
        if y + glyph_height >= height - padding / 2 {
            break;
        }
        draw_bitmap_text(
            &mut image,
            padding,
            y,
            line,
            scale,
            image::Rgb([32, 33, 36]),
        );
    }

    image
}

fn draw_snapshot_rule(image: &mut image::RgbImage, x: u32, y: u32, width: u32) {
    let color = image::Rgb([220, 224, 230]);
    for dx in 0..width {
        put_pixel_if_in_bounds(image, x + dx, y, color);
    }
}

fn draw_bitmap_text(
    image: &mut image::RgbImage,
    x: u32,
    y: u32,
    text: &str,
    scale: u32,
    color: image::Rgb<u8>,
) {
    use font8x8::UnicodeFonts;

    let glyph_width = 8 * scale;
    let char_gap = scale;
    let mut cursor_x = x;
    let right_edge = image.width().saturating_sub(x);

    for ch in text.chars() {
        if cursor_x + glyph_width >= right_edge {
            break;
        }

        if ch == ' ' {
            cursor_x += glyph_width / 2 + char_gap;
            continue;
        }

        if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
            draw_bitmap_glyph(image, cursor_x, y, &glyph, scale, color);
        } else {
            draw_missing_glyph(image, cursor_x, y, scale, color);
        }

        cursor_x += glyph_width + char_gap;
    }
}

fn draw_bitmap_glyph(
    image: &mut image::RgbImage,
    x: u32,
    y: u32,
    glyph: &[u8; 8],
    scale: u32,
    color: image::Rgb<u8>,
) {
    for (row, bits) in glyph.iter().copied().enumerate() {
        for column in 0..8u32 {
            if bits & (1 << column) == 0 {
                continue;
            }
            let pixel_x = x + column * scale;
            let pixel_y = y + row as u32 * scale;
            fill_pixel_block(image, pixel_x, pixel_y, scale, color);
        }
    }
}

fn draw_missing_glyph(
    image: &mut image::RgbImage,
    x: u32,
    y: u32,
    scale: u32,
    color: image::Rgb<u8>,
) {
    let size = 8 * scale;
    for offset in 0..size {
        fill_pixel_block(image, x + offset, y, scale, color);
        fill_pixel_block(
            image,
            x + offset,
            y + size.saturating_sub(scale),
            scale,
            color,
        );
        fill_pixel_block(image, x, y + offset, scale, color);
        fill_pixel_block(
            image,
            x + size.saturating_sub(scale),
            y + offset,
            scale,
            color,
        );
    }
}

fn fill_pixel_block(
    image: &mut image::RgbImage,
    x: u32,
    y: u32,
    scale: u32,
    color: image::Rgb<u8>,
) {
    for dx in 0..scale {
        for dy in 0..scale {
            put_pixel_if_in_bounds(image, x + dx, y + dy, color);
        }
    }
}

fn put_pixel_if_in_bounds(image: &mut image::RgbImage, x: u32, y: u32, color: image::Rgb<u8>) {
    if x < image.width() && y < image.height() {
        image.put_pixel(x, y, color);
    }
}
