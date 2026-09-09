use super::*;
use gpui::{TestAppContext, VisualTestContext};

fn visual_test_app<'a>(
    cx: &'a mut TestAppContext,
    source: &str,
    cursor: usize,
) -> (Entity<MarkionApp>, &'a mut VisualTestContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    (app, cx)
}

fn assert_current_visual_caret(app: &Entity<MarkionApp>, cx: &mut App, label: &str) {
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let observation = tab
            .visual_last_caret_paint
            .as_ref()
            .unwrap_or_else(|| panic!("{label}: current frame did not publish caret geometry"));
        assert_eq!(
            observation.frame_generation, tab.visual_frame_generation,
            "{label}: caret observation came from an older frame"
        );
        assert_eq!(
            observation.document_instance,
            tab.document.instance_id(),
            "{label}: caret observation belongs to another document"
        );
        assert_eq!(
            observation.document_version,
            tab.document.version(),
            "{label}: caret observation belongs to another version"
        );
        assert_eq!(
            observation.source_cursor,
            tab.cursor_offset(),
            "{label}: painted caret lags canonical selection"
        );
        assert_eq!(observation.source_selection, tab.selected_range);
        assert_eq!(
            tab.visual_list_blocks[observation.block_index].id, observation.block_id,
            "{label}: caret observation belongs to a stale visual block"
        );
        assert!(observation.bounds.size.height > Pixels::ZERO);
        if tab.selected_range.is_empty() {
            assert!(
                observation.caret_emitted,
                "{label}: caret quad was not emitted"
            );
        }
    });
}

#[gpui::test]
fn visual_enter_paints_the_new_tail_caret_after_one_action(cx: &mut TestAppContext) {
    let source = "Body";
    let (app, cx) = visual_test_app(cx, source, source.len());

    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "Body\n");
        assert_eq!(app.active_tab().cursor_offset(), 5);
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "single Enter"));
}

#[gpui::test]
fn visual_enter_paints_the_new_mid_paragraph_caret_after_one_action(cx: &mut TestAppContext) {
    let source = "AB";
    let (app, cx) = visual_test_app(cx, source, 1);

    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "A\nB");
        assert_eq!(app.active_tab().cursor_offset(), 2);
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "mid-paragraph Enter"));
}

#[gpui::test]
fn visual_cached_cross_block_down_paints_after_one_action(cx: &mut TestAppContext) {
    let source = "# A\nB";
    let cursor = source.find('A').unwrap() + 1;
    let (app, cx) = visual_test_app(cx, source, cursor);

    app.update(cx, |app, _| {
        assert!(
            app.active_tab()
                .visual_navigation_snapshots
                .contains_key(&1)
        );
    });
    cx.dispatch_action(Down);
    cx.run_until_parked();

    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "cached cross-block Down"));
}

#[gpui::test]
fn visual_cached_cross_block_up_paints_after_one_action(cx: &mut TestAppContext) {
    let source = "# A\nB";
    let (app, cx) = visual_test_app(cx, source, source.len());

    cx.dispatch_action(Up);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let owner = visual_block_index_for_offset(
            &tab.visual_list_blocks,
            tab.cursor_offset(),
            tab.document.text().len(),
        )
        .expect("Up target owner");
        assert_eq!(owner, 0);
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "cached cross-block Up"));
}

#[gpui::test]
fn visual_cached_cross_block_select_down_paints_after_one_action(cx: &mut TestAppContext) {
    let source = "# A\nB";
    let cursor = source.find('A').unwrap() + 1;
    let (app, cx) = visual_test_app(cx, source, cursor);

    cx.dispatch_action(SelectDown);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert!(!app.active_tab().selected_range.is_empty());
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "cached Select Down"));
}

#[gpui::test]
fn visual_cached_cross_block_select_up_paints_after_one_action(cx: &mut TestAppContext) {
    let source = "# A\nB";
    let (app, cx) = visual_test_app(cx, source, source.len());

    cx.dispatch_action(SelectUp);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert!(!app.active_tab().selected_range.is_empty());
        assert!(app.active_tab().selection_reversed);
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "cached Select Up"));
}

#[gpui::test]
fn visual_same_block_down_paints_after_one_action(cx: &mut TestAppContext) {
    let source = "wrapped words ".repeat(100);
    let (app, cx) = visual_test_app(cx, &source, 1);

    cx.dispatch_action(Down);
    cx.run_until_parked();

    app.update(cx, |app, _| assert!(app.active_tab().cursor_offset() > 1));
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "same-block Down"));
}

#[gpui::test]
fn visual_blank_line_is_one_immediately_painted_navigation_stop(cx: &mut TestAppContext) {
    let source = "A\n\nB";
    let (app, cx) = visual_test_app(cx, source, 1);
    let (version, blocks, preview, text_handle, dirty, undo_len, redo_len, highlight) =
        app.update(cx, |app, _| {
            let highlight = app.highlighted_code(Some("rust"), "let repaint_probe = true;");
            let tab = app.active_tab_mut();
            (
                tab.document.version(),
                tab.document.visual_blocks_shared(),
                tab.document.preview_blocks_shared(),
                tab.shared_document_text(),
                tab.document.is_dirty(),
                tab.undo_stack.len(),
                tab.redo_stack.len(),
                highlight,
            )
        });

    cx.dispatch_action(Down);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let owner =
            visual_block_index_for_offset(&blocks, tab.cursor_offset(), tab.document.text().len())
                .expect("blank-line owner");
        assert!(matches!(blocks[owner].kind, VisualBlockKind::Whitespace));
        assert_eq!(tab.document.text(), source);
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.is_dirty(), dirty);
        assert_eq!(tab.undo_stack.len(), undo_len);
        assert_eq!(tab.redo_stack.len(), redo_len);
        assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
        assert!(Arc::ptr_eq(&preview, &tab.document.preview_blocks_shared()));
        assert_eq!(tab.shared_document_text().as_ptr(), text_handle.as_ptr());
        assert!(tab.visual_preferred_x.is_some());
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "blank-line stop"));

    cx.dispatch_action(Down);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let highlight_after = app.highlighted_code(Some("rust"), "let repaint_probe = true;");
        assert!(Rc::ptr_eq(&highlight, &highlight_after));
        assert!(app.active_tab().cursor_offset() >= source.find('B').unwrap());
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().document.version(), version);
        assert_eq!(app.active_tab().document.is_dirty(), dirty);
        assert_eq!(app.active_tab().undo_stack.len(), undo_len);
        assert_eq!(app.active_tab().redo_stack.len(), redo_len);
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
        assert!(Arc::ptr_eq(
            &preview,
            &app.active_tab().document.preview_blocks_shared()
        ));
        assert_eq!(
            app.active_tab().shared_document_text().as_ptr(),
            text_handle.as_ptr()
        );
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "after blank-line stop"));
}

#[gpui::test]
fn visual_virtualized_navigation_paints_without_a_follow_up_input(cx: &mut TestAppContext) {
    let first = (0..260).map(|_| "wide ").collect::<String>();
    let source = format!("{first}\n\nsecond block");
    let cursor = first.len() - 1;
    let second_start = source.find("second block").unwrap();
    let (app, cx) = visual_test_app(cx, &source, cursor);

    cx.dispatch_action(Down);
    cx.run_until_parked();
    cx.dispatch_action(Down);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert!(app.active_tab().cursor_offset() >= second_start);
        assert!(app.active_tab().pending_visual_navigation.is_none());
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "virtualized Down"));

    let idle_frame = app.update(cx, |app, _| {
        (
            app.active_tab().visual_frame_generation,
            app.active_tab().cursor_offset(),
        )
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(
            (
                app.active_tab().visual_frame_generation,
                app.active_tab().cursor_offset(),
            ),
            idle_frame,
            "an otherwise idle window must not need incidental invalidation"
        );
    });
    cx.update(|_, cx| assert_current_visual_caret(&app, cx, "idle virtualized Down"));
}

#[gpui::test]
fn stale_deferred_navigation_cannot_override_newer_pointer_or_document_state(
    cx: &mut TestAppContext,
) {
    let source = "# A\nB";
    let (app, cx) = visual_test_app(cx, source, source.find('A').unwrap());

    app.update(cx, |app, cx| {
        let expected = app
            .visual_navigation_request(1, VisualNavigationDirection::Down, false, px(12.))
            .expect("valid deferred request");
        let newer = PendingVisualNavigation {
            extend_selection: true,
            preferred_x: px(48.),
            ..expected
        };
        let initial_cursor = app.cursor_offset();
        let tab = app.active_tab_mut();
        tab.pending_visual_navigation = Some(newer);
        tab.visual_navigation_completion_queued = Some(expected);

        app.complete_deferred_visual_navigation(expected, cx);

        assert_eq!(app.cursor_offset(), initial_cursor);
        assert_eq!(app.active_tab().pending_visual_navigation, Some(newer));
        assert!(
            app.active_tab()
                .visual_navigation_completion_queued
                .is_none()
        );

        let tab = app.active_tab_mut();
        tab.pending_visual_navigation = Some(expected);
        tab.visual_navigation_completion_queued = Some(expected);
        app.move_to(0, cx);
        app.complete_deferred_visual_navigation(expected, cx);
        assert_eq!(app.cursor_offset(), 0);
        assert!(app.active_tab().pending_visual_navigation.is_none());
        assert!(
            app.active_tab()
                .visual_navigation_completion_queued
                .is_none()
        );

        let expected = app
            .visual_navigation_request(1, VisualNavigationDirection::Down, false, px(12.))
            .expect("valid request before document change");
        let tab = app.active_tab_mut();
        tab.pending_visual_navigation = Some(expected);
        tab.visual_navigation_completion_queued = Some(expected);
        tab.document = MarkdownDocument::from_text("changed # A\nB");
        app.complete_deferred_visual_navigation(expected, cx);
        assert!(app.active_tab().pending_visual_navigation.is_none());
        assert!(
            app.active_tab()
                .visual_navigation_completion_queued
                .is_none()
        );
        assert_ne!(
            app.active_tab().document.instance_id(),
            expected.document_instance
        );
    });
}

#[gpui::test]
fn visual_caret_follow_frames_expire_without_an_idle_redraw_loop(cx: &mut TestAppContext) {
    let source = (0..60)
        .map(|index| format!("paragraph {index}"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let target = source.find("paragraph 30").unwrap();
    let (app, cx) = visual_test_app(cx, &source, target);

    app.update(cx, |app, cx| {
        app.active_tab_mut().visual_caret_follow_frames = 2;
        cx.notify();
    });
    cx.run_until_parked();
    // GPUI's headless test platform does not synthesize a compositor tick for
    // `request_animation_frame`, so explicitly drive that requested frame.
    cx.update(|window, cx| {
        window.refresh();
        let _ = window.draw(cx);
    });
    let settled_frame = app.update(cx, |app, _| {
        assert_eq!(app.active_tab().visual_caret_follow_frames, 0);
        app.active_tab().visual_frame_generation
    });

    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().visual_frame_generation, settled_frame);
        app.active_tab()
            .visual_list
            .scroll_to(gpui::ListOffset::default());
    });
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let top = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(top.item_ix, 0);
        assert_eq!(
            top.offset_in_item,
            px(0.),
            "expired caret following must not undo a manual scroll"
        );
    });

    app.update(cx, |app, cx| {
        app.typewriter_mode = true;
        app.move_to(target + "paragraph ".len(), cx);
    });
    cx.run_until_parked();
    cx.update(|window, cx| {
        for _ in 0..TYPEWRITER_VISUAL_REFINEMENT_FRAMES + 1 {
            window.refresh();
            let _ = window.draw(cx);
        }
    });
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().visual_caret_follow_frames, 0);
        assert!(app.active_tab().typewriter_recenter.is_none());
    });
}
