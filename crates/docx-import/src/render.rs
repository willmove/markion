use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::Cursor,
    path::Path,
};

use image::ImageReader;
use roxmltree::{Document, Node};
use sha2::{Digest, Sha256};

use crate::{
    AssetId, Diagnostic, DiagnosticCode, DiagnosticSeverity, ImportContext, ImportError,
    ImportSummary, MarkdownChunk, PreparedAsset, PreparedImport, package::DocxPackage,
};

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const M_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

#[derive(Default)]
struct ChunkWriter {
    chunks: Vec<MarkdownChunk>,
    recovered_content: bool,
}

impl ChunkWriter {
    fn text(&mut self, value: impl AsRef<str>) {
        let value = value.as_ref();
        if value.is_empty() {
            return;
        }
        if let Some(MarkdownChunk::Text(previous)) = self.chunks.last_mut() {
            previous.push_str(value);
        } else {
            self.chunks.push(MarkdownChunk::Text(value.to_owned()));
        }
    }

    fn asset(&mut self, id: AssetId) {
        self.recovered_content = true;
        self.chunks.push(MarkdownChunk::AssetUrl(id));
    }

    fn recovered_text(&mut self, value: impl AsRef<str>) {
        if value
            .as_ref()
            .chars()
            .any(|character| !character.is_whitespace())
        {
            self.recovered_content = true;
        }
        self.text(value);
    }

    fn byte_len_without_assets(&self) -> usize {
        self.chunks
            .iter()
            .map(|chunk| match chunk {
                MarkdownChunk::Text(text) => text.len(),
                MarkdownChunk::AssetUrl(_) => 0,
            })
            .sum()
    }

    fn trim_paragraph_break(&mut self) {
        if let Some(MarkdownChunk::Text(text)) = self.chunks.last_mut() {
            if let Some(trimmed) = text.strip_suffix("\n\n") {
                *text = trimmed.to_owned();
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Relationship {
    target: String,
    kind: String,
    external: bool,
}

#[derive(Debug, Clone, Default)]
struct Style {
    based_on: Option<String>,
    heading: Option<usize>,
    num_id: Option<String>,
    level: usize,
    run: RunFormatting,
}

#[derive(Debug, Clone, Default)]
struct RunFormatting {
    bold: bool,
    italic: bool,
    strike: bool,
    underline: bool,
    vertical: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct NumberLevel {
    format: String,
    text: String,
    start: u64,
}

#[derive(Default)]
struct Numbering {
    nums: HashMap<String, String>,
    levels: HashMap<(String, usize), NumberLevel>,
    start_overrides: HashMap<(String, usize), u64>,
    counters: HashMap<(String, usize), u64>,
}

struct Converter<'a> {
    package: DocxPackage,
    context: &'a ImportContext,
    relationships: HashMap<String, Relationship>,
    styles: HashMap<String, Style>,
    numbering: Numbering,
    bookmarks: HashMap<String, String>,
    assets: Vec<PreparedAsset>,
    asset_digests: HashMap<[u8; 32], AssetId>,
    diagnostics: Vec<Diagnostic>,
    summary: ImportSummary,
    footnote_references: HashSet<String>,
    footnote_definitions: HashSet<String>,
}

pub(crate) fn convert(
    package: DocxPackage,
    context: &ImportContext,
) -> Result<PreparedImport, ImportError> {
    let main_name = package.main_document.clone();
    let relationships = parse_relationships(&package, &main_name, context)?;
    let styles = parse_styles(&package, context)?;
    let numbering = parse_numbering(&package, context)?;
    let main_xml = package.required_text(&main_name)?.to_owned();
    crate::package::validate_xml(&main_name, &main_xml, context)?;
    let document = Document::parse(&main_xml).map_err(|error| ImportError::InvalidXml {
        part: main_name.clone(),
        message: error.to_string(),
    })?;
    let root = document.root_element();
    if root.tag_name().namespace() != Some(W_NS) || root.tag_name().name() != "document" {
        return Err(ImportError::InvalidXml {
            part: main_name.clone(),
            message: "unsupported WordprocessingML namespace or root".into(),
        });
    }

    let bookmarks = root
        .descendants()
        .filter(|node| is_w(*node, "bookmarkStart"))
        .filter_map(|node| {
            let name = attr_local(node, "name")?;
            Some((name.to_owned(), slug(name)))
        })
        .collect();
    let mut converter = Converter {
        package,
        context,
        relationships,
        styles,
        numbering,
        bookmarks,
        assets: Vec::new(),
        asset_digests: HashMap::new(),
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::LayoutNormalized,
            severity: DiagnosticSeverity::Info,
            part: Some(main_name.clone()),
            location: None,
            context: None,
        }],
        summary: ImportSummary {
            paragraphs: 0,
            tables: 0,
            images: 0,
            footnotes: 0,
            revisions_accepted: false,
        },
        footnote_references: HashSet::new(),
        footnote_definitions: HashSet::new(),
    };
    converter.inventory(root, &main_name);
    let body = root
        .children()
        .find(|node| is_w(*node, "body"))
        .ok_or_else(|| ImportError::InvalidXml {
            part: main_name.clone(),
            message: "missing document body".into(),
        })?;
    let mut writer = ChunkWriter::default();
    converter.render_blocks(body, &main_name, &mut writer)?;
    converter.render_footnotes(&mut writer)?;
    let missing_footnotes: Vec<_> = converter
        .footnote_references
        .difference(&converter.footnote_definitions)
        .cloned()
        .collect();
    for id in missing_footnotes {
        converter.diagnostic(
            DiagnosticCode::UnsupportedContent,
            DiagnosticSeverity::ContentLoss,
            &main_name,
            Some(format!("footnote {id}")),
            Some("missing footnote definition".into()),
        );
    }

    if writer.byte_len_without_assets() > context.limits.max_markdown_bytes {
        return Err(ImportError::LimitExceeded("generated Markdown bytes"));
    }
    if !writer.recovered_content {
        return Err(ImportError::EmptyDocument);
    }
    let package_stats = converter.package.stats.clone();
    Ok(PreparedImport {
        chunks: writer.chunks,
        assets: converter.assets,
        diagnostics: converter.diagnostics,
        summary: converter.summary,
        package: package_stats,
    })
}

impl Converter<'_> {
    fn checkpoint(&self) -> Result<(), ImportError> {
        self.context.checkpoint()
    }

    fn diagnostic(
        &mut self,
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        part: &str,
        location: Option<String>,
        context: Option<String>,
    ) {
        self.diagnostics.push(Diagnostic {
            code,
            severity,
            part: Some(part.to_owned()),
            location,
            context: context.map(|value| truncate_context(&value)),
        });
    }

    fn inventory(&mut self, root: Node<'_, '_>, part: &str) {
        let mut unsupported = BTreeMap::<&str, usize>::new();
        let mut move_ranges = BTreeMap::<String, [usize; 4]>::new();
        for node in root.descendants().filter(Node::is_element) {
            match node.tag_name().name() {
                "ins" | "del" | "moveFrom" | "moveTo" | "pPrChange" | "rPrChange"
                | "tblPrChange" | "tcPrChange" => self.summary.revisions_accepted = true,
                "moveFromRangeStart" | "moveFromRangeEnd" | "moveToRangeStart"
                | "moveToRangeEnd" => {
                    self.summary.revisions_accepted = true;
                    if let Some(id) = attr_local(node, "id") {
                        let counts = move_ranges.entry(id.to_owned()).or_default();
                        let index = match node.tag_name().name() {
                            "moveFromRangeStart" => 0,
                            "moveFromRangeEnd" => 1,
                            "moveToRangeStart" => 2,
                            _ => 3,
                        };
                        counts[index] += 1;
                    }
                }
                "altChunk" | "txbxContent" | "object" | "oleObject" | "commentReference"
                | "endnoteReference" => {
                    *unsupported.entry(node.tag_name().name()).or_default() += 1
                }
                "instrText" => {
                    self.diagnostic(
                        DiagnosticCode::FieldResultPreserved,
                        DiagnosticSeverity::Info,
                        part,
                        None,
                        node.text().map(ToOwned::to_owned),
                    );
                }
                _ => {}
            }
        }
        if self.summary.revisions_accepted {
            self.diagnostic(
                DiagnosticCode::RevisionsAccepted,
                DiagnosticSeverity::Info,
                part,
                None,
                None,
            );
        }
        for (id, counts) in move_ranges {
            if counts[0] != counts[1] || counts[2] != counts[3] {
                self.diagnostic(
                    DiagnosticCode::AmbiguousRevision,
                    DiagnosticSeverity::ContentLoss,
                    part,
                    None,
                    Some(format!("move range {id}")),
                );
            }
        }
        for (kind, count) in unsupported {
            self.diagnostic(
                DiagnosticCode::UnsupportedContent,
                DiagnosticSeverity::ContentLoss,
                part,
                Some(format!("{count} {kind} element(s)")),
                root.descendants()
                    .find(|node| node.tag_name().name() == kind)
                    .and_then(|node| node.text())
                    .map(ToOwned::to_owned),
            );
        }
        let related_parts: Vec<_> = self
            .relationships
            .values()
            .filter(|relationship| {
                relationship.kind.ends_with("/header")
                    || relationship.kind.ends_with("/footer")
                    || relationship.kind.ends_with("/endnotes")
                    || relationship.kind.ends_with("/comments")
            })
            .map(|relationship| relationship.kind.clone())
            .collect();
        for kind in related_parts {
            self.diagnostic(
                DiagnosticCode::UnsupportedContent,
                DiagnosticSeverity::ContentLoss,
                part,
                Some(kind.rsplit('/').next().unwrap_or("related content").into()),
                None,
            );
        }
    }

    fn render_blocks(
        &mut self,
        parent: Node<'_, '_>,
        part: &str,
        writer: &mut ChunkWriter,
    ) -> Result<(), ImportError> {
        for node in parent.children().filter(Node::is_element) {
            self.checkpoint()?;
            if excluded_revision(node) {
                continue;
            }
            match node.tag_name().name() {
                "p" if node.tag_name().namespace() == Some(W_NS) => {
                    self.render_paragraph(node, part, writer, None)?;
                    if paragraph_mark_deleted(node) {
                        writer.trim_paragraph_break();
                    }
                }
                "tbl" if node.tag_name().namespace() == Some(W_NS) => {
                    self.render_table(node, part, writer)?;
                }
                "sdt" | "customXml" | "ins" | "moveTo" => {
                    if let Some(content) = node.children().find(|child| is_w(*child, "sdtContent"))
                    {
                        self.render_blocks(content, part, writer)?;
                    } else {
                        self.render_blocks(node, part, writer)?;
                    }
                }
                "sectPr" => {}
                "altChunk" => writer.text("[Unsupported embedded document content]\n\n"),
                _ => {
                    let text = visible_text(node);
                    if !text.trim().is_empty() {
                        writer.recovered_text(escape_markdown_text(text.trim()));
                        writer.text("\n\n");
                    }
                }
            }
        }
        Ok(())
    }

    fn render_paragraph(
        &mut self,
        paragraph: Node<'_, '_>,
        part: &str,
        writer: &mut ChunkWriter,
        prefix_override: Option<&str>,
    ) -> Result<(), ImportError> {
        self.summary.paragraphs += 1;
        let location = Some(format!("paragraph {}", self.summary.paragraphs));
        if let Some(math_para) = paragraph
            .descendants()
            .find(|node| is_m(*node, "oMathPara"))
        {
            let tex = omml_children(math_para);
            if tex.is_empty() || !omml_supported(math_para) {
                let fallback = visible_text(paragraph);
                self.diagnostic(
                    DiagnosticCode::UnsupportedEquation,
                    DiagnosticSeverity::ContentLoss,
                    part,
                    location,
                    Some(fallback.clone()),
                );
                if fallback.trim().is_empty() {
                    writer.text("[Unsupported equation]\n\n");
                } else {
                    writer.recovered_text(escape_markdown_text(fallback.trim()));
                    writer.text("\n\n");
                }
            } else {
                writer.text("$$\n");
                writer.recovered_text(&tex);
                writer.text("\n$$\n\n");
            }
            return Ok(());
        }

        let style_id = paragraph
            .children()
            .find(|node| is_w(*node, "pPr"))
            .and_then(|properties| properties.children().find(|node| is_w(*node, "pStyle")))
            .and_then(|node| attr_local(node, "val"));
        let (heading, style_num, style_level) =
            self.resolve_style(style_id, part, location.clone())?;
        let properties = paragraph.children().find(|node| is_w(*node, "pPr"));
        let direct_num = properties
            .and_then(|node| node.children().find(|child| is_w(*child, "numPr")))
            .and_then(|num| {
                let id = num
                    .children()
                    .find(|node| is_w(*node, "numId"))
                    .and_then(|node| attr_local(node, "val"))?;
                let level = num
                    .children()
                    .find(|node| is_w(*node, "ilvl"))
                    .and_then(|node| attr_local(node, "val"))
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(0);
                Some((id.to_owned(), level))
            });

        if let Some(prefix) = prefix_override {
            writer.text(prefix);
        } else if let Some(level) = heading {
            let emitted = level.min(6);
            if level > 6 {
                self.diagnostic(
                    DiagnosticCode::HeadingDepthReduced,
                    DiagnosticSeverity::FormattingLoss,
                    part,
                    location.clone(),
                    None,
                );
            }
            writer.text("#".repeat(emitted));
            writer.text(" ");
        } else if let Some((num_id, level)) =
            direct_num.or_else(|| style_num.map(|id| (id, style_level)))
        {
            let (indent, marker, normalized) = self.numbering.marker(&num_id, level);
            writer.text("    ".repeat(indent));
            writer.text(&marker);
            writer.text(" ");
            if normalized {
                self.diagnostic(
                    DiagnosticCode::NumberingNormalized,
                    DiagnosticSeverity::FormattingLoss,
                    part,
                    location.clone(),
                    None,
                );
            }
        }

        self.render_inline_children(paragraph, part, writer, location)?;
        writer.text("\n\n");
        Ok(())
    }

    fn resolve_style(
        &mut self,
        style_id: Option<&str>,
        part: &str,
        _location: Option<String>,
    ) -> Result<(Option<usize>, Option<String>, usize), ImportError> {
        let mut current = style_id.map(ToOwned::to_owned);
        let mut heading = None;
        let mut num_id = None;
        let mut level = 0;
        let mut seen = HashMap::<String, ()>::new();
        for _ in 0..=self.context.limits.max_style_depth {
            let Some(id) = current.take() else {
                return Ok((heading, num_id, level));
            };
            if seen.insert(id.clone(), ()).is_some() {
                return Err(ImportError::InvalidXml {
                    part: part.into(),
                    message: format!("style inheritance cycle at {id}"),
                });
            }
            let Some(style) = self.styles.get(&id) else {
                return Ok((heading, num_id, level));
            };
            heading = heading.or(style.heading);
            if num_id.is_none() {
                num_id = style.num_id.clone();
                level = style.level;
            }
            current = style.based_on.clone();
        }
        Err(ImportError::LimitExceeded("style inheritance depth"))
    }

    fn render_inline_children(
        &mut self,
        parent: Node<'_, '_>,
        part: &str,
        writer: &mut ChunkWriter,
        location: Option<String>,
    ) -> Result<(), ImportError> {
        for node in parent.children().filter(Node::is_element) {
            self.checkpoint()?;
            if excluded_revision(node) {
                continue;
            }
            if is_w(node, "pPr") || is_w(node, "bookmarkEnd") {
                continue;
            }
            if is_w(node, "bookmarkStart") {
                if let Some(name) = attr_local(node, "name")
                    && let Some(anchor) = self.bookmarks.get(name)
                {
                    writer.text(format!("<a id=\"{}\"></a>", escape_html_attr(anchor)));
                }
            } else if is_w(node, "r") {
                self.render_run(node, part, writer, location.clone())?;
            } else if is_w(node, "hyperlink") {
                self.render_hyperlink(node, part, writer, location.clone())?;
            } else if is_m(node, "oMath") {
                let tex = omml_children(node);
                if tex.is_empty() || !omml_supported(node) {
                    let fallback = visible_text(node);
                    self.diagnostic(
                        DiagnosticCode::UnsupportedEquation,
                        DiagnosticSeverity::ContentLoss,
                        part,
                        location.clone(),
                        Some(fallback.clone()),
                    );
                    if fallback.trim().is_empty() {
                        writer.text("[Unsupported equation]");
                    } else {
                        writer.recovered_text(escape_markdown_text(fallback.trim()));
                    }
                } else {
                    writer.text("$");
                    writer.recovered_text(tex);
                    writer.text("$");
                }
            } else if matches!(
                node.tag_name().name(),
                "ins" | "moveTo" | "smartTag" | "sdt"
            ) {
                self.render_inline_children(node, part, writer, location.clone())?;
            } else if is_w(node, "fldSimple") {
                self.diagnostic(
                    DiagnosticCode::FieldResultPreserved,
                    DiagnosticSeverity::Info,
                    part,
                    location.clone(),
                    attr_local(node, "instr").map(ToOwned::to_owned),
                );
                self.render_inline_children(node, part, writer, location.clone())?;
            } else {
                let text = visible_text(node);
                if !text.is_empty() {
                    writer.recovered_text(escape_markdown_text(&text));
                }
            }
        }
        Ok(())
    }

    fn render_run(
        &mut self,
        run: Node<'_, '_>,
        part: &str,
        writer: &mut ChunkWriter,
        location: Option<String>,
    ) -> Result<(), ImportError> {
        if let Some(drawing) = run.descendants().find(|node| is_w(*node, "drawing")) {
            self.render_image(drawing, part, writer, location.clone())?;
        }
        if let Some(reference) = run
            .descendants()
            .find(|node| is_w(*node, "footnoteReference"))
            .and_then(|node| attr_local(node, "id"))
        {
            self.footnote_references.insert(reference.to_owned());
            writer.text(format!("[^fn{reference}]"));
        }
        let properties = run.children().find(|node| is_w(*node, "rPr"));
        let mut text = String::new();
        for child in run.children().filter(Node::is_element) {
            if excluded_revision(child) {
                continue;
            }
            match child.tag_name().name() {
                "t" if child.tag_name().namespace() == Some(W_NS) => {
                    text.push_str(child.text().unwrap_or_default())
                }
                "tab" if child.tag_name().namespace() == Some(W_NS) => text.push_str("    "),
                "br" | "cr" if child.tag_name().namespace() == Some(W_NS) => text.push_str("  \n"),
                "noBreakHyphen" => text.push('\u{2011}'),
                "softHyphen" => text.push('\u{00ad}'),
                _ => {}
            }
        }
        if text.is_empty() {
            return Ok(());
        }
        let escaped = escape_markdown_text(&text);
        let style_id = properties
            .and_then(|node| node.children().find(|child| is_w(*child, "rStyle")))
            .and_then(|node| attr_local(node, "val"));
        let inherited = self.resolve_run_style(style_id, part)?;
        let bold = has_on(properties, "b") || inherited.bold;
        let italic = has_on(properties, "i") || inherited.italic;
        let strike =
            has_on(properties, "strike") || has_on(properties, "dstrike") || inherited.strike;
        let underline = properties
            .and_then(|node| node.children().find(|child| is_w(*child, "u")))
            .and_then(|node| attr_local(node, "val"))
            .is_some_and(|value| value != "none" && value != "0")
            || inherited.underline;
        let vert = properties
            .and_then(|node| node.children().find(|child| is_w(*child, "vertAlign")))
            .and_then(|node| attr_local(node, "val"))
            .map(ToOwned::to_owned)
            .or(inherited.vertical);
        let mut before = String::new();
        let mut after = String::new();
        for (active, open, close) in [
            (bold, "**", "**"),
            (italic, "*", "*"),
            (strike, "~~", "~~"),
            (underline, "<u>", "</u>"),
            (vert.as_deref() == Some("superscript"), "<sup>", "</sup>"),
            (vert.as_deref() == Some("subscript"), "<sub>", "</sub>"),
        ] {
            if active {
                before.push_str(open);
                after.insert_str(0, close);
            }
        }
        writer.text(before);
        writer.recovered_text(escaped);
        writer.text(after);
        Ok(())
    }

    fn resolve_run_style(
        &self,
        style_id: Option<&str>,
        part: &str,
    ) -> Result<RunFormatting, ImportError> {
        let mut current = style_id.map(ToOwned::to_owned);
        let mut resolved = RunFormatting::default();
        let mut seen = HashMap::<String, ()>::new();
        for _ in 0..=self.context.limits.max_style_depth {
            let Some(id) = current.take() else {
                return Ok(resolved);
            };
            if seen.insert(id.clone(), ()).is_some() {
                return Err(ImportError::InvalidXml {
                    part: part.into(),
                    message: format!("style inheritance cycle at {id}"),
                });
            }
            let Some(style) = self.styles.get(&id) else {
                return Ok(resolved);
            };
            resolved.bold |= style.run.bold;
            resolved.italic |= style.run.italic;
            resolved.strike |= style.run.strike;
            resolved.underline |= style.run.underline;
            if resolved.vertical.is_none() {
                resolved.vertical = style.run.vertical.clone();
            }
            current = style.based_on.clone();
        }
        Err(ImportError::LimitExceeded("style inheritance depth"))
    }

    fn render_hyperlink(
        &mut self,
        node: Node<'_, '_>,
        part: &str,
        writer: &mut ChunkWriter,
        location: Option<String>,
    ) -> Result<(), ImportError> {
        let mut label_writer = ChunkWriter::default();
        self.render_inline_children(node, part, &mut label_writer, location.clone())?;
        writer.recovered_content |= label_writer.recovered_content;
        let label = flatten_text(&label_writer.chunks);
        let anchor = attr_local(node, "anchor");
        let relation = attr_local(node, "id")
            .and_then(|id| self.relationships.get(id))
            .cloned();
        let destination = if let Some(anchor) = anchor {
            self.bookmarks.get(anchor).map(|slug| format!("#{slug}"))
        } else if let Some(relation) = relation.as_ref() {
            if relation.external && safe_link(&relation.target) {
                Some(relation.target.clone())
            } else {
                None
            }
        } else {
            None
        };
        if let Some(destination) = destination {
            writer.text("[");
            writer.text(label);
            writer.text("](");
            writer.text(escape_markdown_url(&destination));
            writer.text(")");
        } else {
            self.diagnostic(
                if anchor.is_some() {
                    DiagnosticCode::MissingBookmark
                } else {
                    DiagnosticCode::UnsafeHyperlink
                },
                DiagnosticSeverity::FormattingLoss,
                part,
                location,
                relation.map(|item| item.target),
            );
            writer.text(label);
        }
        Ok(())
    }

    fn render_image(
        &mut self,
        drawing: Node<'_, '_>,
        part: &str,
        writer: &mut ChunkWriter,
        location: Option<String>,
    ) -> Result<(), ImportError> {
        let blip = drawing
            .descendants()
            .find(|node| node.is_element() && node.tag_name().name() == "blip");
        let relation_id =
            blip.and_then(|node| attr_local(node, "embed").or_else(|| attr_local(node, "link")));
        let alt = drawing
            .descendants()
            .find(|node| node.is_element() && node.tag_name().name() == "docPr")
            .and_then(|node| node.attribute("descr").or_else(|| node.attribute("title")))
            .unwrap_or("image");
        let Some(relationship) = relation_id
            .and_then(|id| self.relationships.get(id))
            .cloned()
        else {
            self.diagnostic(
                DiagnosticCode::MissingImage,
                DiagnosticSeverity::ContentLoss,
                part,
                location,
                Some(alt.into()),
            );
            writer.text(format!("[Missing image: {}]", escape_markdown_text(alt)));
            return Ok(());
        };
        if relationship.external {
            self.diagnostic(
                DiagnosticCode::ExternalResourceBlocked,
                DiagnosticSeverity::ContentLoss,
                part,
                location,
                Some(relationship.target),
            );
            writer.text(format!(
                "[Blocked external image: {}]",
                escape_markdown_text(alt)
            ));
            return Ok(());
        }
        let target = self
            .package
            .resolve_from(&self.package.main_document, &relationship.target)?;
        let Some(bytes) = self.package.bytes(&target).map(ToOwned::to_owned) else {
            self.diagnostic(
                DiagnosticCode::MissingImage,
                DiagnosticSeverity::ContentLoss,
                part,
                location,
                Some(target),
            );
            writer.text(format!("[Missing image: {}]", escape_markdown_text(alt)));
            return Ok(());
        };
        let Some(extension) = image_extension(&target, &bytes) else {
            self.diagnostic(
                DiagnosticCode::UnsupportedImage,
                DiagnosticSeverity::ContentLoss,
                part,
                location,
                Some(target),
            );
            writer.text(format!(
                "[Unsupported image: {}]",
                escape_markdown_text(alt)
            ));
            return Ok(());
        };
        if bytes.len() > self.context.limits.max_image_bytes {
            return Err(ImportError::LimitExceeded("individual image bytes"));
        }
        let current_total: usize = self.assets.iter().map(|asset| asset.bytes.len()).sum();
        if current_total.saturating_add(bytes.len()) > self.context.limits.max_total_image_bytes {
            return Err(ImportError::LimitExceeded("aggregate image bytes"));
        }
        let reader = match ImageReader::new(Cursor::new(&bytes)).with_guessed_format() {
            Ok(reader) => reader,
            Err(_) => {
                self.diagnostic(
                    DiagnosticCode::UnsupportedImage,
                    DiagnosticSeverity::ContentLoss,
                    part,
                    location,
                    Some(target),
                );
                writer.text(format!(
                    "[Unsupported image: {}]",
                    escape_markdown_text(alt)
                ));
                return Ok(());
            }
        };
        let (width, height) = match reader.into_dimensions() {
            Ok(dimensions) => dimensions,
            Err(_) => {
                self.diagnostic(
                    DiagnosticCode::UnsupportedImage,
                    DiagnosticSeverity::ContentLoss,
                    part,
                    location,
                    Some(target),
                );
                writer.text(format!(
                    "[Unsupported image: {}]",
                    escape_markdown_text(alt)
                ));
                return Ok(());
            }
        };
        if u64::from(width).saturating_mul(u64::from(height)) > self.context.limits.max_image_pixels
        {
            return Err(ImportError::LimitExceeded("image dimensions"));
        }
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        let id = if let Some(id) = self.asset_digests.get(&digest) {
            *id
        } else {
            if self.assets.len() >= self.context.limits.max_images {
                return Err(ImportError::LimitExceeded("image count"));
            }
            let id = AssetId(self.assets.len());
            let stem = Path::new(&target)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("image")
                .to_owned();
            self.assets.push(PreparedAsset {
                id,
                suggested_stem: stem,
                extension: extension.into(),
                bytes,
                width,
                height,
            });
            self.asset_digests.insert(digest, id);
            self.summary.images += 1;
            id
        };
        writer.text("![");
        writer.text(escape_markdown_label(alt));
        writer.text("](");
        writer.asset(id);
        writer.text(")");
        if drawing
            .descendants()
            .any(|node| node.is_element() && node.tag_name().name() == "anchor")
        {
            self.diagnostic(
                DiagnosticCode::FloatingImageNormalized,
                DiagnosticSeverity::FormattingLoss,
                part,
                location,
                None,
            );
        }
        Ok(())
    }

    fn render_table(
        &mut self,
        table: Node<'_, '_>,
        part: &str,
        writer: &mut ChunkWriter,
    ) -> Result<(), ImportError> {
        self.summary.tables += 1;
        let rows: Vec<_> = table.children().filter(|node| is_w(*node, "tr")).collect();
        let layouts: Vec<_> = rows.iter().copied().map(table_cell_layout).collect();
        let has_nested_table = rows.iter().any(|row| {
            row.descendants()
                .any(|node| is_w(node, "tbl") && node != table)
        });
        let merged = has_nested_table
            || layouts
                .iter()
                .flatten()
                .any(|(_, _, span, merge)| *span > 1 || !matches!(merge, VerticalMerge::None));
        if merged {
            self.diagnostic(
                DiagnosticCode::MergedTableHtml,
                DiagnosticSeverity::Info,
                part,
                Some(format!("table {}", self.summary.tables)),
                None,
            );
            if has_nested_table {
                self.diagnostic(
                    DiagnosticCode::ComplexTableSimplified,
                    DiagnosticSeverity::FormattingLoss,
                    part,
                    Some(format!("table {}", self.summary.tables)),
                    Some("nested table".into()),
                );
            }
            writer.text("<table>\n");
            for (row_index, cells) in layouts.iter().enumerate() {
                writer.text("<tr>");
                for (cell, column, colspan, merge) in cells {
                    if *merge == VerticalMerge::Continue {
                        continue;
                    }
                    let rowspan = if *merge == VerticalMerge::Restart {
                        layouts
                            .iter()
                            .skip(row_index + 1)
                            .take_while(|next_row| {
                                next_row
                                    .iter()
                                    .any(|(_, next_column, next_span, next_merge)| {
                                        *next_merge == VerticalMerge::Continue
                                            && *next_column <= *column
                                            && *column < next_column.saturating_add(*next_span)
                                    })
                            })
                            .count()
                            + 1
                    } else {
                        1
                    };
                    writer.text("<td");
                    if *colspan > 1 {
                        writer.text(format!(" colspan=\"{colspan}\""));
                    }
                    if rowspan > 1 {
                        writer.text(format!(" rowspan=\"{rowspan}\""));
                    }
                    writer.text(">");
                    let content = visible_text(*cell);
                    writer.recovered_text(escape_html_text(&content));
                    writer.text("</td>");
                }
                writer.text("</tr>\n");
            }
            writer.text("</table>\n\n");
            return Ok(());
        }

        let row_cells: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                row.children()
                    .filter(|node| is_w(*node, "tc"))
                    .map(|cell| normalize_cell_text(&visible_text(cell)))
                    .collect()
            })
            .collect();
        let width = row_cells.iter().map(Vec::len).max().unwrap_or(0);
        if width == 0 {
            return Ok(());
        }
        if row_cells
            .iter()
            .flatten()
            .any(|cell| !cell.trim().is_empty())
        {
            writer.recovered_content = true;
        }
        let header = rows
            .first()
            .is_some_and(|row| row.descendants().any(|node| is_w(node, "tblHeader")));
        if header {
            write_gfm_row(writer, &row_cells[0], width);
        } else {
            write_gfm_row(writer, &[], width);
        }
        write_gfm_separator(writer, width);
        for (index, row) in row_cells.iter().enumerate() {
            if header && index == 0 {
                continue;
            }
            write_gfm_row(writer, row, width);
        }
        writer.text("\n");
        Ok(())
    }

    fn render_footnotes(&mut self, writer: &mut ChunkWriter) -> Result<(), ImportError> {
        let relationship = self
            .relationships
            .values()
            .find(|relationship| relationship.kind.ends_with("/footnotes"))
            .cloned();
        let Some(relationship) = relationship else {
            return Ok(());
        };
        let part = self
            .package
            .resolve_from(&self.package.main_document, &relationship.target)?;
        let Some(xml) = self
            .package
            .xml(&part, self.context)?
            .map(ToOwned::to_owned)
        else {
            return Ok(());
        };
        let document = Document::parse(&xml).map_err(|error| ImportError::InvalidXml {
            part: part.clone(),
            message: error.to_string(),
        })?;
        for footnote in document
            .descendants()
            .filter(|node| is_w(*node, "footnote"))
        {
            let Some(id) = attr_local(footnote, "id") else {
                continue;
            };
            if id.starts_with('-') {
                continue;
            }
            let text = visible_text(footnote);
            if text.trim().is_empty() {
                continue;
            }
            self.footnote_definitions.insert(id.to_owned());
            writer.text(format!("[^fn{id}]: "));
            let mut first = true;
            for paragraph in footnote.children().filter(|node| is_w(*node, "p")) {
                if !first {
                    writer.text("\n    ");
                }
                first = false;
                let mut inline = ChunkWriter::default();
                self.render_inline_children(
                    paragraph,
                    &part,
                    &mut inline,
                    Some(format!("footnote {id}")),
                )?;
                writer.recovered_content |= inline.recovered_content;
                append_chunks(writer, inline.chunks);
            }
            writer.text("\n\n");
            self.summary.footnotes += 1;
        }
        Ok(())
    }
}

impl Numbering {
    fn marker(&mut self, num_id: &str, level: usize) -> (usize, String, bool) {
        let abstract_id = self.nums.get(num_id).cloned().unwrap_or_default();
        let definition = self
            .levels
            .get(&(abstract_id.clone(), level))
            .cloned()
            .unwrap_or_else(|| NumberLevel {
                format: "bullet".into(),
                text: "•".into(),
                start: 1,
            });
        if definition.format == "bullet" {
            return (level, "-".into(), false);
        }
        let start = self
            .start_overrides
            .get(&(num_id.to_owned(), level))
            .copied()
            .unwrap_or(definition.start);
        let counter = self
            .counters
            .entry((num_id.to_owned(), level))
            .or_insert(start);
        let value = *counter;
        *counter = counter.saturating_add(1);
        self.counters
            .retain(|(id, nested), _| id != num_id || *nested <= level);
        let normalized = definition.format != "decimal" || definition.text != "%1.";
        let marker = if normalized {
            let mut label = definition.text.clone();
            for source_level in 0..=8 {
                let placeholder = format!("%{}", source_level + 1);
                if !label.contains(&placeholder) {
                    continue;
                }
                let level_definition = self
                    .levels
                    .get(&(abstract_id.clone(), source_level))
                    .unwrap_or(&definition);
                let source_value = if source_level == level {
                    value
                } else {
                    self.counters
                        .get(&(num_id.to_owned(), source_level))
                        .copied()
                        .unwrap_or(level_definition.start)
                        .saturating_sub(1)
                        .max(level_definition.start)
                };
                label = label.replace(
                    &placeholder,
                    &format_number(&level_definition.format, source_value),
                );
            }
            format!("1. {label}")
        } else {
            format!("{value}.")
        };
        (level, marker, normalized)
    }
}

fn format_number(format: &str, value: u64) -> String {
    match format {
        "upperRoman" => roman(value).unwrap_or_else(|| value.to_string()),
        "lowerRoman" => roman(value)
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_else(|| value.to_string()),
        "chineseCounting" | "chineseCountingThousand" | "chineseLegalSimplified" => {
            chinese_number(value).unwrap_or_else(|| value.to_string())
        }
        _ => value.to_string(),
    }
}

fn roman(mut value: u64) -> Option<String> {
    if value == 0 || value > 3_999 {
        return None;
    }
    let mut output = String::new();
    for (amount, symbol) in [
        (1_000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while value >= amount {
            output.push_str(symbol);
            value -= amount;
        }
    }
    Some(output)
}

fn chinese_number(value: u64) -> Option<String> {
    if value == 0 || value > 9_999 {
        return None;
    }
    let digits = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    let units = [(1_000, "千"), (100, "百"), (10, "十")];
    let mut remaining = value;
    let mut output = String::new();
    let mut pending_zero = false;
    for (unit, label) in units {
        let digit = remaining / unit;
        remaining %= unit;
        if digit > 0 {
            if pending_zero && !output.is_empty() {
                output.push_str(digits[0]);
            }
            if !(unit == 10 && digit == 1 && output.is_empty()) {
                output.push_str(digits[digit as usize]);
            }
            output.push_str(label);
            pending_zero = false;
        } else if !output.is_empty() && remaining > 0 {
            pending_zero = true;
        }
    }
    if remaining > 0 {
        if pending_zero {
            output.push_str(digits[0]);
        }
        output.push_str(digits[remaining as usize]);
    }
    Some(output)
}

fn parse_relationships(
    package: &DocxPackage,
    source: &str,
    context: &ImportContext,
) -> Result<HashMap<String, Relationship>, ImportError> {
    let path = package.relationship_part(source);
    let Some(xml) = package.xml(&path, context)? else {
        return Ok(HashMap::new());
    };
    let document = Document::parse(xml).map_err(|error| ImportError::InvalidXml {
        part: path.clone(),
        message: error.to_string(),
    })?;
    let mut relationships = HashMap::new();
    for node in document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "Relationship")
    {
        let Some(id) = node.attribute("Id") else {
            continue;
        };
        relationships.insert(
            id.to_owned(),
            Relationship {
                target: node.attribute("Target").unwrap_or_default().to_owned(),
                kind: node.attribute("Type").unwrap_or_default().to_owned(),
                external: node
                    .attribute("TargetMode")
                    .is_some_and(|value| value.eq_ignore_ascii_case("External")),
            },
        );
    }
    Ok(relationships)
}

fn parse_styles(
    package: &DocxPackage,
    context: &ImportContext,
) -> Result<HashMap<String, Style>, ImportError> {
    let main_dir = Path::new(&package.main_document)
        .parent()
        .unwrap_or(Path::new(""))
        .to_string_lossy();
    let path = format!("{main_dir}/styles.xml");
    let Some(xml) = package.xml(&path, context)? else {
        return Ok(HashMap::new());
    };
    let document = Document::parse(xml).map_err(|error| ImportError::InvalidXml {
        part: path,
        message: error.to_string(),
    })?;
    let mut styles = HashMap::new();
    for node in document.descendants().filter(|node| is_w(*node, "style")) {
        let Some(id) = attr_local(node, "styleId") else {
            continue;
        };
        let based_on = node
            .children()
            .find(|child| is_w(*child, "basedOn"))
            .and_then(|node| attr_local(node, "val"))
            .map(ToOwned::to_owned);
        let ppr = node.children().find(|child| is_w(*child, "pPr"));
        let heading = ppr
            .and_then(|node| node.children().find(|child| is_w(*child, "outlineLvl")))
            .and_then(|node| attr_local(node, "val"))
            .and_then(|value| value.parse::<usize>().ok())
            .map(|level| level + 1)
            .or_else(|| heading_from_style(id));
        let num = ppr.and_then(|node| node.children().find(|child| is_w(*child, "numPr")));
        let num_id = num
            .and_then(|node| node.children().find(|child| is_w(*child, "numId")))
            .and_then(|node| attr_local(node, "val"))
            .filter(|id| *id != "0")
            .map(ToOwned::to_owned);
        let level = num
            .and_then(|node| node.children().find(|child| is_w(*child, "ilvl")))
            .and_then(|node| attr_local(node, "val"))
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let run_properties = node.children().find(|child| is_w(*child, "rPr"));
        let underline = run_properties
            .and_then(|properties| properties.children().find(|child| is_w(*child, "u")))
            .and_then(|node| attr_local(node, "val"))
            .is_some_and(|value| value != "none" && value != "0");
        let vertical = run_properties
            .and_then(|properties| {
                properties
                    .children()
                    .find(|child| is_w(*child, "vertAlign"))
            })
            .and_then(|node| attr_local(node, "val"))
            .map(ToOwned::to_owned);
        styles.insert(
            id.to_owned(),
            Style {
                based_on,
                heading,
                num_id,
                level,
                run: RunFormatting {
                    bold: has_on(run_properties, "b"),
                    italic: has_on(run_properties, "i"),
                    strike: has_on(run_properties, "strike") || has_on(run_properties, "dstrike"),
                    underline,
                    vertical,
                },
            },
        );
    }
    Ok(styles)
}

fn parse_numbering(
    package: &DocxPackage,
    context: &ImportContext,
) -> Result<Numbering, ImportError> {
    let main_dir = Path::new(&package.main_document)
        .parent()
        .unwrap_or(Path::new(""))
        .to_string_lossy();
    let path = format!("{main_dir}/numbering.xml");
    let Some(xml) = package.xml(&path, context)? else {
        return Ok(Numbering::default());
    };
    let document = Document::parse(xml).map_err(|error| ImportError::InvalidXml {
        part: path,
        message: error.to_string(),
    })?;
    let mut numbering = Numbering::default();
    for abstract_num in document
        .descendants()
        .filter(|node| is_w(*node, "abstractNum"))
    {
        let Some(id) = attr_local(abstract_num, "abstractNumId") else {
            continue;
        };
        for level in abstract_num.children().filter(|node| is_w(*node, "lvl")) {
            let index = attr_local(level, "ilvl")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            numbering.levels.insert(
                (id.to_owned(), index),
                NumberLevel {
                    format: child_val(level, "numFmt").unwrap_or("bullet").into(),
                    text: child_val(level, "lvlText").unwrap_or("•").into(),
                    start: child_val(level, "start")
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(1),
                },
            );
        }
    }
    for num in document.descendants().filter(|node| is_w(*node, "num")) {
        if let (Some(id), Some(abstract_id)) =
            (attr_local(num, "numId"), child_val(num, "abstractNumId"))
        {
            numbering.nums.insert(id.into(), abstract_id.into());
            for override_node in num.children().filter(|node| is_w(*node, "lvlOverride")) {
                let level = attr_local(override_node, "ilvl")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(0);
                if let Some(start) = child_val(override_node, "startOverride")
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    numbering
                        .start_overrides
                        .insert((id.to_owned(), level), start);
                }
            }
        }
    }
    Ok(numbering)
}

fn append_chunks(writer: &mut ChunkWriter, chunks: Vec<MarkdownChunk>) {
    for chunk in chunks {
        match chunk {
            MarkdownChunk::Text(text) => writer.text(text),
            MarkdownChunk::AssetUrl(id) => writer.asset(id),
        }
    }
}

fn flatten_text(chunks: &[MarkdownChunk]) -> String {
    chunks
        .iter()
        .filter_map(|chunk| match chunk {
            MarkdownChunk::Text(text) => Some(text.as_str()),
            MarkdownChunk::AssetUrl(_) => None,
        })
        .collect()
}

fn is_w(node: Node<'_, '_>, name: &str) -> bool {
    node.is_element() && node.tag_name().namespace() == Some(W_NS) && node.tag_name().name() == name
}

fn is_m(node: Node<'_, '_>, name: &str) -> bool {
    node.is_element() && node.tag_name().namespace() == Some(M_NS) && node.tag_name().name() == name
}

fn attr_local<'a>(node: Node<'a, 'a>, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attribute| attribute.name() == name)
        .map(|attribute| attribute.value())
}

fn child_val<'a>(node: Node<'a, 'a>, child: &str) -> Option<&'a str> {
    node.children()
        .find(|node| is_w(*node, child))
        .and_then(|node| attr_local(node, "val"))
}

fn has_on(properties: Option<Node<'_, '_>>, name: &str) -> bool {
    properties
        .and_then(|node| node.children().find(|child| is_w(*child, name)))
        .is_some_and(|node| !matches!(attr_local(node, "val"), Some("0" | "false" | "off")))
}

fn excluded_revision(node: Node<'_, '_>) -> bool {
    is_w(node, "del") || is_w(node, "moveFrom")
}

fn paragraph_mark_deleted(paragraph: Node<'_, '_>) -> bool {
    paragraph
        .children()
        .find(|node| is_w(*node, "pPr"))
        .is_some_and(|properties| {
            properties
                .descendants()
                .any(|node| is_w(node, "del") && node.ancestors().any(|parent| is_w(parent, "rPr")))
        })
}

fn omml_supported(math: Node<'_, '_>) -> bool {
    math.descendants()
        .filter(|node| node.is_element() && node.tag_name().namespace() == Some(M_NS))
        .all(|node| {
            matches!(
                node.tag_name().name(),
                "oMathPara"
                    | "oMath"
                    | "r"
                    | "rPr"
                    | "t"
                    | "f"
                    | "fPr"
                    | "num"
                    | "den"
                    | "rad"
                    | "radPr"
                    | "deg"
                    | "degHide"
                    | "e"
                    | "sSup"
                    | "sSub"
                    | "sSubSup"
                    | "sup"
                    | "sub"
                    | "d"
                    | "dPr"
                    | "begChr"
                    | "endChr"
                    | "nary"
                    | "naryPr"
                    | "chr"
                    | "limLoc"
                    | "ctrlPr"
            )
        })
}

fn visible_text(node: Node<'_, '_>) -> String {
    let mut output = String::new();
    collect_visible_text(node, &mut output);
    output
}

fn collect_visible_text(node: Node<'_, '_>, output: &mut String) {
    if excluded_revision(node) {
        return;
    }
    if node.is_text() {
        output.push_str(node.text().unwrap_or_default());
        return;
    }
    if is_w(node, "tab") {
        output.push_str("    ");
    } else if is_w(node, "br") || is_w(node, "cr") {
        output.push('\n');
    } else {
        for child in node.children() {
            collect_visible_text(child, output);
        }
    }
}

fn omml_children(node: Node<'_, '_>) -> String {
    let mut output = String::new();
    for child in node.children().filter(Node::is_element) {
        let value = match child.tag_name().name() {
            "r" => visible_text(child),
            "f" => format!(
                "\\frac{{{}}}{{{}}}",
                child
                    .children()
                    .find(|node| is_m(*node, "num"))
                    .map(omml_children)
                    .unwrap_or_default(),
                child
                    .children()
                    .find(|node| is_m(*node, "den"))
                    .map(omml_children)
                    .unwrap_or_default()
            ),
            "rad" => {
                let degree = child
                    .children()
                    .find(|node| is_m(*node, "deg"))
                    .map(omml_children)
                    .unwrap_or_default();
                let expression = child
                    .children()
                    .find(|node| is_m(*node, "e"))
                    .map(omml_children)
                    .unwrap_or_default();
                if degree.trim().is_empty() {
                    format!("\\sqrt{{{expression}}}")
                } else {
                    format!("\\sqrt[{degree}]{{{expression}}}")
                }
            }
            "sSup" => script(child, "sup", true),
            "sSub" => script(child, "sub", false),
            "sSubSup" => format!(
                "{}_{{{}}}^{{{}}}",
                named_math_child(child, "e"),
                named_math_child(child, "sub"),
                named_math_child(child, "sup")
            ),
            "d" => format!("\\left({}\\right)", named_math_child(child, "e")),
            "nary" => {
                let symbol = child
                    .descendants()
                    .find(|node| is_m(*node, "chr"))
                    .and_then(|node| attr_local(node, "val"))
                    .unwrap_or("∫");
                let command = match symbol {
                    "∑" => "\\sum",
                    "∏" => "\\prod",
                    "∮" => "\\oint",
                    _ => "\\int",
                };
                format!(
                    "{command}_{{{}}}^{{{}}} {}",
                    named_math_child(child, "sub"),
                    named_math_child(child, "sup"),
                    named_math_child(child, "e")
                )
            }
            "oMath" | "oMathPara" | "num" | "den" | "e" | "deg" | "sub" | "sup" => {
                omml_children(child)
            }
            _ if child.tag_name().namespace() == Some(M_NS) => omml_children(child),
            _ => String::new(),
        };
        output.push_str(&value);
    }
    output
}

fn script(node: Node<'_, '_>, script_name: &str, superscript: bool) -> String {
    let expression = named_math_child(node, "e");
    let script = named_math_child(node, script_name);
    if superscript {
        format!("{expression}^{{{script}}}")
    } else {
        format!("{expression}_{{{script}}}")
    }
}

fn named_math_child(node: Node<'_, '_>, name: &str) -> String {
    node.children()
        .find(|child| is_m(*child, name))
        .map(omml_children)
        .unwrap_or_default()
}

fn heading_from_style(id: &str) -> Option<usize> {
    let compact = id.replace(' ', "").to_ascii_lowercase();
    compact
        .strip_prefix("heading")
        .or_else(|| compact.strip_prefix("标题"))
        .and_then(|suffix| suffix.parse().ok())
}

fn image_extension(target: &str, bytes: &[u8]) -> Option<&'static str> {
    let extension = Path::new(target)
        .extension()
        .and_then(|value| value.to_str())?
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" if bytes.starts_with(b"\x89PNG\r\n\x1a\n") => Some("png"),
        "jpg" | "jpeg" if bytes.starts_with(&[0xff, 0xd8, 0xff]) => Some("jpg"),
        "gif" if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") => Some("gif"),
        "webp" if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") => Some("webp"),
        _ => None,
    }
}

fn safe_link(target: &str) -> bool {
    let lower = target.trim().to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:")
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    for character in value.chars() {
        if character.is_alphanumeric() || character == '_' || character == '-' {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_owned()
}

fn escape_markdown_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(
            character,
            '\\' | '\u{0060}' | '*' | '_' | '{' | '}' | '[' | ']' | '<' | '>' | '#'
        ) {
            output.push('\\');
        }
        output.push(character);
    }
    output
}

fn escape_markdown_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

fn escape_markdown_url(value: &str) -> String {
    value
        .replace(' ', "%20")
        .replace('(', "%28")
        .replace(')', "%29")
}

fn normalize_cell_text(value: &str) -> String {
    escape_markdown_text(value.trim())
        .replace('|', "\\|")
        .replace('\n', "<br>")
}

fn write_gfm_row(writer: &mut ChunkWriter, cells: &[String], width: usize) {
    writer.text("|");
    for index in 0..width {
        writer.text(" ");
        writer.text(cells.get(index).map(String::as_str).unwrap_or(""));
        writer.text(" |");
    }
    writer.text("\n");
}

fn write_gfm_separator(writer: &mut ChunkWriter, width: usize) {
    writer.text("|");
    for _ in 0..width {
        writer.text(" --- |");
    }
    writer.text("\n");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VerticalMerge {
    None,
    Restart,
    Continue,
}

fn table_cell_layout<'a, 'input>(
    row: Node<'a, 'input>,
) -> Vec<(Node<'a, 'input>, usize, usize, VerticalMerge)> {
    let mut column = 0usize;
    row.children()
        .filter(|node| is_w(*node, "tc"))
        .map(|cell| {
            let properties = cell.children().find(|node| is_w(*node, "tcPr"));
            let span = properties
                .and_then(|node| node.children().find(|child| is_w(*child, "gridSpan")))
                .and_then(|node| attr_local(node, "val"))
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1)
                .max(1);
            let merge = properties
                .and_then(|node| node.children().find(|child| is_w(*child, "vMerge")))
                .map(|node| match attr_local(node, "val") {
                    Some("restart") => VerticalMerge::Restart,
                    _ => VerticalMerge::Continue,
                })
                .unwrap_or(VerticalMerge::None);
            let start = column;
            column = column.saturating_add(span);
            (cell, start, span, merge)
        })
        .collect()
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_html_attr(value: &str) -> String {
    escape_html_text(value).replace('"', "&quot;")
}

fn truncate_context(value: &str) -> String {
    value.chars().take(160).collect()
}
