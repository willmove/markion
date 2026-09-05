use super::*;

impl MarkionApp {
    pub(super) fn show_find(&mut self, _: &ShowFind, _: &mut Window, cx: &mut Context<Self>) {
        self.dismiss_visual_block_menu();
        self.search_visible = true;
        self.search_form = SearchPanelForm::Find;
        self.replace_visible = false;
        self.search_focus = Some(SearchField::Find);
        self.search_control_focus = Some(SearchOverlayControl::FindField);
        self.file_tree_query_focused = false;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        let tab = self.active_tab();
        let selected = tab.safe_selected_range();
        let text_owned = if !matches!(self.view_mode, ViewMode::Read)
            && self.search_query.buffer.is_empty()
            && !selected.is_empty()
        {
            Some(tab.document.text()[selected.clone()].to_string())
        } else {
            None
        };
        if let Some(text) = text_owned {
            self.search_query.set_text(text);
        }
        self.search_generation = None;
        self.refresh_search_matches();
        if let Some(index) = self.current_search_index {
            self.select_search_match(index, cx);
        }
        self.status = self.search_summary().into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn show_replace(&mut self, _: &ShowReplace, _: &mut Window, cx: &mut Context<Self>) {
        self.dismiss_visual_block_menu();
        self.search_visible = true;
        self.search_form = SearchPanelForm::Replace;
        self.replace_visible = !matches!(self.view_mode, ViewMode::Read);
        self.search_focus = Some(SearchField::Find);
        self.search_control_focus = Some(SearchOverlayControl::FindField);
        self.file_tree_query_focused = false;
        self.input_marked_len = 0;
        let tab = self.active_tab();
        let selected = tab.safe_selected_range();
        let text_owned = if !matches!(self.view_mode, ViewMode::Read)
            && self.search_query.buffer.is_empty()
            && !selected.is_empty()
        {
            Some(tab.document.text()[selected.clone()].to_string())
        } else {
            None
        };
        if let Some(text) = text_owned {
            self.search_query.set_text(text);
        }
        self.search_generation = None;
        self.refresh_search_matches();
        if let Some(index) = self.current_search_index {
            self.select_search_match(index, cx);
        }
        self.status = self.search_summary().into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn find_next(&mut self, _: &FindNext, _: &mut Window, cx: &mut Context<Self>) {
        let was_visible = self.search_visible;
        self.search_visible = true;
        self.refresh_search_matches();
        if self.search_matches.is_empty() {
            self.status = self.search_summary().into();
            cx.notify();
            return;
        }
        if !was_visible {
            if let Some(index) = self.current_search_index {
                self.select_search_match(index, cx);
            }
            return;
        }
        let next = self
            .current_search_index
            .map(|index| (index + 1) % self.search_matches.len())
            .unwrap_or(0);
        self.select_search_match(next, cx);
    }

    pub(super) fn find_previous(
        &mut self,
        _: &FindPrevious,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let was_visible = self.search_visible;
        self.search_visible = true;
        self.refresh_search_matches();
        if self.search_matches.is_empty() {
            self.status = self.search_summary().into();
            cx.notify();
            return;
        }
        if !was_visible {
            if let Some(index) = self.current_search_index {
                self.select_search_match(index, cx);
            }
            return;
        }
        let previous = self
            .current_search_index
            .map(|index| {
                if index == 0 {
                    self.search_matches.len() - 1
                } else {
                    index - 1
                }
            })
            .unwrap_or(self.search_matches.len() - 1);
        self.select_search_match(previous, cx);
    }

    pub(super) fn replace_current_match(
        &mut self,
        _: &ReplaceCurrentMatch,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.view_mode, ViewMode::Read) {
            self.status = t(self.language, Msg::SearchReadReplaceUnavailable).into();
            cx.notify();
            return;
        }
        self.refresh_search_matches();
        let range = self
            .current_search_index
            .and_then(|index| self.search_matches.get(index))
            .and_then(|target| match target {
                SearchTarget::Source(found) => Some(found.range.clone()),
                SearchTarget::ReadPreview(_) => None,
            });
        let Some(range) = range else {
            self.status = t(self.language, Msg::StatusNoMatchSelected).into();
            cx.notify();
            return;
        };

        let snapshot = self.snapshot();
        let search_options = self.search_options();
        let replace_text = self.replace_text.buffer.clone();
        let tab = self.active_tab_mut();
        let result = tab
            .document
            .replace_current_match(range, &search_options, &replace_text);
        match result {
            Ok(result) if result.replacements > 0 => {
                self.commit_undo_snapshot(snapshot);
                let tab = self.active_tab_mut();
                if let Some(range) = result.selected_range {
                    tab.selected_range = range;
                }
                tab.selection_reversed = false;
                tab.marked_range = None;
                self.search_generation = None;
                self.after_document_changed(cx);
                if let Some(index) = self.current_search_index {
                    self.select_search_match(index, cx);
                }
                self.status = t(self.language, Msg::StatusReplacedCurrent).into();
            }
            Ok(_) => {
                self.status = t(self.language, Msg::StatusNoMatchSelected).into();
            }
            Err(err) => {
                self.status = self.trf(Msg::StatusReplaceFailed, &[err.message()]);
            }
        }
        cx.notify();
    }

    pub(super) fn replace_all_matches(
        &mut self,
        _: &ReplaceAllMatches,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.view_mode, ViewMode::Read) {
            self.status = t(self.language, Msg::SearchReadReplaceUnavailable).into();
            cx.notify();
            return;
        }
        if !matches!(self.search_result, SearchResultState::Ready) {
            self.status = t(self.language, Msg::StatusNoMatchesToReplace).into();
            cx.notify();
            return;
        }
        let snapshot = self.snapshot();
        let search_options = self.search_options();
        let replace_text = self.replace_text.buffer.clone();
        let tab = self.active_tab_mut();
        let result = tab
            .document
            .replace_all_matches(&search_options, &replace_text);
        match result {
            Ok(result) if result.replacements > 0 => {
                self.commit_undo_snapshot(snapshot);
                let tab = self.active_tab_mut();
                tab.selected_range = 0..0;
                tab.selection_reversed = false;
                tab.marked_range = None;
                self.search_generation = None;
                self.after_document_changed(cx);
                self.status = self.trf(
                    Msg::StatusReplacedMatches,
                    &[&result.replacements.to_string()],
                );
            }
            Ok(_) => {
                self.status = t(self.language, Msg::StatusNoMatchesToReplace).into();
            }
            Err(err) => {
                self.status = self.trf(Msg::StatusReplaceFailed, &[err.message()]);
            }
        }
        cx.notify();
    }

    pub(super) fn toggle_find_case_sensitive(
        &mut self,
        _: &ToggleFindCaseSensitive,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_case_sensitive = !self.search_case_sensitive;
        self.search_generation = None;
        self.refresh_search_matches();
        if let Some(index) = self.current_search_index {
            self.select_search_match(index, cx);
        }
        self.status = if self.search_case_sensitive {
            t(self.language, Msg::StatusCaseSensitiveFind).into()
        } else {
            t(self.language, Msg::StatusCaseInsensitiveFind).into()
        };
        cx.notify();
    }

    pub(super) fn toggle_find_regex(
        &mut self,
        _: &ToggleFindRegex,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_regex = !self.search_regex;
        self.search_generation = None;
        self.refresh_search_matches();
        if let Some(index) = self.current_search_index {
            self.select_search_match(index, cx);
        }
        self.status = t(
            self.language,
            if self.search_regex {
                Msg::StatusRegexFind
            } else {
                Msg::StatusLiteralFind
            },
        )
        .into();
        cx.notify();
    }

    pub(super) fn apply_language(&mut self, language: Language, cx: &mut Context<Self>) {
        if self.language == language {
            self.active_menu = None;
            return;
        }
        self.language = language;
        self.persist_preferences();
        // Native (OS) menus were installed with English labels at startup;
        // re-translate them so the menu bar matches the new language.
        install_menus(self.language, self.heading_menu_max_level, cx);
        self.status = t(self.language, Msg::StatusLanguageSet).into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn about(&mut self, _: &AboutMarkion, _: &mut Window, cx: &mut Context<Self>) {
        self.about_dialog_open = true;
        self.status = t(self.language, Msg::StatusAboutMarkion).into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn close_about_dialog(&mut self, cx: &mut Context<Self>) {
        if !self.about_dialog_open {
            return;
        }
        self.about_dialog_open = false;
        cx.notify();
    }

    pub(super) fn show_markdown_reference(
        &mut self,
        _: &ShowMarkdownReference,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.markdown_reference_open = true;
        self.markdown_reference_scroll
            .set_offset(point(px(0.), px(0.)));
        self.status = t(self.language, Msg::StatusMarkdownReference).into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn close_markdown_reference(&mut self, cx: &mut Context<Self>) {
        if !self.markdown_reference_open {
            return;
        }
        self.markdown_reference_open = false;
        self.markdown_reference_scroll
            .set_offset(point(px(0.), px(0.)));
        cx.notify();
    }

    pub(super) fn open_about_link(&mut self, link: AboutLink, cx: &mut Context<Self>) {
        cx.open_url(link.url());
    }

    pub(super) fn open_markdown_tutorial_link(&mut self, cx: &mut Context<Self>) {
        cx.open_url(super::kenhuang_markdown_tutorial_url(self.language));
    }

    pub(super) fn report_issue(
        &mut self,
        _: &ReportIssue,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.open_url(GITHUB_ISSUES_URL);
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn open_online_docs(
        &mut self,
        _: &OpenOnlineDocs,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.open_url(GITHUB_DOCS_URL);
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn show_shortcuts(
        &mut self,
        _: &ShowShortcuts,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The shortcut reference lives in the Preferences panel now, so F1
        // opens Preferences directly on its Shortcuts tab.
        self.preferences_tab = PreferencesTab::Shortcuts;
        self.shortcut_platform = ShortcutPlatform::current();
        self.shortcut_category = ShortcutCategory::Files;
        if self.shortcut_capture.take().is_some() {
            self.rebind_keys(cx);
        }
        self.preferences_panel_open = true;
        self.dismiss_visual_block_menu();
        self.file_tree_context_menu = None;
        self.preview_context_menu = None;
        self.tab_context_menu = None;
        self.status = t(self.language, Msg::StatusKeyboardShortcuts).into();
        self.active_menu = None;
        window.focus(&self.preferences_panel_focus);
        cx.notify();
    }

    pub(super) fn select_preferences_tab(&mut self, tab: PreferencesTab, cx: &mut Context<Self>) {
        if self.preferences_tab == tab {
            return;
        }
        self.preferences_tab = tab;
        // Leaving the Shortcuts tab must not strand a capturing row with the
        // keymap cleared.
        if self.shortcut_capture.take().is_some() {
            self.rebind_keys(cx);
        }
        // Entering the Export tab refreshes the pandoc availability line.
        if tab == PreferencesTab::Export {
            self.refresh_pandoc_availability(cx);
        }
        cx.notify();
    }

    pub(super) fn select_shortcut_platform(
        &mut self,
        platform: ShortcutPlatform,
        cx: &mut Context<Self>,
    ) {
        if self.shortcut_platform != platform {
            self.shortcut_platform = platform;
            cx.notify();
        }
    }

    pub(super) fn select_shortcut_category(
        &mut self,
        category: ShortcutCategory,
        cx: &mut Context<Self>,
    ) {
        if self.shortcut_category != category {
            self.shortcut_category = category;
            cx.notify();
        }
    }

    /// Close the Preferences panel from a keyboard path (Escape), restoring
    /// editor focus and dropping any in-flight shortcut capture.
    pub(super) fn close_preferences_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.preferences_panel_open {
            return;
        }
        if self.shortcut_capture.take().is_some() {
            self.rebind_keys(cx);
        }
        self.preferences_panel_open = false;
        window.focus(&self.focus_handle);
        cx.notify();
    }
}
