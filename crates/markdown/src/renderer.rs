//! Renderer: converts an AST back to well-formatted Markdown source.

use crate::ast::{Alignment, Block, Document, Inline, ListItem, TableCell};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Render a [`Document`] back to Markdown source text.
///
/// The output includes any YAML front matter followed by the block-level content.
pub fn render_to_markdown(doc: &Document) -> String {
    let mut out = String::new();

    // Emit YAML front matter
    if let Some(fm) = &doc.metadata {
        out.push_str("---\n");
        if let Some(title) = &fm.title {
            out.push_str(&format!("title: {}\n", title));
        }
        if let Some(author) = &fm.author {
            out.push_str(&format!("author: {}\n", author));
        }
        if let Some(date) = &fm.date {
            out.push_str(&format!("date: {}\n", date));
        }
        if !fm.tags.is_empty() {
            out.push_str("tags:\n");
            for tag in &fm.tags {
                out.push_str(&format!("  - {}\n", tag));
            }
        }
        // Emit any extra custom keys
        for (key, value) in &fm.custom {
            // Skip keys already emitted above
            if matches!(key.as_str(), "title" | "author" | "date" | "tags") {
                continue;
            }
            if let Ok(s) = serde_yaml::to_string(value) {
                let s = s.trim_end_matches('\n');
                out.push_str(&format!("{}: {}\n", key, s));
            }
        }
        out.push_str("---\n\n");
    }

    // Emit blocks
    render_blocks(&doc.blocks, &mut out, 0);
    out
}

// ---------------------------------------------------------------------------
// Block rendering
// ---------------------------------------------------------------------------

/// Render a slice of blocks into `out`, indented by `indent` spaces.
fn render_blocks(blocks: &[Block], out: &mut String, indent: usize) {
    for (i, block) in blocks.iter().enumerate() {
        render_block(block, out, indent);
        // Separate blocks with a blank line, except inside list items where
        // the caller controls spacing.
        if i + 1 < blocks.len() {
            out.push('\n');
        }
    }
}

fn render_block(block: &Block, out: &mut String, indent: usize) {
    let prefix = " ".repeat(indent);
    match block {
        Block::Heading { level, content, .. } => {
            out.push_str(&prefix);
            for _ in 0..*level {
                out.push('#');
            }
            out.push(' ');
            render_inlines(content, out);
            out.push('\n');
        }

        Block::Paragraph { content, .. } => {
            // Emit each inline; wrap the whole paragraph with a trailing newline.
            out.push_str(&prefix);
            let mut line = String::new();
            render_inlines(content, &mut line);
            // Escape a leading token that would otherwise be reinterpreted as a
            // block-level marker (e.g. `0.` becoming an ordered-list item).
            out.push_str(&escape_leading_block_marker(&line));
            out.push('\n');
        }

        Block::CodeBlock { lang, code, .. } => {
            out.push_str(&prefix);
            out.push_str("```");
            if let Some(l) = lang {
                out.push_str(l);
            }
            out.push('\n');
            // Indent each code line
            for line in code.split('\n') {
                if line.is_empty() {
                    out.push('\n');
                } else {
                    out.push_str(&prefix);
                    out.push_str(line);
                    out.push('\n');
                }
            }
            // Remove extra trailing newline added by the loop above (the code
            // string already ends with '\n' in the common case).
            if out.ends_with("\n\n") && code.ends_with('\n') {
                out.pop();
            }
            out.push_str(&prefix);
            out.push_str("```\n");
        }

        Block::MathBlock { latex, .. } => {
            out.push_str(&prefix);
            out.push_str("$$\n");
            out.push_str(&prefix);
            out.push_str(latex);
            if !latex.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&prefix);
            out.push_str("$$\n");
        }

        Block::Table {
            headers,
            rows,
            alignment,
            ..
        } => {
            render_table(headers, rows, alignment, out, indent);
        }

        Block::List {
            items,
            ordered,
            start,
            ..
        } => {
            render_list(items, *ordered, *start, out, indent);
        }

        Block::BlockQuote { content, .. } => {
            // Each line of the inner content gets a `> ` prefix.
            let mut inner = String::new();
            render_blocks(content, &mut inner, 0);
            for line in inner.lines() {
                out.push_str(&prefix);
                out.push_str("> ");
                out.push_str(line);
                out.push('\n');
            }
        }

        Block::HorizontalRule { .. } => {
            out.push_str(&prefix);
            out.push_str("***\n");
        }

        Block::FootnoteDefinition { label, content, .. } => {
            out.push_str(&prefix);
            out.push_str(&format!("[^{}]: ", label));
            // Render the first block inline if it's a paragraph
            if let Some(Block::Paragraph {
                content: inlines, ..
            }) = content.first()
            {
                render_inlines(inlines, out);
                out.push('\n');
                // Render remaining blocks indented
                if content.len() > 1 {
                    let continuation_indent = indent + 4;
                    for block in &content[1..] {
                        render_block(block, out, continuation_indent);
                    }
                }
            } else {
                out.push('\n');
                let continuation_indent = indent + 4;
                render_blocks(content, out, continuation_indent);
            }
        }

        Block::HtmlBlock { content, .. } => {
            out.push_str(&prefix);
            out.push_str(content);
            if !content.ends_with('\n') {
                out.push('\n');
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Inline rendering
// ---------------------------------------------------------------------------

/// Escape a leading token in `text` that would otherwise be reinterpreted as a
/// block-level marker when the text is placed at the start of a line or of a
/// list item's content.
///
/// The main case is an ordered-list marker: a run of digits followed by `.` or
/// `)` and then whitespace or end-of-string. For example, rendering a list item
/// whose text is `0.` naively produces `1. 0.`, which CommonMark re-parses as a
/// *nested* ordered list. Inserting a backslash (`1. 0\.`) keeps it as literal
/// text that round-trips back to the same AST.
fn escape_leading_block_marker(text: &str) -> String {
    let bytes = text.as_bytes();

    // Count leading ASCII digits.
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }

    // CommonMark ordered-list markers allow at most 9 digits.
    if i > 0 && i <= 9 && i < bytes.len() && (bytes[i] == b'.' || bytes[i] == b')') {
        let after = i + 1;
        let is_marker = after == bytes.len() || bytes[after] == b' ' || bytes[after] == b'\t';
        if is_marker {
            let mut escaped = String::with_capacity(text.len() + 1);
            escaped.push_str(&text[..i]);
            escaped.push('\\');
            escaped.push_str(&text[i..]);
            return escaped;
        }
    }

    text.to_string()
}

fn render_inlines(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        render_inline(inline, out);
    }
}

fn render_inline(inline: &Inline, out: &mut String) {
    match inline {
        Inline::Text(s) => {
            out.push_str(s);
        }
        Inline::Strong(inner) => {
            out.push_str("**");
            render_inlines(inner, out);
            out.push_str("**");
        }
        Inline::Emphasis(inner) => {
            out.push('*');
            render_inlines(inner, out);
            out.push('*');
        }
        Inline::Strikethrough(inner) => {
            out.push_str("~~");
            render_inlines(inner, out);
            out.push_str("~~");
        }
        Inline::Code(s) => {
            out.push('`');
            out.push_str(s);
            out.push('`');
        }
        Inline::InlineMath(s) => {
            out.push('$');
            out.push_str(s);
            out.push('$');
        }
        Inline::Link { text, url, title } => {
            out.push('[');
            render_inlines(text, out);
            out.push_str("](");
            out.push_str(url);
            if let Some(t) = title {
                out.push_str(" \"");
                out.push_str(t);
                out.push('"');
            }
            out.push(')');
        }
        Inline::Image { alt, url, title } => {
            out.push_str("![");
            out.push_str(alt);
            out.push_str("](");
            out.push_str(url);
            if let Some(t) = title {
                out.push_str(" \"");
                out.push_str(t);
                out.push('"');
            }
            out.push(')');
        }
        Inline::LineBreak => {
            out.push_str("  \n");
        }

        Inline::Superscript(inner) => {
            out.push('^');
            render_inlines(inner, out);
            out.push('^');
        }

        Inline::Subscript(inner) => {
            out.push('~');
            render_inlines(inner, out);
            out.push('~');
        }

        Inline::Highlight(inner) => {
            out.push_str("==");
            render_inlines(inner, out);
            out.push_str("==");
        }

        Inline::Emoji {
            shortcode: _,
            unicode,
        } => {
            // Prefer unicode for rendering, but could use shortcode with a flag
            out.push_str(unicode);
        }

        Inline::FootnoteReference(label) => {
            out.push_str(&format!("[^{}]", label));
        }

        Inline::HtmlInline(html) => {
            out.push_str(html);
        }
    }
}

// ---------------------------------------------------------------------------
// Table rendering
// ---------------------------------------------------------------------------

fn render_table(
    headers: &[TableCell],
    rows: &[Vec<TableCell>],
    alignment: &[Alignment],
    out: &mut String,
    indent: usize,
) {
    let prefix = " ".repeat(indent);
    let col_count = headers.len();

    // Compute max width for each column for pretty-printing
    let mut col_widths: Vec<usize> = headers.iter().map(|c| cell_text_width(c)).collect();

    for row in rows {
        for (ci, cell) in row.iter().enumerate() {
            let w = cell_text_width(cell);
            if ci < col_widths.len() {
                col_widths[ci] = col_widths[ci].max(w);
            }
        }
    }
    // Ensure minimum separator width (3 chars for `:--` / `---`)
    for w in &mut col_widths {
        *w = (*w).max(3);
    }

    // Header row
    out.push_str(&prefix);
    out.push('|');
    for (ci, cell) in headers.iter().enumerate() {
        let w = col_widths.get(ci).copied().unwrap_or(3);
        let text = cell_to_string(cell);
        out.push_str(&format!(" {:<width$} |", text, width = w));
    }
    out.push('\n');

    // Separator row
    out.push_str(&prefix);
    out.push('|');
    for ci in 0..col_count {
        let w = col_widths.get(ci).copied().unwrap_or(3);
        let align = alignment.get(ci).copied().unwrap_or(Alignment::None);
        let sep = match align {
            Alignment::None | Alignment::Left => format!(" :{} |", "-".repeat(w - 1)),
            Alignment::Center => format!(" :{}:  |", "-".repeat(w.saturating_sub(2))),
            Alignment::Right => format!(" {}: |", "-".repeat(w - 1)),
        };
        out.push_str(&sep);
    }
    out.push('\n');

    // Data rows
    for row in rows {
        out.push_str(&prefix);
        out.push('|');
        for ci in 0..col_count {
            let w = col_widths.get(ci).copied().unwrap_or(3);
            let text = row.get(ci).map(cell_to_string).unwrap_or_default();
            out.push_str(&format!(" {:<width$} |", text, width = w));
        }
        out.push('\n');
    }
}

fn cell_to_string(cell: &TableCell) -> String {
    let mut s = String::new();
    render_inlines(&cell.content, &mut s);
    s
}

fn cell_text_width(cell: &TableCell) -> usize {
    cell_to_string(cell).chars().count()
}

// ---------------------------------------------------------------------------
// List rendering
// ---------------------------------------------------------------------------

fn render_list(
    items: &[ListItem],
    ordered: bool,
    start: Option<u32>,
    out: &mut String,
    indent: usize,
) {
    let prefix = " ".repeat(indent);
    let mut counter = start.unwrap_or(1);

    for item in items {
        // Build the bullet/number marker
        let marker = if ordered {
            format!("{}. ", counter)
        } else {
            "- ".to_string()
        };
        counter += 1;

        out.push_str(&prefix);
        out.push_str(&marker);

        // Task list checkbox
        if let Some(checked) = item.checked {
            if checked {
                out.push_str("[x] ");
            } else {
                out.push_str("[ ] ");
            }
        }

        // Inline content on the same line as the bullet. Escape a leading
        // token that would otherwise be reinterpreted as a nested block-level
        // marker (e.g. an item whose text is `0.` turning into a nested list).
        let mut content = String::new();
        render_inlines(&item.content, &mut content);
        out.push_str(&escape_leading_block_marker(&content));
        out.push('\n');

        // Block content (indented continuation)
        let continuation_indent = indent + marker.len();
        for block in &item.blocks {
            render_block(block, out, continuation_indent);
        }

        // Nested sub-items
        if !item.sub_items.is_empty() {
            // Sub-items are indented by 2 spaces (standard Markdown)
            render_list(&item.sub_items, false, None, out, indent + 2);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Alignment, Block, Document, Inline, ListItem, TableCell};
    use crate::parser::{Parser, ParserOptions};

    fn make_para(text: &str) -> Block {
        Block::Paragraph {
            content: vec![Inline::Text(text.into())],
            id: 0,
        }
    }

    // --- Headings ---

    #[test]
    fn render_heading_levels() {
        for level in 1u8..=6 {
            let doc = Document::new(vec![Block::Heading {
                level,
                content: vec![Inline::Text("Title".into())],
                id: 0,
            }]);
            let md = render_to_markdown(&doc);
            let expected_prefix = "#".repeat(level as usize) + " Title";
            assert!(
                md.starts_with(&expected_prefix),
                "level {level}: got {md:?}"
            );
        }
    }

    // --- Paragraph ---

    #[test]
    fn render_paragraph() {
        let doc = Document::new(vec![make_para("Hello world")]);
        let md = render_to_markdown(&doc);
        assert_eq!(md.trim(), "Hello world");
    }

    // --- Inline elements ---

    #[test]
    fn render_strong() {
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Strong(vec![Inline::Text("bold".into())])],
            id: 0,
        }]);
        assert!(render_to_markdown(&doc).contains("**bold**"));
    }

    #[test]
    fn render_emphasis() {
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Emphasis(vec![Inline::Text("em".into())])],
            id: 0,
        }]);
        assert!(render_to_markdown(&doc).contains("*em*"));
    }

    #[test]
    fn render_strikethrough() {
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Strikethrough(vec![Inline::Text("del".into())])],
            id: 0,
        }]);
        assert!(render_to_markdown(&doc).contains("~~del~~"));
    }

    #[test]
    fn render_code_inline() {
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Code("x = 1".into())],
            id: 0,
        }]);
        assert!(render_to_markdown(&doc).contains("`x = 1`"));
    }

    #[test]
    fn render_link() {
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Link {
                text: vec![Inline::Text("click".into())],
                url: "https://example.com".into(),
                title: None,
            }],
            id: 0,
        }]);
        assert!(render_to_markdown(&doc).contains("[click](https://example.com)"));
    }

    #[test]
    fn render_image() {
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Image {
                alt: "logo".into(),
                url: "img.png".into(),
                title: None,
            }],
            id: 0,
        }]);
        assert!(render_to_markdown(&doc).contains("![logo](img.png)"));
    }

    // --- Code block ---

    #[test]
    fn render_code_block_preserves_whitespace() {
        let code = "  indented\n\ttabbed\n";
        let doc = Document::new(vec![Block::CodeBlock {
            lang: Some("rust".into()),
            code: code.to_string(),
            id: 0,
        }]);
        let md = render_to_markdown(&doc);
        assert!(md.contains("```rust"));
        assert!(md.contains("  indented"));
        assert!(md.contains("\ttabbed"));
    }

    // --- Horizontal rule ---

    #[test]
    fn render_horizontal_rule() {
        let doc = Document::new(vec![Block::HorizontalRule { id: 0 }]);
        assert!(render_to_markdown(&doc).contains("***"));
    }

    // --- Block quote ---

    #[test]
    fn render_block_quote() {
        let doc = Document::new(vec![Block::BlockQuote {
            content: vec![make_para("quoted")],
            id: 0,
        }]);
        assert!(render_to_markdown(&doc).contains("> quoted"));
    }

    // --- Lists ---

    #[test]
    fn render_unordered_list() {
        let doc = Document::new(vec![Block::List {
            items: vec![
                ListItem::simple(vec![Inline::Text("a".into())]),
                ListItem::simple(vec![Inline::Text("b".into())]),
            ],
            ordered: false,
            start: None,
            id: 0,
        }]);
        let md = render_to_markdown(&doc);
        assert!(md.contains("- a"));
        assert!(md.contains("- b"));
    }

    #[test]
    fn render_ordered_list() {
        let doc = Document::new(vec![Block::List {
            items: vec![
                ListItem::simple(vec![Inline::Text("first".into())]),
                ListItem::simple(vec![Inline::Text("second".into())]),
            ],
            ordered: true,
            start: Some(1),
            id: 0,
        }]);
        let md = render_to_markdown(&doc);
        assert!(md.contains("1. first"));
        assert!(md.contains("2. second"));
    }

    #[test]
    fn render_task_list() {
        let doc = Document::new(vec![Block::List {
            items: vec![
                ListItem::task(vec![Inline::Text("done".into())], true),
                ListItem::task(vec![Inline::Text("todo".into())], false),
            ],
            ordered: false,
            start: None,
            id: 0,
        }]);
        let md = render_to_markdown(&doc);
        assert!(md.contains("- [x] done"));
        assert!(md.contains("- [ ] todo"));
    }

    // --- Table ---

    #[test]
    fn render_table() {
        let doc = Document::new(vec![Block::Table {
            headers: vec![TableCell::text("A"), TableCell::text("B")],
            rows: vec![vec![TableCell::text("1"), TableCell::text("2")]],
            alignment: vec![Alignment::Left, Alignment::Right],
            id: 0,
        }]);
        let md = render_to_markdown(&doc);
        assert!(md.contains("| A "));
        assert!(md.contains("| B "));
        assert!(md.contains("| 1 "));
        assert!(md.contains("| 2 "));
    }

    // --- YAML front matter ---

    #[test]
    fn render_yaml_front_matter() {
        use crate::ast::YamlFrontMatter;
        use std::collections::HashMap;
        let fm = YamlFrontMatter {
            title: Some("Test".into()),
            author: None,
            date: None,
            tags: vec!["a".into()],
            custom: HashMap::new(),
        };
        let mut doc = Document::new(vec![]);
        doc.metadata = Some(fm);
        let md = render_to_markdown(&doc);
        assert!(md.starts_with("---\n"));
        assert!(md.contains("title: Test"));
        assert!(md.contains("  - a"));
        assert!(md.contains("---\n"));
    }

    // --- Round-trip test ---

    #[test]
    fn round_trip_basic_document() {
        let original = "# Hello\n\nThis is a **bold** paragraph.\n\n- item 1\n- item 2\n";
        let parser = Parser::new(ParserOptions::default());
        let doc1 = parser.parse(original).unwrap();
        let rendered = render_to_markdown(&doc1);
        let doc2 = parser.parse(&rendered).unwrap();

        // Both documents should have the same number of blocks
        assert_eq!(doc1.blocks.len(), doc2.blocks.len());

        // First block should be a heading in both
        assert!(matches!(doc1.blocks[0], Block::Heading { level: 1, .. }));
        assert!(matches!(doc2.blocks[0], Block::Heading { level: 1, .. }));
    }
}
