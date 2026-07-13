//! Debug test to see what events pulldown-cmark emits

use pulldown_cmark::Parser as PdParser;

#[test]
fn debug_pulldown_html_events() {
    let md = "This has <span>inline HTML</span> content.\n";
    let parser = PdParser::new(md);

    println!("Events for inline HTML:");
    for (i, event) in parser.enumerate() {
        println!("{}: {:?}", i, event);
    }
}

#[test]
fn debug_pulldown_url_events() {
    let md = "Visit http://example.com for info.\n";
    let parser = PdParser::new(md);

    println!("Events for bare URL:");
    for (i, event) in parser.enumerate() {
        println!("{}: {:?}", i, event);
    }
}

#[test]
fn debug_pulldown_block_html_events() {
    let md = "<div>\nBlock HTML\n</div>\n";
    let parser = PdParser::new(md);

    println!("Events for block HTML:");
    for (i, event) in parser.enumerate() {
        println!("{}: {:?}", i, event);
    }
}
