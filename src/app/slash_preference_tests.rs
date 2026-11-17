use super::*;
use gpui::{Modifiers, TestAppContext};

#[gpui::test]
fn slash_preference_controls_typed_markers_and_leaves_enter_literal(cx: &mut TestAppContext) {
    for enabled in [true, false] {
        for marker in ["/", "、"] {
            let (app, cx) = cx.add_window_view(|_, cx| {
                let mut app = MarkionApp::new(cx);
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(""))];
                app.view_mode = ViewMode::VisualEdit;
                assert!(app.slash_commands_enabled);
                if !enabled {
                    app.toggle_slash_commands(cx);
                }
                app
            });
            cx.update(|window, cx| {
                cx.clear_key_bindings();
                bind_app_keys(cx, &BTreeMap::new());
                window.focus(&app.read(cx).focus_handle);
                window.activate_window();
            });
            cx.simulate_input(marker);
            cx.run_until_parked();
            app.update(cx, |app, _| {
                assert_eq!(app.slash_commands.is_some(), enabled);
                assert_eq!(app.active_tab().document.text(), marker);
            });
            cx.simulate_input("h2");
            cx.run_until_parked();
            app.update(cx, |app, _| {
                assert_eq!(app.slash_commands.is_some(), enabled);
                assert_eq!(app.active_tab().document.text(), format!("{marker}h2"));
            });
            cx.simulate_keystrokes("enter");
            app.update(cx, |app, _| {
                assert_eq!(
                    app.active_tab().document.text(),
                    if enabled {
                        "## ".to_string()
                    } else {
                        format!("{marker}h2\n")
                    }
                );
            });
        }
    }
}

#[gpui::test]
fn slash_preference_live_toggle_preserves_state_and_rejects_pending_commands(
    cx: &mut TestAppContext,
) {
    let source = "/h2原段落 with spaces\r\n";
    let app = cx.new(|cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = 3..3;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    app.update(cx, |app, cx| {
        app.sync_slash_command_state(cx);
        let query = app.slash_commands.as_ref().unwrap().query.clone();
        let tab = app.active_tab();
        let version = tab.document.version();
        let dirty = tab.document.is_dirty();
        let visual = tab.document.visual_blocks_shared();
        let preview = tab.document.preview_blocks_shared();
        let stats = tab.document.stats();
        let outline = tab.document.outline();
        let history = (tab.undo_stack.len(), tab.redo_stack.len());

        app.toggle_slash_commands(cx);
        assert!(!app.current_preferences().slash_commands_enabled);
        assert!(app.slash_commands.is_none());
        assert!(!app.confirm_selected_slash_command(cx));
        app.execute_slash_command(
            query,
            SlashPaletteCommand::Block(SlashCommand::Heading(2)),
            cx,
        );
        for _ in 0..3 {
            app.sync_slash_command_state(cx);
            assert!(app.slash_commands.is_none());
        }
        app.toggle_slash_commands(cx);
        assert!(app.slash_commands.is_some());

        let tab = app.active_tab();
        assert_eq!(tab.document.text(), source);
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.is_dirty(), dirty);
        assert_eq!(tab.selected_range, 3..3);
        assert_eq!((tab.undo_stack.len(), tab.redo_stack.len()), history);
        assert!(Arc::ptr_eq(&visual, &tab.document.visual_blocks_shared()));
        assert!(Arc::ptr_eq(&preview, &tab.document.preview_blocks_shared()));
        assert_eq!(tab.document.stats(), stats);
        assert_eq!(tab.document.outline(), outline);
        assert!(app.confirm_selected_slash_command(cx));
        assert_eq!(
            app.active_tab().document.text(),
            "## 原段落 with spaces\r\n"
        );
        assert_eq!(app.active_tab().undo_stack.len(), history.0 + 1);
    });
}

#[gpui::test]
fn slash_preference_does_not_disable_emoji_or_block_commands(cx: &mut TestAppContext) {
    let app = cx.new(|cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(":smi"))];
        app.active_tab_mut().selected_range = 4..4;
        app.view_mode = ViewMode::VisualEdit;
        app.toggle_slash_commands(cx);
        app
    });
    app.update(cx, |app, cx| {
        app.sync_emoji_completer_state(cx);
        assert!(app.emoji_completer.is_some());
        let blocks = app.active_tab().document.visual_blocks_shared();
        let target = BlockTarget::from_block(app.active_tab().document.version(), &blocks[0]);
        app.open_visual_block_menu(target, point(px(20.), px(20.)), cx);
        assert!(app.block_menu.is_some());
        app.activate_visual_block_menu_item(
            BlockMenuItem::Transform(BlockTransform::Heading(2)),
            None,
            cx,
        );
        assert_eq!(app.active_tab().document.text(), "## :smi");

        let path = PathBuf::from("example.png");
        app.tabs = vec![EditorTab::new_image(
            path.clone(),
            PreviewImageKey::from_local_path(&path),
        )];
        app.dismiss_visual_block_menu();
        app.toggle_slash_commands(cx);
        assert!(app.slash_commands_enabled);
        assert!(app.slash_commands.is_none());
    });
}

#[gpui::test]
fn slash_preference_general_control_persists_and_reset_restores_enabled(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = path.clone();
        app.preferences_panel_open = true;
        app.preferences_tab = PreferencesTab::General;
        app
    });
    cx.simulate_resize(size(px(1000.), px(800.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).preferences_panel_focus);
        window.activate_window();
    });
    cx.run_until_parked();
    let mut button = None;
    for offset in (0..=1200).step_by(80) {
        app.update(cx, |app, cx| {
            app.preferences_general_scroll
                .set_offset(point(px(0.), px(-(offset as f32))));
            cx.notify();
        });
        cx.run_until_parked();
        if let Some(row) = cx.debug_bounds("preference-slash-commands") {
            let body = app.update(cx, |app, _| app.preferences_general_scroll.bounds());
            let point = point(row.right() - px(24.), row.center().y);
            if body.contains(&point) {
                button = Some(point);
                break;
            }
        }
    }
    let button = button.expect("slash preference should be reachable in the General panel");
    cx.simulate_mouse_down(button, MouseButton::Left, Modifiers::none());
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: button,
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    let (text, version, blocks) = app.update(cx, |app, _| {
        assert!(!app.slash_commands_enabled);
        assert!(!load_app_preferences(&path).unwrap().slash_commands_enabled);
        let doc = &app.active_tab().document;
        (
            doc.text().to_string(),
            doc.version(),
            doc.visual_blocks_shared(),
        )
    });
    cx.dispatch_action(ResetPreferences);
    let reset = app.update(cx, |app, _| {
        t(app.language, Msg::DialogButtonReset).to_string()
    });
    cx.simulate_prompt_answer(&reset);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.slash_commands_enabled);
        assert!(load_app_preferences(&path).unwrap().slash_commands_enabled);
        assert_eq!(app.active_tab().document.text(), text);
        assert_eq!(app.active_tab().document.version(), version);
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
    for language in Language::all() {
        assert!(t(*language, Msg::PrefPanelSlashCommands).contains('/'));
    }
}
