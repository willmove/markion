//! Preferences panel Editor tab: Code Block formatting settings, external
//! formatter path pickers, and asynchronous tool availability probes.

use super::root_view::{
    pane_scrollbar_view, preference_boolean_row, preference_font_action, preference_option_button,
    preference_path_row, preference_section_header,
};
use super::*;
use markion::code_format::{CodeLanguage, FormatterProbeStatus};
use markion::{CodeFormatMsg, JsTsFormatterBackend, code_format_t, code_format_tf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CodeFormatterPathKind {
    Biome,
    Prettier,
    GoogleJavaFormat,
    JavaRuntime,
    Rustfmt,
    Ruff,
}

impl CodeFormatterPathKind {
    fn tool_name(self, language: Language) -> &'static str {
        match self {
            Self::Biome => "Biome",
            Self::Prettier => "Prettier",
            Self::GoogleJavaFormat => "google-java-format",
            Self::JavaRuntime => code_format_t(language, CodeFormatMsg::PathJavaRuntime),
            Self::Rustfmt => "rustfmt",
            Self::Ruff => "ruff",
        }
    }

    fn picker_title(self, language: Language) -> String {
        let msg = if self == Self::GoogleJavaFormat {
            CodeFormatMsg::PrefPickJavaTitle
        } else {
            CodeFormatMsg::PrefPickTitle
        };
        code_format_tf(language, msg, &[self.tool_name(language)])
    }

    fn current_path(self, prefs: &markion::CodeBlockFormatPreferences) -> Option<&str> {
        match self {
            Self::Biome => prefs.biome_path.as_deref(),
            Self::Prettier => prefs.prettier_path.as_deref(),
            Self::GoogleJavaFormat => prefs.google_java_format_path.as_deref(),
            Self::JavaRuntime => prefs.java_path.as_deref(),
            Self::Rustfmt => prefs.rustfmt_path.as_deref(),
            Self::Ruff => prefs.ruff_path.as_deref(),
        }
    }

    fn set_path(self, prefs: &mut markion::CodeBlockFormatPreferences, value: Option<String>) {
        let slot = match self {
            Self::Biome => &mut prefs.biome_path,
            Self::Prettier => &mut prefs.prettier_path,
            Self::GoogleJavaFormat => &mut prefs.google_java_format_path,
            Self::JavaRuntime => &mut prefs.java_path,
            Self::Rustfmt => &mut prefs.rustfmt_path,
            Self::Ruff => &mut prefs.ruff_path,
        };
        *slot = value;
    }
}

impl MarkionApp {
    pub(super) fn toggle_code_format_on_paste(&mut self, cx: &mut Context<Self>) {
        self.code_block_preferences.format_on_paste = !self.code_block_preferences.format_on_paste;
        self.status = code_format_t(
            self.language,
            if self.code_block_preferences.format_on_paste {
                CodeFormatMsg::StatusFormatOnPasteOn
            } else {
                CodeFormatMsg::StatusFormatOnPasteOff
            },
        )
        .into();
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_js_ts_formatter_backend(
        &mut self,
        backend: JsTsFormatterBackend,
        cx: &mut Context<Self>,
    ) {
        if self.code_block_preferences.js_ts_backend == backend {
            return;
        }
        self.code_block_preferences.js_ts_backend = backend;
        self.persist_preferences();
        self.refresh_code_formatter_availability(cx);
    }

    pub(super) fn choose_code_formatter_path(
        &mut self,
        kind: CodeFormatterPathKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title = kind.picker_title(self.language);
        let picked = prompt_for_open_file_titled(window, &title, None);
        cx.spawn(async move |this, cx| {
            if let Some(path) = picked.await {
                let display_path = path.display().to_string();
                let _ = this.update(cx, |app, cx| {
                    kind.set_path(&mut app.code_block_preferences, Some(display_path.clone()));
                    let tool = kind.tool_name(app.language);
                    app.status = code_format_tf(
                        app.language,
                        CodeFormatMsg::StatusFormatterPathSet,
                        &[tool, &display_path],
                    )
                    .into();
                    app.persist_preferences();
                    app.refresh_code_formatter_availability(cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn reset_code_formatter_path(
        &mut self,
        kind: CodeFormatterPathKind,
        cx: &mut Context<Self>,
    ) {
        kind.set_path(&mut self.code_block_preferences, None);
        let tool = kind.tool_name(self.language);
        self.status = code_format_tf(
            self.language,
            CodeFormatMsg::StatusFormatterPathReset,
            &[tool],
        )
        .into();
        self.persist_preferences();
        self.refresh_code_formatter_availability(cx);
    }

    pub(super) fn refresh_code_formatter_availability(&mut self, cx: &mut Context<Self>) {
        let registry = self.code_formatter_registry.clone();
        let prefs = self.code_block_preferences.clone();
        self.code_formatter_availability_cached = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let availability = cx
                .background_spawn(async move { registry.probe_all(&prefs) })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.code_formatter_availability_cached = Some(availability);
                cx.notify();
            });
        })
        .detach();
    }
}

fn code_language_name(language: Language, code_lang: CodeLanguage) -> &'static str {
    let msg = match code_lang {
        CodeLanguage::Json => CodeFormatMsg::LangJson,
        CodeLanguage::JavaScript => CodeFormatMsg::LangJavaScript,
        CodeLanguage::TypeScript => CodeFormatMsg::LangTypeScript,
        CodeLanguage::Java => CodeFormatMsg::LangJava,
        CodeLanguage::Rust => CodeFormatMsg::LangRust,
        CodeLanguage::Python => CodeFormatMsg::LangPython,
    };
    code_format_t(language, msg)
}

pub(super) fn preferences_editor_body(
    app: &MarkionApp,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let language = app.language;
    let prefs = &app.code_block_preferences;
    let default_path_label = code_format_t(language, CodeFormatMsg::PrefDefaultPath);

    let path_row = |kind: CodeFormatterPathKind, label: &'static str| {
        let value = kind
            .current_path(prefs)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| default_path_label.to_string());
        let customized = kind
            .current_path(prefs)
            .is_some_and(|v| !v.trim().is_empty());
        preference_path_row(
            label,
            value,
            customized,
            palette,
            language,
            cx.listener(move |app, _: &MouseUpEvent, window, cx| {
                app.choose_code_formatter_path(kind, window, cx);
            }),
            cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                app.reset_code_formatter_path(kind, cx);
            }),
        )
    };

    let availability_rows: Vec<Div> = CodeLanguage::ALL
        .into_iter()
        .map(|code_lang| {
            let lang_label = code_language_name(language, code_lang);
            let status: Option<&FormatterProbeStatus> = app
                .code_formatter_availability_cached
                .as_ref()
                .and_then(|cached| cached.status_for(code_lang));
            let (badge, detail, color) = match status {
                None => (
                    code_format_t(language, CodeFormatMsg::PrefChecking).to_string(),
                    String::new(),
                    palette.muted,
                ),
                Some(status) if status.available => {
                    let detail = match &status.resolved_path {
                        Some(path) => format!("{} · {}", status.tool_name, path.display()),
                        None => status.tool_name.to_string(),
                    };
                    (
                        code_format_t(language, CodeFormatMsg::PrefAvailable).to_string(),
                        detail,
                        rgb(0x16a34a),
                    )
                }
                Some(status) => (
                    code_format_t(language, CodeFormatMsg::PrefUnavailable).to_string(),
                    status
                        .detail
                        .clone()
                        .unwrap_or_else(|| status.tool_name.to_string()),
                    rgb(0xd97706),
                ),
            };
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .px_1()
                .py(px(2.))
                .text_size(px(11.5))
                .child(div().text_color(palette.text).child(lang_label))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .min_w_0()
                        .child(div().flex_none().text_color(color).child(badge))
                        .when(!detail.is_empty(), |row| {
                            row.child(
                                div()
                                    .max_w(px(280.))
                                    .truncate()
                                    .text_color(palette.muted)
                                    .child(detail),
                            )
                        }),
                )
        })
        .collect();

    div()
        .relative()
        .flex_1()
        .min_h_0()
        .child(
            div()
                .id("preferences-editor-body")
                .size_full()
                .px_4()
                .py_3()
                .overflow_y_scroll()
                .scrollbar_width(px(PANE_SCROLLBAR_RESERVED_WIDTH))
                .track_scroll(&app.preferences_editor_scroll)
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(preference_section_header(code_format_t(
                            language,
                            CodeFormatMsg::PrefCodeBlocksSection,
                        )))
                        .child(
                            div()
                                .id("preferences-code-format-on-paste")
                                .debug_selector(|| "preferences-code-format-on-paste".to_string())
                                .child(preference_boolean_row(
                                    code_format_t(language, CodeFormatMsg::PrefFormatOnPaste),
                                    prefs.format_on_paste,
                                    language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        app.toggle_code_format_on_paste(cx);
                                    }),
                                )),
                        )
                        .child(
                            div()
                                .px_1()
                                .text_size(px(10.5))
                                .text_color(palette.muted)
                                .child(code_format_t(
                                    language,
                                    CodeFormatMsg::PrefFormatOnPasteHint,
                                )),
                        )
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_1()
                                .py_1()
                                .gap_3()
                                .text_size(px(12.))
                                .child(
                                    div().text_color(palette.muted).child(code_format_t(
                                        language,
                                        CodeFormatMsg::PrefJsTsBackend,
                                    )),
                                )
                                .child(
                                    div().flex().flex_wrap().gap_2().children([
                                        preference_option_button(
                                            code_format_t(language, CodeFormatMsg::PrefBackendAuto)
                                                .to_string(),
                                            prefs.js_ts_backend == JsTsFormatterBackend::Auto,
                                            palette,
                                            cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                                app.set_js_ts_formatter_backend(
                                                    JsTsFormatterBackend::Auto,
                                                    cx,
                                                );
                                            }),
                                        ),
                                        preference_option_button(
                                            code_format_t(
                                                language,
                                                CodeFormatMsg::PrefBackendBiome,
                                            )
                                            .to_string(),
                                            prefs.js_ts_backend == JsTsFormatterBackend::Biome,
                                            palette,
                                            cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                                app.set_js_ts_formatter_backend(
                                                    JsTsFormatterBackend::Biome,
                                                    cx,
                                                );
                                            }),
                                        ),
                                        preference_option_button(
                                            code_format_t(
                                                language,
                                                CodeFormatMsg::PrefBackendPrettier,
                                            )
                                            .to_string(),
                                            prefs.js_ts_backend == JsTsFormatterBackend::Prettier,
                                            palette,
                                            cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                                app.set_js_ts_formatter_backend(
                                                    JsTsFormatterBackend::Prettier,
                                                    cx,
                                                );
                                            }),
                                        ),
                                    ]),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(preference_section_header(code_format_t(
                            language,
                            CodeFormatMsg::PrefFormatterConfiguration,
                        )))
                        .child(
                            div()
                                .px_1()
                                .text_size(px(11.))
                                .text_color(palette.muted)
                                .child(code_format_t(language, CodeFormatMsg::PrefBuiltinJson)),
                        )
                        .child(path_row(CodeFormatterPathKind::Biome, "Biome"))
                        .child(path_row(CodeFormatterPathKind::Prettier, "Prettier"))
                        .child(path_row(
                            CodeFormatterPathKind::GoogleJavaFormat,
                            "google-java-format",
                        ))
                        .child(path_row(
                            CodeFormatterPathKind::JavaRuntime,
                            code_format_t(language, CodeFormatMsg::PathJavaRuntime),
                        ))
                        .child(path_row(CodeFormatterPathKind::Rustfmt, "rustfmt"))
                        .child(path_row(CodeFormatterPathKind::Ruff, "ruff")),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(preference_section_header(code_format_t(
                                    language,
                                    CodeFormatMsg::PrefFormatterAvailability,
                                )))
                                .child(preference_font_action(
                                    code_format_t(language, CodeFormatMsg::PrefRefreshAvailability)
                                        .into(),
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        app.refresh_code_formatter_availability(cx);
                                    }),
                                )),
                        )
                        .child(
                            div()
                                .p_2()
                                .rounded_md()
                                .border_1()
                                .border_color(palette.border)
                                .bg(palette.surface_bg)
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(availability_rows),
                        )
                        .child(
                            div()
                                .px_1()
                                .text_size(px(10.5))
                                .text_color(palette.muted)
                                .child(code_format_t(language, CodeFormatMsg::PrefNoDownloadHint)),
                        ),
                ),
        )
        .child(pane_scrollbar_view(
            PaneScrollTarget::PreferencesEditor,
            &app.preferences_editor_scroll,
            palette,
            cx,
        ))
}
