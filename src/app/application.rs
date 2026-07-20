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
        let preferences = load_app_preferences(&preferences_path).unwrap_or_default();
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
        Self {
            tabs: vec![initial_tab],
            active_tab: 0,
            focus_handle: cx.focus_handle(),
            active_menu: None,
            status: t(Language::default(), Msg::StatusReady).into(),
            confirming_close: false,
            allow_close: false,
            preferences_path,
            theme: AppTheme::from_name(&preferences.theme).unwrap_or(AppTheme::Paper),
            custom_theme,
            custom_themes,
            themes_dir,
            selected_theme_name,
            preferences_panel_open: false,
            shortcut_panel_open: false,
            shortcut_panel_focus: cx.focus_handle(),
            shortcut_platform: ShortcutPlatform::current(),
            shortcut_category: ShortcutCategory::Files,
            focus_mode: preferences.focus_mode,
            typewriter_mode: preferences.typewriter_mode,
            code_line_numbers: preferences.code_line_numbers,
            preview_adaptive_width: preferences.preview_adaptive_width,
            editor_font_size: preferences.editor_font_size,
            rendered_font_size: preferences.rendered_font_size,
            paragraph_spacing: preferences.paragraph_spacing,
            heading_menu_max_level: preferences.heading_menu_max_level,
            sync_scroll: preferences.sync_scroll,
            syncing_scroll: false,
            language: Language::from_code(&preferences.language),
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
            input_marked_len: 0,
            selected_tree_path: None,
            collapsed_tree_paths: HashSet::new(),
            file_tree_needs_initial_collapse: false,
            file_tree_context_menu: None,
            preview_context_menu: None,
            pending_name_input: None,
            search_visible: false,
            replace_visible: false,
            search_query: String::new(),
            replace_text: String::new(),
            search_case_sensitive: false,
            search_regex: false,
            search_focus: None,
            search_matches: Vec::new(),
            current_search_index: None,
            pane_scrollbar_drag: None,
            auto_save_preferences: preferences.auto_save,
            export_preferences: preferences.export.clone(),
            recovery_dir: default_recovery_dir(),
            highlight_cache: RefCell::new(HashMap::new()),
            diagram_cache: DiagramCache::new(DIAGRAM_CACHE_CAPACITY),
            math_cache: MathCache::new(MATH_CACHE_CAPACITY),
        }
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
        self.active_tab = index;
        // Selecting in another tab's preview must not leave a drag in progress
        // on the previous tab; clear the drag flag on all tabs for safety.
        for tab in &mut self.tabs {
            tab.preview_is_selecting = false;
            tab.clear_visual_caret_affinity();
            tab.finish_undo_capture();
            tab.marked_range = None;
        }
        self.refresh_search_matches();
        cx.notify();
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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Ok(files) = list_recovery_files(&self.recovery_dir) else {
            return;
        };
        let Some(path) = files.last().cloned() else {
            return;
        };

        let detail = tf(
            self.language,
            Msg::DialogRestoreDetail,
            &[&path.display().to_string()],
        );
        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogRestoreTitle),
            Some(&detail),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonRestore)),
                PromptButton::cancel(self.tr(Msg::DialogButtonDiscard)),
            ],
            cx,
        );

        self.status = t(self.language, Msg::StatusRecoveryAvailable).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let restore = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                if restore {
                    match load_recovery_file(&path) {
                        Ok(recovery) => {
                            app.open_in_new_tab(
                                MarkdownDocument::recovered(recovery.text, recovery.original_path),
                                cx,
                            );
                            let _ = delete_recovery_file(&path);
                            app.status = t(app.language, Msg::StatusRecoveredDocument).into();
                        }
                        Err(err) => {
                            app.status =
                                tf(app.language, Msg::StatusRecoveryFailed, &[&err.to_string()])
                                    .into();
                        }
                    }
                } else {
                    let _ = delete_recovery_file(&path);
                    app.status = t(app.language, Msg::StatusRecoveryDiscarded).into();
                }
                cx.notify();
            });
        })
        .detach();
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
                self.set_workspace_root(path);
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

    pub(super) fn set_workspace_root(&mut self, root: PathBuf) {
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
            });
        }

        self.workspace_root = root;
    }

    pub(super) fn update_workspace_root_from_document(&mut self, cx: &mut Context<Self>) {
        let Some(document_path) = self.active_tab().document.path().map(Path::to_path_buf) else {
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

        self.set_workspace_root(next_root);
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
        cx.spawn(async move |this, cx| {
            // Run the filesystem traversal off the main thread.
            let scanned = cx
                .background_executor()
                .spawn(async move { FileTree::scan(&scan_root) })
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
        if let Some(recovery) = self.active_tab_mut().last_recovery_file.take() {
            let _ = delete_recovery_file(recovery);
        }
    }

    /// Open `document` in a brand-new tab and make it active. Used by new
    /// untitled tabs and crash-recovery restore; filesystem-backed opens should
    /// go through the path helpers so already-open files can reuse their tab.
    pub(super) fn open_in_new_tab(&mut self, document: MarkdownDocument, cx: &mut Context<Self>) {
        let tab = self.editor_tab_for_document(document);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.refresh_search_matches();
        cx.notify();
    }

    pub(super) fn editor_tab_for_document(&self, document: MarkdownDocument) -> EditorTab {
        let mut tab = EditorTab::new(document);
        tab.line_height = px(self.typography_metrics().editor_line_height);
        tab
    }

    /// Replace the active tab's document in place: discard its recovery file,
    /// reset its selection/undo/scroll state, and load `document`. Used by
    /// File→New and File→Open (single-tab behaviour continuity).
    pub(super) fn replace_active_tab(
        &mut self,
        document: MarkdownDocument,
        cx: &mut Context<Self>,
    ) {
        let tab = self.active_tab_mut();
        if let Some(recovery) = tab.last_recovery_file.take() {
            let _ = delete_recovery_file(recovery);
        }
        tab.document = document;
        tab.selected_range = 0..0;
        tab.selection_reversed = false;
        tab.marked_range = None;
        tab.undo_stack.clear();
        tab.redo_stack.clear();
        tab.editor_scroll = ScrollHandle::new();
        tab.reset_preview_list();
        tab.last_lines.clear();
        tab.line_offsets.clear();
        tab.line_heights.clear();
        tab.last_bounds = None;
        self.refresh_search_matches();
        cx.notify();
    }

    pub(super) fn open_tree_file(
        &mut self,
        path: PathBuf,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // With multi-document tabs, opening from the file tree creates a new
        // tab for an unopened file rather than replacing (and risking loss of)
        // the active document, so no dirty-guard prompt is needed here.
        self.open_tree_file_confirmed(path, cx);
    }

    pub(super) fn open_tree_file_confirmed(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.open_file_in_new_tab_from_path(path, cx);
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
                if tab.autosave_generation != generation || !tab.document.is_dirty() {
                    return;
                }

                let tab = &mut app.tabs[active_index];
                match tab.document.autosave(&recovery_dir) {
                    Ok(AutosaveOutcome::NoChanges) => {}
                    Ok(AutosaveOutcome::SavedFile(path)) => {
                        if let Some(recovery) = tab.last_recovery_file.take() {
                            let _ = delete_recovery_file(recovery);
                        }
                        app.status = tf(
                            app.language,
                            Msg::StatusAutoSaved,
                            &[&path.display().to_string()],
                        )
                        .into();
                    }
                    Ok(AutosaveOutcome::SavedRecovery(path)) => {
                        if let Some(previous) = tab.last_recovery_file.replace(path.clone())
                            && previous != path
                        {
                            let _ = delete_recovery_file(previous);
                        }
                        app.status = tf(
                            app.language,
                            Msg::StatusRecoverySaved,
                            &[&path.display().to_string()],
                        )
                        .into();
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "auto-save failed");
                        app.status =
                            tf(app.language, Msg::StatusAutoSaveFailed, &[&err.to_string()]).into();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn search_options(&self) -> SearchOptions {
        SearchOptions {
            query: self.search_query.clone(),
            case_sensitive: self.search_case_sensitive,
            regex: self.search_regex,
        }
    }

    pub(super) fn refresh_search_matches(&mut self) {
        // Skip the full-document regex scan entirely when the find bar is
        // closed: matches are recomputed on demand in show_find/show_replace,
        // so there is no point paying for it on every keystroke while typing.
        if !self.search_visible || self.search_query.is_empty() {
            if !self.search_visible {
                self.search_matches.clear();
                self.current_search_index = None;
            }
            return;
        }

        match self
            .active_tab()
            .document
            .find_matches(&self.search_options())
        {
            Ok(matches) => {
                self.search_matches = matches;
                self.current_search_index = self
                    .current_search_index
                    .filter(|index| *index < self.search_matches.len());
            }
            Err(err) => {
                self.search_matches.clear();
                self.current_search_index = None;
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
        self.refresh_search_matches();
        cx.notify();
    }

    pub(super) fn select_search_match(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(found) = self.search_matches.get(index).cloned() {
            self.current_search_index = Some(index);
            let tab = self.active_tab_mut();
            tab.selected_range = found.range.clone();
            tab.selection_reversed = false;
            tab.marked_range = None;
            tab.visual_cursor_reveal_pending = true;
            tab.visual_caret_bounds = None;
            self.scroll_editor_to_offset(found.range.start);
            self.status = self.trf(
                Msg::StatusMatchPosition,
                &[
                    &(index + 1).to_string(),
                    &self.search_matches.len().to_string(),
                    &found.line.to_string(),
                    &found.column.to_string(),
                ],
            );
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

    pub(super) fn scroll_editor_to_offset(&self, offset: usize) {
        self.active_tab().scroll_editor_to_offset(offset);
    }

    pub(super) fn center_cursor_if_typewriter(&self) {
        if self.typewriter_mode {
            self.active_tab()
                .scroll_editor_typewriter_to_offset(self.active_tab().cursor_offset());
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
            heading_menu_max_level: self.heading_menu_max_level,
            sync_scroll: self.sync_scroll,
            sidebar_visible: self.sidebar_visible,
            sidebar_tab: self.sidebar_tab,
            language: self.language.code().to_string(),
            auto_save: self.auto_save_preferences,
            export: self.export_preferences.clone(),
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
        self.status = self.trf(Msg::StatusTheme, &[&self.theme_label()]);
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn persist_preferences(&mut self) {
        if let Err(err) = save_app_preferences(&self.preferences_path, &self.current_preferences())
        {
            self.status = self.trf(Msg::StatusPreferencesSaveFailed, &[&err.to_string()]);
        }
    }

    pub(super) fn active_search_text_mut(&mut self) -> Option<&mut String> {
        match self.search_focus {
            Some(SearchField::Find) => Some(&mut self.search_query),
            Some(SearchField::Replace) => Some(&mut self.replace_text),
            None => None,
        }
    }

    pub(super) fn has_text_input_focus(&self) -> bool {
        self.pending_name_input.is_some()
            || self.file_tree_query_focused
            || self.search_focus.is_some()
    }

    pub(super) fn active_input_text_mut(&mut self) -> Option<&mut String> {
        if self.pending_name_input.is_some() {
            self.pending_name_input
                .as_mut()
                .map(|pending| &mut pending.buffer)
        } else if self.file_tree_query_focused {
            Some(&mut self.file_tree_query)
        } else {
            self.active_search_text_mut()
        }
    }

    pub(super) fn after_input_changed(&mut self, cx: &mut Context<Self>) {
        if self.pending_name_input.is_some() {
            // The name prompt edits a single buffer; no search/tree filtering
            // runs while it is open.
            self.status = t(self.language, Msg::StatusNamingEntry).into();
        } else if self.file_tree_query_focused {
            self.status = self.file_tree_summary().into();
        } else {
            self.refresh_search_matches();
            self.status = self.search_summary().into();
        }
        cx.notify();
    }

    /// Insert text into the focused redirected field, first removing any
    /// trailing IME composition. `keep_marked` records the new text as the
    /// active composition (still being edited) instead of committing it.
    pub(super) fn insert_redirected_text(
        &mut self,
        text: &str,
        keep_marked: bool,
        cx: &mut Context<Self>,
    ) {
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

    pub(super) fn push_text_input(&mut self, text: &str, cx: &mut Context<Self>) {
        self.insert_redirected_text(text, false, cx);
    }

    pub(super) fn pop_text_input(&mut self, cx: &mut Context<Self>) -> bool {
        self.input_marked_len = 0;
        if let Some(target) = self.active_input_text_mut() {
            target.pop();
            self.after_input_changed(cx);
            true
        } else {
            false
        }
    }

    pub(super) fn search_summary(&self) -> String {
        if self.search_query.is_empty() {
            t(self.language, Msg::StatusFindQueryEmpty).to_string()
        } else if self.search_matches.is_empty() {
            t(self.language, Msg::StatusNoMatches).to_string()
        } else {
            tf(
                self.language,
                Msg::StatusMatches,
                &[&self.search_matches.len().to_string()],
            )
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
