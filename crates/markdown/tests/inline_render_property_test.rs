//! Property-based test for inline Markdown syntax rendering (Task 11.3).
//!
//! Implements Property 2 from the design document:
//! - Property 2: 行内语法渲染隐藏标记
//!
//! *For any* 行内 Markdown 语法（粗体 `**text**`、斜体 `*text*`、删除线
//! `~~text~~`、代码 `` `code` ``），渲染后的输出应该包含格式化效果但不包含原始
//! 语法标记字符。
//!
//! **Validates: Requirements 1.1**
//!
//! ## What "rendered output" means here
//!
//! In the live-preview editing model (Requirement 1.1), rendering an inline
//! element means showing its *formatted* text while *hiding* the raw Markdown
//! syntax markers (`**`, `*`, `~~`, `` ` ``). In this codebase that behavior is
//! realized by the parser: it turns the source text into structural inline AST
//! nodes (`Strong`, `Emphasis`, `Strikethrough`, `Code`) whose payload is the
//! plain visible content, with the marker characters consumed by the parse.
//!
//! This test therefore verifies rendering by:
//!   1. parsing the inline syntax into the AST (the "formatting effect" is the
//!      presence of the corresponding structural node), and
//!   2. extracting the visible text that the renderer would display (markers
//!      hidden) and asserting it preserves the content but contains none of the
//!      raw marker characters.

use markdown::ast::{Block, Document, Inline};
use markdown::parser::Parser;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_parser() -> Parser {
    Parser::default()
}

/// The four inline syntaxes covered by Property 2.
#[derive(Debug, Clone, Copy)]
enum InlineKind {
    Strong,
    Emphasis,
    Strikethrough,
    Code,
}

impl InlineKind {
    /// The raw marker character(s) that must NOT appear in the rendered output.
    fn markers(self) -> &'static [char] {
        match self {
            InlineKind::Strong => &['*'],
            InlineKind::Emphasis => &['*'],
            InlineKind::Strikethrough => &['~'],
            InlineKind::Code => &['`'],
        }
    }

    /// Wrap `content` in this kind's Markdown syntax.
    fn wrap(self, content: &str) -> String {
        match self {
            InlineKind::Strong => format!("**{}**", content),
            InlineKind::Emphasis => format!("*{}*", content),
            InlineKind::Strikethrough => format!("~~{}~~", content),
            InlineKind::Code => format!("`{}`", content),
        }
    }

    /// Whether an inline node is the structural node produced by this syntax
    /// (i.e. the "formatting effect" is present in the AST).
    fn matches_node(self, inline: &Inline) -> bool {
        matches!(
            (self, inline),
            (InlineKind::Strong, Inline::Strong(_))
                | (InlineKind::Emphasis, Inline::Emphasis(_))
                | (InlineKind::Strikethrough, Inline::Strikethrough(_))
                | (InlineKind::Code, Inline::Code(_))
        )
    }
}

/// Return the inline content of the first paragraph in the document, if any.
fn first_paragraph_content(doc: &Document) -> Option<&Vec<Inline>> {
    doc.blocks.iter().find_map(|b| match b {
        Block::Paragraph { content, .. } => Some(content),
        _ => None,
    })
}

/// Extract the *visible* text the renderer would display for a slice of inline
/// nodes, with all syntax markers hidden. This descends into container inlines
/// (strong / emphasis / strikethrough) and includes inline-code payloads.
fn render_visible_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) => out.push_str(s),
            Inline::Code(s) => out.push_str(s),
            Inline::Strong(inner)
            | Inline::Emphasis(inner)
            | Inline::Strikethrough(inner)
            | Inline::Superscript(inner)
            | Inline::Subscript(inner)
            | Inline::Highlight(inner) => out.push_str(&render_visible_text(inner)),
            Inline::Link { text, .. } => out.push_str(&render_visible_text(text)),
            Inline::Emoji { unicode, .. } => out.push_str(unicode),
            _ => {}
        }
    }
    out
}

/// Recursively test whether any inline node in the tree satisfies `pred`.
fn any_inline<F: Fn(&Inline) -> bool + Copy>(inlines: &[Inline], pred: F) -> bool {
    inlines.iter().any(|inline| {
        if pred(inline) {
            return true;
        }
        match inline {
            Inline::Strong(inner)
            | Inline::Emphasis(inner)
            | Inline::Strikethrough(inner)
            | Inline::Superscript(inner)
            | Inline::Subscript(inner)
            | Inline::Highlight(inner) => any_inline(inner, pred),
            Inline::Link { text, .. } => any_inline(text, pred),
            _ => false,
        }
    })
}

/// Strategy producing one of the four inline kinds.
fn inline_kind_strategy() -> impl Strategy<Value = InlineKind> {
    prop_oneof![
        Just(InlineKind::Strong),
        Just(InlineKind::Emphasis),
        Just(InlineKind::Strikethrough),
        Just(InlineKind::Code),
    ]
}

// ---------------------------------------------------------------------------
// Property 2: 行内语法渲染隐藏标记
// Validates: Requirements 1.1
// ---------------------------------------------------------------------------

proptest! {
    /// For any of the four inline Markdown syntaxes wrapping alphanumeric
    /// content, parsing (the model of live-preview rendering) must:
    ///   1. produce the corresponding structural inline node (the formatting
    ///      effect is present), and
    ///   2. yield a visible rendered text that contains the original content
    ///      but none of the raw syntax marker characters (`*`, `~`, `` ` ``).
    #[test]
    fn prop2_inline_syntax_hides_markers(
        kind in inline_kind_strategy(),
        // Content excludes marker characters so the assertions target the
        // syntax markers themselves rather than incidental content characters.
        content in "[a-zA-Z0-9]{1,15}",
    ) {
        // Surround the syntax with plain text so it is parsed as inline content
        // inside a paragraph (and not, e.g., a setext heading edge case).
        let md = format!("lead {} tail\n", kind.wrap(&content));
        let doc = default_parser().parse(&md).unwrap();

        let para = first_paragraph_content(&doc)
            .expect("expected a paragraph");

        // (1) Formatting effect present: the expected structural node exists.
        prop_assert!(
            any_inline(para, |i| kind.matches_node(i)),
            "expected {:?} node for markdown: {:?}\nparsed inlines: {:?}",
            kind, md, para
        );

        // (2a) Visible text preserves the formatted content.
        let visible = render_visible_text(para);
        prop_assert!(
            visible.contains(&content),
            "rendered output {:?} should contain content {:?} for markdown: {:?}",
            visible, content, md
        );

        // (2b) Visible text hides the raw syntax marker characters.
        for marker in kind.markers() {
            prop_assert!(
                !visible.contains(*marker),
                "rendered output {:?} should not contain syntax marker {:?} for markdown: {:?}",
                visible, marker, md
            );
        }
    }
}
