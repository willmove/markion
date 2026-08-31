//! Markdown → structured preview parsing helpers.
//!
//! These functions turn `pulldown-cmark` event streams into [`PreviewBlock`]s,
//! resolve inline styling, and handle extended inline syntax
//! (`==highlight==`, `^sup^`, `~sub~`, `:emoji:`, bare autolinks). The main
//! `compute_preview_blocks` driver lives on `MarkdownDocument` and calls into
//! this module; everything here is pure / stateless.

use std::ops::Range;

use pulldown_cmark::{BlockQuoteKind, HeadingLevel, Options};

use crate::escape::escape_html_attribute;
use crate::model::{
    HtmlImgLength, InlineSpan, InlineStyle, MathSource, PreviewBlock, RichText, VisualHtmlImage,
};
use crate::table::TableDraft;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HtmlAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlListMarker {
    Disc,
    Decimal(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlPreviewPart {
    Text {
        text: RichText,
        centered: bool,
        heading_level: Option<u8>,
        list_marker: Option<HtmlListMarker>,
        pre: bool,
        align: HtmlAlign,
    },
    Image {
        alt: String,
        url: String,
        title: Option<String>,
        centered: bool,
        width: Option<HtmlImgLength>,
        height: Option<HtmlImgLength>,
        align: HtmlAlign,
    },
    /// A raw HTML `<table>` block resolved into a row/column grid, honoring
    /// `rowspan`/`colspan`. Produced instead of flattening the table to text.
    Table { grid: HtmlTableGrid },
}

/// A resolved HTML table ready for rendering. `rows` holds one entry per visual
/// row; each row is an ordered list of slots (cells + spacers) whose `colspan`
/// values sum to `columns`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HtmlTableGrid {
    /// Total number of logical columns in the table.
    pub columns: usize,
    /// True if any cell declares `rowspan > 1`. The renderer uses a fixed row
    /// height in that case so spanning cells align with the rows they cover.
    pub has_rowspan: bool,
    /// One `Vec` of slots per visual row, in top-to-bottom order.
    pub rows: Vec<Vec<HtmlTableCell>>,
}

/// One slot in a resolved HTML table row. A spacer slot (`is_spacer == true`)
/// marks columns covered by a `rowspan` started in an earlier row; it reserves
/// horizontal width (via `colspan`) but draws nothing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HtmlTableCell {
    /// Resolved inline content (bold/italic/code/links via the shared pipeline).
    pub content: RichText,
    /// Optional cell image extracted from `<img>` inside `<td>`/`<th>`.
    pub image: Option<HtmlTableCellImage>,
    /// Number of columns this slot occupies (>= 1).
    pub colspan: usize,
    /// Number of rows this cell spans (>= 1). 0 for spacer slots.
    pub rowspan: usize,
    /// True for `<th>` cells (rendered with header emphasis).
    pub is_header: bool,
    /// True for spacer slots covering a rowspan from above.
    pub is_spacer: bool,
}

/// An `<img>` found inside an HTML table cell.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HtmlTableCellImage {
    pub alt: String,
    pub url: String,
    pub title: Option<String>,
    pub width: Option<HtmlImgLength>,
    pub height: Option<HtmlImgLength>,
}

impl HtmlAlign {
    fn combine(self, other: Self) -> Self {
        if other == Self::Start { self } else { other }
    }
}

pub(crate) struct ListItemDraft {
    pub level: usize,
    pub ordered: bool,
    pub index: Option<u64>,
    pub checked: Option<bool>,
    pub spans: Vec<InlineSpan>,
    pub source_range: Range<usize>,
}

pub(crate) struct ImageDraft {
    pub alt: String,
    pub url: String,
    pub title: Option<String>,
    pub source_range: Range<usize>,
}

#[derive(Default)]
pub(crate) struct InlineStateDraft {
    pub bold: usize,
    pub italic: usize,
    pub strikethrough: usize,
    pub links: Vec<String>,
}

impl InlineStateDraft {
    pub fn style(&self) -> InlineStyle {
        InlineStyle {
            bold: self.bold > 0,
            italic: self.italic > 0,
            strikethrough: self.strikethrough > 0,
            ..InlineStyle::default()
        }
    }

    pub fn link(&self) -> Option<&str> {
        self.links.last().map(String::as_str)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ListLevelDraft {
    pub ordered: bool,
    pub next_index: u64,
}

pub(crate) fn flush_list_item(blocks: &mut Vec<PreviewBlock>, item: Option<ListItemDraft>) {
    if let Some(item) = item {
        let text = finish_rich_text(item.spans);
        // Empty unordered/ordered items still own their marker line (`- `,
        // `1. `) so Visual Edit can keep them as list rows instead of
        // Unsupported source islands. Task items were already kept via the
        // checkbox even when the payload was empty.
        blocks.push(PreviewBlock::ListItem {
            level: item.level,
            ordered: item.ordered,
            index: item.index,
            checked: item.checked,
            text,
            source_range: item.source_range,
        });
    }
}

/// Routes one run of inline text to whichever block draft is currently open,
/// mirroring the old plain-text routing priority. Styled targets receive
/// spans; image alts, code bodies, and table cells stay plain text.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_preview_rich(
    heading: &mut Option<(u8, Vec<InlineSpan>, Range<usize>)>,
    paragraph: &mut Option<(Vec<InlineSpan>, Range<usize>)>,
    quote: &mut Vec<InlineSpan>,
    quote_depth: usize,
    list_item: &mut Option<ListItemDraft>,
    image: &mut Option<ImageDraft>,
    code: &mut Option<(Option<String>, String, Range<usize>)>,
    table: &mut Option<TableDraft>,
    text: &str,
    style: InlineStyle,
    link: Option<&str>,
    parse_extended: bool,
) {
    if let Some(image) = image.as_mut() {
        image.alt.push_str(text);
        return;
    }
    if let Some((_, code, _)) = code.as_mut() {
        code.push_str(text);
        return;
    }
    if let Some(table) = table.as_mut() {
        if parse_extended {
            append_extended_text(&mut table.current_cell, text, style, link);
        } else {
            append_span(&mut table.current_cell, text, style, link);
        }
        return;
    }

    let spans = if let Some((_, spans, _)) = heading.as_mut() {
        spans
    } else if let Some(item) = list_item.as_mut() {
        &mut item.spans
    } else if let Some((paragraph, _)) = paragraph.as_mut() {
        paragraph
    } else if quote_depth > 0 {
        quote
    } else {
        return;
    };

    if parse_extended {
        append_extended_text(spans, text, style, link);
    } else {
        append_span(spans, text, style, link);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_preview_math(
    heading: &mut Option<(u8, Vec<InlineSpan>, Range<usize>)>,
    paragraph: &mut Option<(Vec<InlineSpan>, Range<usize>)>,
    quote: &mut Vec<InlineSpan>,
    quote_depth: usize,
    list_item: &mut Option<ListItemDraft>,
    image: &mut Option<ImageDraft>,
    code: &mut Option<(Option<String>, String, Range<usize>)>,
    table: &mut Option<TableDraft>,
    math: MathSource,
    style: InlineStyle,
    link: Option<&str>,
) {
    if let Some(image) = image.as_mut() {
        image.alt.push_str(&math.authored);
        return;
    }
    if let Some((_, code, _)) = code.as_mut() {
        code.push_str(&math.authored);
        return;
    }
    if let Some(table) = table.as_mut() {
        table.current_cell.push(InlineSpan {
            text: math.authored.clone(),
            style,
            link: link.map(str::to_string),
            math: Some(math),
        });
        return;
    }

    let spans = if let Some((_, spans, _)) = heading.as_mut() {
        spans
    } else if let Some(item) = list_item.as_mut() {
        &mut item.spans
    } else if let Some((paragraph, _)) = paragraph.as_mut() {
        paragraph
    } else if quote_depth > 0 {
        quote
    } else {
        return;
    };

    spans.push(InlineSpan {
        text: math.authored.clone(),
        style,
        link: link.map(str::to_string),
        math: Some(math),
    });
}

/// Appends text to the span list, merging with the previous span when the
/// style and link target match.
pub(crate) fn append_span(
    spans: &mut Vec<InlineSpan>,
    text: &str,
    style: InlineStyle,
    link: Option<&str>,
) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = spans.last_mut()
        && last.style == style
        && last.link.as_deref() == link
        && last.math.is_none()
    {
        last.text.push_str(text);
        return;
    }
    spans.push(InlineSpan {
        text: text.to_string(),
        style,
        link: link.map(str::to_string),
        math: None,
    });
}

/// Parses extended inline syntax (`==highlight==`, `^sup^`, `~sub~`, emoji
/// shortcodes, bare autolinks) inside one text run and appends styled spans.
pub(crate) fn append_extended_text(
    spans: &mut Vec<InlineSpan>,
    text: &str,
    style: InlineStyle,
    link: Option<&str>,
) {
    for segment in parse_extended_inline_segments(text) {
        append_extended_segment(spans, &segment, style, link);
    }
}

fn append_extended_segment(
    spans: &mut Vec<InlineSpan>,
    segment: &ExtendedInlineSegment,
    style: InlineStyle,
    link: Option<&str>,
) {
    match segment {
        ExtendedInlineSegment::Text(text) => append_span(spans, text, style, link),
        ExtendedInlineSegment::Emoji(emoji) => append_span(spans, emoji, style, link),
        ExtendedInlineSegment::AutoLink(url) => {
            let href = if url.starts_with("www.") {
                format!("https://{url}")
            } else {
                url.clone()
            };
            append_span(spans, url, style, Some(&href));
        }
        ExtendedInlineSegment::Highlight(children) => {
            let mut style = style;
            style.highlight = true;
            for child in children {
                append_extended_segment(spans, child, style, link);
            }
        }
        ExtendedInlineSegment::Superscript(children) => {
            let mut style = style;
            style.superscript = true;
            for child in children {
                append_extended_segment(spans, child, style, link);
            }
        }
        ExtendedInlineSegment::Subscript(children) => {
            let mut style = style;
            style.subscript = true;
            for child in children {
                append_extended_segment(spans, child, style, link);
            }
        }
    }
}

/// Normalizes accumulated spans into a [`RichText`]: trims every line, drops
/// blank lines, joins the survivors with `\n`, and merges equal-style
/// neighbors. This mirrors what `clean_preview_text` does for plain strings.
pub(crate) fn finish_rich_text(spans: Vec<InlineSpan>) -> RichText {
    let mut lines: Vec<Vec<InlineSpan>> = vec![Vec::new()];
    for span in spans {
        let mut first = true;
        for part in span.text.split('\n') {
            if !first {
                lines.push(Vec::new());
            }
            first = false;
            if !part.is_empty() {
                lines
                    .last_mut()
                    .expect("lines is non-empty")
                    .push(InlineSpan {
                        text: part.to_string(),
                        style: span.style,
                        link: span.link.clone(),
                        math: (!span.text.contains('\n') && part == span.text)
                            .then(|| span.math.clone())
                            .flatten(),
                    });
            }
        }
    }

    let mut merged: Vec<InlineSpan> = Vec::new();
    let mut emitted_line = false;
    for mut line in lines {
        while let Some(first) = line.first_mut() {
            let trimmed = first.text.trim_start();
            if trimmed.is_empty() {
                line.remove(0);
            } else {
                if trimmed.len() != first.text.len() {
                    first.text = trimmed.to_string();
                }
                break;
            }
        }
        while let Some(last) = line.last_mut() {
            let trimmed = last.text.trim_end();
            if trimmed.is_empty() {
                line.pop();
            } else {
                if trimmed.len() != last.text.len() {
                    last.text = trimmed.to_string();
                }
                break;
            }
        }
        if line.is_empty() {
            continue;
        }
        if emitted_line {
            append_span(&mut merged, "\n", InlineStyle::default(), None);
        }
        emitted_line = true;
        for span in line {
            if span.math.is_some() {
                merged.push(span);
            } else {
                append_span(&mut merged, &span.text, span.style, span.link.as_deref());
            }
        }
    }

    let text = merged.iter().map(|span| span.text.as_str()).collect();
    RichText {
        text,
        spans: merged,
    }
}

pub(crate) fn push_nonempty_block(blocks: &mut Vec<PreviewBlock>, block: PreviewBlock) {
    match &block {
        PreviewBlock::Paragraph { text, .. } => {
            if !text.is_empty() {
                blocks.push(block);
            }
        }
        PreviewBlock::Heading { .. } => {
            // Empty ATX headings (`##`, `###     `) are valid CommonMark and
            // must stay in the block stream so Visual Edit / Read can reserve
            // heading-row height instead of treating the marker as a gap.
            blocks.push(block);
        }
        PreviewBlock::BlockQuote {
            children, alert, ..
        } => {
            // An alert quote with no body still owns its marker line, so it
            // must survive into the block model for visual rendering.
            if !children.is_empty() || alert.is_some() {
                blocks.push(block);
            }
        }
        PreviewBlock::Image { url, .. } => {
            if !url.is_empty() {
                blocks.push(block);
            }
        }
        _ => blocks.push(block),
    }
}

pub(crate) fn clean_preview_text(text: &str) -> String {
    let text = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    render_extended_inline_plain(&text)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExtendedInlineSegment {
    Text(String),
    Highlight(Vec<ExtendedInlineSegment>),
    Superscript(Vec<ExtendedInlineSegment>),
    Subscript(Vec<ExtendedInlineSegment>),
    Emoji(&'static str),
    AutoLink(String),
}

/// Delimited extended syntax shared by Preview and the source-ranged Visual
/// Edit model. Ranges returned by [`extended_inline_matches`] are relative to
/// the input slice and always land on UTF-8 boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExtendedInlineKind {
    Highlight,
    Superscript,
    Subscript,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExtendedInlineMatch {
    pub kind: ExtendedInlineKind,
    pub source_range: Range<usize>,
    pub content_range: Range<usize>,
}

/// Finds every valid delimited extended-inline construct, including constructs
/// nested inside another construct. The grammar is intentionally the same one
/// used by Preview's recursive segment parser below.
pub(crate) fn extended_inline_matches(text: &str) -> Vec<ExtendedInlineMatch> {
    fn collect(text: &str, base: usize, matches: &mut Vec<ExtendedInlineMatch>) {
        let mut index = 0usize;
        while index < text.len() {
            let rest = &text[index..];
            if rest.starts_with("~~") {
                index += 2;
                continue;
            }
            if let Some(found) = consume_extended_inline_delimiter(rest) {
                let inner = &rest[found.content_range.clone()];
                let content_start = base + index + found.content_range.start;
                matches.push(ExtendedInlineMatch {
                    kind: found.kind,
                    source_range: base + index + found.source_range.start
                        ..base + index + found.source_range.end,
                    content_range: content_start..base + index + found.content_range.end,
                });
                collect(inner, content_start, matches);
                index += found.source_range.end;
                continue;
            }

            index += rest
                .chars()
                .next()
                .expect("non-empty extended-inline remainder")
                .len_utf8();
        }
    }

    let mut matches = Vec::new();
    collect(text, 0, &mut matches);
    matches
}

pub(crate) fn render_extended_inline_plain(text: &str) -> String {
    let mut output = String::new();
    for segment in parse_extended_inline_segments(text) {
        match segment {
            ExtendedInlineSegment::Text(text) | ExtendedInlineSegment::AutoLink(text) => {
                output.push_str(&text);
            }
            ExtendedInlineSegment::Highlight(children)
            | ExtendedInlineSegment::Superscript(children)
            | ExtendedInlineSegment::Subscript(children) => {
                output.push_str(&render_extended_segments_plain(&children));
            }
            ExtendedInlineSegment::Emoji(emoji) => output.push_str(emoji),
        }
    }
    output
}

fn render_extended_segments_plain(segments: &[ExtendedInlineSegment]) -> String {
    let mut output = String::new();
    for segment in segments {
        match segment {
            ExtendedInlineSegment::Text(text) | ExtendedInlineSegment::AutoLink(text) => {
                output.push_str(text);
            }
            ExtendedInlineSegment::Highlight(children)
            | ExtendedInlineSegment::Superscript(children)
            | ExtendedInlineSegment::Subscript(children) => {
                output.push_str(&render_extended_segments_plain(children));
            }
            ExtendedInlineSegment::Emoji(emoji) => output.push_str(emoji),
        }
    }
    output
}

pub(crate) fn render_extended_html_text_nodes(html: &str) -> String {
    let mut output = String::new();
    let mut index = 0usize;
    let mut raw_text_depth = 0usize;
    // pulldown's math containers hold raw LaTeX; extended inline syntax
    // (superscript & co.) must not rewrite formula payloads.
    let mut math_depth = 0usize;

    while index < html.len() {
        if html[index..].starts_with('<') {
            let tag_end = html[index..]
                .find('>')
                .map_or(html.len(), |relative| index + relative + 1);
            let tag = &html[index..tag_end];
            if let Some((name, closing)) = html_tag_name(tag) {
                if matches!(name.as_str(), "code" | "pre" | "script" | "style") {
                    if closing {
                        raw_text_depth = raw_text_depth.saturating_sub(1);
                    } else if !tag.ends_with("/>") {
                        raw_text_depth += 1;
                    }
                }
                if matches!(name.as_str(), "span" | "div") {
                    if closing {
                        math_depth = math_depth.saturating_sub(1);
                    } else if !tag.ends_with("/>") && (math_depth > 0 || is_math_container_tag(tag))
                    {
                        math_depth += 1;
                    }
                }
            }
            output.push_str(tag);
            index = tag_end;
            continue;
        }

        let next_tag = html[index..]
            .find('<')
            .map_or(html.len(), |relative| index + relative);
        let text = &html[index..next_tag];
        if raw_text_depth == 0 && math_depth == 0 {
            output.push_str(&render_extended_inline_html_text_node(text));
        } else {
            output.push_str(text);
        }
        index = next_tag;
    }

    output
}

pub fn html_preview_plain_text(html: &str) -> String {
    html_preview_parts(html)
        .into_iter()
        .filter_map(|part| match part {
            HtmlPreviewPart::Text { text, .. } if !text.is_empty() => Some(text.text),
            HtmlPreviewPart::Image { alt, url, .. } => {
                if alt.is_empty() {
                    Some(url)
                } else {
                    Some(alt)
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Recognizes exactly one complete, non-closing raw-HTML `<img …>` tag with a
/// non-empty `src` attribute — the narrow exact form the Visual Edit inline
/// projection can map byte-for-byte. `source` must be a single tag slice (no
/// surrounding content, no second tag, no inner angle brackets), so anything
/// else — other tags, partial tags, comments, missing `src` — returns `None`
/// and callers keep their conservative fallback.
pub(crate) fn parse_inline_html_image(source: &str) -> Option<VisualHtmlImage> {
    let trimmed = source.trim();
    let inner = trimmed.strip_prefix('<')?.strip_suffix('>')?;
    if inner.trim().is_empty() || inner.contains('<') || inner.contains('>') {
        return None;
    }
    let tag = ParsedHtmlTag::parse(trimmed)?;
    if tag.closing || tag.name != "img" {
        return None;
    }
    let url = tag.attr("src").filter(|url| !url.is_empty())?;
    Some(VisualHtmlImage {
        alt: tag.attr("alt").unwrap_or_default(),
        url,
        title: tag.attr("title").filter(|title| !title.is_empty()),
        width: tag.attr("width").as_deref().and_then(parse_html_img_length),
        height: tag
            .attr("height")
            .as_deref()
            .and_then(parse_html_img_length),
    })
}

pub(crate) fn parse_html_img_length(value: &str) -> Option<HtmlImgLength> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(percent) = trimmed.strip_suffix('%') {
        let n: u16 = percent.trim().parse().ok()?;
        return Some(HtmlImgLength::Percent(n));
    }
    let px = trimmed
        .strip_suffix("px")
        .or_else(|| trimmed.strip_suffix("PX"))
        .unwrap_or(trimmed)
        .trim();
    let n: u32 = px.parse().ok()?;
    (n > 0).then_some(HtmlImgLength::Px(n))
}

pub fn resolve_html_img_display_size(
    width: Option<HtmlImgLength>,
    height: Option<HtmlImgLength>,
    intrinsic_w: f32,
    intrinsic_h: f32,
) -> Option<(f32, f32)> {
    if width.is_none() && height.is_none() {
        return None;
    }
    let resolve = |len: HtmlImgLength, intrinsic: f32| match len {
        HtmlImgLength::Px(px) => px as f32,
        HtmlImgLength::Percent(percent) => intrinsic * f32::from(percent) / 100.0,
    };
    match (width, height) {
        (Some(width), Some(height)) => {
            Some((resolve(width, intrinsic_w), resolve(height, intrinsic_h)))
        }
        (Some(width), None) => {
            let width = resolve(width, intrinsic_w);
            let height = if intrinsic_w > 0.0 {
                width * intrinsic_h / intrinsic_w
            } else {
                intrinsic_h
            };
            Some((width, height))
        }
        (None, Some(height)) => {
            let height = resolve(height, intrinsic_h);
            let width = if intrinsic_h > 0.0 {
                height * intrinsic_w / intrinsic_h
            } else {
                intrinsic_w
            };
            Some((width, height))
        }
        (None, None) => None,
    }
}

/// One style flag the supported inline-HTML subset can contribute to the
/// Visual Edit inline projection. Each kind maps to exactly one `InlineStyle`
/// flag; the mapping itself lives in the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineHtmlStyleKind {
    Emphasis,
    Strong,
    Strikethrough,
    Code,
    Highlight,
    Subscript,
    Superscript,
}

/// Recognized form of one supported inline-HTML tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineHtmlStyleTag {
    Open { kind: InlineHtmlStyleKind },
    Close { kind: InlineHtmlStyleKind },
    LineBreak,
}

/// Recognizes exactly one complete inline-HTML tag from the narrow subset the
/// Visual Edit inline projection can hide as a marker: the style pairs
/// `em`/`i`, `strong`/`b`, `s`/`del`/`strike`, `code`, `mark`, `sub`, `sup`
/// (tag names case-insensitive) and the void line-break forms `<br>`, `<br/>`,
/// `<br />`. Ignorable presentation attributes `class`, `id`, and `clear` are
/// accepted without mapping to style. Like `parse_inline_html_image`, `source`
/// must be a single tag slice. Anything else — unknown tags, non-ignorable
/// attributes, self-closing style pairs, closing `</br>` — returns `None` so
/// callers keep the conservative fallback.
pub(crate) fn parse_inline_html_style_tag(source: &str) -> Option<InlineHtmlStyleTag> {
    let trimmed = source.trim();
    let inner = trimmed.strip_prefix('<')?.strip_suffix('>')?;
    if inner.trim().is_empty() || inner.contains('<') || inner.contains('>') {
        return None;
    }
    let tag = ParsedHtmlTag::parse(trimmed)?;
    if !tag
        .attrs
        .iter()
        .all(|(name, _)| matches!(name.as_str(), "class" | "id" | "clear"))
    {
        return None;
    }
    let kind = match tag.name.as_str() {
        "em" | "i" => InlineHtmlStyleKind::Emphasis,
        "strong" | "b" => InlineHtmlStyleKind::Strong,
        "s" | "del" | "strike" => InlineHtmlStyleKind::Strikethrough,
        "code" => InlineHtmlStyleKind::Code,
        "mark" => InlineHtmlStyleKind::Highlight,
        "sub" => InlineHtmlStyleKind::Subscript,
        "sup" => InlineHtmlStyleKind::Superscript,
        "br" if !tag.closing => return Some(InlineHtmlStyleTag::LineBreak),
        _ => return None,
    };
    if tag.self_closing {
        return None;
    }
    Some(if tag.closing {
        InlineHtmlStyleTag::Close { kind }
    } else {
        InlineHtmlStyleTag::Open { kind }
    })
}

pub fn html_preview_parts(html: &str) -> Vec<HtmlPreviewPart> {
    let mut parts = Vec::new();
    let mut index = 0usize;
    while index < html.len() {
        let rest = &html[index..];
        let Some(relative) = find_html_table_start(rest) else {
            parts.extend(HtmlPreviewBuilder::new(rest).finish());
            break;
        };
        let table_start = index + relative;
        if table_start > index {
            parts.extend(HtmlPreviewBuilder::new(&html[index..table_start]).finish());
        }
        match parse_html_table_at(html, table_start) {
            Some((grid, table_end)) => {
                parts.push(HtmlPreviewPart::Table { grid });
                index = table_end.max(table_start + 1);
            }
            None => {
                parts.extend(HtmlPreviewBuilder::new(&html[table_start..]).finish());
                break;
            }
        }
    }
    parts
}

fn find_html_table_start(html: &str) -> Option<usize> {
    html.to_ascii_lowercase().find("<table")
}

/// Resolves a raw HTML `<table>...</table>` into a grid honoring `rowspan` and
/// `colspan`. Returns `None` for non-table HTML or any structure that cannot be
/// turned into a grid, so callers fall back to the flattener instead of
/// panicking.
///
/// The grid uses browser-style placement: each `<td>`/`<th>` is dropped at the
/// next free column in its row, skipping cells still held open by a `rowspan`
/// from an earlier row. Spacer slots mark those held-open cells so the renderer
/// can reserve width without drawing content.
#[cfg(test)]
pub(crate) fn parse_html_table_grid(html: &str) -> Option<HtmlTableGrid> {
    parse_html_table_at(html, 0).map(|(grid, _)| grid)
}

fn parse_html_table_at(html: &str, start: usize) -> Option<(HtmlTableGrid, usize)> {
    HtmlTableParser::new(&html[start..])
        .parse()
        .map(|(grid, end)| (grid, start + end))
}

struct HtmlTableParser<'a> {
    html: &'a str,
    index: usize,
    /// Whether the parser is still inside a `<table>` element.
    in_table: bool,
    /// Whether we ever opened a `<table>` (stays true after it closes).
    saw_table: bool,
    /// Whether the current row is inside `<thead>` (forced header cells) —
    /// any explicit `<th>` is a header regardless of this flag.
    in_thead: bool,
    /// Per-column remaining rows held open by an active rowspan. Index = column.
    row_spans: Vec<usize>,
    current_row: Option<Vec<HtmlTableCell>>,
    current_cell_spans: Vec<InlineSpan>,
    grid: HtmlTableGrid,
    failed: bool,
    /// Pending cell flags captured on `<td>`/`<th>` open, consumed on close.
    pending_is_header: bool,
    pending_colspan: usize,
    pending_rowspan: usize,
    /// True between a `<td>`/`<th>` opening tag and its close (or the next open).
    cell_open: bool,
    /// Inline style depths tracked while inside a cell.
    bold_depth: usize,
    italic_depth: usize,
    code_depth: usize,
    strike_depth: usize,
    /// Byte end of the outer `</table>` (or the slice end if unclosed).
    table_end: Option<usize>,
    /// Open `<a href>` destinations while inside a cell.
    links: Vec<String>,
    /// Last `<img>` captured in the current cell.
    pending_image: Option<HtmlTableCellImage>,
}

impl<'a> HtmlTableParser<'a> {
    fn new(html: &'a str) -> Self {
        Self {
            html,
            index: 0,
            in_table: false,
            saw_table: false,
            in_thead: false,
            row_spans: Vec::new(),
            current_row: None,
            current_cell_spans: Vec::new(),
            grid: HtmlTableGrid::default(),
            failed: false,
            pending_is_header: false,
            pending_colspan: 1,
            pending_rowspan: 1,
            cell_open: false,
            bold_depth: 0,
            italic_depth: 0,
            code_depth: 0,
            strike_depth: 0,
            table_end: None,
            links: Vec::new(),
            pending_image: None,
        }
    }

    fn parse(mut self) -> Option<(HtmlTableGrid, usize)> {
        while self.index < self.html.len() {
            if self.html[self.index..].starts_with('<')
                && let Some(tag_end) = find_html_tag_end(self.html, self.index)
            {
                let tag = &self.html[self.index..tag_end];
                self.handle_tag(tag);
                self.index = tag_end;
                if self.failed {
                    return None;
                }
                if self.table_end.is_some() {
                    break;
                }
                continue;
            }
            let next_tag = self.html[self.index..]
                .find('<')
                .map_or(self.html.len(), |relative| self.index + relative);
            let text = &self.html[self.index..next_tag];
            if self.cell_open {
                self.append_cell_text(text);
            }
            self.index = next_tag;
        }

        // Flush a trailing open row so unclosed tables still render their cells.
        self.flush_row();
        if !self.saw_table || self.grid.rows.is_empty() {
            return None;
        }
        Some((self.grid, self.table_end.unwrap_or(self.html.len())))
    }

    fn handle_tag(&mut self, tag: &str) {
        let Some(parsed) = ParsedHtmlTag::parse(tag) else {
            return;
        };
        match parsed.name.as_str() {
            "table" => {
                if parsed.closing {
                    self.flush_row();
                    self.in_table = false;
                    self.table_end = Some(self.index + tag.len());
                } else if !parsed.self_closing {
                    if self.in_table {
                        // Nested tables are out of scope; bail to the flattener.
                        self.failed = true;
                    } else {
                        self.in_table = true;
                        self.saw_table = true;
                    }
                }
            }
            "thead" => {
                if parsed.closing {
                    self.in_thead = false;
                } else if !parsed.self_closing {
                    self.in_thead = true;
                }
            }
            "tbody" | "tfoot" | "colgroup" | "caption" => {
                // Section containers: ignored; rows inside them still parse.
            }
            "tr" => {
                if parsed.closing || parsed.self_closing {
                    self.flush_row();
                } else {
                    // Unclosed previous `<tr>`: flush before starting the new row.
                    if self.current_row.is_some() {
                        self.flush_row();
                    }
                    self.begin_row();
                }
            }
            "td" | "th" => {
                let is_header = parsed.name == "th" || self.in_thead;
                if parsed.closing {
                    self.finish_cell();
                } else {
                    // Tolerate an unclosed previous cell: finish it first.
                    if self.cell_open {
                        self.finish_cell();
                    }
                    let colspan = parse_span_attr(parsed.attr("colspan"));
                    let rowspan = parse_span_attr(parsed.attr("rowspan"));
                    self.begin_cell(is_header, colspan, rowspan);
                    if parsed.self_closing {
                        self.finish_cell();
                    }
                }
            }
            "br" if self.cell_open => {
                self.append_cell_text("\n");
            }
            "strong" | "b" if self.cell_open => {
                self.adjust_depth(parsed.closing, |s| &mut s.bold_depth)
            }
            "em" | "i" if self.cell_open => {
                self.adjust_depth(parsed.closing, |s| &mut s.italic_depth)
            }
            "code" | "kbd" | "samp" if self.cell_open => {
                self.adjust_depth(parsed.closing, |s| &mut s.code_depth)
            }
            "s" | "del" | "strike" if self.cell_open => {
                self.adjust_depth(parsed.closing, |s| &mut s.strike_depth)
            }
            "a" if self.cell_open => {
                if parsed.closing {
                    self.links.pop();
                } else if let Some(href) = parsed.attr("href") {
                    self.links.push(href);
                }
            }
            "img" if self.cell_open && !parsed.closing => {
                if let Some(url) = parsed.attr("src") {
                    self.pending_image = Some(HtmlTableCellImage {
                        alt: parsed.attr("alt").unwrap_or_default(),
                        url,
                        title: parsed.attr("title").filter(|title| !title.is_empty()),
                        width: parsed
                            .attr("width")
                            .as_deref()
                            .and_then(parse_html_img_length),
                        height: parsed
                            .attr("height")
                            .as_deref()
                            .and_then(parse_html_img_length),
                    });
                }
            }
            _ => {}
        }
    }

    fn begin_row(&mut self) {
        self.current_row = Some(Vec::new());
        if self.row_spans.is_empty() {
            self.row_spans.push(0);
        }
    }

    fn begin_cell(&mut self, is_header: bool, colspan: usize, rowspan: usize) {
        self.pending_is_header = is_header;
        self.pending_colspan = colspan;
        self.pending_rowspan = rowspan;
        self.current_cell_spans.clear();
        self.links.clear();
        self.pending_image = None;
        self.cell_open = true;
    }

    fn finish_cell(&mut self) {
        if !self.cell_open {
            return;
        }
        self.cell_open = false;
        if self.current_row.is_none() {
            self.current_cell_spans.clear();
            return;
        }
        let colspan = self.pending_colspan.max(1);
        let rowspan = self.pending_rowspan.max(1);
        let is_header = self.pending_is_header;
        let content = finish_rich_text(std::mem::take(&mut self.current_cell_spans));
        let image = self.pending_image.take();
        self.place_cell(HtmlTableCell {
            content,
            image,
            colspan,
            rowspan,
            is_header,
            is_spacer: false,
        });
        self.pending_is_header = false;
        self.pending_colspan = 1;
        self.pending_rowspan = 1;
    }

    /// Places a cell at the next free column, skipping rowspan-held columns,
    /// and emits spacer slots for held-open columns. Decrement rowspan counters.
    fn place_cell(&mut self, mut cell: HtmlTableCell) {
        if cell.colspan == 0 {
            cell.colspan = 1;
        }
        if cell.rowspan == 0 {
            cell.rowspan = 1;
        }
        // Snapshot the columns already consumed in the current row and the
        // rowspan-held columns, so we never hold a borrow of self across pushes.
        let (col, spacers) = match self.current_row.as_ref() {
            Some(row) => {
                let used = row.iter().map(|c| c.colspan).sum::<usize>();
                let mut advance = used;
                let mut to_add = Vec::new();
                while advance < self.row_spans.len() && self.row_spans[advance] > 0 {
                    // Each held-open column gets a single-column spacer; runs of
                    // consecutive held columns become multiple width-1 spacers,
                    // which is visually equivalent for border alignment.
                    to_add.push(advance);
                    advance += 1;
                }
                (advance, to_add)
            }
            None => (0usize, Vec::new()),
        };

        let end_col = col + cell.colspan;
        while self.row_spans.len() < end_col {
            self.row_spans.push(0);
        }
        if cell.rowspan > 1 {
            for c in col..end_col {
                self.row_spans[c] = self.row_spans[c].max(cell.rowspan);
            }
            self.grid.has_rowspan = true;
        }

        let row = self
            .current_row
            .as_mut()
            .expect("row in progress when placing a cell");
        for _ in spacers {
            row.push(HtmlTableCell {
                content: RichText::default(),
                image: None,
                colspan: 1,
                rowspan: 0,
                is_header: false,
                is_spacer: true,
            });
        }
        row.push(cell);
        let new_columns = row.iter().map(|c| c.colspan).sum::<usize>();
        if new_columns > self.grid.columns {
            self.grid.columns = new_columns;
        }
    }

    fn flush_row(&mut self) {
        // First, finish a pending open cell.
        if self.cell_open {
            self.finish_cell();
        }
        if let Some(mut row) = self.current_row.take() {
            // Top up trailing spacer slots for any remaining rowspan-held columns.
            let mut col = row.iter().map(|c| c.colspan).sum::<usize>();
            while col < self.row_spans.len() && self.row_spans[col] > 0 {
                row.push(HtmlTableCell {
                    content: RichText::default(),
                    image: None,
                    colspan: 1,
                    rowspan: 0,
                    is_header: false,
                    is_spacer: true,
                });
                col += 1;
            }
            // Decrement rowspan counters for the row we just finished.
            for c in 0..self.row_spans.len() {
                if self.row_spans[c] > 0 {
                    self.row_spans[c] -= 1;
                }
            }
            if !row.is_empty() {
                let row_columns = row.iter().map(|c| c.colspan).sum::<usize>();
                if row_columns > self.grid.columns {
                    self.grid.columns = row_columns;
                }
                self.grid.rows.push(row);
            }
        }
        self.pending_is_header = false;
        self.pending_colspan = 1;
        self.pending_rowspan = 1;
    }

    fn append_cell_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let decoded = decode_html_entities(text);
        let style = self.cell_style();
        let link = self.links.last().cloned();
        for ch in decoded.chars() {
            if ch.is_whitespace() {
                // Collapse runs of whitespace to a single space, mirroring HTML.
                if self.current_cell_spans.last().is_some_and(|span| {
                    span.math.is_none() && span.style == style && span.text.ends_with(' ')
                }) {
                    continue;
                }
                append_span(&mut self.current_cell_spans, " ", style, link.as_deref());
                continue;
            }
            let mut buf = [0u8; 4];
            append_span(
                &mut self.current_cell_spans,
                ch.encode_utf8(&mut buf),
                style,
                link.as_deref(),
            );
        }
    }

    fn cell_style(&self) -> InlineStyle {
        InlineStyle {
            bold: self.bold_depth > 0,
            italic: self.italic_depth > 0,
            code: self.code_depth > 0,
            strikethrough: self.strike_depth > 0,
            ..InlineStyle::default()
        }
    }

    /// Adjust an inline-style depth counter on opening/closing style tags,
    /// matching the saturating add/sub used by the HTML flattener.
    fn adjust_depth(&mut self, closing: bool, get: impl Fn(&mut Self) -> &mut usize) {
        let depth = get(self);
        if closing {
            *depth = depth.saturating_sub(1);
        } else {
            *depth += 1;
        }
    }
}

/// Parses a `rowspan`/`colspan` attribute value into a `usize >= 1`. Non-numeric,
/// zero, or negative values (per HTML spec, `0` is clamped to `1`) fall back to 1.
fn parse_span_attr(value: Option<String>) -> usize {
    value
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|n| *n >= 1)
        .unwrap_or(1)
}

struct HtmlPreviewBuilder<'a> {
    html: &'a str,
    index: usize,
    parts: Vec<HtmlPreviewPart>,
    spans: Vec<InlineSpan>,
    style: InlineStyle,
    pending_space: Option<HtmlPendingSpace>,
    bold_depth: usize,
    italic_depth: usize,
    code_depth: usize,
    strike_depth: usize,
    links: Vec<String>,
    centered_depth: usize,
    text_centered: bool,
    underline_depth: usize,
    pre_depth: usize,
    heading_level: Option<u8>,
    align: HtmlAlign,
    align_stack: Vec<HtmlAlign>,
    colors: Vec<Option<u32>>,
    list_stack: Vec<Option<u64>>,
    list_marker: Option<HtmlListMarker>,
    text_heading_level: Option<u8>,
    text_list_marker: Option<HtmlListMarker>,
    text_pre: bool,
    text_align: HtmlAlign,
}

struct HtmlPendingSpace {
    style: InlineStyle,
    link: Option<String>,
    centered: bool,
}

impl<'a> HtmlPreviewBuilder<'a> {
    fn new(html: &'a str) -> Self {
        Self {
            html,
            index: 0,
            parts: Vec::new(),
            spans: Vec::new(),
            style: InlineStyle::default(),
            pending_space: None,
            bold_depth: 0,
            italic_depth: 0,
            code_depth: 0,
            strike_depth: 0,
            links: Vec::new(),
            centered_depth: 0,
            text_centered: false,
            underline_depth: 0,
            pre_depth: 0,
            heading_level: None,
            align: HtmlAlign::Start,
            align_stack: Vec::new(),
            colors: Vec::new(),
            list_stack: Vec::new(),
            list_marker: None,
            text_heading_level: None,
            text_list_marker: None,
            text_pre: false,
            text_align: HtmlAlign::Start,
        }
    }

    fn finish(mut self) -> Vec<HtmlPreviewPart> {
        while self.index < self.html.len() {
            if self.html[self.index..].starts_with('<')
                && let Some(tag_end) = find_html_tag_end(self.html, self.index)
            {
                let tag = &self.html[self.index..tag_end];
                self.handle_tag(tag);
                self.index = tag_end;
                continue;
            }

            let next_tag = self.html[self.index..]
                .find('<')
                .map_or(self.html.len(), |relative| self.index + relative);
            let text = &self.html[self.index..next_tag];
            self.push_text(text);
            self.index = next_tag;
        }
        self.flush_text();
        self.parts
    }

    fn handle_tag(&mut self, tag: &str) {
        let Some(parsed) = ParsedHtmlTag::parse(tag) else {
            return;
        };
        if parsed.name == "script" || parsed.name == "style" {
            if !parsed.closing
                && let Some(end) = find_html_closing_tag(self.html, self.index, &parsed.name)
            {
                self.index = end;
            }
            return;
        }

        match parsed.name.as_str() {
            "br" => self.push_line_break(),
            "pre" => {
                self.pending_space = None;
                if parsed.closing {
                    self.flush_text();
                    self.pre_depth = self.pre_depth.saturating_sub(1);
                } else if !parsed.self_closing {
                    if self.has_text() {
                        self.flush_text();
                    }
                    self.pre_depth += 1;
                    self.apply_open_align(&parsed);
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.pending_space = None;
                let level = parsed.name.as_bytes()[1].saturating_sub(b'0');
                if parsed.closing {
                    self.flush_text();
                    self.heading_level = None;
                    self.pop_align();
                } else if !parsed.self_closing {
                    if self.has_text() {
                        self.flush_text();
                    }
                    self.heading_level = Some(level);
                    self.apply_open_align(&parsed);
                }
            }
            "ul" => {
                if parsed.closing {
                    self.list_stack.pop();
                    self.pop_align();
                } else if !parsed.self_closing {
                    self.list_stack.push(None);
                    self.apply_open_align(&parsed);
                }
            }
            "ol" => {
                if parsed.closing {
                    self.list_stack.pop();
                    self.pop_align();
                } else if !parsed.self_closing {
                    self.list_stack.push(Some(1));
                    self.apply_open_align(&parsed);
                }
            }
            "p" | "div" | "section" | "article" | "header" | "footer" | "li" | "tr" | "table"
            | "blockquote" => {
                self.pending_space = None;
                if parsed.name == "li" && !parsed.closing && !parsed.self_closing {
                    self.list_marker = Some(match self.list_stack.last_mut() {
                        Some(Some(counter)) => {
                            let marker = HtmlListMarker::Decimal(*counter);
                            *counter = counter.saturating_add(1);
                            marker
                        }
                        _ => HtmlListMarker::Disc,
                    });
                }
                if parsed.closing {
                    self.flush_text();
                    if parsed.name == "li" {
                        self.list_marker = None;
                    }
                    self.pop_align();
                    self.push_line_break();
                } else {
                    if self.has_text() {
                        self.flush_text();
                    }
                    self.apply_open_align(&parsed);
                }
            }
            "u" => {
                if parsed.closing {
                    self.underline_depth = self.underline_depth.saturating_sub(1);
                } else if !parsed.self_closing {
                    self.underline_depth += 1;
                }
                self.update_style();
            }
            "font" => {
                if parsed.closing {
                    self.pop_color();
                } else if !parsed.self_closing {
                    self.push_color(parsed.attr("color").as_deref().and_then(parse_css_color));
                }
                self.update_style();
            }
            "span" => {
                if parsed.closing {
                    self.pop_color();
                    self.pop_align();
                } else if !parsed.self_closing {
                    self.apply_open_align(&parsed);
                    self.push_color(parsed.attr("style").as_deref().and_then(color_from_style));
                }
                self.update_style();
            }
            "strong" | "b" => {
                if parsed.closing {
                    self.bold_depth = self.bold_depth.saturating_sub(1);
                    self.pop_color();
                } else if !parsed.self_closing {
                    self.bold_depth += 1;
                    self.push_color(parsed.attr("style").as_deref().and_then(color_from_style));
                }
                self.update_style();
            }
            "em" | "i" => {
                if parsed.closing {
                    self.italic_depth = self.italic_depth.saturating_sub(1);
                    self.pop_color();
                } else if !parsed.self_closing {
                    self.italic_depth += 1;
                    self.push_color(parsed.attr("style").as_deref().and_then(color_from_style));
                }
                self.update_style();
            }
            "code" | "kbd" | "samp" => {
                if parsed.closing {
                    self.code_depth = self.code_depth.saturating_sub(1);
                } else if !parsed.self_closing {
                    self.code_depth += 1;
                }
                self.update_style();
            }
            "s" | "del" | "strike" => {
                if parsed.closing {
                    self.strike_depth = self.strike_depth.saturating_sub(1);
                } else if !parsed.self_closing {
                    self.strike_depth += 1;
                }
                self.update_style();
            }
            "a" => {
                if parsed.closing {
                    self.links.pop();
                } else if let Some(href) = parsed.attr("href") {
                    self.links.push(href);
                }
            }
            "img" if !parsed.closing => {
                if let Some(url) = parsed.attr("src") {
                    self.pending_space = None;
                    self.flush_text();
                    let align = self.current_align().combine(parsed.html_align());
                    self.parts.push(HtmlPreviewPart::Image {
                        alt: parsed.attr("alt").unwrap_or_default(),
                        url,
                        title: parsed.attr("title").filter(|title| !title.is_empty()),
                        centered: align == HtmlAlign::Center,
                        width: parsed
                            .attr("width")
                            .as_deref()
                            .and_then(parse_html_img_length),
                        height: parsed
                            .attr("height")
                            .as_deref()
                            .and_then(parse_html_img_length),
                        align,
                    });
                }
            }
            _ => {}
        }
    }

    fn update_style(&mut self) {
        self.style.bold = self.bold_depth > 0;
        self.style.italic = self.italic_depth > 0;
        self.style.code = self.code_depth > 0;
        self.style.strikethrough = self.strike_depth > 0;
        self.style.underline = self.underline_depth > 0;
        self.style.color = self.colors.iter().rev().find_map(|color| *color);
    }

    fn apply_open_align(&mut self, parsed: &ParsedHtmlTag) {
        self.align_stack.push(self.align);
        self.align = self.align.combine(parsed.html_align());
        self.centered_depth = usize::from(self.align == HtmlAlign::Center);
    }

    fn pop_align(&mut self) {
        self.align = self.align_stack.pop().unwrap_or(HtmlAlign::Start);
        self.centered_depth = usize::from(self.align == HtmlAlign::Center);
    }

    fn current_align(&self) -> HtmlAlign {
        self.align
    }

    fn push_color(&mut self, color: Option<u32>) {
        self.colors.push(color);
    }

    fn pop_color(&mut self) {
        self.colors.pop();
    }

    fn capture_text_meta(&mut self) {
        if self.spans.is_empty() {
            self.text_heading_level = self.heading_level;
            self.text_list_marker = self.list_marker;
            self.text_pre = self.pre_depth > 0;
            self.text_align = self.current_align();
            self.text_centered = self.text_align == HtmlAlign::Center;
        }
    }

    fn push_text(&mut self, text: &str) {
        let decoded = decode_html_entities(text);
        if self.pre_depth > 0 {
            for ch in decoded.chars() {
                if ch == '\n' {
                    self.push_line_break();
                    continue;
                }
                self.capture_text_meta();
                let mut buf = [0u8; 4];
                self.push_visible(ch.encode_utf8(&mut buf));
            }
            return;
        }
        for ch in decoded.chars() {
            if ch.is_whitespace() {
                self.pending_space = Some(HtmlPendingSpace {
                    style: self.style,
                    link: self.links.last().cloned(),
                    centered: self.centered_depth > 0,
                });
                continue;
            }
            self.push_pending_space();
            self.capture_text_meta();
            let mut buf = [0u8; 4];
            self.push_visible(ch.encode_utf8(&mut buf));
        }
    }

    fn push_line_break(&mut self) {
        self.pending_space = None;
        if self.has_text() && !self.ends_with_line_break() {
            self.text_centered |= self.centered_depth > 0;
            append_span(
                &mut self.spans,
                "\n",
                InlineStyle::default(),
                self.links.last().map(String::as_str),
            );
        }
    }

    fn push_pending_space(&mut self) {
        let Some(pending) = self.pending_space.take() else {
            return;
        };
        if self.needs_space_before_text() {
            self.text_centered |= pending.centered;
            append_span(&mut self.spans, " ", pending.style, pending.link.as_deref());
        }
    }

    fn push_visible(&mut self, text: &str) {
        self.capture_text_meta();
        self.text_centered |= self.centered_depth > 0 || self.current_align() == HtmlAlign::Center;
        append_span(
            &mut self.spans,
            text,
            self.style,
            self.links.last().map(String::as_str),
        );
    }

    fn flush_text(&mut self) {
        self.pending_space = None;
        let pre = self.text_pre || self.pre_depth > 0;
        let spans = std::mem::take(&mut self.spans);
        let text = if pre {
            let joined: String = spans.iter().map(|span| span.text.as_str()).collect();
            RichText {
                text: joined,
                spans,
            }
        } else {
            finish_rich_text(spans)
        };
        if self.text_list_marker.is_none() {
            self.text_list_marker = self.list_marker;
        }
        if self.text_heading_level.is_none() {
            self.text_heading_level = self.heading_level;
        }
        if self.text_align == HtmlAlign::Start {
            self.text_align = self.current_align();
        }
        if !text.is_empty() {
            let align = if self.text_centered || self.text_align == HtmlAlign::Center {
                HtmlAlign::Center
            } else {
                self.text_align
            };
            self.parts.push(HtmlPreviewPart::Text {
                text,
                centered: align == HtmlAlign::Center,
                heading_level: self.text_heading_level,
                list_marker: self.text_list_marker.take(),
                pre,
                align,
            });
            self.list_marker = None;
        }
        self.text_centered = false;
        self.text_heading_level = None;
        self.text_list_marker = None;
        self.text_pre = false;
        self.text_align = HtmlAlign::Start;
    }

    fn has_text(&self) -> bool {
        self.spans.iter().any(|span| !span.text.trim().is_empty())
    }

    fn ends_with_line_break(&self) -> bool {
        self.spans
            .last()
            .is_some_and(|span| span.text.ends_with('\n'))
    }

    fn needs_space_before_text(&self) -> bool {
        self.spans
            .last()
            .and_then(|span| span.text.chars().last())
            .is_some_and(|ch| !ch.is_whitespace())
    }
}

struct ParsedHtmlTag {
    name: String,
    closing: bool,
    self_closing: bool,
    attrs: Vec<(String, String)>,
}

impl ParsedHtmlTag {
    fn parse(tag: &str) -> Option<Self> {
        let mut rest = tag.strip_prefix('<')?.strip_suffix('>')?.trim();
        if rest.starts_with('!') || rest.starts_with('?') {
            return None;
        }
        let closing = rest.starts_with('/');
        if closing {
            rest = rest[1..].trim_start();
        }
        let self_closing = rest.ends_with('/');
        if self_closing {
            rest = rest[..rest.len() - 1].trim_end();
        }
        let name_end = rest
            .char_indices()
            .find_map(|(index, ch)| (!ch.is_ascii_alphanumeric()).then_some(index))
            .unwrap_or(rest.len());
        let name = rest[..name_end].to_ascii_lowercase();
        if name.is_empty() {
            return None;
        }
        Some(Self {
            name,
            closing,
            self_closing,
            attrs: parse_html_attrs(&rest[name_end..]),
        })
    }

    fn attr(&self, name: &str) -> Option<String> {
        self.attrs
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.clone())
    }

    fn html_align(&self) -> HtmlAlign {
        if let Some(value) = self.attr("align") {
            return match value.to_ascii_lowercase().as_str() {
                "center" => HtmlAlign::Center,
                "right" => HtmlAlign::End,
                "left" => HtmlAlign::Start,
                _ => HtmlAlign::Start,
            };
        }
        if let Some(style) = self.attr("style") {
            let lower = style.to_ascii_lowercase();
            if css_decl_contains(&lower, "text-align", "center") {
                return HtmlAlign::Center;
            }
            if css_decl_contains(&lower, "text-align", "right") {
                return HtmlAlign::End;
            }
            if css_decl_contains(&lower, "text-align", "left") {
                return HtmlAlign::Start;
            }
        }
        HtmlAlign::Start
    }
}

fn css_decl_contains(style: &str, property: &str, value: &str) -> bool {
    let needle = format!("{property}:");
    style.split(';').any(|decl| {
        let decl = decl.trim();
        decl.starts_with(&needle)
            && decl[needle.len()..]
                .split_whitespace()
                .next()
                .is_some_and(|token| token.trim_end_matches(';') == value)
    })
}

fn color_from_style(style: &str) -> Option<u32> {
    let lower = style.to_ascii_lowercase();
    for (index, _) in lower.match_indices("color:") {
        if index > 0 {
            let previous = lower.as_bytes()[index - 1];
            if previous == b'-' || previous.is_ascii_alphabetic() {
                continue;
            }
        }
        let rest = style[index + "color:".len()..].trim();
        let token_end = rest.find(';').unwrap_or(rest.len());
        return parse_css_color(rest[..token_end].trim());
    }
    None
}

fn parse_css_color(value: &str) -> Option<u32> {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();
    if lower.contains("url(") || lower.contains("var(") || lower.contains("expression") {
        return None;
    }
    if let Some(hex) = lower.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    if let Some(inner) = lower.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let mut parts = inner.split(',').map(|part| part.trim().parse::<u8>().ok());
        let red = parts.next()??;
        let green = parts.next()??;
        let blue = parts.next()??;
        if parts.next().is_some() {
            return None;
        }
        return Some((u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue));
    }
    None
}

fn parse_hex_color(hex: &str) -> Option<u32> {
    let hex = hex.trim();
    match hex.len() {
        3 => {
            let red = u32::from_str_radix(&hex[0..1], 16).ok()?;
            let green = u32::from_str_radix(&hex[1..2], 16).ok()?;
            let blue = u32::from_str_radix(&hex[2..3], 16).ok()?;
            Some((red << 20) | (red << 16) | (green << 12) | (green << 8) | (blue << 4) | blue)
        }
        6 if hex.chars().all(|ch| ch.is_ascii_hexdigit()) => u32::from_str_radix(hex, 16).ok(),
        _ => None,
    }
}

fn find_html_tag_end(html: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, ch) in html[start..].char_indices() {
        if relative == 0 {
            continue;
        }
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (None, '"' | '\'') => quote = Some(ch),
            (None, '>') => return Some(start + relative + 1),
            _ => {}
        }
    }
    None
}

fn find_html_closing_tag(html: &str, start: usize, name: &str) -> Option<usize> {
    let needle = format!("</{name}");
    html[start..]
        .to_ascii_lowercase()
        .find(&needle)
        .and_then(|relative| find_html_tag_end(html, start + relative))
}

fn parse_html_attrs(input: &str) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    let mut index = 0usize;
    while index < input.len() {
        while input[index..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            index += input[index..].chars().next().unwrap().len_utf8();
            if index >= input.len() {
                return attrs;
            }
        }
        let name_start = index;
        while index < input.len()
            && input[index..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        {
            index += input[index..].chars().next().unwrap().len_utf8();
        }
        if index == name_start {
            break;
        }
        let name = input[name_start..index].to_ascii_lowercase();
        while index < input.len()
            && input[index..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
        {
            index += input[index..].chars().next().unwrap().len_utf8();
        }
        let mut value = String::new();
        if input[index..].starts_with('=') {
            index += 1;
            while index < input.len()
                && input[index..]
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace)
            {
                index += input[index..].chars().next().unwrap().len_utf8();
            }
            if let Some(quote @ ('"' | '\'')) = input[index..].chars().next() {
                index += quote.len_utf8();
                let value_start = index;
                while index < input.len() && !input[index..].starts_with(quote) {
                    index += input[index..].chars().next().unwrap().len_utf8();
                }
                value = decode_html_entities(&input[value_start..index]);
                if index < input.len() {
                    index += quote.len_utf8();
                }
            } else {
                let value_start = index;
                while index < input.len()
                    && input[index..]
                        .chars()
                        .next()
                        .is_some_and(|ch| !ch.is_whitespace())
                {
                    index += input[index..].chars().next().unwrap().len_utf8();
                }
                value = decode_html_entities(&input[value_start..index]);
            }
        }
        attrs.push((name, value));
    }
    attrs
}

fn decode_html_entities(text: &str) -> String {
    let mut output = String::new();
    let mut index = 0usize;
    while index < text.len() {
        if text[index..].starts_with('&')
            && let Some(end) = text[index + 1..].find(';')
        {
            let entity = &text[index + 1..index + 1 + end];
            if let Some(decoded) = decode_html_entity(entity) {
                output.push(decoded);
                index += end + 2;
                continue;
            }
        }
        let ch = text[index..].chars().next().unwrap();
        output.push(ch);
        index += ch.len_utf8();
    }
    output
}

fn decode_html_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        "nbsp" => Some(' '),
        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

fn is_math_container_tag(tag: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    lower.contains("math-inline") || lower.contains("math-display")
}

fn html_tag_name(tag: &str) -> Option<(String, bool)> {
    let mut rest = tag.strip_prefix('<')?.trim_start();
    if rest.starts_with('!') || rest.starts_with('?') {
        return None;
    }
    let closing = rest.starts_with('/');
    if closing {
        rest = rest[1..].trim_start();
    }
    let name = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    (!name.is_empty()).then_some((name, closing))
}

fn render_extended_inline_html_text_node(text: &str) -> String {
    let mut output = String::new();
    for segment in parse_extended_inline_segments(text) {
        render_extended_segment_html(&segment, &mut output);
    }
    output
}

fn render_extended_segment_html(segment: &ExtendedInlineSegment, output: &mut String) {
    match segment {
        ExtendedInlineSegment::Text(text) => output.push_str(text),
        ExtendedInlineSegment::Highlight(children) => {
            output.push_str("<mark>");
            render_extended_segments_html(children, output);
            output.push_str("</mark>");
        }
        ExtendedInlineSegment::Superscript(children) => {
            output.push_str("<sup>");
            render_extended_segments_html(children, output);
            output.push_str("</sup>");
        }
        ExtendedInlineSegment::Subscript(children) => {
            output.push_str("<sub>");
            render_extended_segments_html(children, output);
            output.push_str("</sub>");
        }
        ExtendedInlineSegment::Emoji(emoji) => output.push_str(emoji),
        ExtendedInlineSegment::AutoLink(url) => {
            let href = if url.starts_with("www.") {
                format!("https://{url}")
            } else {
                url.clone()
            };
            output.push_str(&format!(
                "<a href=\"{}\">{}</a>",
                escape_html_attribute(&href),
                url
            ));
        }
    }
}

fn render_extended_segments_html(segments: &[ExtendedInlineSegment], output: &mut String) {
    for segment in segments {
        render_extended_segment_html(segment, output);
    }
}

fn parse_extended_inline_segments(text: &str) -> Vec<ExtendedInlineSegment> {
    let mut segments = Vec::new();
    let mut index = 0usize;

    while index < text.len() {
        let rest = &text[index..];

        if rest.starts_with("~~") {
            push_extended_text(&mut segments, "~~");
            index += 2;
            continue;
        }
        if let Some(found) = consume_extended_inline_delimiter(rest) {
            let inner = &rest[found.content_range.clone()];
            let children = parse_extended_inline_segments(inner);
            segments.push(match found.kind {
                ExtendedInlineKind::Highlight => ExtendedInlineSegment::Highlight(children),
                ExtendedInlineKind::Superscript => ExtendedInlineSegment::Superscript(children),
                ExtendedInlineKind::Subscript => ExtendedInlineSegment::Subscript(children),
            });
            index += found.source_range.end;
            continue;
        }

        if let Some(stripped) = rest.strip_prefix(':')
            && let Some(end) = stripped.find(':')
        {
            let shortcode = &stripped[..end];
            if let Some(emoji) = emoji_for_shortcode(shortcode) {
                segments.push(ExtendedInlineSegment::Emoji(emoji));
                index += end + 2;
                continue;
            }
        }

        if let Some((url, consumed)) = consume_autolink(rest) {
            segments.push(ExtendedInlineSegment::AutoLink(url.to_string()));
            index += consumed;
            continue;
        }

        let next = rest.chars().next().expect("non-empty rest");
        push_extended_text(&mut segments, &rest[..next.len_utf8()]);
        index += next.len_utf8();
    }

    segments
}

fn consume_extended_inline_delimiter(text: &str) -> Option<ExtendedInlineMatch> {
    if let Some(stripped) = text.strip_prefix("==")
        && let Some(end) = stripped.find("==")
    {
        let inner = &stripped[..end];
        if !inner.trim().is_empty() {
            return Some(ExtendedInlineMatch {
                kind: ExtendedInlineKind::Highlight,
                source_range: 0..end + 4,
                content_range: 2..end + 2,
            });
        }
    }

    if let Some(stripped) = text.strip_prefix('^')
        && let Some(end) = stripped.find('^')
    {
        let inner = &stripped[..end];
        if is_valid_short_inline_extent(inner) {
            return Some(ExtendedInlineMatch {
                kind: ExtendedInlineKind::Superscript,
                source_range: 0..end + 2,
                content_range: 1..end + 1,
            });
        }
    }

    if !text.starts_with("~~")
        && let Some(stripped) = text.strip_prefix('~')
        && let Some(end) = stripped.find('~')
    {
        let inner = &stripped[..end];
        if is_valid_short_inline_extent(inner) {
            return Some(ExtendedInlineMatch {
                kind: ExtendedInlineKind::Subscript,
                source_range: 0..end + 2,
                content_range: 1..end + 1,
            });
        }
    }

    None
}

fn push_extended_text(segments: &mut Vec<ExtendedInlineSegment>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(ExtendedInlineSegment::Text(previous)) = segments.last_mut() {
        previous.push_str(text);
    } else {
        segments.push(ExtendedInlineSegment::Text(text.to_string()));
    }
}

fn is_valid_short_inline_extent(text: &str) -> bool {
    !text.trim().is_empty() && !text.contains('\n') && text.chars().count() <= 80
}

fn consume_autolink(text: &str) -> Option<(&str, usize)> {
    let prefix = ["https://", "http://", "www."]
        .into_iter()
        .find(|prefix| text.starts_with(prefix))?;
    if !is_autolink_boundary(text, 0) {
        return None;
    }

    let mut end = prefix.len();
    for (relative, ch) in text[prefix.len()..].char_indices() {
        if ch.is_whitespace() || matches!(ch, '<' | '"' | '\'') {
            break;
        }
        end = prefix.len() + relative + ch.len_utf8();
    }

    while end > prefix.len() {
        let Some(ch) = text[..end].chars().next_back() else {
            break;
        };
        if matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']') {
            end -= ch.len_utf8();
        } else {
            break;
        }
    }

    let url = &text[..end];
    (url.contains('.') && end > prefix.len()).then_some((url, end))
}

fn is_autolink_boundary(text: &str, start: usize) -> bool {
    if start == 0 {
        return true;
    }
    text[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '(' | '[' | '{'))
}

fn emoji_for_shortcode(shortcode: &str) -> Option<&'static str> {
    if shortcode.is_empty()
        || shortcode.len() > 32
        || !shortcode.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '+')
        })
    {
        return None;
    }

    match shortcode {
        "smile" | "slightly_smiling_face" => Some("🙂"),
        "heart" => Some("❤️"),
        "+1" | "thumbsup" => Some("👍"),
        "-1" | "thumbsdown" => Some("👎"),
        "check" | "white_check_mark" => Some("✅"),
        "x" => Some("❌"),
        "warning" => Some("⚠️"),
        "bulb" | "idea" => Some("💡"),
        "rocket" => Some("🚀"),
        "fire" => Some("🔥"),
        "star" => Some("⭐"),
        "book" => Some("📘"),
        "memo" => Some("📝"),
        "bug" => Some("🐛"),
        "sparkles" => Some("✨"),
        _ => None,
    }
}

pub(crate) fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_GFM
}

/// Visual Edit needs byte-identical visible text so every rendered character
/// maps back to authored source. Preview/Read/export keep smart punctuation.
pub(crate) fn visual_markdown_options() -> Options {
    let mut options = markdown_options();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    options
}

pub(crate) fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

pub(crate) fn gfm_alert_kind(kind: BlockQuoteKind) -> crate::AlertKind {
    match kind {
        BlockQuoteKind::Note => crate::AlertKind::Note,
        BlockQuoteKind::Tip => crate::AlertKind::Tip,
        BlockQuoteKind::Important => crate::AlertKind::Important,
        BlockQuoteKind::Warning => crate::AlertKind::Warning,
        BlockQuoteKind::Caution => crate::AlertKind::Caution,
    }
}

pub(crate) fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod extended_inline_range_tests {
    use super::{ExtendedInlineKind, extended_inline_matches, render_extended_inline_plain};

    #[test]
    fn matches_nested_utf8_extended_inline_ranges() {
        let source = "==高^2^亮== and H~2~O";
        let matches = extended_inline_matches(source);
        let found = matches
            .iter()
            .map(|item| {
                (
                    item.kind,
                    &source[item.source_range.clone()],
                    &source[item.content_range.clone()],
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            found,
            vec![
                (ExtendedInlineKind::Highlight, "==高^2^亮==", "高^2^亮"),
                (ExtendedInlineKind::Superscript, "^2^", "2"),
                (ExtendedInlineKind::Subscript, "~2~", "2"),
            ]
        );
        assert!(matches.iter().all(|item| {
            source.is_char_boundary(item.source_range.start)
                && source.is_char_boundary(item.source_range.end)
                && source.is_char_boundary(item.content_range.start)
                && source.is_char_boundary(item.content_range.end)
        }));
        assert_eq!(render_extended_inline_plain(source), "高2亮 and H2O");
    }

    #[test]
    fn rejects_invalid_and_strikethrough_extended_delimiters() {
        for source in [
            "== ==",
            "^ ^",
            "~ ~",
            "^line\nbreak^",
            "~~strikethrough~~",
            "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^",
        ] {
            assert!(
                extended_inline_matches(source).is_empty(),
                "unexpected match for {source:?}"
            );
        }
        assert_eq!(
            render_extended_inline_plain("~~strikethrough~~"),
            "~~strikethrough~~"
        );
    }

    #[test]
    fn preserves_adjacent_subscript_constructs() {
        let source = "~a~~b~";
        let matches = extended_inline_matches(source);
        assert_eq!(matches.len(), 2);
        assert!(
            matches
                .iter()
                .all(|item| item.kind == ExtendedInlineKind::Subscript)
        );
        assert_eq!(
            matches
                .iter()
                .map(|item| &source[item.source_range.clone()])
                .collect::<Vec<_>>(),
            vec!["~a~", "~b~"]
        );
        assert_eq!(render_extended_inline_plain(source), "ab");
    }
}

#[cfg(test)]
mod inline_html_image_tests {
    use super::parse_inline_html_image;

    #[test]
    fn recognizes_exact_img_tags_with_attrs() {
        let image = parse_inline_html_image(r#"<img src="a.png" alt="A" title="T">"#).unwrap();
        assert_eq!(image.url, "a.png");
        assert_eq!(image.alt, "A");
        assert_eq!(image.title.as_deref(), Some("T"));

        let image = parse_inline_html_image("<img src=b.png />").unwrap();
        assert_eq!(image.url, "b.png");
        assert_eq!(image.alt, "");
        assert!(image.title.is_none());

        let image = parse_inline_html_image(" <IMG SRC='c.png' ALT='C'> ").unwrap();
        assert_eq!(image.url, "c.png");
        assert_eq!(image.alt, "C");
        assert!(image.title.is_none());

        let image = parse_inline_html_image(r#"<img src="a&amp;b.png">"#).unwrap();
        assert_eq!(image.url, "a&b.png");

        let image = parse_inline_html_image(
            r#"<img src="assets/markion-logo.svg" alt="Markion logo" width="128" height="128">"#,
        )
        .unwrap();
        assert_eq!(image.width, Some(crate::model::HtmlImgLength::Px(128)));
        assert_eq!(image.height, Some(crate::model::HtmlImgLength::Px(128)));
    }

    #[test]
    fn rejects_non_exact_forms() {
        assert!(parse_inline_html_image("<img>").is_none(), "missing src");
        assert!(
            parse_inline_html_image(r#"<img src="">"#).is_none(),
            "empty src"
        );
        assert!(parse_inline_html_image("</img>").is_none(), "closing tag");
        assert!(
            parse_inline_html_image(r#"<a href="x">"#).is_none(),
            "non-img tag"
        );
        assert!(
            parse_inline_html_image(r#"<img src="a.png"><img src="b.png">"#).is_none(),
            "two tags in one slice"
        );
        assert!(
            parse_inline_html_image(r#"<img alt="a > b" src="x.png">"#).is_none(),
            "inner angle brackets stay conservative"
        );
        assert!(parse_inline_html_image("text").is_none());
        assert!(parse_inline_html_image("<!-- comment -->").is_none());
        assert!(parse_inline_html_image("<img").is_none(), "partial tag");
    }
}

#[cfg(test)]
mod inline_html_style_tag_tests {
    use super::{InlineHtmlStyleKind, InlineHtmlStyleTag, parse_inline_html_style_tag};

    #[test]
    fn recognizes_supported_style_pairs() {
        use InlineHtmlStyleKind as K;
        use InlineHtmlStyleTag as T;
        let cases = [
            ("<em>", T::Open { kind: K::Emphasis }),
            ("<i>", T::Open { kind: K::Emphasis }),
            ("</EM>", T::Close { kind: K::Emphasis }),
            ("<strong>", T::Open { kind: K::Strong }),
            ("<B>", T::Open { kind: K::Strong }),
            ("</b>", T::Close { kind: K::Strong }),
            (
                "<s>",
                T::Open {
                    kind: K::Strikethrough,
                },
            ),
            (
                "<del>",
                T::Open {
                    kind: K::Strikethrough,
                },
            ),
            (
                "</strike>",
                T::Close {
                    kind: K::Strikethrough,
                },
            ),
            ("<code>", T::Open { kind: K::Code }),
            ("</code>", T::Close { kind: K::Code }),
            ("<mark>", T::Open { kind: K::Highlight }),
            ("<sub>", T::Open { kind: K::Subscript }),
            (
                "</sup>",
                T::Close {
                    kind: K::Superscript,
                },
            ),
            ("<br>", T::LineBreak),
            ("<br/>", T::LineBreak),
            ("<br />", T::LineBreak),
            ("<BR>", T::LineBreak),
            ("<em class=\"x\">", T::Open { kind: K::Emphasis }),
            ("</em class=\"x\">", T::Close { kind: K::Emphasis }),
            ("<br class=\"clear\">", T::LineBreak),
            ("<br id=\"b\" class=\"clear\">", T::LineBreak),
            ("<code class=\"l\">", T::Open { kind: K::Code }),
        ];
        for (source, expected) in cases {
            assert_eq!(parse_inline_html_style_tag(source), Some(expected));
        }
    }

    #[test]
    fn rejects_unsupported_forms() {
        let rejected = [
            "<a>",
            "</a>",
            "<u>",
            "<span>",
            "<em title>",
            "<em/>",
            "</br>",
            "text",
            "<em",
            "<em>x</em>",
            "<!-- c -->",
            "<img src=\"a.png\">",
        ];
        for source in rejected {
            assert!(parse_inline_html_style_tag(source).is_none(), "{source:?}");
        }
    }
}

#[cfg(test)]
mod html_table_tests {
    use super::{HtmlPreviewPart, html_preview_parts, parse_html_table_grid};

    /// Returns `(row_index, col_index, text, colspan, rowspan, is_header)` for
    /// every non-spacer cell in the grid, in document order.
    fn grid_cells(grid: &super::HtmlTableGrid) -> Vec<(usize, usize, String, usize, usize, bool)> {
        let mut out = Vec::new();
        for (row_index, row) in grid.rows.iter().enumerate() {
            let mut col = 0usize;
            for cell in row {
                if cell.is_spacer {
                    col += cell.colspan;
                    continue;
                }
                out.push((
                    row_index,
                    col,
                    cell.content.text.clone(),
                    cell.colspan,
                    cell.rowspan,
                    cell.is_header,
                ));
                col += cell.colspan;
            }
        }
        out
    }

    #[test]
    fn rowspan_three_places_cell_across_rows() {
        // The exact table from the bug report: a `12 V` cell spans three rows.
        let html = "<table>\n\
<tr><th>电源接口</th><th>峰值电流</th><th>最大持续时间</th></tr>\n\
<tr><td rowspan=\"3\">12 V</td><td>20 A</td><td>200 us</td></tr>\n\
<tr><td>17 A</td><td>1 ms</td></tr>\n\
<tr><td>13 A</td><td>5 ms</td></tr>\n\
</table>";
        let grid = parse_html_table_grid(html).expect("table should parse");
        assert!(grid.has_rowspan);
        assert_eq!(grid.columns, 3);
        assert_eq!(grid.rows.len(), 4);

        let cells = grid_cells(&grid);
        // The `12 V` cell is a header-spanning body cell at row 1, column 0,
        // colspan 1, rowspan 3.
        let twelve = cells
            .iter()
            .find(|c| c.2 == "12 V")
            .unwrap_or_else(|| panic!("12 V cell missing; got {cells:?}"));
        assert_eq!(twelve.0, 1, "12 V is in row 1");
        assert_eq!(twelve.1, 0, "12 V is in column 0");
        assert_eq!(twelve.3, 1, "12 V colspan is 1");
        assert_eq!(twelve.4, 3, "12 V rowspan is 3");

        // The cells in rows 2 and 3 are shifted to columns 1 and 2.
        let seventeen = cells.iter().find(|c| c.2 == "17 A").unwrap();
        assert_eq!(seventeen.0, 2);
        assert_eq!(seventeen.1, 1, "17 A shifts to column 1");
        let one_ms = cells.iter().find(|c| c.2 == "1 ms").unwrap();
        assert_eq!(one_ms.0, 2);
        assert_eq!(one_ms.1, 2, "1 ms shifts to column 2");
        let thirteen = cells.iter().find(|c| c.2 == "13 A").unwrap();
        assert_eq!(thirteen.0, 3);
        assert_eq!(thirteen.1, 1);
    }

    #[test]
    fn colspan_widens_cell_across_columns() {
        let html = "<table>\
<tr><th>A</th><th>B</th><th>C</th></tr>\
<tr><td colspan=\"2\">wide</td><td>x</td></tr>\
</table>";
        let grid = parse_html_table_grid(html).expect("table should parse");
        assert_eq!(grid.columns, 3);
        let cells = grid_cells(&grid);
        let wide = cells.iter().find(|c| c.2 == "wide").unwrap();
        assert_eq!(wide.0, 1);
        assert_eq!(wide.1, 0);
        assert_eq!(wide.3, 2, "wide cell colspan is 2");
        // The trailing cell lands in column 2.
        let x = cells.iter().find(|c| c.2 == "x").unwrap();
        assert_eq!(x.1, 2);
    }

    #[test]
    fn combined_rowspan_and_colspan() {
        let html = "<table>\
<tr><th>A</th><th>B</th><th>C</th></tr>\
<tr><td rowspan=\"2\" colspan=\"2\">block</td><td>1</td></tr>\
<tr><td>2</td></tr>\
</table>";
        let grid = parse_html_table_grid(html).expect("table should parse");
        assert!(grid.has_rowspan);
        let cells = grid_cells(&grid);
        let block = cells.iter().find(|c| c.2 == "block").unwrap();
        assert_eq!(block.1, 0);
        assert_eq!(block.3, 2, "block colspan is 2");
        assert_eq!(block.4, 2, "block rowspan is 2");
        // Row 2 only has one real cell, which lands in column 2 (columns 0/1
        // are still held by the rowspan from above).
        let two = cells.iter().find(|c| c.2 == "2").unwrap();
        assert_eq!(two.0, 2);
        assert_eq!(two.1, 2);
    }

    #[test]
    fn invalid_span_values_fall_back_to_one() {
        let html = "<table>\
<tr><td rowspan=\"zero\">a</td><td colspan=\"-1\">b</td></tr>\
<tr><td>c</td><td>d</td></tr>\
</table>";
        let grid = parse_html_table_grid(html).expect("table should parse");
        assert!(!grid.has_rowspan, "zero/negative spans collapse to 1");
        let cells = grid_cells(&grid);
        assert_eq!(cells.len(), 4);
        // Every cell is a single-cell footprint.
        assert!(cells.iter().all(|c| c.3 == 1 && c.4 == 1));
    }

    #[test]
    fn non_numeric_span_falls_back_to_one() {
        let html = "<table><tr><td rowspan=\"wide\">a</td><td>b</td></tr></table>";
        let grid = parse_html_table_grid(html).expect("table should parse");
        assert!(!grid.has_rowspan);
    }

    #[test]
    fn malformed_table_returns_none_and_falls_back_to_text() {
        // Not a table at all.
        assert!(parse_html_table_grid("<p>hello</p>").is_none());
        // Nested table is out of scope.
        assert!(parse_html_table_grid("<table><table></table></table>").is_none());
        // An empty table with no rows cannot become a grid.
        assert!(parse_html_table_grid("<table></table>").is_none());

        // A truncated-but-structured table (one open cell) is tolerated and
        // rendered as a single-cell table rather than crashing.
        let parts = html_preview_parts("<table><tr><td>oops");
        assert_eq!(parts.len(), 1, "got {parts:?}");
        assert!(matches!(parts[0], HtmlPreviewPart::Table { .. }));

        // Non-table HTML always goes through the flattener.
        let parts = html_preview_parts("<div>still text</div>");
        assert!(
            parts
                .iter()
                .any(|p| matches!(p, HtmlPreviewPart::Text { .. }))
        );
    }

    #[test]
    fn unclosed_cell_tags_are_tolerated() {
        // No closing `</td>`/`</tr>` — browsers auto-close; we should too.
        let html = "<table><tr><th>H1<th>H2<tr><td>a<td>b</table>";
        let grid = parse_html_table_grid(html).expect("table should parse");
        let texts: Vec<&str> = grid
            .rows
            .iter()
            .flat_map(|row| row.iter().filter(|c| !c.is_spacer))
            .map(|c| c.content.text.as_str())
            .collect();
        assert_eq!(texts, vec!["H1", "H2", "a", "b"]);
    }

    #[test]
    fn inline_formatting_resolves_inside_cells() {
        let html = "<table><tr><td><strong>bold</strong> and <em>it</em></td></tr></table>";
        let grid = parse_html_table_grid(html).expect("table should parse");
        let cell = grid.rows[0]
            .iter()
            .find(|c| !c.is_spacer)
            .expect("a real cell");
        assert_eq!(cell.content.text, "bold and it");
        // The "bold" run carries the bold style.
        let bold_span = cell
            .content
            .spans
            .iter()
            .find(|s| s.text.contains("bold"))
            .expect("bold span");
        assert!(bold_span.style.bold, "bold span should be bold");
        let italic_span = cell
            .content
            .spans
            .iter()
            .find(|s| s.text.contains("it"))
            .expect("italic span");
        assert!(italic_span.style.italic, "it span should be italic");
    }

    #[test]
    fn html_preview_parts_routes_table_to_table_part() {
        let html = "<table><tr><th>X</th></tr><tr><td>1</td></tr></table>";
        let parts = html_preview_parts(html);
        assert_eq!(parts.len(), 1, "one table part expected, got {parts:?}");
        match &parts[0] {
            HtmlPreviewPart::Table { grid } => {
                assert_eq!(grid.columns, 1);
                assert_eq!(grid.rows.len(), 2);
            }
            other => panic!("expected Table part, got {other:?}"),
        }
    }

    #[test]
    fn html_preview_parts_finds_table_inside_wrapper() {
        let html = "<div align=\"center\"><table><tr><td>A</td></tr></table></div>";
        let parts = html_preview_parts(html);
        assert!(
            parts
                .iter()
                .any(|part| matches!(part, HtmlPreviewPart::Table { .. })),
            "wrapped table should still be a grid, got {parts:?}"
        );
    }

    #[test]
    fn html_preview_parts_keeps_caption_after_table() {
        let html = "<table><tr><td>A</td></tr></table><p>caption</p>";
        let parts = html_preview_parts(html);
        assert!(matches!(parts.first(), Some(HtmlPreviewPart::Table { .. })));
        assert!(
            parts.iter().any(|part| matches!(
                part,
                HtmlPreviewPart::Text { text, .. } if text.text.contains("caption")
            )),
            "suffix caption must remain, got {parts:?}"
        );
    }

    #[test]
    fn html_preview_parts_honors_img_pixel_size() {
        let parts = html_preview_parts(
            "<p align=\"center\"><img src=\"assets/markion-logo.svg\" alt=\"Markion logo\" width=\"128\" height=\"128\"></p>",
        );
        match &parts[0] {
            HtmlPreviewPart::Image {
                url,
                alt,
                centered,
                width,
                height,
                ..
            } => {
                assert_eq!(url, "assets/markion-logo.svg");
                assert_eq!(alt, "Markion logo");
                assert!(*centered);
                assert_eq!(*width, Some(crate::model::HtmlImgLength::Px(128)));
                assert_eq!(*height, Some(crate::model::HtmlImgLength::Px(128)));
            }
            other => panic!("expected sized image, got {other:?}"),
        }
    }

    #[test]
    fn non_table_html_still_uses_flattener() {
        let parts = html_preview_parts("<p>hello <strong>world</strong></p>");
        assert!(
            parts
                .iter()
                .any(|p| matches!(p, HtmlPreviewPart::Text { .. }))
        );
        assert!(
            !parts
                .iter()
                .any(|p| matches!(p, HtmlPreviewPart::Table { .. }))
        );
    }

    #[test]
    fn header_cells_are_marked() {
        let html = "<table><tr><th>H</th></tr><tr><td>b</td></tr></table>";
        let grid = parse_html_table_grid(html).expect("table should parse");
        let header = grid.rows[0]
            .iter()
            .find(|c| !c.is_spacer)
            .expect("header cell");
        assert!(header.is_header);
        let body = grid.rows[1]
            .iter()
            .find(|c| !c.is_spacer)
            .expect("body cell");
        assert!(!body.is_header);
    }

    #[test]
    fn html_preview_parts_headings_lists_pre_align_underline_and_color() {
        let parts = html_preview_parts(
            "<h1>Title</h1><ul><li>one</li></ul><ol><li>two</li></ol>\
<pre>  spaced\nline</pre><p align=\"right\"><u>under</u></p>\
<span style=\"color:#cc0000\">red</span>\
<span style=\"color:rgb(0, 128, 0)\">green</span>\
<span style=\"color:url(evil)\">ignored</span>",
        );
        let heading = parts.iter().find_map(|part| match part {
            HtmlPreviewPart::Text {
                text,
                heading_level,
                ..
            } if text.text.contains("Title") => Some(*heading_level),
            _ => None,
        });
        assert_eq!(heading, Some(Some(1)));

        let disc = parts.iter().find_map(|part| match part {
            HtmlPreviewPart::Text {
                text, list_marker, ..
            } if text.text.contains("one") => Some(*list_marker),
            _ => None,
        });
        assert_eq!(disc, Some(Some(super::HtmlListMarker::Disc)));

        let numbered = parts.iter().find_map(|part| match part {
            HtmlPreviewPart::Text {
                text, list_marker, ..
            } if text.text.contains("two") => Some(*list_marker),
            _ => None,
        });
        assert_eq!(numbered, Some(Some(super::HtmlListMarker::Decimal(1))));

        let pre = parts
            .iter()
            .find_map(|part| match part {
                HtmlPreviewPart::Text { text, pre, .. } if *pre => Some(text.text.as_str()),
                _ => None,
            })
            .expect("pre part");
        assert!(
            pre.contains("  spaced"),
            "pre keeps leading spaces: {pre:?}"
        );
        assert!(pre.contains('\n') || pre.contains("spaced"), "{pre:?}");

        let right = parts.iter().find(|part| match part {
            HtmlPreviewPart::Text { text, .. } => text.text.contains("under"),
            _ => false,
        });
        match right {
            Some(HtmlPreviewPart::Text { text, align, .. }) => {
                assert_eq!(*align, super::HtmlAlign::End);
                assert!(
                    text.spans.iter().any(|span| span.style.underline),
                    "underline flag, got {:?}",
                    text.spans
                );
            }
            other => panic!("expected right-aligned underlined text, got {other:?}"),
        }

        let red = parts.iter().find_map(|part| match part {
            HtmlPreviewPart::Text { text, .. } => text
                .spans
                .iter()
                .find(|span| span.text.contains("red"))
                .map(|span| span.style.color),
            _ => None,
        });
        assert_eq!(red, Some(Some(0xcc0000)));

        let green = parts.iter().find_map(|part| match part {
            HtmlPreviewPart::Text { text, .. } => text
                .spans
                .iter()
                .find(|span| span.text.contains("green"))
                .map(|span| span.style.color),
            _ => None,
        });
        assert_eq!(green, Some(Some(0x008000)));

        let ignored = parts.iter().find_map(|part| match part {
            HtmlPreviewPart::Text { text, .. } => text
                .spans
                .iter()
                .find(|span| span.text.contains("ignored"))
                .map(|span| span.style.color),
            _ => None,
        });
        assert_eq!(ignored, Some(None));
    }

    #[test]
    fn html_table_cell_parses_image_and_link() {
        let html = "<table><tr>\
<td><img src=\"pic.png\" alt=\"P\" width=\"32\"></td>\
<td><a href=\"https://example.com\">go</a></td>\
</tr></table>";
        let grid = parse_html_table_grid(html).expect("table");
        let image_cell = grid.rows[0]
            .iter()
            .find(|cell| cell.image.is_some())
            .expect("image cell");
        let image = image_cell.image.as_ref().unwrap();
        assert_eq!(image.url, "pic.png");
        assert_eq!(image.alt, "P");
        assert_eq!(image.width, Some(crate::model::HtmlImgLength::Px(32)));

        let link_cell = grid.rows[0]
            .iter()
            .find(|cell| cell.content.text.contains("go"))
            .expect("link cell");
        assert!(
            link_cell.content.spans.iter().any(
                |span| span.text == "go" && span.link.as_deref() == Some("https://example.com")
            )
        );
    }
}
