use markdown::ast::Block;
use markdown::parser::Parser;

fn main() {
    let parser = Parser::default();

    // Test 1: Bold text
    println!("Test 1 - Bold text:");
    let md1 = "- [x] Complete **important** task\n";
    let doc1 = parser.parse(md1).unwrap();
    if let Block::List { items, .. } = &doc1.blocks[0] {
        println!("  checked: {:?}", items[0].checked);
        println!("  content: {:?}", items[0].content);
        println!("  blocks: {:?}", items[0].blocks);
    }

    // Test 2: Link
    println!("\nTest 2 - Link:");
    let md2 = "- [x] Read [documentation](https://example.com)\n";
    let doc2 = parser.parse(md2).unwrap();
    if let Block::List { items, .. } = &doc2.blocks[0] {
        println!("  checked: {:?}", items[0].checked);
        println!("  content: {:?}", items[0].content);
        println!("  blocks: {:?}", items[0].blocks);
    }
}
