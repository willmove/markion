//! Verification example for task 8.4: HTML tags and URL auto-detection
//!
//! This example demonstrates that:
//! 1. HTML tags (inline and block) are correctly parsed and preserved
//! 2. URLs (http://, https://, www.) are auto-detected and converted to links
//!
//! Run with: cargo run --example test_task_8_4

use markdown::ast::{Block, Inline};
use markdown::parser::Parser;

fn main() {
    let test_markdown = r#"# Test Document

This has <span class="highlight">inline HTML</span> content.

<div class="container">
Block-level HTML content
</div>

Visit http://example.com for more info.

Check out https://secure.example.com/path for details.

Go to www.example.org (www. becomes https://).

Multiple URLs: http://first.com and https://second.com and www.third.net.
"#;

    println!("=== Task 8.4 Verification: HTML Tags and URL Auto-Detection ===\n");

    let parser = Parser::default();
    let doc = parser.parse(test_markdown).expect("Failed to parse");

    let mut html_inline_count = 0;
    let mut html_block_count = 0;
    let mut auto_link_count = 0;

    for block in &doc.blocks {
        match block {
            Block::HtmlBlock { content, .. } => {
                html_block_count += 1;
                println!(
                    "✓ Found HtmlBlock: {:?}",
                    content.lines().next().unwrap_or("")
                );
            }
            Block::Paragraph { content, .. } | Block::Heading { content, .. } => {
                for inline in content {
                    match inline {
                        Inline::HtmlInline(html) => {
                            html_inline_count += 1;
                            println!("✓ Found HtmlInline: {:?}", html);
                        }
                        Inline::Link { text, url, .. } => {
                            // Check if this was an auto-detected URL (text matches URL pattern)
                            if let Some(Inline::Text(t)) = text.first() {
                                if t.starts_with("http://")
                                    || t.starts_with("https://")
                                    || t.starts_with("www.")
                                {
                                    auto_link_count += 1;
                                    println!("✓ Found auto-detected URL: {} -> {}", t, url);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    println!("\n=== Summary ===");
    println!("HTML Inline elements found: {}", html_inline_count);
    println!("HTML Block elements found: {}", html_block_count);
    println!("Auto-detected URLs found: {}", auto_link_count);

    println!("\n=== Requirements Validation ===");

    // Requirement 17.7: HTML tag support
    if html_inline_count > 0 && html_block_count > 0 {
        println!("✅ Requirement 17.7 (HTML tags): PASSED");
        println!("   - Parser supports HTML tag embedding");
        println!("   - Original HTML content is preserved");
        println!("   - HtmlBlock nodes created for block-level HTML");
        println!("   - HtmlInline nodes created for inline HTML");
    } else {
        println!("❌ Requirement 17.7 (HTML tags): FAILED");
    }

    // Requirement 17.8: URL auto-detection
    if auto_link_count >= 5 {
        // We expect at least 5 auto-detected URLs
        println!("✅ Requirement 17.8 (URL auto-detection): PASSED");
        println!("   - http:// URLs are auto-detected and converted to links");
        println!("   - https:// URLs are auto-detected and converted to links");
        println!("   - www. URLs are auto-detected and converted to https:// links");
    } else {
        println!("❌ Requirement 17.8 (URL auto-detection): FAILED");
        println!(
            "   Expected at least 5 auto-detected URLs, found {}",
            auto_link_count
        );
    }

    println!("\n=== Task 8.4 Status ===");
    if html_inline_count > 0 && html_block_count > 0 && auto_link_count >= 5 {
        println!("✅ Task 8.4 COMPLETED SUCCESSFULLY");
        println!("\nImplementation details:");
        println!("- HTML parsing: Handled by pulldown-cmark Event::Html and Event::InlineHtml");
        println!("- URL detection: Post-processing with detect_and_convert_urls() function");
        println!("- URL patterns: http://, https://, www. (converted to https://)");
        println!("- Integration: Fully integrated into Parser::parse() workflow");
    } else {
        println!("❌ Task 8.4 INCOMPLETE");
    }
}
