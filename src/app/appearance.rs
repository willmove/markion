use super::*;

impl MarkionApp {
    pub(super) fn cycle_theme(&mut self, _: &CycleTheme, _: &mut Window, cx: &mut Context<Self>) {
        // Cycle through the full combined list (built-ins + user themes) so the
        // shortcut visits every theme the Preferences panel exposes.
        let themes = self.available_themes();
        if themes.is_empty() {
            return;
        }
        let current_index = themes
            .iter()
            .position(|theme| theme.name.eq_ignore_ascii_case(&self.selected_theme_name))
            .unwrap_or(0);
        let next = themes[(current_index + 1) % themes.len()].name.clone();
        self.apply_theme_by_name(&next, cx);
        self.active_menu = None;
    }

    pub(super) fn theme_label(&self) -> String {
        let name = self.active_theme_definition().name;
        let is_custom = self.custom_themes.iter().any(|theme| theme.name == name);
        if is_custom {
            tf(self.language, Msg::CustomThemeLabel, &[&name])
        } else {
            name
        }
    }

    pub(super) fn show_preferences(
        &mut self,
        _: &ShowPreferences,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The Preferences panel is rendered in-app (see `preferences_panel_view`),
        // so opening it is just a flag flip. Refresh the custom-theme list so a
        // theme file dropped into the themes dir since launch shows up.
        self.ensure_sample_custom_theme();
        self.custom_themes = list_theme_definitions(&self.themes_dir).unwrap_or_default();
        self.preferences_tab = PreferencesTab::General;
        if self.shortcut_capture.take().is_some() {
            self.rebind_keys(cx);
        }
        self.preferences_panel_open = true;
        self.active_menu = None;
        self.dismiss_visual_block_menu();
        window.focus(&self.preferences_panel_focus);
        cx.notify();
    }

    pub(super) fn ensure_sample_custom_theme(&mut self) {
        if self.themes_dir.exists() {
            return;
        }
        let sample = ThemeDefinition {
            name: "Midnight".to_string(),
            is_dark: true,
            colors: ThemeColors {
                app_bg: 0x10131a,
                panel_bg: 0x171b24,
                surface_bg: 0x0f1720,
                text: 0xe5edf5,
                muted: 0x91a4b7,
                border: 0x2b3544,
                active_bg: 0x23304a,
                active_text: 0x9ec5ff,
            },
        };
        let path = self.themes_dir.join("midnight.toml");
        if let Err(err) = save_theme_definition(path, &sample) {
            self.status = self.trf(Msg::StatusSampleThemeSaveFailed, &[&err.to_string()]);
        }
    }

    pub(super) fn reset_preferences(
        &mut self,
        _: &ResetPreferences,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogResetTitle),
            Some(self.tr(Msg::DialogResetDetail)),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonReset)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );
        self.active_menu = None;
        self.status = t(self.language, Msg::StatusWaitingPreferenceResetConfirm).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                if confirmed {
                    let preferences = AppPreferences::default();
                    app.theme = AppTheme::from_name(&preferences.theme).unwrap_or(AppTheme::Paper);
                    app.custom_theme = None;
                    app.selected_theme_name = preferences.theme.clone();
                    app.preferences_panel_open = false;
                    app.focus_mode = preferences.focus_mode;
                    app.typewriter_mode = preferences.typewriter_mode;
                    app.code_line_numbers = preferences.code_line_numbers;
                    app.preview_adaptive_width = preferences.preview_adaptive_width;
                    app.editor_font_size = preferences.editor_font_size;
                    app.rendered_font_size = preferences.rendered_font_size;
                    app.paragraph_spacing = preferences.paragraph_spacing;
                    app.refresh_typography_measurements(true, true);
                    app.heading_menu_max_level = preferences.heading_menu_max_level;
                    app.sync_scroll = preferences.sync_scroll;
                    app.sidebar_visible = preferences.sidebar_visible;
                    app.sidebar_tab = preferences.sidebar_tab;
                    // Reset also restores the default interface language.
                    app.language = Language::from_code(&preferences.language);
                    app.clear_shortcut_overrides(cx);
                    app.persist_preferences();
                    install_menus(app.language, app.heading_menu_max_level, cx);
                    app.status = t(app.language, Msg::StatusPreferencesReset).into();
                } else {
                    app.status = t(app.language, Msg::StatusPreferenceResetCanceled).into();
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn toggle_focus_mode(
        &mut self,
        _: &ToggleFocusMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_mode = !self.focus_mode;
        self.status = t(
            self.language,
            if self.focus_mode {
                Msg::StatusFocusModeOn
            } else {
                Msg::StatusFocusModeOff
            },
        )
        .into();
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn toggle_typewriter_mode(
        &mut self,
        _: &ToggleTypewriterMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.typewriter_mode = !self.typewriter_mode;
        self.center_cursor_if_typewriter();
        self.status = t(
            self.language,
            if self.typewriter_mode {
                Msg::StatusTypewriterModeOn
            } else {
                Msg::StatusTypewriterModeOff
            },
        )
        .into();
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn toggle_code_line_numbers(
        &mut self,
        _: &ToggleCodeLineNumbers,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.code_line_numbers = !self.code_line_numbers;
        self.status = t(
            self.language,
            if self.code_line_numbers {
                Msg::StatusCodeLineNumbersOn
            } else {
                Msg::StatusCodeLineNumbersOff
            },
        )
        .into();
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn toggle_preview_adaptive_width(&mut self, cx: &mut Context<Self>) {
        self.preview_adaptive_width = !self.preview_adaptive_width;
        self.status = t(
            self.language,
            if self.preview_adaptive_width {
                Msg::StatusPreviewAdaptiveWidthOn
            } else {
                Msg::StatusPreviewAdaptiveWidthOff
            },
        )
        .into();
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_editor_font_size(&mut self, value: i64, cx: &mut Context<Self>) {
        let value = normalize_editor_font_size(value);
        if self.editor_font_size == value {
            return;
        }
        self.editor_font_size = value;
        self.refresh_typography_measurements(true, false);
        self.center_cursor_if_typewriter();
        self.status = self.trf(Msg::StatusEditorFontSize, &[&format!("{value}px")]);
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_rendered_font_size(&mut self, value: i64, cx: &mut Context<Self>) {
        let value = normalize_rendered_font_size(value);
        if self.rendered_font_size == value {
            return;
        }
        self.rendered_font_size = value;
        self.refresh_typography_measurements(false, true);
        self.status = self.trf(Msg::StatusRenderedFontSize, &[&format!("{value}px")]);
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_paragraph_spacing(&mut self, value: i64, cx: &mut Context<Self>) {
        let value = normalize_paragraph_spacing(value);
        if self.paragraph_spacing == value {
            return;
        }
        self.paragraph_spacing = value;
        self.refresh_typography_measurements(false, true);
        self.status = self.trf(Msg::StatusParagraphSpacing, &[&format!("{value}px")]);
        self.persist_preferences();
        cx.notify();
    }

    /// Invalidates only presentation measurements affected by typography.
    /// Document versions, derived Markdown caches, highlights, shared text,
    /// selection/history, and the list block `Arc`s remain untouched.
    pub(super) fn refresh_typography_measurements(
        &mut self,
        editor_changed: bool,
        rendered_changed: bool,
    ) {
        let metrics = self.typography_metrics();
        for tab in &mut self.tabs {
            let Some(tab) = tab.document_tab_mut() else {
                continue;
            };
            if editor_changed {
                tab.invalidate_source_layout();
                tab.line_height = px(metrics.editor_line_height);
                *tab.measured_height_cache.borrow_mut() = None;
            }
            if rendered_changed {
                invalidate_list_measurements_around_scroll_anchor(&tab.preview_list);
                invalidate_list_measurements_around_scroll_anchor(&tab.visual_list);
                tab.visual_caret_bounds = None;
                tab.visual_marked_range_bounds = None;
                tab.visual_input_bounds = None;
                tab.visual_navigation_snapshots.clear();
                tab.visual_navigation_snapshot_ids.clear();
                tab.sync_scroll_state.invalidate_geometry();
            }
        }
    }

    pub(super) fn toggle_sync_scroll(&mut self, cx: &mut Context<Self>) {
        self.sync_scroll = !self.sync_scroll;
        // Seed from the current pane positions on the next Split frame; toggling
        // the preference never yanks either pane immediately.
        for tab in &mut self.tabs {
            let Some(tab) = tab.document_tab_mut() else {
                continue;
            };
            tab.sync_scroll_state.reset();
        }
        self.status = t(
            self.language,
            if self.sync_scroll {
                Msg::StatusSyncScrollOn
            } else {
                Msg::StatusSyncScrollOff
            },
        )
        .into();
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn toggle_show_hidden_files(&mut self, cx: &mut Context<Self>) {
        self.show_hidden_files = !self.show_hidden_files;
        self.status = t(
            self.language,
            if self.show_hidden_files {
                Msg::StatusShowHiddenFilesOn
            } else {
                Msg::StatusShowHiddenFilesOff
            },
        )
        .into();
        self.persist_preferences();
        // Re-scan under the new visibility rule so hidden entries appear or
        // disappear on the next render, before notifying the view.
        self.refresh_file_tree(cx);
        cx.notify();
    }

    pub(super) fn mark_sync_scroll_driver(&mut self, driver: PaneScrollTarget) {
        if let Some(tab) = self.active_tab_mut().document_tab_mut() {
            tab.sync_scroll_state.mark_driver(driver);
        }
    }

    /// Source-mapped Split Preview coupling. Each render observes the two
    /// independent scroll states, resolves one driver, maps its top content
    /// edge through preview block source ranges, and writes only the follower.
    pub(super) fn reconcile_sync_scroll(&mut self, cx: &mut Context<Self>) {
        if !sync_scroll_is_active(self.view_mode, self.sync_scroll) {
            return;
        }
        let Some(tab) = self.tabs[self.active_tab].document_tab_mut() else {
            return;
        };
        let editor_max = f32::from(tab.editor_scroll.max_offset().height.max(px(0.)));
        let preview_max = f32::from(
            tab.preview_list
                .max_offset_for_scrollbar()
                .height
                .max(px(0.)),
        );
        let editor_offset = f32::from(-tab.editor_scroll.offset().y)
            .max(0.)
            .min(editor_max);
        let preview_offset = f32::from(-tab.preview_list.scroll_px_offset_for_scrollbar().y)
            .max(0.)
            .min(preview_max);
        let logical_top = tab.preview_list.logical_scroll_top();
        let preview_position = SyncPreviewPosition {
            item_ix: logical_top.item_ix,
            offset_in_item: f32::from(logical_top.offset_in_item),
        };

        // A newly measured/reflowed follower may normalize the target once
        // after our render-time write. Retry that expected target once before
        // raw-offset driver detection can mistake the normalization for input.
        if tab.sync_scroll_state.driver_hint.is_none()
            && !tab.sync_scroll_state.expected_follower_retried
            && let Some(expected) = tab.sync_scroll_state.expected_follower
        {
            let mismatched = match expected {
                ExpectedSyncFollower::Editor(expected_offset) => {
                    (editor_offset - expected_offset).abs() > SYNC_SCROLL_PIXEL_EPSILON
                }
                ExpectedSyncFollower::Preview(expected_position) => {
                    preview_position.item_ix != expected_position.item_ix
                        || (preview_position.offset_in_item - expected_position.offset_in_item)
                            .abs()
                            > SYNC_SCROLL_PIXEL_EPSILON
                }
            };
            if mismatched {
                match expected {
                    ExpectedSyncFollower::Editor(expected_offset) => {
                        tab.editor_scroll
                            .set_offset(point(px(0.), px(-expected_offset)));
                        tab.sync_scroll_state.last_editor_offset = Some(expected_offset);
                    }
                    ExpectedSyncFollower::Preview(expected_position) => {
                        tab.preview_list.scroll_to(gpui::ListOffset {
                            item_ix: expected_position.item_ix,
                            offset_in_item: px(expected_position.offset_in_item),
                        });
                        tab.sync_scroll_state.last_preview_position = Some(expected_position);
                    }
                }
                tab.sync_scroll_state.expected_follower_retried = true;
                cx.notify();
                return;
            }
        }

        // A coarse jump to an unmeasured virtual row is refined after the row
        // has gone through one layout. New user input cancels the pending step.
        if tab.sync_scroll_state.driver_hint.is_some() {
            tab.sync_scroll_state.pending_preview_refinement = None;
        } else if let Some(pending) = tab.sync_scroll_state.pending_preview_refinement {
            let valid = pending.version == tab.document.version()
                && tab.preview_reflects_version == Some(pending.version)
                && pending.item_ix < tab.preview_list_blocks.len();
            if !valid {
                tab.sync_scroll_state.pending_preview_refinement = None;
            } else if let Some(bounds) = tab.preview_list.bounds_for_item(pending.item_ix) {
                tab.sync_scroll_state.pending_preview_refinement = None;
                if bounds.size.height > px(0.) {
                    let offset = (f32::from(bounds.size.height) * pending.progress).max(0.);
                    tab.preview_list.scroll_to(gpui::ListOffset {
                        item_ix: pending.item_ix,
                        offset_in_item: px(offset),
                    });
                    let actual = SyncPreviewPosition {
                        item_ix: pending.item_ix,
                        offset_in_item: offset,
                    };
                    tab.sync_scroll_state.last_editor_offset = Some(editor_offset);
                    tab.sync_scroll_state.last_preview_position = Some(actual);
                    tab.sync_scroll_state.expected_follower =
                        Some(ExpectedSyncFollower::Preview(actual));
                    tab.sync_scroll_state.expected_follower_retried = false;
                    cx.notify();
                    return;
                }
            } else {
                // The coarse `scroll_to` makes this row the next layout's
                // anchor. Schedule exactly that post-layout refinement frame.
                cx.notify();
                return;
            }
        }

        let Some(driver) =
            select_sync_scroll_driver(&mut tab.sync_scroll_state, editor_offset, preview_position)
        else {
            return;
        };

        if (driver == PaneScrollTarget::Editor && editor_max <= 1.)
            || (driver == PaneScrollTarget::Preview && preview_max <= 1.)
        {
            return;
        }
        let version = tab.document.version();
        if !tab.source_layout_is_current()
            || !sync_scroll_mapping_is_current(
                version,
                tab.source_layout_key,
                tab.preview_reflects_version,
                !tab.preview_list_blocks.is_empty(),
            )
        {
            tab.sync_scroll_state.deferred_driver = Some(driver);
            return;
        }

        match driver {
            PaneScrollTarget::Editor => {
                let source_offset = if editor_offset >= editor_max - SYNC_SCROLL_PIXEL_EPSILON {
                    tab.document.text().len()
                } else {
                    let Some(offset) = tab.source_offset_for_content_y(editor_offset) else {
                        tab.sync_scroll_state.deferred_driver = Some(driver);
                        return;
                    };
                    offset
                };
                let Some(anchor) = preview_anchor_for_source_offset(
                    &tab.preview_list_blocks,
                    source_offset,
                    tab.document.text().len(),
                ) else {
                    return;
                };
                match anchor {
                    PreviewScrollAnchor::Start => {
                        tab.preview_list.scroll_to(gpui::ListOffset::default());
                    }
                    PreviewScrollAnchor::End => {
                        tab.preview_list
                            .set_offset_from_scrollbar(point(px(0.), px(-preview_max)));
                    }
                    PreviewScrollAnchor::Block { item_ix } => {
                        let range = tab.preview_list_blocks[item_ix].source_range();
                        let Some(start_y) = tab.source_content_y_for_offset(range.start) else {
                            tab.sync_scroll_state.deferred_driver = Some(driver);
                            return;
                        };
                        let Some(mut end_y) = tab.source_content_y_for_offset(range.end) else {
                            tab.sync_scroll_state.deferred_driver = Some(driver);
                            return;
                        };
                        end_y = end_y.max(start_y + f32::from(tab.line_height));
                        let progress = if source_offset < range.start {
                            0.
                        } else {
                            sync_interval_progress(editor_offset, start_y, end_y)
                        };
                        if let Some(bounds) = tab.preview_list.bounds_for_item(item_ix) {
                            tab.preview_list.scroll_to(gpui::ListOffset {
                                item_ix,
                                offset_in_item: px(f32::from(bounds.size.height) * progress),
                            });
                        } else {
                            tab.preview_list.scroll_to(gpui::ListOffset {
                                item_ix,
                                offset_in_item: px(0.),
                            });
                            tab.sync_scroll_state.pending_preview_refinement =
                                Some(PendingPreviewRefinement {
                                    version,
                                    item_ix,
                                    progress,
                                });
                        }
                    }
                }
                let actual = tab.preview_list.logical_scroll_top();
                let actual = SyncPreviewPosition {
                    item_ix: actual.item_ix,
                    offset_in_item: f32::from(actual.offset_in_item),
                };
                tab.sync_scroll_state.last_preview_position = Some(actual);
                tab.sync_scroll_state.expected_follower =
                    Some(ExpectedSyncFollower::Preview(actual));
                tab.sync_scroll_state.expected_follower_retried = false;
            }
            PaneScrollTarget::Preview => {
                let target = if preview_offset <= SYNC_SCROLL_PIXEL_EPSILON {
                    0.
                } else if preview_offset >= preview_max - SYNC_SCROLL_PIXEL_EPSILON
                    || preview_position.item_ix >= tab.preview_list_blocks.len()
                {
                    editor_max
                } else {
                    let item_ix = preview_position.item_ix;
                    let Some(bounds) = tab.preview_list.bounds_for_item(item_ix) else {
                        tab.sync_scroll_state.deferred_driver = Some(driver);
                        cx.notify();
                        return;
                    };
                    let progress = sync_interval_progress(
                        preview_position.offset_in_item,
                        0.,
                        f32::from(bounds.size.height),
                    );
                    let range = tab.preview_list_blocks[item_ix].source_range();
                    let Some(start_y) = tab.source_content_y_for_offset(range.start) else {
                        tab.sync_scroll_state.deferred_driver = Some(driver);
                        return;
                    };
                    let Some(mut end_y) = tab.source_content_y_for_offset(range.end) else {
                        tab.sync_scroll_state.deferred_driver = Some(driver);
                        return;
                    };
                    end_y = end_y.max(start_y + f32::from(tab.line_height));
                    sync_interpolate(start_y, end_y, progress).clamp(0., editor_max)
                };
                tab.editor_scroll.set_offset(point(px(0.), px(-target)));
                let actual = f32::from(-tab.editor_scroll.offset().y)
                    .max(0.)
                    .min(editor_max);
                tab.sync_scroll_state.last_editor_offset = Some(actual);
                tab.sync_scroll_state.expected_follower =
                    Some(ExpectedSyncFollower::Editor(actual));
                tab.sync_scroll_state.expected_follower_retried = false;
            }
        }
        cx.notify();
    }
}

/// Marks every virtual-list item unmeasured while keeping the current logical
/// item as the scroll anchor. Replacing the whole `0..count` range at once
/// makes GPUI move an anchor inside that range to item zero; splitting around
/// the anchor preserves the user's document position across typography reflow.
pub(super) fn invalidate_list_measurements_around_scroll_anchor(list: &ListState) {
    let count = list.item_count();
    if count == 0 {
        return;
    }
    let scroll_top = list.logical_scroll_top();
    let anchor = scroll_top.item_ix.min(count - 1);
    if anchor + 1 < count {
        list.splice(anchor + 1..count, count - anchor - 1);
    }
    if anchor > 0 {
        list.splice(0..anchor, anchor);
    }
    list.splice(anchor..anchor + 1, 1);
    list.scroll_to(gpui::ListOffset {
        item_ix: anchor,
        offset_in_item: scroll_top.offset_in_item,
    });
}
