//! Source-ranged model used by the Visual Edit surface.

use std::ops::Range;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::frontmatter::split_front_matter;
use crate::model::{
    InlineStyle, PreviewBlock, VisualBlock, VisualBlockKind, VisualInlineRun,
    VisualSourceIslandKind,
};
use crate::parse::markdown_options;

pub(crate) fn build_visual_blocks(text: &str, preview: &[PreviewBlock]) -> Vec<VisualBlock> {
    let mut blocks = Vec::with_capacity(preview.len() + 1);
    if let Some((_, body_start)) = split_front_matter(text)
        && body_start > 0
    {
        blocks.push(source_island(
            0..body_start,
            VisualSourceIslandKind::FrontMatter,
        ));
    }

    let mut covered_until = blocks.last().map_or(0, |block| block.source_range.end);
    for preview_block in preview {
        let range = preview_block.source_range().clone();
        if range.start > covered_until && !text[covered_until..range.start].trim().is_empty() {
            blocks.push(source_island(
                covered_until..range.start,
                VisualSourceIslandKind::Unsupported,
            ));
        }

        let mut block = visual_block_from_preview(text, preview_block);
        if block.source_range.start < covered_until {
            block.source_island = Some(VisualSourceIslandKind::Unsupported);
        }
        covered_until = covered_until.max(range.end);
        blocks.push(block);
    }

    if covered_until < text.len() && !text[covered_until..].trim().is_empty() {
        blocks.push(source_island(
            covered_until..text.len(),
            VisualSourceIslandKind::Unsupported,
        ));
    }
    blocks
}

fn source_island(range: Range<usize>, kind: VisualSourceIslandKind) -> VisualBlock {
    VisualBlock {
        kind: VisualBlockKind::Unsupported,
        source_range: range,
        editable_runs: Vec::new(),
        marker_ranges: Vec::new(),
        source_island: Some(kind),
    }
}

fn visual_block_from_preview(text: &str, block: &PreviewBlock) -> VisualBlock {
    let source_range = block.source_range().clone();
    let (kind, source_island) = match block {
        PreviewBlock::Heading { level, .. } => (VisualBlockKind::Heading { level: *level }, None),
        PreviewBlock::Paragraph { .. } => (VisualBlockKind::Paragraph, None),
        PreviewBlock::ListItem {
            level,
            ordered,
            index,
            checked,
            ..
        } => (
            VisualBlockKind::ListItem {
                level: *level,
                ordered: *ordered,
                index: *index,
                checked: *checked,
            },
            None,
        ),
        PreviewBlock::BlockQuote { .. } => (VisualBlockKind::BlockQuote, None),
        PreviewBlock::CodeBlock { language, .. } => (
            VisualBlockKind::CodeBlock {
                language: language.clone(),
            },
            Some(VisualSourceIslandKind::Code),
        ),
        PreviewBlock::MathBlock { .. } => (
            VisualBlockKind::MathBlock,
            Some(VisualSourceIslandKind::Math),
        ),
        PreviewBlock::Html { .. } => (
            VisualBlockKind::Unsupported,
            Some(VisualSourceIslandKind::Html),
        ),
        PreviewBlock::Image {
            alt, url, title, ..
        } => (
            VisualBlockKind::Image {
                alt: alt.clone(),
                url: url.clone(),
                title: title.clone(),
            },
            Some(VisualSourceIslandKind::Image),
        ),
        PreviewBlock::Rule { .. } => (VisualBlockKind::Rule, None),
        PreviewBlock::Table {
            rows, alignments, ..
        } => (
            VisualBlockKind::Table {
                rows: rows.clone(),
                alignments: alignments.clone(),
            },
            Some(VisualSourceIslandKind::Table),
        ),
    };

    let (editable_runs, contains_html) = inline_runs(text, source_range.clone());
    let marker_ranges = marker_ranges(source_range.clone(), &editable_runs);
    VisualBlock {
        kind,
        source_range,
        editable_runs,
        marker_ranges,
        source_island: source_island.or(contains_html.then_some(VisualSourceIslandKind::Html)),
    }
}

fn inline_runs(text: &str, block_range: Range<usize>) -> (Vec<VisualInlineRun>, bool) {
    let source = &text[block_range.clone()];
    let mut runs = Vec::new();
    let mut style = InlineStyle::default();
    let mut link_targets: Vec<Option<Range<usize>>> = Vec::new();
    let mut inline_depth = 0usize;
    let mut contains_html = false;

    for (event, relative_range) in Parser::new_ext(source, markdown_options()).into_offset_iter() {
        let event_range =
            block_range.start + relative_range.start..block_range.start + relative_range.end;
        match event {
            Event::Start(Tag::Strong) => {
                style.bold = true;
                inline_depth += 1;
            }
            Event::End(TagEnd::Strong) => {
                style.bold = false;
                inline_depth = inline_depth.saturating_sub(1);
            }
            Event::Start(Tag::Emphasis) => {
                style.italic = true;
                inline_depth += 1;
            }
            Event::End(TagEnd::Emphasis) => {
                style.italic = false;
                inline_depth = inline_depth.saturating_sub(1);
            }
            Event::Start(Tag::Strikethrough) => {
                style.strikethrough = true;
                inline_depth += 1;
            }
            Event::End(TagEnd::Strikethrough) => {
                style.strikethrough = false;
                inline_depth = inline_depth.saturating_sub(1);
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                inline_depth += 1;
                link_targets.push(find_link_target(text, &event_range, &dest_url));
            }
            Event::End(TagEnd::Link) => {
                link_targets.pop();
                inline_depth = inline_depth.saturating_sub(1);
            }
            Event::Text(visible) => push_run(
                &mut runs,
                text,
                visible.as_ref(),
                event_range,
                style,
                link_targets.last().cloned().flatten(),
                inline_depth > 1,
            ),
            Event::Code(visible) => {
                let mut code_style = style;
                code_style.code = true;
                push_run(
                    &mut runs,
                    text,
                    visible.as_ref(),
                    event_range,
                    code_style,
                    link_targets.last().cloned().flatten(),
                    inline_depth > 1,
                );
            }
            Event::SoftBreak | Event::HardBreak => push_run(
                &mut runs,
                text,
                "\n",
                event_range,
                style,
                link_targets.last().cloned().flatten(),
                inline_depth > 1,
            ),
            Event::FootnoteReference(visible) => {
                let mut footnote_style = style;
                footnote_style.superscript = true;
                push_run(
                    &mut runs,
                    text,
                    visible.as_ref(),
                    event_range,
                    footnote_style,
                    link_targets.last().cloned().flatten(),
                    inline_depth > 1,
                );
            }
            Event::InlineMath(visible) | Event::DisplayMath(visible) => push_run(
                &mut runs,
                text,
                visible.as_ref(),
                event_range,
                style,
                link_targets.last().cloned().flatten(),
                inline_depth > 1,
            ),
            Event::Html(_) | Event::InlineHtml(_) => contains_html = true,
            _ => {}
        }
    }
    (runs, contains_html)
}

fn push_run(
    runs: &mut Vec<VisualInlineRun>,
    source: &str,
    visible: &str,
    event_range: Range<usize>,
    style: InlineStyle,
    link_target_range: Option<Range<usize>>,
    force_fallback: bool,
) {
    if visible.is_empty() {
        return;
    }
    let event_source = &source[event_range.clone()];
    let exact = event_source
        .find(visible)
        .map(|relative| event_range.start + relative..event_range.start + relative + visible.len());
    let conservative_fallback = force_fallback || exact.is_none();
    let content_range = exact.unwrap_or_else(|| event_range.clone());
    runs.push(VisualInlineRun {
        visible_text: visible.to_string(),
        source_range: event_range,
        content_range,
        style,
        link_target_range,
        conservative_fallback,
    });
}

fn find_link_target(
    source: &str,
    event_range: &Range<usize>,
    target: &str,
) -> Option<Range<usize>> {
    source[event_range.clone()]
        .rfind(target)
        .map(|relative| event_range.start + relative..event_range.start + relative + target.len())
}

fn marker_ranges(block_range: Range<usize>, runs: &[VisualInlineRun]) -> Vec<Range<usize>> {
    let mut content = runs
        .iter()
        .filter(|run| !run.conservative_fallback)
        .map(|run| run.content_range.clone())
        .collect::<Vec<_>>();
    content.sort_by_key(|range| range.start);
    let mut markers = Vec::new();
    let mut cursor = block_range.start;
    for range in content {
        if range.start > cursor {
            markers.push(cursor..range.start);
        }
        cursor = cursor.max(range.end);
    }
    if cursor < block_range.end {
        markers.push(cursor..block_range.end);
    }
    markers
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        MarkdownDocument, MarkdownFormat, TableEdit, VisualBlockKind, VisualSourceIslandKind,
    };

    #[test]
    fn maps_common_inline_runs_to_exact_source_content() {
        let source = "# Hello **bold** and [site](https://example.com)\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].kind,
            VisualBlockKind::Heading { level: 1 }
        ));
        assert_eq!(&source[blocks[0].source_range.clone()], source);

        let bold = blocks[0]
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "bold")
            .unwrap();
        assert_eq!(&source[bold.content_range.clone()], "bold");
        assert!(bold.style.bold);

        let link = blocks[0]
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "site")
            .unwrap();
        assert_eq!(&source[link.content_range.clone()], "site");
        assert_eq!(
            &source[link.link_target_range.clone().unwrap()],
            "https://example.com"
        );
        assert!(
            blocks[0]
                .marker_ranges
                .iter()
                .any(|range| &source[range.clone()] == "**")
        );
    }

    #[test]
    fn uses_conservative_fallback_when_visible_text_is_not_byte_exact() {
        let doc = MarkdownDocument::from_text("***nested***");
        let blocks = doc.visual_blocks();
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.conservative_fallback)
        );
    }

    #[test]
    fn classifies_complex_constructs_as_source_islands() {
        let doc = MarkdownDocument::from_text(
            "---\ntitle: Demo\n---\n\n```rust\nfn main() {}\n```\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n",
        );
        let blocks = doc.visual_blocks();
        assert_eq!(
            blocks[0].source_island,
            Some(VisualSourceIslandKind::FrontMatter)
        );
        assert!(
            blocks
                .iter()
                .any(|block| block.source_island == Some(VisualSourceIslandKind::Code))
        );
        assert!(
            blocks
                .iter()
                .any(|block| block.source_island == Some(VisualSourceIslandKind::Table))
        );
    }

    #[test]
    fn visual_cache_reuses_version_and_invalidates_on_text_change() {
        let mut doc = MarkdownDocument::from_text("alpha");
        let first = doc.visual_blocks_shared();
        let second = doc.visual_blocks_shared();
        assert!(Arc::ptr_eq(&first, &second));
        let version = doc.version();

        doc.replace_range(0..5, "beta");
        let third = doc.visual_blocks_shared();
        assert!(!Arc::ptr_eq(&first, &third));
        assert_ne!(version, doc.version());
        assert_eq!(third[0].editable_runs[0].visible_text, "beta");
    }

    #[test]
    fn visual_table_uses_existing_source_edit_path() {
        let mut doc = MarkdownDocument::from_text("| A | B |\n| --- | --- |\n| 1 | 2 |\n");
        let table = doc
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
            .unwrap();
        assert!(
            table
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback),
            "plain table cells should remain renderable as a visual grid"
        );
        let original = doc.text().to_string();
        let result = doc
            .edit_table_at(table.source_range.start, TableEdit::AddRow)
            .unwrap();
        assert_ne!(doc.text(), original);
        assert!(result.selected_range.start >= result.table_range.start);
        assert!(matches!(
            doc.visual_blocks()[0].kind,
            VisualBlockKind::Table { .. }
        ));
    }

    #[test]
    fn visual_run_formatting_mutates_markdown_source() {
        let mut doc = MarkdownDocument::from_text("hello world");
        let run = doc.visual_blocks()[0].editable_runs[0].clone();
        let result = doc.apply_markdown_format(run.content_range, MarkdownFormat::Bold);
        assert_eq!(doc.text(), "**hello world**");
        assert!(doc.is_dirty());
        assert_eq!(result, 2..13);
    }

    #[test]
    fn welcome_prose_stays_visual_outside_the_focused_block() {
        let doc = MarkdownDocument::from_text(crate::DEFAULT_WELCOME_MARKDOWN);
        let blocks = doc.visual_blocks();
        assert!(crate::DEFAULT_WELCOME_MARKDOWN.starts_with("# Welcome to Markion\n"));
        for marker in [
            "**bold**",
            "![Local image placeholder]",
            "- [ ] Export when ready",
            "| Syntax | Example | Purpose |",
            "```rust",
            "$E = mc^2$",
            "[^links]:",
            "==highlighted text==",
            "H~2~O",
            "x^2^",
        ] {
            assert!(
                crate::DEFAULT_WELCOME_MARKDOWN.contains(marker),
                "welcome document is missing {marker:?}"
            );
        }

        let editable_blocks = blocks
            .iter()
            .filter(|block| block.source_island.is_none())
            .collect::<Vec<_>>();
        assert!(
            editable_blocks.len() >= 10,
            "expected substantial directly editable prose: {blocks:?}"
        );
        assert!(editable_blocks.iter().any(|block| {
            block
                .editable_runs
                .iter()
                .any(|run| run.visible_text.contains("starter document"))
        }));
        assert!(
            blocks.iter().any(|block| {
                matches!(block.source_island, Some(VisualSourceIslandKind::Image))
            })
        );
        assert!(
            blocks
                .iter()
                .any(|block| { matches!(block.source_island, Some(VisualSourceIslandKind::Code)) })
        );
        assert!(
            blocks
                .iter()
                .any(|block| { matches!(block.source_island, Some(VisualSourceIslandKind::Math)) })
        );
        assert!(
            blocks.iter().any(|block| {
                matches!(block.source_island, Some(VisualSourceIslandKind::Table))
            })
        );

        assert!(editable_blocks.iter().any(|block| {
            matches!(block.kind, VisualBlockKind::ListItem { .. })
                && !block.editable_runs.is_empty()
                && block
                    .editable_runs
                    .iter()
                    .all(|run| !run.conservative_fallback)
        }));
    }
}
