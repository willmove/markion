//! Safe presentation data: no HTML execution, image fetches, or clickable invented sources.
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Paragraph,
    Heading,
    Code,
    Quote,
    List,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: Kind,
    pub text: String,
}
pub fn blocks(text: &str) -> Vec<Block> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut kind = Kind::Paragraph;
    let flush = |result: &mut Vec<Block>, current: &mut String, kind: &Kind| {
        if !current.is_empty() {
            result.push(Block {
                kind: kind.clone(),
                text: std::mem::take(current),
            });
        }
    };
    for event in Parser::new_ext(
        text,
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS,
    ) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph
                | Tag::Heading { .. }
                | Tag::CodeBlock(_)
                | Tag::BlockQuote(_)
                | Tag::Item => {
                    flush(&mut result, &mut current, &kind);
                    kind = match tag {
                        Tag::Heading { .. } => Kind::Heading,
                        Tag::CodeBlock(_) => Kind::Code,
                        Tag::BlockQuote(_) => Kind::Quote,
                        Tag::Item => Kind::List,
                        _ => Kind::Paragraph,
                    };
                    if kind == Kind::List {
                        current.push_str("• ");
                    }
                }
                _ => {}
            },
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::CodeBlock
                | TagEnd::Item
                | TagEnd::BlockQuote(_),
            ) => flush(&mut result, &mut current, &kind),
            Event::Text(t)
            | Event::Code(t)
            | Event::Html(t)
            | Event::InlineHtml(t)
            | Event::InlineMath(t)
            | Event::DisplayMath(t) => current.push_str(&t),
            Event::SoftBreak | Event::HardBreak => current.push('\n'),
            Event::TaskListMarker(done) => current.push_str(if done { "☑ " } else { "☐ " }),
            Event::Rule => {
                flush(&mut result, &mut current, &kind);
                result.push(Block {
                    kind: Kind::Paragraph,
                    text: "────────".into(),
                });
            }
            _ => {}
        }
    }
    flush(&mut result, &mut current, &kind);
    if result.is_empty() {
        result.push(Block {
            kind: Kind::Paragraph,
            text: String::new(),
        });
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn untrusted_markdown_has_no_active_content() {
        let rendered = blocks(
            "# Title\n\n<img src=\"https://example.test/tracker\">\n\n![x](https://example.test/pixel) [invented](missing.md)\n\n```rs\nlet x = 1;\n```",
        );
        assert_eq!(rendered[0].kind, Kind::Heading);
        assert!(rendered.iter().any(|b| b.kind == Kind::Code));
        assert!(!rendered.iter().any(|b| b.text.contains("missing.md")));
    }
}
