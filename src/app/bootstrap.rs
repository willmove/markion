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
                        app.discard_current_recovery_file();
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
                MenuItem::action(t(language, Msg::ItemKeyboardShortcuts), ShowShortcuts),
                MenuItem::action(t(language, Msg::ItemAboutMarkion), AboutMarkion),
            ],
        },
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

    Application::new().run(move |cx: &mut App| {
        if let Err(error) = network::install_http_client(cx) {
            tracing::error!(%error, "failed to initialize HTTP client; remote images are disabled");
        }

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
            KeyBinding::new(menu_shortcuts::SELECT_ALL.binding, SelectAll, None),
            KeyBinding::new(menu_shortcuts::PASTE.binding, Paste, None),
            KeyBinding::new(menu_shortcuts::COPY.binding, Copy, None),
            KeyBinding::new(menu_shortcuts::CUT.binding, Cut, None),
            KeyBinding::new(menu_shortcuts::UNDO.binding, Undo, None),
            KeyBinding::new(menu_shortcuts::REDO.binding, Redo, None),
            KeyBinding::new(menu_shortcuts::BOLD.binding, Bold, None),
            KeyBinding::new(menu_shortcuts::ITALIC.binding, Italic, None),
            KeyBinding::new(menu_shortcuts::INLINE_CODE.binding, InlineCode, None),
            KeyBinding::new(menu_shortcuts::INSERT_LINK.binding, InsertLink, None),
            KeyBinding::new(menu_shortcuts::INSERT_IMAGE.binding, InsertImage, None),
            KeyBinding::new(menu_shortcuts::HEADING_1.binding, Heading1, None),
            KeyBinding::new(menu_shortcuts::HEADING_2.binding, Heading2, None),
            KeyBinding::new(menu_shortcuts::HEADING_3.binding, Heading3, None),
            KeyBinding::new(menu_shortcuts::HEADING_4.binding, Heading4, None),
            KeyBinding::new(menu_shortcuts::HEADING_5.binding, Heading5, None),
            KeyBinding::new(menu_shortcuts::HEADING_6.binding, Heading6, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("enter", InsertNewline, None),
            KeyBinding::new("tab", Indent, None),
            KeyBinding::new("shift-tab", Outdent, None),
            KeyBinding::new(menu_shortcuts::NEW_DOCUMENT.binding, NewDocument, None),
            KeyBinding::new(menu_shortcuts::OPEN_DOCUMENT.binding, OpenDocument, None),
            KeyBinding::new(menu_shortcuts::SAVE_DOCUMENT.binding, SaveDocument, None),
            KeyBinding::new(
                menu_shortcuts::SAVE_DOCUMENT_AS.binding,
                SaveDocumentAs,
                None,
            ),
            KeyBinding::new(menu_shortcuts::EXPORT_HTML.binding, ExportHtml, None),
            KeyBinding::new(
                menu_shortcuts::EXPORT_PLAIN_HTML.binding,
                ExportPlainHtml,
                None,
            ),
            KeyBinding::new(menu_shortcuts::EXPORT_PDF.binding, ExportPdf, None),
            KeyBinding::new(menu_shortcuts::EXPORT_LATEX.binding, ExportLatex, None),
            KeyBinding::new(menu_shortcuts::EXPORT_DOCX.binding, ExportDocx, None),
            KeyBinding::new(menu_shortcuts::EXPORT_PNG.binding, ExportPng, None),
            KeyBinding::new(menu_shortcuts::EXPORT_JPEG.binding, ExportJpeg, None),
            KeyBinding::new(
                menu_shortcuts::TOGGLE_VIEW_MODE.binding,
                ToggleViewMode,
                None,
            ),
            KeyBinding::new(menu_shortcuts::SET_EDIT_MODE.binding, SetEditMode, None),
            KeyBinding::new(
                menu_shortcuts::SET_VISUAL_EDIT_MODE.binding,
                SetVisualEditMode,
                None,
            ),
            KeyBinding::new(
                menu_shortcuts::SET_SPLIT_PREVIEW_MODE.binding,
                SetSplitPreviewMode,
                None,
            ),
            KeyBinding::new(menu_shortcuts::SET_READ_MODE.binding, SetReadMode, None),
            // NB: no `secondary-b` for the sidebar — that collides with Bold.
            // Use Ctrl/Cmd+Shift+B instead.
            KeyBinding::new(menu_shortcuts::TOGGLE_SIDEBAR.binding, ToggleSidebar, None),
            KeyBinding::new(
                menu_shortcuts::TOGGLE_FILE_TREE.binding,
                ToggleFileTree,
                None,
            ),
            KeyBinding::new("secondary-alt-f", FocusFileTreeSearch, None),
            KeyBinding::new("escape", ClearFileTreeSearch, None),
            KeyBinding::new("f5", RefreshFileTree, None),
            KeyBinding::new("secondary-alt-n", CreateTreeFile, None),
            KeyBinding::new("secondary-alt-shift-n", CreateTreeFolder, None),
            KeyBinding::new("f2", RenameTreeEntry, None),
            KeyBinding::new("secondary-delete", DeleteTreeEntry, None),
            KeyBinding::new(menu_shortcuts::TOGGLE_OUTLINE.binding, ToggleOutline, None),
            KeyBinding::new(menu_shortcuts::CYCLE_THEME.binding, CycleTheme, None),
            KeyBinding::new(
                menu_shortcuts::TOGGLE_FOCUS_MODE.binding,
                ToggleFocusMode,
                None,
            ),
            KeyBinding::new(
                menu_shortcuts::TOGGLE_TYPEWRITER_MODE.binding,
                ToggleTypewriterMode,
                None,
            ),
            KeyBinding::new(
                menu_shortcuts::TOGGLE_CODE_LINE_NUMBERS.binding,
                ToggleCodeLineNumbers,
                None,
            ),
            KeyBinding::new(menu_shortcuts::FORMAT_TABLE.binding, FormatTable, None),
            KeyBinding::new(menu_shortcuts::TABLE_ADD_ROW.binding, TableAddRow, None),
            KeyBinding::new(
                menu_shortcuts::TABLE_DELETE_ROW.binding,
                TableDeleteRow,
                None,
            ),
            KeyBinding::new(
                menu_shortcuts::TABLE_MOVE_ROW_UP.binding,
                TableMoveRowUp,
                None,
            ),
            KeyBinding::new(
                menu_shortcuts::TABLE_MOVE_ROW_DOWN.binding,
                TableMoveRowDown,
                None,
            ),
            KeyBinding::new(
                menu_shortcuts::TABLE_ADD_COLUMN.binding,
                TableAddColumn,
                None,
            ),
            KeyBinding::new(
                menu_shortcuts::TABLE_DELETE_COLUMN.binding,
                TableDeleteColumn,
                None,
            ),
            KeyBinding::new(menu_shortcuts::SHOW_FIND.binding, ShowFind, None),
            KeyBinding::new(menu_shortcuts::SHOW_REPLACE.binding, ShowReplace, None),
            KeyBinding::new(menu_shortcuts::FIND_NEXT.binding, FindNext, None),
            KeyBinding::new(menu_shortcuts::FIND_PREVIOUS.binding, FindPrevious, None),
            KeyBinding::new(
                menu_shortcuts::SHOW_PREFERENCES.binding,
                ShowPreferences,
                None,
            ),
            KeyBinding::new(menu_shortcuts::SHOW_SHORTCUTS.binding, ShowShortcuts, None),
            KeyBinding::new(menu_shortcuts::QUIT.binding, Quit, None),
            KeyBinding::new(menu_shortcuts::NEXT_TAB.binding, NextTab, None),
            KeyBinding::new(menu_shortcuts::PREV_TAB.binding, PrevTab, None),
            KeyBinding::new(menu_shortcuts::OPEN_IN_NEW_TAB.binding, OpenInNewTab, None),
            KeyBinding::new(menu_shortcuts::CLOSE_TAB.binding, CloseTab, None),
        ]);
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
                app.apply_startup_open_intent(startup_intent, cx);
                app.check_recovery_on_startup(window, cx);
                // The file tree is intentionally NOT scanned on startup. With
                // only the in-memory welcome document open there is no chosen
                // workspace directory, and showing the program's own working
                // directory is not useful for a Markdown editor. The tree stays
                // `None` (empty-state placeholder) until a real file is opened,
                // at which point `update_workspace_root_from_document` scans
                // that file's parent directory.
                cx.activate(true);
            })
            .unwrap();
        cx.activate(true);
    });
}
