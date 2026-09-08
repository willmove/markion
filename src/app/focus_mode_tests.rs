use super::*;
use gpui::TestAppContext;

fn assert_focus_rows(app: &MarkionApp, current: usize, cx: &mut Context<MarkionApp>) {
    let blocks = app.active_tab().document.visual_blocks_shared();
    for (index, block) in blocks.iter().enumerate() {
        let mut row = preview::visual_block_view(app, block, index, None, 1., cx);
        let opacity = row.style().opacity.unwrap_or(1.);
        if !app.focus_mode || index == current {
            assert_eq!(opacity, 1., "row {index} must remain fully visible");
        } else {
            assert!(
                opacity > 0. && opacity <= 0.5,
                "row {index} must be visibly dimmed: {opacity}"
            );
        }
    }
}

#[gpui::test]
fn visual_focus_mode_dims_non_current_rendered_paragraph(cx: &mut TestAppContext) {
    let app = cx.new(|cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "first\n\nsecond",
        ))];
        app.view_mode = ViewMode::VisualEdit;
        app.focus_mode = true;
        let blocks = app.active_tab().document.visual_blocks_shared();
        app.active_tab_mut().sync_visual_list(&blocks);
        app
    });
    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|block| block.source_range.start == 7)
            .unwrap();
        let mut row = preview::visual_block_view(app, &blocks[index], index, None, 1., cx);
        assert!(
            row.style().opacity.unwrap_or(1.) < 1.,
            "focus mode must dim the non-current rendered paragraph"
        );
    });
}

#[gpui::test]
fn visual_focus_mode_covers_structured_rows_and_source_island_returns(cx: &mut TestAppContext) {
    let app = cx.new(MarkionApp::new);
    for source in [
        "# 标题\n\n第一段 **加粗**\n\n第二段 [链接](relative.md)",
        "> first quote\n>\n> second quote\n\nafter",
        "- first item\n- second item\n\nafter",
        "```rust\nlet x = 1;\n```\n\nafter",
        "---\ntitle: sample\n---\n\nafter",
        "before\n\n```unclosed\ncode",
        "| A | B |\n| --- | --- |\n| a | b |\n\nafter",
        "<div>HTML</div>\n\nafter",
        "$$\nx^2\n$$\n\nafter",
        "![image](missing-focus-test.png)\n\nafter",
        "before\n\n",
        "",
    ] {
        app.update(cx, |app, cx| {
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
            app.active_tab = 0;
            app.view_mode = ViewMode::VisualEdit;
            let blocks = app.active_tab().document.visual_blocks_shared();
            app.active_tab_mut().sync_visual_list(&blocks);
            for theme in [AppTheme::Paper, AppTheme::Ink] {
                app.theme = theme;
                for enabled in [false, true] {
                    app.focus_mode = enabled;
                    for block in blocks.iter() {
                        let offset = block.source_range.start;
                        app.active_tab_mut().selected_range = offset..offset;
                        // Math's closing boundary can still belong to the
                        // preceding formula. Dimming must agree with the
                        // existing caret painter across all rendering branches.
                        let current =
                            preview::visual_block_index_for_offset(&blocks, offset, source.len())
                                .expect("each source-backed row start has a caret owner");
                        assert_focus_rows(app, current, cx);
                    }
                }
            }
        });
    }
}

#[gpui::test]
fn visual_focus_mode_follows_selection_tabs_and_views_without_mutation(cx: &mut TestAppContext) {
    let app = cx.new(|cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("first\n\nsecond")),
            EditorTab::new(MarkdownDocument::from_text("other\n\nlast")),
        ];
        app.focus_mode = true;
        app
    });
    app.update(cx, |app, cx| {
        for tab_index in [0, 1, 0] {
            app.active_tab = tab_index;
            let tab = app.active_tab_mut();
            let blocks = tab.document.visual_blocks_shared();
            let previews = tab.document.preview_blocks_shared();
            let text = tab.shared_document_text();
            let version = tab.document.version();
            let undo_len = tab.undo_stack.len();
            let redo_len = tab.redo_stack.len();
            let len = tab.document.text().len();
            tab.sync_visual_list(&blocks);
            let last = blocks.len() - 1;
            for view in [
                ViewMode::Edit,
                ViewMode::Read,
                ViewMode::Split,
                ViewMode::VisualEdit,
            ] {
                app.set_view_mode(view, cx);
            }
            // Forward selection puts the caret at EOF; reversed selection
            // puts it at the beginning, rather than lighting every selected row.
            app.active_tab_mut().selected_range = 0..len;
            app.active_tab_mut().selection_reversed = false;
            assert_focus_rows(app, last, cx);
            app.active_tab_mut().selection_reversed = true;
            assert_focus_rows(app, 0, cx);
            app.focus_mode = false;
            assert_focus_rows(app, 0, cx);
            app.focus_mode = true;
            assert_focus_rows(app, 0, cx);
            let tab = app.active_tab_mut();
            assert_eq!(tab.document.version(), version);
            assert!(!tab.document.is_dirty());
            assert_eq!(tab.undo_stack.len(), undo_len);
            assert_eq!(tab.redo_stack.len(), redo_len);
            assert_eq!(tab.shared_document_text().as_ptr(), text.as_ptr());
            assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
            assert!(Arc::ptr_eq(
                &previews,
                &tab.document.preview_blocks_shared()
            ));
        }
    });
}

#[gpui::test]
fn visual_focus_mode_follows_pointer_and_keyboard_in_window(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "first paragraph\n\nsecond paragraph\n\nthird paragraph",
        ))];
        app.view_mode = ViewMode::VisualEdit;
        app.focus_mode = true;
        app.typewriter_mode = false;
        app
    });
    cx.simulate_resize(size(px(640.), px(480.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let second = cx
        .debug_bounds("visual-document-row-2")
        .expect("second paragraph painted");
    cx.simulate_click(second.center(), gpui::Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, cx| assert_focus_rows(app, 2, cx));
    // Move across the paragraph separator with the real keyboard action.
    app.update(cx, |app, cx| app.move_to(17, cx));
    cx.dispatch_action(Left);
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        assert_eq!(app.active_tab().cursor_offset(), 16);
        assert_focus_rows(app, 1, cx);
        assert!(!app.active_tab().document.is_dirty());
        assert!(app.active_tab().undo_stack.is_empty());
    });
}
