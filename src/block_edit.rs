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

/// A complete canonical unit, including a list subtree or an enclosing quote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBlockUnit {
    pub blocks: Range<usize>,
    pub source_range: Range<usize>,
}

pub fn source_block_unit(
    source: &str,
    blocks: &[VisualBlock],
    index: usize,
) -> Result<SourceBlockUnit, BlockEditError> {
    let block = blocks.get(index).ok_or(BlockEditError::Stale)?;
    let mut first = index;
    let mut last = index + 1;
    let mut range = block.source_range.clone();
    if let Some(quote) = &block.quote_context {
        range = quote.group_source_range.clone();
        while first > 0 && blocks[first - 1].source_range.start >= range.start {
            first -= 1;
        }
        while last < blocks.len() && blocks[last].source_range.start < range.end {
            last += 1;
        }
    } else if matches!(block.kind, VisualBlockKind::ListItem { .. }) {
        let subtree_end = block.list_subtree_end.ok_or(BlockEditError::Ambiguous)?;
        while let Some(candidate) = blocks
            .get(last)
            .filter(|next| next.source_range.start < subtree_end)
        {
            range.end = range.end.max(candidate.source_range.end);
            last += 1;
        }
    } else if block.list_depth > 0 {
        return Err(BlockEditError::Unsupported);
    }
    while last < blocks.len() && matches!(blocks[last].kind, VisualBlockKind::Whitespace) {
        last += 1;
    }
    range.end = blocks
        .get(last)
        .map_or(source.len(), |next| next.source_range.start);
    if range.start > range.end
        || range.end > source.len()
        || !source.is_char_boundary(range.start)
        || !source.is_char_boundary(range.end)
        || (range.start > 0 && !source[..range.start].ends_with('\n'))
    {
        return Err(BlockEditError::Ambiguous);
    }
    Ok(SourceBlockUnit {
        blocks: first..last,
        source_range: range,
    })
}

/// Move a complete range using only the span between the old and new sites.
pub fn move_source_units(
    source: &str,
    version: u64,
    blocks: &[VisualBlock],
    moving: Range<usize>,
    insertion: usize,
) -> Result<BlockEdit, BlockEditError> {
    if moving.is_empty()
        || insertion > source.len()
        || (insertion >= moving.start && insertion <= moving.end)
        || source.get(moving.clone()).is_none()
        || !source.is_char_boundary(insertion)
    {
        return Err(BlockEditError::Ambiguous);
    }
    let (range, left, right, moved_first) = if insertion < moving.start {
        (
            insertion..moving.end,
            moving.clone(),
            insertion..moving.start,
            true,
        )
    } else {
        (
            moving.start..insertion,
            moving.end..insertion,
            moving.clone(),
            false,
        )
    };
    let line_ending = preferred_line_ending(source);
    let tight = tight_list_join(source, blocks, &left, &right);
    let suffix_range = range.end..source.len();
    let preserve_list_boundary = tight
        && !source[suffix_range.clone()].trim().is_empty()
        && !tight_list_join(source, blocks, &right, &suffix_range);
    let (mut left, right) = (&source[left], &source[right]);
    let mut trailing_gap = "";
    if preserve_list_boundary && trailing_line_breaks(right) == 1 {
        // The old final sibling owns the gap before the next block. Keep
        // that gap at this boundary when its sibling takes its place.
        let tail_start = left.trim_end_matches([' ', '\t', '\r', '\n']).len();
        if let Some(offset) = left[tail_start..].find('\n') {
            let gap_start = tail_start + offset + 1;
            if left[gap_start..].contains('\n') {
                (left, trailing_gap) = left.split_at(gap_start);
            }
        }
    }
    let separator = block_join_separator(left, right, line_ending, if tight { 1 } else { 2 });
    let selected_start = if moved_first {
        range.start
    } else {
        range.start + left.len() + separator.len()
    };
    let mut replacement = format!("{left}{separator}{right}{trailing_gap}");
    if preserve_list_boundary {
        let suffix_separator =
            block_join_separator(&replacement, &source[suffix_range], line_ending, 2);
        replacement.push_str(&suffix_separator);
    }
    Ok(BlockEdit {
        document_version: version,
        range,
        replacement,
        selection_after: selected_start..selected_start,
    })
}

fn trailing_line_breaks(text: &str) -> usize {
    let start = text.trim_end_matches([' ', '\t', '\r', '\n']).len();
    text[start..].bytes().filter(|&byte| byte == b'\n').count()
}

fn block_join_separator(left: &str, right: &str, ending: &str, minimum: usize) -> String {
    if left.is_empty() || right.is_empty() {
        return String::new();
    }
    let right_start = right.len() - right.trim_start_matches([' ', '\t', '\r', '\n']).len();
    let existing = trailing_line_breaks(left)
        + right[..right_start]
            .bytes()
            .filter(|&byte| byte == b'\n')
            .count();
    let needs_line_start = !left.ends_with('\n') && !right.starts_with(['\r', '\n']);
    ending.repeat(
        minimum
            .saturating_sub(existing)
            .max(usize::from(needs_line_start)),
    )
}

/// A single line ending is sufficient only inside the same original list.
/// Use canonical subtree ownership: the last visible row may be nested code
/// or a child item rather than the parent whose sibling is on the other side.
fn tight_list_join(
    source: &str,
    blocks: &[VisualBlock],
    left: &Range<usize>,
    right: &Range<usize>,
) -> bool {
    let Some((right_index, right_block)) = blocks.iter().enumerate().find(|(_, block)| {
        block.source_range.start >= right.start
            && block.source_range.start < right.end
            && !matches!(block.kind, VisualBlockKind::Whitespace)
    }) else {
        return false;
    };
    let Some(signature) = list_signature(source, right_block) else {
        return false;
    };
    let left_index = blocks
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, block)| {
            block.source_range.start >= left.start && block.source_range.start < left.end
        })
        .find_map(|(index, block)| {
            (list_signature(source, block) == Some(signature)
                && source_block_unit(source, blocks, index)
                    .is_ok_and(|unit| unit.source_range.end == left.end))
            .then_some(index)
        });
    let Some(left_index) = left_index else {
        return false;
    };
    let start = left_index.min(right_index);
    let end = left_index.max(right_index);
    let mut subtree_end = 0;
    blocks[start..=end].iter().all(|block| {
        if list_signature(source, block) == Some(signature) {
            subtree_end = block.list_subtree_end.unwrap_or(block.source_range.end);
            return true;
        }
        matches!(block.kind, VisualBlockKind::Whitespace) || block.source_range.start < subtree_end
    })
}

fn list_signature(source: &str, block: &VisualBlock) -> Option<(usize, bool, u8)> {
    let VisualBlockKind::ListItem { level, ordered, .. } = block.kind else {
        return None;
    };
    if block.quote_context.is_some() {
        return None;
    }
    let prefix = source
        .get(block.block_prefix.as_ref()?.source_range.clone())?
        .trim_start_matches([' ', '\t']);
    let marker = prefix.bytes().find(|b| !b.is_ascii_digit())?;
    Some((level, ordered, marker))
}

pub fn indent_subtree(
    source: &str,
    version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    caret: usize,
    outdent: bool,
) -> Result<BlockEdit, BlockEditError> {
    let (index, block) = validate_block_target(version, blocks, target)?;
    let VisualBlockKind::ListItem { level, .. } = block.kind else {
        return Err(BlockEditError::Unsupported);
    };
    if block.quote_context.is_some() {
        return Err(BlockEditError::Unsupported);
    }
    let unit = source_block_unit(source, blocks, index)?;
    let line_at = |block: &VisualBlock| {
        source[block.source_range.start..]
            .lines()
            .next()
            .unwrap_or("")
    };
    let indent = line_indent_columns(line_at(block));
    let (new_indent, new_level) = if outdent {
        let parent_level = level
            .checked_sub(1)
            .filter(|depth| *depth > 0)
            .ok_or(BlockEditError::Unsupported)?;
        let parent = blocks[..index].iter().rev().find(|candidate| {
            matches!(candidate.kind, VisualBlockKind::ListItem { level, .. } if level == parent_level)
                && candidate.list_subtree_end.is_some_and(|end| end > unit.source_range.start)
        }).ok_or(BlockEditError::Unsupported)?;
        (line_indent_columns(line_at(parent)), parent_level)
    } else {
        let (previous_index, previous) = blocks[..index].iter().enumerate().rev()
            .find(|(_, candidate)| {
                matches!(candidate.kind, VisualBlockKind::ListItem { level: depth, .. } if depth <= level)
            })
            .filter(|(_, candidate)| {
                matches!(candidate.kind, VisualBlockKind::ListItem { level: depth, .. } if depth == level)
                    && candidate.quote_context.is_none()
            })
            .ok_or(BlockEditError::Unsupported)?;
        let previous_unit = source_block_unit(source, blocks, previous_index)?;
        if previous_unit.source_range.end != unit.source_range.start {
            return Err(BlockEditError::Unsupported);
        }
        let minimum = list_content_column(line_at(previous)).ok_or(BlockEditError::Ambiguous)?;
        let existing_child = blocks[previous_index + 1..index].iter().find(|candidate| {
            matches!(candidate.kind, VisualBlockKind::ListItem { level: depth, .. } if depth == level + 1)
        });
        let fallback =
            if matches!(
                previous.kind,
                VisualBlockKind::ListItem { ordered: false, .. }
            ) && matches!(block.kind, VisualBlockKind::ListItem { ordered: false, .. })
            {
                minimum.max(line_indent_columns(line_at(previous)) + 4)
            } else {
                minimum
            };
        let target = existing_child
            .map(|child| line_indent_columns(line_at(child)))
            .unwrap_or(fallback)
            .clamp(minimum, minimum + 3);
        (target, level + 1)
    };
    if (outdent && new_indent >= indent) || (!outdent && new_indent <= indent) {
        return Err(BlockEditError::Ambiguous);
    }
    let delta = new_indent as isize - indent as isize;
    let mut replacement = String::new();
    let mut old_offset = unit.source_range.start;
    let mut new_caret = caret;
    for line in source[unit.source_range.clone()].split_inclusive('\n') {
        if line.trim().is_empty() {
            replacement.push_str(line);
        } else {
            let prefix = line.len() - line.trim_start_matches([' ', '\t']).len();
            let columns = line_indent_columns(line)
                .checked_add_signed(delta)
                .ok_or(BlockEditError::Ambiguous)?;
            replacement.push_str(&" ".repeat(columns));
            replacement.push_str(&line[prefix..]);
            if old_offset <= caret {
                let before = prefix.min(caret - old_offset);
                let after = if before == prefix {
                    columns
                } else {
                    before.min(columns)
                };
                new_caret = new_caret + after - before;
            }
        }
        old_offset += line.len();
    }

    let expected: Vec<_> = blocks[unit.blocks.clone()]
        .iter()
        .filter_map(|block| match block.kind {
            VisualBlockKind::ListItem { level: depth, .. } => Some(depth + new_level - level),
            _ => None,
        })
        .collect();
    if !reindent_preserves_subtree(source, &unit.source_range, &replacement, 0, &expected) {
        if outdent {
            return Err(BlockEditError::Ambiguous);
        }
        // An ordered marker that starts above one cannot interrupt a
        // paragraph. A blank line gives that child its own block.
        let ending = preferred_line_ending(source);
        replacement.insert_str(0, ending);
        if !reindent_preserves_subtree(
            source,
            &unit.source_range,
            &replacement,
            ending.len(),
            &expected,
        ) {
            return Err(BlockEditError::Ambiguous);
        }
        new_caret += ending.len();
    }
    Ok(BlockEdit {
        document_version: version,
        range: unit.source_range,
        replacement,
        selection_after: new_caret..new_caret,
    })
}

fn line_indent_columns(line: &str) -> usize {
    line.bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .fold(0, |column, byte| {
            if byte == b'\t' {
                column + 4 - column % 4
            } else {
                column + 1
            }
        })
}

fn list_content_column(line: &str) -> Option<usize> {
    let marker = line.trim_start_matches([' ', '\t']);
    let digits = marker.bytes().take_while(u8::is_ascii_digit).count();
    let marker_len = match marker.as_bytes().get(digits) {
        Some(b'.' | b')') if digits > 0 => digits + 1,
        Some(b'-' | b'+' | b'*') if digits == 0 => 1,
        _ => return None,
    };
    let column = line_indent_columns(line) + marker_len;
    let rest = &marker[marker_len..];
    let end = rest
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .fold(column, |col, byte| {
            if byte == b'\t' {
                col + 4 - col % 4
            } else {
                col + 1
            }
        });
    let padding = end - column;
    Some(
        column
            + if (1..=4).contains(&padding) && !rest.trim().is_empty() {
                padding
            } else {
                1
            },
    )
}

fn reindent_preserves_subtree(
    source: &str,
    range: &Range<usize>,
    replacement: &str,
    prefix_len: usize,
    expected: &[usize],
) -> bool {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};

    let candidate = format!(
        "{}{}{}",
        &source[..range.start],
        replacement,
        &source[range.end..]
    );
    let root_start = range.start + prefix_len;
    let payload_end = range.start + replacement.trim_end().len();
    let mut depth = 0;
    let mut seen = 0;
    for (event, span) in
        Parser::new_ext(&candidate, crate::parse::markdown_options()).into_offset_iter()
    {
        if span.start >= payload_end {
            break;
        }
        match event {
            Event::Start(Tag::List(_)) => depth += 1,
            Event::End(TagEnd::List(_)) => depth -= 1,
            Event::Start(Tag::Item) if span.start >= root_start => {
                if expected.get(seen) != Some(&depth) {
                    return false;
                }
                if seen == 0 {
                    let start = candidate[..span.start]
                        .rfind('\n')
                        .map_or(0, |index| index + 1);
                    if start != root_start
                        || span.end < payload_end
                        || span.end > range.start + replacement.len()
                    {
                        return false;
                    }
                }
                seen += 1;
            }
            _ => {}
        }
    }
    seen == expected.len()
}

pub fn slash_query_at(source: &str, cursor: usize, document_version: u64) -> Option<SlashQuery> {
    if cursor > source.len() || !source.is_char_boundary(cursor) {
        return None;
    }
    let line_start = source[..cursor].rfind('\n').map_or(0, |index| index + 1);
    // Existing paragraph content and CRLF after the caret are not part of
    // the typed command. Including them made opening depend on the suffix
    // and could consume that suffix when the command was confirmed.
    let before_caret = source.get(line_start..cursor)?;
    let (marker_index, marker_len) =
        before_caret
            .char_indices()
            .rev()
            .find_map(|(index, character)| {
                let valid_prefix = index == 0
                    || before_caret[..index]
                        .chars()
                        .next_back()
                        .is_some_and(char::is_whitespace);
                (matches!(character, '/' | '、') && valid_prefix)
                    .then_some((index, character.len_utf8()))
            })?;
    if before_caret[..marker_index].contains("://") {
        return None;
    }
    let query = &before_caret[marker_index + marker_len..];
    if query.contains(['/', '\\', '、']) || query.chars().any(char::is_whitespace) {
        return None;
    }
    Some(SlashQuery {
        document_version,
        source_range: line_start + marker_index..cursor,
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
    let authored_query = source.get(query.source_range.clone());
    if query.document_version != current_version
        || !authored_query.is_some_and(|authored| {
            authored
                .strip_prefix('/')
                .or_else(|| authored.strip_prefix('、'))
                == Some(query.query.as_str())
        })
    {
        return Err(BlockEditError::Stale);
    }
    let line_ending = preferred_line_ending(source);
    let (mut replacement, caret) = slash_template(command, line_ending);
    if matches!(
        command,
        SlashCommand::CodeBlock | SlashCommand::Divider | SlashCommand::Table
    ) && source[query.source_range.end..]
        .split('\n')
        .next()
        .is_some_and(|suffix| !suffix.trim_end_matches('\r').is_empty())
    {
        replacement.push_str(line_ending);
        // GFM accepts pipe-less body rows. A blank line prevents the
        // existing paragraph from becoming an extra row of the new table.
        if command == SlashCommand::Table {
            replacement.push_str(line_ending);
        }
    }
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
        && quote.depth != 1
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
    !matches!(
        block.kind,
        VisualBlockKind::Whitespace | VisualBlockKind::Unsupported
    ) && (block.list_depth == 0 || matches!(block.kind, VisualBlockKind::ListItem { .. }))
}

pub fn block_can_transform_at(blocks: &[VisualBlock], index: usize) -> bool {
    blocks.get(index).is_some_and(block_can_transform) && !block_has_nested_children(blocks, index)
}

pub fn block_can_reorder_at(blocks: &[VisualBlock], index: usize) -> bool {
    blocks.get(index).is_some_and(block_can_reorder)
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
    if matches!(transform, BlockTransform::Divider | BlockTransform::Table)
        && !body.trim().is_empty()
    {
        return Err(BlockEditError::Unsupported);
    }
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
    let selection_after = if block
        .quote_context
        .as_ref()
        .is_some_and(|quote| quote.edge != VisualQuoteGroupEdge::Only)
        && !matches!(transform, BlockTransform::Quote)
    {
        replacement.insert_str(0, line_ending);
        if !replacement.ends_with('\n') {
            replacement.push_str(line_ending);
        }
        replacement.push_str(line_ending);
        selection_after.start + line_ending.len()..selection_after.end + line_ending.len()
    } else {
        selection_after
    };
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
    let (index, block) = validate_block_target(current_version, blocks, target)?;
    if !block_can_reorder_at(blocks, index) {
        return Err(BlockEditError::Unsupported);
    }
    let unit = source_unit(source, blocks, index)?;
    let authored = source.get(unit.clone()).ok_or(BlockEditError::Ambiguous)?;
    // A list item outside a quote is separated from its copy by its own marker,
    // so the copy keeps the item's boundary instead of a blank line that would
    // loosen the list.
    let sibling_bounded =
        block.quote_context.is_none() && matches!(block.kind, VisualBlockKind::ListItem { .. });
    let separator = match trailing_line_endings(authored) {
        "" => preferred_line_ending(source).repeat(if sibling_bounded { 1 } else { 2 }),
        "\n" | "\r" | "\r\n" if !sibling_bounded => preferred_line_ending(source).to_string(),
        _ => String::new(),
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

/// Insert an editable empty paragraph after the complete source unit.
pub fn add_paragraph_below(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
) -> Result<BlockEdit, BlockEditError> {
    let (index, _) = validate_block_target(current_version, blocks, target)?;
    if !block_can_reorder_at(blocks, index) {
        return Err(BlockEditError::Unsupported);
    }
    let unit = source_block_unit(source, blocks, index)?;
    let authored = source
        .get(unit.source_range.clone())
        .ok_or(BlockEditError::Ambiguous)?;
    let ending = preferred_line_ending(source);
    let trailing = authored
        .chars()
        .rev()
        .take_while(|c| matches!(c, '\r' | '\n'))
        .filter(|c| *c == '\n')
        .count();
    let padding = 2usize.saturating_sub(trailing);
    let insertion = unit.source_range.end;
    let caret = insertion + padding * ending.len();
    Ok(BlockEdit {
        document_version: current_version,
        range: insertion..insertion,
        replacement: ending.repeat(padding + 1),
        selection_after: caret..caret,
    })
}

/// Insert a fresh block after the complete clicked source unit, without
/// transforming its content or consuming a selection elsewhere in the document.
pub fn insert_block_below(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    command: SlashCommand,
) -> Result<BlockEdit, BlockEditError> {
    let (template, caret) = slash_template(command, preferred_line_ending(source));
    insert_template_below(
        source,
        current_version,
        blocks,
        target,
        &template,
        caret,
        command == SlashCommand::Divider,
    )
}

pub fn insert_callout_below(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    kind: crate::AlertKind,
) -> Result<BlockEdit, BlockEditError> {
    let template = callout_template(source, kind);
    let caret = template.len();
    insert_template_below(
        source,
        current_version,
        blocks,
        target,
        &template,
        caret,
        false,
    )
}

/// Insert confirmed Markdown content as a standalone block after the complete
/// target source unit, isolating it from both the target and any following block.
pub fn insert_content_below(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    content: &str,
) -> Result<BlockEdit, BlockEditError> {
    insert_template_below(
        source,
        current_version,
        blocks,
        target,
        content,
        content.len(),
        false,
    )
}

fn callout_template(source: &str, kind: crate::AlertKind) -> String {
    format!(
        "> [!{}]{}> ",
        kind.label().to_uppercase(),
        preferred_line_ending(source)
    )
}

/// Exact content range of a plain blank line, excluding its LF/CRLF.
/// Structural quote/list gaps and offsets belonging to the next row are not
/// plain insertion targets, even when their visual kind is Whitespace.
pub fn empty_paragraph_range(
    source: &str,
    block: &VisualBlock,
    offset: usize,
) -> Option<Range<usize>> {
    if !matches!(block.kind, VisualBlockKind::Whitespace)
        || block.quote_context.is_some()
        || block.list_depth > 0
        || block.source_island.is_some()
        || offset < block.source_range.start
        || offset > block.source_range.end
        || (offset == block.source_range.end && offset != source.len())
        || !source.is_char_boundary(offset)
    {
        return None;
    }
    let start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let end = source[offset..]
        .find('\n')
        .map_or(source.len(), |relative| offset + relative);
    let end = if end > start && source.as_bytes()[end - 1] == b'\r' {
        end - 1
    } else {
        end
    };
    (start >= block.source_range.start
        && end <= block.source_range.end
        && offset <= end
        && source[start..end]
            .chars()
            .all(|ch| matches!(ch, ' ' | '\t')))
    .then_some(start..end)
}

pub fn insert_block_at_empty_paragraph(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    offset: usize,
    command: SlashCommand,
) -> Result<BlockEdit, BlockEditError> {
    let (template, caret) = slash_template(command, preferred_line_ending(source));
    insert_template_at_empty_paragraph(
        source,
        current_version,
        blocks,
        target,
        offset,
        &template,
        caret,
        command == SlashCommand::Divider,
    )
}

pub fn insert_callout_at_empty_paragraph(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    offset: usize,
    kind: crate::AlertKind,
) -> Result<BlockEdit, BlockEditError> {
    let template = callout_template(source, kind);
    let caret = template.len();
    insert_template_at_empty_paragraph(
        source,
        current_version,
        blocks,
        target,
        offset,
        &template,
        caret,
        false,
    )
}

fn insert_template_at_empty_paragraph(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    offset: usize,
    template: &str,
    caret: usize,
    continuation_paragraph: bool,
) -> Result<BlockEdit, BlockEditError> {
    let (_, block) = validate_block_target(current_version, blocks, target)?;
    let range = empty_paragraph_range(source, block, offset).ok_or(BlockEditError::Unsupported)?;
    let ending = preferred_line_ending(source);
    if template.is_empty() {
        // Inserting Paragraph on a blank row creates another editable line.
        return Ok(BlockEdit {
            document_version: current_version,
            replacement: ending.to_string(),
            selection_after: range.start..range.start,
            range,
        });
    }
    let before = &source[..range.start];
    let trailing = before
        .chars()
        .rev()
        .take_while(|ch| matches!(ch, '\r' | '\n'))
        .filter(|ch| *ch == '\n')
        .count();
    let padding = if before.is_empty() {
        0
    } else {
        2usize.saturating_sub(trailing)
    };
    let after = &source[range.end..];
    let leading = after
        .chars()
        .take_while(|ch| matches!(ch, '\r' | '\n'))
        .filter(|ch| *ch == '\n')
        .count();
    let minimum = if after.trim().is_empty() { 1usize } else { 2 }
        * if continuation_paragraph { 2 } else { 1 };
    let replacement = format!(
        "{}{template}{}",
        ending.repeat(padding),
        ending.repeat(minimum.saturating_sub(leading))
    );
    let position = range.start
        + padding * ending.len()
        + caret
        + if continuation_paragraph {
            2 * ending.len()
        } else {
            0
        };
    Ok(BlockEdit {
        document_version: current_version,
        range,
        replacement,
        selection_after: position..position,
    })
}

fn insert_template_below(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    template: &str,
    caret: usize,
    continuation_paragraph: bool,
) -> Result<BlockEdit, BlockEditError> {
    let mut edit = add_paragraph_below(source, current_version, blocks, target)?;
    if !template.is_empty() {
        let offset = edit.selection_after.start - edit.range.start;
        edit.replacement.truncate(offset);
        edit.replacement.push_str(template);
        // Isolate the next block, including pipe-less prose after a GFM table
        // and paragraphs following a list/quote. Never rewrite existing bytes.
        let ending = preferred_line_ending(source);
        let separators = if edit.range.end < source.len() { 2 } else { 1 };
        edit.replacement
            .push_str(&ending.repeat(separators * if continuation_paragraph { 2 } else { 1 }));
        // Keep divider typing off its marker and before any following block.
        let position = edit.range.start
            + offset
            + caret
            + if continuation_paragraph {
                2 * ending.len()
            } else {
                0
            };
        edit.selection_after = position..position;
    }
    Ok(edit)
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

    let raw_insertion = match placement {
        BlockPlacement::Before => destination_unit.start,
        BlockPlacement::After => destination_unit.end,
    };
    move_source_units(source, current_version, blocks, moving_unit, raw_insertion)
}

pub fn adjacent_reorder_target(
    source: &str,
    current_version: u64,
    blocks: &[VisualBlock],
    target: &BlockTarget,
    forward: bool,
) -> Result<BlockTarget, BlockEditError> {
    let (index, block) = validate_block_target(current_version, blocks, target)?;
    let unit = source_block_unit(source, blocks, index)?;
    let depth = |block: &VisualBlock| match block.kind {
        VisualBlockKind::ListItem { level, .. } => level.saturating_sub(1),
        _ => block.list_depth,
    };
    let eligible = |candidate_index: usize| {
        block_can_reorder_at(blocks, candidate_index)
            && depth(&blocks[candidate_index]) == depth(block)
            && source_block_unit(source, blocks, candidate_index)
                .is_ok_and(|candidate| !ranges_overlap(&unit.source_range, &candidate.source_range))
    };
    let candidate = if forward {
        blocks
            .iter()
            .enumerate()
            .skip(unit.blocks.end)
            .find(|(candidate_index, _)| eligible(*candidate_index))
            .map(|(_, block)| block)
    } else {
        blocks
            .iter()
            .enumerate()
            .take(unit.blocks.start)
            .rev()
            .find(|(candidate_index, _)| eligible(*candidate_index))
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
        let authored = source
            .get(payload.source_range.clone())
            .ok_or(BlockEditError::Ambiguous)?;
        // An indented code body sheds its per-line indentation, or the
        // transformed block would still parse as indented code.
        let dedented;
        let payload = if payload.kind == crate::VisualEditorFieldKind::IndentedCodePayload {
            dedented = crate::indented_code_display(authored, payload.source_range.start).0;
            dedented.as_str()
        } else {
            authored
        };
        return Ok(payload
            .strip_suffix("\r\n")
            .or_else(|| payload.strip_suffix('\n'))
            .unwrap_or(payload)
            .to_string());
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
    Ok(source_block_unit(source, blocks, index)?.source_range)
}

fn block_has_nested_children(blocks: &[VisualBlock], index: usize) -> bool {
    let Some(VisualBlockKind::ListItem { level, .. }) = blocks.get(index).map(|block| &block.kind)
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
    use crate::{MarkdownDocument, PreviewBlock};

    #[test]
    fn issue_52_list_indent_moves_exactly_one_level_and_keeps_children() {
        for (source, item, outdent, expected) in [
            (
                "1. Gather\n2. Inspect\n    1. Notes\n3. Publish\n",
                "Inspect",
                false,
                "1. Gather\n\n   2. Inspect\n       1. Notes\n3. Publish\n",
            ),
            (
                "- Atlas\n  - Chapter\n    - Earlier\n    - Parcel\n      - Detail\n- Tail\n",
                "Parcel",
                true,
                "- Atlas\n  - Chapter\n    - Earlier\n  - Parcel\n    - Detail\n- Tail\n",
            ),
        ] {
            let doc = MarkdownDocument::from_text(source);
            let blocks = doc.visual_blocks_shared();
            let offset = source.find(item).unwrap();
            let block = blocks
                .iter()
                .find(|b| b.source_range.contains(&offset))
                .unwrap();
            let edit = indent_subtree(
                source,
                doc.version(),
                &blocks,
                &BlockTarget::from_block(doc.version(), block),
                offset,
                outdent,
            )
            .unwrap();
            let mut changed = source.to_string();
            changed.replace_range(edit.range, &edit.replacement);
            assert_eq!(changed, expected);
            assert_eq!(&changed[edit.selection_after.start..][..item.len()], item);
        }
    }

    #[test]
    fn issue_54_list_indent_keeps_a_continuation_paragraph_with_its_item() {
        for ending in ["\n", "\r\n"] {
            let source = "- Dock\n- Inspect\n    - Lens\n    - Frame\n\n    Caption.\n\n- Exit\n"
                .replace('\n', ending);
            let expected = "- Dock\n    - Inspect\n        - Lens\n        - Frame\n\n        Caption.\n\n- Exit\n"
                .replace('\n', ending);
            let doc = MarkdownDocument::from_text(&source);
            let blocks = doc.visual_blocks_shared();
            let offset = source.find("Inspect").unwrap();
            let block = blocks
                .iter()
                .find(|b| b.source_range.contains(&offset))
                .unwrap();
            let edit = indent_subtree(
                &source,
                doc.version(),
                &blocks,
                &BlockTarget::from_block(doc.version(), block),
                offset,
                false,
            )
            .unwrap();
            let mut changed = source.clone();
            changed.replace_range(edit.range, &edit.replacement);
            assert_eq!(changed, expected);
            assert!(
                MarkdownDocument::from_text(&changed)
                    .render_html_fragment()
                    .contains("<li>Frame</li>\n</ul>\n<p>Caption.</p>\n</li>")
            );
        }
    }

    #[test]
    fn issue_53_reorder_keeps_the_gap_before_independent_prose() {
        for ending in ["\n", "\r\n"] {
            let source =
                "# Garden plan\n\n- East\n  - Carrots\n- West\n  - Beans\n\nWater after sunset.\n"
                    .replace('\n', ending);
            let expected =
                "# Garden plan\n\n- West\n  - Beans\n- East\n  - Carrots\n\nWater after sunset.\n"
                    .replace('\n', ending);
            let doc = MarkdownDocument::from_text(&source);
            let blocks = doc.visual_blocks_shared();
            let target = |name: &str| {
                let offset = source.find(name).unwrap();
                BlockTarget::from_block(
                    doc.version(),
                    blocks
                        .iter()
                        .find(|b| b.source_range.contains(&offset))
                        .unwrap(),
                )
            };
            for (moved, anchor, placement) in [
                ("East", "West", BlockPlacement::After),
                ("West", "East", BlockPlacement::Before),
            ] {
                let edit = reorder_block(
                    &source,
                    doc.version(),
                    &blocks,
                    &target(moved),
                    &target(anchor),
                    placement,
                )
                .unwrap();
                let mut changed = source.clone();
                changed.replace_range(edit.range, &edit.replacement);
                assert_eq!(changed, expected);
                assert!(
                    MarkdownDocument::from_text(&changed)
                        .render_html_fragment()
                        .ends_with("</ul>\n<p>Water after sunset.</p>\n")
                );
            }
        }
    }

    #[test]
    fn empty_paragraph_insertions_use_the_exact_line_and_preserve_neighbors() {
        for ending in ["\n", "\r\n"] {
            for (raw, logical_offset) in [
                ("", 0),
                (" \t", 1),
                ("before\n\nnext", 7),
                ("before\n\n\n\nnext", 8),
                ("before\n", 7),
            ] {
                let source = raw.replace('\n', ending);
                let offset = raw[..logical_offset].replace('\n', ending).len();
                let doc = MarkdownDocument::from_text(&source);
                let blocks = doc.visual_blocks_shared();
                let block = blocks
                    .iter()
                    .find(|block| empty_paragraph_range(&source, block, offset).is_some())
                    .unwrap_or_else(|| {
                        panic!("empty target for {source:?} at {offset}: {blocks:?}")
                    });
                let target = BlockTarget::from_block(doc.version(), block);
                let edits = SlashCommand::ALL
                    .into_iter()
                    .map(|command| {
                        insert_block_at_empty_paragraph(
                            &source,
                            doc.version(),
                            &blocks,
                            &target,
                            offset,
                            command,
                        )
                    })
                    .chain(
                        [
                            crate::AlertKind::Note,
                            crate::AlertKind::Tip,
                            crate::AlertKind::Important,
                            crate::AlertKind::Warning,
                            crate::AlertKind::Caution,
                        ]
                        .into_iter()
                        .map(|kind| {
                            insert_callout_at_empty_paragraph(
                                &source,
                                doc.version(),
                                &blocks,
                                &target,
                                offset,
                                kind,
                            )
                        }),
                    );
                for edit in edits {
                    let edit = edit.unwrap();
                    assert_eq!(
                        edit.range,
                        empty_paragraph_range(&source, block, offset).unwrap()
                    );
                    let mut result = source.clone();
                    result.replace_range(edit.range.clone(), &edit.replacement);
                    assert_eq!(&result[..edit.range.start], &source[..edit.range.start]);
                    assert!(result.ends_with(&source[edit.range.end..]));
                    assert!(result.is_char_boundary(edit.selection_after.start));
                    if raw.contains("next") {
                        let parsed = MarkdownDocument::from_text(&result);
                        assert!(
                            parsed.visual_blocks_shared().iter().any(|block| matches!(
                                block.kind,
                                VisualBlockKind::Paragraph
                            ) && result
                                [block.source_range.clone()]
                            .trim()
                                == "next"),
                            "neighbor swallowed: {result:?}"
                        );
                    }
                    if source.contains("\r\n") {
                        assert!(!result.replace("\r\n", "").contains('\n'));
                    }
                }
                assert_eq!(
                    insert_block_at_empty_paragraph(
                        &source,
                        doc.version() + 1,
                        &blocks,
                        &target,
                        offset,
                        SlashCommand::Heading(1)
                    ),
                    Err(BlockEditError::Stale)
                );
                assert!(
                    insert_callout_at_empty_paragraph(
                        &source,
                        doc.version(),
                        &blocks,
                        &target,
                        source.len() + 1,
                        crate::AlertKind::Note
                    )
                    .is_err()
                );
            }
        }
        let source = "> [!NOTE]\n> \n";
        let doc = MarkdownDocument::from_text(source);
        assert!(doc.visual_blocks_shared().iter().all(|block| {
            empty_paragraph_range(source, block, source.find("> \n").unwrap() + 2).is_none()
        }));
    }

    #[test]
    fn inserted_blocks_preserve_complete_units_neighbors_endings_and_targets() {
        for ending in ["\n", "\r\n"] {
            for unit in [
                "clicked 中文🙂",
                "- parent\n  - child",
                "> quote\n> continuation",
                "```rust\nlet x = 1;\n```",
                "<!-- markion-cols:30,70 -->\n| 名称 | 内容 |\n| :--- | ---: |\n| 中文🙂 | **keep** |",
            ] {
                for terminal in [false, true] {
                    let source = format!(
                        "before\n\n{unit}{}",
                        if terminal { "" } else { "\n\nafter" }
                    )
                    .replace('\n', ending);
                    let doc = MarkdownDocument::from_text(&source);
                    let blocks = doc.visual_blocks_shared();
                    let block = blocks
                        .iter()
                        .find(|b| b.source_range.start == "before\n\n".replace('\n', ending).len())
                        .unwrap();
                    let target = BlockTarget::from_block(doc.version(), block);
                    let edits = SlashCommand::ALL
                        .into_iter()
                        .map(|command| {
                            insert_block_below(&source, doc.version(), &blocks, &target, command)
                        })
                        .chain(
                            [
                                crate::AlertKind::Note,
                                crate::AlertKind::Tip,
                                crate::AlertKind::Important,
                                crate::AlertKind::Warning,
                                crate::AlertKind::Caution,
                            ]
                            .into_iter()
                            .map(|kind| {
                                insert_callout_below(&source, doc.version(), &blocks, &target, kind)
                            }),
                        );
                    for edit in edits {
                        let edit = edit.unwrap();
                        assert!(edit.range.is_empty());
                        let prefix = format!("before\n\n{unit}").replace('\n', ending);
                        assert!(edit.range.start >= prefix.len());
                        let mut result = source.clone();
                        result.replace_range(edit.range.clone(), &edit.replacement);
                        assert!(result.starts_with(&prefix));
                        assert!(terminal || result.ends_with(&format!("{ending}{ending}after")));
                        assert!(result.is_char_boundary(edit.selection_after.start));
                        let parsed = MarkdownDocument::from_text(&result);
                        if !terminal {
                            assert!(
                                parsed.visual_blocks_shared().iter().any(|block| matches!(
                                    block.kind,
                                    VisualBlockKind::Paragraph
                                ) && result
                                    [block.source_range.clone()]
                                .trim()
                                    == "after"),
                                "neighbor was swallowed: {result:?}"
                            );
                        }
                        if ending == "\r\n" {
                            assert!(!result.replace("\r\n", "").contains('\n'));
                        }
                    }
                    assert_eq!(
                        insert_block_below(
                            &source,
                            doc.version() + 1,
                            &blocks,
                            &target,
                            SlashCommand::Heading(2)
                        ),
                        Err(BlockEditError::Stale)
                    );
                    let mut invalid = target.clone();
                    invalid.source_range.end -= 1;
                    assert_eq!(
                        insert_callout_below(
                            &source,
                            doc.version(),
                            &blocks,
                            &invalid,
                            crate::AlertKind::Note
                        ),
                        Err(BlockEditError::Stale)
                    );
                }
            }
        }
    }

    #[test]
    fn advanced_subtree_moves_keep_nested_code_and_reject_self_drops() {
        let source = "intro\n\n- parent\n  - child\n\n  ```rust\n  let x = 1;\n  ```\n\nlast";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|b| matches!(b.kind, VisualBlockKind::ListItem { level: 1, .. }))
            .unwrap();
        let unit = source_block_unit(source, &blocks, index).unwrap();
        assert!(source[unit.source_range.clone()].contains("let x = 1;"));
        assert!(
            move_source_units(
                source,
                doc.version(),
                &blocks,
                unit.source_range.clone(),
                unit.source_range.start + 2
            )
            .is_err()
        );
        let edit = move_source_units(
            source,
            doc.version(),
            &blocks,
            unit.source_range,
            source.len(),
        )
        .unwrap();
        let mut result = source.to_string();
        result.replace_range(edit.range, &edit.replacement);
        assert!(result.starts_with("intro\n\nlast\n\n- parent\n  - child"));
        assert!(result.ends_with("  ```\n\n"));
    }

    #[test]
    fn advanced_quote_extraction_splits_containers_and_subtree_outdent_is_crlf_safe() {
        let source = "> before\n>\n> middle\n>\n> after\n\nlast";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let middle = blocks
            .iter()
            .find(|b| {
                b.editable_runs
                    .iter()
                    .any(|run| run.visible_text == "middle")
            })
            .unwrap();
        let edit = transform_block(
            source,
            doc.version(),
            &blocks,
            &BlockTarget::from_block(doc.version(), middle),
            BlockTransform::Text,
        )
        .unwrap();
        let mut result = source.to_string();
        result.replace_range(edit.range, &edit.replacement);
        let parsed = MarkdownDocument::from_text(result).preview_blocks();
        assert_eq!(
            parsed
                .iter()
                .filter(|b| matches!(b, crate::PreviewBlock::BlockQuote { .. }))
                .count(),
            2
        );
        assert!(parsed.iter().any(|b| matches!(b, crate::PreviewBlock::Paragraph { text, .. } if text.text.trim() == "middle")));

        let source = "- root\r\n  - parent\r\n    - 子项\r\n\r\nend";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let parent = blocks
            .iter()
            .find(|b| matches!(b.kind, VisualBlockKind::ListItem { level: 2, .. }))
            .unwrap();
        let edit = indent_subtree(
            source,
            doc.version(),
            &blocks,
            &BlockTarget::from_block(doc.version(), parent),
            source.find("parent").unwrap(),
            true,
        )
        .unwrap();
        let mut result = source.to_string();
        result.replace_range(edit.range, &edit.replacement);
        assert_eq!(result, "- root\r\n- parent\r\n  - 子项\r\n\r\nend");
    }

    #[test]
    fn add_below_owns_a_blank_paragraph_after_complete_units() {
        for ending in ["\n", "\r\n"] {
            for first in [
                "段落😀",
                "- parent\n  - child",
                "> before\n>\n> after",
                "## First",
            ] {
                let first = first.replace('\n', ending);
                let source = format!("{first}{ending}{ending}### Next{ending}");
                let doc = MarkdownDocument::from_text(&source);
                let blocks = doc.visual_blocks_shared();
                let target = BlockTarget::from_block(doc.version(), &blocks[0]);
                let edit = add_paragraph_below(&source, doc.version(), &blocks, &target).unwrap();
                assert_eq!(
                    edit.range,
                    source.find("### Next").unwrap()..source.find("### Next").unwrap()
                );
                assert_eq!(edit.replacement, ending);
                assert_eq!(edit.selection_after, edit.range.start..edit.range.start);
                let mut result = source.clone();
                result.replace_range(edit.range, &edit.replacement);
                assert_eq!(
                    result,
                    format!("{first}{ending}{ending}{ending}### Next{ending}")
                );
                let mut stale = target.clone();
                stale.document_version = doc.version().wrapping_add(1);
                assert_eq!(
                    add_paragraph_below(&source, doc.version(), &blocks, &stale),
                    Err(BlockEditError::Stale)
                );
            }
        }
        for source in ["tail", "tail\n", "tail\n\n"] {
            let doc = MarkdownDocument::from_text(source);
            let blocks = doc.visual_blocks_shared();
            let target = BlockTarget::from_block(doc.version(), &blocks[0]);
            let edit = add_paragraph_below(source, doc.version(), &blocks, &target).unwrap();
            let mut result = source.to_string();
            result.replace_range(edit.range, &edit.replacement);
            assert_eq!(result, "tail\n\n\n");
            assert_eq!(edit.selection_after, 6..6);
        }
    }

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
    fn slash_query_accepts_chinese_marker_and_rejects_urls() {
        let source = "前文 、h2";
        let query = slash_query_at(source, source.len(), 7).unwrap();
        assert_eq!(query.query, "h2");
        let edit = slash_command_edit(source, 7, &query, SlashCommand::Heading(2)).unwrap();
        assert_eq!(edit.replacement, "## ");
        assert!(slash_query_at("https://example.com", 19, 7).is_none());
        assert!(slash_query_at("prefix/query", 12, 7).is_none());
        assert!(slash_query_at("/has space", 10, 7).is_none());
        assert!(slash_query_at("/https://example.com", 20, 7).is_none());
        assert!(slash_query_at("/usr/bin", 8, 7).is_none());
        assert!(slash_query_at("/foo\\bar", 8, 7).is_none());
    }

    #[test]
    fn slash_queries_stop_at_caret_and_preserve_paragraph_suffix_and_crlf() {
        for ending in ["\n", "\r\n"] {
            for (marker, typed) in [("/", ""), ("/", "h2"), ("、", "标题")] {
                for suffix in ["", "Existing paragraph with spaces", "中文 **段落** 🙂"] {
                    let prefix = format!("前文{ending}{ending}  ");
                    let source = format!("{prefix}{marker}{typed}{suffix}{ending}next");
                    let cursor = prefix.len() + marker.len() + typed.len();
                    let query = slash_query_at(&source, cursor, 7).unwrap();
                    assert_eq!(query.query, typed);
                    assert_eq!(query.source_range, prefix.len()..cursor);
                    let edit =
                        slash_command_edit(&source, 7, &query, SlashCommand::Heading(2)).unwrap();
                    let mut result = source.clone();
                    result.replace_range(edit.range, &edit.replacement);
                    assert_eq!(result, format!("{prefix}## {suffix}{ending}next"));
                    assert_eq!(edit.selection_after, prefix.len() + 3..prefix.len() + 3);
                }
            }
        }
        assert!(slash_query_at("/中文", 2, 7).is_none());
        assert!(slash_query_at("/", 2, 7).is_none());
    }

    #[test]
    fn slash_standalone_templates_separate_existing_prose_without_consuming_it() {
        for ending in ["\n", "\r\n"] {
            for command in [
                SlashCommand::CodeBlock,
                SlashCommand::Divider,
                SlashCommand::Table,
            ] {
                let source = format!("/{suffix}{ending}next", suffix = "原段落 with spaces");
                let query = slash_query_at(&source, 1, 7).unwrap();
                let edit = slash_command_edit(&source, 7, &query, command).unwrap();
                let (template, caret) = slash_template(command, ending);
                let separator = ending.repeat(if command == SlashCommand::Table { 2 } else { 1 });
                assert_eq!(edit.replacement, format!("{template}{separator}"));
                assert_eq!(edit.selection_after, caret..caret);
                let mut result = source;
                result.replace_range(edit.range, &edit.replacement);
                assert_eq!(
                    result,
                    format!("{template}{separator}原段落 with spaces{ending}next")
                );
                let document = MarkdownDocument::from_text(result);
                assert!(document.visual_blocks_shared().iter().any(|block| {
                    matches!(block.kind, VisualBlockKind::Paragraph)
                        && document.text()[block.source_range.clone()]
                            .contains("原段落 with spaces")
                }));

                let empty_line = format!("/{ending}next");
                let query = slash_query_at(&empty_line, 1, 7).unwrap();
                let edit = slash_command_edit(&empty_line, 7, &query, command).unwrap();
                assert_eq!(edit.replacement, template);
            }
        }
    }

    #[test]
    fn slash_confirmation_rejects_a_mismatched_query_even_at_the_same_version() {
        let mut query = slash_query_at("/h2", 3, 7).unwrap();
        query.query = "h1".into();
        assert_eq!(
            slash_command_edit("/h2", 7, &query, SlashCommand::Heading(2)),
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
    fn indented_code_transforms_with_its_indentation_removed() {
        let doc = MarkdownDocument::from_text("    let a = 1;\n    let b = 2;\n");
        let index = doc
            .visual_blocks()
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .expect("indented code block");
        let (blocks, target) = target_for(&doc, index);
        let edit = transform_block(
            doc.text(),
            doc.version(),
            &blocks,
            &target,
            BlockTransform::Text,
        )
        .expect("transform indented code");
        let mut transformed = doc.text().to_string();
        transformed.replace_range(edit.range, &edit.replacement);
        assert!(
            !transformed.contains("    let b"),
            "the body lines are no longer indented: {transformed:?}"
        );
        assert!(transformed.contains("let a = 1;") && transformed.contains("let b = 2;"));
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
        ] {
            let edit =
                transform_block(doc.text(), doc.version(), &blocks, &target, transform).unwrap();
            if transform == BlockTransform::CodeBlock {
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
    fn template_transformations_reject_nonempty_content() {
        for source in [
            "中文 **keep**🙂",
            "## Heading",
            "```rust\nlet keep = 1;\n```",
            "<!-- markion-cols:30,70 -->\n| A | B |\n| :--- | ---: |\n| keep | 中文🙂 |",
        ] {
            let doc = MarkdownDocument::from_text(source);
            let (blocks, target) = target_for(&doc, 0);
            for transform in [BlockTransform::Divider, BlockTransform::Table] {
                assert_eq!(
                    transform_block(source, doc.version(), &blocks, &target, transform),
                    Err(BlockEditError::Unsupported),
                    "{transform:?}: {source:?}"
                );
            }
        }
        let doc = MarkdownDocument::from_text(" ");
        let (blocks, target) = target_for(&doc, 0);
        for transform in [BlockTransform::Divider, BlockTransform::Table] {
            assert!(
                transform_block(doc.text(), doc.version(), &blocks, &target, transform).is_ok()
            );
        }
    }

    #[test]
    fn inserted_dividers_own_a_blank_continuation_before_neighbors() {
        for ending in ["\n", "\r\n"] {
            for raw in [
                "<!-- markion-cols:30,70 -->\n| A | B |\n| :--- | ---: |\n| 中文🙂 | **keep** |",
                "<!-- markion-cols:30,70 -->\n| A | B |\n| :--- | ---: |\n| 中文🙂 | **keep** |\n\nnext",
                "paragraph\n\nnext",
                "```rust\nlet keep = 1;\n```\n\nnext",
                "",
                "before\n\nnext",
                "before\n\n\n\nnext",
                "before\n",
            ] {
                let source = raw.replace('\n', ending);
                let doc = MarkdownDocument::from_text(&source);
                let blocks = doc.visual_blocks_shared();
                let actual_ending = preferred_line_ending(&source);
                let blank_offset = if raw.starts_with("before") {
                    Some("before\n".replace('\n', ending).len())
                } else if source.is_empty() {
                    Some(0)
                } else {
                    None
                };
                let block = blocks
                    .iter()
                    .find(|block| {
                        blank_offset.map_or(block.source_range.start == 0, |offset| {
                            empty_paragraph_range(&source, block, offset).is_some()
                        })
                    })
                    .unwrap();
                let target = BlockTarget::from_block(doc.version(), block);
                let edit = if let Some(offset) = blank_offset {
                    insert_block_at_empty_paragraph(
                        &source,
                        doc.version(),
                        &blocks,
                        &target,
                        offset,
                        SlashCommand::Divider,
                    )
                } else {
                    insert_block_below(
                        &source,
                        doc.version(),
                        &blocks,
                        &target,
                        SlashCommand::Divider,
                    )
                }
                .unwrap();
                let mut result = source.clone();
                result.replace_range(edit.range.clone(), &edit.replacement);
                assert_eq!(&result[..edit.range.start], &source[..edit.range.start]);
                assert!(result.ends_with(&source[edit.range.end..]));
                let caret = edit.selection_after.start;
                assert!(
                    result[..caret].ends_with(&format!("---{actual_ending}{actual_ending}")),
                    "{result:?} at {caret}"
                );
                assert!(caret == result.len() || result[caret..].starts_with(actual_ending));
                // Typing belongs to its own paragraph, never the rule or the following block.
                result.insert_str(caret, "continued 中文🙂");
                let parsed = MarkdownDocument::from_text(&result);
                let parsed_blocks = parsed.visual_blocks_shared();
                assert_eq!(
                    parsed_blocks
                        .iter()
                        .filter(|block| matches!(block.kind, VisualBlockKind::Rule))
                        .count(),
                    1
                );
                for text in ["continued 中文🙂", "next"] {
                    if text == "next" && !raw.contains("next") {
                        continue;
                    }
                    assert!(
                        parsed_blocks.iter().any(|block| matches!(
                            block.kind,
                            VisualBlockKind::Paragraph
                        ) && result[block.source_range.clone()]
                            .trim()
                            == text),
                        "{text} merged: {result:?}"
                    );
                }
                if ending == "\r\n" && source.contains(ending) {
                    assert!(!result.replace(ending, "").contains('\n'));
                }
            }
        }
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
        let mut result = doc.text().to_string();
        result.replace_range(moved.range.clone(), &moved.replacement);
        assert_eq!(result, "two\n\none\n\nthree");
        assert!(moved.range.end < doc.text().len());

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
    fn duplicate_tight_list_item_keeps_the_sibling_boundary() {
        for (source, nth, expected) in [
            ("- a\n- b\n", 0, "- a\n- a\n- b\n"),
            ("- a\n- b\n", 1, "- a\n- b\n- b\n"),
            ("- a\n- b", 1, "- a\n- b\n- b"),
            ("- a\r\n- b\r\n", 0, "- a\r\n- a\r\n- b\r\n"),
            ("- a\n- b\n\nAfter\n", 0, "- a\n- a\n- b\n\nAfter\n"),
            (
                "- a\n  - x\n  - y\n- b\n",
                1,
                "- a\n  - x\n  - x\n  - y\n- b\n",
            ),
        ] {
            let doc = MarkdownDocument::from_text(source);
            let blocks = doc.visual_blocks_shared();
            let item = blocks
                .iter()
                .filter(|block| matches!(block.kind, VisualBlockKind::ListItem { .. }))
                .nth(nth)
                .unwrap();
            let target = BlockTarget::from_block(doc.version(), item);
            let edit = duplicate_block(doc.text(), doc.version(), &blocks, &target).unwrap();
            let mut duplicated = source.to_string();
            duplicated.replace_range(edit.range, &edit.replacement);
            assert_eq!(duplicated, expected, "{source:?}");
        }
    }

    #[test]
    fn duplicate_single_newline_paragraph_before_another_block_stays_independent() {
        let source = "Review notes.\n# Agenda\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let paragraph = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .unwrap();
        let target = BlockTarget::from_block(doc.version(), paragraph);
        let edit = duplicate_block(doc.text(), doc.version(), &blocks, &target).unwrap();
        let mut duplicated = source.to_string();
        duplicated.replace_range(edit.range, &edit.replacement);
        assert_eq!(duplicated, "Review notes.\n\nReview notes.\n# Agenda\n");
    }

    #[test]
    fn issue_93_duplicate_single_newline_paragraphs_stay_independent() {
        for ending in ["\n", "\r\n"] {
            let source = format!("Review notes.{ending}");
            let doc = MarkdownDocument::from_text(&source);
            let blocks = doc.visual_blocks_shared();
            let paragraph = blocks
                .iter()
                .find(|block| matches!(block.kind, VisualBlockKind::Paragraph))
                .unwrap();
            let target = BlockTarget::from_block(doc.version(), paragraph);
            let edit = duplicate_block(doc.text(), doc.version(), &blocks, &target).unwrap();
            let expected = format!("Review notes.{ending}{ending}Review notes.{ending}");
            assert_eq!(edit.replacement, expected);
            let duplicated = MarkdownDocument::from_text(&expected);
            assert_eq!(
                duplicated
                    .preview_blocks()
                    .iter()
                    .filter(|block| matches!(block, PreviewBlock::Paragraph { .. }))
                    .count(),
                2
            );

            let blocks = duplicated.visual_blocks_shared();
            let paragraphs = blocks
                .iter()
                .filter(|block| matches!(block.kind, VisualBlockKind::Paragraph))
                .collect::<Vec<_>>();
            let second = BlockTarget::from_block(duplicated.version(), paragraphs[1]);
            let delete =
                delete_block(duplicated.text(), duplicated.version(), &blocks, &second).unwrap();
            let mut restored = duplicated.text().to_string();
            restored.replace_range(delete.range, &delete.replacement);
            assert!(restored.starts_with("Review notes."));
            let restored = MarkdownDocument::from_text(&restored);
            assert_eq!(
                restored
                    .preview_blocks()
                    .iter()
                    .filter(|block| matches!(block, PreviewBlock::Paragraph { .. }))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn reorder_separators_keep_terminal_paragraphs_distinct() {
        for ending in ["\n", "\r\n"] {
            for (tail, added) in [
                (String::new(), ending.repeat(2)),
                (ending.to_string(), ending.to_string()),
                (ending.repeat(2), String::new()),
                (ending.repeat(3), String::new()),
                (format!("{ending} \t{ending}"), String::new()),
                (format!("{ending}  "), ending.to_string()),
            ] {
                let source = format!("前段😀{ending}{ending}末段中文{tail}");
                let doc = MarkdownDocument::from_text(&source);
                let blocks = doc.visual_blocks_shared();
                let paragraphs = blocks
                    .iter()
                    .filter(|b| matches!(b.kind, VisualBlockKind::Paragraph))
                    .collect::<Vec<_>>();
                assert_eq!(paragraphs.len(), 2);
                let first = BlockTarget::from_block(doc.version(), paragraphs[0]);
                let last = BlockTarget::from_block(doc.version(), paragraphs[1]);
                let expected = format!("末段中文{tail}{added}前段😀{ending}{ending}");
                for (moving, target, placement, caret) in [
                    (
                        &first,
                        &last,
                        BlockPlacement::After,
                        format!("末段中文{tail}{added}").len(),
                    ),
                    (&last, &first, BlockPlacement::Before, 0),
                ] {
                    let edit =
                        reorder_block(&source, doc.version(), &blocks, moving, target, placement)
                            .unwrap();
                    let mut result = source.clone();
                    result.replace_range(edit.range.clone(), &edit.replacement);
                    assert_eq!(result, expected, "{ending:?}, {tail:?}, {placement:?}");
                    assert_eq!(edit.selection_after, caret..caret);
                    assert_eq!(
                        MarkdownDocument::from_text(&result)
                            .preview_blocks()
                            .iter()
                            .filter(|b| matches!(b, crate::PreviewBlock::Paragraph { .. }))
                            .count(),
                        2
                    );
                }
            }
        }
    }

    #[test]
    fn reorder_separators_keep_tight_list_subtrees_and_quote_groups() {
        for ending in ["\n", "\r\n"] {
            for source in [
                "- parent\n- sibling\n",
                "- parent\n- sibling",
                "- parent\n  - child\n  ```rust\n  code\n  ```\n- sibling\n",
                "- parent\n  > quoted\n- sibling\n",
                "1. parent\n2. sibling\n",
                "1) parent\n2) sibling\n",
                "- parent\n  - first\n  - last\n",
            ] {
                let source = source.replace('\n', ending);
                let doc = MarkdownDocument::from_text(&source);
                let blocks = doc.visual_blocks_shared();
                let items = blocks
                    .iter()
                    .filter(|b| matches!(b.kind, VisualBlockKind::ListItem { .. }))
                    .collect::<Vec<_>>();
                let (first, last) = if source.contains("first") {
                    (items[1], items[2])
                } else {
                    (items[0], *items.last().unwrap())
                };
                let first_target = BlockTarget::from_block(doc.version(), first);
                let last_target = BlockTarget::from_block(doc.version(), last);
                let first_index = blocks.iter().position(|b| b.id == first.id).unwrap();
                let last_index = blocks.iter().position(|b| b.id == last.id).unwrap();
                let a = source_block_unit(&source, &blocks, first_index)
                    .unwrap()
                    .source_range;
                let b = source_block_unit(&source, &blocks, last_index)
                    .unwrap()
                    .source_range;
                let gap = if source[b.clone()].ends_with('\n') {
                    ""
                } else {
                    ending
                };
                let expected = format!(
                    "{}{}{gap}{}{}",
                    &source[..a.start],
                    &source[b.clone()],
                    &source[a.clone()],
                    &source[b.end..]
                );
                for (moving, target, placement) in [
                    (&first_target, &last_target, BlockPlacement::After),
                    (&last_target, &first_target, BlockPlacement::Before),
                ] {
                    let edit =
                        reorder_block(&source, doc.version(), &blocks, moving, target, placement)
                            .unwrap();
                    let mut result = source.clone();
                    result.replace_range(edit.range, &edit.replacement);
                    assert_eq!(result, expected, "tight subtree {source:?}");
                    assert_eq!(
                        MarkdownDocument::from_text(&result)
                            .render_html_fragment()
                            .matches("<p>")
                            .count(),
                        doc.render_html_fragment().matches("<p>").count(),
                        "reordering must not make a tight list loose: {source:?}"
                    );
                }
            }
            let source = format!("> first{ending}{ending}> last{ending}");
            let doc = MarkdownDocument::from_text(&source);
            let blocks = doc.visual_blocks_shared();
            let content = blocks
                .iter()
                .filter(|b| b.quote_context.is_some())
                .collect::<Vec<_>>();
            let edit = reorder_block(
                &source,
                doc.version(),
                &blocks,
                &BlockTarget::from_block(doc.version(), content[0]),
                &BlockTarget::from_block(doc.version(), *content.last().unwrap()),
                BlockPlacement::After,
            )
            .unwrap();
            let mut result = source.clone();
            result.replace_range(edit.range, &edit.replacement);
            assert_eq!(
                result,
                format!("> last{ending}{ending}> first{ending}{ending}")
            );
            assert_eq!(
                MarkdownDocument::from_text(&result)
                    .render_html_fragment()
                    .matches("<blockquote>")
                    .count(),
                2
            );
        }
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
        assert!(block_can_reorder_at(&blocks, parent_index));

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
        assert!(block_can_transform(first));
    }

    #[test]
    fn insert_content_below_isolates_inline_markdown_with_matching_line_endings() {
        for (ending, terminal) in [("\n", false), ("\r\n", false), ("\n", true), ("\r\n", true)] {
            let table = "| A | B |\n| --- | --- |\n| 1 | 2 |".replace('\n', ending);
            let source = if terminal {
                table.clone()
            } else {
                format!("{table}{ending}{ending}After")
            };
            let doc = MarkdownDocument::from_text(&source);
            let blocks = doc.visual_blocks_shared();
            let table_block = blocks
                .iter()
                .find(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
                .unwrap();
            let target = BlockTarget::from_block(doc.version(), table_block);
            let edit = insert_content_below(
                doc.text(),
                doc.version(),
                &blocks,
                &target,
                "[Docs](https://example.com)",
            )
            .unwrap();
            let mut next = source.clone();
            next.replace_range(edit.range, &edit.replacement);
            let expected = if terminal {
                format!("{table}{ending}{ending}[Docs](https://example.com){ending}")
            } else {
                format!("{table}{ending}{ending}[Docs](https://example.com){ending}{ending}After")
            };
            assert_eq!(next, expected);
            assert!(edit.selection_after.is_empty());
            assert!(next[..edit.selection_after.start].ends_with("[Docs](https://example.com)"));
        }
    }
}
