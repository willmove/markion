use super::*;

fn visual_block_transform(block: &VisualBlock) -> Option<BlockTransform> {
    match &block.kind {
        VisualBlockKind::Heading { level } => Some(BlockTransform::Heading(*level)),
        VisualBlockKind::Paragraph | VisualBlockKind::Whitespace => Some(BlockTransform::Text),
        VisualBlockKind::ListItem {
            ordered, checked, ..
        } => Some(if checked.is_some() {
            BlockTransform::TaskList
        } else if *ordered {
            BlockTransform::NumberedList
        } else {
            BlockTransform::BulletedList
        }),
        VisualBlockKind::BlockQuote => Some(BlockTransform::Quote),
        VisualBlockKind::CodeBlock { .. } => Some(BlockTransform::CodeBlock),
        VisualBlockKind::Rule => Some(BlockTransform::Divider),
        VisualBlockKind::Table { .. } => Some(BlockTransform::Table),
        _ => None,
    }
}

pub(super) fn block_menu_root_index_for_transform(transform: BlockTransform) -> usize {
    match transform {
        BlockTransform::Text | BlockTransform::Heading(_) => 0,
        BlockTransform::BulletedList | BlockTransform::NumberedList | BlockTransform::TaskList => 1,
        BlockTransform::Quote => 2,
        BlockTransform::CodeBlock => 3,
        BlockTransform::Divider => 4,
        BlockTransform::Table => 5,
    }
}

fn block_menu_submenu_current_index(
    submenu: BlockMenuSubmenu,
    current: Option<BlockTransform>,
) -> usize {
    current
        .and_then(|current| {
            submenu
                .items()
                .iter()
                .position(|item| *item == BlockMenuItem::Transform(current))
        })
        .unwrap_or(0)
}

pub(super) fn visual_selection_format_target_for_block(
    tab: &EditorTab,
    blocks: &[VisualBlock],
    target: &BlockTarget,
) -> Option<VisualSelectionFormatTarget> {
    let selection = &tab.selected_range;
    if selection.is_empty()
        || !tab.document.text().is_char_boundary(selection.start)
        || !tab.document.text().is_char_boundary(selection.end)
    {
        return None;
    }
    let (_, block) = validate_block_target(tab.document.version(), blocks, target).ok()?;
    if block.source_island.is_some()
        || !block.editable_runs.iter().any(|run| {
            !run.conservative_fallback
                && run.math.is_none()
                && run.html_image.is_none()
                && run.content_range.start <= selection.start
                && run.content_range.end >= selection.end
        })
    {
        return None;
    }
    Some(VisualSelectionFormatTarget {
        document_version: tab.document.version(),
        range: selection.clone(),
        block_id: block.id,
    })
}

impl MarkionApp {
    pub(super) fn sync_slash_command_state(&mut self, cx: &mut Context<Self>) {
        let query = if self.block_menu.is_none()
            && self.recovery_manager.is_none()
            && self.link_editor.is_none()
            && matches!(self.view_mode, ViewMode::VisualEdit)
            && self.active_tab().selected_range.is_empty()
            && self.active_tab().marked_range.is_none()
        {
            slash_query_at(
                self.active_tab().document.text(),
                self.active_tab().cursor_offset(),
                self.active_tab().document.version(),
            )
        } else {
            None
        };
        let Some(query) = query else {
            if self.slash_commands.take().is_some() {
                cx.notify();
            }
            return;
        };
        if self.dismissed_slash_query.as_ref() == Some(&query) {
            if self.slash_commands.take().is_some() {
                cx.notify();
            }
            return;
        }
        self.dismissed_slash_query = None;
        if let Some(state) = &mut self.slash_commands
            && state.query == query
        {
            let count = localized_slash_commands(self.language, &query.query).len();
            state.selected = state.selected.min(count.saturating_sub(1));
            return;
        }
        self.slash_commands = Some(SlashCommandState { query, selected: 0 });
        cx.notify();
    }

    pub(super) fn move_slash_selection(&mut self, forward: bool, cx: &mut Context<Self>) -> bool {
        let Some(state) = &mut self.slash_commands else {
            return false;
        };
        let count = localized_slash_commands(self.language, &state.query.query).len();
        if count == 0 {
            return true;
        }
        state.selected = if forward {
            (state.selected + 1) % count
        } else if state.selected == 0 {
            count - 1
        } else {
            state.selected - 1
        };
        cx.notify();
        true
    }

    pub(super) fn confirm_selected_slash_command(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(state) = self.slash_commands.clone() else {
            return false;
        };
        let commands = localized_slash_commands(self.language, &state.query.query);
        let Some(command) = commands.get(state.selected).copied() else {
            return true;
        };
        self.execute_slash_command(state.query, command, cx);
        true
    }

    pub(super) fn execute_slash_command(
        &mut self,
        query: SlashQuery,
        command: SlashCommand,
        cx: &mut Context<Self>,
    ) {
        match slash_command_edit(
            self.active_tab().document.text(),
            self.active_tab().document.version(),
            &query,
            command,
        ) {
            Ok(edit) => self.apply_exact_block_edit(
                edit,
                p1_t(self.language, P1Msg::SlashCommands).into(),
                cx,
            ),
            Err(error) => self.report_block_edit_error(error, cx),
        }
    }

    pub(super) fn open_visual_block_menu(
        &mut self,
        target: BlockTarget,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(presentation) = self.block_menu_presentation_for_target(&target) else {
            if self.dismiss_visual_block_menu() {
                cx.notify();
            }
            return;
        };
        let selection_format = {
            let tab = self.active_tab();
            let blocks = tab.document.visual_blocks_shared();
            visual_selection_format_target_for_block(tab, &blocks, &target)
        };
        let root_selected = block_menu_root_index_for_transform(presentation.current)
            + if selection_format.is_some() {
                BLOCK_MENU_SELECTION_FORMAT_ITEMS.len()
            } else {
                0
            };
        self.block_menu = Some(BlockMenuState {
            target,
            selection_format,
            anchor,
            root_selected,
            submenu: None,
            submenu_selected: 0,
        });
        self.slash_commands = None;
        self.preview_context_menu = None;
        self.file_tree_context_menu = None;
        self.tab_context_menu = None;
        self.active_menu = None;
        self.link_editor = None;
        cx.notify();
    }

    pub(super) fn show_visual_block_context_menu(
        &mut self,
        _: &ShowVisualBlockContextMenu,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !matches!(self.view_mode, ViewMode::VisualEdit) {
            return;
        }
        let tab = self.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let Some(index) =
            visual_block_index_for_offset(&blocks, tab.cursor_offset(), tab.document.text().len())
        else {
            return;
        };
        if !block_can_transform_at(&blocks, index) {
            return;
        }
        let target = BlockTarget::from_block(tab.document.version(), &blocks[index]);
        let anchor = tab
            .visual_caret_bounds
            .map(|bounds| point(bounds.left(), bounds.bottom()))
            .or_else(|| {
                tab.visual_input_bounds
                    .map(|bounds| point(bounds.left() + px(12.), bounds.top() + px(12.)))
            });
        if let Some(anchor) = anchor {
            self.open_visual_block_menu(target, anchor, cx);
        }
    }

    pub(super) fn block_menu_presentation(&self) -> Option<BlockMenuPresentation> {
        self.block_menu
            .as_ref()
            .and_then(|menu| self.block_menu_presentation_for_target(&menu.target))
    }

    fn block_menu_presentation_for_target(
        &self,
        target: &BlockTarget,
    ) -> Option<BlockMenuPresentation> {
        let tab = self.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let (index, block) = validate_block_target(tab.document.version(), &blocks, target).ok()?;
        if !block_can_transform_at(&blocks, index) {
            return None;
        }
        let current = visual_block_transform(block)?;
        let can_duplicate_or_delete = block_can_reorder_at(&blocks, index);
        Some(BlockMenuPresentation {
            current,
            can_duplicate_or_delete,
            can_move_up: can_duplicate_or_delete
                && adjacent_reorder_target(tab.document.version(), &blocks, target, false).is_ok(),
            can_move_down: can_duplicate_or_delete
                && adjacent_reorder_target(tab.document.version(), &blocks, target, true).is_ok(),
        })
    }

    pub(super) fn select_visual_block_menu_root(
        &mut self,
        index: usize,
        open_submenu: bool,
        cx: &mut Context<Self>,
    ) {
        let current = self.block_menu_presentation().map(|model| model.current);
        let Some(state) = &mut self.block_menu else {
            return;
        };
        let items = state.root_items();
        state.root_selected = index.min(items.len().saturating_sub(1));
        let submenu = match items.get(state.root_selected).copied() {
            Some(BlockMenuItem::Submenu(submenu)) if open_submenu => Some(submenu),
            _ => None,
        };
        if state.submenu != submenu {
            state.submenu = submenu;
            state.submenu_selected = state
                .submenu
                .map(|submenu| block_menu_submenu_current_index(submenu, current))
                .unwrap_or(0);
        }
        cx.notify();
    }

    pub(super) fn select_visual_block_menu_submenu(
        &mut self,
        submenu: BlockMenuSubmenu,
        index: usize,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = &mut self.block_menu else {
            return;
        };
        state.submenu = Some(submenu);
        state.submenu_selected = index.min(submenu.items().len().saturating_sub(1));
        cx.notify();
    }

    pub(super) fn move_visual_block_menu_selection(
        &mut self,
        forward: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(presentation) = self.block_menu_presentation() else {
            return false;
        };
        let Some(state) = &mut self.block_menu else {
            return false;
        };
        if let Some(submenu) = state.submenu {
            let items = submenu.items();
            if items.is_empty() {
                return true;
            }
            for _ in 0..items.len() {
                state.submenu_selected = if forward {
                    (state.submenu_selected + 1) % items.len()
                } else if state.submenu_selected == 0 {
                    items.len() - 1
                } else {
                    state.submenu_selected - 1
                };
                if presentation.item_enabled(items[state.submenu_selected]) {
                    break;
                }
            }
        } else {
            let items = state.root_items();
            if items.is_empty() {
                return true;
            }
            for _ in 0..items.len() {
                state.root_selected = if forward {
                    (state.root_selected + 1) % items.len()
                } else if state.root_selected == 0 {
                    items.len() - 1
                } else {
                    state.root_selected - 1
                };
                if presentation.item_enabled(items[state.root_selected]) {
                    break;
                }
            }
        }
        cx.notify();
        true
    }

    pub(super) fn enter_visual_block_menu_submenu(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(state) = self.block_menu.clone() else {
            return false;
        };
        if state.submenu.is_some() {
            return true;
        }
        let Some(BlockMenuItem::Submenu(submenu)) =
            state.root_items().get(state.root_selected).copied()
        else {
            return true;
        };
        let current = self.block_menu_presentation().map(|model| model.current);
        let Some(state) = &mut self.block_menu else {
            return false;
        };
        state.submenu = Some(submenu);
        state.submenu_selected = block_menu_submenu_current_index(submenu, current);
        cx.notify();
        true
    }

    pub(super) fn leave_visual_block_menu_submenu(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(state) = &mut self.block_menu else {
            return false;
        };
        if state.submenu.take().is_some() {
            cx.notify();
        }
        true
    }

    pub(super) fn confirm_visual_block_menu(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(state) = self.block_menu.clone() else {
            return false;
        };
        let item = if let Some(submenu) = state.submenu {
            submenu.items().get(state.submenu_selected).copied()
        } else {
            state.root_items().get(state.root_selected).copied()
        };
        let Some(item) = item else {
            return true;
        };
        self.activate_visual_block_menu_item(item, cx);
        true
    }

    pub(super) fn activate_visual_block_menu_item(
        &mut self,
        item: BlockMenuItem,
        cx: &mut Context<Self>,
    ) {
        let Some(presentation) = self.block_menu_presentation() else {
            self.close_visual_block_menu(cx);
            return;
        };
        if !presentation.item_enabled(item) {
            return;
        }
        let Some(target) = self.block_menu.as_ref().map(|menu| menu.target.clone()) else {
            return;
        };
        match item {
            BlockMenuItem::SelectionFormat(action) => {
                let selection_format = self
                    .block_menu
                    .as_ref()
                    .and_then(|menu| menu.selection_format.clone());
                let selection_is_current = selection_format.as_ref().is_some_and(|captured| {
                    let tab = self.active_tab();
                    let blocks = tab.document.visual_blocks_shared();
                    visual_selection_format_target_for_block(tab, &blocks, &target).as_ref()
                        == Some(captured)
                });
                if !selection_is_current {
                    self.close_visual_block_menu(cx);
                    return;
                }
                self.dismiss_visual_block_menu();
                match action {
                    SelectionFormatAction::Bold => self.apply_markdown_format(
                        MarkdownFormat::Bold,
                        self.tr(Msg::StatusFmtBold).into(),
                        cx,
                    ),
                    SelectionFormatAction::Italic => self.apply_markdown_format(
                        MarkdownFormat::Italic,
                        self.tr(Msg::StatusFmtItalic).into(),
                        cx,
                    ),
                    SelectionFormatAction::InlineCode => self.apply_markdown_format(
                        MarkdownFormat::InlineCode,
                        self.tr(Msg::StatusFmtInlineCode).into(),
                        cx,
                    ),
                    SelectionFormatAction::Link => self.open_link_editor(cx),
                }
            }
            BlockMenuItem::Submenu(submenu) => {
                let current = Some(presentation.current);
                if let Some(state) = &mut self.block_menu {
                    state.submenu = Some(submenu);
                    state.submenu_selected = block_menu_submenu_current_index(submenu, current);
                }
                cx.notify();
            }
            BlockMenuItem::Transform(transform) => {
                self.transform_visual_block(target, transform, cx)
            }
            BlockMenuItem::Duplicate => self.duplicate_visual_block(target, cx),
            BlockMenuItem::MoveUp => self.move_visual_block(target, false, cx),
            BlockMenuItem::MoveDown => self.move_visual_block(target, true, cx),
            BlockMenuItem::Delete => self.delete_visual_block(target, cx),
        }
    }

    pub(super) fn dismiss_visual_block_menu(&mut self) -> bool {
        self.block_menu.take().is_some()
    }

    pub(super) fn close_visual_block_menu(&mut self, cx: &mut Context<Self>) {
        if self.dismiss_visual_block_menu() {
            cx.notify();
        }
    }

    pub(super) fn transform_visual_block(
        &mut self,
        target: BlockTarget,
        transform: BlockTransform,
        cx: &mut Context<Self>,
    ) {
        let blocks = self.active_tab().document.visual_blocks_shared();
        match transform_block(
            self.active_tab().document.text(),
            self.active_tab().document.version(),
            &blocks,
            &target,
            transform,
        ) {
            Ok(edit) => {
                self.apply_exact_block_edit(edit, p1_t(self.language, P1Msg::TurnInto).into(), cx)
            }
            Err(error) => self.report_block_edit_error(error, cx),
        }
    }

    pub(super) fn duplicate_visual_block(&mut self, target: BlockTarget, cx: &mut Context<Self>) {
        let blocks = self.active_tab().document.visual_blocks_shared();
        match duplicate_block(
            self.active_tab().document.text(),
            self.active_tab().document.version(),
            &blocks,
            &target,
        ) {
            Ok(edit) => self.apply_exact_block_edit(
                edit,
                p1_t(self.language, P1Msg::BlockDuplicated).into(),
                cx,
            ),
            Err(error) => self.report_block_edit_error(error, cx),
        }
    }

    pub(super) fn delete_visual_block(&mut self, target: BlockTarget, cx: &mut Context<Self>) {
        let blocks = self.active_tab().document.visual_blocks_shared();
        match delete_block(
            self.active_tab().document.text(),
            self.active_tab().document.version(),
            &blocks,
            &target,
        ) {
            Ok(edit) => self.apply_exact_block_edit(
                edit,
                p1_t(self.language, P1Msg::BlockDeleted).into(),
                cx,
            ),
            Err(error) => self.report_block_edit_error(error, cx),
        }
    }

    pub(super) fn move_visual_block(
        &mut self,
        target: BlockTarget,
        forward: bool,
        cx: &mut Context<Self>,
    ) {
        let blocks = self.active_tab().document.visual_blocks_shared();
        let destination = match adjacent_reorder_target(
            self.active_tab().document.version(),
            &blocks,
            &target,
            forward,
        ) {
            Ok(destination) => destination,
            Err(error) => {
                self.report_block_edit_error(error, cx);
                return;
            }
        };
        let placement = if forward {
            BlockPlacement::After
        } else {
            BlockPlacement::Before
        };
        self.reorder_visual_block(target, destination, placement, cx);
    }

    pub(super) fn reorder_visual_block(
        &mut self,
        moving: BlockTarget,
        destination: BlockTarget,
        placement: BlockPlacement,
        cx: &mut Context<Self>,
    ) {
        let blocks = self.active_tab().document.visual_blocks_shared();
        match reorder_block(
            self.active_tab().document.text(),
            self.active_tab().document.version(),
            &blocks,
            &moving,
            &destination,
            placement,
        ) {
            Ok(edit) => {
                self.apply_exact_block_edit(edit, p1_t(self.language, P1Msg::BlockMoved).into(), cx)
            }
            Err(error) => self.report_block_edit_error(error, cx),
        }
    }

    fn apply_exact_block_edit(
        &mut self,
        edit: BlockEdit,
        status: SharedString,
        cx: &mut Context<Self>,
    ) {
        tracing::debug!(
            target: "markion::editing",
            op = "exact_block_edit",
            range = ?edit.range,
            replacement_len = edit.replacement.len(),
            document_version = edit.document_version,
            "block menu edit"
        );
        self.active_tab_mut().finish_undo_capture();
        let snapshot = self.snapshot();
        // The edit is bound to the version it was computed against; the
        // checked boundary rejects a block command that outlived an edit.
        let mutation = {
            let tab = self.active_tab_mut();
            CheckedMutation::range(
                tab.document.instance_id(),
                edit.document_version,
                MutationOrigin::ExactBlockEdit,
                edit.range.clone(),
                tab.document
                    .text()
                    .get(edit.range.clone())
                    .map(str::to_string)
                    .unwrap_or_default(),
                edit.replacement.clone(),
            )
        };
        if self
            .apply_document_mutation("exact_block_edit", mutation)
            .is_none()
        {
            cx.notify();
            return;
        }
        self.commit_undo_snapshot(snapshot);
        let tab = self.active_tab_mut();
        tab.selected_range = edit.selection_after;
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.slash_commands = None;
        self.dismissed_slash_query = None;
        self.dismiss_visual_block_menu();
        self.status = status;
        self.after_document_changed(cx);
        cx.notify();
    }

    fn report_block_edit_error(&mut self, error: BlockEditError, cx: &mut Context<Self>) {
        self.slash_commands = None;
        self.dismissed_slash_query = None;
        self.dismiss_visual_block_menu();
        self.status = p1_t(
            self.language,
            match error {
                BlockEditError::Stale => P1Msg::BlockStale,
                BlockEditError::Unsupported | BlockEditError::Ambiguous => P1Msg::BlockUnsupported,
            },
        )
        .into();
        cx.notify();
    }

    pub(super) fn request_image_import(
        &mut self,
        inputs: Vec<PendingImageInput>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if inputs.is_empty() {
            return;
        }
        if self.active_tab().document.path().is_none() {
            self.pending_image_import = Some(inputs);
            self.status = p0_t(self.language, P0Msg::SaveBeforeImage).into();
            self.save_document_as(&SaveDocumentAs, window, cx);
            return;
        }
        self.insert_image_inputs(inputs, cx);
    }

    pub(super) fn flush_pending_image_import(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = self.pending_image_import.take() else {
            return;
        };
        if self.active_tab().document.path().is_some() {
            self.insert_image_inputs(inputs, cx);
        }
    }

    fn insert_image_inputs(&mut self, inputs: Vec<PendingImageInput>, cx: &mut Context<Self>) {
        let Some(document_path) = self.active_tab().document.path().map(Path::to_path_buf) else {
            self.pending_image_import = Some(inputs);
            return;
        };
        let mut markdown = Vec::with_capacity(inputs.len());
        let mut failures = Vec::new();
        for input in inputs {
            match import_image_bytes(&document_path, &input.stem, &input.extension, &input.bytes) {
                Ok(imported) => markdown.push(serialize_inline_image(
                    &input.stem,
                    &imported.relative_url,
                    None,
                    None,
                )),
                Err(err) => failures.push(err.to_string()),
            }
        }
        if markdown.is_empty() {
            self.status = failures.join("; ").into();
            cx.notify();
            return;
        }

        self.active_tab_mut().finish_undo_capture();
        let snapshot = self.snapshot();
        let replacement = markdown.join("\n");
        let selected = self.active_tab().selected_range.clone();
        let insertion_start = selected.start;
        let mutation = {
            let tab = self.active_tab_mut();
            tab.document.prepare_range_mutation(
                MutationOrigin::MarkdownFormat,
                selected,
                &replacement,
            )
        };
        if self
            .apply_document_mutation("insert_image_inputs", mutation)
            .is_none()
        {
            cx.notify();
            return;
        }
        self.commit_undo_snapshot(snapshot);
        let tab = self.active_tab_mut();
        tab.selected_range =
            insertion_start + replacement.len()..insertion_start + replacement.len();
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.status = if failures.is_empty() {
            t(self.language, Msg::StatusFmtImage).into()
        } else {
            p0_tf(
                self.language,
                P0Msg::ImagePartialFailure,
                &[&failures.join("; ")],
            )
            .into()
        };
        self.after_document_changed(cx);
        cx.notify();
    }

    pub(super) fn set_image_presentation_at(
        &mut self,
        offset: usize,
        presentation: ImagePresentation,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = inline_image_at(self.active_tab().document.text(), offset) else {
            return;
        };
        let replacement = serialize_inline_image(
            &target.label,
            &target.url,
            target.title.as_deref(),
            Some(presentation),
        );
        self.replace_exact_inline_target(target.source_range, replacement, cx);
    }

    pub(super) fn replace_image_resource_at(&mut self, offset: usize, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(p0_t(self.language, P0Msg::ChooseReplacementImage).into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(source_path) = paths.into_iter().next() else {
                return;
            };
            let _ = this.update(cx, |app, cx| {
                if !image_extension_supported(&source_path) {
                    app.status = p0_t(app.language, P0Msg::UnsupportedImage).into();
                    cx.notify();
                    return;
                }
                let Some(target) = inline_image_at(app.active_tab().document.text(), offset) else {
                    app.status = p0_t(app.language, P0Msg::ImageSourceAmbiguous).into();
                    cx.notify();
                    return;
                };
                let Some(document_path) = app.active_tab().document.path().map(Path::to_path_buf)
                else {
                    app.status = p0_t(app.language, P0Msg::SaveBeforeImage).into();
                    cx.notify();
                    return;
                };
                match import_image_file(&document_path, &source_path) {
                    Ok(imported) => {
                        let replacement = serialize_inline_image(
                            &target.label,
                            &imported.relative_url,
                            target.title.as_deref(),
                            target.presentation,
                        );
                        app.replace_exact_inline_target(target.source_range, replacement, cx);
                    }
                    Err(err) => {
                        app.status =
                            p0_tf(app.language, P0Msg::ImageReplaceFailed, &[&err.to_string()])
                                .into();
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    fn replace_exact_inline_target(
        &mut self,
        source_range: Range<usize>,
        replacement: String,
        cx: &mut Context<Self>,
    ) {
        self.active_tab_mut().finish_undo_capture();
        let snapshot = self.snapshot();
        let start = source_range.start;
        let mutation = {
            let tab = self.active_tab_mut();
            tab.document.prepare_range_mutation(
                MutationOrigin::MarkdownFormat,
                source_range,
                &replacement,
            )
        };
        if self
            .apply_document_mutation("replace_exact_inline_target", mutation)
            .is_none()
        {
            cx.notify();
            return;
        }
        self.commit_undo_snapshot(snapshot);
        let tab = self.active_tab_mut();
        tab.selected_range = start..start + replacement.len();
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.status = t(self.language, Msg::StatusFmtImage).into();
        self.after_document_changed(cx);
        cx.notify();
    }

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
        tracing::debug!(target: "markion::editing", op = "undo", "undo invoked");
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
        tracing::debug!(target: "markion::editing", op = "redo", "redo invoked");
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
        tracing::debug!(
            target: "markion::editing",
            op = "apply_markdown_format",
            selection = ?self.active_tab().selected_range,
            "format action"
        );
        self.active_tab_mut().finish_undo_capture();
        let snapshot = self.snapshot();
        let tab = self.active_tab_mut();
        let selected_range = tab.selected_range.clone();
        let new_range = tab.document.apply_markdown_format(selected_range, format);
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
        tracing::debug!(
            target: "markion::editing",
            op = "table_edit",
            offset,
            "table toolbar or source command"
        );
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
        if matches!(self.view_mode, ViewMode::VisualEdit) {
            self.open_link_editor(cx);
        } else {
            self.apply_markdown_format(
                MarkdownFormat::Link,
                self.tr(Msg::StatusFmtLink).into(),
                cx,
            );
        }
    }

    pub(super) fn open_link_editor(&mut self, cx: &mut Context<Self>) {
        self.dismiss_visual_block_menu();
        let selected = self.active_tab().safe_selected_range();
        let cursor = self.active_tab().cursor_offset();
        let existing = inline_link_at(self.active_tab().document.text(), cursor);
        let (source_range, label, url, title) = if let Some(link) = existing {
            (
                link.source_range,
                link.label,
                link.url,
                link.title.unwrap_or_default(),
            )
        } else if !selected.is_empty() {
            (
                selected.clone(),
                self.active_tab().document.text()[selected].to_string(),
                String::new(),
                String::new(),
            )
        } else {
            (selected, "link text".into(), String::new(), String::new())
        };
        self.link_editor = Some(LinkEditorState {
            source_range,
            document_version: self.active_tab().document.version(),
            label,
            url,
            title,
            field: LinkEditorField::Url,
        });
        self.input_marked_len = 0;
        self.status = p0_t(self.language, P0Msg::EditingLink).into();
        cx.notify();
    }

    pub(super) fn focus_link_editor_field(
        &mut self,
        field: LinkEditorField,
        cx: &mut Context<Self>,
    ) {
        if let Some(editor) = self.link_editor.as_mut() {
            editor.field = field;
            self.input_marked_len = 0;
            cx.notify();
        }
    }

    pub(super) fn cancel_link_editor(&mut self, cx: &mut Context<Self>) {
        self.link_editor = None;
        self.input_marked_len = 0;
        self.status = t(self.language, Msg::StatusCanceled).into();
        cx.notify();
    }

    pub(super) fn confirm_link_editor(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = self.link_editor.take() else {
            return;
        };
        self.input_marked_len = 0;
        if editor.url.trim().is_empty() {
            self.link_editor = Some(editor);
            self.status = p0_t(self.language, P0Msg::LinkUrlRequired).into();
            cx.notify();
            return;
        }
        if self.active_tab().document.version() != editor.document_version
            || editor.source_range.end > self.active_tab().document.text().len()
        {
            self.status = p0_t(self.language, P0Msg::LinkStale).into();
            cx.notify();
            return;
        }
        let replacement = serialize_inline_link(
            &editor.label,
            editor.url.trim(),
            (!editor.title.trim().is_empty()).then_some(editor.title.trim()),
        );
        self.active_tab_mut().finish_undo_capture();
        let snapshot = self.snapshot();
        let start = editor.source_range.start;
        // The link editor captured its source range at `document_version`;
        // the checked boundary enforces exactly that generation.
        let mutation = {
            let tab = self.active_tab_mut();
            CheckedMutation::range(
                tab.document.instance_id(),
                editor.document_version,
                MutationOrigin::MarkdownFormat,
                editor.source_range.clone(),
                tab.document
                    .text()
                    .get(editor.source_range.clone())
                    .map(str::to_string)
                    .unwrap_or_default(),
                &replacement,
            )
        };
        if self
            .apply_document_mutation("confirm_link_editor", mutation)
            .is_none()
        {
            cx.notify();
            return;
        }
        self.commit_undo_snapshot(snapshot);
        let tab = self.active_tab_mut();
        tab.selected_range = start..start + replacement.len();
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.status = t(self.language, Msg::StatusFmtLink).into();
        self.after_document_changed(cx);
        cx.notify();
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
        if !self.active_tab().requires_discard_confirmation() {
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

    /// Save / Don't Save / Cancel. Don't Save uses `PromptButton::new` (Other)
    /// so Windows TaskDialog IDs stay unique — three `ok` buttons all map to
    /// IDOK and the first would win.
    pub(super) fn unsaved_choice_buttons(&self) -> [PromptButton; 3] {
        [
            PromptButton::ok(self.tr(Msg::DialogButtonSave)),
            PromptButton::new(self.tr(Msg::DialogButtonDontSave)),
            PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
        ]
    }

    pub(super) fn abort_unsaved_prompt(&mut self, canceled: bool, cx: &mut Context<Self>) {
        self.confirming_close = false;
        self.active_menu = None;
        self.status = t(
            self.language,
            if canceled {
                Msg::StatusCanceled
            } else {
                Msg::StatusExitCanceled
            },
        )
        .into();
        cx.notify();
    }

    pub(super) fn finish_unsaved_exit(
        &mut self,
        window_handle: gpui::AnyWindowHandle,
        kind: UnsavedExitKind,
        cx: &mut Context<Self>,
    ) {
        self.confirming_close = false;
        self.allow_close = true;
        self.status = t(self.language, Msg::StatusExitingMarkion).into();
        cx.notify();
        if let Ok(bounds) = window_handle.update(cx, |_, window, _| window.window_bounds()) {
            self.apply_window_bounds(bounds);
        }
        self.flush_layout();
        match kind {
            UnsavedExitKind::MenuQuit => {
                let _ = window_handle.update(cx, |_, window, _| window.remove_window());
                cx.quit();
            }
            UnsavedExitKind::WindowClose => {
                cx.quit();
            }
        }
    }

    pub(super) fn request_quit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.begin_unsaved_exit(window, cx, UnsavedExitKind::MenuQuit);
    }

    pub(super) fn begin_unsaved_exit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        kind: UnsavedExitKind,
    ) {
        if self.confirming_close {
            return;
        }

        self.active_menu = None;
        if !self.tabs.iter().any(EditorTab::is_dirty) {
            self.finish_unsaved_exit(window.window_handle(), kind, cx);
            return;
        }

        let count = self
            .tabs
            .iter()
            .filter(|tab| tab.is_dirty())
            .count()
            .to_string();
        let title = t(self.language, Msg::DialogExitTitle);
        let detail = tf(self.language, Msg::DialogExitDetail, &[&count]);
        let buttons = self.unsaved_choice_buttons();
        let waiting = match kind {
            UnsavedExitKind::MenuQuit => Msg::StatusWaitingExitConfirm,
            UnsavedExitKind::WindowClose => Msg::StatusWaitingQuitConfirm,
        };
        let answer = window.prompt(PromptLevel::Warning, title, Some(&detail), &buttons, cx);
        self.status = t(self.language, waiting).into();
        self.confirming_close = true;
        cx.notify();
        let window_handle = window.window_handle();
        let targets: Vec<TabContextTarget> = self
            .tabs
            .iter()
            .enumerate()
            .filter(|(_, tab)| tab.is_dirty())
            .map(|(index, tab)| TabContextTarget::capture(index, tab))
            .collect();

        cx.spawn(async move |this, cx| {
            let choice = UnsavedChoice::from_prompt(answer.await);
            match choice {
                UnsavedChoice::Save => {
                    Self::save_dirty_tabs_then_exit(this, cx, window_handle, kind, targets).await;
                }
                UnsavedChoice::Discard => {
                    let _ = this.update(cx, |app, cx| {
                        app.discard_all_tab_recovery_files();
                        app.finish_unsaved_exit(window_handle, kind, cx);
                    });
                }
                UnsavedChoice::Cancel => {
                    let _ = this.update(cx, |app, cx| {
                        app.abort_unsaved_prompt(false, cx);
                    });
                }
            }
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
        self.dismiss_visual_block_menu();
        self.tab_context_menu = None;
        self.active_menu = if self.active_menu == Some(menu) {
            None
        } else {
            Some(menu)
        };
        self.open_recent_submenu_open = false;
        cx.notify();
    }

    pub(super) fn hover_menu(&mut self, menu: AppMenu, cx: &mut Context<Self>) {
        let next_menu = menu_after_hover(self.active_menu, menu);
        if next_menu != self.active_menu {
            self.active_menu = next_menu;
            self.open_recent_submenu_open = false;
            cx.notify();
        }
    }

    pub(super) fn open_open_recent_submenu(&mut self, cx: &mut Context<Self>) {
        if self.active_menu == Some(AppMenu::File) && !self.open_recent_submenu_open {
            self.open_recent_submenu_open = true;
            cx.notify();
        }
    }

    pub(super) fn close_open_recent_submenu(&mut self, cx: &mut Context<Self>) {
        if self.open_recent_submenu_open {
            self.open_recent_submenu_open = false;
            cx.notify();
        }
    }

    pub(super) fn toggle_open_recent_submenu(&mut self, cx: &mut Context<Self>) {
        if self.active_menu != Some(AppMenu::File) {
            return;
        }
        self.open_recent_submenu_open = !self.open_recent_submenu_open;
        cx.notify();
    }

    pub(super) fn close_menu(
        &mut self,
        _: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        eprintln!("[menu-debug] close_menu, active={:?}", self.active_menu);
        // Each mouse-down starts a fresh click: clear the click-away flag left
        // over from the previous click before deciding what this click does.
        self.name_editor_click_away = false;
        let had_transient_ui = self.active_menu.is_some()
            || self.file_tree_context_menu.is_some()
            || self.preview_context_menu.is_some()
            || self.tab_context_menu.is_some()
            || self.block_menu.is_some();
        if had_transient_ui {
            self.active_menu = None;
            self.open_recent_submenu_open = false;
            self.file_tree_context_menu = None;
            self.preview_context_menu = None;
            self.tab_context_menu = None;
            self.dismiss_visual_block_menu();
        }
        // A left mouse-down anywhere below the menu bar is a click-away for
        // the inline name editor: commit through the same pipeline as Enter
        // (Explorer semantics) instead of silently discarding the typed name.
        // The editor's own row stops propagation, so clicks inside the field
        // position the caret rather than landing here. The flag lets the
        // subsequent mouse-up (tree-row open, tab switch) consume this click.
        if self.pending_name_input.is_some() {
            self.confirm_pending_name(&ConfirmPendingName, window, cx);
            self.name_editor_click_away = true;
        }
        if had_transient_ui || self.name_editor_click_away {
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
        self.tab_context_menu = None;
        self.dismiss_visual_block_menu();
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

    /// Open the tab-bar context menu for the tab at `index`. The tab's
    /// identity is captured alongside the index so a dispatch after the tab
    /// vector mutated cancels instead of acting on the wrong tab.
    pub(super) fn show_tab_context_menu(
        &mut self,
        index: usize,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(tab) = self.tabs.get(index) else {
            return;
        };
        let target = TabContextTarget::capture(index, tab);
        self.active_menu = None;
        self.file_tree_context_menu = None;
        self.preview_context_menu = None;
        self.dismiss_visual_block_menu();
        self.pending_name_input = None;
        self.tab_context_menu = Some(TabContextMenu { target, position });
        cx.notify();
    }

    /// Dispatch a tab context-menu action. Switch-then-operate: the clicked
    /// tab becomes active before the action runs (the `×` button idiom). The
    /// captured target identity is re-validated first — the menu can stay
    /// open across tab mutations, and a stale index must cancel instead of
    /// acting on whatever tab now sits at that position.
    pub(super) fn handle_tab_context_action(
        &mut self,
        action: TabContextAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(menu) = self.tab_context_menu.take() else {
            return;
        };
        let target = menu.target;
        if !self
            .tabs
            .get(target.index)
            .is_some_and(|tab| target.matches(tab))
        {
            self.active_menu = None;
            self.status = t(self.language, Msg::StatusCanceled).into();
            cx.notify();
            return;
        }
        let index = target.index;
        match action {
            TabContextAction::CloseTab => {
                self.switch_active_tab(index, cx);
                self.close_tab(&CloseTab, window, cx);
            }
            TabContextAction::CloseOthers => {
                self.switch_active_tab(index, cx);
                self.close_other_tabs(index, window, cx);
            }
            TabContextAction::CloseToTheRight => {
                self.switch_active_tab(index, cx);
                self.close_tabs_to_the_right(index, window, cx);
            }
            TabContextAction::Rename => {
                let Some(path) = target.path.clone() else {
                    return;
                };
                self.switch_active_tab(index, cx);
                // Same rule as the file-tree rename: refuse while dirty so
                // unsaved edits cannot be lost to a reopen from the new path.
                if self.active_tab().is_dirty() {
                    self.status = t(self.language, Msg::StatusSaveBeforeRename).into();
                    cx.notify();
                    return;
                }
                let parent = path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| self.workspace_root.clone());
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("")
                    .to_string();
                self.selected_tree_path = Some(path.clone());
                self.open_name_prompt(PendingNameKind::Rename, parent, Some(path), &file_name, cx);
            }
            TabContextAction::CopyPath => {
                let Some(path) = target.path.clone() else {
                    return;
                };
                let display = path.display().to_string();
                cx.write_to_clipboard(ClipboardItem::new_string(display.clone()));
                self.status = self.trf(Msg::StatusCopiedPath, &[&display]);
                cx.notify();
            }
            TabContextAction::RevealInFileManager => {
                let Some(path) = target.path.clone() else {
                    return;
                };
                match reveal_in_system_file_manager(&path, true) {
                    Ok(()) => {
                        self.status = self.trf(
                            Msg::StatusShownInFileManager,
                            &[&path.display().to_string()],
                        );
                    }
                    Err(err) => {
                        self.status =
                            self.trf(Msg::StatusShowInFileManagerFailed, &[&err.to_string()]);
                    }
                }
                cx.notify();
            }
        }
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
        self.search_control_focus = Some(SearchOverlayControl::Next);
        self.find_next(&FindNext, window, cx);
    }

    pub(super) fn click_find_previous(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_control_focus = Some(SearchOverlayControl::Previous);
        self.find_previous(&FindPrevious, window, cx);
    }

    pub(super) fn click_replace_current(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_control_focus = Some(SearchOverlayControl::ReplaceCurrent);
        self.replace_current_match(&ReplaceCurrentMatch, window, cx);
    }

    pub(super) fn click_replace_all(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_control_focus = Some(SearchOverlayControl::ReplaceAll);
        self.replace_all_matches(&ReplaceAllMatches, window, cx);
    }

    pub(super) fn click_close_search(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_control_focus = Some(SearchOverlayControl::Close);
        self.close_search_overlay(cx);
    }

    pub(super) fn click_toggle_case(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_control_focus = Some(SearchOverlayControl::MatchCase);
        self.toggle_find_case_sensitive(&ToggleFindCaseSensitive, window, cx);
    }

    pub(super) fn click_toggle_regex(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_control_focus = Some(SearchOverlayControl::Regex);
        self.toggle_find_regex(&ToggleFindRegex, window, cx);
    }

    pub(super) fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.move_search_caret(SearchCaretMove::Left, cx) {
            return;
        }
        if self.pending_name_input.is_some() {
            self.move_name_caret(NameCaretMove::Left, cx);
            return;
        }
        if self.leave_visual_block_menu_submenu(cx) {
            return;
        }
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
        if self.move_search_caret(SearchCaretMove::Right, cx) {
            return;
        }
        if self.pending_name_input.is_some() {
            self.move_name_caret(NameCaretMove::Right, cx);
            return;
        }
        if self.enter_visual_block_menu_submenu(cx) {
            return;
        }
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
        if self.move_search_caret(SearchCaretMove::SelectLeft, cx) {
            return;
        }
        if self.pending_name_input.is_some() {
            self.move_name_caret(NameCaretMove::SelectLeft, cx);
            return;
        }
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
        if self.move_search_caret(SearchCaretMove::SelectRight, cx) {
            return;
        }
        if self.pending_name_input.is_some() {
            self.move_name_caret(NameCaretMove::SelectRight, cx);
            return;
        }
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
        if self.move_visual_block_menu_selection(false, cx) {
            return;
        }
        self.sync_slash_command_state(cx);
        if self.move_slash_selection(false, cx) {
            return;
        }
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
        if self.move_visual_block_menu_selection(true, cx) {
            return;
        }
        self.sync_slash_command_state(cx);
        if self.move_slash_selection(true, cx) {
            return;
        }
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
        if self.move_search_caret(SearchCaretMove::SelectAll, cx) {
            return;
        }
        if self.pending_name_input.is_some() {
            self.move_name_caret(NameCaretMove::SelectAll, cx);
            return;
        }
        self.move_to(0, cx);
        let len = self.active_tab().document.text().len();
        self.select_to(len, cx);
    }

    pub(super) fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        if self.move_search_caret(SearchCaretMove::Home, cx) {
            return;
        }
        if self.pending_name_input.is_some() {
            self.move_name_caret(NameCaretMove::Home, cx);
            return;
        }
        if let Some(target) = self.visual_painted_line_boundary(false) {
            self.move_to(target, cx);
            return;
        }
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.line_start_at(cursor);
        self.move_to(target, cx);
    }

    pub(super) fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        if self.move_search_caret(SearchCaretMove::End, cx) {
            return;
        }
        if self.pending_name_input.is_some() {
            self.move_name_caret(NameCaretMove::End, cx);
            return;
        }
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
        if self.search_visible && self.search_control_focus.is_some() {
            self.find_next(&FindNext, _window, cx);
            return;
        }
        if self.confirm_visual_block_menu(cx) {
            return;
        }
        // When the inline name prompt is open, Enter commits the name instead
        // of inserting a newline into the document.
        if self.pending_name_input.is_some() {
            self.confirm_pending_name(&ConfirmPendingName, _window, cx);
            return;
        }
        if self.link_editor.is_some() {
            self.confirm_link_editor(cx);
            return;
        }
        self.sync_slash_command_state(cx);
        if self.confirm_selected_slash_command(cx) {
            return;
        }
        let selected = self.active_tab().selected_range.clone();
        if matches!(self.view_mode, ViewMode::VisualEdit)
            && let Some(field) = self.active_tab().document.visual_editor_field_at(&selected)
        {
            match field.kind {
                VisualEditorFieldKind::TableCell { .. } => {
                    if let Some(target) = self
                        .active_tab()
                        .document
                        .visual_editor_tab_target(&selected, true)
                    {
                        self.move_to_visual_editor_target(target, cx);
                    }
                    return;
                }
                VisualEditorFieldKind::CodePayload
                | VisualEditorFieldKind::MathPayload
                | VisualEditorFieldKind::HtmlSource
                | VisualEditorFieldKind::ImageSource => {
                    self.active_tab_mut().pending_text_edit_intent = Some(UndoCaptureKind::Atomic);
                    self.replace_text_in_range(None, "\n", _window, cx);
                    self.active_tab_mut().finish_undo_capture();
                    return;
                }
                // Image field kinds are retained on the enum but no longer
                // produced by any visual editor, so they are unreachable here.
                VisualEditorFieldKind::ImageAlt
                | VisualEditorFieldKind::ImageDestination
                | VisualEditorFieldKind::ImageTitle => {}
                // Enter in the language field commits the token instead of
                // inserting a newline into the fence's info string.
                VisualEditorFieldKind::CodeInfo => return,
            }
        }
        let cursor = self.active_tab().selected_range.start;
        let structural_edit = (matches!(self.view_mode, ViewMode::VisualEdit)
            && selected.is_empty())
        .then(|| self.active_tab().document.visual_enter_edit(cursor))
        .flatten();
        tracing::debug!(
            target: "markion::editing",
            op = "insert_newline",
            cursor,
            selection_len = selected.len(),
            structural = structural_edit.is_some(),
            "enter pressed"
        );
        self.push_undo_snapshot();
        if let Some(edit) = structural_edit {
            let mutation = {
                let tab = self.active_tab_mut();
                tab.document.prepare_range_mutation(
                    MutationOrigin::StructuralEdit,
                    edit.range.clone(),
                    &edit.replacement,
                )
            };
            if self
                .apply_document_mutation("insert_newline_structural", mutation)
                .is_none()
            {
                cx.notify();
                return;
            }
            let tab = self.active_tab_mut();
            tab.selected_range = edit.selection_after;
        } else {
            if !selected.is_empty() {
                let mutation = {
                    let tab = self.active_tab_mut();
                    tab.document.prepare_range_mutation(
                        MutationOrigin::StructuralEdit,
                        selected,
                        "",
                    )
                };
                if self
                    .apply_document_mutation("insert_newline_replace_selection", mutation)
                    .is_none()
                {
                    cx.notify();
                    return;
                }
            }
            let tab = self.active_tab_mut();
            let new_cursor = tab.document.insert_markdown_newline(cursor);
            tab.selected_range = new_cursor..new_cursor;
        }
        let tab = self.active_tab_mut();
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.status = t(self.language, Msg::StatusEditing).into();
        self.after_document_changed(cx);
        cx.notify();
    }

    pub(super) fn search_previous_or_newline(
        &mut self,
        _: &SearchPreviousOrNewline,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.search_visible && self.search_control_focus.is_some() {
            self.find_previous(&FindPrevious, window, cx);
        } else {
            self.insert_newline(&InsertNewline, window, cx);
        }
    }

    pub(super) fn indent(&mut self, _: &Indent, window: &mut Window, cx: &mut Context<Self>) {
        if self.search_focus.is_some() {
            self.cycle_search_overlay_focus(true, cx);
            return;
        }
        if self.search_visible && self.search_control_focus.is_some() {
            self.cycle_search_overlay_focus(true, cx);
            return;
        }
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
        if self.search_visible && self.search_control_focus.is_some() {
            self.cycle_search_overlay_focus(false, cx);
            return;
        }
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
                .or_else(|| {
                    self.active_tab()
                        .document
                        .visual_atomic_token_edit(self.cursor_offset(), false)
                })
        {
            self.push_undo_snapshot();
            let mutation = {
                let tab = self.active_tab_mut();
                tab.document.prepare_range_mutation(
                    MutationOrigin::StructuralEdit,
                    edit.range.clone(),
                    &edit.replacement,
                )
            };
            if self
                .apply_document_mutation("backspace_visual_edit", mutation)
                .is_none()
            {
                cx.notify();
                return;
            }
            let tab = self.active_tab_mut();
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
        if self.delete_text_input_forward(cx) {
            return;
        }
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

        if matches!(self.view_mode, ViewMode::VisualEdit)
            && self.active_tab().selected_range.is_empty()
            && let Some(edit) = self
                .active_tab()
                .document
                .visual_atomic_token_edit(self.cursor_offset(), true)
        {
            self.push_undo_snapshot();
            let mutation = {
                let tab = self.active_tab_mut();
                tab.document.prepare_range_mutation(
                    MutationOrigin::StructuralEdit,
                    edit.range.clone(),
                    &edit.replacement,
                )
            };
            if self
                .apply_document_mutation("delete_visual_atomic_token", mutation)
                .is_none()
            {
                cx.notify();
                return;
            }
            let tab = self.active_tab_mut();
            tab.selected_range = edit.selection_after;
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.status = t(self.language, Msg::StatusEditing).into();
            self.after_document_changed(cx);
            cx.notify();
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
        let Some(item) = cx.read_from_clipboard() else {
            self.status = t(self.language, Msg::StatusClipboardEmpty).into();
            cx.notify();
            return;
        };
        if !self.has_text_input_focus()
            && let Some(image) = item.entries().iter().find_map(|entry| match entry {
                ClipboardEntry::Image(image) => Some(image),
                ClipboardEntry::String(_) => None,
            })
        {
            let extension = match image.format {
                ImageFormat::Png => "png",
                ImageFormat::Jpeg => "jpg",
                ImageFormat::Webp => "webp",
                ImageFormat::Gif => "gif",
                ImageFormat::Svg => "svg",
                ImageFormat::Bmp => "bmp",
                ImageFormat::Tiff => "tiff",
            };
            self.request_image_import(
                vec![PendingImageInput {
                    stem: "pasted-image".into(),
                    extension: extension.into(),
                    bytes: image.bytes.clone(),
                }],
                window,
                cx,
            );
        } else if let Some(text) = item.text() {
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
        if let Some(text) = self
            .focused_search_field()
            .and_then(SearchFieldState::selected_text)
            .map(ToString::to_string)
        {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.status = t(self.language, Msg::StatusCopiedSelection).into();
            cx.notify();
            return;
        }
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
        let selected = self.active_tab().safe_selected_range();
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
        if self.search_focus.is_some() {
            if let Some(text) = self
                .focused_search_field()
                .and_then(SearchFieldState::selected_text)
                .map(ToString::to_string)
            {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                if let Some(field) = self.focused_search_field_mut() {
                    field.replace_selection("", false);
                }
                self.search_generation = None;
                self.after_input_changed(cx);
            } else {
                self.status = t(self.language, Msg::StatusNothingToCut).into();
                cx.notify();
            }
            return;
        }
        let selected = self.active_tab().safe_selected_range();
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
        // While the inline name editor is open, a click in the editor pane is
        // a click-away: do NOT move the document caret or start a selection.
        // The ancestor workspace-row handler (close_menu) commits the name on
        // the same mouse-down; this guard only needs to keep the editor inert.
        if self.pending_name_input.is_some() {
            return;
        }
        // Clicking into the editor returns text-input focus to the document,
        // otherwise typed characters keep flowing into the file-tree filter
        // or search fields that last held focus.
        self.file_tree_query_focused = false;
        self.search_focus = None;
        self.search_control_focus = None;
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
        if !document_tab_band_visible(self.tabs.len()) {
            // Single-tab case: render nothing (tab bar hidden).
            return div();
        }
        let active = self.active_tab;
        // The strip scrolls horizontally instead of clipping, so tabs beyond
        // the available width stay reachable. A plain vertical wheel over an
        // x-only scroll container scrolls it (GPUI routes the delta for us).
        let document_bar = div()
            .id("tab-bar-scroll")
            .h_full()
            .flex_1()
            .min_w_0()
            .px_2()
            .flex()
            .items_end()
            .gap_1()
            .overflow_x_scroll()
            .track_scroll(&self.tab_bar_scroll);
        let document_bar =
            document_bar.children(self.tabs.iter().enumerate().map(|(index, tab)| {
                let is_active = index == active;
                let name = tab.title();
                let dirty = tab.is_dirty();
                let label: SharedString = name.clone().into();
                // Tooltip data must be captured separately from the truncated
                // label: it restores the full title (and path) on hover.
                let tooltip_title: SharedString = name.into();
                let tooltip_path: Option<SharedString> =
                    tab.path().map(|path| path.display().to_string().into());
                let tooltip_palette = palette;
                // Theme-driven so tabs stay legible on dark palettes (the previous
                // hard-coded light hexes rendered white tabs with light text).
                let bg = if is_active {
                    palette.surface_bg
                } else {
                    palette.panel_bg
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
                let hover_bg = if is_active {
                    palette.surface_bg
                } else {
                    palette.active_bg
                };
                div()
                    .id(ElementId::named_usize("document-tab", index))
                    .tooltip(move |_, cx| {
                        cx.new(|_| TabTooltip {
                            palette: tooltip_palette,
                            title: tooltip_title.clone(),
                            path: tooltip_path.clone(),
                        })
                        .into()
                    })
                    .max_w(px(DOCUMENT_TAB_MAX_WIDTH))
                    .min_w(px(DOCUMENT_TAB_MIN_WIDTH))
                    .flex_shrink()
                    .px_2()
                    .py_1()
                    .rounded_t_md()
                    .border_1()
                    .when(is_active, |style| {
                        style.border_t_2().border_b_0().mb(px(-1.))
                    })
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
                            // The click that click-away-committed the inline
                            // name editor — or one made while it is still open
                            // (a refused commit) — must not switch tabs.
                            if app.pending_name_input.is_some() || app.name_editor_click_away {
                                app.name_editor_click_away = false;
                                cx.notify();
                                return;
                            }
                            // The captured `index` is fixed at render time; a tab
                            // close/open since then may have shifted positions, so
                            // guard against a stale out-of-range index.
                            if index < app.tabs.len() {
                                app.switch_active_tab(index, cx);
                            }
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Right,
                        cx.listener(move |app, event: &MouseUpEvent, _window, cx| {
                            if index < app.tabs.len() {
                                app.show_tab_context_menu(index, event.position, cx);
                            }
                        }),
                    )
                    .on_mouse_up(
                        // Middle-click closes, mirroring the browser/editor
                        // convention: switch-then-close, exactly like `×`.
                        MouseButton::Middle,
                        cx.listener(move |app, _: &MouseUpEvent, window, cx| {
                            if index < app.tabs.len() {
                                app.switch_active_tab(index, cx);
                                app.close_tab(&CloseTab, window, cx);
                            }
                        }),
                    )
                    .child(
                        // Truncating wrapper bounds long titles to the tab's
                        // maximum width; the close control and dirty marker
                        // below are flex-shrink-0 so they survive it.
                        div().min_w_0().truncate().child(label),
                    )
                    .when(dirty, |tab_view| {
                        // Unsaved state gets its own non-shrinking element so
                        // label truncation can never hide it.
                        tab_view.child(
                            div()
                                .flex_shrink_0()
                                .text_size(px(10.))
                                .text_color(text_color)
                                .child("•"),
                        )
                    })
                    .child(
                        div()
                            .ml_1()
                            .px_1()
                            .flex_shrink_0()
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
            }));

        div()
            .h(px(document_tab_band_height(self.tabs.len())))
            .border_b_1()
            .border_color(palette.border)
            .bg(palette.panel_bg)
            .flex()
            .child(document_bar)
            // Pinned action region outside the scroll container: the "+" must
            // stay reachable no matter how far the strip is scrolled.
            .child(
                div()
                    .flex_shrink_0()
                    .h_full()
                    .flex()
                    .items_end()
                    .pr_2()
                    .child(div().w(px(1.)).h(px(16.)).mb_2().mr_2().bg(palette.border))
                    .child(
                        // Trailing "+" opens a fresh empty tab (mirrors File → New Tab).
                        div()
                            .id("new-tab-button")
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
                    ),
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
            return self.move_visual_vertical_from_whitespace(direction, extend_selection, cx);
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
            && let Some(target) = snapshot.closest_source_on_line(line_index, preferred_x)
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

        let Some(first_target) = (match direction {
            VisualNavigationDirection::Up => block_index.checked_sub(1),
            VisualNavigationDirection::Down => (block_index + 1
                < self.active_tab().visual_list_blocks.len())
            .then_some(block_index + 1),
        }) else {
            return true;
        };

        // A blank-line `Whitespace` row is a first-class empty line: land on
        // an existing newline in that range instead of skipping it. Landing
        // is source-offset based (no layout snapshot), so it works even when
        // the row has not been measured yet. A further Up/Down from the gap
        // uses `move_visual_vertical_from_whitespace` because whitespace rows
        // do not register wrapped-line snapshots.
        if self
            .active_tab()
            .visual_list_blocks
            .get(first_target)
            .is_some_and(|block| matches!(block.kind, VisualBlockKind::Whitespace))
        {
            let range = self.active_tab().visual_list_blocks[first_target]
                .source_range
                .clone();
            let text = self.active_tab().document.text();
            let from_above = matches!(direction, VisualNavigationDirection::Down);
            let target = whitespace_navigation_offset(range, text, from_above);
            if extend_selection {
                self.select_to(target, cx);
            } else {
                self.move_to(target, cx);
            }
            let tab = self.active_tab_mut();
            tab.visual_preferred_x = Some(preferred_x);
            tab.visual_list.scroll_to_reveal_item(first_target);
            return true;
        }

        let version = self.active_tab().document.version();
        let pending = PendingVisualNavigation {
            document_version: version,
            target_block: first_target,
            direction,
            extend_selection,
            preferred_x,
        };
        let tab = self.active_tab_mut();
        tab.visual_preferred_x = Some(preferred_x);
        tab.pending_visual_navigation = Some(pending);
        tab.visual_list.scroll_to_reveal_item(first_target);
        cx.notify();
        true
    }

    /// Whitespace rows do not register wrapped-line snapshots. Walk covered
    /// newlines, then hand off to the adjacent visual block while keeping
    /// `preferred_x`.
    fn move_visual_vertical_from_whitespace(
        &mut self,
        direction: VisualNavigationDirection,
        extend_selection: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let cursor = self.cursor_offset();
        let Some(block_index) = visual_block_index_for_offset(
            &self.active_tab().visual_list_blocks,
            cursor,
            self.active_tab().document.text().len(),
        ) else {
            return false;
        };
        let Some(block) = self.active_tab().visual_list_blocks.get(block_index) else {
            return false;
        };
        if !matches!(block.kind, VisualBlockKind::Whitespace) {
            return false;
        }
        let range = block.source_range.clone();
        let text = self.active_tab().document.text().to_string();
        let line = whitespace_caret_line(range.clone(), cursor, &text);
        let line_count = whitespace_painted_line_count(range.clone(), &text);
        let preferred_x = self.active_tab().visual_preferred_x.unwrap_or(Pixels::ZERO);
        let adjacent_line = match direction {
            VisualNavigationDirection::Up => line.checked_sub(1),
            VisualNavigationDirection::Down => (line + 1 < line_count).then_some(line + 1),
        };
        if let Some(line) = adjacent_line {
            let target = whitespace_source_at_line(range, line, &text);
            if extend_selection {
                self.select_to(target, cx);
            } else {
                self.move_to(target, cx);
            }
            self.active_tab_mut().visual_preferred_x = Some(preferred_x);
            return true;
        }

        let Some(first_target) = (match direction {
            VisualNavigationDirection::Up => block_index.checked_sub(1),
            VisualNavigationDirection::Down => (block_index + 1
                < self.active_tab().visual_list_blocks.len())
            .then_some(block_index + 1),
        }) else {
            return true;
        };

        if self
            .active_tab()
            .visual_list_blocks
            .get(first_target)
            .is_some_and(|block| matches!(block.kind, VisualBlockKind::Whitespace))
        {
            let range = self.active_tab().visual_list_blocks[first_target]
                .source_range
                .clone();
            let from_above = matches!(direction, VisualNavigationDirection::Down);
            let target = whitespace_navigation_offset(range, &text, from_above);
            if extend_selection {
                self.select_to(target, cx);
            } else {
                self.move_to(target, cx);
            }
            let tab = self.active_tab_mut();
            tab.visual_preferred_x = Some(preferred_x);
            tab.visual_list.scroll_to_reveal_item(first_target);
            return true;
        }

        let version = self.active_tab().document.version();
        let pending = PendingVisualNavigation {
            document_version: version,
            target_block: first_target,
            direction,
            extend_selection,
            preferred_x,
        };
        let tab = self.active_tab_mut();
        tab.visual_preferred_x = Some(preferred_x);
        tab.pending_visual_navigation = Some(pending);
        tab.visual_list.scroll_to_reveal_item(first_target);
        cx.notify();
        true
    }

    fn visual_painted_line_boundary(&self, end: bool) -> Option<usize> {
        let (_, snapshot) = self.current_visual_navigation_snapshot()?;
        let line_index = snapshot.line_index_for_source(self.cursor_offset())?;
        snapshot.line_boundary_source(line_index, end)
    }

    /// Fallback navigation target for a callout title row: its marker line
    /// end, just inside the line. Title rows render no text runs, so the
    /// layout snapshot other blocks rely on never exists for them.
    fn callout_title_navigation_target(&self, block_index: usize) -> Option<usize> {
        let tab = self.active_tab();
        let block = tab.visual_list_blocks.get(block_index)?;
        if !matches!(block.kind, VisualBlockKind::CalloutTitle { .. }) {
            return None;
        }
        let range = &block.source_range;
        callout_marker_line_caret_target(tab.document.text(), range)
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
            // Whitespace rows and callout titles have no wrapped-line
            // snapshot. Park on an existing source offset instead of dropping
            // the pending move.
            if let Some(block) = self
                .active_tab()
                .visual_list_blocks
                .get(pending.target_block)
                .filter(|block| matches!(block.kind, VisualBlockKind::Whitespace))
            {
                let from_above = matches!(pending.direction, VisualNavigationDirection::Down);
                let target = whitespace_navigation_offset(
                    block.source_range.clone(),
                    self.active_tab().document.text(),
                    from_above,
                );
                self.active_tab_mut().pending_visual_navigation = None;
                self.active_tab_mut().visual_preferred_x = Some(pending.preferred_x);
                if pending.extend_selection {
                    self.select_to(target, cx);
                } else {
                    self.move_to(target, cx);
                }
                return;
            }
            // A callout title row has no rendered text runs, so no layout
            // snapshot exists to land on. Park the caret just inside the
            // marker line's end instead — the reveal projection then shows
            // the authored `> [!NOTE]` verbatim.
            if let Some(target) = self.callout_title_navigation_target(pending.target_block) {
                self.active_tab_mut().pending_visual_navigation = None;
                self.active_tab_mut().visual_preferred_x = Some(pending.preferred_x);
                if pending.extend_selection {
                    self.select_to(target, cx);
                } else {
                    self.move_to(target, cx);
                }
            }
            return;
        };
        let line_index = match pending.direction {
            VisualNavigationDirection::Up => snapshot.lines.len().checked_sub(1),
            VisualNavigationDirection::Down => (!snapshot.lines.is_empty()).then_some(0),
        };
        let Some(target) = line_index
            .and_then(|index| snapshot.closest_source_on_line(index, pending.preferred_x))
        else {
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

    /// Opens a URL or jumps to a footnote definition from a Visual Edit icon.
    pub(super) fn activate_visual_navigation(
        &mut self,
        target: &VisualNavigationTarget,
        cx: &mut Context<Self>,
    ) {
        match target {
            VisualNavigationTarget::Url(url) => {
                if !url.trim().is_empty() {
                    cx.open_url(url);
                }
            }
            VisualNavigationTarget::Footnote { label } => {
                let blocks = self.active_tab().document.visual_blocks_shared();
                let Some(block_index) = blocks.iter().position(|block| {
                    matches!(
                        &block.kind,
                        VisualBlockKind::FootnoteDefinition { label: def }
                            if def == label
                    )
                }) else {
                    return;
                };
                let offset = blocks[block_index].source_range.start;
                self.move_to(offset, cx);
                self.active_tab_mut()
                    .visual_list
                    .scroll_to_reveal_item(block_index);
                cx.notify();
            }
        }
    }

    pub(super) fn move_to_visual_editor_target(
        &mut self,
        range: Range<usize>,
        cx: &mut Context<Self>,
    ) {
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

/// Hover tooltip for a document tab: restores the full title (and the full
/// path for file-backed tabs) that strip truncation may have hidden.
struct TabTooltip {
    palette: ThemePalette,
    title: SharedString,
    path: Option<SharedString>,
}

impl Render for TabTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let palette = self.palette;
        div()
            .py_1()
            .px_2()
            .rounded_md()
            .border_1()
            .border_color(palette.border)
            .bg(palette.panel_bg)
            .text_size(px(12.))
            .text_color(palette.active_text)
            .child(self.title.clone())
            .when_some(self.path.clone(), |tooltip, path| {
                tooltip.child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.muted)
                        .child(path),
                )
            })
    }
}
