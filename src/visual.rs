//! Source-ranged model used by the Visual Edit surface.

use std::ops::Range;

use pulldown_cmark::{Event, LinkType, Parser, Tag, TagEnd};

use crate::frontmatter::split_front_matter;
use crate::model::{
    AlertKind, InlineStyle, MathDelimiter, MathLayoutStyle, MathSource, PreviewBlock, VisualBlock,
    VisualBlockEditor, VisualBlockId, VisualBlockKind, VisualBlockPrefix, VisualBlockPrefixKind,
    VisualBoundaryCandidates, VisualCaretAffinity, VisualEditorField, VisualEditorFieldKind,
    VisualHtmlImage, VisualInlineRun, VisualNavigationTarget, VisualProjection,
    VisualProjectionSegment, VisualProjectionSpan, VisualQuoteContext, VisualQuoteGroupEdge,
    VisualRevealGroup, VisualRevealKind, VisualSourceIslandKind, VisualTableCell,
};
use crate::source_mapped::{is_closing_fence, is_reference_definition, opening_fence};
use crate::table::table_cell_source_ranges;
use crate::text_util::char_run_range;

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
                if segment.display_range.len() == segment.source_range.len() {
                    let exact = segment.source_range.start + display - segment.display_range.start;
                    return VisualBoundaryCandidates {
                        display_offset: display,
                        upstream_source: exact,
                        downstream_source: exact,
                    };
                }
                // Non-identity segment (a rendered atom such as `<br>` or a
                // table cell): an interior display position has no linear
                // source mapping, so it resolves ambiguously between the
                // atom's source edges instead of an interpolated offset that
                // could land mid-marker or off a UTF-8 boundary.
                return VisualBoundaryCandidates {
                    display_offset: display,
                    upstream_source: segment.source_range.start,
                    downstream_source: segment.source_range.end,
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

    /// Canonical source range selected by double-clicking the display text
    /// at `display`.
    ///
    /// The visible run (`text_util::char_run_range`) bounds the selection.
    /// Edges resolve to the innermost source side so hidden Markdown syntax
    /// at the selection's edges is excluded (`**word**` selects `word`), and
    /// a run strictly inside a rendered atom selects that atom's full source
    /// range. Returns `None` when no non-empty range resolves, leaving the
    /// caller on the plain caret-placement path.
    pub fn word_selection_range(&self, display: usize) -> Option<Range<usize>> {
        let run = char_run_range(&self.text, display);
        let atom_start = self
            .segments
            .iter()
            .find(|segment| {
                run.start > segment.display_range.start
                    && run.start < segment.display_range.end
                    && segment.display_range.len() != segment.source_range.len()
            })
            .map(|segment| segment.source_range.start);
        let atom_end = self
            .segments
            .iter()
            .find(|segment| {
                run.end > segment.display_range.start
                    && run.end < segment.display_range.end
                    && segment.display_range.len() != segment.source_range.len()
            })
            .map(|segment| segment.source_range.end);
        let start = atom_start.unwrap_or_else(|| self.boundary_candidates(run.start).downstream_source);
        let end = atom_end.unwrap_or_else(|| self.boundary_candidates(run.end).upstream_source);
        (start < end).then_some(start..end)
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
                matches!(
                    group.kind,
                    VisualRevealKind::Math | VisualRevealKind::HtmlImage
                ),
            )
        })
        .map(|group| group.source_range.clone())
        .collect::<Vec<_>>();
    if let Some(prefix) = &block.block_prefix {
        // Caret at prefix.end is the first title position (and the usual
        // caret for an empty heading/list). Empty payload rows also reveal
        // the whole prefix whenever the caret owns the block.
        let empty_payload = block.editable_runs.is_empty();
        let caret_on_empty_row = empty_payload
            && (block.source_range.contains(&source_cursor)
                || source_cursor == block.source_range.end
                || (!source_selection.is_empty()
                    && source_selection.start < block.source_range.end
                    && source_selection.end > block.source_range.start));
        if endpoint_is_active(&prefix.source_range, true) || caret_on_empty_row {
            revealed_source_ranges.push(prefix.source_range.clone());
        }
    }
    if let Some(quote) = &block.quote_context {
        revealed_source_ranges.extend(
            quote
                .marker_ranges
                .iter()
                .filter(|range| endpoint_is_active(range, false))
                .cloned(),
        );
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
                        range.start <= run.content_range.start && run.content_range.end <= range.end
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
use crate::parse::{
    ExtendedInlineKind, InlineHtmlStyleKind, InlineHtmlStyleTag, extended_inline_matches,
    parse_inline_html_image, parse_inline_html_style_tag, visual_markdown_options,
};

#[derive(Clone)]
struct VisualLeaf<'a> {
    block: &'a PreviewBlock,
    quote_group: Option<Range<usize>>,
    /// Alert kind of the enclosing quote group, when it opens with a
    /// `[!NOTE]`-style marker line.
    alert: Option<AlertKind>,
    /// Body-less alert group: no preview child exists, so the leaf renders
    /// the group's marker line itself as a callout title row.
    marker_only: bool,
}

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

    // Blockquotes are containers only. Their ordered leaf children become
    // visual rows; projecting the container as well would make parent and
    // child source ranges overlap and duplicate content on screen.
    let mut expanded: Vec<VisualLeaf<'_>> = Vec::new();
    for block in preview {
        match block {
            PreviewBlock::BlockQuote {
                children,
                alert,
                source_range,
            } => {
                if children.is_empty() && alert.is_some() {
                    expanded.push(VisualLeaf {
                        block,
                        quote_group: Some(source_range.clone()),
                        alert: *alert,
                        marker_only: true,
                    });
                }
                for child in children {
                    expanded.push(VisualLeaf {
                        block: child,
                        quote_group: Some(source_range.clone()),
                        alert: *alert,
                        marker_only: false,
                    });
                }
            }
            _ => expanded.push(VisualLeaf {
                block,
                quote_group: None,
                alert: None,
                marker_only: false,
            }),
        }
    }

    // A malformed preview, quote-group, or derived visual range must never
    // reach the string slicing below. Drop such leaves from semantic
    // projection; the coverage loop then represents their canonical bytes
    // through the ordinary source-backed gap fallback instead of panicking.
    expanded.retain(|leaf| {
        is_valid_source_range(text, leaf.block.source_range())
            && leaf
                .quote_group
                .as_ref()
                .is_none_or(|group| is_valid_source_range(text, group))
    });

    let source_ranges = expanded
        .iter()
        .map(|leaf| {
            if leaf.marker_only {
                return leaf
                    .quote_group
                    .clone()
                    .unwrap_or_else(|| leaf.block.source_range().clone());
            }
            let range = visual_block_source_range(text, leaf.block);
            leaf.quote_group.as_ref().map_or(range.clone(), |group| {
                quoted_leaf_source_range(text, range, group)
            })
        })
        .collect::<Vec<_>>();
    let (expanded, mut source_ranges): (Vec<_>, Vec<_>) = expanded
        .into_iter()
        .zip(source_ranges)
        .filter(|(_, range)| is_valid_source_range(text, range))
        .unzip();
    for index in 0..source_ranges.len().saturating_sub(1) {
        let nested_list_start = match (expanded[index].block, expanded[index + 1].block) {
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

    // A paragraph/heading that still swallows nested Markdown images would
    // overlap those Image leaves. Split the parent into disjoint prose
    // slices around each contained image before the coverage loop, so the
    // overlap guard does not force-mark the images Unsupported.
    let (expanded, source_ranges) = partition_prose_around_nested_images(expanded, source_ranges);

    let mut covered_until = blocks.last().map_or(0, |block| block.source_range.end);
    for (leaf, range) in expanded.iter().zip(source_ranges) {
        if range.start > covered_until {
            let gap_range = covered_until..range.start;
            // An alert group's leading marker line (`> [!NOTE]`) is block
            // structure: no inline event owns its bytes, so instead of the
            // generic gap fallback it becomes the group's callout title row.
            // The gap containing the group start is the leading one (gaps
            // are disjoint), and the marker line is the line holding that
            // start — including any indentation before the `>`. Only that
            // line belongs to the title: lines before it (e.g. the blank
            // line after a previous block — the gap starts at the previous
            // block's end, which may be outside the quote group) and after
            // it (e.g. a bare `>` separator) keep the normal gap handling.
            let alert_title = leaf
                .quote_group
                .as_ref()
                .filter(|group| gap_range.contains(&group.start))
                .zip(leaf.alert)
                .map(|(group, kind)| {
                    let marker_line_start = text[..group.start]
                        .rfind('\n')
                        .map_or(gap_range.start, |index| index + 1)
                        .max(gap_range.start);
                    let marker_line_end = text[marker_line_start..gap_range.end]
                        .find('\n')
                        .map_or(gap_range.end, |relative| marker_line_start + relative + 1)
                        .min(gap_range.end);
                    (marker_line_start..marker_line_end, kind, (*group).clone())
                });
            match alert_title {
                Some((marker_range, kind, group)) => {
                    if marker_range.start > gap_range.start {
                        blocks.push(gap_block(
                            text,
                            gap_range.start..marker_range.start,
                            None,
                            &mut allocate_id,
                        ));
                    }
                    blocks.push(callout_title_block(
                        text,
                        marker_range.clone(),
                        kind,
                        &group,
                        &mut allocate_id,
                    ));
                    if marker_range.end < gap_range.end {
                        blocks.push(gap_block(
                            text,
                            marker_range.end..gap_range.end,
                            Some(&group),
                            &mut allocate_id,
                        ));
                    }
                }
                None => {
                    let quote_group = leaf.quote_group.as_ref().filter(|group| {
                        gap_range.start >= group.start && gap_range.end <= group.end
                    });
                    blocks.push(gap_block(text, gap_range, quote_group, &mut allocate_id));
                }
            }
        }

        if leaf.marker_only {
            let kind = leaf.alert.expect("marker-only leaf carries the alert kind");
            let group = leaf.quote_group.clone().unwrap_or_else(|| range.clone());
            blocks.push(callout_title_block(
                text,
                range.clone(),
                kind,
                &group,
                &mut allocate_id,
            ));
            covered_until = covered_until.max(range.end);
            continue;
        }

        let quote_context = leaf.quote_group.as_ref().map(|group| {
            quote_context_for_row(
                text,
                range.clone(),
                leaf.block.source_range().clone(),
                group.clone(),
            )
        });
        let mut block = visual_block_from_preview(
            text,
            leaf.block,
            range.clone(),
            quote_context,
            &reference_definitions,
            &mut allocate_id,
        );
        let overlaps_same_quote_group = block.quote_context.as_ref().is_some_and(|quote| {
            blocks
                .last()
                .and_then(|previous| previous.quote_context.as_ref())
                .is_some_and(|previous| previous.group_source_range == quote.group_source_range)
                && block.source_range.start < covered_until
        });
        debug_assert!(
            !overlaps_same_quote_group,
            "visual leaves overlap inside one quote group: covered through {covered_until}, next={:?}",
            block.source_range
        );
        if block.source_range.start < covered_until {
            block.source_island = Some(VisualSourceIslandKind::Unsupported);
        }
        covered_until = covered_until.max(range.end);
        blocks.push(block);
    }

    if covered_until < text.len() {
        blocks.push(gap_block(
            text,
            covered_until..text.len(),
            None,
            &mut allocate_id,
        ));
    }
    assign_quote_group_edges(&mut blocks);
    blocks
}

fn is_image_partition_parent(leaf: &VisualLeaf<'_>) -> bool {
    !leaf.marker_only
        && matches!(
            leaf.block,
            PreviewBlock::Paragraph { .. } | PreviewBlock::Heading { .. }
        )
}

fn image_range_contained_in_parent(image: &Range<usize>, parent: &Range<usize>) -> bool {
    image.start >= parent.start && image.end <= parent.end
}

fn later_partition_parent_owns_image(
    expanded: &[VisualLeaf<'_>],
    source_ranges: &[Range<usize>],
    image_index: usize,
) -> bool {
    let image_range = &source_ranges[image_index];
    expanded
        .iter()
        .zip(source_ranges.iter())
        .skip(image_index + 1)
        .any(|(leaf, parent_range)| {
            is_image_partition_parent(leaf)
                && image_range_contained_in_parent(image_range, parent_range)
        })
}

fn partition_prose_around_nested_images<'a>(
    expanded: Vec<VisualLeaf<'a>>,
    source_ranges: Vec<Range<usize>>,
) -> (Vec<VisualLeaf<'a>>, Vec<Range<usize>>) {
    let mut out_leaves = Vec::with_capacity(expanded.len());
    let mut out_ranges = Vec::with_capacity(source_ranges.len());
    let mut consumed = vec![false; expanded.len()];
    let mut index = 0;
    while index < expanded.len() {
        let leaf = &expanded[index];
        let parent_range = source_ranges[index].clone();

        if matches!(leaf.block, PreviewBlock::Image { .. }) {
            if consumed[index]
                || later_partition_parent_owns_image(&expanded, &source_ranges, index)
            {
                index += 1;
                continue;
            }
            out_leaves.push(leaf.clone());
            out_ranges.push(parent_range);
            index += 1;
            continue;
        }

        if !is_image_partition_parent(leaf) {
            out_leaves.push(leaf.clone());
            out_ranges.push(parent_range);
            index += 1;
            continue;
        }

        let mut nested: Vec<(usize, Range<usize>)> = expanded
            .iter()
            .zip(source_ranges.iter())
            .enumerate()
            .filter(|(look, (nested_leaf, image_range))| {
                !consumed[*look]
                    && matches!(nested_leaf.block, PreviewBlock::Image { .. })
                    && image_range_contained_in_parent(image_range, &parent_range)
            })
            .map(|(look, (_, image_range))| (look, image_range.clone()))
            .collect();
        nested.sort_by_key(|(_, image_range)| image_range.start);

        if nested.is_empty() {
            out_leaves.push(leaf.clone());
            out_ranges.push(parent_range);
            index += 1;
            continue;
        }

        let mut cursor = parent_range.start;
        for (image_index, image_range) in &nested {
            if image_range.start > cursor {
                out_leaves.push(leaf.clone());
                out_ranges.push(cursor..image_range.start);
            }
            let mut image_leaf = expanded[*image_index].clone();
            // Images extracted from a quote are top-level preview blocks, so
            // copy the parent's quote group onto the image row.
            if image_leaf.quote_group.is_none() {
                image_leaf.quote_group = leaf.quote_group.clone();
                image_leaf.alert = leaf.alert;
            }
            out_leaves.push(image_leaf);
            out_ranges.push(image_range.clone());
            consumed[*image_index] = true;
            cursor = cursor.max(image_range.end);
        }
        if cursor < parent_range.end {
            out_leaves.push(leaf.clone());
            out_ranges.push(cursor..parent_range.end);
        }
        index += 1;
    }
    (out_leaves, out_ranges)
}

/// A range may index the canonical text only when it is ordered, in-bounds,
/// and both endpoints land on UTF-8 character boundaries.
fn is_valid_source_range(text: &str, range: &Range<usize>) -> bool {
    range.start <= range.end
        && range.end <= text.len()
        && text.is_char_boundary(range.start)
        && text.is_char_boundary(range.end)
}

fn quoted_leaf_source_range(
    text: &str,
    mut range: Range<usize>,
    group: &Range<usize>,
) -> Range<usize> {
    let line_start = text[..range.start].rfind('\n').map_or(0, |index| index + 1);
    range.start = line_start.max(group.start);
    range.end = range.end.min(group.end);
    range
}

fn quote_prefix_on_line(
    text: &str,
    line_start: usize,
    line_end: usize,
) -> Option<(Range<usize>, usize)> {
    let line = &text[line_start..line_end];
    let bytes = line.as_bytes();
    let mut cursor = bytes
        .iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let mut depth = 0;
    while bytes.get(cursor) == Some(&b'>') {
        depth += 1;
        cursor += 1;
        if matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
            cursor += 1;
        }
    }
    (depth > 0).then_some((line_start..line_start + cursor, depth))
}

fn quote_context_for_row(
    text: &str,
    row_range: Range<usize>,
    leaf_source_range: Range<usize>,
    group_source_range: Range<usize>,
) -> VisualQuoteContext {
    let mut marker_ranges = Vec::new();
    let mut depth = 0;
    let mut line_start = row_range.start;
    while line_start < row_range.end {
        let line_end = text[line_start..row_range.end]
            .find('\n')
            .map_or(row_range.end, |relative| line_start + relative);
        if let Some((prefix, line_depth)) = quote_prefix_on_line(text, line_start, line_end) {
            marker_ranges.push(prefix);
            depth = depth.max(line_depth);
        }
        if line_end >= row_range.end {
            break;
        }
        line_start = line_end + 1;
    }
    VisualQuoteContext {
        depth: depth.max(1),
        marker_ranges,
        leaf_source_range,
        group_source_range,
        edge: VisualQuoteGroupEdge::Middle,
    }
}

fn quote_gap_is_structural_only(slice: &str) -> bool {
    slice.lines().all(|line| {
        let trimmed = line.trim_start_matches([' ', '\t']);
        if trimmed.is_empty() {
            return true;
        }
        let mut rest = trimmed;
        let mut saw_quote = false;
        while let Some(after) = rest.strip_prefix('>') {
            saw_quote = true;
            rest = after.trim_start_matches([' ', '\t']);
        }
        saw_quote && rest.is_empty()
    })
}

fn assign_quote_group_edges(blocks: &mut [VisualBlock]) {
    let mut index = 0;
    while index < blocks.len() {
        let Some(group) = blocks[index]
            .quote_context
            .as_ref()
            .map(|quote| quote.group_source_range.clone())
        else {
            index += 1;
            continue;
        };
        let start = index;
        while index + 1 < blocks.len()
            && blocks[index + 1]
                .quote_context
                .as_ref()
                .is_some_and(|quote| quote.group_source_range == group)
        {
            index += 1;
        }
        let end = index;
        for (offset, block) in blocks[start..=end].iter_mut().enumerate() {
            let quote = block.quote_context.as_mut().expect("quote group member");
            quote.edge = match (offset == 0, start + offset == end) {
                (true, true) => VisualQuoteGroupEdge::Only,
                (true, false) => VisualQuoteGroupEdge::First,
                (false, true) => VisualQuoteGroupEdge::Last,
                (false, false) => VisualQuoteGroupEdge::Middle,
            };
        }
        index += 1;
    }
}

fn gap_block(
    text: &str,
    range: Range<usize>,
    quote_group: Option<&Range<usize>>,
    allocate_id: &mut impl FnMut() -> VisualBlockId,
) -> VisualBlock {
    let slice = &text[range.clone()];
    let standalone_quote_gap = quote_group.is_none()
        && quote_gap_is_structural_only(slice)
        && slice
            .lines()
            .any(|line| line.trim_start_matches([' ', '\t']).starts_with('>'));
    if slice.trim().is_empty()
        || standalone_quote_gap
        || (quote_group.is_some() && quote_gap_is_structural_only(slice))
    {
        let quote_group = quote_group
            .cloned()
            .or_else(|| standalone_quote_gap.then(|| range.clone()));
        let quote_context = quote_group
            .map(|group| quote_context_for_row(text, range.clone(), range.clone(), group));
        VisualBlock {
            id: allocate_id(),
            kind: VisualBlockKind::Whitespace,
            source_range: range,
            editable_runs: Vec::new(),
            reveal_groups: Vec::new(),
            marker_ranges: Vec::new(),
            block_prefix: None,
            height_signature: Some(slice.bytes().filter(|byte| *byte == b'\n').count() as u32),
            quote_context,
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
            height_signature: None,
            quote_context: None,
            source_island: None,
            editor: None,
        }
    } else {
        source_island(range, VisualSourceIslandKind::Unsupported, allocate_id)
    }
}

fn callout_title_block(
    text: &str,
    range: Range<usize>,
    kind: AlertKind,
    group: &Range<usize>,
    allocate_id: &mut impl FnMut() -> VisualBlockId,
) -> VisualBlock {
    // Every byte of the marker line is structural and behaves like a quote
    // prefix: one marker range covers `> [!NOTE]` up to (not including) the
    // trailing newline, so any caret inside the line reveals it verbatim
    // instead of revealing fragments. The row itself carries no editable
    // runs; the trailing newline stays unowned like on whitespace rows.
    let mut quote_context =
        quote_context_for_row(text, range.clone(), range.clone(), group.clone());
    let line_end = text[range.clone()]
        .find('\n')
        .map_or(range.end, |relative| range.start + relative);
    quote_context.marker_ranges = vec![range.start..line_end];
    VisualBlock {
        id: allocate_id(),
        kind: VisualBlockKind::CalloutTitle { kind },
        source_range: range,
        editable_runs: Vec::new(),
        reveal_groups: Vec::new(),
        marker_ranges: Vec::new(),
        block_prefix: None,
        height_signature: None,
        quote_context: Some(quote_context),
        source_island: None,
        editor: None,
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
        height_signature: None,
        quote_context: None,
        source_island: Some(kind),
        editor: None,
    }
}

fn visual_block_from_preview(
    text: &str,
    block: &PreviewBlock,
    source_range: Range<usize>,
    quote_context: Option<VisualQuoteContext>,
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
        PreviewBlock::Html { html, .. } => (VisualBlockKind::Html { html: html.clone() }, None),
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

    let block_prefix = block_prefix(text, &kind, source_range.clone(), quote_context.as_ref());
    let inline_source_range = if matches!(kind, VisualBlockKind::ListItem { .. }) {
        block_prefix.as_ref().map_or_else(
            || source_range.clone(),
            |prefix| prefix.source_range.end..source_range.end,
        )
    } else {
        quote_context
            .as_ref()
            .and_then(|quote| quote.marker_ranges.first())
            .filter(|prefix| prefix.start == source_range.start)
            .map_or_else(
                || source_range.clone(),
                |prefix| prefix.end..source_range.end,
            )
    };
    let (mut editable_runs, reveal_groups, _) = if matches!(kind, VisualBlockKind::Html { .. }) {
        // Rendered HTML blocks present through the HTML-parts pipeline,
        // not the inline projection. Keeping their runs empty preserves
        // the focused source-island affordance (the empty-runs gate in
        // the view layer) for editing raw HTML.
        (Vec::new(), Vec::new(), false)
    } else {
        inline_runs(text, inline_source_range, reference_definitions)
    };
    append_trailing_horizontal_whitespace_run(
        text,
        &source_range,
        block_prefix.as_ref(),
        &reveal_groups,
        &mut editable_runs,
    );
    if quote_context.is_some() {
        synthesize_quote_softbreak_runs(text, &source_range, &mut editable_runs);
    }
    let marker_ranges = marker_ranges(source_range.clone(), &editable_runs);
    let editor = visual_block_editor(text, block, source_range.clone());
    if editor.is_some() {
        source_island = None;
    }
    // A rendered HTML block (VisualBlockKind::Html) is shown via the
    // HTML-parts pipeline, not as a raw-source box, so it must never carry
    // a source island regardless of any inline-HTML detected here. Prose
    // blocks whose inline HTML is solely complete `<img>` tags render those
    // as image atoms, so they keep their normal presentation too.
    let source_island = if matches!(kind, VisualBlockKind::Html { .. }) {
        None
    } else {
        source_island
    };
    VisualBlock {
        id: allocate_id(),
        kind,
        source_range,
        editable_runs,
        reveal_groups,
        marker_ranges,
        block_prefix,
        height_signature: None,
        quote_context,
        source_island,
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
            let info = info_range.as_ref().map_or_else(
                || opening_fence.end..opening_fence.end,
                |range| {
                    // `info_range` covers the already-trimmed info string, so
                    // the first token ends at its first whitespace.
                    let slice = &text[range.clone()];
                    let token_end = slice.find(char::is_whitespace).unwrap_or(slice.len());
                    range.start..range.start + token_end
                },
            );
            Some(VisualBlockEditor::Code {
                opening_fence,
                payload: VisualEditorField {
                    kind: VisualEditorFieldKind::CodePayload,
                    source_range: payload_range,
                },
                info_range,
                info: VisualEditorField {
                    kind: VisualEditorFieldKind::CodeInfo,
                    source_range: info,
                },
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
        PreviewBlock::Image { .. } => {
            // Conservative whole-span proof: the block must be exactly one
            // complete inline image whose label and destination bounds
            // resolve without guessing. The closing `)` must be unescaped
            // (otherwise the authored title could swallow it) and, outside an
            // angle destination, no unescaped `)` may appear before it.
            // Reference-style and multiline forms end with `]` or fail the
            // scan and keep `editor: None` (today's island fallback) instead
            // of a guessed payload range.
            let authored = text.get(source_range.clone())?;
            if !authored.starts_with("![")
                || authored.len() < 6
                || authored.contains(['\n', '\r'])
                || !crate::inline_edit::find_unescaped(authored, 0, b')')
                    .is_some_and(|close| close == authored.len() - 1)
            {
                return None;
            }
            let destination = crate::inline_edit::authored_image_destination_range(authored)?;
            let destination_inner = &authored[destination.clone()];
            if !destination_inner.starts_with('<')
                && crate::inline_edit::find_unescaped(destination_inner, 0, b')').is_some()
            {
                return None;
            }
            Some(VisualBlockEditor::Image {
                payload: VisualEditorField {
                    kind: VisualEditorFieldKind::ImageSource,
                    source_range,
                },
            })
        }
        PreviewBlock::Html { .. } => Some(VisualBlockEditor::Html {
            payload: VisualEditorField {
                kind: VisualEditorFieldKind::HtmlSource,
                source_range,
            },
        }),
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
    if closing.is_none() {
        // A fence nested inside a list item retains the list's indentation on
        // its payload and closing-fence lines (pulldown-cmark reports the
        // block range starting at the opening backticks, so the opening line
        // passes the strict check above while the indented closing line does
        // not). Measure the payload's common indentation and accept a closing
        // fence at up to that depth. pulldown-cmark already fixed this block's
        // extent, so no payload line can look like a valid closing fence.
        // Gate on the opening fence itself being indented 4+ in the document:
        // a top-level fence's payload line that merely looks like an indented
        // fence (e.g. an unclosed fence) must not be misread as the closing.
        let opening_line_indent = text[..source_range.start]
            .rsplit('\n')
            .next()
            .unwrap_or("")
            .bytes()
            .take_while(|byte| *byte == b' ')
            .count();
        let payload = &source[opening_end..];
        let payload_indent = payload
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start_matches(' ').len())
            .min();
        if opening_line_indent >= 4
            && let Some(payload_indent) = payload_indent
            && payload_indent > 3
        {
            let mut offset = opening_end;
            for line_with_newline in source[opening_end..].split_inclusive('\n') {
                let line = line_with_newline.trim_end_matches(['\r', '\n']);
                let trimmed = line.trim_start_matches(' ');
                let indent = line.len() - trimmed.len();
                let run = trimmed.chars().take_while(|ch| *ch == marker).count();
                if indent <= payload_indent && run >= marker_len && trimmed[run..].trim().is_empty()
                {
                    closing = Some((offset, indent, run));
                }
                offset += line_with_newline.len();
            }
        }
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
        html_image: None,
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

/// Number of distinct kinds in [`InlineHtmlStyleKind`].
const HTML_STYLE_KIND_COUNT: usize = 7;

/// One open supported inline-HTML style tag awaiting its close.
struct HtmlStyleFrame {
    kind: InlineHtmlStyleKind,
    open_range: Range<usize>,
}

fn html_style_index(kind: InlineHtmlStyleKind) -> usize {
    match kind {
        InlineHtmlStyleKind::Emphasis => 0,
        InlineHtmlStyleKind::Strong => 1,
        InlineHtmlStyleKind::Strikethrough => 2,
        InlineHtmlStyleKind::Code => 3,
        InlineHtmlStyleKind::Highlight => 4,
        InlineHtmlStyleKind::Subscript => 5,
        InlineHtmlStyleKind::Superscript => 6,
    }
}

/// Composes the Markdown tag style with the open inline-HTML style depths.
fn with_html_style(base: InlineStyle, html_depths: &[usize; HTML_STYLE_KIND_COUNT]) -> InlineStyle {
    InlineStyle {
        italic: base.italic || html_depths[0] > 0,
        bold: base.bold || html_depths[1] > 0,
        strikethrough: base.strikethrough || html_depths[2] > 0,
        code: base.code || html_depths[3] > 0,
        highlight: base.highlight || html_depths[4] > 0,
        subscript: base.subscript || html_depths[5] > 0,
        superscript: base.superscript || html_depths[6] > 0,
        ..base
    }
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
    let mut markdown_style = InlineStyle::default();
    // Depth counters for supported inline-HTML style tags, kept separate from
    // `markdown_style` so an HTML pair and a Markdown pair nesting each other
    // (e.g. `<em>a *b* c</em>`) cannot clear each other's flags on close.
    let mut html_depths = [0usize; HTML_STYLE_KIND_COUNT];
    let mut html_style_stack: Vec<HtmlStyleFrame> = Vec::new();
    let mut html_element_ranges: Vec<Range<usize>> = Vec::new();
    let mut html_pairing_failed = false;
    let mut link_stack: Vec<(Option<Range<usize>>, String)> = Vec::new();
    let mut contains_non_image_html = false;
    // Relative end of the last leaf (content) event. pulldown-cmark resolves
    // `\X` escapes and merges the escaped character into the following Text
    // event while leaving the backslash byte uncovered between events, so an
    // uncovered one-byte `\` gap before a Text event starting with ASCII
    // punctuation marks an escape whose character must be claimed separately.
    let mut previous_leaf_end = 0usize;
    let mut image_open: Option<(String, Option<String>, Range<usize>, String)> = None;

    for (event, relative_range) in
        Parser::new_ext(parse_input, visual_markdown_options()).into_offset_iter()
    {
        if relative_range.start >= source.len() {
            break;
        }
        let event_range =
            block_range.start + relative_range.start..block_range.start + relative_range.end;
        let current_link = link_stack.last().cloned();
        let current_link_target = current_link.as_ref().and_then(|(range, _)| range.clone());
        let current_link_nav = current_link
            .as_ref()
            .map(|(_, url)| VisualNavigationTarget::Url(url.clone()));
        let is_leaf_event = matches!(
            event,
            Event::Text(_)
                | Event::Code(_)
                | Event::SoftBreak
                | Event::HardBreak
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
        );
        let style = with_html_style(markdown_style, &html_depths);
        match event {
            Event::Start(Tag::Strong) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Strong,
                    source_range: event_range.clone(),
                    link_target_range: None,
                });
                markdown_style.bold = true;
            }
            Event::End(TagEnd::Strong) => {
                markdown_style.bold = false;
            }
            Event::Start(Tag::Emphasis) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Emphasis,
                    source_range: event_range.clone(),
                    link_target_range: None,
                });
                markdown_style.italic = true;
            }
            Event::End(TagEnd::Emphasis) => {
                markdown_style.italic = false;
            }
            Event::Start(Tag::Strikethrough) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Strikethrough,
                    source_range: event_range.clone(),
                    link_target_range: None,
                });
                markdown_style.strikethrough = true;
            }
            Event::End(TagEnd::Strikethrough) => {
                markdown_style.strikethrough = false;
            }
            Event::Start(Tag::Link {
                dest_url,
                link_type,
                ..
            }) => {
                // pulldown-cmark reports a collapsed reference link's
                // (`[label][]`) tag range as `[label]` only; extend it to
                // cover the trailing `[]` so the reveal group exposes the
                // complete authored syntax.
                let mut link_range = event_range.clone();
                if !matches!(link_type, LinkType::Autolink | LinkType::Email)
                    && text[link_range.clone()].ends_with(']')
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
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                image_open = Some((
                    dest_url.to_string(),
                    (!title.is_empty()).then(|| title.to_string()),
                    event_range,
                    String::new(),
                ));
            }
            Event::End(TagEnd::Image) => {
                if let Some((url, title, start_range, alt)) = image_open.take() {
                    let range = if start_range.end > start_range.start {
                        start_range.start..start_range.end.max(event_range.end)
                    } else {
                        event_range.clone()
                    };
                    let range = range.start.max(block_range.start)..range.end.min(block_range.end);
                    if range.start < range.end
                        && text.is_char_boundary(range.start)
                        && text.is_char_boundary(range.end)
                    {
                        candidates.push(RevealCandidate {
                            kind: VisualRevealKind::HtmlImage,
                            source_range: range.clone(),
                            link_target_range: None,
                        });
                        runs.push(VisualInlineRun {
                            visible_text: text[range.clone()].to_string(),
                            source_range: range.clone(),
                            content_range: range.clone(),
                            style,
                            link_target_range: current_link_target,
                            navigation: current_link_nav,
                            math: None,
                            html_image: Some(VisualHtmlImage {
                                alt: alt.trim().to_string(),
                                url,
                                title,
                                width: None,
                                height: None,
                            }),
                            conservative_fallback: false,
                        });
                        previous_leaf_end = previous_leaf_end
                            .max((range.end - block_range.start).min(source.len()));
                    }
                }
            }
            Event::Text(visible) => {
                if let Some((_, _, _, alt)) = image_open.as_mut() {
                    alt.push_str(visible.as_ref());
                } else {
                    let mut visible_text = visible.as_ref();
                    let mut text_start = relative_range.start;
                    // Claim a leading escaped character merged into this Text
                    // event: the backslash gap sits uncovered before the event and
                    // the first source byte is the escaped ASCII punctuation.
                    if relative_range.start == previous_leaf_end + 1
                        && source.as_bytes().get(previous_leaf_end) == Some(&b'\\')
                        && source
                            .as_bytes()
                            .get(relative_range.start)
                            .is_some_and(|byte| byte.is_ascii_punctuation())
                        && visible_text
                            .starts_with(&source[relative_range.start..relative_range.start + 1])
                    {
                        candidates.push(RevealCandidate {
                            kind: VisualRevealKind::Escape,
                            source_range: block_range.start + previous_leaf_end
                                ..block_range.start + relative_range.start + 1,
                            link_target_range: None,
                        });
                        push_run(
                            &mut runs,
                            text,
                            &source[relative_range.start..relative_range.start + 1],
                            block_range.start + relative_range.start
                                ..block_range.start + relative_range.start + 1,
                            style,
                            current_link_target.clone(),
                            current_link_nav.clone(),
                            false,
                        );
                        visible_text = &visible_text[1..];
                        text_start = relative_range.start + 1;
                    }
                    if text_start < relative_range.end {
                        push_text_runs(
                            &mut runs,
                            &mut candidates,
                            text,
                            visible_text,
                            block_range.start + text_start..block_range.start + relative_range.end,
                            style,
                            current_link_target,
                            current_link_nav,
                        );
                    }
                }
            }
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
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, _, _, alt)) = image_open.as_mut() {
                    alt.push(' ');
                } else {
                    push_run(
                        &mut runs,
                        text,
                        "\n",
                        event_range,
                        style,
                        current_link_target,
                        current_link_nav,
                        false,
                    );
                }
            }
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
                    html_image: None,
                    conservative_fallback: false,
                });
            }
            // A complete inline `<img …>` tag is the one inline-HTML form the
            // projection maps byte-for-byte: it becomes an image run revealed
            // as its exact authored source when focused. A narrow subset of
            // style tags and `<br>` renders as hidden markers with styled
            // content; anything else keeps the whole-block source-island
            // fallback. The block-level `Html` event also carries a lone
            // leading tag when a list item's or quote leaf's slice starts
            // with the image, so both event kinds run through the same exact
            // recognizers.
            Event::Html(_) | Event::InlineHtml(_) => {
                let authored = text[event_range.clone()].to_string();
                if let Some(image) = parse_inline_html_image(&authored) {
                    candidates.push(RevealCandidate {
                        kind: VisualRevealKind::HtmlImage,
                        source_range: event_range.clone(),
                        link_target_range: None,
                    });
                    runs.push(VisualInlineRun {
                        visible_text: authored,
                        source_range: event_range.clone(),
                        content_range: event_range,
                        style,
                        link_target_range: current_link_target,
                        navigation: current_link_nav,
                        math: None,
                        html_image: Some(image),
                        conservative_fallback: false,
                    });
                } else if let Some(tag) = parse_inline_html_style_tag(&authored) {
                    match tag {
                        InlineHtmlStyleTag::LineBreak => {
                            // One display newline mapped atomically onto the
                            // tag bytes, exactly like soft/hard breaks except
                            // that caret resolution is boundary-only.
                            candidates.push(RevealCandidate {
                                kind: VisualRevealKind::InlineHtml,
                                source_range: event_range.clone(),
                                link_target_range: None,
                            });
                            runs.push(VisualInlineRun {
                                visible_text: "\n".to_string(),
                                source_range: event_range.clone(),
                                content_range: event_range,
                                style,
                                link_target_range: current_link_target,
                                navigation: current_link_nav,
                                math: None,
                                html_image: None,
                                conservative_fallback: false,
                            });
                        }
                        InlineHtmlStyleTag::Open { kind } => {
                            html_style_stack.push(HtmlStyleFrame {
                                kind,
                                open_range: event_range.clone(),
                            });
                            html_depths[html_style_index(kind)] += 1;
                            // The reveal candidate is registered when the
                            // matching close arrives so it spans the complete
                            // element source.
                        }
                        InlineHtmlStyleTag::Close { kind } => {
                            if html_style_stack
                                .last()
                                .is_some_and(|frame| frame.kind == kind)
                            {
                                let frame = html_style_stack.pop().expect("checked top frame");
                                html_depths[html_style_index(kind)] =
                                    html_depths[html_style_index(kind)].saturating_sub(1);
                                html_element_ranges.push(frame.open_range.start..event_range.end);
                                candidates.push(RevealCandidate {
                                    kind: VisualRevealKind::InlineHtml,
                                    source_range: frame.open_range.start..event_range.end,
                                    link_target_range: None,
                                });
                            } else {
                                // A stray or crossing close spoils the block.
                                html_pairing_failed = true;
                                runs.push(VisualInlineRun {
                                    visible_text: authored,
                                    source_range: event_range.clone(),
                                    content_range: event_range,
                                    style,
                                    link_target_range: current_link_target,
                                    navigation: current_link_nav,
                                    math: None,
                                    html_image: None,
                                    conservative_fallback: true,
                                });
                            }
                        }
                    }
                } else {
                    // Non-image inline HTML outside the supported subset (e.g.
                    // `<a href=…>` wrappers, attributed or unknown tags)
                    // cannot map to a rendered form, so it stays as a
                    // byte-exact source run. The run is marked conservative so
                    // the projection shows the authored markup verbatim
                    // instead of guessing a rendered form. A block that mixes
                    // such runs with image runs still renders its images (the
                    // view layer exempts image-bearing blocks from the
                    // whole-block source-island gate).
                    contains_non_image_html = true;
                    if !authored.is_empty() {
                        runs.push(VisualInlineRun {
                            visible_text: authored,
                            source_range: event_range.clone(),
                            content_range: event_range,
                            style,
                            link_target_range: current_link_target,
                            navigation: current_link_nav,
                            math: None,
                            html_image: None,
                            conservative_fallback: true,
                        });
                    }
                }
            }
            _ => {}
        }
        if is_leaf_event {
            previous_leaf_end = previous_leaf_end.max(relative_range.end.min(source.len()));
        }
    }
    // One bad tag spoils the block: a stray/crossing close or a tag left
    // unclosed at the end of the block keeps the whole-block conservative
    // fallback. Styled runs already emitted inside a suspect element are
    // demoted so the mixed image path can never show a half-guessed styled
    // form, and unclosed open-tag bytes become conservative source runs.
    let html_pairing_failed = html_pairing_failed || !html_style_stack.is_empty();
    if html_pairing_failed {
        contains_non_image_html = true;
        let suspect_ranges = html_element_ranges
            .iter()
            .cloned()
            .chain(
                html_style_stack
                    .iter()
                    .map(|frame| frame.open_range.start..block_range.end),
            )
            .collect::<Vec<_>>();
        for run in runs.iter_mut() {
            if run.math.is_none()
                && run.html_image.is_none()
                && suspect_ranges.iter().any(|range| {
                    run.content_range.start >= range.start && run.content_range.end <= range.end
                })
            {
                run.conservative_fallback = true;
            }
        }
        for frame in &html_style_stack {
            runs.push(VisualInlineRun {
                visible_text: text[frame.open_range.clone()].to_string(),
                source_range: frame.open_range.clone(),
                content_range: frame.open_range.clone(),
                style: InlineStyle::default(),
                link_target_range: None,
                navigation: None,
                math: None,
                html_image: None,
                conservative_fallback: true,
            });
        }
    }
    let reveal_groups = build_reveal_groups(text, &block_range, &mut runs, candidates);
    (runs, reveal_groups, contains_non_image_html)
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
        if let Some(spans) = decoded_text_matches(event_source, visible)
            && push_decoded_text_runs(
                runs,
                candidates,
                source,
                event_source,
                &spans,
                event_range.clone(),
                base_style,
                link_target_range.clone(),
                navigation.clone(),
            )
        {
            return;
        }
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
    push_identity_text_runs(
        runs,
        candidates,
        source,
        event_source,
        event_range,
        base_style,
        link_target_range,
        navigation,
    );
}

/// Splits a text event whose parser transformation is proven to consist of
/// decoded spans — backslash escapes and HTML entity references (see
/// `decoded_text_matches`). Each escape becomes a one-byte content run plus
/// an `Escape` reveal candidate — the backslash byte stays uncovered so
/// `marker_ranges` hides it — each entity becomes a single-character run
/// plus an `Entity` reveal candidate covering the full authored token, and
/// the remaining segments keep the identity handling, extended inline
/// markers included. Extended markers compose only when every decoded span
/// is disjoint from the construct or fully inside its content; any other
/// overlap is unproven and returns `false` so the caller keeps the
/// conservative fallback.
fn push_decoded_text_runs(
    runs: &mut Vec<VisualInlineRun>,
    candidates: &mut Vec<RevealCandidate>,
    source: &str,
    event_source: &str,
    spans: &[DecodedSpan],
    event_range: Range<usize>,
    base_style: InlineStyle,
    link_target_range: Option<Range<usize>>,
    navigation: Option<VisualNavigationTarget>,
) -> bool {
    let extended = extended_inline_matches(event_source);
    let conflicting = extended.iter().any(|item| {
        spans.iter().any(|span| {
            let overlaps = span.range.start < item.source_range.end
                && item.source_range.start < span.range.end;
            let inside_content = span.range.start >= item.content_range.start
                && span.range.end <= item.content_range.end;
            overlaps && !inside_content
        })
    });
    if conflicting {
        return false;
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
    for span in spans {
        boundaries.extend([span.range.start, span.range.end]);
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
        let span = spans.iter().find(|span| {
            span.range.start == local_range.start && span.range.end == local_range.end
        });
        match span {
            Some(span) if span.kind == DecodedSpanKind::Escape => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Escape,
                    source_range: event_range.start + local_range.start
                        ..event_range.start + local_range.end,
                    link_target_range: None,
                });
                push_run(
                    runs,
                    source,
                    &event_source[span.range.start + 1..span.range.end],
                    event_range.start + span.range.start + 1..event_range.start + span.range.end,
                    style,
                    link_target_range.clone(),
                    navigation.clone(),
                    false,
                );
            }
            Some(span) => {
                candidates.push(RevealCandidate {
                    kind: VisualRevealKind::Entity,
                    source_range: event_range.start + local_range.start
                        ..event_range.start + local_range.end,
                    link_target_range: None,
                });
                push_decoded_entity_run(
                    runs,
                    &span.decoded,
                    event_range.start + span.range.start..event_range.start + span.range.end,
                    style,
                    link_target_range.clone(),
                    navigation.clone(),
                );
            }
            None => {
                push_run(
                    runs,
                    source,
                    &event_source[local_range.clone()],
                    event_range.start + local_range.start..event_range.start + local_range.end,
                    style,
                    link_target_range.clone(),
                    navigation.clone(),
                    false,
                );
            }
        }
    }
    true
}

/// Emits the run for one decoded entity token: the parser's visible text
/// backed by the full authored token range. Multi-codepoint names occupy one
/// run whose `visible_text` is the full decoded string. The decoded text
/// generally does not occur inside the token's source bytes, so `push_run`'s
/// substring proof cannot apply; the run carries the token as both its source
/// and content range and never triggers the conservative fallback.
fn push_decoded_entity_run(
    runs: &mut Vec<VisualInlineRun>,
    decoded: &str,
    token_range: Range<usize>,
    style: InlineStyle,
    link_target_range: Option<Range<usize>>,
    navigation: Option<VisualNavigationTarget>,
) {
    runs.push(VisualInlineRun {
        visible_text: decoded.to_string(),
        source_range: token_range.clone(),
        content_range: token_range,
        style,
        link_target_range,
        navigation,
        math: None,
        html_image: None,
        conservative_fallback: false,
    });
}

/// One proven parser transformation inside a text event slice: either a
/// backslash escape (two source bytes rendering one punctuation character)
/// or an HTML entity reference (the full authored `&…;` token rendering the
/// parser's decoded string, one or more characters).
#[derive(Clone, PartialEq, Eq)]
enum DecodedSpanKind {
    Escape,
    Entity,
}

#[derive(Clone, PartialEq, Eq)]
struct DecodedSpan {
    /// Byte range of the authored syntax inside the event slice.
    range: Range<usize>,
    kind: DecodedSpanKind,
    /// The text the parser emits for the span.
    decoded: String,
}

/// Proves that the difference between an event slice and the parser's
/// visible text consists only of backslash escapes and HTML entity
/// references, by reconstruction: applying the parser's escape and entity
/// decoding rules must reproduce the visible text exactly. Returns the
/// proven spans, or `None` when the difference cannot be explained, so
/// the caller keeps the conservative fallback.
fn decoded_text_matches(event_source: &str, visible: &str) -> Option<Vec<DecodedSpan>> {
    let bytes = event_source.as_bytes();
    let mut spans = Vec::new();
    let mut reconstructed = String::with_capacity(visible.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\'
            && index + 1 < bytes.len()
            && bytes[index + 1].is_ascii_punctuation()
        {
            // Escaped ASCII punctuation is one ASCII byte, so re-encoding it
            // as `char` cannot split a multi-byte sequence.
            let decoded = bytes[index + 1] as char;
            spans.push(DecodedSpan {
                range: index..index + 2,
                kind: DecodedSpanKind::Escape,
                decoded: decoded.to_string(),
            });
            reconstructed.push(decoded);
            index += 2;
            continue;
        }
        if bytes[index] == b'&'
            && let Some(token_len) = entity_token_len(&event_source[index..])
            && let Some(decoded) = decode_entity_token(&event_source[index..index + token_len])
        {
            spans.push(DecodedSpan {
                range: index..index + token_len,
                kind: DecodedSpanKind::Entity,
                decoded: decoded.clone(),
            });
            reconstructed.push_str(&decoded);
            index += token_len;
            continue;
        }
        let ch = event_source[index..].chars().next()?;
        reconstructed.push(ch);
        index += ch.len_utf8();
    }
    (reconstructed == visible).then_some(spans)
}

/// Length in bytes of one entity reference starting at `source` (including
/// the leading `&` and trailing `;`), mirroring the parser's entity scan
/// shape: `&#` with up to 7 decimal or `&#x` with up to 6 hexadecimal
/// digits, or `&` with an ASCII alphanumeric name. The decode itself happens
/// in `decode_entity_token`; unknown names return a length here but no
/// decode, so the authored bytes stay literal.
fn entity_token_len(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut end = 1;
    if bytes.get(end) == Some(&b'#') {
        end += 1;
        let is_hex = end < bytes.len() && bytes[end] | 0x20 == b'x';
        if is_hex {
            end += 1;
        }
        let digit_count = if is_hex {
            bytes[end..]
                .iter()
                .take(6)
                .take_while(|byte| byte.is_ascii_hexdigit())
                .count()
        } else {
            bytes[end..]
                .iter()
                .take(7)
                .take_while(|byte| byte.is_ascii_digit())
                .count()
        };
        if digit_count == 0 {
            return None;
        }
        end += digit_count;
        return (bytes.get(end) == Some(&b';')).then_some(end + 1);
    }
    end += bytes[end..]
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric())
        .count();
    (bytes.get(end) == Some(&b';')).then_some(end + 1)
}

/// Decodes one complete entity token (`&…;`) to the single character the
/// parser emits, mirroring its shape rules: numeric references map their
/// code point, and named references resolve through the proven
/// single-character table. Zero, surrogate, and out-of-range code points
/// return `None` even though the parser substitutes U+FFFD — the projection
/// degrades to the conservative fallback instead of guessing — as does any
/// unknown name, so those authored bytes stay literal.
fn decode_entity_token(token: &str) -> Option<String> {
    let bytes = token.as_bytes();
    if bytes.len() < 4 || bytes[0] != b'&' || bytes[bytes.len() - 1] != b';' {
        return None;
    }
    if bytes[1] == b'#' {
        let (digits, radix) = if bytes[2] | 0x20 == b'x' {
            (&bytes[3..bytes.len() - 1], 16u32)
        } else {
            (&bytes[2..bytes.len() - 1], 10u32)
        };
        let max_digits = if radix == 16 { 6 } else { 7 };
        if digits.is_empty() || digits.len() > max_digits {
            return None;
        }
        let mut codepoint = 0u32;
        for byte in digits {
            let digit = match byte {
                b'0'..=b'9' => u32::from(byte - b'0'),
                _ if radix == 16 => {
                    let lower = byte | 0x20;
                    if (b'a'..=b'f').contains(&lower) {
                        u32::from(lower - b'a' + 10)
                    } else {
                        return None;
                    }
                }
                _ => return None,
            };
            codepoint = codepoint.checked_mul(radix)?.checked_add(digit)?;
        }
        return if codepoint == 0 {
            None
        } else {
            char::from_u32(codepoint).map(|ch| ch.to_string())
        };
    }
    named_entity_decode(&token[1..token.len() - 1])
}

fn named_entity_char(name: &str) -> Option<char> {
    NAMED_ENTITY_DECODES
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, decoded)| *decoded)
}

fn named_entity_decode(name: &str) -> Option<String> {
    if let Some(decoded) = named_entity_char(name) {
        return Some(decoded.to_string());
    }
    NAMED_ENTITY_DECODES_MULTI
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, decoded)| (*decoded).to_string())
}

/// Named HTML entities proven to decode to exactly one character, verified
/// byte-for-byte against the parser's entity table. Multi-codepoint names
/// live in `NAMED_ENTITY_DECODES_MULTI`. Unknown names stay conservative.
const NAMED_ENTITY_DECODES: &[(&str, char)] = &[
    // Core.
    ("AMP", '&'),
    ("AElig", '\u{00C6}'),
    ("Aacute", '\u{00C1}'),
    ("Acirc", '\u{00C2}'),
    ("Agrave", '\u{00C0}'),
    ("Atilde", '\u{00C3}'),
    ("Auml", '\u{00C4}'),
    ("Aring", '\u{00C5}'),
    ("COPY", '\u{00A9}'),
    ("Ccedil", '\u{00C7}'),
    ("ETH", '\u{00D0}'),
    ("Eacute", '\u{00C9}'),
    ("Ecirc", '\u{00CA}'),
    ("Egrave", '\u{00C8}'),
    ("Euml", '\u{00CB}'),
    ("GT", '>'),
    ("Iacute", '\u{00CD}'),
    ("Icirc", '\u{00CE}'),
    ("Igrave", '\u{00CC}'),
    ("Iuml", '\u{00CF}'),
    ("LT", '<'),
    ("Ntilde", '\u{00D1}'),
    ("Oacute", '\u{00D3}'),
    ("Ocirc", '\u{00D4}'),
    ("Ograve", '\u{00D2}'),
    ("Oslash", '\u{00D8}'),
    ("Otilde", '\u{00D5}'),
    ("Ouml", '\u{00D6}'),
    ("QUOT", '"'),
    ("REG", '\u{00AE}'),
    ("THORN", '\u{00DE}'),
    ("TRADE", '\u{2122}'),
    ("Uacute", '\u{00DA}'),
    ("Ucirc", '\u{00DB}'),
    ("Ugrave", '\u{00D9}'),
    ("Uuml", '\u{00DC}'),
    ("Yacute", '\u{00DD}'),
    ("Yuml", '\u{0178}'),
    ("amp", '&'),
    ("apos", '\''),
    ("gt", '>'),
    ("lt", '<'),
    ("nbsp", '\u{00A0}'),
    ("quot", '"'),
    ("copy", '\u{00A9}'),
    ("reg", '\u{00AE}'),
    ("trade", '\u{2122}'),
    // Typography.
    ("bull", '\u{2022}'),
    ("dagger", '\u{2020}'),
    ("Dagger", '\u{2021}'),
    ("hellip", '\u{2026}'),
    ("mdash", '\u{2014}'),
    ("ndash", '\u{2013}'),
    ("lsquo", '\u{2018}'),
    ("rsquo", '\u{2019}'),
    ("sbquo", '\u{201A}'),
    ("ldquo", '\u{201C}'),
    ("rdquo", '\u{201D}'),
    ("bdquo", '\u{201E}'),
    ("laquo", '\u{00AB}'),
    ("raquo", '\u{00BB}'),
    ("lsaquo", '\u{2039}'),
    ("rsaquo", '\u{203A}'),
    ("prime", '\u{2032}'),
    ("Prime", '\u{2033}'),
    ("permil", '\u{2030}'),
    ("frasl", '\u{2044}'),
    // Math and logic.
    ("plusmn", '\u{00B1}'),
    ("times", '\u{00D7}'),
    ("divide", '\u{00F7}'),
    ("minus", '\u{2212}'),
    ("plus", '\u{002B}'),
    ("lowast", '\u{2217}'),
    ("infin", '\u{221E}'),
    ("le", '\u{2264}'),
    ("ge", '\u{2265}'),
    ("ne", '\u{2260}'),
    ("equiv", '\u{2261}'),
    ("asymp", '\u{2248}'),
    ("approx", '\u{2248}'),
    ("cong", '\u{2245}'),
    ("sim", '\u{223C}'),
    ("prop", '\u{221D}'),
    ("sum", '\u{2211}'),
    ("prod", '\u{220F}'),
    ("radic", '\u{221A}'),
    ("part", '\u{2202}'),
    ("int", '\u{222B}'),
    ("nabla", '\u{2207}'),
    ("there4", '\u{2234}'),
    ("because", '\u{2235}'),
    ("and", '\u{2227}'),
    ("or", '\u{2228}'),
    ("cap", '\u{2229}'),
    ("cup", '\u{222A}'),
    ("sub", '\u{2282}'),
    ("sup", '\u{2283}'),
    ("nsub", '\u{2284}'),
    ("ang", '\u{2220}'),
    ("perp", '\u{22A5}'),
    ("sdot", '\u{22C5}'),
    ("not", '\u{00AC}'),
    ("deg", '\u{00B0}'),
    ("micro", '\u{00B5}'),
    // Arrows and geometry.
    ("larr", '\u{2190}'),
    ("rarr", '\u{2192}'),
    ("uarr", '\u{2191}'),
    ("darr", '\u{2193}'),
    ("harr", '\u{2194}'),
    ("lceil", '\u{2308}'),
    ("rceil", '\u{2309}'),
    ("lfloor", '\u{230A}'),
    ("rfloor", '\u{230B}'),
    ("lang", '\u{27E8}'),
    ("rang", '\u{27E9}'),
    ("loz", '\u{25CA}'),
    ("star", '\u{2606}'),
    // Symbols.
    ("sect", '\u{00A7}'),
    ("para", '\u{00B6}'),
    ("middot", '\u{00B7}'),
    ("acute", '\u{00B4}'),
    ("cedil", '\u{00B8}'),
    ("iexcl", '\u{00A1}'),
    ("iquest", '\u{00BF}'),
    ("circ", '\u{02C6}'),
    ("tilde", '\u{02DC}'),
    ("check", '\u{2713}'),
    ("cross", '\u{2717}'),
    ("female", '\u{2640}'),
    ("male", '\u{2642}'),
    ("spades", '\u{2660}'),
    ("clubs", '\u{2663}'),
    ("hearts", '\u{2665}'),
    ("diams", '\u{2666}'),
    ("sharp", '\u{266F}'),
    ("flat", '\u{266D}'),
    ("natural", '\u{266E}'),
    // Currency.
    ("cent", '\u{00A2}'),
    ("pound", '\u{00A3}'),
    ("curren", '\u{00A4}'),
    ("yen", '\u{00A5}'),
    ("euro", '\u{20AC}'),
    // Fractions and supercripts.
    ("frac14", '\u{00BC}'),
    ("frac12", '\u{00BD}'),
    ("frac34", '\u{00BE}'),
    ("sup1", '\u{00B9}'),
    ("sup2", '\u{00B2}'),
    ("sup3", '\u{00B3}'),
    // Lowercase accented letters.
    ("agrave", '\u{00E0}'),
    ("aacute", '\u{00E1}'),
    ("acirc", '\u{00E2}'),
    ("atilde", '\u{00E3}'),
    ("auml", '\u{00E4}'),
    ("aring", '\u{00E5}'),
    ("aelig", '\u{00E6}'),
    ("ccedil", '\u{00E7}'),
    ("egrave", '\u{00E8}'),
    ("eacute", '\u{00E9}'),
    ("ecirc", '\u{00EA}'),
    ("euml", '\u{00EB}'),
    ("igrave", '\u{00EC}'),
    ("iacute", '\u{00ED}'),
    ("icirc", '\u{00EE}'),
    ("iuml", '\u{00EF}'),
    ("eth", '\u{00F0}'),
    ("ntilde", '\u{00F1}'),
    ("ograve", '\u{00F2}'),
    ("oacute", '\u{00F3}'),
    ("ocirc", '\u{00F4}'),
    ("otilde", '\u{00F5}'),
    ("ouml", '\u{00F6}'),
    ("oslash", '\u{00F8}'),
    ("ugrave", '\u{00F9}'),
    ("uacute", '\u{00FA}'),
    ("ucirc", '\u{00FB}'),
    ("uuml", '\u{00FC}'),
    ("yacute", '\u{00FD}'),
    ("thorn", '\u{00FE}'),
    ("szlig", '\u{00DF}'),
    ("yuml", '\u{00FF}'),
    ("ordf", '\u{00AA}'),
    ("ordm", '\u{00BA}'),
    ("shy", '\u{00AD}'),
    ("macr", '\u{00AF}'),
    ("uml", '\u{00A8}'),
    ("oelig", '\u{0153}'),
    ("OElig", '\u{0152}'),
    ("scaron", '\u{0161}'),
    ("Scaron", '\u{0160}'),
    ("fnof", '\u{0192}'),
    ("ensp", '\u{2002}'),
    ("emsp", '\u{2003}'),
    ("thinsp", '\u{2009}'),
    ("zwnj", '\u{200C}'),
    ("zwj", '\u{200D}'),
    ("lrm", '\u{200E}'),
    ("rlm", '\u{200F}'),
];

/// Named HTML entities that decode to more than one code point. Each entry is
/// proven against pulldown-cmark the same way as the single-character table.
const NAMED_ENTITY_DECODES_MULTI: &[(&str, &str)] = &[("NotEqualTilde", "\u{2242}\u{0338}")];

/// Emits an identity-mapped text slice: the slice equals its visible text, so
/// it is pushed directly or split around extended inline markers.
fn push_identity_text_runs(
    runs: &mut Vec<VisualInlineRun>,
    candidates: &mut Vec<RevealCandidate>,
    source: &str,
    event_source: &str,
    event_range: Range<usize>,
    base_style: InlineStyle,
    link_target_range: Option<Range<usize>>,
    navigation: Option<VisualNavigationTarget>,
) {
    let extended = extended_inline_matches(event_source);
    if extended.is_empty() {
        push_run(
            runs,
            source,
            event_source,
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
        html_image: None,
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

/// Validates the element-pair shape of an `InlineHtml` reveal candidate: the
/// slice must open with a supported style tag and end with the closing tag of
/// the same kind. Content between the tags may contain anything, including
/// nested supported pairs.
fn looks_like_markdown_image(source: &str) -> bool {
    let source = source.trim();
    source.starts_with("![")
        && source.len() >= 5
        && ((source.contains("](") && source.ends_with(')'))
            || (source.contains("][") && source.ends_with(']')))
}

fn inline_html_pair_is_exact(source: &str) -> bool {
    let Some(open_end) = source.find('>') else {
        return false;
    };
    let Some(InlineHtmlStyleTag::Open { kind: open_kind }) =
        parse_inline_html_style_tag(&source[..=open_end])
    else {
        return false;
    };
    let Some(close_start) = source.rfind('<') else {
        return false;
    };
    close_start > open_end
        && matches!(
            parse_inline_html_style_tag(&source[close_start..]),
            Some(InlineHtmlStyleTag::Close { kind }) if kind == open_kind
        )
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
        VisualRevealKind::HtmlImage => {
            parse_inline_html_image(source).is_some() || looks_like_markdown_image(source)
        }
        VisualRevealKind::Escape => {
            source.len() == 2
                && source.as_bytes()[0] == b'\\'
                && source.as_bytes()[1].is_ascii_punctuation()
        }
        VisualRevealKind::Entity => decode_entity_token(source).is_some(),
        VisualRevealKind::InlineHtml => {
            matches!(
                parse_inline_html_style_tag(source),
                Some(InlineHtmlStyleTag::LineBreak)
            ) || inline_html_pair_is_exact(source)
        }
        VisualRevealKind::Math => {
            (source.starts_with("$$") && source.ends_with("$$") && source.len() >= 4)
                || (source.starts_with('$') && source.ends_with('$') && source.len() >= 2)
        }
        VisualRevealKind::Link => {
            let is_angle_autolink = source.starts_with('<')
                && source.ends_with('>')
                && source.len() >= 3
                && !source[1..source.len() - 1].contains(['<', '>', ' ', '\n']);
            if is_angle_autolink {
                candidate.link_target_range.as_ref().is_some_and(|target| {
                    target.start >= range.start
                        && target.end <= range.end
                        && text.is_char_boundary(target.start)
                        && text.is_char_boundary(target.end)
                        && target.start > range.start
                        && target.end < range.end
                })
            } else if !source.starts_with('[') {
                false
            } else if source.ends_with(')') && source.contains("](") {
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
    quote_context: Option<&VisualQuoteContext>,
) -> Option<VisualBlockPrefix> {
    let line_end = text[block_range.clone()]
        .find('\n')
        .map_or(block_range.end, |relative| block_range.start + relative);
    let line = &text[block_range.start..line_end];
    let quote_prefix_len = quote_context
        .and_then(|quote| quote.marker_ranges.first())
        .filter(|range| range.start == block_range.start)
        .map_or(0, |range| range.end - block_range.start);
    let prefix_base = block_range.start + quote_prefix_len;
    let indentation_len = line[quote_prefix_len..]
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let marker_start = quote_prefix_len + indentation_len;
    let indentation_range = prefix_base..prefix_base + indentation_len;

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
        source_range: prefix_base..block_range.start + prefix_end,
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
    let outer_indentation_len = line
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let bytes = line.as_bytes();

    let mut quote_end = outer_indentation_len;
    let mut quote_depth = 0;
    while bytes.get(quote_end) == Some(&b'>') {
        quote_depth += 1;
        quote_end += 1;
        if matches!(bytes.get(quote_end), Some(b' ' | b'\t')) {
            quote_end += 1;
        }
    }

    let (prefix_base, indentation_len) = if quote_depth > 0 {
        let inner_indent = line[quote_end..]
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
        (quote_end, inner_indent)
    } else {
        (0, outer_indentation_len)
    };
    let marker_start = prefix_base + indentation_len;
    let indentation_range = line_start + prefix_base..line_start + prefix_base + indentation_len;

    let parsed_inner = if bytes.get(marker_start) == Some(&b'#') {
        let level = line[marker_start..]
            .bytes()
            .take_while(|byte| *byte == b'#')
            .count();
        if !(1..=6).contains(&level) || bytes.get(marker_start + level) != Some(&b' ') {
            None
        } else {
            Some((
                skip_ascii_spacing(line, marker_start + level),
                VisualBlockPrefixKind::Heading { level: level as u8 },
            ))
        }
    } else if matches!(bytes.get(marker_start), Some(b'-' | b'+' | b'*')) {
        let after_marker = skip_ascii_spacing(line, marker_start + 1);
        if after_marker == marker_start + 1 {
            None
        } else if let Some(task) = bytes.get(after_marker..after_marker + 3)
            && task.first() == Some(&b'[')
            && task.last() == Some(&b']')
            && matches!(task[1], b' ' | b'x' | b'X')
        {
            Some((
                skip_ascii_spacing(line, after_marker + 3),
                VisualBlockPrefixKind::TaskList {
                    level: 1,
                    checked: !matches!(task[1], b' '),
                },
            ))
        } else {
            Some((
                after_marker,
                VisualBlockPrefixKind::UnorderedList { level: 1 },
            ))
        }
    } else {
        let digits_start = marker_start;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == digits_start || !matches!(bytes.get(end), Some(b'.' | b')')) {
            None
        } else {
            let index = line[digits_start..end].parse::<u64>().ok()?;
            let after_marker = skip_ascii_spacing(line, end + 1);
            (after_marker > end + 1).then_some((
                after_marker,
                VisualBlockPrefixKind::OrderedList { level: 1, index },
            ))
        }
    };

    let (source_start, prefix_end, kind, indentation_range) =
        if let Some((prefix_end, kind)) = parsed_inner {
            (prefix_base, prefix_end, kind, indentation_range)
        } else if quote_depth > 0 {
            (
                0,
                quote_end,
                VisualBlockPrefixKind::BlockQuote { depth: quote_depth },
                line_start..line_start + outer_indentation_len,
            )
        } else {
            return None;
        };

    Some(VisualBlockPrefix {
        kind,
        indentation_range,
        source_range: line_start + source_start..line_start + prefix_end,
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

/// Quoted leaves keep later lines' `> ` markers inside the inline slice, so
/// pulldown-cmark splits a lazy-continuation paragraph into separate blocks
/// without emitting a soft break — the newline byte ends up unowned and the
/// projection merges the lines. Give the first newline of each interior gap a
/// synthetic soft-break run owning exactly that byte; the surrounding `> `
/// bytes stay marker-hidden. Interior only: the trailing newline stays
/// unowned like on unquoted paragraphs, and rows without a quote context
/// already receive real soft-break events.
fn synthesize_quote_softbreak_runs(
    text: &str,
    block_range: &Range<usize>,
    runs: &mut Vec<VisualInlineRun>,
) {
    let mut content = runs
        .iter()
        .filter(|run| !run.conservative_fallback)
        .map(|run| run.content_range.clone())
        .collect::<Vec<_>>();
    content.sort_by_key(|range| range.start);
    let mut newline_ranges = Vec::new();
    let mut cursor = block_range.start;
    for range in &content {
        if range.start > cursor {
            let gap = cursor..range.start;
            if let Some(relative) = text[gap.clone()].find('\n') {
                newline_ranges.push(cursor + relative..cursor + relative + 1);
            }
        }
        cursor = cursor.max(range.end);
    }
    for range in newline_ranges {
        runs.push(VisualInlineRun {
            visible_text: "\n".to_string(),
            source_range: range.clone(),
            content_range: range,
            style: InlineStyle::default(),
            link_target_range: None,
            navigation: None,
            math: None,
            html_image: None,
            conservative_fallback: false,
        });
    }
    runs.sort_by_key(|run| run.content_range.start);
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

    use super::{
        build_visual_blocks, build_visual_projection, build_visual_projection_with_marked_range,
        fenced_payload_ranges,
    };
    use crate::{
        AlertKind, BlockTarget, BlockTransform, MarkdownDocument, MarkdownFormat, PreviewBlock,
        RichText, SlashCommand, TableEdit, VisualBlockEditor, VisualBlockId, VisualBlockKind,
        VisualBlockPrefixKind, VisualCaretAffinity, VisualEditorFieldKind, VisualNavigationTarget,
        VisualProjection, VisualProjectionSegment, VisualQuoteGroupEdge, VisualRevealKind,
        VisualSourceIslandKind, slash_command_edit, slash_query_at, transform_block,
    };

    #[test]
    fn quoted_mixed_chinese_fixture_projects_once_without_overlap_or_islands() {
        let source = "> **写在前面：**\n>\n> 这半年，装机的人都被同一件事按在地上摩擦——**内存疯了**。\n>\n> 这篇文章要回答三个问题：\n> 1. **到底疯到什么程度**——用数字说话；\n> 2. **为什么内存造不快**——从一个电容讲起；\n> 3. **AI 怎么改变产能去向**——HBM 更消耗晶圆。\n>\n> 看完之后，你应该能自己判断。\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();

        assert_eq!(blocks.first().unwrap().source_range.start, 0);
        assert_eq!(blocks.last().unwrap().source_range.end, source.len());
        assert!(blocks.windows(2).all(|pair| {
            pair[0].source_range.end == pair[1].source_range.start
                && source.is_char_boundary(pair[0].source_range.end)
        }));
        assert!(blocks.iter().all(|block| {
            !matches!(
                block.kind,
                VisualBlockKind::Unsupported | VisualBlockKind::BlockQuote
            ) && block.source_island.is_none()
                && block.quote_context.is_some()
        }));
        assert_eq!(
            blocks
                .iter()
                .filter(|block| matches!(block.kind, VisualBlockKind::ListItem { .. }))
                .count(),
            3
        );

        let quoted = blocks
            .iter()
            .filter_map(|block| block.quote_context.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(quoted.first().unwrap().edge, VisualQuoteGroupEdge::First);
        assert_eq!(quoted.last().unwrap().edge, VisualQuoteGroupEdge::Last);
        assert!(
            quoted
                .iter()
                .flat_map(|quote| &quote.marker_ranges)
                .all(|range| { source[range.clone()].trim_start().starts_with('>') })
        );

        let rendered = blocks
            .iter()
            .map(|block| {
                let cursor = block
                    .editable_runs
                    .first()
                    .map_or(block.source_range.end, |run| run.content_range.start);
                build_visual_projection(source, block, cursor..cursor, cursor).text
            })
            .collect::<String>();
        for phrase in [
            "写在前面",
            "到底疯到什么程度",
            "为什么内存造不快",
            "AI 怎么改变产能去向",
        ] {
            assert_eq!(rendered.matches(phrase).count(), 1, "{phrase}: {rendered}");
        }
    }

    #[test]
    fn visual_quote_keeps_ascii_punctuation_source_exact() {
        let source = "> \"double\" and 'single' -- dash\n> 1. \"item\" --- long\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert!(blocks.iter().all(|block| {
            block.source_island.is_none()
                && block
                    .editable_runs
                    .iter()
                    .all(|run| !run.conservative_fallback)
        }));
        let rendered = blocks
            .iter()
            .map(|block| {
                let cursor = block.editable_runs[0].content_range.start;
                build_visual_projection(source, block, cursor..cursor, cursor).text
            })
            .collect::<String>();
        assert!(rendered.contains("\"double\" and 'single' -- dash"));
        assert!(rendered.contains("\"item\" --- long"));
        let preview_text = doc.preview_blocks()[0].plain_text();
        assert!(preview_text.contains('“') && preview_text.contains('’'));
    }

    #[test]
    fn gfm_alert_fixture_renders_title_row_without_source_island() {
        let source = "> [!NOTE]\n> \u{4f7f}\u{7528} GLM Coding Plan \u{65f6}\u{ff0c}\u{9700}\u{8981}\u{914d}\u{7f6e}\u{4e13}\u{5c5e}\u{7684} Coding API \u{7aef}\u{70b9} [https://open.bigmodel.cn/api/coding/paas/v4](https://open.bigmodel.cn/api/coding/paas/v4) \u{800c}\u{4e0d}\u{662f}\u{901a}\u{7528} API \u{7aef}\u{70b9}\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(
            blocks[0].kind,
            VisualBlockKind::CalloutTitle {
                kind: AlertKind::Note
            }
        ));
        assert_eq!(&source[blocks[0].source_range.clone()], "> [!NOTE]\n");
        assert!(blocks[0].editable_runs.is_empty());
        let quote = blocks[0]
            .quote_context
            .as_ref()
            .expect("title joins the group");
        assert_eq!(quote.marker_ranges, vec![0..9]);
        assert_eq!(quote.edge, VisualQuoteGroupEdge::First);
        assert!(blocks[0].source_island.is_none());

        assert!(matches!(blocks[1].kind, VisualBlockKind::Paragraph));
        assert!(blocks[1].source_island.is_none());
        assert_eq!(
            blocks[1].quote_context.as_ref().unwrap().edge,
            VisualQuoteGroupEdge::Last
        );

        // Full-document byte coverage stays contiguous without overlaps.
        assert!(
            blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end == pair[1].source_range.start)
        );

        // The body still renders its link as one editable run.
        assert!(blocks[1].editable_runs.iter().any(|run| run.visible_text
            == "https://open.bigmodel.cn/api/coding/paas/v4"
            && run.link_target_range.is_some()));
    }

    #[test]
    fn gfm_alert_title_row_reveals_marker_line_on_focus() {
        let source = "> [!WARNING]\n> body\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let title = &blocks[0];

        // Unfocused: nothing of the structural line is projected; the view
        // shows only the decorative label.
        let outside = build_visual_projection(source, title, 100..100, 100);
        assert!(outside.text.is_empty());

        // Caret anywhere inside the marker line reveals it verbatim.
        for cursor in [0, 2, 5, 8] {
            let focused = build_visual_projection(source, title, cursor..cursor, cursor);
            assert_eq!(focused.text, "> [!WARNING]", "cursor {cursor}");
        }
    }

    #[test]
    fn gfm_alert_with_separator_after_marker_splits_title_and_whitespace() {
        let source = "> [!NOTE]\n>\n> body\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks.len(), 3);
        assert!(matches!(
            blocks[0].kind,
            VisualBlockKind::CalloutTitle { .. }
        ));
        assert_eq!(&source[blocks[0].source_range.clone()], "> [!NOTE]\n");
        assert!(matches!(blocks[1].kind, VisualBlockKind::Whitespace));
        assert_eq!(&source[blocks[1].source_range.clone()], ">\n");
        assert!(matches!(blocks[2].kind, VisualBlockKind::Paragraph));
        assert!(
            blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end == pair[1].source_range.start)
        );
    }

    #[test]
    fn body_less_alert_renders_single_title_row() {
        let source = "> [!TIP]\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].kind,
            VisualBlockKind::CalloutTitle {
                kind: AlertKind::Tip
            }
        ));
        assert_eq!(blocks[0].source_range, 0..source.len());
        assert_eq!(blocks[0].source_island, None);
        assert_eq!(
            blocks[0].quote_context.as_ref().unwrap().edge,
            VisualQuoteGroupEdge::Only
        );
    }

    #[test]
    fn gfm_alert_after_paragraph_still_renders_title_row() {
        // The gap holding the marker line starts at the previous block's end,
        // outside the quote group — the title row must still be found.
        for line_ending in ["\n", "\r\n"] {
            let source = format!(
                "intro paragraph{le}{le}> [!NOTE]{le}> body text{le}",
                le = line_ending
            );
            let doc = MarkdownDocument::from_text(&source);
            let blocks = doc.visual_blocks_shared();

            assert!(
                blocks.iter().all(|block| block.source_island.is_none()),
                "no source islands: {blocks:?}"
            );
            let title = blocks
                .iter()
                .position(|block| {
                    matches!(
                        block.kind,
                        VisualBlockKind::CalloutTitle {
                            kind: AlertKind::Note
                        }
                    )
                })
                .expect("callout title row exists");
            assert_eq!(
                source[blocks[title].source_range.clone()].trim_end(),
                "> [!NOTE]"
            );
            assert!(matches!(
                blocks[title - 1].kind,
                VisualBlockKind::Whitespace
            ));
            assert!(blocks[title - 1].quote_context.is_none());
            assert!(
                blocks
                    .iter()
                    .any(|block| matches!(block.kind, VisualBlockKind::Paragraph)
                        && block.quote_context.is_some())
            );
            // Full-document byte coverage stays contiguous without overlaps.
            assert_eq!(blocks.first().unwrap().source_range.start, 0);
            assert_eq!(blocks.last().unwrap().source_range.end, source.len());
            assert!(
                blocks
                    .windows(2)
                    .all(|pair| pair[0].source_range.end == pair[1].source_range.start)
            );
        }
    }

    #[test]
    fn quoted_multiline_paragraph_keeps_softbreak() {
        let source = "> first line\n> second line\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        let cursor = block.editable_runs[0].content_range.start;
        let projection = build_visual_projection(source, block, cursor..cursor, cursor);
        assert_eq!(projection.text, "first line\nsecond line");
        // Display order follows source order and every segment maps back.
        assert!(
            projection
                .segments
                .windows(2)
                .all(|pair| pair[0].source_range.end <= pair[1].source_range.start)
        );
        // The newline byte is owned by a run; only the `> ` markers and the
        // trailing newline hide.
        assert_eq!(block.marker_ranges, vec![0..2, 13..15, 26..27]);
    }

    #[test]
    fn quoted_hard_break_line_still_breaks() {
        let source = "> a  \n> b\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        let cursor = block.editable_runs[0].content_range.start;
        let projection = build_visual_projection(source, block, cursor..cursor, cursor);
        assert_eq!(projection.text, "a\nb");
    }

    #[test]
    fn unknown_alert_marker_stays_literal_on_own_line() {
        let source = "> [!CUSTOM]\n> body text\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert!(
            blocks
                .iter()
                .all(|block| matches!(block.kind, VisualBlockKind::Paragraph))
        );
        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        let cursor = block.editable_runs[0].content_range.start;
        let projection = build_visual_projection(source, block, cursor..cursor, cursor);
        assert_eq!(projection.text, "[!CUSTOM]\nbody text");
        assert!(block.source_island.is_none());
    }

    #[test]
    fn marker_line_with_trailing_text_stays_literal() {
        // `> [!NOTE] extra` is not a GFM alert upstream: it must stay plain
        // paragraph text (on its own line once soft breaks are preserved).
        let source = "> [!NOTE] extra\n> body\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert!(
            blocks
                .iter()
                .all(|block| matches!(block.kind, VisualBlockKind::Paragraph))
        );
        assert_eq!(blocks.len(), 1);
        let cursor = blocks[0].editable_runs[0].content_range.start;
        let projection = build_visual_projection(source, &blocks[0], cursor..cursor, cursor);
        assert_eq!(projection.text, "[!NOTE] extra\nbody");
    }

    #[test]
    fn unquoted_multiline_paragraph_projection_is_unchanged() {
        let source = "alpha\nbeta\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        let cursor = block.editable_runs[0].content_range.start;
        let projection = build_visual_projection(source, block, cursor..cursor, cursor);
        assert_eq!(projection.text, "alpha\nbeta");
        // Only the trailing newline stays structural, as before.
        assert_eq!(block.marker_ranges, vec![source.len() - 1..source.len()]);
    }

    #[test]
    fn quoted_nested_lists_and_blank_separators_remain_partitioned() {
        let source = "> - parent\n>   - child\n> - [x] done\n>\n> outro\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert!(
            blocks
                .windows(2)
                .all(|pair| { pair[0].source_range.end == pair[1].source_range.start })
        );
        assert!(
            blocks
                .iter()
                .all(|block| { block.source_island.is_none() && block.quote_context.is_some() })
        );
        let projected = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .map(|block| {
                let run = &block.editable_runs[0];
                // Interior of the payload: caret at content start == prefix.end
                // now reveals the list marker (Typora-style). Duplicate-text
                // checks must not use that boundary.
                let cursor = run.content_range.start + 1;
                build_visual_projection(source, block, cursor..cursor, cursor).text
            })
            .collect::<Vec<_>>();
        assert_eq!(projected, ["parent", "child", "done", "outro"]);
    }

    #[test]
    fn list_nested_blockquote_list_has_complete_safe_visual_coverage() {
        let source = "- outer\n\n  > - inner\n";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();

        assert_eq!(blocks.first().unwrap().source_range.start, 0);
        assert_eq!(blocks.last().unwrap().source_range.end, source.len());
        assert!(blocks.iter().all(|block| {
            block.source_range.start <= block.source_range.end
                && block.source_range.end <= source.len()
                && source.is_char_boundary(block.source_range.start)
                && source.is_char_boundary(block.source_range.end)
        }));
        assert!(
            blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end == pair[1].source_range.start),
            "visual coverage is not contiguous: {blocks:#?}"
        );
        assert!(
            blocks
                .iter()
                .all(|block| block.source_island != Some(VisualSourceIslandKind::Unsupported)),
            "supported nesting degraded to an unsupported source island: {blocks:#?}"
        );
    }

    #[test]
    fn malformed_preview_ranges_fall_back_without_panicking() {
        // Reversed, out-of-bounds, and non-UTF-8-boundary leaves must be
        // omitted from semantic projection; the coverage loop then keeps
        // every canonical byte through the source-backed gap fallback.
        // `café` is 5 bytes, so 0..4 splits `é` mid-character.
        let source = "café\nalpha\n";
        let preview = [
            PreviewBlock::Paragraph {
                text: RichText::plain("split"),
                source_range: 0..4,
            },
            PreviewBlock::Paragraph {
                text: RichText::plain("reversed"),
                source_range: 9..6,
            },
            PreviewBlock::Paragraph {
                text: RichText::plain("out-of-bounds"),
                source_range: 6..(source.len() + 8),
            },
        ];
        let blocks = build_visual_blocks(source, &preview, VisualBlockId::fresh);
        assert_eq!(blocks.first().unwrap().source_range.start, 0);
        assert_eq!(blocks.last().unwrap().source_range.end, source.len());
        assert!(
            blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end == pair[1].source_range.start),
            "malformed leaves broke contiguous coverage: {blocks:#?}"
        );
        assert!(
            blocks
                .iter()
                .all(|block| block.source_island == Some(VisualSourceIslandKind::Unsupported)),
            "malformed leaves must degrade to source-backed islands: {blocks:#?}"
        );
    }

    #[test]
    fn quoted_leaf_outside_its_group_falls_back_without_panicking() {
        // The original crash topology at the projection layer: a valid child
        // range that lies entirely before its quote group clamps to a
        // reversed range. The leaf must be dropped and its bytes covered by
        // source-backed fallback instead of slicing the reversed range.
        let source = "- outer\n\n  > - inner\n";
        let preview = [PreviewBlock::BlockQuote {
            children: vec![PreviewBlock::ListItem {
                level: 0,
                ordered: false,
                index: None,
                checked: None,
                text: RichText::plain("outer"),
                source_range: 0..7,
            }],
            alert: None,
            source_range: 9..source.len(),
        }];
        let blocks = build_visual_blocks(source, &preview, VisualBlockId::fresh);
        assert_eq!(blocks.first().unwrap().source_range.start, 0);
        assert_eq!(blocks.last().unwrap().source_range.end, source.len());
        assert!(
            blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end == pair[1].source_range.start),
            "reversed quoted leaf broke contiguous coverage: {blocks:#?}"
        );
        assert!(
            blocks.iter().all(|block| {
                block.source_range.start <= block.source_range.end
                    && block.source_range.end <= source.len()
                    && source.is_char_boundary(block.source_range.start)
                    && source.is_char_boundary(block.source_range.end)
            }),
            "coverage contains an invalid range: {blocks:#?}"
        );
    }

    #[test]
    fn empty_quote_marker_is_quote_context_whitespace() {
        let source = "> \n";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        assert!(matches!(blocks.as_slice(), [block]
        if matches!(block.kind, VisualBlockKind::Whitespace)
            && block.source_range == (0..source.len())
            && block.source_island.is_none()
            && block.quote_context.as_ref().is_some_and(|quote| {
                quote.depth == 1
                    && quote.marker_ranges == vec![0..2]
                    && quote.edge == VisualQuoteGroupEdge::Only
            })));
    }

    #[test]
    fn quote_and_list_prefix_layers_reveal_independently() {
        let source = "> > 3. item";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::ListItem { .. }))
            .unwrap();
        let quote = block.quote_context.as_ref().unwrap();
        assert_eq!(quote.depth, 2);
        assert_eq!(&source[quote.marker_ranges[0].clone()], "> > ");
        let list = block.block_prefix.as_ref().unwrap();
        assert_eq!(
            list.kind,
            VisualBlockPrefixKind::OrderedList { level: 1, index: 3 }
        );
        assert_eq!(&source[list.source_range.clone()], "3. ");

        let content_cursor = source.find("item").unwrap() + 1;
        let hidden = build_visual_projection(
            source,
            block,
            content_cursor..content_cursor,
            content_cursor,
        );
        assert_eq!(hidden.text, "item");

        let quote_cursor = quote.marker_ranges[0].start;
        let quote_revealed =
            build_visual_projection(source, block, quote_cursor..quote_cursor, quote_cursor);
        assert_eq!(quote_revealed.text, "> > item");
        assert_eq!(
            quote_revealed
                .source_for_display(quote_revealed.display_for_source(quote_cursor).unwrap()),
            quote_cursor
        );

        let list_cursor = list.source_range.start;
        let list_revealed =
            build_visual_projection(source, block, list_cursor..list_cursor, list_cursor);
        assert_eq!(list_revealed.text, "3. item");
    }

    fn assert_empty_atx_heading(source: &str, level: u8) {
        let doc = MarkdownDocument::from_text(source);
        let preview = doc.preview_blocks();
        let heading = preview
            .iter()
            .find(|block| matches!(block, PreviewBlock::Heading { .. }))
            .unwrap_or_else(|| panic!("expected preview heading for {source:?}, got {preview:?}"));
        match heading {
            PreviewBlock::Heading {
                level: got,
                text,
                source_range,
            } => {
                assert_eq!(*got, level, "source: {source:?}");
                assert!(text.is_empty(), "source: {source:?}");
                let slice = source[source_range.clone()].trim_end_matches(['\n', '\r']);
                assert!(
                    slice.chars().all(|ch| ch == '#' || ch == ' ' || ch == '\t'),
                    "heading range {slice:?} for {source:?}"
                );
            }
            other => panic!("expected heading, got {other:?}"),
        }
        let outline = doc.outline();
        assert!(
            outline.iter().any(|heading| heading.level == level),
            "outline should keep empty heading {source:?}: {outline:?}"
        );
        let blocks = doc.visual_blocks();
        let block = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Heading { .. }))
            .unwrap_or_else(|| {
                panic!(
                    "expected visual heading for {source:?}, got {:?}",
                    blocks.iter().map(|block| &block.kind).collect::<Vec<_>>()
                )
            });
        assert_eq!(block.source_island, None, "source: {source:?}");
        assert!(
            !matches!(block.kind, VisualBlockKind::Unsupported),
            "source: {source:?}"
        );
        let prefix = block.block_prefix.as_ref().expect("heading prefix");
        assert_eq!(
            prefix.kind,
            VisualBlockPrefixKind::Heading { level },
            "source: {source:?}"
        );
        assert!(
            source[prefix.source_range.clone()].starts_with(&"#".repeat(level as usize)),
            "prefix {:?} for {source:?}",
            &source[prefix.source_range.clone()]
        );
    }

    fn assert_empty_list_item(source: &str) {
        let doc = MarkdownDocument::from_text(source);
        let preview = doc.preview_blocks();
        assert!(
            preview.iter().any(|block| {
                matches!(block, PreviewBlock::ListItem { text, .. } if text.is_empty())
            }),
            "expected empty preview list item for {source:?}, got {preview:?}"
        );
        let blocks = doc.visual_blocks();
        let block = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::ListItem { .. }))
            .unwrap_or_else(|| {
                panic!(
                    "expected visual list item for {source:?}, got {:?}",
                    blocks.iter().map(|block| &block.kind).collect::<Vec<_>>()
                )
            });
        assert_eq!(block.source_island, None, "source: {source:?}");
        assert!(
            !matches!(block.kind, VisualBlockKind::Unsupported),
            "source: {source:?}"
        );
        assert!(
            block.block_prefix.is_some(),
            "empty list item should keep a structural prefix: {source:?}"
        );
    }

    #[test]
    fn empty_atx_headings_stay_heading_blocks_not_source_islands() {
        for (source, level) in [
            ("#", 1_u8),
            ("#\n", 1),
            ("##", 2),
            ("##\n", 2),
            ("## ", 2),
            ("###", 3),
            ("###     ", 3),
            ("###     \n", 3),
            ("######", 6),
            ("######\n", 6),
        ] {
            assert_empty_atx_heading(source, level);
        }
    }

    #[test]
    fn empty_list_items_stay_list_rows_not_source_islands() {
        for source in ["- ", "* ", "1. ", "1) ", "- [ ] "] {
            assert_empty_list_item(source);
            assert_empty_list_item(&format!("{source}\n"));
        }
    }

    #[test]
    fn empty_heading_and_list_prefix_reveal_on_caret() {
        let source = "###";
        let doc = MarkdownDocument::from_text(source);
        let block = &doc.visual_blocks()[0];
        let prefix = block.block_prefix.as_ref().expect("heading prefix");
        let at_end = prefix.source_range.end;
        let revealed = build_visual_projection(source, block, at_end..at_end, at_end);
        assert!(
            revealed.text.starts_with("###"),
            "empty heading should reveal hashes, got {:?}",
            revealed.text
        );
        assert!(
            revealed
                .revealed_source_ranges
                .iter()
                .any(|range| range == &prefix.source_range)
        );

        let titled = "# Hello";
        let doc = MarkdownDocument::from_text(titled);
        let block = &doc.visual_blocks()[0];
        let prefix = block.block_prefix.as_ref().expect("heading prefix");
        assert_eq!(&titled[prefix.source_range.clone()], "# ");
        let at_title_start = prefix.source_range.end;
        let at_start = build_visual_projection(
            titled,
            block,
            at_title_start..at_title_start,
            at_title_start,
        );
        assert!(
            at_start.text.starts_with("# "),
            "caret at first title character should reveal prefix, got {:?}",
            at_start.text
        );
        let interior = titled.find('e').unwrap();
        let hidden = build_visual_projection(titled, block, interior..interior, interior);
        assert_eq!(hidden.text, "Hello");
        assert!(hidden.revealed_source_ranges.is_empty());
    }

    #[test]
    fn slash_heading_template_stays_a_visual_heading() {
        let source = "/h2";
        let query = slash_query_at(source, source.len(), 1).unwrap();
        let edit = slash_command_edit(source, 1, &query, SlashCommand::Heading(2)).unwrap();
        assert_eq!(edit.replacement, "## ");
        let doc = MarkdownDocument::from_text(&edit.replacement);
        let block = &doc.visual_blocks()[0];
        assert!(matches!(block.kind, VisualBlockKind::Heading { level: 2 }));
        assert_eq!(block.source_island, None);

        let paragraph = MarkdownDocument::from_text("body");
        let blocks = paragraph.visual_blocks_shared();
        let target = BlockTarget::from_block(paragraph.version(), &blocks[0]);
        let edit = transform_block(
            paragraph.text(),
            paragraph.version(),
            &blocks,
            &target,
            BlockTransform::Heading(2),
        )
        .unwrap();
        let mut text = paragraph.text().to_string();
        text.replace_range(edit.range.clone(), &edit.replacement);
        let after = MarkdownDocument::from_text(&text);
        let heading = after
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Heading { level: 2 }))
            .expect("transformed heading");
        assert_eq!(heading.source_island, None);
    }

    #[test]
    fn remaining_source_islands_stay_source_backed() {
        let front = MarkdownDocument::from_text("---\ntitle: Demo\n---\n\nBody");
        let island = front
            .visual_blocks()
            .into_iter()
            .find(|block| block.source_island == Some(VisualSourceIslandKind::FrontMatter))
            .expect("front matter island");
        assert!(
            front.text()[island.source_range.clone()].starts_with("---"),
            "front matter island should cover the YAML region"
        );
        assert!(front.text()[island.source_range.clone()].contains("title: Demo"));

        let unclosed = "```rust\nfn main() {}\n";
        let block = MarkdownDocument::from_text(unclosed)
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .expect("unclosed fence");
        assert_eq!(block.source_island, Some(VisualSourceIslandKind::Code));
        assert!(block.editor.is_none());
        assert_eq!(&unclosed[block.source_range.clone()], unclosed);
    }

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

        let quoted_source = "> quote\n";
        let quoted = MarkdownDocument::from_text(quoted_source).visual_blocks();
        let context = quoted[0].quote_context.as_ref().expect("quote context");
        assert_eq!(context.depth, 1);
        assert_eq!(&quoted_source[context.marker_ranges[0].clone()], "> ");

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
    fn escaped_punctuation_renders_with_hidden_backslash_and_reveal_groups() {
        let source = r"escaped \*marker\. done";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback),
            "escapes must not demote the paragraph to a source island"
        );
        let visible: String = block
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert_eq!(visible, "escaped *marker. done");
        let escapes: Vec<&str> = block
            .reveal_groups
            .iter()
            .filter(|group| group.kind == VisualRevealKind::Escape)
            .map(|group| &source[group.source_range.clone()])
            .collect();
        assert_eq!(escapes, vec![r"\*", r"\."]);
        // Only the backslash bytes stay hidden; every escaped character is a
        // one-byte identity content run.
        let markers: Vec<&str> = block
            .marker_ranges
            .iter()
            .map(|range| &source[range.clone()])
            .collect();
        assert_eq!(markers, vec![r"\", r"\"]);
        let star = block
            .editable_runs
            .iter()
            .find(|run| run.content_range.len() == 1 && &source[run.content_range.clone()] == "*")
            .expect("escaped star keeps a one-byte content run");
        assert_eq!(star.source_range, star.content_range);
    }

    #[test]
    fn escaped_backslash_and_projection_reveal_round_trip() {
        let source = r"literal \\ and \* star";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        let visible: String = block
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert_eq!(visible, r"literal \ and * star");
        let escapes: Vec<&str> = block
            .reveal_groups
            .iter()
            .filter(|group| group.kind == VisualRevealKind::Escape)
            .map(|group| &source[group.source_range.clone()])
            .collect();
        assert_eq!(escapes, vec![r"\\", r"\*"]);

        let star_byte = source.find('*').unwrap();
        let projection =
            build_visual_projection_with_marked_range(source, &blocks[0], 0..0, star_byte, None);
        assert!(
            projection
                .revealed_source_ranges
                .contains(&(star_byte - 1..star_byte + 1)),
            "caret in the escaped star reveals the complete \\* group"
        );

        let projection =
            build_visual_projection_with_marked_range(source, &blocks[0], 0..0, 0, None);
        assert_eq!(projection.text, r"literal \ and * star");
    }

    #[test]
    fn escapes_compose_with_strong_and_highlight() {
        let source = r"**bold \* star** and ==mark \. dot==";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        let star_run = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "*")
            .expect("escaped star renders literally");
        assert!(star_run.style.bold, "escape inside strong keeps the style");
        let dot_run = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text == ".")
            .expect("escaped dot renders literally");
        assert!(!dot_run.conservative_fallback);
        // Parser-level styling (strong/emphasis/links) applies across events,
        // but slice-level extended markers cannot span the escape gap: the
        // `==` pair loses its highlight styling, matching Split Preview's
        // per-event extended parsing for the same source.
        assert!(!dot_run.style.highlight);
        let kinds: Vec<VisualRevealKind> =
            block.reveal_groups.iter().map(|group| group.kind).collect();
        assert!(kinds.contains(&VisualRevealKind::Escape));
        assert!(kinds.contains(&VisualRevealKind::Strong));
        assert!(!kinds.contains(&VisualRevealKind::Highlight));
    }

    #[test]
    fn entity_references_render_with_progressive_reveal() {
        let source = "fish &amp; chips &#39;quoted&#39; &#x2014; tail";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback),
            "proven entity references render instead of demoting the paragraph"
        );
        let visible: String = block
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert_eq!(visible, "fish & chips 'quoted' \u{2014} tail");

        // Every decoded character is one run whose source and content range
        // is the complete authored token.
        for (token, decoded) in [("&amp;", "&"), ("&#39;", "'"), ("&#x2014;", "\u{2014}")] {
            let run = block
                .editable_runs
                .iter()
                .find(|run| run.visible_text == decoded)
                .unwrap_or_else(|| panic!("decoded run for {token}"));
            assert_eq!(&source[run.source_range.clone()], token);
            assert_eq!(run.source_range, run.content_range);
        }

        let entities: Vec<&str> = block
            .reveal_groups
            .iter()
            .filter(|group| group.kind == VisualRevealKind::Entity)
            .map(|group| &source[group.source_range.clone()])
            .collect();
        assert_eq!(entities, vec!["&amp;", "&#39;", "&#39;", "&#x2014;"]);
        // Entity tokens are fully covered content, so nothing stays hidden.
        assert!(block.marker_ranges.is_empty());
    }

    #[test]
    fn entity_projection_reveal_round_trip() {
        let source = "fish &amp; chips";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        let token_start = source.find("&amp;").unwrap();

        // Caret anywhere inside the token reveals the complete authored form.
        for cursor in token_start..token_start + "&amp;".len() {
            let projection =
                build_visual_projection_with_marked_range(source, block, 0..0, cursor, None);
            assert!(
                projection
                    .revealed_source_ranges
                    .contains(&(token_start..token_start + "&amp;".len())),
                "caret at {cursor} reveals the whole entity token"
            );
            assert_eq!(projection.text, "fish &amp; chips");
        }

        // Caret outside keeps the decoded rendering.
        let projection = build_visual_projection_with_marked_range(source, block, 0..0, 0, None);
        assert_eq!(projection.text, "fish & chips");
        assert!(projection.revealed_source_ranges.is_empty());

        // The single display character maps back to the token boundaries:
        // before it resolves to the token start, after it to the token end.
        let amp_display = projection.text.find('&').unwrap();
        let before = projection.boundary_candidates(amp_display);
        assert_eq!(before.upstream_source, token_start);
        assert_eq!(before.downstream_source, token_start);
        let after = projection.boundary_candidates(amp_display + 1);
        assert_eq!(after.upstream_source, token_start + "&amp;".len());
        assert_eq!(after.downstream_source, token_start + "&amp;".len());
    }

    #[test]
    fn unproven_entity_forms_stay_conservative() {
        // Names outside the maintained tables and numeric forms the parser
        // maps to U+FFFD cannot be proven, so those runs stay conservative.
        for source in [
            "value &angst; end",
            "value &#0; end",
            "value &#xD800; end",
            "value &#x110000; end",
            "value &#1114112; end",
        ] {
            let doc = MarkdownDocument::from_text(source);
            let blocks = doc.visual_blocks();
            assert!(
                blocks[0]
                    .editable_runs
                    .iter()
                    .any(|run| run.conservative_fallback),
                "unproven entity form stays conservative: {source}"
            );
        }
    }

    #[test]
    fn numeric_entity_forms_decode_like_the_parser() {
        let source = "a &#65; b &#X41; c &#8212; d &#x2014; e";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback),
            "complete numeric references render"
        );
        let visible: String = block
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert_eq!(visible, "a A b A c \u{2014} d \u{2014} e");
    }

    #[test]
    fn entities_compose_with_escapes_and_strong() {
        let source = r"**a &amp; \* b**";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        let visible: String = block
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert_eq!(visible, "a & * b");
        assert!(
            block.editable_runs.iter().all(|run| run.style.bold),
            "entity and escape runs keep the enclosing strong style"
        );
        let kinds: Vec<VisualRevealKind> =
            block.reveal_groups.iter().map(|group| group.kind).collect();
        assert!(kinds.contains(&VisualRevealKind::Strong));
        assert!(kinds.contains(&VisualRevealKind::Entity));
        assert!(kinds.contains(&VisualRevealKind::Escape));

        // Caret inside the entity activates only the containing strong group.
        let token_start = source.find("&amp;").unwrap();
        let projection =
            build_visual_projection_with_marked_range(source, block, 0..0, token_start + 1, None);
        assert_eq!(
            projection.revealed_source_ranges,
            vec![0..source.len()],
            "the outermost containing group is revealed exactly once"
        );
        assert_eq!(projection.text, source);
    }

    #[test]
    fn mixed_escapes_and_entities_in_one_event() {
        let source = r"a \* b &amp; c &#8212; d";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback),
            "one event mixing escapes and entities proves as a whole"
        );
        let visible: String = block
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert_eq!(visible, "a * b & c \u{2014} d");
    }

    #[test]
    fn entities_stay_byte_exact_beside_multibyte_text() {
        let source = "世界 &amp; 中文 &#8212; 完";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        let visible: String = block
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert_eq!(visible, "世界 & 中文 \u{2014} 完");
        let entity_run = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "&")
            .expect("decoded ampersand run");
        assert_eq!(&source[entity_run.content_range.clone()], "&amp;");
    }

    #[test]
    fn named_entity_table_matches_pulldown_decoding() {
        use pulldown_cmark::{Event, Parser};

        for &(name, decoded) in super::NAMED_ENTITY_DECODES {
            let reference = format!("&{name};");
            let mut visible = String::new();
            for event in Parser::new_ext(&reference, crate::parse::visual_markdown_options()) {
                if let Event::Text(chunk) = event {
                    visible.push_str(&chunk);
                }
            }
            assert_eq!(
                visible,
                decoded.to_string(),
                "table entry for {reference} disagrees with the parser"
            );
        }
        for &(name, decoded) in super::NAMED_ENTITY_DECODES_MULTI {
            let reference = format!("&{name};");
            let mut visible = String::new();
            for event in Parser::new_ext(&reference, crate::parse::visual_markdown_options()) {
                if let Event::Text(chunk) = event {
                    visible.push_str(&chunk);
                }
            }
            assert_eq!(
                visible, decoded,
                "multi-codepoint table entry for {reference} disagrees with the parser"
            );
        }
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
    fn quoted_interaction_projections_reuse_one_per_version_cache() {
        let source = "> intro **bold**\n>\n> 1. first\n> 2. second\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let counters = doc.source_mapped_derivation_counters();
        let version = doc.version();

        for block in blocks.iter() {
            for cursor in block
                .quote_context
                .iter()
                .flat_map(|quote| quote.marker_ranges.iter().map(|range| range.start))
                .chain(
                    block
                        .editable_runs
                        .iter()
                        .map(|run| run.content_range.start),
                )
            {
                let _ = build_visual_projection(source, block, cursor..cursor, cursor);
                let end = block.source_range.end.min(cursor + 1);
                let _ = build_visual_projection(source, block, cursor..end, end);
            }
        }

        assert_eq!(doc.version(), version);
        assert_eq!(doc.source_mapped_derivation_counters(), counters);
        assert!(Arc::ptr_eq(&blocks, &doc.visual_blocks_shared()));
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
                    ) || block.quote_context.is_some()
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
                let run = block.editable_runs.first().expect("list item content");
                let cursor = run.content_range.start + 1;
                build_visual_projection(source, block, cursor..cursor, cursor).text
            })
            .collect::<Vec<_>>();
        assert_eq!(
            projected,
            ["parent", "child", "grandchild", "ordered", "nested"]
        );
    }

    #[test]
    fn list_item_with_nested_fenced_code_renders_rows_without_raw_boxes() {
        let source = "- first item with [link](https://example.com)\n    \n    ```\n    export A=1\n    ```\n    \n- second item\n    \n    ```\n    export B=2\n    ```\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();

        // Full-document byte coverage stays contiguous without overlaps.
        assert_eq!(blocks.first().unwrap().source_range.start, 0);
        assert_eq!(blocks.last().unwrap().source_range.end, source.len());
        assert!(
            blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end == pair[1].source_range.start)
        );

        // No raw-source fallback rows: every construct is owned by its renderer.
        assert!(
            blocks.iter().all(|block| {
                !matches!(block.kind, VisualBlockKind::Unsupported) && block.source_island.is_none()
            }),
            "unexpected source island: {:?}",
            blocks
                .iter()
                .map(|block| (&block.kind, &block.source_island, &block.source_range))
                .collect::<Vec<_>>()
        );

        let list_rows = blocks
            .iter()
            .filter(|block| matches!(block.kind, VisualBlockKind::ListItem { .. }))
            .collect::<Vec<_>>();
        assert_eq!(list_rows.len(), 2);
        let code_rows = blocks
            .iter()
            .filter(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .collect::<Vec<_>>();
        assert_eq!(code_rows.len(), 2);
        for row in &code_rows {
            assert!(
                matches!(row.editor, Some(VisualBlockEditor::Code { .. })),
                "nested fence keeps its source-backed code editor"
            );
        }

        // Item text and code payload each project exactly once.
        let rendered = blocks
            .iter()
            .map(|block| {
                let cursor = block
                    .editable_runs
                    .first()
                    .map_or(block.source_range.end, |run| run.content_range.start);
                build_visual_projection(source, block, cursor..cursor, cursor).text
            })
            .collect::<String>();
        assert_eq!(rendered.matches("first item").count(), 1, "{rendered}");
        assert_eq!(rendered.matches("export A=1").count(), 1, "{rendered}");
        assert_eq!(rendered.matches("second item").count(), 1, "{rendered}");
        assert_eq!(rendered.matches("export B=2").count(), 1, "{rendered}");
        assert!(
            !rendered.contains("[link](https://example.com)"),
            "link syntax stays hidden: {rendered}"
        );
    }

    #[test]
    fn nested_fence_with_list_indentation_resolves_editor_ranges() {
        let source = "- item\n\n    ```sh\n    export A=1\n    ```\n";
        let fence_start = source.find("```").unwrap();
        let fence_end = source.rfind("```").unwrap() + 3;
        let (payload, info, opening, closing) =
            fenced_payload_ranges(source, fence_start..fence_end, '`', '~')
                .expect("nested fence resolves its editor ranges");
        assert_eq!(&source[opening], "```");
        assert_eq!(info.map(|range| &source[range]), Some("sh"));
        assert_eq!(&source[payload], "    export A=1\n");
        assert_eq!(&source[closing], "```");
    }

    #[test]
    fn nested_fence_lenient_scan_handles_edge_cases() {
        // Blank first payload line plus payload lines that merely start with
        // backticks: a shorter run or trailing info text is never a closing.
        let source = "- item\n\n    ```\n    \n    ``\n    ```text\n    real\n    ```\n";
        let fence_start = source.find("```").unwrap();
        let fence_end = source.rfind("```").unwrap() + 3;
        let (payload, info, opening, closing) =
            fenced_payload_ranges(source, fence_start..fence_end, '`', '~')
                .expect("nested fence with tricky payload");
        assert_eq!(&source[opening], "```");
        assert!(info.is_none());
        assert_eq!(&source[payload], "    \n    ``\n    ```text\n    real\n");
        assert_eq!(&source[closing], "```");

        // Tilde fence nested in an ordered list (content indent three, but an
        // author may indent deeper and pulldown-cmark still nests the fence).
        let source = "1. item\n\n    ~~~rust\n    let x = 1;\n    ~~~\n";
        let fence_start = source.find("~~~").unwrap();
        let fence_end = source.rfind("~~~").unwrap() + 3;
        let (payload, info, opening, closing) =
            fenced_payload_ranges(source, fence_start..fence_end, '`', '~')
                .expect("nested tilde fence");
        assert_eq!(&source[opening], "~~~");
        assert_eq!(info.map(|range| &source[range]), Some("rust"));
        assert_eq!(&source[payload], "    let x = 1;\n");
        assert_eq!(&source[closing], "~~~");
    }

    #[test]
    fn top_level_fence_with_fence_like_payload_line_stays_unresolved() {
        // CommonMark: a closing fence may be indented at most three spaces, so
        // this block is unclosed and its "fence-looking" line is payload. The
        // lenient nested-list scan must not fabricate a closing fence here.
        let source = "  ```\n    ```\n";
        let fence_start = source.find("```").unwrap();
        assert!(fenced_payload_ranges(source, fence_start..source.len(), '`', '~').is_none());
        // Same guard for a column-zero unclosed fence.
        let source = "```\n    ```\n";
        assert!(fenced_payload_ranges(source, 0..source.len(), '`', '~').is_none());
    }

    #[test]
    fn nested_code_partition_boundary_keeps_exact_caret_positions() {
        let source = "- first item\n    \n    ```\n    export A=1\n    ```\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let item = &blocks[0];
        assert!(matches!(item.kind, VisualBlockKind::ListItem { .. }));
        // The truncated item owns its trailing whitespace line through exact
        // runs, and every display caret in the item row round-trips to source.
        assert!(item.source_range.end > source.find("first item").unwrap());
        let cursor = source.find("first item").unwrap() + 3;
        let projection = build_visual_projection(source, item, cursor..cursor, cursor);
        assert_eq!(projection.text.trim_end(), "first item");
        for display in 0..=projection.text.len() {
            if !projection.text.is_char_boundary(display) {
                continue;
            }
            let source_offset = projection.source_for_display(display);
            assert!(
                item.source_range.contains(&source_offset)
                    || source_offset == item.source_range.end
            );
        }
    }

    #[test]
    fn minimax_fixture_list_nested_code_renders_without_raw_boxes() {
        // Structure reported in 大模型服务API和账号信息2026Q3.md (MiniMax
        // section): bullets whose indented continuations hold fenced blocks.
        let source = "### 国内版MiniMax开放平台（Token Plan）\n\n有效期至：**03/26/2027**\n对应账号：`willmove@163.com`\n\n**Token Plan API Key：**\n\n```\nsk-cp-example\n```\n\n\n- 推荐使用 Anthropic API 兼容，具体查看：[Anthropic SDK](https://platform.minimaxi.com/docs/api-reference/text-anthropic-api)\n    \n    ```\n    export ANTHROPIC_BASE_URL=https://api.minimaxi.com/anthropic\n    export ANTHROPIC_API_KEY=${YOUR_API_KEY}\n    ```\n    \n- 使用 OpenAI API 兼容，具体查看：[OpenAI SDK](https://platform.minimaxi.com/docs/api-reference/text-openai-api)\n    \n    ```\n    export OPENAI_BASE_URL=https://api.minimaxi.com/v1\n    export OPENAI_API_KEY=${YOUR_API_KEY}\n    ```\n\n### 海外 MiniMax API keys\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();

        assert_eq!(blocks.first().unwrap().source_range.start, 0);
        assert_eq!(blocks.last().unwrap().source_range.end, source.len());
        assert!(
            blocks
                .windows(2)
                .all(|pair| pair[0].source_range.end == pair[1].source_range.start)
        );
        assert!(blocks.iter().all(|block| {
            !matches!(block.kind, VisualBlockKind::Unsupported) && block.source_island.is_none()
        }));

        let code_rows = blocks
            .iter()
            .filter(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .collect::<Vec<_>>();
        assert_eq!(code_rows.len(), 3);
        assert!(
            code_rows
                .iter()
                .all(|row| matches!(row.editor, Some(VisualBlockEditor::Code { .. })))
        );

        let rendered = blocks
            .iter()
            .map(|block| {
                let cursor = block
                    .editable_runs
                    .first()
                    .map_or(block.source_range.end, |run| run.content_range.start);
                build_visual_projection(source, block, cursor..cursor, cursor).text
            })
            .collect::<String>();
        for phrase in [
            "推荐使用 Anthropic API 兼容",
            "export ANTHROPIC_BASE_URL=https://api.minimaxi.com/anthropic",
            "使用 OpenAI API 兼容",
            "export OPENAI_BASE_URL=https://api.minimaxi.com/v1",
        ] {
            assert_eq!(rendered.matches(phrase).count(), 1, "{phrase}: {rendered}");
        }
        assert!(
            !rendered.contains("[Anthropic SDK](https://platform.minimaxi.com"),
            "link syntax stays hidden: {rendered}"
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
        // The parser substitutes U+FFFD for the surrogate code point, a
        // transformation the projection prover intentionally cannot prove.
        let doc = MarkdownDocument::from_text("A &#xD800; B");
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
            info,
            closing_fence,
        }) = block.editor
        else {
            panic!("ordinary closed fence should have direct metadata");
        };
        assert_eq!(&source[opening_fence], "~~~");
        assert_eq!(&source[payload.source_range], "  let 名称 = 1;\r\n\r\n");
        assert_eq!(&source[info_range.expect("info range")], "rust extra");
        assert_eq!(&source[info.source_range], "rust");
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
            info,
            closing_fence,
        } = block
            .editor
            .expect("diagram fence should have a payload editor")
        else {
            panic!("expected Code editor for diagram fence");
        };
        assert_eq!(&source[opening_fence], "```");
        assert_eq!(
            &source[payload.source_range.clone()],
            "flowchart LR\nA --> B\n"
        );
        assert_eq!(&source[info_range.expect("info range")], "mermaid");
        assert_eq!(&source[info.source_range], "mermaid");
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
    fn inline_image_editor_proves_exact_spans_and_refuses_ambiguous_forms() {
        let source = "![替代\\]文本](images/a\\)b.png '标题')";
        let block = MarkdownDocument::from_text(source)
            .visual_blocks()
            .remove(0);
        let VisualBlockKind::Image { alt, url, title } = &block.kind else {
            panic!("expected image block, got {:?}", block.kind);
        };
        assert_eq!(alt, "替代]文本");
        assert_eq!(url, "images/a)b.png");
        assert_eq!(title.as_deref(), Some("标题"));
        let Some(VisualBlockEditor::Image { payload }) = &block.editor else {
            panic!("single-line inline image should carry an Image payload editor");
        };
        assert_eq!(payload.source_range, block.source_range);
        // Editor-driven blocks drop the conservative source-island kind,
        // matching the other payload-editor blocks.
        assert!(block.source_island.is_none());

        for ambiguous in [
            // Reference-style: no proven destination parentheses.
            "![alt][asset]\n\n[asset]: image.png",
            // Multiline: provable by the parser but outside the payload proof.
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

    fn assert_complete_disjoint_coverage(blocks: &[crate::VisualBlock], source: &str) {
        assert!(!blocks.is_empty(), "expected visual rows for {source:?}");
        assert_eq!(blocks[0].source_range.start, 0);
        assert_eq!(blocks.last().unwrap().source_range.end, source.len());
        for pair in blocks.windows(2) {
            assert!(
                pair[0].source_range.end <= pair[1].source_range.start,
                "overlapping visual rows {:?} then {:?}",
                pair[0].source_range,
                pair[1].source_range
            );
            assert_eq!(
                pair[0].source_range.end, pair[1].source_range.start,
                "gap between visual rows {:?} and {:?}",
                pair[0].source_range, pair[1].source_range
            );
        }
    }

    fn content_kinds(blocks: &[crate::VisualBlock]) -> Vec<&VisualBlockKind> {
        blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .map(|block| &block.kind)
            .collect()
    }

    #[test]
    fn mixed_paragraph_image_without_blank_line_keeps_inline_atom() {
        let source = "**已订阅Google AI Pro**\n![image.png](https://example.com/a.png)";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, source);

        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "kinds: {:?}", content_kinds(&blocks));
        assert!(
            matches!(content[0].kind, VisualBlockKind::Paragraph),
            "mixed image should stay in the parent paragraph, got {:?}",
            content[0].kind
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .any(|run| run.html_image.as_ref().is_some_and(|image| {
                    image.url == "https://example.com/a.png" && image.alt == "image.png"
                })),
            "expected an inline markdown image atom, got {:?}",
            content[0].editable_runs
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .any(|run| run.visible_text.contains("已订阅Google AI Pro")
                    && run.html_image.is_none()),
            "leading prose missing: {:?}",
            content[0].editable_runs
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .filter(|run| run.html_image.is_none())
                .all(|run| !run.visible_text.contains("![")
                    && !run.visible_text.contains("https://example.com/a.png")),
            "image syntax leaked into prose runs: {:?}",
            content[0].editable_runs
        );
        assert_ne!(
            content[0].source_island,
            Some(VisualSourceIslandKind::Unsupported)
        );
    }

    #[test]
    fn same_line_and_multiple_inline_images_stay_in_one_row() {
        let source = "hello ![alt](url) world";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, source);
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "kinds: {:?}", content_kinds(&blocks));
        assert!(matches!(content[0].kind, VisualBlockKind::Paragraph));
        assert!(content[0].editable_runs.iter().any(|run| {
            run.html_image
                .as_ref()
                .is_some_and(|image| image.url == "url")
        }));
        assert!(
            content[0]
                .editable_runs
                .iter()
                .any(|run| run.visible_text.contains("hello") && run.html_image.is_none())
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .any(|run| run.visible_text.contains("world") && run.html_image.is_none())
        );

        let source = "a ![one](one.png) b ![two](two.png) c";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, source);
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "kinds: {:?}", content_kinds(&blocks));
        let images = content[0]
            .editable_runs
            .iter()
            .filter_map(|run| run.html_image.as_ref().map(|image| image.url.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(images, ["one.png", "two.png"]);
        assert!(
            content
                .iter()
                .all(|block| { block.source_island != Some(VisualSourceIslandKind::Unsupported) })
        );
    }

    #[test]
    fn image_only_and_blank_line_separated_images_stay_unpartitioned() {
        let source = "![solo](solo.png)";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, source);
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "kinds: {:?}", content_kinds(&blocks));
        assert!(matches!(content[0].kind, VisualBlockKind::Image { .. }));

        let source = "Intro\n\n![alt](url)";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, source);
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 2, "kinds: {:?}", content_kinds(&blocks));
        assert!(matches!(content[0].kind, VisualBlockKind::Paragraph));
        assert!(matches!(content[1].kind, VisualBlockKind::Image { .. }));
        assert!(content[0].source_range.end <= content[1].source_range.start);
    }

    #[test]
    fn quoted_paragraph_with_nested_image_keeps_quote_context() {
        let source = "> text\n> ![alt](url)";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, source);
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "kinds: {:?}", content_kinds(&blocks));
        assert!(
            content[0].editable_runs.iter().any(|run| run
                .html_image
                .as_ref()
                .is_some_and(|image| image.url == "url")),
            "expected an inline image atom in the quoted paragraph, got {:?}",
            content_kinds(&blocks)
        );
        assert!(content.iter().all(|block| block.quote_context.is_some()));
        assert!(
            content
                .iter()
                .all(|block| { block.source_island != Some(VisualSourceIslandKind::Unsupported) })
        );
    }

    #[test]
    fn leading_same_line_image_plus_trailing_prose_stays_inline() {
        let source = "![image.png](https://example.com/a.png)和其他瀚博半导体商标均为瀚博。";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, source);

        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "kinds: {:?}", content_kinds(&blocks));
        assert!(
            matches!(content[0].kind, VisualBlockKind::Paragraph),
            "leading image plus trailing prose should stay one paragraph, got {:?}",
            content[0].kind
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .any(|run| run.html_image.as_ref().is_some_and(|image| {
                    image.url == "https://example.com/a.png" && image.alt == "image.png"
                })),
            "expected an inline markdown image atom: {:?}",
            content[0].editable_runs
        );
        assert!(
            content
                .iter()
                .all(|block| block.source_island != Some(VisualSourceIslandKind::Unsupported)),
            "leading image must not overlap into an Unsupported island"
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .filter(|run| run.html_image.is_none())
                .all(|run| !run.visible_text.contains("![")
                    && !run.visible_text.contains("https://example.com/a.png")
                    && !run.visible_text.contains("image.png")),
            "image syntax leaked into trailing prose: {:?}",
            content[0].editable_runs
        );
        assert!(
            content[0].editable_runs.iter().any(|run| run
                .visible_text
                .contains("和其他瀚博半导体商标均为瀚博。")
                && run.html_image.is_none()),
            "trailing prose missing: {:?}",
            content[0].editable_runs
        );
    }

    #[test]
    fn leading_image_in_heading_and_quoted_paragraph_stays_inline() {
        let heading = "# ![alt](url) heading rest";
        let blocks = MarkdownDocument::from_text(heading).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, heading);
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "kinds: {:?}", content_kinds(&blocks));
        assert!(
            matches!(content[0].kind, VisualBlockKind::Heading { .. }),
            "expected a single heading row, got {:?}",
            content_kinds(&blocks)
        );
        assert!(content[0].editable_runs.iter().any(|run| {
            run.html_image
                .as_ref()
                .is_some_and(|image| image.url == "url")
        }));
        assert!(
            content
                .iter()
                .all(|block| block.source_island != Some(VisualSourceIslandKind::Unsupported))
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .filter(|run| run.html_image.is_none())
                .all(|run| !run.visible_text.contains("![") && !run.visible_text.contains("url")),
            "image syntax leaked into heading runs"
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .any(|run| run.visible_text.contains("heading rest") && run.html_image.is_none())
        );

        let quoted = "> ![alt](url) quoted rest";
        let blocks = MarkdownDocument::from_text(quoted).visual_blocks();
        assert_complete_disjoint_coverage(&blocks, quoted);
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "kinds: {:?}", content_kinds(&blocks));
        assert!(
            content[0].editable_runs.iter().any(|run| run
                .html_image
                .as_ref()
                .is_some_and(|image| image.url == "url")),
            "expected an inline image atom in the quote, got {:?}",
            content_kinds(&blocks)
        );
        assert!(content.iter().all(|block| block.quote_context.is_some()));
        assert!(
            content
                .iter()
                .all(|block| block.source_island != Some(VisualSourceIslandKind::Unsupported))
        );
        assert!(
            content[0]
                .editable_runs
                .iter()
                .filter(|run| run.html_image.is_none())
                .all(|run| !run.visible_text.contains("![") && !run.visible_text.contains("url")),
            "image syntax leaked into quoted prose"
        );
    }

    #[test]
    fn list_item_inline_markdown_image_stays_unpartitioned() {
        let source = "- hello ![alt](url) world";
        let blocks = MarkdownDocument::from_text(source).visual_blocks();
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert!(
            content
                .iter()
                .any(|block| matches!(block.kind, VisualBlockKind::ListItem { .. })),
            "list item row must remain, got {:?}",
            content_kinds(&blocks)
        );
        assert!(
            content
                .iter()
                .all(|block| !matches!(block.kind, VisualBlockKind::Paragraph)),
            "inline image must not emit a continuation paragraph (second bullet), got {:?}",
            content_kinds(&blocks)
        );
        assert!(
            content[0].editable_runs.iter().any(|run| run
                .html_image
                .as_ref()
                .is_some_and(|image| image.url == "url")),
            "list item should keep the markdown image as an inline atom, got {:?}",
            content[0].editable_runs
        );
        assert!(
            content
                .iter()
                .all(|block| block.source_island != Some(VisualSourceIslandKind::Unsupported))
        );
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
    fn block_image_editor_covers_exactly_the_authored_span() {
        for source in [
            "![alt](pic.png)",
            "![A\\]lt](<note.assets/a b.png> \"Caption {width=50 align=right}\")",
            "![alt](pic.png \"title\")\r\n",
        ] {
            let block = MarkdownDocument::from_text(source)
                .visual_blocks()
                .into_iter()
                .find(|block| matches!(block.kind, VisualBlockKind::Image { .. }))
                .unwrap_or_else(|| panic!("expected block image for {source:?}"));
            let Some(VisualBlockEditor::Image { payload }) = block.editor else {
                panic!("proven image span should carry an Image editor for {source:?}");
            };
            assert_eq!(payload.kind, VisualEditorFieldKind::ImageSource);
            assert_eq!(payload.source_range, block.source_range);
            let authored = &source[payload.source_range.clone()];
            assert!(authored.starts_with("![") && authored.ends_with(')'));
            assert!(block.source_island.is_none());
        }
    }

    #[test]
    fn unprovable_image_spans_keep_the_source_island() {
        // Reference-style and multiline image forms cannot be proven by the
        // whole-span byte scan, so they must not get an Image payload editor.
        for source in [
            "![alt][ref]\n\n[ref]: pic.png",
            "![alt](image.png\n \"title\")",
        ] {
            let block = MarkdownDocument::from_text(source)
                .visual_blocks()
                .into_iter()
                .find(|block| matches!(block.kind, VisualBlockKind::Image { .. }));
            if let Some(block) = block {
                assert!(
                    !matches!(block.editor, Some(VisualBlockEditor::Image { .. })),
                    "unprovable image span must not gain an Image editor for {source:?}"
                );
            }
        }
    }

    #[test]
    fn code_info_field_covers_first_token_or_inserts_after_the_fence() {
        // Bare fence: empty insertion range directly after the opening fence.
        let bare = "```\nlet x = 1;\n```";
        let doc = MarkdownDocument::from_text(bare);
        let block = doc
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .expect("code block");
        let Some(VisualBlockEditor::Code {
            opening_fence, info, ..
        }) = block.editor
        else {
            panic!("bare fence should have direct metadata");
        };
        assert!(info.source_range.is_empty());
        assert_eq!(info.source_range.start, opening_fence.end);
        assert_eq!(info.kind, VisualEditorFieldKind::CodeInfo);

        // Multi-token info strings edit only the first token, LF and CRLF.
        for (source, token) in [
            ("```rust extra\nlet x = 1;\n```", "rust"),
            ("```rust extra\r\nlet x = 1;\r\n```", "rust"),
            ("```toml\r\nx = 1\r\n```", "toml"),
        ] {
            let block = MarkdownDocument::from_text(source)
                .visual_blocks()
                .into_iter()
                .find(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
                .expect("code block");
            let Some(VisualBlockEditor::Code { info, .. }) = block.editor else {
                panic!("fence should have direct metadata for {source:?}");
            };
            assert_eq!(&source[info.source_range.clone()], token);
        }
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
            "![Markion logo](assets/markion.png",
            "- [ ] Export when ready",
            "| Syntax | Example | Purpose |",
            "<table",
            "colspan=\"2\"",
            "rowspan=\"2\"",
            "<kbd>Ctrl</kbd>",
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
    fn welcome_html_table_parses_as_a_spanned_grid() {
        let source = crate::DEFAULT_WELCOME_MARKDOWN;
        let start = source.find("<table>").expect("welcome HTML table");
        let end = source
            .find("</table>")
            .map(|index| index + "</table>".len())
            .expect("welcome HTML table end");
        let parts = crate::html_preview_parts(&source[start..end]);
        let crate::HtmlPreviewPart::Table { grid } =
            parts.first().expect("HTML table preview part")
        else {
            panic!("welcome HTML table must parse as a grid, got {parts:?}");
        };
        assert!(grid.has_rowspan);
        assert!(grid.columns >= 3);
        assert!(grid.rows.len() >= 3);
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
            source[footnote_def.source_range.clone()].contains("Links can point to project pages"),
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
                .any(|run| run.visible_text == "Markion repository" && !run.conservative_fallback),
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
            footnote_para.editable_runs.iter().any(|run| matches!(
                &run.navigation,
                Some(VisualNavigationTarget::Url(url))
                    if url == "https://github.com/willmove/markion"
            )),
            "inline link must expose a URL navigation target"
        );
    }

    #[test]
    fn html_block_maps_to_rendered_visual_block_not_source_island() {
        // A raw HTML table must become a rendered VisualBlockKind::Html (not
        // Unsupported + a source island), so Visual Edit shows the rendered
        // grid instead of a verbatim source box.
        let source = "<table>\n<tr><th>A</th><th>B</th></tr>\n\
<tr><td rowspan=\"2\">x</td><td>1</td></tr>\n\
<tr><td>2</td></tr>\n\
</table>\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();

        let html_block = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Html { .. }))
            .expect("HTML block should map to VisualBlockKind::Html");

        // The source island must be cleared so the `always_source` /
        // `focused_conservative` gates in visual_block_view do not force the
        // raw-source box when the block is unfocused.
        assert!(
            html_block.source_island.is_none(),
            "HTML visual block must not carry a source island"
        );
        assert!(
            matches!(html_block.editor, Some(VisualBlockEditor::Html { .. })),
            "HTML visual block must expose a source payload editor"
        );
        match &html_block.kind {
            VisualBlockKind::Html { html } => {
                assert!(html.contains("<table"));
                assert!(html.contains("rowspan"));
            }
            other => panic!("expected Html kind, got {other:?}"),
        }

        // No HTML block should remain as the legacy Unsupported mapping.
        assert!(
            blocks
                .iter()
                .all(|block| !matches!(block.kind, VisualBlockKind::Unsupported)),
            "no block should fall back to Unsupported"
        );
    }

    #[test]
    fn visual_edit_renders_standalone_img_line_as_html_block() {
        let doc = MarkdownDocument::from_text("<img src=\"a.png\" alt=\"A\">\n\nText");
        let blocks = doc.visual_blocks_shared();

        match &blocks[0].kind {
            VisualBlockKind::Html { html } => assert!(html.contains("a.png")),
            other => panic!("expected Html kind, got {other:?}"),
        }
        assert!(
            blocks[0].source_island.is_none(),
            "rendered HTML block must not carry a source island"
        );
    }

    #[test]
    fn visual_edit_renders_inline_html_image_as_exact_run() {
        let source = "Hello <img src=\"badge.png\" alt=\"Badge\"> world";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = blocks.first().expect("paragraph block");

        assert!(matches!(block.kind, VisualBlockKind::Paragraph));
        assert!(
            block.source_island.is_none(),
            "img-only inline HTML must not island the whole block"
        );

        let tag_start = source.find("<img").expect("tag start");
        let tag_end = source.find('>').expect("tag end") + 1;
        let run = block
            .editable_runs
            .iter()
            .find(|run| run.html_image.is_some())
            .expect("image run");
        assert_eq!(run.source_range, tag_start..tag_end);
        assert_eq!(run.content_range, tag_start..tag_end);
        assert_eq!(run.visible_text, &source[tag_start..tag_end]);
        assert!(!run.conservative_fallback);
        let image = run.html_image.as_ref().unwrap();
        assert_eq!(image.url, "badge.png");
        assert_eq!(image.alt, "Badge");
        assert!(image.title.is_none());

        assert!(
            block
                .editable_runs
                .iter()
                .any(|run| run.visible_text.contains("Hello") && run.html_image.is_none())
        );
        assert!(
            block
                .editable_runs
                .iter()
                .any(|run| run.visible_text.contains("world") && run.html_image.is_none())
        );
        assert!(
            block
                .reveal_groups
                .iter()
                .any(|group| group.kind == VisualRevealKind::HtmlImage
                    && group.source_range == (tag_start..tag_end))
        );
    }

    #[test]
    fn visual_edit_reveals_inline_html_image_source_on_caret_entry() {
        let source = "Hello <img src=\"badge.png\" alt=\"Badge\"> world";
        let doc = MarkdownDocument::from_text(source);
        let block = doc.visual_blocks_shared()[0].clone();
        let tag_start = source.find("<img").expect("tag start");
        let tag_end = source.find('>').expect("tag end") + 1;

        let inside = tag_start + 3;
        let projection = build_visual_projection(source, &block, inside..inside, inside);
        assert_eq!(projection.revealed_source_ranges, vec![tag_start..tag_end]);
        assert!(
            projection
                .text
                .contains("<img src=\"badge.png\" alt=\"Badge\">")
        );

        // The caret resting right after the tag keeps the source editable,
        // mirroring the math delimiter-group endpoint rule.
        let projection = build_visual_projection(source, &block, tag_end..tag_end, tag_end);
        assert_eq!(projection.revealed_source_ranges, vec![tag_start..tag_end]);

        let outside = 2;
        let projection = build_visual_projection(source, &block, outside..outside, outside);
        assert!(projection.revealed_source_ranges.is_empty());
        assert!(
            projection
                .text
                .contains("<img src=\"badge.png\" alt=\"Badge\">"),
            "unfocused projection keeps the authored tag as one rendered piece"
        );
    }

    #[test]
    fn visual_edit_inline_html_images_span_prose_contexts() {
        // HTML-only list/quote images are nested Html blocks (P0). Mixed
        // prose plus `<img>` stays on the inline atom path.
        let doc = MarkdownDocument::from_text("- <img src=\"dot.png\" alt=\"dot\">");
        let blocks = doc.visual_blocks_shared();
        assert!(
            blocks.iter().any(|block| {
                matches!(block.kind, VisualBlockKind::Html { .. }) && block.source_island.is_none()
            }),
            "html-only list image is a nested Html block, got {:?}",
            blocks.iter().map(|b| &b.kind).collect::<Vec<_>>()
        );

        let doc = MarkdownDocument::from_text("- hello <img src=\"mix.png\" alt=\"m\">");
        let blocks = doc.visual_blocks_shared();
        let item = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::ListItem { .. }))
            .expect("mixed list item");
        assert!(item.source_island.is_none());
        assert!(
            item.editable_runs
                .iter()
                .any(|run| run.html_image.is_some())
        );

        let doc = MarkdownDocument::from_text("> <img src=\"q.png\" alt=\"q\">");
        let blocks = doc.visual_blocks_shared();
        assert!(
            blocks.iter().any(|block| {
                matches!(block.kind, VisualBlockKind::Html { .. }) && block.source_island.is_none()
            }),
            "html-only quote image is an Html child, got {:?}",
            blocks.iter().map(|b| &b.kind).collect::<Vec<_>>()
        );

        let doc = MarkdownDocument::from_text("# Title <img src=\"i.png\">");
        let blocks = doc.visual_blocks_shared();
        assert!(matches!(blocks[0].kind, VisualBlockKind::Heading { .. }));
        assert!(blocks[0].source_island.is_none());
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.html_image.is_some())
        );

        // A GFM table cell shows the flattened alt text (Read-mode parity)
        // instead of collapsing the whole table into a source island.
        let doc = MarkdownDocument::from_text(
            "| a | b |\n|---|---|\n| <img src=\"g.png\" alt=\"G\"> | x |",
        );
        let blocks = doc.visual_blocks_shared();
        assert!(matches!(blocks[0].kind, VisualBlockKind::Table { .. }));
        assert!(blocks[0].source_island.is_none());
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.html_image.is_some())
        );
        match &blocks[0].kind {
            VisualBlockKind::Table { rows, .. } => {
                assert_eq!(rows[1][0].text, "G");
            }
            other => panic!("expected table, got {other:?}"),
        }
    }

    #[test]
    fn inline_html_br_resolves_atomically_to_tag_boundaries() {
        let source = "one<br>two";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let projection =
            build_visual_projection_with_marked_range(source, &blocks[0], 0..0, 0, None);
        assert_eq!(projection.text, "one\ntwo");
        // The break is one display char mapped onto the tag bytes, so the
        // boundaries on both sides resolve to the tag's source edges and no
        // interior display position can exist.
        let before_break = projection.boundary_candidates(3);
        assert_eq!(before_break.upstream_source, 3);
        assert_eq!(before_break.downstream_source, 3);
        let after_break = projection.boundary_candidates(4);
        assert_eq!(after_break.upstream_source, 7);
        assert_eq!(after_break.downstream_source, 7);
        // Mid-tag source offsets clamp to the atom's display end.
        assert_eq!(projection.display_for_source(5), Some(4));
        // Activating the tag boundary reveals the authored source in place.
        let revealed = build_visual_projection_with_marked_range(source, &blocks[0], 0..0, 3, None);
        assert_eq!(revealed.text, "one<br>two");
    }

    #[test]
    fn visual_edit_renders_supported_inline_html_with_hidden_tags() {
        // Pure supported style pairs render as styled prose with hidden tag
        // markers and no whole-block HTML source island.
        let source = "text <em>em</em> more";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        assert_eq!(block.source_island, None, "supported pairs stay visual");
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        let em_run = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "em")
            .expect("tag content renders as a run");
        assert!(em_run.style.italic);
        assert_eq!(&source[em_run.content_range.clone()], "em");
        let groups: Vec<(VisualRevealKind, &str)> = block
            .reveal_groups
            .iter()
            .map(|group| (group.kind, &source[group.source_range.clone()]))
            .collect();
        assert_eq!(
            groups,
            vec![(VisualRevealKind::InlineHtml, "<em>em</em>")],
            "the complete element source is one reveal group"
        );
        // Only the tag bytes stay hidden.
        let markers: Vec<&str> = block
            .marker_ranges
            .iter()
            .map(|range| &source[range.clone()])
            .collect();
        assert_eq!(markers, vec!["<em>", "</em>"]);
        // Unfocused projection hides the tags; a caret inside the element
        // reveals the complete authored source in place.
        let unfocused = build_visual_projection_with_marked_range(source, block, 0..0, 0, None);
        assert_eq!(unfocused.text, "text em more");
        let em_byte = source.find("em>").unwrap() + 2;
        let focused = build_visual_projection_with_marked_range(source, block, 0..0, em_byte, None);
        assert_eq!(focused.text, "text <em>em</em> more");
    }

    #[test]
    fn inline_html_styles_compose_with_markdown_formatting() {
        let source = "<em>a *b* c</em> and <strong>md **bold**</strong>";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        let italic = |needle: &str| {
            block
                .editable_runs
                .iter()
                .find(|run| run.visible_text == needle)
                .unwrap_or_else(|| panic!("missing run {needle}"))
                .style
                .italic
        };
        // Content after an inner Markdown pair keeps the enclosing HTML style.
        assert!(italic("a "));
        assert!(italic("b"));
        assert!(italic(" c"));
        assert!(
            !italic(" and "),
            "content past the closing tag drops the style"
        );
        // HTML strong and Markdown bold compose.
        let bold_run = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "bold")
            .expect("nested markdown bold run");
        assert!(bold_run.style.bold);
    }

    #[test]
    fn inline_html_br_renders_authored_line_break_run() {
        let source = "one<br/>two <br> three<br />four";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        assert_eq!(block.source_island, None);
        let breaks: Vec<&str> = block
            .editable_runs
            .iter()
            .filter(|run| run.visible_text == "\n")
            .map(|run| &source[run.content_range.clone()])
            .collect();
        assert_eq!(breaks, vec!["<br/>", "<br>", "<br />"]);
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        let br_groups = block
            .reveal_groups
            .iter()
            .filter(|group| group.kind == VisualRevealKind::InlineHtml)
            .count();
        assert_eq!(br_groups, 3);
    }

    #[test]
    fn visual_edit_keeps_conservative_fallback_for_unsupported_inline_html() {
        // Unsupported inline HTML stays mixed: conservative tag runs, no
        // whole-block island, focused and unfocused alike.
        let doc = MarkdownDocument::from_text("text <a href=\"u\">link</a> more");
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks[0].source_island, None);
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.conservative_fallback && run.visible_text.contains("<a")),
            "unsupported tags stay as conservative atoms"
        );
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| !run.conservative_fallback && run.visible_text.contains("text")),
            "surrounding prose stays rendered"
        );

        // Ignorable class/id/clear on supported names keep the styled path.
        let source = "x <em class=\"q\">y</em> z";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks[0].source_island, None);
        let em_run = blocks[0]
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "y")
            .expect("classed em still renders");
        assert!(em_run.style.italic);
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );

        // An unclosed supported tag demotes inner runs but does not island.
        let doc = MarkdownDocument::from_text("unclosed <em>em text");
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        assert_eq!(block.source_island, None);
        assert!(
            block
                .editable_runs
                .iter()
                .any(|run| run.conservative_fallback && run.visible_text == "<em>"),
            "the unclosed tag itself stays visible as a conservative run"
        );
        assert!(
            block
                .editable_runs
                .iter()
                .any(|run| run.conservative_fallback && run.visible_text.contains("em text")),
            "content inside the unclosed element is demoted"
        );

        // A stray close without an open also stays mixed.
        let doc = MarkdownDocument::from_text("stray </strong> close");
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks[0].source_island, None);
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.conservative_fallback && run.visible_text.contains("</strong>"))
        );
    }

    #[test]
    fn inline_html_image_mixed_blocks_keep_image_runs() {
        // `<br>` and `<img>` are both supported now, so their block stays
        // visual with both rendered.
        let doc = MarkdownDocument::from_text("Hello <br> <img src=\"x.png\"> world");
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks[0].source_island, None);
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.html_image.is_some()),
            "the image run renders alongside the line break"
        );
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.visible_text == "\n")
        );

        // The README badge pattern (`<a href><img></a>`) emits the image run
        // plus conservative source runs for the unsupported `<a>` wrappers.
        let doc = MarkdownDocument::from_text("plain <a href=\"u\"><img src=\"x.png\"></a> end");
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks[0].source_island, None);
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.html_image.is_some()),
            "a-wrapped image keeps its image run"
        );
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.conservative_fallback && run.visible_text.contains("<a")),
            "a-wrapped image keeps conservative source runs for the wrappers"
        );

        // An unclosed `<em>` before an image demotes the styled run while the
        // image run survives; the paragraph stays mixed, not an island.
        let doc = MarkdownDocument::from_text("bad <em>styled <img src=\"x.png\"> tail");
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        assert_eq!(block.source_island, None);
        assert!(
            block
                .editable_runs
                .iter()
                .any(|run| run.html_image.is_some()),
            "image atoms are never demoted by the pairing failure"
        );
    }

    #[test]
    fn unsupported_inline_html_stays_mixed_layout() {
        let source = "Hello <span>x</span> world";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].source_island, None);
        assert!(
            blocks[0]
                .editable_runs
                .iter()
                .any(|run| run.conservative_fallback && run.visible_text.contains("<span>"))
        );
        let visible: String = blocks[0]
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert!(visible.contains("Hello"));
        assert!(visible.contains("world"));
    }

    #[test]
    fn residual_named_entities_and_multi_codepoint_decode() {
        let source = "a &ordf; b &NotEqualTilde; c";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback)
        );
        let visible: String = block
            .editable_runs
            .iter()
            .map(|run| run.visible_text.as_str())
            .collect();
        assert_eq!(visible, "a \u{00AA} b \u{2242}\u{0338} c");
        let multi = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text == "\u{2242}\u{0338}")
            .expect("multi-codepoint entity is one run");
        assert_eq!(&source[multi.content_range.clone()], "&NotEqualTilde;");
        assert_eq!(multi.source_range, multi.content_range);
    }

    #[test]
    fn angle_bracket_autolinks_use_link_reveal() {
        let source = "see <https://example.com> and <user@example.com>";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = &blocks[0];
        assert_eq!(block.source_island, None);
        assert!(
            block
                .editable_runs
                .iter()
                .all(|run| !run.conservative_fallback),
            "autolinks stay proven, got {:?}",
            block.editable_runs
        );
        let url_run = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text.contains("https://example.com"))
            .expect("url autolink visible");
        assert!(url_run.navigation.is_some());
        let email_run = block
            .editable_runs
            .iter()
            .find(|run| run.visible_text.contains("user@example.com"))
            .expect("email autolink visible");
        assert!(email_run.navigation.is_some());
        let kinds: Vec<_> = block
            .reveal_groups
            .iter()
            .map(|group| (group.kind, &source[group.source_range.clone()]))
            .collect();
        assert!(
            kinds
                .iter()
                .any(|(kind, slice)| *kind == VisualRevealKind::Link
                    && *slice == "<https://example.com>"),
            "url autolink reveal, got {kinds:?}"
        );
        assert!(
            kinds
                .iter()
                .any(|(kind, slice)| *kind == VisualRevealKind::Link
                    && *slice == "<user@example.com>"),
            "email autolink reveal, got {kinds:?}"
        );
    }

    fn paragraph_projection(source: &str) -> VisualProjection {
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let block = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .expect("paragraph block");
        let cursor = block
            .editable_runs
            .first()
            .map_or(block.source_range.start, |run| run.content_range.start);
        build_visual_projection(source, block, cursor..cursor, cursor)
    }

    #[test]
    fn word_selection_maps_plain_words_one_to_one() {
        let source = "hello world";
        let projection = paragraph_projection(source);
        assert_eq!(projection.text, "hello world");
        let range = projection.word_selection_range(7).expect("word range");
        assert_eq!(range, 6..11);
        assert_eq!(&source[range], "world");
    }

    #[test]
    fn word_selection_excludes_hidden_syntax_at_its_edges() {
        let source = "bold **word** tail";
        let projection = paragraph_projection(source);
        assert_eq!(projection.text, "bold word tail");
        let display = projection.text.find("word").expect("visible word") + 1;
        let range = projection
            .word_selection_range(display)
            .expect("content-only range");
        let content = source.match_indices("word").next().unwrap().0;
        assert_eq!(range, content..content + "word".len());
        assert_eq!(&source[range], "word");
    }

    #[test]
    fn word_selection_spans_hidden_syntax_inside_the_run() {
        // Source "bo**ld** now": the visible word "bold" spans the split
        // emphasis, so the selection covers "bo**ld" in one contiguous
        // source range while the trailing edge markers stay outside.
        let source = "bo**ld** now";
        let projection = paragraph_projection(source);
        assert_eq!(projection.text, "bold now");
        let display = projection.text.find("bold").expect("visible word") + 1;
        let range = projection
            .word_selection_range(display)
            .expect("contiguous range");
        assert_eq!(range, 0..6);
        assert_eq!(&source[range], "bo**ld");
    }

    #[test]
    fn word_selection_on_identity_math_text_selects_the_token() {
        // While an inline formula is projected as identity source text
        // (rendered-atom fallback), double click selects the token under the
        // pointer, and the delimiters select as single-character runs.
        let source = "value $x=1$ here";
        let projection = paragraph_projection(source);
        assert_eq!(projection.text, "value $x=1$ here");
        let range = projection.word_selection_range(7).expect("token range");
        assert_eq!(range, 7..8);
        assert_eq!(&source[range], "x");
        assert_eq!(
            projection.word_selection_range(8),
            Some(8..9),
            "the = operator is its own run"
        );
        assert_eq!(
            projection.word_selection_range(6),
            Some(6..7),
            "the opening delimiter is its own run"
        );
    }

    #[test]
    fn word_selection_inside_a_non_identity_segment_selects_the_atom_source() {
        // Display " 𝑥=1y " where display 1..8 is a rendered atom backed by
        // the source range 1..6 (e.g. `$x=1$`): a run whose edge falls
        // strictly inside the atom selects the atom's full source range.
        let projection = VisualProjection {
            text: " 𝑥=1y ".to_string(),
            segments: vec![
                VisualProjectionSegment {
                    display_range: 0..1,
                    source_range: 0..1,
                },
                VisualProjectionSegment {
                    display_range: 1..8,
                    source_range: 1..6,
                },
                VisualProjectionSegment {
                    display_range: 8..9,
                    source_range: 6..7,
                },
            ],
            spans: Vec::new(),
            revealed_source_ranges: Vec::new(),
            source_anchor: 0,
        };
        // '1' occupies display 6..7, strictly inside the atom.
        assert_eq!(projection.word_selection_range(6), Some(1..6));
        // The '𝑥' run starts at the atom's display edge and ends inside it.
        assert_eq!(projection.word_selection_range(1), Some(1..6));
        // 'y' at display 7..8 also resolves through the atom's source end.
        assert_eq!(projection.word_selection_range(7), Some(1..6));
        // The surrounding whitespace stays on its own identity segments.
        assert_eq!(projection.word_selection_range(0), Some(0..1));
        assert_eq!(projection.word_selection_range(8), Some(6..7));
    }

    #[test]
    fn word_selection_returns_none_when_nothing_resolves() {
        let projection = VisualProjection {
            text: String::new(),
            segments: Vec::new(),
            spans: Vec::new(),
            revealed_source_ranges: Vec::new(),
            source_anchor: 0,
        };
        assert_eq!(projection.word_selection_range(0), None);
    }
}
