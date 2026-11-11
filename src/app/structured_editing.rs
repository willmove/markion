use super::*;
use markion::selection::{BlockSelection, CellSelection, CellSelectionAxis, SelectionState};

impl MarkionApp {
    pub(super) fn on_visual_block_drag_move(
        &mut self,
        event: &DragMoveEvent<DraggedVisualBlock>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab().is_image() {
            return;
        }
        let target = self.active_document_target();
        let bounds = self.active_tab().visual_list.viewport_bounds();
        let y = f32::from(event.event.position.y);
        let top = f32::from(bounds.top());
        let bottom = f32::from(bounds.bottom());
        let x = event.event.position.x;
        let velocity = if !matches!(self.view_mode, ViewMode::VisualEdit)
            || event.drag(cx).target.document_version != target.1
            || x < bounds.left()
            || x > bounds.right()
            || bottom <= top
        {
            0.
        } else if y < top + 40. {
            -((top + 40. - y) / 40.).clamp(0., 1.) * 20.
        } else if y > bottom - 40. {
            ((y - bottom + 40.) / 40.).clamp(0., 1.) * 20.
        } else {
            0.
        };
        if velocity == 0. {
            self.visual_drag_scroll = None;
            return;
        }
        if let Some(state) = self.visual_drag_scroll.as_mut() {
            state.2 = velocity;
            return;
        }
        self.visual_drag_epoch = self.visual_drag_epoch.wrapping_add(1);
        let epoch = self.visual_drag_epoch;
        self.visual_drag_scroll = Some((target.0, target.1, velocity, epoch));
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let keep = this
                    .update(cx, |app, cx| {
                        let Some((document, version, velocity, current)) = app.visual_drag_scroll
                        else {
                            return false;
                        };
                        if current != epoch {
                            return false;
                        }
                        if !cx.has_active_drag()
                            || app.active_tab().is_image()
                            || app.active_document_target() != (document, version)
                        {
                            app.visual_drag_scroll = None;
                            return false;
                        }
                        app.active_tab().visual_list.scroll_by(px(velocity));
                        cx.notify();
                        true
                    })
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
        })
        .detach();
    }

    pub(super) fn move_blocks_up(
        &mut self,
        _: &MoveBlocksUp,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_selected_source_blocks(false, cx);
    }

    pub(super) fn move_blocks_down(
        &mut self,
        _: &MoveBlocksDown,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_selected_source_blocks(true, cx);
    }

    fn move_selected_source_blocks(&mut self, forward: bool, cx: &mut Context<Self>) {
        if !matches!(self.view_mode, ViewMode::VisualEdit)
            || self.has_text_input_focus()
            || self.active_tab().is_image()
        {
            return;
        }
        if self.active_tab().structured_selection.is_none() {
            self.escape_source_selection(cx);
        }
        let Some(SelectionState::Blocks(selection)) =
            self.active_tab().structured_selection.clone()
        else {
            return;
        };
        let blocks = self.active_tab().document.visual_blocks_shared();
        let Ok(indices) = selection.indices(&self.active_tab().document, &blocks) else {
            return;
        };
        let next = if forward {
            (indices.end..blocks.len()).find(|&i| block_can_reorder_at(&blocks, i))
        } else {
            (0..indices.start)
                .rev()
                .find(|&i| block_can_reorder_at(&blocks, i))
        };
        let Some(next) = next else {
            return;
        };
        let target = BlockTarget::from_block(self.active_tab().document.version(), &blocks[next]);
        if let Ok(edit) = selection.move_to(&self.active_tab().document, &target, forward) {
            let len = selection
                .range(&self.active_tab().document)
                .map_or(0, |range| range.len());
            let start = edit.selection_after.start;
            let version = self.active_tab().document.version();
            self.apply_exact_block_edit(edit, p1_t(self.language, P1Msg::BlockMoved).into(), cx);
            if self.active_tab().document.version() == version {
                return;
            }
            let blocks = self.active_tab().document.visual_blocks_shared();
            let first = blocks.iter().find(|block| {
                block.source_range.start >= start
                    && !matches!(block.kind, VisualBlockKind::Whitespace)
            });
            let last = blocks.iter().rev().find(|block| {
                block.source_range.start < start + len
                    && !matches!(block.kind, VisualBlockKind::Whitespace)
            });
            if let (Some(first), Some(last)) = (first, last) {
                let selection = BlockSelection::new(&self.active_tab().document, first.id, last.id);
                if let Ok(range) = selection.range(&self.active_tab().document) {
                    self.active_tab_mut().selected_range = range;
                    self.active_tab_mut().structured_selection =
                        Some(SelectionState::Blocks(selection));
                }
            }
        }
    }
    pub(super) fn indent_visual_subtree(&mut self, outdent: bool, cx: &mut Context<Self>) -> bool {
        if !matches!(self.view_mode, ViewMode::VisualEdit) || self.active_tab().is_image() {
            return false;
        }
        let blocks = self.active_tab().document.visual_blocks_shared();
        let Some(index) = visual_block_index_for_offset(
            &blocks,
            self.cursor_offset(),
            self.active_tab().document.text().len(),
        ) else {
            return false;
        };
        let VisualBlockKind::ListItem { level, .. } = blocks[index].kind else {
            return false;
        };
        let Ok(unit) = markion::block_edit::source_block_unit(
            self.active_tab().document.text(),
            &blocks,
            index,
        ) else {
            return false;
        };
        let nested = blocks[unit.blocks.clone()].iter().skip(1).any(|block| {
            matches!(block.kind, VisualBlockKind::ListItem { level: child, .. } if child > level)
                || block.list_depth >= level
        });
        if !nested || !self.active_tab().selected_range.is_empty() {
            return false;
        }
        let result = markion::block_edit::indent_subtree(
            self.active_tab().document.text(),
            self.active_tab().document.version(),
            &blocks,
            &BlockTarget::from_block(self.active_tab().document.version(), &blocks[index]),
            self.cursor_offset(),
            outdent,
        );
        if let Ok(edit) = result {
            self.apply_exact_block_edit(edit, t(self.language, Msg::StatusEditing).into(), cx);
        }
        true
    }
    pub(super) fn reveal_visual_code_at(&mut self, offset: usize) {
        if self.active_tab().is_image() {
            return;
        }
        let tab = self.active_tab_mut();
        if let Some(dormant) = tab.dormant_visual_state.as_mut() {
            dormant.folded_code.retain(|range| !range.contains(&offset));
        }
        if let Some(index) = tab
            .visual_list_blocks
            .iter()
            .position(|block| block.source_range.contains(&offset))
        {
            let id = tab.visual_list_blocks[index].id;
            if tab.folded_visual_code_blocks.remove(&id) {
                tab.visual_list.splice(index..index + 1, 1);
            }
        }
    }

    pub(super) fn select_table_axis(
        &mut self,
        target: VisualTableAxisTarget,
        extend: bool,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab().is_image() {
            return;
        }
        let blocks = self.active_tab().document.visual_blocks_shared();
        if revalidate_visual_table_axis_target(
            target,
            self.active_tab().document.version(),
            &blocks,
        )
        .is_none()
        {
            return;
        }
        let Some(index) = blocks.iter().position(|block| block.id == target.block_id) else {
            return;
        };
        let (cell, axis) = match target.axis {
            VisualTableAxis::Row(row) => ((row, 0), CellSelectionAxis::Rows),
            VisualTableAxis::Column(column) => ((0, column), CellSelectionAxis::Columns),
        };
        self.select_table_cells(index, cell, axis, extend, cx);
    }
    pub(super) fn select_source_blocks(
        &mut self,
        index: usize,
        extend: bool,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab().is_image() {
            return;
        }
        let blocks = self.active_tab().document.visual_blocks_shared();
        let Some(block) = blocks.get(index) else {
            return;
        };
        let anchor = if extend {
            match self.active_tab().structured_selection.as_ref() {
                Some(SelectionState::Blocks(selection)) => selection.anchor,
                _ => block.id,
            }
        } else {
            block.id
        };
        let selection = BlockSelection::new(&self.active_tab().document, anchor, block.id);
        let Ok(range) = selection.range(&self.active_tab().document) else {
            return;
        };
        let tab = self.active_tab_mut();
        tab.finish_undo_capture();
        tab.selected_range = range;
        tab.selection_reversed = false;
        tab.structured_selection = Some(SelectionState::Blocks(selection));
        tab.cell_drag_anchor = None;
        tab.marked_range = None;
        tab.clear_preview_selection();
        self.selection_toolbar_suppressed = true;
        cx.notify();
    }

    pub(super) fn escape_source_selection(&mut self, cx: &mut Context<Self>) -> bool {
        if !matches!(self.view_mode, ViewMode::VisualEdit)
            || self.has_text_input_focus()
            || self.file_tree_query_focused
            || self.active_tab().is_image()
        {
            return false;
        }
        if self.active_tab_mut().structured_selection.take().is_some() {
            let offset = self.active_tab().selected_range.start;
            self.move_to(offset, cx);
            return true;
        }
        let blocks = self.active_tab().document.visual_blocks_shared();
        let Some(index) = visual_block_index_for_offset(
            &blocks,
            self.cursor_offset(),
            self.active_tab().document.text().len(),
        ) else {
            return false;
        };
        self.select_source_blocks(index, false, cx);
        self.active_tab().structured_selection.is_some()
    }

    pub(super) fn navigate_source_selection(
        &mut self,
        forward: bool,
        extend: bool,
        horizontal: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.active_tab().is_image() || !matches!(self.view_mode, ViewMode::VisualEdit) {
            return false;
        }
        let Some(selection) = self.active_tab().structured_selection.clone() else {
            return false;
        };
        match selection {
            SelectionState::Blocks(selection) => {
                let blocks = self.active_tab().document.visual_blocks_shared();
                if selection
                    .indices(&self.active_tab().document, &blocks)
                    .is_err()
                {
                    self.active_tab_mut().structured_selection = None;
                    return true;
                }
                let Some(head) = blocks.iter().position(|block| block.id == selection.head) else {
                    self.active_tab_mut().structured_selection = None;
                    return true;
                };
                let Ok(unit) = markion::block_edit::source_block_unit(
                    self.active_tab().document.text(),
                    &blocks,
                    head,
                ) else {
                    return true;
                };
                let next = if forward {
                    (unit.blocks.end..blocks.len())
                        .find(|&index| !matches!(blocks[index].kind, VisualBlockKind::Whitespace))
                } else {
                    (0..unit.blocks.start)
                        .rev()
                        .find(|&index| !matches!(blocks[index].kind, VisualBlockKind::Whitespace))
                };
                if let Some(next) = next {
                    self.select_source_blocks(next, extend, cx);
                }
            }
            SelectionState::Cells(mut selection) => {
                if selection.target.document_version != self.active_tab().document.version()
                    || selection.document != self.active_tab().document.instance_id()
                {
                    self.active_tab_mut().structured_selection = None;
                    return true;
                }
                let blocks = self.active_tab().document.visual_blocks_shared();
                let Some(block) = blocks
                    .iter()
                    .find(|block| block.id == selection.target.block_id)
                else {
                    return true;
                };
                let VisualBlockKind::Table { rows, .. } = &block.kind else {
                    return true;
                };
                let max = if horizontal {
                    rows.iter().map(Vec::len).max().unwrap_or(1)
                } else {
                    rows.len()
                };
                let value = if horizontal {
                    &mut selection.head.1
                } else {
                    &mut selection.head.0
                };
                *value = if forward {
                    (*value + 1).min(max.saturating_sub(1))
                } else {
                    value.saturating_sub(1)
                };
                if !extend {
                    selection.anchor = selection.head;
                }
                let offset = match block.editor.as_ref() {
                    Some(VisualBlockEditor::Table { cells }) => cells
                        .iter()
                        .find(|cell| (cell.row, cell.column) == selection.head)
                        .map(|cell| cell.field.source_range.start),
                    _ => None,
                };
                if let Some(offset) = offset {
                    self.active_tab_mut().selected_range = offset..offset;
                }
                self.active_tab_mut().structured_selection = Some(SelectionState::Cells(selection));
                cx.notify();
            }
            SelectionState::Text { .. } => return false,
        }
        true
    }

    pub(super) fn copy_source_selection(&mut self, cx: &mut Context<Self>) -> bool {
        if self.active_tab().is_image() || !matches!(self.view_mode, ViewMode::VisualEdit) {
            return false;
        }
        let Some(selection) = self.active_tab().structured_selection.clone() else {
            return false;
        };
        let result = match selection {
            SelectionState::Blocks(selection) => selection.clipboard(&self.active_tab().document),
            SelectionState::Cells(selection) => selection.clipboard(&self.active_tab().document),
            SelectionState::Text { .. } => return false,
        };
        if let Ok(copy) = result {
            cx.write_to_clipboard(ClipboardItem::new_html(copy.plain, copy.html));
            self.status = t(self.language, Msg::StatusCopiedSelection).into();
        } else {
            self.active_tab_mut().structured_selection = None;
        }
        cx.notify();
        true
    }

    pub(super) fn delete_source_selection(&mut self, cx: &mut Context<Self>) -> bool {
        if self.active_tab().is_image() || !matches!(self.view_mode, ViewMode::VisualEdit) {
            return false;
        }
        let Some(selection) = self.active_tab().structured_selection.clone() else {
            return false;
        };
        let result = match selection {
            SelectionState::Blocks(selection) => selection.delete(&self.active_tab().document),
            SelectionState::Cells(selection) => selection.clear(&self.active_tab().document),
            SelectionState::Text { .. } => return false,
        };
        match result {
            Ok(edit) => {
                self.apply_exact_block_edit(edit, t(self.language, Msg::StatusEditing).into(), cx)
            }
            Err(_) => {
                self.active_tab_mut().structured_selection = None;
                cx.notify();
            }
        }
        true
    }

    pub(super) fn select_table_cells(
        &mut self,
        block_index: usize,
        cell: (usize, usize),
        axis: CellSelectionAxis,
        extend: bool,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab().is_image() {
            return;
        }
        let blocks = self.active_tab().document.visual_blocks_shared();
        let Some(block) = blocks.get(block_index) else {
            return;
        };
        let mut selection = match self.active_tab().structured_selection.as_ref() {
            Some(SelectionState::Cells(selection))
                if extend
                    && selection.target.block_id == block.id
                    && selection.target.document_version
                        == self.active_tab().document.version() =>
            {
                selection.clone()
            }
            _ => self
                .active_tab()
                .cell_drag_anchor
                .as_ref()
                .filter(|selection| extend && selection.target.block_id == block.id)
                .cloned()
                .unwrap_or_else(|| CellSelection::new(&self.active_tab().document, block, cell)),
        };
        selection.head = cell;
        selection.axis = axis;
        let offset = match block.editor.as_ref() {
            Some(VisualBlockEditor::Table { cells }) => cells
                .iter()
                .find(|field| (field.row, field.column) == cell)
                .map(|field| field.field.source_range.start),
            _ => None,
        };
        let tab = self.active_tab_mut();
        if let Some(offset) = offset {
            tab.selected_range = offset..offset;
        }
        tab.finish_undo_capture();
        tab.structured_selection = Some(SelectionState::Cells(selection));
        tab.is_selecting = false;
        tab.marked_range = None;
        self.selection_toolbar_suppressed = true;
        cx.notify();
    }

    pub(super) fn structured_pointer_down(
        &mut self,
        block_index: usize,
        source: usize,
        event: &MouseDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.active_tab().is_image() {
            return false;
        }
        let blocks = self.active_tab().document.visual_blocks_shared();
        let Some(block) = blocks.get(block_index) else {
            return false;
        };
        if let Some(VisualBlockEditor::Table { cells }) = block.editor.as_ref()
            && let Some(cell) = cells.iter().find(|cell| {
                source >= cell.field.source_range.start && source <= cell.field.source_range.end
            })
        {
            let selection =
                CellSelection::new(&self.active_tab().document, block, (cell.row, cell.column));
            if event.modifiers.shift || event.modifiers.alt {
                if event.modifiers.shift
                    && !matches!(
                        self.active_tab().structured_selection,
                        Some(SelectionState::Cells(_))
                    )
                {
                    let cursor = self.cursor_offset();
                    if let Some(anchor) = cells.iter().find(|cell| {
                        cursor >= cell.field.source_range.start
                            && cursor <= cell.field.source_range.end
                    }) {
                        let anchor = CellSelection::new(
                            &self.active_tab().document,
                            block,
                            (anchor.row, anchor.column),
                        );
                        self.active_tab_mut().cell_drag_anchor = Some(anchor);
                    }
                }
                self.select_table_cells(
                    block_index,
                    (cell.row, cell.column),
                    CellSelectionAxis::Rectangle,
                    event.modifiers.shift,
                    cx,
                );
                self.active_tab_mut().cell_drag_anchor = Some(selection);
                return true;
            }
            self.active_tab_mut().cell_drag_anchor = Some(selection);
        } else {
            self.active_tab_mut().cell_drag_anchor = None;
        }
        if event.modifiers.shift
            && matches!(
                self.active_tab().structured_selection,
                Some(SelectionState::Blocks(_))
            )
        {
            self.select_source_blocks(block_index, true, cx);
            return true;
        }
        self.active_tab_mut().structured_selection = None;
        false
    }

    pub(super) fn paste_table_matrix(&mut self, text: &str, cx: &mut Context<Self>) -> bool {
        if self.active_tab().is_image() || !matches!(self.view_mode, ViewMode::VisualEdit) {
            return false;
        }
        let selection = match self.active_tab().structured_selection.as_ref() {
            Some(SelectionState::Cells(selection)) => Some(selection.clone()),
            _ if text.contains('\t') => {
                let blocks = self.active_tab().document.visual_blocks_shared();
                blocks.iter().find_map(|block| {
                    let VisualBlockEditor::Table { cells } = block.editor.as_ref()? else {
                        return None;
                    };
                    let cell = cells.iter().find(|cell| {
                        self.cursor_offset() >= cell.field.source_range.start
                            && self.cursor_offset() <= cell.field.source_range.end
                    })?;
                    Some(CellSelection::new(
                        &self.active_tab().document,
                        block,
                        (cell.row, cell.column),
                    ))
                })
            }
            _ => None,
        };
        let Some(selection) = selection else {
            return false;
        };
        if let Ok(edit) = selection.paste(&self.active_tab().document, text) {
            self.apply_exact_block_edit(edit, t(self.language, Msg::StatusEditing).into(), cx);
        }
        true
    }

    pub(super) fn append_final_table_row(&mut self, cx: &mut Context<Self>) -> bool {
        if !matches!(self.view_mode, ViewMode::VisualEdit) || self.active_tab().is_image() {
            return false;
        }
        let blocks = self.active_tab().document.visual_blocks_shared();
        let selected = self.active_tab().safe_selected_range();
        let Some((block, rows)) = blocks.iter().find_map(|block| {
            let VisualBlockEditor::Table { cells } = block.editor.as_ref()? else {
                return None;
            };
            let last = cells.last()?;
            (selected.start >= last.field.source_range.start
                && selected.end <= last.field.source_range.end)
                .then_some((block, last.row + 1))
        }) else {
            return false;
        };
        let Some(target) = VisualTableAxisTarget::for_block(
            self.active_tab().document.version(),
            block,
            VisualTableAxis::Row(rows - 1),
        ) else {
            return false;
        };
        self.apply_visual_table_structure_edit(
            target,
            TableStructureEdit::InsertRow { at: rows },
            cx,
        );
        true
    }

    pub(super) fn toggle_visual_code_fold(&mut self, target: BlockTarget, cx: &mut Context<Self>) {
        if self.active_tab().is_image() {
            return;
        }
        let blocks = self.active_tab().document.visual_blocks_shared();
        if validate_block_target(self.active_tab().document.version(), &blocks, &target).is_err() {
            return;
        }
        let tab = self.active_tab_mut();
        if !tab.folded_visual_code_blocks.remove(&target.block_id) {
            tab.folded_visual_code_blocks.insert(target.block_id);
        }
        // Re-measure just this row; folding is presentation-only.
        if let Some(index) = blocks.iter().position(|block| block.id == target.block_id) {
            tab.visual_list.splice(index..index + 1, 1);
        }
        cx.notify();
    }
}
