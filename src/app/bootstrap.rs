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
            KeyBinding::new("secondary-a", SelectAll, None),
            KeyBinding::new("secondary-v", Paste, None),
            KeyBinding::new("secondary-c", Copy, None),
            KeyBinding::new("secondary-x", Cut, None),
            KeyBinding::new("secondary-z", Undo, None),
            KeyBinding::new("secondary-shift-z", Redo, None),
            KeyBinding::new("secondary-y", Redo, None),
            KeyBinding::new("secondary-b", Bold, None),
            KeyBinding::new("secondary-i", Italic, None),
            KeyBinding::new("secondary-e", InlineCode, None),
            KeyBinding::new("secondary-k", InsertLink, None),
            KeyBinding::new("secondary-shift-i", InsertImage, None),
            KeyBinding::new("secondary-1", Heading1, None),
            KeyBinding::new("secondary-2", Heading2, None),
            KeyBinding::new("secondary-3", Heading3, None),
            KeyBinding::new("secondary-4", Heading4, None),
            KeyBinding::new("secondary-5", Heading5, None),
            KeyBinding::new("secondary-6", Heading6, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("enter", InsertNewline, None),
            KeyBinding::new("tab", Indent, None),
            KeyBinding::new("shift-tab", Outdent, None),
            KeyBinding::new("secondary-n", NewDocument, None),
            KeyBinding::new("secondary-o", OpenDocument, None),
            KeyBinding::new("secondary-s", SaveDocument, None),
            KeyBinding::new("secondary-shift-s", SaveDocumentAs, None),
            KeyBinding::new("secondary-shift-h", ExportHtml, None),
            KeyBinding::new("secondary-alt-shift-h", ExportPlainHtml, None),
            KeyBinding::new("secondary-shift-p", ExportPdf, None),
            KeyBinding::new("secondary-shift-l", ExportLatex, None),
            KeyBinding::new("secondary-shift-d", ExportDocx, None),
            KeyBinding::new("secondary-shift-g", ExportPng, None),
            KeyBinding::new("secondary-alt-shift-g", ExportJpeg, None),
            KeyBinding::new("secondary-shift-v", ToggleViewMode, None),
            KeyBinding::new("secondary-alt-1", SetEditMode, None),
            KeyBinding::new("secondary-alt-4", SetVisualEditMode, None),
            KeyBinding::new("secondary-alt-2", SetSplitPreviewMode, None),
            KeyBinding::new("secondary-alt-3", SetReadMode, None),
            // NB: no `secondary-b` for the sidebar — that collides with Bold.
            // Use Ctrl/Cmd+Shift+B instead.
            KeyBinding::new("secondary-shift-b", ToggleSidebar, None),
            KeyBinding::new("secondary-shift-f", ToggleFileTree, None),
            KeyBinding::new("secondary-alt-f", FocusFileTreeSearch, None),
            KeyBinding::new("escape", ClearFileTreeSearch, None),
            KeyBinding::new("f5", RefreshFileTree, None),
            KeyBinding::new("secondary-alt-n", CreateTreeFile, None),
            KeyBinding::new("secondary-alt-shift-n", CreateTreeFolder, None),
            KeyBinding::new("f2", RenameTreeEntry, None),
            KeyBinding::new("secondary-delete", DeleteTreeEntry, None),
            KeyBinding::new("f6", ToggleOutline, None),
            KeyBinding::new("secondary-shift-t", CycleTheme, None),
            KeyBinding::new("f7", ToggleFocusMode, None),
            KeyBinding::new("f8", ToggleTypewriterMode, None),
            KeyBinding::new("secondary-shift-4", ToggleCodeLineNumbers, None),
            KeyBinding::new("secondary-shift-m", FormatTable, None),
            KeyBinding::new("secondary-alt-enter", TableAddRow, None),
            KeyBinding::new("secondary-alt-backspace", TableDeleteRow, None),
            KeyBinding::new("secondary-alt-up", TableMoveRowUp, None),
            KeyBinding::new("secondary-alt-down", TableMoveRowDown, None),
            KeyBinding::new("secondary-alt-right", TableAddColumn, None),
            KeyBinding::new("secondary-alt-left", TableDeleteColumn, None),
            KeyBinding::new("secondary-f", ShowFind, None),
            KeyBinding::new("secondary-h", ShowReplace, None),
            KeyBinding::new("f3", FindNext, None),
            KeyBinding::new("shift-f3", FindPrevious, None),
            KeyBinding::new("secondary-comma", ShowPreferences, None),
            KeyBinding::new("f1", ShowShortcuts, None),
            KeyBinding::new("secondary-q", Quit, None),
            KeyBinding::new("ctrl-tab", NextTab, None),
            KeyBinding::new("ctrl-shift-tab", PrevTab, None),
            KeyBinding::new("secondary-t", OpenInNewTab, None),
            KeyBinding::new("secondary-w", CloseTab, None),
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
