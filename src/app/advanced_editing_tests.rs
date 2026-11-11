use super::*;
use gpui::TestAppContext;
use markion::selection::{CellSelectionAxis, SelectionState};

#[gpui::test]
fn visual_table_rectangle_clear_and_undo_preserve_visible_viewport(cx: &mut TestAppContext) {
    let source = format!(
        "{}| Name | Kind | Value |\n| --- | --- | --- |\n| first | bold | 10 |\n| second | emoji 😀 | 20 |\n| last | Tab | tail |\n\nending paragraph\n",
        "leading paragraph\n\n".repeat(5)
    );
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.simulate_resize(size(px(900.), px(720.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        app.active_tab().visual_list.scroll_to(gpui::ListOffset {
            item_ix: 2,
            offset_in_item: px(0.),
        });
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let table = blocks
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
            .unwrap();
        let list = &app.active_tab().visual_list;
        let bounds = list.bounds_for_item(table).expect("table is painted");
        assert_eq!(
            visual_caret_pixel_delta(
                list.viewport_bounds(),
                bounds,
                px(app.typography_metrics().preview_row_line_height),
            ),
            None,
            "test table must be fully inside the viewport inset"
        );
        app.select_table_cells(table, (1, 0), CellSelectionAxis::Rectangle, false, cx);
        app.select_table_cells(table, (2, 1), CellSelectionAxis::Rectangle, true, cx);
    });
    cx.run_until_parked();
    let anchor = app.update(cx, |app, cx| {
        let anchor = app.active_tab().visual_list.logical_scroll_top();
        assert!(app.delete_source_selection(cx));
        anchor
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let top = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(
            (top.item_ix, top.offset_in_item),
            (anchor.item_ix, anchor.offset_in_item)
        );
        assert!(!app.active_tab().document.text().contains("emoji"));
    });
    cx.dispatch_action(Undo);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let top = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(
            (top.item_ix, top.offset_in_item),
            (anchor.item_ix, anchor.offset_in_item)
        );
        assert_eq!(app.active_tab().document.text(), source);
        assert!(matches!(
            app.active_tab().structured_selection,
            Some(SelectionState::Cells(_))
        ));
    });
}

fn assert_native_ime_pause_is_one_undo(cx: &mut TestAppContext, pause_before_update: bool) {
    let source = "中 **bold** 😀 e\u{301} suffix";
    let cursor = source.find("bold").unwrap() + "bold".len();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(app, None, "ni", None, window, cx);
            app.active_tab_mut()
                .undo_capture
                .as_mut()
                .unwrap()
                .last_edit_at -= SEMANTIC_UNDO_TIMEOUT + Duration::from_secs(1);
            if pause_before_update {
                EntityInputHandler::replace_and_mark_text_in_range(
                    app, None, "ni hao", None, window, cx,
                );
            }
            EntityInputHandler::replace_text_in_range(app, None, "你好", window, cx);
            assert_eq!(
                app.active_tab().undo_stack.len(),
                1,
                "candidate-selection pauses must not snapshot transient Pinyin"
            );
            assert!(app.active_tab().marked_range.is_none());
            assert!(app.active_tab().undo_capture.is_none());
        });
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().selected_range, cursor..cursor);
    });
    cx.dispatch_action(Redo);
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().document.text(),
            format!("{}你好{}", &source[..cursor], &source[cursor..])
        );
    });
}

#[gpui::test]
fn native_ime_pause_before_mark_update_keeps_one_undo(cx: &mut TestAppContext) {
    assert_native_ime_pause_is_one_undo(cx, true);
}

#[gpui::test]
fn native_ime_pause_before_commit_keeps_one_undo(cx: &mut TestAppContext) {
    assert_native_ime_pause_is_one_undo(cx, false);
}

fn insert_checked_for_test(app: &mut MarkionApp, text: &str) {
    let mutation = app.active_tab().document.prepare_range_mutation(
        MutationOrigin::PlatformTextInput,
        0..0,
        text,
    );
    app.apply_document_mutation("advanced_test_input", mutation)
        .unwrap();
}

fn table_plain_rows(app: &MarkionApp) -> Vec<Vec<String>> {
    app.active_tab()
        .document
        .preview_blocks()
        .into_iter()
        .find_map(|block| match block {
            PreviewBlock::Table { rows, .. } => Some(
                rows.iter()
                    .map(|row| row.iter().map(|cell| cell.text.clone()).collect())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap()
}

#[gpui::test]
fn advanced_block_selection_copy_delete_move_and_stale_commands(cx: &mut TestAppContext) {
    let source = "one **bold**\n\ntwo\n\nthree";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let version = app.active_tab().document.version();
        app.select_source_blocks(0, false, cx);
        assert!(app.navigate_source_selection(true, true, false, cx));
        assert_eq!(
            &app.active_tab().document.text()[app.active_tab().selected_range.clone()],
            "one **bold**\n\ntwo\n\n"
        );
        assert_eq!(app.active_tab().document.version(), version);
        assert!(std::sync::Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
        app.copy_source_selection(cx);
        let clipboard = cx.read_from_clipboard().unwrap();
        assert!(clipboard.text().unwrap().contains("one bold"));
        assert!(clipboard.html().unwrap().contains("<strong>bold</strong>"));
    });
    cx.dispatch_action(Delete);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "three");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source);
        assert!(matches!(
            app.active_tab().structured_selection,
            Some(SelectionState::Blocks(_))
        ));
    });
    app.update(cx, |app, cx| {
        app.select_source_blocks(0, false, cx);
        app.navigate_source_selection(true, true, false, cx);
    });
    cx.dispatch_action(MoveBlocksDown);
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().document.text(),
            "three\n\none **bold**\n\ntwo\n\n"
        );
        assert!(matches!(
            app.active_tab().structured_selection,
            Some(SelectionState::Blocks(_))
        ));
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, cx| {
        app.select_source_blocks(0, false, cx);
        insert_checked_for_test(app, "new ");
        let text = app.active_tab().document.text().to_string();
        let history = app.active_tab().undo_stack.len();
        assert!(app.delete_source_selection(cx));
        assert_eq!(app.active_tab().document.text(), text);
        assert_eq!(app.active_tab().undo_stack.len(), history);
    });
}

#[gpui::test]
fn advanced_table_rectangle_matrix_paste_and_final_tab_are_atomic(cx: &mut TestAppContext) {
    let source = "before\n\n| A | B |\n| :--- | ---: |\n| a | b |\n| c | d |\n\nafter";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|b| matches!(b.kind, VisualBlockKind::Table { .. }))
            .unwrap();
        app.select_table_cells(index, (1, 0), CellSelectionAxis::Rectangle, false, cx);
        app.select_table_cells(index, (2, 1), CellSelectionAxis::Rectangle, true, cx);
        app.copy_source_selection(cx);
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().unwrap(),
            "a\tb\nc\td"
        );
    });
    cx.dispatch_action(Delete);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().undo_stack.len(), 1)
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source)
    });
    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|b| matches!(b.kind, VisualBlockKind::Table { .. }))
            .unwrap();
        app.select_table_cells(index, (1, 0), CellSelectionAxis::Rectangle, false, cx);
        assert!(app.paste_table_matrix("p\tq\nr\ts", cx));
        assert_eq!(table_plain_rows(app)[1], vec!["p", "q"]);
        let blocks = app.active_tab().document.visual_blocks_shared();
        let field = blocks
            .iter()
            .find_map(|b| match b.editor.as_ref() {
                Some(VisualBlockEditor::Table { cells }) => cells.last().map(|c| c.field.clone()),
                _ => None,
            })
            .unwrap();
        app.move_to(field.source_range.end, cx);
    });
    let version = app.read_with(cx, |app, _| app.active_tab().document.version());
    let history = app.read_with(cx, |app, _| app.active_tab().undo_stack.len());
    cx.dispatch_action(Indent);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.version(), version + 1);
        assert_eq!(app.active_tab().undo_stack.len(), history + 1);
        let field = app
            .active_tab()
            .document
            .visual_editor_field_at(&app.active_tab().selected_range)
            .unwrap();
        assert_eq!(
            field.kind,
            VisualEditorFieldKind::TableCell { row: 3, column: 0 }
        );
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(table_plain_rows(app)[2], vec!["r", "s"])
    });
}

#[gpui::test]
fn advanced_code_fold_is_presentation_only_and_navigation_reveals_payload(cx: &mut TestAppContext) {
    let source = "```rust\nfn first() {}\nfn second() {}\nfn third() {}\n```\n\nafter";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let height = cx
        .debug_bounds("visual-document-row-0")
        .unwrap()
        .size
        .height;
    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let version = app.active_tab().document.version();
        app.toggle_visual_code_fold(BlockTarget::from_block(version, &blocks[0]), cx);
        assert_eq!(app.active_tab().document.version(), version);
        assert!(app.active_tab().undo_stack.is_empty());
        assert!(std::sync::Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("visual-document-row-0")
            .unwrap()
            .size
            .height
            < height
    );
    app.update(cx, |app, cx| {
        app.move_to(source.find("second").unwrap(), cx);
        assert!(app.active_tab().folded_visual_code_blocks.is_empty());
        assert_eq!(app.active_tab().document.text(), source);
    });
}

#[gpui::test]
fn advanced_image_presentation_and_subtree_tab_keep_one_undo_step(cx: &mut TestAppContext) {
    let source = "![说明](image.png)\n\n- first\n- parent\n  - child";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    app.update(cx, |app, cx| {
        app.set_image_presentation_at(
            2,
            ImagePresentation {
                width_percent: 50,
                alignment: ImageAlignment::Center,
            },
            cx,
        );
        let image = inline_image_at(app.active_tab().document.text(), 2).unwrap();
        assert_eq!(image.label, "说明");
        assert_eq!(image.presentation.unwrap().width_percent, 50);
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, cx| {
        app.move_to(source.find("parent").unwrap(), cx)
    });
    cx.dispatch_action(Indent);
    app.update(cx, |app, _| {
        assert!(
            app.active_tab()
                .document
                .text()
                .ends_with("- first\n    - parent\n      - child")
        );
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source)
    });
}

#[gpui::test]
fn advanced_large_document_search_is_deferred_and_rejects_replaced_document(
    cx: &mut TestAppContext,
) {
    let app = cx.new(|cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(format!(
            "needle\n\n{}",
            "body\n\n".repeat(12_000)
        )))];
        app.view_mode = ViewMode::VisualEdit;
        app.search_visible = true;
        app.search_query.buffer = "needle".into();
        app.refresh_search_matches();
        assert_eq!(app.search_matches.len(), 1);
        app
    });
    app.update(cx, |app, cx| {
        insert_checked_for_test(app, "new ");
        app.after_document_changed(cx);
        assert!(app.search_matches.is_empty());
        assert!(app.search_generation.is_none());
    });
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_millis(121));
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        assert_eq!(app.search_matches.len(), 1);
        assert_eq!(
            app.search_generation.as_ref().unwrap().document_version,
            app.active_tab().document.version()
        );
        insert_checked_for_test(app, "next ");
        app.after_document_changed(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "different document",
        ))];
        app.refresh_search_matches();
    });
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_millis(121));
    cx.run_until_parked();
    app.update(cx, |app, _| assert!(app.search_matches.is_empty()));
}

#[gpui::test]
fn advanced_dormant_tabs_restore_selection_and_folding_without_retaining_visual_caches(
    cx: &mut TestAppContext,
) {
    let app = cx.new(|cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "```rust\nlet x = 1;\n```\n\ntext",
        ))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        app.active_tab_mut().sync_visual_list(&blocks);
        let old_id = blocks[0].id;
        app.select_source_blocks(0, false, cx);
        app.toggle_visual_code_fold(
            BlockTarget::from_block(app.active_tab().document.version(), &blocks[0]),
            cx,
        );
        app.active_tab_mut().enter_dormant();
        assert!(app.active_tab().structured_selection.is_none());
        assert!(app.active_tab().folded_visual_code_blocks.is_empty());
        assert!(
            app.active_tab()
                .document
                .memory_breakdown()
                .sites
                .iter()
                .all(|site| !site.populated)
        );
        let blocks = app.active_tab().document.visual_blocks_shared();
        assert_ne!(old_id, blocks[0].id);
        app.active_tab_mut().sync_visual_list(&blocks);
        assert!(matches!(
            app.active_tab().structured_selection,
            Some(SelectionState::Blocks(_))
        ));
        assert!(
            app.active_tab()
                .folded_visual_code_blocks
                .contains(&blocks[0].id)
        );
        app.active_tab_mut().enter_dormant();
        insert_checked_for_test(app, "changed\n\n");
        let blocks = app.active_tab().document.visual_blocks_shared();
        app.active_tab_mut().sync_visual_list(&blocks);
        assert!(app.active_tab().structured_selection.is_none());
        assert!(app.active_tab().folded_visual_code_blocks.is_empty());
    });
}
