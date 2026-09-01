//! Exact, GPUI-free block mutations for Visual Edit.
//!
//! Markdown remains canonical. Callers capture undo state, validate an
//! immutable [`BlockTarget`], then apply the returned [`BlockEdit`] as one
//! source replacement.

use std::ops::Range;

use crate::{VisualBlock, VisualBlockEditor, VisualBlockId, VisualBlockKind, VisualQuoteGroupEdge};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockTransform {
    Text,
    Heading(u8),
    BulletedList,
    NumberedList,
    TaskList,
    Quote,
    CodeBlock,
    Divider,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlashCommand {
    Text,
    Heading(u8),
    BulletedList,
    NumberedList,
    TaskList,
    Quote,
    CodeBlock,
    Divider,
    Table,
}

impl SlashCommand {
    pub const ALL: [Self; 14] = [
        Self::Text,
        Self::Heading(1),
        Self::Heading(2),
        Self::Heading(3),
        Self::Heading(4),
        Self::Heading(5),
        Self::Heading(6),
        Self::BulletedList,
        Self::NumberedList,
        Self::TaskList,
        Self::Quote,
        Self::CodeBlock,
        Self::Divider,
        Self::Table,
    ];

    pub fn search_terms(self) -> &'static str {
        match self {
            Self::Text => "text paragraph plain",
            Self::Heading(1) => "heading 1 h1 title",
            Self::Heading(2) => "heading 2 h2 subtitle",
            Self::Heading(3) => "heading 3 h3",
            Self::Heading(4) => "heading 4 h4",
            Self::Heading(5) => "heading 5 h5",
            Self::Heading(6) => "heading 6 h6",
            Self::Heading(_) => "heading",
            Self::BulletedList => "bulleted unordered list",
            Self::NumberedList => "numbered ordered list",
            Self::TaskList => "task checklist todo",
            Self::Quote => "quote blockquote",
            Self::CodeBlock => "code fenced",
            Self::Divider => "divider rule horizontal",
            Self::Table => "table grid",
        }
    }

    pub fn transform(self) -> BlockTransform {
        match self {
            Self::Text => BlockTransform::Text,
            Self::Heading(level) => BlockTransform::Heading(level),
            Self::BulletedList => BlockTransform::BulletedList,
            Self::NumberedList => BlockTransform::NumberedList,
            Self::TaskList => BlockTransform::TaskList,
            Self::Quote => BlockTransform::Quote,
            Self::CodeBlock => BlockTransform::CodeBlock,
            Self::Divider => BlockTransform::Divider,
            Self::Table => BlockTransform::Table,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashQuery {
    pub document_version: u64,
    pub source_range: Range<usize>,
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockTarget {
    pub document_version: u64,
    pub block_id: VisualBlockId,
    pub source_range: Range<usize>,
}

impl BlockTarget {
    pub fn from_block(document_version: u64, block: &VisualBlock) -> Self {
        Self {
            document_version,
            block_id: block.id,
            source_range: block.source_range.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEdit {
    /// Version of the source the edit was computed against. Applying the
    /// edit through the checked mutation boundary rejects a stale value
    /// instead of splicing it into newer text.
    pub document_version: u64,
    pub range: Range<usize>,
    pub replacement: String,
    pub selection_after: Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPlacement {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEditError {
    Stale,
    Unsupported,
    Ambiguous,
}

pub fn slash_query_at(source: &str, cursor: usize, document_version: u64) -> Option<SlashQuery> {
    if cursor > source.len() || !source.is_char_boundary(cursor) {
        return None;
    }
    let line_start = source[..cursor].rfind('\n').map_or(0, |index| index + 1);
    let line_end = source[cursor..]
        .find('\n')
        .map_or(source.len(), |relative| cursor + relative);
    let line = source.get(line_start..line_end)?;
    let trimmed = line.trim_start_matches([' ', '\t']);
    let indentation = line.len() - trimmed.len();
    let query = trimmed.strip_prefix('/')?;
    if query.contains(['\r', '\n']) || query.chars().any(char::is_whitespace) {
        return None;
    }
    Some(SlashQuery {
        document_version,
        source_range: line_start + indentation..line_end,
        query: query.to_string(),
    })
}

pub fn filtered_slash_commands(query: &str) -> Vec<SlashCommand> {
    let needle = query.trim().to_lowercase();
    SlashCommand::ALL
        .into_iter()
        .filter(|command| needle.is_empty() || command.search_terms().contains(&needle))
        .collect()
}

pub fn slash_command_edit(
    source: &str,
    current_version: u64,
    query: &SlashQuery,
    command: SlashCommand,
) -> Result<BlockEdit, BlockEditError> {
    if query.document_version != current_version
        || source.get(query.source_range.clone()).is_none()
        || !source[query.source_range.clone()].starts_with('/')
    {
        return Err(BlockEditError::Stale);
    }
    let line_ending = preferred_line_ending(source);
    let (replacement, caret) = slash_template(command, line_ending);
    let start = query.source_range.start;
    Ok(BlockEdit {
        document_version: current_version,
        range: query.source_range.clone(),
        replacement,
        selection_after: start + caret..start + caret,
    })
}

pub fn validate_block_target<'a>(
    current_version: u64,
    blocks: &'a [VisualBlock],
    target: &BlockTarget,
) -> Result<(usize, &'a VisualBlock), BlockEditError> {
    if target.document_version != current_version {
        return Err(BlockEditError::Stale);
    }
    let Some((index, block)) = blocks.iter().enumerate().find(|(_, block)| {
        block.id == target.block_id && block.source_range == target.source_range
    }) else {
        return Err(BlockEditError::Stale);
    };
    if blocks.iter().enumerate().any(|(other_index, other)| {
        other_index != index
            && !matches!(other.kind, VisualBlockKind::Whitespace)
            && ranges_overlap(&block.source_range, &other.source_range)
    }) {
        return Err(BlockEditError::Ambiguous);
    }
    Ok((index, block))
}

pub fn block_can_transform(block: &VisualBlock) -> bool {
    let exact_code_island = matches!(block.kind, VisualBlockKind::CodeBlock { .. })
        && matches!(block.editor, Some(VisualBlockEditor::Code { .. }));
    if block.source_island.is_some() && !exact_code_island {
        return false;
    }
    if let Some(quote) = &block.quote_context
        && (quote.depth != 1 || quote.edge != VisualQuoteGroupEdge::Only)
    {
        return false;
    }
    match block.kind {
        VisualBlockKind::Heading { .. }
        | VisualBlockKind::Paragraph
        | VisualBlockKind::BlockQuote
        | VisualBlockKind::CodeBlock { .. }
        | VisualBlockKind::Rule
        | VisualBlockKind::Table { .. }
        | VisualBlockKind::Whitespace => true,
        VisualBlockKind::ListItem { level, .. } => level <= 1,
        _ => false,
    }
}

pub fn block_can_reorder(block: &VisualBlock) -> bool {
    block_can_transform(block)
        && block.quote_context.is_none()
        && !matches!(block.kind, VisualBlockKind::Whitespace)
}

pub fn block_can_transform_at(blocks: &[VisualBlock], index: usize) -> bool {
    blocks.get(index).is_some_and(block_can_transform) && !block_has_nested_children(blocks, index)
}

pub fn block_can_reorder_at(blocks: &[VisualBlock], index: usize) -> bool {
    blocks.get(index).is_some_and(block_can_reorder) && !block_has_nested_children(blocks, index)
}

pub fn transform_block(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    transform: BlockTransform,
) -> Result<BlockEdit, BlockEditError> {
    let (index, block) = validate_block_target(current_version, blocks, target)?;
    if !block_can_transform_at(blocks, index) {
        return Err(BlockEditError::Unsupported);
    }
    let body = block_body(source, block)?;
    let authored = source
        .get(block.source_range.clone())
        .ok_or(BlockEditError::Ambiguous)?;
    let trailing = trailing_line_endings(authored);
    let body = body.strip_suffix(trailing).unwrap_or(&body);
    let line_ending = preferred_line_ending(source);
    let mut replacement = serialize_transform(transform, body, line_ending);
    let start = block.source_range.start;
    let selection_after = transform_caret(transform, &replacement, start, line_ending);
    replacement.push_str(trailing);
    Ok(BlockEdit {
        document_version: current_version,
        range: block.source_range.clone(),
        replacement,
        selection_after,
    })
}

pub fn duplicate_block(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
) -> Result<BlockEdit, BlockEditError> {
    let (index, _) = validate_block_target(current_version, blocks, target)?;
    if !block_can_reorder_at(blocks, index) {
        return Err(BlockEditError::Unsupported);
    }
    let unit = source_unit(source, blocks, index)?;
    let authored = source.get(unit.clone()).ok_or(BlockEditError::Ambiguous)?;
    let separator = if unit.end == source.len() && !authored.ends_with(['\n', '\r']) {
        preferred_line_ending(source).repeat(2)
    } else {
        String::new()
    };
    let replacement = format!("{authored}{separator}{authored}");
    let second_start = unit.start + authored.len() + separator.len();
    Ok(BlockEdit {
        document_version: current_version,
        range: unit,
        replacement,
        selection_after: second_start..second_start,
    })
}

pub fn delete_block(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
) -> Result<BlockEdit, BlockEditError> {
    let (index, _) = validate_block_target(current_version, blocks, target)?;
    if !block_can_reorder_at(blocks, index) {
        return Err(BlockEditError::Unsupported);
    }
    let unit = source_unit(source, blocks, index)?;
    Ok(BlockEdit {
        document_version: current_version,
        selection_after: unit.start..unit.start,
        range: unit,
        replacement: String::new(),
    })
}

pub fn reorder_block(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    moving: &BlockTarget,
    destination: &BlockTarget,
    placement: BlockPlacement,
) -> Result<BlockEdit, BlockEditError> {
    let (moving_index, _) = validate_block_target(current_version, blocks, moving)?;
    let (destination_index, _) = validate_block_target(current_version, blocks, destination)?;
    if moving_index == destination_index {
        return Err(BlockEditError::Ambiguous);
    }
    if !block_can_reorder_at(blocks, moving_index)
        || !block_can_reorder_at(blocks, destination_index)
    {
        return Err(BlockEditError::Unsupported);
    }
    let moving_unit = source_unit(source, blocks, moving_index)?;
    let destination_unit = source_unit(source, blocks, destination_index)?;
    if ranges_overlap(&moving_unit, &destination_unit) {
        return Err(BlockEditError::Ambiguous);
    }

    let mut moved = source
        .get(moving_unit.clone())
        .ok_or(BlockEditError::Ambiguous)?
        .to_string();
    let raw_insertion = match placement {
        BlockPlacement::Before => destination_unit.start,
        BlockPlacement::After => destination_unit.end,
    };
    let mut replacement = source.to_string();
    replacement.replace_range(moving_unit.clone(), "");
    let insertion = if moving_unit.start < raw_insertion {
        raw_insertion - moving_unit.len()
    } else {
        raw_insertion
    };
    let line_ending = preferred_line_ending(source);
    if matches!(placement, BlockPlacement::After)
        && insertion > 0
        && !replacement[..insertion].ends_with('\n')
    {
        moved.insert_str(0, &line_ending.repeat(2));
    }
    if insertion < replacement.len()
        && !moved.ends_with(['\n', '\r'])
        && !replacement[insertion..].starts_with(['\n', '\r'])
    {
        moved.push_str(&line_ending.repeat(2));
    }
    let selected_start = insertion
        + moved
            .bytes()
            .take_while(|byte| matches!(byte, b'\r' | b'\n'))
            .count();
    replacement.insert_str(insertion, &moved);
    Ok(BlockEdit {
        document_version: current_version,
        range: 0..source.len(),
        replacement,
        selection_after: selected_start..selected_start,
    })
}

pub fn adjacent_reorder_target(
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    forward: bool,
) -> Result<BlockTarget, BlockEditError> {
    let (index, _) = validate_block_target(current_version, blocks, target)?;
    let candidate = if forward {
        blocks
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(candidate_index, _)| block_can_reorder_at(blocks, *candidate_index))
            .map(|(_, block)| block)
    } else {
        blocks
            .iter()
            .enumerate()
            .take(index)
            .rev()
            .find(|(candidate_index, _)| block_can_reorder_at(blocks, *candidate_index))
            .map(|(_, block)| block)
    }
    .ok_or(BlockEditError::Unsupported)?;
    Ok(BlockTarget::from_block(current_version, candidate))
}

fn block_body(source: &str, block: &VisualBlock) -> Result<String, BlockEditError> {
    let authored = source
        .get(block.source_range.clone())
        .ok_or(BlockEditError::Ambiguous)?;
    if matches!(block.kind, VisualBlockKind::Rule) {
        return Ok(String::new());
    }
    if let Some(VisualBlockEditor::Code { payload, .. }) = block.editor.as_ref() {
        let payload = source
            .get(payload.source_range.clone())
            .ok_or(BlockEditError::Ambiguous)?;
        return Ok(payload
            .strip_suffix("\r\n")
            .or_else(|| payload.strip_suffix('\n'))
            .unwrap_or(payload)
            .to_string());
    }
    if matches!(block.kind, VisualBlockKind::Table { .. }) {
        return Ok(authored.to_string());
    }

    let mut ranges = Vec::new();
    if let Some(prefix) = &block.block_prefix {
        ranges.push(prefix.source_range.clone());
    }
    if let Some(quote) = &block.quote_context {
        ranges.extend(quote.marker_ranges.iter().cloned());
    }
    let mut body = authored.to_string();
    ranges.sort_by_key(|range| range.start);
    for range in ranges.into_iter().rev() {
        if range.start < block.source_range.start || range.end > block.source_range.end {
            return Err(BlockEditError::Ambiguous);
        }
        body.replace_range(
            range.start - block.source_range.start..range.end - block.source_range.start,
            "",
        );
    }
    Ok(body)
}

fn serialize_transform(transform: BlockTransform, body: &str, line_ending: &str) -> String {
    match transform {
        BlockTransform::Text => body.to_string(),
        BlockTransform::Heading(level) => {
            let level = level.clamp(1, 6);
            let text = body.lines().map(str::trim).collect::<Vec<_>>().join(" ");
            format!("{} {text}", "#".repeat(level as usize))
        }
        BlockTransform::BulletedList => prefix_nonempty_lines(body, "- ", line_ending),
        BlockTransform::NumberedList => prefix_nonempty_lines(body, "1. ", line_ending),
        BlockTransform::TaskList => prefix_nonempty_lines(body, "- [ ] ", line_ending),
        BlockTransform::Quote => prefix_all_lines(body, "> ", line_ending),
        BlockTransform::CodeBlock => {
            let fence = code_fence_for(body);
            format!("{fence}{line_ending}{body}{line_ending}{fence}")
        }
        BlockTransform::Divider => "---".to_string(),
        BlockTransform::Table => table_template(line_ending),
    }
}

fn transform_caret(
    transform: BlockTransform,
    replacement: &str,
    start: usize,
    line_ending: &str,
) -> Range<usize> {
    let relative = match transform {
        BlockTransform::Text => replacement.len(),
        BlockTransform::Heading(level) => (level.clamp(1, 6) as usize + 1).min(replacement.len()),
        BlockTransform::BulletedList => 2.min(replacement.len()),
        BlockTransform::NumberedList => 3.min(replacement.len()),
        BlockTransform::TaskList => 6.min(replacement.len()),
        BlockTransform::Quote => 2.min(replacement.len()),
        BlockTransform::CodeBlock => replacement
            .find(line_ending)
            .map_or(replacement.len(), |index| index + line_ending.len()),
        BlockTransform::Divider => replacement.len(),
        BlockTransform::Table => 2,
    };
    start + relative..start + relative
}

fn slash_template(command: SlashCommand, line_ending: &str) -> (String, usize) {
    match command {
        SlashCommand::Text => (String::new(), 0),
        SlashCommand::Heading(level) => {
            let value = format!("{} ", "#".repeat(level.clamp(1, 6) as usize));
            let caret = value.len();
            (value, caret)
        }
        SlashCommand::BulletedList => ("- ".to_string(), 2),
        SlashCommand::NumberedList => ("1. ".to_string(), 3),
        SlashCommand::TaskList => ("- [ ] ".to_string(), 6),
        SlashCommand::Quote => ("> ".to_string(), 2),
        SlashCommand::CodeBlock => {
            let value = format!("```{line_ending}{line_ending}```");
            let caret = 3 + line_ending.len();
            (value, caret)
        }
        SlashCommand::Divider => ("---".to_string(), 3),
        SlashCommand::Table => {
            let value = table_template(line_ending);
            (value, 2)
        }
    }
}

fn table_template(line_ending: &str) -> String {
    ["|  |  |", "| --- | --- |", "|  |  |"].join(line_ending)
}

fn prefix_nonempty_lines(body: &str, prefix: &str, line_ending: &str) -> String {
    body.split(line_ending)
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join(line_ending)
}

fn prefix_all_lines(body: &str, prefix: &str, line_ending: &str) -> String {
    body.split(line_ending)
        .map(|line| {
            if line.is_empty() {
                prefix.trim_end().to_string()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join(line_ending)
}

fn code_fence_for(body: &str) -> String {
    let longest = body
        .split(|ch| ch != '`')
        .map(str::len)
        .max()
        .unwrap_or_default();
    "`".repeat(longest.saturating_add(1).max(3))
}

fn preferred_line_ending(source: &str) -> &'static str {
    if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn trailing_line_endings(source: &str) -> &str {
    let start = source.trim_end_matches(['\r', '\n']).len();
    &source[start..]
}

fn source_unit(
    source: &str,
    blocks: &[VisualBlock],
    index: usize,
) -> Result<Range<usize>, BlockEditError> {
    let block = blocks.get(index).ok_or(BlockEditError::Stale)?;
    if block.source_range.start > block.source_range.end
        || block.source_range.end > source.len()
        || !source.is_char_boundary(block.source_range.start)
        || !source.is_char_boundary(block.source_range.end)
        || (block.source_range.start > 0 && !source[..block.source_range.start].ends_with('\n'))
    {
        return Err(BlockEditError::Ambiguous);
    }
    let end = blocks
        .iter()
        .skip(index + 1)
        .find(|candidate| !matches!(candidate.kind, VisualBlockKind::Whitespace))
        .map_or(source.len(), |candidate| candidate.source_range.start);
    if end < block.source_range.end || end > source.len() || !source.is_char_boundary(end) {
        return Err(BlockEditError::Ambiguous);
    }
    Ok(block.source_range.start..end)
}

fn block_has_nested_children(blocks: &[VisualBlock], index: usize) -> bool {
    let Some(VisualBlock {
        kind: VisualBlockKind::ListItem { level, .. },
        ..
    }) = blocks.get(index)
    else {
        return false;
    };
    for candidate in blocks.iter().skip(index + 1) {
        match candidate.kind {
            VisualBlockKind::Whitespace => continue,
            VisualBlockKind::ListItem {
                level: candidate_level,
                ..
            } => return candidate_level > *level,
            _ => return false,
        }
    }
    false
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarkdownDocument;

    fn target_for(doc: &MarkdownDocument, index: usize) -> (Vec<VisualBlock>, BlockTarget) {
        let blocks = doc.visual_blocks_shared().as_ref().clone();
        let target = BlockTarget::from_block(doc.version(), &blocks[index]);
        (blocks, target)
    }

    #[test]
    fn slash_query_and_templates_are_versioned_utf8_and_crlf_safe() {
        let source = "前文\r\n  /h2";
        let query = slash_query_at(source, source.len(), 7).unwrap();
        assert_eq!(query.source_range, 10..13);
        assert_eq!(query.query, "h2");
        let edit = slash_command_edit(source, 7, &query, SlashCommand::Heading(2)).unwrap();
        assert_eq!(edit.replacement, "## ");
        assert_eq!(edit.selection_after, 13..13);
        assert_eq!(
            slash_command_edit(source, 8, &query, SlashCommand::Heading(2)),
            Err(BlockEditError::Stale)
        );
    }

    #[test]
    fn slash_templates_cover_the_complete_command_set() {
        for command in SlashCommand::ALL {
            let query = slash_query_at("/", 1, 1).unwrap();
            let edit = slash_command_edit("/", 1, &query, command).unwrap();
            assert!(edit.selection_after.start <= edit.replacement.len());
        }
        assert_eq!(
            filtered_slash_commands("task"),
            vec![SlashCommand::TaskList]
        );
    }

    #[test]
    fn transforms_common_blocks_with_one_exact_replacement() {
        let doc = MarkdownDocument::from_text("## Hello **world**");
        let (blocks, target) = target_for(&doc, 0);
        let edit = transform_block(
            doc.text(),
            doc.version(),
            &blocks,
            &target,
            BlockTransform::TaskList,
        )
        .unwrap();
        assert_eq!(edit.range, 0..doc.text().len());
        assert_eq!(edit.replacement, "- [ ] Hello **world**");

        let code = MarkdownDocument::from_text("```rust\nlet x = 1;\n```");
        let (blocks, target) = target_for(&code, 0);
        let edit = transform_block(
            code.text(),
            code.version(),
            &blocks,
            &target,
            BlockTransform::Quote,
        )
        .unwrap();
        assert_eq!(edit.replacement, "> let x = 1;");

        let adjacent = MarkdownDocument::from_text("one\n\ntwo");
        let (blocks, target) = target_for(&adjacent, 0);
        let edit = transform_block(
            adjacent.text(),
            adjacent.version(),
            &blocks,
            &target,
            BlockTransform::TaskList,
        )
        .unwrap();
        let mut transformed = adjacent.text().to_string();
        transformed.replace_range(edit.range, &edit.replacement);
        assert_eq!(transformed, "- [ ] one\n\ntwo");
    }

    #[test]
    fn transforms_to_all_serialized_shapes_and_preserves_crlf() {
        let doc = MarkdownDocument::from_text("alpha\r\nbeta");
        let (blocks, target) = target_for(&doc, 0);
        for transform in [
            BlockTransform::Text,
            BlockTransform::Heading(6),
            BlockTransform::BulletedList,
            BlockTransform::NumberedList,
            BlockTransform::TaskList,
            BlockTransform::Quote,
            BlockTransform::CodeBlock,
            BlockTransform::Divider,
            BlockTransform::Table,
        ] {
            let edit =
                transform_block(doc.text(), doc.version(), &blocks, &target, transform).unwrap();
            if matches!(transform, BlockTransform::CodeBlock | BlockTransform::Table) {
                assert!(edit.replacement.contains("\r\n"));
            }
        }

        let spaced = MarkdownDocument::from_text("  文本  \n\nnext");
        let (blocks, target) = target_for(&spaced, 0);
        let edit = transform_block(
            spaced.text(),
            spaced.version(),
            &blocks,
            &target,
            BlockTransform::TaskList,
        )
        .unwrap();
        let mut transformed = spaced.text().to_string();
        transformed.replace_range(edit.range, &edit.replacement);
        assert_eq!(transformed, "- [ ]   文本  \n\nnext");
    }

    #[test]
    fn duplicate_delete_and_reorder_share_complete_source_units() {
        let doc = MarkdownDocument::from_text("one\n\ntwo\n\nthree");
        let blocks = doc.visual_blocks_shared();
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        let one = BlockTarget::from_block(doc.version(), content[0]);
        let two = BlockTarget::from_block(doc.version(), content[1]);
        let duplicate = duplicate_block(doc.text(), doc.version(), &blocks, &one).unwrap();
        assert_eq!(duplicate.replacement, "one\n\none\n\n");
        let delete = delete_block(doc.text(), doc.version(), &blocks, &two).unwrap();
        assert_eq!(delete.range, 5..10);
        let moved = reorder_block(
            doc.text(),
            doc.version(),
            &blocks,
            &one,
            &two,
            BlockPlacement::After,
        )
        .unwrap();
        assert_eq!(moved.replacement, "two\n\none\n\nthree");

        let two_blocks = MarkdownDocument::from_text("one\n\ntwo");
        let blocks = two_blocks.visual_blocks_shared();
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        let first = BlockTarget::from_block(two_blocks.version(), content[0]);
        let last = BlockTarget::from_block(two_blocks.version(), content[1]);
        let moved = reorder_block(
            two_blocks.text(),
            two_blocks.version(),
            &blocks,
            &first,
            &last,
            BlockPlacement::After,
        )
        .unwrap();
        assert_eq!(moved.replacement, "two\n\none\n\n");
        let duplicate =
            duplicate_block(two_blocks.text(), two_blocks.version(), &blocks, &last).unwrap();
        assert_eq!(duplicate.replacement, "two\n\ntwo");
    }

    #[test]
    fn mixed_inline_image_deletes_as_one_prose_row() {
        let source = "hello ![alt](url) world";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        assert_eq!(content.len(), 1, "mixed image should be one row");
        assert!(matches!(content[0].kind, VisualBlockKind::Paragraph));
        assert!(
            content[0]
                .editable_runs
                .iter()
                .any(|run| run.html_image.is_some())
        );

        let target = BlockTarget::from_block(doc.version(), content[0]);
        let edit = delete_block(doc.text(), doc.version(), &blocks, &target)
            .expect("a single mixed paragraph is a deletable block");
        let mut remaining = source.to_string();
        remaining.replace_range(edit.range, &edit.replacement);
        assert!(
            !remaining.contains("![alt](url)"),
            "deleting the mixed row removes the image syntax, got {remaining:?}"
        );
        assert!(!remaining.contains("hello"));

        let adjacent = "Intro\n\n![solo](solo.png)";
        let doc = MarkdownDocument::from_text(adjacent);
        let blocks = doc.visual_blocks_shared();
        let content = blocks
            .iter()
            .filter(|block| !matches!(block.kind, VisualBlockKind::Whitespace))
            .collect::<Vec<_>>();
        let leading = content
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .expect("leading prose");
        let image = content
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Image { .. }))
            .expect("standalone image row");
        let target = BlockTarget::from_block(doc.version(), leading);
        let edit = delete_block(doc.text(), doc.version(), &blocks, &target)
            .expect("leading paragraph starts at a line boundary");
        assert!(
            edit.range.end <= image.source_range.start,
            "leading delete {:?} swallowed image {:?}",
            edit.range,
            image.source_range
        );
        let mut remaining = adjacent.to_string();
        remaining.replace_range(edit.range, &edit.replacement);
        assert!(remaining.contains("![solo](solo.png)"));
        assert!(!remaining.contains("Intro"));
    }

    #[test]
    fn stale_nested_and_ambiguous_quote_targets_are_rejected() {
        let nested = MarkdownDocument::from_text("- parent\n  - child");
        let blocks = nested.visual_blocks_shared();
        let child = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::ListItem { level: 2, .. }))
            .unwrap();
        let target = BlockTarget::from_block(nested.version(), child);
        assert_eq!(
            transform_block(
                nested.text(),
                nested.version(),
                &blocks,
                &target,
                BlockTransform::Text,
            ),
            Err(BlockEditError::Unsupported)
        );
        let parent_index = blocks
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::ListItem { level: 1, .. }))
            .unwrap();
        assert!(!block_can_transform_at(&blocks, parent_index));
        assert!(!block_can_reorder_at(&blocks, parent_index));

        let quote = MarkdownDocument::from_text("> one\n>\n> two");
        let blocks = quote.visual_blocks_shared();
        let first = blocks
            .iter()
            .find(|block| block.quote_context.is_some())
            .unwrap();
        let stale = BlockTarget {
            document_version: quote.version() + 1,
            ..BlockTarget::from_block(quote.version(), first)
        };
        assert_eq!(
            transform_block(
                quote.text(),
                quote.version(),
                &blocks,
                &stale,
                BlockTransform::Text,
            ),
            Err(BlockEditError::Stale)
        );
        assert!(!block_can_transform(first));
    }
}
