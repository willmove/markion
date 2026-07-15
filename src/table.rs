//! Markdown table parsing, formatting, and source-range lookup.
//!
//! Pure algorithms over the raw document text — no `MarkdownDocument` state.

use std::ops::Range;

use crate::model::TableAlignment;
use crate::text_util::clamp_to_char_boundary;

#[derive(Default)]
pub(crate) struct TableDraft {
    pub rows: Vec<Vec<String>>,
    pub alignments: Vec<TableAlignment>,
    pub current_row: Option<Vec<String>>,
    pub current_cell: String,
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
    let line_index = if row_index == 0 { 0 } else { row_index + 1 };
    let line_start = nth_line_start(&formatted, line_index)?;
    let line_end = formatted[line_start..]
        .find('\n')
        .map_or(formatted.len(), |index| line_start + index);
    let line = &formatted[line_start..line_end];
    let columns = table.column_count();
    let column_index = column_index.min(columns.saturating_sub(1));
    let mut content_start = 0usize;
    let mut content_end = 0usize;
    let mut current_column = 0usize;
    let mut in_cell = false;
    let mut seen_space_after_pipe = false;

    for (index, ch) in line.char_indices() {
        if ch == '|' {
            if in_cell {
                if current_column == column_index {
                    content_end = index.saturating_sub(1).max(content_start);
                    break;
                }
                current_column += 1;
            }
            in_cell = true;
            seen_space_after_pipe = false;
            continue;
        }

        if in_cell && !seen_space_after_pipe {
            seen_space_after_pipe = true;
            content_start = index + ch.len_utf8();
            if current_column == column_index && ch != ' ' {
                content_start = index;
            }
        }
    }

    if current_column == column_index && content_end == 0 {
        content_end = line.len().saturating_sub(1).max(content_start);
    }

    Some(line_start + content_start..line_start + content_end)
}

fn nth_line_start(text: &str, line_index: usize) -> Option<usize> {
    if line_index == 0 {
        return Some(0);
    }

    let mut current_line = 0usize;
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            current_line += 1;
            if current_line == line_index {
                return Some(index + 1);
            }
        }
    }

    None
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
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn table_column_at_line(line: &str, byte_column: usize) -> usize {
    let byte_column = clamp_to_char_boundary(line, byte_column.min(line.len()));
    let before_cursor = &line[..byte_column];
    let pipe_count = before_cursor.chars().filter(|ch| *ch == '|').count();
    if line.trim_start().starts_with('|') {
        pipe_count.saturating_sub(1)
    } else {
        pipe_count
    }
}
