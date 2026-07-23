//! Source-ranged model used by the Visual Edit surface.

use std::ops::Range;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::frontmatter::split_front_matter;
use crate::model::{
    InlineStyle, MathDelimiter, MathLayoutStyle, MathSource, PreviewBlock, VisualBlock,
    VisualBlockEditor, VisualBlockId, VisualBlockKind, VisualBlockPrefix, VisualBlockPrefixKind,
    VisualBoundaryCandidates, VisualCaretAffinity, VisualEditorField, VisualEditorFieldKind,
    VisualInlineRun, VisualNavigationTarget, VisualProjection, VisualProjectionSegment,
    VisualProjectionSpan, VisualRevealGroup, VisualRevealKind, VisualSourceIslandKind,
    VisualTableCell,
};
use crate::table::table_cell_source_ranges;
use crate::source_mapped::{is_closing_fence, is_reference_definition, opening_fence};

/// Collects the document's link reference definition lines so that per-block
/// parsing in `inline_runs` can resolve reference-style links whose
/// definitions live in other blocks. Lines inside fenced code blocks are
/// skipped (they are code, not definitions), and footnote definitions
/// (`[^label]:`) are excluded — `^` cannot start a link label, and keeping
/// them out guarantees the appended suffix produces no parser events of its
/// own interest to block mapping.
fn collect_link_reference_definitions(text: &str) -> String {
    let mut definitions = String::new();
    let mut fence: Option<(char, usize)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some((marker, minimum)) = fence {
            if is_closing_fence(trimmed, marker, minimum) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = opening_fence(trimmed) {
            fence = Some(open);
            continue;
        }
        // Link reference definitions allow at most three leading spaces.
        if line.len() - trimmed.len() <= 3
            && !trimmed.starts_with("[^")
            && is_reference_definition(trimmed)
        {
            definitions.push_str(trimmed);
            definitions.push('\n');
        }
    }
    definitions
}

/// Collects minimal `[^label]:` stubs so per-block `inline_runs` parsing can
/// emit `FootnoteReference` events. Full definition bodies are intentionally
/// omitted — stubs are enough for pulldown-cmark to resolve references, and
/// they emit no in-block events past the prose slice.
fn collect_footnote_definition_stubs(text: &str) -> String {
    let mut stubs = String::new();
    let mut fence: Option<(char, usize)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some((marker, minimum)) = fence {
            if is_closing_fence(trimmed, marker, minimum) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = opening_fence(trimmed) {
            fence = Some(open);
            continue;
        }
        if line.len() - trimmed.len() <= 3
            && let Some(label) = footnote_definition_label(trimmed)
        {
            stubs.push_str("[^");
            stubs.push_str(label);
            stubs.push_str("]:\n");
        }
    }
    stubs
}

fn footnote_definition_label(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("[^")?;
    let close = rest.find("]:")?;
    let label = &rest[..close];
    (!label.is_empty()).then_some(label)
}

/// True when every non-blank line in `slice` is a link reference definition
/// (not a footnote definition). Used to classify uncovered gaps that would
/// otherwise become Unsupported source islands.
fn is_link_reference_definition_gap(slice: &str) -> bool {
    let mut saw_definition = false;
    for line in slice.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("[^") || !is_reference_definition(trimmed) {
            return false;
        }
        saw_definition = true;
    }
    saw_definition
}

impl VisualProjection {
    pub fn boundary_candidates(&self, display: usize) -> VisualBoundaryCandidates {
        let display = clamp_to_char_boundary(&self.text, display);
        let Some(first) = self.segments.first() else {
            return VisualBoundaryCandidates {
                display_offset: display,
                upstream_source: self.source_anchor,
                downstream_source: self.source_anchor,
            };
        };

        for segment in &self.segments {
            if display > segment.display_range.start && display < segment.display_range.end {
                let exact = segment.source_range.start + display - segment.display_range.start;
                return VisualBoundaryCandidates {
                    display_offset: display,
                    upstream_source: exact,
                    downstream_source: exact,
                };
            }
        }

        let upstream = self
            .segments
            .iter()
            .rev()
            .find(|segment| segment.display_range.end <= display)
            .map(|segment| segment.source_range.end)
            .unwrap_or(first.source_range.start);
        let downstream = self
            .segments
            .iter()
            .find(|segment| segment.display_range.start >= display)
            .map(|segment| segment.source_range.start)
            .or_else(|| self.segments.last().map(|segment| segment.source_range.end))
            .unwrap_or(upstream);
        VisualBoundaryCandidates {
            display_offset: display,
            upstream_source: upstream,
            downstream_source: downstream,
        }
    }

    pub fn source_for_display_with_affinity(
        &self,
        display: usize,
        affinity: VisualCaretAffinity,
    ) -> usize {
        self.boundary_candidates(display).resolve(affinity)
    }

    pub fn source_for_display(&self, display: usize) -> usize {
        self.source_for_display_with_affinity(display, VisualCaretAffinity::Upstream)
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

    pub fn affinity_for_source(&self, source: usize) -> Option<VisualCaretAffinity> {
        let display = self.display_for_source(source)?;
        let candidates = self.boundary_candidates(display);
        if !candidates.is_ambiguous() {
            return None;
        }
        if source == candidates.upstream_source {
            Some(VisualCaretAffinity::Upstream)
        } else if source == candidates.downstream_source {
            Some(VisualCaretAffinity::Downstream)
        } else {
            None
        }
    }

    pub fn source_is_exactly_projected(&self, source: usize) -> bool {
        self.segments.iter().any(|segment| {
            source >= segment.source_range.start && source <= segment.source_range.end
        })
    }

    pub fn display_range_for_source_range(&self, source: Range<usize>) -> Option<Range<usize>> {
        let start = self.display_for_source(source.start)?;
        let end = self.display_for_source(source.end)?;
        (self.source_is_exactly_projected(source.start)
            && self.source_is_exactly_projected(source.end))
        .then_some(start.min(end)..start.max(end))
    }
}

fn clamp_to_char_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
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
    build_visual_projection_with_marked_range(source, block, source_selection, source_cursor, None)
}

pub fn build_visual_projection_with_marked_range(
    source: &str,
    block: &VisualBlock,
    source_selection: Range<usize>,
    source_cursor: usize,
    marked_range: Option<Range<usize>>,
) -> VisualProjection {
    let endpoint_is_active = |range: &Range<usize>, include_end: bool| {
        range.contains(&source_cursor)
            || (include_end && source_cursor == range.end)
            || (!source_selection.is_empty()
                && (range.contains(&source_selection.start)
                    || range.contains(&source_selection.end)
                    || (include_end
                        && (source_selection.start == range.end
                            || source_selection.end == range.end))))
            || marked_range.as_ref().is_some_and(|marked| {
                !marked.is_empty() && marked.start < range.end && marked.end > range.start
            })
    };
    let mut revealed_source_ranges = block
        .reveal_groups
        .iter()
        .filter(|group| {
            endpoint_is_active(
                &group.source_range,
                matches!(group.kind, VisualRevealKind::Math),
            )
        })
        .map(|group| group.source_range.clone())
        .collect::<Vec<_>>();
    if let Some(prefix) = &block.block_prefix
        && endpoint_is_active(&prefix.source_range, false)
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

    let link_group_ranges = block
        .reveal_groups
        .iter()
        .filter(|group| group.kind == VisualRevealKind::Link)
        .map(|group| &group.source_range)
        .collect::<Vec<_>>();
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
                // A run belongs to a link when it carries a local destination
                // (inline link) or sits inside a resolved reference-style
                // link's reveal group, whose target lives in another block.
                let in_link = run.link_target_range.is_some()
                    || link_group_ranges.iter().any(|range| {
                        range.start <= run.content_range.start
                            && run.content_range.end <= range.end
                    });
                projection.spans.push(VisualProjectionSpan {
                    display_range,
                    style: run.style,
                    link: in_link,
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

pub(crate) fn build_visual_blocks(
    text: &str,
    preview: &[PreviewBlock],
    mut allocate_id: impl FnMut() -> VisualBlockId,
) -> Vec<VisualBlock> {
    // Link / footnote definitions are document-scoped; per-block parsing needs
    // them appended to resolve references (see `inline_runs`).
    let mut reference_definitions = collect_link_reference_definitions(text);
    let footnote_stubs = collect_footnote_definition_stubs(text);
    if !footnote_stubs.is_empty() {
        if !reference_definitions.is_empty() && !reference_definitions.ends_with('\n') {
            reference_definitions.push('\n');
        }
        reference_definitions.push_str(&footnote_stubs);
    }
    let mut blocks = Vec::with_capacity(preview.len() + 1);
    if let Some((_, body_start)) = split_front_matter(text)
        && body_start > 0
    {
        blocks.push(source_island(
            0..body_start,
            VisualSourceIslandKind::FrontMatter,
            &mut allocate_id,
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
            blocks.push(gap_block(
                text,
                covered_until..range.start,
                &mut allocate_id,
            ));
        }

        let mut block = visual_block_from_preview(
            text,
            preview_block,
            range.clone(),
            &reference_definitions,
            &mut allocate_id,
        );
        if block.source_range.start < covered_until {
            block.source_island = Some(VisualSourceIslandKind::Unsupported);
        }
        covered_until = covered_until.max(range.end);
        blocks.push(block);
    }

    if covered_until < text.len() {
        blocks.push(gap_block(text, covered_until..text.len(), &mut allocate_id));
    }
    blocks
}

fn gap_block(
    text: &str,
    range: Range<usize>,
    allocate_id: &mut impl FnMut() -> VisualBlockId,
) -> VisualBlock {
    let slice = &text[range.clone()];
    if slice.trim().is_empty() {
        VisualBlock {
            id: allocate_id(),
            kind: VisualBlockKind::Whitespace,
            source_range: range,
            editable_runs: Vec::new(),
            reveal_groups: Vec::new(),
            marker_ranges: Vec::new(),
            block_prefix: None,
            source_island: None,
            editor: None,
        }
    } else if is_link_reference_definition_gap(slice) {
        VisualBlock {
            id: allocate_id(),
            kind: VisualBlockKind::ReferenceDefinition,
            source_range: range,
            editable_runs: Vec::new(),
            reveal_groups: Vec::new(),
            marker_ranges: Vec::new(),
            block_prefix: None,
            source_island: None,
            editor: None,
        }
    } else {
        source_island(range, VisualSourceIslandKind::Unsupported, allocate_id)
    }
}

fn source_island(
    range: Range<usize>,
    kind: VisualSourceIslandKind,
    allocate_id: &mut impl FnMut() -> VisualBlockId,
) -> VisualBlock {
    VisualBlock {
        id: allocate_id(),
        kind: VisualBlockKind::Unsupported,
        source_range: range,
        editable_runs: Vec::new(),
        reveal_groups: Vec::new(),
        marker_ranges: Vec::new(),
        block_prefix: None,
        source_island: Some(kind),
        editor: None,
    }
}

fn visual_block_from_preview(
    text: &str,
    block: &PreviewBlock,
    source_range: Range<usize>,
    reference_definitions: &str,
    allocate_id: &mut impl FnMut() -> VisualBlockId,
) -> VisualBlock {
    let (kind, mut source_island) = match block {
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
        PreviewBlock::MathBlock {
            latex,
            authored,
            delimiter,
            ..
        } => (
            VisualBlockKind::MathBlock {
                latex: latex.clone(),
                authored: authored.clone(),
                delimiter: *delimiter,
            },
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
        PreviewBlock::FootnoteDefinition { label, .. } => (
            VisualBlockKind::FootnoteDefinition {
                label: label.clone(),
            },
            None,
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
    let (mut editable_runs, reveal_groups, contains_html) =
        inline_runs(text, inline_source_range, reference_definitions);
    append_trailing_horizontal_whitespace_run(
        text,
        &source_range,
        block_prefix.as_ref(),
        &reveal_groups,
        &mut editable_runs,
    );
    let marker_ranges = marker_ranges(source_range.clone(), &editable_runs);
    let editor = visual_block_editor(text, block, source_range.clone());
    if editor.is_some() {
        source_island = None;
    }
    VisualBlock {
        id: allocate_id(),
        kind,
        source_range,
        editable_runs,
        reveal_groups,
        marker_ranges,
        block_prefix,
        source_island: source_island.or(contains_html.then_some(VisualSourceIslandKind::Html)),
        editor,
    }
}

fn visual_block_editor(
    text: &str,
    block: &PreviewBlock,
    source_range: Range<usize>,
) -> Option<VisualBlockEditor> {
    match block {
        PreviewBlock::CodeBlock { language, .. } => {
            let (payload_range, info_range, opening_fence, closing_fence) =
                fenced_payload_ranges(text, source_range, '`', '~')?;
            // Diagram fences (e.g. `mermaid`) used to bail out here and fall
            // back to a complete source island, because Visual Edit had no way
            // to present a rendered diagram. The view layer now routes diagram
            // fences through `visual_diagram_editor`, which layers a rendered
            // image on top of this same payload editor — so the source-backed
            // editing contract is preserved while the diagram becomes visible.
            // Keep the editor's source ranges identical to a normal fence.
            let _ = language;
            Some(VisualBlockEditor::Code {
                opening_fence,
                payload: VisualEditorField {
                    kind: VisualEditorFieldKind::CodePayload,
                    source_range: payload_range,
                },
                info_range,
                closing_fence,
            })
        }
        PreviewBlock::MathBlock { delimiter, .. } => {
            let (payload_range, opening_delimiter, closing_delimiter) = match delimiter {
                MathDelimiter::DisplayDollar => dollar_math_payload_ranges(text, source_range)?,
                MathDelimiter::Fenced => {
                    let (payload, _, opening, closing) =
                        fenced_payload_ranges(text, source_range, '`', '~')?;
                    (payload, opening, closing)
                }
                MathDelimiter::InlineDollar => return None,
            };
            Some(VisualBlockEditor::Math {
                opening_delimiter,
                payload: VisualEditorField {
                    kind: VisualEditorFieldKind::MathPayload,
                    source_range: payload_range,
                },
                closing_delimiter,
            })
        }
        PreviewBlock::Table { rows, .. } => {
            let source = text.get(source_range.clone())?;
            let cell_ranges = table_cell_source_ranges(source)?;
            if cell_ranges.len() != rows.iter().map(Vec::len).sum::<usize>() {
                return None;
            }
            Some(VisualBlockEditor::Table {
                cells: cell_ranges
                    .into_iter()
                    .map(|cell| {
                        let source_range = source_range.start + cell.source_range.start
                            ..source_range.start + cell.source_range.end;
                        VisualTableCell {
                            row: cell.row,
                            column: cell.column,
                            field: VisualEditorField {
                                kind: VisualEditorFieldKind::TableCell {
                                    row: cell.row,
                                    column: cell.column,
                                },
                                source_range,
                            },
                        }
                    })
                    .collect(),
            })
        }
        _ => None,
    }
}

fn fenced_payload_ranges(
    text: &str,
    source_range: Range<usize>,
    first_marker: char,
    second_marker: char,
) -> Option<(
    Range<usize>,
    Option<Range<usize>>,
    Range<usize>,
    Range<usize>,
)> {
    let source = text.get(source_range.clone())?;
    let opening_end = source.find('\n').map_or(source.len(), |offset| offset + 1);
    let opening = source[..opening_end].trim_end_matches(['\r', '\n']);
    let indentation = opening.len() - opening.trim_start_matches(' ').len();
    if indentation > 3 {
        return None;
    }
    let opening_trimmed = &opening[indentation..];
    let marker = opening_trimmed.chars().next()?;
    if marker != first_marker && marker != second_marker {
        return None;
    }
    let marker_len = opening_trimmed
        .chars()
        .take_while(|ch| *ch == marker)
        .count();
    if marker_len < 3 {
        return None;
    }
    let info_local_start = indentation + marker_len;
    let info = &opening[info_local_start..];
    let leading = info.len() - info.trim_start().len();
    let trailing = info.len() - info.trim_end().len();
    let info_range = (leading + trailing < info.len()).then(|| {
        source_range.start + info_local_start + leading
            ..source_range.start + opening.len() - trailing
    });

    let opening_fence =
        source_range.start + indentation..source_range.start + indentation + marker_len;
    let mut closing = None;
    let mut offset = opening_end;
    for line_with_newline in source[opening_end..].split_inclusive('\n') {
        let line = line_with_newline.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim_start_matches(' ');
        let indent = line.len() - trimmed.len();
        let run = trimmed.chars().take_while(|ch| *ch == marker).count();
        if indent <= 3 && run >= marker_len && trimmed[run..].trim().is_empty() {
            closing = Some((offset, indent, run));
        }
        offset += line_with_newline.len();
    }
    let (closing_start, closing_indent, closing_len) = closing?;
    let closing_fence = source_range.start + closing_start + closing_indent
        ..source_range.start + closing_start + closing_indent + closing_len;
    Some((
        source_range.start + opening_end..source_range.start + closing_start,
        info_range,
        opening_fence,
        closing_fence,
    ))
}

fn dollar_math_payload_ranges(
    text: &str,
    source_range: Range<usize>,
) -> Option<(Range<usize>, Range<usize>, Range<usize>)> {
    let source = text.get(source_range.clone())?;
    let trimmed_end = source.trim_end_matches(['\r', '\n']);
    if !source.starts_with("$$") || !trimmed_end.ends_with("$$") || trimmed_end.len() < 4 {
        return None;
    }
    let closing_start = trimmed_end.len() - 2;
    let payload_start = if source[2..].starts_with("\r\n") {
        4
    } else if source[2..].starts_with('\n') {
        3
    } else {
        2
    };
    if payload_start > closing_start {
        return None;
    }
    Some((
        source_range.start + payload_start..source_range.start + closing_start,
        source_range.start..source_range.start + 2,
        source_range.start + closing_start..source_range.start + closing_start + 2,
    ))
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
        navigation: None,
        math: None,
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
    reference_definitions: &str,
) -> (Vec<VisualInlineRun>, Vec<VisualRevealGroup>, bool) {
    let source = &text[block_range.clone()];
    // Reference-style links resolve against document-scoped definitions that
    // live outside this block's slice. Appending them after a blank line lets
    // pulldown-cmark resolve the references without shifting any in-block
    // event offset: consumed definitions emit no events, and the loop below
    // stops at the first event past the block slice (e.g. from a malformed
    // line the parser could not consume as a definition).
    let owned_parse_input;
    let parse_input = if reference_definitions.is_empty() {
        source
    } else {
        let mut input = String::with_capacity(source.len() + 2 + reference_definitions.len());
        input.push_str(source);
        if !input.ends_with('\n') {
            input.push('\n');
        }
        input.push('\n');
        input.push_str(reference_definitions);
        owned_parse_input = input;
        &owned_parse_input
    };
    let mut runs = Vec::new();
    let mut candidates = Vec::new();
    let mut style = InlineStyle::default();
    let mut link_stack: Vec<(Option<Range<usize>>, String)> = Vec::new();
    let mut contains_html = false;

    for (event, relative_range) in Parser::new_ext(parse_input, markdown_options()).into_offset_iter()
    {
        if relative_range.start >= source.len() {
            break;
        }
        let event_range =
            block_range.start + relative_range.start..block_range.start + relative_range.end;
        let current_link = link_stack.last().cloned();
        let current_link_target = current_link.as_ref().and_then(|(range, _)| range.clone());
        let current_link_nav = current_link.as_ref().map(|(_, url)| {
            VisualNavigationTarget::Url(url.clone())
        });
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
                // pulldown-cmark reports a collapsed reference link's
                // (`[label][]`) tag range as `[label]` only; extend it to
                // cover the trailing `[]` so the reveal group exposes the
                // complete authored syntax.
                let mut link_range = event_range.clone();
                if text[link_range.clone()].ends_with(']')
                    && text[link_range.end..].starts_with("[]")
                {
                    link_range.end += 2;
                }
                let dest = dest_url.to_string();
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Link,
                    source_range: link_range.clone(),
                    link_target_range: find_link_target(text, &link_range, &dest),
                });
                link_stack.push((find_link_target(text, &link_range, &dest), dest));
            }
            Event::End(TagEnd::Link) => {
                link_stack.pop();
            }
            Event::Text(visible) => push_text_runs(
                &mut runs,
                &mut candidates,
                text,
                visible.as_ref(),
                event_range,
                style,
                current_link_target,
                current_link_nav,
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
                    current_link_target,
                    current_link_nav,
                    false,
                );
            }
            Event::SoftBreak | Event::HardBreak => push_run(
                &mut runs,
                text,
                "\n",
                event_range,
                style,
                current_link_target,
                current_link_nav,
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
                    current_link_target,
                    Some(VisualNavigationTarget::Footnote {
                        label: visible.to_string(),
                    }),
                    false,
                );
            }
            Event::InlineMath(visible) | Event::DisplayMath(visible) => {
                let delimiter = if text[event_range.clone()].starts_with("$$") {
                    MathDelimiter::DisplayDollar
                } else {
                    MathDelimiter::InlineDollar
                };
                let math_style = if delimiter == MathDelimiter::InlineDollar {
                    MathLayoutStyle::Text
                } else {
                    MathLayoutStyle::Display
                };
                let authored = text[event_range.clone()].to_string();
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Math,
                    source_range: event_range.clone(),
                    link_target_range: None,
                });
                runs.push(VisualInlineRun {
                    visible_text: authored.clone(),
                    source_range: event_range.clone(),
                    content_range: event_range.clone(),
                    style,
                    link_target_range: current_link_target,
                    navigation: current_link_nav,
                    math: Some(MathSource {
                        latex: visible.to_string(),
                        authored,
                        style: math_style,
                        delimiter,
                        source_range: event_range,
                    }),
                    conservative_fallback: false,
                });
            }
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
    navigation: Option<VisualNavigationTarget>,
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
            navigation,
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
            navigation,
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
            navigation.clone(),
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
    navigation: Option<VisualNavigationTarget>,
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
        navigation,
        math: None,
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
        VisualRevealKind::Math => {
            (source.starts_with("$$") && source.ends_with("$$") && source.len() >= 4)
                || (source.starts_with('$') && source.ends_with('$') && source.len() >= 2)
        }
        VisualRevealKind::Link => {
            if !source.starts_with('[') {
                return false;
            }
            if source.ends_with(')') && source.contains("](") {
                // Inline link: the destination is local, so it must map
                // byte-exactly inside the revealed range.
                candidate.link_target_range.as_ref().is_some_and(|target| {
                    target.start >= range.start
                        && target.end <= range.end
                        && text.is_char_boundary(target.start)
                        && text.is_char_boundary(target.end)
                })
            } else {
                // Reference-style link (full `[text][label]`, collapsed
                // `[label][]`, shortcut `[label]`): the use is local and the
                // parser-resolved tag range is exact by construction; the
                // destination lives in a definition block elsewhere in the
                // document, so no local target range is required.
                source.ends_with(']')
            }
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

    use super::{build_visual_projection, build_visual_projection_with_marked_range};
    use crate::{
        MarkdownDocument, MarkdownFormat, TableEdit, VisualBlockEditor, VisualBlockKind,
        VisualBlockPrefixKind, VisualCaretAffinity, VisualEditorFieldKind, VisualNavigationTarget,
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
        for source_offset in nested_range.filter(|offset| source.is_char_boundary(*offset)) {
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
    fn collapsed_marker_boundaries_expose_both_source_sides() {
        let source = "plain **世界** tail";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let version = doc.version();
        let block = &blocks[0];
        let cursor = source.find("plain").unwrap();
        let projection = build_visual_projection(source, block, cursor..cursor, cursor);
        let bold_display_start = projection.text.find("世界").unwrap();
        let candidates = projection.boundary_candidates(bold_display_start);

        assert!(candidates.is_ambiguous());
        assert_eq!(
            candidates.resolve(VisualCaretAffinity::Upstream),
            source.find("**").unwrap()
        );
        assert_eq!(
            candidates.resolve(VisualCaretAffinity::Downstream),
            source.find("世界").unwrap()
        );
        assert!(source.is_char_boundary(candidates.upstream_source));
        assert!(source.is_char_boundary(candidates.downstream_source));
        assert_eq!(doc.version(), version);
        assert!(Arc::ptr_eq(&blocks, &doc.visual_blocks_shared()));
    }

    #[test]
    fn marked_range_reveals_and_identity_maps_its_exact_syntax_group() {
        let source = "plain **世界** tail";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let marked_start = source.find("世界").unwrap();
        let marked = marked_start..marked_start + "世界".len();
        let projection = build_visual_projection_with_marked_range(
            source,
            &blocks[0],
            0..0,
            0,
            Some(marked.clone()),
        );

        assert_eq!(projection.text, source);
        assert_eq!(
            projection.display_range_for_source_range(marked.clone()),
            Some(marked)
        );
        assert_eq!(projection.revealed_source_ranges.len(), 1);
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
    fn math_projection_reveals_only_the_focused_complete_delimiter_group() {
        let source = "before $E=mc^2$ middle $a+b$ after";
        let doc = MarkdownDocument::from_text(source);
        let version = doc.version();
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        let first_start = source.find("$E=mc^2$").unwrap();
        let first_end = first_start + "$E=mc^2$".len();
        let second = source.find("$a+b$").unwrap();

        let focused = build_visual_projection(source, block, first_start..first_start, first_start);
        assert!(focused.text.contains("$E=mc^2$"));
        assert!(
            focused
                .revealed_source_ranges
                .contains(&(first_start..first_end))
        );
        assert!(
            !focused
                .revealed_source_ranges
                .iter()
                .any(|range| range.start == second)
        );

        let trailing = build_visual_projection(source, block, first_end..first_end, first_end);
        assert!(trailing.text.contains("$E=mc^2$"));
        assert_eq!(doc.version(), version);
        assert!(Arc::ptr_eq(&blocks, &doc.visual_blocks_shared()));
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
    fn complex_constructs_use_direct_editors_only_when_ranges_are_exact() {
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
                .any(|block| matches!(block.editor, Some(VisualBlockEditor::Code { .. })))
        );
        assert!(
            blocks
                .iter()
                .any(|block| matches!(block.editor, Some(VisualBlockEditor::Table { .. })))
        );
    }

    #[test]
    fn inline_math_run_keeps_exact_source_and_reveal_group() {
        let source = "前 $\\frac{a}{b}$ 后";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let run = blocks[0]
            .editable_runs
            .iter()
            .find(|run| run.math.is_some())
            .expect("semantic math run");
        let math = run.math.as_ref().unwrap();
        assert_eq!(math.latex, "\\frac{a}{b}");
        assert_eq!(math.authored, "$\\frac{a}{b}$");
        assert_eq!(&source[math.source_range.clone()], math.authored);
        assert!(blocks[0].reveal_groups.iter().any(|group| {
            group.kind == VisualRevealKind::Math
                && &source[group.source_range.clone()] == "$\\frac{a}{b}$"
        }));
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
    fn direct_code_metadata_preserves_exact_fence_info_and_payload_ranges() {
        let source = "~~~   rust extra\r\n  let 名称 = 1;\r\n\r\n~~~~\r\n";
        let doc = MarkdownDocument::from_text(source);
        let block = doc
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .expect("code block");
        let Some(VisualBlockEditor::Code {
            opening_fence,
            payload,
            info_range,
            closing_fence,
        }) = block.editor
        else {
            panic!("ordinary closed fence should have direct metadata");
        };
        assert_eq!(&source[opening_fence], "~~~");
        assert_eq!(&source[payload.source_range], "  let 名称 = 1;\r\n\r\n");
        assert_eq!(&source[info_range.expect("info range")], "rust extra");
        assert_eq!(&source[closing_fence], "~~~~");
        assert!(block.source_island.is_none());
    }

    #[test]
    fn unclosed_fences_remain_complete_source_islands() {
        // Unclosed fences cannot yield a payload range, so they fall back to
        // the complete source island regardless of language.
        let source = "```rust\nfn main() {}\n";
        let block = MarkdownDocument::from_text(source)
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .expect("code block");
        assert!(block.editor.is_none(), "unexpected editor for {source:?}");
        assert_eq!(block.source_island, Some(VisualSourceIslandKind::Code));
    }

    #[test]
    fn diagram_fence_carries_source_backed_payload_editor() {
        // A closed diagram fence now carries a `Code` payload editor (the same
        // source-backed affordance as any other fenced code block) so the view
        // layer can layer a rendered diagram on top. Source ranges stay exact.
        let source = "```mermaid\nflowchart LR\nA --> B\n```";
        let block = MarkdownDocument::from_text(source)
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .expect("code block");
        let VisualBlockEditor::Code {
            opening_fence,
            payload,
            info_range,
            closing_fence,
        } = block.editor.expect("diagram fence should have a payload editor")
        else {
            panic!("expected Code editor for diagram fence");
        };
        assert_eq!(&source[opening_fence], "```");
        assert_eq!(
            &source[payload.source_range.clone()],
            "flowchart LR\nA --> B\n"
        );
        assert_eq!(&source[info_range.expect("info range")], "mermaid");
        assert_eq!(&source[closing_fence], "```");
        // Payload lies strictly inside the block's source range, and the block
        // range fully covers the authored fence.
        assert!(block.source_range.start <= payload.source_range.start);
        assert!(payload.source_range.end <= block.source_range.end);
        assert_eq!(&source[block.source_range.clone()], source);
        // Editor-driven blocks drop the conservative source-island kind: the
        // payload editor is the source-backed path, matching math blocks.
        assert!(block.source_island.is_none());
    }

    #[test]
    fn direct_math_metadata_preserves_display_and_fenced_delimiters() {
        for (source, expected_payload) in [
            ("$$\n\\alpha + β\n$$", "\\alpha + β\n"),
            ("```math extra\n\\frac{甲}{2}\n```", "\\frac{甲}{2}\n"),
        ] {
            let block = MarkdownDocument::from_text(source)
                .visual_blocks()
                .into_iter()
                .find(|block| matches!(block.kind, VisualBlockKind::MathBlock { .. }))
                .expect("math block");
            let Some(VisualBlockEditor::Math {
                opening_delimiter,
                payload,
                closing_delimiter,
            }) = block.editor
            else {
                panic!("exact block math should have direct metadata: {source:?}");
            };
            assert!(matches!(&source[opening_delimiter], "$$" | "```"));
            assert_eq!(&source[payload.source_range], expected_payload);
            assert!(matches!(&source[closing_delimiter], "$$" | "```"));
            assert_eq!(payload.kind, VisualEditorFieldKind::MathPayload);
            assert!(block.source_island.is_none());
        }
    }

    #[test]
    fn inline_image_has_no_direct_editor_but_keeps_source_island() {
        let source = "![替代\\]文本](images/a\\)b.png '标题')";
        let block = MarkdownDocument::from_text(source)
            .visual_blocks()
            .remove(0);
        let VisualBlockKind::Image {
            alt,
            url,
            title,
        } = &block.kind
        else {
            panic!("expected image block, got {:?}", block.kind);
        };
        assert_eq!(alt, "替代]文本");
        assert_eq!(url, "images/a)b.png");
        assert_eq!(title.as_deref(), Some("标题"));
        assert!(block.editor.is_none(), "image should have no direct editor");
        assert_eq!(block.source_island, Some(VisualSourceIslandKind::Image));

        for ambiguous in [
            "![alt][asset]\n\n[asset]: image.png",
            "![alt](<image with spaces.png>)",
            "![alt](image.png\n \"title\")",
        ] {
            let image = MarkdownDocument::from_text(ambiguous)
                .visual_blocks()
                .into_iter()
                .find(|block| matches!(block.kind, VisualBlockKind::Image { .. }))
                .expect("image block");
            assert!(
                image.editor.is_none(),
                "unexpected editor for {ambiguous:?}"
            );
            assert_eq!(image.source_island, Some(VisualSourceIslandKind::Image));
        }
    }

    #[test]
    fn direct_table_metadata_covers_every_header_and_body_cell() {
        let source = "| 名称 | 值\\|文本 |\n| :--- | ---: |\n| 甲 | 2 |";
        let block = MarkdownDocument::from_text(source)
            .visual_blocks()
            .remove(0);
        let Some(VisualBlockEditor::Table { cells }) = block.editor else {
            panic!("exact GFM table should have direct metadata");
        };
        assert_eq!(cells.len(), 4);
        assert_eq!(&source[cells[1].field.source_range.clone()], "值\\|文本");
        assert!(cells.iter().all(|cell| {
            cell.field.kind
                == (VisualEditorFieldKind::TableCell {
                    row: cell.row,
                    column: cell.column,
                })
        }));
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
        for editor in ["code", "math", "table"] {
            assert!(
                blocks.iter().any(|block| match (editor, &block.editor) {
                    ("code", Some(VisualBlockEditor::Code { .. }))
                    | ("math", Some(VisualBlockEditor::Math { .. }))
                    | ("table", Some(VisualBlockEditor::Table { .. })) => true,
                    _ => false,
                }),
                "welcome document is missing direct {editor} metadata"
            );
        }

        assert!(editable_blocks.iter().any(|block| {
            matches!(block.kind, VisualBlockKind::ListItem { .. })
                && !block.editable_runs.is_empty()
                && block
                    .editable_runs
                    .iter()
                    .all(|run| !run.conservative_fallback)
        }));
    }

    #[test]
    fn collects_link_reference_definitions_outside_fenced_code() {
        let text = "# Title\n\n[alpha]: https://a.example\n\n```\n[beta]: https://b.example\n```\n\n   [gamma]: https://g.example \"Title\"\n\n[^note]: a footnote, not a link definition\n\n    [indented]: https://i.example\n";
        let definitions = super::collect_link_reference_definitions(text);
        assert!(definitions.contains("[alpha]: https://a.example"));
        assert!(definitions.contains("[gamma]: https://g.example \"Title\""));
        assert!(
            !definitions.contains("[beta]"),
            "definition inside a fenced code block must not be collected"
        );
        assert!(
            !definitions.contains("[^note]"),
            "footnote definitions are not link reference definitions"
        );
        assert!(
            !definitions.contains("[indented]"),
            "definitions indented four or more spaces are code blocks"
        );
    }

    #[test]
    fn reference_style_links_resolve_against_document_definitions() {
        let source = "See the [Markion repository][markion-repo], the [docs][], and [markion].\n\n[markion-repo]: https://github.com/willmove/markion\n[docs]: https://example.com/docs\n[markion]: https://markion.dev\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let paragraph = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .expect("reference-link paragraph");

        for label in ["Markion repository", "docs", "markion"] {
            let run = paragraph
                .editable_runs
                .iter()
                .find(|run| run.visible_text == label)
                .unwrap_or_else(|| panic!("missing rendered label run {label:?}"));
            assert!(
                !run.conservative_fallback,
                "reference link label {label:?} must stay visual"
            );
            assert!(
                paragraph
                    .editable_runs
                    .iter()
                    .all(|other| other.visible_text != "[markion-repo]"
                        && other.visible_text != "["
                        && other.visible_text != "]"),
                "reference brackets must not render as literal text"
            );
        }

        let link_groups = paragraph
            .reveal_groups
            .iter()
            .filter(|group| group.kind == VisualRevealKind::Link)
            .map(|group| &source[group.source_range.clone()])
            .collect::<Vec<_>>();
        assert!(link_groups.contains(&"[Markion repository][markion-repo]"));
        assert!(link_groups.contains(&"[docs][]"));
        assert!(link_groups.contains(&"[markion]"));

        // Every run and reveal group stays inside the paragraph block: the
        // appended definitions must not leak into the block's source mapping.
        assert!(
            paragraph
                .editable_runs
                .iter()
                .all(|run| run.source_range.end <= paragraph.source_range.end
                    && run.content_range.end <= paragraph.source_range.end)
        );
        assert!(
            paragraph
                .reveal_groups
                .iter()
                .all(|group| group.source_range.end <= paragraph.source_range.end)
        );

        // The rendered label is styled as a link even though its destination
        // lives in the definition block (no local target range).
        let projection =
            build_visual_projection(source, paragraph, 0..0, paragraph.source_range.start);
        assert!(projection.spans.iter().any(|span| {
            span.link && &projection.text[span.display_range.clone()] == "Markion repository"
        }));
    }

    #[test]
    fn definition_inside_fenced_code_does_not_create_link() {
        let source = "```\n[x]: https://x.example\n```\n\nSee [text][x].\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let paragraph = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .expect("paragraph after code block");
        assert!(
            paragraph
                .reveal_groups
                .iter()
                .all(|group| group.kind != VisualRevealKind::Link),
            "a definition inside a code fence must not resolve the reference"
        );
    }

    #[test]
    fn undefined_reference_stays_literal_text() {
        let source = "See [text][missing].\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let paragraph = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .expect("paragraph");
        assert!(
            paragraph
                .reveal_groups
                .iter()
                .all(|group| group.kind != VisualRevealKind::Link)
        );
        assert!(
            paragraph
                .editable_runs
                .iter()
                .any(|run| run.visible_text.contains("[text][missing]")
                    || run.visible_text == "text")
        );
    }

    #[test]
    fn notes_sample_footnotes_and_link_defs_render_without_islands() {
        let source = "## Notes\n\n\
Visit the [Markion project page](https://github.com/willmove/markion), or use the reference link below.[^links]\n\n\
Reference-style links work too: [Markion repository][markion-repo].\n\n\
[^links]: Links can point to project pages, files, and useful references.\n\n\
[markion-repo]: https://github.com/willmove/markion\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();

        let footnote_para = blocks
            .iter()
            .find(|block| {
                matches!(block.kind, VisualBlockKind::Paragraph)
                    && source[block.source_range.clone()].contains("[^links]")
            })
            .expect("paragraph with footnote reference");
        let footnote_run = footnote_para
            .editable_runs
            .iter()
            .find(|run| run.style.superscript && run.visible_text == "links")
            .expect("superscript footnote label run");
        assert!(
            !footnote_run.conservative_fallback,
            "footnote reference must stay visual"
        );
        assert!(
            footnote_para
                .editable_runs
                .iter()
                .all(|run| run.visible_text != "[" && run.visible_text != "]"),
            "footnote markers must not render as literal bracket runs"
        );

        let footnote_def = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::FootnoteDefinition { .. }))
            .expect("footnote definition block");
        assert!(footnote_def.source_island.is_none());
        assert!(
            source[footnote_def.source_range.clone()].contains("[^links]:"),
            "footnote definition range must cover the marker"
        );
        assert!(
            source[footnote_def.source_range.clone()]
                .contains("Links can point to project pages"),
            "footnote definition range must cover the body"
        );
        assert!(
            blocks.iter().all(|block| {
                !(matches!(block.kind, VisualBlockKind::Paragraph)
                    && source[block.source_range.clone()]
                        .starts_with("Links can point to project pages"))
            }),
            "footnote body must not also appear as an ordinary paragraph"
        );

        let link_def = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::ReferenceDefinition))
            .expect("link reference definition block");
        assert!(link_def.source_island.is_none());
        assert!(source[link_def.source_range.clone()].contains("[markion-repo]:"));

        let reference_para = blocks
            .iter()
            .find(|block| {
                matches!(block.kind, VisualBlockKind::Paragraph)
                    && source[block.source_range.clone()].contains("[Markion repository]")
            })
            .expect("reference-style link paragraph");
        assert!(
            reference_para
                .editable_runs
                .iter()
                .any(|run| run.visible_text == "Markion repository"
                    && !run.conservative_fallback),
            "reference-style link must remain resolved"
        );
        assert!(
            reference_para.editable_runs.iter().any(|run| {
                matches!(
                    &run.navigation,
                    Some(VisualNavigationTarget::Url(url))
                        if url == "https://github.com/willmove/markion"
                )
            }),
            "reference-style link must expose a URL navigation target"
        );
        assert!(
            footnote_para.editable_runs.iter().any(|run| {
                matches!(
                    &run.navigation,
                    Some(VisualNavigationTarget::Footnote { label }) if label == "links"
                )
            }),
            "footnote reference must expose a footnote navigation target"
        );
        assert!(
            footnote_para
                .editable_runs
                .iter()
                .any(|run| matches!(
                    &run.navigation,
                    Some(VisualNavigationTarget::Url(url))
                        if url == "https://github.com/willmove/markion"
                )),
            "inline link must expose a URL navigation target"
        );
    }
}
