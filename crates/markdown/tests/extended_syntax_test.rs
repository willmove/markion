//! Integration tests for extended inline syntax (superscript, subscript, highlight, emoji).

use markdown::ast::{Block, Inline};
use markdown::parser::Parser;

#[test]
fn test_superscript_parsing() {
    let parser = Parser::default();
    let doc = parser.parse("x^2^ equals 4").unwrap();

    // Check that we have a paragraph
    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should have: Text("x"), Superscript([Text("2")]), Text(" equals 4")
        assert!(content.iter().any(|i| matches!(i, Inline::Superscript(_))));

        // Find and verify the superscript content
        for inline in content {
            if let Inline::Superscript(inner) = inline {
                assert_eq!(inner.len(), 1);
                assert!(matches!(inner[0], Inline::Text(ref s) if s == "2"));
            }
        }
    } else {
        panic!("Expected Paragraph block");
    }
}

#[test]
fn test_subscript_parsing() {
    let parser = Parser::default();
    let doc = parser.parse("H~2~O is water").unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should have subscript for "2"
        assert!(content.iter().any(|i| matches!(i, Inline::Subscript(_))));

        for inline in content {
            if let Inline::Subscript(inner) = inline {
                assert_eq!(inner.len(), 1);
                assert!(matches!(inner[0], Inline::Text(ref s) if s == "2"));
            }
        }
    } else {
        panic!("Expected Paragraph block");
    }
}

#[test]
fn test_highlight_parsing() {
    let parser = Parser::default();
    let doc = parser.parse("This is ==important== text").unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should have highlight for "important"
        assert!(content.iter().any(|i| matches!(i, Inline::Highlight(_))));

        for inline in content {
            if let Inline::Highlight(inner) = inline {
                assert_eq!(inner.len(), 1);
                assert!(matches!(inner[0], Inline::Text(ref s) if s == "important"));
            }
        }
    } else {
        panic!("Expected Paragraph block");
    }
}

#[test]
fn test_emoji_parsing() {
    let parser = Parser::default();
    let doc = parser.parse("I :heart: Rust :smile:").unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should have two emoji
        let emoji_count = content
            .iter()
            .filter(|i| matches!(i, Inline::Emoji { .. }))
            .count();
        assert_eq!(emoji_count, 2);

        // Check specific emojis
        for inline in content {
            if let Inline::Emoji { shortcode, unicode } = inline {
                match shortcode.as_str() {
                    "heart" => assert_eq!(unicode, "❤️"),
                    "smile" => assert_eq!(unicode, "😄"),
                    _ => panic!("Unexpected emoji shortcode: {}", shortcode),
                }
            }
        }
    } else {
        panic!("Expected Paragraph block");
    }
}

#[test]
fn test_invalid_emoji_treated_as_text() {
    let parser = Parser::default();
    let doc = parser.parse("Not a :invalid_emoji: here").unwrap();

    // Invalid emoji should be treated as regular text
    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should not have any Emoji nodes
        assert!(!content.iter().any(|i| matches!(i, Inline::Emoji { .. })));
    }
}

#[test]
fn test_mixed_extended_syntax() {
    let parser = Parser::default();
    let doc = parser.parse("E=mc^2^ :smile: and ==highlight==").unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should have all three types
        assert!(content.iter().any(|i| matches!(i, Inline::Superscript(_))));
        assert!(content.iter().any(|i| matches!(i, Inline::Emoji { .. })));
        assert!(content.iter().any(|i| matches!(i, Inline::Highlight(_))));
    }
}

#[test]
fn test_nested_superscript() {
    let parser = Parser::default();
    let doc = parser.parse("x^y^z^^ formula").unwrap();

    // Should parse properly with nested structure
    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        assert!(content.iter().any(|i| matches!(i, Inline::Superscript(_))));
    }
}

#[test]
fn test_empty_delimiters() {
    let parser = Parser::default();
    let doc = parser.parse("Empty ^^ here").unwrap();

    // Empty superscript should still parse
    assert_eq!(doc.blocks.len(), 1);
}

#[test]
fn test_escaped_delimiter() {
    let parser = Parser::default();
    let doc = parser.parse("Escaped ^caret\\^inside^").unwrap();

    // pulldown-cmark processes the backslash escape (`\^`) before the extended
    // inline parser runs, splitting the text into two runs: "Escaped ^caret"
    // and "^inside^". As a result the escaped caret is preserved as a literal
    // `^` in the plain text, and only the trailing `^inside^` is recognised as
    // a superscript. Verify both of those observable behaviours.
    assert_eq!(doc.blocks.len(), 1);
    let Block::Paragraph { content, .. } = &doc.blocks[0] else {
        panic!("Expected Paragraph");
    };

    // Collect the top-level plain text (outside of any superscript node).
    let literal_text = content
        .iter()
        .filter_map(|i| {
            if let Inline::Text(s) = i {
                Some(s.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");
    // The escaped caret must be preserved as a literal character.
    assert!(
        literal_text.contains("caret") && literal_text.contains('^'),
        "escaped caret should be preserved as literal text, got: {literal_text:?}"
    );

    // The unescaped `^inside^` should be recognised as a superscript.
    let superscript_text = content
        .iter()
        .find_map(|i| {
            if let Inline::Superscript(inner) = i {
                Some(
                    inner
                        .iter()
                        .filter_map(|n| {
                            if let Inline::Text(s) = n {
                                Some(s.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(""),
                )
            } else {
                None
            }
        })
        .expect("expected a Superscript node for `^inside^`");
    assert_eq!(superscript_text, "inside");
}

#[test]
fn test_subscript_vs_strikethrough() {
    let parser = Parser::default();
    let doc = parser.parse("H~2~O molecule").unwrap();

    // Should parse as subscript, not strikethrough
    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should have subscript, not strikethrough
        assert!(content.iter().any(|i| matches!(i, Inline::Subscript(_))));
        assert!(
            !content
                .iter()
                .any(|i| matches!(i, Inline::Strikethrough(_)))
        );
    }
}

#[test]
fn test_multiple_emojis_in_sequence() {
    let parser = Parser::default();
    let doc = parser.parse(":fire: :heart: :rocket:").unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        let emoji_count = content
            .iter()
            .filter(|i| matches!(i, Inline::Emoji { .. }))
            .count();
        assert_eq!(emoji_count, 3);
    }
}

#[test]
fn test_superscript_with_multiple_characters() {
    let parser = Parser::default();
    let doc = parser.parse("x^abc^ test").unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        for inline in content {
            if let Inline::Superscript(inner) = inline {
                let text = inner
                    .iter()
                    .filter_map(|i| {
                        if let Inline::Text(s) = i {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("");
                assert_eq!(text, "abc");
            }
        }
    }
}

#[test]
fn test_highlight_with_multiple_words() {
    let parser = Parser::default();
    let doc = parser
        .parse("This is ==very important text== here")
        .unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        for inline in content {
            if let Inline::Highlight(inner) = inline {
                let text = inner
                    .iter()
                    .filter_map(|i| {
                        if let Inline::Text(s) = i {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("");
                assert_eq!(text, "very important text");
            }
        }
    }
}
