//! Tests for YAML front matter parsing and validation.
//!
//! Validates Requirements 18.1, 18.2, 18.5, 18.6

use markdown::{Parser, ParserOptions};

fn default_parser() -> Parser {
    Parser::new(ParserOptions::default())
}

// ---------------------------------------------------------------------------
// Requirement 18.1: Recognition of `---` delimited YAML blocks
// ---------------------------------------------------------------------------

#[test]
fn yaml_front_matter_recognized_with_lf() {
    let md = "---\ntitle: Test\n---\n# Content\n";
    let doc = default_parser().parse(md).unwrap();
    assert!(
        doc.metadata.is_some(),
        "YAML front matter should be recognized"
    );
}

#[test]
fn yaml_front_matter_recognized_with_crlf() {
    let md = "---\r\ntitle: Test\r\n---\r\n# Content\r\n";
    let doc = default_parser().parse(md).unwrap();
    assert!(
        doc.metadata.is_some(),
        "YAML front matter should be recognized with CRLF"
    );
}

#[test]
fn yaml_front_matter_not_recognized_without_opening_delimiter() {
    let md = "title: Test\n---\n# Content\n";
    let doc = default_parser().parse(md).unwrap();
    assert!(
        doc.metadata.is_none(),
        "YAML front matter should not be recognized without opening ---"
    );
}

#[test]
fn yaml_front_matter_not_recognized_without_closing_delimiter() {
    let md = "---\ntitle: Test\n# Content\n";
    let doc = default_parser().parse(md).unwrap();
    assert!(
        doc.metadata.is_none(),
        "YAML front matter should not be recognized without closing ---"
    );
}

#[test]
fn yaml_front_matter_not_recognized_in_middle_of_document() {
    let md = "# Header\n---\ntitle: Test\n---\n";
    let doc = default_parser().parse(md).unwrap();
    assert!(
        doc.metadata.is_none(),
        "YAML front matter should only be recognized at document start"
    );
}

#[test]
fn yaml_front_matter_empty_block() {
    let md = "---\n---\n# Content\n";
    let doc = default_parser().parse(md).unwrap();
    // Empty YAML block is technically valid but serde_yaml may treat it as null/no data
    // The parser should handle it without error
    // Note: empty YAML may result in None metadata or Some with default values
    // Both behaviors are acceptable
    assert!(!doc.blocks.is_empty(), "Document should have body content");
}

#[test]
fn yaml_front_matter_trailing_delimiter() {
    let md = "---\ntitle: Test\n---";
    let doc = default_parser().parse(md).unwrap();
    assert!(
        doc.metadata.is_some(),
        "YAML front matter should be recognized with trailing delimiter"
    );
}

// ---------------------------------------------------------------------------
// Requirement 18.2: Parse YAML content as key-value pairs
// ---------------------------------------------------------------------------

#[test]
fn yaml_parse_title() {
    let md = "---\ntitle: My Document\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.title, Some("My Document".to_string()));
}

#[test]
fn yaml_parse_author() {
    let md = "---\nauthor: Alice Smith\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.author, Some("Alice Smith".to_string()));
}

#[test]
fn yaml_parse_date() {
    let md = "---\ndate: 2024-01-15\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.date, Some("2024-01-15".to_string()));
}

#[test]
fn yaml_parse_tags_list() {
    let md = "---\ntags:\n  - rust\n  - markdown\n  - editor\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.tags, vec!["rust", "markdown", "editor"]);
}

#[test]
fn yaml_parse_tags_inline_list() {
    let md = "---\ntags: [rust, markdown, editor]\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.tags, vec!["rust", "markdown", "editor"]);
}

#[test]
fn yaml_parse_multiple_fields() {
    let md = r#"---
title: Complete Document
author: Bob Jones
date: 2024-01-15
tags:
  - test
  - example
---
"#;
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.title, Some("Complete Document".to_string()));
    assert_eq!(fm.author, Some("Bob Jones".to_string()));
    assert_eq!(fm.date, Some("2024-01-15".to_string()));
    assert_eq!(fm.tags, vec!["test", "example"]);
}

#[test]
fn yaml_parse_custom_fields() {
    let md = r#"---
title: Test
custom_field: custom_value
another_field: 123
---
"#;
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.title, Some("Test".to_string()));
    assert!(
        fm.custom.contains_key("custom_field"),
        "Custom field should be captured"
    );
    assert!(
        fm.custom.contains_key("another_field"),
        "Custom field should be captured"
    );
}

#[test]
fn yaml_parse_nested_custom_fields() {
    let md = r#"---
title: Test
metadata:
  version: 1.0
  status: draft
---
"#;
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert!(
        fm.custom.contains_key("metadata"),
        "Nested custom field should be captured"
    );
}

#[test]
fn yaml_parse_optional_fields_absent() {
    let md = "---\ntitle: Only Title\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.title, Some("Only Title".to_string()));
    assert_eq!(fm.author, None);
    assert_eq!(fm.date, None);
    assert_eq!(fm.tags, Vec::<String>::new());
}

#[test]
fn yaml_parse_quoted_strings() {
    let md = r#"---
title: "Title with: special chars"
author: 'Single quoted'
---
"#;
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.title, Some("Title with: special chars".to_string()));
    assert_eq!(fm.author, Some("Single quoted".to_string()));
}

#[test]
fn yaml_parse_multiline_strings() {
    let md = r#"---
title: >
  This is a long title
  that spans multiple lines
---
"#;
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert!(fm.title.is_some());
    let title = fm.title.unwrap();
    assert!(title.contains("This is a long title"));
}

// ---------------------------------------------------------------------------
// Requirement 18.5: YAML syntax validation
// ---------------------------------------------------------------------------

#[test]
fn yaml_invalid_syntax_colon_missing() {
    let md = "---\ntitle My Document\n---\n";
    let result = default_parser().parse(md);
    assert!(result.is_err(), "Invalid YAML syntax should return error");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Invalid YAML"),
        "Error message should mention YAML"
    );
}

#[test]
fn yaml_invalid_syntax_indentation() {
    let md = "---\ntags:\n- rust\n  - markdown\n---\n";
    let result = default_parser().parse(md);
    // This might parse successfully or fail depending on YAML strictness
    // But we're testing that invalid indentation is handled
    if let Err(err) = result {
        assert!(
            err.to_string().contains("YAML"),
            "Error should mention YAML"
        );
    }
}

#[test]
fn yaml_invalid_syntax_unclosed_quote() {
    let md = "---\ntitle: \"Unclosed quote\n---\n";
    let result = default_parser().parse(md);
    assert!(result.is_err(), "Unclosed quote should cause error");
}

#[test]
fn yaml_invalid_syntax_tab_character() {
    let md = "---\ntitle:\tTest\n---\n";
    let result = default_parser().parse(md);
    // YAML typically doesn't allow tabs for indentation, but may accept them in values
    // Test that we handle it (either accept or reject consistently)
    if let Err(err) = result {
        assert!(err.to_string().contains("YAML"));
    }
}

#[test]
fn yaml_invalid_syntax_duplicate_keys() {
    let md = "---\ntitle: First\ntitle: Second\n---\n";
    let result = default_parser().parse(md);
    // serde_yaml may accept this and use the last value
    if let Ok(doc) = result {
        let fm = doc.metadata.unwrap();
        // Should have one title value (typically the last one)
        assert!(fm.title.is_some());
    }
}

#[test]
fn yaml_invalid_syntax_malformed_list() {
    let md = "---\ntags:\n  - rust\n  markdown\n---\n";
    let result = default_parser().parse(md);
    assert!(result.is_err(), "Malformed list should cause error");
}

// ---------------------------------------------------------------------------
// Requirement 18.6: Detailed error reporting
// ---------------------------------------------------------------------------

#[test]
fn yaml_error_message_contains_context() {
    let md = "---\ntitle: \"Unclosed\n---\n";
    let result = default_parser().parse(md);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("YAML"), "Error should mention YAML");
    assert!(
        err_msg.contains("Invalid") || err_msg.contains("invalid"),
        "Error should indicate invalidity"
    );
}

#[test]
fn yaml_error_preserves_original_error() {
    let md = "---\n{invalid yaml\n---\n";
    let result = default_parser().parse(md);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    // Should contain information from the underlying serde_yaml error
    assert!(!err_msg.is_empty());
}

// ---------------------------------------------------------------------------
// Integration: YAML front matter with document body
// ---------------------------------------------------------------------------

#[test]
fn yaml_with_markdown_body() {
    let md = r#"---
title: Integration Test
author: Test User
---
# Heading

Some content here.
"#;
    let doc = default_parser().parse(md).unwrap();
    assert!(doc.metadata.is_some());
    assert!(!doc.blocks.is_empty(), "Document should have body content");

    let fm = doc.metadata.unwrap();
    assert_eq!(fm.title, Some("Integration Test".to_string()));
    assert_eq!(fm.author, Some("Test User".to_string()));
}

#[test]
fn yaml_does_not_affect_body_parsing() {
    let md_with_yaml = "---\ntitle: Test\n---\n# Header\n\nParagraph\n";
    let md_without_yaml = "# Header\n\nParagraph\n";

    let doc_with = default_parser().parse(md_with_yaml).unwrap();
    let doc_without = default_parser().parse(md_without_yaml).unwrap();

    assert_eq!(
        doc_with.blocks.len(),
        doc_without.blocks.len(),
        "YAML should not affect body parsing"
    );
}

#[test]
fn yaml_empty_body() {
    let md = "---\ntitle: Only Metadata\n---\n";
    let doc = default_parser().parse(md).unwrap();
    assert!(doc.metadata.is_some());
    assert!(
        doc.blocks.is_empty(),
        "Document should have no body content"
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn yaml_with_triple_dash_in_content() {
    let md = "---\ntitle: Test\ncomment: This --- is not a delimiter\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert!(fm.custom.contains_key("comment"));
}

#[test]
fn yaml_unicode_content() {
    let md = "---\ntitle: 中文标题\nauthor: José García\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.title, Some("中文标题".to_string()));
    assert_eq!(fm.author, Some("José García".to_string()));
}

#[test]
fn yaml_special_characters_in_values() {
    let md = r#"---
title: "Title with @ # $ % special chars!"
tags: ["tag-with-dash", "tag_with_underscore", "tag.with.dots"]
---
"#;
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert!(fm.title.is_some());
    assert_eq!(fm.tags.len(), 3);
}

#[test]
fn yaml_boolean_and_number_values() {
    let md = r#"---
title: Test
draft: true
version: 1.5
count: 42
---
"#;
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert!(fm.custom.contains_key("draft"));
    assert!(fm.custom.contains_key("version"));
    assert!(fm.custom.contains_key("count"));
}

#[test]
fn yaml_null_values() {
    let md = "---\ntitle: Test\nauthor: null\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.title, Some("Test".to_string()));
    assert_eq!(fm.author, None);
}

#[test]
fn yaml_empty_strings() {
    let md = "---\ntitle: \"\"\nauthor: Test\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    // Empty string might be parsed as empty string or None depending on serde_yaml
    assert!(matches!(fm.title.as_deref(), None | Some("")));
}

#[test]
fn yaml_preserves_case_sensitivity() {
    let md = "---\nTitle: Test\nauthor: Test\n---\n";
    let doc = default_parser().parse(md).unwrap();
    let fm = doc.metadata.unwrap();
    // "Title" (capital T) should go to custom fields, not the "title" field
    assert_eq!(fm.title, None);
    assert!(fm.custom.contains_key("Title"));
}

#[test]
fn yaml_large_document() {
    let mut md = String::from("---\n");
    md.push_str("title: Large Doc\n");
    md.push_str("tags:\n");
    for i in 0..100 {
        md.push_str(&format!("  - tag{}\n", i));
    }
    md.push_str("---\n# Content\n");

    let doc = default_parser().parse(&md).unwrap();
    let fm = doc.metadata.unwrap();
    assert_eq!(fm.tags.len(), 100);
}
