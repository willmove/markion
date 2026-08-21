use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OpenPathIntent {
    ReplaceActive,
    OpenInNewTab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExternalDropIntent {
    OpenDocument,
    ImportImage,
    Ignore,
}

pub(super) fn classify_external_drop_path(path: &Path) -> ExternalDropIntent {
    if is_markdown_path(path) {
        ExternalDropIntent::OpenDocument
    } else if image_extension_supported(path) {
        ExternalDropIntent::ImportImage
    } else {
        ExternalDropIntent::Ignore
    }
}

impl MarkionApp {
    pub(super) fn set_view_mode(&mut self, view_mode: ViewMode, cx: &mut Context<Self>) {
        assign_view_mode(&mut self.view_mode, view_mode);
        if let Some(tab) = self.active_tab_mut().document_tab_mut() {
            tab.sync_scroll_state.reset();
        }
        self.slash_commands = None;
        self.dismissed_slash_query = None;
        self.dismiss_visual_block_menu();
        self.active_tab_mut().clear_visual_caret_affinity();
        self.active_tab_mut().clear_visual_navigation_intent();
        self.active_tab_mut().finish_undo_capture();
        self.active_tab_mut().marked_range = None;
        if matches!(view_mode, ViewMode::VisualEdit) {
            let tab = self.active_tab_mut();
            tab.visual_cursor_reveal_pending = true;
            tab.visual_caret_bounds = None;
        }
        self.status = self.view_mode_status().into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn view_mode_status(&self) -> &'static str {
        t(self.language, view_mode_status_message(self.view_mode))
    }

    pub(super) fn toggle_view_mode(
        &mut self,
        _: &ToggleViewMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_view_mode(self.view_mode.next(), cx);
    }

    pub(super) fn set_edit_mode(
        &mut self,
        _: &SetEditMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_view_mode(ViewMode::Edit, cx);
    }

    pub(super) fn set_visual_edit_mode(
        &mut self,
        _: &SetVisualEditMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_view_mode(ViewMode::VisualEdit, cx);
    }

    pub(super) fn set_split_preview_mode(
        &mut self,
        _: &SetSplitPreviewMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_view_mode(ViewMode::Split, cx);
    }

    pub(super) fn set_read_mode(
        &mut self,
        _: &SetReadMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_view_mode(ViewMode::Read, cx);
    }

    pub(super) fn toggle_sidebar(
        &mut self,
        _: &ToggleSidebar,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar_visible = !self.sidebar_visible;
        // No lazy scan here: on the welcome document the Files tab shows an
        // empty-state placeholder by design (the tree has no chosen root until
        // a real file is opened). See `update_workspace_root_from_document`.
        self.status = t(
            self.language,
            if self.sidebar_visible {
                Msg::StatusSidebarShown
            } else {
                Msg::StatusSidebarHidden
            },
        )
        .into();
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    /// Switch the sidebar to a tab and persist the choice. The file tree is
    /// not lazily scanned here: on the welcome document the Files tab shows an
    /// empty-state placeholder by design.
    pub(super) fn set_sidebar_tab(&mut self, tab: SidebarTab, cx: &mut Context<Self>) {
        if self.sidebar_tab == tab {
            return;
        }
        self.sidebar_tab = tab;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn select_preferences_sidebar_tab(
        &mut self,
        tab: SidebarTab,
        cx: &mut Context<Self>,
    ) {
        let changed = !self.sidebar_visible || self.sidebar_tab != tab;
        self.sidebar_visible = true;
        self.sidebar_tab = tab;
        if tab == SidebarTab::Files && self.file_tree.is_none() {
            self.refresh_file_tree(cx);
        }
        self.status = t(
            self.language,
            match tab {
                SidebarTab::Files => Msg::StatusFileTreeShown,
                SidebarTab::Outline => Msg::StatusOutlineShown,
            },
        )
        .into();
        if changed {
            self.persist_preferences();
        }
        cx.notify();
    }

    /// Drag handler for the editor/preview divider. The event bounds are the
    /// main content row, so the cursor's x offset within them maps directly to
    /// the editor's share of the width.
    pub(super) fn on_editor_split_drag(
        &mut self,
        event: &DragMoveEvent<DraggedEditorSplitHandle>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bounds = event.bounds;
        let left = f32::from(bounds.left());
        let width = f32::from(bounds.right() - bounds.left());
        if width <= 0. {
            return;
        }
        let cursor_x = f32::from(event.event.position.x);
        let ratio =
            ((cursor_x - left) / width).clamp(EDITOR_SPLIT_RATIO_MIN, EDITOR_SPLIT_RATIO_MAX);
        if (ratio - self.editor_split_ratio).abs() > f32::EPSILON {
            self.editor_split_ratio = ratio;
            cx.notify();
        }
    }

    pub(super) fn on_editor_split_drop(
        &mut self,
        _: &DraggedEditorSplitHandle,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The ratio is updated continuously during drag; drop just finalizes.
        cx.notify();
    }

    /// Drag handler for the sidebar divider. The sidebar starts at the window's
    /// left edge, so the cursor x is the new sidebar width (clamped).
    pub(super) fn on_sidebar_resize_drag(
        &mut self,
        event: &DragMoveEvent<DraggedSidebarHandle>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_width =
            f32::from(event.event.position.x).clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        if (new_width - self.sidebar_width).abs() > f32::EPSILON {
            self.sidebar_width = new_width;
            cx.notify();
        }
    }

    pub(super) fn toggle_outline(
        &mut self,
        _: &ToggleOutline,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Smart toggle: if the sidebar is already showing the Outline tab,
        // hide the sidebar; otherwise reveal it and switch to Outline.
        if self.sidebar_visible && self.sidebar_tab == SidebarTab::Outline {
            self.sidebar_visible = false;
            self.status = t(self.language, Msg::StatusSidebarHidden).into();
        } else {
            self.sidebar_visible = true;
            self.set_sidebar_tab(SidebarTab::Outline, cx);
            self.status = t(self.language, Msg::StatusOutlineShown).into();
        }
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn toggle_file_tree(
        &mut self,
        _: &ToggleFileTree,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Smart toggle: if the sidebar is already showing the Files tab, hide
        // the sidebar; otherwise reveal it and switch to Files.
        if self.sidebar_visible && self.sidebar_tab == SidebarTab::Files {
            self.sidebar_visible = false;
            self.status = t(self.language, Msg::StatusSidebarHidden).into();
        } else {
            self.sidebar_visible = true;
            self.set_sidebar_tab(SidebarTab::Files, cx);
            if self.file_tree.is_none() {
                self.refresh_file_tree(cx);
            }
            self.status = t(self.language, Msg::StatusFileTreeShown).into();
        }
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn focus_file_tree_search(
        &mut self,
        _: &FocusFileTreeSearch,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The filter box only exists on the Files tab, so make sure the sidebar
        // is visible and showing Files before focusing it.
        self.sidebar_visible = true;
        self.set_sidebar_tab(SidebarTab::Files, cx);
        self.file_tree_query_focused = true;
        self.search_focus = None;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        self.status = t(self.language, Msg::StatusFilteringFiles).into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn clear_file_tree_search(
        &mut self,
        _: &ClearFileTreeSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.slash_commands.is_some() {
            self.dismissed_slash_query = self.slash_commands.take().map(|state| state.query);
            cx.notify();
            return;
        }
        if self.block_menu.is_some() {
            self.close_visual_block_menu(cx);
            return;
        }
        if self.preferences_panel_open {
            self.close_preferences_panel(window, cx);
            return;
        }
        // Escape first cancels any open name prompt (create/rename); only if
        // none is open does it fall back to clearing the file-tree filter.
        if self.pending_name_input.is_some() {
            self.pending_name_input = None;
            self.input_marked_len = 0;
            self.active_menu = None;
            self.file_tree_context_menu = None;
            self.status = t(self.language, Msg::StatusCanceled).into();
            cx.notify();
            return;
        }
        if self.search_visible {
            self.close_search_overlay(cx);
            return;
        }
        self.file_tree_query.clear();
        self.file_tree_query_focused = false;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        self.status = self.file_tree_summary().into();
        self.active_menu = None;
        self.file_tree_context_menu = None;
        cx.notify();
    }

    pub(super) fn refresh_file_tree_action(
        &mut self,
        _: &RefreshFileTree,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.refresh_file_tree(cx);
        self.status = t(self.language, Msg::StatusFileTreeRefreshed).into();
        self.active_menu = None;
        self.pending_name_input = None;
        self.file_tree_context_menu = None;
        cx.notify();
    }

    pub(super) fn open_file_in_new_tab_from_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if let Err(error) = self.open_supported_path(path, OpenPathIntent::OpenInNewTab, cx) {
            self.status = self.trf(Msg::StatusOpenFailed, &[&error]);
            cx.notify();
        }
    }

    /// Tab target for a non-explicit open (File → Open, file-tree click,
    /// drag-drop, Open Recent), resolved from the persisted
    /// open-in-current-tab preference. While the preference is on, only a
    /// safe-to-replace active tab (image, untitled, or clean document) is
    /// replaced in place; a dirty editable tab diverts the open to a new tab
    /// so gestures never prompt and never discard unsaved work.
    pub(super) fn default_open_intent(&self) -> OpenPathIntent {
        if !self.open_in_current_tab || !self.active_tab().is_safe_to_replace() {
            OpenPathIntent::OpenInNewTab
        } else {
            OpenPathIntent::ReplaceActive
        }
    }

    pub(super) fn open_supported_path(
        &mut self,
        path: PathBuf,
        intent: OpenPathIntent,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let display_path = path.display().to_string();
        if self.focus_existing_tab_for_path(&path, cx) {
            self.record_recent_path(&path);
            self.active_menu = None;
            self.status = self.trf(Msg::StatusOpened, &[&display_path]);
            cx.notify();
            return Ok(());
        }

        if image_extension_supported(&path) {
            match intent {
                OpenPathIntent::ReplaceActive => {
                    self.replace_active_tab_with_image(path.clone(), cx)
                }
                OpenPathIntent::OpenInNewTab => self.open_image_in_new_tab(path.clone(), cx),
            }
        } else if is_markdown_path(&path) || is_text_path(&path) {
            let document = MarkdownDocument::open(&path).map_err(|error| error.to_string())?;
            match intent {
                OpenPathIntent::ReplaceActive => self.replace_active_tab(document, cx),
                OpenPathIntent::OpenInNewTab => self.open_in_new_tab(document, cx),
            }
        } else {
            return Err(self
                .trf(Msg::StatusUnsupportedFile, &[&display_path])
                .to_string());
        }

        self.update_workspace_root_from_document(cx);
        self.record_recent_path(&path);
        self.active_menu = None;
        self.status = self.trf(Msg::StatusOpened, &[&display_path]);
        cx.notify();
        Ok(())
    }

    /// Handle files dragged in from the OS file manager and dropped onto the
    /// editor or preview pane. GPUI publishes the dragged paths as an
    /// [`ExternalPaths`] drag value (its platform layer turns an OS file drag
    /// into an internal drag on `Entered`, then fires `on_drop::<ExternalPaths>`
    /// on `Submit`); this handler is registered on both pane `div`s.
    ///
    /// The first dropped Markdown file follows the default open-target
    /// preference (replacing a safe-to-replace active tab when the
    /// open-in-current-tab preference is on); every subsequent one opens in
    /// its own new tab, so a multi-file drop cannot strand earlier files by
    /// thrashing replace. Non-Markdown files and directories are skipped
    /// silently; if nothing was opened the status bar is left untouched.
    ///
    /// A multi-file drop arrives as a single event with all paths bundled in
    /// one `ExternalPaths`, so this loops rather than taking the first path.
    pub(super) fn handle_external_drop(
        &mut self,
        dragged: &ExternalPaths,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut images = Vec::new();
        let mut documents = Vec::new();
        for path in dragged.paths() {
            if !path.is_file() {
                continue;
            }
            match classify_external_drop_path(path) {
                ExternalDropIntent::OpenDocument => documents.push(path.clone()),
                ExternalDropIntent::ImportImage => match fs::read(path) {
                    Ok(bytes) => images.push(PendingImageInput {
                        stem: path
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .unwrap_or("image")
                            .to_string(),
                        extension: path
                            .extension()
                            .and_then(|extension| extension.to_str())
                            .unwrap_or("png")
                            .to_string(),
                        bytes,
                    }),
                    Err(err) => {
                        self.status = p0_tf(
                            self.language,
                            P0Msg::DroppedImageReadFailed,
                            &[&err.to_string()],
                        )
                        .into();
                    }
                },
                ExternalDropIntent::Ignore => {}
            }
        }
        self.open_dropped_documents(&documents, cx);
        if !images.is_empty() {
            self.request_image_import(images, window, cx);
        }
    }

    /// Open a batch of dropped Markdown paths in order: the first follows the
    /// default open-target rule and every subsequent one appends a new tab,
    /// so a multi-file drop cannot strand earlier files by thrashing replace.
    pub(super) fn open_dropped_documents(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let mut opened_any = false;
        for path in paths {
            let intent = if opened_any {
                OpenPathIntent::OpenInNewTab
            } else {
                self.default_open_intent()
            };
            opened_any = true;
            if let Err(error) = self.open_supported_path(path.clone(), intent, cx) {
                self.status = self.trf(Msg::StatusOpenFailed, &[&error]);
                cx.notify();
            }
        }
    }

    pub(super) fn show_file_tree_context_menu(
        &mut self,
        target: FileTreeContextTarget,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.active_menu = None;
        self.dismiss_visual_block_menu();
        self.file_tree_query_focused = false;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        self.tab_context_menu = None;
        match &target {
            FileTreeContextTarget::Workspace => self.selected_tree_path = None,
            FileTreeContextTarget::Directory(path) | FileTreeContextTarget::File(path) => {
                self.selected_tree_path = Some(path.clone());
            }
        }
        self.file_tree_context_menu = Some(FileTreeContextMenu { target, position });
        cx.notify();
    }

    /// Open the inline name editor for a create/rename file-tree action. The
    /// editor captures keystrokes into its buffer via the redirected-text-input
    /// path; Enter commits (`confirm_pending_name`), Escape cancels, and a
    /// click outside the field commits through the same pipeline (Explorer
    /// semantics). The initial selection pre-selects the base name for a
    /// rename (extension preserved) or the whole prefilled name otherwise.
    pub(super) fn open_name_prompt(
        &mut self,
        kind: PendingNameKind,
        parent: PathBuf,
        target: Option<PathBuf>,
        prefill: &str,
        cx: &mut Context<Self>,
    ) {
        // Close any other transient focus so the editor owns input routing.
        self.active_menu = None;
        self.dismiss_visual_block_menu();
        self.file_tree_context_menu = None;
        self.tab_context_menu = None;
        self.file_tree_query_focused = false;
        self.search_focus = None;
        self.input_marked_len = 0;
        self.name_editor_click_away = false;
        self.pending_name_input = Some(PendingNameInput::new(kind, parent, target, prefill));
        self.status = t(self.language, Msg::StatusNamingEntry).into();
        cx.notify();
    }

    pub(super) fn handle_file_tree_context_action(
        &mut self,
        action: FileTreeContextAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self
            .file_tree_context_menu
            .as_ref()
            .map(|menu| menu.target.clone())
        else {
            return;
        };
        self.file_tree_context_menu = None;
        match action {
            FileTreeContextAction::Open => {
                if let FileTreeContextTarget::File(path) = target {
                    self.open_tree_file(path, window, cx);
                }
            }
            FileTreeContextAction::OpenInNewTab => {
                if let FileTreeContextTarget::File(path) = target {
                    self.open_file_in_new_tab_from_path(path, cx);
                }
            }
            FileTreeContextAction::CreateFile => {
                let parent = match target {
                    FileTreeContextTarget::Directory(path) => path,
                    FileTreeContextTarget::File(path) => path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.workspace_root.clone()),
                    FileTreeContextTarget::Workspace => self.workspace_root.clone(),
                };
                self.selected_tree_path = Some(parent.clone());
                self.open_name_prompt(PendingNameKind::CreateFile, parent, None, "untitled.md", cx);
            }
            FileTreeContextAction::CreateFolder => {
                let parent = match target {
                    FileTreeContextTarget::Directory(path) => path,
                    FileTreeContextTarget::File(path) => path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.workspace_root.clone()),
                    FileTreeContextTarget::Workspace => self.workspace_root.clone(),
                };
                self.selected_tree_path = Some(parent.clone());
                self.open_name_prompt(
                    PendingNameKind::CreateFolder,
                    parent,
                    None,
                    "New Folder",
                    cx,
                );
            }
            FileTreeContextAction::Rename => {
                let Some(path) = (match target {
                    FileTreeContextTarget::Directory(path) | FileTreeContextTarget::File(path) => {
                        Some(path)
                    }
                    FileTreeContextTarget::Workspace => None,
                }) else {
                    return;
                };
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
            FileTreeContextAction::Delete => {
                self.selected_tree_path = match target {
                    FileTreeContextTarget::Directory(path) | FileTreeContextTarget::File(path) => {
                        Some(path)
                    }
                    FileTreeContextTarget::Workspace => None,
                };
                self.delete_tree_entry(&DeleteTreeEntry, window, cx);
            }
            FileTreeContextAction::ShowInFileManager => {
                let path = target.path(&self.workspace_root);
                match reveal_in_system_file_manager(
                    &path,
                    target.kind() == FileTreeContextTargetKind::File,
                ) {
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
            FileTreeContextAction::Refresh => {
                self.refresh_file_tree_action(&RefreshFileTree, window, cx);
            }
            FileTreeContextAction::FilterFiles => {
                self.sidebar_visible = true;
                self.set_sidebar_tab(SidebarTab::Files, cx);
                self.file_tree_query_focused = true;
                self.search_focus = None;
                self.input_marked_len = 0;
                self.status = t(self.language, Msg::StatusFilteringFiles).into();
                cx.notify();
            }
        }
    }

    pub(super) fn selected_tree_parent(&self) -> PathBuf {
        self.selected_tree_path
            .as_ref()
            .map(|path| {
                if path.is_dir() {
                    path.clone()
                } else {
                    path.parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.workspace_root.clone())
                }
            })
            .unwrap_or_else(|| self.workspace_root.clone())
    }

    pub(super) fn create_tree_file(
        &mut self,
        _: &CreateTreeFile,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Open the inline name prompt against the selected entry's parent (or
        // the workspace root). The actual file is created on Enter via
        // `confirm_pending_name`.
        let parent = self.selected_tree_parent();
        self.selected_tree_path = Some(parent.clone());
        self.open_name_prompt(PendingNameKind::CreateFile, parent, None, "untitled.md", cx);
    }

    pub(super) fn create_tree_folder(
        &mut self,
        _: &CreateTreeFolder,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let parent = self.selected_tree_parent();
        self.selected_tree_path = Some(parent.clone());
        self.open_name_prompt(
            PendingNameKind::CreateFolder,
            parent,
            None,
            "New Folder",
            cx,
        );
    }

    pub(super) fn rename_tree_entry(
        &mut self,
        _: &RenameTreeEntry,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = self.selected_tree_path.clone() else {
            self.status = t(self.language, Msg::StatusSelectTreeEntryFirst).into();
            cx.notify();
            return;
        };
        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.workspace_root.clone());
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        self.open_name_prompt(PendingNameKind::Rename, parent, Some(path), &file_name, cx);
    }

    /// Commit the inline name prompt: create/rename the entry using the typed
    /// buffer. An empty buffer is rejected without touching the filesystem.
    pub(super) fn confirm_pending_name(
        &mut self,
        _: &ConfirmPendingName,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pending) = self.pending_name_input.take() else {
            return;
        };
        self.input_marked_len = 0;
        self.active_menu = None;
        self.file_tree_context_menu = None;

        let name = pending.buffer.trim();
        if name.is_empty() {
            self.status = t(self.language, Msg::StatusNameRequired).into();
            // Restore the prompt so the user can try again.
            self.pending_name_input = Some(pending);
            cx.notify();
            return;
        }

        match pending.kind {
            PendingNameKind::CreateFile => {
                let result = self
                    .file_tree
                    .get_or_insert_with(|| {
                        FileTree::scan_with_options(&self.workspace_root, self.show_hidden_files)
                            .unwrap_or(FileTree {
                                root: self.workspace_root.clone(),
                                entries: Vec::new(),
                                show_hidden: self.show_hidden_files,
                            })
                    })
                    .create_unique_file(&pending.parent, name);
                match result {
                    Ok(path) => {
                        self.selected_tree_path = Some(path.clone());
                        self.status = self.trf(Msg::StatusCreated, &[&path.display().to_string()]);
                    }
                    Err(err) => {
                        self.status = self.trf(Msg::StatusCreateFileFailed, &[&err.to_string()]);
                    }
                }
            }
            PendingNameKind::CreateFolder => {
                let result = self
                    .file_tree
                    .get_or_insert_with(|| {
                        FileTree::scan_with_options(&self.workspace_root, self.show_hidden_files)
                            .unwrap_or(FileTree {
                                root: self.workspace_root.clone(),
                                entries: Vec::new(),
                                show_hidden: self.show_hidden_files,
                            })
                    })
                    .create_unique_directory(&pending.parent, name);
                match result {
                    Ok(path) => {
                        self.selected_tree_path = Some(path.clone());
                        self.status = self.trf(Msg::StatusCreated, &[&path.display().to_string()]);
                    }
                    Err(err) => {
                        self.status = self.trf(Msg::StatusCreateFolderFailed, &[&err.to_string()]);
                    }
                }
            }
            PendingNameKind::Rename => {
                let Some(target) = pending.target.clone() else {
                    self.status = t(self.language, Msg::StatusRenameFailed).into();
                    cx.notify();
                    return;
                };
                // Refuse renaming the active document while it is dirty; the
                // user should save first to avoid losing unsaved edits.
                let needs_save = self.active_tab().path() == Some(target.as_path())
                    && self.active_tab().is_dirty();
                if needs_save {
                    self.status = t(self.language, Msg::StatusSaveBeforeRename).into();
                    cx.notify();
                    return;
                }
                let result = self
                    .file_tree
                    .as_mut()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file tree unavailable"))
                    .and_then(|tree| tree.rename_unique(&target, name));
                match result {
                    Ok(new_path) => {
                        // Keep any open content tab attached to its renamed
                        // filesystem path. Image claims use path identity, so
                        // release the old key before replacing it.
                        let matching: Vec<usize> = self
                            .tabs
                            .iter()
                            .enumerate()
                            .filter_map(|(i, tab)| {
                                (tab.path() == Some(target.as_path())).then_some(i)
                            })
                            .collect();
                        for index in matching {
                            self.release_tab_image_claims(index, cx);
                            match &mut self.tabs[index] {
                                WorkspaceTab::Document(tab) => {
                                    if let Ok(document) = MarkdownDocument::open(&new_path) {
                                        tab.document = document;
                                    }
                                }
                                WorkspaceTab::Image(image) => {
                                    image.path = new_path.clone();
                                    image.key = PreviewImageKey::from_local_path(&new_path);
                                }
                            }
                        }
                        self.selected_tree_path = Some(new_path.clone());
                        self.status =
                            self.trf(Msg::StatusRenamedTo, &[&new_path.display().to_string()]);
                    }
                    Err(err) => {
                        self.status = self.trf(Msg::StatusRenameFailed, &[&err.to_string()]);
                    }
                }
            }
        }
        cx.notify();
    }

    pub(super) fn delete_tree_entry(
        &mut self,
        _: &DeleteTreeEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = self.selected_tree_path.clone() else {
            self.status = t(self.language, Msg::StatusSelectTreeEntryFirst).into();
            cx.notify();
            return;
        };

        let detail = tf(
            self.language,
            Msg::DialogDeleteDetail,
            &[&path.display().to_string()],
        );
        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogDeleteTitle),
            Some(&detail),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonDelete)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );
        // A non-empty folder is removed recursively, which is destructive and
        // not undoable, so prepare a second confirmation that specifically
        // calls out that the folder *and all of its contents* will be removed.
        // It must not be created until the first prompt has been accepted:
        // GPUI custom prompts are not re-entrant and a second immediate call
        // replaces the first prompt before the user can answer it.
        let recursive_prompt = if path.is_dir() && dir_is_non_empty(&path) {
            let detail = tf(
                self.language,
                Msg::DialogDeleteFolderRecursiveDetail,
                &[&path.display().to_string()],
            );
            Some((
                self.tr(Msg::DialogDeleteFolderRecursiveTitle).to_string(),
                detail,
                self.tr(Msg::DialogButtonDelete).to_string(),
                self.tr(Msg::DialogButtonCancel).to_string(),
            ))
        } else {
            None
        };
        let window_handle = window.window_handle();
        self.active_menu = None;
        self.status = t(self.language, Msg::StatusWaitingDeleteConfirm).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let recursive_confirmed = match (confirmed, recursive_prompt) {
                (true, Some((title, detail, delete_label, cancel_label))) => {
                    let second = cx.update_window(window_handle, |_, window, cx| {
                        window.prompt(
                            PromptLevel::Warning,
                            &title,
                            Some(&detail),
                            &[
                                PromptButton::ok(delete_label),
                                PromptButton::cancel(cancel_label),
                            ],
                            cx,
                        )
                    });
                    match second {
                        Ok(second) => matches!(second.await, Ok(0)),
                        Err(_) => false,
                    }
                }
                (true, None) => true,
                (false, _) => false,
            };
            let _ = this.update(cx, |app, cx| {
                if !recursive_confirmed {
                    app.status = t(app.language, Msg::StatusDeleteCanceled).into();
                    cx.notify();
                    return;
                }

                let was_active = app.active_tab().path() == Some(path.as_path());
                let result = app
                    .file_tree
                    .as_mut()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file tree unavailable"))
                    .and_then(|tree| tree.delete(&path));
                match result {
                    Ok(()) => {
                        app.selected_tree_path = None;
                        // Any content tab for the deleted file (or a descendant
                        // of a deleted folder) becomes a fresh document. This
                        // also releases an image viewer's cache claim.
                        let affected: Vec<usize> = app
                            .tabs
                            .iter()
                            .enumerate()
                            .filter_map(|(index, tab)| {
                                tab.path()
                                    .is_some_and(|p| p == path.as_path() || p.starts_with(&path))
                                    .then_some(index)
                            })
                            .collect();
                        for index in affected {
                            app.release_tab_image_claims(index, cx);
                            app.tabs[index] = app.editor_tab_for_document(MarkdownDocument::new());
                        }
                        let _ = was_active;
                        app.status = app.trf(Msg::StatusDeleted, &[&path.display().to_string()]);
                    }
                    Err(err) => app.status = app.trf(Msg::StatusDeleteFailed, &[&err.to_string()]),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
