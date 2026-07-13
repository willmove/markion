use markdown::ast::{Block, Document};
use markdown::parser::Parser;
use markdown::renderer::render_to_markdown;

fn main() {
    let parser = Parser::default();

    // Test: Two horizontal rules
    println!("Original document with 2 HRs:");
    let doc1 = Document::new(vec![
        Block::HorizontalRule { id: 0 },
        Block::HorizontalRule { id: 1 },
    ]);

    println!("  blocks: {}", doc1.blocks.len());

    let rendered = render_to_markdown(&doc1);
    println!("\nRendered markdown (len={}):", rendered.len());
    for (i, ch) in rendered.chars().enumerate() {
        if ch == '\n' {
            println!("  [{}]: \\n", i);
        } else {
            println!("  [{}]: '{}'", i, ch);
        }
    }

    println!("\nParsing rendered markdown:");
    let doc2 = parser.parse(&rendered).unwrap();
    println!("  blocks: {}", doc2.blocks.len());

    for (i, block) in doc2.blocks.iter().enumerate() {
        println!("  block {}: {:?}", i, block);
    }
}
