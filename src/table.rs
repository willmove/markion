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
}
