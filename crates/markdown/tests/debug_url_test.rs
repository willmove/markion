//! Debug test to understand how pulldown-cmark handles URLs

use markdown::parser::Parser;

#[test]
fn debug_url_parsing() {
    let md = "Visit http://example.com for more info.\n";
    let parser = Parser::default();
    let doc = parser.parse(md).unwrap();

    println!("Parsed document: {:#?}", doc);
}

#[test]
fn debug_html_parsing() {
    let md = "This has <span>inline HTML</span> content.\n";
    let parser = Parser::default();
    let doc = parser.parse(md).unwrap();

    println!("Parsed document: {:#?}", doc);
}
