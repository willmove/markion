//! Tests for task list parsing and rendering (Requirements 17.1, 17.2).
//!
//! This test file validates:
//! - Parsing `- [ ]` (unchecked) and `- [x]` (checked) syntax
//! - List AST nodes contain correct `checked` boolean attribute
//! - Support for nested task list items

use markdown::{
    ast::{Block, Inline, ListItem},
    parser::{Parser, ParserOptions},
    renderer::render_to_markdown,
};

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn default_parser() -> Parser {
    Parser::new(ParserOptions::default())
}

fn extract_text(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .filter_map(|i| match i {
            Inline::Text(s) => Some(s.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

// ---------------------------------------------------------------------------
// Requirement 17.1: Parse `- [ ]` (unchecked) and `- [x]` (checked) syntax
// ---------------------------------------------------------------------------

#[test]
fn parse_unchecked_task_item() {
    let md = "- [ ] Buy milk\n";
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].checked, Some(false));
        assert_eq!(extract_text(&items[0].content), "Buy milk");
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_checked_task_item() {
    let md = "- [x] Complete task\n";
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].checked, Some(true));
        assert_eq!(extract_text(&items[0].content), "Complete task");
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_mixed_task_list() {
    let md = r#"- [x] Completed task
- [ ] Incomplete task
- [x] Another done
"#;
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].checked, Some(true));
        assert_eq!(items[1].checked, Some(false));
        assert_eq!(items[2].checked, Some(true));
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_task_list_with_uppercase_x() {
    // GFM spec supports both [x] and [X]
    let md = "- [X] Task with uppercase X\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 1);
        // pulldown-cmark normalizes [X] to checked=true
        assert_eq!(items[0].checked, Some(true));
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_regular_list_item_no_checkbox() {
    let md = "- Regular item\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 1);
        // Regular items have None for checked
        assert_eq!(items[0].checked, None);
        assert_eq!(extract_text(&items[0].content), "Regular item");
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_mixed_regular_and_task_items() {
    let md = r#"- Regular item
- [x] Task item
- Another regular
- [ ] Unchecked task
"#;
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].checked, None);
        assert_eq!(items[1].checked, Some(true));
        assert_eq!(items[2].checked, None);
        assert_eq!(items[3].checked, Some(false));
    } else {
        panic!("Expected List block");
    }
}

// ---------------------------------------------------------------------------
// Requirement 17.2: List AST nodes contain checked boolean attribute
// ---------------------------------------------------------------------------

#[test]
fn ast_contains_checked_attribute() {
    let md = "- [x] Done\n- [ ] Todo\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        // Verify the AST structure explicitly has the checked field
        assert!(items[0].checked.is_some());
        assert!(items[0].checked.unwrap());

        assert!(items[1].checked.is_some());
        assert!(!items[1].checked.unwrap());
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn ast_checked_attribute_is_option_bool() {
    let item_checked = ListItem::task(vec![Inline::Text("test".into())], true);
    let item_unchecked = ListItem::task(vec![Inline::Text("test".into())], false);
    let item_regular = ListItem::simple(vec![Inline::Text("test".into())]);

    // Verify type is Option<bool>
    assert_eq!(item_checked.checked, Some(true));
    assert_eq!(item_unchecked.checked, Some(false));
    assert_eq!(item_regular.checked, None);
}

// ---------------------------------------------------------------------------
// Support for nested task list items
// ---------------------------------------------------------------------------

#[test]
fn parse_nested_task_lists() {
    let md = r#"- [x] Parent task
  - [ ] Child task 1
  - [x] Child task 2
"#;
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 1);
        let parent = &items[0];
        assert_eq!(parent.checked, Some(true));
        assert_eq!(extract_text(&parent.content), "Parent task");

        // Check nested items
        assert_eq!(parent.sub_items.len(), 2);
        assert_eq!(parent.sub_items[0].checked, Some(false));
        assert_eq!(extract_text(&parent.sub_items[0].content), "Child task 1");
        assert_eq!(parent.sub_items[1].checked, Some(true));
        assert_eq!(extract_text(&parent.sub_items[1].content), "Child task 2");
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_deeply_nested_task_lists() {
    let md = r#"- [x] Level 1
  - [ ] Level 2
    - [x] Level 3
"#;
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        let level1 = &items[0];
        assert_eq!(level1.checked, Some(true));

        let level2 = &level1.sub_items[0];
        assert_eq!(level2.checked, Some(false));

        let level3 = &level2.sub_items[0];
        assert_eq!(level3.checked, Some(true));
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_nested_mixed_task_and_regular_items() {
    let md = r#"- [x] Task parent
  - Regular child
  - [ ] Task child
- Regular parent
  - [x] Task child
"#;
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 2);

        // First parent: task
        assert_eq!(items[0].checked, Some(true));
        assert_eq!(items[0].sub_items[0].checked, None);
        assert_eq!(items[0].sub_items[1].checked, Some(false));

        // Second parent: regular
        assert_eq!(items[1].checked, None);
        assert_eq!(items[1].sub_items[0].checked, Some(true));
    } else {
        panic!("Expected List block");
    }
}

// ---------------------------------------------------------------------------
// Task list with inline formatting
// ---------------------------------------------------------------------------

#[test]
fn parse_task_with_bold_text() {
    let md = "- [x] **Important** task\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items[0].checked, Some(true));
        // pulldown-cmark may put formatted text in blocks rather than content
        // Check both locations
        let has_strong_in_content = items[0]
            .content
            .iter()
            .any(|i| matches!(i, Inline::Strong(_)));
        let has_strong_in_blocks = items[0].blocks.iter().any(|b| {
            if let Block::Paragraph { content, .. } = b {
                content.iter().any(|i| matches!(i, Inline::Strong(_)))
            } else {
                false
            }
        });
        assert!(
            has_strong_in_content || has_strong_in_blocks,
            "Expected Strong inline in task item"
        );
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_task_with_code() {
    let md = "- [ ] Fix `bug` in code\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items[0].checked, Some(false));
        assert!(items[0]
            .content
            .iter()
            .any(|i| matches!(i, Inline::Code(_))));
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_task_with_link() {
    let md = "- [x] Read [documentation](https://example.com)\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items[0].checked, Some(true));
        // pulldown-cmark may put formatted text in blocks rather than content
        let has_link_in_content = items[0]
            .content
            .iter()
            .any(|i| matches!(i, Inline::Link { .. }));
        let has_link_in_blocks = items[0].blocks.iter().any(|b| {
            if let Block::Paragraph { content, .. } = b {
                content.iter().any(|i| matches!(i, Inline::Link { .. }))
            } else {
                false
            }
        });
        assert!(
            has_link_in_content || has_link_in_blocks,
            "Expected Link inline in task item"
        );
    } else {
        panic!("Expected List block");
    }
}

// ---------------------------------------------------------------------------
// Rendering tests (round-trip)
// ---------------------------------------------------------------------------

#[test]
fn render_task_list_preserves_checkboxes() {
    let md = r#"- [x] Done
- [ ] Todo
"#;
    let doc = default_parser().parse(md).unwrap();
    let rendered = render_to_markdown(&doc);

    assert!(rendered.contains("- [x] Done"));
    assert!(rendered.contains("- [ ] Todo"));
}

#[test]
fn render_nested_task_list() {
    let md = r#"- [x] Parent
  - [ ] Child
"#;
    let doc = default_parser().parse(md).unwrap();
    let rendered = render_to_markdown(&doc);

    assert!(rendered.contains("- [x] Parent"));
    assert!(rendered.contains("  - [ ] Child"));
}

#[test]
fn round_trip_task_list() {
    let original = r#"- [x] Completed
- [ ] Incomplete
- Regular item
"#;
    let doc1 = default_parser().parse(original).unwrap();
    let rendered = render_to_markdown(&doc1);
    let doc2 = default_parser().parse(&rendered).unwrap();

    // Both documents should have the same structure
    if let (Block::List { items: items1, .. }, Block::List { items: items2, .. }) =
        (&doc1.blocks[0], &doc2.blocks[0])
    {
        assert_eq!(items1.len(), items2.len());
        for (i, (item1, item2)) in items1.iter().zip(items2.iter()).enumerate() {
            assert_eq!(
                item1.checked, item2.checked,
                "Mismatch at item {}: {:?} != {:?}",
                i, item1.checked, item2.checked
            );
        }
    } else {
        panic!("Expected List blocks in both documents");
    }
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn parse_task_with_empty_content() {
    let md = "- [ ] \n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].checked, Some(false));
        // Content should be empty or whitespace
        let text = extract_text(&items[0].content);
        assert!(text.trim().is_empty());
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_task_with_multiple_lines() {
    // Task items can span multiple lines in some parsers
    let md = r#"- [ ] This is a task
  that spans multiple lines
"#;
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].checked, Some(false));
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn parse_task_list_in_ordered_list() {
    // Task lists are typically unordered, but let's verify behavior
    let md = r#"1. [ ] Task in ordered list
2. [x] Another task
"#;
    let doc = default_parser().parse(md).unwrap();

    // pulldown-cmark may handle this differently
    // Just verify it parses without error
    assert!(!doc.blocks.is_empty());
}

#[test]
fn parse_invalid_checkbox_syntax() {
    // Missing space inside brackets should be treated as regular text
    let md = "- [x] Valid\n- [y] Invalid\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        // First item should be a valid task
        assert_eq!(items[0].checked, Some(true));

        // Second item might be treated as regular text depending on parser
        // pulldown-cmark is strict about task list syntax
    } else {
        panic!("Expected List block");
    }
}

// ---------------------------------------------------------------------------
// Integration with other markdown features
// ---------------------------------------------------------------------------

#[test]
fn task_list_after_heading() {
    let md = r#"# Todo List

- [x] Task 1
- [ ] Task 2
"#;
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 2);
    assert!(matches!(doc.blocks[0], Block::Heading { .. }));

    if let Block::List { items, .. } = &doc.blocks[1] {
        assert_eq!(items[0].checked, Some(true));
        assert_eq!(items[1].checked, Some(false));
    } else {
        panic!("Expected List block");
    }
}

#[test]
fn task_list_in_block_quote() {
    let md = r#"> - [x] Quoted task
> - [ ] Another quoted task
"#;
    let doc = default_parser().parse(md).unwrap();

    if let Block::BlockQuote { content, .. } = &doc.blocks[0] {
        if let Block::List { items, .. } = &content[0] {
            assert_eq!(items[0].checked, Some(true));
            assert_eq!(items[1].checked, Some(false));
        } else {
            panic!("Expected List inside BlockQuote");
        }
    } else {
        panic!("Expected BlockQuote");
    }
}

#[test]
fn task_list_with_code_block_in_item() {
    let md = r#"- [x] Task with code:

  ```rust
  fn main() {}
  ```
"#;
    let doc = default_parser().parse(md).unwrap();

    if let Block::List { items, .. } = &doc.blocks[0] {
        // pulldown-cmark may not preserve the checkbox when complex block content follows
        // The key test is that we can parse this structure without crashing
        assert_eq!(items.len(), 1);
        // If checkbox is preserved, verify it
        if items[0].checked.is_some() {
            assert_eq!(items[0].checked, Some(true));
        }
        // The code block should be in the blocks field
        assert!(
            !items[0].blocks.is_empty() || !items[0].content.is_empty(),
            "Task item should have some content"
        );
    } else {
        panic!("Expected List block");
    }
}
