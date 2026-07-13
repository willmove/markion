//! Property-based tests for YAML front matter parsing.
//!
//! **Validates: Requirements 18.1, 18.2, 18.5, 18.6**
//!
//! These tests exercise three design properties:
//! - **Property 23: YAML Front Matter 识别** — any document that opens with `---`
//!   and contains a matching closing `---` has its YAML front matter recognized and
//!   separated from the document body.
//! - **Property 24: YAML 键值对解析** — any syntactically valid YAML front matter is
//!   parsed into the correct structured key/value representation.
//! - **Property 25: 无效 YAML 返回错误** — any syntactically invalid YAML front matter
//!   returns an error instead of panicking.

use markdown::{Parser, ParserOptions};
use proptest::prelude::*;

fn default_parser() -> Parser {
    Parser::new(ParserOptions::default())
}

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

/// A value safe to embed inside a double-quoted YAML scalar.
/// Restricted to characters that require no escaping so the round-trip is exact.
fn arb_safe_value() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{0,20}".prop_map(|s| s)
}

/// A single tag: a non-empty alphanumeric token.
fn arb_tag() -> impl Strategy<Value = String> {
    "[A-Za-z0-9]{1,10}".prop_map(|s| s)
}

/// The set of front matter fields we assert against.
#[derive(Debug, Clone)]
struct FrontMatterFields {
    title: String,
    author: Option<String>,
    date: Option<String>,
    tags: Vec<String>,
    body_marker: String,
}

/// Generate a set of valid front matter fields. `title` is always present so the
/// YAML block is guaranteed non-empty and metadata is guaranteed to be produced.
fn arb_front_matter_fields() -> impl Strategy<Value = FrontMatterFields> {
    (
        arb_safe_value(),
        prop::option::of(arb_safe_value()),
        prop::option::of(arb_safe_value()),
        prop::collection::vec(arb_tag(), 0..5),
        "[A-Za-z0-9]{1,10}",
    )
        .prop_map(
            |(title, author, date, tags, body_marker)| FrontMatterFields {
                title,
                author,
                date,
                tags,
                body_marker,
            },
        )
}

/// Render a `FrontMatterFields` into a full markdown document with a heading body.
fn render_document(f: &FrontMatterFields) -> String {
    let mut yaml = String::new();
    yaml.push_str(&format!("title: \"{}\"\n", f.title));
    if let Some(a) = &f.author {
        yaml.push_str(&format!("author: \"{}\"\n", a));
    }
    if let Some(d) = &f.date {
        yaml.push_str(&format!("date: \"{}\"\n", d));
    }
    if !f.tags.is_empty() {
        let quoted: Vec<String> = f.tags.iter().map(|t| format!("\"{}\"", t)).collect();
        yaml.push_str(&format!("tags: [{}]\n", quoted.join(", ")));
    }
    format!("---\n{yaml}---\n# {}\n", f.body_marker)
}

/// Generate reliably-invalid YAML front matter content (no closing tokens).
fn arb_invalid_yaml() -> impl Strategy<Value = String> {
    ("[a-z]{1,10}", "[a-z]{1,10}", 0usize..4).prop_map(|(k, v, variant)| match variant {
        // Unclosed double-quoted scalar
        0 => format!("{k}: \"{v}"),
        // Unclosed single-quoted scalar
        1 => format!("{k}: '{v}"),
        // Unclosed flow sequence
        2 => format!("{k}: [{v}, {v}"),
        // Unclosed flow mapping
        _ => format!("{k}: {{{v}: {v}"),
    })
}

// ---------------------------------------------------------------------------
// Property 23: YAML Front Matter 识别 (Validates: Requirement 18.1)
// ---------------------------------------------------------------------------

proptest! {
    /// For any document opening with `---` and containing a closing `---`,
    /// the parser recognizes the YAML front matter and separates it from the body.
    #[test]
    fn prop_yaml_front_matter_recognized(fields in arb_front_matter_fields()) {
        let md = render_document(&fields);
        let doc = default_parser().parse(&md).expect("valid front matter should parse");

        // Front matter recognized.
        prop_assert!(doc.metadata.is_some(), "front matter should be recognized");

        // Body was separated: exactly one heading block carrying the body marker.
        prop_assert_eq!(doc.blocks.len(), 1, "body should be separated from front matter");
    }
}

// ---------------------------------------------------------------------------
// Property 24: YAML 键值对解析 (Validates: Requirement 18.2)
// ---------------------------------------------------------------------------

proptest! {
    /// For any syntactically valid YAML front matter, the parser produces the
    /// correct structured key/value representation.
    #[test]
    fn prop_yaml_key_values_parsed(fields in arb_front_matter_fields()) {
        let md = render_document(&fields);
        let doc = default_parser().parse(&md).expect("valid front matter should parse");
        let fm = doc.metadata.expect("metadata should be present");

        prop_assert_eq!(fm.title, Some(fields.title));
        prop_assert_eq!(fm.author, fields.author);
        prop_assert_eq!(fm.date, fields.date);
        prop_assert_eq!(fm.tags, fields.tags);
    }
}

// ---------------------------------------------------------------------------
// Property 25: 无效 YAML 返回错误 (Validates: Requirements 18.5, 18.6)
// ---------------------------------------------------------------------------

proptest! {
    /// For any syntactically invalid YAML front matter, the parser returns an
    /// error (and does not panic). The error message references YAML.
    #[test]
    fn prop_invalid_yaml_returns_error(yaml in arb_invalid_yaml()) {
        let md = format!("---\n{yaml}\n---\n# Body\n");
        let result = default_parser().parse(&md);

        prop_assert!(result.is_err(), "invalid YAML should return an error: {:?}", yaml);
        let msg = result.unwrap_err().to_string();
        prop_assert!(msg.contains("YAML"), "error should mention YAML, got: {msg}");
    }
}
