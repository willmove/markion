//! Escaping policies mirrored from `markion-docx-import`
//! (`crates/docx-import/src/render.rs`), plus the line-start block-marker
//! protection that keeps pasted prose from forming unintended structures.

pub(crate) fn escape_markdown_text(value: &str) -> String {
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

pub(crate) fn escape_markdown_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

pub(crate) fn escape_markdown_url(value: &str) -> String {
    value
        .replace(' ', "%20")
        .replace('(', "%28")
        .replace(')', "%29")
}

pub(crate) fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn escape_html_attr(value: &str) -> String {
    escape_html_text(value).replace('"', "&quot;")
}

/// Escapes a leading block-marker lookalike in text that begins a line:
/// `-`/`+`/`>` (also covering `---`-style runs) and ordered-marker
/// lookalikes (`1.`, `1)`). Applied after `escape_markdown_text`, so the
/// inserted backslash must not be escaped again.
pub(crate) fn escape_line_start_marker(text: &str) -> String {
    let (indent, body) = text.split_at(usize::from(text.starts_with(' ')));
    let escaped = match body.chars().next() {
        Some('>') => Some(format!("\\{body}")),
        Some(marker @ ('-' | '+')) => {
            let rest = &body[marker.len_utf8()..];
            if rest.is_empty()
                || rest.starts_with(' ')
                || (marker == '-' && rest.chars().all(|ch| ch == '-'))
            {
                Some(format!("\\{body}"))
            } else {
                None
            }
        }
        Some(first) if first.is_ascii_digit() => {
            let digits = body.len()
                - body
                    .trim_start_matches(|ch: char| ch.is_ascii_digit())
                    .len();
            let after = &body[digits..];
            match after.chars().next() {
                Some('.' | ')') => {
                    let tail = &after[1..];
                    if tail.is_empty() || tail.starts_with(' ') {
                        Some(format!("{}\\{after}", &body[..digits]))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    };
    match escaped {
        Some(escaped) => format!("{indent}{escaped}"),
        None => text.to_owned(),
    }
}
