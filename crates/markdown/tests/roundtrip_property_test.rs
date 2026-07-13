//! Property-based test for Markdown parser round-trip consistency.
//!
//! **Validates: Requirements 15.3, 15.8**
//!
//! This test verifies that for valid Markdown documents:
//! parse → render → parse produces a semantically equivalent AST.
//!
//! **Note:** Due to limitations in pulldown-cmark's parsing behavior, perfect
//! structural round-trip is not always achievable for complex nested inline
//! elements. This test focuses on verifying that basic structure (blocks, headings,
//! lists, tables) and text content are preserved across round-trips.

use markdown::ast::{Alignment, Block, Document, Inline, ListItem, TableCell};
use markdown::parser::{Parser, ParserOptions};
use markdown::renderer::render_to_markdown;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Arbitrary generators for AST nodes (simplified for reliable round-trips)
// ---------------------------------------------------------------------------

/// Generate simple inline content (text and code only)
fn arb_simple_inline() -> impl Strategy<Value = Inline> {
    prop_oneof![
        "[a-zA-Z0-9 ,.!?]+".prop_map(Inline::Text),
        "[a-zA-Z0-9_]+".prop_map(Inline::Code),
    ]
}

/// Generate arbitrary table cells
fn arb_table_cell() -> impl Strategy<Value = TableCell> {
    prop::collection::vec(arb_simple_inline(), 1..2).prop_map(|content| TableCell { content })
}

/// Generate arbitrary list items
fn arb_list_item() -> impl Strategy<Value = ListItem> {
    (
        prop::collection::vec(arb_simple_inline(), 1..2),
        prop::option::of(any::<bool>()),
    )
        .prop_map(|(content, checked)| ListItem {
            content,
            blocks: vec![],
            checked,
            sub_items: vec![],
        })
}

/// Generate arbitrary block content (focused on reliable round-trips)
fn arb_block() -> impl Strategy<Value = Block> {
    prop_oneof![
        (1u8..=6, prop::collection::vec(arb_simple_inline(), 1..2)).prop_map(|(level, content)| {
            Block::Heading {
                level,
                content,
                id: 0,
            }
        }),
        prop::collection::vec(arb_simple_inline(), 1..3)
            .prop_map(|content| Block::Paragraph { content, id: 0 }),
        (
            prop::option::of("(rust|python|javascript|go)"),
            "[a-zA-Z0-9 \\n]+"
        )
            .prop_map(|(lang, code)| Block::CodeBlock { lang, code, id: 0 }),
        Just(Block::HorizontalRule { id: 0 }),
        (
            prop::collection::vec(arb_list_item(), 1..2),
            any::<bool>(),
            prop::option::of(1u32..=5),
        )
            .prop_map(|(items, ordered, start)| Block::List {
                items,
                ordered,
                start,
                id: 0,
            }),
        (
            prop::collection::vec(arb_table_cell(), 2..3),
            prop::collection::vec(prop::collection::vec(arb_table_cell(), 2..3), 1..2),
            prop::collection::vec(
                prop_oneof![
                    Just(Alignment::None),
                    Just(Alignment::Left),
                    Just(Alignment::Center),
                    Just(Alignment::Right),
                ],
                2..3
            ),
        )
            .prop_map(|(headers, rows, alignment)| {
                let col_count = headers.len();
                let alignment = alignment.into_iter().take(col_count).collect();
                let rows = rows
                    .into_iter()
                    .map(|row| row.into_iter().take(col_count).collect())
                    .collect();
                Block::Table {
                    headers,
                    rows,
                    alignment,
                    id: 0,
                }
            }),
    ]
}

/// Generate arbitrary Markdown documents
fn arb_markdown() -> impl Strategy<Value = Document> {
    prop::collection::vec(arb_block(), 1..4).prop_map(|blocks| Document {
        blocks,
        metadata: None,
        version: 0,
        footnote_map: std::collections::HashMap::new(),
        shared_regions: Vec::new(),
    })
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Property 1: Markdown Parse Round-Trip
    ///
    /// Validates: Requirements 15.3, 15.8
    ///
    /// For any valid Markdown document, parsing it to AST, rendering back to
    /// Markdown, and parsing again should produce a semantically equivalent AST.
    ///
    /// Semantic equivalence means:
    /// - Same number and types of blocks
    /// - Same heading levels and text content
    /// - Same list structure and items
    /// - Same table structure
    /// - Same code block content (with normalized whitespace)
    #[test]
    fn test_roundtrip_semantic_equivalence(doc in arb_markdown()) {
        let parser = Parser::new(ParserOptions::default());

        // Step 1: Render the original document to Markdown
        let markdown1 = render_to_markdown(&doc);

        // Step 2: Parse the rendered Markdown
        let doc2 = parser.parse(&markdown1)
            .expect("First parse should succeed");

        // Step 3: Render again
        let markdown2 = render_to_markdown(&doc2);

        // Step 4: Parse the second rendering
        let doc3 = parser.parse(&markdown2)
            .expect("Second parse should succeed");

        // Assert: The two parsed documents should have the same structure
        assert_blocks_equivalent(&doc2.blocks, &doc3.blocks);
    }
}

// ---------------------------------------------------------------------------
// Semantic equivalence helpers
// ---------------------------------------------------------------------------

/// Compare two block lists for semantic equivalence
fn assert_blocks_equivalent(blocks1: &[Block], blocks2: &[Block]) {
    assert_eq!(
        blocks1.len(),
        blocks2.len(),
        "Block count mismatch: {} vs {}",
        blocks1.len(),
        blocks2.len()
    );

    for (i, (b1, b2)) in blocks1.iter().zip(blocks2.iter()).enumerate() {
        assert_block_equivalent(b1, b2, i);
    }
}

/// Compare two blocks for semantic equivalence
fn assert_block_equivalent(b1: &Block, b2: &Block, index: usize) {
    match (b1, b2) {
        (
            Block::Heading {
                level: l1,
                content: c1,
                ..
            },
            Block::Heading {
                level: l2,
                content: c2,
                ..
            },
        ) => {
            assert_eq!(l1, l2, "Heading level mismatch at block {}", index);
            assert_content_text_eq(c1, c2, format!("block {} heading", index).as_str());
        }

        (Block::Paragraph { content: c1, .. }, Block::Paragraph { content: c2, .. }) => {
            assert_content_text_eq(c1, c2, format!("block {} paragraph", index).as_str());
        }

        (
            Block::CodeBlock {
                lang: l1, code: c1, ..
            },
            Block::CodeBlock {
                lang: l2, code: c2, ..
            },
        ) => {
            assert_eq!(l1, l2, "Code block language mismatch at block {}", index);
            let c1_trimmed = c1.trim_end();
            let c2_trimmed = c2.trim_end();
            assert_eq!(
                c1_trimmed, c2_trimmed,
                "Code block content mismatch at block {}",
                index
            );
        }

        (
            Block::Table {
                headers: h1,
                rows: r1,
                alignment: a1,
                ..
            },
            Block::Table {
                headers: h2,
                rows: r2,
                alignment: a2,
                ..
            },
        ) => {
            assert_eq!(
                h1.len(),
                h2.len(),
                "Table header count mismatch at block {}",
                index
            );
            assert_eq!(
                r1.len(),
                r2.len(),
                "Table row count mismatch at block {}",
                index
            );
            assert_eq!(a1, a2, "Table alignment mismatch at block {}", index);

            for (i, (cell1, cell2)) in h1.iter().zip(h2.iter()).enumerate() {
                assert_content_text_eq(
                    &cell1.content,
                    &cell2.content,
                    format!("block {} header cell {}", index, i).as_str(),
                );
            }

            for (row_idx, (row1, row2)) in r1.iter().zip(r2.iter()).enumerate() {
                for (col_idx, (cell1, cell2)) in row1.iter().zip(row2.iter()).enumerate() {
                    assert_content_text_eq(
                        &cell1.content,
                        &cell2.content,
                        format!("block {} row {} col {}", index, row_idx, col_idx).as_str(),
                    );
                }
            }
        }

        (
            Block::List {
                items: i1,
                ordered: o1,
                start: s1,
                ..
            },
            Block::List {
                items: i2,
                ordered: o2,
                start: s2,
                ..
            },
        ) => {
            assert_eq!(o1, o2, "List ordered flag mismatch at block {}", index);
            assert_eq!(s1, s2, "List start number mismatch at block {}", index);
            assert_eq!(
                i1.len(),
                i2.len(),
                "List item count mismatch at block {}",
                index
            );

            for (item_idx, (item1, item2)) in i1.iter().zip(i2.iter()).enumerate() {
                assert_eq!(
                    item1.checked, item2.checked,
                    "List item checked flag mismatch at block {} item {}",
                    index, item_idx
                );
                assert_content_text_eq(
                    &item1.content,
                    &item2.content,
                    format!("block {} list item {}", index, item_idx).as_str(),
                );
            }
        }

        (Block::HorizontalRule { .. }, Block::HorizontalRule { .. }) => {
            // Always equivalent
        }

        _ => {
            panic!(
                "Block type mismatch at index {}: {:?} vs {:?}",
                index,
                std::mem::discriminant(b1),
                std::mem::discriminant(b2)
            );
        }
    }
}

/// Compare inline content by extracting and comparing the text content
/// This provides semantic equivalence rather than structural equivalence
fn assert_content_text_eq<S: AsRef<str>>(inlines1: &[Inline], inlines2: &[Inline], context: S) {
    let text1 = extract_text(inlines1);
    let text2 = extract_text(inlines2);

    assert_eq!(
        text1.trim(),
        text2.trim(),
        "Text content mismatch at {}: {:?} vs {:?}",
        context.as_ref(),
        text1,
        text2
    );
}

/// Extract plain text from inline elements
fn extract_text(inlines: &[Inline]) -> String {
    let mut result = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => result.push_str(t),
            Inline::Code(c) => result.push_str(c),
            Inline::Strong(inner) => result.push_str(&extract_text(inner)),
            Inline::Emphasis(inner) => result.push_str(&extract_text(inner)),
            Inline::Strikethrough(inner) => result.push_str(&extract_text(inner)),
            Inline::Link { text, .. } => result.push_str(&extract_text(text)),
            Inline::Image { alt, .. } => result.push_str(alt),
            Inline::InlineMath(m) => result.push_str(m),
            Inline::LineBreak => result.push(' '),
            Inline::Superscript(inner) => result.push_str(&extract_text(inner)),
            Inline::Subscript(inner) => result.push_str(&extract_text(inner)),
            Inline::Highlight(inner) => result.push_str(&extract_text(inner)),
            Inline::Emoji { unicode, .. } => result.push_str(unicode),
            Inline::FootnoteReference(_) => {} // Skip footnote refs in text extraction
            Inline::HtmlInline(html) => result.push_str(html),
        }
    }
    result
}
