//! Debug test for URL followed by punctuation

use markdown::parser::Parser;

#[test]
fn debug_url_with_punctuation() {
    let md = "Check https://example.com. Also see http://other.com!\n";
    let parser = Parser::default();
    let doc = parser.parse(md).unwrap();

    println!("Parsed document: {:#?}", doc);
}
