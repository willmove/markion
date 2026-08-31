//! Markdown table parsing, formatting, and source-range lookup.
//!
//! Pure algorithms over the raw document text — no `MarkdownDocument` state.

use std::ops::Range;

use crate::model::{InlineSpan, RichText, TableAlignment};
use crate::text_util::clamp_to_char_boundary;

#[derive(Default)]
pub(crate) struct TableDraft {
    pub rows: Vec<Vec<RichText>>,
    pub alignments: Vec<TableAlignment>,
    pub current_row: Option<Vec<RichText>>,
    pub current_cell: Vec<InlineSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownTable {
    pub rows: Vec<Vec<String>>,
    pub alignments: Vec<TableAlignment>,
}

impl MarkdownTable {
    pub fn column_count(&self) -> usize {
        self.rows.iter().map(Vec::len).max().unwrap_or(0).max(1)
    }

    pub fn normalize(&mut self) {
        let columns = self.column_count();
        for row in &mut self.rows {
            row.resize(columns, String::new());
        }
        self.alignments.resize(columns, TableAlignment::Default);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TablePosition {
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TableCellSourceRange {
    pub row: usize,
    pub column: usize,
    pub source_range: Range<usize>,
}

pub(crate) fn table_range_at(text: &str, byte_index: usize) -> Option<Range<usize>> {
    if text.is_empty() {
        return None;
    }

    let index = clamp_to_char_boundary(text, byte_index);
    let (mut start, mut end) = line_bounds_for_table_lookup(text, index)?;

    if !is_markdown_table_candidate(&text[start..end]) {
        if index == start && start > 0 {
            let (previous_start, previous_end) = previous_line_bounds(text, start)?;
            if is_markdown_table_candidate(&text[previous_start..previous_end]) {
                start = previous_start;
                end = previous_end;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    while let Some((previous_start, previous_end)) = previous_line_bounds(text, start) {
        if !is_markdown_table_candidate(&text[previous_start..previous_end]) {
            break;
        }
        start = previous_start;
    }

    while let Some((next_start, next_end)) = next_line_bounds(text, end) {
        if !is_markdown_table_candidate(&text[next_start..next_end]) {
            break;
        }
        end = next_end;
    }

    let range = start..end;
    parse_markdown_table(&text[range.clone()]).map(|_| range)
}

pub(crate) fn table_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut offset = 0usize;

    while offset < text.len() {
        let line_end = text[offset..]
            .find('\n')
            .map_or(text.len(), |relative| offset + relative);
        if is_markdown_table_candidate(&text[offset..line_end])
            && let Some(range) = table_range_at(text, offset)
            && ranges
                .last()
                .is_none_or(|last: &Range<usize>| *last != range)
        {
            offset = range.end;
            ranges.push(range);
            if offset < text.len() && text[offset..].starts_with('\n') {
                offset += 1;
            }
            continue;
        }

        offset = if line_end < text.len() {
            line_end + 1
        } else {
            text.len()
        };
    }

    ranges
}

/// Drop trailing CR/LF from a pulldown-cmark table event range so the
/// preview block owns table lines only. Cell mapping and whole-table
/// replacement use this slice; the event *start* is what keeps tables
/// from sorting to offset 0.
pub(crate) fn table_preview_source_range(text: &str, event_range: Range<usize>) -> Range<usize> {
    let start = event_range.start.min(text.len());
    let mut end = event_range.end.min(text.len());
    if start > end {
        return start..start;
    }
    while end > start && matches!(text.as_bytes().get(end - 1), Some(b'\n' | b'\r')) {
        end -= 1;
    }
    start..end
}

pub(crate) fn table_position_at(source: &str, byte_index: usize) -> Option<TablePosition> {
    let index = clamp_to_char_boundary(source, byte_index);
    let (line_start, line_end) = line_bounds_for_table_lookup(source, index)?;
    let line = &source[line_start..line_end];
    let line_index = source[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let separator_index = source
        .lines()
        .position(is_markdown_table_separator_line)
        .unwrap_or(1);
    let row = if line_index <= separator_index {
        0
    } else {
        line_index - 1
    };
    let column = table_column_at_line(line, index - line_start);

    Some(TablePosition { row, column })
}

pub(crate) fn parse_markdown_table(source: &str) -> Option<MarkdownTable> {
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() < 2 {
        return None;
    }

    let separator_index = lines
        .iter()
        .position(|line| is_markdown_table_separator_line(line))?;
    if separator_index == 0 {
        return None;
    }
    let alignments = split_markdown_table_row(lines[separator_index])
        .iter()
        .map(|cell| parse_table_alignment(cell))
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if index == separator_index {
            continue;
        }
        if !is_markdown_table_candidate(line) {
            return None;
        }
        rows.push(split_markdown_table_row(line));
    }

    if rows.is_empty() {
        return None;
    }

    let mut table = MarkdownTable { rows, alignments };
    table.normalize();
    Some(table)
}

pub(crate) fn table_cell_source_ranges(source: &str) -> Option<Vec<TableCellSourceRange>> {
    let lines = source.split_inclusive('\n').collect::<Vec<_>>();
    if lines.len() < 2 {
        return None;
    }
    let separator_index = lines
        .iter()
        .position(|line| is_markdown_table_separator_line(line.trim_end_matches(['\r', '\n'])))?;
    if separator_index == 0 {
        return None;
    }
    let expected_columns =
        markdown_table_cell_ranges(lines[separator_index].trim_end_matches(['\r', '\n'])).len();
    if expected_columns < 2 {
        return None;
    }

    let mut result = Vec::new();
    let mut source_offset = 0usize;
    let mut logical_row = 0usize;
    for (line_index, line_with_newline) in lines.iter().enumerate() {
        let line = line_with_newline.trim_end_matches(['\r', '\n']);
        let ranges = markdown_table_cell_ranges(line);
        if ranges.len() != expected_columns {
            return None;
        }
        if line_index != separator_index {
            for (column, range) in ranges.into_iter().enumerate() {
                result.push(TableCellSourceRange {
                    row: logical_row,
                    column,
                    source_range: source_offset + range.start..source_offset + range.end,
                });
            }
            logical_row += 1;
        }
        source_offset += line_with_newline.len();
    }
    Some(result)
}

pub(crate) fn format_markdown_table(table: &MarkdownTable) -> String {
    let columns = table.column_count();
    let mut rows = table.rows.clone();
    for row in &mut rows {
        row.resize(columns, String::new());
    }

    let widths = (0..columns)
        .map(|column| {
            rows.iter()
                .map(|row| row[column].chars().count())
                .max()
                .unwrap_or(0)
                .max(3)
        })
        .collect::<Vec<_>>();

    let mut output = String::new();
    if let Some(header) = rows.first() {
        output.push_str(&format_markdown_table_row(header, &widths));
        output.push('\n');
        output.push_str(&format_markdown_table_separator(&widths, &table.alignments));
    }

    for row in rows.iter().skip(1) {
        output.push('\n');
        output.push_str(&format_markdown_table_row(row, &widths));
    }

    output
}

fn format_markdown_table_row(row: &[String], widths: &[usize]) -> String {
    let mut output = String::from("|");
    for (cell, width) in row.iter().zip(widths.iter().copied()) {
        output.push(' ');
        output.push_str(cell);
        let padding = width.saturating_sub(cell.chars().count());
        output.extend(std::iter::repeat_n(' ', padding));
        output.push_str(" |");
    }
    output
}

fn format_markdown_table_separator(widths: &[usize], alignments: &[TableAlignment]) -> String {
    let mut output = String::from("|");
    for (column, width) in widths.iter().enumerate() {
        let hyphens = "-".repeat((*width).max(3));
        let marker = match alignments
            .get(column)
            .copied()
            .unwrap_or(TableAlignment::Default)
        {
            TableAlignment::Default => hyphens,
            TableAlignment::Left => format!(":{hyphens}"),
            TableAlignment::Center => format!(":{hyphens}:"),
            TableAlignment::Right => format!("{hyphens}:"),
        };
        output.push(' ');
        output.push_str(&marker);
        output.push_str(" |");
    }
    output
}

fn parse_table_alignment(cell: &str) -> TableAlignment {
    let trimmed = cell.trim();
    let left = trimmed.starts_with(':');
    let right = trimmed.ends_with(':');
    match (left, right) {
        (true, true) => TableAlignment::Center,
        (true, false) => TableAlignment::Left,
        (false, true) => TableAlignment::Right,
        (false, false) => TableAlignment::Default,
    }
}

pub(crate) fn formatted_table_cell_range(
    table: &MarkdownTable,
    row_index: usize,
    column_index: usize,
) -> Option<Range<usize>> {
    if table.rows.is_empty() {
        return None;
    }
    let formatted = format_markdown_table(table);
    table_cell_source_ranges(&formatted)?
        .into_iter()
        .find(|cell| cell.row == row_index && cell.column == column_index)
        .map(|cell| cell.source_range)
}

fn line_bounds_for_table_lookup(text: &str, byte_index: usize) -> Option<(usize, usize)> {
    if text.is_empty() {
        return None;
    }
    let index = clamp_to_char_boundary(text, byte_index.min(text.len()));
    let start = text[..index].rfind('\n').map_or(0, |index| index + 1);
    let end = text[index..]
        .find('\n')
        .map_or(text.len(), |line_end| index + line_end);
    Some((start, end))
}

fn previous_line_bounds(text: &str, line_start: usize) -> Option<(usize, usize)> {
    if line_start == 0 {
        return None;
    }

    let previous_end = line_start - 1;
    let previous_start = text[..previous_end]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    Some((previous_start, previous_end))
}

fn next_line_bounds(text: &str, line_end: usize) -> Option<(usize, usize)> {
    if line_end >= text.len() {
        return None;
    }

    let next_start = line_end + 1;
    let next_end = text[next_start..]
        .find('\n')
        .map_or(text.len(), |index| next_start + index);
    Some((next_start, next_end))
}

fn is_markdown_table_candidate(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains('|') && split_markdown_table_row(line).len() >= 2
}

pub(crate) fn is_markdown_table_separator_line(line: &str) -> bool {
    let cells = split_markdown_table_row(line);
    cells.len() >= 2
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            trimmed.chars().filter(|ch| *ch == '-').count() >= 3
                && trimmed.chars().all(|ch| matches!(ch, '-' | ':' | ' '))
        })
}

fn split_markdown_table_row(line: &str) -> Vec<String> {
    markdown_table_cell_ranges(line)
        .into_iter()
        .map(|range| line[range].to_string())
        .collect()
}

fn markdown_table_cell_ranges(line: &str) -> Vec<Range<usize>> {
    let content_start = line.len() - line.trim_start().len();
    let content_end = line.trim_end().len();
    if content_start >= content_end {
        return Vec::new();
    }
    let content = &line[content_start..content_end];
    let mut delimiters = Vec::new();
    let mut escaped = false;
    for (offset, ch) in content.char_indices() {
        if ch == '\\' {
            escaped = !escaped;
            continue;
        }
        if ch == '|' && !escaped {
            delimiters.push(content_start + offset);
        }
        escaped = false;
    }

    let leading_pipe = delimiters.first() == Some(&content_start);
    let trailing_pipe = delimiters.last() == Some(&(content_end - 1));
    let mut ranges = Vec::new();
    let mut cell_start = if leading_pipe {
        content_start + 1
    } else {
        content_start
    };
    for delimiter in delimiters {
        if delimiter < cell_start {
            continue;
        }
        ranges.push(cell_start..delimiter);
        cell_start = delimiter + 1;
    }
    if !trailing_pipe {
        ranges.push(cell_start..content_end);
    }

    ranges
        .into_iter()
        .map(|Range { mut start, mut end }| {
            while start < end && line.as_bytes()[start].is_ascii_whitespace() {
                start += 1;
            }
            while end > start && line.as_bytes()[end - 1].is_ascii_whitespace() {
                end -= 1;
            }
            start..end
        })
        .collect()
}

fn table_column_at_line(line: &str, byte_column: usize) -> usize {
    let byte_column = clamp_to_char_boundary(line, byte_column.min(line.len()));
    let before_cursor = &line[..byte_column];
    let pipe_count = unescaped_pipe_count(before_cursor);
    if line.trim_start().starts_with('|') {
        pipe_count.saturating_sub(1)
    } else {
        pipe_count
    }
}

fn unescaped_pipe_count(text: &str) -> usize {
    let mut escaped = false;
    let mut count = 0usize;
    for ch in text.chars() {
        if ch == '\\' {
            escaped = !escaped;
            continue;
        }
        if ch == '|' && !escaped {
            count += 1;
        }
        escaped = false;
    }
    count
}

/// Horizontal padding already applied by GFM table cells (`p_2` on each side).
const TABLE_CELL_HORIZONTAL_PADDING_PX: f32 = 16.0;
const TABLE_COLUMN_ASCII_EM: f32 = 0.55;
const TABLE_COLUMN_WIDE_EM: f32 = 1.0;
const TABLE_COLUMN_MIN_EMS: f32 = 2.0;
/// Hard ceiling: no column may exceed this many equal-shares of the table.
const TABLE_COLUMN_MAX_SHARE_MULTIPLE: f32 = 3.0;
/// Header wrap budget used to raise a column's minimum recommended width.
const TABLE_HEADER_MAX_WRAP_LINES: f32 = 3.0;

/// Recommended flex-grow weights for GFM table columns, one per column.
///
/// Estimates each cell from its rendered plain text (not focused Visual Edit
/// source markup): ASCII glyphs at `0.55em`, other scalars at `1em`, plus cell
/// padding. A column's preferred width is the max over its cells, then raised
/// to a header minimum (short parenthesis units and a three-line wrap budget),
/// with leftover body width compressed so long paragraphs cannot linearly
/// starve unit headers. Finally each column is capped at three equal-shares of
/// the table (`3 / n`). Callers apply these as `flex_grow` with `flex_basis 0`
/// so the table still fills the document content column.
pub fn table_column_flex_weights(rows: &[Vec<RichText>], table_font_size: f32) -> Vec<f32> {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    if columns == 0 {
        return Vec::new();
    }
    let font_size = table_font_size.max(1.0);
    let floor = table_column_min_width(font_size);
    let mut preferred = vec![floor; columns];
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            preferred[index] =
                preferred[index].max(estimate_table_cell_width(&cell.text, font_size));
        }
    }
    let mut mins = vec![floor; columns];
    if let Some(header) = rows.first() {
        for (index, cell) in header.iter().enumerate() {
            mins[index] = mins[index].max(header_column_min_width(&cell.text, font_size, floor));
        }
    }
    for (weight, min) in preferred.iter_mut().zip(mins.iter()) {
        *weight = (*weight).max(*min);
        let extra = (*weight - *min).max(0.0);
        *weight = *min + compress_column_extra(extra, floor);
    }
    clamp_column_share_cap(&mut preferred, &mins);
    preferred
}

fn table_column_min_width(font_size: f32) -> f32 {
    TABLE_CELL_HORIZONTAL_PADDING_PX + TABLE_COLUMN_MIN_EMS * font_size
}

fn estimate_table_cell_width(text: &str, font_size: f32) -> f32 {
    estimate_text_content_width(text, font_size) + TABLE_CELL_HORIZONTAL_PADDING_PX
}

fn estimate_text_content_width(text: &str, font_size: f32) -> f32 {
    text.chars().map(|ch| glyph_em_width(ch) * font_size).sum()
}

fn glyph_em_width(ch: char) -> f32 {
    if ch.is_ascii() {
        TABLE_COLUMN_ASCII_EM
    } else {
        TABLE_COLUMN_WIDE_EM
    }
}

fn header_column_min_width(text: &str, font_size: f32, floor: f32) -> f32 {
    let content = estimate_text_content_width(text, font_size);
    let three_line = TABLE_CELL_HORIZONTAL_PADDING_PX + content / TABLE_HEADER_MAX_WRAP_LINES;
    let unsplittable =
        TABLE_CELL_HORIZONTAL_PADDING_PX + max_short_paren_unit_width(text, font_size);
    floor.max(three_line).max(unsplittable)
}

fn max_short_paren_unit_width(text: &str, font_size: f32) -> f32 {
    let chars: Vec<char> = text.chars().collect();
    let mut max_width = 0.0f32;
    let mut index = 0;
    while index < chars.len() {
        if let Some(close) = matching_short_paren_close(&chars, index) {
            let width: f32 = chars[index..=close]
                .iter()
                .map(|ch| glyph_em_width(*ch) * font_size)
                .sum();
            max_width = max_width.max(width);
            index = close + 1;
        } else {
            index += 1;
        }
    }
    max_width
}

fn matching_short_paren_close(chars: &[char], open_at: usize) -> Option<usize> {
    let open = *chars.get(open_at)?;
    let close = matching_close_paren(open)?;
    for (offset, &ch) in chars.iter().enumerate().skip(open_at + 1) {
        if ch == open {
            return None;
        }
        if ch == close {
            let inner_len = chars[open_at + 1..offset]
                .iter()
                .filter(|c| !c.is_whitespace())
                .count();
            return ((1..=3).contains(&inner_len)).then_some(offset);
        }
    }
    None
}

fn matching_close_paren(open: char) -> Option<char> {
    match open {
        '(' => Some(')'),
        '（' => Some('）'),
        _ => None,
    }
}

fn compress_column_extra(extra: f32, typical: f32) -> f32 {
    if extra <= 0.0 || typical <= 0.0 {
        return 0.0;
    }
    (extra * typical).sqrt()
}

fn clamp_column_share_cap(weights: &mut [f32], mins: &[f32]) {
    let columns = weights.len();
    if columns <= 1 {
        return;
    }
    for _ in 0..2 {
        let total: f32 = weights.iter().sum();
        if total <= 0.0 {
            return;
        }
        let cap = TABLE_COLUMN_MAX_SHARE_MULTIPLE / columns as f32 * total;
        for (weight, min) in weights.iter_mut().zip(mins.iter()) {
            let allowed = cap.max(*min);
            if *weight > allowed {
                *weight = allowed;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_cell_ranges_skip_padding_and_separator_with_utf8_and_escaped_pipes() {
        let source = "| 名称 | a\\|b |\r\n| :--- | ---: |\r\n| 甲 | 2 |";
        let cells = table_cell_source_ranges(source).expect("exact table cell ranges");
        let values = cells
            .iter()
            .map(|cell| {
                (
                    cell.row,
                    cell.column,
                    source[cell.source_range.clone()].to_string(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                (0, 0, "名称".into()),
                (0, 1, "a\\|b".into()),
                (1, 0, "甲".into()),
                (1, 1, "2".into()),
            ]
        );
        let parsed = parse_markdown_table(source).expect("parsed table");
        assert_eq!(
            parsed.alignments,
            vec![TableAlignment::Left, TableAlignment::Right]
        );
    }

    #[test]
    fn cell_ranges_accept_authored_rows_without_outer_pipes() {
        let source = "A | B\n--- | ---\none | two";
        let cells = table_cell_source_ranges(source).expect("exact table cell ranges");
        assert_eq!(cells.len(), 4);
        assert_eq!(&source[cells[0].source_range.clone()], "A");
        assert_eq!(&source[cells[3].source_range.clone()], "two");
    }

    #[test]
    fn formatted_cell_range_tracks_semantic_content_after_width_reflow() {
        let mut table =
            parse_markdown_table("| A | B |\n| --- | --- |\n| x | y |").expect("parsed table");
        table.rows[1][0] = "宽字符 and longer".into();
        let formatted = format_markdown_table(&table);
        let range = formatted_table_cell_range(&table, 1, 0).expect("formatted cell");
        assert_eq!(&formatted[range], "宽字符 and longer");
        assert_eq!(parse_markdown_table(&formatted).expect("round trip"), table);
    }

    #[test]
    fn preview_source_range_drops_trailing_newlines_keeps_start() {
        let text = "intro\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n\nafter";
        let start = text.find("| A | B |").unwrap();
        let table = "| A | B |\n| --- | --- |\n| 1 | 2 |";
        let table_end = start + table.len();
        let event = start..table_end + 2;
        let range = table_preview_source_range(text, event);
        assert_eq!(range.start, start);
        assert_eq!(&text[range], table);
    }

    #[test]
    fn table_position_ignores_escaped_pipe_delimiters() {
        let source = "| A | B |\n| --- | --- |\n| a\\|b | c |";
        let escaped_pipe = source.find("\\|").unwrap() + 1;
        let second_cell = source.rfind(" c ").unwrap() + 1;
        assert_eq!(
            table_position_at(source, escaped_pipe),
            Some(TablePosition { row: 1, column: 0 })
        );
        assert_eq!(
            table_position_at(source, second_cell),
            Some(TablePosition { row: 1, column: 1 })
        );
    }

    #[test]
    fn content_column_weights_prefer_long_description_columns() {
        let rows = vec![
            vec![RichText::plain("名称"), RichText::plain("说明")],
            vec![RichText::plain("操作系统"), RichText::plain("Ubuntu")],
            vec![
                RichText::plain("CPU"),
                RichText::plain("Intel(R) Xeon(R) Platinum 8358 CPU @ 2.60GHz"),
            ],
        ];
        let weights = table_column_flex_weights(&rows, 12.0);
        assert_eq!(weights.len(), 2);
        assert!(
            weights[0] < weights[1],
            "short 名称 column should be narrower than 说明: {weights:?}"
        );
    }

    #[test]
    fn one_column_table_keeps_a_positive_weight() {
        let weights = table_column_flex_weights(&[vec![RichText::plain("command")]], 12.0);
        assert_eq!(weights.len(), 1);
        assert!(weights[0] > 0.0);
    }

    #[test]
    fn empty_and_ragged_columns_stay_at_the_readable_floor() {
        let font_size = 12.0;
        let floor = table_column_min_width(font_size);
        let empty = table_column_flex_weights(
            &[vec![RichText::plain("a"), RichText::plain("")]],
            font_size,
        );
        assert_eq!(empty[1], floor);

        let ragged = table_column_flex_weights(
            &[
                vec![RichText::plain("aa")],
                vec![RichText::plain("b"), RichText::plain("")],
            ],
            font_size,
        );
        assert_eq!(ragged.len(), 2);
        assert_eq!(ragged[1], floor);
        assert!(table_column_flex_weights(&[], font_size).is_empty());
    }

    #[test]
    fn wide_scalars_count_heavier_than_the_same_number_of_ascii_letters() {
        let font_size = 12.0;
        let ascii = table_column_flex_weights(&[vec![RichText::plain("aaa")]], font_size);
        let cjk = table_column_flex_weights(&[vec![RichText::plain("甲甲甲")]], font_size);
        assert!(
            cjk[0] > ascii[0],
            "CJK cells should out-weigh equal-length ASCII: ascii={ascii:?} cjk={cjk:?}"
        );
    }

    #[test]
    fn column_weights_use_rendered_plain_text_not_source_markup() {
        let font_size = 12.0;
        let rendered = table_column_flex_weights(&[vec![RichText::plain("bold")]], font_size);
        let markup = table_column_flex_weights(&[vec![RichText::plain("**bold**")]], font_size);
        assert!(
            rendered[0] < markup[0],
            "callers must pass rendered cell text, not source markup: {rendered:?} vs {markup:?}"
        );
    }

    fn six_column_server_spec_rows() -> Vec<Vec<RichText>> {
        let long_config = "CPU: Intel Xeon Gold 64C; 内存: 1TB; 系统盘: 480G SSD; 数据盘: 16TB";
        let long_note = "整机尺寸 875×447×44mm，满载重量约 32kg，需预留前后维护通道";
        vec![
            vec![
                RichText::plain("设备类型"),
                RichText::plain("关键配置"),
                RichText::plain("品牌型号"),
                RichText::plain("实际功率（W）"),
                RichText::plain("数量"),
                RichText::plain("备注"),
            ],
            vec![
                RichText::plain("训练服务器"),
                RichText::plain(long_config),
                RichText::plain("浪潮 CS5868H3"),
                RichText::plain("245"),
                RichText::plain("102台"),
                RichText::plain(long_note),
            ],
        ]
    }

    fn linear_content_weights(rows: &[Vec<RichText>], font_size: f32) -> Vec<f32> {
        let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
        let floor = table_column_min_width(font_size);
        let mut weights = vec![floor; columns];
        for row in rows {
            for (index, cell) in row.iter().enumerate() {
                weights[index] =
                    weights[index].max(estimate_table_cell_width(&cell.text, font_size));
            }
        }
        weights
    }

    fn share(weights: &[f32], index: usize) -> f32 {
        let total: f32 = weights.iter().sum();
        weights[index] / total
    }

    #[test]
    fn six_column_table_caps_long_columns_and_protects_unit_headers() {
        let font_size = 12.0;
        let rows = six_column_server_spec_rows();
        let weights = table_column_flex_weights(&rows, font_size);
        let linear = linear_content_weights(&rows, font_size);
        let total: f32 = weights.iter().sum();
        assert_eq!(weights.len(), 6);
        for (index, weight) in weights.iter().enumerate() {
            assert!(
                *weight / total <= 0.5 + 1e-4,
                "column {index} share {} exceeds 3/6",
                *weight / total
            );
        }
        assert!(
            share(&weights, 3) > share(&linear, 3),
            "实际功率（W） should gain share vs an uncapped linear split: balanced={weights:?} linear={linear:?}"
        );
        assert!(
            share(&weights, 1) <= 0.5 + 1e-4,
            "关键配置 must stay within three equal-shares"
        );
    }

    #[test]
    fn short_header_parenthesis_units_set_a_one_line_minimum() {
        let font_size = 12.0;
        let unit = max_short_paren_unit_width("实际功率（W）", font_size);
        let ascii = max_short_paren_unit_width("Power (W)", font_size);
        assert!(unit > 0.0, "fullwidth （W） should be unsplittable");
        assert!(ascii > 0.0, "ASCII (W) should be unsplittable");
        let min = header_column_min_width(
            "实际功率（W）",
            font_size,
            table_column_min_width(font_size),
        );
        assert!(
            min + 1e-4 >= unit + TABLE_CELL_HORIZONTAL_PADDING_PX,
            "header min {min} should cover the parenthesis unit {unit}"
        );
    }

    #[test]
    fn long_header_minimum_covers_a_three_line_wrap_budget() {
        let font_size = 12.0;
        let header = "这是一段需要折行的很长列表头文字内容";
        let content = estimate_text_content_width(header, font_size);
        let min = header_column_min_width(header, font_size, table_column_min_width(font_size));
        assert!(
            min + 1e-4 >= TABLE_CELL_HORIZONTAL_PADDING_PX + content / 3.0,
            "header min {min} should be at least one third of unwrapped width {content}"
        );
    }
}
