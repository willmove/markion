use markdown::ast::{Block, Document, Inline, ListItem};
use markdown::parser::Parser;
use markdown::renderer::render_to_markdown;

fn main() {
    let parser = Parser::default();

    // Test: Ordered list with item "0."
    println!("Original document with ordered list:");
    let doc1 = Document::new(vec![Block::List {
        items: vec![ListItem {
            content: vec![Inline::Text("0.".into())],
            blocks: vec![],
            checked: None,
            sub_items: vec![],
        }],
        ordered: true,
        start: None,
        id: 0,
    }]);

    println!("  blocks: {}", doc1.blocks.len());

    let rendered = render_to_markdown(&doc1);
    println!("\nRendered markdown:");
    println!("---");
    println!("{}", rendered);
    println!("---");

    println!("\nParsing rendered markdown:");
    let doc2 = parser.parse(&rendered).unwrap();
    println!("  blocks: {}", doc2.blocks.len());

    for (i, block) in doc2.blocks.iter().enumerate() {
        println!("  block {}: {:?}", i, block);
    }
}
