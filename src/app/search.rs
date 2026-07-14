use super::*;

impl MarkionApp {
    pub(super) fn show_find(&mut self, _: &ShowFind, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.replace_visible = false;
        self.search_focus = Some(SearchField::Find);
        self.file_tree_query_focused = false;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        let tab = self.active_tab();
        let selected = tab.selected_range.clone();
        let text_owned = if self.search_query.is_empty() && !selected.is_empty() {
            Some(tab.document.text()[selected.clone()].to_string())
        } else {
            None
        };
        if let Some(text) = text_owned {
            self.search_query = text;
        }
        self.refresh_search_matches();
        self.status = self.search_summary().into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn show_replace(&mut self, _: &ShowReplace, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.replace_visible = true;
        self.search_focus = Some(SearchField::Find);
        self.file_tree_query_focused = false;
        self.input_marked_len = 0;
        let tab = self.active_tab();
        let selected = tab.selected_range.clone();
        let text_owned = if self.search_query.is_empty() && !selected.is_empty() {
            Some(tab.document.text()[selected.clone()].to_string())
        } else {
            None
        };
        if let Some(text) = text_owned {
            self.search_query = text;
        }
        self.refresh_search_matches();
        self.status = self.search_summary().into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn find_next(&mut self, _: &FindNext, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.refresh_search_matches();
        if self.search_matches.is_empty() {
            self.status = self.search_summary().into();
            cx.notify();
            return;
        }
        let next = self
            .current_search_index
            .map(|index| (index + 1) % self.search_matches.len())
            .unwrap_or_else(|| {
                self.search_matches
                    .iter()
                    .position(|found| found.range.start >= self.cursor_offset())
                    .unwrap_or(0)
            });
        self.select_search_match(next, cx);
    }

    pub(super) fn find_previous(
        &mut self,
        _: &FindPrevious,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_visible = true;
        self.refresh_search_matches();
        if self.search_matches.is_empty() {
            self.status = self.search_summary().into();
            cx.notify();
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
            .unwrap_or_else(|| {
                self.search_matches
                    .iter()
                    .rposition(|found| found.range.end <= self.cursor_offset())
                    .unwrap_or(self.search_matches.len() - 1)
            });
        self.select_search_match(previous, cx);
    }

    pub(super) fn replace_current_match(
        &mut self,
        _: &ReplaceCurrentMatch,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.refresh_search_matches();
        let range = self
            .current_search_index
            .and_then(|index| self.search_matches.get(index))
            .map(|found| found.range.clone())
            .or_else(|| {
                (!self.active_tab().selected_range.is_empty())
                    .then(|| self.active_tab().selected_range.clone())
            });
        let Some(range) = range else {
            self.status = t(self.language, Msg::StatusNoMatchSelected).into();
            cx.notify();
            return;
        };

        let snapshot = self.snapshot();
        let search_options = self.search_options();
        let replace_text = self.replace_text.clone();
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
                self.after_document_changed(cx);
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
        let snapshot = self.snapshot();
        let search_options = self.search_options();
        let replace_text = self.replace_text.clone();
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
        self.refresh_search_matches();
        self.status = if self.search_case_sensitive {
            "Case-sensitive find".into()
        } else {
            "Case-insensitive find".into()
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
        self.refresh_search_matches();
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

    pub(super) fn about(&mut self, _: &AboutMarkion, window: &mut Window, cx: &mut Context<Self>) {
        let detail = tf(
            self.language,
            Msg::DialogAboutDetail,
            &[env!("CARGO_PKG_VERSION"), GITHUB_REPO_URL],
        );
        let _ = window.prompt(
            PromptLevel::Info,
            self.tr(Msg::DialogAboutTitle),
            Some(&detail),
            &[PromptButton::ok(self.tr(Msg::DialogButtonOk))],
            cx,
        );
        self.status = t(self.language, Msg::StatusAboutMarkion).into();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn show_shortcuts(
        &mut self,
        _: &ShowShortcuts,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let _ = window.prompt(
            PromptLevel::Info,
            self.tr(Msg::DialogShortcutsTitle),
            Some(shortcut_reference(
                self.language,
                self.heading_menu_max_level,
            )),
            &[PromptButton::ok(self.tr(Msg::DialogButtonOk))],
            cx,
        );
        self.status = t(self.language, Msg::StatusKeyboardShortcuts).into();
        self.active_menu = None;
        cx.notify();
    }
}
