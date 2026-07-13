use pulldown_cmark::{Options, Parser};

fn main() {
    let md = "1. 0.\n";
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_GFM;

    println!("Parsing: {:?}", md);
    let parser = Parser::new_ext(md, opts);

    for (i, event) in parser.enumerate() {
        println!("{}: {:?}", i, event);
    }
}
