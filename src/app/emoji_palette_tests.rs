use super::*;
use gpui::{Modifiers, TestAppContext, VisualTestContext};

fn assert_near(actual: Point<Pixels>, expected: Point<Pixels>) {
    // Anchored overlays round their final origin to physical pixels.
    assert!(
        f32::from(actual.x - expected.x).abs() <= 1.,
        "{actual:?} vs {expected:?}"
    );
    assert!(
        f32::from(actual.y - expected.y).abs() <= 1.,
        "{actual:?} vs {expected:?}"
    );
}

fn assert_compact_candidates(cx: &mut VisualTestContext) -> Bounds<Pixels> {
    let panel = cx.debug_bounds("emoji-completer-palette").unwrap();
    assert_eq!(panel.size.width, px(248.));
    assert!(
        panel.size.height >= px(240.) && panel.size.height <= px(280.),
        "taller palette: {panel:?}"
    );
    let grid = cx.debug_bounds("emoji-completer-grid").unwrap();
    assert_eq!(grid.size.height, px(190.), "six rows with five 2px gaps");
    let first = cx.debug_bounds("emoji-completer-0").unwrap();
    for index in 0..EMOJI_PALETTE_LIMIT {
        let button = cx
            .debug_bounds(super::root_view::emoji_completer_element_id(index))
            .unwrap();
        assert_eq!(button.size, size(px(36.), px(30.)));
        assert!(button.left() >= grid.left() && button.right() <= grid.right());
        assert!(button.top() >= grid.top() && button.bottom() <= grid.bottom());
        assert_eq!(button.left(), first.left() + px((index % 6) as f32 * 38.));
        assert_eq!(button.top(), first.top() + px((index / 6) as f32 * 32.));
    }
    let first = cx.debug_bounds("emoji-completer-0").unwrap();
    let second = cx.debug_bounds("emoji-completer-1").unwrap();
    let sixth = cx.debug_bounds("emoji-completer-5").unwrap();
    let seventh = cx.debug_bounds("emoji-completer-6").unwrap();
    assert_eq!(second.left() - first.right(), px(2.));
    assert_eq!(sixth.top(), first.top());
    assert_eq!(seventh.top() - first.bottom(), px(2.));
    assert!(cx.debug_bounds("emoji-completer-caption").is_some());
    panel
}

#[gpui::test]
fn emoji_palette_matches_slash_geometry_at_block_starts(cx: &mut TestAppContext) {
    for prefix in ["", "# ", "- ", "> ", "above\n\n"] {
        let source = format!("{prefix}:");
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
            app.active_tab_mut().selected_range = source.len()..source.len();
            app.view_mode = ViewMode::VisualEdit;
            app
        });
        cx.simulate_resize(size(px(1000.), px(700.)));
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        cx.run_until_parked();
        let emoji = assert_compact_candidates(cx);
        app.update(cx, |app, _| {
            let caret = app.active_tab().visual_caret_bounds.unwrap();
            assert_near(
                emoji.origin,
                point(caret.left() + px(8.), caret.bottom() + px(8.)),
            );
        });
        app.update(cx, |app, cx| {
            let source = format!("{prefix}/");
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
            app.active_tab_mut().selected_range = source.len()..source.len();
            cx.notify();
        });
        cx.run_until_parked();
        let slash = cx.debug_bounds("slash-command-palette").unwrap();
        let slash_button = cx.debug_bounds("slash-command-0").unwrap();
        assert_eq!(emoji.size.width, slash.size.width);
        assert_eq!(slash_button.size, size(px(36.), px(30.)));
        app.update(cx, |app, _| {
            let caret = app.active_tab().visual_caret_bounds.unwrap();
            assert_near(
                slash.origin,
                point(caret.left() + px(8.), caret.bottom() + px(8.)),
            );
        });
    }
}

#[gpui::test]
fn emoji_palette_hover_pointer_and_undo_preserve_exact_source(cx: &mut TestAppContext) {
    let source = ":原段落 with spaces\r\n";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let (version, blocks) = app.update(cx, |app, _| {
        (
            app.active_tab().document.version(),
            app.active_tab().document.visual_blocks_shared(),
        )
    });
    let button = cx.debug_bounds("emoji-completer-7").unwrap();
    cx.simulate_event(MouseMoveEvent {
        position: button.center(),
        ..Default::default()
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.emoji_completer.as_ref().unwrap().selected, 7);
        assert_eq!(app.active_tab().document.version(), version);
        assert_eq!(app.active_tab().selected_range, 1..1);
        assert!(app.active_tab().undo_stack.is_empty());
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
    let name = emoji_shortcodes_matching("")[7].0;
    cx.simulate_click(button.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().document.text(),
            format!(":{name}:原段落 with spaces\r\n")
        );
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().selected_range, 1..1);
    });
}

#[gpui::test]
fn emoji_palette_filter_keyboard_and_escape_keep_existing_semantics(cx: &mut TestAppContext) {
    for confirmation in 0..3 {
        let confirm = confirmation != 0;
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(":s"))];
            app.active_tab_mut().selected_range = 2..2;
            app.view_mode = ViewMode::VisualEdit;
            app
        });
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        cx.run_until_parked();
        let (version, blocks) = app.update(cx, |app, _| {
            assert_eq!(app.emoji_completer.as_ref().unwrap().query.query, "s");
            (
                app.active_tab().document.version(),
                app.active_tab().document.visual_blocks_shared(),
            )
        });
        assert!(cx.debug_bounds("emoji-completer-0").is_some());
        cx.dispatch_action(Down);
        app.update(cx, |app, _| {
            assert_eq!(app.emoji_completer.as_ref().unwrap().selected, 1);
            assert_eq!(app.active_tab().document.version(), version);
            assert!(Arc::ptr_eq(
                &blocks,
                &app.active_tab().document.visual_blocks_shared()
            ));
        });
        if confirm {
            if confirmation == 1 {
                cx.dispatch_action(InsertNewline);
            } else {
                cx.dispatch_action(Indent);
            }
            app.update(cx, |app, _| {
                let name = emoji_shortcodes_matching("s")[1].0;
                assert_eq!(app.active_tab().document.text(), format!(":{name}:"));
                assert_eq!(app.active_tab().undo_stack.len(), 1);
            });
            cx.dispatch_action(Undo);
        } else {
            cx.dispatch_action(ClearFileTreeSearch);
        }
        app.update(cx, |app, _| {
            assert_eq!(app.active_tab().document.text(), ":s");
            assert_eq!(app.active_tab().selected_range, 2..2);
            if !confirm {
                assert!(app.emoji_completer.is_none());
                assert_eq!(app.active_tab().document.version(), version);
                assert!(app.active_tab().undo_stack.is_empty());
            }
        });
    }
}

#[gpui::test]
fn emoji_palette_uses_source_caret_and_fits_viewport(cx: &mut TestAppContext) {
    for mode in [ViewMode::Edit, ViewMode::VisualEdit] {
        let source = format!(
            "{}:\n{}",
            "paragraph before\n\n".repeat(4),
            "tail\n".repeat(80)
        );
        let cursor = source.find(':').unwrap() + 1;
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
            app.active_tab_mut().selected_range = cursor..cursor;
            app.view_mode = mode;
            // Geometry from a previous surface must not move source completion.
            if matches!(mode, ViewMode::Edit) {
                app.active_tab_mut().visual_caret_bounds = Some(Bounds::new(
                    point(px(850.), px(600.)),
                    size(px(2.), px(24.)),
                ));
            }
            app
        });
        cx.simulate_resize(size(px(1000.), px(700.)));
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        cx.run_until_parked();
        let panel = cx.debug_bounds("emoji-completer-palette").unwrap();
        app.update(cx, |app, _| {
            let tab = app.active_tab();
            let position = if matches!(mode, ViewMode::Edit) {
                let position = tab
                    .layout_point_for_offset(
                        tab.cursor_offset(),
                        tab.last_bounds.unwrap(),
                        tab.line_height,
                    )
                    .unwrap();
                position + point(px(0.), tab.line_height)
            } else {
                let caret = tab.visual_caret_bounds.unwrap();
                point(caret.left(), caret.bottom())
            };
            assert_near(panel.origin, position + point(px(8.), px(8.)));
        });
        if matches!(mode, ViewMode::Edit) {
            app.update(cx, |app, cx| {
                let scroll = &app.active_tab().editor_scroll;
                scroll.set_offset(scroll.offset() + point(px(0.), px(-40.)));
                cx.notify();
            });
            cx.run_until_parked();
            let scrolled = cx.debug_bounds("emoji-completer-palette").unwrap();
            assert_eq!(scrolled.origin, panel.origin + point(px(0.), px(-40.)));
        }
        cx.simulate_resize(size(px(420.), px(280.)));
        cx.run_until_parked();
        let panel = cx.debug_bounds("emoji-completer-palette").unwrap();
        assert!(
            panel.left() >= px(0.) && panel.right() <= px(420.),
            "{panel:?}"
        );
        // Anchored rounds its origin to physical pixels; fractional text
        // metrics can leave the outer border within one pixel of the edge.
        assert!(
            panel.top() >= px(-1.) && panel.bottom() <= px(281.),
            "{panel:?}"
        );
    }
}

#[gpui::test]
fn emoji_completion_displays_every_candidate_immediately(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(":"))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for (name, glyph) in emoji_shortcodes_matching("") {
        for suffix in ["", "中文 suffix\r\n"] {
            let source = format!(":{suffix}");
            app.update(cx, |app, cx| {
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                app.active_tab_mut().selected_range = 1..1;
                cx.notify();
            });
            cx.run_until_parked();
            app.update(cx, |app, cx| {
                let query = app.emoji_completer.as_ref().unwrap().query.clone();
                app.execute_emoji_completion(query, name, cx);
            });
            cx.run_until_parked();
            app.update(cx, |app, _| {
                let tab = app.active_tab();
                assert_eq!(tab.document.text(), format!(":{name}:{suffix}"));
                let (display, _) = tab
                    .visual_last_projection
                    .as_ref()
                    .expect("painted projection");
                assert!(
                    display.contains(glyph),
                    "{name}: displayed {display:?}, expected {glyph:?}"
                );
                assert!(
                    !display.contains(&format!(":{name}:")),
                    "{name}: raw shortcode {display:?}"
                );
            });
        }
    }
}

#[gpui::test]
fn emoji_completion_keeps_consecutive_lines_rendered_while_focused(cx: &mut TestAppContext) {
    for newline in ["\n", "\r\n"] {
        let source = format!(
            ":book:{newline}{newline}:bug:{newline}:+1:{newline}:book:{newline}:-1:{newline}:check"
        );
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
            app.active_tab_mut().selected_range = source.len()..source.len();
            app.view_mode = ViewMode::VisualEdit;
            app
        });
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        cx.run_until_parked();
        cx.dispatch_action(InsertNewline);
        cx.run_until_parked();
        app.update(cx, |app, _| {
            let tab = app.active_tab();
            assert_eq!(tab.document.text(), format!("{source}:"));
            let display = &tab.visual_last_projection.as_ref().unwrap().0;
            for glyph in ["🐛", "👍", "📘", "👎", "✅"] {
                assert!(
                    display.contains(glyph),
                    "caret remains in emoji block: {display:?}"
                );
            }
            assert_eq!(tab.undo_stack.len(), 1);
        });
        cx.dispatch_action(Undo);
        app.update(cx, |app, _| {
            assert_eq!(app.active_tab().document.text(), source)
        });
    }
}

#[gpui::test]
fn emoji_palette_browses_past_first_window_and_confirms_the_visible_candidate(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(":"))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let panel = assert_compact_candidates(cx);
    let (version, blocks) = app.update(cx, |app, _| {
        (
            app.active_tab().document.version(),
            app.active_tab().document.visual_blocks_shared(),
        )
    });
    for _ in 0..EMOJI_PALETTE_LIMIT {
        cx.dispatch_action(Down);
    }
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(
            app.emoji_completer.as_ref().unwrap().selected,
            EMOJI_PALETTE_LIMIT
        );
        assert_eq!(app.active_tab().document.version(), version);
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
    assert_eq!(assert_compact_candidates(cx).size, panel.size);
    cx.simulate_event(gpui::ScrollWheelEvent {
        position: panel.center(),
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    app.update(cx, |app, _| {
        assert_eq!(
            app.emoji_completer.as_ref().unwrap().selected,
            EMOJI_PALETTE_LIMIT * 2
        );
        assert_eq!(app.active_tab().document.version(), version);
    });
    let button = cx.debug_bounds("emoji-completer-0").unwrap();
    let name = emoji_shortcodes_matching("")[EMOJI_PALETTE_LIMIT * 2].0;
    cx.simulate_click(button.center(), Modifiers::none());
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), format!(":{name}:"))
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), ":")
    });
}
