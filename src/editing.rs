//! Plain-text editing helpers used by `MarkdownDocument` editing methods:
//! paragraph ranges, list-marker continuation, indent/outdent math, and
//! heading/line-offset bookkeeping. All functions are pure / stateless.

use std::ops::Range;

use crate::text_util::clamp_to_char_boundary;

pub(crate) fn paragraph_range_at(text: &str, byte_index: usize) -> Range<usize> {
    if text.is_empty() {
        return 0..0;
    }

    let index = clamp_to_char_boundary(text, byte_index);
    let mut start = text[..index]
        .rfind("\n\n")
        .map_or(0, |position| position + 2);
    while start < text.len() && text[start..].starts_with('\n') {
        start += 1;
    }

    let mut end = text[index..]
        .find("\n\n")
        .map_or(text.len(), |position| index + position);
    while end > start && text[..end].ends_with('\n') {
        end -= 1;
    }

    start..end.max(start)
}

pub(crate) fn adjust_offset_for_line_marker_removal(
    offset: usize,
    line_start: usize,
    marker_len: usize,
    delta: &mut isize,
) {
    if offset >= line_start + marker_len {
        *delta -= marker_len as isize;
    } else if offset > line_start {
        *delta -= (offset - line_start) as isize;
    }
}

pub(crate) fn adjust_offset_for_line_insert(
    offset: usize,
    line_start: usize,
    insert_len: usize,
    shift_at_line_start: bool,
    delta: &mut isize,
) {
    if offset > line_start || (shift_at_line_start && offset == line_start) {
        *delta += insert_len as isize;
    }
}

pub(crate) fn heading_level_at(text: &str, line_start: usize) -> Option<u8> {
    let marker_len = heading_marker_len_at(text, line_start);
    (marker_len > 0).then(|| (marker_len - 1) as u8)
}

pub(crate) fn heading_marker_len_at(text: &str, line_start: usize) -> usize {
    let line_end = text[line_start..]
        .find('\n')
        .map_or(text.len(), |line_end| line_start + line_end);
    let line = &text[line_start..line_end];
    let hash_count = line.bytes().take_while(|byte| *byte == b'#').count();

    if (1..=6).contains(&hash_count) && line.as_bytes().get(hash_count) == Some(&b' ') {
        hash_count + 1
    } else {
        0
    }
}

pub(crate) fn selected_line_starts(text: &str, range: std::ops::Range<usize>) -> Vec<usize> {
    if text.is_empty() {
        return Vec::new();
    }

    let start = clamp_to_char_boundary(text, range.start);
    let mut end = clamp_to_char_boundary(text, range.end);
    if end > start && end > 0 && text.as_bytes().get(end - 1) == Some(&b'\n') {
        end -= 1;
    }

    let first_line_start = text[..start].rfind('\n').map_or(0, |index| index + 1);
    let last_line_start = text[..end].rfind('\n').map_or(0, |index| index + 1);
    let mut starts = vec![first_line_start];
    let mut search_from = first_line_start;
    while let Some(relative_newline) = text[search_from..].find('\n') {
        let next_start = search_from + relative_newline + 1;
        if next_start > last_line_start || next_start >= text.len() {
            break;
        }
        starts.push(next_start);
        search_from = next_start;
    }
    starts
}

pub(crate) fn line_outdent_len(text: &str, line_start: usize) -> usize {
    if text[line_start..].starts_with('\t') {
        return 1;
    }

    let mut len = 0usize;
    for ch in text[line_start..].chars().take(4) {
        if ch == ' ' {
            len += 1;
        } else {
            break;
        }
    }
    len
}

pub(crate) fn leading_whitespace(text: &str) -> &str {
    let end = text
        .char_indices()
        .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))
        .unwrap_or(text.len());
    &text[..end]
}

pub(crate) fn markdown_continuation(before_cursor: &str) -> String {
    let indent = leading_whitespace(before_cursor);
    let rest = &before_cursor[indent.len()..];

    if rest == ">" || rest.starts_with("> ") {
        return format!("{indent}> ");
    }

    if let Some(marker) = unordered_list_marker(rest) {
        return format!("{indent}{marker}");
    }

    if let Some(marker) = ordered_list_marker(rest) {
        return format!("{indent}{marker}");
    }

    indent.to_string()
}

pub(crate) fn is_empty_list_marker(before_cursor: &str) -> bool {
    let indent = leading_whitespace(before_cursor);
    let rest = before_cursor[indent.len()..].trim_end();

    matches!(rest, "-" | "*" | "+") || is_empty_task_marker(rest) || empty_ordered_marker(rest)
}

pub(crate) fn unordered_list_marker(rest: &str) -> Option<String> {
    let marker = rest.chars().next()?;
    if !matches!(marker, '-' | '*' | '+') {
        return None;
    }

    let marker_len = marker.len_utf8();
    let after_marker = rest.get(marker_len..)?;
    if !after_marker.starts_with(' ') {
        return None;
    }

    let body = &after_marker[1..];
    let body_lower = body.to_ascii_lowercase();
    if body_lower.starts_with("[ ] ") || body_lower.starts_with("[x] ") {
        Some(format!("{marker} [ ] "))
    } else {
        Some(format!("{marker} "))
    }
}

pub(crate) fn ordered_list_marker(rest: &str) -> Option<String> {
    let digit_end = rest
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .map(|(index, ch)| index + ch.len_utf8())
        .last()?;
    let number = rest[..digit_end].parse::<u64>().ok()?;
    let delimiter = rest[digit_end..].chars().next()?;
    if !matches!(delimiter, '.' | ')') {
        return None;
    }
    let after_delimiter = &rest[digit_end + delimiter.len_utf8()..];
    if !after_delimiter.starts_with(' ') {
        return None;
    }
    Some(format!("{}{delimiter} ", number + 1))
}

fn is_empty_task_marker(rest: &str) -> bool {
    matches!(
        rest.to_ascii_lowercase().as_str(),
        "- [ ]" | "- [x]" | "* [ ]" | "* [x]" | "+ [ ]" | "+ [x]"
    )
}

fn empty_ordered_marker(rest: &str) -> bool {
    let Some(digit_end) = rest
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .map(|(index, ch)| index + ch.len_utf8())
        .last()
    else {
        return false;
    };
    let Some(delimiter) = rest[digit_end..].chars().next() else {
        return false;
    };
    matches!(delimiter, '.' | ')') && rest[digit_end + delimiter.len_utf8()..].trim().is_empty()
}
