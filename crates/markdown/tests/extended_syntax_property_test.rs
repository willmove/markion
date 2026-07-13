//! Property-based tests for extended Markdown syntax (Task 8.5).
//!
//! Implements Properties 16-22 from the design document:
//! - Property 16: Task list parsing (`checked` attribute correctness)
//! - Property 17: Footnote parsing and reference/definition association
//! - Property 18: Superscript / subscript parsing
//! - Property 19: Highlight mark parsing
//! - Property 20: Emoji shortcode → Unicode conversion
//! - Property 21: HTML tag preservation / parsing
//! - Property 22: URL auto-detection
//!
//! **Validates: Requirements 17.1, 17.3, 17.4, 17.5, 17.6, 17.7, 17.8**

use std::collections::HashSet;

use markdown::ast::{Block, Inline};
use markdown::emoji::shortcode_to_unicode;
use markdown::parser::Parser;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_parser() -> Parser {
    Parser::default()
}

/// Recursively extract plain text from a slice of inline nodes, descending into
/// container inlines (superscript, subscript, highlight, etc.).
fn extract_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) => out.push_str(s),
            Inline::Code(s) => out.push_str(s),
            Inline::Strong(inner)
            | Inline::Emphasis(inner)
            | Inline::Strikethrough(inner)
            | Inline::Superscript(inner)
            | Inline::Subscript(inner)
            | Inline::Highlight(inner) => out.push_str(&extract_text(inner)),
            Inline::Link { text, .. } => out.push_str(&extract_text(text)),
            Inline::Emoji { unicode, .. } => out.push_str(unicode),
            _ => {}
        }
    }
    out
}

/// Return the inline content of the first paragraph in the document, if any.
fn first_paragraph_content(doc: &markdown::ast::Document) -> Option<&Vec<Inline>> {
    doc.blocks.iter().find_map(|b| match b {
        Block::Paragraph { content, .. } => Some(content),
        _ => None,
    })
}

/// A curated set of shortcodes known to exist in the emoji map.
const KNOWN_SHORTCODES: &[&str] = &[
    "smile", "heart", "thumbsup", "rocket", "fire", "joy", "wink", "cry", "star", "tada", "pizza",
    "dog", "cat", "book", "check", "sparkles",
];

/// Inline-safe HTML tag names (won't be promoted to block-level HTML).
const INLINE_TAGS: &[&str] = &[
    "span", "em", "strong", "b", "i", "mark", "sub", "sup", "small", "abbr",
];

// ---------------------------------------------------------------------------
// Property 16: Task list parsing
// Validates: Requirements 17.1
// ---------------------------------------------------------------------------

proptest! {
    /// For any sequence of task-list items with random checked states, the
    /// parser must create a `List` whose items carry the correct `checked`
    /// boolean attribute (`Some(true)` for `[x]`, `Some(false)` for `[ ]`).
    #[test]
    fn prop16_task_list_checked_correctness(
        states in prop::collection::vec(any::<bool>(), 1..8),
        words in prop::collection::vec("[a-z]{1,10}", 1..8),
    ) {
        // Build one markdown line per state, reusing generated words cyclically.
        let mut md = String::new();
        for (i, checked) in states.iter().enumerate() {
            let word = &words[i % words.len()];
            let marker = if *checked { "[x]" } else { "[ ]" };
            md.push_str(&format!("- {} {}\n", marker, word));
        }

        let doc = default_parser().parse(&md).unwrap();

        // The first (and only) block should be a list.
        let list_block = doc.blocks.iter().find(|b| matches!(b, Block::List { .. }));
        prop_assert!(list_block.is_some(), "expected a List block, got: {:?}", doc.blocks);

        if let Some(Block::List { items, .. }) = list_block {
            prop_assert_eq!(items.len(), states.len(),
                "item count mismatch for markdown:\n{}", md);
            for (i, item) in items.iter().enumerate() {
                prop_assert_eq!(item.checked, Some(states[i]),
                    "checked mismatch at item {} for markdown:\n{}", i, md);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Property 17: Footnote parsing and association
// Validates: Requirements 17.3
// ---------------------------------------------------------------------------

proptest! {
    /// For any document containing footnote references `[^id]` together with
    /// their definitions `[^id]: text`, every reference must be associated with
    /// a definition (present in `footnote_map`), and every reference label must
    /// be recognized as an `Inline::FootnoteReference`.
    #[test]
    fn prop17_footnote_reference_definition_association(
        labels in prop::collection::hash_set("[a-z][a-z0-9]{0,5}", 1..5),
    ) {
        let labels: Vec<String> = labels.into_iter().collect();

        // Paragraph referencing every footnote.
        let mut md = String::from("Body text");
        for label in &labels {
            md.push_str(&format!(" [^{}]", label));
        }
        md.push_str(".\n\n");
        // Definitions, one per line.
        for label in &labels {
            md.push_str(&format!("[^{}]: Note for {}.\n", label, label));
        }

        let doc = default_parser().parse(&md).unwrap();

        // Every label must have an association in the footnote map.
        for label in &labels {
            prop_assert!(
                doc.footnote_map.contains_key(label),
                "footnote_map missing label {:?} for markdown:\n{}", label, md
            );
        }

        // Every referenced label must appear as an Inline::FootnoteReference.
        let content = first_paragraph_content(&doc)
            .expect("expected a paragraph with footnote references");
        let referenced: HashSet<&str> = content
            .iter()
            .filter_map(|i| match i {
                Inline::FootnoteReference(l) => Some(l.as_str()),
                _ => None,
            })
            .collect();
        for label in &labels {
            prop_assert!(
                referenced.contains(label.as_str()),
                "reference for label {:?} not found in paragraph for markdown:\n{}", label, md
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property 18: Superscript / subscript parsing
// Validates: Requirements 17.4
// ---------------------------------------------------------------------------

proptest! {
    /// For any `^text^` syntax, the parser must create a `Superscript` node
    /// preserving the inner text.
    #[test]
    fn prop18_superscript_parsing(content in "[a-z0-9]{1,8}") {
        let md = format!("base^{}^ tail", content);
        let doc = default_parser().parse(&md).unwrap();

        let para = first_paragraph_content(&doc)
            .expect("expected a paragraph");
        let sup = para.iter().find_map(|i| match i {
            Inline::Superscript(inner) => Some(inner),
            _ => None,
        });
        prop_assert!(sup.is_some(), "expected Superscript node for markdown: {}", md);
        prop_assert_eq!(extract_text(sup.unwrap()), content.clone(),
            "superscript content mismatch for markdown: {}", md);
    }

    /// For any `~text~` syntax, the parser must create a `Subscript` node
    /// preserving the inner text (and must not treat it as strikethrough).
    ///
    /// Note: subscript uses a single tilde, which collides with GFM's
    /// single-tilde strikethrough when the delimiters are flanked by
    /// whitespace. Per the parser's design, double-tilde `~~` is strikethrough
    /// (handled by pulldown-cmark) while single-tilde subscript is recognized
    /// in the intraword context (e.g. `H~2~O`). The generator therefore embeds
    /// the subscript between word characters, matching the meaningful input
    /// space for subscript syntax.
    #[test]
    fn prop18_subscript_parsing(content in "[a-z0-9]{1,8}") {
        let md = format!("H~{}~O molecule", content);
        let doc = default_parser().parse(&md).unwrap();

        let para = first_paragraph_content(&doc)
            .expect("expected a paragraph");
        let sub = para.iter().find_map(|i| match i {
            Inline::Subscript(inner) => Some(inner),
            _ => None,
        });
        prop_assert!(sub.is_some(), "expected Subscript node for markdown: {}", md);
        prop_assert_eq!(extract_text(sub.unwrap()), content.clone(),
            "subscript content mismatch for markdown: {}", md);
        // Must not be interpreted as strikethrough.
        prop_assert!(
            !para.iter().any(|i| matches!(i, Inline::Strikethrough(_))),
            "single-tilde subscript should not be strikethrough for markdown: {}", md
        );
    }
}

// ---------------------------------------------------------------------------
// Property 19: Highlight mark parsing
// Validates: Requirements 17.5
// ---------------------------------------------------------------------------

proptest! {
    /// For any `==text==` syntax, the parser must create a `Highlight` node
    /// preserving the marked text content.
    #[test]
    fn prop19_highlight_parsing(content in "[a-z0-9]{1,12}") {
        let md = format!("start =={}== end", content);
        let doc = default_parser().parse(&md).unwrap();

        let para = first_paragraph_content(&doc)
            .expect("expected a paragraph");
        let hl = para.iter().find_map(|i| match i {
            Inline::Highlight(inner) => Some(inner),
            _ => None,
        });
        prop_assert!(hl.is_some(), "expected Highlight node for markdown: {}", md);
        prop_assert_eq!(extract_text(hl.unwrap()), content.clone(),
            "highlight content mismatch for markdown: {}", md);
    }
}

// ---------------------------------------------------------------------------
// Property 20: Emoji shortcode conversion
// Validates: Requirements 17.6
// ---------------------------------------------------------------------------

proptest! {
    /// For any valid emoji shortcode, the parser must convert it into an
    /// `Inline::Emoji` node whose `unicode` matches the shortcode mapping.
    #[test]
    fn prop20_emoji_shortcode_conversion(idx in 0usize..KNOWN_SHORTCODES.len()) {
        let shortcode = KNOWN_SHORTCODES[idx];
        let expected = shortcode_to_unicode(shortcode)
            .expect("curated shortcode must exist in the emoji map");

        let md = format!("react :{}: now", shortcode);
        let doc = default_parser().parse(&md).unwrap();

        let para = first_paragraph_content(&doc)
            .expect("expected a paragraph");
        let emoji = para.iter().find_map(|i| match i {
            Inline::Emoji { shortcode: sc, unicode } => Some((sc.as_str(), unicode.as_str())),
            _ => None,
        });
        prop_assert!(emoji.is_some(), "expected Emoji node for markdown: {}", md);
        let (sc, unicode) = emoji.unwrap();
        prop_assert_eq!(sc, shortcode, "shortcode mismatch for markdown: {}", md);
        prop_assert_eq!(unicode, expected, "unicode mismatch for markdown: {}", md);
    }
}

// ---------------------------------------------------------------------------
// Property 21: HTML tag preservation / parsing
// Validates: Requirements 17.7
// ---------------------------------------------------------------------------

proptest! {
    /// For any embedded inline HTML tag, the parser must produce
    /// `Inline::HtmlInline` node(s) that preserve the original HTML content.
    #[test]
    fn prop21_html_tag_preservation(
        tag_idx in 0usize..INLINE_TAGS.len(),
        inner in "[a-z0-9 ]{1,10}",
    ) {
        let tag = INLINE_TAGS[tag_idx];
        let md = format!("prefix <{tag}>{inner}</{tag}> suffix\n", tag = tag, inner = inner);
        let doc = default_parser().parse(&md).unwrap();

        let para = first_paragraph_content(&doc)
            .expect("expected a paragraph");

        // Collect all raw HTML fragments preserved as HtmlInline nodes.
        let html_fragments: String = para
            .iter()
            .filter_map(|i| match i {
                Inline::HtmlInline(html) => Some(html.as_str()),
                _ => None,
            })
            .collect();

        prop_assert!(
            !html_fragments.is_empty(),
            "expected at least one HtmlInline node for markdown: {}", md
        );
        // Original opening and closing tags must be preserved verbatim.
        prop_assert!(
            html_fragments.contains(&format!("<{}>", tag)),
            "opening tag <{}> not preserved (got {:?}) for markdown: {}", tag, html_fragments, md
        );
        prop_assert!(
            html_fragments.contains(&format!("</{}>", tag)),
            "closing tag </{}> not preserved (got {:?}) for markdown: {}", tag, html_fragments, md
        );
    }
}

// ---------------------------------------------------------------------------
// Property 22: URL auto-detection
// Validates: Requirements 17.8
// ---------------------------------------------------------------------------

proptest! {
    /// For any bare URL (http://, https://, or www.) embedded in text, the
    /// parser must auto-detect it and produce an `Inline::Link` with the
    /// expected URL (www. prefixed URLs are normalized to https://www.).
    #[test]
    fn prop22_url_auto_detection(
        scheme_idx in 0usize..3usize,
        domain in "[a-z]{2,10}",
        tld_idx in 0usize..4usize,
        path in prop::option::of("[a-z0-9]{1,6}"),
    ) {
        let scheme = ["http://", "https://", "www."][scheme_idx];
        let tld = ["com", "org", "net", "io"][tld_idx];
        let path_part = path.as_ref().map(|p| format!("/{}", p)).unwrap_or_default();

        // The URL as it appears in the source text.
        let bare_url = format!("{}{}.{}{}", scheme, domain, tld, path_part);
        // Expected normalized URL stored on the Link node.
        let expected_url = if scheme == "www." {
            format!("https://{}", bare_url)
        } else {
            bare_url.clone()
        };

        let md = format!("Visit {} today\n", bare_url);
        let doc = default_parser().parse(&md).unwrap();

        let para = first_paragraph_content(&doc)
            .expect("expected a paragraph");
        let found = para.iter().any(|i| matches!(
            i,
            Inline::Link { url, .. } if *url == expected_url
        ));
        prop_assert!(
            found,
            "expected auto-detected Link with url {:?} for markdown: {}\nparsed inlines: {:?}",
            expected_url, md, para
        );
    }
}
