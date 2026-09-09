//! Tolerant, GUI-free HTML to Markdown conversion for clipboard rich text.
//!
//! The crate hand-rolls a forgiving tokenizer and tree builder for
//! machine-generated HTML (Word, Google Docs, browsers) and renders the
//! surviving content with the same marker vocabulary as Markion's word import
//! runtime (`**`, `*`, `~~`, `<u>`, `<sub>`, `<sup>`) and the escaping policy
//! of `markion-docx-import`. It owns no I/O, carries no dependencies, and
//! never panics on malformed input.

mod dom;
mod escape;
mod render;

/// Converts clipboard HTML into Markion-compatible Markdown.
/// Returns an empty string when the HTML carries no convertible content.
pub fn html_to_markdown(html: &str) -> String {
    if html.trim().is_empty() {
        return String::new();
    }
    let document = dom::parse(html);
    render::render_document(&document)
}

#[cfg(test)]
mod tests;
