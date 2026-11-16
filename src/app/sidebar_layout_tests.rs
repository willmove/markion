use super::*;
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use markion::FileTreeFileKind;

fn small_file_tree(root: &Path) -> FileTree {
    FileTree {
        root: root.to_path_buf(),
        show_hidden: false,
        entries: (0..3)
            .map(|index| FileTreeEntry {
                path: root.join(format!("note-{index}.md")),
                name: format!("note-{index}.md"),
                depth: 0,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
            })
            .collect(),
    }
}

#[gpui::test]
fn sidebar_tab_bar_keeps_content_gap_across_themes_widths_and_tabs(cx: &mut TestAppContext) {
    let root = PathBuf::from("/tmp/markion-sidebar-tab-gap");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "# Root\n\n## Child\n",
        ))];
        app.workspace_root = root.clone();
        app.file_tree = Some(small_file_tree(&root));
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Files;
        app
    });

    for theme in [AppTheme::Paper, AppTheme::Ink] {
        for width in [520., 1000.] {
            cx.simulate_resize(size(px(width), px(500.)));
            app.update(cx, |app, cx| {
                app.theme = theme;
                app.custom_theme = None;
                app.selected_theme_name = theme.name().to_string();
                cx.notify();
            });
            cx.run_until_parked();

            for (tab, content_selector) in [
                (SidebarTab::Files, "file-tree-workspace-root"),
                (SidebarTab::Outline, "outline-heading-row-0"),
            ] {
                app.update(cx, |app, cx| app.set_sidebar_tab(tab, cx));
                cx.run_until_parked();

                let tab_bar = cx
                    .debug_bounds("sidebar-tab-bar")
                    .expect("sidebar tab bar should render");
                let content = cx
                    .debug_bounds(content_selector)
                    .unwrap_or_else(|| panic!("{content_selector} should render"));
                let actual_gap = f32::from(content.top()) - f32::from(tab_bar.bottom());
                assert!(
                    (actual_gap - SIDEBAR_TAB_CONTENT_GAP).abs() < 0.01,
                    "{tab:?} content should stay {SIDEBAR_TAB_CONTENT_GAP}px below the tab bar in {} at {width}px, got {actual_gap}px",
                    theme.name()
                );
            }
        }
    }
}

#[gpui::test]
fn outline_position_docks_beside_every_document_mode_and_keeps_files_left(cx: &mut TestAppContext) {
    let root = PathBuf::from("/tmp/markion-separated-sidebar-layout");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("# Root\n\n## Child\n")),
            EditorTab::new(MarkdownDocument::from_text("# Second\n")),
        ];
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Outline;
        app.workspace_root = root.clone();
        app.file_tree = Some(small_file_tree(&root));
        app
    });
    cx.simulate_resize(size(px(1200.), px(620.)));

    for theme in [AppTheme::Paper, AppTheme::Ink] {
        for position in [OutlinePosition::Left, OutlinePosition::Right] {
            for mode in [
                ViewMode::Edit,
                ViewMode::VisualEdit,
                ViewMode::Split,
                ViewMode::Read,
            ] {
                app.update(cx, |app, cx| {
                    app.theme = theme;
                    app.view_mode = mode;
                    app.set_sidebar_tab(SidebarTab::Outline, cx);
                    app.set_outline_position(position, cx);
                    cx.notify();
                });
                cx.run_until_parked();
                let sidebar = cx.debug_bounds("workspace-sidebar").unwrap();
                let document = cx.debug_bounds("document-workspace-column").unwrap();
                let divider = cx.debug_bounds("sidebar-resize-container").unwrap();
                assert_eq!(sidebar.top(), document.top());
                assert_eq!(sidebar.bottom(), document.bottom());
                assert_eq!(sidebar.right(), divider.left());
                assert_eq!(divider.right(), document.left());
                if position == OutlinePosition::Right {
                    let outline = cx.debug_bounds("workspace-outline").unwrap();
                    let outline_divider = cx.debug_bounds("outline-resize-container").unwrap();
                    let file_row = cx
                        .debug_bounds(
                            "file-tree-row:/tmp/markion-separated-sidebar-layout/note-0.md",
                        )
                        .unwrap();
                    let heading = cx.debug_bounds("outline-heading-label-1").unwrap();
                    assert_eq!(document.right(), outline_divider.left());
                    assert_eq!(outline_divider.right(), outline.left());
                    assert!(
                        file_row.left() >= sidebar.left() && file_row.right() <= sidebar.right()
                    );
                    assert!(heading.left() >= outline.left() && heading.right() <= outline.right());
                    assert_eq!(
                        cx.debug_bounds("files-pane-header").unwrap().top(),
                        cx.debug_bounds("outline-pane-header").unwrap().top()
                    );
                }
                assert!(cx.debug_bounds("outline-heading-label-1").is_some());
            }
        }
    }

    app.update(cx, |app, cx| {
        app.set_outline_position(OutlinePosition::Right, cx);
        app.set_sidebar_tab(SidebarTab::Files, cx);
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("workspace-sidebar").unwrap().right()
            < cx.debug_bounds("document-workspace-column").unwrap().left()
    );

    // The docked AI panel reserves space outside the complete workspace,
    // including a right-side outline, so neither panel covers the other.
    app.update(cx, |app, cx| {
        app.set_sidebar_tab(SidebarTab::Outline, cx);
        app.ai_ui.open = true;
        cx.notify();
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("workspace-outline").unwrap().right()
            <= cx.debug_bounds("ai-panel").unwrap().left()
    );
}

#[gpui::test]
fn right_outline_and_files_have_independent_visibility_controls(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let preferences_path = dir.path().join("config.toml");
    let root = dir.path().join("workspace");
    std::fs::create_dir(&root).unwrap();
    for index in 0..3 {
        std::fs::write(
            root.join(format!("note-{index}.md")),
            "# Root\n\n## Child\n",
        )
        .unwrap();
    }
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = preferences_path.clone();
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "# Root\n\n## Child\n",
        ))];
        app.outline_position = OutlinePosition::Right;
        app.workspace_root = root.clone();
        app.file_tree = Some(small_file_tree(&root));
        app
    });
    cx.simulate_resize(size(px(1200.), px(760.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("sidebar-tab-bar").is_none(),
        "detached panes have no shared tab switch"
    );
    // GPUI's test query requires a static selector, including for temp paths.
    let file_selector: &'static str =
        Box::leak(format!("file-tree-row:{}", root.join("note-0.md").display()).into_boxed_str());
    let file_before = cx.debug_bounds(file_selector).unwrap();
    let outline_before = cx.debug_bounds("workspace-outline").unwrap();
    cx.simulate_click(file_before.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.files_panel_visible() && app.outline_panel_visible());
        assert_eq!(
            app.active_tab().document.path(),
            Some(root.join("note-0.md").as_path())
        );
    });

    cx.dispatch_action(ToggleOutline);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.files_panel_visible());
        assert!(!app.outline_panel_visible());
    });
    assert_eq!(cx.debug_bounds(file_selector).unwrap(), file_before);
    assert_eq!(
        cx.debug_bounds("document-workspace-column")
            .unwrap()
            .right(),
        cx.debug_bounds("workspace-row").unwrap().right()
    );
    cx.dispatch_action(ToggleOutline);
    cx.dispatch_action(ToggleFileTree);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(!app.files_panel_visible());
        assert!(app.outline_panel_visible());
    });
    assert_eq!(
        cx.debug_bounds("workspace-outline").unwrap(),
        outline_before
    );
    assert_eq!(
        cx.debug_bounds("document-workspace-column").unwrap().left(),
        cx.debug_bounds("workspace-row").unwrap().left()
    );

    cx.dispatch_action(ToggleSidebar);
    app.update(cx, |app, _| assert!(!app.sidebars_visible()));
    cx.dispatch_action(ToggleSidebar);
    app.update(cx, |app, _| {
        assert!(app.files_panel_visible() && app.outline_panel_visible())
    });

    app.update(cx, |app, cx| {
        app.preferences_panel_open = true;
        app.preferences_general_scroll
            .set_offset(point(px(0.), px(-250.)));
        cx.notify();
    });
    cx.run_until_parked();
    for selector in ["preferences-files-visible", "preferences-outline-visible"] {
        let control = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(control.center(), Modifiers::none());
        cx.run_until_parked();
    }
    let saved = markion::load_app_preferences(&preferences_path).unwrap();
    assert!(!saved.sidebar_visible && !saved.outline_visible);
    app.update(cx, |app, cx| {
        app.set_outline_position(OutlinePosition::Left, cx);
        app.set_outline_position(OutlinePosition::Right, cx);
        assert!(!app.sidebars_visible());
        app.select_preferences_sidebar_tab(SidebarTab::Outline, cx);
        assert!(!app.sidebar_visible && app.outline_visible);
        app.set_outline_position(OutlinePosition::Left, cx);
        assert!(app.sidebar_visible);
        assert_eq!(app.sidebar_tab, SidebarTab::Outline);
    });
}

#[gpui::test]
fn outline_position_controls_persist_and_preserve_document_state(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = path.clone();
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "# Root\n\n## Child\n\n## Sibling\n",
        ))];
        app.sidebar_tab = SidebarTab::Outline;
        app.preferences_panel_open = true;
        app.language = Language::ZhHans;
        app.active_tab_mut().selected_range = 2..6;
        app.active_tab_mut().push_undo_snapshot();
        app
    });
    cx.simulate_resize(size(px(1200.), px(760.)));
    cx.run_until_parked();
    let (version, text, preview, visual, selected, undo_len, dirty) = app.update(cx, |app, _| {
        let tab = app.active_tab_mut();
        let headings = tab.document.outline();
        let projection = tab.outline_projection(&headings, None);
        tab.toggle_outline_node(projection.rows[0].key.clone());
        (
            tab.document.version(),
            tab.document.text().to_string(),
            tab.document.preview_blocks_shared(),
            tab.document.visual_blocks_shared(),
            tab.selected_range.clone(),
            tab.undo_stack.len(),
            tab.document.is_dirty(),
        )
    });
    // Scroll the position row into the General panel's visible body.
    app.update(cx, |app, cx| {
        app.preferences_general_scroll
            .set_offset(point(px(0.), px(-250.)));
        cx.notify();
    });
    cx.run_until_parked();
    for (position, selector) in [
        (OutlinePosition::Right, "preferences-outline-position-right"),
        (OutlinePosition::Left, "preferences-outline-position-left"),
    ] {
        let control = cx
            .debug_bounds(selector)
            .expect("position choice should render");
        cx.simulate_click(control.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            markion::load_app_preferences(&path)
                .unwrap()
                .outline_position,
            position
        );
        app.update(cx, |app, _| {
            assert_eq!(app.current_preferences().outline_position, position);
            let tab = app.active_tab();
            assert_eq!(tab.document.version(), version);
            assert_eq!(tab.document.text(), text);
            assert_eq!(tab.document.is_dirty(), dirty);
            assert_eq!(tab.selected_range, selected);
            assert_eq!(tab.undo_stack.len(), undo_len);
            assert_eq!(tab.outline_folding.borrow().collapsed_keys().len(), 1);
            assert!(Arc::ptr_eq(&preview, &tab.document.preview_blocks_shared()));
            assert!(Arc::ptr_eq(&visual, &tab.document.visual_blocks_shared()));
        });
    }
    app.update(cx, |app, cx| {
        app.preferences_panel_open = false;
        app.sidebar_visible = false;
        app.set_outline_position(OutlinePosition::Right, cx);
        assert!(!app.sidebar_visible);
        cx.notify();
    });
    cx.run_until_parked();
    let workspace = cx.debug_bounds("workspace-row").unwrap();
    let document = cx.debug_bounds("document-workspace-column").unwrap();
    assert_eq!(document.left(), workspace.left());
    assert_eq!(document.right(), workspace.right());
    app.update(cx, |app, cx| {
        app.select_preferences_sidebar_tab(SidebarTab::Outline, cx);
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.outline_position, OutlinePosition::Right);
        assert!(app.outline_visible);
        assert!(
            !app.sidebar_visible,
            "showing Outline must leave hidden Files alone"
        );
    });
}

fn drag_sidebar(cx: &mut VisualTestContext, selector: &'static str, delta: f32) {
    let start = cx.debug_bounds(selector).unwrap().center();
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    for step in 1..=4 {
        cx.simulate_mouse_move(
            point(start.x + px(delta * step as f32 / 4.), start.y),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();
    }
    cx.simulate_mouse_up(
        point(start.x + px(delta), start.y),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.run_until_parked();
}

#[gpui::test]
fn outline_position_resize_uses_inner_edge_and_preserves_width_limits(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.session_path = dir.path().join("session.toml");
        app.sidebar_tab = SidebarTab::Outline;
        app.outline_position = OutlinePosition::Right;
        app
    });
    cx.simulate_resize(size(px(1400.), px(620.)));
    for right_outline in [false, true] {
        let selector = if right_outline {
            "outline-resize-handle"
        } else {
            "sidebar-resize-handle"
        };
        app.update(cx, |app, cx| {
            app.sidebar_width = DEFAULT_SIDEBAR_WIDTH;
            app.outline_width = DEFAULT_SIDEBAR_WIDTH;
            cx.notify();
        });
        cx.run_until_parked();
        let direction = if right_outline { -1. } else { 1. };
        drag_sidebar(cx, selector, 60. * direction);
        app.update(cx, |app, _| {
            let (width, other) = if right_outline {
                (app.outline_width, app.sidebar_width)
            } else {
                (app.sidebar_width, app.outline_width)
            };
            assert!((width - (DEFAULT_SIDEBAR_WIDTH + 60.)).abs() <= 1.);
            assert_eq!(other, DEFAULT_SIDEBAR_WIDTH);
        });
        drag_sidebar(cx, selector, 400. * direction);
        app.update(cx, |app, _| {
            assert_eq!(
                if right_outline {
                    app.outline_width
                } else {
                    app.sidebar_width
                },
                SIDEBAR_MAX_WIDTH
            )
        });
        drag_sidebar(cx, selector, -400. * direction);
        app.update(cx, |app, _| {
            assert_eq!(
                if right_outline {
                    app.outline_width
                } else {
                    app.sidebar_width
                },
                SIDEBAR_MIN_WIDTH
            )
        });

        app.update(cx, |app, cx| {
            if right_outline {
                app.sidebar_width = 310.;
            } else {
                app.outline_width = 310.;
            }
            cx.notify();
        });
        cx.run_until_parked();
        let handle = cx.debug_bounds(selector).unwrap().center();
        cx.simulate_event(MouseDownEvent {
            position: handle,
            button: MouseButton::Left,
            modifiers: Modifiers::none(),
            click_count: 2,
            first_mouse: false,
        });
        cx.simulate_event(MouseUpEvent {
            position: handle,
            button: MouseButton::Left,
            modifiers: Modifiers::none(),
            click_count: 2,
        });
        cx.run_until_parked();
        app.update(cx, |app, _| {
            assert_eq!(
                if right_outline {
                    app.outline_width
                } else {
                    app.sidebar_width
                },
                DEFAULT_SIDEBAR_WIDTH
            );
            assert_eq!(
                if right_outline {
                    app.sidebar_width
                } else {
                    app.outline_width
                },
                310.
            );
            app.flush_layout();
            // App session writes are disabled in GPUI tests; exercise the
            // storage adapter explicitly against this test's temporary file.
            markion::save_session_state(&app.session_path, &app.session).unwrap();
            let saved = markion::load_session_state(&app.session_path).unwrap();
            assert_eq!(saved.layout.sidebar_width, Some(app.sidebar_width));
            assert_eq!(saved.layout.outline_width, Some(app.outline_width));
        });
    }
}

#[gpui::test]
fn outline_position_reset_restores_left_and_keeps_saved_width(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = path.clone();
        app.outline_position = OutlinePosition::Right;
        app.sidebar_width = 310.;
        app.outline_width = 270.;
        app.sync_layout_panes();
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.dispatch_action(ResetPreferences);
    cx.simulate_prompt_answer(t(Language::En, Msg::DialogButtonReset));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.outline_position, OutlinePosition::Left);
        assert_eq!(app.sidebar_width, 310.);
        assert_eq!(app.session.layout.sidebar_width, Some(310.));
        assert_eq!(app.session.layout.outline_width, Some(270.));
    });
    assert_eq!(
        markion::load_app_preferences(&path)
            .unwrap()
            .outline_position,
        OutlinePosition::Left
    );
}
