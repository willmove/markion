use crate::parse::{
    HtmlInlineState, HtmlPreviewPart, html_preview_parts, parse_html_inline_element,
};
use crate::{InlineStyle, MarkdownDocument, PreviewBlock, RichText};

fn rich_blocks(blocks: &[PreviewBlock]) -> Vec<&RichText> {
    let mut out = Vec::new();
    for block in blocks {
        match block {
            PreviewBlock::Paragraph { text, .. }
            | PreviewBlock::Heading { text, .. }
            | PreviewBlock::ListItem { text, .. } => out.push(text),
            PreviewBlock::Table { rows, .. } => out.extend(rows.iter().flatten()),
            PreviewBlock::BlockQuote { children, .. } => out.extend(rich_blocks(children)),
            _ => {}
        }
    }
    out
}

#[test]
fn inline_html_shared_styles_links_and_contexts() {
    let body = r#"前 <span style="color:#c00"><strong title="a > b">粗体</strong> <a href="https://example.com/?a=1&amp;b=2"><u>链接</u></a></span> <kbd>Ctrl</kbd> <samp>输出</samp> <span class="plain">普通</span> 后"#;
    for source in [
        body.into(),
        format!("# {body}"),
        format!("- {body}"),
        format!("> {body}"),
        format!("| A |\n|---|\n| {body} |"),
    ] {
        let doc = MarkdownDocument::from_text(&source);
        let preview = doc.preview_blocks_shared();
        let visual = doc.visual_blocks_shared();
        let texts = rich_blocks(&preview);
        for (needle, property) in [
            ("粗体", 0),
            ("链接", 1),
            ("Ctrl", 2),
            ("输出", 2),
            ("普通", 3),
            ("后", 3),
        ] {
            let read = texts
                .iter()
                .flat_map(|text| &text.spans)
                .find(|span| span.text.contains(needle))
                .unwrap();
            let edit = visual
                .iter()
                .flat_map(|block| &block.editable_runs)
                .find(|run| run.visible_text.contains(needle))
                .unwrap();
            assert_eq!(read.style, edit.style, "{source}: {needle}");
            assert!(!edit.conservative_fallback, "{source}: {needle}");
            match property {
                0 => {
                    assert!(read.style.bold);
                    assert_eq!(read.style.color, Some(0xcc0000));
                }
                1 => {
                    assert!(read.style.underline);
                    assert_eq!(read.link.as_deref(), Some("https://example.com/?a=1&b=2"));
                }
                2 => assert!(read.style.code),
                _ => assert_eq!(read.style, InlineStyle::default()),
            }
        }
    }
}

#[test]
fn inline_html_shared_blocks_cells_and_state_reset() {
    let body = r#"<span style="color:rgb(12,34,56)"><u>A</u><mark>B</mark><sup>C</sup><sub>D</sub></span> E"#;
    let parts = html_preview_parts(&format!("<p>{body}</p>"));
    let HtmlPreviewPart::Text {
        text: paragraph, ..
    } = &parts[0]
    else {
        panic!()
    };
    let parts = html_preview_parts(&format!(
        "<table><tr><td>{body}</td><td><b><a href='x'>bad</td><td>clean</td></tr></table>"
    ));
    let HtmlPreviewPart::Table { grid } = &parts[0] else {
        panic!()
    };
    assert_eq!(paragraph, &grid.rows[0][0].content);
    let clean = &grid.rows[0][2].content;
    assert_eq!(clean.text, "clean");
    assert!(
        clean
            .spans
            .iter()
            .all(|span| span.style.is_plain() && span.link.is_none())
    );
    for (letter, check) in [("A", 0), ("B", 1), ("C", 2), ("D", 3)] {
        let span = paragraph.spans.iter().find(|s| s.text == letter).unwrap();
        assert_eq!(span.style.color, Some(0x0c2238));
        assert!(match check {
            0 => span.style.underline,
            1 => span.style.highlight,
            2 => span.style.superscript,
            _ => span.style.subscript,
        });
    }
    assert!(paragraph.spans.last().unwrap().style.is_plain());
}

#[test]
fn inline_html_explicit_breaks_survive_every_parser() {
    for body in ["<br>A<br><br>B<br>", "<br><br>"] {
        let doc = MarkdownDocument::from_text(body);
        let expected = if body.contains('A') {
            "\nA\n\nB\n"
        } else {
            "\n\n"
        };
        let blocks = doc.preview_blocks_shared();
        assert_eq!(rich_blocks(&blocks)[0].text, expected);
        let visual = doc.visual_blocks_shared();
        let projection = crate::build_visual_projection(
            body,
            &visual[0],
            body.len() + 1..body.len() + 1,
            body.len() + 1,
        );
        assert_eq!(projection.text, expected);
        for html in [
            format!("<p>{body}</p>"),
            format!("<table><tr><td>{body}</td></tr></table>"),
        ] {
            let parts = html_preview_parts(&html);
            let text = match &parts[0] {
                HtmlPreviewPart::Text { text, .. } => text,
                HtmlPreviewPart::Table { grid } => &grid.rows[0][0].content,
                _ => panic!(),
            };
            assert_eq!(text.text, expected, "{html}");
        }
    }
}

#[test]
fn inline_html_source_reveal_and_linked_images_are_exact() {
    let source = r#"前 <a href="https://example.com/?a=1&amp;b=2" title="a > b"><span style="color:#123">中文</span><img src="x.png" alt="图"></a> 后"#;
    let doc = MarkdownDocument::from_text(source);
    let version = doc.version();
    let visual = doc.visual_blocks_shared();
    let block = &visual[0];
    assert!(
        block
            .editable_runs
            .iter()
            .all(|run| !run.conservative_fallback)
    );
    let run = block
        .editable_runs
        .iter()
        .find(|run| run.html_image.is_some())
        .unwrap();
    assert!(run.navigation.is_some());
    let cursor = source.find("中文").unwrap();
    let shown = crate::build_visual_projection(source, block, cursor..cursor, cursor);
    assert_eq!(shown.text, source);
    assert_eq!(shown.revealed_source_ranges.len(), 1);
    for offset in shown.revealed_source_ranges[0]
        .clone()
        .filter(|i| source.is_char_boundary(*i))
    {
        assert_eq!(
            shown.source_for_display(shown.display_for_source(offset).unwrap()),
            offset
        );
    }
    assert_eq!(doc.version(), version);
    assert!(std::sync::Arc::ptr_eq(&visual, &doc.visual_blocks_shared()));
    let preview = doc.preview_blocks_shared();
    let image = rich_blocks(&preview)
        .iter()
        .flat_map(|r| &r.spans)
        .find(|s| s.image.is_some())
        .unwrap();
    assert_eq!(image.link.as_deref(), Some("https://example.com/?a=1&b=2"));
    let parts = html_preview_parts("<a href='https://example.com'><img src='x.png'></a>");
    assert!(
        matches!(&parts[0], HtmlPreviewPart::Image {link:Some(link),..} if link=="https://example.com")
    );
}

#[test]
fn inline_html_nested_colors_and_literal_forms() {
    let mut state = HtmlInlineState::default();
    for tag in [
        "<span style='color:#123'>",
        "<font color='#abc'>",
        "<span title='neutral'>",
    ] {
        assert!(state.handle(tag));
    }
    assert_eq!(state.style(InlineStyle::default()).color, Some(0xaabbcc));
    state.handle("</span>");
    state.handle("</font>");
    assert_eq!(state.style(InlineStyle::default()).color, Some(0x112233));
    state.handle("</span>");
    assert!(state.style(InlineStyle::default()).is_plain());
    for source in [
        "<span style='color:#你'>",
        "<span onclick='x'>",
        "<span style='position:fixed'>",
    ] {
        assert!(parse_html_inline_element(source).is_none());
    }
    for source in [
        r"前 \<span>字\</span> 后",
        "前 &lt;span&gt;字&lt;/span&gt; 后",
        "前 `<span>字</span>` 后",
    ] {
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.preview_blocks_shared();
        assert!(rich_blocks(&blocks)[0].text.contains("<span>字</span>"));
    }
}
