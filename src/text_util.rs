//! Small string/offset helpers shared across parsing, editing, and rendering.

/// Clamp `index` to the nearest UTF-8 character boundary at or before it.
pub fn clamp_to_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Clamp both ends of a range to character boundaries, keeping `end >= start`.
pub fn clamp_range_to_char_boundaries(
    text: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let start = clamp_to_char_boundary(text, range.start);
    let end = clamp_to_char_boundary(text, range.end).max(start);
    start..end
}

/// Add a signed delta to an offset without underflow.
pub fn offset_with_delta(offset: usize, delta: isize) -> usize {
    if delta >= 0 {
        offset + delta as usize
    } else {
        offset.saturating_sub((-delta) as usize)
    }
}

/// 1-based (line, column) for a byte offset, clamped to a char boundary.
pub fn line_column_at(text: &str, offset: usize) -> (usize, usize) {
    let offset = clamp_to_char_boundary(text, offset);
    let line_start = text[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = text[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = text[line_start..offset].chars().count() + 1;
    (line, column)
}

/// Trimmed text of the 1-based `line_number`, or empty if out of range.
pub fn line_snippet_at(text: &str, line_number: usize) -> String {
    text.lines()
        .nth(line_number.saturating_sub(1))
        .unwrap_or_default()
        .trim()
        .to_string()
}
