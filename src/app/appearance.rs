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
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The Preferences panel is rendered in-app (see `preferences_panel_view`),
        // so opening it is just a flag flip. Refresh the custom-theme list so a
        // theme file dropped into the themes dir since launch shows up.
        self.ensure_sample_custom_theme();
        self.custom_themes = list_theme_definitions(&self.themes_dir).unwrap_or_default();
        self.preferences_panel_open = true;
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn close_preferences(&mut self, cx: &mut Context<Self>) {
        self.preferences_panel_open = false;
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
                    app.heading_menu_max_level = preferences.heading_menu_max_level;
                    app.sync_scroll = preferences.sync_scroll;
                    app.sidebar_visible = preferences.sidebar_visible;
                    app.sidebar_tab = preferences.sidebar_tab;
                    // Reset also restores the default interface language.
                    app.language = Language::from_code(&preferences.language);
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

    pub(super) fn toggle_sync_scroll(&mut self, cx: &mut Context<Self>) {
        self.sync_scroll = !self.sync_scroll;
        // Drop the cached fractions so the next Split frame reconciles from
        // whatever the current scroll positions imply, instead of treating a
        // pane as "unchanged" and skipping the first coupling.
        for tab in &mut self.tabs {
            tab.sync_scroll_editor_fraction = None;
            tab.sync_scroll_preview_fraction = None;
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

    /// Proportional scroll coupling for Split Preview + Sync scroll. See
    /// [`sync_scroll_is_active`] / [`sync_fraction`]. Runs once per render.
    ///
    /// Reads each pane's current scroll offset and scrollable range, computes
    /// fractions, and — when Sync scroll is active — writes the driving pane's
    /// fraction to the other pane. The driving pane is whichever pane's
    /// *cached* fraction no longer matches its freshly-read fraction (i.e. the
    /// one the user/system just moved). After writing, both cached fractions
    /// are set to the driver's fraction, so the next frame sees no change and
    /// the write does not recur (one-frame convergence, no feedback loop).
    ///
    /// `syncing_scroll` guards against re-entrancy within the same frame. A
    /// small epsilon stops sub-pixel drift from re-triggering writes.
    pub(super) fn reconcile_sync_scroll(&mut self) {
        if self.syncing_scroll || !sync_scroll_is_active(self.view_mode, self.sync_scroll) {
            return;
        }
        // Borrow the active tab mutably once; we read offsets from the
        // scroll handle / list state (which are fields on the tab) and write
        // back to them, plus the cached fractions. `Pixels(pub(crate) f32)` is
        // private, so the raw `f32` values come via `f32::from` (the public
        // `impl From<Pixels> for f32`), keeping `sync_fraction` a pure f32 helper.
        let tab = &mut self.tabs[self.active_tab];
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

        let editor_frac = sync_fraction(editor_offset, editor_max);
        let preview_frac = sync_fraction(preview_offset, preview_max);

        // If neither pane has scrollable range, there is nothing to couple.
        if editor_max <= 1. && preview_max <= 1. {
            tab.sync_scroll_editor_fraction = Some(editor_frac);
            tab.sync_scroll_preview_fraction = Some(preview_frac);
            return;
        }

        // Determine the driver: the pane whose stored fraction drifted from its
        // current fraction. First-frame (None) seeds the cache without writing,
        // so we don't yank a pane on the very first Split render.
        let editor_changed = tab.sync_scroll_editor_fraction.map_or(false, |stored| {
            (stored - editor_frac).abs() > SYNC_SCROLL_EPSILON
        });
        let preview_changed = tab.sync_scroll_preview_fraction.map_or(false, |stored| {
            (stored - preview_frac).abs() > SYNC_SCROLL_EPSILON
        });

        // Seed caches on the first observed frame (or after a reset) without
        // driving, so the next real change is the first to couple.
        let needs_seed = tab
            .sync_scroll_editor_fraction
            .zip(tab.sync_scroll_preview_fraction)
            .is_none();

        self.syncing_scroll = true;
        if !needs_seed && editor_changed && !preview_changed && preview_max > 1. {
            // Editor drove: pull the preview to the editor's fraction.
            let target = (editor_frac * preview_max).clamp(0., preview_max);
            tab.preview_list
                .set_offset_from_scrollbar(point(px(0.), px(-target)));
            tab.sync_scroll_preview_fraction = Some(editor_frac);
            tab.sync_scroll_editor_fraction = Some(editor_frac);
        } else if !needs_seed && preview_changed && !editor_changed && editor_max > 1. {
            // Preview drove: pull the editor to the preview's fraction.
            let target = (preview_frac * editor_max).clamp(0., editor_max);
            tab.editor_scroll.set_offset(point(px(0.), px(-target)));
            tab.sync_scroll_editor_fraction = Some(preview_frac);
            tab.sync_scroll_preview_fraction = Some(preview_frac);
        } else {
            // No single clear driver (both moved, or neither moved): just record
            // the current state so a future single-pane change can be detected.
            tab.sync_scroll_editor_fraction = Some(editor_frac);
            tab.sync_scroll_preview_fraction = Some(preview_frac);
        }
        self.syncing_scroll = false;
    }
}
