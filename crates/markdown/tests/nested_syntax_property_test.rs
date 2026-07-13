//! Property-based test for Parser nested-syntax handling.
//!
//! **Property 14: 嵌套语法正确解析 (Nested syntax correctly parsed)**
//!
//! **Validates: Requirements 15.6**
//!
//! For any nested Markdown structure (a code block inside a list item, or a
//! table inside a block quote), the parser must produce an AST that correctly
//! represents the nesting hierarchy: the inner block must appear as a child of
//! the outer block, not be dropped or flattened to the top level.

use markdown::ast::Block;
use markdown::parser::{Parser, ParserOptions};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

/// A short alphanumeric word, safe to embed in Markdown without triggering
/// any special syntax.
fn arb_word() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9]{0,7}".prop_map(|s| s)
}

/// A single line of "code": alphanumeric/underscore tokens separated by spaces.
/// Never contains backticks or newlines, so it can't close a fenced block.
fn arb_code_line() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_]{1,6}", 1..4).prop_map(|toks| toks.join(" "))
}

/// A supported language identifier for a fenced code block.
fn arb_lang() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rust"),
        Just("python"),
        Just("javascript"),
        Just("go"),
        Just("c"),
    ]
    .prop_map(String::from)
}

/// A nested-structure fixture: carries both the generated Markdown source and
/// the expected structural facts about the resulting AST.
#[derive(Debug, Clone)]
enum NestedCase {
    /// A fenced code block nested inside a list item.
    CodeInList {
        item_text: String,
        lang: String,
        code_lines: Vec<String>,
    },
    /// A GFM table nested inside a block quote.
    TableInBlockQuote {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

fn arb_code_in_list() -> impl Strategy<Value = NestedCase> {
    (
        arb_word(),
        arb_lang(),
        prop::collection::vec(arb_code_line(), 1..4),
    )
        .prop_map(|(item_text, lang, code_lines)| NestedCase::CodeInList {
            item_text,
            lang,
            code_lines,
        })
}

fn arb_table_in_blockquote() -> impl Strategy<Value = NestedCase> {
    (2usize..=3)
        .prop_flat_map(|cols| {
            let headers = prop::collection::vec(arb_word(), cols..=cols);
            let rows = prop::collection::vec(prop::collection::vec(arb_word(), cols..=cols), 1..4);
            (headers, rows)
        })
        .prop_map(|(headers, rows)| NestedCase::TableInBlockQuote { headers, rows })
}

fn arb_nested_case() -> impl Strategy<Value = NestedCase> {
    prop_oneof![arb_code_in_list(), arb_table_in_blockquote()]
}

// ---------------------------------------------------------------------------
// Markdown construction
// ---------------------------------------------------------------------------

impl NestedCase {
    /// Render this fixture to Markdown source text.
    fn to_markdown(&self) -> String {
        match self {
            NestedCase::CodeInList {
                item_text,
                lang,
                code_lines,
            } => {
                // List item, then a blank line, then a fenced code block whose
                // lines are indented by 2 spaces so they nest inside the item.
                let mut md = String::new();
                md.push_str(&format!("- {item_text}\n\n"));
                md.push_str(&format!("  ```{lang}\n"));
                for line in code_lines {
                    md.push_str(&format!("  {line}\n"));
                }
                md.push_str("  ```\n");
                md
            }
            NestedCase::TableInBlockQuote { headers, rows } => {
                // A GFM table with every line prefixed by "> " to nest it inside
                // a block quote.
                let mut md = String::new();
                md.push_str(&format!("> | {} |\n", headers.join(" | ")));
                let sep: Vec<&str> = headers.iter().map(|_| "---").collect();
                md.push_str(&format!("> | {} |\n", sep.join(" | ")));
                for row in rows {
                    md.push_str(&format!("> | {} |\n", row.join(" | ")));
                }
                md
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Property test
// ---------------------------------------------------------------------------

proptest! {
    /// Property 14: 嵌套语法正确解析
    ///
    /// Validates: Requirements 15.6
    ///
    /// The parser must represent nested structures faithfully in the AST:
    /// - a code block written inside a list item appears in that item's
    ///   `blocks`, preserving language and content;
    /// - a table written inside a block quote appears in that quote's
    ///   `content`, preserving its row/column dimensions.
    #[test]
    fn test_nested_syntax_correctly_parsed(case in arb_nested_case()) {
        let parser = Parser::new(ParserOptions::default());
        let md = case.to_markdown();
        let doc = parser.parse(&md).expect("parsing nested markdown should succeed");

        match &case {
            NestedCase::CodeInList { lang, code_lines, .. } => {
                // The document must contain a List at the top level.
                let list = doc.blocks.iter().find_map(|b| match b {
                    Block::List { items, .. } => Some(items),
                    _ => None,
                });
                let items = list.unwrap_or_else(|| {
                    panic!("expected a top-level List block, got: {:?}", doc.blocks)
                });

                // Some list item must carry the code block as a nested block,
                // preserving the language identifier and code content.
                let nested_code = items.iter().flat_map(|it| it.blocks.iter()).find_map(|b| {
                    if let Block::CodeBlock { lang, code, .. } = b {
                        Some((lang.clone(), code.clone()))
                    } else {
                        None
                    }
                });

                let (found_lang, found_code) = nested_code.unwrap_or_else(|| {
                    panic!(
                        "expected a CodeBlock nested inside a list item, but the \
                         list items had no nested code block. Items: {:?}",
                        items
                    )
                });

                prop_assert_eq!(
                    found_lang.as_deref(),
                    Some(lang.as_str()),
                    "nested code block language should be preserved"
                );

                let expected_code = code_lines.join("\n");
                prop_assert_eq!(
                    found_code.trim_end(),
                    expected_code.trim_end(),
                    "nested code block content should be preserved"
                );
            }

            NestedCase::TableInBlockQuote { headers, rows } => {
                // The document must contain a BlockQuote at the top level.
                let quote_content = doc.blocks.iter().find_map(|b| match b {
                    Block::BlockQuote { content, .. } => Some(content),
                    _ => None,
                });
                let content = quote_content.unwrap_or_else(|| {
                    panic!("expected a top-level BlockQuote block, got: {:?}", doc.blocks)
                });

                // The block quote must contain a Table with matching dimensions.
                let table = content.iter().find_map(|b| match b {
                    Block::Table { headers, rows, .. } => Some((headers.len(), rows.len())),
                    _ => None,
                });
                let (n_headers, n_rows) = table.unwrap_or_else(|| {
                    panic!(
                        "expected a Table nested inside the block quote, got: {:?}",
                        content
                    )
                });

                prop_assert_eq!(
                    n_headers,
                    headers.len(),
                    "nested table column count should match"
                );
                prop_assert_eq!(n_rows, rows.len(), "nested table row count should match");
            }
        }
    }
}
