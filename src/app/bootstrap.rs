use super::*;

pub(super) fn install_window_close_guard(
    window: &mut Window,
    app_entity: Entity<MarkionApp>,
    cx: &mut App,
) {
    window.on_window_should_close(cx, move |window, cx| {
        let (allow_close, is_dirty, confirming_close, language) = {
            let app = app_entity.read(cx);
            (
                app.allow_close,
                app.tabs.iter().any(|t| t.document.is_dirty()),
                app.confirming_close,
                app.language,
            )
        };

        if allow_close || !is_dirty {
            return true;
        }

        if confirming_close {
            return false;
        }

        let answer = window.prompt(
            PromptLevel::Warning,
            t(language, Msg::DialogExitTitle),
            Some(t(language, Msg::DialogExitDetail)),
            &[
                PromptButton::ok(t(language, Msg::DialogButtonExitWithoutSaving)),
                PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
            ],
            cx,
        );

        app_entity.update(cx, |app, cx| {
            app.confirming_close = true;
            app.active_menu = None;
            app.status = t(app.language, Msg::StatusWaitingQuitConfirm).into();
            cx.notify();
        });

        let app_entity = app_entity.clone();
        cx.spawn(async move |cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let _ = cx.update(|cx| {
                app_entity.update(cx, |app, cx| {
                    app.confirming_close = false;
                    if confirmed {
                        app.discard_all_tab_recovery_files();
                        app.allow_close = true;
                        cx.quit();
                    } else {
                        app.status = t(app.language, Msg::StatusExitCanceled).into();
                        cx.notify();
                    }
                });
            });
        })
        .detach();

        false
    });
}

pub(super) fn install_menus(language: Language, heading_menu_max_level: u8, cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: t(language, Msg::MenuFile).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemNew), NewDocument),
                MenuItem::action(t(language, Msg::ItemOpen), OpenDocument),
                MenuItem::action(t(language, Msg::ItemOpenFolder), OpenFolder),
                MenuItem::action(t(language, Msg::ItemSave), SaveDocument),
                MenuItem::action(t(language, Msg::ItemSaveAs), SaveDocumentAs),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemNewTab), NewTab),
                MenuItem::action(t(language, Msg::ItemOpenInNewTab), OpenInNewTab),
                MenuItem::action(t(language, Msg::ItemCloseTab), CloseTab),
                MenuItem::action(t(language, Msg::ItemNextTab), NextTab),
                MenuItem::action(t(language, Msg::ItemPrevTab), PrevTab),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemPreferences), ShowPreferences),
                MenuItem::action(t(language, Msg::ItemResetPreferences), ResetPreferences),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemExit), Quit),
            ],
        },
        Menu {
            name: t(language, Msg::MenuEdit).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemUndo), Undo),
                MenuItem::action(t(language, Msg::ItemRedo), Redo),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemCopy), Copy),
                MenuItem::action(t(language, Msg::ItemCut), Cut),
                MenuItem::action(t(language, Msg::ItemPaste), Paste),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemSelectAll), SelectAll),
            ],
        },
        Menu {
            name: t(language, Msg::MenuView).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemToggleView), ToggleViewMode),
                MenuItem::action(t(language, Msg::ItemEditMode), SetEditMode),
                MenuItem::action(t(language, Msg::ItemVisualEditMode), SetVisualEditMode),
                MenuItem::action(t(language, Msg::ItemSplitPreviewMode), SetSplitPreviewMode),
                MenuItem::action(t(language, Msg::ItemReadMode), SetReadMode),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemToggleSidebar), ToggleSidebar),
                MenuItem::action(t(language, Msg::ItemFiles), ToggleFileTree),
                MenuItem::action(t(language, Msg::ItemOutline), ToggleOutline),
                MenuItem::action(t(language, Msg::ItemFocusMode), ToggleFocusMode),
                MenuItem::action(t(language, Msg::ItemTypewriterMode), ToggleTypewriterMode),
                MenuItem::action(t(language, Msg::ItemCodeLineNumbers), ToggleCodeLineNumbers),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemFind), ShowFind),
                MenuItem::action(t(language, Msg::ItemReplace), ShowReplace),
                MenuItem::action(t(language, Msg::ItemFindNext), FindNext),
                MenuItem::action(t(language, Msg::ItemFindPrevious), FindPrevious),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemCycleTheme), CycleTheme),
            ],
        },
        Menu {
            name: t(language, Msg::MenuFormat).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemBold), Bold),
                MenuItem::action(t(language, Msg::ItemItalic), Italic),
                MenuItem::action(t(language, Msg::ItemInlineCode), InlineCode),
                MenuItem::action(t(language, Msg::ItemLink), InsertLink),
                MenuItem::action(t(language, Msg::ItemImage), InsertImage),
                MenuItem::separator(),
            ]
            .into_iter()
            .chain(heading_native_menu_items(language, heading_menu_max_level))
            .chain([
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemBullets), UnorderedList),
                MenuItem::action(t(language, Msg::ItemNumbers), OrderedList),
                MenuItem::action(t(language, Msg::ItemTask), TaskList),
                MenuItem::action(t(language, Msg::ItemQuote), BlockQuote),
                MenuItem::action(t(language, Msg::ItemCodeFence), CodeFence),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemFormatTable), FormatTable),
                MenuItem::action(t(language, Msg::ItemAddTableRow), TableAddRow),
                MenuItem::action(t(language, Msg::ItemDeleteTableRow), TableDeleteRow),
                MenuItem::action(t(language, Msg::ItemMoveRowUp), TableMoveRowUp),
                MenuItem::action(t(language, Msg::ItemMoveRowDown), TableMoveRowDown),
                MenuItem::action(t(language, Msg::ItemAddTableColumn), TableAddColumn),
                MenuItem::action(t(language, Msg::ItemDeleteTableColumn), TableDeleteColumn),
            ])
            .collect(),
        },
        Menu {
            name: t(language, Msg::MenuExport).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemExportHtml), ExportHtml),
                MenuItem::action(t(language, Msg::ItemExportPlainHtml), ExportPlainHtml),
                MenuItem::action(t(language, Msg::ItemExportPdf), ExportPdf),
                MenuItem::action(t(language, Msg::ItemExportLatex), ExportLatex),
                MenuItem::action(t(language, Msg::ItemExportDocx), ExportDocx),
                MenuItem::action(t(language, Msg::ItemExportPng), ExportPng),
                MenuItem::action(t(language, Msg::ItemExportJpeg), ExportJpeg),
            ],
        },
        Menu {
            name: t(language, Msg::MenuHelp).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemCheckForUpdates), CheckForUpdates),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemAboutMarkion), AboutMarkion),
            ],
        },
    ]);
}

/// Shortcut overrides loaded from `config.toml` before the window exists, so
/// the very first keymap already honours customized bindings. Preferences are
/// loaded again by `MarkionApp::new`; this early read only feeds keybinding.
fn startup_shortcut_overrides() -> BTreeMap<String, String> {
    let preferences = load_app_preferences(default_preferences_path()).unwrap_or_default();
    sanitized_shortcut_overrides(&preferences.shortcut_overrides)
}

/// Bind the complete application keymap: fixed core-editing keys, fixed
/// file-tree keys, and every registry action at its effective binding
/// (override when present and valid, else default). Callers that rebind at
/// runtime must `clear_key_bindings()` first; this function is the single
/// binding code path so a rebind restores the full set.
pub(super) fn bind_app_keys(cx: &mut App, overrides: &BTreeMap<String, String>) {
    let eff = |shortcut: &MenuShortcut| shortcut.effective_binding(overrides);
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, None),
        KeyBinding::new("delete", Delete, None),
        KeyBinding::new("left", Left, None),
        KeyBinding::new("right", Right, None),
        KeyBinding::new("up", Up, None),
        KeyBinding::new("down", Down, None),
        KeyBinding::new("shift-left", SelectLeft, None),
        KeyBinding::new("shift-right", SelectRight, None),
        KeyBinding::new("shift-up", SelectUp, None),
        KeyBinding::new("shift-down", SelectDown, None),
        // `secondary-` maps to `cmd` on macOS and `ctrl` on Windows/Linux,
        // so shortcuts match each platform's convention.
        KeyBinding::new(eff(&menu_shortcuts::SELECT_ALL), SelectAll, None),
        KeyBinding::new(eff(&menu_shortcuts::PASTE), Paste, None),
        KeyBinding::new(eff(&menu_shortcuts::COPY), Copy, None),
        KeyBinding::new(eff(&menu_shortcuts::CUT), Cut, None),
        KeyBinding::new(eff(&menu_shortcuts::UNDO), Undo, None),
        KeyBinding::new(eff(&menu_shortcuts::REDO), Redo, None),
        KeyBinding::new(eff(&menu_shortcuts::BOLD), Bold, None),
        KeyBinding::new(eff(&menu_shortcuts::ITALIC), Italic, None),
        KeyBinding::new(eff(&menu_shortcuts::INLINE_CODE), InlineCode, None),
        KeyBinding::new(eff(&menu_shortcuts::INSERT_LINK), InsertLink, None),
        KeyBinding::new(eff(&menu_shortcuts::INSERT_IMAGE), InsertImage, None),
        KeyBinding::new(eff(&menu_shortcuts::HEADING_1), Heading1, None),
        KeyBinding::new(eff(&menu_shortcuts::HEADING_2), Heading2, None),
        KeyBinding::new(eff(&menu_shortcuts::HEADING_3), Heading3, None),
        KeyBinding::new(eff(&menu_shortcuts::HEADING_4), Heading4, None),
        KeyBinding::new(eff(&menu_shortcuts::HEADING_5), Heading5, None),
        KeyBinding::new(eff(&menu_shortcuts::HEADING_6), Heading6, None),
        KeyBinding::new("home", Home, None),
        KeyBinding::new("end", End, None),
        KeyBinding::new("enter", InsertNewline, None),
        KeyBinding::new("shift-f10", ShowVisualBlockContextMenu, None),
        KeyBinding::new("tab", Indent, None),
        KeyBinding::new("shift-tab", Outdent, None),
        KeyBinding::new(eff(&menu_shortcuts::NEW_DOCUMENT), NewDocument, None),
        KeyBinding::new(eff(&menu_shortcuts::OPEN_DOCUMENT), OpenDocument, None),
        KeyBinding::new(eff(&menu_shortcuts::SAVE_DOCUMENT), SaveDocument, None),
        KeyBinding::new(eff(&menu_shortcuts::SAVE_DOCUMENT_AS), SaveDocumentAs, None),
        KeyBinding::new(eff(&menu_shortcuts::EXPORT_HTML), ExportHtml, None),
        KeyBinding::new(
            eff(&menu_shortcuts::EXPORT_PLAIN_HTML),
            ExportPlainHtml,
            None,
        ),
        KeyBinding::new(eff(&menu_shortcuts::EXPORT_PDF), ExportPdf, None),
        KeyBinding::new(eff(&menu_shortcuts::EXPORT_LATEX), ExportLatex, None),
        KeyBinding::new(eff(&menu_shortcuts::EXPORT_DOCX), ExportDocx, None),
        KeyBinding::new(eff(&menu_shortcuts::EXPORT_PNG), ExportPng, None),
        KeyBinding::new(eff(&menu_shortcuts::EXPORT_JPEG), ExportJpeg, None),
        KeyBinding::new(eff(&menu_shortcuts::TOGGLE_VIEW_MODE), ToggleViewMode, None),
        KeyBinding::new(eff(&menu_shortcuts::SET_EDIT_MODE), SetEditMode, None),
        KeyBinding::new(
            eff(&menu_shortcuts::SET_VISUAL_EDIT_MODE),
            SetVisualEditMode,
            None,
        ),
        KeyBinding::new(
            eff(&menu_shortcuts::SET_SPLIT_PREVIEW_MODE),
            SetSplitPreviewMode,
            None,
        ),
        KeyBinding::new(eff(&menu_shortcuts::SET_READ_MODE), SetReadMode, None),
        // NB: no `secondary-b` for the sidebar — that collides with Bold.
        // Use Ctrl/Cmd+Shift+B instead.
        KeyBinding::new(eff(&menu_shortcuts::TOGGLE_SIDEBAR), ToggleSidebar, None),
        KeyBinding::new(eff(&menu_shortcuts::TOGGLE_FILE_TREE), ToggleFileTree, None),
        KeyBinding::new(
            eff(&menu_shortcuts::FOCUS_FILE_TREE_SEARCH),
            FocusFileTreeSearch,
            None,
        ),
        KeyBinding::new("escape", ClearFileTreeSearch, None),
        KeyBinding::new("f5", RefreshFileTree, None),
        KeyBinding::new("secondary-alt-n", CreateTreeFile, None),
        KeyBinding::new("secondary-alt-shift-n", CreateTreeFolder, None),
        KeyBinding::new("f2", RenameTreeEntry, None),
        KeyBinding::new("secondary-delete", DeleteTreeEntry, None),
        KeyBinding::new(eff(&menu_shortcuts::TOGGLE_OUTLINE), ToggleOutline, None),
        KeyBinding::new(eff(&menu_shortcuts::CYCLE_THEME), CycleTheme, None),
        KeyBinding::new(
            eff(&menu_shortcuts::TOGGLE_FOCUS_MODE),
            ToggleFocusMode,
            None,
        ),
        KeyBinding::new(
            eff(&menu_shortcuts::TOGGLE_TYPEWRITER_MODE),
            ToggleTypewriterMode,
            None,
        ),
        KeyBinding::new(
            eff(&menu_shortcuts::TOGGLE_CODE_LINE_NUMBERS),
            ToggleCodeLineNumbers,
            None,
        ),
        KeyBinding::new(eff(&menu_shortcuts::FORMAT_TABLE), FormatTable, None),
        KeyBinding::new(eff(&menu_shortcuts::TABLE_ADD_ROW), TableAddRow, None),
        KeyBinding::new(eff(&menu_shortcuts::TABLE_DELETE_ROW), TableDeleteRow, None),
        KeyBinding::new(
            eff(&menu_shortcuts::TABLE_MOVE_ROW_UP),
            TableMoveRowUp,
            None,
        ),
        KeyBinding::new(
            eff(&menu_shortcuts::TABLE_MOVE_ROW_DOWN),
            TableMoveRowDown,
            None,
        ),
        KeyBinding::new(eff(&menu_shortcuts::TABLE_ADD_COLUMN), TableAddColumn, None),
        KeyBinding::new(
            eff(&menu_shortcuts::TABLE_DELETE_COLUMN),
            TableDeleteColumn,
            None,
        ),
        KeyBinding::new(eff(&menu_shortcuts::SHOW_FIND), ShowFind, None),
        KeyBinding::new(eff(&menu_shortcuts::SHOW_REPLACE), ShowReplace, None),
        KeyBinding::new(eff(&menu_shortcuts::FIND_NEXT), FindNext, None),
        KeyBinding::new(eff(&menu_shortcuts::FIND_PREVIOUS), FindPrevious, None),
        KeyBinding::new(
            eff(&menu_shortcuts::SHOW_PREFERENCES),
            ShowPreferences,
            None,
        ),
        KeyBinding::new(eff(&menu_shortcuts::SHOW_SHORTCUTS), ShowShortcuts, None),
        KeyBinding::new(eff(&menu_shortcuts::QUIT), Quit, None),
        KeyBinding::new(eff(&menu_shortcuts::NEXT_TAB), NextTab, None),
        KeyBinding::new(eff(&menu_shortcuts::PREV_TAB), PrevTab, None),
        KeyBinding::new(eff(&menu_shortcuts::OPEN_IN_NEW_TAB), OpenInNewTab, None),
        KeyBinding::new(eff(&menu_shortcuts::CLOSE_TAB), CloseTab, None),
        // Developer diagnostic — not listed in menus or the shortcut reference.
        KeyBinding::new("ctrl-shift-alt-m", ReportMemory, None),
    ]);
}

pub(super) fn run() {
    run_with_startup_intent(StartupOpenIntent::from_env_args());
}

pub(super) fn run_with_startup_intent(startup_intent: StartupOpenIntent) {
    // Diagnostic file logging (daily rotation in the Markion log dir). Failures
    // are non-fatal: the editor starts without file logging.
    let log_dir = markion::init_logging();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_dir = ?log_dir,
        "Markion starting"
    );

    // Load the syntect grammar registry off the main thread so the first
    // highlighted code block never blocks the typing path (~100ms of grammar
    // parsing happens here instead of on first use).
    std::thread::spawn(markion::warm_highlighter);

    Application::new()
        .with_assets(crate::ui::icon::IconAssets)
        .run(move |cx: &mut App| {
        if let Err(error) = network::install_http_client(cx) {
            tracing::error!(%error, "failed to initialize HTTP client; remote images are disabled");
        }

        // Bind the full keymap with any customized shortcuts from config.toml.
        bind_app_keys(cx, &startup_shortcut_overrides());
        // Install the native menu once with the default language; the window
        // hook below re-installs it after the saved language preference has
        // been loaded, so the OS menu bar honours the user's choice on launch.
        install_menus(Language::default(), DEFAULT_HEADING_MENU_MAX_LEVEL, cx);

        let bounds = Bounds::centered(None, size(px(1180.), px(760.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some(SharedString::from(MARKION_WINDOW_TITLE)),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    app_id: Some(MARKION_APP_ID.to_string()),
                    ..Default::default()
                },
                |_, cx| cx.new(MarkionApp::new),
            )
            .unwrap();

        let startup_intent = startup_intent.clone();
        window
            .update(cx, |app, window, cx| {
                install_window_close_guard(window, cx.entity(), cx);
                window.focus(&app.focus_handle(cx));
                // Re-translate the native menu now that the saved language
                // preference has been loaded by `MarkionApp::new`.
                install_menus(app.language, app.heading_menu_max_level, cx);
                app.apply_startup_open_intent(startup_intent.clone(), cx);
                app.restore_session_on_startup(&startup_intent, cx);
                app.check_recovery_on_startup(window, cx);
                app.arm_external_file_poll(cx);
                // File-tree scanning is driven by session restore, CLI folder
                // open, or opening a document — not by the process CWD.
                cx.activate(true);
            })
            .unwrap();
        cx.activate(true);
    });
}
