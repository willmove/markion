//! Ephemeral, source-backed selection and atomic structured commands.
//! No rendered tree is writable and no GPUI types enter this module.

use std::ops::Range;

use crate::block_edit::{
    BlockEdit, BlockEditError, BlockTarget, move_source_units, source_block_unit,
    validate_block_target,
};
use crate::{
    CheckedMutation, DocumentInstanceId, MarkdownDocument, MutationOrigin, MutationReceipt,
    MutationRejection, VisualBlock, VisualBlockEditor, VisualBlockId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionState {
    Text { range: Range<usize>, reversed: bool },
    Blocks(BlockSelection),
    Cells(CellSelection),
}

/// Cache-free selection information retained by semantic undo history.
#[derive(Clone, Debug)]
pub enum SelectionBookmark {
    Blocks {
        anchor: usize,
        head: usize,
    },
    Cells {
        table: usize,
        anchor: (usize, usize),
        head: (usize, usize),
        axis: CellSelectionAxis,
    },
}

impl SelectionState {
    pub fn bookmark(&self, document: &MarkdownDocument) -> Option<SelectionBookmark> {
        let blocks = document.visual_blocks_shared();
        match self {
            Self::Blocks(selection) => {
                selection.indices(document, &blocks).ok()?;
                Some(SelectionBookmark::Blocks {
                    anchor: blocks
                        .iter()
                        .find(|block| block.id == selection.anchor)?
                        .source_range
                        .start,
                    head: blocks
                        .iter()
                        .find(|block| block.id == selection.head)?
                        .source_range
                        .start,
                })
            }
            Self::Cells(selection) => {
                selection.table(document).ok()?;
                Some(SelectionBookmark::Cells {
                    table: selection.target.source_range.start,
                    anchor: selection.anchor,
                    head: selection.head,
                    axis: selection.axis,
                })
            }
            Self::Text { .. } => None,
        }
    }
}

impl SelectionBookmark {
    pub fn restore(&self, document: &MarkdownDocument) -> Option<SelectionState> {
        let blocks = document.visual_blocks_shared();
        match self {
            Self::Blocks { anchor, head } => {
                let anchor = blocks
                    .iter()
                    .find(|block| block.source_range.start == *anchor)?
                    .id;
                let head = blocks
                    .iter()
                    .find(|block| block.source_range.start == *head)?
                    .id;
                let selection = BlockSelection::new(document, anchor, head);
                selection.range(document).ok()?;
                Some(SelectionState::Blocks(selection))
            }
            Self::Cells {
                table,
                anchor,
                head,
                axis,
            } => {
                let block = blocks.iter().find(|block| {
                    block.source_range.start == *table
                        && matches!(block.editor, Some(VisualBlockEditor::Table { .. }))
                })?;
                let mut selection = CellSelection::new(document, block, *anchor);
                selection.head = *head;
                selection.axis = *axis;
                selection.table(document).ok()?;
                Some(SelectionState::Cells(selection))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockSelection {
    pub document: DocumentInstanceId,
    pub version: u64,
    pub anchor: VisualBlockId,
    pub head: VisualBlockId,
}

impl BlockSelection {
    pub fn new(document: &MarkdownDocument, anchor: VisualBlockId, head: VisualBlockId) -> Self {
        Self {
            document: document.instance_id(),
            version: document.version(),
            anchor,
            head,
        }
    }

    pub fn indices(
        &self,
        document: &MarkdownDocument,
        blocks: &[VisualBlock],
    ) -> Result<Range<usize>, BlockEditError> {
        if self.document != document.instance_id() || self.version != document.version() {
            return Err(BlockEditError::Stale);
        }
        let a = blocks
            .iter()
            .position(|block| block.id == self.anchor)
            .ok_or(BlockEditError::Stale)?;
        let b = blocks
            .iter()
            .position(|block| block.id == self.head)
            .ok_or(BlockEditError::Stale)?;
        let a = source_block_unit(document.text(), blocks, a)?;
        let b = source_block_unit(document.text(), blocks, b)?;
        Ok(a.blocks.start.min(b.blocks.start)..a.blocks.end.max(b.blocks.end))
    }

    pub fn range(&self, document: &MarkdownDocument) -> Result<Range<usize>, BlockEditError> {
        let blocks = document.visual_blocks_shared();
        self.indices(document, &blocks)?;
        let a = blocks
            .iter()
            .position(|block| block.id == self.anchor)
            .ok_or(BlockEditError::Stale)?;
        let b = blocks
            .iter()
            .position(|block| block.id == self.head)
            .ok_or(BlockEditError::Stale)?;
        let first = source_block_unit(document.text(), &blocks, a)?;
        let last = source_block_unit(document.text(), &blocks, b)?;
        Ok(first.source_range.start.min(last.source_range.start)
            ..first.source_range.end.max(last.source_range.end))
    }

    pub fn delete(&self, document: &MarkdownDocument) -> Result<BlockEdit, BlockEditError> {
        let range = self.range(document)?;
        Ok(BlockEdit {
            document_version: self.version,
            selection_after: range.start..range.start,
            range,
            replacement: String::new(),
        })
    }

    pub fn move_to(
        &self,
        document: &MarkdownDocument,
        destination: &BlockTarget,
        after: bool,
    ) -> Result<BlockEdit, BlockEditError> {
        let blocks = document.visual_blocks_shared();
        let (index, _) = validate_block_target(document.version(), &blocks, destination)?;
        let destination = source_block_unit(document.text(), &blocks, index)?;
        move_source_units(
            document.text(),
            self.version,
            self.range(document)?,
            if after {
                destination.source_range.end
            } else {
                destination.source_range.start
            },
        )
    }

    pub fn clipboard(
        &self,
        document: &MarkdownDocument,
    ) -> Result<SelectionClipboard, BlockEditError> {
        let range = self.range(document)?;
        let markdown = document.text()[range].to_string();
        let fragment = MarkdownDocument::from_text(&markdown);
        Ok(SelectionClipboard {
            plain: fragment.plain_text_preview(),
            html: fragment.render_html_fragment(),
            markdown,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellSelectionAxis {
    Rectangle,
    Rows,
    Columns,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellSelection {
    pub document: DocumentInstanceId,
    pub target: BlockTarget,
    pub anchor: (usize, usize),
    pub head: (usize, usize),
    pub axis: CellSelectionAxis,
}

impl CellSelection {
    pub fn new(document: &MarkdownDocument, block: &VisualBlock, cell: (usize, usize)) -> Self {
        Self {
            document: document.instance_id(),
            target: BlockTarget::from_block(document.version(), block),
            anchor: cell,
            head: cell,
            axis: CellSelectionAxis::Rectangle,
        }
    }

    fn table(
        &self,
        document: &MarkdownDocument,
    ) -> Result<crate::table::MarkdownTable, BlockEditError> {
        if self.document != document.instance_id() {
            return Err(BlockEditError::Stale);
        }
        let blocks = document.visual_blocks_shared();
        let (_, block) = validate_block_target(document.version(), &blocks, &self.target)?;
        if !matches!(block.editor, Some(VisualBlockEditor::Table { .. })) {
            return Err(BlockEditError::Unsupported);
        }
        let authored = &document.text()[block.source_range.clone()];
        let table = crate::table::parse_markdown_table(
            &authored[crate::table_column_width_prefix_len(authored)..],
        )
        .ok_or(BlockEditError::Ambiguous)?;
        if [self.anchor, self.head]
            .iter()
            .any(|&(row, col)| row >= table.rows.len() || col >= table.column_count())
        {
            return Err(BlockEditError::Stale);
        }
        Ok(table)
    }

    pub fn bounds(&self, rows: usize, columns: usize) -> (Range<usize>, Range<usize>) {
        let mut row = self.anchor.0.min(self.head.0)..self.anchor.0.max(self.head.0) + 1;
        let mut col = self.anchor.1.min(self.head.1)..self.anchor.1.max(self.head.1) + 1;
        match self.axis {
            CellSelectionAxis::Rows => col = 0..columns,
            CellSelectionAxis::Columns => row = 0..rows,
            CellSelectionAxis::Rectangle => {}
        }
        (row, col)
    }

    pub fn clipboard(
        &self,
        document: &MarkdownDocument,
    ) -> Result<SelectionClipboard, BlockEditError> {
        let table = self.table(document)?;
        let (rows, columns) = self.bounds(table.rows.len(), table.column_count());
        let mut plain = Vec::new();
        let mut html = String::from("<table>");
        for row in rows {
            let mut line = Vec::new();
            html.push_str("<tr>");
            for col in columns.clone() {
                let text = table.rows[row].get(col).map(String::as_str).unwrap_or("");
                let fragment = MarkdownDocument::from_text(text);
                line.push(
                    fragment
                        .plain_text_preview()
                        .trim_end_matches('\n')
                        .to_string(),
                );
                html.push_str("<td>");
                html.push_str(&fragment.render_html_fragment());
                html.push_str("</td>");
            }
            plain.push(line.join("\t"));
            html.push_str("</tr>");
        }
        html.push_str("</table>");
        let plain = plain.join("\n");
        Ok(SelectionClipboard {
            markdown: plain.clone(),
            plain,
            html,
        })
    }

    pub fn clear(&self, document: &MarkdownDocument) -> Result<BlockEdit, BlockEditError> {
        let table = self.table(document)?;
        let (rows, columns) = self.bounds(table.rows.len(), table.column_count());
        let updates: Vec<_> = rows
            .flat_map(|row| columns.clone().map(move |col| (row, col, String::new())))
            .collect();
        self.edit_cells(document, &updates)
    }

    pub fn paste(
        &self,
        document: &MarkdownDocument,
        tsv: &str,
    ) -> Result<BlockEdit, BlockEditError> {
        let mut table = self.table(document)?;
        table.normalize();
        let (rows, columns) = self.bounds(table.rows.len(), table.column_count());
        let values: Vec<Vec<&str>> = tsv
            .trim_end_matches(['\r', '\n'])
            .split('\n')
            .map(|line| line.trim_end_matches('\r').split('\t').collect())
            .collect();
        let width = values.iter().map(Vec::len).max().unwrap_or(1);
        let needed_columns = table.column_count().max(columns.start + width);
        let needed_rows = table.rows.len().max(rows.start + values.len());
        let same_shape = needed_rows == table.rows.len() && needed_columns == table.column_count();
        if needed_rows.saturating_mul(needed_columns) > 1_000_000 {
            return Err(BlockEditError::Unsupported);
        }
        table.rows.resize_with(needed_rows, Vec::new);
        for row in &mut table.rows {
            row.resize(needed_columns, String::new());
        }
        table.normalize();
        let mut updates = Vec::new();
        for (r, row) in values.iter().enumerate() {
            for (c, value) in row.iter().enumerate() {
                table.rows[rows.start + r][columns.start + c] =
                    crate::sanitize_visual_field_replacement(
                        document.text(),
                        crate::VisualEditorFieldKind::TableCell {
                            row: rows.start + r,
                            column: columns.start + c,
                        },
                        &(0..0),
                        value,
                    );
                updates.push((
                    rows.start + r,
                    columns.start + c,
                    table.rows[rows.start + r][columns.start + c].clone(),
                ));
            }
        }
        if same_shape && let Ok(edit) = self.edit_cells(document, &updates) {
            return Ok(edit);
        }
        self.edit(document, table, (rows.start, columns.start))
    }

    fn edit_cells(
        &self,
        document: &MarkdownDocument,
        updates: &[(usize, usize, String)],
    ) -> Result<BlockEdit, BlockEditError> {
        let blocks = document.visual_blocks_shared();
        let (_, block) = validate_block_target(document.version(), &blocks, &self.target)?;
        let Some(VisualBlockEditor::Table { cells }) = block.editor.as_ref() else {
            return Err(BlockEditError::Unsupported);
        };
        let mut edits = Vec::new();
        let authored = &document.text()[self.target.source_range.clone()];
        let markdown = &authored[crate::table_column_width_prefix_len(authored)..];
        for (row, column, replacement) in updates {
            let line = markdown
                .lines()
                .nth(if *row == 0 { 0 } else { row + 1 })
                .ok_or(BlockEditError::Ambiguous)?;
            if *column >= crate::table::markdown_table_cell_ranges(line).len() {
                if replacement.is_empty() {
                    continue;
                }
                return Err(BlockEditError::Unsupported);
            }
            let cell = cells
                .iter()
                .find(|cell| cell.row == *row && cell.column == *column)
                .ok_or(BlockEditError::Ambiguous)?;
            edits.push((cell.field.source_range.clone(), replacement));
        }
        edits.sort_by_key(|(range, _)| range.start);
        let Some((first, _)) = edits.first() else {
            let offset = cells
                .first()
                .ok_or(BlockEditError::Ambiguous)?
                .field
                .source_range
                .start;
            return Ok(BlockEdit {
                document_version: self.target.document_version,
                range: offset..offset,
                replacement: String::new(),
                selection_after: offset..offset,
            });
        };
        let range = first.start..edits.last().unwrap().0.end;
        let mut replacement = document.text()[range.clone()].to_string();
        for (field, value) in edits.into_iter().rev() {
            replacement.replace_range(field.start - range.start..field.end - range.start, value);
        }
        Ok(BlockEdit {
            document_version: self.target.document_version,
            selection_after: range.start..range.start,
            range,
            replacement,
        })
    }

    fn edit(
        &self,
        document: &MarkdownDocument,
        table: crate::table::MarkdownTable,
        cell: (usize, usize),
    ) -> Result<BlockEdit, BlockEditError> {
        let blocks = document.visual_blocks_shared();
        let (_, block) = validate_block_target(document.version(), &blocks, &self.target)?;
        if block.quote_context.is_some() || block.list_depth > 0 {
            return Err(BlockEditError::Unsupported);
        }
        let authored = &document.text()[self.target.source_range.clone()];
        let prefix_len = crate::table_column_width_prefix_len(authored);
        let mut replacement = crate::table::format_markdown_table(&table);
        if authored.contains("\r\n") {
            replacement = replacement.replace('\n', "\r\n");
        }
        let cells = crate::table::table_cell_source_ranges(&replacement)
            .ok_or(BlockEditError::Ambiguous)?;
        let field = cells
            .iter()
            .find(|field| (field.row, field.column) == cell)
            .ok_or(BlockEditError::Ambiguous)?;
        let mut prefix = authored[..prefix_len].to_string();
        let old_columns = self.table(document)?.column_count();
        if table.column_count() != old_columns
            && let Some(percents) = crate::authored_table_column_percents(authored, old_columns)
        {
            let mut weights: Vec<f32> = percents.iter().map(|&value| value as f32).collect();
            weights.resize(table.column_count(), 100. / old_columns as f32);
            if let Some(percents) = crate::percents_from_flex_weights(&weights) {
                prefix = format!(
                    "{}{}",
                    crate::table::format_table_column_width_comment(&percents),
                    if authored.contains("\r\n") {
                        "\r\n"
                    } else {
                        "\n"
                    }
                );
            }
        }
        let start = self.target.source_range.start + prefix.len() + field.source_range.start;
        replacement.insert_str(0, &prefix);
        Ok(BlockEdit {
            document_version: self.target.document_version,
            range: self.target.source_range.clone(),
            replacement,
            selection_after: start..start,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionClipboard {
    pub plain: String,
    pub html: String,
    pub markdown: String,
}

#[derive(Clone, Debug)]
pub struct EditTransaction {
    pub mutation: CheckedMutation,
    pub selection_after: Range<usize>,
}

impl EditTransaction {
    pub fn from_edit(document: &MarkdownDocument, edit: BlockEdit) -> Result<Self, BlockEditError> {
        if document.version() != edit.document_version {
            return Err(BlockEditError::Stale);
        }
        let expected = document
            .text()
            .get(edit.range.clone())
            .ok_or(BlockEditError::Ambiguous)?;
        let new_len = document.text().len() - edit.range.len() + edit.replacement.len();
        if edit.selection_after.start > edit.selection_after.end
            || edit.selection_after.end > new_len
        {
            return Err(BlockEditError::Ambiguous);
        }
        // Validate resulting offsets without cloning the whole document.
        for offset in [edit.selection_after.start, edit.selection_after.end] {
            let valid = if offset < edit.range.start {
                document.text().is_char_boundary(offset)
            } else if offset <= edit.range.start + edit.replacement.len() {
                edit.replacement.is_char_boundary(offset - edit.range.start)
            } else {
                document
                    .text()
                    .is_char_boundary(offset - edit.replacement.len() + edit.range.len())
            };
            if !valid {
                return Err(BlockEditError::Ambiguous);
            }
        }
        Ok(Self {
            mutation: CheckedMutation::range(
                document.instance_id(),
                edit.document_version,
                MutationOrigin::ExactBlockEdit,
                edit.range,
                expected,
                edit.replacement,
            ),
            selection_after: edit.selection_after,
        })
    }

    pub fn apply(
        self,
        document: &mut MarkdownDocument,
    ) -> Result<(MutationReceipt, Range<usize>), MutationRejection> {
        document
            .apply_checked_mutation(self.mutation)
            .map(|receipt| (receipt, self.selection_after))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocks_include_descendants_and_transactions_reject_stale_or_wrong_document() {
        let source = "first\r\n\r\n- parent\r\n  - child 中文\r\n\r\nlast";
        let mut doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let parent = blocks
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    crate::VisualBlockKind::ListItem { level: 1, .. }
                )
            })
            .unwrap();
        let selection = BlockSelection::new(&doc, parent.id, parent.id);
        let range = selection.range(&doc).unwrap();
        assert_eq!(&source[range], "- parent\r\n  - child 中文\r\n\r\n");
        let edit = selection.delete(&doc).unwrap();
        let transaction = EditTransaction::from_edit(&doc, edit).unwrap();
        let mut other = MarkdownDocument::from_text(source);
        assert!(transaction.clone().apply(&mut other).is_err());
        transaction.clone().apply(&mut doc).unwrap();
        assert_eq!(doc.text(), "first\r\n\r\nlast");
        assert!(transaction.apply(&mut doc).is_err());
    }

    #[test]
    fn cell_rectangle_copy_clear_and_matrix_paste_preserve_alignment() {
        let source = "before\n\n| A | B |\n| :--- | ---: |\n| **中** | d |\n| e | f |\n\nafter";
        let mut doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks_shared();
        let table = blocks
            .iter()
            .find(|block| matches!(block.kind, crate::VisualBlockKind::Table { .. }))
            .unwrap();
        let mut cells = CellSelection::new(&doc, table, (1, 0));
        cells.head = (2, 1);
        let copy = cells.clipboard(&doc).unwrap();
        assert_eq!(copy.plain, "中\td\ne\tf");
        assert!(copy.html.contains("<strong>中</strong>"));
        let clear = cells.clear(&doc).unwrap();
        assert!(!clear.replacement.contains("**中**"));
        let mut cleared = source.to_string();
        cleared.replace_range(clear.range, &clear.replacement);
        assert!(cleared.contains("| :--- | ---: |"));
        let edit = cells.paste(&doc, "x|y\t😀\r\nq\tr\r\ns\tt").unwrap();
        EditTransaction::from_edit(&doc, edit)
            .unwrap()
            .apply(&mut doc)
            .unwrap();
        assert!(doc.text().starts_with("before\n\n"));
        assert!(doc.text().ends_with("\n\nafter"));
        let table = crate::table::parse_markdown_table(
            &doc.text()[crate::table::table_ranges(doc.text())[0].clone()],
        )
        .unwrap();
        assert_eq!(table.rows.len(), 4);
        assert_eq!(table.rows[1][0], "x\\|y");
        assert_eq!(table.rows[1][1], "😀");
        assert_eq!(table.alignments[1], crate::TableAlignment::Right);
        assert!(cells.clear(&doc).is_err());
    }
}
