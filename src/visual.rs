//! Source-ranged model used by the Visual Edit surface.

use std::ops::Range;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::frontmatter::split_front_matter;
use crate::model::{
    InlineStyle, PreviewBlock, VisualBlock, VisualBlockKind, VisualBlockPrefix,
    VisualBlockPrefixKind, VisualInlineRun, VisualProjection, VisualProjectionSegment,
    VisualProjectionSpan, VisualRevealGroup, VisualRevealKind, VisualSourceIslandKind,
};

impl VisualProjection {
    pub fn source_for_display(&self, display: usize) -> usize {
        let Some(first) = self.segments.first() else {
            return self.source_anchor;
        };
        for segment in &self.segments {
            if display <= segment.display_range.end {
                let local = display.saturating_sub(segment.display_range.start);
                return segment.source_range.start + local.min(segment.source_range.len());
            }
        }
        self.segments
            .last()
            .map_or(first.source_range.start, |segment| segment.source_range.end)
    }

    pub fn display_for_source(&self, source: usize) -> Option<usize> {
        let first = self.segments.first()?;
        for segment in &self.segments {
            if source >= segment.source_range.start && source <= segment.source_range.end {
                return Some(
                    segment.display_range.start
                        + source
                            .saturating_sub(segment.source_range.start)
                            .min(segment.display_range.len()),
                );
            }
        }
        if source < first.source_range.start {
            return Some(first.display_range.start);
        }
        for pair in self.segments.windows(2) {
            let previous = &pair[0];
            let next = &pair[1];
            if source > previous.source_range.end && source < next.source_range.start {
                let distance_to_previous = source - previous.source_range.end;
                let distance_to_next = next.source_range.start - source;
                return Some(if distance_to_previous <= distance_to_next {
                    previous.display_range.end
                } else {
                    next.display_range.start
                });
            }
        }
        self.segments
            .last()
            .map(|segment| segment.display_range.end)
    }
}

#[derive(Debug)]
enum ProjectionPiece {
    Rendered(usize),
    Source(Range<usize>),
}

impl ProjectionPiece {
    fn source_range<'a>(&'a self, block: &'a VisualBlock) -> &'a Range<usize> {
        match self {
            Self::Rendered(index) => &block.editable_runs[*index].content_range,
            Self::Source(range) => range,
        }
    }
}

pub fn build_visual_projection(
    source: &str,
    block: &VisualBlock,
    source_selection: Range<usize>,
    source_cursor: usize,
) -> VisualProjection {
    let endpoint_is_active = |range: &Range<usize>| {
        range.contains(&source_cursor)
            || (!source_selection.is_empty()
                && (range.contains(&source_selection.start)
                    || range.contains(&source_selection.end)))
    };
    let mut revealed_source_ranges = block
        .reveal_groups
        .iter()
        .filter(|group| endpoint_is_active(&group.source_range))
        .map(|group| group.source_range.clone())
        .collect::<Vec<_>>();
    if let Some(prefix) = &block.block_prefix
        && endpoint_is_active(&prefix.source_range)
    {
        revealed_source_ranges.push(prefix.source_range.clone());
    }
    // A caret inside nested syntax activates every containing reveal group.
    // Keep only the outermost range so source is emitted exactly once and the
    // display/source mapping stays monotonic.
    revealed_source_ranges.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.end.cmp(&left.end))
    });
    let mut normalized_ranges: Vec<Range<usize>> = Vec::new();
    for range in revealed_source_ranges {
        if normalized_ranges
            .iter()
            .any(|outer| outer.start <= range.start && outer.end >= range.end)
        {
            continue;
        }
        normalized_ranges.push(range);
    }
    let revealed_source_ranges = normalized_ranges;

    let mut pieces = revealed_source_ranges
        .iter()
        .cloned()
        .map(ProjectionPiece::Source)
        .collect::<Vec<_>>();
    for (index, run) in block.editable_runs.iter().enumerate() {
        if !revealed_source_ranges.iter().any(|range| {
            run.content_range.start >= range.start && run.content_range.end <= range.end
        }) {
            pieces.push(ProjectionPiece::Rendered(index));
        }
    }
    pieces.sort_by_key(|piece| {
        let range = piece.source_range(block);
        (range.start, range.end)
    });

    let mut projection = VisualProjection {
        text: String::new(),
        segments: Vec::with_capacity(pieces.len()),
        spans: Vec::with_capacity(pieces.len()),
        revealed_source_ranges,
        source_anchor: block.source_range.start,
    };
    for piece in pieces {
        match piece {
            ProjectionPiece::Rendered(index) => {
                let run = &block.editable_runs[index];
                let display_start = projection.text.len();
                projection.text.push_str(&run.visible_text);
                let display_range = display_start..projection.text.len();
                projection.segments.push(VisualProjectionSegment {
                    display_range: display_range.clone(),
                    source_range: run.content_range.clone(),
                });
                projection.spans.push(VisualProjectionSpan {
                    display_range,
                    style: run.style,
                    link: run.link_target_range.is_some(),
                    source: false,
                });
            }
            ProjectionPiece::Source(source_range) => {
                let display_start = projection.text.len();
                projection.text.push_str(&source[source_range.clone()]);
                let display_range = display_start..projection.text.len();
                projection.segments.push(VisualProjectionSegment {
                    display_range: display_range.clone(),
                    source_range,
                });
                projection.spans.push(VisualProjectionSpan {
                    display_range,
                    style: InlineStyle::default(),
                    link: false,
                    source: true,
                });
            }
        }
    }
    projection
}
use crate::parse::{ExtendedInlineKind, extended_inline_matches, markdown_options};

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

    let mut source_ranges = preview
        .iter()
        .map(|block| visual_block_source_range(text, block))
        .collect::<Vec<_>>();
    for index in 0..source_ranges.len().saturating_sub(1) {
        let nested_list_start = match (&preview[index], &preview[index + 1]) {
            (
                PreviewBlock::ListItem { level, .. },
                PreviewBlock::ListItem {
                    level: nested_level,
                    ..
                },
            ) if nested_level > level => Some(source_ranges[index + 1].start),
            _ => None,
        };
        if let Some(nested_list_start) = nested_list_start
            && nested_list_start > source_ranges[index].start
            && nested_list_start < source_ranges[index].end
        {
            // pulldown-cmark reports a parent list item's tag range as the
            // entire nested subtree. Partition that overlap at the first child
            // item so each visual row owns only its direct source text.
            source_ranges[index].end = nested_list_start;
        }
    }

    let mut covered_until = blocks.last().map_or(0, |block| block.source_range.end);
    for (preview_block, range) in preview.iter().zip(source_ranges) {
        if range.start > covered_until {
            blocks.push(gap_block(text, covered_until..range.start));
        }

        let mut block = visual_block_from_preview(text, preview_block, range.clone());
        if block.source_range.start < covered_until {
            block.source_island = Some(VisualSourceIslandKind::Unsupported);
        }
        covered_until = covered_until.max(range.end);
        blocks.push(block);
    }

    if covered_until < text.len() {
        blocks.push(gap_block(text, covered_until..text.len()));
    }
    blocks
}

fn gap_block(text: &str, range: Range<usize>) -> VisualBlock {
    if text[range.clone()].trim().is_empty() {
        VisualBlock {
            kind: VisualBlockKind::Whitespace,
            source_range: range,
            editable_runs: Vec::new(),
            reveal_groups: Vec::new(),
            marker_ranges: Vec::new(),
            block_prefix: None,
            source_island: None,
        }
    } else {
        source_island(range, VisualSourceIslandKind::Unsupported)
    }
}

fn source_island(range: Range<usize>, kind: VisualSourceIslandKind) -> VisualBlock {
    VisualBlock {
        kind: VisualBlockKind::Unsupported,
        source_range: range,
        editable_runs: Vec::new(),
        reveal_groups: Vec::new(),
        marker_ranges: Vec::new(),
        block_prefix: None,
        source_island: Some(kind),
    }
}

fn visual_block_from_preview(
    text: &str,
    block: &PreviewBlock,
    source_range: Range<usize>,
) -> VisualBlock {
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

    let block_prefix = block_prefix(text, &kind, source_range.clone());
    let inline_source_range = if matches!(kind, VisualBlockKind::ListItem { .. }) {
        block_prefix.as_ref().map_or_else(
            || source_range.clone(),
            |prefix| prefix.source_range.end..source_range.end,
        )
    } else {
        source_range.clone()
    };
    let (mut editable_runs, reveal_groups, contains_html) = inline_runs(text, inline_source_range);
    append_trailing_horizontal_whitespace_run(
        text,
        &source_range,
        block_prefix.as_ref(),
        &reveal_groups,
        &mut editable_runs,
    );
    let marker_ranges = marker_ranges(source_range.clone(), &editable_runs);
    VisualBlock {
        kind,
        source_range,
        editable_runs,
        reveal_groups,
        marker_ranges,
        block_prefix,
        source_island: source_island.or(contains_html.then_some(VisualSourceIslandKind::Html)),
    }
}

fn append_trailing_horizontal_whitespace_run(
    text: &str,
    block_range: &Range<usize>,
    block_prefix: Option<&VisualBlockPrefix>,
    reveal_groups: &[VisualRevealGroup],
    runs: &mut Vec<VisualInlineRun>,
) {
    let mut line_end = block_range.end;
    while line_end > block_range.start && matches!(text.as_bytes()[line_end - 1], b'\r' | b'\n') {
        line_end -= 1;
    }
    let mut whitespace_start = line_end;
    while whitespace_start > block_range.start
        && matches!(text.as_bytes()[whitespace_start - 1], b' ' | b'\t')
    {
        whitespace_start -= 1;
    }

    let represented_end = runs
        .iter()
        .map(|run| run.content_range.end)
        .chain(reveal_groups.iter().map(|group| group.source_range.end))
        .chain(block_prefix.map(|prefix| prefix.source_range.end))
        .max()
        .unwrap_or(block_range.start);
    let whitespace_start = whitespace_start.max(represented_end);
    if whitespace_start >= line_end {
        return;
    }

    let range = whitespace_start..line_end;
    runs.push(VisualInlineRun {
        visible_text: text[range.clone()].to_string(),
        source_range: range.clone(),
        content_range: range,
        style: InlineStyle::default(),
        link_target_range: None,
        conservative_fallback: false,
    });
    runs.sort_by_key(|run| (run.content_range.start, run.content_range.end));
}

fn visual_block_source_range(text: &str, block: &PreviewBlock) -> Range<usize> {
    let mut range = block.source_range().clone();
    if matches!(
        block,
        PreviewBlock::Heading { .. }
            | PreviewBlock::ListItem { .. }
            | PreviewBlock::BlockQuote { .. }
    ) {
        let line_start = text[..range.start].rfind('\n').map_or(0, |index| index + 1);
        if text[line_start..range.start]
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'\t'))
        {
            range.start = line_start;
        }
    }
    range
}

#[derive(Debug)]
struct RevealCandidate {
    kind: VisualRevealKind,
    source_range: Range<usize>,
    link_target_range: Option<Range<usize>>,
}

fn inline_runs(
    text: &str,
    block_range: Range<usize>,
) -> (Vec<VisualInlineRun>, Vec<VisualRevealGroup>, bool) {
    let source = &text[block_range.clone()];
    let mut runs = Vec::new();
    let mut candidates = Vec::new();
    let mut style = InlineStyle::default();
    let mut link_targets: Vec<Option<Range<usize>>> = Vec::new();
    let mut contains_html = false;

    for (event, relative_range) in Parser::new_ext(source, markdown_options()).into_offset_iter() {
        let event_range =
            block_range.start + relative_range.start..block_range.start + relative_range.end;
        match event {
            Event::Start(Tag::Strong) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Strong,
                    source_range: event_range.clone(),
                    link_target_range: None,
                });
                style.bold = true;
            }
            Event::End(TagEnd::Strong) => {
                style.bold = false;
            }
            Event::Start(Tag::Emphasis) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Emphasis,
                    source_range: event_range.clone(),
                    link_target_range: None,
                });
                style.italic = true;
            }
            Event::End(TagEnd::Emphasis) => {
                style.italic = false;
            }
            Event::Start(Tag::Strikethrough) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Strikethrough,
                    source_range: event_range.clone(),
                    link_target_range: None,
                });
                style.strikethrough = true;
            }
            Event::End(TagEnd::Strikethrough) => {
                style.strikethrough = false;
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Link,
                    source_range: event_range.clone(),
                    link_target_range: find_link_target(text, &event_range, &dest_url),
                });
                link_targets.push(find_link_target(text, &event_range, &dest_url));
            }
            Event::End(TagEnd::Link) => {
                link_targets.pop();
            }
            Event::Text(visible) => push_text_runs(
                &mut runs,
                &mut candidates,
                text,
                visible.as_ref(),
                event_range,
                style,
                link_targets.last().cloned().flatten(),
            ),
            Event::Code(visible) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::InlineCode,
                    source_range: event_range.clone(),
                    link_target_range: None,
                });
                let mut code_style = style;
                code_style.code = true;
                push_run(
                    &mut runs,
                    text,
                    visible.as_ref(),
                    event_range,
                    code_style,
                    link_targets.last().cloned().flatten(),
                    false,
                );
            }
            Event::SoftBreak | Event::HardBreak => push_run(
                &mut runs,
                text,
                "\n",
                event_range,
                style,
                link_targets.last().cloned().flatten(),
                false,
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
                    false,
                );
            }
            Event::InlineMath(visible) | Event::DisplayMath(visible) => push_run(
                &mut runs,
                text,
                visible.as_ref(),
                event_range,
                style,
                link_targets.last().cloned().flatten(),
                false,
            ),
            Event::Html(_) | Event::InlineHtml(_) => contains_html = true,
            _ => {}
        }
    }
    let mut reveal_groups = build_reveal_groups(text, &block_range, &mut runs, candidates);
    if contains_markdown_escape(source)
        && let Some(run) = runs.first_mut()
    {
        run.conservative_fallback = true;
        reveal_groups.clear();
    }
    (runs, reveal_groups, contains_html)
}

fn push_text_runs(
    runs: &mut Vec<VisualInlineRun>,
    candidates: &mut Vec<RevealCandidate>,
    source: &str,
    visible: &str,
    event_range: Range<usize>,
    base_style: InlineStyle,
    link_target_range: Option<Range<usize>>,
) {
    let event_source = &source[event_range.clone()];
    if event_source != visible {
        push_run(
            runs,
            source,
            visible,
            event_range,
            base_style,
            link_target_range,
            true,
        );
        return;
    }

    let extended = extended_inline_matches(event_source);
    if extended.is_empty() {
        push_run(
            runs,
            source,
            visible,
            event_range,
            base_style,
            link_target_range,
            false,
        );
        return;
    }

    let mut boundaries = vec![0, event_source.len()];
    let mut marker_ranges = Vec::with_capacity(extended.len() * 2);
    for item in &extended {
        boundaries.extend([
            item.source_range.start,
            item.content_range.start,
            item.content_range.end,
            item.source_range.end,
        ]);
        marker_ranges.push(item.source_range.start..item.content_range.start);
        marker_ranges.push(item.content_range.end..item.source_range.end);
        candidates.push(RevealCandidate {
            kind: match item.kind {
                ExtendedInlineKind::Highlight => VisualRevealKind::Highlight,
                ExtendedInlineKind::Superscript => VisualRevealKind::Superscript,
                ExtendedInlineKind::Subscript => VisualRevealKind::Subscript,
            },
            source_range: event_range.start + item.source_range.start
                ..event_range.start + item.source_range.end,
            link_target_range: None,
        });
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    for pair in boundaries.windows(2) {
        let local_range = pair[0]..pair[1];
        if local_range.is_empty()
            || marker_ranges
                .iter()
                .any(|marker| marker.start <= local_range.start && marker.end >= local_range.end)
        {
            continue;
        }

        let mut style = base_style;
        for item in &extended {
            if item.content_range.start <= local_range.start
                && item.content_range.end >= local_range.end
            {
                match item.kind {
                    ExtendedInlineKind::Highlight => style.highlight = true,
                    ExtendedInlineKind::Superscript => style.superscript = true,
                    ExtendedInlineKind::Subscript => style.subscript = true,
                }
            }
        }
        let global_range =
            event_range.start + local_range.start..event_range.start + local_range.end;
        push_run(
            runs,
            source,
            &event_source[local_range],
            global_range,
            style,
            link_target_range.clone(),
            false,
        );
    }
}

fn contains_markdown_escape(source: &str) -> bool {
    source
        .as_bytes()
        .windows(2)
        .any(|pair| pair[0] == b'\\' && pair[1].is_ascii_punctuation())
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
    let escaped_source = exact.is_some() && event_source != visible && event_source.contains('\\');
    let conservative_fallback = force_fallback || exact.is_none() || escaped_source;
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

fn build_reveal_groups(
    text: &str,
    block_range: &Range<usize>,
    runs: &mut [VisualInlineRun],
    mut candidates: Vec<RevealCandidate>,
) -> Vec<VisualRevealGroup> {
    candidates.sort_by_key(|candidate| (candidate.source_range.start, candidate.source_range.end));

    let ambiguous_overlap = candidates.iter().enumerate().any(|(index, candidate)| {
        candidates[index + 1..].iter().any(|other| {
            let overlaps = candidate.source_range.start < other.source_range.end
                && other.source_range.start < candidate.source_range.end;
            let candidate_contains_other = candidate.source_range.start <= other.source_range.start
                && candidate.source_range.end >= other.source_range.end;
            let other_contains_candidate = other.source_range.start <= candidate.source_range.start
                && other.source_range.end >= candidate.source_range.end;
            overlaps && !candidate_contains_other && !other_contains_candidate
        })
    });
    let mut invalid = ambiguous_overlap;
    let mut groups = Vec::new();

    if !ambiguous_overlap {
        for candidate in candidates {
            if !reveal_candidate_is_exact(text, block_range, &candidate) {
                invalid = true;
                continue;
            }
            let content_ranges = runs
                .iter()
                .filter(|run| {
                    run.content_range.start >= candidate.source_range.start
                        && run.content_range.end <= candidate.source_range.end
                })
                .map(|run| run.content_range.clone())
                .collect::<Vec<_>>();
            let content_is_exact = !content_ranges.is_empty()
                && runs
                    .iter()
                    .filter(|run| {
                        run.content_range.start >= candidate.source_range.start
                            && run.content_range.end <= candidate.source_range.end
                    })
                    .all(|run| !run.conservative_fallback);
            if !content_is_exact {
                invalid = true;
                continue;
            }
            groups.push(VisualRevealGroup {
                kind: candidate.kind,
                source_range: candidate.source_range,
                content_ranges,
                link_target_range: candidate.link_target_range,
            });
        }
    }

    if invalid && let Some(run) = runs.first_mut() {
        run.conservative_fallback = true;
        groups.clear();
    }
    groups
}

fn reveal_candidate_is_exact(
    text: &str,
    block_range: &Range<usize>,
    candidate: &RevealCandidate,
) -> bool {
    let range = &candidate.source_range;
    if range.is_empty()
        || range.start < block_range.start
        || range.end > block_range.end
        || !text.is_char_boundary(range.start)
        || !text.is_char_boundary(range.end)
    {
        return false;
    }
    let source = &text[range.clone()];
    match candidate.kind {
        VisualRevealKind::Strong => {
            (source.starts_with("**") && source.ends_with("**") && source.len() >= 4)
                || (source.starts_with("__") && source.ends_with("__") && source.len() >= 4)
        }
        VisualRevealKind::Emphasis => {
            (source.starts_with('*') && source.ends_with('*') && source.len() >= 2)
                || (source.starts_with('_') && source.ends_with('_') && source.len() >= 2)
        }
        VisualRevealKind::Strikethrough => {
            source.starts_with("~~") && source.ends_with("~~") && source.len() >= 4
        }
        VisualRevealKind::InlineCode => source.starts_with('`') && source.ends_with('`'),
        VisualRevealKind::Link => {
            source.starts_with('[')
                && source.ends_with(')')
                && source.contains("](")
                && candidate.link_target_range.as_ref().is_some_and(|target| {
                    target.start >= range.start
                        && target.end <= range.end
                        && text.is_char_boundary(target.start)
                        && text.is_char_boundary(target.end)
                })
        }
        VisualRevealKind::Highlight
        | VisualRevealKind::Superscript
        | VisualRevealKind::Subscript => extended_inline_matches(source).iter().any(|item| {
            let expected_kind = match candidate.kind {
                VisualRevealKind::Highlight => ExtendedInlineKind::Highlight,
                VisualRevealKind::Superscript => ExtendedInlineKind::Superscript,
                VisualRevealKind::Subscript => ExtendedInlineKind::Subscript,
                _ => unreachable!("matched extended reveal kind"),
            };
            item.kind == expected_kind && item.source_range == (0..source.len())
        }),
    }
}

fn block_prefix(
    text: &str,
    kind: &VisualBlockKind,
    block_range: Range<usize>,
) -> Option<VisualBlockPrefix> {
    let line_end = text[block_range.clone()]
        .find('\n')
        .map_or(block_range.end, |relative| block_range.start + relative);
    let line = &text[block_range.start..line_end];
    let indentation_len = line
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let marker_start = indentation_len;
    let indentation_range = block_range.start..block_range.start + indentation_len;

    let (prefix_end, prefix_kind) = match kind {
        VisualBlockKind::Heading { level } => {
            let marker_len = line[marker_start..]
                .bytes()
                .take_while(|byte| *byte == b'#')
                .count();
            if marker_len != *level as usize {
                return None;
            }
            let end = skip_ascii_spacing(line, marker_start + marker_len);
            (end, VisualBlockPrefixKind::Heading { level: *level })
        }
        VisualBlockKind::BlockQuote => {
            let mut cursor = marker_start;
            let mut depth = 0;
            while line.as_bytes().get(cursor) == Some(&b'>') {
                depth += 1;
                cursor += 1;
                cursor = skip_ascii_spacing(line, cursor);
            }
            if depth == 0 {
                return None;
            }
            (cursor, VisualBlockPrefixKind::BlockQuote { depth })
        }
        VisualBlockKind::ListItem {
            level,
            ordered,
            index,
            checked,
        } => {
            let bytes = line.as_bytes();
            let mut cursor = marker_start;
            let parsed_index;
            if *ordered {
                let digits_start = cursor;
                while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
                    cursor += 1;
                }
                if cursor == digits_start || !matches!(bytes.get(cursor), Some(b'.' | b')')) {
                    return None;
                }
                parsed_index = line[digits_start..cursor].parse::<u64>().ok();
                cursor += 1;
            } else {
                if !matches!(bytes.get(cursor), Some(b'-' | b'+' | b'*')) {
                    return None;
                }
                parsed_index = None;
                cursor += 1;
            }
            let after_marker = skip_ascii_spacing(line, cursor);
            if after_marker == cursor {
                return None;
            }
            cursor = after_marker;
            if let Some(is_checked) = checked {
                let task = bytes.get(cursor..cursor + 3)?;
                if task.first() != Some(&b'[')
                    || task.last() != Some(&b']')
                    || !matches!(task[1], b' ' | b'x' | b'X')
                {
                    return None;
                }
                cursor = skip_ascii_spacing(line, cursor + 3);
                (
                    cursor,
                    VisualBlockPrefixKind::TaskList {
                        level: *level,
                        checked: *is_checked,
                    },
                )
            } else if *ordered {
                (
                    cursor,
                    VisualBlockPrefixKind::OrderedList {
                        level: *level,
                        index: index.or(parsed_index).unwrap_or(1),
                    },
                )
            } else {
                (
                    cursor,
                    VisualBlockPrefixKind::UnorderedList { level: *level },
                )
            }
        }
        _ => return None,
    };

    Some(VisualBlockPrefix {
        kind: prefix_kind,
        indentation_range,
        source_range: block_range.start..block_range.start + prefix_end,
    })
}

fn skip_ascii_spacing(text: &str, mut offset: usize) -> usize {
    while matches!(text.as_bytes().get(offset), Some(b' ' | b'\t')) {
        offset += 1;
    }
    offset
}

pub(crate) fn structural_prefix_at(text: &str, byte_index: usize) -> Option<VisualBlockPrefix> {
    if text.is_empty() {
        return None;
    }
    let mut cursor = byte_index.min(text.len());
    while cursor > 0 && !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    let line_start = text[..cursor].rfind('\n').map_or(0, |index| index + 1);
    let line_end = text[cursor..]
        .find('\n')
        .map_or(text.len(), |relative| cursor + relative);
    let line = &text[line_start..line_end];
    let indentation_len = line
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let indentation_range = line_start..line_start + indentation_len;
    let marker_start = indentation_len;
    let bytes = line.as_bytes();

    let (prefix_end, kind) = if bytes.get(marker_start) == Some(&b'#') {
        let level = line[marker_start..]
            .bytes()
            .take_while(|byte| *byte == b'#')
            .count();
        if !(1..=6).contains(&level) || bytes.get(marker_start + level) != Some(&b' ') {
            return None;
        }
        (
            skip_ascii_spacing(line, marker_start + level),
            VisualBlockPrefixKind::Heading { level: level as u8 },
        )
    } else if bytes.get(marker_start) == Some(&b'>') {
        let mut end = marker_start;
        let mut depth = 0;
        while bytes.get(end) == Some(&b'>') {
            depth += 1;
            end += 1;
            end = skip_ascii_spacing(line, end);
        }
        (end, VisualBlockPrefixKind::BlockQuote { depth })
    } else if matches!(bytes.get(marker_start), Some(b'-' | b'+' | b'*')) {
        let marker = bytes[marker_start];
        let after_marker = skip_ascii_spacing(line, marker_start + 1);
        if after_marker == marker_start + 1 {
            return None;
        }
        if let Some(task) = bytes.get(after_marker..after_marker + 3)
            && task.first() == Some(&b'[')
            && task.last() == Some(&b']')
            && matches!(task[1], b' ' | b'x' | b'X')
        {
            (
                skip_ascii_spacing(line, after_marker + 3),
                VisualBlockPrefixKind::TaskList {
                    level: 1,
                    checked: !matches!(task[1], b' '),
                },
            )
        } else {
            let _ = marker;
            (
                after_marker,
                VisualBlockPrefixKind::UnorderedList { level: 1 },
            )
        }
    } else {
        let digits_start = marker_start;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == digits_start || !matches!(bytes.get(end), Some(b'.' | b')')) {
            return None;
        }
        let index = line[digits_start..end].parse::<u64>().ok()?;
        let after_marker = skip_ascii_spacing(line, end + 1);
        if after_marker == end + 1 {
            return None;
        }
        (
            after_marker,
            VisualBlockPrefixKind::OrderedList { level: 1, index },
        )
    };

    Some(VisualBlockPrefix {
        kind,
        indentation_range,
        source_range: line_start..line_start + prefix_end,
    })
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

    use super::build_visual_projection;
    use crate::{
        MarkdownDocument, MarkdownFormat, TableEdit, VisualBlockKind, VisualBlockPrefixKind,
        VisualRevealKind, VisualSourceIslandKind,
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
    fn derives_exact_reveal_groups_and_structural_prefixes() {
        let source = "# Hé **世界** and _italic_ plus ~~gone~~ and `code` [站点](https://example.com \"Title\")\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let block = &blocks[0];

        let prefix = block.block_prefix.as_ref().expect("heading prefix");
        assert_eq!(prefix.kind, VisualBlockPrefixKind::Heading { level: 1 });
        assert_eq!(&source[prefix.source_range.clone()], "# ");
        assert_eq!(prefix.indentation_range, 0..0);

        let group_sources = block
            .reveal_groups
            .iter()
            .map(|group| (group.kind, &source[group.source_range.clone()]))
            .collect::<Vec<_>>();
        assert!(group_sources.contains(&(VisualRevealKind::Strong, "**世界**")));
        assert!(group_sources.contains(&(VisualRevealKind::Emphasis, "_italic_")));
        assert!(group_sources.contains(&(VisualRevealKind::Strikethrough, "~~gone~~")));
        assert!(group_sources.contains(&(VisualRevealKind::InlineCode, "`code`")));
        assert!(group_sources.contains(&(
            VisualRevealKind::Link,
            "[站点](https://example.com \"Title\")"
        )));

        let link = block
            .reveal_groups
            .iter()
            .find(|group| group.kind == VisualRevealKind::Link)
            .unwrap();
        assert_eq!(
            &source[link.link_target_range.clone().unwrap()],
            "https://example.com"
        );
        assert!(block.reveal_groups.iter().all(|group| {
            source.is_char_boundary(group.source_range.start)
                && source.is_char_boundary(group.source_range.end)
                && group.content_ranges.iter().all(|range| {
                    source.is_char_boundary(range.start) && source.is_char_boundary(range.end)
                })
        }));
    }

    #[test]
    fn derives_supported_quote_and_list_prefixes() {
        let cases = [
            (
                "> quote\n",
                VisualBlockPrefixKind::BlockQuote { depth: 1 },
                "> ",
            ),
            (
                "3. ordered\n",
                VisualBlockPrefixKind::OrderedList { level: 1, index: 3 },
                "3. ",
            ),
            (
                "- [x] done\n",
                VisualBlockPrefixKind::TaskList {
                    level: 1,
                    checked: true,
                },
                "- [x] ",
            ),
        ];

        for (source, expected_kind, expected_prefix) in cases {
            let doc = MarkdownDocument::from_text(source);
            let blocks = doc.visual_blocks();
            let block = blocks
                .iter()
                .find(|block| block.block_prefix.is_some())
                .expect("supported block prefix");
            let prefix = block.block_prefix.as_ref().unwrap();
            assert_eq!(prefix.kind, expected_kind, "source: {source:?}");
            assert_eq!(
                &source[prefix.source_range.clone()],
                expected_prefix,
                "source: {source:?}"
            );
        }

        let nested_source = "- parent\n  - nested\n";
        let nested = MarkdownDocument::from_text(nested_source).visual_blocks();
        let prefix = nested
            .iter()
            .find_map(|block| {
                block.block_prefix.as_ref().filter(|prefix| {
                    matches!(
                        prefix.kind,
                        VisualBlockPrefixKind::UnorderedList { level: 2 }
                    )
                })
            })
            .expect("nested list prefix");
        assert_eq!(&nested_source[prefix.source_range.clone()], "  - ");
        assert_eq!(&nested_source[prefix.indentation_range.clone()], "  ");
    }

    #[test]
    fn escaped_inline_syntax_uses_conservative_fallback() {
        let source = r"escaped \*marker\*";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        assert!(blocks[0].reveal_groups.is_empty());
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.conservative_fallback)
        );
    }

    #[test]
    fn nested_and_extended_inline_runs_stay_visual() {
        let source =
            "plain *italic* **bold** ***both*** ~~gone~~ `code` [link](url) ==mark== H~2~O x^2^";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];

        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        assert_eq!(
            block
                .editable_runs
                .iter()
                .map(|run| run.visible_text.as_str())
                .collect::<String>(),
            "plain italic bold both gone code link mark H2O x2"
        );

        let both = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "both")
            .unwrap();
        assert!(both.style.bold && both.style.italic);
        assert!(
            block
                .editable_runs
                .iter()
                .find(|run| run.visible_text == "mark")
                .unwrap()
                .style
                .highlight
        );
        assert!(
            block
                .editable_runs
                .iter()
                .find(|run| run.visible_text == "2" && run.style.subscript)
                .is_some()
        );
        assert!(
            block
                .editable_runs
                .iter()
                .find(|run| run.visible_text == "2" && run.style.superscript)
                .is_some()
        );

        let group_sources = block
            .reveal_groups
            .iter()
            .map(|group| (group.kind, &source[group.source_range.clone()]))
            .collect::<Vec<_>>();
        assert!(group_sources.contains(&(VisualRevealKind::Highlight, "==mark==")));
        assert!(group_sources.contains(&(VisualRevealKind::Subscript, "~2~")));
        assert!(group_sources.contains(&(VisualRevealKind::Superscript, "^2^")));
    }

    #[test]
    fn nested_projection_reveals_one_outermost_group_and_reuses_cache() {
        let source = "before ***世界*** after ==高亮==";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let version = doc.version();
        let block = &blocks[0];
        let plain_cursor = source.find("before").unwrap();
        let plain =
            build_visual_projection(source, block, plain_cursor..plain_cursor, plain_cursor);
        assert_eq!(plain.text, "before 世界 after 高亮");
        assert!(plain.revealed_source_ranges.is_empty());

        let nested_cursor = source.find("世界").unwrap();
        let nested =
            build_visual_projection(source, block, nested_cursor..nested_cursor, nested_cursor);
        let nested_range = source.find("***").unwrap()..source.find("***").unwrap() + 12;
        assert_eq!(nested.revealed_source_ranges, vec![nested_range.clone()]);
        assert_eq!(nested.text, "before ***世界*** after 高亮");
        for source_offset in nested_range {
            let display = nested.display_for_source(source_offset).unwrap();
            assert_eq!(nested.source_for_display(display), source_offset);
        }

        let highlight_cursor = source.find("高亮").unwrap();
        let highlight = build_visual_projection(
            source,
            block,
            highlight_cursor..highlight_cursor,
            highlight_cursor,
        );
        assert_eq!(highlight.text, "before 世界 after ==高亮==");
        assert_eq!(doc.version(), version);
        assert!(Arc::ptr_eq(&blocks, &doc.visual_blocks_shared()));
    }

    #[test]
    fn reveal_metadata_reuses_the_per_version_visual_cache() {
        let doc = MarkdownDocument::from_text("plain **bold** and [link](target)");
        let first = doc.visual_blocks_shared();
        let second = doc.visual_blocks_shared();
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first[0].reveal_groups.len(), 2);
    }

    #[test]
    fn projection_reveals_only_the_active_inline_group() {
        let source = "plain **世界** and [site](url \"Title\")";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];

        let plain_cursor = source.find("plain").unwrap() + 1;
        let plain =
            build_visual_projection(source, block, plain_cursor..plain_cursor, plain_cursor);
        assert_eq!(plain.text, "plain 世界 and site");
        assert!(plain.revealed_source_ranges.is_empty());

        let bold_cursor = source.find("世界").unwrap();
        let bold = build_visual_projection(source, block, bold_cursor..bold_cursor, bold_cursor);
        assert_eq!(bold.text, "plain **世界** and site");
        assert_eq!(
            bold.revealed_source_ranges
                .iter()
                .map(|range| &source[range.clone()])
                .collect::<Vec<_>>(),
            vec!["**世界**"]
        );

        let link_cursor = source.find("site").unwrap();
        let link = build_visual_projection(source, block, link_cursor..link_cursor, link_cursor);
        assert_eq!(link.text, "plain 世界 and [site](url \"Title\")");
        assert_eq!(
            link.revealed_source_ranges
                .iter()
                .map(|range| &source[range.clone()])
                .collect::<Vec<_>>(),
            vec!["[site](url \"Title\")"]
        );
    }

    #[test]
    fn projection_preserves_trailing_horizontal_whitespace() {
        for (source, expected) in [
            ("## heading ", "heading "),
            ("- item ", "item "),
            ("> quote ", "quote "),
        ] {
            let doc = MarkdownDocument::from_text(source);
            let blocks = doc.visual_blocks_shared();
            let block = blocks
                .iter()
                .find(|block| {
                    matches!(
                        block.kind,
                        VisualBlockKind::Heading { .. }
                            | VisualBlockKind::ListItem { .. }
                            | VisualBlockKind::BlockQuote
                    )
                })
                .expect("supported visual block");
            let cursor = source.len();
            let projection = build_visual_projection(source, block, cursor..cursor, cursor);

            assert_eq!(projection.text, expected, "source: {source:?}");
            assert_eq!(
                projection.display_for_source(cursor),
                Some(expected.len()),
                "source: {source:?}"
            );
            assert_eq!(
                projection.source_for_display(expected.len()),
                cursor,
                "source: {source:?}"
            );
        }
    }

    #[test]
    fn nested_list_visual_blocks_do_not_duplicate_descendant_text() {
        let source = "- parent\n  - child\n    - grandchild\n1. ordered\n   1. nested\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let list_blocks = blocks
            .iter()
            .filter(|block| matches!(block.kind, VisualBlockKind::ListItem { .. }))
            .collect::<Vec<_>>();

        assert_eq!(list_blocks.len(), 5);
        assert!(
            list_blocks
                .iter()
                .all(|block| block.source_island.is_none())
        );
        assert!(
            list_blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end <= pair[1].source_range.start)
        );

        let projected = list_blocks
            .iter()
            .map(|block| {
                let cursor = block
                    .editable_runs
                    .first()
                    .expect("list item content")
                    .content_range
                    .start;
                build_visual_projection(source, block, cursor..cursor, cursor).text
            })
            .collect::<Vec<_>>();
        assert_eq!(
            projected,
            ["parent", "child", "grandchild", "ordered", "nested"]
        );
    }

    #[test]
    fn projection_mapping_round_trips_utf8_and_revealed_markers() {
        let source = "α **世界** omega";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        let marker_cursor = source.find("**").unwrap();
        let projection =
            build_visual_projection(source, block, marker_cursor..marker_cursor, marker_cursor);

        for segment in &projection.segments {
            for source_offset in segment.source_range.clone() {
                if source.is_char_boundary(source_offset) {
                    let display = projection.display_for_source(source_offset).unwrap();
                    assert_eq!(
                        projection.source_for_display(display),
                        source_offset,
                        "source offset {source_offset} in {segment:?}"
                    );
                }
            }
            let source_end = segment.source_range.end;
            if source.is_char_boundary(source_end) {
                let display = projection.display_for_source(source_end).unwrap();
                assert_eq!(projection.source_for_display(display), source_end);
            }
        }
        assert!(projection.text.contains("**世界**"));
    }

    #[test]
    fn projection_maps_hidden_markers_to_stable_boundaries_until_revealed() {
        let source = "before **bold** after";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        let plain_cursor = source.find("before").unwrap();
        let hidden =
            build_visual_projection(source, block, plain_cursor..plain_cursor, plain_cursor);
        let marker = source.find("**").unwrap();
        let boundary = hidden.display_for_source(marker).unwrap();
        assert!(boundary <= hidden.text.len());
        assert_eq!(hidden.text, "before bold after");

        let revealed = build_visual_projection(source, block, marker..marker, marker);
        let display = revealed.display_for_source(marker).unwrap();
        assert_eq!(revealed.source_for_display(display), marker);
        assert_eq!(revealed.text, "before **bold** after");
    }

    #[test]
    fn cross_run_projection_selection_keeps_source_endpoints_and_cache() {
        let source = "start **bold** middle [link](url) end";
        let doc = MarkdownDocument::from_text(source);
        let version = doc.version();
        let blocks = doc.visual_blocks_shared();
        let selection = source.find("start").unwrap()..source.find(" end").unwrap();
        let projection =
            build_visual_projection(source, &blocks[0], selection.clone(), selection.end);

        assert_eq!(projection.text, "start bold middle link end");
        assert!(projection.display_for_source(selection.start).is_some());
        assert!(projection.display_for_source(selection.end).is_some());
        assert_eq!(doc.version(), version);
        let cached_again = doc.visual_blocks_shared();
        assert!(Arc::ptr_eq(&blocks, &cached_again));
    }

    #[test]
    fn uses_conservative_fallback_when_visible_text_is_not_byte_exact() {
        let doc = MarkdownDocument::from_text("A &amp; B");
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
    fn whitespace_gaps_and_trailing_source_are_preserved_as_visual_rows() {
        let source = "first\n\n\nsecond\n\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();

        assert_eq!(blocks.first().unwrap().source_range.start, 0);
        assert_eq!(blocks.last().unwrap().source_range.end, source.len());
        assert!(
            blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end == pair[1].source_range.start),
            "visual rows must cover the canonical source without caret gaps: {blocks:#?}"
        );

        let whitespace = blocks
            .iter()
            .filter(|block| matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert!(!whitespace.is_empty());
        assert!(whitespace.iter().all(|block| {
            !block.source_range.is_empty()
                && source[block.source_range.clone()].trim().is_empty()
                && block.source_island.is_none()
        }));

        let cached = doc.visual_blocks_shared();
        let cached_again = doc.visual_blocks_shared();
        assert!(Arc::ptr_eq(&cached, &cached_again));
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
        let inline_formatting = editable_blocks
            .iter()
            .find(|block| {
                block
                    .editable_runs
                    .iter()
                    .any(|run| run.visible_text.contains("Write with"))
            })
            .expect("welcome inline-formatting paragraph stays visual");
        assert!(
            inline_formatting
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        assert!(inline_formatting.editable_runs.iter().any(|run| {
            run.visible_text == "bold italic" && run.style.bold && run.style.italic
        }));
        assert!(
            inline_formatting
                .editable_runs
                .iter()
                .any(|run| run.style.highlight)
        );
        assert!(
            inline_formatting
                .editable_runs
                .iter()
                .any(|run| run.style.subscript)
        );
        assert!(
            inline_formatting
                .editable_runs
                .iter()
                .any(|run| run.style.superscript)
        );
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
