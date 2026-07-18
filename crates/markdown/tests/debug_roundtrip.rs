//! Debug test to understand the round-trip issue

use markdown::ast::{Block, Document, Inline};
use markdown::parser::{Parser, ParserOptions};
use markdown::renderer::render_to_markdown;

#[test]
fn debug_inline_math_in_link() {
    // Create a document with inline math inside a link
    let doc = Document {
        blocks: vec![Block::BlockQuote {
            content: vec![Block::BlockQuote {
                content: vec![Block::Paragraph {
                    content: vec![Inline::Link {
                        text: vec![Inline::InlineMath("*-*a*0*".to_string())],
                        url: "http://a".to_string(),
                        title: None,
                    }],
                    id: 0,
                }],
                id: 0,
            }],
            id: 0,
        }],
        metadata: None,
        version: 0,
        footnote_map: std::collections::HashMap::new(),
    };

    let parser = Parser::new(ParserOptions::default());

    // Step 1: Render the original document to Markdown
    let markdown1 = render_to_markdown(&doc);
    println!("=== Markdown 1 (rendered) ===");
    println!("{}", markdown1);
    println!();

    // Step 2: Parse the rendered Markdown
    let doc2 = parser
        .parse(&markdown1)
        .expect("First parse should succeed");
    println!("=== Document 2 (parsed) ===");
    println!("{:#?}", doc2);
    println!();

    // Step 3: Render again
    let markdown2 = render_to_markdown(&doc2);
    println!("=== Markdown 2 (re-rendered) ===");
    println!("{}", markdown2);
    println!();

    // Step 4: Parse the second rendering
    let doc3 = parser
        .parse(&markdown2)
        .expect("Second parse should succeed");
    println!("=== Document 3 (re-parsed) ===");
    println!("{:#?}", doc3);
}
