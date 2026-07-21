//! Conservative source-ranged incremental derivation for Visual Edit.
//!
//! Markdown has document-wide constructs, so this module only reuses regions
//! whose boundaries and source lineage are unambiguous. Full derivation stays
//! the correctness fallback.

use std::{ops::Range, sync::Arc};

use crate::{
    Heading, InlineSpan, MarkdownDocument, MathSource, PreviewBlock, RichText, VisualBlock,
    VisualBlockEditor, VisualBlockPrefix, VisualEditorField, VisualInlineRun, VisualProjection,
    VisualRevealGroup, VisualStructuralEdit,
};

const MAX_PENDING_EDITS: usize = 32;
const MAX_PENDING_CHANGED_BYTES: usize = 256 * 1024;

/// One UTF-8-safe replacement over the previous source version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceEdit {
    pub old_range: Range<usize>,
    pub new_len: usize,
    pub old_version: u64,
    pub new_version: u64,
}

impl SourceEdit {
    pub fn new(
        old_source: &str,
        old_range: Range<usize>,
        new_len: usize,
        old_version: u64,
        new_version: u64,
    ) -> Option<Self> {
        (old_range.start <= old_range.end
            && old_range.end <= old_source.len()
            && old_source.is_char_boundary(old_range.start)
            && old_source.is_char_boundary(old_range.end)
            && new_version == old_version.wrapping_add(1))
        .then_some(Self {
            old_range,
            new_len,
            old_version,
            new_version,
        })
    }

    fn delta(&self) -> Option<isize> {
        let new_len = isize::try_from(self.new_len).ok()?;
        let old_len = isize::try_from(self.old_range.len()).ok()?;
        new_len.checked_sub(old_len)
    }

    /// Map a range only when the edit is wholly outside it. An insertion at a
    /// range start belongs to the suffix; an insertion at its end belongs to
    /// the prefix.
    pub fn map_unchanged_range(&self, range: &Range<usize>) -> Option<Range<usize>> {
        if range.end <= self.old_range.start {
            return Some(range.clone());
        }
        if range.start >= self.old_range.end {
            return shift_range(range, self.delta()?);
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PendingSourceEdits {
    Full,
    Incremental(Vec<SourceEdit>),
}

impl PendingSourceEdits {
    pub fn record(&mut self, edit: SourceEdit) {
        let Self::Incremental(edits) = self else {
            return;
        };
        let consecutive = edits
            .last()
            .map_or(true, |previous| previous.new_version == edit.old_version);
        let changed_bytes = edits.iter().try_fold(0usize, |total, current| {
            total
                .checked_add(current.old_range.len())?
                .checked_add(current.new_len)
        });
        if !consecutive
            || edits.len() >= MAX_PENDING_EDITS
            || changed_bytes
                .and_then(|total| total.checked_add(edit.old_range.len() + edit.new_len))
                .is_none_or(|total| total > MAX_PENDING_CHANGED_BYTES)
        {
            *self = Self::Full;
            return;
        }
        edits.push(edit);
    }

    pub fn reset_incremental(&mut self) {
        *self = Self::Incremental(Vec::new());
    }

    pub fn edits(&self) -> Option<&[SourceEdit]> {
        match self {
            Self::Full => None,
            Self::Incremental(edits) => Some(edits),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DerivationCounters {
    pub parsed_regions: usize,
    pub reused_regions: usize,
    pub full_fallbacks: usize,
}

#[derive(Debug, Clone)]
struct CachedRegion {
    text: Arc<str>,
    blocks: Arc<Vec<PreviewBlock>>,
    headings: Arc<Vec<Heading>>,
}

#[derive(Debug, Clone)]
pub(crate) struct SourceMappedCache {
    pub version: u64,
    pub source: Arc<str>,
    regions: Vec<CachedRegion>,
    pub blocks: Arc<Vec<PreviewBlock>>,
    pub headings: Vec<Heading>,
    pub counters: DerivationCounters,
}

impl SourceMappedCache {
    pub fn derive_full(source: &str, version: u64) -> Self {
        let (blocks, headings) = MarkdownDocument::derive_preview_and_outline(source);
        Self::seed_from_full(source, version, blocks, headings, 1)
    }

    pub fn from_cached_full(
        source: &str,
        version: u64,
        blocks: Arc<Vec<PreviewBlock>>,
        headings: Vec<Heading>,
    ) -> Self {
        let parts = split_regions(source);
        let regions = partition_full_derivation(&parts, &blocks, &headings).unwrap_or_else(|| {
            vec![CachedRegion {
                text: Arc::from(source),
                blocks: blocks.clone(),
                headings: Arc::new(headings.clone()),
            }]
        });
        Self {
            version,
            source: Arc::from(source),
            regions,
            blocks,
            headings,
            counters: DerivationCounters {
                parsed_regions: 0,
                reused_regions: 0,
                full_fallbacks: 0,
            },
        }
    }

    pub fn update(previous: &Self, source: &str, version: u64, edits: &[SourceEdit]) -> Self {
        if edits.is_empty() && previous.version == version && previous.source.as_ref() == source {
            return previous.clone();
        }
        if !valid_edit_chain(previous, source, version, edits)
            || requires_full_parse(previous.source.as_ref())
            || requires_full_parse(source)
        {
            return Self::full_fallback(source, version, previous.counters);
        }

        let parts = split_regions(source);
        if parts.is_empty() {
            return Self::seed_from_full(
                source,
                version,
                Vec::new(),
                Vec::new(),
                previous.counters.full_fallbacks,
            );
        }

        let mut prefix = 0usize;
        while prefix < previous.regions.len()
            && prefix < parts.len()
            && previous.regions[prefix].text.as_ref() == parts[prefix].1
        {
            prefix += 1;
        }
        let mut suffix = 0usize;
        while suffix < previous.regions.len().saturating_sub(prefix)
            && suffix < parts.len().saturating_sub(prefix)
            && previous.regions[previous.regions.len() - 1 - suffix]
                .text
                .as_ref()
                == parts[parts.len() - 1 - suffix].1
        {
            suffix += 1;
        }

        let mut counters = previous.counters;
        let mut regions = Vec::with_capacity(parts.len());
        for (index, (_, part)) in parts.iter().enumerate() {
            let reused = if index < prefix {
                Some(previous.regions[index].clone())
            } else if index >= parts.len() - suffix {
                let old_index = previous.regions.len() - (parts.len() - index);
                Some(previous.regions[old_index].clone())
            } else {
                None
            };
            if let Some(region) = reused {
                counters.reused_regions += 1;
                regions.push(region);
            } else {
                counters.parsed_regions += 1;
                regions.push(parse_region(part));
            }
        }

        let Some((blocks, headings)) = assemble_regions(&regions, &parts) else {
            return Self::full_fallback(source, version, counters);
        };

        // In test/debug builds, full derivation is the executable correctness
        // oracle. A mismatch is never published as an incremental mapping.
        #[cfg(debug_assertions)]
        {
            let full = MarkdownDocument::derive_preview_and_outline(source);
            if blocks != full.0 || headings != full.1 {
                return Self::seed_from_full(
                    source,
                    version,
                    full.0,
                    full.1,
                    counters.full_fallbacks + 1,
                );
            }
        }

        Self {
            version,
            source: Arc::from(source),
            regions,
            blocks: Arc::new(blocks),
            headings,
            counters,
        }
    }

    fn full_fallback(source: &str, version: u64, mut counters: DerivationCounters) -> Self {
        counters.full_fallbacks += 1;
        let (blocks, headings) = MarkdownDocument::derive_preview_and_outline(source);
        let mut cache =
            Self::seed_from_full(source, version, blocks, headings, counters.full_fallbacks);
        cache.counters.parsed_regions = counters.parsed_regions;
        cache.counters.reused_regions = counters.reused_regions;
        cache
    }

    fn seed_from_full(
        source: &str,
        version: u64,
        blocks: Vec<PreviewBlock>,
        headings: Vec<Heading>,
        full_fallbacks: usize,
    ) -> Self {
        let parts = split_regions(source);
        let regions = partition_full_derivation(&parts, &blocks, &headings).unwrap_or_else(|| {
            vec![CachedRegion {
                text: Arc::from(source),
                blocks: Arc::new(blocks.clone()),
                headings: Arc::new(headings.clone()),
            }]
        });
        Self {
            version,
            source: Arc::from(source),
            regions,
            blocks: Arc::new(blocks),
            headings,
            counters: DerivationCounters {
                parsed_regions: 1,
                reused_regions: 0,
                full_fallbacks,
            },
        }
    }
}

fn valid_edit_chain(
    previous: &SourceMappedCache,
    source: &str,
    version: u64,
    edits: &[SourceEdit],
) -> bool {
    let Some(first) = edits.first() else {
        return false;
    };
    if first.old_version != previous.version
        || edits.last().is_none_or(|last| last.new_version != version)
        || edits
            .windows(2)
            .any(|pair| pair[0].new_version != pair[1].old_version)
    {
        return false;
    }
    let mut len = previous.source.len();
    for edit in edits {
        if edit.old_range.end > len {
            return false;
        }
        let Some(next) = len
            .checked_sub(edit.old_range.len())
            .and_then(|value| value.checked_add(edit.new_len))
        else {
            return false;
        };
        len = next;
    }
    len == source.len()
}

fn parse_region(text: &str) -> CachedRegion {
    let (blocks, headings) = MarkdownDocument::derive_preview_and_outline(text);
    CachedRegion {
        text: Arc::from(text),
        blocks: Arc::new(blocks),
        headings: Arc::new(headings),
    }
}

fn partition_full_derivation(
    parts: &[(usize, &str)],
    blocks: &[PreviewBlock],
    headings: &[Heading],
) -> Option<Vec<CachedRegion>> {
    if parts.is_empty() {
        return Some(Vec::new());
    }
    let mut regions = Vec::with_capacity(parts.len());
    for &(start, text) in parts {
        let end = start.checked_add(text.len())?;
        let mut local_blocks = Vec::new();
        for block in blocks {
            let range = block.source_range();
            if range.start >= start && range.end <= end {
                let mut block = block.clone();
                shift_preview_block(&mut block, -(isize::try_from(start).ok()?))?;
                local_blocks.push(block);
            } else if range.start < end && range.end > start {
                return None;
            }
        }
        let mut local_headings = Vec::new();
        for heading in headings {
            if heading.offset >= start && heading.offset <= end {
                let mut heading = heading.clone();
                shift_heading(&mut heading, -(isize::try_from(start).ok()?))?;
                local_headings.push(heading);
            }
        }
        regions.push(CachedRegion {
            text: Arc::from(text),
            blocks: Arc::new(local_blocks),
            headings: Arc::new(local_headings),
        });
    }
    Some(regions)
}

fn assemble_regions(
    regions: &[CachedRegion],
    parts: &[(usize, &str)],
) -> Option<(Vec<PreviewBlock>, Vec<Heading>)> {
    let mut blocks = Vec::new();
    let mut headings = Vec::new();
    for (region, &(start, _)) in regions.iter().zip(parts) {
        let delta = isize::try_from(start).ok()?;
        for block in region.blocks.iter() {
            let mut block = block.clone();
            shift_preview_block(&mut block, delta)?;
            blocks.push(block);
        }
        for heading in region.headings.iter() {
            let mut heading = heading.clone();
            shift_heading(&mut heading, delta)?;
            headings.push(heading);
        }
    }
    Some((blocks, headings))
}

fn split_regions(source: &str) -> Vec<(usize, &str)> {
    if source.is_empty() {
        return Vec::new();
    }
    let mut boundaries = vec![0usize];
    let mut offset = 0usize;
    let mut in_fence: Option<(char, usize)> = None;
    let mut pending_break = false;
    for line in source.split_inclusive('\n') {
        let line_start = offset;
        offset += line.len();
        let raw = line.trim_end_matches(['\n', '\r']);
        let trimmed = raw.trim_start();
        if let Some((marker, min_len)) = in_fence {
            if is_closing_fence(trimmed, marker, min_len) {
                in_fence = None;
            }
            continue;
        }
        if let Some(fence) = opening_fence(trimmed) {
            if pending_break && line_start > *boundaries.last().unwrap() {
                boundaries.push(line_start);
            }
            pending_break = false;
            in_fence = Some(fence);
            continue;
        }
        if raw.trim().is_empty() {
            pending_break = true;
            continue;
        }
        if pending_break {
            if !starts_with_continuation(raw) && line_start > *boundaries.last().unwrap() {
                boundaries.push(line_start);
            }
            pending_break = false;
        }
    }
    boundaries
        .iter()
        .copied()
        .zip(
            boundaries
                .iter()
                .copied()
                .skip(1)
                .chain(std::iter::once(source.len())),
        )
        .map(|(start, end)| (start, &source[start..end]))
        .collect()
}

pub(crate) fn opening_fence(trimmed: &str) -> Option<(char, usize)> {
    let marker = trimmed.chars().next()?;
    if !matches!(marker, '`' | '~') {
        return None;
    }
    let count = trimmed.chars().take_while(|ch| *ch == marker).count();
    (count >= 3).then_some((marker, count))
}

pub(crate) fn is_closing_fence(trimmed: &str, marker: char, minimum: usize) -> bool {
    let count = trimmed.chars().take_while(|ch| *ch == marker).count();
    count >= minimum && trimmed[count..].trim().is_empty()
}

fn starts_with_continuation(raw: &str) -> bool {
    if raw.starts_with('\t') || raw.starts_with("    ") {
        return true;
    }
    let trimmed = raw.trim_start();
    if trimmed.starts_with('>') || trimmed.starts_with('|') || trimmed.starts_with('<') {
        return true;
    }
    let bytes = trimmed.as_bytes();
    if bytes
        .first()
        .is_some_and(|byte| matches!(byte, b'-' | b'*' | b'+'))
        && bytes.get(1).is_none_or(|byte| matches!(byte, b' ' | b'\t'))
    {
        return true;
    }
    let digits = trimmed.chars().take_while(|ch| ch.is_ascii_digit()).count();
    digits > 0
        && trimmed
            .chars()
            .nth(digits)
            .is_some_and(|delimiter| matches!(delimiter, '.' | ')'))
}

fn requires_full_parse(source: &str) -> bool {
    if crate::frontmatter::split_front_matter(source).is_some() {
        return true;
    }
    let mut fence: Option<(char, usize)> = None;
    for line in source.lines() {
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
        if trimmed.starts_with('<')
            || trimmed.starts_with("[^")
            || is_reference_definition(trimmed)
            || has_reference_use(trimmed)
        {
            return true;
        }
    }
    fence.is_some()
}

pub(crate) fn is_reference_definition(line: &str) -> bool {
    line.strip_prefix('[')
        .and_then(|rest| rest.find("]:").map(|close| close))
        .is_some_and(|close| close > 0)
}

fn has_reference_use(line: &str) -> bool {
    line.contains("][") || line.contains("] []")
}

pub(crate) fn map_range_through_edits(
    range: &Range<usize>,
    edits: &[SourceEdit],
) -> Option<Range<usize>> {
    edits.iter().try_fold(range.clone(), |current, edit| {
        edit.map_unchanged_range(&current)
    })
}

pub(crate) fn reconcile_visual_block_ids(
    old_source: &str,
    new_source: &str,
    old_blocks: &[VisualBlock],
    new_blocks: &mut [VisualBlock],
    edits: &[SourceEdit],
) {
    for old in old_blocks {
        let Some(mapped_range) = map_range_through_edits(&old.source_range, edits) else {
            continue;
        };
        let Some(old_slice) = old_source.get(old.source_range.clone()) else {
            continue;
        };
        let Some(new_slice) = new_source.get(mapped_range.clone()) else {
            continue;
        };
        if old_slice != new_slice {
            continue;
        }
        let Some(candidate) = new_blocks
            .iter_mut()
            .find(|block| block.source_range == mapped_range)
        else {
            continue;
        };
        let mut shifted = old.clone();
        let Some(delta) = isize::try_from(mapped_range.start)
            .ok()
            .and_then(|new_start| {
                isize::try_from(old.source_range.start)
                    .ok()
                    .and_then(|old_start| new_start.checked_sub(old_start))
            })
        else {
            continue;
        };
        if shift_visual_block(&mut shifted, delta).is_none() {
            continue;
        }
        shifted.id = candidate.id;
        if shifted == *candidate {
            candidate.id = old.id;
        }
    }
}

pub(crate) fn shift_range(range: &Range<usize>, delta: isize) -> Option<Range<usize>> {
    Some(shift_offset(range.start, delta)?..shift_offset(range.end, delta)?)
}

fn shift_offset(offset: usize, delta: isize) -> Option<usize> {
    if delta >= 0 {
        offset.checked_add(usize::try_from(delta).ok()?)
    } else {
        offset.checked_sub(delta.unsigned_abs())
    }
}

fn shift_math(math: &mut MathSource, delta: isize) -> Option<()> {
    math.source_range = shift_range(&math.source_range, delta)?;
    Some(())
}

fn shift_inline_span(span: &mut InlineSpan, delta: isize) -> Option<()> {
    if let Some(math) = span.math.as_mut() {
        shift_math(math, delta)?;
    }
    Some(())
}

fn shift_rich_text(text: &mut RichText, delta: isize) -> Option<()> {
    for span in &mut text.spans {
        shift_inline_span(span, delta)?;
    }
    Some(())
}

pub(crate) fn shift_preview_block(block: &mut PreviewBlock, delta: isize) -> Option<()> {
    match block {
        PreviewBlock::Heading {
            text, source_range, ..
        }
        | PreviewBlock::Paragraph { text, source_range }
        | PreviewBlock::ListItem {
            text, source_range, ..
        }
        | PreviewBlock::BlockQuote { text, source_range } => {
            *source_range = shift_range(source_range, delta)?;
            shift_rich_text(text, delta)?;
        }
        PreviewBlock::CodeBlock { source_range, .. }
        | PreviewBlock::MathBlock { source_range, .. }
        | PreviewBlock::Html { source_range, .. }
        | PreviewBlock::Image { source_range, .. }
        | PreviewBlock::Rule { source_range }
        | PreviewBlock::Table { source_range, .. } => {
            *source_range = shift_range(source_range, delta)?;
        }
    }
    Some(())
}

pub(crate) fn shift_heading(heading: &mut Heading, delta: isize) -> Option<()> {
    heading.offset = shift_offset(heading.offset, delta)?;
    Some(())
}

fn shift_run(run: &mut VisualInlineRun, delta: isize) -> Option<()> {
    run.source_range = shift_range(&run.source_range, delta)?;
    run.content_range = shift_range(&run.content_range, delta)?;
    if let Some(range) = run.link_target_range.as_mut() {
        *range = shift_range(range, delta)?;
    }
    if let Some(math) = run.math.as_mut() {
        shift_math(math, delta)?;
    }
    Some(())
}

fn shift_reveal_group(group: &mut VisualRevealGroup, delta: isize) -> Option<()> {
    group.source_range = shift_range(&group.source_range, delta)?;
    for range in &mut group.content_ranges {
        *range = shift_range(range, delta)?;
    }
    if let Some(range) = group.link_target_range.as_mut() {
        *range = shift_range(range, delta)?;
    }
    Some(())
}

fn shift_prefix(prefix: &mut VisualBlockPrefix, delta: isize) -> Option<()> {
    prefix.indentation_range = shift_range(&prefix.indentation_range, delta)?;
    prefix.source_range = shift_range(&prefix.source_range, delta)?;
    Some(())
}

fn shift_editor_field(field: &mut VisualEditorField, delta: isize) -> Option<()> {
    field.source_range = shift_range(&field.source_range, delta)?;
    Some(())
}

fn shift_block_editor(editor: &mut VisualBlockEditor, delta: isize) -> Option<()> {
    match editor {
        VisualBlockEditor::Code {
            opening_fence,
            payload,
            info_range,
            closing_fence,
        } => {
            *opening_fence = shift_range(opening_fence, delta)?;
            shift_editor_field(payload, delta)?;
            if let Some(range) = info_range {
                *range = shift_range(range, delta)?;
            }
            *closing_fence = shift_range(closing_fence, delta)?;
        }
        VisualBlockEditor::Math {
            opening_delimiter,
            payload,
            closing_delimiter,
        } => {
            *opening_delimiter = shift_range(opening_delimiter, delta)?;
            shift_editor_field(payload, delta)?;
            *closing_delimiter = shift_range(closing_delimiter, delta)?;
        }
        VisualBlockEditor::Image {
            alt,
            destination,
            title,
        } => {
            shift_editor_field(alt, delta)?;
            shift_editor_field(destination, delta)?;
            if let Some(title) = title {
                shift_editor_field(title, delta)?;
            }
        }
        VisualBlockEditor::Table { cells } => {
            for cell in cells {
                shift_editor_field(&mut cell.field, delta)?;
            }
        }
    }
    Some(())
}

pub(crate) fn shift_visual_block(block: &mut VisualBlock, delta: isize) -> Option<()> {
    block.source_range = shift_range(&block.source_range, delta)?;
    for run in &mut block.editable_runs {
        shift_run(run, delta)?;
    }
    for group in &mut block.reveal_groups {
        shift_reveal_group(group, delta)?;
    }
    for range in &mut block.marker_ranges {
        *range = shift_range(range, delta)?;
    }
    if let Some(prefix) = block.block_prefix.as_mut() {
        shift_prefix(prefix, delta)?;
    }
    if let Some(editor) = block.editor.as_mut() {
        shift_block_editor(editor, delta)?;
    }
    Some(())
}

#[allow(dead_code)]
pub(crate) fn shift_projection(projection: &mut VisualProjection, delta: isize) -> Option<()> {
    for segment in &mut projection.segments {
        segment.source_range = shift_range(&segment.source_range, delta)?;
    }
    for range in &mut projection.revealed_source_ranges {
        *range = shift_range(range, delta)?;
    }
    projection.source_anchor = shift_offset(projection.source_anchor, delta)?;
    Some(())
}

#[allow(dead_code)]
pub(crate) fn shift_structural_edit(edit: &mut VisualStructuralEdit, delta: isize) -> Option<()> {
    edit.range = shift_range(&edit.range, delta)?;
    edit.selection_after = shift_range(&edit.selection_after, delta)?;
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_chain_maps_utf8_safe_prefix_and_suffix_ranges() {
        let source = "alpha\n\n中间\n\nomega";
        let insertion = source.find("中").unwrap();
        let edit = SourceEdit::new(source, insertion..insertion, "新".len(), 4, 5).unwrap();
        assert_eq!(edit.map_unchanged_range(&(0..5)), Some(0..5));
        let suffix = source.find("omega").unwrap();
        assert_eq!(
            edit.map_unchanged_range(&(suffix..source.len())),
            Some(suffix + 3..source.len() + 3)
        );
        assert_eq!(
            edit.map_unchanged_range(&(insertion - 1..insertion + 3)),
            None
        );
        assert!(SourceEdit::new(source, insertion + 1..insertion + 1, 0, 4, 5).is_none());
    }

    #[test]
    fn incremental_regions_equal_full_derivation_and_reuse_suffix() {
        let old = "# One\n\nFirst paragraph.\n\nSecond paragraph.\n";
        let previous = SourceMappedCache::derive_full(old, 10);
        let start = old.find("First").unwrap();
        let mut current = old.to_string();
        current.replace_range(start..start + 5, "Changed");
        let edit = SourceEdit::new(old, start..start + 5, 7, 10, 11).unwrap();
        let next = SourceMappedCache::update(&previous, &current, 11, &[edit]);
        let full = MarkdownDocument::derive_preview_and_outline(&current);
        assert_eq!(next.blocks.as_ref(), &full.0);
        assert_eq!(next.headings, full.1);
        assert!(next.counters.reused_regions >= 2);
    }

    #[test]
    fn global_constructs_force_full_fallback() {
        let old = "See [site][ref].\n\n[ref]: https://example.com\n";
        let previous = SourceMappedCache::derive_full(old, 20);
        let mut current = old.to_string();
        current.insert_str(0, "Now ");
        let edit = SourceEdit::new(old, 0..0, 4, 20, 21).unwrap();
        let next = SourceMappedCache::update(&previous, &current, 21, &[edit]);
        assert!(next.counters.full_fallbacks > previous.counters.full_fallbacks);
    }

    #[test]
    fn range_visitors_shift_nested_math_and_structural_edits() {
        let mut block = MarkdownDocument::from_text("Text $x$\n")
            .preview_blocks()
            .remove(0);
        shift_preview_block(&mut block, 8).unwrap();
        assert_eq!(block.source_range(), &(8..17));
        let mut edit = VisualStructuralEdit {
            range: 2..4,
            replacement: "x".into(),
            selection_after: 3..3,
        };
        shift_structural_edit(&mut edit, 5).unwrap();
        assert_eq!(edit.range, 7..9);
        assert_eq!(edit.selection_after, 8..8);
        assert!(shift_range(&(0..1), -1).is_none());
    }

    #[test]
    fn stable_ids_follow_source_lineage_not_equal_text() {
        let mut document = MarkdownDocument::from_text("same\n\nsame\n\ntail\n");
        let original = document.visual_blocks();
        assert_eq!(original.len(), 5);
        let first_id = original[0].id;
        let second_id = original[2].id;
        let tail_id = original[4].id;

        document.replace_range(0..4, "changed");
        let updated = document.visual_blocks();
        assert_ne!(updated[0].id, first_id);
        assert_eq!(updated[2].id, second_id);
        assert_eq!(updated[4].id, tail_id);
        assert_eq!(updated[2].source_range.start, 9);

        document.insert(0, "prefix\n\n");
        let prefixed = document.visual_blocks();
        assert_eq!(prefixed[4].id, second_id);
        assert_eq!(prefixed[6].id, tail_id);
    }

    #[test]
    fn unchanged_direct_editor_identity_and_delimiter_ranges_shift_after_early_edit() {
        let source = "intro\n\n```rust extra\nlet 名称 = 1;\n```\n";
        let mut document = MarkdownDocument::from_text(source);
        let original = document
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.editor, Some(VisualBlockEditor::Code { .. })))
            .expect("direct code block");
        let original_id = original.id;

        document.insert(0, "前缀 ");
        let shifted = document
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.editor, Some(VisualBlockEditor::Code { .. })))
            .expect("shifted direct code block");
        assert_eq!(shifted.id, original_id);
        let VisualBlockEditor::Code {
            opening_fence,
            payload,
            info_range,
            closing_fence,
        } = shifted.editor.unwrap()
        else {
            unreachable!()
        };
        assert_eq!(&document.text()[opening_fence], "```");
        assert_eq!(&document.text()[info_range.unwrap()], "rust extra");
        assert_eq!(&document.text()[payload.source_range], "let 名称 = 1;\n");
        assert_eq!(&document.text()[closing_fence], "```");
    }

    #[test]
    fn stable_ids_handle_splits_nested_lists_islands_and_document_isolation() {
        let source = "- parent\n  - child\n\n<div>kept</div>\n\ntail\n";
        let mut document = MarkdownDocument::from_text(source);
        let original = document.visual_blocks();
        let child = original
            .iter()
            .find(|block| {
                document.text()[block.source_range.clone()].contains("child")
                    && matches!(block.kind, crate::VisualBlockKind::ListItem { .. })
            })
            .unwrap();
        let child_id = child.id;
        let island_id = original
            .iter()
            .find(|block| block.source_island == Some(crate::VisualSourceIslandKind::Html))
            .unwrap()
            .id;
        let tail_id = original.last().unwrap().id;

        document.insert(0, "intro\n\n");
        let shifted = document.visual_blocks();
        assert!(shifted.iter().any(|block| block.id == child_id));
        assert!(shifted.iter().any(|block| block.id == island_id));
        assert!(shifted.iter().any(|block| block.id == tail_id));

        let tail = document.text().find("tail").unwrap();
        document.insert(tail + 2, "\n\n");
        let split = document.visual_blocks();
        assert!(!split.iter().any(|block| block.id == tail_id));

        let other = MarkdownDocument::from_text(source).visual_blocks();
        assert!(
            other
                .iter()
                .all(|right| original.iter().all(|left| left.id != right.id))
        );
    }

    #[test]
    fn edit_chain_rejects_overflow_nonconsecutive_versions_and_excess_length() {
        let source = "abc";
        assert!(SourceEdit::new(source, 0..4, 0, 1, 2).is_none());
        assert!(SourceEdit::new(source, 1..2, 0, 1, 3).is_none());
        assert!(shift_range(&(usize::MAX..usize::MAX), 1).is_none());

        let mut pending = PendingSourceEdits::Incremental(Vec::new());
        pending.record(SourceEdit::new(source, 0..0, 1, 1, 2).unwrap());
        pending.record(SourceEdit::new(source, 0..0, 1, 4, 5).unwrap());
        assert!(matches!(pending, PendingSourceEdits::Full));
    }

    #[test]
    fn whole_replacement_and_cache_free_clone_rebuild_ids() {
        let mut document = MarkdownDocument::from_text("one\n\ntwo\n");
        let original = document.visual_blocks();
        let clone = document.clone();
        let cloned = clone.visual_blocks();
        assert_ne!(original[0].id, cloned[0].id);

        document.set_text("one\n\ntwo\n");
        assert_eq!(document.visual_blocks()[0].id, original[0].id);
        document.set_text("replacement\n");
        assert_ne!(document.visual_blocks()[0].id, original[0].id);
    }

    #[test]
    fn large_local_edit_reuses_bounded_regions() {
        let source = (0..500)
            .map(|index| format!("paragraph {index}\n\n"))
            .collect::<String>();
        let mut document = MarkdownDocument::from_text(source);
        let original = document.visual_blocks_shared();
        let unchanged_suffix_id = original.last().unwrap().id;
        let before = document.source_mapped_derivation_counters();
        let edit = document.text().find("paragraph 250").unwrap() + "paragraph ".len();
        document.replace_range(edit..edit + 3, "middle");
        let incremental = document.visual_blocks_shared();
        let after = document.source_mapped_derivation_counters();
        assert!(after.reused_regions - before.reused_regions >= 499);
        assert!(after.parsed_regions - before.parsed_regions <= 1);
        assert_eq!(incremental.last().unwrap().id, unchanged_suffix_id);

        let full = MarkdownDocument::from_text(document.text()).visual_blocks();
        assert_eq!(incremental.len(), full.len());
        for (incremental, full) in incremental.iter().zip(&full) {
            let mut normalized = incremental.clone();
            normalized.id = full.id;
            assert_eq!(&normalized, full);
        }
    }

    #[test]
    fn randomized_incremental_visual_model_matches_fresh_full_documents() {
        let mut document = MarkdownDocument::from_text(
            "# Head\n\nplain **bold** and $x$.\n\n- one\n- two\n\n> quote\n\n| a | b |\n| - | - |\n| c | d |\n\n```rs\nfn main() {}\n```\n",
        );
        document.visual_blocks_shared();
        let inserts = ["x", "中", "\n\nnew block\n", " **b** ", ""];
        let mut state = 0x9e37_79b9_u64;
        for _ in 0..80 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let boundaries = document
                .text()
                .char_indices()
                .map(|(offset, _)| offset)
                .chain(std::iter::once(document.text().len()))
                .collect::<Vec<_>>();
            let start_index = state as usize % boundaries.len();
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let max_end = (start_index + 4).min(boundaries.len() - 1);
            let end_index = start_index + state as usize % (max_end - start_index + 1);
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let replacement = inserts[state as usize % inserts.len()];
            document.replace_range(boundaries[start_index]..boundaries[end_index], replacement);

            let incremental = document.visual_blocks();
            let full_document = MarkdownDocument::from_text(document.text());
            let full = full_document.visual_blocks();
            assert_eq!(document.outline(), full_document.outline());
            assert_eq!(incremental.len(), full.len());
            for (incremental, full) in incremental.iter().zip(&full) {
                let mut normalized = incremental.clone();
                normalized.id = full.id;
                assert_eq!(&normalized, full);
                let incremental_projection = crate::build_visual_projection(
                    document.text(),
                    incremental,
                    0..0,
                    incremental.source_range.start,
                );
                let full_projection = crate::build_visual_projection(
                    document.text(),
                    full,
                    0..0,
                    full.source_range.start,
                );
                assert_eq!(incremental_projection, full_projection);
            }
        }
    }

    #[test]
    fn every_visual_variant_stays_full_parse_equivalent_around_utf8_edits() {
        let cases = [
            "# 标题\n",
            "plain **bold** $x$\n",
            "- [x] task\n",
            "> quote\n",
            "```rs\nfn main() {}\n```\n",
            "$$\nx + y\n$$\n",
            "<div>html</div>\n",
            "![alt](image.png)\n",
            "---\n",
            "| a | b |\n| :- | -: |\n| 中 | d |\n",
            "   \n",
        ];
        for source in cases {
            let targets = [
                (0..0, "前"),
                {
                    let middle = source
                        .char_indices()
                        .nth(source.chars().count() / 2)
                        .map_or(source.len(), |(offset, _)| offset);
                    (middle..middle, "中")
                },
                (source.len()..source.len(), "后"),
            ];
            for (range, replacement) in targets {
                let mut incremental = MarkdownDocument::from_text(source);
                incremental.visual_blocks_shared();
                incremental.replace_range(range, replacement);
                let incremental_blocks = incremental.visual_blocks();
                let full = MarkdownDocument::from_text(incremental.text());
                let full_blocks = full.visual_blocks();
                assert_eq!(incremental.outline(), full.outline());
                assert_eq!(incremental_blocks.len(), full_blocks.len());
                for (incremental, full) in incremental_blocks.iter().zip(&full_blocks) {
                    let mut normalized = incremental.clone();
                    normalized.id = full.id;
                    assert_eq!(&normalized, full, "source={source:?}");
                }
            }
        }
    }
}
