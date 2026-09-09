//! DOM to Markdown rendering. Mirrors the tag and marker vocabulary of the
//! repository's JavaScript `htmlToMarkdown` reference
//! (`assets/marknice-workspace/static/marknice-word-import-runtime.js`) and
//! the escaping policy of `markion-docx-import`.

use crate::dom::{Element, Node, is_void_tag};
use crate::escape::{
    escape_html_attr, escape_html_text, escape_line_start_marker, escape_markdown_label,
    escape_markdown_text, escape_markdown_url,
};

/// Elements whose entire subtree is noise for document content. Checkbox
/// inputs are read by the list renderer before this drop applies.
const DROPPED_SUBTREES: &[&str] = &[
    "button", "embed", "form", "head", "iframe", "input", "link", "meta", "noscript", "object",
    "select", "svg", "template", "title", "xml",
];

pub(crate) fn render_document(document: &Element) -> String {
    let mut renderer = Renderer {
        out: String::new(),
        line_start: true,
    };
    render_children(&mut renderer, &document.children);
    let merged = merge_adjacent_markers(&renderer.out);
    let collapsed = collapse_blank_lines(&merged);
    collapsed.trim().to_owned()
}

struct Renderer {
    out: String,
    /// Whether the next emitted character begins a line; gates line-start
    /// block-marker escaping.
    line_start: bool,
}

fn render_children(r: &mut Renderer, children: &[Node]) {
    for child in children {
        render_node(r, child);
    }
}

fn render_node(r: &mut Renderer, node: &Node) {
    match node {
        Node::Text(text) => push_text(r, text),
        Node::Element(element) => render_element(r, element),
    }
}

fn render_element(r: &mut Renderer, element: &Element) {
    let tag = element.tag.as_str();
    if DROPPED_SUBTREES.contains(&tag) {
        return;
    }
    if is_block_tag(tag) {
        ensure_block_start(r);
    }
    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = usize::from(tag.as_bytes()[1] - b'0');
            render_heading(r, element, level);
        }
        "p" | "div" | "section" | "article" | "header" | "footer" | "main" | "aside" | "figure"
        | "figcaption" | "address" | "center" | "o:p" => render_paragraph(r, element),
        "br" => {
            r.out.push('\n');
            r.line_start = true;
        }
        "hr" => {
            r.out.push_str("---\n\n");
            r.line_start = true;
        }
        "strong" | "b" | "em" | "i" | "s" | "strike" | "del" | "u" | "sub" | "sup" => {
            render_formatting(r, element);
        }
        "span" | "font" => {
            let style = parse_inline_style(element);
            let (open, close) = combined_wrappers(&style, false, false, false, false);
            wrap_children(r, &element.children, &open, &close);
        }
        "a" => render_link(r, element),
        "img" => render_image(r, element),
        "ul" => render_list(r, element, false),
        "ol" => render_list(r, element, true),
        // Stray `li` outside a list container degrades to a bullet.
        "li" => {
            render_li(r, &element.children, "- ");
        }
        "blockquote" => render_blockquote(r, element),
        "pre" => render_pre(r, element),
        "code" => render_inline_code(r, element),
        "table" => render_table(r, element),
        // Unknown tags are transparent: keep their children.
        _ => render_children(r, &element.children),
    }
}

fn is_block_tag(tag: &str) -> bool {
    matches!(
        tag,
        "h1" | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "p"
            | "div"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "main"
            | "aside"
            | "figure"
            | "figcaption"
            | "address"
            | "center"
            | "o:p"
            | "hr"
            | "ul"
            | "ol"
            | "li"
            | "blockquote"
            | "pre"
            | "table"
    )
}

/// Block-level children always start on their own line, even after inline
/// content (e.g. a nested list right after text in an `li`).
fn ensure_block_start(r: &mut Renderer) {
    if !r.out.is_empty() && !r.out.ends_with('\n') {
        r.out.push('\n');
    }
    r.line_start = true;
}

fn push_text(r: &mut Renderer, text: &str) {
    let mut collapsed = collapse_whitespace(text);
    // Boundary normalization: a leading space is dropped at line starts and
    // after existing whitespace so inline elements never see doubled spaces.
    if collapsed.starts_with(' ') && (r.out.is_empty() || r.out.ends_with([' ', '\n'])) {
        collapsed.drain(..1);
    }
    if collapsed.is_empty() {
        return;
    }
    if collapsed == " " {
        // A lone space neither confirms nor breaks a line start.
        r.out.push(' ');
        return;
    }
    let escaped = escape_markdown_text(&collapsed);
    let escaped = if r.line_start {
        escape_line_start_marker(&escaped)
    } else {
        escaped
    };
    r.out.push_str(&escaped);
    r.line_start = false;
}

fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for ch in text.chars() {
        if matches!(ch, ' ' | '\t' | '\n' | '\r' | '\u{0C}') {
            pending_space = true;
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(ch);
        }
    }
    if pending_space {
        out.push(' ');
    }
    out
}

/// Renders children into a scratch region and hands back the trimmed result,
/// leaving `r.out` untouched when the block carries no content.
fn block_body(r: &mut Renderer, element: &Element) -> String {
    let start = r.out.len();
    let saved = r.line_start;
    r.line_start = true;
    render_children(r, &element.children);
    let body = r.out[start..].trim().to_owned();
    r.out.truncate(start);
    r.line_start = saved;
    body
}

fn finish_block(r: &mut Renderer) {
    r.out.push_str("\n\n");
    r.line_start = true;
}

fn render_heading(r: &mut Renderer, element: &Element, level: usize) {
    let body = block_body(r, element);
    if body.is_empty() {
        return;
    }
    for _ in 0..level {
        r.out.push('#');
    }
    r.out.push(' ');
    r.out.push_str(&body);
    finish_block(r);
}

fn render_paragraph(r: &mut Renderer, element: &Element) {
    let body = block_body(r, element);
    if body.is_empty() {
        return;
    }
    r.out.push_str(&body);
    finish_block(r);
}

fn render_blockquote(r: &mut Renderer, element: &Element) {
    let body = block_body(r, element);
    if body.is_empty() {
        return;
    }
    for line in body.split('\n') {
        if line.trim().is_empty() {
            r.out.push('>');
        } else {
            r.out.push_str("> ");
            r.out.push_str(line);
        }
        r.out.push('\n');
    }
    r.out.push('\n');
    r.line_start = true;
}

/// Wraps children in paired markers, skipping empty wrappers and trimming
/// whitespace that would otherwise glue to the markers. A wrapper holding
/// only whitespace still separates the words around it.
fn wrap_children(r: &mut Renderer, children: &[Node], open: &str, close: &str) {
    if open.is_empty() {
        render_children(r, children);
        return;
    }
    let start = r.out.len();
    let saved = r.line_start;
    render_children(r, children);
    let region = r.out[start..].to_owned();
    let inner = region.trim();
    r.out.truncate(start);
    if inner.is_empty() {
        r.line_start = saved;
        if !region.is_empty() && !r.out.is_empty() && !r.out.ends_with([' ', '\n']) {
            r.out.push(' ');
        }
        return;
    }
    r.out.push_str(open);
    r.out.push_str(inner);
    r.out.push_str(close);
    r.line_start = false;
}

#[derive(Default)]
struct InlineStyle {
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strike: Option<bool>,
}

/// Parses the `style` attribute into per-property formatting decisions. An
/// explicitly declared property overrides the tag's implied formatting in
/// both directions (`<b style="font-weight:normal">` is not bold, a
/// `<span style="font-weight:700">` is).
fn parse_inline_style(element: &Element) -> InlineStyle {
    let mut style = InlineStyle::default();
    let Some(attr) = element.attr("style") else {
        return style;
    };
    for declaration in attr.split(';') {
        let Some((key, value)) = declaration.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim().to_ascii_lowercase();
        match key.as_str() {
            "font-weight" => {
                let token = value.split_whitespace().next().unwrap_or("");
                style.bold = Some(match token {
                    "bold" | "bolder" => true,
                    _ => token.parse::<u32>().map(|w| w >= 600).unwrap_or(false),
                });
            }
            "mso-bidi-font-weight" => {
                if style.bold.is_none() {
                    style.bold = Some(value.split_whitespace().next() == Some("bold"));
                }
            }
            "font-style" => {
                let token = value.split_whitespace().next().unwrap_or("");
                style.italic = Some(matches!(token, "italic" | "oblique"));
            }
            "text-decoration" | "text-decoration-line" => {
                style.underline = Some(value.contains("underline"));
                style.strike = Some(value.contains("line-through"));
            }
            _ => {}
        }
    }
    style
}

/// Builds the open/close marker pair, outermost first: strong, em, strike,
/// underline. Style flags override the tag implications per property, so a
/// wrapper is never emitted twice for the same effect.
fn combined_wrappers(
    style: &InlineStyle,
    tag_bold: bool,
    tag_italic: bool,
    tag_strike: bool,
    tag_underline: bool,
) -> (String, String) {
    let mut open = String::new();
    let mut close = String::new();
    if style.bold.unwrap_or(tag_bold) {
        open.push_str("**");
        close.insert_str(0, "**");
    }
    if style.italic.unwrap_or(tag_italic) {
        open.push('*');
        close.insert(0, '*');
    }
    if style.strike.unwrap_or(tag_strike) {
        open.push_str("~~");
        close.insert_str(0, "~~");
    }
    if style.underline.unwrap_or(tag_underline) {
        open.push_str("<u>");
        close.insert_str(0, "</u>");
    }
    (open, close)
}

fn render_formatting(r: &mut Renderer, element: &Element) {
    let tag = element.tag.as_str();
    let style = parse_inline_style(element);
    let (mut open, mut close) = combined_wrappers(
        &style,
        matches!(tag, "strong" | "b"),
        matches!(tag, "em" | "i"),
        matches!(tag, "s" | "strike" | "del"),
        tag == "u",
    );
    // Sub/superscript carry no style sources and sit innermost.
    match tag {
        "sub" => {
            open.push_str("<sub>");
            close.insert_str(0, "</sub>");
        }
        "sup" => {
            open.push_str("<sup>");
            close.insert_str(0, "</sup>");
        }
        _ => {}
    }
    wrap_children(r, &element.children, &open, &close);
}

/// `http(s)`, `mailto`, `ftp`, scheme-relative, schemeless, and `#fragment`
/// targets are kept; anything else (e.g. `javascript:`) drops to plain text.
fn safe_href(href: &str) -> bool {
    if href.starts_with('#') || href.starts_with("//") {
        return true;
    }
    if let Some(colon) = href.find(':') {
        let scheme = &href[..colon];
        let valid = !scheme.is_empty()
            && scheme.chars().enumerate().all(|(index, ch)| {
                if index == 0 {
                    ch.is_ascii_alphabetic()
                } else {
                    ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.')
                }
            });
        if valid {
            return matches!(
                scheme.to_ascii_lowercase().as_str(),
                "http" | "https" | "mailto" | "ftp"
            );
        }
    }
    true
}

fn render_link(r: &mut Renderer, element: &Element) {
    let href = element.attr("href").unwrap_or("").trim();
    let start = r.out.len();
    let saved = r.line_start;
    render_children(r, &element.children);
    let text = r.out[start..].trim().to_owned();
    r.out.truncate(start);
    r.line_start = saved;
    let label = if text.is_empty() {
        escape_markdown_label(href)
    } else {
        text
    };
    let body = if !href.is_empty() && safe_href(href) {
        format!("[{label}]({})", escape_markdown_url(href))
    } else {
        label
    };
    if body.is_empty() {
        return;
    }
    let style = parse_inline_style(element);
    let (open, close) = combined_wrappers(&style, false, false, false, false);
    r.out.push_str(&open);
    r.out.push_str(&body);
    r.out.push_str(&close);
    r.line_start = false;
}

fn render_image(r: &mut Renderer, element: &Element) {
    let src = element.attr("src").unwrap_or("").trim();
    let alt = collapse_whitespace(element.attr("alt").unwrap_or(""));
    let alt = escape_markdown_label(alt.trim());
    if src.is_empty() || src.to_ascii_lowercase().starts_with("data:") {
        // `data:` URIs degrade to their alt text instead of inlining
        // megabytes of base64 into the document.
        if !alt.is_empty() {
            r.out.push_str(&alt);
            r.line_start = false;
        }
        return;
    }
    r.out
        .push_str(&format!("![{alt}]({})", escape_markdown_url(src)));
    r.line_start = false;
}

fn render_list(r: &mut Renderer, element: &Element, ordered: bool) {
    let start = r.out.len();
    let saved = r.line_start;
    let mut number = if ordered {
        element
            .attr("start")
            .and_then(|value| value.trim().parse::<i64>().ok())
            .unwrap_or(1)
    } else {
        0
    };
    let mut produced = false;
    r.line_start = true;
    for child in &element.children {
        match child {
            Node::Element(li) if li.tag == "li" => {
                let (checkbox, rest) = split_checkbox(&li.children);
                let marker = if let Some(checked) = checkbox {
                    if checked { "- [x] " } else { "- [ ] " }.to_owned()
                } else if ordered {
                    let marker = format!("{number}. ");
                    number += 1;
                    marker
                } else {
                    "- ".to_owned()
                };
                produced |= render_li(r, rest, &marker);
            }
            Node::Text(text) if text.trim().is_empty() => {}
            // Stray content directly inside a list container is kept as its
            // own block rather than dropped.
            _ => {
                let node_start = r.out.len();
                render_node(r, child);
                let body = r.out[node_start..].trim().to_owned();
                r.out.truncate(node_start);
                if !body.is_empty() {
                    r.out.push_str(&body);
                    r.out.push('\n');
                    produced = true;
                }
            }
        }
    }
    if produced {
        r.out.push('\n');
    } else {
        r.out.truncate(start);
        r.line_start = saved;
        return;
    }
    r.line_start = true;
}

/// Detects a leading task-list checkbox (`<li><input type="checkbox"
/// checked>…`) and returns the checked state plus the remaining children.
fn split_checkbox(children: &[Node]) -> (Option<bool>, &[Node]) {
    for (index, child) in children.iter().enumerate() {
        match child {
            Node::Text(text) if text.trim().is_empty() => continue,
            Node::Element(element)
                if element.tag == "input"
                    && element
                        .attr("type")
                        .is_some_and(|value| value.eq_ignore_ascii_case("checkbox")) =>
            {
                return (
                    Some(element.attr("checked").is_some()),
                    &children[index + 1..],
                );
            }
            _ => return (None, children),
        }
    }
    (None, children)
}

/// Renders one list item: first line carries the marker, continuation lines
/// are indented by the marker width (the CommonMark rule), and blank lines
/// between blocks stay blank. Returns whether anything was emitted.
fn render_li(r: &mut Renderer, children: &[Node], marker: &str) -> bool {
    let start = r.out.len();
    let saved = r.line_start;
    r.line_start = true;
    render_children(r, children);
    let body = r.out[start..].trim().to_owned();
    r.out.truncate(start);
    if body.is_empty() {
        r.line_start = saved;
        return false;
    }
    let indent = " ".repeat(marker.len());
    for (index, line) in body.split('\n').enumerate() {
        if index == 0 {
            r.out.push_str(marker);
            r.out.push_str(line);
        } else if !line.trim().is_empty() {
            r.out.push_str(&indent);
            r.out.push_str(line);
        }
        r.out.push('\n');
    }
    r.line_start = true;
    true
}

fn render_pre(r: &mut Renderer, element: &Element) {
    let mut content = raw_text_of(element);
    // Strip one leading and one trailing newline (browsers emit a newline
    // right after the `<pre>` open tag).
    if let Some(stripped) = content.strip_prefix("\r\n") {
        content = stripped.to_owned();
    } else if let Some(stripped) = content.strip_prefix('\n') {
        content = stripped.to_owned();
    }
    if content.ends_with("\r\n") {
        content.truncate(content.len() - 2);
    } else if content.ends_with('\n') {
        content.truncate(content.len() - 1);
    }
    if content.trim().is_empty() {
        return;
    }
    let language = element
        .children
        .iter()
        .find_map(|child| match child {
            Node::Element(code) if code.tag == "code" => code.attr("class").and_then(|class| {
                class.split_whitespace().find_map(|token| {
                    token.strip_prefix("language-").map(|hint| {
                        hint.chars()
                            .take_while(|ch| {
                                ch.is_ascii_alphanumeric()
                                    || matches!(ch, '_' | '+' | '#' | '.' | '-')
                            })
                            .collect::<String>()
                    })
                })
            }),
            _ => None,
        })
        .unwrap_or_default();
    let fence = if content.contains("```") {
        "~~~"
    } else {
        "```"
    };
    r.out.push_str(fence);
    r.out.push_str(&language);
    r.out.push('\n');
    r.out.push_str(&content);
    r.out.push('\n');
    r.out.push_str(fence);
    finish_block(r);
}

fn render_inline_code(r: &mut Renderer, element: &Element) {
    let content = collapse_whitespace(&raw_text_of(element));
    let content = content.trim();
    if content.is_empty() {
        return;
    }
    let mut longest = 0usize;
    let mut run = 0usize;
    for ch in content.chars() {
        if ch == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    let fence = "`".repeat(longest + 1);
    let pad = content.starts_with('`') || content.ends_with('`');
    r.out.push_str(&fence);
    if pad {
        r.out.push(' ');
    }
    r.out.push_str(content);
    if pad {
        r.out.push(' ');
    }
    r.out.push_str(&fence);
    r.line_start = false;
}

/// Raw descendant text with entity decoding already applied and no
/// whitespace collapsing or Markdown escaping (the `pre`/`code` policy).
fn raw_text_of(element: &Element) -> String {
    let mut out = String::new();
    collect_raw_text(&element.children, &mut out);
    out
}

fn collect_raw_text(children: &[Node], out: &mut String) {
    for child in children {
        match child {
            Node::Text(text) => out.push_str(text),
            Node::Element(element) => {
                if element.tag == "br" {
                    out.push('\n');
                } else if !DROPPED_SUBTREES.contains(&element.tag.as_str()) {
                    collect_raw_text(&element.children, out);
                }
            }
        }
    }
}

fn render_table(r: &mut Renderer, element: &Element) {
    let rows = collect_rows(element);
    if rows.is_empty() {
        return;
    }
    let merged = rows
        .iter()
        .flatten()
        .any(|cell| span_of(cell, "colspan") > 1 || span_of(cell, "rowspan") > 1);
    // A table nested inside a cell cannot survive pipe-table rendering; the
    // whole table degrades to raw HTML like merged cells do.
    let nested = rows.iter().flatten().any(|cell| contains_table(cell));
    if merged || nested {
        let mut raw = String::new();
        serialize_raw(element, &mut raw);
        r.out.push_str(&raw);
        finish_block(r);
        return;
    }
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    if width == 0 {
        return;
    }
    let header = rows
        .iter()
        .position(|row| row.iter().any(|cell| cell.tag == "th"));
    let header_cells = match header {
        Some(index) => render_row_cells(r, &rows[index]),
        None => vec![String::new(); width],
    };
    r.out.push_str(&gfm_row(&header_cells, width));
    r.out.push('\n');
    r.out.push('|');
    for _ in 0..width {
        r.out.push_str(" --- |");
    }
    r.out.push('\n');
    for (index, row) in rows.iter().enumerate() {
        if Some(index) == header {
            continue;
        }
        let cells = render_row_cells(r, row);
        r.out.push_str(&gfm_row(&cells, width));
        r.out.push('\n');
    }
    r.out.push('\n');
    r.line_start = true;
}

fn collect_rows(table: &Element) -> Vec<Vec<&Element>> {
    let mut rows = Vec::new();
    for child in &table.children {
        let Node::Element(section) = child else {
            continue;
        };
        match section.tag.as_str() {
            "tr" => rows.push(collect_cells(section)),
            "thead" | "tbody" | "tfoot" => {
                for grandchild in &section.children {
                    if let Node::Element(row) = grandchild
                        && row.tag == "tr"
                    {
                        rows.push(collect_cells(row));
                    }
                }
            }
            _ => {}
        }
    }
    rows
}

fn collect_cells(row: &Element) -> Vec<&Element> {
    row.children
        .iter()
        .filter_map(|child| match child {
            Node::Element(cell) if cell.tag == "td" || cell.tag == "th" => Some(cell),
            _ => None,
        })
        .collect()
}

fn span_of(cell: &Element, attr: &str) -> usize {
    cell.attr(attr)
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(1)
        .max(1)
}

fn contains_table(element: &Element) -> bool {
    element.children.iter().any(|child| match child {
        Node::Element(child) => child.tag == "table" || contains_table(child),
        Node::Text(_) => false,
    })
}

/// Cells are inline-rendered Markdown; the `normalize_cell_text` policy then
/// protects pipes and folds newlines into `<br>`. Markdown-special
/// characters are already escaped at text-emission time.
fn render_row_cells(r: &mut Renderer, row: &[&Element]) -> Vec<String> {
    row.iter()
        .map(|cell| {
            let start = r.out.len();
            let saved = r.line_start;
            r.line_start = true;
            render_children(r, &cell.children);
            let body = r.out[start..].trim().to_owned();
            r.out.truncate(start);
            r.line_start = saved;
            body.replace('|', "\\|").replace('\n', "<br>")
        })
        .collect()
}

fn gfm_row(cells: &[String], width: usize) -> String {
    let mut row = String::from("|");
    for index in 0..width {
        row.push(' ');
        row.push_str(cells.get(index).map(String::as_str).unwrap_or(""));
        row.push_str(" |");
    }
    row
}

/// Re-serializes an element subtree back to single-line raw HTML (used for
/// merged/nested table fallback). Attribute order and values are preserved
/// and open/close tags match.
fn serialize_raw(element: &Element, out: &mut String) {
    out.push('<');
    out.push_str(&element.tag);
    for (name, value) in &element.attrs {
        out.push(' ');
        out.push_str(name);
        out.push_str("=\"");
        out.push_str(&escape_html_attr(&value.replace(['\n', '\r'], " ")));
        out.push('"');
    }
    out.push('>');
    if is_void_tag(&element.tag) {
        return;
    }
    for child in &element.children {
        match child {
            Node::Text(text) => {
                out.push_str(&escape_html_text(&text.replace(['\n', '\r'], " ")));
            }
            Node::Element(child) => serialize_raw(child, out),
        }
    }
    out.push_str("</");
    out.push_str(&element.tag);
    out.push('>');
}

/// Merges adjacent same markers (`**A****B**` → `**AB**`, `~~A~~~~B~~` →
/// `~~AB~~`, `*A**B*` → `*AB*`), porting the JS reference passes. Fenced
/// code blocks are skipped entirely and inline code spans are opaque, so
/// literal marker text inside code is never rewritten.
fn merge_adjacent_markers(input: &str) -> String {
    transform_outside_fences(input, |segment| {
        let mut current = segment.to_owned();
        for marker in ["**", "*", "~~"] {
            for _ in 0..8 {
                let next = match marker {
                    "*" => merge_single_marker(&current),
                    _ => merge_double_marker(&current, marker.chars().next().unwrap_or('*')),
                };
                if next == current {
                    break;
                }
                current = next;
            }
        }
        current
    })
}

fn merge_double_marker(text: &str, marker: char) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '`' {
            let end = code_span_end(&chars, index);
            out.extend(&chars[index..end]);
            index = end;
            continue;
        }
        if chars[index] == '\\' {
            out.push(chars[index]);
            if index + 1 < chars.len() {
                out.push(chars[index + 1]);
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if chars[index] == marker
            && index + 1 < chars.len()
            && chars[index + 1] == marker
            && let Some((merged, end)) = try_double_merge(&chars, index, marker)
        {
            out.push_str(&merged);
            index = end;
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

/// Matches `MM run MM [space] MM run MM` at `start` (the JS
/// `\*\*([^*]+?)\*\*( ?)\*\*([^*]+?)\*\*` shape, generalized over the marker
/// character), returning the merged replacement and the end position. Strike
/// markers merge only when directly adjacent: a struck-through space renders
/// differently from two separate strikes.
fn try_double_merge(chars: &[char], start: usize, marker: char) -> Option<(String, usize)> {
    let mut index = start + 2;
    let a_start = index;
    index = scan_marker_run(chars, index, marker);
    if index == a_start || index + 1 >= chars.len() || chars[index + 1] != marker {
        return None;
    }
    let a: String = chars[a_start..index].iter().collect();
    index += 2;
    let space = marker != '~' && index < chars.len() && chars[index] == ' ';
    if space {
        index += 1;
    }
    if index + 1 >= chars.len() || chars[index] != marker || chars[index + 1] != marker {
        return None;
    }
    index += 2;
    let b_start = index;
    index = scan_marker_run(chars, index, marker);
    if index == b_start || index + 1 >= chars.len() || chars[index + 1] != marker {
        return None;
    }
    let b: String = chars[b_start..index].iter().collect();
    index += 2;
    let mut merged = String::new();
    merged.push(marker);
    merged.push(marker);
    merged.push_str(&a);
    if space {
        merged.push(' ');
    }
    merged.push_str(&b);
    merged.push(marker);
    merged.push(marker);
    Some((merged, index))
}

fn merge_single_marker(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '`' {
            let end = code_span_end(&chars, index);
            out.extend(&chars[index..end]);
            index = end;
            continue;
        }
        if chars[index] == '\\' {
            out.push(chars[index]);
            if index + 1 < chars.len() {
                out.push(chars[index + 1]);
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if chars[index] == '*'
            && (index == 0 || chars[index - 1] != '*')
            && let Some((merged, end)) = try_single_merge(&chars, index)
        {
            out.push_str(&merged);
            index = end;
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

/// Matches `* run * [space] * run *` at `start` with the JS lookarounds
/// (`(?<!\*)` up front, `(?!\*)` at the end).
fn try_single_merge(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut index = start + 1;
    let a_start = index;
    index = scan_marker_run(chars, index, '*');
    if index == a_start || index >= chars.len() {
        return None;
    }
    let a: String = chars[a_start..index].iter().collect();
    index += 1;
    let space = index < chars.len() && chars[index] == ' ';
    if space {
        index += 1;
    }
    if index >= chars.len() || chars[index] != '*' {
        return None;
    }
    index += 1;
    let b_start = index;
    index = scan_marker_run(chars, index, '*');
    if index == b_start || index >= chars.len() {
        return None;
    }
    let b: String = chars[b_start..index].iter().collect();
    if index + 1 < chars.len() && chars[index + 1] == '*' {
        return None;
    }
    index += 1;
    let mut merged = String::from("*");
    merged.push_str(&a);
    if space {
        merged.push(' ');
    }
    merged.push_str(&b);
    merged.push('*');
    Some((merged, index))
}

/// Scans a run of non-marker characters starting at `from`; escaped
/// characters (`\*`) belong to the run, so the marker they quote is not a
/// candidate delimiter. Returns the index of the terminating marker or the
/// end of the input.
fn scan_marker_run(chars: &[char], from: usize, marker: char) -> usize {
    let mut index = from;
    while index < chars.len() {
        if chars[index] == '\\' && index + 1 < chars.len() {
            index += 2;
            continue;
        }
        if chars[index] == marker {
            break;
        }
        index += 1;
    }
    index
}

/// Returns the index just past an inline code span (a backtick run closed by
/// a run of the same length), or just past the opening run when unmatched.
fn code_span_end(chars: &[char], start: usize) -> usize {
    let mut open = 0usize;
    while start + open < chars.len() && chars[start + open] == '`' {
        open += 1;
    }
    let mut index = start + open;
    while index < chars.len() {
        if chars[index] == '`' {
            let mut close = 0usize;
            while index + close < chars.len() && chars[index + close] == '`' {
                close += 1;
            }
            if close == open {
                return index + close;
            }
            index += close;
        } else {
            index += 1;
        }
    }
    start + open
}

/// Collapses 3+ consecutive newlines to exactly one blank line, leaving
/// fenced code block content untouched.
fn collapse_blank_lines(input: &str) -> String {
    transform_outside_fences(input, |segment| {
        let mut out = String::with_capacity(segment.len());
        let mut run = 0usize;
        for ch in segment.chars() {
            if ch == '\n' {
                run += 1;
                if run <= 2 {
                    out.push(ch);
                }
            } else {
                run = 0;
                out.push(ch);
            }
        }
        out
    })
}

/// Applies `transform` to the parts of the input outside fenced code blocks
/// (```` ``` ````/`~~~` fences, optionally blockquote-prefixed).
fn transform_outside_fences(input: &str, transform: impl Fn(&str) -> String) -> String {
    let mut out = String::with_capacity(input.len());
    let mut outside = String::new();
    let mut fence: Option<&'static str> = None;
    for line in input.split_inclusive('\n') {
        let stripped = strip_quote_prefixes(line.trim_end_matches('\n'));
        let line_fence = fence_open(stripped);
        match fence {
            None => {
                if let Some(opened) = line_fence {
                    out.push_str(&transform(&outside));
                    outside.clear();
                    fence = Some(opened);
                    out.push_str(line);
                } else {
                    outside.push_str(line);
                }
            }
            Some(open) => {
                out.push_str(line);
                if stripped == open {
                    fence = None;
                }
            }
        }
    }
    out.push_str(&transform(&outside));
    out
}

/// A line opens a fence when it starts with a 3-char backtick/tilde run and
/// the rest of the line carries no further marker characters (an info string
/// at most). This keeps inline code spans with long fences from being
/// mistaken for block fences.
fn fence_open(stripped: &str) -> Option<&'static str> {
    for fence in ["```", "~~~"] {
        if let Some(rest) = stripped.strip_prefix(fence) {
            let marker = fence.chars().next()?;
            if !rest.contains(marker) {
                return Some(fence);
            }
        }
    }
    None
}

fn strip_quote_prefixes(line: &str) -> &str {
    let mut rest = line.trim_start_matches(' ');
    while let Some(after) = rest.strip_prefix('>') {
        rest = after.strip_prefix(' ').unwrap_or(after);
    }
    rest
}
