use super::*;
use gpui::{Modifiers, TestAppContext};

fn test_debug_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

#[gpui::test]
fn block_grip_click_menu_transform_and_exclusive_boundary(cx: &mut TestAppContext) {
    const SOURCE: &str = "Paragraph **bold**\n\n### Next heading\n";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
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
    let grip = cx.debug_bounds("visual-block-drag-grip-0").unwrap();
    cx.simulate_mouse_down(grip.center(), MouseButton::Left, Modifiers::none());
    app.update(cx, |app, _| {
        assert!(
            app.block_menu.is_none(),
            "mouse-down must leave dragging available"
        )
    });
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: grip.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let menu = app
            .block_menu
            .as_ref()
            .expect("completed grip click opens quick settings");
        assert!(menu.from_grip);
        assert_eq!(tab.selected_range, 0..SOURCE.find("###").unwrap());
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
        assert!(tab.undo_stack.is_empty());
        let next = blocks
            .iter()
            .find(|b| matches!(b.kind, VisualBlockKind::Heading { .. }))
            .unwrap();
        let projection = build_visual_projection(
            SOURCE,
            next,
            tab.selected_range.clone(),
            tab.cursor_offset(),
        );
        assert_eq!(projection.text, "Next heading");
    });
    let panel = cx.debug_bounds("visual-block-menu-panel").unwrap();
    for index in (0..12).filter(|index| *index != 6) {
        let id = test_debug_selector(format!("visual-block-transform-{index}"));
        let bounds = cx
            .debug_bounds(id)
            .expect("all direct transforms are rendered");
        assert!(
            panel.contains(&bounds.center()),
            "{id} must be inside the popup"
        );
        assert_eq!(bounds.size, size(px(36.), px(30.)));
    }
    assert!(cx.debug_bounds("visual-block-transform-6").is_none());
    assert!(cx.debug_bounds("visual-block-transform-12").is_none());
    assert!(cx.debug_bounds("visual-block-transform-13").is_none());
    for id in [
        "visual-block-add-divider-below",
        "visual-block-add-table-below",
    ] {
        let bounds = cx.debug_bounds(id).unwrap();
        assert_eq!(bounds.size, size(px(36.), px(30.)));
        assert!(panel.contains(&bounds.center()));
        assert!(bounds.top() >= cx.debug_bounds("visual-block-add-below").unwrap().top());
    }
    let heading = cx.debug_bounds("visual-block-transform-2").unwrap();
    cx.simulate_click(heading.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), format!("## {SOURCE}"));
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        assert!(app.block_menu.is_none());
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), SOURCE)
    });
}

#[gpui::test]
fn block_grip_preference_shift_drag_and_stale_gestures(cx: &mut TestAppContext) {
    const SOURCE: &str = "one\n\ntwo\n\n### Next";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let grip = cx.debug_bounds("visual-block-drag-grip-0").unwrap();
    cx.simulate_click(grip.center(), Modifiers::none());
    cx.run_until_parked();
    let selection = app.update(cx, |app, cx| {
        assert!(app.block_menu.as_ref().unwrap().from_grip);
        let range = app.active_tab().selected_range.clone();
        app.toggle_block_handle_menu(cx);
        assert!(app.block_menu.is_none());
        assert_eq!(app.active_tab().selected_range, range);
        assert!(!app.current_preferences().block_handle_menu);
        range
    });
    cx.simulate_click(grip.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        assert!(app.block_menu.is_none());
        assert_eq!(app.active_tab().selected_range, selection);
        app.toggle_block_handle_menu(cx);
    });
    let blocks = app.update(cx, |app, _| {
        app.active_tab().document.visual_blocks_shared()
    });
    let second = blocks
        .iter()
        .position(|b| b.source_range.start == 5)
        .unwrap();
    let second_grip = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-block-drag-grip-{second}"
        )))
        .unwrap();
    cx.simulate_click(
        second_grip.center(),
        Modifiers {
            shift: true,
            ..Modifiers::none()
        },
    );
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.block_menu.is_none());
        assert_eq!(
            app.active_tab().selected_range,
            0..SOURCE.find("###").unwrap()
        );
    });
    cx.simulate_mouse_down(grip.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_event(MouseMoveEvent {
        position: grip.center() + point(px(50.), px(5.)),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        assert!(cx.has_active_drag());
        assert!(app.block_menu.is_none());
        assert_eq!(
            app.active_tab().selected_range,
            0..SOURCE.find("###").unwrap()
        );
        assert_eq!(app.active_tab().document.text(), SOURCE);
    });
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: grip.center() + point(px(50.), px(5.)),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        assert!(
            app.block_menu.is_none(),
            "drag completion cannot become a popup click"
        );
        let stale = BlockTarget::from_block(
            app.active_tab().document.version().wrapping_add(1),
            &blocks[0],
        );
        let selection = app.active_tab().selected_range.clone();
        app.open_visual_block_grip_menu(stale, point(px(20.), px(20.)), cx);
        assert!(app.block_menu.is_none());
        assert_eq!(app.active_tab().selected_range, selection);
    });
}

#[gpui::test]
fn block_grip_copy_cut_add_below_and_undo_preserve_next_heading(cx: &mut TestAppContext) {
    const SOURCE: &str = "段落 **bold**😀\r\n\r\n### 下一标题\r\n";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let target = app.update(cx, |app, _| {
        let doc = &app.active_tab().document;
        BlockTarget::from_block(doc.version(), &doc.visual_blocks_shared()[0])
    });
    app.update(cx, |app, cx| {
        app.open_visual_block_grip_menu(target.clone(), point(px(20.), px(20.)), cx);
        app.activate_visual_block_menu_item(BlockMenuItem::CopyBlock, None, cx);
        assert_eq!(app.active_tab().document.text(), SOURCE);
        assert!(app.active_tab().undo_stack.is_empty());
        let copy = cx.read_from_clipboard().unwrap();
        assert!(copy.text().unwrap().contains("段落 bold😀"));
        assert!(!copy.text().unwrap().contains("下一标题"));
        assert!(copy.html().unwrap().contains("<strong>bold</strong>"));
        app.open_visual_block_grip_menu(target.clone(), point(px(20.), px(20.)), cx);
        app.activate_visual_block_menu_item(BlockMenuItem::CutBlock, None, cx);
        assert_eq!(app.active_tab().document.text(), "### 下一标题\r\n");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, cx| {
        assert_eq!(app.active_tab().document.text(), SOURCE);
        let doc = &app.active_tab().document;
        let target = BlockTarget::from_block(doc.version(), &doc.visual_blocks_shared()[0]);
        app.open_visual_block_grip_menu(target, point(px(20.), px(20.)), cx);
        app.activate_visual_block_menu_item(BlockMenuItem::AddBelow, None, cx);
        assert_eq!(
            app.active_tab().document.text(),
            SOURCE.replacen("\r\n\r\n###", "\r\n\r\n\r\n###", 1)
        );
        let tab = app.active_tab();
        assert_eq!(tab.selected_range, target_end(SOURCE)..target_end(SOURCE));
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), SOURCE);
        assert!(matches!(
            app.active_tab().structured_selection,
            Some(markion::selection::SelectionState::Blocks(_))
        ));
    });
}

fn target_end(source: &str) -> usize {
    source.find("###").unwrap()
}

#[gpui::test]
fn block_grip_template_insertions_preserve_table_bytes_selection_and_history(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for ending in ["\n", "\r\n"] {
        for terminal in [false, true] {
            for action in [BlockMenuItem::AddDividerBelow, BlockMenuItem::AddTableBelow] {
                app.update_in(cx, |app, window, cx| {
                    let table = "<!-- markion-cols:30,70 -->\n| 名称 | 内容 |\n| :--- | ---: |\n| 中文🙂 | **keep** |".replace('\n', ending);
                    let source = format!("before{ending}{ending}{table}{}", if terminal { String::new() } else { format!("{ending}{ending}after") });
                    app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                    app.view_mode = ViewMode::VisualEdit;
                    let doc = &app.active_tab().document;
                    let blocks = doc.visual_blocks_shared();
                    let target = BlockTarget::from_block(doc.version(), blocks.iter().find(|b| matches!(b.kind, VisualBlockKind::Table { .. })).unwrap());
                    let version = doc.version();
                    let dirty = doc.is_dirty();
                    app.open_visual_block_grip_menu(target, point(px(30.), px(40.)), cx);
                    let selected = app.active_tab().selected_range.clone();
                    let structured = app.active_tab().structured_selection.clone();
                    let menu = app.block_menu.as_ref().unwrap();
                    let root = menu.root_items();
                    let below = root.iter().position(|i| *i == BlockMenuItem::AddBelow).unwrap();
                    assert_eq!(&root[below + 1..below + 3], &[BlockMenuItem::AddDividerBelow, BlockMenuItem::AddTableBelow]);
                    for removed in [BlockTransform::Divider, BlockTransform::Table] {
                        assert!(!root.contains(&BlockMenuItem::Transform(removed)));
                        app.activate_visual_block_menu_item(BlockMenuItem::Transform(removed), Some(window), cx);
                    }
                    assert_eq!(app.active_tab().document.text(), source);
                    assert_eq!(app.active_tab().document.version(), version);
                    assert_eq!(app.active_tab().document.is_dirty(), dirty);
                    assert_eq!(app.active_tab().selected_range, selected);
                    assert_eq!(app.active_tab().structured_selection, structured);
                    assert!(Arc::ptr_eq(&blocks, &app.active_tab().document.visual_blocks_shared()));
                    app.activate_visual_block_menu_item(action, Some(window), cx);
                    assert!(app.block_menu.is_none());
                    let tab = app.active_tab();
                    let inserted = tab.document.text().to_string();
                    let caret = tab.selected_range.clone();
                    assert!(inserted.starts_with(&format!("before{ending}{ending}{table}")), "{action:?}: {inserted:?}");
                    assert!(terminal || inserted.ends_with("after"));
                    assert_eq!(inserted.matches("markion-cols:30,70").count(), 1);
                    assert_eq!(tab.document.version(), version + 1);
                    assert_eq!(tab.undo_stack.len(), 1);
                    assert!(caret.is_empty());
                    let expected_tables = if action == BlockMenuItem::AddTableBelow { 2 } else { 1 };
                    assert_eq!(tab.document.visual_blocks_shared().iter().filter(|b| matches!(b.kind, VisualBlockKind::Table { .. })).count(), expected_tables);
                    if action == BlockMenuItem::AddDividerBelow {
                        assert!(inserted[..caret.start].ends_with(&format!("---{ending}{ending}")));
                        app.replace_text_in_range(None, "continued 中文🙂", window, cx);
                        let tab = app.active_tab();
                        for paragraph in ["continued 中文🙂", "after"] {
                            if paragraph == "after" && terminal { continue; }
                            assert!(tab.document.visual_blocks_shared().iter().any(|b| matches!(b.kind, VisualBlockKind::Paragraph) && tab.document.text()[b.source_range.clone()].trim() == paragraph));
                        }
                        app.undo(&Undo, window, cx);
                        assert_eq!(app.active_tab().document.text(), inserted);
                    }
                    app.undo(&Undo, window, cx);
                    assert_eq!(app.active_tab().document.text(), source);
                    assert_eq!(app.active_tab().selected_range, selected);
                    let tab = app.active_tab();
                    let Some(markion::selection::SelectionState::Blocks(restored)) = &tab.structured_selection else { panic!("Undo must restore the whole-table selection"); };
                    assert_eq!(restored.range(&tab.document).unwrap(), selected);
                    app.redo(&Redo, window, cx);
                    assert_eq!(app.active_tab().document.text(), inserted);
                    assert_eq!(app.active_tab().selected_range, caret);
                    app.undo(&Undo, window, cx);
                    let doc = &app.active_tab().document;
                    let target = BlockTarget::from_block(doc.version(), doc.visual_blocks_shared().iter().find(|b| matches!(b.kind, VisualBlockKind::Table { .. })).unwrap());
                    app.open_visual_block_grip_menu(target, point(px(30.), px(40.)), cx);
                    app.block_menu.as_mut().unwrap().target.document_version += 1;
                    let cache = app.active_tab().document.visual_blocks_shared();
                    app.activate_visual_block_menu_item(action, Some(window), cx);
                    assert!(app.block_menu.is_none());
                    assert_eq!(app.active_tab().document.text(), source);
                    assert_eq!(app.active_tab().selected_range, selected);
                    assert!(app.active_tab().undo_stack.is_empty());
                    assert!(Arc::ptr_eq(&cache, &app.active_tab().document.visual_blocks_shared()));
                });
            }
        }
    }
}

#[gpui::test]
fn block_grip_insert_below_buttons_support_pointer_and_small_viewport_keyboard(
    cx: &mut TestAppContext,
) {
    const SOURCE: &str = "before\n\n| A | B |\n| --- | --- |\n| 中文🙂 | keep |\n\nafter";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.view_mode = ViewMode::VisualEdit;
        app.sidebar_visible = false;
        app.outline_visible = false;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for (width, height, theme, keyboard) in [
        (1180., 900., AppTheme::Paper, false),
        (340., 280., AppTheme::Ink, true),
    ] {
        cx.simulate_resize(size(px(width), px(height)));
        for action in [BlockMenuItem::AddDividerBelow, BlockMenuItem::AddTableBelow] {
            app.update(cx, |app, cx| {
                app.theme = theme;
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
                cx.notify();
                if keyboard {
                    let doc = &app.active_tab().document;
                    let target = BlockTarget::from_block(
                        doc.version(),
                        doc.visual_blocks_shared()
                            .iter()
                            .find(|b| matches!(b.kind, VisualBlockKind::Table { .. }))
                            .unwrap(),
                    );
                    app.open_visual_block_grip_menu(
                        target,
                        point(px(width - 10.), px(height - 10.)),
                        cx,
                    );
                }
            });
            cx.run_until_parked();
            if !keyboard {
                let index = app.read_with(cx, |app, _| {
                    app.active_tab()
                        .document
                        .visual_blocks_shared()
                        .iter()
                        .position(|b| matches!(b.kind, VisualBlockKind::Table { .. }))
                        .unwrap()
                });
                let grip = cx
                    .debug_bounds(test_debug_selector(format!(
                        "visual-block-drag-grip-{index}"
                    )))
                    .unwrap();
                cx.simulate_click(grip.center(), Modifiers::none());
                cx.run_until_parked();
            }
            let selected = app.read_with(cx, |app, _| app.active_tab().selected_range.clone());
            app.read_with(cx, |app, _| {
                assert!(
                    app.block_menu.as_ref().is_some_and(|menu| menu.from_grip),
                    "menu for {action:?}, keyboard={keyboard}"
                );
            });
            if keyboard {
                for _ in 0..40 {
                    let reached = app.read_with(cx, |app, _| {
                        let menu = app.block_menu.as_ref().unwrap();
                        menu.root_items()[menu.root_selected] == action
                    });
                    if reached {
                        break;
                    }
                    cx.dispatch_action(Down);
                    cx.run_until_parked();
                }
                app.read_with(cx, |app, _| {
                    let menu = app.block_menu.as_ref().unwrap();
                    assert_eq!(menu.root_items()[menu.root_selected], action);
                });
            }
            let button = cx
                .debug_bounds(super::preview::block_menu_item_element_id(action))
                .unwrap();
            let panel = cx.debug_bounds("visual-block-menu-panel").unwrap();
            assert!(panel.contains(&button.center()));
            assert_eq!(button.size, size(px(36.), px(30.)));
            assert!(cx.debug_bounds("visual-block-transform-12").is_none());
            assert!(cx.debug_bounds("visual-block-transform-13").is_none());
            if keyboard {
                cx.dispatch_action(InsertNewline);
            } else {
                cx.simulate_click(button.center(), Modifiers::none());
            }
            cx.run_until_parked();
            app.update_in(cx, |app, window, cx| {
                assert!(app.block_menu.is_none());
                assert!(
                    app.active_tab()
                        .document
                        .text()
                        .starts_with("before\n\n| A | B |\n| --- | --- |\n| 中文🙂 | keep |")
                );
                assert_eq!(
                    app.active_tab().undo_stack.len(),
                    1,
                    "{action:?}, keyboard={keyboard}, status={}, source={:?}",
                    app.status,
                    app.active_tab().document.text()
                );
                app.undo(&Undo, window, cx);
                assert_eq!(app.active_tab().document.text(), SOURCE);
                assert_eq!(app.active_tab().selected_range, selected);
            });
        }
    }
}

#[gpui::test]
fn block_grip_compact_keyboard_overflow_and_dismissal(cx: &mut TestAppContext) {
    const SOURCE: &str = "one\n\n### Next";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.view_mode = ViewMode::VisualEdit;
        app.sidebar_visible = false;
        app
    });
    cx.simulate_resize(size(px(340.), px(260.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for theme in [AppTheme::Paper, AppTheme::Ink] {
        app.update(cx, |app, cx| {
            app.theme = theme;
            let doc = &app.active_tab().document;
            let target = BlockTarget::from_block(doc.version(), &doc.visual_blocks_shared()[0]);
            app.open_visual_block_grip_menu(target, point(px(330.), px(250.)), cx);
        });
        cx.run_until_parked();
        let panel = cx.debug_bounds("visual-block-menu-panel").unwrap();
        let viewport = cx.update(|window, _| window.viewport_size());
        assert!(panel.left() >= px(0.) && panel.top() >= px(0.));
        assert!(panel.right() <= viewport.width + px(0.5));
        assert!(panel.bottom() <= viewport.height + px(0.5));
        let top = app.update(cx, |app, _| {
            app.active_tab().visual_list.logical_scroll_top()
        });
        cx.dispatch_action(Right);
        app.update(cx, |app, _| {
            assert_eq!(app.block_menu.as_ref().unwrap().root_selected, 1)
        });
        cx.dispatch_action(Left);
        cx.dispatch_action(Up);
        cx.run_until_parked();
        let last = cx.debug_bounds("visual-block-delete").unwrap();
        let panel = cx.debug_bounds("visual-block-menu-panel").unwrap();
        assert!(
            panel.contains(&last.center()),
            "keyboard navigation scrolls the last action into view"
        );
        app.update(cx, |app, _| {
            let menu = app.block_menu.as_ref().unwrap();
            assert_eq!(menu.root_items()[menu.root_selected], BlockMenuItem::Delete);
            let after = app.active_tab().visual_list.logical_scroll_top();
            assert_eq!(after.item_ix, top.item_ix);
            assert_eq!(after.offset_in_item, top.offset_in_item);
        });
        cx.dispatch_action(ClearFileTreeSearch);
        app.update(cx, |app, _| {
            assert!(app.block_menu.is_none());
            assert_eq!(app.active_tab().selected_range, 0..5);
            assert_eq!(app.active_tab().document.text(), SOURCE);
            assert!(app.active_tab().undo_stack.is_empty());
        });
    }
}

#[gpui::test]
fn block_grip_stale_command_preserves_source_and_clipboard(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "one\n\n### next",
        ))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    app.update(cx, |app, cx| {
        let doc = &app.active_tab().document;
        let target = BlockTarget::from_block(doc.version(), &doc.visual_blocks_shared()[0]);
        app.open_visual_block_grip_menu(target, point(px(10.), px(10.)), cx);
        app.active_tab_mut()
            .document
            .set_text("external\n\n### next");
        let version = app.active_tab().document.version();
        cx.write_to_clipboard(ClipboardItem::new_string("sentinel".to_string()));
        app.activate_visual_block_menu_item(BlockMenuItem::CutBlock, None, cx);
        assert!(app.block_menu.is_none());
        assert_eq!(app.active_tab().document.text(), "external\n\n### next");
        assert_eq!(app.active_tab().document.version(), version);
        assert!(app.active_tab().undo_stack.is_empty());
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("sentinel")
        );
    });
}

#[gpui::test]
fn reorder_drag_preserves_terminal_paragraphs_and_exact_undo(cx: &mut TestAppContext) {
    for ending in ["\n", "\r\n"] {
        for forward in [false, true] {
            for count in [1, 2] {
                let source =
                    format!("first 😀{ending}{ending}middle 中文{ending}{ending}last{ending}");
                let (app, vcx) = cx.add_window_view(|_, cx| {
                    let mut app = MarkionApp::new(cx);
                    app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                    app.view_mode = ViewMode::VisualEdit;
                    app
                });
                vcx.update(|window, cx| {
                    window.focus(&app.read(cx).focus_handle);
                    window.activate_window();
                });
                let (
                    moving,
                    destination,
                    selection,
                    version,
                    cached,
                    stale_moving,
                    stale_destination,
                ) = app.update(vcx, |app, cx| {
                    let doc = &app.active_tab().document;
                    let blocks = doc.visual_blocks_shared();
                    let indices = blocks
                        .iter()
                        .enumerate()
                        .filter_map(|(i, b)| {
                            matches!(b.kind, VisualBlockKind::Paragraph).then_some(i)
                        })
                        .collect::<Vec<_>>();
                    let lo = if forward { 0 } else { 3 - count };
                    let hi = lo + count - 1;
                    let dest = if forward { 2 } else { 0 };
                    let version = doc.version();
                    let stale_moving = BlockTarget::from_block(version, &blocks[indices[lo]]);
                    let stale_destination =
                        BlockTarget::from_block(version, &blocks[indices[dest]]);
                    app.select_source_blocks(indices[lo], false, cx);
                    if hi != lo {
                        app.select_source_blocks(indices[hi], true, cx);
                    }
                    (
                        indices[lo],
                        indices[dest],
                        app.active_tab().selected_range.clone(),
                        version,
                        blocks,
                        stale_moving,
                        stale_destination,
                    )
                });
                vcx.run_until_parked();
                let grip = vcx
                    .debug_bounds(test_debug_selector(format!(
                        "visual-block-drag-grip-{moving}"
                    )))
                    .unwrap();
                let side = if forward { "after" } else { "before" };
                let drop = vcx
                    .debug_bounds(test_debug_selector(format!(
                        "visual-block-drop-{side}-{destination}"
                    )))
                    .unwrap();
                vcx.simulate_mouse_down(grip.center(), MouseButton::Left, Modifiers::none());
                vcx.simulate_event(MouseMoveEvent {
                    position: drop.center(),
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Modifiers::none(),
                });
                vcx.run_until_parked();
                app.update(vcx, |app, _| {
                    assert!(app.block_menu.is_none());
                    assert_eq!(app.active_tab().document.text(), source);
                    assert_eq!(app.active_tab().document.version(), version);
                    assert_eq!(app.active_tab().selected_range, selection);
                    assert!(Arc::ptr_eq(
                        &cached,
                        &app.active_tab().document.visual_blocks_shared()
                    ));
                });
                vcx.simulate_event(MouseUpEvent {
                    button: MouseButton::Left,
                    position: drop.center(),
                    modifiers: Modifiers::none(),
                    click_count: 1,
                });
                vcx.run_until_parked();
                let names = match (forward, count) {
                    (true, 1) => ["middle 中文", "last", "first 😀"],
                    (true, 2) | (false, 1) => ["last", "first 😀", "middle 中文"],
                    (false, 2) => ["middle 中文", "last", "first 😀"],
                    _ => unreachable!(),
                };
                let expected = format!(
                    "{}{ending}{ending}{}{ending}{ending}{}{ending}{ending}",
                    names[0], names[1], names[2]
                );
                app.update(vcx, |app, _| {
                    let tab = app.active_tab();
                    assert_eq!(
                        tab.document.text(),
                        expected,
                        "{ending:?}, forward={forward}, count={count}"
                    );
                    assert_eq!(
                        tab.document
                            .preview_blocks()
                            .iter()
                            .filter(|b| matches!(b, PreviewBlock::Paragraph { .. }))
                            .count(),
                        3
                    );
                    assert_eq!(tab.document.version(), version + 1);
                    assert_eq!(tab.undo_stack.len(), 1);
                    assert_eq!(
                        tab.cursor_offset(),
                        if forward {
                            expected.find("first 😀").unwrap()
                        } else {
                            0
                        }
                    );
                    assert!(app.block_menu.is_none());
                });
                vcx.dispatch_action(Undo);
                app.update(vcx, |app, cx| {
                    assert_eq!(app.active_tab().document.text(), source);
                    assert_eq!(app.active_tab().selected_range, selection);
                    assert!(matches!(
                        app.active_tab().structured_selection,
                        Some(markion::selection::SelectionState::Blocks(_))
                    ));
                    let version = app.active_tab().document.version();
                    app.reorder_visual_block(
                        stale_moving,
                        stale_destination,
                        if forward {
                            BlockPlacement::After
                        } else {
                            BlockPlacement::Before
                        },
                        cx,
                    );
                    assert_eq!(app.active_tab().document.text(), source);
                    assert_eq!(app.active_tab().document.version(), version);
                    assert!(app.active_tab().undo_stack.is_empty());
                });
            }
        }
    }
}

#[gpui::test]
fn reorder_keyboard_and_menu_preserve_terminal_paragraphs(cx: &mut TestAppContext) {
    for ending in ["\n", "\r\n"] {
        let source = format!("first{ending}{ending}last{ending}");
        let expected = format!("last{ending}{ending}first{ending}{ending}");
        let (app, vcx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
            app.view_mode = ViewMode::VisualEdit;
            app
        });
        vcx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        app.update(vcx, |app, cx| app.select_source_blocks(0, false, cx));
        vcx.dispatch_action(MoveBlocksDown);
        app.update(vcx, |app, _| {
            let tab = app.active_tab();
            assert_eq!(tab.document.text(), expected);
            assert_eq!(
                &tab.document.text()[tab.selected_range.clone()],
                format!("first{ending}{ending}")
            );
            assert_eq!(tab.undo_stack.len(), 1);
        });
        vcx.dispatch_action(Undo);
        app.update(vcx, |app, cx| {
            assert_eq!(app.active_tab().document.text(), source);
            let doc = &app.active_tab().document;
            let blocks = doc.visual_blocks_shared();
            let last = blocks
                .iter()
                .rev()
                .find(|b| matches!(b.kind, VisualBlockKind::Paragraph))
                .unwrap();
            let target = BlockTarget::from_block(doc.version(), last);
            app.open_visual_block_grip_menu(target, point(px(20.), px(20.)), cx);
            app.activate_visual_block_menu_item(BlockMenuItem::MoveUp, None, cx);
            assert_eq!(app.active_tab().document.text(), expected);
            assert_eq!(app.active_tab().undo_stack.len(), 1);
        });
        vcx.dispatch_action(Undo);
        app.update(vcx, |app, _| {
            assert_eq!(app.active_tab().document.text(), source)
        });
    }
}

#[gpui::test]
fn block_grip_preference_persists_resets_and_keeps_document_caches(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    let (text, version, blocks) = app.update(cx, |app, cx| {
        app.preferences_path = path.clone();
        assert!(app.block_handle_menu);
        app.toggle_block_handle_menu(cx);
        assert!(!load_app_preferences(&path).unwrap().block_handle_menu);
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
        assert!(app.block_handle_menu);
        assert!(load_app_preferences(&path).unwrap().block_handle_menu);
        assert_eq!(app.active_tab().document.text(), text);
        assert_eq!(app.active_tab().document.version(), version);
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
    for language in Language::all() {
        for message in [BlockGripMsg::ShowMenu, BlockGripMsg::AddBelow] {
            assert!(!block_grip_t(*language, message).is_empty());
        }
    }
}
