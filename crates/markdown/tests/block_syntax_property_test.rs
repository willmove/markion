//! Property-based test for Parser block-level syntax AST creation.
//!
//! **Property 3: 块级语法 AST 创建**
//!
//! **Validates: Requirements 1.2**
//!
//! For any block-level Markdown syntax (heading `# heading`, list `- item`,
//! blockquote `> quote`), the parser should create the corresponding type of
//! Block AST node.

use markdown::ast::Block;
use markdown::parser::{Parser, ParserOptions};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

/// The kind of block we expect the parser to produce, along with any relevant
/// structural expectations (e.g. heading level, list ordering).
#[derive(Debug, Clone)]
enum ExpectedBlock {
    Heading { level: u8 },
    List { ordered: bool },
    BlockQuote,
}

/// Generate a short, safe piece of inline text made of alphanumeric words.
///
/// Restricting to alphanumerics (plus single spaces) avoids accidentally
/// producing other Markdown constructs inside the block content.
fn arb_text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,8}( [a-zA-Z0-9]{1,8}){0,3}"
        .prop_map(|s| s.trim().to_string())
        .prop_filter("non-empty text", |s| !s.is_empty())
}

/// Generate a heading: `#`..`######` followed by a space and text.
fn arb_heading() -> impl Strategy<Value = (String, ExpectedBlock)> {
    (1u8..=6, arb_text()).prop_map(|(level, text)| {
        let hashes = "#".repeat(level as usize);
        let markdown = format!("{hashes} {text}\n");
        (markdown, ExpectedBlock::Heading { level })
    })
}

/// Generate an unordered list with one or more items using a `-`, `*`, or `+`
/// marker.
fn arb_unordered_list() -> impl Strategy<Value = (String, ExpectedBlock)> {
    (
        prop_oneof![Just('-'), Just('*'), Just('+')],
        prop::collection::vec(arb_text(), 1..4),
    )
        .prop_map(|(marker, items)| {
            let mut markdown = String::new();
            for item in &items {
                markdown.push_str(&format!("{marker} {item}\n"));
            }
            (markdown, ExpectedBlock::List { ordered: false })
        })
}

/// Generate an ordered list with one or more numbered items.
fn arb_ordered_list() -> impl Strategy<Value = (String, ExpectedBlock)> {
    prop::collection::vec(arb_text(), 1..4).prop_map(|items| {
        let mut markdown = String::new();
        for (idx, item) in items.iter().enumerate() {
            markdown.push_str(&format!("{}. {item}\n", idx + 1));
        }
        (markdown, ExpectedBlock::List { ordered: true })
    })
}

/// Generate a blockquote with one or more quoted lines.
fn arb_blockquote() -> impl Strategy<Value = (String, ExpectedBlock)> {
    prop::collection::vec(arb_text(), 1..4).prop_map(|lines| {
        let mut markdown = String::new();
        for line in &lines {
            markdown.push_str(&format!("> {line}\n"));
        }
        (markdown, ExpectedBlock::BlockQuote)
    })
}

/// Generate any of the supported block-level syntaxes together with the block
/// type we expect the parser to produce.
fn arb_block_syntax() -> impl Strategy<Value = (String, ExpectedBlock)> {
    prop_oneof![
        arb_heading(),
        arb_unordered_list(),
        arb_ordered_list(),
        arb_blockquote(),
    ]
}

// ---------------------------------------------------------------------------
// Property test
// ---------------------------------------------------------------------------

proptest! {
    /// Property 3: Block-level syntax AST creation
    ///
    /// Validates: Requirements 1.2
    ///
    /// For any generated block-level Markdown syntax (heading, list, or
    /// blockquote), the parser creates a single top-level block of the
    /// corresponding `Block` variant, with matching structural attributes
    /// (heading level, list ordering).
    #[test]
    fn test_block_syntax_creates_expected_ast_node(
        (markdown, expected) in arb_block_syntax()
    ) {
        let parser = Parser::new(ParserOptions::default());
        let doc = parser
            .parse(&markdown)
            .expect("parsing valid block syntax should succeed");

        prop_assert!(
            !doc.blocks.is_empty(),
            "expected at least one block for input {:?}",
            markdown
        );

        // The block-level syntaxes we generate each produce exactly one
        // top-level block.
        prop_assert_eq!(
            doc.blocks.len(),
            1,
            "expected exactly one top-level block for input {:?}, got {:?}",
            markdown,
            doc.blocks
        );

        let block = &doc.blocks[0];

        match &expected {
            ExpectedBlock::Heading { level } => {
                match block {
                    Block::Heading { level: actual_level, .. } => {
                        prop_assert_eq!(
                            actual_level,
                            level,
                            "heading level mismatch for input {:?}",
                            markdown
                        );
                    }
                    other => prop_assert!(
                        false,
                        "expected Heading for input {:?}, got {:?}",
                        markdown,
                        other
                    ),
                }
            }
            ExpectedBlock::List { ordered } => {
                match block {
                    Block::List { ordered: actual_ordered, .. } => {
                        prop_assert_eq!(
                            actual_ordered,
                            ordered,
                            "list ordering mismatch for input {:?}",
                            markdown
                        );
                    }
                    other => prop_assert!(
                        false,
                        "expected List for input {:?}, got {:?}",
                        markdown,
                        other
                    ),
                }
            }
            ExpectedBlock::BlockQuote => {
                prop_assert!(
                    matches!(block, Block::BlockQuote { .. }),
                    "expected BlockQuote for input {:?}, got {:?}",
                    markdown,
                    block
                );
            }
        }
    }
}
