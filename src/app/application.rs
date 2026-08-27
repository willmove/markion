use super::*;

impl MarkionApp {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        let document = MarkdownDocument::from_text(markion::DEFAULT_WELCOME_MARKDOWN);
        let workspace_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        // Defer the file tree scan out of the window-creation path. Scanning the
        // workspace synchronously here freezes the first frame (and the whole UI)
        // when the working directory is large. We start with no tree and let the
        // background scan (scheduled by the caller) populate it once ready.
        let file_tree = None;
        let preferences_path = default_preferences_path();
        // Unit tests must not read/write the developer's real config.toml.
        let preferences = if cfg!(test) {
            AppPreferences::default()
        } else {
            load_app_preferences(&preferences_path).unwrap_or_default()
        };
        let session_path = default_session_path();
        // Unit tests must not read/write the developer's real session.toml.
        let session = if cfg!(test) {
            SessionState::default()
        } else {
            load_session_state(&session_path).unwrap_or_default()
        };
        let typography = DocumentTypographyMetrics::new(
            preferences.editor_font_size,
            preferences.rendered_font_size,
            preferences.paragraph_spacing,
        );
        let mut initial_tab = EditorTab::new(document);
        initial_tab.line_height = px(typography.editor_line_height);
        let themes_dir = default_themes_dir();
        let custom_themes = list_theme_definitions(&themes_dir).unwrap_or_default();
        let custom_theme = preferences
            .custom_theme
            .as_deref()
            .and_then(|name| custom_themes.iter().find(|theme| theme.name == name))
            .cloned();
        // Resolve the active theme by name. Custom-theme names take precedence
        // (matching the pre-panel behaviour), otherwise the plain `theme` name
        // is used. Unknown names fall back to Paper.
        let selected_theme_name = custom_theme
            .as_ref()
            .map(|theme| theme.name.clone())
            .or_else(|| {
                let name = preferences.theme.trim();
                (!name.is_empty()).then(|| name.to_string())
            })
            .unwrap_or_else(|| "Paper".to_string());
        let app = Self {
            tabs: vec![initial_tab],
            active_tab: 0,
            focus_handle: cx.focus_handle(),
            active_menu: None,
            open_recent_submenu_open: false,
            about_dialog_open: false,
            status: t(Language::default(), Msg::StatusReady).into(),
            publishing_service: None,
            browser_launcher: Arc::new(publishing::DefaultBrowserLauncher),
            git_branch_state: GitBranchState::default(),
            confirming_close: false,
            allow_close: false,
            preferences_path,
            session_path,
            session,
            theme: AppTheme::from_name(&preferences.theme).unwrap_or(AppTheme::Paper),
            custom_theme,
            custom_themes,
            themes_dir,
            selected_theme_name,
            preferences_panel_open: false,
            preferences_tab: PreferencesTab::default(),
            preferences_panel_focus: cx.focus_handle(),
            preferences_general_scroll: ScrollHandle::new(),
            preferences_categories_scroll: ScrollHandle::new(),
            preferences_actions_scroll: ScrollHandle::new(),
            preferences_export_scroll: ScrollHandle::new(),
            pandoc_available_cached: None,
            shortcut_platform: ShortcutPlatform::current(),
            shortcut_category: ShortcutCategory::Files,
            shortcut_overrides: sanitized_shortcut_overrides(&preferences.shortcut_overrides),
            shortcut_capture: None,
            focus_mode: preferences.focus_mode,
            typewriter_mode: preferences.typewriter_mode,
            code_line_numbers: preferences.code_line_numbers,
            preview_adaptive_width: preferences.preview_adaptive_width,
            editor_font_size: preferences.editor_font_size,
            rendered_font_size: preferences.rendered_font_size,
            paragraph_spacing: preferences.paragraph_spacing,
            editor_font_family: preferences.editor_font_family,
            rendered_font_family: preferences.rendered_font_family,
            code_font_family: preferences.code_font_family,
            resolved_font_families: ResolvedFontFamilies::default(),
            font_picker: None,
            installed_font_names: Vec::new(),
            heading_menu_max_level: preferences.heading_menu_max_level,
            sync_scroll: preferences.sync_scroll,
            show_hidden_files: preferences.show_hidden_files,
            open_in_current_tab: preferences.open_in_current_tab,
            language: Language::from_code(&preferences.language),
            check_for_updates_on_startup: preferences.check_for_updates_on_startup,
            last_update_check: preferences.last_update_check,
            view_mode: ViewMode::default_mode(),
            workspace_root,
            editor_split_ratio: 0.5,
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
            file_tree,
            sidebar_visible: preferences.sidebar_visible,
            sidebar_tab: preferences.sidebar_tab,
            file_tree_query: String::new(),
            file_tree_query_focused: false,
            file_tree_scroll: ScrollHandle::new(),
            outline_scroll: ScrollHandle::new(),
            tab_bar_scroll: ScrollHandle::new(),
            #[cfg(test)]
            last_tab_strip_reveal: None,
            input_marked_len: 0,
            selected_tree_path: None,
            collapsed_tree_paths: HashSet::new(),
            file_tree_needs_initial_collapse: false,
            file_tree_context_menu: None,
            preview_context_menu: None,
            tab_context_menu: None,
            pending_name_input: None,
            name_editor_click_away: false,
            pending_image_import: None,
            link_editor: None,
            recovery_manager: None,
            slash_commands: None,
            dismissed_slash_query: None,
            block_menu: None,
            search_visible: false,
            replace_visible: false,
            search_form: SearchPanelForm::Find,
            search_query: SearchFieldState::default(),
            replace_text: SearchFieldState::default(),
            search_case_sensitive: false,
            search_regex: false,
            search_focus: None,
            search_control_focus: None,
            search_matches: Vec::new(),
            current_search_index: None,
            search_result: SearchResultState::Idle,
            search_generation: None,
            search_field_bounds: [None, None],
            pane_scrollbar_drag: None,
            auto_save_preferences: preferences.auto_save,
            export_preferences: preferences.export.clone(),
            recovery_dir: default_recovery_dir(),
            highlight_cache: RefCell::new(HashMap::new()),
            diagram_cache: DiagramCache::new(DIAGRAM_CACHE_CAPACITY),
            preview_image_cache: PreviewImageCache::new(PREVIEW_IMAGE_CACHE_CAPACITY),
            math_cache: MathCache::new(MATH_CACHE_CAPACITY),
        };
        // Resolve per-plane font families once from the loaded preferences
        // and the initial theme before the first render.
        let mut app = app;
        app.recompute_resolved_font_families();
        app
    }

    /// The currently active tab (read access).
    ///
    /// `active_tab` is clamped to `tabs.len().saturating_sub(1)` before indexing
    /// so a transiently-out-of-range index (e.g. right after a tab close, before
    /// the next render updates the tab-bar closures) cannot panic. This is a
    /// defence-in-depth: the close/switch handlers also keep the index valid,
    /// but tab-bar click closures capture an `index` at render time that can be
    /// stale by the time they fire.
    pub(super) fn active_tab(&self) -> &EditorTab {
        let idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
        &self.tabs[idx]
    }

    /// The currently active tab (mutable access). See [`active_tab`](Self::active_tab)
    /// for the clamping rationale.
    pub(super) fn active_tab_mut(&mut self) -> &mut EditorTab {
        let idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
        &mut self.tabs[idx]
    }

    pub(super) fn focus_existing_tab_for_path(
        &mut self,
        path: &Path,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(index) = find_tab_with_document_path(&self.tabs, path) else {
            return false;
        };
        self.switch_active_tab(index, cx);
        self.update_workspace_root_from_document(cx);
        true
    }

    /// Switch the active tab index and clear preview selection on the newly
    /// active tab's sibling context (each tab keeps its own selection, but we
    /// still refresh search / notify so the UI settles on the new tab).
    pub(super) fn switch_active_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        let previous = self.active_tab;
        // Selecting in another tab's preview must not leave a drag in progress
        // on the previous tab; clear the drag flag on all tabs for safety.
        for tab in &mut self.tabs {
            if tab.is_document() {
                tab.preview_is_selecting = false;
                tab.clear_visual_caret_affinity();
                tab.finish_undo_capture();
                tab.marked_range = None;
            }
        }
        if previous != index {
            if self.tabs[previous].is_document() && self.tabs[index].is_image() {
                // Image viewing must not invalidate the document's derived
                // Markdown caches. Only release its decoded-image claims.
                self.release_tab_image_claims(previous, cx);
            } else {
                // Normal document-to-document dormancy keeps the existing
                // memory policy; leaving an image releases its viewer claim.
                let released_keys = self.tabs[previous].enter_dormant();
                let dropped = self.preview_image_cache.release_all(released_keys.iter());
                for image in dropped {
                    cx.drop_image(image, None);
                }
            }
        }
        self.active_tab = index;
        if previous != index {
            self.reveal_active_tab_in_strip();
        }
        self.tab_context_menu = None;
        self.slash_commands = None;
        self.dismissed_slash_query = None;
        self.dismiss_visual_block_menu();
        if self.active_tab().is_image() {
            self.search_visible = false;
            self.replace_visible = false;
            self.search_focus = None;
            self.search_control_focus = None;
            self.link_editor = None;
            self.preview_context_menu = None;
        }
        if self.active_tab().is_document() {
            self.refresh_search_matches();
        } else {
            self.search_matches.clear();
            self.current_search_index = None;
        }
        self.sync_and_persist_session();
        self.sync_git_branch_context(cx);
        cx.notify();
    }

    /// Ask the tab strip to scroll the active tab fully into view with the
    /// minimal scroll amount. GPUI's `FirstVisible` strategy makes this a
    /// no-op when the tab is already visible, and the pending reveal is
    /// consumed by the strip's next layout pass (callers already notify).
    pub(super) fn reveal_active_tab_in_strip(&mut self) {
        self.tab_bar_scroll.scroll_to_item(self.active_tab);
        #[cfg(test)]
        {
            self.last_tab_strip_reveal = Some(self.active_tab);
        }
    }

    pub(super) fn begin_preview_selection(
        &mut self,
        block_index: usize,
        run_id: PreviewTextRunId,
        index: usize,
        run_text: SharedString,
        cx: &mut Context<Self>,
    ) {
        let offset = clamp_preview_offset(run_text.as_ref(), index);
        let caret = PreviewCaret {
            block_index,
            run_id,
            offset,
        };
        let tab = self.active_tab_mut();
        tab.preview_is_selecting = true;
        tab.preview_selection = Some(PreviewSelection {
            anchor: caret,
            head: caret,
        });
        // Preview interaction takes over; stop any in-progress editor drag.
        tab.is_selecting = false;
        self.preview_context_menu = None;
        cx.notify();
    }

    pub(super) fn update_preview_selection_head(
        &mut self,
        block_index: usize,
        run_id: PreviewTextRunId,
        index: usize,
        run_text: SharedString,
        cx: &mut Context<Self>,
    ) {
        let offset = clamp_preview_offset(run_text.as_ref(), index);
        let tab = self.active_tab_mut();
        if !tab.preview_is_selecting {
            return;
        }
        let Some(selection) = tab.preview_selection.as_mut() else {
            return;
        };
        let head = PreviewCaret {
            block_index,
            run_id,
            offset,
        };
        if selection.head != head {
            selection.head = head;
            cx.notify();
        }
    }

    pub(super) fn end_preview_selection(&mut self, cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        tab.preview_is_selecting = false;
        cx.notify();
    }

    /// Cached `SharedString` copy of the active tab's document text for the
    /// current version. Cloning the returned value is an `Arc` bump, not a
    /// text copy.
    pub(super) fn shared_document_text(&self) -> SharedString {
        self.active_tab().shared_document_text()
    }

    /// Syntax highlighting memoized across edits; see `highlight_cache`.
    pub(super) fn highlighted_code(
        &self,
        language: Option<&str>,
        code: &str,
    ) -> Rc<Vec<Vec<HighlightedSpan>>> {
        let key = (language.map(str::to_string), code.to_string());
        if let Some(cached) = self.highlight_cache.borrow().get(&key) {
            return cached.clone();
        }
        let highlighted = Rc::new(highlight_code(code, language));
        let mut cache = self.highlight_cache.borrow_mut();
        if cache.len() >= 128 {
            cache.clear();
        }
        cache.insert(key, highlighted.clone());
        highlighted
    }

    pub(super) fn check_recovery_on_startup(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Ok(entries) = inspect_recovery_files(&self.recovery_dir) else {
            return;
        };
        if entries.is_empty() {
            return;
        }
        self.dismiss_visual_block_menu();
        self.recovery_manager = Some(RecoveryManagerState { entries });
        self.status = t(self.language, Msg::StatusRecoveryAvailable).into();
        cx.notify();
    }

    pub(super) fn restore_recovery_entry(
        &mut self,
        recovery_path: &Path,
        cx: &mut Context<Self>,
    ) -> io::Result<()> {
        let recovery = load_recovery_file(recovery_path)?;
        let original_path = recovery.original_path.clone();
        let document = MarkdownDocument::recovered_with_identity(
            recovery.text,
            recovery.original_path,
            recovery.disk_identity,
        );

        let reusable_index = original_path.as_deref().and_then(|path| {
            find_tab_with_document_path(&self.tabs, path).filter(|index| {
                self.tabs[*index]
                    .document_tab()
                    .is_some_and(|tab| !tab.document.is_dirty())
            })
        });
        if let Some(index) = reusable_index {
            self.switch_active_tab(index, cx);
            self.replace_active_tab(document, cx);
        } else {
            self.open_in_new_tab(document, cx);
        }
        self.active_tab_mut().last_recovery_file = Some(recovery_path.to_path_buf());
        self.remove_recovery_manager_entry(recovery_path);
        self.status = p1_tf(
            self.language,
            P1Msg::RecoveryRestored,
            &[&recovery_path.display().to_string()],
        )
        .into();
        self.sync_and_persist_session();
        cx.notify();
        Ok(())
    }

    pub(super) fn restore_all_recovery_entries(&mut self, cx: &mut Context<Self>) {
        let paths = self
            .recovery_manager
            .as_ref()
            .map(|manager| {
                manager
                    .entries
                    .iter()
                    .filter(|entry| entry.is_readable())
                    .map(|entry| entry.recovery_path.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut failed = false;
        for path in paths {
            failed |= self.restore_recovery_entry(&path, cx).is_err();
        }
        if failed {
            self.status = p1_tf(
                self.language,
                P1Msg::RecoverySomeFailed,
                &[p1_t(self.language, P1Msg::RecoveryUnknown)],
            )
            .into();
        }
        cx.notify();
    }

    pub(super) fn discard_recovery_entry(&mut self, recovery_path: &Path, cx: &mut Context<Self>) {
        match delete_recovery_file(recovery_path) {
            Ok(()) => {
                self.remove_recovery_manager_entry(recovery_path);
                self.status = p1_tf(
                    self.language,
                    P1Msg::RecoveryDiscarded,
                    &[&recovery_path.display().to_string()],
                )
                .into();
            }
            Err(err) => {
                self.status = p1_tf(
                    self.language,
                    P1Msg::RecoverySomeFailed,
                    &[&err.to_string()],
                )
                .into();
            }
        }
        cx.notify();
    }

    pub(super) fn discard_all_recovery_entries(&mut self, cx: &mut Context<Self>) {
        let paths = self
            .recovery_manager
            .as_ref()
            .map(|manager| {
                manager
                    .entries
                    .iter()
                    .map(|entry| entry.recovery_path.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut failed = false;
        for path in paths {
            if delete_recovery_file(&path).is_ok() {
                self.remove_recovery_manager_entry(&path);
            } else {
                failed = true;
            }
        }
        self.status = if failed {
            p1_tf(
                self.language,
                P1Msg::RecoverySomeFailed,
                &[p1_t(self.language, P1Msg::RecoveryUnknown)],
            )
            .into()
        } else {
            t(self.language, Msg::StatusRecoveryDiscarded).into()
        };
        cx.notify();
    }

    pub(super) fn close_recovery_manager(&mut self, cx: &mut Context<Self>) {
        self.recovery_manager = None;
        cx.notify();
    }

    fn remove_recovery_manager_entry(&mut self, recovery_path: &Path) {
        let Some(manager) = &mut self.recovery_manager else {
            return;
        };
        manager
            .entries
            .retain(|entry| entry.recovery_path != recovery_path);
        let empty = manager.entries.is_empty();
        if empty {
            self.recovery_manager = None;
        }
    }

    pub(super) fn apply_startup_open_intent(
        &mut self,
        intent: StartupOpenIntent,
        cx: &mut Context<Self>,
    ) {
        match intent {
            StartupOpenIntent::None => {}
            StartupOpenIntent::File(path) => {
                let display_path = path.display().to_string();
                match MarkdownDocument::open(&path) {
                    Ok(document) => {
                        self.replace_active_tab(document, cx);
                        self.update_workspace_root_from_document(cx);
                        self.record_recent_path(&path);
                        self.active_menu = None;
                        self.status = self.trf(Msg::StatusOpened, &[&display_path]);
                    }
                    Err(err) => {
                        tracing::warn!(path = ?path, error = %err, "startup file open failed");
                        self.active_menu = None;
                        self.status = self.trf(Msg::StatusOpenFailed, &[&err.to_string()]);
                    }
                }
                cx.notify();
            }
            StartupOpenIntent::Folder(path) => {
                let display_path = path.display().to_string();
                self.set_workspace_root(path, cx);
                self.sidebar_visible = true;
                self.sidebar_tab = SidebarTab::Files;
                self.active_menu = None;
                self.persist_preferences();
                self.schedule_file_tree_scan(Some(display_path), cx);
                cx.notify();
            }
            StartupOpenIntent::Invalid { path, reason } => {
                let detail = startup_open_failure_detail(&path, reason);
                tracing::warn!(
                    path = ?path,
                    reason = ?reason,
                    "startup path could not be opened"
                );
                self.active_menu = None;
                self.status = self.trf(Msg::StatusOpenFailed, &[&detail]);
                cx.notify();
            }
        }
    }

    pub(super) fn after_document_changed(&mut self, cx: &mut Context<Self>) {
        if self.active_tab().is_image() {
            return;
        }
        self.slash_commands = None;
        self.dismissed_slash_query = None;
        self.dismiss_visual_block_menu();
        let tab = self.active_tab_mut();
        tab.clear_visual_caret_affinity();
        tab.clear_visual_navigation_intent();
        tab.visual_navigation_snapshots.clear();
        tab.visual_navigation_snapshot_ids.clear();
        tab.visual_cursor_reveal_pending = true;
        tab.visual_caret_bounds = None;
        tab.visual_marked_range_bounds = None;
        #[cfg(test)]
        {
            tab.visual_last_projection = None;
            tab.visual_last_projection_styles = None;
        }
        self.refresh_search_matches();
        self.center_cursor_if_typewriter();
        self.schedule_autosave(cx);
    }

    pub(super) fn check_external_changes(&mut self, cx: &mut Context<Self>) {
        for index in 0..self.tabs.len() {
            if self.tabs[index].is_image() {
                continue;
            }
            let state = match self.tabs[index].document.check_disk_state() {
                Ok(state) => state,
                Err(err) => {
                    tracing::warn!(error = %err, "external file check failed");
                    continue;
                }
            };
            match state {
                DiskState::Unchanged => self.tabs[index].external_conflict = None,
                DiskState::Modified if !self.tabs[index].document.is_dirty() => {
                    self.release_tab_image_claims(index, cx);
                    let tab = &mut self.tabs[index];
                    match tab.document.reload_from_disk() {
                        Ok(()) => {
                            tab.external_conflict = None;
                            tab.selected_range = 0..0;
                            tab.selection_reversed = false;
                            tab.marked_range = None;
                            tab.undo_stack.clear();
                            tab.redo_stack.clear();
                            tab.reset_preview_list();
                            if index == self.active_tab {
                                self.status = p0_t(self.language, P0Msg::ExternalReloaded).into();
                            }
                        }
                        Err(err) => {
                            tab.external_conflict = Some(DiskState::Modified);
                            tracing::warn!(error = %err, "external file reload failed");
                        }
                    }
                }
                DiskState::Modified | DiskState::Missing => {
                    self.tabs[index].external_conflict = Some(state);
                    if index == self.active_tab {
                        self.status = match state {
                            DiskState::Missing => {
                                p0_t(self.language, P0Msg::ExternalMissing).into()
                            }
                            _ => p0_t(self.language, P0Msg::ExternalConflict).into(),
                        };
                    }
                }
            }
        }
        cx.notify();
    }

    pub(super) fn arm_external_file_poll(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(Duration::from_secs(2)).await;
                if this
                    .update(cx, |app, cx| app.check_external_changes(cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    pub(super) fn set_workspace_root(&mut self, root: PathBuf, cx: &mut Context<Self>) {
        let root = comparable_document_path(&root);
        let root_changed =
            workspace_root_needs_reset(&self.workspace_root, self.file_tree.is_some(), &root);

        if root_changed {
            self.collapsed_tree_paths.clear();
            self.file_tree_needs_initial_collapse = true;
            self.selected_tree_path = None;
            self.file_tree_scroll = ScrollHandle::new();
            self.file_tree = Some(FileTree {
                root: root.clone(),
                entries: Vec::new(),
                show_hidden: false,
            });
        }

        self.workspace_root = root;
        self.sync_and_persist_session();
        self.sync_git_branch_context(cx);
    }

    pub(super) fn update_workspace_root_from_document(&mut self, cx: &mut Context<Self>) {
        self.sync_git_branch_context(cx);
        let Some(document_path) = self.active_tab().path().map(Path::to_path_buf) else {
            return;
        };
        let current_root = self
            .file_tree
            .as_ref()
            .map(|_| self.workspace_root.as_path());
        let Some(next_root) = workspace_root_for_document(current_root, &document_path) else {
            return;
        };

        if self.file_tree.is_some()
            && scan_result_matches_workspace(&self.workspace_root, &next_root)
        {
            return;
        }

        self.set_workspace_root(next_root, cx);
        self.refresh_file_tree(cx);
    }

    pub(super) fn refresh_file_tree(&mut self, cx: &mut Context<Self>) {
        self.schedule_file_tree_scan(None, cx);
    }

    /// Scans the workspace on a background thread so the UI never blocks on a
    /// large directory tree. The previous synchronous scan was the dominant
    /// cause of the startup stall: it ran on the main thread during window
    /// creation and could walk tens of thousands of entries.
    pub(super) fn schedule_file_tree_scan(
        &mut self,
        opened_folder_display: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let requested_root = self.workspace_root.clone();
        let scan_root = requested_root.clone();
        let show_hidden = self.show_hidden_files;
        cx.spawn(async move |this, cx| {
            // Run the filesystem traversal off the main thread.
            let scanned = cx
                .background_executor()
                .spawn(async move { FileTree::scan_with_options(&scan_root, show_hidden) })
                .await;
            let _ = this.update(cx, |app, cx| {
                if !scan_result_matches_workspace(&requested_root, &app.workspace_root) {
                    return;
                }

                update_file_tree_collapse_state_from_scan(
                    &scanned,
                    &mut app.collapsed_tree_paths,
                    &mut app.file_tree_needs_initial_collapse,
                );
                match scanned {
                    Ok(tree) => {
                        app.file_tree = Some(tree);
                        if let Some(path) = opened_folder_display.as_deref() {
                            app.status = app.trf(Msg::StatusOpenedFolder, &[path]);
                        }
                        if app
                            .selected_tree_path
                            .as_ref()
                            .is_some_and(|path| !path.exists())
                        {
                            app.selected_tree_path = None;
                        }
                    }
                    Err(err) => {
                        app.status = app.trf(Msg::StatusOpenFolderFailed, &[&err.to_string()]);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn discard_current_recovery_file(&mut self) {
        if self.active_tab().is_image() {
            return;
        }
        if let Some(recovery) = self.active_tab_mut().last_recovery_file.take() {
            let _ = delete_recovery_file(recovery);
        }
    }

    pub(super) fn discard_all_tab_recovery_files(&mut self) {
        for tab in &mut self.tabs {
            if tab.is_document()
                && let Some(recovery) = tab.last_recovery_file.take()
            {
                let _ = delete_recovery_file(recovery);
            }
        }
    }

    /// Open `document` in a brand-new tab and make it active. Used by new
    /// untitled tabs and crash-recovery restore; filesystem-backed opens should
    /// go through the path helpers so already-open files can reuse their tab.
    pub(super) fn open_in_new_tab(&mut self, document: MarkdownDocument, cx: &mut Context<Self>) {
        let tab = self.editor_tab_for_document(document);
        self.open_tab_in_new_tab(tab, cx);
    }

    pub(super) fn open_image_in_new_tab(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let tab = self.editor_tab_for_image(path);
        self.open_tab_in_new_tab(tab, cx);
    }

    fn open_tab_in_new_tab(&mut self, tab: EditorTab, cx: &mut Context<Self>) {
        let previous = self.active_tab;
        let opening_image = tab.is_image();
        // Opening a new tab leaves the previous one inactive — same dormancy
        // policy as switch_active_tab.
        if self.tabs[previous].is_document() {
            self.tabs[previous].finish_undo_capture();
            self.tabs[previous].preview_is_selecting = false;
            self.tabs[previous].clear_visual_caret_affinity();
            self.tabs[previous].marked_range = None;
        }
        if self.tabs[previous].is_document() && opening_image {
            self.release_tab_image_claims(previous, cx);
        } else {
            let released = self.tabs[previous].enter_dormant();
            let dropped = self.preview_image_cache.release_all(released.iter());
            for image in dropped {
                cx.drop_image(image, None);
            }
        }
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.tab_context_menu = None;
        if self.active_tab().is_image() {
            self.search_visible = false;
            self.replace_visible = false;
            self.search_focus = None;
            self.search_control_focus = None;
            self.link_editor = None;
            self.preview_context_menu = None;
        }
        if self.active_tab().is_document() {
            self.refresh_search_matches();
        } else {
            self.search_matches.clear();
            self.current_search_index = None;
        }
        self.sync_git_branch_context(cx);
        self.reveal_active_tab_in_strip();
        cx.notify();
    }

    pub(super) fn editor_tab_for_document(&self, document: MarkdownDocument) -> EditorTab {
        let mut tab = EditorTab::new(document);
        tab.line_height = px(self.typography_metrics().editor_line_height);
        tab
    }

    pub(super) fn editor_tab_for_image(&self, path: PathBuf) -> EditorTab {
        EditorTab::new_image(path.clone(), PreviewImageKey::from_local_path(&path))
    }

    /// Replace the active tab's document in place: discard its recovery file,
    /// reset its selection/undo/scroll state, and load `document`. Used by
    /// File→New and File→Open (single-tab behaviour continuity).
    pub(super) fn replace_active_tab(
        &mut self,
        document: MarkdownDocument,
        cx: &mut Context<Self>,
    ) {
        let tab = self.editor_tab_for_document(document);
        self.replace_active_with_tab(tab, cx);
    }

    pub(super) fn replace_active_tab_with_image(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let tab = self.editor_tab_for_image(path);
        self.replace_active_with_tab(tab, cx);
    }

    fn replace_active_with_tab(&mut self, tab: EditorTab, cx: &mut Context<Self>) {
        let active = self.active_tab;
        self.release_tab_image_claims(active, cx);
        if self.tabs[active].is_document()
            && let Some(recovery) = self.tabs[active].last_recovery_file.take()
        {
            let _ = delete_recovery_file(recovery);
        }
        self.tabs[active] = tab;
        // The replaced tab no longer exists; any menu targeting it is stale.
        self.tab_context_menu = None;
        if self.active_tab().is_image() {
            self.search_visible = false;
            self.replace_visible = false;
            self.search_focus = None;
            self.search_control_focus = None;
            self.link_editor = None;
            self.preview_context_menu = None;
        }
        if self.active_tab().is_document() {
            self.refresh_search_matches();
        } else {
            self.search_matches.clear();
            self.current_search_index = None;
        }
        self.sync_git_branch_context(cx);
        cx.notify();
    }

    pub(super) fn open_tree_file(
        &mut self,
        path: PathBuf,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Opening from the file tree follows the default open-target
        // preference: a safe-to-replace active tab (image, untitled, or clean
        // document) is replaced in place, anything else — including a dirty
        // document — appends a new tab, so no dirty-guard prompt is ever
        // needed here.
        self.open_tree_file_confirmed(path, cx);
    }

    pub(super) fn open_tree_file_confirmed(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let intent = self.default_open_intent();
        if let Err(error) = self.open_supported_path(path, intent, cx) {
            self.status = self.trf(Msg::StatusOpenFailed, &[&error]);
            cx.notify();
        }
    }

    /// The active tab's preview blocks, re-parsed only when typing has settled.
    ///
    /// Split/Read renders call this instead of `preview_blocks_shared()`
    /// directly. While keystrokes arrive faster than [`PREVIEW_DEBOUNCE`] it
    /// returns the blocks from the previous parse (so a keystroke's render does
    /// not pay a full-document parse), arms a timer that re-renders once the
    /// pause is long enough, and caps staleness at [`PREVIEW_MAX_STALE`] so the
    /// preview keeps moving during a continuous typing burst.
    ///
    /// The parse itself runs on a background thread (`spawn_preview_parse`), so
    /// the frames where it fires no longer stall the UI; renders between spawn
    /// and landing keep showing the previous blocks. Only the very first parse
    /// of a document is synchronous, so the pane never flashes empty.
    pub(super) fn preview_blocks_debounced(
        &mut self,
        cx: &mut Context<Self>,
    ) -> std::sync::Arc<Vec<PreviewBlock>> {
        let version = self.active_tab().document.version();
        let now = Instant::now();

        let tab = self.active_tab_mut();
        if version != tab.preview_seen_version {
            tab.preview_seen_version = version;
            tab.preview_changed_at = Some(now);
            tab.preview_debounce_generation = tab.preview_debounce_generation.wrapping_add(1);
            self.arm_preview_debounce(cx);
        }

        let tab = self.active_tab();
        if tab.preview_reflects_version == Some(version) {
            return tab.preview_list_blocks.clone();
        }
        if tab.preview_reflects_version.is_none() {
            // Nothing parsed yet (fresh/replaced document, or the first
            // Split/Read render): parse inline so this frame shows content
            // instead of a blank pane while a background parse runs.
            let blocks = self.active_tab().document.preview_blocks_shared();
            let tab = self.active_tab_mut();
            tab.preview_reflects_version = Some(version);
            tab.preview_reflects_at = Some(Instant::now());
            return blocks;
        }
        let since_change = tab.preview_changed_at.map(|at| now.duration_since(at));
        let since_parse = tab.preview_reflects_at.map(|at| now.duration_since(at));
        // One parse in flight at a time: while one runs, keep returning the
        // stale blocks; its landing notifies, and that render re-evaluates
        // whether the document moved on and another parse is due.
        if self.active_tab().preview_parse_inflight.is_none()
            && should_parse_preview_now(since_change, since_parse)
        {
            self.spawn_preview_parse(version, cx);
        }
        self.active_tab().preview_list_blocks.clone()
    }

    /// Parse the active tab's text on a background thread and fold the result
    /// back into the tab (and its document's derived caches) when it lands.
    /// The landing is matched to its tab by a globally unique task id rather
    /// than tab index — closing another tab shifts indices, and replacing the
    /// document clears the marker so a stale result is dropped, never applied.
    pub(super) fn spawn_preview_parse(&mut self, version: u64, cx: &mut Context<Self>) {
        let task_id = next_preview_parse_id();
        let text = self.active_tab().document.text().to_string();
        self.active_tab_mut().preview_parse_inflight = Some(task_id);
        cx.spawn(async move |this, cx| {
            let (blocks, headings) = cx
                .background_spawn(
                    async move { MarkdownDocument::derive_preview_and_outline(&text) },
                )
                .await;
            let _ = this.update(cx, |app, cx| {
                let Some(tab) = app
                    .tabs
                    .iter_mut()
                    .find(|tab| tab.preview_parse_inflight == Some(task_id))
                else {
                    return;
                };
                tab.preview_parse_inflight = None;
                let blocks = std::sync::Arc::new(blocks);
                // Version-gated: refused if the document changed while the
                // parse ran. The blocks still go on screen (slightly stale
                // beats frozen mid-burst) and the version mismatch makes the
                // next render schedule a fresh parse.
                tab.document
                    .install_derived(version, blocks.clone(), headings);
                tab.preview_reflects_version = Some(version);
                tab.preview_reflects_at = Some(Instant::now());
                tab.sync_preview_list(&blocks);
                cx.notify();
            });
        })
        .detach();
    }

    /// Arm a timer that re-renders once the debounce window has passed with no
    /// further edits. Every edit bumps the tab's generation, so of the timers
    /// in flight only the one armed by the *latest* edit survives its
    /// generation check — earlier ones fire and do nothing.
    pub(super) fn arm_preview_debounce(&mut self, cx: &mut Context<Self>) {
        let active_index = self.active_tab;
        let generation = self.active_tab().preview_debounce_generation;
        cx.spawn(async move |this, cx| {
            Timer::after(PREVIEW_DEBOUNCE).await;
            let _ = this.update(cx, |app, cx| {
                let Some(tab) = app.tabs.get(active_index) else {
                    return;
                };
                if tab.preview_debounce_generation != generation {
                    return;
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn schedule_autosave(&mut self, cx: &mut Context<Self>) {
        if self.active_tab().is_image() {
            return;
        }
        // Bump the generation even when disabled so a pending timer from a
        // previous schedule is invalidated.
        let active_index = self.active_tab;
        let autosave_enabled = self.auto_save_preferences.enabled;
        let delay = Duration::from_secs(self.auto_save_preferences.delay_secs.max(1));
        let recovery_dir = self.recovery_dir.clone();
        let tab = self.active_tab_mut();
        tab.autosave_generation = tab.autosave_generation.wrapping_add(1);
        if !autosave_enabled {
            return;
        }
        let generation = tab.autosave_generation;

        cx.spawn(async move |this, cx| {
            Timer::after(delay).await;
            let _ = this.update(cx, |app, cx| {
                // Validate the tab still exists and its generation matches, so
                // a tab switch (or close) between schedule and fire does not
                // autosave the wrong tab or a removed one.
                let Some(tab) = app.tabs.get(active_index) else {
                    return;
                };
                if tab.is_image()
                    || tab.autosave_generation != generation
                    || !tab.document.is_dirty()
                {
                    return;
                }

                let tab = &mut app.tabs[active_index];
                let recovery = match tab
                    .document
                    .save_recovery_copy_with_id(&recovery_dir, tab.recovery_id)
                {
                    Ok(path) => {
                        if let Some(previous) = tab.last_recovery_file.replace(path.clone())
                            && previous != path
                        {
                            let _ = delete_recovery_file(previous);
                        }
                        path
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "recovery snapshot failed");
                        app.status =
                            tf(app.language, Msg::StatusAutoSaveFailed, &[&err.to_string()]).into();
                        cx.notify();
                        return;
                    }
                };

                if tab.document.path().is_none() {
                    app.status = tf(
                        app.language,
                        Msg::StatusRecoverySaved,
                        &[&recovery.display().to_string()],
                    )
                    .into();
                } else {
                    match tab.document.save() {
                        Ok(()) => {
                            if let Some(recovery) = tab.last_recovery_file.take() {
                                let _ = delete_recovery_file(recovery);
                            }
                            let path = tab.document.path().unwrap();
                            app.status = tf(
                                app.language,
                                Msg::StatusAutoSaved,
                                &[&path.display().to_string()],
                            )
                            .into();
                        }
                        Err(err) => {
                            if err.kind() == io::ErrorKind::AlreadyExists {
                                tab.external_conflict = Some(DiskState::Modified);
                            }
                            tracing::warn!(error = %err, "auto-save failed after recovery");
                            app.status =
                                tf(app.language, Msg::StatusAutoSaveFailed, &[&err.to_string()])
                                    .into();
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn search_options(&self) -> SearchOptions {
        SearchOptions {
            query: self.search_query.buffer.clone(),
            case_sensitive: self.search_case_sensitive,
            regex: self.search_regex,
        }
    }

    pub(super) fn refresh_search_matches(&mut self) {
        if !self.search_visible {
            self.search_matches.clear();
            self.current_search_index = None;
            self.search_result = SearchResultState::Idle;
            self.search_generation = None;
            return;
        }

        if self.search_query.buffer.is_empty() {
            self.search_matches.clear();
            self.current_search_index = None;
            self.search_result = SearchResultState::Idle;
            self.search_generation = None;
            return;
        }

        let domain = if matches!(self.view_mode, ViewMode::Read) {
            SearchDomain::ReadPreview
        } else {
            SearchDomain::Source
        };
        self.replace_visible =
            self.search_form == SearchPanelForm::Replace && domain == SearchDomain::Source;
        let version = self.active_tab().document.version();
        let key = SearchGenerationKey {
            tab_index: self.active_tab,
            document_version: version,
            domain,
            query: self.search_query.buffer.clone(),
            case_sensitive: self.search_case_sensitive,
            regex: self.search_regex,
        };
        if self.search_generation.as_ref() == Some(&key) {
            return;
        }

        if domain == SearchDomain::ReadPreview
            && self.active_tab().preview_reflects_version != Some(version)
        {
            self.search_matches.clear();
            self.current_search_index = None;
            self.search_result = SearchResultState::PendingPreview;
            // Do not retain the generation: installation of current-version
            // cached blocks must cause the same query to run again.
            self.search_generation = None;
            return;
        }

        let options = self.search_options();
        let matches = match domain {
            SearchDomain::Source => {
                self.active_tab()
                    .document
                    .find_matches(&options)
                    .map(|matches| {
                        matches
                            .into_iter()
                            .map(SearchTarget::Source)
                            .collect::<Vec<_>>()
                    })
            }
            SearchDomain::ReadPreview => SearchPattern::compile(&options).map(|pattern| {
                preview_search_matches(&self.active_tab().preview_list_blocks, &pattern)
                    .into_iter()
                    .map(SearchTarget::ReadPreview)
                    .collect()
            }),
        };

        self.search_generation = Some(key);
        match matches {
            Ok(matches) if matches.is_empty() => {
                self.search_matches.clear();
                self.current_search_index = None;
                self.search_result = SearchResultState::NoMatches;
            }
            Ok(matches) => {
                let origin = match domain {
                    SearchDomain::Source => self.cursor_offset(),
                    SearchDomain::ReadPreview => {
                        self.active_tab().preview_list.logical_scroll_top().item_ix
                    }
                };
                let current = matches
                    .iter()
                    .position(|target| match target {
                        SearchTarget::Source(found) => found.range.start >= origin,
                        SearchTarget::ReadPreview(found) => found.block_index >= origin,
                    })
                    .unwrap_or(0);
                self.search_matches = matches;
                self.current_search_index = Some(current);
                self.search_result = SearchResultState::Ready;
            }
            Err(err) => {
                self.search_matches.clear();
                self.current_search_index = None;
                self.search_result = SearchResultState::InvalidPattern(err.message().to_string());
                self.status = self.trf(Msg::StatusFindFailed, &[err.message()]);
            }
        }
    }

    pub(super) fn close_search_overlay(&mut self, cx: &mut Context<Self>) {
        hide_search_overlay_state(
            &mut self.search_visible,
            &mut self.replace_visible,
            &mut self.search_focus,
            &mut self.input_marked_len,
        );
        self.search_control_focus = None;
        self.search_field_bounds = [None, None];
        self.search_query.marked_range = None;
        self.replace_text.marked_range = None;
        self.refresh_search_matches();
        cx.notify();
    }

    pub(super) fn select_search_match(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(found) = self.search_matches.get(index).cloned() {
            self.current_search_index = Some(index);
            match found {
                SearchTarget::Source(found) => {
                    let tab = self.active_tab_mut();
                    tab.selected_range = found.range.clone();
                    tab.selection_reversed = false;
                    tab.marked_range = None;
                    tab.visual_cursor_reveal_pending = true;
                    tab.visual_caret_bounds = None;
                    match self.view_mode {
                        ViewMode::VisualEdit => {
                            let blocks = self.active_tab().document.visual_blocks_shared();
                            if let Some(item_ix) = visual_block_index_for_offset(
                                &blocks,
                                found.range.start,
                                self.active_tab().document.text().len(),
                            ) {
                                self.active_tab().visual_list.scroll_to_reveal_item(item_ix);
                            }
                        }
                        ViewMode::Edit | ViewMode::Split => {
                            self.scroll_editor_to_offset(found.range.start);
                        }
                        ViewMode::Read => {}
                    }
                    self.status = self.trf(
                        Msg::StatusMatchPosition,
                        &[
                            &(index + 1).to_string(),
                            &self.search_matches.len().to_string(),
                            &found.line.to_string(),
                            &found.column.to_string(),
                        ],
                    );
                }
                SearchTarget::ReadPreview(found) => {
                    self.active_tab()
                        .preview_list
                        .scroll_to_reveal_item(found.block_index);
                    self.status = self.trf(
                        Msg::StatusMatches,
                        &[&self.search_matches.len().to_string()],
                    );
                }
            }
            cx.notify();
        }
    }

    pub(super) fn jump_to_offset(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = clamp_to_text_boundary(self.active_tab().document.text(), offset);
        let tab = self.active_tab_mut();
        tab.selected_range = offset..offset;
        tab.selection_reversed = false;
        tab.marked_range = None;
        tab.visual_cursor_reveal_pending = true;
        tab.visual_caret_bounds = None;
        self.scroll_editor_to_offset(offset);
        self.status = t(self.language, Msg::StatusJumpedToHeading).into();
        cx.notify();
    }

    /// Navigate to an outline heading while preserving the canonical source
    /// position used by active-section highlighting and later mode switches.
    /// Read mode additionally moves its visible virtualized preview to the
    /// exact heading block. Visual Edit top-aligns the heading block: the
    /// generic cursor reveal only scrolls minimally, which places headings
    /// below the viewport at the pane bottom instead of its top. All other
    /// modes retain `jump_to_offset` behavior.
    pub(super) fn navigate_to_outline_heading(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = clamp_to_text_boundary(self.active_tab().document.text(), offset);
        self.jump_to_offset(offset, cx);
        match self.view_mode {
            ViewMode::Read => {
                let target = preview_heading_index_for_source_offset(
                    &self.active_tab().preview_list_blocks,
                    offset,
                );
                if let Some(item_ix) = target {
                    self.active_tab().preview_list.scroll_to(gpui::ListOffset {
                        item_ix,
                        offset_in_item: px(0.),
                    });
                }
            }
            ViewMode::VisualEdit => {
                let tab = self.active_tab();
                let blocks = tab.document.visual_blocks_shared();
                let text_len = tab.document.text().len();
                let target = visual_block_index_for_offset(&blocks, offset, text_len);
                if let Some(item_ix) = target {
                    self.active_tab().visual_list.scroll_to(gpui::ListOffset {
                        item_ix,
                        offset_in_item: px(0.),
                    });
                }
            }
            _ => {}
        }
    }

    /// Toggle one disclosure node in the active document's session-only
    /// outline state. Image tabs deliberately have no corresponding state.
    pub(super) fn toggle_outline_section(&mut self, key: OutlineNodeKey, cx: &mut Context<Self>) {
        let Some(tab) = self.active_tab().document_tab() else {
            return;
        };
        if tab.toggle_outline_node(key).is_some() {
            cx.notify();
        }
    }

    pub(super) fn scroll_editor_to_offset(&mut self, offset: usize) {
        self.active_tab_mut().scroll_editor_to_offset(offset);
    }

    pub(super) fn center_cursor_if_typewriter(&mut self) {
        if self.typewriter_mode {
            let offset = self.active_tab().cursor_offset();
            self.active_tab_mut()
                .scroll_editor_typewriter_to_offset(offset);
        }
    }

    pub(super) fn current_preferences(&self) -> AppPreferences {
        // Persist the active selection by name. A selection that resolves to a
        // custom `.theme` file is written to `custom_theme` (so the loader
        // re-resolves it next launch); any other built-in name is written to
        // `theme`.
        let is_custom = self
            .custom_themes
            .iter()
            .any(|theme| theme.name.eq_ignore_ascii_case(&self.selected_theme_name));
        let (theme_name, custom_theme_name) = if is_custom {
            (
                self.theme.name().to_string(),
                Some(self.selected_theme_name.clone()),
            )
        } else {
            (self.selected_theme_name.clone(), None)
        };
        AppPreferences {
            theme: theme_name,
            custom_theme: custom_theme_name,
            focus_mode: self.focus_mode,
            typewriter_mode: self.typewriter_mode,
            code_line_numbers: self.code_line_numbers,
            preview_adaptive_width: self.preview_adaptive_width,
            editor_font_size: self.editor_font_size,
            rendered_font_size: self.rendered_font_size,
            paragraph_spacing: self.paragraph_spacing,
            editor_font_family: self.editor_font_family.clone(),
            rendered_font_family: self.rendered_font_family.clone(),
            code_font_family: self.code_font_family.clone(),
            heading_menu_max_level: self.heading_menu_max_level,
            sync_scroll: self.sync_scroll,
            show_hidden_files: self.show_hidden_files,
            open_in_current_tab: self.open_in_current_tab,
            sidebar_visible: self.sidebar_visible,
            sidebar_tab: self.sidebar_tab,
            language: self.language.code().to_string(),
            check_for_updates_on_startup: self.check_for_updates_on_startup,
            last_update_check: self.last_update_check.clone(),
            auto_save: self.auto_save_preferences,
            export: self.export_preferences.clone(),
            shortcut_overrides: self.shortcut_overrides.clone(),
        }
    }

    /// Translate a static UI message in the active language.
    pub(super) fn tr(&self, msg: Msg) -> &'static str {
        t(self.language, msg)
    }

    /// Translate a templated UI message with positional arguments.
    pub(super) fn trf(&self, msg: Msg, args: &[&str]) -> SharedString {
        tf(self.language, msg, args).into()
    }

    /// All themes the Preferences panel can offer: built-ins first (in their
    /// canonical order), then user-loaded `.theme` files.
    pub(super) fn available_themes(&self) -> Vec<ThemeDefinition> {
        let mut themes = builtin_theme_definitions();
        for custom in &self.custom_themes {
            // Skip a user theme that shadows a built-in name — built-ins win
            // so the legacy name-to-palette mapping stays stable.
            if !themes.iter().any(|theme| theme.name == custom.name) {
                themes.push(custom.clone());
            }
        }
        themes
    }

    /// Resolve the active theme definition by `selected_theme_name`, checking
    /// built-ins first, then custom themes, then falling back to Paper.
    pub(super) fn active_theme_definition(&self) -> ThemeDefinition {
        let name = self.selected_theme_name.trim();
        builtin_theme_definitions()
            .into_iter()
            .find(|theme| theme.name.eq_ignore_ascii_case(name))
            .or_else(|| {
                self.custom_themes
                    .iter()
                    .find(|theme| theme.name.eq_ignore_ascii_case(name))
                    .cloned()
            })
            .unwrap_or_else(|| {
                builtin_theme_definitions()
                    .into_iter()
                    .next()
                    .expect("builtin theme table is non-empty")
            })
    }

    pub(super) fn palette(&self) -> ThemePalette {
        theme_palette_from_definition(&self.active_theme_definition())
    }

    /// Apply a theme by its display name (used by the Preferences panel and by
    /// `cycle_theme`). Updates both the name-based selection and the legacy
    /// `theme`/`custom_theme` fields so old code paths keep working.
    pub(super) fn apply_theme_by_name(&mut self, name: &str, cx: &mut Context<Self>) {
        self.selected_theme_name = name.trim().to_string();
        let resolved = self.active_theme_definition();
        // Keep the legacy `custom_theme` field in sync: set it only when the
        // selection is a user-loaded `.theme` file.
        self.custom_theme = self
            .custom_themes
            .iter()
            .find(|theme| theme.name.eq_ignore_ascii_case(name.trim()))
            .cloned();
        // And the legacy `theme` enum, resolved from the built-in six only.
        self.theme = AppTheme::from_name(&resolved.name).unwrap_or(AppTheme::Paper);
        // A new theme may carry different `[fonts]` contributions; re-resolve
        // and invalidate typography only when an effective family changed.
        let previous_fonts = self.resolved_font_families.clone();
        if self.recompute_resolved_font_families() != previous_fonts {
            self.refresh_typography_measurements(true, true);
        }
        self.status = self.trf(Msg::StatusTheme, &[&self.theme_label()]);
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn persist_preferences(&mut self) {
        if cfg!(test) && self.preferences_path == default_preferences_path() {
            return;
        }
        if let Err(err) = save_app_preferences(&self.preferences_path, &self.current_preferences())
        {
            self.status = self.trf(Msg::StatusPreferencesSaveFailed, &[&err.to_string()]);
        }
    }

    /// Snapshot open saved tabs / workspace root into `self.session` and write
    /// `session.toml`. Best-effort: failures are logged via the status bar.
    pub(super) fn sync_and_persist_session(&mut self) {
        self.session.open_files = session_open_files_from_paths(
            self.tabs
                .iter()
                .filter(|tab| tab.is_document())
                .map(|tab| tab.document.path()),
        );
        self.session.active_file = self
            .active_tab()
            .is_document()
            .then(|| self.active_tab().document.path())
            .flatten()
            .map(comparable_document_path);
        self.session.workspace_root = if self.file_tree.is_some() {
            Some(comparable_document_path(&self.workspace_root))
        } else {
            None
        };
        self.persist_session();
    }

    pub(super) fn persist_session(&mut self) {
        if cfg!(test) {
            return;
        }
        if let Err(err) = save_session_state(&self.session_path, &self.session) {
            tracing::warn!(error = %err, "failed to save session.toml");
        }
    }

    /// Record a Markdown path in the recent-files list and persist session.
    pub(super) fn record_recent_path(&mut self, path: &Path) {
        let path = comparable_document_path(path);
        self.session.touch_recent(path);
        self.sync_and_persist_session();
    }

    /// Restore workspace root and saved tabs from the loaded session when there
    /// is no CLI open intent. Missing paths are skipped.
    pub(super) fn restore_session_on_startup(
        &mut self,
        intent: &StartupOpenIntent,
        cx: &mut Context<Self>,
    ) {
        if !should_restore_session(intent) {
            return;
        }

        let (workspace_root, open_files, active_file) = filter_restorable_session(&self.session);

        if open_files.is_empty() {
            if let Some(root) = workspace_root {
                self.set_workspace_root(root, cx);
                self.schedule_file_tree_scan(None, cx);
            }
            return;
        }

        let mut opened_any = false;
        let mut replaced_initial = false;
        for path in open_files.iter() {
            match MarkdownDocument::open(path) {
                Ok(document) => {
                    if !replaced_initial {
                        self.replace_active_tab(document, cx);
                        replaced_initial = true;
                    } else {
                        self.open_in_new_tab(document, cx);
                    }
                    opened_any = true;
                }
                Err(err) => {
                    tracing::warn!(path = ?path, error = %err, "session restore skipped file");
                }
            }
        }

        if !opened_any {
            if let Some(root) = workspace_root {
                self.set_workspace_root(root, cx);
                self.schedule_file_tree_scan(None, cx);
            }
            return;
        }

        if let Some(active) = active_file.as_ref() {
            let _ = self.focus_existing_tab_for_path(active, cx);
        }

        if let Some(root) = workspace_root {
            // Prefer the persisted workspace root when it still exists; otherwise
            // fall back to deriving the root from the active document.
            self.set_workspace_root(root, cx);
            self.schedule_file_tree_scan(None, cx);
        } else {
            self.update_workspace_root_from_document(cx);
        }

        // Refresh recent list with restored paths and rewrite pruned session.
        for path in &open_files {
            if path.exists() {
                self.session.touch_recent(comparable_document_path(path));
            }
        }
        self.sync_and_persist_session();
    }

    pub(super) fn clear_recent_files(
        &mut self,
        _: &ClearRecentFiles,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.session.clear_recent();
        self.persist_session();
        self.active_menu = None;
        self.open_recent_submenu_open = false;
        self.status = t(self.language, Msg::StatusRecentFilesCleared).into();
        cx.notify();
    }

    pub(super) fn open_recent_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let display_path = path.display().to_string();
        if !path.is_file() {
            self.session.remove_recent(&comparable_document_path(&path));
            self.persist_session();
            self.active_menu = None;
            self.open_recent_submenu_open = false;
            self.status = self.trf(Msg::StatusOpenFailed, &[&display_path]);
            cx.notify();
            return;
        }

        let intent = self.default_open_intent();
        match self.open_supported_path(path.clone(), intent, cx) {
            Ok(()) => {
                self.open_recent_submenu_open = false;
            }
            Err(error) => {
                self.session.remove_recent(&comparable_document_path(&path));
                self.persist_session();
                self.active_menu = None;
                self.open_recent_submenu_open = false;
                self.status = self.trf(Msg::StatusOpenFailed, &[&error]);
            }
        }
        cx.notify();
    }

    pub(super) fn has_text_input_focus(&self) -> bool {
        self.pending_name_input.is_some()
            || self.link_editor.is_some()
            || self.file_tree_query_focused
            || (self.search_visible && self.search_control_focus.is_some())
    }

    pub(super) fn active_input_text_mut(&mut self) -> Option<&mut String> {
        if self.pending_name_input.is_some() {
            return self
                .pending_name_input
                .as_mut()
                .map(|pending| &mut pending.buffer);
        }
        match self.link_editor.as_mut() {
            Some(editor) => Some(match editor.field {
                LinkEditorField::Label => &mut editor.label,
                LinkEditorField::Url => &mut editor.url,
                LinkEditorField::Title => &mut editor.title,
            }),
            None if self.file_tree_query_focused => Some(&mut self.file_tree_query),
            None => None,
        }
    }

    pub(super) fn focused_search_field(&self) -> Option<&SearchFieldState> {
        match self.search_focus {
            Some(SearchField::Find) => Some(&self.search_query),
            Some(SearchField::Replace) => Some(&self.replace_text),
            None => None,
        }
    }

    pub(super) fn focused_search_field_mut(&mut self) -> Option<&mut SearchFieldState> {
        match self.search_focus {
            Some(SearchField::Find) => Some(&mut self.search_query),
            Some(SearchField::Replace) => Some(&mut self.replace_text),
            None => None,
        }
    }

    pub(super) fn after_input_changed(&mut self, cx: &mut Context<Self>) {
        if self.pending_name_input.is_some() {
            // The name prompt edits a single buffer; no search/tree filtering
            // runs while it is open.
            self.status = t(self.language, Msg::StatusNamingEntry).into();
        } else if self.link_editor.is_some() {
            self.status = p0_t(self.language, P0Msg::EditingLink).into();
        } else if self.file_tree_query_focused {
            self.status = self.file_tree_summary().into();
        } else {
            self.refresh_search_matches();
            if self.search_focus == Some(SearchField::Find)
                && let Some(index) = self.current_search_index
            {
                self.select_search_match(index, cx);
            }
            self.status = self.search_summary().into();
        }
        cx.notify();
    }

    /// Insert text into the focused redirected field, first removing any
    /// trailing IME composition. `keep_marked` records the new text as the
    /// active composition (still being edited) instead of committing it.
    /// The inline name editor is caret/selection-aware (text replaces the
    /// selection at the caret); the other redirected fields keep their
    /// append-only behavior.
    pub(super) fn insert_redirected_text(
        &mut self,
        text: &str,
        keep_marked: bool,
        cx: &mut Context<Self>,
    ) {
        if self.pending_name_input.is_some() {
            self.insert_redirected_name_text(text, keep_marked, cx);
            return;
        }
        if let Some(field) = self.focused_search_field_mut() {
            field.replace_selection(text, keep_marked);
            self.input_marked_len = 0;
            self.search_generation = None;
            self.after_input_changed(cx);
            return;
        }
        let marked = self.input_marked_len;
        let Some(target) = self.active_input_text_mut() else {
            return;
        };
        let keep = target.len().saturating_sub(marked.min(target.len()));
        target.truncate(keep);
        target.push_str(text);
        self.input_marked_len = if keep_marked { text.len() } else { 0 };
        self.after_input_changed(cx);
    }

    /// Caret-aware insert into the inline name editor's buffer: the active
    /// IME composition (which always ends at the caret) and the selection are
    /// replaced by `text`, and the caret lands after it.
    fn insert_redirected_name_text(
        &mut self,
        text: &str,
        keep_marked: bool,
        cx: &mut Context<Self>,
    ) {
        let marked = self.input_marked_len;
        let Some(pending) = self.pending_name_input.as_mut() else {
            return;
        };
        pending.clamp_to_boundaries();
        let composition_start = pending.cursor.saturating_sub(marked.min(pending.cursor));
        let selection = pending.selection();
        let start = composition_start.min(selection.start);
        let end = pending.cursor.max(selection.end);
        pending.buffer.replace_range(start..end, text);
        pending.cursor = start + text.len();
        pending.anchor = pending.cursor;
        self.input_marked_len = if keep_marked { text.len() } else { 0 };
        self.after_input_changed(cx);
    }

    /// Move the inline name editor's caret/selection. Routed here from the
    /// Left/Right/Home/End/Select* action handlers while the editor is open
    /// so those keys never reach the document caret.
    pub(super) fn move_name_caret(&mut self, movement: NameCaretMove, cx: &mut Context<Self>) {
        let Some(pending) = self.pending_name_input.as_mut() else {
            return;
        };
        pending.clamp_to_boundaries();
        let selection = pending.selection();
        match movement {
            NameCaretMove::Left => {
                pending.cursor = if selection.is_empty() {
                    previous_name_boundary(&pending.buffer, pending.cursor)
                } else {
                    selection.start
                };
                pending.anchor = pending.cursor;
            }
            NameCaretMove::Right => {
                pending.cursor = if selection.is_empty() {
                    next_name_boundary(&pending.buffer, pending.cursor)
                } else {
                    selection.end
                };
                pending.anchor = pending.cursor;
            }
            NameCaretMove::Home => {
                pending.cursor = 0;
                pending.anchor = 0;
            }
            NameCaretMove::End => {
                let len = pending.buffer.len();
                pending.cursor = len;
                pending.anchor = len;
            }
            NameCaretMove::SelectLeft => {
                pending.cursor = previous_name_boundary(&pending.buffer, pending.cursor);
            }
            NameCaretMove::SelectRight => {
                pending.cursor = next_name_boundary(&pending.buffer, pending.cursor);
            }
            NameCaretMove::SelectAll => {
                pending.anchor = 0;
                pending.cursor = pending.buffer.len();
            }
        }
        self.after_input_changed(cx);
    }

    /// Delete the selection (or the char before the caret) in the inline name
    /// editor. When an IME composition is active it is removed instead.
    fn pop_name_text(&mut self, cx: &mut Context<Self>) {
        let marked = self.input_marked_len;
        let Some(pending) = self.pending_name_input.as_mut() else {
            return;
        };
        pending.clamp_to_boundaries();
        if marked > 0 {
            let composition_start = pending.cursor.saturating_sub(marked.min(pending.cursor));
            pending
                .buffer
                .replace_range(composition_start..pending.cursor, "");
            pending.cursor = composition_start;
            pending.anchor = composition_start;
            self.input_marked_len = 0;
        } else {
            let selection = pending.selection();
            if selection.is_empty() {
                let start = previous_name_boundary(&pending.buffer, pending.cursor);
                pending.buffer.replace_range(start..pending.cursor, "");
                pending.cursor = start;
            } else {
                pending.buffer.replace_range(selection.clone(), "");
                pending.cursor = selection.start;
            }
            pending.anchor = pending.cursor;
        }
        self.after_input_changed(cx);
    }

    /// Delete the selection (or the char after the caret) in the inline name
    /// editor — the Delete-key counterpart of `pop_name_text`.
    fn delete_name_text_forward(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = self.pending_name_input.as_mut() else {
            return;
        };
        pending.clamp_to_boundaries();
        self.input_marked_len = 0;
        let selection = pending.selection();
        if selection.is_empty() {
            let end = next_name_boundary(&pending.buffer, pending.cursor);
            pending.buffer.replace_range(pending.cursor..end, "");
        } else {
            pending.buffer.replace_range(selection.clone(), "");
            pending.cursor = selection.start;
        }
        pending.anchor = pending.cursor;
        self.after_input_changed(cx);
    }

    pub(super) fn push_text_input(&mut self, text: &str, cx: &mut Context<Self>) {
        self.insert_redirected_text(text, false, cx);
    }

    pub(super) fn move_search_caret(
        &mut self,
        movement: SearchCaretMove,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(field) = self.focused_search_field_mut() else {
            return false;
        };
        field.move_caret(movement);
        self.input_marked_len = 0;
        cx.notify();
        true
    }

    pub(super) fn cycle_search_overlay_focus(&mut self, forward: bool, cx: &mut Context<Self>) {
        if !self.search_visible {
            return;
        }
        let mut stops = vec![
            SearchOverlayControl::FindField,
            SearchOverlayControl::Previous,
            SearchOverlayControl::Next,
            SearchOverlayControl::MatchCase,
            SearchOverlayControl::Regex,
        ];
        if self.replace_visible {
            stops.insert(1, SearchOverlayControl::ReplaceField);
            stops.extend([
                SearchOverlayControl::ReplaceCurrent,
                SearchOverlayControl::ReplaceAll,
            ]);
        }
        stops.push(SearchOverlayControl::Close);
        let current = self
            .search_control_focus
            .and_then(|control| stops.iter().position(|candidate| *candidate == control))
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % stops.len()
        } else if current == 0 {
            stops.len() - 1
        } else {
            current - 1
        };
        let control = stops[next];
        self.search_control_focus = Some(control);
        self.search_focus = match control {
            SearchOverlayControl::FindField => Some(SearchField::Find),
            SearchOverlayControl::ReplaceField => Some(SearchField::Replace),
            _ => None,
        };
        cx.notify();
    }

    pub(super) fn replace_search_text_utf16(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        marked: bool,
        selected_utf16: Option<Range<usize>>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(field) = self.focused_search_field_mut() else {
            return false;
        };
        field.clamp_to_boundaries();
        let range = range_utf16
            .map(|range| field.range_from_utf16(range))
            .or_else(|| field.marked_range.clone())
            .unwrap_or_else(|| field.selection());
        field.buffer.replace_range(range.clone(), new_text);
        let inserted_end = range.start + new_text.len();
        if let Some(selected) = selected_utf16 {
            let inserted = SearchFieldState::new(new_text);
            field.anchor = range.start + inserted.utf16_to_byte(selected.start);
            field.cursor = range.start + inserted.utf16_to_byte(selected.end);
        } else {
            field.cursor = inserted_end;
            field.anchor = inserted_end;
        }
        field.marked_range = marked.then_some(range.start..inserted_end);
        self.input_marked_len = 0;
        self.search_generation = None;
        self.after_input_changed(cx);
        true
    }

    pub(super) fn pop_text_input(&mut self, cx: &mut Context<Self>) -> bool {
        if self.pending_name_input.is_some() {
            self.pop_name_text(cx);
            return true;
        }
        if let Some(field) = self.focused_search_field_mut() {
            field.backspace();
            self.search_generation = None;
            self.after_input_changed(cx);
            return true;
        }
        self.input_marked_len = 0;
        if let Some(target) = self.active_input_text_mut() {
            target.pop();
            self.after_input_changed(cx);
            true
        } else {
            false
        }
    }

    /// Delete-forward variant of `pop_text_input` for the Delete key; returns
    /// true when the inline name editor consumed the event.
    pub(super) fn delete_text_input_forward(&mut self, cx: &mut Context<Self>) -> bool {
        if self.pending_name_input.is_some() {
            self.delete_name_text_forward(cx);
            return true;
        }
        if let Some(field) = self.focused_search_field_mut() {
            field.delete_forward();
            self.search_generation = None;
            self.after_input_changed(cx);
            return true;
        }
        false
    }

    pub(super) fn search_summary(&self) -> String {
        match &self.search_result {
            SearchResultState::Idle => t(self.language, Msg::StatusFindQueryEmpty).to_string(),
            SearchResultState::PendingPreview => t(self.language, Msg::SearchUpdating).to_string(),
            SearchResultState::InvalidPattern(error) => {
                tf(self.language, Msg::SearchInvalidRegex, &[error])
            }
            SearchResultState::NoMatches => t(self.language, Msg::StatusNoMatches).to_string(),
            SearchResultState::Ready => tf(
                self.language,
                Msg::StatusMatches,
                &[&self.search_matches.len().to_string()],
            ),
        }
    }

    pub(super) fn file_tree_summary(&self) -> String {
        let count = self
            .file_tree
            .as_ref()
            .map(|tree| tree.filtered_entries_limited(&self.file_tree_query, 0).1)
            .unwrap_or(0);
        let msg = if self.file_tree_query.is_empty() {
            Msg::StatusFilesVisible
        } else {
            Msg::StatusFileMatches
        };
        tf(self.language, msg, &[&count.to_string()])
    }
}
