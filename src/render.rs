//! Markdown → target-format rendering helpers (HTML / LaTeX / plain text).
//!
//! The driver methods (`render_html_document`, `render_latex_document`, …) live
//! on `MarkdownDocument`; this module holds the format-specific free functions
//! and the shared stylesheet string they emit into.

use crate::escape::{decode_basic_html_entities, escape_html_attribute, escape_html_text};
use crate::math::render_math;
use crate::model::{RichText, TableAlignment};

/// Default stylesheet embedded in the styled HTML export.
pub(crate) const DEFAULT_CSS: &str = r#"
:root {
  color-scheme: light;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  line-height: 1.58;
  color: #202124;
  background: #fbfbfa;
}
body {
  max-width: 860px;
  margin: 48px auto;
  padding: 0 32px 72px;
}
pre, code {
  font-family: "JetBrains Mono", "SFMono-Regular", Consolas, monospace;
}
pre {
  overflow: auto;
  padding: 16px;
  border-radius: 8px;
  background: #f1f3f4;
}
blockquote {
  margin-left: 0;
  padding-left: 18px;
  border-left: 4px solid #d0d7de;
  color: #57606a;
}
table {
  border-collapse: collapse;
}
.markion-diagram {
  overflow: auto;
  margin: 1em 0;
}
.markion-diagram svg {
  display: block;
  max-width: 100%;
  height: auto;
}
th, td {
  border: 1px solid #d0d7de;
  padding: 6px 10px;
}
"#;

/// Wraps inline/block math containers emitted by `pulldown-cmark`'s HTML
/// backend with rendered-math metadata so the export carries validity state
/// and a readable fallback.
pub(crate) fn annotate_math_html(html: &str) -> String {
    let mut output = String::new();
    let mut index = 0usize;

    while index < html.len() {
        let Some(tag_start) = html[index..].find('<').map(|relative| index + relative) else {
            output.push_str(&html[index..]);
            break;
        };
        output.push_str(&html[index..tag_start]);

        let Some(tag_end) = html[tag_start..]
            .find('>')
            .map(|relative| tag_start + relative + 1)
        else {
            output.push_str(&html[tag_start..]);
            break;
        };
        let tag = &html[tag_start..tag_end];
        let Some((display, close)) = math_tag_kind(tag) else {
            output.push_str(tag);
            index = tag_end;
            continue;
        };
        let Some(content_end) = html[tag_end..]
            .find(close)
            .map(|relative| tag_end + relative)
        else {
            output.push_str(&html[tag_start..]);
            break;
        };

        let escaped_latex = &html[tag_end..content_end];
        let latex = decode_basic_html_entities(escaped_latex);
        let rendered = render_math(latex.trim(), display);
        let tag = if display { "div" } else { "span" };
        let class = if rendered.error.is_some() {
            if display {
                "math math-display math-error"
            } else {
                "math math-inline math-error"
            }
        } else if display {
            "math math-display"
        } else {
            "math math-inline"
        };
        let title = rendered
            .error
            .as_ref()
            .map(|error| format!(" title=\"{}\"", escape_html_attribute(error)))
            .unwrap_or_default();
        output.push_str(&format!(
            "<{tag} class=\"{class}\" data-latex=\"{}\" data-valid=\"{}\"{title}>{}</{tag}>",
            escape_html_attribute(rendered.latex.trim()),
            rendered.error.is_none(),
            escape_html_text(&rendered.text)
        ));
        index = content_end + close.len();
    }

    output
}

fn math_tag_kind(tag: &str) -> Option<(bool, &'static str)> {
    let lower = tag.to_ascii_lowercase();
    let is_span = lower.starts_with("<span");
    let is_div = lower.starts_with("<div");
    if !is_span && !is_div {
        return None;
    }
    if !(lower.contains("math")
        && (lower.contains("math-inline") || lower.contains("math-display")))
    {
        return None;
    }
    if lower.starts_with("</") {
        return None;
    }

    if lower.contains("math-display") {
        Some((true, if is_div { "</div>" } else { "</span>" }))
    } else {
        Some((false, "</span>"))
    }
}

/// Escapes text for LaTeX output.
pub(crate) fn escape_latex(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '\\' => "\\textbackslash{}".to_string(),
            '{' => "\\{".to_string(),
            '}' => "\\}".to_string(),
            '$' => "\\$".to_string(),
            '&' => "\\&".to_string(),
            '%' => "\\%".to_string(),
            '#' => "\\#".to_string(),
            '_' => "\\_".to_string(),
            '^' => "\\textasciicircum{}".to_string(),
            '~' => "\\textasciitilde{}".to_string(),
            _ => ch.to_string(),
        })
        .collect()
}

/// Normalizes path separators and escapes spaces for LaTeX `\\includegraphics`.
pub(crate) fn escape_latex_path(path: &str) -> String {
    path.replace('\\', "/").replace(' ', "\\ ")
}

/// Renders inline text for LaTeX, preserving `$...$` math runs verbatim while
/// escaping the surrounding prose.
pub(crate) fn render_latex_inline_text(text: &str) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('$') {
        output.push_str(&escape_latex(&rest[..start]));
        let after_start = &rest[start + 1..];
        if let Some(end) = after_start.find('$') {
            let formula = &after_start[..end];
            output.push('$');
            output.push_str(formula);
            output.push('$');
            rest = &after_start[end + 1..];
        } else {
            output.push_str("\\$");
            rest = after_start;
        }
    }
    output.push_str(&escape_latex(rest));
    output
}

/// Renders resolved rich text (preview inline spans) to LaTeX, mapping the
/// style flags to their LaTeX commands. Plain prose keeps `$...$` math runs
/// verbatim; inline code is `\texttt` with full escaping.
pub(crate) fn render_latex_rich_text(rich: &RichText) -> String {
    if rich.spans.is_empty() {
        return render_latex_inline_text(&rich.text);
    }
    let mut output = String::new();
    for span in &rich.spans {
        let mut piece = if span.style.code {
            format!("\\texttt{{{}}}", escape_latex(&span.text))
        } else {
            render_latex_inline_text(&span.text)
        };
        if span.style.bold {
            piece = format!("\\textbf{{{piece}}}");
        }
        if span.style.italic {
            piece = format!("\\textit{{{piece}}}");
        }
        if span.style.strikethrough {
            piece = format!("\\sout{{{piece}}}");
        }
        if span.style.highlight {
            piece = format!("\\hl{{{piece}}}");
        }
        if span.style.superscript {
            piece = format!("\\textsuperscript{{{piece}}}");
        }
        if span.style.subscript {
            piece = format!("\\textsubscript{{{piece}}}");
        }
        if let Some(url) = &span.link {
            piece = format!("\\href{{{}}}{{{piece}}}", escape_latex_href_url(url));
        }
        output.push_str(&piece);
    }
    output
}

/// Escapes a URL for use as the first argument of `\href`.
fn escape_latex_href_url(url: &str) -> String {
    url.chars()
        .map(|ch| match ch {
            '\\' => "\\textbackslash{}".to_string(),
            '%' => "\\%".to_string(),
            '#' => "\\#".to_string(),
            '&' => "\\&".to_string(),
            '{' => "\\{".to_string(),
            '}' => "\\}".to_string(),
            _ => ch.to_string(),
        })
        .collect()
}

/// Maps a fence language identifier to the name `listings` knows, if any.
/// Anything else renders as an untyped `lstlisting` block.
pub(crate) fn latex_listing_language(language: Option<&str>) -> Option<&'static str> {
    match language?.trim().to_ascii_lowercase().as_str() {
        "python" | "py" => Some("Python"),
        "java" => Some("Java"),
        "c" => Some("C"),
        "cpp" | "c++" => Some("C++"),
        "sql" => Some("SQL"),
        "bash" | "sh" | "shell" => Some("bash"),
        "ruby" | "rb" => Some("Ruby"),
        "perl" => Some("Perl"),
        "php" => Some("PHP"),
        "haskell" | "hs" => Some("Haskell"),
        "html" => Some("HTML"),
        "xml" => Some("XML"),
        "r" => Some("R"),
        "matlab" => Some("Matlab"),
        "pascal" => Some("Pascal"),
        "fortran" => Some("Fortran"),
        "lisp" => Some("Lisp"),
        "prolog" => Some("Prolog"),
        "erlang" => Some("erlang"),
        "awk" => Some("Awk"),
        _ => None,
    }
}

/// Appends one `\item` line, rendering task-list state as checkbox symbols.
pub(crate) fn push_latex_list_item(output: &mut String, checked: Option<bool>, text: &RichText) {
    let marker = match checked {
        Some(true) => "$\\boxtimes$ ",
        Some(false) => "$\\square$ ",
        None => "",
    };
    output.push_str(&format!(
        "\\item {}{}\n",
        marker,
        render_latex_rich_text(text)
    ));
}

/// Renders a Markdown table as a LaTeX `longtable`, deriving the column spec
/// from the separator-row alignments (`Default` → `l`).
pub(crate) fn render_latex_table(rows: &[Vec<String>], alignments: &[TableAlignment]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let columns = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
    let column_spec: String = (0..columns)
        .map(|column| match alignments.get(column) {
            Some(TableAlignment::Center) => 'c',
            Some(TableAlignment::Right) => 'r',
            _ => 'l',
        })
        .collect();
    let mut output = format!("\\begin{{longtable}}{{{column_spec}}}\n");
    for (index, row) in rows.iter().enumerate() {
        let mut cells = row
            .iter()
            .map(|cell| escape_latex(cell))
            .collect::<Vec<_>>();
        cells.resize(columns, String::new());
        output.push_str(&cells.join(" & "));
        output.push_str(" \\\\\n");
        if index == 0 {
            output.push_str("\\hline\n");
        }
    }
    output.push_str("\\end{longtable}");
    output
}
