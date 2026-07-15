//! Markdown → structured preview parsing helpers.
//!
//! These functions turn `pulldown-cmark` event streams into [`PreviewBlock`]s,
//! resolve inline styling, and handle extended inline syntax
//! (`==highlight==`, `^sup^`, `~sub~`, `:emoji:`, bare autolinks). The main
//! `compute_preview_blocks` driver lives on `MarkdownDocument` and calls into
//! this module; everything here is pure / stateless.

use std::ops::Range;

use pulldown_cmark::{HeadingLevel, Options};

use crate::escape::escape_html_attribute;
use crate::model::{InlineSpan, InlineStyle, PreviewBlock, RichText};
use crate::table::TableDraft;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlPreviewPart {
    Text {
        text: RichText,
        centered: bool,
    },
    Image {
        alt: String,
        url: String,
        title: Option<String>,
        centered: bool,
    },
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
        if !text.is_empty() || item.checked.is_some() {
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
}

/// Routes one run of inline text to whichever block draft is currently open,
/// mirroring the old plain-text routing priority. Styled targets receive
/// spans; image alts, code bodies, and table cells stay plain text.
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
        table.current_cell.push_str(text);
        return;
    }

    let spans = if let Some((_, spans, _)) = heading.as_mut() {
        spans
    } else if let Some(item) = list_item.as_mut() {
        &mut item.spans
    } else if quote_depth > 0 {
        quote
    } else if let Some((paragraph, _)) = paragraph.as_mut() {
        paragraph
    } else {
        return;
    };

    if parse_extended {
        append_extended_text(spans, text, style, link);
    } else {
        append_span(spans, text, style, link);
    }
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
    if let Some(last) = spans.last_mut() {
        if last.style == style && last.link.as_deref() == link {
            last.text.push_str(text);
            return;
        }
    }
    spans.push(InlineSpan {
        text: text.to_string(),
        style,
        link: link.map(str::to_string),
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
            append_span(&mut merged, &span.text, span.style, span.link.as_deref());
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
        PreviewBlock::Heading { text, .. }
        | PreviewBlock::Paragraph { text, .. }
        | PreviewBlock::BlockQuote { text, .. } => {
            if !text.is_empty() {
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

pub fn html_preview_parts(html: &str) -> Vec<HtmlPreviewPart> {
    HtmlPreviewBuilder::new(html).finish()
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
        }
    }

    fn finish(mut self) -> Vec<HtmlPreviewPart> {
        while self.index < self.html.len() {
            if self.html[self.index..].starts_with('<') {
                if let Some(tag_end) = find_html_tag_end(self.html, self.index) {
                    let tag = &self.html[self.index..tag_end];
                    self.handle_tag(tag);
                    self.index = tag_end;
                    continue;
                }
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
            if !parsed.closing {
                if let Some(end) = find_html_closing_tag(self.html, self.index, &parsed.name) {
                    self.index = end;
                }
            }
            return;
        }

        match parsed.name.as_str() {
            "br" => self.push_line_break(),
            "p" | "div" | "section" | "article" | "header" | "footer" | "li" | "tr" | "table"
            | "blockquote" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.pending_space = None;
                if parsed.closing {
                    if self.centered_depth > 0 {
                        self.centered_depth -= 1;
                    }
                    self.push_line_break();
                } else {
                    if self.has_text() {
                        self.push_line_break();
                    }
                    if parsed.is_centered() {
                        self.centered_depth += 1;
                    }
                }
            }
            "strong" | "b" => {
                if parsed.closing {
                    self.bold_depth = self.bold_depth.saturating_sub(1);
                } else if !parsed.self_closing {
                    self.bold_depth += 1;
                }
                self.update_style();
            }
            "em" | "i" => {
                if parsed.closing {
                    self.italic_depth = self.italic_depth.saturating_sub(1);
                } else if !parsed.self_closing {
                    self.italic_depth += 1;
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
                    self.parts.push(HtmlPreviewPart::Image {
                        alt: parsed.attr("alt").unwrap_or_default(),
                        url,
                        title: parsed.attr("title").filter(|title| !title.is_empty()),
                        centered: self.centered_depth > 0 || parsed.is_centered(),
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
    }

    fn push_text(&mut self, text: &str) {
        let decoded = decode_html_entities(text);
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
        self.text_centered |= self.centered_depth > 0;
        append_span(
            &mut self.spans,
            text,
            self.style,
            self.links.last().map(String::as_str),
        );
    }

    fn flush_text(&mut self) {
        self.pending_space = None;
        let text = finish_rich_text(std::mem::take(&mut self.spans));
        if !text.is_empty() {
            self.parts.push(HtmlPreviewPart::Text {
                text,
                centered: self.text_centered,
            });
        }
        self.text_centered = false;
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

    fn is_centered(&self) -> bool {
        self.attr("align")
            .is_some_and(|value| value.eq_ignore_ascii_case("center"))
            || self
                .attr("style")
                .is_some_and(|value| value.to_ascii_lowercase().contains("text-align: center"))
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
        if text[index..].starts_with('&') {
            if let Some(end) = text[index + 1..].find(';') {
                let entity = &text[index + 1..index + 1 + end];
                if let Some(decoded) = decode_html_entity(entity) {
                    output.push(decoded);
                    index += end + 2;
                    continue;
                }
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

        if rest.starts_with("==") {
            if let Some(end) = rest[2..].find("==") {
                let inner = &rest[2..2 + end];
                if !inner.trim().is_empty() {
                    segments.push(ExtendedInlineSegment::Highlight(
                        parse_extended_inline_segments(inner),
                    ));
                    index += end + 4;
                    continue;
                }
            }
        }

        if rest.starts_with('^') {
            if let Some(end) = rest[1..].find('^') {
                let inner = &rest[1..1 + end];
                if is_valid_short_inline_extent(inner) {
                    segments.push(ExtendedInlineSegment::Superscript(
                        parse_extended_inline_segments(inner),
                    ));
                    index += end + 2;
                    continue;
                }
            }
        }

        if rest.starts_with('~') && !rest.starts_with("~~") {
            if let Some(end) = rest[1..].find('~') {
                let inner = &rest[1..1 + end];
                if is_valid_short_inline_extent(inner) {
                    segments.push(ExtendedInlineSegment::Subscript(
                        parse_extended_inline_segments(inner),
                    ));
                    index += end + 2;
                    continue;
                }
            }
        }

        if rest.starts_with(':') {
            if let Some(end) = rest[1..].find(':') {
                let shortcode = &rest[1..1 + end];
                if let Some(emoji) = emoji_for_shortcode(shortcode) {
                    segments.push(ExtendedInlineSegment::Emoji(emoji));
                    index += end + 2;
                    continue;
                }
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
