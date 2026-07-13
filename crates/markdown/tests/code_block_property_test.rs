//! Property-based tests for code block parsing.
//!
//! **Property 4: 代码块识别 (Code block recognition)**
//! **Property 5: 代码块空白保留 (Code block whitespace preservation)**
//!
//! **Validates: Requirements 2.1, 2.4**
//!
//! - Property 4: For any triple-backtick fenced block (with or without a
//!   language identifier), the parser must create a `CodeBlock` AST node and
//!   preserve the language identifier.
//! - Property 5: For any code content containing whitespace (spaces, tabs,
//!   newlines), parsing and re-rendering must preserve every whitespace
//!   character without normalization.

use markdown::ast::Block;
use markdown::parser::{Parser, ParserOptions};
use markdown::renderer::render_to_markdown;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

/// A curated list of language identifiers that are safe to round-trip.
/// Notably excludes "math" and "$$", which the parser maps to `MathBlock`.
const LANGUAGES: &[&str] = &[
    "rust",
    "python",
    "javascript",
    "typescript",
    "go",
    "c",
    "cpp",
    "java",
    "ruby",
    "html",
    "css",
    "json",
    "bash",
    "kotlin",
    "swift",
];

/// Generate an optional language identifier for a fenced code block.
fn arb_lang() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        // No language identifier (plain code block).
        Just(None),
        // A known language identifier.
        proptest::sample::select(LANGUAGES).prop_map(|s| Some(s.to_string())),
    ]
}

/// Generate a single line of code content that is rich in whitespace.
///
/// Each line is composed of a leading indentation (spaces/tabs) followed by
/// text made of alphanumerics interspersed with spaces and tabs. Lines never
/// contain backticks (which would break the fence) or newlines (added when
/// joining lines into a body).
fn arb_code_line() -> impl Strategy<Value = String> {
    ("[ \\t]{0,4}", "[a-zA-Z0-9 \\t]{0,12}").prop_map(|(indent, text)| format!("{indent}{text}"))
}

/// Generate a code body: one or more lines, each terminated by a newline.
///
/// A trailing newline is included on every line so that the body matches the
/// exact string that `pulldown-cmark` produces for a fenced code block (which
/// always ends the code content with a newline before the closing fence).
fn arb_code_body() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_code_line(), 1..6).prop_map(|lines| {
        let mut body = String::new();
        for line in lines {
            body.push_str(&line);
            body.push('\n');
        }
        body
    })
}

/// Build a fenced code block Markdown source from an optional language and a
/// code body (which already ends with a newline).
fn build_source(lang: &Option<String>, body: &str) -> String {
    let lang_str = lang.as_deref().unwrap_or("");
    format!("```{lang_str}\n{body}```\n")
}

/// Extract the single `CodeBlock` from a parsed document, asserting there is
/// exactly one block and it is a code block.
fn expect_single_code_block(md: &str) -> (Option<String>, String) {
    let parser = Parser::new(ParserOptions::default());
    let doc = parser.parse(md).expect("parse should succeed");
    prop_assert_eq_len(doc.blocks.len());
    match &doc.blocks[0] {
        Block::CodeBlock { lang, code, .. } => (lang.clone(), code.clone()),
        other => panic!("Expected CodeBlock, got {other:?}"),
    }
}

/// Small helper to assert exactly one block was produced.
fn prop_assert_eq_len(len: usize) {
    assert_eq!(len, 1, "Expected exactly one block, got {len}");
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Property 4: 代码块识别 (Code block recognition)
    ///
    /// Validates: Requirements 2.1
    ///
    /// For any triple-backtick fenced block, with or without a language
    /// identifier, the parser creates a `CodeBlock` node and preserves the
    /// language identifier exactly.
    #[test]
    fn test_code_block_recognition(lang in arb_lang(), body in arb_code_body()) {
        let source = build_source(&lang, &body);
        let (parsed_lang, _code) = expect_single_code_block(&source);
        prop_assert_eq!(
            parsed_lang.as_deref(),
            lang.as_deref(),
            "Language identifier not preserved for source: {:?}",
            source
        );
    }

    /// Property 5: 代码块空白保留 (Code block whitespace preservation)
    ///
    /// Validates: Requirements 2.4
    ///
    /// For any code content containing whitespace, parsing preserves every
    /// whitespace character exactly (no normalization), and rendering the AST
    /// back to Markdown followed by re-parsing preserves the same content.
    #[test]
    fn test_code_block_whitespace_preservation(lang in arb_lang(), body in arb_code_body()) {
        let source = build_source(&lang, &body);

        // Step 1: parse preserves the exact whitespace of the code body.
        let (parsed_lang, parsed_code) = expect_single_code_block(&source);
        prop_assert_eq!(
            &parsed_code,
            &body,
            "Whitespace not preserved on parse for source: {:?}",
            source
        );

        // Step 2: render → parse preserves the same code content and language.
        let parser = Parser::new(ParserOptions::default());
        let doc = parser.parse(&source).expect("parse should succeed");
        let rendered = render_to_markdown(&doc);
        let doc2 = parser.parse(&rendered).expect("re-parse should succeed");

        prop_assert_eq!(doc2.blocks.len(), 1, "Re-parse should yield one block");
        match &doc2.blocks[0] {
            Block::CodeBlock { lang: lang2, code: code2, .. } => {
                prop_assert_eq!(
                    code2,
                    &parsed_code,
                    "Whitespace not preserved through render+parse. rendered: {:?}",
                    rendered
                );
                prop_assert_eq!(
                    lang2.as_deref(),
                    parsed_lang.as_deref(),
                    "Language not preserved through render+parse"
                );
            }
            other => panic!("Expected CodeBlock after render+parse, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests (specific examples / edge cases)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn code_block_without_language_has_none() {
        let (lang, _) = expect_single_code_block("```\nlet x = 1;\n```\n");
        assert_eq!(lang, None);
    }

    #[test]
    fn code_block_with_language_preserved() {
        let (lang, code) = expect_single_code_block("```rust\nfn main() {}\n```\n");
        assert_eq!(lang.as_deref(), Some("rust"));
        assert!(code.contains("fn main()"));
    }

    #[test]
    fn code_block_preserves_leading_spaces_and_tabs() {
        let body = "    four spaces\n\ttab indent\n  two spaces\n";
        let source = format!("```python\n{body}```\n");
        let (_, code) = expect_single_code_block(&source);
        assert_eq!(code, body);
    }

    #[test]
    fn code_block_preserves_blank_lines() {
        let body = "line1\n\n\nline2\n";
        let source = format!("```\n{body}```\n");
        let (_, code) = expect_single_code_block(&source);
        assert_eq!(code, body);
    }

    #[test]
    fn code_block_preserves_whitespace_through_render() {
        let body = "\tindented\n  spaced\n";
        let source = format!("```go\n{body}```\n");
        let parser = Parser::new(ParserOptions::default());
        let doc = parser.parse(&source).unwrap();
        let rendered = render_to_markdown(&doc);
        let doc2 = parser.parse(&rendered).unwrap();
        match &doc2.blocks[0] {
            Block::CodeBlock { code, .. } => assert_eq!(code, body),
            other => panic!("Expected CodeBlock, got {other:?}"),
        }
    }
}
