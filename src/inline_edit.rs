use std::ops::Range;

use pulldown_cmark::{Event, LinkType, Parser, Tag};

use crate::parse::markdown_options;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImagePresentation {
    pub width_percent: u8,
    pub alignment: ImageAlignment,
}

impl Default for ImagePresentation {
    fn default() -> Self {
        Self {
            width_percent: 100,
            alignment: ImageAlignment::Center,
        }
    }
}

pub const IMAGE_WIDTH_PERCENT_MIN: u8 = 10;
pub const IMAGE_WIDTH_PERCENT_MAX: u8 = 100;
const IMAGE_WIDTH_SNAP_PRESETS: [u8; 4] = [25, 50, 75, 100];
const IMAGE_WIDTH_SNAP_DISTANCE: u8 = 3;

pub fn clamp_image_width_percent(percent: u8) -> u8 {
    percent.clamp(IMAGE_WIDTH_PERCENT_MIN, IMAGE_WIDTH_PERCENT_MAX)
}

pub fn snap_image_width_percent(percent: u8) -> u8 {
    let clamped = clamp_image_width_percent(percent);
    for preset in IMAGE_WIDTH_SNAP_PRESETS {
        if clamped.abs_diff(preset) <= IMAGE_WIDTH_SNAP_DISTANCE {
            return preset;
        }
    }
    clamped
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineMarkdownTarget {
    pub source_range: Range<usize>,
    pub label: String,
    pub url: String,
    pub title: Option<String>,
    pub presentation: Option<ImagePresentation>,
}

pub fn inline_link_at(source: &str, offset: usize) -> Option<InlineMarkdownTarget> {
    inline_target_at(source, offset, false)
}

pub fn inline_image_at(source: &str, offset: usize) -> Option<InlineMarkdownTarget> {
    inline_target_at(source, offset, true)
}

/// True when `authored` is one complete single-line Markdown image: inline
/// `![alt](dest)`, reference `![alt][label]`, collapsed `![alt][]`, or
/// shortcut `![alt]`. Multiline and unclosed forms return `false`.
pub fn proven_single_line_image_span(authored: &str) -> bool {
    if !authored.starts_with("![") || authored.contains(['\n', '\r']) {
        return false;
    }
    let Some(label_end) = find_label_end(authored, 2) else {
        return false;
    };
    let after = label_end + 1;
    if after == authored.len() {
        return true;
    }
    match authored.as_bytes().get(after) {
        Some(b'(') => find_unescaped(authored, after + 1, b')')
            .is_some_and(|close| close + 1 == authored.len()),
        Some(b'[') => find_unescaped(authored, after + 1, b']')
            .is_some_and(|close| close + 1 == authored.len()),
        _ => false,
    }
}

/// Byte range of the authored destination inside a complete inline-image span
/// `![label](destination "title")`, relative to the span start. Returns
/// `None` for any span whose destination cannot be located with certainty;
/// callers must skip those conservatively rather than guess.
pub fn authored_image_destination_range(authored: &str) -> Option<Range<usize>> {
    if !authored.starts_with("![") || !authored.ends_with(')') {
        return None;
    }
    let label_end = find_label_end(authored, 2)?;
    if authored.as_bytes().get(label_end + 1) != Some(&b'(') {
        return None;
    }
    let inner_start = label_end + 2;
    let inner = &authored[inner_start..authored.len() - 1];
    let (relative_start, relative_end) = if inner.starts_with('<') {
        let close = find_unescaped(inner, 1, b'>')?;
        (1, close)
    } else {
        let end = inner.find(char::is_whitespace).unwrap_or(inner.len());
        (0, end)
    };
    if relative_start >= relative_end {
        return None;
    }
    Some(relative_start + inner_start..relative_end + inner_start)
}

fn inline_target_at(source: &str, offset: usize, image: bool) -> Option<InlineMarkdownTarget> {
    let offset = offset.min(source.len());
    Parser::new_ext(source, markdown_options())
        .into_offset_iter()
        .find_map(|(event, range)| {
            let (is_image, link_type, destination, title) = match event {
                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    ..
                }) => (false, link_type, dest_url, title),
                Event::Start(Tag::Image {
                    link_type,
                    dest_url,
                    title,
                    ..
                }) => (true, link_type, dest_url, title),
                _ => return None,
            };
            if is_image != image
                || link_type != LinkType::Inline
                || offset < range.start
                || offset > range.end
            {
                return None;
            }
            let authored = source.get(range.clone())?;
            let prefix = if image { "![" } else { "[" };
            if !authored.starts_with(prefix) || !authored.ends_with(')') {
                return None;
            }
            let label_end = find_label_end(authored, prefix.len())?;
            if authored.as_bytes().get(label_end + 1) != Some(&b'(') {
                return None;
            }
            let label = unescape_markdown(&authored[prefix.len()..label_end]);
            let (title, presentation) = if image {
                split_presentation_title(title.as_ref())
            } else {
                ((!title.is_empty()).then(|| title.to_string()), None)
            };
            Some(InlineMarkdownTarget {
                source_range: range,
                label,
                url: destination.to_string(),
                title,
                presentation,
            })
        })
}

pub fn serialize_inline_link(label: &str, url: &str, title: Option<&str>) -> String {
    serialize_target(false, label, url, title, None)
}

pub fn serialize_inline_image(
    alt: &str,
    url: &str,
    title: Option<&str>,
    presentation: Option<ImagePresentation>,
) -> String {
    serialize_target(true, alt, url, title, presentation)
}

fn serialize_target(
    image: bool,
    label: &str,
    url: &str,
    title: Option<&str>,
    presentation: Option<ImagePresentation>,
) -> String {
    let mut rendered = String::new();
    if image {
        rendered.push('!');
    }
    rendered.push('[');
    rendered.push_str(&escape_label(label));
    rendered.push_str("](");
    rendered.push_str(&escape_destination(url));
    let title = compose_title(title, presentation);
    if let Some(title) = title.filter(|title| !title.is_empty()) {
        rendered.push_str(" \"");
        rendered.push_str(&title.replace('\\', "\\\\").replace('"', "\\\""));
        rendered.push('"');
    }
    rendered.push(')');
    rendered
}

fn escape_label(label: &str) -> String {
    label
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

pub(crate) fn escape_destination(url: &str) -> String {
    if url.chars().any(char::is_whitespace) || url.contains(['(', ')', '<', '>']) {
        format!("<{}>", url.replace('\\', "\\\\").replace('>', "\\>"))
    } else {
        url.replace('\\', "\\\\")
    }
}

/// Serializes an inline image while retaining the authored Markdown inside
/// the alt brackets. Used when materializing one resolved reference image so
/// emphasis and other inline formatting are not flattened or redirected
/// through a shared definition.
pub(crate) fn serialize_inline_image_with_authored_alt(
    authored_alt: &str,
    url: &str,
    title: Option<&str>,
) -> String {
    let mut rendered = String::from("![");
    rendered.push_str(authored_alt);
    rendered.push_str("](");
    rendered.push_str(&escape_destination(url));
    if let Some(title) = title.filter(|title| !title.is_empty()) {
        rendered.push_str(" \"");
        rendered.push_str(&title.replace('\\', "\\\\").replace('"', "\\\""));
        rendered.push('"');
    }
    rendered.push(')');
    rendered
}

pub(crate) fn authored_image_alt_source(authored: &str) -> Option<&str> {
    if !authored.starts_with("![") {
        return None;
    }
    let bytes = authored.as_bytes();
    let mut depth = 0usize;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate().skip(2) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        match byte {
            b'[' => depth += 1,
            b']' if depth == 0 => return authored.get(2..index),
            b']' => depth -= 1,
            _ => {}
        }
    }
    None
}

pub(crate) fn unescape_markdown(value: &str) -> String {
    // CommonMark backslash escapes apply only before ASCII punctuation;
    // a backslash before any other character (including Windows path
    // separators before alphanumerics or non-ASCII text) stays literal.
    // This must match pulldown-cmark's destination/label semantics exactly
    // so parsed destinations can be attributed back to authored spans.
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some(next) if next.is_ascii_punctuation() => {
                    out.push(*next);
                    chars.next();
                }
                _ => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn find_label_end(source: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut escaped = false;
    for (index, byte) in source.as_bytes().iter().copied().enumerate().skip(start) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        match byte {
            b'[' => depth += 1,
            b']' if depth == 0 => return Some(index),
            b']' => depth -= 1,
            _ => {}
        }
    }
    None
}

pub(crate) fn find_unescaped(source: &str, start: usize, needle: u8) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate().skip(start) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if byte == needle {
            return Some(index);
        }
    }
    None
}

fn compose_title(title: Option<&str>, presentation: Option<ImagePresentation>) -> Option<String> {
    let title = title.unwrap_or_default().trim();
    match presentation {
        Some(presentation) => {
            let alignment = match presentation.alignment {
                ImageAlignment::Left => "left",
                ImageAlignment::Center => "center",
                ImageAlignment::Right => "right",
            };
            let metadata = format!(
                "{{width={} align={alignment}}}",
                presentation
                    .width_percent
                    .clamp(IMAGE_WIDTH_PERCENT_MIN, IMAGE_WIDTH_PERCENT_MAX)
            );
            Some(if title.is_empty() {
                metadata
            } else {
                format!("{title} {metadata}")
            })
        }
        None => (!title.is_empty()).then(|| title.to_string()),
    }
}

fn split_presentation_title(title: &str) -> (Option<String>, Option<ImagePresentation>) {
    let trimmed = title.trim();
    let Some(open) = trimmed.rfind("{width=") else {
        return ((!trimmed.is_empty()).then(|| trimmed.to_string()), None);
    };
    let metadata = &trimmed[open..];
    if !metadata.ends_with('}') {
        return ((!trimmed.is_empty()).then(|| trimmed.to_string()), None);
    }
    let body = &metadata[1..metadata.len() - 1];
    let mut width = None;
    let mut alignment = None;
    for part in body.split_whitespace() {
        if let Some(value) = part.strip_prefix("width=") {
            width = value.parse::<u8>().ok().filter(|value| {
                (IMAGE_WIDTH_PERCENT_MIN..=IMAGE_WIDTH_PERCENT_MAX).contains(value)
            });
        } else if let Some(value) = part.strip_prefix("align=") {
            alignment = match value {
                "left" => Some(ImageAlignment::Left),
                "center" => Some(ImageAlignment::Center),
                "right" => Some(ImageAlignment::Right),
                _ => None,
            };
        }
    }
    let Some(width) = width else {
        return ((!trimmed.is_empty()).then(|| trimmed.to_string()), None);
    };
    let Some(alignment) = alignment else {
        return ((!trimmed.is_empty()).then(|| trimmed.to_string()), None);
    };
    let caption = trimmed[..open].trim_end();
    (
        (!caption.is_empty()).then(|| caption.to_string()),
        Some(ImagePresentation {
            width_percent: width,
            alignment,
        }),
    )
}

pub(crate) fn image_presentation_from_title(title: Option<&str>) -> Option<ImagePresentation> {
    title.and_then(|title| split_presentation_title(title).1)
}

/// A simple HTML anchor: `<a>` with only `href` and an optional `title`, text
/// content, and no nested tags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleHtmlAnchor {
    pub source_range: Range<usize>,
    pub label: String,
    pub url: String,
    pub title: Option<String>,
}

pub fn simple_html_anchor_at(source: &str, offset: usize) -> Option<SimpleHtmlAnchor> {
    let offset = offset.min(source.len());
    let lower = source.to_ascii_lowercase();
    let mut search_from = 0;
    let mut found = None;
    while let Some(rel) = lower[search_from..].find("<a") {
        let start = search_from + rel;
        let after_name = start + 2;
        let next = source[after_name..].chars().next();
        if !matches!(next, Some(ch) if ch.is_ascii_whitespace() || ch == '>' || ch == '/') {
            search_from = after_name.max(search_from + 1);
            continue;
        }
        let Some(gt) = source[after_name..].find('>') else {
            break;
        };
        let tag_end = after_name + gt + 1;
        if source[start..tag_end].ends_with("/>") {
            search_from = tag_end;
            continue;
        }
        let Some(close_rel) = lower[tag_end..].find("</a>") else {
            search_from = tag_end;
            continue;
        };
        let content_end = tag_end + close_rel;
        let end = content_end + "</a>".len();
        let content = &source[tag_end..content_end];
        if let Some(anchor) = parse_simple_anchor(source, start, tag_end, content, end) {
            if offset >= anchor.source_range.start && offset < anchor.source_range.end {
                found = Some(anchor);
            }
        }
        search_from = start + 2;
    }
    found
}

pub fn serialize_html_anchor(label: &str, url: &str, title: Option<&str>) -> String {
    let mut rendered = String::from("<a href=\"");
    push_escaped_attr(&mut rendered, url);
    rendered.push('"');
    if let Some(title) = title.filter(|title| !title.is_empty()) {
        rendered.push_str(" title=\"");
        push_escaped_attr(&mut rendered, title);
        rendered.push('"');
    }
    rendered.push('>');
    push_escaped_text(&mut rendered, label);
    rendered.push_str("</a>");
    rendered
}

fn parse_simple_anchor(
    source: &str,
    start: usize,
    tag_end: usize,
    content: &str,
    end: usize,
) -> Option<SimpleHtmlAnchor> {
    if content.contains('<') {
        return None;
    }
    let open = &source[start + 2..tag_end - 1];
    let mut href = None;
    let mut title = None;
    let mut index = 0;
    let bytes = open.as_bytes();
    while index < open.len() {
        while index < open.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= open.len() {
            break;
        }
        if bytes[index] == b'/' {
            return None;
        }
        let name_start = index;
        while index < open.len() && is_attr_name(bytes[index]) {
            index += 1;
        }
        if name_start == index {
            return None;
        }
        let name = open[name_start..index].to_ascii_lowercase();
        while index < open.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= open.len() || bytes[index] != b'=' {
            return None;
        }
        index += 1;
        while index < open.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= open.len() {
            return None;
        }
        let quote = bytes[index];
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        index += 1;
        let value_start = index;
        while index < open.len() && bytes[index] != quote {
            index += 1;
        }
        if index >= open.len() {
            return None;
        }
        let value = open[value_start..index].to_string();
        index += 1;
        match name.as_str() {
            "href" if href.is_none() => href = Some(value),
            "title" if title.is_none() => title = Some(value),
            _ => return None,
        }
    }
    Some(SimpleHtmlAnchor {
        source_range: start..end,
        label: content.to_string(),
        url: href?,
        title,
    })
}

fn is_attr_name(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b':' || byte == b'_'
}

fn push_escaped_attr(out: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            _ => out.push(ch),
        }
    }
}

fn push_escaped_text(out: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_inline_link_roundtrips_utf8_and_escaped_label() {
        let source = "前 [标\\]签](<docs/a b.md> \"标题\") 后";
        let target = inline_link_at(source, source.find("docs").unwrap()).unwrap();
        assert_eq!(target.label, "标]签");
        assert_eq!(target.url, "docs/a b.md");
        assert_eq!(target.title.as_deref(), Some("标题"));
        assert_eq!(
            serialize_inline_link(&target.label, &target.url, target.title.as_deref()),
            "[标\\]签](<docs/a b.md> \"标题\")"
        );
    }

    #[test]
    fn issue_82_inline_link_supports_balanced_brackets_in_label() {
        let source = "Read [Minutes [draft]](old.md \"Notes\") today.";
        let target = inline_link_at(source, source.find("nutes").unwrap()).unwrap();
        assert_eq!(target.label, "Minutes [draft]");
        assert_eq!(target.url, "old.md");
        assert_eq!(target.title.as_deref(), Some("Notes"));
        assert_eq!(
            &source[target.source_range.clone()],
            "[Minutes [draft]](old.md \"Notes\")"
        );

        let replacement = serialize_inline_link(&target.label, "new.md", target.title.as_deref());
        let mut revised = source.to_string();
        revised.replace_range(target.source_range, &replacement);
        assert_eq!(
            revised,
            "Read [Minutes \\[draft\\]](new.md \"Notes\") today."
        );
        let reparsed = inline_link_at(&revised, revised.find("nutes").unwrap()).unwrap();
        assert_eq!(reparsed.label, "Minutes [draft]");
        assert_eq!(reparsed.url, "new.md");
        assert!(!revised.contains("old.md"));
    }

    #[test]
    fn reference_links_are_conservative() {
        assert!(inline_link_at("[label][target]", 3).is_none());
        assert!(inline_link_at("[label][]", 3).is_none());
    }

    #[test]
    fn unescape_follows_commonmark_backslash_rules() {
        // Escapes apply only before ASCII punctuation.
        assert_eq!(unescape_markdown("a\\*b"), "a*b");
        assert_eq!(unescape_markdown("\\\\\\*"), "\\*");
        // A backslash before a non-punctuation character stays literal, which
        // is exactly how pulldown parses Windows-style path destinations.
        assert_eq!(
            unescape_markdown("..\\shared\\图片.png"),
            "..\\shared\\图片.png"
        );
        assert_eq!(unescape_markdown("a\\.b"), "a.b");
    }

    #[test]
    fn image_presentation_is_valid_title_metadata() {
        let presentation = ImagePresentation {
            width_percent: 50,
            alignment: ImageAlignment::Right,
        };
        let rendered = serialize_inline_image(
            "A]lt",
            "note.assets/a b.png",
            Some("Caption"),
            Some(presentation),
        );
        assert_eq!(
            rendered,
            "![A\\]lt](<note.assets/a b.png> \"Caption {width=50 align=right}\")"
        );
        let parsed = inline_image_at(&rendered, 5).unwrap();
        assert_eq!(parsed.presentation, Some(presentation));
        assert_eq!(parsed.title.as_deref(), Some("Caption"));
    }

    #[test]
    fn image_width_accepts_integer_percents_from_10_to_100() {
        let presentation = ImagePresentation {
            width_percent: 40,
            alignment: ImageAlignment::Center,
        };
        let rendered = serialize_inline_image("alt", "a.png", None, Some(presentation));
        assert!(rendered.contains("{width=40 align=center}"));
        let parsed = inline_image_at(&rendered, 2).unwrap();
        assert_eq!(parsed.presentation, Some(presentation));

        let rejected = "![alt](a.png \"{width=9 align=center}\")";
        assert!(inline_image_at(rejected, 2).unwrap().presentation.is_none());
        assert_eq!(snap_image_width_percent(48), 50);
        assert_eq!(snap_image_width_percent(40), 40);
        assert_eq!(clamp_image_width_percent(3), 10);
    }

    #[test]
    fn proven_single_line_image_covers_reference_and_inline_forms() {
        assert!(proven_single_line_image_span("![alt](url.png)"));
        assert!(proven_single_line_image_span("![alt][ref]"));
        assert!(proven_single_line_image_span("![alt][]"));
        assert!(proven_single_line_image_span("![logo]"));
        assert!(!proven_single_line_image_span(
            "![alt](url.png\n \"title\")"
        ));
        assert!(!proven_single_line_image_span("![alt](url.png"));
        assert!(!proven_single_line_image_span("![alt][ref"));
        assert!(inline_image_at("![alt][ref]", 3).is_none());
    }

    #[test]
    fn simple_html_anchor_round_trips_and_rejects_nested_markup() {
        let source = r#"<a href="guide.md" title="Guide">Guide</a>"#;
        let anchor = simple_html_anchor_at(source, source.find("Guide</").unwrap()).unwrap();
        assert_eq!(anchor.label, "Guide");
        assert_eq!(anchor.url, "guide.md");
        assert_eq!(anchor.title.as_deref(), Some("Guide"));
        assert_eq!(anchor.source_range, 0..source.len());
        assert!(
            simple_html_anchor_at(
                r#"<a href="https://example.com"><img src="b.png" alt="B"></a>"#,
                10
            )
            .is_none()
        );
        assert!(simple_html_anchor_at(r#"<a href="x" class="y">z</a>"#, 20).is_none());
    }

    #[test]
    fn serialize_html_anchor_preserves_shape() {
        assert_eq!(
            serialize_html_anchor("Guide", "guide.md", Some("Guide")),
            r#"<a href="guide.md" title="Guide">Guide</a>"#
        );
        assert_eq!(
            serialize_html_anchor("简体中文", "README.zh-CN.md", None),
            r#"<a href="README.zh-CN.md">简体中文</a>"#
        );
    }
}
