//! Tests for HTML tag detection and URL auto-detection.
//!
//! **Validates: Requirements 17.7, 17.8**

use markdown::ast::{Block, Inline};
use markdown::parser::Parser;

fn default_parser() -> Parser {
    Parser::default()
}

// ---------------------------------------------------------------------------
// HTML Tag Tests (Requirement 17.7)
// ---------------------------------------------------------------------------

#[test]
fn parse_inline_html() {
    let md = "This has <span>inline HTML</span> content.\n";
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Check that we have HtmlInline nodes
        let has_html = content.iter().any(|inline| {
            matches!(inline, Inline::HtmlInline(html) if html.contains("<span>") || html.contains("</span>"))
        });
        assert!(has_html, "Expected HtmlInline nodes for inline HTML tags");
    } else {
        panic!("Expected Paragraph block");
    }
}

#[test]
fn parse_block_html() {
    let md = "<div>\nBlock HTML content\n</div>\n";
    let doc = default_parser().parse(md).unwrap();

    // Check that we have an HtmlBlock
    let has_html_block = doc.blocks.iter().any(
        |block| matches!(block, Block::HtmlBlock { content, .. } if content.contains("<div>")),
    );
    assert!(has_html_block, "Expected HtmlBlock for block-level HTML");
}

#[test]
fn preserve_html_content() {
    let md = "<div class=\"test\" id=\"main\">Content</div>\n";
    let doc = default_parser().parse(md).unwrap();

    if let Some(Block::HtmlBlock { content, .. }) = doc.blocks.first() {
        // Verify original HTML is preserved
        assert!(
            content.contains("class=\"test\""),
            "HTML attributes should be preserved"
        );
        assert!(
            content.contains("id=\"main\""),
            "HTML attributes should be preserved"
        );
    }
}

#[test]
fn html_mixed_with_markdown() {
    let md = "Regular text with <em>HTML emphasis</em> and **markdown bold**.\n";
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should have both HTML and Markdown inline elements
        let has_html = content.iter().any(|i| matches!(i, Inline::HtmlInline(_)));
        let has_strong = content.iter().any(|i| matches!(i, Inline::Strong(_)));
        assert!(has_html, "Should contain HTML inline elements");
        assert!(has_strong, "Should contain Markdown strong elements");
    }
}

// ---------------------------------------------------------------------------
// URL Auto-Detection Tests (Requirement 17.8)
// ---------------------------------------------------------------------------

#[test]
fn detect_http_url() {
    let md = "Visit http://example.com for more info.\n";
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Check that the URL was converted to a Link
        let has_link = content.iter().any(
            |inline| matches!(inline, Inline::Link { url, .. } if url == "http://example.com"),
        );
        assert!(
            has_link,
            "http:// URLs should be auto-detected and converted to links"
        );
    } else {
        panic!("Expected Paragraph block");
    }
}

#[test]
fn detect_https_url() {
    let md = "Check out https://secure.example.com/path?query=value for details.\n";
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        let has_link = content.iter().any(|inline| {
            matches!(inline, Inline::Link { url, .. }
                if url.starts_with("https://secure.example.com"))
        });
        assert!(
            has_link,
            "https:// URLs should be auto-detected and converted to links"
        );
    }
}

#[test]
fn detect_www_url() {
    let md = "Go to www.example.com for more.\n";
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // www. URLs should be converted to https://www.
        let has_link = content.iter().any(
            |inline| matches!(inline, Inline::Link { url, .. } if url == "https://www.example.com"),
        );
        assert!(
            has_link,
            "www. URLs should be auto-detected and converted to https://www. links"
        );
    }
}

#[test]
fn detect_multiple_urls() {
    let md = "Visit http://example.com and https://other.com or www.third.com.\n";
    let doc = default_parser().parse(md).unwrap();

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        let link_count = content
            .iter()
            .filter(|i| matches!(i, Inline::Link { .. }))
            .count();
        assert!(
            link_count >= 2,
            "Multiple URLs should be detected (found {})",
            link_count
        );
    }
}

#[test]
fn url_with_path_and_query() {
    let md = "API docs: https://api.example.com/v1/docs?format=json&lang=en\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        let has_full_url = content.iter().any(|inline| {
            matches!(inline, Inline::Link { url, .. }
                if url.contains("/v1/docs") && url.contains("format=json"))
        });
        assert!(
            has_full_url,
            "URLs with paths and query parameters should be fully captured"
        );
    }
}

#[test]
fn url_not_inside_explicit_link() {
    // Markdown already has a link - pulldown-cmark handles this
    let md = "[link text](https://example.com)\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Should have exactly one link
        let link_count = content
            .iter()
            .filter(|i| matches!(i, Inline::Link { .. }))
            .count();
        assert_eq!(
            link_count, 1,
            "Explicit markdown links should not be double-converted"
        );
    }
}

#[test]
fn url_followed_by_punctuation() {
    let md = "Check https://example.com. Also see http://other.com!\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        // Both URLs should be detected, punctuation should not be included
        let urls: Vec<_> = content
            .iter()
            .filter_map(|inline| {
                if let Inline::Link { url, .. } = inline {
                    Some(url.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            urls.contains(&"https://example.com"),
            "First URL should be detected"
        );
        assert!(
            urls.contains(&"http://other.com"),
            "Second URL should be detected"
        );

        // URLs should not contain trailing punctuation
        for url in &urls {
            assert!(
                !url.ends_with('.') && !url.ends_with('!'),
                "URLs should not include trailing punctuation: {}",
                url
            );
        }
    }
}

#[test]
fn url_in_parentheses() {
    let md = "See the docs (https://example.com/docs) for details.\n";
    let doc = default_parser().parse(md).unwrap();

    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        let has_link = content.iter().any(|inline| {
            matches!(inline, Inline::Link { url, .. } if url.starts_with("https://example.com"))
        });
        assert!(has_link, "URLs in parentheses should be detected");
    }
}

// ---------------------------------------------------------------------------
// Round-trip Tests
// ---------------------------------------------------------------------------

#[test]
fn html_block_round_trip() {
    let original = "<div class=\"container\">\nContent\n</div>\n\nSome text.\n";
    let doc = default_parser().parse(original).unwrap();
    let rendered = markdown::renderer::render_to_markdown(&doc);

    // Re-parse the rendered output
    let doc2 = default_parser().parse(&rendered).unwrap();

    // Should have the same number of blocks
    assert_eq!(doc.blocks.len(), doc2.blocks.len());
}

#[test]
fn url_detection_round_trip() {
    let original = "Visit http://example.com for info.\n";
    let doc = default_parser().parse(original).unwrap();
    let rendered = markdown::renderer::render_to_markdown(&doc);

    // The rendered output should contain a proper markdown link
    assert!(
        rendered.contains("[") && rendered.contains("](http://example.com)"),
        "Auto-detected URLs should render as proper markdown links"
    );
}
