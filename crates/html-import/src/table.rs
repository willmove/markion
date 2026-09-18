//! GFM table emission shared by HTML table rendering and TSV clipboard paste.

use crate::escape::escape_markdown_text;

/// Converts rectangular tab-separated plain text into a GFM table whose first
/// row is the header. Returns `None` when the payload is not a ≥2-row
/// rectangle with a tab on every line (including a single cell).
pub fn tsv_to_markdown(text: &str) -> Option<String> {
    let lines = tsv_lines(text)?;
    if lines.len() < 2 || lines.iter().any(|line| !line.contains('\t')) {
        return None;
    }
    let rows: Vec<Vec<String>> = lines
        .iter()
        .map(|line| {
            line.split('\t')
                .map(normalize_tsv_cell)
                .collect::<Vec<String>>()
        })
        .collect();
    let width = rows.first()?.len();
    if width == 0 || rows.iter().any(|row| row.len() != width) {
        return None;
    }
    Some(emit_gfm_table(&rows))
}

fn tsv_lines(text: &str) -> Option<Vec<&str>> {
    let trimmed = text.trim_end_matches(['\n', '\r']);
    if trimmed.is_empty() {
        return None;
    }
    let mut lines: Vec<&str> = trimmed.lines().collect();
    while lines.first().is_some_and(|line| line.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    if lines.is_empty() { None } else { Some(lines) }
}

fn normalize_tsv_cell(value: &str) -> String {
    escape_markdown_text(value)
        .replace('|', "\\|")
        .replace('\n', "<br>")
}

/// Emits a GFM pipe table. `rows[0]` is the header; later rows are the body.
/// Cells are assumed to already be escaped for pipes and Markdown.
pub(crate) fn emit_gfm_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    out.push_str(&gfm_row(&rows[0], width));
    out.push('\n');
    out.push('|');
    for _ in 0..width {
        out.push_str(" --- |");
    }
    for row in &rows[1..] {
        out.push('\n');
        out.push_str(&gfm_row(row, width));
    }
    out
}

pub(crate) fn gfm_row(cells: &[String], width: usize) -> String {
    let mut row = String::from("|");
    for index in 0..width {
        row.push(' ');
        row.push_str(cells.get(index).map(String::as_str).unwrap_or(""));
        row.push_str(" |");
    }
    row
}
