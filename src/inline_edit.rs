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

/// Byte range of the authored destination inside a complete inline-image span
/// `![label](destination "title")`, relative to the span start. Returns
/// `None` for any span whose destination cannot be located with certainty;
/// callers must skip those conservatively rather than guess.
pub fn authored_image_destination_range(authored: &str) -> Option<Range<usize>> {
    if !authored.starts_with("![") || !authored.ends_with(')') {
        return None;
    }
    let label_end = find_unescaped(authored, 2, b']')?;
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
            let label_end = find_unescaped(authored, prefix.len(), b']')?;
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

fn escape_destination(url: &str) -> String {
    if url.chars().any(char::is_whitespace) || url.contains(['(', ')', '<', '>']) {
        format!("<{}>", url.replace('\\', "\\\\").replace('>', "\\>"))
    } else {
        url.replace('\\', "\\\\")
    }
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
                presentation.width_percent.clamp(25, 100)
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
            width = value
                .parse::<u8>()
                .ok()
                .filter(|value| matches!(value, 25 | 50 | 75 | 100));
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
        assert_eq!(unescape_markdown("..\\shared\\图片.png"), "..\\shared\\图片.png");
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
}
