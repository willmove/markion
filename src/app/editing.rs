use super::*;

impl MarkionApp {
    pub(super) fn snapshot(&self) -> EditorSnapshot {
        self.active_tab().snapshot()
    }

    pub(super) fn push_undo_snapshot(&mut self) {
        self.active_tab_mut().push_undo_snapshot();
    }

    pub(super) fn commit_undo_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.active_tab_mut().commit_undo_snapshot(snapshot);
    }

    pub(super) fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if self.active_tab_mut().apply_undo() {
            self.active_menu = None;
            self.after_document_changed(cx);
            self.status = t(self.language, Msg::StatusUndo).into();
        } else {
            self.status = t(self.language, Msg::StatusNothingToUndo).into();
        }
        cx.notify();
    }

    pub(super) fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if self.active_tab_mut().apply_redo() {
            self.active_menu = None;
            self.after_document_changed(cx);
            self.status = t(self.language, Msg::StatusRedo).into();
        } else {
            self.status = t(self.language, Msg::StatusNothingToRedo).into();
        }
        cx.notify();
    }

    pub(super) fn apply_markdown_format(
        &mut self,
        format: MarkdownFormat,
        status: SharedString,
        cx: &mut Context<Self>,
    ) {
        self.active_tab_mut().finish_undo_capture();
        let snapshot = self.snapshot();
        let tab = self.active_tab_mut();
        let new_range = tab
            .document
            .apply_markdown_format(tab.selected_range.clone(), format);
        let changed = tab.document.text() != snapshot.document.text();
        if changed {
            self.commit_undo_snapshot(snapshot);
            let tab = self.active_tab_mut();
            tab.selected_range = new_range;
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.active_menu = None;
            self.status = status;
            self.after_document_changed(cx);
        } else {
            self.status = t(self.language, Msg::StatusNoFormattingChange).into();
        }
        cx.notify();
    }

    pub(super) fn apply_table_edit(
        &mut self,
        edit: TableEdit,
        status: SharedString,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit_at(self.cursor_offset(), edit, status, cx);
    }

    pub(super) fn apply_table_edit_at(
        &mut self,
        offset: usize,
        edit: TableEdit,
        status: SharedString,
        cx: &mut Context<Self>,
    ) {
        self.active_tab_mut().finish_undo_capture();
        let snapshot = self.snapshot();
        let tab = self.active_tab_mut();
        let result = tab.document.edit_table_at(offset, edit);
        let changed = tab.document.text() != snapshot.document.text();
        let new_range = result.as_ref().map(|r| r.selected_range.clone());
        if changed {
            self.commit_undo_snapshot(snapshot);
            let tab = self.active_tab_mut();
            if let Some(range) = new_range {
                tab.selected_range = range;
            }
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.active_menu = None;
            self.status = status;
            self.after_document_changed(cx);
        } else if result.is_some() {
            self.active_menu = None;
            self.status = t(self.language, Msg::StatusTableAlreadyFormatted).into();
        } else {
            self.active_menu = None;
            self.status = t(self.language, Msg::StatusNoTableAtCursor).into();
        }
        cx.notify();
    }

    pub(super) fn bold(&mut self, _: &Bold, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(MarkdownFormat::Bold, self.tr(Msg::StatusFmtBold).into(), cx);
    }

    pub(super) fn italic(&mut self, _: &Italic, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::Italic,
            self.tr(Msg::StatusFmtItalic).into(),
            cx,
        );
    }

    pub(super) fn inline_code(&mut self, _: &InlineCode, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::InlineCode,
            self.tr(Msg::StatusFmtInlineCode).into(),
            cx,
        );
    }

    pub(super) fn insert_link(&mut self, _: &InsertLink, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(MarkdownFormat::Link, self.tr(Msg::StatusFmtLink).into(), cx);
    }

    pub(super) fn insert_image(&mut self, _: &InsertImage, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::Image,
            self.tr(Msg::StatusFmtImage).into(),
            cx,
        );
    }

    pub(super) fn apply_heading_level(&mut self, level: u8, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::Heading(level),
            self.trf(Msg::StatusFmtHeading, &[&level.to_string()]),
            cx,
        );
    }

    pub(super) fn heading1(&mut self, _: &Heading1, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(1, cx);
    }

    pub(super) fn heading2(&mut self, _: &Heading2, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(2, cx);
    }

    pub(super) fn heading3(&mut self, _: &Heading3, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(3, cx);
    }

    pub(super) fn heading4(&mut self, _: &Heading4, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(4, cx);
    }

    pub(super) fn heading5(&mut self, _: &Heading5, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(5, cx);
    }

    pub(super) fn heading6(&mut self, _: &Heading6, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(6, cx);
    }

    pub(super) fn set_heading_menu_max_level(&mut self, max_level: u8, cx: &mut Context<Self>) {
        let max_level = normalize_heading_menu_max_level(max_level);
        if self.heading_menu_max_level == max_level {
            return;
        }
        self.heading_menu_max_level = max_level;
        self.persist_preferences();
        install_menus(self.language, self.heading_menu_max_level, cx);
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn unordered_list(
        &mut self,
        _: &UnorderedList,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_markdown_format(
            MarkdownFormat::UnorderedList,
            self.tr(Msg::StatusFmtBulletedList).into(),
            cx,
        );
    }

    pub(super) fn ordered_list(&mut self, _: &OrderedList, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::OrderedList,
            self.tr(Msg::StatusFmtNumberedList).into(),
            cx,
        );
    }

    pub(super) fn task_list(&mut self, _: &TaskList, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::TaskList,
            self.tr(Msg::StatusFmtTaskList).into(),
            cx,
        );
    }

    pub(super) fn block_quote(&mut self, _: &BlockQuote, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::BlockQuote,
            self.tr(Msg::StatusFmtBlockQuote).into(),
            cx,
        );
    }

    pub(super) fn code_fence(&mut self, _: &CodeFence, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::CodeFence,
            self.tr(Msg::StatusFmtCodeBlock).into(),
            cx,
        );
    }

    pub(super) fn format_table(&mut self, _: &FormatTable, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_table_edit(
            TableEdit::Format,
            self.tr(Msg::StatusFmtFormatTable).into(),
            cx,
        );
    }

    pub(super) fn table_add_row(
        &mut self,
        _: &TableAddRow,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit(TableEdit::AddRow, self.tr(Msg::StatusFmtAddRow).into(), cx);
    }

    pub(super) fn table_delete_row(
        &mut self,
        _: &TableDeleteRow,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit(
            TableEdit::DeleteRow,
            self.tr(Msg::StatusFmtDeleteRow).into(),
            cx,
        );
    }

    pub(super) fn table_move_row_up(
        &mut self,
        _: &TableMoveRowUp,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit(
            TableEdit::MoveRowUp,
            self.tr(Msg::StatusFmtMoveRowUp).into(),
            cx,
        );
    }

    pub(super) fn table_move_row_down(
        &mut self,
        _: &TableMoveRowDown,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit(
            TableEdit::MoveRowDown,
            self.tr(Msg::StatusFmtMoveRowDown).into(),
            cx,
        );
    }

    pub(super) fn table_add_column(
        &mut self,
        _: &TableAddColumn,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit(
            TableEdit::AddColumn,
            self.tr(Msg::StatusFmtAddColumn).into(),
            cx,
        );
    }

    pub(super) fn table_delete_column(
        &mut self,
        _: &TableDeleteColumn,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit(
            TableEdit::DeleteColumn,
            self.tr(Msg::StatusFmtDeleteColumn).into(),
            cx,
        );
    }

    pub(super) fn confirm_discard_then(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        message: Msg,
        detail: Msg,
        on_confirm: fn(&mut Self, &mut Context<Self>),
    ) {
        if !self.active_tab().document.is_dirty() {
            on_confirm(self, cx);
            return;
        }

        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(message),
            Some(self.tr(detail)),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonDiscard)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusWaitingConfirm).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                if confirmed {
                    on_confirm(app, cx);
                } else {
                    app.active_menu = None;
                    app.status = t(app.language, Msg::StatusCanceled).into();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn request_quit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.confirming_close {
            return;
        }

        self.active_menu = None;
        if !self.tabs.iter().any(|t| t.document.is_dirty()) {
            self.allow_close = true;
            self.status = t(self.language, Msg::StatusExitingMarkion).into();
            cx.notify();
            window.remove_window();
            cx.quit();
            return;
        }

        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogExitTitle),
            Some(self.tr(Msg::DialogExitDetail)),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonExitWithoutSaving)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );

        self.confirming_close = true;
        self.status = t(self.language, Msg::StatusWaitingExitConfirm).into();
        cx.notify();
        let window_handle = window.window_handle();

        cx.spawn(async move |this, cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                app.confirming_close = false;
                if confirmed {
                    app.discard_current_recovery_file();
                    app.allow_close = true;
                    app.status = t(app.language, Msg::StatusExitingMarkion).into();
                    cx.notify();
                    let _ = window_handle.update(cx, |_, window, _| window.remove_window());
                    cx.quit();
                } else {
                    app.status = t(app.language, Msg::StatusExitCanceled).into();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn toggle_menu(&mut self, menu: AppMenu, cx: &mut Context<Self>) {
        eprintln!(
            "[menu-debug] toggle_menu({menu:?}), was {:?}",
            self.active_menu
        );
        self.file_tree_context_menu = None;
        self.pending_name_input = None;
        self.active_menu = if self.active_menu == Some(menu) {
            None
        } else {
            Some(menu)
        };
        cx.notify();
    }

    pub(super) fn hover_menu(&mut self, menu: AppMenu, cx: &mut Context<Self>) {
        let next_menu = menu_after_hover(self.active_menu, menu);
        if next_menu != self.active_menu {
            self.active_menu = next_menu;
            cx.notify();
        }
    }

    pub(super) fn close_menu(
        &mut self,
        _: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        eprintln!("[menu-debug] close_menu, active={:?}", self.active_menu);
        if self.active_menu.is_some()
            || self.file_tree_context_menu.is_some()
            || self.preview_context_menu.is_some()
            || self.pending_name_input.is_some()
        {
            self.active_menu = None;
            self.file_tree_context_menu = None;
            self.preview_context_menu = None;
            self.pending_name_input = None;
            cx.notify();
        }
    }

    pub(super) fn show_preview_context_menu(
        &mut self,
        position: Point<Pixels>,
        link_url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.active_menu = None;
        self.file_tree_context_menu = None;
        // Pane chrome and selectable runs may both handle the same right-click.
        // Prefer a resolved link over a later `None` from the pane surface.
        if let Some(existing) = &mut self.preview_context_menu {
            existing.position = position;
            if link_url.is_some() {
                existing.link_url = link_url;
            }
        } else {
            self.preview_context_menu = Some(PreviewContextMenu { position, link_url });
        }
        cx.notify();
    }

    pub(super) fn select_all_preview_text(&mut self, cx: &mut Context<Self>) {
        let blocks = self.active_tab().preview_list_blocks.clone();
        let mut first: Option<PreviewCaret> = None;
        let mut last: Option<PreviewCaret> = None;
        for (block_index, block) in blocks.iter().enumerate() {
            for run_id in preview_block_runs(block) {
                let Some(text) = preview_run_plain_text(block, run_id) else {
                    continue;
                };
                if text.is_empty() {
                    continue;
                }
                let start = PreviewCaret {
                    block_index,
                    run_id,
                    offset: 0,
                };
                let end = PreviewCaret {
                    block_index,
                    run_id,
                    offset: text.len(),
                };
                if first.is_none() {
                    first = Some(start);
                }
                last = Some(end);
            }
        }
        if let (Some(anchor), Some(head)) = (first, last) {
            let tab = self.active_tab_mut();
            tab.preview_selection = Some(PreviewSelection { anchor, head });
            tab.preview_is_selecting = false;
            self.status = t(self.language, Msg::StatusPreviewSelectedAll).into();
        }
        cx.notify();
    }

    pub(super) fn handle_preview_context_action(
        &mut self,
        action: PreviewContextAction,
        cx: &mut Context<Self>,
    ) {
        let link_url = self
            .preview_context_menu
            .as_ref()
            .and_then(|menu| menu.link_url.clone());
        self.preview_context_menu = None;
        match action {
            PreviewContextAction::SelectAll => {
                self.select_all_preview_text(cx);
            }
            PreviewContextAction::CopyPlain => {
                let blocks = self.active_tab().preview_list_blocks.clone();
                if let Some(text) = self
                    .active_tab()
                    .preview_selection
                    .as_ref()
                    .and_then(|sel| preview_selection_plain_text(sel, &blocks))
                {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                    self.status = t(self.language, Msg::StatusCopiedPreviewPlain).into();
                } else {
                    self.status = t(self.language, Msg::StatusNothingToCopy).into();
                }
                cx.notify();
            }
            PreviewContextAction::CopyMarkdown => {
                let blocks = self.active_tab().preview_list_blocks.clone();
                let document = self.active_tab().document.text().to_string();
                if let Some(md) = self
                    .active_tab()
                    .preview_selection
                    .as_ref()
                    .and_then(|sel| preview_selection_markdown(sel, &blocks, &document))
                {
                    cx.write_to_clipboard(ClipboardItem::new_string(md));
                    self.status = t(self.language, Msg::StatusCopiedPreviewMarkdown).into();
                } else {
                    self.status = t(self.language, Msg::StatusNothingToCopy).into();
                }
                cx.notify();
            }
            PreviewContextAction::CopyHtml => {
                let blocks = self.active_tab().preview_list_blocks.clone();
                let document = self.active_tab().document.text().to_string();
                if let Some(md) = self
                    .active_tab()
                    .preview_selection
                    .as_ref()
                    .and_then(|sel| preview_selection_markdown(sel, &blocks, &document))
                {
                    let html = MarkdownDocument::from_text(&md).render_html_fragment();
                    cx.write_to_clipboard(ClipboardItem::new_string(html));
                    self.status = t(self.language, Msg::StatusCopiedPreviewHtml).into();
                } else {
                    self.status = t(self.language, Msg::StatusNothingToCopy).into();
                }
                cx.notify();
            }
            PreviewContextAction::CopyLinkAddress => {
                if let Some(url) = link_url {
                    cx.write_to_clipboard(ClipboardItem::new_string(url));
                    self.status = t(self.language, Msg::StatusCopiedLinkAddress).into();
                } else {
                    self.status = t(self.language, Msg::StatusNothingToCopy).into();
                }
                cx.notify();
            }
        }
    }

    pub(super) fn toggle_file_menu(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_menu(AppMenu::File, cx);
    }

    pub(super) fn toggle_edit_menu(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_menu(AppMenu::Edit, cx);
    }

    pub(super) fn toggle_view_menu(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_menu(AppMenu::View, cx);
    }

    pub(super) fn toggle_format_menu(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_menu(AppMenu::Format, cx);
    }

    pub(super) fn toggle_export_menu(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_menu(AppMenu::Export, cx);
    }

    pub(super) fn toggle_help_menu(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_menu(AppMenu::Help, cx);
    }

    pub(super) fn click_find_next(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.find_next(&FindNext, window, cx);
    }

    pub(super) fn click_find_previous(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.find_previous(&FindPrevious, window, cx);
    }

    pub(super) fn click_replace_current(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_current_match(&ReplaceCurrentMatch, window, cx);
    }

    pub(super) fn click_replace_all(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_all_matches(&ReplaceAllMatches, window, cx);
    }

    pub(super) fn click_close_search(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_search_overlay(cx);
    }

    pub(super) fn click_toggle_case(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_find_case_sensitive(&ToggleFindCaseSensitive, window, cx);
    }

    pub(super) fn click_toggle_regex(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_find_regex(&ToggleFindRegex, window, cx);
    }

    pub(super) fn focus_find_field(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_visible = true;
        self.search_focus = Some(SearchField::Find);
        self.file_tree_query_focused = false;
        self.input_marked_len = 0;
        self.status = t(self.language, Msg::StatusEditingFindQuery).into();
        cx.notify();
    }

    pub(super) fn focus_replace_field(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_visible = true;
        self.replace_visible = true;
        self.search_focus = Some(SearchField::Replace);
        self.file_tree_query_focused = false;
        self.input_marked_len = 0;
        self.status = t(self.language, Msg::StatusEditingReplacement).into();
        cx.notify();
    }

    pub(super) fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        let (is_empty, start) = {
            let tab = self.active_tab();
            (tab.selected_range.is_empty(), tab.selected_range.start)
        };
        if is_empty {
            if matches!(self.view_mode, ViewMode::VisualEdit)
                && let Some(target) = self
                    .active_tab()
                    .document
                    .visual_editor_edge_target(start, false)
            {
                self.move_to(target, cx);
                return;
            }
            let boundary = self
                .visual_affinity_horizontal_target(VisualCaretAffinity::Upstream)
                .unwrap_or_else(|| self.previous_boundary(self.cursor_offset()));
            self.move_to(boundary, cx);
        } else {
            self.move_to(start, cx);
        }
    }

    pub(super) fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        let (is_empty, end) = {
            let tab = self.active_tab();
            (tab.selected_range.is_empty(), tab.selected_range.end)
        };
        if is_empty {
            if matches!(self.view_mode, ViewMode::VisualEdit)
                && let Some(target) = self
                    .active_tab()
                    .document
                    .visual_editor_edge_target(end, true)
            {
                self.move_to(target, cx);
                return;
            }
            let boundary = self
                .visual_affinity_horizontal_target(VisualCaretAffinity::Downstream)
                .unwrap_or_else(|| self.next_boundary(end));
            self.move_to(boundary, cx);
        } else {
            self.move_to(end, cx);
        }
    }

    pub(super) fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.view_mode, ViewMode::VisualEdit)
            && let Some(target) = self
                .active_tab()
                .document
                .visual_editor_edge_target(self.cursor_offset(), false)
        {
            self.move_to(target, cx);
            return;
        }
        let boundary = self.previous_boundary(self.cursor_offset());
        self.select_to(boundary, cx);
    }

    pub(super) fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.view_mode, ViewMode::VisualEdit)
            && let Some(target) = self
                .active_tab()
                .document
                .visual_editor_edge_target(self.cursor_offset(), true)
        {
            self.move_to(target, cx);
            return;
        }
        let boundary = self.next_boundary(self.cursor_offset());
        self.select_to(boundary, cx);
    }

    pub(super) fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        if self.move_visual_vertical(VisualNavigationDirection::Up, false, cx) {
            return;
        }
        let (is_empty, boundary_start, cursor) = {
            let tab = self.active_tab();
            (
                tab.selected_range.is_empty(),
                tab.selected_range.start,
                tab.cursor_offset(),
            )
        };
        let offset = if is_empty { cursor } else { boundary_start };
        let target = self.active_tab().document.previous_line_offset(offset);
        self.move_to(target, cx);
    }

    pub(super) fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        if self.move_visual_vertical(VisualNavigationDirection::Down, false, cx) {
            return;
        }
        let (is_empty, boundary_end, cursor) = {
            let tab = self.active_tab();
            (
                tab.selected_range.is_empty(),
                tab.selected_range.end,
                tab.cursor_offset(),
            )
        };
        let offset = if is_empty { cursor } else { boundary_end };
        let target = self.active_tab().document.next_line_offset(offset);
        self.move_to(target, cx);
    }

    pub(super) fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        if self.move_visual_vertical(VisualNavigationDirection::Up, true, cx) {
            return;
        }
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.previous_line_offset(cursor);
        self.select_to(target, cx);
    }

    pub(super) fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        if self.move_visual_vertical(VisualNavigationDirection::Down, true, cx) {
            return;
        }
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.next_line_offset(cursor);
        self.select_to(target, cx);
    }

    pub(super) fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        let len = self.active_tab().document.text().len();
        self.select_to(len, cx);
    }

    pub(super) fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(target) = self.visual_painted_line_boundary(false) {
            self.move_to(target, cx);
            return;
        }
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.line_start_at(cursor);
        self.move_to(target, cx);
    }

    pub(super) fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(target) = self.visual_painted_line_boundary(true) {
            self.move_to(target, cx);
            return;
        }
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.line_end_at(cursor);
        self.move_to(target, cx);
    }

    pub(super) fn insert_newline(
        &mut self,
        _: &InsertNewline,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // When the inline name prompt is open, Enter commits the name instead
        // of inserting a newline into the document.
        if self.pending_name_input.is_some() {
            self.confirm_pending_name(&ConfirmPendingName, _window, cx);
            return;
        }
        let selected = self.active_tab().selected_range.clone();
        if matches!(self.view_mode, ViewMode::VisualEdit)
            && let Some(field) = self.active_tab().document.visual_editor_field_at(&selected)
        {
            match field.kind {
                VisualEditorFieldKind::ImageAlt
                | VisualEditorFieldKind::ImageDestination
                | VisualEditorFieldKind::ImageTitle
                | VisualEditorFieldKind::TableCell { .. } => {
                    if let Some(target) = self
                        .active_tab()
                        .document
                        .visual_editor_tab_target(&selected, true)
                    {
                        self.move_to_visual_editor_target(target, cx);
                    }
                    return;
                }
                VisualEditorFieldKind::CodePayload | VisualEditorFieldKind::MathPayload => {
                    self.active_tab_mut().pending_text_edit_intent = Some(UndoCaptureKind::Atomic);
                    self.replace_text_in_range(None, "\n", _window, cx);
                    self.active_tab_mut().finish_undo_capture();
                    return;
                }
            }
        }
        let cursor = self.active_tab().selected_range.start;
        let structural_edit = (matches!(self.view_mode, ViewMode::VisualEdit)
            && selected.is_empty())
        .then(|| self.active_tab().document.visual_enter_edit(cursor))
        .flatten();
        self.push_undo_snapshot();
        let tab = self.active_tab_mut();
        if let Some(edit) = structural_edit {
            tab.document.replace_range(edit.range, &edit.replacement);
            tab.selected_range = edit.selection_after;
        } else {
            if !selected.is_empty() {
                tab.document.replace_range(selected, "");
            }
            let new_cursor = tab.document.insert_markdown_newline(cursor);
            tab.selected_range = new_cursor..new_cursor;
        }
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.status = t(self.language, Msg::StatusEditing).into();
        self.after_document_changed(cx);
        cx.notify();
    }

    pub(super) fn indent(&mut self, _: &Indent, window: &mut Window, cx: &mut Context<Self>) {
        let selected = self.active_tab().selected_range.clone();
        if matches!(self.view_mode, ViewMode::VisualEdit)
            && let Some(target) = self
                .active_tab()
                .document
                .visual_editor_tab_target(&selected, true)
        {
            self.move_to_visual_editor_target(target, cx);
            return;
        }
        if self.has_text_input_focus() {
            self.push_text_input("    ", cx);
            return;
        }

        if self.active_tab().selected_range.is_empty() {
            self.replace_text_in_range(None, "    ", window, cx);
        } else {
            let snapshot = self.snapshot();
            let selected = self.active_tab().selected_range.clone();
            let tab = self.active_tab_mut();
            tab.selected_range = tab.document.indent_lines(selected);
            let changed = tab.document.text() != snapshot.document.text();
            if changed {
                self.commit_undo_snapshot(snapshot);
            }
            let tab = self.active_tab_mut();
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.status = t(self.language, Msg::StatusIndentedSelection).into();
            if changed {
                self.after_document_changed(cx);
            }
            cx.notify();
        }
    }

    pub(super) fn outdent(&mut self, _: &Outdent, _: &mut Window, cx: &mut Context<Self>) {
        let selected = self.active_tab().selected_range.clone();
        if matches!(self.view_mode, ViewMode::VisualEdit)
            && let Some(target) = self
                .active_tab()
                .document
                .visual_editor_tab_target(&selected, false)
        {
            self.move_to_visual_editor_target(target, cx);
            return;
        }
        let snapshot = self.snapshot();
        let selected = self.active_tab().selected_range.clone();
        let tab = self.active_tab_mut();
        tab.selected_range = tab.document.outdent_lines(selected);
        let changed = tab.document.text() != snapshot.document.text();
        if changed {
            self.commit_undo_snapshot(snapshot);
        }
        let tab = self.active_tab_mut();
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.status = t(
            self.language,
            if changed {
                Msg::StatusOutdentedSelection
            } else {
                Msg::StatusNothingToOutdent
            },
        )
        .into();
        if changed {
            self.after_document_changed(cx);
        }
        cx.notify();
    }

    pub(super) fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.pop_text_input(cx) {
            return;
        }

        if matches!(self.view_mode, ViewMode::VisualEdit)
            && self.active_tab().selected_range.is_empty()
            && let Some(target) = self
                .active_tab()
                .document
                .visual_editor_edge_target(self.cursor_offset(), false)
        {
            self.move_to(target, cx);
            return;
        }

        if matches!(self.view_mode, ViewMode::VisualEdit)
            && self.active_tab().selected_range.is_empty()
            && let Some(edit) = self
                .active_tab()
                .document
                .visual_backspace_edit(self.cursor_offset())
        {
            self.push_undo_snapshot();
            let tab = self.active_tab_mut();
            tab.document.replace_range(edit.range, &edit.replacement);
            tab.selected_range = edit.selection_after;
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.status = t(self.language, Msg::StatusEditing).into();
            self.after_document_changed(cx);
            cx.notify();
            return;
        }

        if self.active_tab().selected_range.is_empty() {
            let boundary = self.previous_boundary(self.cursor_offset());
            self.select_to(boundary, cx);
            self.active_tab_mut().pending_text_edit_intent = Some(UndoCaptureKind::Delete);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(super) fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.pop_text_input(cx) {
            return;
        }

        if matches!(self.view_mode, ViewMode::VisualEdit)
            && self.active_tab().selected_range.is_empty()
            && let Some(target) = self
                .active_tab()
                .document
                .visual_editor_edge_target(self.cursor_offset(), true)
        {
            self.move_to(target, cx);
            return;
        }

        if self.active_tab().selected_range.is_empty() {
            let boundary = self.next_boundary(self.cursor_offset());
            self.select_to(boundary, cx);
            self.active_tab_mut().pending_text_edit_intent = Some(UndoCaptureKind::Delete);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(super) fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            if self.has_text_input_focus() {
                self.push_text_input(&text, cx);
                return;
            }
            self.active_tab_mut().pending_text_edit_intent = Some(UndoCaptureKind::Atomic);
            self.replace_text_in_range(None, &text, window, cx);
            self.active_tab_mut().finish_undo_capture();
        } else {
            self.status = t(self.language, Msg::StatusClipboardEmpty).into();
            cx.notify();
        }
    }

    pub(super) fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        let blocks = self.active_tab().preview_list_blocks.clone();
        if preview_selection_takes_copy_precedence(
            self.active_tab().preview_selection.as_ref(),
            &blocks,
        ) && let Some(text) = self
            .active_tab()
            .preview_selection
            .as_ref()
            .and_then(|sel| preview_selection_plain_text(sel, &blocks))
        {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.status = t(self.language, Msg::StatusCopiedSelection).into();
            cx.notify();
            return;
        }
        let selected = self.active_tab().selected_range.clone();
        if !selected.is_empty() {
            let text = self.active_tab().document.text()[selected].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.status = t(self.language, Msg::StatusCopiedSelection).into();
        } else {
            self.status = t(self.language, Msg::StatusNothingToCopy).into();
        }
        cx.notify();
    }

    pub(super) fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        let selected = self.active_tab().selected_range.clone();
        if !selected.is_empty() {
            let text = self.active_tab().document.text()[selected].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.replace_text_in_range(None, "", window, cx);
            self.status = t(self.language, Msg::StatusCutSelection).into();
            cx.notify();
        } else {
            self.status = t(self.language, Msg::StatusNothingToCut).into();
            cx.notify();
        }
    }

    pub(super) fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Clicking into the editor returns text-input focus to the document,
        // otherwise typed characters keep flowing into the file-tree filter
        // or search fields that last held focus.
        self.file_tree_query_focused = false;
        self.search_focus = None;
        self.input_marked_len = 0;
        // Source-editor selection clears any preview selection so Copy routes
        // back to the editor.
        self.active_tab_mut().clear_preview_selection();
        self.active_tab_mut().is_selecting = true;
        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    pub(super) fn on_mouse_up(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        _: &mut Context<Self>,
    ) {
        self.active_tab_mut().is_selecting = false;
    }

    pub(super) fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab().is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    /// Horizontal tab bar shown only when more than one tab is open. Each tab
    /// shows the file name (+ `*` when dirty), the active tab is highlighted,
    /// clicking switches to it, and the `×` button closes it. Styled to match
    /// the existing `menu_title_button` idiom (GPUI 0.2.2 has no native tab bar).
    pub(super) fn tab_bar_view(&self, cx: &mut Context<Self>) -> Div {
        let palette = self.palette();
        if self.tabs.len() <= 1 {
            // Single-tab case: render nothing (tab bar hidden).
            return div();
        }
        let active = self.active_tab;
        let bar = div()
            .h(px(30.))
            .px_2()
            .border_b_1()
            .border_color(palette.border)
            .bg(palette.panel_bg)
            .flex()
            .items_center()
            .gap_1();
        bar.children(self.tabs.iter().enumerate().map(|(index, tab)| {
            let is_active = index == active;
            let name = title_from_path(tab.document.path()).to_string();
            let dirty = tab.document.is_dirty();
            let label: SharedString = if dirty {
                format!("{name} *").into()
            } else {
                name.into()
            };
            // Theme-driven so tabs stay legible on dark palettes (the previous
            // hard-coded light hexes rendered white tabs with light text).
            let bg = if is_active {
                palette.active_bg
            } else {
                palette.surface_bg
            };
            let text_color = if is_active {
                palette.active_text
            } else {
                palette.muted
            };
            let border = if is_active {
                palette.active_text
            } else {
                palette.border
            };
            let hover_bg = palette.active_bg;
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .border_b_2()
                .border_color(border)
                .bg(bg)
                .text_color(text_color)
                .text_size(px(12.))
                .cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .flex()
                .items_center()
                .gap_1()
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                        // The captured `index` is fixed at render time; a tab
                        // close/open since then may have shifted positions, so
                        // guard against a stale out-of-range index.
                        if index < app.tabs.len() {
                            app.switch_active_tab(index, cx);
                        }
                    }),
                )
                .child(label)
                .child(
                    div()
                        .ml_1()
                        .px_1()
                        .text_size(px(11.))
                        .cursor_pointer()
                        .hover(move |style| style.bg(border))
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |app, _: &MouseUpEvent, window, cx| {
                                // Same staleness guard as the tab click above.
                                if index < app.tabs.len() {
                                    app.switch_active_tab(index, cx);
                                    app.close_tab(&CloseTab, window, cx);
                                }
                            }),
                        )
                        .child("×"),
                )
        }))
        .child(
            // Trailing "+" opens a fresh empty tab (mirrors File → New Tab).
            div()
                .id("new-tab-button")
                .ml_1()
                .px_2()
                .py_1()
                .rounded_md()
                .text_size(px(15.))
                .text_color(palette.muted)
                .cursor_pointer()
                .hover(move |style| style.bg(palette.active_bg))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |app, _: &MouseUpEvent, window, cx| {
                        app.new_tab(&NewTab, window, cx);
                    }),
                )
                .child("+"),
        )
    }

    pub(super) fn cursor_offset(&self) -> usize {
        self.active_tab().cursor_offset()
    }

    fn current_visual_navigation_snapshot(&self) -> Option<(usize, VisualNavigationSnapshot)> {
        if !matches!(self.view_mode, ViewMode::VisualEdit) {
            return None;
        }
        let tab = self.active_tab();
        let cursor = tab.cursor_offset();
        let block_index = visual_block_index_for_offset(
            &tab.visual_list_blocks,
            cursor,
            tab.document.text().len(),
        )?;
        let snapshot = tab.visual_navigation_snapshots.get(&block_index)?;
        if snapshot.document_version != tab.document.version()
            || snapshot.source_island
            || tab.visual_navigation_snapshot_ids.get(&block_index)
                != tab
                    .visual_list_blocks
                    .get(block_index)
                    .map(|block| &block.id)
        {
            return None;
        }
        Some((block_index, snapshot.clone()))
    }

    fn move_visual_vertical(
        &mut self,
        direction: VisualNavigationDirection,
        extend_selection: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some((block_index, snapshot)) = self.current_visual_navigation_snapshot() else {
            return false;
        };
        let cursor = self.cursor_offset();
        let line_index = self
            .active_tab()
            .visual_navigation_position
            .filter(|position| {
                position.document_version == snapshot.document_version
                    && position.block_index == block_index
                    && position.source_offset == cursor
                    && position.line_index < snapshot.lines.len()
            })
            .map(|position| position.line_index)
            .or_else(|| snapshot.line_index_for_source(cursor));
        let Some(line_index) = line_index else {
            return false;
        };
        let preferred_x = self
            .active_tab()
            .visual_preferred_x
            .or_else(|| snapshot.caret_x_for_source(cursor))
            .unwrap_or(Pixels::ZERO);
        let adjacent_line = match direction {
            VisualNavigationDirection::Up => line_index.checked_sub(1),
            VisualNavigationDirection::Down => {
                (line_index + 1 < snapshot.lines.len()).then_some(line_index + 1)
            }
        };
        if let Some(line_index) = adjacent_line
            && let Some(target) = snapshot.lines[line_index].closest_source(preferred_x)
        {
            if extend_selection {
                self.select_to(target, cx);
            } else {
                self.move_to(target, cx);
            }
            let tab = self.active_tab_mut();
            tab.visual_preferred_x = Some(preferred_x);
            tab.visual_navigation_position = Some(VisualNavigationPosition {
                document_version: snapshot.document_version,
                block_index,
                line_index,
                source_offset: target,
            });
            return true;
        }

        let target_block = match direction {
            VisualNavigationDirection::Up => block_index.checked_sub(1),
            VisualNavigationDirection::Down => (block_index + 1
                < self.active_tab().visual_list_blocks.len())
            .then_some(block_index + 1),
        };
        let Some(target_block) = target_block else {
            return true;
        };
        if let Some(block) = self.active_tab().visual_list_blocks.get(target_block)
            && matches!(block.kind, VisualBlockKind::Whitespace)
        {
            let target = match direction {
                VisualNavigationDirection::Up => block.source_range.end,
                VisualNavigationDirection::Down => block.source_range.start,
            };
            if extend_selection {
                self.select_to(target, cx);
            } else {
                self.move_to(target, cx);
            }
            self.active_tab_mut().visual_preferred_x = Some(preferred_x);
            return true;
        }
        let version = self.active_tab().document.version();
        let pending = PendingVisualNavigation {
            document_version: version,
            target_block,
            direction,
            extend_selection,
            preferred_x,
        };
        let tab = self.active_tab_mut();
        tab.visual_preferred_x = Some(preferred_x);
        tab.pending_visual_navigation = Some(pending);
        tab.visual_list.scroll_to_reveal_item(target_block);
        cx.notify();
        true
    }

    fn visual_painted_line_boundary(&self, end: bool) -> Option<usize> {
        let (_, snapshot) = self.current_visual_navigation_snapshot()?;
        let line = snapshot
            .lines
            .get(snapshot.line_index_for_source(self.cursor_offset())?)?;
        if end {
            line.carets
                .iter()
                .max_by(|left, right| {
                    left.x
                        .to_f64()
                        .total_cmp(&right.x.to_f64())
                        .then_with(|| left.source_offset.cmp(&right.source_offset))
                })
                .map(|caret| caret.source_offset)
        } else {
            line.carets
                .iter()
                .min_by(|left, right| {
                    left.x
                        .to_f64()
                        .total_cmp(&right.x.to_f64())
                        .then_with(|| left.source_offset.cmp(&right.source_offset))
                })
                .map(|caret| caret.source_offset)
        }
    }

    pub(super) fn complete_pending_visual_navigation(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = self.active_tab().pending_visual_navigation else {
            return;
        };
        if pending.document_version != self.active_tab().document.version() {
            self.active_tab_mut().clear_visual_navigation_intent();
            return;
        }
        let Some(snapshot) = self
            .active_tab()
            .visual_navigation_snapshots
            .get(&pending.target_block)
            .filter(|snapshot| snapshot.document_version == pending.document_version)
            .filter(|_| {
                self.active_tab()
                    .visual_list_blocks
                    .get(pending.target_block)
                    .is_some_and(|block| {
                        self.active_tab()
                            .visual_navigation_snapshot_ids
                            .get(&pending.target_block)
                            == Some(&block.id)
                    })
            })
            .cloned()
        else {
            return;
        };
        let line = match pending.direction {
            VisualNavigationDirection::Up => snapshot.lines.last(),
            VisualNavigationDirection::Down => snapshot.lines.first(),
        };
        let Some(target) = line.and_then(|line| line.closest_source(pending.preferred_x)) else {
            return;
        };
        self.active_tab_mut().pending_visual_navigation = None;
        if pending.extend_selection {
            self.select_to(target, cx);
        } else {
            self.move_to(target, cx);
        }
        let target_line = match pending.direction {
            VisualNavigationDirection::Up => snapshot.lines.len().saturating_sub(1),
            VisualNavigationDirection::Down => 0,
        };
        let tab = self.active_tab_mut();
        tab.visual_preferred_x = Some(pending.preferred_x);
        tab.visual_navigation_position = Some(VisualNavigationPosition {
            document_version: pending.document_version,
            block_index: pending.target_block,
            line_index: target_line,
            source_offset: target,
        });
    }

    fn visual_affinity_horizontal_target(&self, direction: VisualCaretAffinity) -> Option<usize> {
        if !matches!(self.view_mode, ViewMode::VisualEdit) {
            return None;
        }
        let tab = self.active_tab();
        let affinity = tab.current_visual_caret_affinity()?;
        if affinity == direction {
            return None;
        }
        let cursor = tab.cursor_offset();
        let block_index = visual_block_index_for_offset(
            &tab.visual_list_blocks,
            cursor,
            tab.document.text().len(),
        )?;
        let block = tab.visual_list_blocks.get(block_index)?;
        let projection = build_visual_projection(
            tab.document.text(),
            block,
            tab.selected_range.clone(),
            cursor,
        );
        let display = projection.display_for_source(cursor)?;
        let candidates = projection.boundary_candidates(display);
        candidates
            .is_ambiguous()
            .then(|| candidates.resolve(direction))
    }

    pub(super) fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        tab.selected_range = offset..offset;
        tab.selection_reversed = false;
        tab.clear_visual_caret_affinity();
        tab.clear_visual_navigation_intent();
        tab.finish_undo_capture();
        tab.marked_range = None;
        tab.visual_cursor_reveal_pending = true;
        tab.visual_caret_bounds = None;
        self.center_cursor_if_typewriter();
        cx.notify();
    }

    fn move_to_visual_editor_target(&mut self, range: Range<usize>, cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        tab.selected_range = range;
        tab.selection_reversed = false;
        tab.clear_visual_caret_affinity();
        tab.clear_visual_navigation_intent();
        tab.finish_undo_capture();
        tab.marked_range = None;
        tab.visual_cursor_reveal_pending = true;
        tab.visual_caret_bounds = None;
        self.center_cursor_if_typewriter();
        cx.notify();
    }

    pub(super) fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        tab.clear_visual_caret_affinity();
        tab.clear_visual_navigation_intent();
        tab.finish_undo_capture();
        tab.marked_range = None;
        if tab.selection_reversed {
            tab.selected_range.start = offset;
        } else {
            tab.selected_range.end = offset;
        }
        if tab.selected_range.end < tab.selected_range.start {
            tab.selection_reversed = !tab.selection_reversed;
            tab.selected_range = tab.selected_range.end..tab.selected_range.start;
        }
        tab.visual_cursor_reveal_pending = true;
        tab.visual_caret_bounds = None;
        self.center_cursor_if_typewriter();
        cx.notify();
    }

    pub(super) fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        self.active_tab().index_for_mouse_position(position)
    }

    pub(super) fn previous_boundary(&self, offset: usize) -> usize {
        self.active_tab().previous_boundary(offset)
    }

    pub(super) fn next_boundary(&self, offset: usize) -> usize {
        self.active_tab().next_boundary(offset)
    }
}
